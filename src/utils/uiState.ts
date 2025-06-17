import type { SupportedChain } from "../types/appstate";

export interface UIState {
  showSettings: boolean;
  proposedSessionIdInput: string;
  totalParticipants: number;
  threshold: number;
  chain: SupportedChain;
  timestamp: number;
}

const UI_STATE_KEY = "mpc_wallet_ui_preferences";

export class UIStateManager {
  // Save ONLY UI preferences to localStorage (not real-time connection state)
  static saveUIState(uiState: Partial<UIState>): void {
    const stateToSave = {
      ...uiState,
      timestamp: Date.now(),
    };
    
    try {
      localStorage.setItem(UI_STATE_KEY, JSON.stringify(stateToSave));
      console.log("[UIStateManager] Saved UI preferences to localStorage:", stateToSave);
    } catch (error) {
      console.warn("[UIStateManager] Failed to save UI preferences:", error);
    }
  }

  // Load ONLY UI preferences from localStorage (not real-time connection states)
  static loadUIState(): Partial<UIState> | null {
    try {
      const stored = localStorage.getItem(UI_STATE_KEY);
      if (stored) {
        const uiState = JSON.parse(stored);
        // Check if state is not too old (1 hour)
        if (Date.now() - uiState.timestamp < 60 * 60 * 1000) {
          console.log("[UIStateManager] Loaded UI preferences from localStorage:", uiState);
          return uiState;
        } else {
          console.log("[UIStateManager] UI preferences expired, using defaults");
          localStorage.removeItem(UI_STATE_KEY);
        }
      }
    } catch (error) {
      console.warn("[UIStateManager] Failed to load UI preferences:", error);
      localStorage.removeItem(UI_STATE_KEY);
    }
    return null;
  }

  // Throttled save function to prevent excessive localStorage writes
  private static throttleTimer: number | null = null;
  private static lastSavedState = "";

  static throttledSaveUIState(uiState: Partial<UIState>): void {
    const currentStateStr = JSON.stringify(uiState);

    // Only save if UI preferences actually changed
    if (currentStateStr !== this.lastSavedState) {
      // Clear existing timer
      if (this.throttleTimer) {
        clearTimeout(this.throttleTimer);
      }

      // Set new timer
      this.throttleTimer = window.setTimeout(() => {
        this.saveUIState(uiState);
        this.lastSavedState = currentStateStr;
        this.throttleTimer = null;
      }, 500); // 500ms throttle
    }
  }
}
