//! Differential UI Updates
//!
//! This module provides efficient UI update mechanisms that only update
//! components that have actually changed, reducing rendering overhead.

use crate::elm::model::{Model, Screen};
use crate::elm::message::Message;
use std::collections::HashSet;

/// Tracks which UI components need updates
#[derive(Debug, Clone, Default)]
pub struct DifferentialState {
    /// Components that need re-rendering
    dirty_components: HashSet<ComponentId>,
    /// Whether the screen has changed
    screen_changed: bool,
    /// Whether modal state changed
    modal_changed: bool,
    /// Whether notifications changed
    notifications_changed: bool,
}

/// Component identifiers for differential updates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentId {
    MainMenu,
    WalletList,
    WalletDetail,
    CreateWallet,
    ModeSelection,
    CurveSelection,
    ThresholdConfig,
    JoinSession,
    DKGProgress,
    Modal,
    NotificationBar,
    StatusBar,
}

impl DifferentialState {
    /// Create a new differential state tracker
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Mark a component as needing update
    pub fn mark_dirty(&mut self, component: ComponentId) {
        self.dirty_components.insert(component);
    }
    
    /// Check if a component needs updating
    pub fn needs_update(&self, component: ComponentId) -> bool {
        self.dirty_components.contains(&component)
    }
    
    /// Clear all dirty flags after rendering
    pub fn clear_dirty(&mut self) {
        self.dirty_components.clear();
        self.screen_changed = false;
        self.modal_changed = false;
        self.notifications_changed = false;
    }
    
    /// Calculate what needs updating based on model changes
    pub fn calculate_updates(&mut self, old_model: &Model, new_model: &Model, message: &Message) {
        // Check for screen changes
        if !screens_equal(&old_model.current_screen, &new_model.current_screen) {
            self.screen_changed = true;
            self.mark_component_for_screen(&new_model.current_screen);
        }
        
        // Check for modal changes
        if old_model.ui_state.modal != new_model.ui_state.modal {
            self.modal_changed = true;
            self.mark_dirty(ComponentId::Modal);
        }
        
        // Check for notification changes
        if old_model.ui_state.notifications != new_model.ui_state.notifications {
            self.notifications_changed = true;
            self.mark_dirty(ComponentId::NotificationBar);
        }
        
        // Check for wallet list changes
        if old_model.wallet_state.wallets.len() != new_model.wallet_state.wallets.len() {
            self.mark_dirty(ComponentId::WalletList);
            self.mark_dirty(ComponentId::MainMenu); // Main menu shows wallet count
        }
        
        // Check for selection changes
        if old_model.ui_state.selected_indices != new_model.ui_state.selected_indices {
            // Only update the currently focused component
            self.mark_component_for_screen(&new_model.current_screen);
        }
        
        // Check for wallet config changes (curve selection, threshold, etc.)
        if old_model.wallet_config.curve != new_model.wallet_config.curve {
            // Curve changed, update CurveSelection component if on that screen
            if matches!(new_model.current_screen, Screen::CurveSelection) {
                self.mark_dirty(ComponentId::CurveSelection);
            }
        }
        
        if old_model.wallet_config.total_participants != new_model.wallet_config.total_participants
            || old_model.wallet_config.threshold != new_model.wallet_config.threshold {
            // Threshold config changed
            if matches!(new_model.current_screen, Screen::ThresholdConfig) {
                self.mark_dirty(ComponentId::ThresholdConfig);
            }
        }
        
        // Check for UI threshold config changes (the actual values being edited)
        if old_model.ui_state.threshold_config.participants != new_model.ui_state.threshold_config.participants
            || old_model.ui_state.threshold_config.threshold != new_model.ui_state.threshold_config.threshold
            || old_model.ui_state.threshold_config.selected_field != new_model.ui_state.threshold_config.selected_field {
            // UI threshold config changed
            if matches!(new_model.current_screen, Screen::ThresholdConfig) {
                self.mark_dirty(ComponentId::ThresholdConfig);
            }
        }
        
        // Check for session_invites changes
        if old_model.session_invites != new_model.session_invites {
            // Sessions list changed, update JoinSession component
            if matches!(new_model.current_screen, Screen::JoinSession) {
                self.mark_dirty(ComponentId::JoinSession);
            }
        }
        
        // Check for JoinSession state changes
        if old_model.ui_state.join_session.selected_tab != new_model.ui_state.join_session.selected_tab
            || old_model.ui_state.join_session.selected_session != new_model.ui_state.join_session.selected_session {
            // JoinSession state changed
            if matches!(new_model.current_screen, Screen::JoinSession) {
                self.mark_dirty(ComponentId::JoinSession);
            }
        }
        
        // Handle specific message types that always need updates
        match message {
            Message::ScrollUp | Message::ScrollDown => {
                self.mark_component_for_screen(&new_model.current_screen);
            }
            Message::Refresh => {
                // Refresh current screen
                self.mark_component_for_screen(&new_model.current_screen);
            }
            Message::WalletsLoaded { .. } => {
                self.mark_dirty(ComponentId::WalletList);
            }
            Message::SessionsLoaded { .. } => {
                // Sessions updated, mark JoinSession component as dirty
                if matches!(new_model.current_screen, Screen::JoinSession) {
                    self.mark_dirty(ComponentId::JoinSession);
                }
            }
            Message::ShowNotification { .. } | Message::ClearNotification { .. } => {
                self.mark_dirty(ComponentId::NotificationBar);
            }
            _ => {}
        }
    }
    
    /// Mark the component for the current screen as dirty
    fn mark_component_for_screen(&mut self, screen: &Screen) {
        let component = match screen {
            Screen::MainMenu | Screen::Welcome => ComponentId::MainMenu,
            Screen::ManageWallets => ComponentId::WalletList,
            Screen::WalletDetail { .. } => ComponentId::WalletDetail,
            Screen::CreateWallet(_) => ComponentId::CreateWallet,
            Screen::ModeSelection => ComponentId::ModeSelection,
            Screen::CurveSelection => ComponentId::CurveSelection,
            Screen::ThresholdConfig => ComponentId::ThresholdConfig,
            Screen::JoinSession => ComponentId::JoinSession,
            Screen::DKGProgress { .. } => ComponentId::DKGProgress,
            _ => ComponentId::MainMenu,
        };
        self.mark_dirty(component);
    }
    
    /// Check if we need a full remount (screen change)
    pub fn needs_remount(&self) -> bool {
        self.screen_changed
    }
    
    /// Check if only component properties need updating
    pub fn needs_property_update(&self) -> bool {
        !self.dirty_components.is_empty() && !self.screen_changed
    }
}

/// Check if two screens are equal (considering parameters)
fn screens_equal(a: &Screen, b: &Screen) -> bool {
    match (a, b) {
        (Screen::Welcome, Screen::Welcome) => true,
        (Screen::MainMenu, Screen::MainMenu) => true,
        (Screen::ManageWallets, Screen::ManageWallets) => true,
        (Screen::WalletDetail { wallet_id: id1 }, Screen::WalletDetail { wallet_id: id2 }) => id1 == id2,
        (Screen::CreateWallet(step1), Screen::CreateWallet(step2)) => step1 == step2,
        (Screen::ModeSelection, Screen::ModeSelection) => true,
        (Screen::CurveSelection, Screen::CurveSelection) => true,
        (Screen::ThresholdConfig, Screen::ThresholdConfig) => true,
        (Screen::JoinSession, Screen::JoinSession) => true,
        (Screen::DKGProgress { .. }, Screen::DKGProgress { .. }) => true,
        (Screen::Settings, Screen::Settings) => true,
        _ => false,
    }
}

/// Efficient component updater that only modifies changed properties
pub struct ComponentUpdater {
    differential_state: DifferentialState,
}

impl ComponentUpdater {
    /// Create a new component updater
    pub fn new() -> Self {
        Self {
            differential_state: DifferentialState::new(),
        }
    }
    
    /// Process a model update and determine what needs re-rendering
    pub fn process_update(&mut self, old_model: &Model, new_model: &Model, message: &Message) -> UpdateStrategy {
        self.differential_state.calculate_updates(old_model, new_model, message);
        
        if self.differential_state.needs_remount() {
            UpdateStrategy::FullRemount
        } else if self.differential_state.needs_property_update() {
            UpdateStrategy::PartialUpdate {
                components: self.differential_state.dirty_components.clone(),
            }
        } else {
            UpdateStrategy::NoUpdate
        }
    }
    
    /// Clear the differential state after rendering
    pub fn clear(&mut self) {
        self.differential_state.clear_dirty();
    }
}

/// Strategy for updating the UI
#[derive(Debug, Clone)]
pub enum UpdateStrategy {
    /// No update needed
    NoUpdate,
    /// Full remount of all components
    FullRemount,
    /// Partial update of specific components
    PartialUpdate {
        components: HashSet<ComponentId>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_differential_state() {
        let mut state = DifferentialState::new();
        
        // Mark components as dirty
        state.mark_dirty(ComponentId::MainMenu);
        state.mark_dirty(ComponentId::WalletList);
        
        assert!(state.needs_update(ComponentId::MainMenu));
        assert!(state.needs_update(ComponentId::WalletList));
        assert!(!state.needs_update(ComponentId::Modal));
        
        // Clear dirty flags
        state.clear_dirty();
        assert!(!state.needs_update(ComponentId::MainMenu));
    }
    
    #[test]
    fn test_screen_equality() {
        assert!(screens_equal(&Screen::MainMenu, &Screen::MainMenu));
        assert!(!screens_equal(&Screen::MainMenu, &Screen::Settings));
        
        assert!(screens_equal(
            &Screen::WalletDetail { wallet_id: "123".to_string() },
            &Screen::WalletDetail { wallet_id: "123".to_string() }
        ));
        
        assert!(!screens_equal(
            &Screen::WalletDetail { wallet_id: "123".to_string() },
            &Screen::WalletDetail { wallet_id: "456".to_string() }
        ));
    }
}