use tuirealm::{Sub, SubClause, SubEventClause};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use crate::elm::components::{Id, UserEvent};

/// Get subscriptions for a specific component ID
pub fn get_subscriptions_for_component(id: &Id) -> Vec<Sub<Id, UserEvent>> {
    match id {
        Id::MainMenu | Id::WalletList | Id::CreateWallet | Id::CurveSelection | 
        Id::ThresholdConfig | Id::JoinSession | Id::DKGProgress | Id::SDCardManager |
        Id::OfflineDKGProcess | Id::WalletDetail => {
            // Navigation components
            SubscriptionManager::navigation_subscriptions()
        }
        Id::SessionIdInput | Id::RecoveryPhrase => {
            // Input components
            SubscriptionManager::input_subscriptions()
        }
        Id::Modal | Id::NotificationBar => {
            // These don't need keyboard subscriptions
            vec![]
        }
        _ => {
            // Default to navigation subscriptions for safety
            SubscriptionManager::navigation_subscriptions()
        }
    }
}

/// Check if a component should auto-subscribe to events
pub fn should_auto_subscribe(id: &Id) -> bool {
    match id {
        Id::Modal | Id::NotificationBar => false,
        _ => true
    }
}

/// Manages keyboard event subscriptions for all components
/// This ensures keyboard events are properly routed to active components
pub struct SubscriptionManager;

impl SubscriptionManager {
    /// Get standard keyboard subscriptions for navigation components
    pub fn navigation_subscriptions() -> Vec<Sub<Id, UserEvent>> {
        
        vec![
            // Arrow keys
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Up,
                    modifiers: KeyModifiers::empty(),
                }),
                SubClause::Always
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Down,
                    modifiers: KeyModifiers::empty(),
                }),
                SubClause::Always
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Left,
                    modifiers: KeyModifiers::empty(),
                }),
                SubClause::Always
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Right,
                    modifiers: KeyModifiers::empty(),
                }),
                SubClause::Always
            ),
            
            // Enter and Escape
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Enter,
                    modifiers: KeyModifiers::empty(),
                }),
                SubClause::Always
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Esc,
                    modifiers: KeyModifiers::empty(),
                }),
                SubClause::Always
            ),
            
            // Tab navigation
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Tab,
                    modifiers: KeyModifiers::empty(),
                }),
                SubClause::Always
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::BackTab,
                    modifiers: KeyModifiers::SHIFT,
                }),
                SubClause::Always
            ),
            
            // Quit
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char('q'),
                    modifiers: KeyModifiers::empty(),
                }),
                SubClause::Always
            ),
            Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char('Q'),
                    modifiers: KeyModifiers::SHIFT,
                }),
                SubClause::Always
            ),
        ]
    }
    
    /// Get subscriptions for input components that need character input
    pub fn input_subscriptions() -> Vec<Sub<Id, UserEvent>> {
        let mut subs = Self::navigation_subscriptions();
        
        // Add character input subscriptions
        for c in b'a'..=b'z' {
            subs.push(Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char(c as char),
                    modifiers: KeyModifiers::empty(),
                }),
                SubClause::Always
            ));
            subs.push(Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char((c as char).to_ascii_uppercase()),
                    modifiers: KeyModifiers::SHIFT,
                }),
                SubClause::Always
            ));
        }
        
        // Numbers
        for c in b'0'..=b'9' {
            subs.push(Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char(c as char),
                    modifiers: KeyModifiers::empty(),
                }),
                SubClause::Always
            ));
        }
        
        // Special characters
        for c in [' ', '-', '_', '.', '/', '\\', ':', ';', ',', '!', '?', '@', '#', '$', '%', '^', '&', '*', '(', ')', '[', ']', '{', '}', '<', '>', '=', '+', '|', '~', '`', '"', '\''].iter() {
            subs.push(Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char(*c),
                    modifiers: KeyModifiers::empty(),
                }),
                SubClause::Always
            ));
        }
        
        // Backspace and Delete
        subs.push(Sub::new(
            SubEventClause::Keyboard(KeyEvent {
                code: Key::Backspace,
                modifiers: KeyModifiers::empty(),
            }),
            SubClause::Always
        ));
        subs.push(Sub::new(
            SubEventClause::Keyboard(KeyEvent {
                code: Key::Delete,
                modifiers: KeyModifiers::empty(),
            }),
            SubClause::Always
        ));
        
        subs
    }
    
    /// Get subscriptions for numeric input components
    pub fn numeric_subscriptions() -> Vec<Sub<Id, UserEvent>> {
        let mut subs = Self::navigation_subscriptions();
        
        // Numbers only
        for c in b'0'..=b'9' {
            subs.push(Sub::new(
                SubEventClause::Keyboard(KeyEvent {
                    code: Key::Char(c as char),
                    modifiers: KeyModifiers::empty(),
                }),
                SubClause::Always
            ));
        }
        
        // Backspace
        subs.push(Sub::new(
            SubEventClause::Keyboard(KeyEvent {
                code: Key::Backspace,
                modifiers: KeyModifiers::empty(),
            }),
            SubClause::Always
        ));
        
        subs
    }
}