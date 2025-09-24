//! User-Friendly Error Handling
//!
//! This module provides error translation and user-friendly error messages
//! for the TUI application.

use std::collections::HashMap;

/// Error categories for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    Network,
    Validation,
    Crypto,
    Storage,
    User,
    System,
}

impl ErrorCategory {
    /// Get emoji icon for category
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Network => "🌐",
            Self::Validation => "⚠️",
            Self::Crypto => "🔐",
            Self::Storage => "💾",
            Self::User => "👤",
            Self::System => "⚙️",
        }
    }
    
    /// Get category name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Network => "Network",
            Self::Validation => "Validation",
            Self::Crypto => "Security",
            Self::Storage => "Storage",
            Self::User => "User",
            Self::System => "System",
        }
    }
}

/// User-friendly error with recovery actions
#[derive(Debug, Clone)]
pub struct UserFriendlyError {
    pub code: String,
    pub category: ErrorCategory,
    pub title: String,
    pub description: String,
    pub details: Option<String>,
    pub recovery_actions: Vec<RecoveryAction>,
    pub help_link: Option<String>,
}

impl UserFriendlyError {
    /// Create a new user-friendly error
    pub fn new(code: impl Into<String>, category: ErrorCategory, title: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            category,
            title: title.into(),
            description: String::new(),
            details: None,
            recovery_actions: Vec::new(),
            help_link: None,
        }
    }
    
    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
    
    /// Add technical details (hidden by default)
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
    
    /// Add recovery action
    pub fn with_action(mut self, action: RecoveryAction) -> Self {
        self.recovery_actions.push(action);
        self
    }
    
    /// Add help link
    pub fn with_help(mut self, link: impl Into<String>) -> Self {
        self.help_link = Some(link.into());
        self
    }
    
    /// Format for display
    pub fn format_display(&self, show_details: bool) -> String {
        let mut output = String::new();
        
        // Header with icon and code
        output.push_str(&format!("{} {} [{}]\n", 
            self.category.icon(), 
            self.title,
            self.code
        ));
        
        // Description
        if !self.description.is_empty() {
            output.push_str(&format!("\n{}\n", self.description));
        }
        
        // Technical details (if requested)
        if show_details {
            if let Some(ref details) = self.details {
                output.push_str(&format!("\nTechnical Details:\n{}\n", details));
            }
        }
        
        // Recovery actions
        if !self.recovery_actions.is_empty() {
            output.push_str("\nWhat you can do:\n");
            for (i, action) in self.recovery_actions.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, action.description()));
            }
        }
        
        // Help link
        if let Some(ref link) = self.help_link {
            output.push_str(&format!("\nMore help: {}\n", link));
        }
        
        output
    }
}

/// Recovery actions users can take
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    Retry {
        description: String,
    },
    Configure {
        setting: String,
        description: String,
    },
    CheckNetwork {
        description: String,
    },
    UpdateSoftware {
        version: Option<String>,
    },
    ContactSupport {
        error_code: String,
    },
    UseOfflineMode,
    RestoreBackup,
    ClearCache,
    Restart,
}

impl RecoveryAction {
    /// Get action description
    pub fn description(&self) -> String {
        match self {
            Self::Retry { description } => description.clone(),
            Self::Configure { description, .. } => description.clone(),
            Self::CheckNetwork { description } => description.clone(),
            Self::UpdateSoftware { version } => {
                if let Some(v) = version {
                    format!("Update to version {}", v)
                } else {
                    "Update to latest version".to_string()
                }
            }
            Self::ContactSupport { error_code } => {
                format!("Contact support with error code {}", error_code)
            }
            Self::UseOfflineMode => "Switch to offline mode".to_string(),
            Self::RestoreBackup => "Restore from backup".to_string(),
            Self::ClearCache => "Clear cache and retry".to_string(),
            Self::Restart => "Restart the application".to_string(),
        }
    }
    
    /// Get keyboard shortcut for action
    pub fn shortcut(&self) -> Option<char> {
        match self {
            Self::Retry { .. } => Some('r'),
            Self::Configure { .. } => Some('c'),
            Self::CheckNetwork { .. } => Some('n'),
            Self::UpdateSoftware { .. } => Some('u'),
            Self::ContactSupport { .. } => Some('s'),
            Self::UseOfflineMode => Some('o'),
            Self::RestoreBackup => Some('b'),
            Self::ClearCache => Some('x'),
            Self::Restart => Some('q'),
        }
    }
}

/// Error translator for converting technical errors to user-friendly ones
pub struct ErrorTranslator {
    translations: HashMap<String, UserFriendlyError>,
}

impl ErrorTranslator {
    /// Create a new error translator
    pub fn new() -> Self {
        let mut translator = Self {
            translations: HashMap::new(),
        };
        translator.register_common_errors();
        translator
    }
    
    /// Register common error translations
    fn register_common_errors(&mut self) {
        // Network errors
        self.register(
            "N001",
            UserFriendlyError::new("N001", ErrorCategory::Network, "Connection Failed")
                .with_description("Unable to connect to the coordination server. This might be due to network issues or server maintenance.")
                .with_action(RecoveryAction::Retry { 
                    description: "Try connecting again".to_string() 
                })
                .with_action(RecoveryAction::CheckNetwork {
                    description: "Check your internet connection".to_string()
                })
                .with_action(RecoveryAction::UseOfflineMode)
        );
        
        self.register(
            "N002",
            UserFriendlyError::new("N002", ErrorCategory::Network, "Peer Connection Failed")
                .with_description("Cannot establish connection with other participants. This might be due to firewall or NAT issues.")
                .with_action(RecoveryAction::Configure {
                    setting: "firewall".to_string(),
                    description: "Check firewall settings".to_string(),
                })
                .with_action(RecoveryAction::UseOfflineMode)
        );
        
        // Validation errors
        self.register(
            "V001",
            UserFriendlyError::new("V001", ErrorCategory::Validation, "Invalid Configuration")
                .with_description("The threshold must be less than or equal to the total number of participants.")
                .with_action(RecoveryAction::Configure {
                    setting: "threshold".to_string(),
                    description: "Adjust threshold settings".to_string(),
                })
        );
        
        self.register(
            "V002",
            UserFriendlyError::new("V002", ErrorCategory::Validation, "Invalid Address")
                .with_description("The provided address is not valid. Please check for typos or copy it again.")
                .with_action(RecoveryAction::Retry {
                    description: "Enter address again".to_string(),
                })
        );
        
        // Keystore errors
        self.register(
            "K001",
            UserFriendlyError::new("K001", ErrorCategory::Storage, "Keystore Locked")
                .with_description("Your keystore is locked for security. Please enter your password to continue.")
                .with_action(RecoveryAction::Retry {
                    description: "Enter password".to_string(),
                })
        );
        
        self.register(
            "K002",
            UserFriendlyError::new("K002", ErrorCategory::Storage, "Keystore Corrupted")
                .with_description("The wallet data appears to be damaged. You may need to restore from a backup.")
                .with_action(RecoveryAction::RestoreBackup)
                .with_action(RecoveryAction::ContactSupport {
                    error_code: "K002".to_string(),
                })
        );
        
        // DKG errors
        self.register(
            "D001",
            UserFriendlyError::new("D001", ErrorCategory::Crypto, "Not Enough Participants")
                .with_description("Waiting for more participants to join the key generation session.")
                .with_action(RecoveryAction::Retry {
                    description: "Wait and refresh".to_string(),
                })
        );
        
        self.register(
            "D002",
            UserFriendlyError::new("D002", ErrorCategory::Crypto, "Key Generation Failed")
                .with_description("The key generation process encountered an error. This might be due to a participant dropping out.")
                .with_action(RecoveryAction::Retry {
                    description: "Start new session".to_string(),
                })
        );
    }
    
    /// Register a custom error translation
    pub fn register(&mut self, code: impl Into<String>, error: UserFriendlyError) {
        self.translations.insert(code.into(), error);
    }
    
    /// Translate an error code to user-friendly error
    pub fn translate(&self, code: &str) -> Option<&UserFriendlyError> {
        self.translations.get(code)
    }
    
    /// Translate a technical error to user-friendly format
    pub fn translate_error(&self, error: &dyn std::error::Error) -> UserFriendlyError {
        // Try to extract error code from the error message
        let error_str = error.to_string();
        
        // Check for known patterns
        if error_str.contains("Connection refused") || error_str.contains("Network") {
            self.translate("N001").cloned().unwrap_or_else(|| {
                UserFriendlyError::new("N999", ErrorCategory::Network, "Network Error")
                    .with_description(self.simplify_message(&error_str))
                    .with_details(error_str)
            })
        } else if error_str.contains("Invalid") || error_str.contains("Validation") {
            self.translate("V001").cloned().unwrap_or_else(|| {
                UserFriendlyError::new("V999", ErrorCategory::Validation, "Validation Error")
                    .with_description(self.simplify_message(&error_str))
                    .with_details(error_str)
            })
        } else {
            // Generic error
            UserFriendlyError::new("E999", ErrorCategory::System, "Unexpected Error")
                .with_description("Something went wrong. Please try again or contact support if the problem persists.")
                .with_details(error_str)
                .with_action(RecoveryAction::Retry {
                    description: "Try again".to_string(),
                })
                .with_action(RecoveryAction::ContactSupport {
                    error_code: "E999".to_string(),
                })
        }
    }
    
    /// Simplify technical error message for users
    fn simplify_message(&self, technical: &str) -> String {
        let simplified = technical
            .replace("ECONNREFUSED", "connection was refused")
            .replace("ETIMEDOUT", "connection timed out")
            .replace("ENOTFOUND", "server not found")
            .replace("std::io::Error", "")
            .replace("anyhow::Error", "")
            .replace("()", "");
        
        // Capitalize first letter
        if let Some(first_char) = simplified.chars().next() {
            format!("{}{}", 
                first_char.to_uppercase(), 
                &simplified[first_char.len_utf8()..]
            )
        } else {
            simplified
        }
    }
}

impl Default for ErrorTranslator {
    fn default() -> Self {
        Self::new()
    }
}

/// Error dialog for displaying errors to users
#[derive(Debug, Clone)]
pub struct ErrorDialog {
    pub error: UserFriendlyError,
    pub selected_action: usize,
    pub show_details: bool,
}

impl ErrorDialog {
    /// Create a new error dialog
    pub fn new(error: UserFriendlyError) -> Self {
        Self {
            error,
            selected_action: 0,
            show_details: false,
        }
    }
    
    /// Navigate to previous action
    pub fn previous_action(&mut self) {
        if self.selected_action > 0 {
            self.selected_action -= 1;
        } else if !self.error.recovery_actions.is_empty() {
            self.selected_action = self.error.recovery_actions.len() - 1;
        }
    }
    
    /// Navigate to next action
    pub fn next_action(&mut self) {
        if !self.error.recovery_actions.is_empty() {
            self.selected_action = (self.selected_action + 1) % self.error.recovery_actions.len();
        }
    }
    
    /// Toggle technical details visibility
    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }
    
    /// Get the currently selected action
    pub fn selected_action(&self) -> Option<&RecoveryAction> {
        self.error.recovery_actions.get(self.selected_action)
    }
    
    /// Execute the selected action
    pub fn execute_action(&self) -> Option<crate::elm::message::Message> {
        use crate::elm::message::Message;
        
        self.selected_action().map(|action| {
            match action {
                RecoveryAction::Retry { .. } => Message::Refresh,
                RecoveryAction::Configure { .. } => Message::Navigate(crate::elm::model::Screen::Settings),
                RecoveryAction::UseOfflineMode => Message::Info {
                    message: "Switching to offline mode...".to_string(),
                },
                RecoveryAction::Restart => Message::Quit,
                _ => Message::None,
            }
        })
    }
}

/// Error history for tracking and debugging
#[derive(Debug, Clone)]
pub struct ErrorHistory {
    entries: Vec<ErrorEntry>,
    max_entries: usize,
}

#[derive(Debug, Clone)]
pub struct ErrorEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub error: UserFriendlyError,
    pub context: HashMap<String, String>,
    pub resolved: bool,
}

impl ErrorHistory {
    /// Create a new error history
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }
    
    /// Add an error to history
    pub fn add(&mut self, error: UserFriendlyError, context: HashMap<String, String>) {
        // Remove oldest if at capacity
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        
        self.entries.push(ErrorEntry {
            timestamp: chrono::Utc::now(),
            error,
            context,
            resolved: false,
        });
    }
    
    /// Mark the last error as resolved
    pub fn mark_resolved(&mut self) {
        if let Some(entry) = self.entries.last_mut() {
            entry.resolved = true;
        }
    }
    
    /// Get recent errors
    pub fn recent(&self, count: usize) -> Vec<&ErrorEntry> {
        let start = self.entries.len().saturating_sub(count);
        self.entries[start..].iter().collect()
    }
    
    /// Get unresolved errors
    pub fn unresolved(&self) -> Vec<&ErrorEntry> {
        self.entries.iter().filter(|e| !e.resolved).collect()
    }
    
    /// Clear history
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for ErrorHistory {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_translation() {
        let translator = ErrorTranslator::new();
        
        let error = translator.translate("N001").unwrap();
        assert_eq!(error.category, ErrorCategory::Network);
        assert!(error.title.contains("Connection"));
        assert!(!error.recovery_actions.is_empty());
    }
    
    #[test]
    fn test_error_dialog() {
        let error = UserFriendlyError::new("T001", ErrorCategory::System, "Test Error")
            .with_action(RecoveryAction::Retry { 
                description: "Try again".to_string() 
            })
            .with_action(RecoveryAction::Restart);
        
        let mut dialog = ErrorDialog::new(error);
        assert_eq!(dialog.selected_action, 0);
        
        dialog.next_action();
        assert_eq!(dialog.selected_action, 1);
        
        dialog.next_action();
        assert_eq!(dialog.selected_action, 0); // Wraps around
    }
    
    #[test]
    fn test_error_history() {
        let mut history = ErrorHistory::new(3);
        
        for i in 1..=5 {
            let error = UserFriendlyError::new(
                format!("E{:03}", i),
                ErrorCategory::System,
                format!("Error {}", i)
            );
            history.add(error, HashMap::new());
        }
        
        // Should only keep last 3
        assert_eq!(history.entries.len(), 3);
        assert_eq!(history.entries[0].error.code, "E003");
    }
}