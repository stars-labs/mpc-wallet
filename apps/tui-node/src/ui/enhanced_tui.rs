//! Enhanced TUI implementation with improved UX
//! Demonstrates the new UI components and patterns

use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, List, ListItem, Clear, Wrap},
    Frame,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crate::utils::state::{AppState, DkgState, MeshStatus};
use crate::ui::{
    theme::{ColorTheme, StatusIndicators, ThemedStyles},
    help::{ContextualHelp, GlobalShortcuts, ModeShortcuts, QuickActions},
    status::{StatusDashboard, ProgressIndicator, ErrorPanel, QuickActionsPanel},
};

/// Enhanced TUI with improved UX
pub struct EnhancedTUI {
    theme: ThemedStyles,
    status_dashboard: StatusDashboard,
    progress_indicator: ProgressIndicator,
    error_panel: ErrorPanel,
    quick_actions: QuickActionsPanel,
    show_help: bool,
    user_level: crate::ui::help::UserLevel,
}

impl EnhancedTUI {
    pub fn new() -> Self {
        Self {
            theme: ThemedStyles::default(),
            status_dashboard: StatusDashboard::new(),
            progress_indicator: ProgressIndicator::new(),
            error_panel: ErrorPanel::new(),
            quick_actions: QuickActionsPanel::new(),
            show_help: false,
            user_level: crate::ui::help::UserLevel::Beginner,
        }
    }
    
    /// Main render function with adaptive layout
    pub fn render<B: Backend, C>(
        &self,
        f: &mut Frame,
        app_state: &AppState<C>,
        ui_mode: &UIMode,
    ) {
        // Create adaptive layout based on current mode
        let chunks = self.create_adaptive_layout(f.size(), ui_mode);
        
        // Render status bar at the top
        self.status_dashboard.render_status_bar(f, chunks[0], app_state);
        
        // Render main content area
        self.render_main_content(f, chunks[1], app_state, ui_mode);
        
        // Render contextual help or quick actions
        if self.show_help {
            self.render_help_panel(f, chunks[2], ui_mode, app_state);
        } else {
            self.render_quick_actions(f, chunks[2], app_state);
        }
        
        // Render error panel if there are errors
        if let Some((error, _)) = &app_state.last_error {
            self.render_error_overlay(f, error);
        }
        
        // Render keyboard shortcuts bar at the bottom
        self.render_shortcuts_bar(f, chunks[3], ui_mode);
    }
    
    /// Create adaptive layout based on UI mode
    fn create_adaptive_layout(&self, area: Rect, ui_mode: &UIMode) -> Vec<Rect> {
        match ui_mode {
            UIMode::DKGInProgress { .. } => {
                // DKG needs more space for progress indicator
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),   // Status bar
                        Constraint::Min(15),     // Main content with progress
                        Constraint::Length(6),   // Help/Actions
                        Constraint::Length(2),   // Shortcuts bar
                    ])
                    .split(area)
            }
            UIMode::WalletList | UIMode::SessionList => {
                // List views need more vertical space
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),   // Status bar
                        Constraint::Min(20),     // List area
                        Constraint::Length(4),   // Help/Actions
                        Constraint::Length(2),   // Shortcuts bar
                    ])
                    .split(area)
            }
            _ => {
                // Default balanced layout
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),   // Status bar
                        Constraint::Min(10),     // Main content
                        Constraint::Length(5),   // Help/Actions
                        Constraint::Length(2),   // Shortcuts bar
                    ])
                    .split(area)
            }
        }
    }
    
    /// Render main content area based on UI mode
    fn render_main_content<B: Backend, C>(
        &self,
        f: &mut Frame,
        area: Rect,
        app_state: &AppState<C>,
        ui_mode: &UIMode,
    ) {
        match ui_mode {
            UIMode::Normal => self.render_home_screen(f, area, app_state),
            UIMode::DKGInProgress { .. } => self.render_dkg_progress(f, area, app_state),
            UIMode::WalletList => self.render_wallet_list(f, area, app_state),
            UIMode::SessionList => self.render_session_list(f, area, app_state),
            _ => self.render_default_content(f, area, app_state),
        }
    }
    
    /// Render home screen with dashboard
    fn render_home_screen<B: Backend, C>(
        &self,
        f: &mut Frame,
        area: Rect,
        app_state: &AppState<C>,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),  // Left sidebar
                Constraint::Percentage(70),  // Main area
            ])
            .split(area);
        
        // Left sidebar with navigation
        self.render_navigation_sidebar(f, chunks[0]);
        
        // Main area with wallet overview
        self.render_wallet_overview(f, chunks[1], app_state);
    }
    
    /// Render navigation sidebar
    fn render_navigation_sidebar<B: Backend>(&self, f: &mut Frame, area: Rect) {
        let menu_items = vec![
            ListItem::new(Line::from(vec![
                Span::raw(format!("{} ", StatusIndicators::WALLET)),
                Span::styled("Wallets", self.theme.primary()),
            ])),
            ListItem::new(Line::from(vec![
                Span::raw("📋 "),
                Span::styled("Sessions", self.theme.primary()),
            ])),
            ListItem::new(Line::from(vec![
                Span::raw(format!("{} ", StatusIndicators::KEY)),
                Span::styled("DKG", self.theme.primary()),
            ])),
            ListItem::new(Line::from(vec![
                Span::raw("✍️ "),
                Span::styled("Sign", self.theme.primary()),
            ])),
            ListItem::new(Line::from(vec![
                Span::raw("📁 "),
                Span::styled("Import/Export", self.theme.primary()),
            ])),
            ListItem::new(Line::from(vec![
                Span::raw("⚙️ "),
                Span::styled("Settings", self.theme.primary()),
            ])),
        ];
        
        let menu = List::new(menu_items)
            .block(
                Block::default()
                    .title(" Navigation ")
                    .borders(Borders::ALL)
                    .border_style(self.theme.accent())
            )
            .highlight_style(self.theme.highlight())
            .highlight_symbol("> ");
        
        f.render_widget(menu, area);
    }
    
    /// Render wallet overview
    fn render_wallet_overview<B: Backend, C>(
        &self,
        f: &mut Frame,
        area: Rect,
        app_state: &AppState<C>,
    ) {
        let content = if app_state.blockchain_addresses.is_empty() {
            vec![
                Line::from(vec![
                    Span::styled("Welcome to MPC Wallet", self.theme.accent()),
                ]),
                Line::default(),
                Line::from(vec![
                    Span::raw("No wallets configured yet."),
                ]),
                Line::default(),
                Line::from(vec![
                    Span::raw("Quick Start:"),
                ]),
                Line::from(vec![
                    Span::raw("  1. Press "),
                    Span::styled("[w]", self.theme.accent()),
                    Span::raw(" to create a wallet"),
                ]),
                Line::from(vec![
                    Span::raw("  2. Press "),
                    Span::styled("[s]", self.theme.accent()),
                    Span::raw(" to join a session"),
                ]),
                Line::from(vec![
                    Span::raw("  3. Press "),
                    Span::styled("[?]", self.theme.accent()),
                    Span::raw(" for help"),
                ]),
            ]
        } else {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Active Wallets", self.theme.accent()),
                ]),
                Line::default(),
            ];
            
            for addr_info in &app_state.blockchain_addresses {
                lines.push(Line::from(vec![
                    Span::raw(format!("{} ", StatusIndicators::KEY)),
                    Span::styled(&addr_info.blockchain, self.theme.info()),
                    Span::raw(": "),
                    Span::styled(&addr_info.address, self.theme.primary()),
                ]));
            }
            
            lines
        };
        
        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .title(" Wallet Overview ")
                    .borders(Borders::ALL)
            )
            .alignment(Alignment::Left);
        
        f.render_widget(paragraph, area);
    }
    
    /// Render DKG progress with detailed steps
    fn render_dkg_progress<B: Backend, C>(
        &self,
        f: &mut Frame,
        area: Rect,
        app_state: &AppState<C>,
    ) {
        let (current, total, steps) = match &app_state.dkg_state {
            DkgState::Idle => (0, 4, vec!["Waiting to start", "Round 1", "Round 2", "Finalize"]),
            DkgState::Round1InProgress => (1, 4, vec!["Starting", "Round 1 ⏳", "Round 2", "Finalize"]),
            DkgState::Round1Complete => (2, 4, vec!["Starting ✓", "Round 1 ✓", "Round 2", "Finalize"]),
            DkgState::Round2InProgress => (3, 4, vec!["Starting ✓", "Round 1 ✓", "Round 2 ⏳", "Finalize"]),
            DkgState::Round2Complete => (3, 4, vec!["Starting ✓", "Round 1 ✓", "Round 2 ✓", "Finalize"]),
            DkgState::Finalizing => (4, 4, vec!["Starting ✓", "Round 1 ✓", "Round 2 ✓", "Finalizing ⏳"]),
            DkgState::Complete => (4, 4, vec!["Starting ✓", "Round 1 ✓", "Round 2 ✓", "Complete ✓"]),
            DkgState::Failed(_) => (0, 4, vec!["Failed ✗", "", "", ""]),
        };
        
        self.progress_indicator.render_progress(
            f,
            area,
            "Distributed Key Generation",
            current,
            total,
            &steps,
        );
    }
    
    /// Render wallet list
    fn render_wallet_list<B: Backend, C>(
        &self,
        f: &mut Frame,
        area: Rect,
        app_state: &AppState<C>,
    ) {
        let items: Vec<ListItem> = app_state.wallet_store
            .iter()
            .map(|(name, _info)| {
                ListItem::new(Line::from(vec![
                    Span::raw(format!("{} ", StatusIndicators::WALLET)),
                    Span::styled(name, self.theme.primary()),
                ]))
            })
            .collect();
        
        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Wallets ")
                    .borders(Borders::ALL)
            )
            .highlight_style(self.theme.highlight())
            .highlight_symbol("> ");
        
        f.render_widget(list, area);
    }
    
    /// Render session list
    fn render_session_list<B: Backend, C>(
        &self,
        f: &mut Frame,
        area: Rect,
        _app_state: &AppState<C>,
    ) {
        // Mock session data for demo
        let items = vec![
            ListItem::new(Line::from(vec![
                Span::raw("🟢 "),
                Span::styled("Active DKG Session", self.theme.success()),
                Span::raw(" (2/3 participants)"),
            ])),
            ListItem::new(Line::from(vec![
                Span::raw("🟡 "),
                Span::styled("Pending Session", self.theme.warning()),
                Span::raw(" (1/2 participants)"),
            ])),
        ];
        
        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Available Sessions ")
                    .borders(Borders::ALL)
            );
        
        f.render_widget(list, area);
    }
    
    /// Render default content
    fn render_default_content<B: Backend, C>(
        &self,
        f: &mut Frame,
        area: Rect,
        _app_state: &AppState<C>,
    ) {
        let content = Paragraph::new("Content area")
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(content, area);
    }
    
    /// Render contextual help panel
    fn render_help_panel<B: Backend, C>(
        &self,
        f: &mut Frame,
        area: Rect,
        ui_mode: &UIMode,
        app_state: &AppState<C>,
    ) {
        let help_text = ContextualHelp::get_help(ui_mode, app_state);
        let help_lines: Vec<Line> = help_text
            .iter()
            .map(|text| Line::from(Span::raw(text)))
            .collect();
        
        let help_widget = Paragraph::new(help_lines)
            .block(
                Block::default()
                    .title(" Help ")
                    .borders(Borders::ALL)
                    .border_style(self.theme.info())
            )
            .wrap(Wrap { trim: true });
        
        f.render_widget(help_widget, area);
    }
    
    /// Render quick actions panel
    fn render_quick_actions<B: Backend, C>(
        &self,
        f: &mut Frame,
        area: Rect,
        app_state: &AppState<C>,
    ) {
        let actions = QuickActions::get_suggested_actions(app_state);
        self.quick_actions.render(f, area, &actions);
    }
    
    /// Render error overlay
    fn render_error_overlay<B: Backend>(&self, f: &mut Frame, error: &str) {
        let area = centered_rect(60, 20, f.size());
        
        // Clear the area first
        f.render_widget(Clear, area);
        
        // Get error suggestions
        let suggestions = crate::ui::help::ContextualHelp::get_error_help("generic");
        let errors = vec![(error.to_string(), suggestions)];
        
        self.error_panel.render(f, area, &errors);
    }
    
    /// Render keyboard shortcuts bar
    fn render_shortcuts_bar<B: Backend>(
        &self,
        f: &mut Frame,
        area: Rect,
        ui_mode: &UIMode,
    ) {
        let shortcuts = ModeShortcuts::get_formatted_shortcuts(ui_mode);
        let shortcut_widget = Paragraph::new(shortcuts)
            .style(self.theme.muted())
            .alignment(Alignment::Center);
        
        f.render_widget(shortcut_widget, area);
    }
    
    /// Handle keyboard input
    pub fn handle_input(&mut self, event: KeyEvent) -> InputResult {
        // Global shortcuts
        match (event.modifiers, event.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('?')) => {
                self.show_help = !self.show_help;
                InputResult::Handled
            }
            (KeyModifiers::CONTROL, KeyCode::Char('q')) => InputResult::Quit,
            (KeyModifiers::CONTROL, KeyCode::Char('r')) => InputResult::Refresh,
            _ => InputResult::NotHandled,
        }
    }
}

/// UI Mode enumeration (simplified for demo)
#[derive(Debug, Clone)]
pub enum UIMode {
    Normal,
    WalletList,
    SessionList,
    CreateWallet { step: usize },
    JoinSession { code: String },
    SessionLobby { session_id: String },
    DKGInProgress { round: usize },
    SignTransaction { tx_id: String },
    WalletDetails { wallet_id: String },
}

/// Input handling result
pub enum InputResult {
    Handled,
    NotHandled,
    Quit,
    Refresh,
}

/// Helper function to create centered rect
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