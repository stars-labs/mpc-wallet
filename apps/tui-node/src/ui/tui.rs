use crate::protocal::signal::SessionInfo;
use crate::handlers::session_handler::WalletSessionConfig;
use crate::utils::appstate_compat::AppState;
use crate::utils::state::{DkgStateDisplay, InternalCommand, MeshStatus}; 
use webrtc_signal_server::ClientMsg;
use crossterm::event::{KeyCode, KeyEvent};
use std::any::TypeId; // Added for ciphersuite check
use chrono; // Added for auto-generated wallet names
use ratatui::{
    Frame, // Add Frame import
    Terminal,
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    text::{Line, Span},                                         // Remove Spans
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap, Clear}, // Keep Wrap
};
use std::collections::HashSet;
use std::io;
use tokio::sync::mpsc; // For command channel // Import SessionInfo
use frost_core::Ciphersuite;

/// Wallet creation templates for simplified UX
#[derive(Debug, Clone, PartialEq)]
pub struct WalletTemplate {
    pub name: &'static str,
    pub description: &'static str,
    pub total: u16,
    pub threshold: u16,
    pub security_level: &'static str,
    pub use_case: &'static str,
}

/// Predefined wallet templates
pub const WALLET_TEMPLATES: &[WalletTemplate] = &[
    WalletTemplate {
        name: "2-of-3 Standard",
        description: "Most common setup - requires 2 out of 3 participants to sign",
        total: 3,
        threshold: 2,
        security_level: "Standard",
        use_case: "Personal/Small team",
    },
    WalletTemplate {
        name: "3-of-5 High Security",
        description: "High security - requires 3 out of 5 participants to sign",
        total: 5,
        threshold: 3,
        security_level: "High",
        use_case: "Business/Organization",
    },
    WalletTemplate {
        name: "2-of-2 Dual Control",
        description: "Both participants required to sign - maximum control",
        total: 2,
        threshold: 2,
        security_level: "Maximum",
        use_case: "Partnership/Joint custody",
    },
    WalletTemplate {
        name: "5-of-9 Enterprise",
        description: "Enterprise setup - requires majority consensus",
        total: 9,
        threshold: 5,
        security_level: "Enterprise",
        use_case: "Large organization",
    },
    WalletTemplate {
        name: "Custom Setup",
        description: "Advanced users only - manually configure threshold values",
        total: 0, // Will be set manually
        threshold: 0, // Will be set manually
        security_level: "Custom",
        use_case: "Advanced configuration",
    },
];

/// Generate auto wallet name with timestamp
pub fn generate_wallet_name() -> String {
    let now = chrono::Utc::now();
    let date = now.format("%Y-%m-%d").to_string();
    let time = now.format("%H%M").to_string();
    format!("Wallet-{}-{}", date, time)
}

/// Generate themed wallet names
pub fn generate_themed_wallet_name() -> String {
    let themes = [
        "Alpha", "Beta", "Gamma", "Delta", "Prime", "Core", "Main", "Secure",
        "Phoenix", "Nexus", "Vault", "Shield", "Guard", "Sentinel", "Fortress"
    ];
    let now = chrono::Utc::now();
    let theme_index = (now.timestamp() % themes.len() as i64) as usize;
    let counter = (now.timestamp() / 3600) % 999; // Changes every hour
    format!("MPC-{}-{:03}", themes[theme_index], counter)
}

#[derive(Debug, Clone, PartialEq)]
pub enum UIMode {
    Normal,
    // Legacy modes
    MainMenu { selected_index: usize },
    SigningRequestPopup { selected_index: usize },
    HelpPopup,
    SigningInitiatePopup { 
        chain_index: usize,
        transaction_input: String,
        input_mode: bool,
    },
    SessionProposalPopup {
        session_name: String,
        total_participants: String,
        threshold: String,
        participants: String,
        selected_field: usize,
    },
    WalletListPopup { selected_index: usize },
    AcceptSessionPopup { 
        selected_index: usize,
        sessions: Vec<SessionInfo>,
    },
    // New Simplified Wallet Creation Flow
    WelcomeScreen,
    PathSelection { selected_index: usize },
    ModeSelection { selected_index: usize },
    CurveSelection { selected_index: usize },
    TemplateSelection { 
        selected_index: usize,
        auto_generated_name: String,
    },
    // Legacy configuration (now only for custom template)
    WalletConfiguration {
        selected_field: usize,
        wallet_name: String,
        description: String,
        total: String,
        threshold: String,
        timeout_hours: String,
        auto_discovery: bool,
        blockchain_configs: Vec<(String, bool)>,
        selected_blockchain: usize,
    },
    SessionDiscovery {
        selected_index: usize,
        filter_text: String,
        input_mode: bool,
    },
    DkgProgress {
        allow_cancel: bool,
    },
    WalletComplete {
        selected_action: usize,
        show_address_details: bool,
    },
}


pub fn draw_main_ui<B: Backend, C: Ciphersuite>(
    terminal: &mut Terminal<B>,
    app: &AppState<C>,
    input: &str,
    input_mode: bool,
    ui_mode: &UIMode,
) -> io::Result<()> {
    terminal.draw(|f| {
        // Check if we're in a full-screen mode that should hide regular panels
        let show_clean_screen = matches!(ui_mode, 
            UIMode::MainMenu { .. } | 
            UIMode::WelcomeScreen | 
            UIMode::PathSelection { .. } |
            UIMode::ModeSelection { .. } |
            UIMode::CurveSelection { .. } |
            UIMode::TemplateSelection { .. } |
            UIMode::WalletConfiguration { .. } |
            UIMode::SessionDiscovery { .. } |
            UIMode::DkgProgress { .. } |
            UIMode::WalletComplete { .. }
        );
        
        if !show_clean_screen {
            // Regular UI - show all panels (devices, log, status, input)
            // Main layout: Title, Devices, Log, Status, Input
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3), // Title area
                    Constraint::Length(5), // Devices area
                    Constraint::Min(5),    // Log area (flexible height)
                    Constraint::Length(8), // Status area (increased height for wrapping)
                    Constraint::Length(3), // Input area
                ])
                .split(f.area());

        let title_text = if app.offline_mode {
            format!(" Device ID: {} [🔒 OFFLINE MODE] ", app.device_id)
        } else {
            format!(" Device ID: {} ", app.device_id)
        };
        
        let title_block = Block::default()
            .title(title_text)
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded); // Use rounded borders
        f.render_widget(title_block, main_chunks[0]);

        let session_participants: HashSet<String> = app
            .session
            .as_ref()
            .map(|s| s.participants.iter().cloned().collect())
            .unwrap_or_default();

        let device_list_items = app
            .devices
            .iter()
            .filter(|p| !p.trim().eq_ignore_ascii_case(app.device_id.trim()))
            .map(|p| {
                let status_str = if session_participants.contains(p) {
                    // First check if there's an explicit status
                    if let Some(s) = app.device_statuses.get(p) {
                        // For clarity, add connection role in the status display
                        let role_prefix = if app.device_id < *p { "→" } else { "←" }; // Simplified comparison
                        format!("{}{:?}", role_prefix, s)
                    } else {
                        // Default for session members not yet reported
                        "Pending".to_string()
                    }
                } else {
                    // If not in session, they shouldn't be connected via WebRTC
                    "N/A".to_string()
                };
                // Add color based on status
                let style = match app.device_statuses.get(p) {
                    Some(webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected) => Style::default().fg(Color::Green),
                    Some(webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connecting) => Style::default().fg(Color::Yellow),
                    Some(webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Failed) |
                    Some(webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Disconnected) |
                    Some(webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Closed) => Style::default().fg(Color::Red),
                    _ => Style::default(),
                };

                ListItem::new(format!("{} ({})", p, status_str)).style(style)
            })
            .collect::<Vec<_>>();

        let devices_widget =
            List::new(device_list_items) // Use the formatted list
                .block(Block::default().title(" Devices (Signaling) ").borders(Borders::ALL));
        f.render_widget(devices_widget, main_chunks[1]);

        let log_text: Vec<Line> = app.log.iter().map(|l| Line::from(l.clone())).collect();
        let log_widget = Paragraph::new(log_text)
            .block(Block::default().title(" Log (Scroll: Up/Down) ").borders(Borders::ALL))
            .wrap(Wrap { trim: false }) // Enable wrapping
            .scroll((app.log_scroll.try_into().unwrap_or(0), 0)); // Apply vertical scroll offset
        f.render_widget(log_widget, main_chunks[2]);

        // --- Status Widget ---
        draw_status_section(f, app, main_chunks[3]);

        // --- Input Widget ---
        let input_title = if input_mode { " Input (Esc to cancel) " } else { " Help " };
        let input_display_text = if input_mode {
            format!("> {}", input)
        } else {
            // Help text for commands
            "Menu: m | Propose: p | Wallets: w | Accept: a | Sign: t | Help: ? | Quit: q".to_string()
        };
        let input_box = Paragraph::new(input_display_text)
            .style(if input_mode { Style::default().fg(Color::Yellow) } else { Style::default() })
            .block(Block::default().title(input_title).borders(Borders::ALL));
        f.render_widget(input_box, main_chunks[4]);

            // --- Cursor for Input Mode ---
            if input_mode {
                // Calculate cursor position based on input text length
                // Add 1 for block border, 2 for the "> " prefix
                let cursor_x = main_chunks[4].x + input.chars().count() as u16 + 3;
                let cursor_y = main_chunks[4].y + 1; // Inside the input box border
                let position = Rect::new(cursor_x, cursor_y, 1, 1);
                f.set_cursor_position(position);
            }
        } // End of regular UI block
        
        // Draw popup if needed - these appear on top of or instead of regular UI
        match ui_mode {
            // Legacy modes
            UIMode::MainMenu { selected_index } => {
                draw_main_menu::<B, C>(f, app, *selected_index);
            }
            UIMode::SigningRequestPopup { selected_index } => {
                draw_signing_popup::<B, C>(f, app, *selected_index);
            }
            UIMode::HelpPopup => {
                draw_help_popup::<B>(f);
            }
            UIMode::SigningInitiatePopup { chain_index, transaction_input, input_mode } => {
                draw_signing_initiate_popup::<B, C>(f, app, *chain_index, transaction_input, *input_mode);
            }
            UIMode::SessionProposalPopup { .. } => {
                draw_session_proposal_popup::<B, C>(f, app, ui_mode);
            }
            UIMode::WalletListPopup { selected_index } => {
                draw_wallet_list_popup::<B, C>(f, app, *selected_index);
            }
            UIMode::AcceptSessionPopup { selected_index, sessions } => {
                draw_accept_session_popup::<B>(f, sessions, *selected_index);
            }
            // New Wallet Creation Flow
            UIMode::WelcomeScreen => {
                draw_welcome_screen::<B, C>(f, app);
            }
            UIMode::PathSelection { selected_index } => {
                draw_path_selection::<B, C>(f, app, *selected_index);
            }
            UIMode::ModeSelection { selected_index } => {
                draw_mode_selection::<B, C>(f, app, *selected_index);
            }
            UIMode::CurveSelection { selected_index } => {
                draw_curve_selection::<B, C>(f, app, *selected_index);
            }
            UIMode::TemplateSelection { selected_index, auto_generated_name } => {
                draw_template_selection::<B, C>(f, app, *selected_index, auto_generated_name);
            }
            UIMode::WalletConfiguration { .. } => {
                draw_wallet_configuration::<B, C>(f, app, ui_mode);
            }
            UIMode::SessionDiscovery { selected_index, filter_text, .. } => {
                draw_session_discovery::<B, C>(f, app, *selected_index, filter_text);
            }
            UIMode::DkgProgress { .. } => {
                draw_dkg_progress::<B, C>(f, app);
            }
            UIMode::WalletComplete { selected_action, show_address_details } => {
                draw_wallet_complete::<B, C>(f, app, *selected_action, *show_address_details);
            }
            UIMode::Normal => {}
        }
    })?;
    Ok(())
}

fn draw_status_section<T: frost_core::Ciphersuite>(
    f: &mut Frame<'_>,
    app: &AppState<T>,
    area: Rect,
) {
    let mut status_items = Vec::new();

    // show curve 
    let c_type_id = TypeId::of::<T>();

    let curve_name = if c_type_id == TypeId::of::<frost_secp256k1::Secp256K1Sha256>() {
        "secp256k1"
    } else if c_type_id == TypeId::of::<frost_ed25519::Ed25519Sha512>() {
        "ed25519"
    } else {
        "unknown"
    };
    status_items.push(Line::from(vec![
        Span::styled("Curve: ", Style::default().fg(Color::Yellow)),
        Span::raw(curve_name),
    ]));

    // Session display - show active session only
    if let Some(session) = &app.session {
        let session_type_str = match &session.session_type {
            crate::protocal::signal::SessionType::DKG => "DKG".to_string(),
            crate::protocal::signal::SessionType::Signing { wallet_name, .. } => format!("Sign[{}]", wallet_name),
        };
        
        status_items.push(Line::from(vec![
            Span::styled("Session: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!(
                "{} {} ({} of {}, threshold {})",
                session.session_id,
                session_type_str,
                session.participants.len(),
                session.total,
                session.threshold
            )),
        ]));
        
        // Add mesh status information as documented in cli_usage.md
        let mesh_status_str = match &app.mesh_status {
            MeshStatus::Incomplete => "Incomplete".to_string(),
            MeshStatus::WebRTCInitiated => "Connecting...".to_string(),
            MeshStatus::PartiallyReady { ready_devices, total_devices } => format!("Partially Ready ({}/{})", ready_devices.len(), total_devices),
            MeshStatus::Ready => "Ready".to_string(),
        };
        
        let mesh_style = match &app.mesh_status {
            MeshStatus::Incomplete => Style::default().fg(Color::Red),
            MeshStatus::WebRTCInitiated => Style::default().fg(Color::Yellow),
            MeshStatus::PartiallyReady { .. } => Style::default().fg(Color::Yellow),
            MeshStatus::Ready => Style::default().fg(Color::Green),
        };
        
        status_items.push(Line::from(vec![
            Span::styled("Mesh Status: ", Style::default().fg(Color::Yellow)),
            Span::styled(mesh_status_str, mesh_style),
        ]));
    } else {
        status_items.push(Line::from(vec![
            Span::styled("Session: ", Style::default().fg(Color::Yellow)),
            Span::raw("None"),
        ]));
        
        status_items.push(Line::from(vec![
            Span::styled("Mesh Status: ", Style::default().fg(Color::Yellow)),
            Span::raw("N/A"),
        ]));
    }

    // Invites display - only show invites that aren't the active session
    let pending_invites: Vec<&SessionInfo> = app
        .invites
        .iter()
        .filter(|invite| {
            app.session
                .as_ref()
                .map(|s| s.session_id != invite.session_id)
                .unwrap_or(true)
        })
        .collect();

    if pending_invites.is_empty() {
        status_items.push(Line::from(vec![
            Span::styled("Invites: ", Style::default().fg(Color::Yellow)),
            Span::raw("None"),
        ]));
    } else {
        status_items.push(Line::from(vec![
            Span::styled("Invites: ", Style::default().fg(Color::Yellow)),
            Span::raw(
                pending_invites
                    .iter()
                    .map(|i| i.session_id.clone())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        ]));
    }

    let dkg_style = if app.dkg_state.is_active() {
        Style::default().fg(Color::Green)
    } else if app.dkg_state.is_completed() {
        Style::default().fg(Color::Blue)
    } else if matches!(app.dkg_state, crate::DkgState::Failed(_)) { // Fix: Use tuple variant pattern
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Gray)
    };

    status_items.push(Line::from(vec![
        Span::styled("DKG Status: ", Style::default().fg(Color::Yellow)),
        Span::styled(app.dkg_state.display_status(), dkg_style),
    ]));

    // Add signing status display
    let signing_style = if app.signing_state.is_active() {
        Style::default().fg(Color::Green)
    } else if matches!(app.signing_state, crate::utils::state::SigningState::Complete { .. }) {
        Style::default().fg(Color::Blue)
    } else if matches!(app.signing_state, crate::utils::state::SigningState::Failed { .. }) {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Gray)
    };

    status_items.push(Line::from(vec![
        Span::styled("Signing Status: ", Style::default().fg(Color::Yellow)),
        Span::styled(app.signing_state.display_status(), signing_style),
    ]));

    // Group Public Key display
    if let Some(group_public_key) = &app.group_public_key {
        let group_key_json = serde_json::to_string(group_public_key).unwrap_or_else(|_| "Error".to_string());
        
        // Extract just the verifying_key (main public key) from the JSON if possible
        let display_key = if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&group_key_json) {
            if let Some(verifying_key) = parsed.get("verifying_key").and_then(|v| v.as_str()) {
                verifying_key.to_string()
            } else {
                // Fallback: show first 16 chars of full JSON
                if group_key_json.len() > 32 {
                    format!("{}...", &group_key_json[..32])
                } else {
                    group_key_json.clone()
                }
            }
        } else {
            "Error parsing key".to_string()
        };
        
        status_items.push(Line::from(vec![
            Span::styled("Group Public Key: ", Style::default().fg(Color::Yellow)),
            Span::styled(display_key, Style::default().fg(Color::Green)),
        ]));
    } else {
        status_items.push(Line::from(vec![
            Span::styled("Group Public Key: ", Style::default().fg(Color::Yellow)),
            Span::raw("N/A"),
        ]));
    }

    // Blockchain Address display - multi-chain support
    if !app.blockchain_addresses.is_empty() {
        status_items.push(Line::from(vec![
            Span::styled("Blockchain Addresses:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));
        
        // Show enabled blockchain addresses
        for blockchain_info in app.blockchain_addresses.iter().filter(|b| b.enabled) {
            status_items.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{}: ", blockchain_info.blockchain), Style::default().fg(Color::Cyan)),
                Span::styled(&blockchain_info.address, Style::default().fg(Color::Green)),
            ]));
        }
    } else if curve_name == "secp256k1" && app.etherum_public_key.is_some() {
        // Legacy display for backward compatibility
        status_items.push(Line::from(vec![
            Span::styled("Ethereum Address: ", Style::default().fg(Color::Yellow)),
            Span::styled(app.etherum_public_key.clone().unwrap(), Style::default().fg(Color::Green)),
        ]));
    } else if curve_name == "ed25519" && app.solana_public_key.is_some() {
        // Legacy display for backward compatibility
        status_items.push(Line::from(vec![
            Span::styled("Solana Address: ", Style::default().fg(Color::Yellow)),
            Span::styled(app.solana_public_key.clone().unwrap(), Style::default().fg(Color::Green)),
        ]));
    } else {
        status_items.push(Line::from(vec![
            Span::styled("Blockchain Address: ", Style::default().fg(Color::Yellow)),
            Span::raw("N/A"),
        ]));        
    }

    // Display additional connection info if in a session
    if let Some(session) = &app.session {
        let connected_devices = app.device_statuses
            .iter()
            .filter(|&(_, &status)| 
                matches!(status, webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected)
            )
            .count();
            
        let total_devices = session.participants.len() - 1; // Exclude self
        
        let connection_status = format!("{}/{} devices connected", connected_devices, total_devices);
        let connection_style = if connected_devices == total_devices {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Yellow)
        };
        
        status_items.push(Line::from(vec![
            Span::styled("WebRTC Connections: ", Style::default().fg(Color::Yellow)),
            Span::styled(connection_status, connection_style),
        ]));
    }

    let status_block = Block::default().title(" Status ").borders(Borders::ALL);
    let status_text = Paragraph::new(status_items)
        .block(status_block)
        .wrap(Wrap { trim: true });
    f.render_widget(status_text, area);
}

// Returns Ok(true) to continue, Ok(false) to quit, Err on error.
fn draw_help_popup<B: Backend>(f: &mut Frame) {
    // Calculate popup size - responsive to screen size
    let area = f.area();
    let popup_width = std::cmp::min(80, area.width.saturating_sub(4));
    let popup_height = std::cmp::min(28, area.height.saturating_sub(4));
    
    let popup_area = Rect::new(
        (area.width.saturating_sub(popup_width)) / 2,
        (area.height.saturating_sub(popup_height)) / 2,
        popup_width,
        popup_height,
    );
    
    // Clear background
    f.render_widget(Clear, popup_area);
    
    // Create help content
    let help_content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("FROST MPC CLI Node - Complete Help", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Quick Keys (Normal Mode):", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        ]),
        Line::from("  m/M         Open main menu"),
        Line::from("  p/P         Create/propose new session"),
        Line::from("  w/W         View wallet list"),
        Line::from("  a/A         Accept session invitations"),
        Line::from("  t/T         Sign transaction"),
        Line::from("  Tab         View pending signing requests"),
        Line::from("  ↑/↓         Scroll log up/down"),
        Line::from("  o           Accept first pending session"),
        Line::from("  s           Save log to file"),
        Line::from("  ?           Show/hide this help"),
        Line::from("  q           Quit application"),
        Line::from("  i           Enter command mode (legacy)"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Basic Commands (press 'i' first):", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        ]),
        Line::from("  /list                       List connected devices"),
        Line::from("  /wallets                    Show your wallets"),
        Line::from("  /propose <name> <total> <threshold> <devices>"),
        Line::from("    • Auto-detects: DKG if new, signing if exists"),
        Line::from("  /accept <session-id>        Accept session proposal"),
        Line::from("  /sign                                 Open signing interface"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Wallet Sharing (Chrome Extension):", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        ]),
        Line::from("  /locate_wallet <wallet>     Show wallet file location"),
        Line::from(""),
        Line::from("  📂 Wallet files location:"),
        Line::from("     ~/.frost_keystore/<device>/<curve>/<wallet>.json"),
        Line::from(""),
        Line::from("  🚀 To share with Chrome extension:"),
        Line::from("     1. Copy the JSON file to teammates"),
        Line::from("     2. Import in Chrome extension"),
        Line::from("     3. Use same password (device ID)"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Offline Mode (Coming Soon):", Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD))
        ]),
        Line::from("  • Air-gapped signing temporarily disabled"),
        Line::from("  • Focus on browser compatibility first"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tips:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        ]),
        Line::from("  • Tab key shows pending signing requests"),
        Line::from("  • Wallets created via DKG are automatically saved"),
        Line::from("  • File location shown after wallet creation"),
        Line::from("  • Same wallet files work in CLI and Chrome"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Examples:", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        ]),
        Line::from("  /propose company-keys 3 2 alice,bob,charlie"),
        Line::from("  /locate_wallet wallet_2of3"),
        Line::from("  Tab: View signing requests"),
        Line::from("  /sign: Open signing interface"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press Esc to close this help", Style::default().fg(Color::Gray))
        ]),
    ];
    
    // Create popup block
    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    
    let paragraph = Paragraph::new(help_content)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });
    
    f.render_widget(paragraph, popup_area);
}

fn draw_signing_initiate_popup<B: Backend, C: Ciphersuite>(
    f: &mut Frame, 
    _app: &AppState<C>, 
    chain_index: usize,
    transaction_input: &str,
    input_mode: bool,
) {
    // List of supported chains
    let chains = vec![
        ("Ethereum", "ethereum", Some(1)),
        ("BSC", "bsc", Some(56)),
        ("Polygon", "polygon", Some(137)),
        ("Arbitrum", "arbitrum", Some(42161)),
        ("Optimism", "optimism", Some(10)),
        ("Avalanche", "avalanche", Some(43114)),
        ("Solana", "solana", None),
        ("Bitcoin", "bitcoin", None),
        ("Bitcoin Testnet", "bitcoin-testnet", None),
    ];
    
    // Calculate popup size
    let popup_width = 80;
    let popup_height = 20;
    
    let area = f.area();
    let popup_area = Rect::new(
        (area.width.saturating_sub(popup_width)) / 2,
        (area.height.saturating_sub(popup_height)) / 2,
        popup_width,
        popup_height,
    );
    
    // Clear background
    f.render_widget(Clear, popup_area);
    
    // Create popup block
    let block = Block::default()
        .title(" Initiate Signing ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);
    
    // Create layout within popup
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(chains.len() as u16 + 2), // Chain selection
            Constraint::Length(3), // Transaction input label
            Constraint::Min(4), // Transaction input area
            Constraint::Length(2), // Instructions
        ])
        .split(inner_area);
    
    // Draw chain selection
    let chain_items: Vec<ListItem> = chains
        .iter()
        .enumerate()
        .map(|(idx, (name, _id, chain_id))| {
            let is_selected = idx == chain_index;
            let style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            let chain_text = if let Some(id) = chain_id {
                format!("{} (Chain ID: {})", name, id)
            } else {
                name.to_string()
            };
            
            ListItem::new(chain_text).style(style)
        })
        .collect();
    
    let chain_list = List::new(chain_items)
        .block(Block::default().title("Select Chain").borders(Borders::ALL))
        .style(Style::default());
    
    f.render_widget(chain_list, chunks[0]);
    
    // Draw transaction input label
    let tx_label = Paragraph::new("Transaction Data (hex):")
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(tx_label, chunks[1]);
    
    // Draw transaction input area
    let tx_input_style = if input_mode {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    
    let tx_input = Paragraph::new(transaction_input)
        .block(Block::default().borders(Borders::ALL))
        .style(tx_input_style)
        .wrap(Wrap { trim: false });
    
    f.render_widget(tx_input, chunks[2]);
    
    // Show cursor if in input mode
    if input_mode {
        let cursor_x = chunks[2].x + 1 + transaction_input.len() as u16;
        let cursor_y = chunks[2].y + 1;
        f.set_cursor_position(Rect::new(cursor_x, cursor_y, 1, 1));
    }
    
    // Instructions
    let instructions = if input_mode {
        "Type transaction data | Enter: Submit | Esc: Exit input mode"
    } else {
        "↑/↓: Select chain | Tab: Edit transaction | Enter: Sign | Esc: Cancel"
    };
    
    let instructions_widget = Paragraph::new(instructions)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    
    f.render_widget(instructions_widget, chunks[3]);
}

fn draw_signing_popup<B: Backend, C: Ciphersuite>(f: &mut Frame, app: &AppState<C>, selected_index: usize) {
    let pending_requests = &app.pending_signing_requests;
    if pending_requests.is_empty() {
        return;
    }
    
    // Calculate popup size
    let popup_width = 80;
    let popup_height = std::cmp::min(20, pending_requests.len() as u16 + 6);
    
    let area = f.area();
    let popup_area = Rect::new(
        (area.width.saturating_sub(popup_width)) / 2,
        (area.height.saturating_sub(popup_height)) / 2,
        popup_width,
        popup_height,
    );
    
    // Clear background
    f.render_widget(Clear, popup_area);
    
    // Create popup block
    let block = Block::default()
        .title(" Pending Signing Requests ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    
    // Create list items
    let items: Vec<ListItem> = pending_requests
        .iter()
        .enumerate()
        .map(|(idx, req)| {
            let is_selected = idx == selected_index;
            let style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            let content = vec![
                Line::from(vec![
                    Span::styled("ID: ", Style::default().fg(Color::Cyan)),
                    Span::raw(&req.signing_id),
                ]),
                Line::from(vec![
                    Span::styled("From: ", Style::default().fg(Color::Cyan)),
                    Span::raw(&req.from_device),
                ]),
                Line::from(vec![
                    Span::styled("Data: ", Style::default().fg(Color::Cyan)),
                    Span::raw(if req.transaction_data.len() > 60 {
                        format!("{}...", &req.transaction_data[..60])
                    } else {
                        req.transaction_data.clone()
                    }),
                ]),
                Line::from(""), // Empty line for spacing
            ];
            
            ListItem::new(content).style(style)
        })
        .collect();
    
    let list = List::new(items)
        .block(block)
        .style(Style::default());
    
    f.render_widget(list, popup_area);
    
    // Instructions at the bottom
    let instructions = Paragraph::new("↑/↓: Navigate | Enter: Accept | Esc: Cancel")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    
    let instruction_area = Rect::new(
        popup_area.x,
        popup_area.y + popup_area.height - 2,
        popup_area.width,
        1,
    );
    
    f.render_widget(instructions, instruction_area);
}

fn draw_main_menu<B: Backend, C: Ciphersuite>(f: &mut Frame, _app: &AppState<C>, selected_index: usize) {
    // Calculate popup size
    let popup_width = 60;
    let popup_height = 12;
    
    let area = f.area();
    let popup_area = Rect::new(
        (area.width.saturating_sub(popup_width)) / 2,
        (area.height.saturating_sub(popup_height)) / 2,
        popup_width,
        popup_height,
    );
    
    // Clear background
    f.render_widget(Clear, popup_area);
    
    // Menu items
    let menu_items = vec![
        ("Create Wallet", "Create a new MPC wallet using DKG"),
        ("Create/Join Session", "Create a new session or join existing"),
        ("List Wallets", "View all available wallets"),
        ("Sign Transaction", "Initiate transaction signing"),
        ("Accept Session", "Accept pending session invitations"),
        ("View Help", "Show help and commands"),
    ];
    
    // Create list items
    let items: Vec<ListItem> = menu_items
        .iter()
        .enumerate()
        .map(|(idx, (title, desc))| {
            let is_selected = idx == selected_index;
            let style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            let content = vec![
                Line::from(vec![
                    Span::styled(format!("  {}", title), style),
                ]),
                Line::from(vec![
                    Span::styled(format!("    {}", desc), Style::default().fg(Color::Gray)),
                ]),
            ];
            
            ListItem::new(content)
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default()
            .title(" Main Menu ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .style(Style::default());
    
    f.render_widget(list, popup_area);
    
    // Instructions
    let instructions = Paragraph::new("↑/↓: Navigate | Enter: Select | Esc: Close")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    
    let instruction_area = Rect::new(
        popup_area.x,
        popup_area.y + popup_area.height - 2,
        popup_area.width,
        1,
    );
    
    f.render_widget(instructions, instruction_area);
}

fn draw_session_proposal_popup<B: Backend, C: Ciphersuite>(f: &mut Frame, _app: &AppState<C>, ui_mode: &UIMode) {
    if let UIMode::SessionProposalPopup { 
        session_name, 
        total_participants, 
        threshold,
        participants,
        selected_field 
    } = ui_mode {
        // Calculate popup size
        let popup_width = 70;
        let popup_height = 15;
        
        let area = f.area();
        let popup_area = Rect::new(
            (area.width.saturating_sub(popup_width)) / 2,
            (area.height.saturating_sub(popup_height)) / 2,
            popup_width,
            popup_height,
        );
        
        // Clear background
        f.render_widget(Clear, popup_area);
        
        // Create form fields
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(2), // Session name
                Constraint::Length(2), // Total participants
                Constraint::Length(2), // Threshold
                Constraint::Length(2), // Participants list
                Constraint::Length(1), // Instructions
            ])
            .split(popup_area);
        
        // Title
        let title = Paragraph::new("Create Session Proposal")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);
        
        // Session name field
        let name_style = if *selected_field == 0 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let name_field = Paragraph::new(format!("Session Name: {}", session_name))
            .style(name_style)
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(name_field, chunks[1]);
        
        // Total participants field
        let total_style = if *selected_field == 1 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let total_field = Paragraph::new(format!("Total Participants: {}", total_participants))
            .style(total_style)
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(total_field, chunks[2]);
        
        // Threshold field
        let threshold_style = if *selected_field == 2 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let threshold_field = Paragraph::new(format!("Threshold: {}", threshold))
            .style(threshold_style)
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(threshold_field, chunks[3]);
        
        // Participants field
        let participants_style = if *selected_field == 3 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let participants_field = Paragraph::new(format!("Participants (comma-separated): {}", participants))
            .style(participants_style)
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(participants_field, chunks[4]);
        
        // Instructions
        let instructions = Paragraph::new("Tab: Next field | Enter: Submit | Esc: Cancel")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(instructions, chunks[5]);
    }
}

fn draw_wallet_list_popup<B: Backend, C: Ciphersuite>(f: &mut Frame, _app: &AppState<C>, _selected_index: usize) {
    // This would show the list of wallets
    // For now, just a placeholder
    let popup_width = 70;
    let popup_height = 20;
    
    let area = f.area();
    let popup_area = Rect::new(
        (area.width.saturating_sub(popup_width)) / 2,
        (area.height.saturating_sub(popup_height)) / 2,
        popup_width,
        popup_height,
    );
    
    // Clear background
    f.render_widget(Clear, popup_area);
    
    let block = Block::default()
        .title(" Wallet List ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    
    let text = vec![
        Line::from("Available wallets will be listed here"),
        Line::from(""),
        Line::from("Press Esc to close"),
    ];
    
    let paragraph = Paragraph::new(text)
        .block(block)
        .style(Style::default());
    
    f.render_widget(paragraph, popup_area);
}

fn draw_accept_session_popup<B: Backend>(f: &mut Frame, sessions: &[SessionInfo], selected_index: usize) {
    if sessions.is_empty() {
        return;
    }
    
    // Calculate popup size
    let popup_width = 80;
    let popup_height = std::cmp::min(20, sessions.len() as u16 * 4 + 6);
    
    let area = f.area();
    let popup_area = Rect::new(
        (area.width.saturating_sub(popup_width)) / 2,
        (area.height.saturating_sub(popup_height)) / 2,
        popup_width,
        popup_height,
    );
    
    // Clear background
    f.render_widget(Clear, popup_area);
    
    // Create list items
    let items: Vec<ListItem> = sessions
        .iter()
        .enumerate()
        .map(|(idx, session)| {
            let is_selected = idx == selected_index;
            let style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            let session_type_str = match &session.session_type {
                crate::protocal::signal::SessionType::DKG => "DKG".to_string(),
                crate::protocal::signal::SessionType::Signing { wallet_name, .. } => {
                    format!("Signing [{}]", wallet_name)
                }
            };
            
            let content = vec![
                Line::from(vec![
                    Span::styled("ID: ", Style::default().fg(Color::Cyan)),
                    Span::raw(&session.session_id),
                    Span::raw(" | "),
                    Span::styled("Type: ", Style::default().fg(Color::Cyan)),
                    Span::raw(session_type_str),
                ]),
                Line::from(vec![
                    Span::styled("Initiator: ", Style::default().fg(Color::Cyan)),
                    Span::raw(&session.proposer_id),
                    Span::raw(" | "),
                    Span::styled("Participants: ", Style::default().fg(Color::Cyan)),
                    Span::raw(format!("{}/{}", session.participants.len(), session.total)),
                ]),
                Line::from(""), // Empty line for spacing
            ];
            
            ListItem::new(content).style(style)
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default()
            .title(" Session Invitations ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)))
        .style(Style::default());
    
    f.render_widget(list, popup_area);
    
    // Instructions
    let instructions = Paragraph::new("↑/↓: Navigate | Enter: Accept | Esc: Cancel")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    
    let instruction_area = Rect::new(
        popup_area.x,
        popup_area.y + popup_area.height - 2,
        popup_area.width,
        1,
    );
    
    f.render_widget(instructions, instruction_area);
}

// ========================================
// NEW WALLET CREATION FLOW UI FUNCTIONS
// ========================================

/// Clean welcome screen with professional BitGo-like design
fn draw_welcome_screen<B: Backend, C: Ciphersuite>(f: &mut Frame, _app: &AppState<C>) {
    let area = f.area();
    
    // Clear entire screen
    f.render_widget(Clear, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([
            Constraint::Length(6),  // Header
            Constraint::Min(8),     // Content
            Constraint::Length(4),  // Footer
        ])
        .split(area);

    // Header with branding
    let header_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("🔑 MPC WALLET", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Enterprise Multi-Party Computation Wallet", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
    ];
    
    let header = Paragraph::new(header_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)));
    f.render_widget(header, chunks[0]);

    // Main welcome content
    let welcome_content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Welcome to MPC Wallet", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("Create secure threshold wallets using distributed key generation."),
        Line::from("No single party ever holds the complete private key."),
        Line::from(""),
        Line::from(vec![
            Span::styled("Key Features:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  • Threshold signatures (m-of-n security)"),
        Line::from("  • Multi-blockchain support (Ethereum, Bitcoin, Solana)"),
        Line::from("  • Air-gapped and online coordination modes"),
        Line::from("  • Enterprise-grade security and compliance"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Getting Started:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("Press Enter to continue to the main menu"),
        Line::from(""),
    ];

    let welcome = Paragraph::new(welcome_content)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Green)));
    f.render_widget(welcome, chunks[1]);

    // Footer instructions
    let footer = Paragraph::new("Enter: Continue  |  q: Quit  |  ?: Help")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Gray)));
    f.render_widget(footer, chunks[2]);
}

/// Path selection screen (Create Wallet, Join Session, Select Wallet)
fn draw_path_selection<B: Backend, C: Ciphersuite>(
    f: &mut Frame, 
    _app: &AppState<C>, 
    selected_index: usize
) {
    let area = f.area();
    f.render_widget(Clear, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(3)
        .constraints([
            Constraint::Length(4),  // Title
            Constraint::Min(12),    // Options
            Constraint::Length(3),  // Instructions
        ])
        .split(area);

    // Title
    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("🚀 What would you like to do?", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)));
    f.render_widget(title, chunks[0]);

    // Path options
    let options = vec![
        ("🆕 Create New Wallet", "Start fresh wallet creation with DKG", "Generate threshold keys for a new wallet"),
        ("🔗 Join Wallet Session", "Join an existing wallet creation", "Participate in someone else's wallet DKG"),
        ("💼 Select Existing Wallet", "Use a previously created wallet", "Access wallets from your keystore"),
    ];

    let items: Vec<ListItem> = options.iter().enumerate().map(|(i, (title, subtitle, description))| {
        let style = if i == selected_index {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };

        ListItem::new(vec![
            Line::from(vec![
                Span::styled(format!("[{}] {}", i + 1, title), style.add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(format!("    {}", subtitle), style.fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled(format!("    💡 {}", description), style.fg(Color::Gray)),
            ]),
            Line::from(""),
        ])
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Choose Your Path"))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));
    f.render_widget(list, chunks[1]);

    // Instructions
    let instructions = Paragraph::new("↑↓: Navigate  Enter: Select  Esc: Back  q: Quit")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, chunks[2]);
}

/// Mode selection screen (Online/Offline/Hybrid)
fn draw_mode_selection<B: Backend, C: Ciphersuite>(
    f: &mut Frame,
    _app: &AppState<C>,
    selected_index: usize,
) {
    let area = f.area();
    f.render_widget(Clear, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(3)
        .constraints([
            Constraint::Length(5),  // Title
            Constraint::Min(15),    // Mode options
            Constraint::Length(3),  // Instructions
        ])
        .split(area);

    // Title with warning
    let title_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("🌐 Select Coordination Mode", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Choose how participants will coordinate during wallet creation", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
    ];
    
    let title = Paragraph::new(title_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Green)));
    f.render_widget(title, chunks[0]);

    // Mode options with detailed descriptions
    let modes = vec![
        (
            "🌐 Online Mode", 
            "Real-time coordination via WebRTC mesh",
            "• Fastest wallet creation (2-5 minutes)\n• All participants must be online simultaneously\n• Direct peer-to-peer communication\n• Best for teams with good connectivity",
            Color::Green
        ),
        (
            "🔒 Offline Mode", 
            "Air-gapped coordination via file exchange",
            "• Maximum security for high-value wallets\n• Participants exchange files via SD card/USB\n• No internet required during key generation\n• Takes longer but most secure (hours/days)",
            Color::Red
        ),
        (
            "🔀 Hybrid Mode", 
            "Online coordination, offline key generation",
            "• Balance of convenience and security\n• Online discovery and coordination\n• Offline cryptographic operations\n• Good compromise for most use cases",
            Color::Yellow
        ),
    ];

    let mode_items: Vec<ListItem> = modes.iter().enumerate().map(|(i, (title, subtitle, details, color))| {
        let style = if i == selected_index {
            Style::default().bg(*color).fg(Color::White)
        } else {
            Style::default()
        };

        let detail_lines: Vec<Line> = details.split('\n').map(|line| {
            Line::from(vec![
                Span::styled(format!("      {}", line), style.fg(Color::Gray)),
            ])
        }).collect();

        let mut lines = vec![
            Line::from(vec![
                Span::styled(format!("[{}] {}", i + 1, title), style.add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(format!("    {}", subtitle), style.fg(Color::Cyan)),
            ]),
        ];
        lines.extend(detail_lines);
        lines.push(Line::from(""));

        ListItem::new(lines)
    }).collect();

    let mode_list = List::new(mode_items)
        .block(Block::default().borders(Borders::ALL).title("Coordination Modes"));
    f.render_widget(mode_list, chunks[1]);

    // Instructions
    let instructions = Paragraph::new("↑↓: Navigate  Enter: Select  Esc: Back")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, chunks[2]);
}

/// Curve selection screen
fn draw_curve_selection<B: Backend, C: Ciphersuite>(
    f: &mut Frame,
    _app: &AppState<C>,
    selected_index: usize,
) {
    let area = f.area();
    f.render_widget(Clear, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(3)
        .constraints([
            Constraint::Length(6),  // Title with warning
            Constraint::Min(10),    // Curve options
            Constraint::Length(3),  // Instructions
        ])
        .split(area);

    // Title with important warning
    let title_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("🔑 Select Cryptographic Curve", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("⚠️  This choice cannot be changed after wallet creation", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Choose the curve that supports your target blockchains", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
    ];
    
    let title = Paragraph::new(title_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)));
    f.render_widget(title, chunks[0]);

    // Curve options
    let curves = vec![
        (
            "secp256k1", 
            "⭐ RECOMMENDED",
            "Ethereum, Bitcoin, BSC, Polygon, Arbitrum, Avalanche",
            "Most widely supported across blockchains and DeFi"
        ),
        (
            "ed25519", 
            "SPECIALIZED",
            "Solana, Near Protocol, Polkadot, Cardano",
            "Faster signatures, more efficient for specific chains"
        ),
    ];

    let curve_items: Vec<ListItem> = curves.iter().enumerate().map(|(i, (curve, badge, chains, note))| {
        let style = if i == selected_index {
            Style::default().bg(Color::Green).fg(Color::White)
        } else {
            Style::default()
        };

        let badge_style = if i == 0 {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        };

        ListItem::new(vec![
            Line::from(vec![
                Span::styled(format!("[{}] {} ", i + 1, curve), style.add_modifier(Modifier::BOLD)),
                Span::styled(*badge, if i == selected_index { style } else { badge_style }),
            ]),
            Line::from(vec![
                Span::styled(format!("    Blockchains: {}", chains), style.fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled(format!("    💡 {}", note), style.fg(Color::Gray)),
            ]),
            Line::from(""),
        ])
    }).collect();

    let curve_list = List::new(curve_items)
        .block(Block::default().borders(Borders::ALL).title("Available Curves"));
    f.render_widget(curve_list, chunks[1]);

    // Instructions
    let instructions = Paragraph::new("↑↓: Navigate  Enter: Select  Esc: Back")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, chunks[2]);
}

/// Template selection screen for simplified wallet creation
fn draw_template_selection<B: Backend, C: Ciphersuite>(
    f: &mut Frame,
    _app: &AppState<C>,
    selected_index: usize,
    auto_generated_name: &str,
) {
    let area = f.area();
    f.render_widget(Clear, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(5),  // Title and wallet name
            Constraint::Min(12),    // Template options
            Constraint::Length(3),  // Instructions
        ])
        .split(area);

    // Title with auto-generated wallet name
    let title_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("🎯 Select Wallet Template", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Wallet Name: ", Style::default().fg(Color::Yellow)),
            Span::styled(auto_generated_name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];
    
    let title = Paragraph::new(title_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Magenta)));
    f.render_widget(title, chunks[0]);

    // Template options
    let template_items: Vec<ListItem> = WALLET_TEMPLATES.iter().enumerate().map(|(i, template)| {
        let style = if i == selected_index {
            Style::default().bg(Color::Magenta).fg(Color::White)
        } else {
            Style::default()
        };
        
        let security_color = match template.security_level {
            "Standard" => Color::Green,
            "High" => Color::Yellow,
            "Maximum" => Color::Red,
            "Enterprise" => Color::Blue,
            "Custom" => Color::Gray,
            _ => Color::White,
        };

        let threshold_display = if template.total == 0 {
            "Custom Values".to_string()
        } else {
            format!("{}-of-{}", template.threshold, template.total)
        };

        ListItem::new(vec![
            Line::from(vec![
                Span::styled(format!("[{}] {} ", i + 1, template.name), style.add_modifier(Modifier::BOLD)),
                Span::styled(format!("({})", threshold_display), if i == selected_index { style } else { Style::default().fg(Color::Cyan) }),
            ]),
            Line::from(vec![
                Span::styled("    📖 ", style),
                Span::styled(template.description, if i == selected_index { style } else { Style::default().fg(Color::Gray) }),
            ]),
            Line::from(vec![
                Span::styled("    🔒 Security: ", style),
                Span::styled(template.security_level, if i == selected_index { style } else { Style::default().fg(security_color) }),
                Span::styled("  🎯 Use Case: ", style),
                Span::styled(template.use_case, if i == selected_index { style } else { Style::default().fg(Color::Yellow) }),
            ]),
            Line::from(""),
        ])
    }).collect();

    let template_list = List::new(template_items)
        .block(Block::default().borders(Borders::ALL).title("🚀 Wallet Templates - No Typing Required!"));
    f.render_widget(template_list, chunks[1]);

    // Instructions
    let instructions = Paragraph::new("↑↓: Navigate  Enter: Create Wallet  Esc: Back  💡 No manual configuration needed!")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, chunks[2]);
}

/// Wallet configuration screen
fn draw_wallet_configuration<B: Backend, C: Ciphersuite>(
    f: &mut Frame,
    _app: &AppState<C>,
    ui_mode: &UIMode,
) {
    if let UIMode::WalletConfiguration { 
        selected_field,
        wallet_name,
        description,
        total,
        threshold,
        timeout_hours,
        auto_discovery,
        blockchain_configs,
        selected_blockchain,
    } = ui_mode {
        let area = f.area();
        f.render_widget(Clear, area);
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(8),  // Basic config
                Constraint::Length(6),  // Threshold config
                Constraint::Length(4),  // Advanced options
                Constraint::Min(6),     // Blockchain selection
                Constraint::Length(3),  // Instructions
            ])
            .split(area);

        // Title
        let title = Paragraph::new("⚙️  Custom Wallet Configuration")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
        f.render_widget(title, chunks[0]);

        // Basic Configuration fields
        let basic_fields = vec![
            format!("Wallet Name: {}", wallet_name),
            format!("Description: {}", if description.is_empty() { "Optional" } else { description }),
            format!("Timeout: {} hours", timeout_hours),
            format!("Auto-Discovery: {}", if *auto_discovery { "Enabled" } else { "Disabled" }),
        ];

        let basic_items: Vec<ListItem> = basic_fields.iter().enumerate().map(|(i, field)| {
            let style = if i == *selected_field {
                Style::default().bg(Color::Cyan).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(field.clone()).style(style)
        }).collect();

        let basic_list = List::new(basic_items)
            .block(Block::default().borders(Borders::ALL).title("Basic Settings"));
        f.render_widget(basic_list, chunks[1]);

        // Threshold Configuration
        let threshold_text = vec![
            Line::from(format!("Total Participants: {} ↕", total)),
            Line::from(format!("Required Signatures: {} ↕", threshold)),
            Line::from(""),
            Line::from(vec![
                Span::styled("Security Scheme: ", Style::default().fg(Color::Yellow)),
                Span::styled(format!("{}-of-{} threshold", threshold, total), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            ]),
        ];

        let threshold_para = Paragraph::new(threshold_text)
            .block(Block::default().borders(Borders::ALL).title("Threshold Configuration"));
        f.render_widget(threshold_para, chunks[2]);

        // Advanced Options (simplified)
        let advanced_text = vec![
            Line::from(vec![
                Span::styled("Coordination Mode: ", Style::default().fg(Color::Yellow)),
                Span::styled("Online WebRTC Mesh", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Security Level: ", Style::default().fg(Color::Yellow)),
                Span::styled("Standard", Style::default().fg(Color::Green)),
            ]),
        ];

        let advanced_para = Paragraph::new(advanced_text)
            .block(Block::default().borders(Borders::ALL).title("Advanced Settings"));
        f.render_widget(advanced_para, chunks[3]);

        // Blockchain Selection
        let blockchain_items: Vec<ListItem> = blockchain_configs.iter().enumerate().map(|(i, (blockchain, enabled))| {
            let status = if *enabled { "✅" } else { "⬜" };
            let style = if i == *selected_blockchain {
                Style::default().bg(Color::Green).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(format!("{} {}", status, blockchain)).style(style)
        }).collect();

        let blockchain_list = List::new(blockchain_items)
            .block(Block::default().borders(Borders::ALL).title("Blockchain Support"));
        f.render_widget(blockchain_list, chunks[4]);

        // Instructions
        let instructions = Paragraph::new("↑↓: Navigate  Enter: Next/Create  Space: Toggle  Tab: Next Section  S: Create  Esc: Back")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(instructions, chunks[5]);
    }
}

/// Session discovery screen for joining existing sessions
fn draw_session_discovery<B: Backend, C: Ciphersuite>(
    f: &mut Frame,
    app: &AppState<C>,
    selected_index: usize,
    filter_text: &str,
) {
    let area = f.area();
    f.render_widget(Clear, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(4),  // Title
            Constraint::Length(3),  // Filter
            Constraint::Length(2),  // Status
            Constraint::Min(10),    // Session list
            Constraint::Length(3),  // Instructions
        ])
        .split(area);

    // Title
    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("🔍 Discover Wallet Sessions", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Magenta)));
    f.render_widget(title, chunks[0]);

    // Filter input
    let filter_para = Paragraph::new(format!("🔎 Filter: {}", filter_text))
        .block(Block::default().borders(Borders::ALL).title("Search"))
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(filter_para, chunks[1]);

    // Status line based on available sessions from state
    let sessions = &app.available_sessions; // This should be populated by the backend
    let status_text = if sessions.is_empty() {
        "🔍 No sessions found - press R to refresh or check your network connection"
    } else {
        &format!("📋 Found {} available wallet creation sessions", sessions.len())
    };
    let status_color = if sessions.is_empty() { Color::Red } else { Color::Green };
    
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(status_color))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[2]);

    // Session list
    if !sessions.is_empty() {
        let items: Vec<ListItem> = sessions.iter().enumerate().map(|(i, session)| {
            let style = if i == selected_index {
                Style::default().bg(Color::Magenta).fg(Color::White)
            } else {
                Style::default()
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!("🔑 {} ", session.wallet_type), style.add_modifier(Modifier::BOLD)),
                    Span::styled("(Active)", Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled(format!("    {} | {}-of-{} | {} participants", 
                        session.curve_type, 
                        session.threshold, 
                        session.total,
                        session.participants_joined,
                    ), style.fg(Color::Gray)),
                ]),
                Line::from(vec![
                    Span::styled(format!("    Created by: {} | Code: {}", 
                        session.creator_device,
                        session.session_code,
                    ), style.fg(Color::Gray)),
                ]),
                Line::from(""),
            ])
        }).collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Available Sessions"));
        f.render_widget(list, chunks[3]);
    } else {
        // Empty state
        let empty_msg = Paragraph::new("🔍 Searching for wallet creation sessions...\n\nMake sure you're connected to the network and try refreshing.")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("No Sessions Found"));
        f.render_widget(empty_msg, chunks[3]);
    }

    // Instructions
    let instructions = Paragraph::new("↑↓: Navigate  Enter: Join  R: Refresh  /: Filter  Esc: Back")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, chunks[4]);
}

/// DKG Progress screen with 6 stages
fn draw_dkg_progress<B: Backend, C: Ciphersuite>(f: &mut Frame, app: &AppState<C>) {
    let area = f.area();
    f.render_widget(Clear, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(4),  // Title
            Constraint::Length(6),  // Progress bar and stage
            Constraint::Length(8),  // Participant status
            Constraint::Min(8),     // Progress log
            Constraint::Length(4),  // Instructions
        ])
        .split(area);

    // Title with wallet name
    let wallet_name = app.wallet_creation_config.as_ref()
        .map(|c| c.wallet_name.as_str())
        .unwrap_or("Wallet");
    let title_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("🔑 Creating Wallet: {}", wallet_name), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];
    let title = Paragraph::new(title_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Green)));
    f.render_widget(title, chunks[0]);

    // Progress bar and stage information
    let progress = app.wallet_creation_progress.as_ref();
    let dkg_stage = &app.dkg_state;
    
    let (progress_ratio, _stage_text, stage_color) = if let Some(prog) = progress {
        let ratio = prog.current_step as f64 / prog.total_steps as f64;
        let color = match dkg_stage {
            crate::utils::state::DkgState::Complete => Color::Green,
            crate::utils::state::DkgState::Failed(_) => Color::Red,
            _ => Color::Yellow,
        };
        (ratio, prog.message.clone(), color)
    } else {
        let (ratio, msg, color) = match dkg_stage {
            crate::utils::state::DkgState::Idle => (0.0, "Ready to start".to_string(), Color::Gray),
            crate::utils::state::DkgState::Round1InProgress => (0.33, "Round 1: Generating commitments".to_string(), Color::Yellow),
            crate::utils::state::DkgState::Round1Complete => (0.5, "Round 1 Complete".to_string(), Color::Yellow),
            crate::utils::state::DkgState::Round2InProgress => (0.66, "Round 2: Distributing key shares".to_string(), Color::Yellow),
            crate::utils::state::DkgState::Round2Complete => (0.8, "Round 2 Complete".to_string(), Color::Yellow),
            crate::utils::state::DkgState::Finalizing => (0.9, "Finalizing wallet".to_string(), Color::Yellow),
            crate::utils::state::DkgState::Complete => (1.0, "✅ Wallet created successfully!".to_string(), Color::Green),
            crate::utils::state::DkgState::Failed(err) => (0.0, format!("❌ Failed: {}", err), Color::Red),
        };
        (ratio, msg, color)
    };

    let progress_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Progress bar
            Constraint::Length(3), // Stage details
        ])
        .split(chunks[1]);

    let progress_bar = ratatui::widgets::Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Overall Progress"))
        .gauge_style(Style::default().fg(stage_color))
        .ratio(progress_ratio)
        .label(format!("{:.0}%", progress_ratio * 100.0));
    f.render_widget(progress_bar, progress_chunks[0]);

    // Stage text shown in progress bar label instead of separate section

    // Participant status
    if let Some(session) = &app.session {
        let participant_items: Vec<ListItem> = session.participants.iter().map(|device_id| {
            let status = if device_id == &app.device_id {
                ("✅", "You (Ready)", Color::Green)
            } else if app.device_statuses.contains_key(device_id) {
                match app.device_statuses[device_id] {
                    webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected => ("✅", "Connected", Color::Green),
                    webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connecting => ("🔄", "Connecting", Color::Yellow),
                    _ => ("⏳", "Pending", Color::Gray),
                }
            } else {
                // Show device ID with waiting status instead of "Unknown"
                ("⏳", "Waiting", Color::Yellow)
            };

            ListItem::new(Line::from(vec![
                Span::raw(format!("{} ", status.0)),
                Span::styled(format!("{}", device_id), Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!(" - {}", status.1), Style::default().fg(status.2)),
            ]))
        }).collect();

        let participant_list = List::new(participant_items)
            .block(Block::default().borders(Borders::ALL).title("Participants"));
        f.render_widget(participant_list, chunks[2]);
    } else {
        let no_session = Paragraph::new("No active session")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Participants"));
        f.render_widget(no_session, chunks[2]);
    }

    // User-friendly stage display instead of technical logs
    let stage_content = match dkg_stage {
        crate::utils::state::DkgState::Idle => {
            vec![
                Line::from(""),
                Line::from(Span::styled("⏳ Waiting to start...", Style::default().fg(Color::Gray))),
                Line::from(""),
                Line::from(Span::styled("💡 Tip: Make sure all participants are ready", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))),
            ]
        }
        crate::utils::state::DkgState::Round1InProgress => {
            vec![
                Line::from(""),
                Line::from(Span::styled("🔗 Stage 1: Connecting to participants...", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(Span::styled("💡 All participants must be online for the process to continue", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))),
                Line::from(Span::styled("💡 This ensures no single party has control of your wallet", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))),
            ]
        }
        crate::utils::state::DkgState::Round1Complete | crate::utils::state::DkgState::Round2InProgress => {
            vec![
                Line::from(""),
                Line::from(Span::styled("🔑 Stage 2: Exchanging secure key shares...", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(Span::styled("💡 Your wallet is being secured with threshold cryptography", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))),
                Line::from(Span::styled("💡 Each participant will hold only a piece of the key", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))),
            ]
        }
        crate::utils::state::DkgState::Round2Complete | crate::utils::state::DkgState::Finalizing => {
            vec![
                Line::from(""),
                Line::from(Span::styled("✨ Stage 3: Finalizing wallet creation...", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(Span::styled("💡 Generating your wallet addresses...", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))),
                Line::from(Span::styled("💡 Almost done! Your secure wallet is being prepared", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))),
            ]
        }
        crate::utils::state::DkgState::Complete => {
            // Show meaningful wallet information
            let wallet_id = app.current_wallet_id.as_deref().unwrap_or("Generated");
            let session_info = app.session.as_ref();
            let threshold_info = session_info
                .map(|s| format!("{}-of-{}", s.threshold, s.total))
                .unwrap_or_else(|| "Unknown".to_string());
            let curve_type = session_info
                .map(|s| s.curve_type.as_str())
                .unwrap_or("Unknown");
            let eth_address = app.etherum_public_key.as_deref()
                .unwrap_or("Generating...");
            
            let mut info_lines = vec![
                Line::from(""),
                Line::from(Span::styled("✅ Wallet Created Successfully!", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(vec![
                    Span::styled("🆔 Wallet ID: ", Style::default().fg(Color::Yellow)),
                    Span::styled(wallet_id, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("🔑 Threshold: ", Style::default().fg(Color::Yellow)),
                    Span::styled(threshold_info, Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("🔗 Curve: ", Style::default().fg(Color::Yellow)),
                    Span::styled(curve_type, Style::default().fg(Color::Cyan)),
                ]),
            ];
            
            // Add session ID for debugging (this determines the group address)
            if let Some(s) = session_info {
                info_lines.push(Line::from(vec![
                    Span::styled("🔍 Session: ", Style::default().fg(Color::Blue)),
                    Span::styled(
                        format!("{}...", &s.session_id[..16.min(s.session_id.len())]),
                        Style::default().fg(Color::Gray)
                    ),
                ]));
            }
            
            // Show group public key if available
            if let Some(verifying_key) = &app.group_public_key {
                // Convert the verifying key to hex string for display
                let key_bytes = verifying_key.serialize().unwrap_or_default();
                let group_key_hex = hex::encode(&key_bytes);
                info_lines.push(Line::from(vec![
                    Span::styled("🔐 Group Key: ", Style::default().fg(Color::Magenta)),
                    Span::styled(
                        format!("{}...", &group_key_hex[..16.min(group_key_hex.len())]),
                        Style::default().fg(Color::Gray)
                    ),
                ]));
            }
            
            info_lines.extend(vec![
                Line::from(vec![
                    Span::styled("📍 Group Address: ", Style::default().fg(Color::Yellow)),
                    Span::styled(eth_address, Style::default().fg(Color::Magenta)),
                ]),
                Line::from(""),
                Line::from(Span::styled("💡 Press Enter or V to view addresses and export options", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))),
            ]);
            
            info_lines
        }
        crate::utils::state::DkgState::Failed(error) => {
            vec![
                Line::from(""),
                Line::from(Span::styled("❌ Wallet creation failed", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(Span::styled(format!("Error: {}", error), Style::default().fg(Color::Red))),
                Line::from(""),
                Line::from(Span::styled("💡 Check that all participants are connected", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))),
                Line::from(Span::styled("💡 Press 'R' to retry or Esc to return", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))),
            ]
        }
    };

    // Show different title based on state
    let section_title = match dkg_stage {
        crate::utils::state::DkgState::Complete => "Wallet Information",
        crate::utils::state::DkgState::Failed(_) => "Error Details",
        _ => "Current Stage",
    };
    
    let stage_para = Paragraph::new(stage_content)
        .block(Block::default().borders(Borders::ALL).title(section_title))
        .alignment(Alignment::Center);
    f.render_widget(stage_para, chunks[3]);

    // Instructions based on current state
    let instructions = match dkg_stage {
        crate::utils::state::DkgState::Complete => {
            "✅ Wallet created successfully! Press Enter or V to view details, D for debug, Esc for menu"
        }
        crate::utils::state::DkgState::Failed(_) => {
            "❌ Wallet creation failed. Press R to retry or Esc to return"
        }
        _ => {
            "🔄 Wallet creation in progress... Press Esc to cancel (not recommended)"
        }
    };

    let instr_para = Paragraph::new(instructions)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Instructions"));
    f.render_widget(instr_para, chunks[4]);
}

/// Wallet completion screen with address display (optimized for performance)
fn draw_wallet_complete<B: Backend, C: Ciphersuite>(
    f: &mut Frame,
    app: &AppState<C>,
    selected_action: usize,
    show_address_details: bool,
) {
    let area = f.area();
    // Only clear if necessary to reduce flickering
    if show_address_details {
        f.render_widget(Clear, area);
    }
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(5),  // Success banner
            Constraint::Length(6),  // Wallet info
            Constraint::Min(8),     // Address details or actions
            Constraint::Length(4),  // Instructions
        ])
        .split(area);

    // Success banner
    let success_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("🎉 Wallet Created Successfully!", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Your threshold wallet is ready to use", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
    ];
    
    let banner = Paragraph::new(success_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Green)));
    f.render_widget(banner, chunks[0]);

    // Wallet information
    let config = app.wallet_creation_config.as_ref();
    let session = app.session.as_ref();
    
    let wallet_info = vec![
        Line::from(vec![
            Span::styled("Wallet Name: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                config.map(|c| c.wallet_name.as_str()).unwrap_or("Unknown"), 
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(vec![
            Span::styled("Threshold: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{}-of-{}", 
                    session.map(|s| s.threshold).unwrap_or(0),
                    session.map(|s| s.total).unwrap_or(0)
                ), 
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(vec![
            Span::styled("Curve: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                session.map(|s| s.curve_type.as_str()).unwrap_or("Unknown"), 
                Style::default().fg(Color::Cyan)
            ),
        ]),
        Line::from(vec![
            Span::styled("Created: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(), 
                Style::default().fg(Color::Gray)
            ),
        ]),
    ];

    let info_para = Paragraph::new(wallet_info)
        .block(Block::default().borders(Borders::ALL).title("Wallet Details"));
    f.render_widget(info_para, chunks[1]);

    // Address details or action menu
    if show_address_details {
        // Show blockchain addresses
        let mut address_lines = vec![
            Line::from(vec![
                Span::styled("Blockchain Addresses:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
        ];

        // Add addresses from app state (with stable rendering)
        if !app.blockchain_addresses.is_empty() {
            // Sort addresses by blockchain name for consistent order
            let mut sorted_addresses: Vec<_> = app.blockchain_addresses
                .iter()
                .filter(|a| a.enabled)
                .collect();
            sorted_addresses.sort_by(|a, b| a.blockchain.cmp(&b.blockchain));
            
            for addr_info in sorted_addresses {
                address_lines.push(Line::from(vec![
                    Span::styled(format!("{}:", addr_info.blockchain), Style::default().fg(Color::Yellow)),
                ]));
                address_lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(&addr_info.address, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                ]));
                address_lines.push(Line::from(""));
            }
        } else {
            // Show the group public key and addresses based on curve type
            let curve_type = app.session.as_ref()
                .map(|s| s.curve_type.as_str())
                .unwrap_or("Unknown");
            
            // Show group public key (this is the main FROST output)
            if let Some(verifying_key) = &app.group_public_key {
                // Convert the verifying key to hex string for display
                let key_bytes = verifying_key.serialize().unwrap_or_default();
                let group_key_hex = hex::encode(&key_bytes);
                address_lines.push(Line::from(vec![
                    Span::styled("Group Public Key (FROST):", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]));
                // Display the key in hex format (truncated for readability)
                let display_key = if group_key_hex.len() > 64 {
                    format!("{}...{}", &group_key_hex[..32], &group_key_hex[group_key_hex.len()-32..])
                } else {
                    group_key_hex.clone()
                };
                address_lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(display_key, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                ]));
                address_lines.push(Line::from(""));
            }
            
            // Show blockchain addresses based on curve compatibility
            use crate::blockchain_config::{CurveType, get_compatible_chains, generate_address_for_chain};
            
            if let Some(curve) = CurveType::from_string(curve_type) {
                let compatible_chains = get_compatible_chains(&curve);
                
                if !compatible_chains.is_empty() {
                    address_lines.push(Line::from(vec![
                        Span::styled(format!("Compatible Blockchains ({} curve):", curve_type), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    ]));
                    address_lines.push(Line::from(""));
                    
                    // Check if we have cached addresses first to avoid regenerating
                    if let Some(addr) = &app.etherum_public_key {
                        // We have a cached address, display it for all compatible Ethereum-style chains
                        for (chain_id, chain_info) in compatible_chains.iter().take(5) {
                            if matches!(chain_id.as_ref(), "ethereum" | "bsc" | "polygon" | "avalanche") {
                                address_lines.push(Line::from(vec![
                                    Span::styled(format!("{} ({}):", chain_info.name, chain_info.symbol), Style::default().fg(Color::Yellow)),
                                ]));
                                address_lines.push(Line::from(vec![
                                    Span::raw("  "),
                                    Span::styled(addr.clone(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                                ]));
                            } else {
                                // Non-Ethereum chain, show pending
                                address_lines.push(Line::from(vec![
                                    Span::styled(format!("{} ({}):", chain_info.name, chain_info.symbol), Style::default().fg(Color::Yellow)),
                                ]));
                                address_lines.push(Line::from(vec![
                                    Span::raw("  "),
                                    Span::styled("(Address generation pending)", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
                                ]));
                            }
                        }
                    } else if let Some(verifying_key) = &app.group_public_key {
                        // No cached address, generate them (but this should be rare)
                        let group_public_key = verifying_key.serialize().unwrap_or_default();
                        {
                            // Show addresses for each compatible chain
                            for (chain_id, chain_info) in compatible_chains.iter().take(5) {
                                match generate_address_for_chain(&group_public_key, curve_type, chain_id) {
                                    Ok(address) => {
                                        address_lines.push(Line::from(vec![
                                            Span::styled(format!("{} ({}):", chain_info.name, chain_info.symbol), Style::default().fg(Color::Yellow)),
                                        ]));
                                        address_lines.push(Line::from(vec![
                                            Span::raw("  "),
                                            Span::styled(address, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                                        ]));
                                    }
                                    Err(_) => {
                                        // Address generation not implemented for this chain
                                        address_lines.push(Line::from(vec![
                                            Span::styled(format!("{} ({}):", chain_info.name, chain_info.symbol), Style::default().fg(Color::Yellow)),
                                        ]));
                                        address_lines.push(Line::from(vec![
                                            Span::raw("  "),
                                            Span::styled("(Address generation pending)", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
                                        ]));
                                    }
                                }
                            }
                            
                            // If there are more compatible chains, mention them
                            if compatible_chains.len() > 5 {
                                address_lines.push(Line::from(""));
                                address_lines.push(Line::from(vec![
                                    Span::styled(format!("+ {} more compatible chains", compatible_chains.len() - 5), Style::default().fg(Color::Gray)),
                                ]));
                            }
                        }
                    } else if let Some(addr) = &app.etherum_public_key {
                        // Fallback to showing just the stored address
                        let (_chain_id, chain_info) = &compatible_chains[0];
                        address_lines.push(Line::from(vec![
                            Span::styled(format!("{} Address:", chain_info.name), Style::default().fg(Color::Yellow)),
                        ]));
                        address_lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(addr, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                        ]));
                        address_lines.push(Line::from(""));
                    }
                } else {
                    // No compatible chains for this curve
                    address_lines.push(Line::from(vec![
                        Span::styled(format!("⚠️  No blockchain addresses available for {} curve", curve_type), Style::default().fg(Color::Red)),
                    ]));
                }
            }
            
            address_lines.push(Line::from(""));
            address_lines.push(Line::from(vec![
                Span::styled("ℹ️  This is your threshold wallet", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
            ]));
            address_lines.push(Line::from(vec![
                Span::styled("ℹ️  All participants share the same group public key", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
            ]));
            if let Some(sol_addr) = &app.solana_public_key {
                address_lines.push(Line::from(""));
                address_lines.push(Line::from(vec![
                    Span::styled("Solana:", Style::default().fg(Color::Yellow)),
                ]));
                address_lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(sol_addr, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                ]));
            }
        }

        // Use a scrollable paragraph with wrap to prevent overflow
        let addresses_para = Paragraph::new(address_lines)
            .block(Block::default().borders(Borders::ALL).title("Blockchain Addresses"))
            .wrap(Wrap { trim: false })
            .scroll((0, 0)); // Fixed scroll position at top
        f.render_widget(addresses_para, chunks[2]);
    } else {
        // Show action menu
        let actions = vec![
            "📋 View Addresses",
            "💾 Export Wallet",
            "🔐 Create Backup",
            "📤 Send Transaction",
            "🏠 Return to Main Menu",
        ];

        let action_items: Vec<ListItem> = actions.iter().enumerate().map(|(i, action)| {
            let style = if i == selected_action {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(action.to_string()).style(style)
        }).collect();

        let action_list = List::new(action_items)
            .block(Block::default().borders(Borders::ALL).title("What would you like to do?"));
        f.render_widget(action_list, chunks[2]);
    }

    // Instructions
    let instructions = if show_address_details {
        "Esc/B: Back to actions  Q: Main menu  C: Copy address (not yet implemented)"
    } else {
        "↑↓: Navigate  Enter: Select  Esc/Q: Main menu"
    };

    let instr_para = Paragraph::new(instructions)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instr_para, chunks[3]);
}

pub fn handle_key_event<C>(
    key: KeyEvent,
    // Add generic parameter here
    app: &mut AppState<C>, // Now mutable and generic
    input: &mut String,                // Now mutable
    input_mode: &mut bool,             // Now mutable
    ui_mode: &mut UIMode,              // Add UI mode
    // Update the sender type to use the new InternalCommand
    cmd_tx: &mpsc::UnboundedSender<InternalCommand<C>>, // Pass as reference
) -> anyhow::Result<bool> where C: Ciphersuite {
    
    match ui_mode {
        UIMode::HelpPopup => {
            // Handle help popup
            match key.code {
                KeyCode::Esc => {
                    // Close help popup
                    *ui_mode = UIMode::Normal;
                }
                _ => {}
            }
        }
        UIMode::SigningRequestPopup { selected_index } => {
            // Handle popup navigation
            match key.code {
                KeyCode::Up => {
                    if *selected_index > 0 {
                        *selected_index -= 1;
                    }
                }
                KeyCode::Down => {
                    if *selected_index < app.pending_signing_requests.len().saturating_sub(1) {
                        *selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    // Accept the selected signing request
                    if let Some(req) = app.pending_signing_requests.get(*selected_index) {
                        let signing_id = req.signing_id.clone();
                        let _ = cmd_tx.send(InternalCommand::AcceptSigning {
                            signing_id: signing_id.clone(),
                        });
                        
                        // Remove from pending list
                        app.pending_signing_requests.remove(*selected_index);
                        
                        // Exit popup if no more requests
                        if app.pending_signing_requests.is_empty() {
                            *ui_mode = UIMode::Normal;
                        } else if *selected_index >= app.pending_signing_requests.len() {
                            *selected_index = app.pending_signing_requests.len() - 1;
                        }
                    }
                }
                KeyCode::Esc => {
                    // Exit popup without accepting
                    *ui_mode = UIMode::Normal;
                }
                _ => {}
            }
        }
        UIMode::SigningInitiatePopup { chain_index, transaction_input, input_mode: popup_input_mode } => {
            if *popup_input_mode {
                // Handle text input for transaction data
                match key.code {
                    KeyCode::Enter => {
                        // Submit the signing request
                        let chains = vec![
                            ("Ethereum", "ethereum", Some(1)),
                            ("BSC", "bsc", Some(56)),
                            ("Polygon", "polygon", Some(137)),
                            ("Arbitrum", "arbitrum", Some(42161)),
                            ("Optimism", "optimism", Some(10)),
                            ("Avalanche", "avalanche", Some(43114)),
                            ("Solana", "solana", None),
                            ("Bitcoin", "bitcoin", None),
                            ("Bitcoin Testnet", "bitcoin-testnet", None),
                        ];
                        
                        if let Some((_, blockchain_id, chain_id)) = chains.get(*chain_index) {
                            let _ = cmd_tx.send(InternalCommand::InitiateSigning {
                                transaction_data: transaction_input.clone(),
                                blockchain: blockchain_id.to_string(),
                                chain_id: *chain_id,
                            });
                        }
                        
                        // Exit popup
                        *ui_mode = UIMode::Normal;
                    }
                    KeyCode::Esc => {
                        // Exit input mode
                        *popup_input_mode = false;
                    }
                    KeyCode::Char(c) => {
                        transaction_input.push(c);
                    }
                    KeyCode::Backspace => {
                        transaction_input.pop();
                    }
                    _ => {}
                }
            } else {
                // Handle chain selection
                match key.code {
                    KeyCode::Up => {
                        if *chain_index > 0 {
                            *chain_index -= 1;
                        }
                    }
                    KeyCode::Down => {
                        let chains_count = 9; // Number of chains in the list
                        if *chain_index < chains_count - 1 {
                            *chain_index += 1;
                        }
                    }
                    KeyCode::Tab => {
                        // Enter transaction input mode
                        *popup_input_mode = true;
                    }
                    KeyCode::Enter => {
                        // Submit with current data
                        let chains = vec![
                            ("Ethereum", "ethereum", Some(1)),
                            ("BSC", "bsc", Some(56)),
                            ("Polygon", "polygon", Some(137)),
                            ("Arbitrum", "arbitrum", Some(42161)),
                            ("Optimism", "optimism", Some(10)),
                            ("Avalanche", "avalanche", Some(43114)),
                            ("Solana", "solana", None),
                            ("Bitcoin", "bitcoin", None),
                            ("Bitcoin Testnet", "bitcoin-testnet", None),
                        ];
                        
                        if let Some((_, blockchain_id, chain_id)) = chains.get(*chain_index) {
                            if !transaction_input.is_empty() {
                                let _ = cmd_tx.send(InternalCommand::InitiateSigning {
                                    transaction_data: transaction_input.clone(),
                                    blockchain: blockchain_id.to_string(),
                                    chain_id: *chain_id,
                                });
                                *ui_mode = UIMode::Normal;
                            } else {
                                *popup_input_mode = true;
                            }
                        }
                    }
                    KeyCode::Esc => {
                        // Exit popup
                        *ui_mode = UIMode::Normal;
                    }
                    _ => {}
                }
            }
        }
        UIMode::MainMenu { selected_index } => {
            match key.code {
                KeyCode::Up => {
                    if *selected_index > 0 {
                        *selected_index -= 1;
                    }
                }
                KeyCode::Down => {
                    if *selected_index < 5 { // We have 6 menu items (0-5)
                        *selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    match *selected_index {
                        0 => {
                            // Create Wallet - go to new wallet creation flow
                            *ui_mode = UIMode::ModeSelection { selected_index: 0 };
                        }
                        1 => {
                            // Create/Join Session - go to session discovery
                            *ui_mode = UIMode::SessionDiscovery {
                                selected_index: 0,
                                filter_text: String::new(),
                                input_mode: false,
                            };
                            let _ = cmd_tx.send(InternalCommand::DiscoverSessions);
                        }
                        2 => {
                            // List Wallets
                            *ui_mode = UIMode::WalletListPopup { selected_index: 0 };
                            let _ = cmd_tx.send(InternalCommand::ListWallets);
                        }
                        3 => {
                            // Sign Transaction
                            *ui_mode = UIMode::SigningInitiatePopup {
                                chain_index: 0,
                                transaction_input: String::new(),
                                input_mode: false,
                            };
                        }
                        4 => {
                            // Accept Session
                            let sessions = app.invites.clone();
                            if sessions.is_empty() {
                                *ui_mode = UIMode::Normal;
                            } else {
                                *ui_mode = UIMode::AcceptSessionPopup {
                                    selected_index: 0,
                                    sessions,
                                };
                            }
                        }
                        5 => {
                            // View Help
                            *ui_mode = UIMode::HelpPopup;
                        }
                        _ => {}
                    }
                }
                KeyCode::Esc => {
                    *ui_mode = UIMode::Normal;
                }
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
                }
                KeyCode::Enter => {
                    // Submit the proposal
                    if let (Ok(total), Ok(thresh)) = (total_participants.parse::<u16>(), threshold.parse::<u16>()) {
                        let participant_list: Vec<String> = participants
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        
                        if !session_name.is_empty() && total >= 2 && thresh >= 1 && thresh <= total && participant_list.len() == total as usize {
                            let _ = cmd_tx.send(InternalCommand::ProposeSession {
                                session_id: session_name.clone(),
                                total,
                                threshold: thresh,
                                participants: participant_list,
                            });
                            *ui_mode = UIMode::Normal;
                        } else {
                        }
                    } else {
                    }
                }
                KeyCode::Esc => {
                    *ui_mode = UIMode::Normal;
                }
                _ => {}
            }
        }
        UIMode::WalletListPopup { selected_index } => {
            match key.code {
                KeyCode::Up => {
                    if *selected_index > 0 {
                        *selected_index -= 1;
                    }
                }
                KeyCode::Down => {
                    // This would need to check against actual wallet count
                    *selected_index += 1;
                }
                KeyCode::Esc => {
                    *ui_mode = UIMode::Normal;
                }
                _ => {}
            }
        }
        UIMode::AcceptSessionPopup { selected_index, sessions } => {
            match key.code {
                KeyCode::Up => {
                    if *selected_index > 0 {
                        *selected_index -= 1;
                    }
                }
                KeyCode::Down => {
                    if *selected_index < sessions.len().saturating_sub(1) {
                        *selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    if let Some(session) = sessions.get(*selected_index) {
                        let _ = cmd_tx.send(InternalCommand::AcceptSessionProposal(session.session_id.clone()));
                        *ui_mode = UIMode::Normal;
                    }
                }
                KeyCode::Esc => {
                    *ui_mode = UIMode::Normal;
                }
                _ => {}
            }
        }

        // ==========================================
        // NEW WALLET CREATION FLOW KEY HANDLING
        // ==========================================

        UIMode::WelcomeScreen => {
            match key.code {
                KeyCode::Enter => {
                    *ui_mode = UIMode::PathSelection { selected_index: 0 };
                }
                KeyCode::Char('q') => {
                    return Ok(true); // Quit application
                }
                KeyCode::Char('?') => {
                    *ui_mode = UIMode::HelpPopup;
                }
                _ => {}
            }
        }

        UIMode::PathSelection { selected_index } => {
            match key.code {
                KeyCode::Up => {
                    *selected_index = selected_index.saturating_sub(1);
                }
                KeyCode::Down => {
                    if *selected_index < 2 { // 3 options total (0-2)
                        *selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    match selected_index {
                        0 => { // Create New Wallet
                            *ui_mode = UIMode::ModeSelection { selected_index: 0 };
                        }
                        1 => { // Join Wallet Session
                            *ui_mode = UIMode::SessionDiscovery { 
                                selected_index: 0, 
                                filter_text: String::new(),
                                input_mode: false,
                            };
                            // Trigger session discovery
                            let _ = cmd_tx.send(InternalCommand::DiscoverSessions);
                        }
                        2 => { // Select Existing Wallet
                            *ui_mode = UIMode::WalletListPopup { selected_index: 0 };
                        }
                        _ => {}
                    }
                }
                KeyCode::Esc => {
                    *ui_mode = UIMode::WelcomeScreen;
                }
                KeyCode::Char('q') => {
                    return Ok(true);
                }
                _ => {}
            }
        }

        UIMode::ModeSelection { selected_index } => {
            match key.code {
                KeyCode::Up => {
                    *selected_index = selected_index.saturating_sub(1);
                }
                KeyCode::Down => {
                    if *selected_index < 2 { // 3 modes: Online, Offline, Hybrid
                        *selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    // Store selected mode and proceed to curve selection
                    let mode = match selected_index {
                        0 => crate::handlers::session_handler::WalletCreationMode::Online,
                        1 => crate::handlers::session_handler::WalletCreationMode::Offline,
                        2 => crate::handlers::session_handler::WalletCreationMode::Hybrid,
                        _ => crate::handlers::session_handler::WalletCreationMode::Online, // Default fallback
                    };
                    app.wallet_creation_mode = Some(mode);
                    *ui_mode = UIMode::CurveSelection { selected_index: 0 };
                }
                KeyCode::Esc => {
                    *ui_mode = UIMode::PathSelection { selected_index: 0 };
                }
                _ => {}
            }
        }

        UIMode::CurveSelection { selected_index } => {
            match key.code {
                KeyCode::Up => {
                    *selected_index = selected_index.saturating_sub(1);
                }
                KeyCode::Down => {
                    if *selected_index < 1 { // 2 curves: secp256k1, ed25519
                        *selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    // Store selected curve and proceed to configuration
                    let curve = match selected_index {
                        0 => "secp256k1".to_string(),
                        1 => "ed25519".to_string(),
                        _ => "secp256k1".to_string(), // Default fallback
                    };
                    app.wallet_creation_curve = Some(curve);
                    
                    // Initialize default configuration based on selections
                    *ui_mode = UIMode::WalletConfiguration {
                        selected_field: 0,
                        wallet_name: "MyWallet".to_string(),
                        description: String::new(),
                        total: "3".to_string(),
                        threshold: "2".to_string(),
                        timeout_hours: "24".to_string(),
                        auto_discovery: true,
                        blockchain_configs: if *selected_index == 0 {
                            // secp256k1 blockchains
                            vec![
                                ("Ethereum".to_string(), true),
                                ("Bitcoin".to_string(), false),
                                ("Polygon".to_string(), false),
                            ]
                        } else {
                            // ed25519 blockchains
                            vec![
                                ("Solana".to_string(), true),
                                ("Near".to_string(), false),
                                ("Polkadot".to_string(), false),
                            ]
                        },
                        selected_blockchain: 0,
                    };
                }
                KeyCode::Esc => {
                    *ui_mode = UIMode::ModeSelection { selected_index: 0 };
                }
                _ => {}
            }
        }

        UIMode::WalletConfiguration { 
            selected_field, 
            wallet_name, 
            description, 
            total, 
            threshold, 
            timeout_hours,
            auto_discovery,
            blockchain_configs,
            selected_blockchain: _,
        } => {
            match key.code {
                KeyCode::Up => {
                    *selected_field = selected_field.saturating_sub(1);
                }
                KeyCode::Down => {
                    if *selected_field < 3 { // Basic fields: name, desc, timeout, auto-discovery
                        *selected_field += 1;
                    }
                }
                KeyCode::Tab => {
                    // Switch to blockchain selection
                    *selected_field = 100; // Special value to indicate blockchain selection mode
                }
                KeyCode::Enter => {
                    // Handle field editing
                    match selected_field {
                        0 => { // Wallet name - simple increment for demo
                            let num = wallet_name.chars().last()
                                .and_then(|c| c.to_digit(10))
                                .unwrap_or(0) + 1;
                            *wallet_name = format!("MyWallet{}", num);
                        }
                        1 => { // Description
                            if description.is_empty() {
                                *description = "Threshold wallet".to_string();
                            } else {
                                description.clear();
                            }
                        }
                        3 => { // Auto-discovery toggle
                            *auto_discovery = !*auto_discovery;
                        }
                        _ => {}
                    }
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    // Start wallet creation - convert UI state to backend configuration and start DKG
                    let curve_name = app.wallet_creation_curve.as_deref().unwrap_or("secp256k1");
                    
                    let mode = app.wallet_creation_mode.as_ref().unwrap_or(&crate::handlers::session_handler::WalletCreationMode::Online);
                    
                    // Clone values needed before creating config
                    let wallet_name_clone = wallet_name.clone();
                    let description_clone = description.clone();
                    let total_clone = total.clone();
                    let threshold_clone = threshold.clone();
                    let timeout_hours_clone = timeout_hours.clone();
                    let auto_discovery_clone = *auto_discovery;
                    let blockchain_configs_clone = blockchain_configs.clone();
                    
                    // Start DKG progress screen
                    *ui_mode = UIMode::DkgProgress { allow_cancel: true };
                    
                    // Send command to start wallet creation
                    let config = WalletSessionConfig {
                        wallet_name: wallet_name_clone,
                        description: if description_clone.is_empty() { None } else { Some(description_clone) },
                        total: total_clone.parse::<u16>().unwrap_or(3u16),
                        threshold: threshold_clone.parse::<u16>().unwrap_or(2u16),
                        curve_type: curve_name.to_string(),
                        mode: mode.clone(),
                        timeout_hours: timeout_hours_clone.parse::<u8>().unwrap_or(24u8),
                        auto_discovery: auto_discovery_clone,
                        blockchain_config: blockchain_configs_clone.iter().map(|(name, enabled)| {
                            crate::handlers::session_handler::BlockchainConfig {
                                blockchain: name.clone(),
                                network: "mainnet".to_string(),
                                enabled: *enabled,
                                chain_id: None,
                            }
                        }).collect(),
                    };
                    
                    let _ = cmd_tx.send(InternalCommand::CreateWalletSession { config });
                }
                KeyCode::Esc => {
                    *ui_mode = UIMode::CurveSelection { selected_index: 0 };
                }
                _ => {}
            }
        }

        UIMode::SessionDiscovery { selected_index, filter_text, input_mode: discovery_input_mode } => {
            if *discovery_input_mode {
                // Handle filter text input
                match key.code {
                    KeyCode::Enter | KeyCode::Esc => {
                        *discovery_input_mode = false;
                    }
                    KeyCode::Char(c) => {
                        filter_text.push(c);
                    }
                    KeyCode::Backspace => {
                        filter_text.pop();
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Up => {
                        *selected_index = selected_index.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        let sessions_count = app.available_sessions.len();
                        if sessions_count > 0 && *selected_index < sessions_count - 1 {
                            *selected_index += 1;
                        }
                    }
                    KeyCode::Enter => {
                        // Join selected session
                        if let Some(session) = app.available_sessions.get(*selected_index) {
                            let _ = cmd_tx.send(InternalCommand::JoinSession(session.session_code.clone()));
                            *ui_mode = UIMode::DkgProgress { allow_cancel: false };
                        }
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        // Refresh sessions
                        let _ = cmd_tx.send(InternalCommand::DiscoverSessions);
                    }
                    KeyCode::Char('/') => {
                        // Start filtering
                        *discovery_input_mode = true;
                    }
                    KeyCode::Esc => {
                        *ui_mode = UIMode::PathSelection { selected_index: 1 };
                    }
                    _ => {}
                }
            }
        }

        UIMode::DkgProgress { allow_cancel } => {
            tracing::debug!("Key pressed in DkgProgress mode: {:?}, DKG state: {:?}", key.code, app.dkg_state);
            match key.code {
                KeyCode::Enter => {
                    tracing::info!("Enter key pressed in DkgProgress mode, DKG state: {:?}", app.dkg_state);
                    // Check if DKG is complete
                    if matches!(app.dkg_state, crate::utils::state::DkgState::Complete) {
                        tracing::info!("DKG is complete, switching to WalletComplete mode");
                        *ui_mode = UIMode::WalletComplete { 
                            selected_action: 0, 
                            show_address_details: true  // Changed to true to show addresses immediately
                        };
                        // Force UI redraw
                        return Ok(true);
                    } else {
                        tracing::warn!("Cannot show wallet details - DKG not complete. State: {:?}", app.dkg_state);
                    }
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    // Retry if failed
                    if let crate::utils::state::DkgState::Failed(_) = app.dkg_state {
                        // TODO: Implement retry logic
                        let _ = cmd_tx.send(InternalCommand::RetryDkg);
                    }
                }
                KeyCode::Char('v') | KeyCode::Char('V') => {
                    tracing::info!("'v' key pressed in DkgProgress mode, DKG state: {:?}", app.dkg_state);
                    // Alternative key to view wallet details (workaround if Enter doesn't work)
                    if matches!(app.dkg_state, crate::utils::state::DkgState::Complete) {
                        tracing::info!("DKG is complete, switching to WalletComplete mode via 'v' key");
                        *ui_mode = UIMode::WalletComplete { 
                            selected_action: 0, 
                            show_address_details: true  // Changed to true to show addresses immediately
                        };
                        // Force UI redraw
                        return Ok(true);
                    } else {
                        tracing::warn!("Cannot show wallet details via 'v' - DKG not complete");
                    }
                }
                KeyCode::Esc => {
                    // Return to main menu if DKG is complete or if cancellation is allowed
                    if matches!(app.dkg_state, crate::utils::state::DkgState::Complete) {
                        tracing::info!("DKG complete, returning to main menu");
                        *ui_mode = UIMode::Normal;
                    } else if *allow_cancel {
                        tracing::info!("Cancelling DKG and returning to main menu");
                        // Cancel DKG and return to main menu
                        let _ = cmd_tx.send(InternalCommand::CancelDkg);
                        *ui_mode = UIMode::PathSelection { selected_index: 0 };
                    }
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    // Debug key to check state
                    tracing::info!("=== DEBUG INFO ===");
                    tracing::info!("DKG State: {:?}", app.dkg_state);
                    tracing::info!("UI Mode: {:?}", ui_mode);
                    tracing::info!("Ethereum Address: {:?}", app.etherum_public_key);
                    tracing::info!("Wallet ID: {:?}", app.current_wallet_id);
                    tracing::info!("Session: {:?}", app.session.as_ref().map(|s| &s.session_id));
                    tracing::info!("==================");
                    
                    // Also show a debug message in the UI
                    app.log.push(format!("Debug: DKG={:?}, Addr={:?}", 
                        matches!(app.dkg_state, crate::utils::state::DkgState::Complete),
                        app.etherum_public_key.as_ref().map(|a| &a[..10.min(a.len())])
                    ));
                }
                _ => {}
            }
        }

        UIMode::WalletComplete { selected_action, show_address_details } => {
            if *show_address_details {
                match key.code {
                    KeyCode::Esc => {
                        *show_address_details = false;
                    }
                    KeyCode::Char('q') => {
                        *ui_mode = UIMode::PathSelection { selected_index: 0 };
                    }
                    KeyCode::Char('c') => {
                        // TODO: Copy selected address to clipboard
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Up => {
                        *selected_action = selected_action.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        if *selected_action < 4 { // 5 actions total (0-4)
                            *selected_action += 1;
                        }
                    }
                    KeyCode::Enter => {
                        match selected_action {
                            0 => { // View Addresses
                                *show_address_details = true;
                            }
                            1 => { // Export Wallet
                            }
                            2 => { // Create Backup
                            }
                            3 => { // Send Transaction
                                *ui_mode = UIMode::SigningInitiatePopup {
                                    chain_index: 0,
                                    transaction_input: String::new(),
                                    input_mode: false,
                                };
                            }
                            4 => { // Return to Main Menu
                                *ui_mode = UIMode::PathSelection { selected_index: 0 };
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Esc => {
                        *ui_mode = UIMode::PathSelection { selected_index: 0 };
                    }
                    _ => {}
                }
            }
        }

        UIMode::TemplateSelection { .. } => {
            // Template selection is handled by TUI provider key handlers
            // This function doesn't need to handle it directly
        }

        UIMode::Normal => {
            if *input_mode {
                // --- Input Mode Key Handling (mostly unchanged) ---
                match key.code {
            KeyCode::Enter => {
                let cmd_str = input.trim().to_string();
                input.clear();
                *input_mode = false; // Exit input mode immediately

                // Parse and handle command
                // Wrap shared messages when sending
                if cmd_str == "/list" {
                    let _ = cmd_tx.send(InternalCommand::SendToServer(ClientMsg::ListDevices));
                } else if cmd_str == "/wallets" || cmd_str.starts_with("/list_wallets") {
                    // Handle the /wallets or /list_wallets command
                    let _ = cmd_tx.send(InternalCommand::ListWallets);
                } else if cmd_str.starts_with("/propose") {
                    // Handle the propose command as per documentation
                    // Format: /propose <session_id> <total> <threshold> <device1,device2,...>
                    let parts: Vec<_> = cmd_str.splitn(5, ' ').collect();
                    if parts.len() == 5 {
                        let session_id = parts[1].to_string();
                        
                        // Parse total participants
                        if let Ok(total) = parts[2].parse::<u16>() {
                            // Parse threshold
                            if let Ok(threshold) = parts[3].parse::<u16>() {
                                // Parse participants list
                                let participants: Vec<String> = parts[4]
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .collect();
                                
                                // Validate inputs
                                if total < 2 {
                                } else if threshold < 1 || threshold > total {
                                    // Threshold must be between 1 and total participants
                                } else if participants.len() != total as usize {
                                    // Number of participants doesn't match the specified total
                                } else {
                                    // Send the command to propose a session
                                    let _ = cmd_tx.send(InternalCommand::ProposeSession {
                                        session_id: session_id.clone(),
                                        total,
                                        threshold,
                                        participants: participants.clone(),
                                    });
                                    
                                    // Proposed session
                                }
                            } else {
                                // Invalid threshold format
                            }
                        } else {
                            // Invalid total format
                        }
                    } else {
                        // Invalid /propose format
                    }
                } else if cmd_str.starts_with("/accept") {
                    // Enhanced /accept command that handles both session proposals and signing requests
                    let parts: Vec<_> = cmd_str.split_whitespace().collect();
                    if parts.len() == 2 {
                        let id = parts[1].to_string();
                        
                        // First, check if it's a session proposal
                        if app.invites.iter().any(|invite| invite.session_id == id) {
                            // Send command to accept the session proposal
                            let _ = cmd_tx.send(InternalCommand::AcceptSessionProposal(id.clone()));
                        }
                        // If not a session proposal, check if it's a signing request
                        else if let Some(signing_id) = app.signing_state.get_signing_id() {
                            if signing_id == id && matches!(app.signing_state, crate::utils::state::SigningState::AwaitingAcceptance { .. }) {
                                // Send command to accept the signing request
                                let _ = cmd_tx.send(InternalCommand::AcceptSigning {
                                    signing_id: id.clone(),
                                });
                            } else {
                            }
                        }
                        // Neither session proposal nor signing request found
                        else {
                        }
                    } else {
                    }
                } else if cmd_str == "/sign" || cmd_str.starts_with("/sign ") {
                    // Open the signing popup instead of parsing command line
                    *input_mode = false; // Exit input mode
                    *ui_mode = UIMode::SigningInitiatePopup {
                        chain_index: 0,
                        transaction_input: String::new(),
                        input_mode: false,
                    };
                } else if cmd_str.starts_with("/init_keystore") {
                    // Handle the /init_keystore command - use convention over configuration
                    
                    // Standard path: ~/.frost_keystore
                    let home_dir = dirs::home_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."));
                    let path = home_dir.join(".frost_keystore").to_string_lossy().into_owned();
                    
                    // Device name based on device ID
                    let device_name = format!("device-{}", app.device_id);
                    
                    let _ = cmd_tx.send(InternalCommand::InitKeystore {
                        path: path.clone(),
                        device_name: device_name.clone(),
                    });
                } else if cmd_str.starts_with("/create_wallet") {
                    // Handle the /create_wallet command - use convention over configuration
                    
                    // First check if DKG is complete
                    if !matches!(app.dkg_state, crate::utils::state::DkgState::Complete) {
                    } 
                    // Check if keystore is initialized
                    else if app.keystore.is_none() {
                    }
                    else {
                    
                    // Generate a wallet name based on the DKG session or date
                    let name = if let Some(session) = &app.session {
                        format!("wallet-{}", session.session_id)
                    } else {
                        // Use current date/time if no session
                        let now = chrono::Local::now();
                        format!("wallet-{}", now.format("%Y-%m-%d-%H%M"))
                    };
                    
                    // Use device ID as a simple default password
                    // In a real app, we might want to generate a secure random password or prompt the user
                    let password = app.device_id.clone();
                    
                    // Create simple description
                    let description = if let Some(session) = &app.session {
                        Some(format!("Threshold {}/{} wallet created on {}", 
                            session.threshold, 
                            session.total,
                            chrono::Local::now().format("%Y-%m-%d %H:%M")
                        ))
                    } else {
                        None
                    };
                    
                    // Default tags based on the cryptographic curve
                    let curve_name = if app.solana_public_key.is_some() {
                        "ed25519"
                    } else if app.etherum_public_key.is_some() {
                        "secp256k1" 
                    } else {
                        "unknown"
                    };
                    
                    let tags = vec![curve_name.to_string()];
                    
                    let _ = cmd_tx.send(InternalCommand::CreateWallet {
                        name: name.clone(),
                        description: description.clone(),
                        password: password.clone(),
                        tags: tags.clone(),
                    });
                    
                    } // close the else block
                } else if cmd_str.starts_with("/locate_wallet") {
                    // Handle the /locate_wallet command - show file path for Chrome extension import
                    let parts: Vec<_> = cmd_str.split_whitespace().collect();
                    if parts.len() == 2 {
                        let wallet_id = parts[1].to_string();
                        
                        let _ = cmd_tx.send(InternalCommand::LocateWallet {
                            wallet_id: wallet_id.clone(),
                        });
                    } else {
                    }
                } else if cmd_str.starts_with("/acceptSign") {
                    // Handle the /acceptSign command
                    let parts: Vec<_> = cmd_str.split_whitespace().collect();
                    if parts.len() == 2 {
                        let signing_id = parts[1].to_string();
                        
                        let _ = cmd_tx.send(InternalCommand::AcceptSigning {
                            signing_id: signing_id.clone(),
                        });
                    } else {
                    }
                } else if cmd_str.starts_with("/relay") {
                    let parts: Vec<_> = cmd_str.splitn(3, ' ').collect();
                    if parts.len() == 3 {
                        let target_device_id = parts[1].to_string();
                        let json_str = parts[2];
                        match serde_json::from_str::<serde_json::Value>(json_str) {
                            Ok(data) => {
                                let _ = cmd_tx.send(InternalCommand::SendToServer(
                                    ClientMsg::Relay {
                                        to: target_device_id.clone(),
                                        data,
                                    },
                                ));
                                app.log
                                    .push(format!("Relaying message to {}", target_device_id));
                            }
                            Err(_e) => {
                            }
                        }
                    } else {
                        // Invalid /relay format
                    }
                } else if cmd_str.starts_with("/send") {
                    // This command now sends a simple text message via WebRTCMessage::SimpleMessage
                    let parts: Vec<_> = cmd_str.splitn(3, ' ').collect();
                    if parts.len() >= 3 {
                        let target_device_id = parts[1].to_string();
                        let message_text = parts[2].to_string();

                        // Always log the send attempt, regardless of connection state
                        // Attempting to send direct message

                        // Send internal command
                        let _ = cmd_tx.send(InternalCommand::SendDirect {
                            to: target_device_id.clone(),
                            message: message_text.clone(),
                        });

                        // Log the command for visibility
                        // Command: /send executed
                    } else {
                        // Invalid /send format
                    }
                } else if cmd_str.starts_with("/offline") {
                    // let parts: Vec<_> = cmd_str.split_whitespace().collect();
                    // if parts.len() == 2 {
                    //     match parts[1] {
                    //         "on" => {
                    //             let _ = cmd_tx.send(InternalCommand::OfflineMode { enabled: true });
                    //         }
                    //         "off" => {
                    //             let _ = cmd_tx.send(InternalCommand::OfflineMode { enabled: false });
                    //         }
                    //         _ => {
                    //         }
                    //     }
                    // } else {
                    // }
                } else if cmd_str.starts_with("/create_signing_request") || 
                          cmd_str.starts_with("/export_signing_request") ||
                          cmd_str.starts_with("/import_signing_request") ||
                          cmd_str == "/offline_sessions" ||
                          cmd_str.starts_with("/review_signing_request") {
                    // Offline commands not implemented yet
                } else if !cmd_str.is_empty() {
                    // Unknown command
                }
            } // End KeyCode::Enter
            KeyCode::Char(c) => {
                input.push(c);
            }
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Esc => {
                *input_mode = false;
                input.clear();
            }
            _ => {}
        } // End match key.code for input mode
    } else {
        // --- Normal Mode Key Handling (Add scroll keys) ---
        match key.code {
            KeyCode::Char('i') => {
                *input_mode = true;
            }
            KeyCode::Char('o') => {
                // Accept the first pending invitation
                if let Some(invite) = app.invites.first() {
                    let session_id = invite.session_id.clone();
                    let _ = cmd_tx.send(InternalCommand::AcceptSessionProposal(session_id.clone()));
                } else {
                }
            }
            KeyCode::Char('q') => {
                return Ok(false); // Signal to quit
            }
            KeyCode::Char('?') => {
                // Show help popup
                *ui_mode = UIMode::HelpPopup;
            }
            KeyCode::Char('s') => {
                // Save log to <device_id>.log
                let filename = format!("{}.log", app.device_id.trim());
                match std::fs::write(&filename, app.log.join("\n")) {
                    Ok(_) => tracing::info!("Log saved to {}", filename),
                    Err(_e) => tracing::error!("Failed to save log: {}", _e),
                }
            }
            KeyCode::Char('d') => {
                // Quick test - Send predefined session proposal
                let session_id = "wallet_2of3".to_string();
                let participants = vec!["mpc-1".to_string(), "mpc-2".to_string(), "mpc-3".to_string()];
                
                let _ = cmd_tx.send(InternalCommand::ProposeSession {
                    session_id: session_id.clone(),
                    total: 3,
                    threshold: 2,
                    participants: participants.clone(),
                });
                
                // Quick test: Proposed session
            }
            KeyCode::Up => {
                app.log_scroll = app.log_scroll.saturating_sub(1);
            }
            KeyCode::Down => {
                app.log_scroll = app.log_scroll.saturating_add(1);
            }
            KeyCode::Tab => {
                // Show signing request popup if there are pending requests
                if !app.pending_signing_requests.is_empty() {
                    *ui_mode = UIMode::SigningRequestPopup { selected_index: 0 };
                }
            }
            KeyCode::Char('m') | KeyCode::Char('M') => {
                // Open main menu
                *ui_mode = UIMode::MainMenu { selected_index: 0 };
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                // Disabled - use the wallet creation flow instead
            }
            KeyCode::Char('w') | KeyCode::Char('W') => {
                // Quick access to wallet list
                *ui_mode = UIMode::WalletListPopup { selected_index: 0 };
                let _ = cmd_tx.send(InternalCommand::ListWallets);
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                // Quick access to accept session
                let sessions = app.invites.clone();
                if sessions.is_empty() {
                } else {
                    *ui_mode = UIMode::AcceptSessionPopup {
                        selected_index: 0,
                        sessions,
                    };
                }
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                // Quick access to sign transaction
                *ui_mode = UIMode::SigningInitiatePopup {
                    chain_index: 0,
                    transaction_input: String::new(),
                    input_mode: false,
                };
            }
            _ => {}
        } // End match key.code for normal mode
    } // End else (normal mode key handling)
    } // End UIMode::Normal case
    } // Close match ui_mode
    Ok(true) // Continue loop by default
}
