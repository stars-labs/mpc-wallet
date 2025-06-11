// ===================================================================
// APPLICATION STATE TYPES
// ===================================================================
//
// This file contains the core application state types for the MPC wallet.
// This is the central state that the UI components interact with and
// represents what the user sees and can interact with.
//
// Key Concepts for Junior Developers:
// - Application State: The current data/status of the entire application
// - Global State: State that needs to be shared across multiple components
// - Device ID: Unique identifier for this device in the MPC network
// - Connected Devices: Other devices currently online and available
// - Session Management: Managing collaborative operations with other devices
// ===================================================================

/**
 * Supported blockchain networks.
 * Grouped by cryptographic curve compatibility.
 */
export type SupportedChain =
  // secp256k1-based chains
  | "ethereum"
  | "polygon"
  | "arbitrum"
  | "optimism"
  | "base"
  // ed25519-based chains
  | "solana"
  | "sui";

/**
 * Mapping of cryptographic curves to their compatible blockchain networks.
 * This enables proper validation while maintaining independence.
 */
export const CURVE_COMPATIBLE_CHAINS = {
  secp256k1: ["ethereum", "polygon", "arbitrum", "optimism", "base"] as const,
  ed25519: ["solana", "sui"] as const,
} as const;

/**
 * Helper function to get compatible chains for a given curve.
 */
export function getCompatibleChains(curve: "secp256k1" | "ed25519"): readonly SupportedChain[] {
  return CURVE_COMPATIBLE_CHAINS[curve];
}

/**
 * Helper function to get the required curve for a given chain.
 */
export function getRequiredCurve(chain: SupportedChain): "secp256k1" | "ed25519" {
  if (CURVE_COMPATIBLE_CHAINS.secp256k1.includes(chain as any)) {
    return "secp256k1";
  }
  if (CURVE_COMPATIBLE_CHAINS.ed25519.includes(chain as any)) {
    return "ed25519";
  }
  throw new Error(`Unknown chain: ${chain}`);
}

/**
 * The main application state interface.
 * This represents all the global state that the UI needs to know about.
 */
export interface AppState {
  // Device and Connection Information
  /** Unique identifier for this device in the MPC network */
  deviceId: string;

  /** List of other device IDs that are currently online and connected */
  connecteddevices: string[];

  /** Whether we have an active WebSocket connection to the signaling server */
  wsConnected: boolean;

  // Session Management
  /** Information about the current active session (if any) */
  sessionInfo: SessionInfo | null;

  /** List of session invitations we've received but not yet responded to */
  invites: SessionInfo[];

  // Network Status
  /** Current status of the mesh network between all session participants */
  meshStatus: MeshStatus;

  /** Current status of the DKG (key generation) process */
  dkgState: DkgState;

  /** Map of device ID to whether we have an active WebRTC connection */
  webrtcConnections: Record<string, boolean>;

  // User Preferences and Settings
  /** Currently selected cryptographic curve for operations */
  curve?: "ed25519" | "secp256k1";

  /** User interface preferences */
  uiPreferences?: {
    /** Whether dark mode is enabled */
    darkMode: boolean;
    /** User's preferred language */
    language: string;
    /** Whether to show advanced features */
    showAdvanced: boolean;
  };

  // UI State Management
  /** Whether the settings panel is currently visible */
  showSettings: boolean;

  /** Current blockchain network selection for UI display */
  chain: SupportedChain;

  // DKG and Address Management
  /** Generated DKG address for the current blockchain */
  dkgAddress: string;

  /** Any error that occurred during DKG address generation */
  dkgError: string;

  // Session Creation UI State
  /** User input for proposed session ID */
  proposedSessionIdInput: string;

  /** Number of total participants for new session creation */
  totalParticipants: number;

  /** Threshold number for new session creation */
  threshold: number;

  /** Tracking session acceptance status by session ID and device ID */
  sessionAcceptanceStatus: Record<string, Record<string, boolean>>;

  // Error Display State
  /** WebSocket connection error message */
  wsError: string;

  // Application Status
  /** Whether the application is currently initializing */
  isInitializing?: boolean;

  /** Any global error message to display to the user */
  globalError?: string;

  /** Whether the user has completed the initial setup */
  setupComplete?: boolean;
}

/**
 * Actions that can be performed to update the application state.
 * This follows the Redux/Flux pattern of describing state changes.
 */
export type AppStateAction =
  // Device and Connection Actions
  | { type: 'SET_DEVICE_ID'; deviceId: string }
  | { type: 'UPDATE_CONNECTED_DEVICES'; devices: string[] }
  | { type: 'SET_WS_CONNECTION'; connected: boolean }

  // Session Actions
  | { type: 'SET_SESSION_INFO'; sessionInfo: SessionInfo | null }
  | { type: 'ADD_INVITE'; invite: SessionInfo }
  | { type: 'REMOVE_INVITE'; sessionId: string }
  | { type: 'CLEAR_INVITES' }

  // Network Status Actions
  | { type: 'UPDATE_MESH_STATUS'; status: MeshStatus }
  | { type: 'UPDATE_DKG_STATE'; state: DkgState }
  | { type: 'UPDATE_WEBRTC_CONNECTIONS'; connections: Record<string, boolean> }
  | { type: 'SET_WEBRTC_CONNECTION'; deviceId: string; connected: boolean }

  // User Preference Actions
  | { type: 'SET_CURVE'; curve: "ed25519" | "secp256k1" }
  | { type: 'UPDATE_UI_PREFERENCES'; preferences: Partial<AppState['uiPreferences']> }

  // UI State Actions
  | { type: 'SET_SHOW_SETTINGS'; show: boolean }
  | { type: 'SET_CHAIN'; chain: SupportedChain }
  | { type: 'SET_DKG_ADDRESS'; address: string }
  | { type: 'SET_DKG_ERROR'; error: string }
  | { type: 'SET_PROPOSED_SESSION_ID'; sessionId: string }
  | { type: 'SET_TOTAL_PARTICIPANTS'; total: number }
  | { type: 'SET_THRESHOLD'; threshold: number }
  | { type: 'UPDATE_SESSION_ACCEPTANCE_STATUS'; sessionId: string; deviceId: string; accepted: boolean }
  | { type: 'SET_WS_ERROR'; error: string }

  // Application Status Actions
  | { type: 'SET_INITIALIZING'; isInitializing: boolean }
  | { type: 'SET_GLOBAL_ERROR'; error: string | null }
  | { type: 'SET_SETUP_COMPLETE'; complete: boolean }

  // Reset Actions
  | { type: 'RESET_STATE' }
  | { type: 'RESET_SESSION_STATE' };

/**
 * The initial/default state of the application.
 */
export const INITIAL_APP_STATE: AppState = {
  deviceId: '',
  connecteddevices: [],
  wsConnected: false,
  sessionInfo: null,
  invites: [],
  meshStatus: { type: MeshStatusType.Incomplete },
  dkgState: DkgState.Idle,
  webrtcConnections: {},
  curve: 'secp256k1', // Default to secp256k1 (most common)
  uiPreferences: {
    darkMode: false,
    language: 'en',
    showAdvanced: false,
  },
  showSettings: false,
  chain: 'ethereum', // Default chain for UI (compatible with secp256k1)
  dkgAddress: '',
  dkgError: '',
  proposedSessionIdInput: '',
  totalParticipants: 3,
  threshold: 2,
  sessionAcceptanceStatus: {},
  wsError: '',
  isInitializing: true,
  setupComplete: false,
};

/**
 * Events that represent significant changes in application state.
 * These can be used for logging, analytics, or triggering side effects.
 */
export type AppStateEvent =
  | { type: 'AppInitialized'; deviceId: string }
  | { type: 'SessionJoined'; sessionId: string }
  | { type: 'SessionLeft'; sessionId: string }
  | { type: 'DkgCompleted'; blockchain: string; address: string }
  | { type: 'ConnectionEstablished'; withDevice: string }
  | { type: 'ConnectionLost'; withDevice: string }
  | { type: 'ErrorOccurred'; error: string; context: string }
  | { type: 'UserActionCompleted'; action: string };

// Import required types from other modules
import type { SessionInfo } from './session';
import type { MeshStatus } from './mesh';
import { MeshStatusType } from './mesh';
import { DkgState } from './dkg';
