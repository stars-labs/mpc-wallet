//! Enhanced status display components for the TUI
//! Provides rich status indicators, progress bars, and information panels

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Gauge, List, ListItem},
    Frame,
};
use crate::utils::appstate_compat::AppState;
use crate::utils::state::{DkgState, MeshStatus};
use crate::ui::theme::{StatusIndicators, ThemedStyles, create_progress_bar};
use std::time::Instant;
use frost_core::Ciphersuite;

/// Rich status dashboard component
pub struct StatusDashboard {
    theme: ThemedStyles,
}

impl StatusDashboard {
    pub fn new() -> Self {
        Self {
            theme: ThemedStyles::default(),
        }
    }
    
    /// Render compact status bar at the top of the screen
    pub fn render_status_bar<C: Ciphersuite>(
        &self,
        f: &mut Frame,
        area: Rect,
        app_state: &AppState<C>,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(20),  // Network status
                Constraint::Length(25),  // DKG status
                Constraint::Length(20),  // Mesh status
                Constraint::Min(20),      // Wallet info
                Constraint::Length(15),  // Time
            ])
            .split(area);
        
        // Network status
        let network_status = self.format_network_status(app_state);
        let network_widget = Paragraph::new(network_status)
            .style(self.theme.primary())
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(network_widget, chunks[0]);
        
        // DKG status
        let dkg_status = self.format_dkg_status(&app_state.dkg_state);
        let dkg_widget = Paragraph::new(dkg_status)
            .style(self.theme.primary())
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(dkg_widget, chunks[1]);
        
        // Mesh status
        let mesh_status = self.format_mesh_status(&app_state.mesh_status, app_state);
        let mesh_widget = Paragraph::new(mesh_status)
            .style(self.theme.primary())
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(mesh_widget, chunks[2]);
        
        // Wallet info
        let wallet_info = self.format_wallet_info(app_state);
        let wallet_widget = Paragraph::new(wallet_info)
            .style(self.theme.primary())
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(wallet_widget, chunks[3]);
        
        // Current time
        let time = chrono::Local::now().format("%H:%M:%S").to_string();
        let time_widget = Paragraph::new(format!("🕐 {}", time))
            .style(self.theme.muted())
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(time_widget, chunks[4]);
    }
    
    fn format_network_status<C: Ciphersuite>(&self, app_state: &AppState<C>) -> Line<'static> {
        let (indicator, text, style) = if app_state.websocket_connected {
            (StatusIndicators::CONNECTED, "Connected", self.theme.success())
        } else if app_state.websocket_connecting {
            (StatusIndicators::CONNECTING, "Connecting", self.theme.warning())
        } else {
            (StatusIndicators::DISCONNECTED, "Offline", self.theme.error())
        };
        
        Line::from(vec![
            Span::raw(format!("{} ", StatusIndicators::NETWORK)),
            Span::raw(format!("{} ", indicator)),
            Span::styled(text, style),
        ])
    }
    
    fn format_dkg_status(&self, dkg_state: &DkgState) -> Line<'static> {
        let (indicator, text, style) = match dkg_state {
            DkgState::Idle => ("⚪", "Ready", self.theme.muted()),
            DkgState::Round1InProgress => (StatusIndicators::IN_PROGRESS, "DKG Round 1", self.theme.warning()),
            DkgState::Round1Complete => ("🟡", "Round 1 Done", self.theme.info()),
            DkgState::Round2InProgress => (StatusIndicators::IN_PROGRESS, "DKG Round 2", self.theme.warning()),
            DkgState::Round2Complete => ("🟡", "Round 2 Done", self.theme.info()),
            DkgState::Finalizing => (StatusIndicators::IN_PROGRESS, "Finalizing", self.theme.warning()),
            DkgState::Complete => (StatusIndicators::SUCCESS, "Complete", self.theme.success()),
            DkgState::Failed(_) => (StatusIndicators::FAILED, "Failed", self.theme.error()),
        };
        
        Line::from(vec![
            Span::raw("DKG: "),
            Span::raw(format!("{} ", indicator)),
            Span::styled(text, style),
        ])
    }
    
    fn format_mesh_status<C: Ciphersuite>(&self, mesh_status: &MeshStatus, _app_state: &AppState<C>) -> Line<'static> {
        let (indicator, text, style) = match mesh_status {
            MeshStatus::Ready => (StatusIndicators::MESH_READY, "Ready".to_string(), self.theme.success()),
            MeshStatus::PartiallyReady { ready_devices, total_devices } => {
                let text = format!("{}/{}", ready_devices.len(), total_devices);
                (StatusIndicators::MESH_PARTIAL, text, self.theme.warning())
            },
            MeshStatus::WebRTCInitiated => {
                ("🔄", "Initiating".to_string(), self.theme.warning())
            },
            MeshStatus::Incomplete => {
                ("⛓", "Connecting".to_string(), self.theme.muted())
            },
        };
        
        Line::from(vec![
            Span::raw("Mesh: "),
            Span::raw(format!("{} ", indicator)),
            Span::styled(text, style),
        ])
    }
    
    fn format_wallet_info<C: Ciphersuite>(&self, app_state: &AppState<C>) -> Line<'static> {
        if let Some(wallet) = app_state.selected_wallet.as_ref() {
            Line::from(vec![
                Span::raw(format!("{} ", StatusIndicators::WALLET)),
                Span::styled(wallet.clone(), self.theme.accent()),
            ])
        } else if !app_state.blockchain_addresses.is_empty() {
            Line::from(vec![
                Span::raw(format!("{} ", StatusIndicators::KEY)),
                Span::styled("Keys Ready", self.theme.success()),
            ])
        } else {
            Line::from(vec![
                Span::raw(format!("{} ", StatusIndicators::WALLET)),
                Span::styled("No Wallet", self.theme.muted()),
            ])
        }
    }
}

/// Progress indicator for multi-step operations
pub struct ProgressIndicator {
    theme: ThemedStyles,
}

impl ProgressIndicator {
    pub fn new() -> Self {
        Self {
            theme: ThemedStyles::default(),
        }
    }
    
    /// Render a progress bar for DKG or other multi-step operations
    pub fn render_progress(
        &self,
        f: &mut Frame,
        area: Rect,
        title: &str,
        current_step: usize,
        total_steps: usize,
        step_descriptions: &[&str],
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title and progress bar
                Constraint::Min(0),     // Step descriptions
            ])
            .split(area);
        
        // Calculate percentage
        let percentage = if total_steps > 0 {
            (current_step as f64 / total_steps as f64 * 100.0) as u16
        } else {
            0
        };
        
        // Create gauge
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(format!("{} ({}/{})", title, current_step, total_steps))
                    .borders(Borders::ALL)
            )
            .gauge_style(self.theme.accent())
            .percent(percentage)
            .label(format!("{}%", percentage));
        
        f.render_widget(gauge, chunks[0]);
        
        // Render step descriptions
        if !step_descriptions.is_empty() && chunks[1].height > 0 {
            let mut items = Vec::new();
            for (i, desc) in step_descriptions.iter().enumerate() {
                let (prefix, style) = if i < current_step {
                    (StatusIndicators::SUCCESS, self.theme.success())
                } else if i == current_step {
                    (StatusIndicators::IN_PROGRESS, self.theme.warning())
                } else {
                    ("⚪", self.theme.muted())
                };
                
                items.push(ListItem::new(format!("{} {}", prefix, desc)).style(style));
            }
            
            let list = List::new(items)
                .block(Block::default().borders(Borders::NONE));
            
            f.render_widget(list, chunks[1]);
        }
    }
    
    /// Render inline progress bar as text
    pub fn render_inline_progress(
        &self,
        current: usize,
        total: usize,
        width: usize,
    ) -> String {
        create_progress_bar(current, total, width)
    }
}

/// Error panel for displaying errors with recovery suggestions
pub struct ErrorPanel {
    theme: ThemedStyles,
}

impl ErrorPanel {
    pub fn new() -> Self {
        Self {
            theme: ThemedStyles::default(),
        }
    }
    
    pub fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        errors: &[(String, Vec<String>)],
    ) {
        if errors.is_empty() {
            return;
        }
        
        let mut text_lines = Vec::new();
        
        for (error, suggestions) in errors {
            // Error message
            text_lines.push(Line::from(vec![
                Span::raw(format!("{} ", StatusIndicators::FAILED)),
                Span::styled(error, self.theme.error()),
            ]));
            
            // Recovery suggestions
            for suggestion in suggestions {
                text_lines.push(Line::from(vec![
                    Span::raw("  → "),
                    Span::styled(suggestion, self.theme.info()),
                ]));
            }
            
            text_lines.push(Line::default()); // Empty line between errors
        }
        
        let error_widget = Paragraph::new(text_lines)
            .block(
                Block::default()
                    .title(" Issues & Notifications ")
                    .borders(Borders::ALL)
                    .border_style(self.theme.error())
            )
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        f.render_widget(error_widget, area);
    }
}

/// Quick actions panel
pub struct QuickActionsPanel {
    theme: ThemedStyles,
}

impl QuickActionsPanel {
    pub fn new() -> Self {
        Self {
            theme: ThemedStyles::default(),
        }
    }
    
    pub fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        actions: &[(String, String)],
    ) {
        let action_text: Vec<Span> = actions
            .iter()
            .flat_map(|(key, desc)| {
                vec![
                    Span::styled(format!("[{}]", key), self.theme.accent()),
                    Span::raw(format!(" {}  ", desc)),
                ]
            })
            .collect();
        
        let widget = Paragraph::new(Line::from(action_text))
            .block(
                Block::default()
                    .title(" Quick Actions ")
                    .borders(Borders::ALL)
            )
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        f.render_widget(widget, area);
    }
}

/// Session timer display
pub struct SessionTimer;

impl SessionTimer {
    pub fn format_duration(start_time: Option<Instant>) -> String {
        match start_time {
            Some(start) => {
                let elapsed = start.elapsed();
                let minutes = elapsed.as_secs() / 60;
                let seconds = elapsed.as_secs() % 60;
                format!("⏱️ {}:{:02}", minutes, seconds)
            }
            None => "⏱️ --:--".to_string(),
        }
    }
}