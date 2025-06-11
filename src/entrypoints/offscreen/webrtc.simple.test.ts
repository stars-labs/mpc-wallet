import { describe, it, expect } from 'bun:test';
import { DkgState, WebRTCManager, MeshStatusType } from './webrtc';

// Dummy send function for WebRTCManager
const dummySend = (_toPeerId: string, _message: any) => { };

describe('WebRTCManager basic functionality', () => {
    // Basic WebRTC functionality tests
    it('should initialize with correct default state', () => {
        const manager = new WebRTCManager('test-id', dummySend);
        expect(manager).toBeDefined();
        expect(manager.dkgState).toBe(DkgState.Idle);
        expect(manager.meshStatus.type).toBe(MeshStatusType.Incomplete);
    });
    
    it('should allow setting session info', () => {
        const manager = new WebRTCManager('test-id', dummySend);
        const sessionInfo = {
            session_id: 'test-session',
            proposer_id: 'initiator',
            participants: ['test-id', 'peer-1', 'peer-2'],
            accepted_peers: ['test-id', 'peer-1', 'peer-2'],
            total: 3,
            threshold: 2
        };
        
        manager.sessionInfo = sessionInfo as any;
        expect(manager.sessionInfo).toBeDefined();
        expect(manager.sessionInfo?.session_id).toBe('test-session');
        expect(manager.sessionInfo?.participants.length).toBe(3);
    });

    // Skip complex mesh readiness tests as they require more setup
});

describe('WebRTCManager state management', () => {
    const sessionInfo = {
        session_id: 'test-session',
        proposer_id: 'initiator',
        participants: ['a', 'b', 'c'],
        accepted_peers: ['a', 'b', 'c'],
        total: 3,
        threshold: 2
    };

    // Test DKG state transitions
    it('should transition through all DKG states', () => {
        const manager = new WebRTCManager('a', dummySend);
        manager.sessionInfo = sessionInfo as any;
        
        // Test all DKG state transitions in order
        (manager as any)._updateDkgState(DkgState.Idle);
        expect(manager.dkgState).toBe(DkgState.Idle);
        
        (manager as any)._updateDkgState(DkgState.Round1InProgress);
        expect(manager.dkgState).toBe(DkgState.Round1InProgress);
        
        (manager as any)._updateDkgState(DkgState.Round1Complete);
        expect(manager.dkgState).toBe(DkgState.Round1Complete);
        
        (manager as any)._updateDkgState(DkgState.Round2InProgress);
        expect(manager.dkgState).toBe(DkgState.Round2InProgress);
        
        (manager as any)._updateDkgState(DkgState.Round2Complete);
        expect(manager.dkgState).toBe(DkgState.Round2Complete);
        
        (manager as any)._updateDkgState(DkgState.Finalizing);
        expect(manager.dkgState).toBe(DkgState.Finalizing);
        
        (manager as any)._updateDkgState(DkgState.Complete);
        expect(manager.dkgState).toBe(DkgState.Complete);
        
        (manager as any)._updateDkgState(DkgState.Failed);
        expect(manager.dkgState).toBe(DkgState.Failed);
    });

    it('should update mesh status correctly', () => {
        const manager = new WebRTCManager('a', dummySend);
        manager.sessionInfo = sessionInfo as any;
        
        // Set and verify incomplete state
        (manager as any)._updateMeshStatus({ 
            type: MeshStatusType.Incomplete
        });
        expect(manager.meshStatus.type).toBe(MeshStatusType.Incomplete);
        
        // Set and verify partially ready state
        (manager as any)._updateMeshStatus({ 
            type: MeshStatusType.PartiallyReady,
            ready_peers: new Set(['a', 'b']),
            total_peers: 3
        });
        expect(manager.meshStatus.type).toBe(MeshStatusType.PartiallyReady);
        
        // Set and verify ready state
        (manager as any)._updateMeshStatus({ 
            type: MeshStatusType.Ready
        });
        expect(manager.meshStatus.type).toBe(MeshStatusType.Ready);
    });
});