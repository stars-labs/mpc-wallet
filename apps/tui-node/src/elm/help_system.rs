//! Contextual Help System
//!
//! This module provides a contextual help system with tooltips,
//! quick help overlays, and interactive tutorials.

use std::collections::HashMap;
use crate::elm::model::{Screen, ComponentId};
use crate::elm::navigation::ShortcutHint;

/// Help system for providing contextual assistance
#[derive(Debug, Clone)]
pub struct HelpSystem {
    help_database: HelpDatabase,
    current_context: HelpContext,
    tutorial_state: Option<TutorialState>,
}

impl HelpSystem {
    /// Create a new help system
    pub fn new() -> Self {
        Self {
            help_database: HelpDatabase::new(),
            current_context: HelpContext::default(),
            tutorial_state: None,
        }
    }
    
    /// Update the current context
    pub fn set_context(&mut self, screen: Screen, component: ComponentId) {
        self.current_context = HelpContext {
            screen,
            focused_component: component,
            user_action: None,
        };
    }
    
    /// Get help for current context
    pub fn get_contextual_help(&self) -> Option<&HelpEntry> {
        self.help_database.get_help(&self.current_context)
    }
    
    /// Get quick tips for current screen
    pub fn get_quick_tips(&self) -> Vec<String> {
        self.help_database.get_tips(&self.current_context.screen)
    }
    
    /// Start interactive tutorial
    pub fn start_tutorial(&mut self, tutorial_type: TutorialType) {
        self.tutorial_state = Some(TutorialState::new(tutorial_type));
    }
    
    /// Advance tutorial to next step
    pub fn next_tutorial_step(&mut self) -> Option<&TutorialStep> {
        if let Some(ref mut tutorial) = self.tutorial_state {
            tutorial.next_step()
        } else {
            None
        }
    }
    
    /// Check if tutorial is active
    pub fn is_tutorial_active(&self) -> bool {
        self.tutorial_state.is_some()
    }
    
    /// End tutorial
    pub fn end_tutorial(&mut self) {
        self.tutorial_state = None;
    }
}

impl Default for HelpSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Current help context
#[derive(Debug, Clone, Default)]
pub struct HelpContext {
    pub screen: Screen,
    pub focused_component: ComponentId,
    pub user_action: Option<UserAction>,
}

/// User actions that might need help
#[derive(Debug, Clone, PartialEq)]
pub enum UserAction {
    Creating,
    Configuring,
    Navigating,
    Signing,
    Troubleshooting,
}

/// Database of help entries
#[derive(Debug, Clone)]
pub struct HelpDatabase {
    entries: HashMap<String, HelpEntry>,
    tips: HashMap<String, Vec<String>>,
}

impl HelpDatabase {
    /// Create and populate help database
    pub fn new() -> Self {
        let mut db = Self {
            entries: HashMap::new(),
            tips: HashMap::new(),
        };
        db.populate();
        db
    }
    
    /// Populate with help content
    fn populate(&mut self) {
        // Main menu help
        self.add_entry(
            "main_menu",
            HelpEntry {
                id: "main_menu".to_string(),
                title: "Main Menu".to_string(),
                content: "The main menu is your starting point for all wallet operations. Use arrow keys to navigate and Enter to select.".to_string(),
                shortcuts: vec![
                    ShortcutHint::new("↑↓", "Navigate options"),
                    ShortcutHint::new("Enter", "Select option"),
                    ShortcutHint::new("n", "Quick: New wallet"),
                    ShortcutHint::new("j", "Quick: Join session"),
                    ShortcutHint::new("w", "Quick: Wallet list"),
                ],
                examples: vec![
                    "Press 'n' to quickly start creating a new wallet".to_string(),
                    "Use number keys 1-6 to jump to menu items".to_string(),
                ],
                related: vec!["navigation".to_string(), "quick_actions".to_string()],
            }
        );
        
        // Threshold configuration help
        self.add_entry(
            "threshold_config",
            HelpEntry {
                id: "threshold_config".to_string(),
                title: "Threshold Configuration".to_string(),
                content: "Set the minimum number of participants needed to authorize transactions. The threshold must be less than or equal to the total participants.".to_string(),
                shortcuts: vec![
                    ShortcutHint::new("←→", "Adjust values"),
                    ShortcutHint::new("Tab", "Switch fields"),
                    ShortcutHint::new("Enter", "Confirm"),
                ],
                examples: vec![
                    "2-of-3: Any 2 out of 3 participants can sign".to_string(),
                    "3-of-5: Requires 3 signatures from 5 participants".to_string(),
                    "Common for personal: 2-of-3".to_string(),
                    "Common for business: 3-of-5 or 4-of-7".to_string(),
                ],
                related: vec!["security".to_string(), "mpc".to_string()],
            }
        );
        
        // DKG process help
        self.add_entry(
            "dkg_process",
            HelpEntry {
                id: "dkg_process".to_string(),
                title: "Distributed Key Generation".to_string(),
                content: "DKG creates key shares among participants without any single party having the complete key. Each participant gets a unique share.".to_string(),
                shortcuts: vec![
                    ShortcutHint::new("Esc", "Cancel process"),
                    ShortcutHint::new("d", "Show details"),
                ],
                examples: vec![
                    "All participants must be online during DKG".to_string(),
                    "The process typically takes 1-2 minutes".to_string(),
                    "If someone drops out, the process must restart".to_string(),
                ],
                related: vec!["frost".to_string(), "security".to_string()],
            }
        );
        
        // Add tips for screens
        self.add_tips(
            "main_menu",
            vec![
                "Use quick keys (n, j, w) for faster navigation".to_string(),
                "Your wallets are encrypted and stored locally".to_string(),
                "Press ? at any time for help".to_string(),
            ]
        );
        
        self.add_tips(
            "create_wallet",
            vec![
                "Choose Online mode for real-time coordination".to_string(),
                "Offline mode is more secure but requires manual coordination".to_string(),
                "Higher thresholds provide more security but less convenience".to_string(),
            ]
        );
        
        self.add_tips(
            "manage_wallets",
            vec![
                "Press / to search your wallets".to_string(),
                "Use 'd' to delete a wallet (requires confirmation)".to_string(),
                "Export wallets regularly for backup".to_string(),
            ]
        );
    }
    
    /// Add a help entry
    fn add_entry(&mut self, key: &str, entry: HelpEntry) {
        self.entries.insert(key.to_string(), entry);
    }
    
    /// Add tips for a screen
    fn add_tips(&mut self, key: &str, tips: Vec<String>) {
        self.tips.insert(key.to_string(), tips);
    }
    
    /// Get help for context
    pub fn get_help(&self, context: &HelpContext) -> Option<&HelpEntry> {
        let key = self.context_to_key(context);
        self.entries.get(&key)
    }
    
    /// Get tips for screen
    pub fn get_tips(&self, screen: &Screen) -> Vec<String> {
        let key = self.screen_to_key(screen);
        self.tips.get(&key).cloned().unwrap_or_default()
    }
    
    /// Convert context to database key
    fn context_to_key(&self, context: &HelpContext) -> String {
        match (&context.screen, &context.focused_component) {
            (Screen::MainMenu, _) => "main_menu".to_string(),
            (Screen::ThresholdConfig, _) => "threshold_config".to_string(),
            (Screen::DKGProgress { .. }, _) => "dkg_process".to_string(),
            _ => "general".to_string(),
        }
    }
    
    /// Convert screen to database key
    fn screen_to_key(&self, screen: &Screen) -> String {
        match screen {
            Screen::MainMenu | Screen::Welcome => "main_menu".to_string(),
            Screen::CreateWallet(_) => "create_wallet".to_string(),
            Screen::ManageWallets => "manage_wallets".to_string(),
            _ => "general".to_string(),
        }
    }
}

/// Help entry with content and metadata
#[derive(Debug, Clone)]
pub struct HelpEntry {
    pub id: String,
    pub title: String,
    pub content: String,
    pub shortcuts: Vec<ShortcutHint>,
    pub examples: Vec<String>,
    pub related: Vec<String>,
}

/// Interactive tutorial system
#[derive(Debug, Clone)]
pub struct TutorialState {
    pub tutorial_type: TutorialType,
    pub steps: Vec<TutorialStep>,
    pub current_step: usize,
    pub completed_steps: Vec<bool>,
}

impl TutorialState {
    /// Create a new tutorial
    pub fn new(tutorial_type: TutorialType) -> Self {
        let steps = tutorial_type.get_steps();
        let completed_steps = vec![false; steps.len()];
        
        Self {
            tutorial_type,
            steps,
            current_step: 0,
            completed_steps,
        }
    }
    
    /// Get current step
    pub fn current_step(&self) -> Option<&TutorialStep> {
        self.steps.get(self.current_step)
    }
    
    /// Move to next step
    pub fn next_step(&mut self) -> Option<&TutorialStep> {
        if self.current_step < self.steps.len() {
            self.completed_steps[self.current_step] = true;
            self.current_step += 1;
        }
        self.current_step()
    }
    
    /// Move to previous step
    pub fn previous_step(&mut self) -> Option<&TutorialStep> {
        if self.current_step > 0 {
            self.current_step -= 1;
        }
        self.current_step()
    }
    
    /// Check if tutorial is complete
    pub fn is_complete(&self) -> bool {
        self.current_step >= self.steps.len()
    }
    
    /// Get progress percentage
    pub fn progress(&self) -> f32 {
        if self.steps.is_empty() {
            0.0
        } else {
            (self.current_step as f32 / self.steps.len() as f32) * 100.0
        }
    }
}

/// Types of tutorials available
#[derive(Debug, Clone, PartialEq)]
pub enum TutorialType {
    FirstWallet,
    JoiningSession,
    SigningTransaction,
    OfflineMode,
}

impl TutorialType {
    /// Get tutorial steps
    pub fn get_steps(&self) -> Vec<TutorialStep> {
        match self {
            Self::FirstWallet => vec![
                TutorialStep {
                    title: "Welcome!".to_string(),
                    instruction: "Let's create your first MPC wallet. This tutorial will guide you through the process.".to_string(),
                    highlight_component: ComponentId::MainMenu,
                    action_required: Some("Press Enter to continue".to_string()),
                    hint: "MPC wallets split keys among multiple parties for enhanced security".to_string(),
                },
                TutorialStep {
                    title: "Choose Mode".to_string(),
                    instruction: "Select 'Online' mode for real-time coordination with other participants.".to_string(),
                    highlight_component: ComponentId::Custom("mode_selection".to_string()),
                    action_required: Some("Select Online and press Enter".to_string()),
                    hint: "Online mode uses WebRTC for secure peer-to-peer communication".to_string(),
                },
                TutorialStep {
                    title: "Select Curve".to_string(),
                    instruction: "Choose 'Secp256k1' for Ethereum and Bitcoin compatibility.".to_string(),
                    highlight_component: ComponentId::Custom("curve_selection".to_string()),
                    action_required: Some("Select Secp256k1 and press Enter".to_string()),
                    hint: "Different curves are used by different blockchains".to_string(),
                },
                TutorialStep {
                    title: "Set Threshold".to_string(),
                    instruction: "Configure how many participants are needed to sign transactions.".to_string(),
                    highlight_component: ComponentId::Custom("threshold_config".to_string()),
                    action_required: Some("Set participants to 3, threshold to 2".to_string()),
                    hint: "2-of-3 means any 2 participants can authorize a transaction".to_string(),
                },
                TutorialStep {
                    title: "Complete Setup".to_string(),
                    instruction: "Review your settings and confirm to start the key generation process.".to_string(),
                    highlight_component: ComponentId::Custom("confirmation".to_string()),
                    action_required: Some("Press Enter to start DKG".to_string()),
                    hint: "The DKG process will coordinate with other participants to generate key shares".to_string(),
                },
            ],
            
            Self::JoiningSession => vec![
                TutorialStep {
                    title: "Join a Session".to_string(),
                    instruction: "Let's join an existing DKG session created by another participant.".to_string(),
                    highlight_component: ComponentId::MainMenu,
                    action_required: Some("Select 'Join Session' from the menu".to_string()),
                    hint: "You'll need the session ID from the coordinator".to_string(),
                },
                // Add more steps...
            ],
            
            _ => vec![], // Other tutorials to be implemented
        }
    }
}

/// Individual tutorial step
#[derive(Debug, Clone)]
pub struct TutorialStep {
    pub title: String,
    pub instruction: String,
    pub highlight_component: ComponentId,
    pub action_required: Option<String>,
    pub hint: String,
}

/// Tooltip for providing context-sensitive help
#[derive(Debug, Clone)]
pub struct Tooltip {
    pub target: ComponentId,
    pub content: String,
    pub position: TooltipPosition,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TooltipPosition {
    Above,
    Below,
    Left,
    Right,
}

impl Tooltip {
    /// Create a new tooltip
    pub fn new(target: ComponentId, content: impl Into<String>) -> Self {
        Self {
            target,
            content: content.into(),
            position: TooltipPosition::Below,
        }
    }
    
    /// Set position
    pub fn with_position(mut self, position: TooltipPosition) -> Self {
        self.position = position;
        self
    }
}

/// Quick help overlay content
pub struct QuickHelp {
    pub screen_name: String,
    pub available_actions: Vec<ShortcutHint>,
    pub context_help: Option<String>,
    pub tips: Vec<String>,
}

impl QuickHelp {
    /// Generate quick help for current screen
    pub fn for_screen(screen: &Screen) -> Self {
        use crate::elm::navigation::get_context_shortcuts;
        
        let screen_name = match screen {
            Screen::MainMenu => "Main Menu",
            Screen::CreateWallet(_) => "Create Wallet",
            Screen::ManageWallets => "Manage Wallets",
            Screen::JoinSession => "Join Session",
            Screen::DKGProgress { .. } => "Key Generation",
            _ => "MPC Wallet",
        }.to_string();
        
        let available_actions = get_context_shortcuts(screen);
        
        let context_help = match screen {
            Screen::ThresholdConfig => Some(
                "The threshold determines how many participants are needed to sign transactions. \
                For example, 2-of-3 means any 2 out of 3 participants can authorize a transaction."
                .to_string()
            ),
            Screen::DKGProgress { .. } => Some(
                "Distributed Key Generation is in progress. All participants must remain \
                connected until the process completes."
                .to_string()
            ),
            _ => None,
        };
        
        let tips = match screen {
            Screen::MainMenu => vec![
                "Press '?' at any time for help".to_string(),
                "Use quick keys for faster navigation".to_string(),
            ],
            Screen::CreateWallet(_) => vec![
                "Higher thresholds mean more security".to_string(),
                "You can always export your wallet for backup".to_string(),
            ],
            _ => vec![],
        };
        
        Self {
            screen_name,
            available_actions,
            context_help,
            tips,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_help_system() {
        let mut help = HelpSystem::new();
        help.set_context(Screen::MainMenu, ComponentId::MainMenu);
        
        let contextual = help.get_contextual_help();
        assert!(contextual.is_some());
        
        let tips = help.get_quick_tips();
        assert!(!tips.is_empty());
    }
    
    #[test]
    fn test_tutorial() {
        let mut tutorial = TutorialState::new(TutorialType::FirstWallet);
        
        assert_eq!(tutorial.current_step, 0);
        assert!(!tutorial.is_complete());
        
        let step = tutorial.current_step().unwrap();
        assert!(step.title.contains("Welcome"));
        
        tutorial.next_step();
        assert_eq!(tutorial.current_step, 1);
        
        // Progress through all steps
        while !tutorial.is_complete() {
            tutorial.next_step();
        }
        
        assert_eq!(tutorial.progress(), 100.0);
    }
    
    #[test]
    fn test_quick_help() {
        let help = QuickHelp::for_screen(&Screen::MainMenu);
        
        assert_eq!(help.screen_name, "Main Menu");
        assert!(!help.available_actions.is_empty());
        assert!(!help.tips.is_empty());
    }
}