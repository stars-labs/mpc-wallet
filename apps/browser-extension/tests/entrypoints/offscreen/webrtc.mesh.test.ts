import { WebRTCManager, MeshStatusType } from '../../../src/entrypoints/offscreen/webrtc';
import { initializeWasmIfNeeded, createTestSessionInfo, dummySend } from './test-utils';
import {  describe, it, expect, beforeAll  } from 'bun:test';
beforeAll(async () => {
    await initializeWasmIfNeeded();
});

describe('WebRTCManager mesh readiness', () => {
    const sessionInfo = createTestSessionInfo();

    it('should transition to PartiallyReady when first MeshReady received', () => {
        const manager = new WebRTCManager('a', dummySend);
        // Set session and initial mesh status
        manager.sessionInfo = sessionInfo as any;
        (manager as any)._updateMeshStatus({
            type: MeshStatusType.Incomplete,
            ready_devices: new Set(['a']) // Initialize with local peer in the set already
        });

        // Simulate receiving MeshReady from 'b'
        (manager as any)._processPeerMeshReady('b');

        expect(manager.meshStatus.type).toBe(MeshStatusType.PartiallyReady);
        const readydevices = (manager.meshStatus as any).ready_devices as Set<string>;
        expect(readydevices.has('a')).toBe(true);
        expect(readydevices.has('b')).toBe(true);
    });

    it('should transition to Ready when all MeshReady received', () => {
        const manager = new WebRTCManager('a', dummySend);
        manager.sessionInfo = sessionInfo as any;
        // Simulate two devices already ready
        (manager as any)._updateMeshStatus({
            type: MeshStatusType.PartiallyReady,
            ready_devices: ['a', 'b'],
            total_devices: 3
        });

        // Now simulate receiving MeshReady from 'c'
        (manager as any)._processPeerMeshReady('c');

        expect(manager.meshStatus.type).toBe(MeshStatusType.Ready);
    });

    it('should handle peer disconnections in mesh', () => {
        const manager = new WebRTCManager('a', dummySend);
        manager.sessionInfo = sessionInfo as any;

        // Set up ready mesh
        (manager as any)._updateMeshStatus({
            type: MeshStatusType.Ready,
            ready_devices: ['a', 'b', 'c'],
            total_devices: 3
        });

        // Simulate peer 'b' disconnecting
        (manager as any)._handlePeerDisconnection('b');

        expect(manager.meshStatus.type).toBe(MeshStatusType.PartiallyReady);
        const readydevices = (manager.meshStatus as any).ready_devices as Set<string>;
        expect(readydevices.has('b')).toBe(false);
        expect(readydevices.has('a')).toBe(true);
        expect(readydevices.has('c')).toBe(true);
    });

    it('should handle mesh status updates correctly', () => {
        const manager = new WebRTCManager('a', dummySend);

        // Initial state should be Incomplete
        expect(manager.meshStatus.type).toBe(MeshStatusType.Incomplete);

        // Update to PartiallyReady
        (manager as any)._updateMeshStatus({
            type: MeshStatusType.PartiallyReady,
            ready_devices: ['a', 'b'],
            total_devices: 3
        });
        expect(manager.meshStatus.type).toBe(MeshStatusType.PartiallyReady);

        // Update to Ready
        (manager as any)._updateMeshStatus({
            type: MeshStatusType.Ready,
            ready_devices: ['a', 'b', 'c'],
            total_devices: 3
        });
        expect(manager.meshStatus.type).toBe(MeshStatusType.Ready);
    });
});

describe('WebRTCManager connection management', () => {
    it('should manage data channels correctly', () => {
        const manager = new WebRTCManager('a', dummySend);

        const mockDataChannel = {
            readyState: 'open',
            send: () => { },
            close: () => { },
            addEventListener: () => { },
            removeEventListener: () => { }
        };

        // Add data channel
        (manager as any).dataChannels.set('b', mockDataChannel);
        expect((manager as any).dataChannels.has('b')).toBe(true);

        // Remove data channel
        (manager as any).dataChannels.delete('b');
        expect((manager as any).dataChannels.has('b')).toBe(false);
    });

    it('should handle WebRTC message routing correctly', () => {
        const manager = new WebRTCManager('a', dummySend);
        let sentMessage: any = null;
        let sentToPeer: string = '';

        // Override WebRTCManager's sendWebRTCAppMessage method
        manager.sendWebRTCAppMessage = (todeviceId: string, message: any) => {
            sentToPeer = todeviceId;
            sentMessage = message;
        };

        const testMessage = {
            webrtc_msg_type: 'MeshReady',
            device_id: 'a'
        };

        // Send message
        (manager as any)._sendWebRTCMessage('b', testMessage);

        expect(sentToPeer).toBe('b');
        expect(sentMessage).toEqual(testMessage);
    });
});
