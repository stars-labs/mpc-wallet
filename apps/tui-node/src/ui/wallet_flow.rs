//! UI components for the wallet creation flow
//!
//! This module implements the UI screens and interactions for the complete
//! wallet creation workflow as documented in the wireframes.

use crate::utils::appstate_compat::AppState;
use crate::handlers::session_handler::WalletSessionConfig;
use crate::protocal::session_types::SessionAnnouncement;
use frost_core::Ciphersuite;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear, Gauge},
};

/// Wallet creation mode selection UI
pub fn draw_wallet_creation_mode_popup<B: ratatui::backend::Backend, C: Ciphersuite>(
    f: &mut Frame,
    _app: &AppState<C>,
    selected_index: usize,
) {
    let popup_area = centered_rect(80, 70, f.area());
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Select Wallet Creation Mode ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
    
    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title/Description
            Constraint::Min(10),   // Mode options
            Constraint::Length(3), // Instructions
        ])
        .split(inner);

    // Description
    let desc = Paragraph::new("Choose how you want to create your wallet:")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center);
    f.render_widget(desc, chunks[0]);

    // Mode options
    let modes = vec![
        ("🌐 Online Mode", "Real-time coordination via WebRTC mesh", "Fastest and most convenient"),
        ("🔒 Offline Mode", "Air-gapped coordination via file exchange", "Maximum security for high-value wallets"),
        ("🔀 Hybrid Mode", "Online coordination, offline key generation", "Balance of convenience and security"),
    ];

    let items: Vec<ListItem> = modes.iter().enumerate().map(|(i, (title, subtitle, note))| {
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
                Span::styled(format!("    {}", subtitle), style),
            ]),
            Line::from(vec![
                Span::styled(format!("    💡 {}", note), style.fg(Color::Gray)),
            ]),
            Line::from(""),
        ])
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Coordination Mode"))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));
    f.render_widget(list, chunks[1]);

    // Instructions
    let instructions = Paragraph::new("↑↓: Navigate  Enter: Select  Esc: Back")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(instructions, chunks[2]);
}

/// Curve selection UI for wallet creation
pub fn draw_curve_selection_popup<B: ratatui::backend::Backend, C: Ciphersuite>(
    f: &mut Frame,
    _app: &AppState<C>,
    selected_index: usize,
) {
    let popup_area = centered_rect(70, 60, f.area());
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Select Cryptographic Curve ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    
    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2), // Warning
            Constraint::Min(8),    // Curve options
            Constraint::Length(3), // Instructions
        ])
        .split(inner);

    // Warning
    let warning = Paragraph::new("⚠️  This choice cannot be changed after wallet creation")
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center);
    f.render_widget(warning, chunks[0]);

    // Curve options
    let curves = vec![
        ("secp256k1", "Ethereum, Bitcoin, BSC, Polygon", "Most widely supported", true),
        ("ed25519", "Solana, Near, Polkadot", "Faster, more efficient", false),
    ];

    let items: Vec<ListItem> = curves.iter().enumerate().map(|(i, (curve, chains, note, recommended))| {
        let style = if i == selected_index {
            Style::default().bg(Color::Green).fg(Color::White)
        } else {
            Style::default()
        };

        let prefix = if *recommended { "⭐ " } else { "   " };

        ListItem::new(vec![
            Line::from(vec![
                Span::styled(format!("{}{}", prefix, curve), style.add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(format!("  Used by: {}", chains), style),
            ]),
            Line::from(vec![
                Span::styled(format!("  💡 {}", note), style.fg(Color::Gray)),
            ]),
            Line::from(""),
        ])
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Available Curves"));
    f.render_widget(list, chunks[1]);

    // Instructions
    let instructions = Paragraph::new("↑↓: Navigate  Enter: Select  Esc: Back")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(instructions, chunks[2]);
}

/// Wallet configuration screen for custom setup
pub fn draw_wallet_configuration_popup<B: ratatui::backend::Backend, C: Ciphersuite>(
    f: &mut Frame,
    _app: &AppState<C>,
    config: &WalletSessionConfig,
    selected_field: usize,
) {
    let popup_area = centered_rect(80, 80, f.area());
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Custom Wallet Configuration ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    
    let inner = block.inner(popup_area);
    f.render_widget(block, inner);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(8),  // Basic config
            Constraint::Length(6),  // Threshold config
            Constraint::Length(6),  // Advanced options
            Constraint::Min(4),     // Blockchain selection
            Constraint::Length(3),  // Instructions
        ])
        .split(inner);

    // Basic Configuration
    let basic_fields = vec![
        format!("Wallet Name: {}", config.wallet_name),
        format!("Description: {}", config.description.as_deref().unwrap_or("None")),
        format!("Curve Type: {}", config.curve_type),
        format!("Mode: {:?}", config.mode),
    ];

    let basic_items: Vec<ListItem> = basic_fields.iter().enumerate().map(|(i, field)| {
        let style = if i == selected_field {
            Style::default().bg(Color::Cyan).fg(Color::White)
        } else {
            Style::default()
        };
        ListItem::new(field.clone()).style(style)
    }).collect();

    let basic_list = List::new(basic_items)
        .block(Block::default().borders(Borders::ALL).title("Basic Configuration"));
    f.render_widget(basic_list, chunks[0]);

    // Threshold Configuration
    let threshold_text = vec![
        Line::from(format!("Total Participants: {} ↕", config.total)),
        Line::from(format!("Required Signatures: {} ↕", config.threshold)),
        Line::from(""),
        Line::from(Span::styled(
            format!("Security: {}-of-{} threshold scheme", config.threshold, config.total),
            Style::default().fg(Color::Green)
        )),
    ];

    let threshold_para = Paragraph::new(threshold_text)
        .block(Block::default().borders(Borders::ALL).title("Threshold Settings"));
    f.render_widget(threshold_para, chunks[1]);

    // Advanced Options
    let advanced_text = vec![
        Line::from(format!("⏰ Session Timeout: {} hours", config.timeout_hours)),
        Line::from(format!("🔍 Auto Discovery: {}", if config.auto_discovery { "Enabled" } else { "Disabled" })),
        Line::from(""),
    ];

    let advanced_para = Paragraph::new(advanced_text)
        .block(Block::default().borders(Borders::ALL).title("Advanced Options"));
    f.render_widget(advanced_para, chunks[2]);

    // Blockchain Selection
    let blockchain_items: Vec<ListItem> = config.blockchain_config.iter().map(|bc| {
        let status = if bc.enabled { "✅" } else { "⬜" };
        let chain_info = match bc.chain_id {
            Some(id) => format!("{} {} (Chain ID: {})", status, bc.blockchain, id),
            None => format!("{} {}", status, bc.blockchain),
        };
        ListItem::new(chain_info)
    }).collect();

    let blockchain_list = List::new(blockchain_items)
        .block(Block::default().borders(Borders::ALL).title("Blockchain Support"));
    f.render_widget(blockchain_list, chunks[3]);

    // Instructions
    let instructions = Paragraph::new("↑↓: Navigate  Enter: Edit  Space: Toggle  Esc: Back  S: Save")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(instructions, chunks[4]);
}

/// Session discovery and joining screen
pub fn draw_session_discovery_popup<B: ratatui::backend::Backend>(
    f: &mut Frame,
    sessions: &[SessionAnnouncement],
    selected_index: usize,
    filter_text: &str,
) {
    let popup_area = centered_rect(90, 80, f.area());
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Discover Wallet Sessions ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    
    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Filter input
            Constraint::Length(2),  // Status line
            Constraint::Min(10),    // Session list
            Constraint::Length(4),  // Details
            Constraint::Length(3),  // Instructions
        ])
        .split(inner);

    // Filter input
    let filter_para = Paragraph::new(format!("Filter: {}", filter_text))
        .block(Block::default().borders(Borders::ALL).title("Search"))
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(filter_para, chunks[0]);

    // Status line
    let status_text = if sessions.is_empty() {
        "🔍 No sessions found - try refreshing or check your network connection"
    } else {
        &format!("📋 Found {} available wallet sessions", sessions.len())
    };
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(if sessions.is_empty() { Color::Red } else { Color::Green }))
        .alignment(Alignment::Center);
    f.render_widget(status, chunks[1]);

    // Session list
    if !sessions.is_empty() {
        let items: Vec<ListItem> = sessions.iter().enumerate().map(|(i, session)| {
            let style = if i == selected_index {
                Style::default().bg(Color::Magenta).fg(Color::White)
            } else {
                Style::default()
            };

            let status_color = match session.status_string().as_str() {
                s if s.starts_with("Open") => Color::Green,
                "Full" => Color::Yellow,
                "Expired" => Color::Red,
                _ => Color::Gray,
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!("🔑 {} ", session.wallet_name), style.add_modifier(Modifier::BOLD)),
                    Span::styled(format!("({})", session.status_string()), Style::default().fg(status_color)),
                ]),
                Line::from(vec![
                    Span::styled(format!("    {} | {}-of-{} | {} | {}", 
                        session.curve_type, 
                        session.threshold, 
                        session.total,
                        session.mode,
                        session.time_remaining()
                    ), style.fg(Color::Gray)),
                ]),
                Line::from(vec![
                    Span::styled(format!("    Created by: {} | Chains: {}", 
                        session.creator_device,
                        session.blockchain_support.join(", ")
                    ), style.fg(Color::Gray)),
                ]),
                Line::from(""),
            ])
        }).collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Available Sessions"));
        f.render_widget(list, chunks[2]);

        // Session details
        if let Some(session) = sessions.get(selected_index) {
            let details_text = vec![
                Line::from(format!("Session ID: {}", session.session_id)),
                Line::from(format!("Description: {}", session.description.as_deref().unwrap_or("None"))),
                Line::from(format!("Participants: {}/{} | Requires Approval: {}", 
                    session.participants_joined, session.total, 
                    if session.requires_approval { "Yes" } else { "No" })),
            ];

            let details = Paragraph::new(details_text)
                .block(Block::default().borders(Borders::ALL).title("Session Details"));
            f.render_widget(details, chunks[3]);
        }
    } else {
        // Empty state
        let empty_msg = Paragraph::new("🔍 Searching for wallet creation sessions...\n\nMake sure you're connected to the network and try refreshing.")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("No Sessions Found"));
        f.render_widget(empty_msg, chunks[2]);
    }

    // Instructions
    let instructions = Paragraph::new("↑↓: Navigate  Enter: Join  R: Refresh  /: Filter  Esc: Back")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(instructions, chunks[4]);
}

/// DKG Progress display screen
pub fn draw_dkg_progress_screen<B: ratatui::backend::Backend, C: Ciphersuite>(
    f: &mut Frame,
    app: &AppState<C>,
) {
    let area = f.area();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(6),  // Progress bar and stage
            Constraint::Length(8),  // Participant status
            Constraint::Min(8),     // Progress details/log
            Constraint::Length(3),  // Instructions
        ])
        .split(area);

    // Title
    let wallet_name = app.wallet_creation_config.as_ref()
        .map(|c| c.wallet_name.as_str())
        .unwrap_or("Wallet");
    let title = Paragraph::new(format!("🔑 Creating Wallet: {}", wallet_name))
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Progress bar and stage
    if let Some(progress) = &app.wallet_creation_progress {
        let progress_ratio = progress.current_step as f64 / progress.total_steps as f64;
        let progress_bar = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Overall Progress"))
            .gauge_style(Style::default().fg(Color::Green))
            .ratio(progress_ratio)
            .label(format!("{} ({}/{})", progress.message, progress.current_step, progress.total_steps));
        f.render_widget(progress_bar, chunks[1]);
    }

    // Participant status
    if let Some(session) = &app.session {
        let participant_items: Vec<ListItem> = session.participants.iter().map(|device_id| {
            let status = if device_id == &app.device_id {
                "✅ You (Ready)"
            } else if app.device_statuses.contains_key(device_id) {
                match app.device_statuses[device_id] {
                    webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected => "✅ Connected",
                    webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connecting => "🔄 Connecting",
                    _ => "⏳ Pending",
                }
            } else {
                // Show device ID with waiting status instead of "Unknown"
                "⏳ Waiting"
            };

            ListItem::new(format!("👤 {} - {}", device_id, status))
        }).collect();

        let participant_list = List::new(participant_items)
            .block(Block::default().borders(Borders::ALL).title("Participants"));
        f.render_widget(participant_list, chunks[2]);
    }

    // Progress details from log
    let recent_logs: Vec<Line> = app.log.iter()
        .rev()
        .take(8)
        .rev()
        .map(|entry| Line::from(entry.clone()))
        .collect();

    let log_para = Paragraph::new(recent_logs)
        .block(Block::default().borders(Borders::ALL).title("Progress Log"))
        .scroll((0, 0));
    f.render_widget(log_para, chunks[3]);

    // Instructions
    let instructions = match &app.dkg_state {
        crate::utils::state::DkgState::Complete => {
            "✅ Wallet created successfully! Press 'q' to return to main menu or 'v' to view wallet details"
        }
        crate::utils::state::DkgState::Failed(_) => {
            "❌ Wallet creation failed. Press 'r' to retry or 'q' to return to main menu"
        }
        _ => {
            "🔄 Wallet creation in progress... Press 'q' to cancel (not recommended)"
        }
    };

    let instr_para = Paragraph::new(instructions)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instr_para, chunks[4]);
}

/// Helper function to create centered popup rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let centered = centered_rect(50, 50, area);
        
        assert_eq!(centered.width, 50);
        assert_eq!(centered.height, 25);
        assert_eq!(centered.x, 25);
        assert_eq!(centered.y, 12);
    }
}