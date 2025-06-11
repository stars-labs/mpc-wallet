// ===================================================================
// SESSION MANAGEMENT TYPES
// ===================================================================
//
// This file contains all types related to MPC wallet session management.
// Sessions are collaborative key generation or signing operations between
// multiple participants in the MPC (Multi-Party Computation) protocol.
//
// Key Concepts for Junior Developers:
// - Session: A collaborative operation involving multiple devices/participants
// - Proposer: The device that initiates a session  
// - Participants: All devices that will participate in the session
// - Threshold: Minimum number of participants needed for operations
// - Total: Maximum number of participants in the session
// ===================================================================

/**
 * Represents a session in the MPC wallet system.
 * A session is a collaborative operation (like key generation or signing)
 * that involves multiple participants working together.
 */
export interface SessionInfo {
    /** Unique identifier for this session */
    session_id: string;

    /** Device ID of the participant who proposed this session */
    proposer_id: string;

    /** Maximum number of participants that can join this session */
    total: number; // u16 in Rust backend

    /** Minimum number of participants needed for operations to succeed */
    threshold: number; // u16 in Rust backend

    /** List of all device IDs that are part of this session */
    participants: string[];

    /** List of device IDs that have accepted to join this session */
    accepted_devices: string[];

    /** Optional status field for session state tracking */
    status?: string;
}

/**
 * Used when proposing a new session to other participants.
 * This is sent over the network to invite others to join.
 */
export interface SessionProposal {
    /** Unique identifier for the proposed session */
    session_id: string;

    /** Maximum number of participants */
    total: number;

    /** Minimum number of participants needed */
    threshold: number;

    /** List of device IDs being invited to participate */
    participants: string[];
}

/**
 * Response sent by a participant when they receive a session proposal.
 * Each participant must respond with whether they accept or reject.
 */
export interface SessionResponse {
    /** The session ID they are responding to */
    session_id: string;

    /** Whether they accept (true) or reject (false) the invitation */
    accepted: boolean;
}

/**
 * Helper type for session validation and utilities
 */
export interface SessionValidation {
    /** Whether the session has enough participants */
    hasMinimumParticipants: boolean;

    /** Whether all invited participants have responded */
    allParticipantsResponded: boolean;

    /** Whether the session can proceed with operations */
    canProceed: boolean;
}

// Utility functions (can be implemented elsewhere)
export type SessionValidator = (session: SessionInfo) => SessionValidation;

// Ensure this file is treated as a module
export { };
