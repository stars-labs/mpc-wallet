//! TUI Provider implementation for AppRunner
//! Bridges the AppRunner abstraction with the terminal UI

use super::provider::UIProvider;
use super::tui::UIMode;
use crate::protocal::signal::SessionInfo;
use crate::utils::state::PendingSigningRequest;
use crate::utils::appstate_compat::AppState;
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use frost_core::Ciphersuite;
use ratatui::{backend::Backend, Terminal};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

/// TUI state that can be safely shared between threads
#[derive(Clone)]
pub struct TuiState {
    pub ui_mode: UIMode,
    pub connection_status: bool,
    pub device_id: String,
    pub devices: Vec<String>,
    pub session_status: String,
    pub session_invites: Vec<SessionInfo>,
    pub active_session: Option<SessionInfo>,
    pub dkg_status: String,
    pub generated_address: Option<String>,
    pub group_public_key: Option<String>,
    pub signing_requests: Vec<PendingSigningRequest>,
    pub signing_status: String,
    pub wallet_list: Vec<String>,
    pub selected_wallet: Option<String>,
    pub logs: Vec<String>,
    pub log_scroll: u16,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub is_busy: bool,
    pub progress: Option<f32>,
    pub wallet_creation_mode: Option<usize>,  // 0: Online, 1: Offline, 2: Hybrid
    pub wallet_creation_curve: Option<usize>, // 0: secp256k1, 1: ed25519
}

impl TuiState {
    pub fn new() -> Self {
        Self {
            ui_mode: UIMode::WelcomeScreen,
            connection_status: false,
            device_id: String::new(),
            devices: Vec::new(),
            session_status: "No active session".to_string(),
            session_invites: Vec::new(),
            active_session: None,
            dkg_status: "Idle".to_string(),
            generated_address: None,
            group_public_key: None,
            signing_requests: Vec::new(),
            signing_status: String::new(),
            wallet_list: Vec::new(),
            selected_wallet: None,
            logs: Vec::new(),
            log_scroll: 0,
            error_message: None,
            success_message: None,
            is_busy: false,
            progress: None,
            wallet_creation_mode: None,
            wallet_creation_curve: None,
        }
    }
}

/// TUI Provider that implements UIProvider for terminal UI
pub struct TuiProvider<B: Backend> {
    pub state: Arc<Mutex<TuiState>>,
    terminal: Arc<Mutex<Terminal<B>>>,
    redraw_tx: mpsc::UnboundedSender<()>,
}

impl<B: Backend + Send + 'static> TuiProvider<B> {
    pub fn new(terminal: Terminal<B>) -> (Self, mpsc::UnboundedReceiver<()>) {
        let (redraw_tx, redraw_rx) = mpsc::unbounded_channel();
        
        let provider = Self {
            state: Arc::new(Mutex::new(TuiState::new())),
            terminal: Arc::new(Mutex::new(terminal)),
            redraw_tx,
        };
        
        (provider, redraw_rx)
    }
    
    // Note: TuiState doesn't implement Clone, return Arc instead if needed
    pub fn get_state_arc(&self) -> Arc<Mutex<TuiState>> {
        self.state.clone()
    }
    
    pub async fn set_ui_mode(&self, mode: UIMode) {
        let mut state = self.state.lock().await;
        state.ui_mode = mode;
        let _ = self.redraw_tx.send(());
    }
    
    pub async fn handle_key_event(&self, key: KeyEvent) -> Option<String> {
        // Log every key event for debugging
        tracing::info!("=== KEY EVENT RECEIVED ===");
        tracing::info!("Key: {:?}", key);
        
        let mut state = self.state.lock().await;
        
        // Log current UI state
        tracing::info!("Current UI Mode: {:?}", state.ui_mode);
        tracing::info!("DKG Status: '{}'", state.dkg_status);
        
        // Store values we might need later
        let wallet_creation_mode = state.wallet_creation_mode;
        let wallet_creation_curve = state.wallet_creation_curve;
        
        match &mut state.ui_mode {
            UIMode::MainMenu { selected_index } => {
                match key.code {
                    KeyCode::Up => {
                        if *selected_index > 0 {
                            *selected_index -= 1;
                            let _ = self.redraw_tx.send(());
                        }
                    }
                    KeyCode::Down => {
                        if *selected_index < 5 { // 6 menu items (0-5)
                            *selected_index += 1;
                            let _ = self.redraw_tx.send(());
                        }
                    }
                    KeyCode::Enter => {
                        match *selected_index {
                            0 => {
                                // Create Wallet - transition to mode selection
                                state.ui_mode = UIMode::ModeSelection { selected_index: 0 };
                                let _ = self.redraw_tx.send(());
                            }
                            1 => {
                                // Create/Join Session - transition to SessionDiscovery
                                state.ui_mode = UIMode::SessionDiscovery {
                                    selected_index: 0,
                                    filter_text: String::new(),
                                    input_mode: false,
                                };
                                let _ = self.redraw_tx.send(());
                                // Trigger session discovery to find available sessions
                                return Some("discover_sessions".to_string());
                            }
                            2 => return Some("list_wallets".to_string()),
                            3 => return Some("view_wallets".to_string()),
                            4 => return Some("sign_transaction".to_string()),
                            5 => return Some("accept_session".to_string()),
                            _ => {}
                        }
                    }
                    KeyCode::Esc => return Some("quit".to_string()),
                    _ => {}
                }
            }
            UIMode::SessionProposalPopup { 
                session_name, 
                total_participants, 
                threshold,
                participants,
                selected_field 
            } => {
                match key.code {
                    KeyCode::Tab => {
                        // Move to next field
                        *selected_field = (*selected_field + 1) % 4;
                        let _ = self.redraw_tx.send(());
                    }
                    KeyCode::Char(c) => {
                        // Add character to current field
                        match selected_field {
                            0 => session_name.push(c),
                            1 => total_participants.push(c),
                            2 => threshold.push(c),
                            3 => participants.push(c),
                            _ => {}
                        }
                        let _ = self.redraw_tx.send(());
                    }
                    KeyCode::Backspace => {
                        // Remove character from current field
                        match selected_field {
                            0 => { session_name.pop(); }
                            1 => { total_participants.pop(); }
                            2 => { threshold.pop(); }
                            3 => { participants.pop(); }
                            _ => {}
                        }
                        let _ = self.redraw_tx.send(());
                    }
                    KeyCode::Enter => {
                        // Submit the proposal - return command for main event loop
                        if !session_name.is_empty() && !total_participants.is_empty() && 
                           !threshold.is_empty() && !participants.is_empty() {
                            let command = format!("propose_session:{}:{}:{}:{}", 
                                session_name, total_participants, threshold, participants);
                            return Some(command);
                        }
                    }
                    KeyCode::Esc => {
                        state.ui_mode = UIMode::MainMenu { selected_index: 0 };
                        let _ = self.redraw_tx.send(());
                    }
                    _ => {}
                }
            }
            // Mode Selection (Online/Offline/Hybrid)
            UIMode::ModeSelection { selected_index } => {
                match key.code {
                    KeyCode::Up => {
                        if *selected_index > 0 {
                            *selected_index -= 1;
                            let _ = self.redraw_tx.send(());
                        }
                    }
                    KeyCode::Down => {
                        if *selected_index < 2 { // 3 options: Online, Offline, Hybrid
                            *selected_index += 1;
                            let _ = self.redraw_tx.send(());
                        }
                    }
                    KeyCode::Enter => {
                        // Store selected mode and transition to curve selection
                        state.wallet_creation_mode = Some(*selected_index);
                        state.ui_mode = UIMode::CurveSelection { selected_index: 0 };
                        let _ = self.redraw_tx.send(());
                    }
                    KeyCode::Esc => {
                        state.ui_mode = UIMode::MainMenu { selected_index: 0 };
                        let _ = self.redraw_tx.send(());
                    }
                    _ => {}
                }
            }
            
            // Curve Selection
            UIMode::CurveSelection { selected_index } => {
                match key.code {
                    KeyCode::Up => {
                        if *selected_index > 0 {
                            *selected_index -= 1;
                            let _ = self.redraw_tx.send(());
                        }
                    }
                    KeyCode::Down => {
                        if *selected_index < 1 { // 2 options: secp256k1, ed25519
                            *selected_index += 1;
                            let _ = self.redraw_tx.send(());
                        }
                    }
                    KeyCode::Enter => {
                        // Store selected curve and transition to template selection
                        state.wallet_creation_curve = Some(*selected_index);
                        
                        // Generate auto wallet name
                        let auto_generated_name = crate::ui::wallet_templates::generate_themed_wallet_name();
                        
                        state.ui_mode = UIMode::TemplateSelection { 
                            selected_index: 0,
                            auto_generated_name,
                        };
                        let _ = self.redraw_tx.send(());
                    }
                    KeyCode::Esc => {
                        state.ui_mode = UIMode::ModeSelection { selected_index: 0 };
                        let _ = self.redraw_tx.send(());
                    }
                    _ => {}
                }
            }
            
            // Template Selection - New streamlined approach
            UIMode::TemplateSelection { selected_index, auto_generated_name } => {
                match key.code {
                    KeyCode::Up => {
                        if *selected_index > 0 {
                            *selected_index -= 1;
                            let _ = self.redraw_tx.send(());
                        }
                    }
                    KeyCode::Down => {
                        if *selected_index < crate::ui::wallet_templates::WALLET_TEMPLATES.len() - 1 {
                            *selected_index += 1;
                            let _ = self.redraw_tx.send(());
                        }
                    }
                    KeyCode::Enter => {
                        // Get selected template and create wallet session immediately
                        let template = &crate::ui::wallet_templates::WALLET_TEMPLATES[*selected_index];
                        
                        // If custom template is selected, go to manual configuration
                        if template.name == "Custom Setup" {
                            state.ui_mode = UIMode::WalletConfiguration { 
                                wallet_name: auto_generated_name.clone(),
                                description: String::new(),
                                total: String::from("3"),
                                threshold: String::from("2"),
                                timeout_hours: String::from("24"),
                                auto_discovery: true,
                                blockchain_configs: vec![("ethereum".to_string(), true)],
                                selected_blockchain: 0,
                                selected_field: 0,
                            };
                            let _ = self.redraw_tx.send(());
                        } else {
                            // Use template values and create session immediately
                            let mode = wallet_creation_mode.unwrap_or(0);
                            let curve = wallet_creation_curve.unwrap_or(0);
                            
                            tracing::info!("🚀 Creating wallet from template: {} ({}/{}) mode={} curve={}", 
                                auto_generated_name, template.threshold, template.total, mode, curve);
                            
                            let command = format!("create_wallet_session:{}:{}:{}:{}:{}", 
                                auto_generated_name, template.total, template.threshold, mode, curve);
                            
                            tracing::info!("📤 Returning command: {}", command);
                            
                            // Update UI to show progress
                            state.ui_mode = UIMode::DkgProgress { allow_cancel: true };
                            let _ = self.redraw_tx.send(());
                            
                            return Some(command);
                        }
                    }
                    KeyCode::Esc => {
                        state.ui_mode = UIMode::CurveSelection { selected_index: 0 };
                        let _ = self.redraw_tx.send(());
                    }
                    _ => {}
                }
            }
            
            // Wallet Configuration (now only for custom templates)
            UIMode::WalletConfiguration { 
                wallet_name,
                description,
                total,
                threshold,
                timeout_hours,
                auto_discovery: _,
                blockchain_configs: _,
                selected_blockchain: _,
                selected_field
            } => {
                match key.code {
                    KeyCode::Tab => {
                        *selected_field = (*selected_field + 1) % 5;
                        let _ = self.redraw_tx.send(());
                    }
                    KeyCode::Char(c) => {
                        // Check if 'S' or 's' is pressed for Start Creation
                        if c == 's' || c == 'S' {
                            if !wallet_name.is_empty() && !total.is_empty() && !threshold.is_empty() {
                                // Use pre-captured values
                                let mode = wallet_creation_mode.unwrap_or(0);
                                let curve = wallet_creation_curve.unwrap_or(0);
                                
                                tracing::info!("🚀 'S' key pressed - Creating wallet session: {} ({}/{})", 
                                    wallet_name, threshold, total);
                                
                                let command = format!("create_wallet_session:{}:{}:{}:{}:{}", 
                                    wallet_name, total, threshold, mode, curve);
                                return Some(command);
                            } else {
                                // Show message that fields are required
                                tracing::warn!("Cannot create session - ensure wallet name, total, and threshold are filled");
                            }
                        } else {
                            // Regular character input for fields (excluding 's'/'S')
                            match selected_field {
                                0 => wallet_name.push(c),
                                1 => description.push(c),
                                2 => total.push(c),
                                3 => threshold.push(c),
                                4 => timeout_hours.push(c),
                                _ => {}
                            }
                            let _ = self.redraw_tx.send(());
                        }
                    }
                    KeyCode::Backspace => {
                        match selected_field {
                            0 => { wallet_name.pop(); }
                            1 => { description.pop(); }
                            2 => { total.pop(); }
                            3 => { threshold.pop(); }
                            4 => { timeout_hours.pop(); }
                            _ => {}
                        }
                        let _ = self.redraw_tx.send(());
                    }
                    KeyCode::Enter => {
                        // If on wallet name field and it's not empty, move to next field
                        if *selected_field == 0 && !wallet_name.is_empty() {
                            *selected_field = 1; // Move to description field
                            let _ = self.redraw_tx.send(());
                        } else if *selected_field == 1 {
                            *selected_field = 2; // Move to total field
                            let _ = self.redraw_tx.send(());
                        } else if *selected_field == 2 && !total.is_empty() {
                            *selected_field = 3; // Move to threshold field
                            let _ = self.redraw_tx.send(());
                        } else if *selected_field == 3 && !threshold.is_empty() {
                            // On threshold field with valid input - auto-create session
                            // Skip the timeout field since it has a default value
                            if !wallet_name.is_empty() && !total.is_empty() && !threshold.is_empty() {
                                // Validate numeric fields
                                if let (Ok(total_num), Ok(threshold_num)) = (total.parse::<u16>(), threshold.parse::<u16>()) {
                                    if total_num >= 2 && threshold_num >= 1 && threshold_num <= total_num {
                                        let mode = wallet_creation_mode.unwrap_or(0);
                                        let curve = wallet_creation_curve.unwrap_or(0);
                                        
                                        tracing::info!("✅ Auto-creating wallet session after threshold: {} ({}/{})", 
                                            wallet_name, threshold, total);
                                        
                                        let command = format!("create_wallet_session:{}:{}:{}:{}:{}", 
                                            wallet_name, total, threshold, mode, curve);
                                        return Some(command);
                                    } else {
                                        tracing::warn!("Invalid threshold/total: threshold must be between 1 and {}", total);
                                        // Move to timeout field if validation fails
                                        *selected_field = 4;
                                        let _ = self.redraw_tx.send(());
                                    }
                                } else {
                                    tracing::warn!("Invalid numeric values for total or threshold");
                                    // Move to timeout field if parsing fails
                                    *selected_field = 4;
                                    let _ = self.redraw_tx.send(());
                                }
                            }
                        } else if *selected_field == 4 && !timeout_hours.is_empty() {
                            // On last field with valid input - auto-create session
                            if !wallet_name.is_empty() && !total.is_empty() && !threshold.is_empty() {
                                // Use pre-captured values
                                let mode = wallet_creation_mode.unwrap_or(0);
                                let curve = wallet_creation_curve.unwrap_or(0);
                                
                                tracing::info!("🚀 Enter key pressed on final field - Auto-creating wallet session: {} ({}/{})", 
                                    wallet_name, threshold, total);
                                
                                let command = format!("create_wallet_session:{}:{}:{}:{}:{}", 
                                    wallet_name, total, threshold, mode, curve);
                                return Some(command);
                            } else {
                                tracing::warn!("Cannot create session - ensure wallet name, total, and threshold are filled");
                            }
                        } else if *selected_field == 4 {
                            // On timeout field but empty - stay on field
                            tracing::info!("Timeout field is empty - please enter a value");
                        }
                    }
                    KeyCode::Esc => {
                        // Go back to template selection with the same generated name
                        let auto_generated_name = wallet_name.clone();
                        state.ui_mode = UIMode::TemplateSelection { 
                            selected_index: crate::ui::wallet_templates::WALLET_TEMPLATES.len() - 1, // Select custom template
                            auto_generated_name,
                        };
                        let _ = self.redraw_tx.send(());
                    }
                    _ => {}
                }
            }
            
            // Session Discovery
            UIMode::SessionDiscovery { filter_text: _, input_mode: _, selected_index } => {
                match key.code {
                    KeyCode::Up => {
                        if *selected_index > 0 {
                            *selected_index -= 1;
                            let _ = self.redraw_tx.send(());
                        }
                    }
                    KeyCode::Down => {
                        // Handled dynamically based on sessions list
                        let _ = self.redraw_tx.send(());
                    }
                    KeyCode::Enter => {
                        // Join selected session
                        return Some(format!("join_session:{}", selected_index));
                    }
                    KeyCode::Char('r') => {
                        // Refresh session list
                        return Some("discover_sessions".to_string());
                    }
                    KeyCode::Esc => {
                        state.ui_mode = UIMode::MainMenu { selected_index: 0 };
                        let _ = self.redraw_tx.send(());
                    }
                    _ => {}
                }
            }
            
            // DKG Progress Screen
            UIMode::DkgProgress { allow_cancel } => {
                let allow_cancel_copy = *allow_cancel;  // Copy the value to avoid borrow issues
                
                // Copy values we need before the match to avoid borrow issues
                let dkg_status_clone = state.dkg_status.clone();
                let generated_address_clone = state.generated_address.clone();
                
                tracing::info!("=== KEY EVENT IN DKG PROGRESS MODE ===");
                tracing::info!("Key code: {:?}", key.code);
                tracing::info!("DKG Status: '{}'", dkg_status_clone);
                tracing::info!("Generated Address: {:?}", generated_address_clone);
                tracing::info!("Allow Cancel: {}", allow_cancel_copy);
                match key.code {
                    KeyCode::Enter | KeyCode::Char('v') | KeyCode::Char('V') => {
                        tracing::info!("Enter/V key detected!");
                        
                        // Check for DKG completion - be VERY flexible
                        let is_complete = state.dkg_status == "DKG Complete" || 
                                        state.dkg_status.to_lowercase().contains("complete") ||
                                        state.dkg_status.contains("successfully") ||
                                        state.generated_address.is_some(); // Also check if we have an address
                        
                        tracing::info!("Is DKG complete check: {} (status='{}', has_address={})", 
                            is_complete, state.dkg_status, state.generated_address.is_some());
                        
                        if is_complete {
                            tracing::info!("✅ Switching to wallet complete view!");
                            state.ui_mode = UIMode::WalletComplete { 
                                selected_action: 0, 
                                show_address_details: true 
                            };
                            // Force immediate redraw
                            drop(state); // Release lock before sending
                            let _ = self.redraw_tx.send(());
                            tracing::info!("✅ UI mode changed and redraw requested");
                            return None; // Don't return a command, just update UI
                        } else {
                            tracing::warn!("❌ DKG not complete. Status '{}' doesn't match", state.dkg_status);
                        }
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        // Debug key - log current state
                        tracing::info!("=== DEBUG KEY PRESSED ===");
                        tracing::info!("UI Mode: {:?}", state.ui_mode);
                        tracing::info!("DKG Status: '{}'", state.dkg_status);
                        tracing::info!("Generated Address: {:?}", state.generated_address);
                        tracing::info!("Selected Wallet: {:?}", state.selected_wallet);
                        tracing::info!("Active Session: {:?}", state.active_session.as_ref().map(|s| &s.session_id));
                        tracing::info!("==================");
                        
                        // Show debug info in UI
                        let debug_msg = format!("DEBUG: Mode={:?}, DKG='{}', Addr={:?}", 
                            state.ui_mode,
                            state.dkg_status,
                            state.generated_address.as_ref().map(|a| &a[..10.min(a.len())])
                        );
                        state.logs.push(debug_msg.clone());
                        tracing::info!("Added to logs: {}", debug_msg);
                        
                        // Force redraw
                        drop(state);
                        let _ = self.redraw_tx.send(());
                        tracing::info!("Debug info displayed and redraw requested");
                        return None;
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        // Retry DKG if failed
                        if state.dkg_status.contains("Failed") || state.dkg_status.contains("Error") {
                            return Some("retry_dkg".to_string());
                        }
                    }
                    KeyCode::Esc => {
                        tracing::info!("Esc key pressed. Current DKG status: '{}'", state.dkg_status);
                        
                        // Check for DKG completion
                        let is_complete = state.dkg_status == "DKG Complete" || 
                                        state.dkg_status == "Complete" ||
                                        state.dkg_status.contains("Complete") ||
                                        state.dkg_status.contains("successfully") ||
                                        state.generated_address.is_some();
                        
                        if is_complete {
                            tracing::info!("DKG complete, returning to main menu");
                            state.ui_mode = UIMode::MainMenu { selected_index: 0 };
                            // Force immediate redraw
                            drop(state);
                            let _ = self.redraw_tx.send(());
                            return None;
                        } else if allow_cancel_copy {
                            tracing::info!("DKG in progress, cancelling...");
                            // Cancel DKG if still in progress
                            return Some("cancel_dkg".to_string());
                        } else {
                            tracing::info!("Cannot cancel at this stage");
                        }
                    }
                    _ => {}
                }
            }
            
            // Wallet Complete Screen
            UIMode::WalletComplete { selected_action, show_address_details } => {
                if *show_address_details {
                    // When showing address details
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('b') | KeyCode::Char('B') => {
                            // Go back to action menu
                            *show_address_details = false;
                            let _ = self.redraw_tx.send(());
                        }
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            // Return to main menu
                            state.ui_mode = UIMode::MainMenu { selected_index: 0 };
                            let _ = self.redraw_tx.send(());
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            // TODO: Copy address to clipboard
                            state.logs.push("Address copy not yet implemented".to_string());
                            let _ = self.redraw_tx.send(());
                        }
                        _ => {}
                    }
                } else {
                    // When showing action menu
                    match key.code {
                        KeyCode::Up => {
                            if *selected_action > 0 {
                                *selected_action -= 1;
                                let _ = self.redraw_tx.send(());
                            }
                        }
                        KeyCode::Down => {
                            if *selected_action < 4 { // 5 actions (0-4)
                                *selected_action += 1;
                                let _ = self.redraw_tx.send(());
                            }
                        }
                        KeyCode::Enter => {
                            match *selected_action {
                                0 => {
                                    // View Addresses
                                    *show_address_details = true;
                                    let _ = self.redraw_tx.send(());
                                }
                                1 => {
                                    // Export Wallet
                                    return Some("export_wallet".to_string());
                                }
                                2 => {
                                    // Create Backup
                                    return Some("create_backup".to_string());
                                }
                                3 => {
                                    // Send Transaction
                                    return Some("send_transaction".to_string());
                                }
                                4 => {
                                    // Return to Main Menu
                                    state.ui_mode = UIMode::MainMenu { selected_index: 0 };
                                    let _ = self.redraw_tx.send(());
                                }
                                _ => {}
                            }
                        }
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                            // Return to main menu
                            state.ui_mode = UIMode::MainMenu { selected_index: 0 };
                            let _ = self.redraw_tx.send(());
                        }
                        _ => {}
                    }
                }
            }
            
            // Welcome Screen
            UIMode::WelcomeScreen => {
                match key.code {
                    KeyCode::Enter => {
                        // Transition to main menu
                        state.ui_mode = UIMode::MainMenu { selected_index: 0 };
                        let _ = self.redraw_tx.send(());
                    }
                    KeyCode::Char('q') => return Some("quit".to_string()),
                    _ => {}
                }
            }
            
            // Handle other UI modes as needed
            _ => {}
        }
        
        None
    }
    
    pub async fn render<C: Ciphersuite>(&self, app_state: &AppState<C>) {
        let mut terminal = self.terminal.lock().await;
        let state = self.state.lock().await;
        let input = String::new(); // TODO: Get from state if needed
        let input_mode = false; // TODO: Get from state if needed
        
        // Call draw_main_ui which handles terminal.draw internally
        let _ = super::tui::draw_main_ui(&mut *terminal, app_state, &input, input_mode, &state.ui_mode);
    }
    
    async fn request_redraw(&self) {
        let _ = self.redraw_tx.send(());
    }
}

impl<B: Backend> Clone for TuiProvider<B> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            terminal: self.terminal.clone(),
            redraw_tx: self.redraw_tx.clone(),
        }
    }
}

#[async_trait]
impl<B: Backend + Send + 'static> UIProvider for TuiProvider<B> {
    async fn set_connection_status(&self, connected: bool) {
        let mut state = self.state.lock().await;
        state.connection_status = connected;
        self.request_redraw().await;
    }
    
    async fn set_device_id(&self, device_id: String) {
        let mut state = self.state.lock().await;
        state.device_id = device_id;
        self.request_redraw().await;
    }
    
    async fn update_device_list(&self, devices: Vec<String>) {
        let mut state = self.state.lock().await;
        state.devices = devices;
        self.request_redraw().await;
    }
    
    async fn update_device_status(&self, _device_id: String, _status: String) {
        // Update device-specific status if needed
        self.request_redraw().await;
    }
    
    async fn update_session_status(&self, status: String) {
        let mut state = self.state.lock().await;
        state.session_status = status;
        self.request_redraw().await;
    }
    
    async fn add_session_invite(&self, invite: SessionInfo) {
        let mut state = self.state.lock().await;
        state.session_invites.push(invite);
        self.request_redraw().await;
    }
    
    async fn remove_session_invite(&self, session_id: String) {
        let mut state = self.state.lock().await;
        state.session_invites.retain(|i| i.session_id != session_id);
        self.request_redraw().await;
    }
    
    async fn set_active_session(&self, session: Option<SessionInfo>) {
        let mut state = self.state.lock().await;
        state.active_session = session;
        self.request_redraw().await;
    }
    
    async fn update_dkg_status(&self, status: String) {
        let mut state = self.state.lock().await;
        let old_status = state.dkg_status.clone();
        tracing::info!("Updating DKG status from '{}' to '{}'", old_status, status);
        state.dkg_status = status.clone();
        
        // If DKG just completed, log additional info
        if status.contains("Complete") && !old_status.contains("Complete") {
            tracing::info!("🎊 DKG transition to complete detected in UI provider!");
            tracing::info!("  Generated address: {:?}", state.generated_address);
            tracing::info!("  Group public key: {:?}", state.group_public_key);
        }
        
        self.request_redraw().await;
    }
    
    async fn set_generated_address(&self, address: Option<String>) {
        let mut state = self.state.lock().await;
        if address != state.generated_address {
            tracing::info!("Setting generated address: {:?}", address);
        }
        state.generated_address = address;
        self.request_redraw().await;
    }
    
    async fn set_group_public_key(&self, key: Option<String>) {
        let mut state = self.state.lock().await;
        state.group_public_key = key;
        self.request_redraw().await;
    }
    
    async fn add_signing_request(&self, request: PendingSigningRequest) {
        let mut state = self.state.lock().await;
        state.signing_requests.push(request);
        self.request_redraw().await;
    }
    
    async fn remove_signing_request(&self, signing_id: String) {
        let mut state = self.state.lock().await;
        state.signing_requests.retain(|r| r.signing_id != signing_id);
        self.request_redraw().await;
    }
    
    async fn update_signing_status(&self, status: String) {
        let mut state = self.state.lock().await;
        state.signing_status = status;
        self.request_redraw().await;
    }
    
    async fn set_signature_result(&self, _signing_id: String, _signature: Vec<u8>) {
        // Handle signature result
        self.request_redraw().await;
    }
    
    async fn update_wallet_list(&self, wallets: Vec<String>) {
        let mut state = self.state.lock().await;
        state.wallet_list = wallets;
        self.request_redraw().await;
    }
    
    async fn set_selected_wallet(&self, wallet_id: Option<String>) {
        let mut state = self.state.lock().await;
        state.selected_wallet = wallet_id;
        self.request_redraw().await;
    }
    
    async fn update_mesh_status(&self, ready_devices: usize, total_devices: usize) {
        let status = format!("Mesh: {}/{} ready", ready_devices, total_devices);
        self.update_session_status(status).await;
    }
    
    async fn show_error(&self, error: String) {
        let mut state = self.state.lock().await;
        state.error_message = Some(error);
        self.request_redraw().await;
    }
    
    async fn show_success(&self, message: String) {
        let mut state = self.state.lock().await;
        state.success_message = Some(message);
        self.request_redraw().await;
    }
    
    async fn set_busy(&self, busy: bool) {
        let mut state = self.state.lock().await;
        state.is_busy = busy;
        self.request_redraw().await;
    }
    
    async fn set_progress(&self, progress: Option<f32>) {
        let mut state = self.state.lock().await;
        state.progress = progress;
        self.request_redraw().await;
    }
    
    async fn add_log(&self, message: String) {
        let mut state = self.state.lock().await;
        state.logs.push(message);
        // Keep only last 100 logs to prevent memory growth
        if state.logs.len() > 100 {
            let drain_count = state.logs.len() - 100;
            state.logs.drain(0..drain_count);
        }
        self.request_redraw().await;
    }
    
    async fn set_logs(&self, logs: Vec<String>) {
        let mut state = self.state.lock().await;
        state.logs = logs;
        self.request_redraw().await;
    }
}