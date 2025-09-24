//! DKG Progress Component - Real-time DKG status display
//!
//! Professional component for displaying the progress of the Distributed Key Generation
//! process in online mode with WebRTC mesh networking.

use crate::elm::components::{Id, UserEvent, MpcWalletComponent};
use crate::elm::message::{Message, DKGRound};

use tuirealm::command::{Cmd, CmdResult};
use tuirealm::event::{Event, Key, KeyEvent};
use ratatui::layout::{Rect, Constraint, Direction, Layout, Alignment};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, BorderType, Paragraph, Gauge, List, ListItem};
use tuirealm::{Component, Frame, MockComponent, Props, State, StateValue};

/// Participant status in the DKG process
#[derive(Debug, Clone)]
pub struct ParticipantInfo {
    pub device_id: String,
    pub status: ParticipantStatus,
    pub round_progress: DKGRound,
    pub is_connected: bool,
    pub webrtc_connected: bool,  // WebRTC connection state
    pub data_channel_open: bool, // Data channel state
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParticipantStatus {
    Waiting,
    WebRTCConnecting,
    DataChannelOpen,
    MeshReady,
    Round1Complete,
    Round2Complete,
    Completed,
    Failed(String),
}

/// Professional DKG progress component
#[derive(Debug, Clone)]
pub struct DKGProgressComponent {
    props: Props,
    session_id: String,
    total_participants: u16,
    threshold: u16,
    participants: Vec<ParticipantInfo>,
    current_round: DKGRound,
    progress_percentage: f64,
    error_message: Option<String>,
    focused: bool,
    selected_action: usize, // 0 = Cancel, 1 = Copy Session ID
    websocket_connected: bool, // Track WebSocket connection status
    mesh_ready_count: usize,  // Track how many participants are mesh-ready
    all_data_channels_open: bool, // Track if all data channels are open
}

impl Default for DKGProgressComponent {
    fn default() -> Self {
        Self::new("DKG-00000000".to_string(), 3, 2)
    }
}

impl DKGProgressComponent {
    pub fn new(session_id: String, total_participants: u16, threshold: u16) -> Self {
        Self {
            props: Props::default(),
            session_id,
            total_participants,
            threshold,
            participants: vec![],
            current_round: DKGRound::Initialization,
            progress_percentage: 0.0,
            error_message: None,
            focused: false,
            selected_action: 0,
            websocket_connected: false, // Default to disconnected
            mesh_ready_count: 0,
            all_data_channels_open: false,
        }
    }
    
    /// Set WebSocket connection status
    pub fn set_websocket_connected(&mut self, connected: bool) {
        self.websocket_connected = connected;
    }
    
    /// Set selected action (0 = Cancel DKG, 1 = Copy Session ID)
    pub fn set_selected_action(&mut self, action: usize) {
        self.selected_action = action;
    }
    
    /// Update the session information
    pub fn set_session_info(&mut self, session_id: String, total: u16, threshold: u16) {
        self.session_id = session_id;
        self.total_participants = total;
        self.threshold = threshold;
    }
    
    /// Add or update a participant
    pub fn update_participant(&mut self, device_id: String, status: ParticipantStatus) {
        if let Some(participant) = self.participants.iter_mut().find(|p| p.device_id == device_id) {
            participant.status = status;
        } else {
            self.participants.push(ParticipantInfo {
                device_id,
                status,
                round_progress: DKGRound::Initialization,
                is_connected: true,
                webrtc_connected: false,
                data_channel_open: false,
            });
        }
        self.update_progress();
    }
    
    /// Update the current DKG round
    pub fn set_round(&mut self, round: DKGRound) {
        self.current_round = round;
        self.update_progress();
    }

    /// Update WebRTC connection status for a participant
    pub fn update_webrtc_status(&mut self, device_id: String, webrtc_connected: bool, data_channel_open: bool) {
        if let Some(participant) = self.participants.iter_mut().find(|p| p.device_id == device_id) {
            participant.webrtc_connected = webrtc_connected;
            participant.data_channel_open = data_channel_open;

            // Update status based on connection state
            if data_channel_open {
                participant.status = ParticipantStatus::DataChannelOpen;
            } else if webrtc_connected {
                participant.status = ParticipantStatus::WebRTCConnecting;
            }
        } else {
            // Add new participant if not exists
            self.participants.push(ParticipantInfo {
                device_id,
                status: if data_channel_open {
                    ParticipantStatus::DataChannelOpen
                } else if webrtc_connected {
                    ParticipantStatus::WebRTCConnecting
                } else {
                    ParticipantStatus::Waiting
                },
                round_progress: DKGRound::Initialization,
                is_connected: webrtc_connected || data_channel_open,
                webrtc_connected,
                data_channel_open,
            });
        }

        // Check if all data channels are open
        self.all_data_channels_open = self.participants.len() >= self.total_participants as usize &&
            self.participants.iter().all(|p| p.data_channel_open);
    }

    /// Update mesh status
    pub fn update_mesh_status(&mut self, ready_count: usize, all_connected: bool) {
        self.mesh_ready_count = ready_count;
        if all_connected {
            // Update all participants to MeshReady if they have data channels open
            for participant in &mut self.participants {
                if participant.data_channel_open {
                    participant.status = ParticipantStatus::MeshReady;
                }
            }
        }
    }
    
    /// Calculate overall progress
    fn update_progress(&mut self) {
        let connected = self.participants.len() as f64;
        let total = self.total_participants as f64;
        
        match self.current_round {
            DKGRound::Initialization => {
                // Initial setup
                self.progress_percentage = 5.0;
            }
            DKGRound::WaitingForParticipants => {
                // Progress based on participants joining
                self.progress_percentage = 5.0 + (connected / total) * 20.0;
            }
            DKGRound::Round1 => {
                // 25% base + progress through round 1
                let round1_complete = self.participants.iter()
                    .filter(|p| matches!(p.status, ParticipantStatus::Round1Complete | ParticipantStatus::Round2Complete | ParticipantStatus::Completed))
                    .count() as f64;
                self.progress_percentage = 25.0 + (round1_complete / total) * 35.0;
            }
            DKGRound::Round2 => {
                // 60% base + progress through round 2
                let round2_complete = self.participants.iter()
                    .filter(|p| matches!(p.status, ParticipantStatus::Round2Complete | ParticipantStatus::Completed))
                    .count() as f64;
                self.progress_percentage = 60.0 + (round2_complete / total) * 35.0;
            }
            DKGRound::Finalization => {
                self.progress_percentage = 95.0;
            }
        }
    }
    
    fn get_round_color(&self) -> Color {
        match self.current_round {
            DKGRound::Initialization => Color::Yellow,
            DKGRound::WaitingForParticipants => Color::Yellow,
            DKGRound::Round1 => Color::Cyan,
            DKGRound::Round2 => Color::Blue,
            DKGRound::Finalization => Color::Green,
        }
    }
    
    fn get_status_symbol(status: &ParticipantStatus) -> &'static str {
        match status {
            ParticipantStatus::Waiting => "⏳",
            ParticipantStatus::WebRTCConnecting => "🔄",
            ParticipantStatus::DataChannelOpen => "📡",
            ParticipantStatus::MeshReady => "🔗",
            ParticipantStatus::Round1Complete => "✓",
            ParticipantStatus::Round2Complete => "✓✓",
            ParticipantStatus::Completed => "✅",
            ParticipantStatus::Failed(_) => "❌",
        }
    }
    
    fn get_status_color(status: &ParticipantStatus) -> Color {
        match status {
            ParticipantStatus::Waiting => Color::Gray,
            ParticipantStatus::WebRTCConnecting => Color::Yellow,
            ParticipantStatus::DataChannelOpen => Color::Cyan,
            ParticipantStatus::MeshReady => Color::Blue,
            ParticipantStatus::Round1Complete => Color::Cyan,
            ParticipantStatus::Round2Complete => Color::Blue,
            ParticipantStatus::Completed => Color::Green,
            ParticipantStatus::Failed(_) => Color::Red,
        }
    }
}

impl MockComponent for DKGProgressComponent {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        // Check if area is too small
        if area.width < 20 || area.height < 15 {
            // Render a simple message if space is insufficient
            let msg = Paragraph::new("Window too small")
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center);
            frame.render_widget(msg, area);
            return;
        }
        
        // Main container
        let block = Block::default()
            .title(" 🔐 DKG Progress - Online Mode ")
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(if self.focused { Color::Cyan } else { Color::Gray }));
        frame.render_widget(block.clone(), area);
        
        // Create inner area for content (accounting for borders)
        let inner_area = block.inner(area);
        
        // Use more flexible constraints
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),     // Header with session info (flexible)
                Constraint::Length(3),  // Progress bar
                Constraint::Min(5),     // Participants list (flexible)
                Constraint::Min(3),     // Actions/Status (flexible)
            ])
            .margin(1)
            .split(inner_area);
        
        // Safely render each section if chunk exists
        if chunks.len() >= 4 {
            // Header - Session Information
            self.render_header(frame, chunks[0]);
            
            // Progress Bar
            self.render_progress_bar(frame, chunks[1]);
            
            // Participants List
            self.render_participants(frame, chunks[2]);
            
            // Actions/Status
            self.render_actions(frame, chunks[3]);
        } else {
            // Fallback: render simple status if layout failed
            let msg = Paragraph::new(format!("DKG Session: {}\nParticipants: {}/{}", 
                self.session_id, self.participants.len(), self.total_participants))
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center);
            frame.render_widget(msg, inner_area);
        }
    }
    
    fn query(&self, attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        self.props.get(attr)
    }
    
    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {
        self.props.set(attr, value);
    }
    
    fn state(&self) -> tuirealm::State {
        State::One(StateValue::String(self.session_id.clone()))
    }
    
    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Move(tuirealm::command::Direction::Left) => {
                if self.selected_action > 0 {
                    self.selected_action -= 1;
                    CmdResult::Changed(self.state())
                } else {
                    CmdResult::None
                }
            }
            Cmd::Move(tuirealm::command::Direction::Right) => {
                if self.selected_action < 1 {
                    self.selected_action += 1;
                    CmdResult::Changed(self.state())
                } else {
                    CmdResult::None
                }
            }
            Cmd::Submit => {
                if self.selected_action == 0 {
                    // Cancel DKG
                    CmdResult::Submit(State::One(StateValue::String("cancel".to_string())))
                } else {
                    // Copy Session ID
                    CmdResult::Submit(State::One(StateValue::String("copy".to_string())))
                }
            }
            _ => CmdResult::None,
        }
    }
}

impl DKGProgressComponent {
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
            ])
            .split(area);
        
        // Session ID with WebSocket status
        let ws_status = if self.websocket_connected {
            Span::styled("🟢 WebSocket Connected", Style::default().fg(Color::Green))
        } else {
            Span::styled("🔴 WebSocket Disconnected", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        };
        
        let session_text = vec![
            Line::from(vec![
                Span::styled("Session ID: ", Style::default().fg(Color::Gray)),
                Span::styled(&self.session_id, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("  |  "),
                ws_status,
            ]),
        ];
        let session_para = Paragraph::new(session_text)
            .alignment(Alignment::Center);
        frame.render_widget(session_para, chunks[0]);
        
        // Configuration
        let config_text = vec![
            Line::from(vec![
                Span::styled("Configuration: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}-of-{} Threshold", self.threshold, self.total_participants),
                    Style::default().fg(Color::Cyan)
                ),
            ]),
        ];
        let config_para = Paragraph::new(config_text)
            .alignment(Alignment::Center);
        frame.render_widget(config_para, chunks[1]);
        
        // Current Round
        let round_text = vec![
            Line::from(vec![
                Span::styled("Current Round: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:?}", self.current_round),
                    Style::default().fg(self.get_round_color()).add_modifier(Modifier::BOLD)
                ),
            ]),
        ];
        let round_para = Paragraph::new(round_text)
            .alignment(Alignment::Center);
        frame.render_widget(round_para, chunks[2]);
        
        // Participants Count with WebRTC details
        let data_channels_open = self.participants.iter().filter(|p| p.data_channel_open).count();
        let webrtc_connected = self.participants.iter().filter(|p| p.webrtc_connected).count();

        let participants_text = vec![
            Line::from(vec![
                Span::styled("P2P Status: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("WebRTC: {}/{} | Channels: {}/{} | Mesh: {}/{}",
                            webrtc_connected, self.total_participants,
                            data_channels_open, self.total_participants,
                            self.mesh_ready_count, self.total_participants),
                    Style::default().fg(if self.all_data_channels_open {
                        Color::Green
                    } else if data_channels_open > 0 {
                        Color::Yellow
                    } else {
                        Color::Red
                    })
                ),
            ]),
        ];
        let participants_para = Paragraph::new(participants_text)
            .alignment(Alignment::Center);
        frame.render_widget(participants_para, chunks[3]);
    }
    
    fn render_progress_bar(&self, frame: &mut Frame, area: Rect) {
        let progress_label = format!(
            "Progress: {:.0}% - {}",
            self.progress_percentage,
            match self.current_round {
                DKGRound::Initialization => "Initializing protocol...",
                DKGRound::WaitingForParticipants => "Waiting for participants...",
                DKGRound::Round1 => "Generating commitments...",
                DKGRound::Round2 => "Exchanging shares...",
                DKGRound::Finalization => "Finalizing DKG...",
            }
        );
        
        // Ensure percentage is valid (0-100) before passing to Gauge
        let safe_percentage = if self.progress_percentage.is_nan() || self.progress_percentage.is_infinite() {
            0
        } else {
            self.progress_percentage.clamp(0.0, 100.0) as u16
        };
        
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(Style::default().fg(self.get_round_color()).bg(Color::Black))
            .percent(safe_percentage)
            .label(progress_label);
        
        frame.render_widget(gauge, area);
    }
    
    fn render_participants(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Participants ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray));
        
        let items: Vec<ListItem> = self.participants
            .iter()
            .map(|p| {
                let status_symbol = Self::get_status_symbol(&p.status);
                let status_color = Self::get_status_color(&p.status);
                // Show detailed connection status
                let connection_symbol = if p.data_channel_open {
                    "🟢"  // Green circle for data channel open
                } else if p.webrtc_connected {
                    "🟡"  // Yellow circle for WebRTC connected
                } else {
                    "🔴"  // Red circle for disconnected
                };
                
                let content = Line::from(vec![
                    Span::raw(format!("  {} ", connection_symbol)),
                    Span::styled(&p.device_id, Style::default().fg(Color::White)),
                    Span::raw(" - "),
                    Span::styled(status_symbol, Style::default().fg(status_color)),
                    Span::raw(" "),
                    Span::styled(
                        match &p.status {
                            ParticipantStatus::Waiting => "Waiting".to_string(),
                            ParticipantStatus::WebRTCConnecting => "WebRTC Connecting".to_string(),
                            ParticipantStatus::DataChannelOpen => "Channel Open".to_string(),
                            ParticipantStatus::MeshReady => "Mesh Ready".to_string(),
                            ParticipantStatus::Round1Complete => "Round 1 Done".to_string(),
                            ParticipantStatus::Round2Complete => "Round 2 Done".to_string(),
                            ParticipantStatus::Completed => "Completed".to_string(),
                            ParticipantStatus::Failed(e) => format!("Failed: {}", e),
                        },
                        Style::default().fg(status_color)
                    ),
                ]);
                
                ListItem::new(content)
            })
            .collect();
        
        // Add placeholder slots for missing participants
        let mut all_items = items;
        for i in self.participants.len()..self.total_participants as usize {
            all_items.push(ListItem::new(Line::from(vec![
                Span::raw("  ⏳ "),
                Span::styled(
                    format!("Waiting for participant {}...", i + 1),
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
                ),
            ])));
        }
        
        let list = List::new(all_items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));
        
        frame.render_widget(list, area);
    }
    
    fn render_actions(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(3),
            ])
            .split(area);
        
        // Error message or status
        if let Some(ref error) = self.error_message {
            let error_text = vec![
                Line::from(vec![
                    Span::styled("⚠️ Error: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::styled(error, Style::default().fg(Color::Red)),
                ]),
            ];
            let error_para = Paragraph::new(error_text)
                .alignment(Alignment::Center);
            frame.render_widget(error_para, chunks[0]);
        } else {
            // Check WebSocket connection first
            let status_text = if !self.websocket_connected {
                "❌ WebSocket disconnected - Cannot proceed without signal server".to_string()
            } else {
                match self.current_round {
                    DKGRound::Initialization => {
                        if self.all_data_channels_open {
                            "🟢 All data channels established! Ready for DKG".to_string()
                        } else if self.participants.iter().any(|p| p.data_channel_open) {
                            "🟡 Establishing data channels...".to_string()
                        } else {
                            "📡 Establishing WebRTC connections...".to_string()
                        }
                    },
                    DKGRound::WaitingForParticipants => {
                        if self.mesh_ready_count == self.total_participants as usize {
                            "🟢 Mesh fully connected! Starting DKG...".to_string()
                        } else {
                            format!("⏳ Mesh formation: {}/{} ready", self.mesh_ready_count, self.total_participants)
                        }
                    },
                    DKGRound::Round1 => "🔄 Round 1: Generating and broadcasting commitments...".to_string(),
                    DKGRound::Round2 => "🔄 Round 2: Generating and distributing shares...".to_string(),
                    DKGRound::Finalization => "🔄 Finalizing key generation...".to_string(),
                }
            };

            let status_color = if !self.websocket_connected {
                Color::Red
            } else {
                self.get_round_color()
            };

            let status_para = Paragraph::new(status_text.as_str())
                .style(Style::default().fg(status_color))
                .alignment(Alignment::Center);
            frame.render_widget(status_para, chunks[0]);
        }
        
        // Action buttons
        let cancel_style = if self.selected_action == 0 {
            Style::default().fg(Color::Black).bg(Color::Red)
        } else {
            Style::default().fg(Color::Red)
        };
        
        let copy_style = if self.selected_action == 1 {
            Style::default().fg(Color::Black).bg(Color::Green)
        } else {
            Style::default().fg(Color::Green)
        };
        
        let actions_text = vec![
            Line::from(vec![
                Span::raw("  "),
                Span::styled(" Cancel DKG ", cancel_style),
                Span::raw("    "),
                Span::styled(" Copy Session ID ", copy_style),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("←→", Style::default().fg(Color::DarkGray)),
                Span::raw(" Switch • "),
                Span::styled("Enter", Style::default().fg(Color::DarkGray)),
                Span::raw(" Select • "),
                Span::styled("Esc", Style::default().fg(Color::DarkGray)),
                Span::raw(" Back"),
            ]),
        ];
        
        let actions_para = Paragraph::new(actions_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::DarkGray))
            );
        frame.render_widget(actions_para, chunks[1]);
    }
}

impl Component<Message, UserEvent> for DKGProgressComponent {
    fn on(&mut self, event: Event<UserEvent>) -> Option<Message> {
        tracing::debug!("🎮 DKGProgress received event: {:?}", event);
        
        match event {
            Event::Keyboard(KeyEvent {
                code: Key::Left,
                ..
            }) => {
                tracing::debug!("🎮 Left arrow key pressed in DKGProgress");
                if self.selected_action > 0 {
                    self.selected_action -= 1;
                    tracing::debug!("🎮 Changed selected_action to {}", self.selected_action);
                    // Return a dummy message to trigger render
                    Some(Message::None)
                } else {
                    None
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::Right,
                ..
            }) => {
                tracing::debug!("🎮 Right arrow key pressed in DKGProgress");
                if self.selected_action < 1 {
                    self.selected_action += 1;
                    tracing::debug!("🎮 Changed selected_action to {}", self.selected_action);
                    // Return a dummy message to trigger render
                    Some(Message::None)
                } else {
                    None
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                ..
            }) => {
                if self.selected_action == 0 {
                    // Cancel DKG
                    Some(Message::CancelDKG)
                } else {
                    // Copy Session ID to clipboard (or show notification)
                    Some(Message::ShowNotification {
                        kind: crate::elm::model::NotificationKind::Info,
                        text: format!("Session ID copied: {}", self.session_id),
                    })
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::Esc,
                ..
            }) => {
                Some(Message::NavigateBack)
            }
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

impl MpcWalletComponent for DKGProgressComponent {
    fn id(&self) -> Id {
        Id::DKGProgress
    }
    
    fn is_visible(&self) -> bool {
        true
    }
    
    fn on_focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}