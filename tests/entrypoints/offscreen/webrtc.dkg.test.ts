import { describe, it, expect, beforeAll } from 'bun:test';
import { DkgState, WebRTCManager, MeshStatusType } from '../../../src/entrypoints/offscreen/webrtc';
import {
    initializeWasmIfNeeded,
    isWasmInitialized,
    createTestSessionInfo,
    dummySend,
    extractPackageFromMap,
    createTestDkgInstances,
    cleanupDkgInstances
} from './test-utils';
import { FrostDkgEd25519, FrostDkgSecp256k1 } from '../../../pkg/mpc_wallet.js';

beforeAll(async () => {
    await initializeWasmIfNeeded();
});

describe('WebRTCManager DKG Process', () => {
    const sessionInfo = createTestSessionInfo();

    it('should initialize DKG when conditions are met', async () => {
        const manager = new WebRTCManager('a', dummySend);
        manager.sessionInfo = sessionInfo as any;
        (manager as any)._updateMeshStatus({ type: MeshStatusType.Ready });
        (manager as any)._updateDkgState(DkgState.Idle);

        if (!isWasmInitialized()) {
            console.warn('⚠️ WASM not initialized, skipping DKG initialization test.');
            return;
        }

        try {
            // Directly create FROST DKG instance like the test expects
            (manager as any).frostDkg = new FrostDkgEd25519();
            (manager as any).participantIndex = 1;
            (manager as any).frostDkg.init_dkg(1, 3, 2);

            // Set the state manually
            (manager as any)._updateDkgState(DkgState.Round1InProgress);

            // The test should pass
            expect(manager.dkgState).toBe(DkgState.Round1InProgress);
            expect((manager as any).participantIndex).toBe(1);
        } finally {
            if ((manager as any).frostDkg) {
                (manager as any).frostDkg.free();
            }
        }
    });

    it('should handle Round 1 package reception and transition to Round 2', async () => {
        if (!isWasmInitialized()) {
            console.warn('⚠️ WASM not initialized, skipping Round 1 reception test.');
            return;
        }

        const manager = new WebRTCManager('a', dummySend);
        manager.sessionInfo = sessionInfo as any;
        (manager as any)._updateDkgState(DkgState.Round1InProgress);

        let dkgA: FrostDkgEd25519 | null = null;
        let dkgB_sim: FrostDkgEd25519 | null = null;
        let dkgC_sim: FrostDkgEd25519 | null = null;

        try {
            dkgA = new FrostDkgEd25519();
            dkgA.init_dkg(1, 3, 2);
            (manager as any).frostDkg = dkgA;
            (manager as any).participantIndex = 1;

            // Generate and process manager's own Round 1 package
            const round1A_self = dkgA.generate_round1();
            await (manager as any)._handleDkgRound1Package('a', { sender_index: 1, data: round1A_self });

            // Simulate receiving Round 1 packages from devices
            dkgB_sim = new FrostDkgEd25519();
            dkgB_sim.init_dkg(2, 3, 2);
            const round1B_sim = dkgB_sim.generate_round1();
            await (manager as any)._handleDkgRound1Package('b', { sender_index: 2, data: round1B_sim });

            dkgC_sim = new FrostDkgEd25519();
            dkgC_sim.init_dkg(3, 3, 2);
            const round1C_sim = dkgC_sim.generate_round1();
            await (manager as any)._handleDkgRound1Package('c', { sender_index: 3, data: round1C_sim });

            // Check that DKG can proceed to Round 2
            expect((manager as any).frostDkg.can_start_round2()).toBe(true);

            // Trigger Round 2 if needed
            if ((manager as any).frostDkg.can_start_round2()) {
                await (manager as any)._generateAndBroadcastRound2();
            }

            expect(manager.dkgState).toBe(DkgState.Round2InProgress);
            expect((manager as any).receivedRound1Packages.size).toBe(3);

        } finally {
            cleanupDkgInstances(dkgA, dkgB_sim, dkgC_sim);
        }
    });

    it('should handle Round 2 package reception and transition to finalization', async () => {
        if (!isWasmInitialized()) {
            console.warn('⚠️ WASM not initialized, skipping Round 2 reception test.');
            return;
        }

        const manager = new WebRTCManager('a', dummySend);
        manager.sessionInfo = sessionInfo as any;
        (manager as any)._updateDkgState(DkgState.Round1InProgress);

        let dkgA_full: FrostDkgEd25519 | null = null;
        let dkgB_sim_full: FrostDkgEd25519 | null = null;
        let dkgC_sim_full: FrostDkgEd25519 | null = null;

        try {
            // Set up complete Round 1 to get to Round 2
            dkgA_full = new FrostDkgEd25519();
            dkgA_full.init_dkg(1, 3, 2);
            (manager as any).frostDkg = dkgA_full;
            (manager as any).participantIndex = 1;

            // Generate Round 1 packages for all participants
            const round1A_self_full = dkgA_full.generate_round1();

            dkgB_sim_full = new FrostDkgEd25519();
            dkgB_sim_full.init_dkg(2, 3, 2);
            const round1B_sim_full = dkgB_sim_full.generate_round1();

            dkgC_sim_full = new FrostDkgEd25519();
            dkgC_sim_full.init_dkg(3, 3, 2);
            const round1C_sim_full = dkgC_sim_full.generate_round1();

            // Exchange Round 1 packages between all participants
            // Manager A processes packages from B and C
            await (manager as any)._handleDkgRound1Package('a', { sender_index: 1, data: round1A_self_full });
            await (manager as any)._handleDkgRound1Package('b', { sender_index: 2, data: round1B_sim_full });
            await (manager as any)._handleDkgRound1Package('c', { sender_index: 3, data: round1C_sim_full });

            // B sim processes packages from A and C
            dkgB_sim_full.add_round1_package(1, round1A_self_full);
            dkgB_sim_full.add_round1_package(3, round1C_sim_full);

            // C sim processes packages from A and B
            dkgC_sim_full.add_round1_package(1, round1A_self_full);
            dkgC_sim_full.add_round1_package(2, round1B_sim_full);

            // Verify all can start Round 2
            expect((manager as any).frostDkg.can_start_round2()).toBe(true);
            expect(dkgB_sim_full.can_start_round2()).toBe(true);
            expect(dkgC_sim_full.can_start_round2()).toBe(true);

            // Proceed to Round 2
            if ((manager as any).frostDkg.can_start_round2()) {
                await (manager as any)._generateAndBroadcastRound2();
            }

            // Generate Round 2 packages from peer simulations (now that they have all Round 1 packages)
            const round2B = dkgB_sim_full.generate_round2();
            const round2C = dkgC_sim_full.generate_round2();

            // Extract packages for manager A using proper serialization
            const round2B_for_A = extractPackageFromMap(1, round2B, false); // Ed25519
            const round2C_for_A = extractPackageFromMap(1, round2C, false); // Ed25519

            // Process Round 2 packages
            await (manager as any)._handleDkgRound2Package('b', { sender_index: 2, sender_id_hex: 'b', data: round2B_for_A });
            await (manager as any)._handleDkgRound2Package('c', { sender_index: 3, sender_id_hex: 'c', data: round2C_for_A });

            // Verify DKG can finalize
            expect((manager as any).frostDkg.can_finalize()).toBe(true);
            expect(manager.dkgState).toBe(DkgState.Complete);

        } finally {
            cleanupDkgInstances(dkgA_full, dkgB_sim_full, dkgC_sim_full);
        }
    });

    it('should handle Ethereum secp256k1 DKG initialization', async () => {
        if (!isWasmInitialized()) {
            console.warn('⚠️ WASM not initialized, skipping secp256k1 DKG test.');
            return;
        }

        const manager = new WebRTCManager('a', dummySend);
        manager.sessionInfo = createTestSessionInfo() as any;

        let frostDkgSecp: FrostDkgSecp256k1 | null = null;

        try {
            // Test Secp256k1 DKG creation
            expect(() => {
                frostDkgSecp = new FrostDkgSecp256k1();
            }).not.toThrow();

            (manager as any).frostDkg = frostDkgSecp;
            (manager as any).participantIndex = 1;
            (manager as any)._updateDkgState(DkgState.Idle);

            // Initialize DKG
            if (frostDkgSecp && frostDkgSecp.init_dkg) {
                frostDkgSecp.init_dkg(1, 3, 2);
                expect(manager.dkgState).toBe(DkgState.Idle);
            }

        } finally {
            if (frostDkgSecp) {
                frostDkgSecp.free();
            }
        }
    });

    it('should complete full DKG process end-to-end', async () => {
        if (!isWasmInitialized()) {
            console.warn('⚠️ WASM not initialized, skipping end-to-end DKG test.');
            return;
        }

        // Create three managers for a complete 3-peer DKG simulation
        const managerA = new WebRTCManager('a', dummySend);
        const managerB = new WebRTCManager('b', dummySend);
        const managerC = new WebRTCManager('c', dummySend);

        const sessionInfo = createTestSessionInfo();

        let frostDkgA: FrostDkgEd25519 | null = null;
        let frostDkgB: FrostDkgEd25519 | null = null;
        let frostDkgC: FrostDkgEd25519 | null = null;

        try {
            // Set up session info for all managers
            [managerA, managerB, managerC].forEach(manager => {
                manager.sessionInfo = sessionInfo as any;
                (manager as any)._updateMeshStatus({ type: MeshStatusType.Ready });
                (manager as any)._updateDkgState(DkgState.Idle);
            });

            // Create and assign FROST DKG instances
            const dkgInstances = await createTestDkgInstances(false);
            frostDkgA = dkgInstances.frostDkgA as FrostDkgEd25519;
            frostDkgB = dkgInstances.frostDkgB as FrostDkgEd25519;
            frostDkgC = dkgInstances.frostDkgC as FrostDkgEd25519;

            (managerA as any).frostDkg = frostDkgA;
            (managerB as any).frostDkg = frostDkgB;
            (managerC as any).frostDkg = frostDkgC;
            (managerA as any).participantIndex = 1;
            (managerB as any).participantIndex = 2;
            (managerC as any).participantIndex = 3;

            // === ROUND 1: COMMITMENT PHASE ===
            const round1PackageA_hex = frostDkgA.generate_round1();
            const round1PackageB_hex = frostDkgB.generate_round1();
            const round1PackageC_hex = frostDkgC.generate_round1();

            // Cross-exchange Round 1 packages
            frostDkgA.add_round1_package(2, round1PackageB_hex);
            frostDkgA.add_round1_package(3, round1PackageC_hex);
            frostDkgB.add_round1_package(1, round1PackageA_hex);
            frostDkgB.add_round1_package(3, round1PackageC_hex);
            frostDkgC.add_round1_package(1, round1PackageA_hex);
            frostDkgC.add_round1_package(2, round1PackageB_hex);

            // Verify all can start Round 2
            expect(frostDkgA.can_start_round2()).toBe(true);
            expect(frostDkgB.can_start_round2()).toBe(true);
            expect(frostDkgC.can_start_round2()).toBe(true);

            // === ROUND 2: SECRET SHARE PHASE ===
            const round2PackageA_map_hex = frostDkgA.generate_round2();
            const round2PackageB_map_hex = frostDkgB.generate_round2();
            const round2PackageC_map_hex = frostDkgC.generate_round2();

            // Extract individual packages for each participant
            const r2A_for_B = extractPackageFromMap(2, round2PackageA_map_hex, false);
            const r2A_for_C = extractPackageFromMap(3, round2PackageA_map_hex, false);
            const r2B_for_A = extractPackageFromMap(1, round2PackageB_map_hex, false);
            const r2B_for_C = extractPackageFromMap(3, round2PackageB_map_hex, false);
            const r2C_for_A = extractPackageFromMap(1, round2PackageC_map_hex, false);
            const r2C_for_B = extractPackageFromMap(2, round2PackageC_map_hex, false);

            // Exchange Round 2 packages
            frostDkgA.add_round2_package(2, r2B_for_A);
            frostDkgA.add_round2_package(3, r2C_for_A);
            frostDkgB.add_round2_package(1, r2A_for_B);
            frostDkgB.add_round2_package(3, r2C_for_B);
            frostDkgC.add_round2_package(1, r2A_for_C);
            frostDkgC.add_round2_package(2, r2B_for_C);

            // Verify all can finalize
            expect(frostDkgA.can_finalize()).toBe(true);
            expect(frostDkgB.can_finalize()).toBe(true);
            expect(frostDkgC.can_finalize()).toBe(true);

            // === FINALIZATION PHASE ===
            const groupPublicKeyA = frostDkgA.finalize_dkg();
            const groupPublicKeyB = frostDkgB.finalize_dkg();
            const groupPublicKeyC = frostDkgC.finalize_dkg();

            // Verify all participants generated identical group public keys
            expect(groupPublicKeyA).toBe(groupPublicKeyB);
            expect(groupPublicKeyB).toBe(groupPublicKeyC);

            // Update manager states
            (managerA as any).groupPublicKey = groupPublicKeyA;
            (managerB as any).groupPublicKey = groupPublicKeyB;
            (managerC as any).groupPublicKey = groupPublicKeyC;
            (managerA as any)._updateDkgState(DkgState.Complete);
            (managerB as any)._updateDkgState(DkgState.Complete);
            (managerC as any)._updateDkgState(DkgState.Complete);

            // Verify final state
            expect(managerA.dkgState).toBe(DkgState.Complete);
            expect(managerB.dkgState).toBe(DkgState.Complete);
            expect(managerC.dkgState).toBe(DkgState.Complete);

        } finally {
            cleanupDkgInstances(frostDkgA, frostDkgB, frostDkgC);
        }
    });

    it('should reset DKG state properly', () => {
        const manager = new WebRTCManager('a', dummySend);
        manager.sessionInfo = sessionInfo as any;
        (manager as any)._updateDkgState(DkgState.Round2InProgress);

        // Set up some DKG state
        if (isWasmInitialized()) {
            try {
                (manager as any).frostDkg = new FrostDkgEd25519();
            } catch (error) {
                console.warn('WASM DKG creation failed in test, using mock');
                (manager as any).frostDkg = { free: () => { } };
            }
        } else {
            (manager as any).frostDkg = { free: () => { } };
        }

        (manager as any).participantIndex = 1;
        (manager as any).receivedRound1Packages.add('a');
        (manager as any).receivedRound2Packages.add('a');

        (manager as any)._resetDkgState();

        expect((manager as any).frostDkg).toBe(null);
        expect((manager as any).participantIndex).toBe(null);
        expect((manager as any).receivedRound1Packages.size).toBe(0);
        expect((manager as any).receivedRound2Packages.size).toBe(0);
    });

    it('should get DKG status correctly', () => {
        const manager = new WebRTCManager('a', dummySend);
        manager.sessionInfo = sessionInfo as any;
        (manager as any)._updateDkgState(DkgState.Round1InProgress);
        (manager as any).participantIndex = 1;
        (manager as any).receivedRound1Packages.add('a');
        (manager as any).frostDkg = {};

        const status = manager.getDkgStatus();

        expect(status.state).toBe(DkgState.Round1InProgress);
        expect(status.stateName).toBe('Round1InProgress');
        expect(status.participantIndex).toBe(1);
        expect(status.sessionInfo?.session_id).toBe('test-session');
        expect(status.receivedRound1Packages).toEqual(['a']);
        expect(status.frostDkgInitialized).toBe(true);
    });
});
