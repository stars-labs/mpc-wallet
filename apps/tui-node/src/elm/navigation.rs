//! Navigation System - Consistent keyboard shortcuts and navigation
//!
//! This module provides a unified navigation system for all components,
//! ensuring consistent behavior across the entire TUI.

use crate::elm::message::Message;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

/// Navigation configuration for the application
#[derive(Debug, Clone)]
pub struct NavigationConfig {
    /// Enable vim-style navigation (hjkl)
    pub vim_mode: bool,
    /// Allow wrap-around in lists (bottom to top, top to bottom)
    pub wrap_around: bool,
    /// Quick action keys (single key shortcuts)
    pub quick_keys: HashMap<char, QuickAction>,
    /// Global shortcuts (work from any screen)
    pub global_shortcuts: HashMap<KeyCombo, GlobalAction>,
}

impl Default for NavigationConfig {
    fn default() -> Self {
        let mut quick_keys = HashMap::new();
        quick_keys.insert('n', QuickAction::NewWallet);
        quick_keys.insert('j', QuickAction::JoinSession);
        quick_keys.insert('w', QuickAction::WalletList);
        quick_keys.insert('s', QuickAction::SignTransaction);
        quick_keys.insert('/', QuickAction::Search);
        quick_keys.insert('?', QuickAction::Help);
        
        let mut global_shortcuts = HashMap::new();
        global_shortcuts.insert(
            KeyCombo::new(KeyCode::Char('q'), KeyModifiers::CONTROL),
            GlobalAction::Quit,
        );
        global_shortcuts.insert(
            KeyCombo::new(KeyCode::Char('r'), KeyModifiers::CONTROL),
            GlobalAction::Refresh,
        );
        global_shortcuts.insert(
            KeyCombo::new(KeyCode::Char('h'), KeyModifiers::CONTROL),
            GlobalAction::Home,
        );
        
        Self {
            vim_mode: true,
            wrap_around: true,
            quick_keys,
            global_shortcuts,
        }
    }
}

/// Key combination for shortcuts
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyCombo {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyCombo {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }
    
    pub fn from_event(event: &KeyEvent) -> Self {
        Self {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}

/// Quick actions accessible with single key press
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuickAction {
    NewWallet,
    JoinSession,
    WalletList,
    SignTransaction,
    Search,
    Help,
}

/// Global actions that work from any screen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalAction {
    Quit,
    Refresh,
    Home,
}

/// Standard navigation actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationAction {
    Up,
    Down,
    Left,
    Right,
    Select,
    Back,
    NextField,
    PreviousField,
    FirstItem,
    LastItem,
    PageUp,
    PageDown,
}

/// Trait for components that support navigation
pub trait NavigationHandler {
    /// Get the navigation configuration
    fn nav_config(&self) -> &NavigationConfig;
    
    /// Handle navigation input and return appropriate message
    fn handle_navigation(&mut self, event: KeyEvent) -> Option<Message> {
        let config = self.nav_config();
        
        // Check global shortcuts first
        let key_combo = KeyCombo::from_event(&event);
        if let Some(action) = config.global_shortcuts.get(&key_combo) {
            return Some(self.handle_global_action(*action));
        }
        
        // Check quick keys (only when no modifiers pressed)
        if event.modifiers.is_empty() {
            if let KeyCode::Char(ch) = event.code {
                if let Some(action) = config.quick_keys.get(&ch) {
                    return Some(self.handle_quick_action(*action));
                }
            }
        }
        
        // Handle standard navigation
        match self.map_key_to_navigation(event) {
            Some(nav_action) => Some(self.handle_nav_action(nav_action)),
            None => None,
        }
    }
    
    /// Map key event to navigation action
    fn map_key_to_navigation(&self, event: KeyEvent) -> Option<NavigationAction> {
        let config = self.nav_config();
        
        match event.code {
            // Arrow keys
            KeyCode::Up => Some(NavigationAction::Up),
            KeyCode::Down => Some(NavigationAction::Down),
            KeyCode::Left => Some(NavigationAction::Left),
            KeyCode::Right => Some(NavigationAction::Right),
            
            // Vim navigation (if enabled)
            KeyCode::Char('k') if config.vim_mode => Some(NavigationAction::Up),
            KeyCode::Char('j') if config.vim_mode => Some(NavigationAction::Down),
            KeyCode::Char('h') if config.vim_mode => Some(NavigationAction::Left),
            KeyCode::Char('l') if config.vim_mode => Some(NavigationAction::Right),
            
            // Common actions
            KeyCode::Enter => Some(NavigationAction::Select),
            KeyCode::Esc => Some(NavigationAction::Back),
            KeyCode::Tab => Some(NavigationAction::NextField),
            KeyCode::BackTab => Some(NavigationAction::PreviousField),
            
            // Jump navigation
            KeyCode::Home => Some(NavigationAction::FirstItem),
            KeyCode::End => Some(NavigationAction::LastItem),
            KeyCode::PageUp => Some(NavigationAction::PageUp),
            KeyCode::PageDown => Some(NavigationAction::PageDown),
            
            // Vim-style jumps
            KeyCode::Char('g') if config.vim_mode && event.modifiers.is_empty() => {
                Some(NavigationAction::FirstItem)
            }
            KeyCode::Char('G') if config.vim_mode && event.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(NavigationAction::LastItem)
            }
            
            _ => None,
        }
    }
    
    /// Handle navigation action - to be implemented by each component
    fn handle_nav_action(&mut self, action: NavigationAction) -> Message;
    
    /// Handle quick action
    fn handle_quick_action(&mut self, action: QuickAction) -> Message {
        match action {
            QuickAction::NewWallet => Message::Navigate(crate::elm::model::Screen::CreateWallet(
                crate::elm::model::CreateWalletStep::ModeSelection
            )),
            QuickAction::JoinSession => Message::Navigate(crate::elm::model::Screen::JoinSession),
            QuickAction::WalletList => Message::Navigate(crate::elm::model::Screen::ManageWallets),
            QuickAction::SignTransaction => Message::Info {
                message: "Select a wallet first to sign".to_string(),
            },
            QuickAction::Search => Message::StartSearch,
            QuickAction::Help => Message::ShowHelp,
        }
    }
    
    /// Handle global action
    fn handle_global_action(&mut self, action: GlobalAction) -> Message {
        match action {
            GlobalAction::Quit => Message::Quit,
            GlobalAction::Refresh => Message::Refresh,
            GlobalAction::Home => Message::NavigateHome,
        }
    }
}

/// Helper trait for list navigation
pub trait ListNavigator {
    /// Get current selected index
    fn selected_index(&self) -> usize;
    
    /// Set selected index
    fn set_selected_index(&mut self, index: usize);
    
    /// Get total number of items
    fn item_count(&self) -> usize;
    
    /// Whether to wrap around at boundaries
    fn wrap_around(&self) -> bool;
    
    /// Navigate up in the list
    fn navigate_up(&mut self) {
        let current = self.selected_index();
        let count = self.item_count();
        
        if count == 0 {
            return;
        }
        
        let new_index = if current > 0 {
            current - 1
        } else if self.wrap_around() {
            count - 1
        } else {
            0
        };
        
        self.set_selected_index(new_index);
    }
    
    /// Navigate down in the list
    fn navigate_down(&mut self) {
        let current = self.selected_index();
        let count = self.item_count();
        
        if count == 0 {
            return;
        }
        
        let new_index = if current < count - 1 {
            current + 1
        } else if self.wrap_around() {
            0
        } else {
            count - 1
        };
        
        self.set_selected_index(new_index);
    }
    
    /// Jump to first item
    fn navigate_first(&mut self) {
        self.set_selected_index(0);
    }
    
    /// Jump to last item
    fn navigate_last(&mut self) {
        let count = self.item_count();
        if count > 0 {
            self.set_selected_index(count - 1);
        }
    }
    
    /// Navigate by page
    fn navigate_page_up(&mut self, page_size: usize) {
        let current = self.selected_index();
        let new_index = current.saturating_sub(page_size);
        self.set_selected_index(new_index);
    }
    
    /// Navigate by page
    fn navigate_page_down(&mut self, page_size: usize) {
        let current = self.selected_index();
        let count = self.item_count();
        let new_index = (current + page_size).min(count.saturating_sub(1));
        self.set_selected_index(new_index);
    }
}

/// Visual hint for available shortcuts
#[derive(Debug, Clone)]
pub struct ShortcutHint {
    pub key: String,
    pub description: String,
    pub available: bool,
}

impl ShortcutHint {
    pub fn new(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
            available: true,
        }
    }
    
    pub fn disabled(mut self) -> Self {
        self.available = false;
        self
    }
}

/// Generate shortcut hints for current context
pub fn get_context_shortcuts(screen: &crate::elm::model::Screen) -> Vec<ShortcutHint> {
    use crate::elm::model::Screen;
    
    let mut hints = vec![
        ShortcutHint::new("↑↓", "Navigate"),
        ShortcutHint::new("Enter", "Select"),
        ShortcutHint::new("Esc", "Back"),
    ];
    
    // Add screen-specific hints
    match screen {
        Screen::MainMenu | Screen::Welcome => {
            hints.push(ShortcutHint::new("n", "New Wallet"));
            hints.push(ShortcutHint::new("j", "Join Session"));
            hints.push(ShortcutHint::new("w", "Wallets"));
        }
        Screen::ManageWallets => {
            hints.push(ShortcutHint::new("n", "New"));
            hints.push(ShortcutHint::new("i", "Import"));
            hints.push(ShortcutHint::new("d", "Delete"));
            hints.push(ShortcutHint::new("/", "Search"));
        }
        Screen::CreateWallet(_) => {
            hints.push(ShortcutHint::new("Tab", "Next Field"));
            hints.push(ShortcutHint::new("←→", "Adjust Values"));
        }
        Screen::JoinSession => {
            hints.push(ShortcutHint::new("r", "Refresh"));
            hints.push(ShortcutHint::new("i", "Info"));
        }
        _ => {}
    }
    
    // Always add help
    hints.push(ShortcutHint::new("?", "Help"));
    
    hints
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_combo() {
        let combo = KeyCombo::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        let combo_from_event = KeyCombo::from_event(&event);
        
        assert_eq!(combo, combo_from_event);
    }
    
    #[test]
    fn test_navigation_config_defaults() {
        let config = NavigationConfig::default();
        
        assert!(config.vim_mode);
        assert!(config.wrap_around);
        assert_eq!(config.quick_keys.get(&'n'), Some(&QuickAction::NewWallet));
        assert_eq!(
            config.global_shortcuts.get(&KeyCombo::new(
                KeyCode::Char('q'),
                KeyModifiers::CONTROL
            )),
            Some(&GlobalAction::Quit)
        );
    }
    
    struct TestList {
        selected: usize,
        items: Vec<String>,
        wrap: bool,
    }
    
    impl ListNavigator for TestList {
        fn selected_index(&self) -> usize {
            self.selected
        }
        
        fn set_selected_index(&mut self, index: usize) {
            self.selected = index;
        }
        
        fn item_count(&self) -> usize {
            self.items.len()
        }
        
        fn wrap_around(&self) -> bool {
            self.wrap
        }
    }
    
    #[test]
    fn test_list_navigation() {
        let mut list = TestList {
            selected: 0,
            items: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            wrap: true,
        };
        
        // Test down navigation
        list.navigate_down();
        assert_eq!(list.selected, 1);
        
        // Test wrap around
        list.selected = 2;
        list.navigate_down();
        assert_eq!(list.selected, 0);
        
        // Test up navigation
        list.navigate_up();
        assert_eq!(list.selected, 2);
        
        // Test jump to first/last
        list.navigate_first();
        assert_eq!(list.selected, 0);
        
        list.navigate_last();
        assert_eq!(list.selected, 2);
    }
}