use crate::protocal::signal::SessionInfo;
use crate::utils::state::{AppState, DkgStateDisplay, InternalCommand, MeshStatus}; 
use webrtc_signal_server::ClientMsg;
use crossterm::event::{KeyCode, KeyEvent};
use std::any::TypeId; // Added for ciphersuite check
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

#[derive(Debug, Clone, PartialEq)]
pub enum UIMode {
    Normal,
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
}


pub fn draw_main_ui<B: Backend, C: Ciphersuite>(
    terminal: &mut Terminal<B>,
    app: &AppState<C>,
    input: &str,
    input_mode: bool,
    ui_mode: &UIMode,
) -> io::Result<()> {
    terminal.draw(|f| {
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
            .scroll((app.log_scroll, 0)); // Apply vertical scroll offset
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
        
        // Draw popup if needed
        match ui_mode {
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
            MeshStatus::PartiallyReady { ready_devices, total_devices } => format!("Partially Ready ({}/{})", ready_devices.len(), total_devices),
            MeshStatus::Ready => "Ready".to_string(),
        };
        
        let mesh_style = match &app.mesh_status {
            MeshStatus::Incomplete => Style::default().fg(Color::Red),
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
                        app.log.push(format!("Accepting signing request: {}", signing_id));
                        
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
                            app.log.push(format!("Initiating signing on {}", blockchain_id));
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
                                app.log.push(format!("Initiating signing on {}", blockchain_id));
                                *ui_mode = UIMode::Normal;
                            } else {
                                app.log.push("Please enter transaction data".to_string());
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
                    if *selected_index < 4 { // We have 5 menu items (0-4)
                        *selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    match *selected_index {
                        0 => {
                            // Create/Join Session
                            *ui_mode = UIMode::SessionProposalPopup {
                                session_name: String::new(),
                                total_participants: String::new(),
                                threshold: String::new(),
                                participants: String::new(),
                                selected_field: 0,
                            };
                        }
                        1 => {
                            // List Wallets
                            *ui_mode = UIMode::WalletListPopup { selected_index: 0 };
                            let _ = cmd_tx.send(InternalCommand::ListWallets);
                        }
                        2 => {
                            // Sign Transaction
                            *ui_mode = UIMode::SigningInitiatePopup {
                                chain_index: 0,
                                transaction_input: String::new(),
                                input_mode: false,
                            };
                        }
                        3 => {
                            // Accept Session
                            let sessions = app.invites.clone();
                            if sessions.is_empty() {
                                app.log.push("No pending session invitations".to_string());
                                *ui_mode = UIMode::Normal;
                            } else {
                                *ui_mode = UIMode::AcceptSessionPopup {
                                    selected_index: 0,
                                    sessions,
                                };
                            }
                        }
                        4 => {
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
                            app.log.push(format!("Proposed session '{}'", session_name));
                            *ui_mode = UIMode::Normal;
                        } else {
                            app.log.push("Invalid input. Check all fields.".to_string());
                        }
                    } else {
                        app.log.push("Invalid number format for total or threshold".to_string());
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
                        app.log.push(format!("Accepting session '{}'", session.session_id));
                        *ui_mode = UIMode::Normal;
                    }
                }
                KeyCode::Esc => {
                    *ui_mode = UIMode::Normal;
                }
                _ => {}
            }
        }
        UIMode::Normal => {
            if *input_mode {
                // --- Input Mode Key Handling (mostly unchanged) ---
        match key.code {
            KeyCode::Enter => {
                let cmd_str = input.trim().to_string();
                input.clear();
                *input_mode = false; // Exit input mode immediately
                app.log.push("Exited input mode.".to_string());

                // Parse and handle command
                // Wrap shared messages when sending
                if cmd_str == "/list" {
                    let _ = cmd_tx.send(InternalCommand::SendToServer(ClientMsg::ListDevices));
                } else if cmd_str == "/wallets" || cmd_str.starts_with("/list_wallets") {
                    // Handle the /wallets or /list_wallets command
                    let _ = cmd_tx.send(InternalCommand::ListWallets);
                    app.log.push("Listing available wallets...".to_string());
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
                                    app.log.push("Total participants must be at least 2".to_string());
                                } else if threshold < 1 || threshold > total {
                                    app.log.push(format!(
                                        "Threshold must be between 1 and {} (total participants)",
                                        total
                                    ));
                                } else if participants.len() != total as usize {
                                    app.log.push(format!(
                                        "Number of participants ({}) doesn't match the specified total ({})",
                                        participants.len(), total
                                    ));
                                } else {
                                    // Send the command to propose a session
                                    let _ = cmd_tx.send(InternalCommand::ProposeSession {
                                        session_id: session_id.clone(),
                                        total,
                                        threshold,
                                        participants: participants.clone(),
                                    });
                                    
                                    app.log.push(format!(
                                        "Proposed session '{}' with {} participants (threshold: {})",
                                        session_id, total, threshold
                                    ));
                                }
                            } else {
                                app.log.push("Invalid threshold value. Must be a positive number.".to_string());
                            }
                        } else {
                            app.log.push("Invalid total value. Must be a positive number.".to_string());
                        }
                    } else {
                        app.log.push(
                            "Invalid /propose format. Use: /propose <session_id> <total> <threshold> <device1,device2,...>"
                                .to_string(),
                        );
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
                            app.log.push(format!("Accepting session proposal '{}'", id));
                        }
                        // If not a session proposal, check if it's a signing request
                        else if let Some(signing_id) = app.signing_state.get_signing_id() {
                            if signing_id == id && matches!(app.signing_state, crate::utils::state::SigningState::AwaitingAcceptance { .. }) {
                                // Send command to accept the signing request
                                let _ = cmd_tx.send(InternalCommand::AcceptSigning {
                                    signing_id: id.clone(),
                                });
                                app.log.push(format!("Accepting signing request '{}'", id));
                            } else {
                                app.log.push(format!("No pending signing request with ID '{}'. Current signing state: {}", id, app.signing_state.display_status()));
                            }
                        }
                        // Neither session proposal nor signing request found
                        else {
                            app.log.push(format!("No session proposal or signing request found with ID '{}'. Use /acceptSign <id> for signing requests or check available invites.", id));
                        }
                    } else {
                        app.log.push("Invalid /accept format. Use: /accept <id> (works for both session proposals and signing requests)".to_string());
                    }
                } else if cmd_str == "/sign" || cmd_str.starts_with("/sign ") {
                    // Open the signing popup instead of parsing command line
                    *input_mode = false; // Exit input mode
                    *ui_mode = UIMode::SigningInitiatePopup {
                        chain_index: 0,
                        transaction_input: String::new(),
                        input_mode: false,
                    };
                    app.log.push("Opening signing interface...".to_string());
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
                    app.log.push(format!("Initializing keystore at {}", path));
                } else if cmd_str.starts_with("/create_wallet") {
                    // Handle the /create_wallet command - use convention over configuration
                    
                    // First check if DKG is complete
                    if !matches!(app.dkg_state, crate::utils::state::DkgState::Complete) {
                        app.log.push("⚠️ DKG process is not complete. Cannot create wallet yet.".to_string());
                        app.log.push("Complete the DKG process first by joining a session and completing key generation.".to_string());
                    } 
                    // Check if keystore is initialized
                    else if app.keystore.is_none() {
                        app.log.push("⚠️ Keystore failed to initialize automatically. Restart the application or check file permissions.".to_string());
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
                    
                    app.log.push(format!("Creating wallet '{}' with DKG key share from the completed session", name));
                    app.log.push("⚙️ Storing FROST threshold signature key share in your keystore...".to_string());
                    app.log.push("🔑 Password set to your device ID. Remember to back up your keystore!".to_string());
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
                        app.log.push("Invalid /locate_wallet format. Use: /locate_wallet <wallet_id>".to_string());
                    }
                } else if cmd_str.starts_with("/acceptSign") {
                    // Handle the /acceptSign command
                    let parts: Vec<_> = cmd_str.split_whitespace().collect();
                    if parts.len() == 2 {
                        let signing_id = parts[1].to_string();
                        
                        let _ = cmd_tx.send(InternalCommand::AcceptSigning {
                            signing_id: signing_id.clone(),
                        });
                        app.log.push(format!("Accepting signing request: {}", signing_id));
                    } else {
                        app.log.push("Invalid /acceptSign format. Use: /acceptSign <signing_id>".to_string());
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
                            Err(e) => {
                                app.log.push(format!("Invalid JSON for /relay: {}", e));
                            }
                        }
                    } else {
                        app.log.push(
                            "Invalid /relay format. Use: /relay <device_id> <json_data>".to_string(),
                        );
                    }
                } else if cmd_str.starts_with("/send") {
                    // This command now sends a simple text message via WebRTCMessage::SimpleMessage
                    let parts: Vec<_> = cmd_str.splitn(3, ' ').collect();
                    if parts.len() >= 3 {
                        let target_device_id = parts[1].to_string();
                        let message_text = parts[2].to_string();

                        // Always log the send attempt, regardless of connection state
                        app.log.push(format!(
                            "Attempting to send direct message to {}: {}",
                            target_device_id, message_text
                        ));

                        // Send internal command
                        let _ = cmd_tx.send(InternalCommand::SendDirect {
                            to: target_device_id.clone(),
                            message: message_text.clone(),
                        });

                        // Log the command for visibility
                        app.log.push(format!(
                            "Command: /send {} {}",
                            target_device_id, message_text
                        ));
                    } else {
                        app.log.push(
                            "Invalid /send format. Use: /send <device_id> <message>".to_string(),
                        );
                    }
                } else if cmd_str.starts_with("/offline") {
                    app.log.push("Offline mode temporarily disabled for browser compatibility focus".to_string());
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
                    //             app.log.push("Usage: /offline <on|off>".to_string());
                    //         }
                    //     }
                    // } else {
                    //     app.log.push("Usage: /offline <on|off>".to_string());
                    // }
                } else if cmd_str.starts_with("/create_signing_request") || 
                          cmd_str.starts_with("/export_signing_request") ||
                          cmd_str.starts_with("/import_signing_request") ||
                          cmd_str == "/offline_sessions" ||
                          cmd_str.starts_with("/review_signing_request") {
                    app.log.push("Offline mode commands temporarily disabled for browser compatibility focus".to_string());
                } else if !cmd_str.is_empty() {
                    app.log.push(format!("Unknown command: {}", cmd_str));
                }
            }
            KeyCode::Char(c) => {
                input.push(c);
            }
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Esc => {
                *input_mode = false;
                input.clear();
                app.log.push("Exited input mode (Esc).".to_string());
            }
            _ => {}
        }
    } else {
        // --- Normal Mode Key Handling (Add scroll keys) ---
        match key.code {
            KeyCode::Char('i') => {
                *input_mode = true;
                app.log.push("Entered input mode.".to_string());
            }
            KeyCode::Char('o') => {
                // Accept the first pending invitation
                if let Some(invite) = app.invites.first() {
                    let session_id = invite.session_id.clone();
                    let _ = cmd_tx.send(InternalCommand::AcceptSessionProposal(session_id.clone()));
                    app.log.push(format!("Accepting session proposal '{}'", session_id));
                } else {
                    app.log.push("No pending session invitations".to_string());
                }
            }
            KeyCode::Char('q') => {
                app.log.push("Quitting...".to_string());
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
                    Ok(_) => app.log.push(format!("Log saved to {}", filename)),
                    Err(e) => app.log.push(format!("Failed to save log: {}", e)),
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
                
                app.log.push(format!(
                    "Quick test: Proposed session '{}' with {} participants (threshold: {})",
                    session_id, 3, 2
                ));
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
                    app.log.push(format!("Viewing {} pending signing requests", app.pending_signing_requests.len()));
                }
            }
            KeyCode::Char('m') | KeyCode::Char('M') => {
                // Open main menu
                *ui_mode = UIMode::MainMenu { selected_index: 0 };
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                // Quick access to session proposal
                *ui_mode = UIMode::SessionProposalPopup {
                    session_name: String::new(),
                    total_participants: String::new(),
                    threshold: String::new(),
                    participants: String::new(),
                    selected_field: 0,
                };
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
                    app.log.push("No pending session invitations".to_string());
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
        }
    }
        }
    }
    Ok(true) // Continue loop by default
}
