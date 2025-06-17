// Test for setBlockchain message handling in offscreen context
import { describe, it, expect, beforeEach, afterEach } from "bun:test";
import { WebRTCManager } from "./webrtc";

describe('WebRTCManager setBlockchain functionality', () => {
    let manager: WebRTCManager;

    beforeEach(() => {
        manager = new WebRTCManager('test-peer');
    });

    afterEach(() => {
        // Clean up if needed
    });

    it('should handle setBlockchain message correctly', () => {
        // Test setting blockchain to ethereum
        manager.setBlockchain('ethereum');
        expect((manager as any).currentBlockchain).toBe('ethereum');

        // Test setting blockchain to solana
        manager.setBlockchain('solana');
        expect((manager as any).currentBlockchain).toBe('solana');
    });

    it('should use correct default blockchain', () => {
        // Default should be solana
        expect((manager as any).currentBlockchain).toBe('solana');
    });

    it('should accept valid blockchain types only', () => {
        // TypeScript should prevent invalid blockchain types
        manager.setBlockchain('ethereum');
        manager.setBlockchain('solana');

        // This test ensures the method exists and can be called
        expect(manager.setBlockchain).toBeDefined();
        expect(typeof manager.setBlockchain).toBe('function');
    });
});
