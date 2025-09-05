//! Join Session Component
//!
//! Professional component for joining existing DKG or signing sessions

use crate::elm::components::{Id, UserEvent, MpcWalletComponent};
use crate::elm::message::Message;

use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::Event;
use ratatui::layout::{Rect, Constraint, Direction as LayoutDirection, Layout, Alignment};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, BorderType, Paragraph, List, ListItem, ListState, Wrap, Tabs};
use tuirealm::{Component, Frame, MockComponent, Props, State, StateValue};

/// Professional join session component
#[derive(Debug, Clone)]
pub struct JoinSessionComponent {
    props: Props,
    selected_tab: usize, // 0 = DKG, 1 = Signing
    selected_session: usize,
    focused: bool,
    sessions: Vec<SessionInfo>,
}

#[derive(Debug, Clone)]
struct SessionInfo {
    id: String,
    session_type: SessionType,
    creator: String,
    status: SessionStatus,
    participants: Vec<String>,
    required: usize,
    joined: usize,
    curve: String,
    mode: String,
    created_at: String,
    expires_in: String,
}

#[derive(Debug, Clone, PartialEq)]
enum SessionType {
    DKG,
    Signing,
}

#[derive(Debug, Clone)]
enum SessionStatus {
    Waiting,
    InProgress,
    Ready,
    Expired,
}

impl Default for JoinSessionComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl JoinSessionComponent {
    pub fn new() -> Self {
        // Mock sessions for demonstration
        let sessions = vec![
            SessionInfo {
                id: "DKG-2025-001".to_string(),
                session_type: SessionType::DKG,
                creator: "alice.eth".to_string(),
                status: SessionStatus::Waiting,
                participants: vec!["alice.eth".to_string(), "bob.eth".to_string()],
                required: 3,
                joined: 2,
                curve: "Secp256k1".to_string(),
                mode: "Online".to_string(),
                created_at: "2 mins ago".to_string(),
                expires_in: "28 mins".to_string(),
            },
            SessionInfo {
                id: "SIGN-2025-042".to_string(),
                session_type: SessionType::Signing,
                creator: "treasury.dao".to_string(),
                status: SessionStatus::Ready,
                participants: vec!["alice.eth".to_string(), "bob.eth".to_string(), "charlie.eth".to_string()],
                required: 2,
                joined: 3,
                curve: "Ed25519".to_string(),
                mode: "Online".to_string(),
                created_at: "5 mins ago".to_string(),
                expires_in: "25 mins".to_string(),
            },
        ];
        
        Self {
            props: Props::default(),
            selected_tab: 0,
            selected_session: 0,
            focused: false,
            sessions,
        }
    }
    
    fn get_filtered_sessions(&self) -> Vec<&SessionInfo> {
        self.sessions
            .iter()
            .filter(|s| {
                if self.selected_tab == 0 {
                    s.session_type == SessionType::DKG
                } else {
                    s.session_type == SessionType::Signing
                }
            })
            .collect()
    }
    
    fn get_status_color(&self, status: &SessionStatus) -> Color {
        match status {
            SessionStatus::Waiting => Color::Yellow,
            SessionStatus::InProgress => Color::Blue,
            SessionStatus::Ready => Color::Green,
            SessionStatus::Expired => Color::Red,
        }
    }
    
    fn get_status_text(&self, status: &SessionStatus) -> &str {
        match status {
            SessionStatus::Waiting => "⏳ Waiting for Participants",
            SessionStatus::InProgress => "🔄 In Progress",
            SessionStatus::Ready => "✅ Ready to Join",
            SessionStatus::Expired => "❌ Expired",
        }
    }
}

impl MockComponent for JoinSessionComponent {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(5),   // Header
                Constraint::Length(3),   // Tabs
                Constraint::Min(0),      // Content
                Constraint::Length(4),   // Footer
            ])
            .margin(1)
            .split(area);
        
        // Header
        self.render_header(frame, chunks[0]);
        
        // Tabs
        self.render_tabs(frame, chunks[1]);
        
        // Main content
        let content_chunks = Layout::default()
            .direction(LayoutDirection::Horizontal)
            .constraints([
                Constraint::Percentage(40),  // Session list
                Constraint::Percentage(60),  // Session details
            ])
            .split(chunks[2]);
        
        self.render_session_list(frame, content_chunks[0]);
        self.render_session_details(frame, content_chunks[1]);
        
        // Footer
        self.render_footer(frame, chunks[3]);
    }
    
    fn query(&self, attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        self.props.get(attr)
    }
    
    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {
        self.props.set(attr, value);
    }
    
    fn state(&self) -> tuirealm::State {
        State::One(StateValue::Usize(self.selected_session))
    }
    
    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Move(Direction::Up) => {
                if self.selected_session > 0 {
                    self.selected_session -= 1;
                }
                CmdResult::Changed(self.state())
            }
            Cmd::Move(Direction::Down) => {
                let max_sessions = self.get_filtered_sessions().len();
                if self.selected_session < max_sessions.saturating_sub(1) {
                    self.selected_session += 1;
                }
                CmdResult::Changed(self.state())
            }
            Cmd::Move(Direction::Left) => {
                self.selected_tab = 0;
                self.selected_session = 0;
                CmdResult::Changed(self.state())
            }
            Cmd::Move(Direction::Right) => {
                self.selected_tab = 1;
                self.selected_session = 0;
                CmdResult::Changed(self.state())
            }
            Cmd::Submit => CmdResult::Submit(self.state()),
            _ => CmdResult::None,
        }
    }
}

impl JoinSessionComponent {
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let header_text = vec![
            "🔗 JOIN EXISTING SESSION",
            "",
            "Participate in active DKG or signing sessions",
            "Sessions are discovered automatically via WebSocket/WebRTC",
        ];
        
        let header = Paragraph::new(header_text.join("\n"))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Session Discovery ")
                    .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            );
        frame.render_widget(header, area);
    }
    
    fn render_tabs(&self, frame: &mut Frame, area: Rect) {
        let titles = vec!["🔑 DKG Sessions", "✍️ Signing Sessions"];
        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray))
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            )
            .select(self.selected_tab);
        frame.render_widget(tabs, area);
    }
    
    fn render_session_list(&self, frame: &mut Frame, area: Rect) {
        let filtered_sessions = self.get_filtered_sessions();
        
        if filtered_sessions.is_empty() {
            let empty_msg = Paragraph::new("No active sessions found\n\nSessions will appear here when:\n• Someone creates a new session\n• You're invited to participate\n• Network discovery is active")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray))
                        .title(" Available Sessions ")
                );
            frame.render_widget(empty_msg, area);
            return;
        }
        
        let items: Vec<ListItem> = filtered_sessions
            .iter()
            .enumerate()
            .map(|(i, session)| {
                let is_selected = i == self.selected_session;
                let status_color = self.get_status_color(&session.status);
                
                let content = format!(
                    "{} {} ({})\n  {} {}/{}",
                    if is_selected { "▶" } else { " " },
                    session.id,
                    session.mode,
                    match session.status {
                        SessionStatus::Waiting => "⏳",
                        SessionStatus::InProgress => "🔄",
                        SessionStatus::Ready => "✅",
                        SessionStatus::Expired => "❌",
                    },
                    session.joined,
                    session.required
                );
                
                ListItem::new(content).style(
                    Style::default().fg(if is_selected { status_color } else { Color::Gray })
                )
            })
            .collect();
        
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray))
                    .title(" Available Sessions ")
            );
        
        let mut list_state = ListState::default();
        list_state.select(Some(self.selected_session));
        
        frame.render_stateful_widget(list, area, &mut list_state);
    }
    
    fn render_session_details(&self, frame: &mut Frame, area: Rect) {
        let filtered_sessions = self.get_filtered_sessions();
        
        if filtered_sessions.is_empty() || self.selected_session >= filtered_sessions.len() {
            let placeholder = Paragraph::new("Select a session to view details")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray))
                        .title(" Session Details ")
                );
            frame.render_widget(placeholder, area);
            return;
        }
        
        let session = filtered_sessions[self.selected_session];
        let status_color = self.get_status_color(&session.status);
        let status_text = self.get_status_text(&session.status);
        
        let details = vec![
            format!("📋 Session ID: {}", session.id),
            format!("👤 Created by: {}", session.creator),
            format!("📊 Status: {}", status_text),
            format!(""),
            format!("🔐 Configuration:"),
            format!("  • Curve: {}", session.curve),
            format!("  • Mode: {} Mode", session.mode),
            format!("  • Threshold: {}-of-{}", session.required, session.participants.len() + 1),
            format!(""),
            format!("👥 Participants ({}/{}):", session.joined, session.required),
        ];
        
        let mut full_details = details;
        for participant in &session.participants {
            full_details.push(format!("  • {}", participant));
        }
        
        full_details.extend(vec![
            format!(""),
            format!("⏰ Timing:"),
            format!("  • Created: {}", session.created_at),
            format!("  • Expires in: {}", session.expires_in),
            format!(""),
            if matches!(session.status, SessionStatus::Ready) {
                "✅ Ready to join! Press Enter to participate".to_string()
            } else if matches!(session.status, SessionStatus::Waiting) {
                format!("⏳ Waiting for {} more participant(s)", session.required - session.joined)
            } else if matches!(session.status, SessionStatus::InProgress) {
                "🔄 Session already in progress".to_string()
            } else {
                "❌ This session has expired".to_string()
            },
        ]);
        
        let details_widget = Paragraph::new(full_details.join("\n"))
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(status_color))
                    .title(format!(" {} Details ", 
                        if session.session_type == SessionType::DKG { "DKG" } else { "Signing" }
                    ))
                    .title_style(Style::default().fg(status_color).add_modifier(Modifier::BOLD))
            );
        
        frame.render_widget(details_widget, area);
    }
    
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer_text = vec![
            format!("Tab: {} | Sessions Found: {}", 
                if self.selected_tab == 0 { "DKG" } else { "Signing" },
                self.get_filtered_sessions().len()
            ),
            "".to_string(),
            "← → Switch Tabs | ↑↓ Select Session | Enter: Join | Esc: Back".to_string(),
            "💡 Sessions expire after 30 minutes of inactivity".to_string(),
        ];
        
        let footer = Paragraph::new(footer_text.join("\n"))
            .style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::ITALIC)
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::DarkGray))
            );
        frame.render_widget(footer, area);
    }
}

impl Component<Message, UserEvent> for JoinSessionComponent {
    fn on(&mut self, event: Event<UserEvent>) -> Option<Message> {
        match event {
            Event::User(UserEvent::FocusGained) => {
                self.focused = true;
                None
            }
            Event::User(UserEvent::FocusLost) => {
                self.focused = false;
                None
            }
            _ => None,
        }
    }
}

impl MpcWalletComponent for JoinSessionComponent {
    fn id(&self) -> Id {
        Id::CreateWallet
    }
    
    fn is_visible(&self) -> bool {
        true
    }
    
    fn on_focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}