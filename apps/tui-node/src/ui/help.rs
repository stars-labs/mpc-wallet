//! Contextual help system for the TUI
//! Provides context-sensitive help, shortcuts, and user guidance

use crate::utils::appstate_compat::AppState;
use crate::ui::tui::UIMode;
use std::collections::HashMap;
use frost_core::Ciphersuite;

/// Global keyboard shortcuts
pub struct GlobalShortcuts;

impl GlobalShortcuts {
    pub const SHORTCUTS: &'static [(&'static str, &'static str)] = &[
        ("Ctrl+Q", "Quit application"),
        ("Ctrl+R", "Refresh/Reconnect"),
        ("Ctrl+L", "Clear log"),
        ("Ctrl+W", "Switch wallet"),
        ("Ctrl+N", "New session"),
        ("Ctrl+?", "Toggle help"),
        ("Tab", "Next section"),
        ("Shift+Tab", "Previous section"),
        ("↑/↓", "Navigate list"),
        ("Enter", "Select/Confirm"),
        ("Esc", "Cancel/Back"),
    ];
    
    pub fn get_formatted_shortcuts() -> Vec<String> {
        Self::SHORTCUTS
            .iter()
            .map(|(key, desc)| format!("[{}] {}", key, desc))
            .collect()
    }
}

/// Context-sensitive help provider
pub struct ContextualHelp;

impl ContextualHelp {
    /// Get help text based on current UI mode and application state
    pub fn get_help<C: Ciphersuite>(ui_mode: &UIMode, _app_state: &AppState<C>) -> Vec<String> {
        match ui_mode {
            UIMode::Normal => vec![
                "💡 Welcome to MPC Wallet CLI".to_string(),
                "• Press 'w' to manage wallets".to_string(),
                "• Press 's' to view sessions".to_string(),
                "• Press 'd' to start DKG".to_string(),
                "• Press '?' for detailed help".to_string(),
            ],
            
            
            UIMode::MainMenu { .. } => vec![
                "📋 Main Menu".to_string(),
                "• Navigate with arrow keys".to_string(),
                "• Press Enter to select option".to_string(),
                "• Press Esc to go back".to_string(),
            ],
            
            UIMode::SigningRequestPopup { .. } => vec![
                "✍️ Signing Request".to_string(),
                "• Review the transaction details".to_string(),
                "• Press 'a' to approve".to_string(),
                "• Press 'r' to reject".to_string(),
                "• Press Esc to cancel".to_string(),
            ],
            
            UIMode::SessionProposalPopup { .. } => vec![
                "🤝 Session Proposal".to_string(),
                "• Configure session parameters".to_string(),
                "• Set threshold and participants".to_string(),
                "• Choose coordination type".to_string(),
                "• Press Enter to create session".to_string(),
            ],
            
            _ => vec![
                "💡 Use arrow keys to navigate".to_string(),
                "• Press Enter to select".to_string(),
                "• Press Esc to go back".to_string(),
            ],
        }
    }
    
    /// Get error recovery suggestions
    pub fn get_error_help(error_type: &str) -> Vec<String> {
        match error_type {
            "connection_failed" => vec![
                "🔴 Connection failed".to_string(),
                "→ Check your internet connection".to_string(),
                "→ Try: Refresh (Ctrl+R)".to_string(),
                "→ Verify firewall settings".to_string(),
            ],
            
            "dkg_timeout" => vec![
                "⏱️ DKG timeout".to_string(),
                "→ Some participants may have disconnected".to_string(),
                "→ Try: Create a new session".to_string(),
                "→ Ensure all participants remain online".to_string(),
            ],
            
            "peer_disconnected" => vec![
                "👥 Peer disconnected".to_string(),
                "→ Waiting for peer to reconnect...".to_string(),
                "→ They may need to refresh their connection".to_string(),
                "→ Consider restarting the session if needed".to_string(),
            ],
            
            "invalid_threshold" => vec![
                "⚠️ Invalid threshold configuration".to_string(),
                "→ Threshold must be ≤ total participants".to_string(),
                "→ Recommended: 2-of-3 or 3-of-5".to_string(),
                "→ Higher thresholds increase security but require more signers".to_string(),
            ],
            
            _ => vec![
                "ℹ️ An error occurred".to_string(),
                "→ Check the log for details".to_string(),
                "→ Try refreshing (Ctrl+R)".to_string(),
                "→ Contact support if the issue persists".to_string(),
            ],
        }
    }
    
    /// Get quick tips based on user action
    pub fn get_quick_tip(action: &str) -> Option<String> {
        let tips = HashMap::from([
            ("wallet_created", "✓ Wallet created! Remember to backup your keystore"),
            ("session_joined", "✓ Joined session! Waiting for other participants"),
            ("dkg_started", "⏳ DKG started! Keep your connection stable"),
            ("dkg_complete", "✓ Keys generated! Your wallet is ready to use"),
            ("transaction_signed", "✓ Transaction signed successfully"),
            ("backup_exported", "✓ Backup exported! Store it securely"),
            ("peer_connected", "✓ New peer connected to the session"),
            ("mesh_ready", "✓ All connections established"),
        ]);
        
        tips.get(action).map(|s| s.to_string())
    }
}

/// Mode-specific keyboard shortcuts
pub struct ModeShortcuts;

impl ModeShortcuts {
    pub fn get_shortcuts(ui_mode: &UIMode) -> Vec<(&'static str, &'static str)> {
        match ui_mode {
            UIMode::Normal => vec![
                ("w", "Wallets"),
                ("s", "Sessions"),
                ("d", "Start DKG"),
                ("i", "Import"),
                ("e", "Export"),
                ("r", "Refresh"),
                ("q", "Quit"),
            ],
            
            
            UIMode::MainMenu { .. } => vec![
                ("↑/↓", "Navigate"),
                ("Enter", "Select"),
                ("Esc", "Back"),
            ],
            
            UIMode::SessionProposalPopup { .. } => vec![
                ("Tab", "Next field"),
                ("Enter", "Create"),
                ("Esc", "Cancel"),
            ],
            
            _ => vec![
                ("Enter", "Confirm"),
                ("Esc", "Back"),
            ],
        }
    }
    
    pub fn get_formatted_shortcuts(ui_mode: &UIMode) -> String {
        let shortcuts = Self::get_shortcuts(ui_mode);
        shortcuts
            .iter()
            .map(|(key, desc)| format!("[{}]{}", key, desc))
            .collect::<Vec<_>>()
            .join("  ")
    }
}

/// Quick action suggestions
pub struct QuickActions;

impl QuickActions {
    pub fn get_suggested_actions<C: Ciphersuite>(app_state: &AppState<C>) -> Vec<(String, String)> {
        let mut actions = Vec::new();
        
        // Check various conditions and suggest actions
        if app_state.blockchain_addresses.is_empty() {
            actions.push(("1".to_string(), "Create your first wallet".to_string()));
        }
        
        if app_state.session.is_some() {
            actions.push(("2".to_string(), "Start DKG process".to_string()));
        } else {
            actions.push(("2".to_string(), "Join or create a session".to_string()));
        }
        
        if !app_state.blockchain_addresses.is_empty() {
            actions.push(("3".to_string(), "View wallet addresses".to_string()));
        }
        
        if app_state.pending_signatures > 0 {
            actions.push((
                "4".to_string(),
                format!("Sign {} pending transaction(s)", app_state.pending_signatures),
            ));
        }
        
        actions.push(("5".to_string(), "View help documentation".to_string()));
        actions.push(("6".to_string(), "Check network status".to_string()));
        
        actions
    }
}

/// User experience level for progressive disclosure
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UserLevel {
    Beginner,
    Advanced,
    Expert,
}

impl UserLevel {
    pub fn should_show_technical_details(&self) -> bool {
        matches!(self, UserLevel::Advanced | UserLevel::Expert)
    }
    
    pub fn should_show_debug_info(&self) -> bool {
        matches!(self, UserLevel::Expert)
    }
    
    pub fn get_description(&self) -> &'static str {
        match self {
            UserLevel::Beginner => "Simple mode - Essential information only",
            UserLevel::Advanced => "Advanced mode - Full feature access",
            UserLevel::Expert => "Expert mode - Debug information enabled",
        }
    }
}