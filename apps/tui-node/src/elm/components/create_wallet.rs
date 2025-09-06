//! Enhanced Create Wallet Component with tui-realm-stdlib
//!
//! Professional-grade UI component for wallet creation using stdlib components

use crate::elm::components::{Id, UserEvent, MpcWalletComponent};
use crate::elm::message::Message;

use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::Event;
use ratatui::layout::{Rect, Constraint, Direction as LayoutDirection, Layout, Alignment};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Borders, BorderType, Block, Paragraph, ListItem, List, ListState};
use tuirealm::{Component, Frame, MockComponent, Props, State, StateValue};

/// Enhanced Create Wallet component using stdlib styling
#[derive(Debug, Clone)]
pub struct CreateWalletComponent {
    props: Props,
    selected: usize,
    focused: bool,
}

#[derive(Debug, Clone)]
struct WalletStep {
    icon: &'static str,
    title: &'static str,
    description: &'static str,
    enabled: bool,
    completed: bool,
}

impl Default for CreateWalletComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl CreateWalletComponent {
    pub fn new() -> Self {
        let props = Props::default();
        
        Self {
            props,
            selected: 0,
            focused: false,
        }
    }
    
    /// Set the selected index
    pub fn set_selected(&mut self, index: usize) {
        let old_selected = self.selected;
        self.selected = index.min(3); // 4 items (0-3)
        tracing::debug!("🎯 CreateWalletComponent::set_selected: {} -> {}", old_selected, self.selected);
    }
    
    fn get_wallet_steps(&self) -> Vec<WalletStep> {
        vec![
            WalletStep {
                icon: "🌐",
                title: "Choose Operation Mode",
                description: "Select Online (WebRTC mesh) or Offline (air-gapped) mode",
                enabled: true,
                completed: false,
            },
            WalletStep {
                icon: "🔐",
                title: "Select Cryptographic Curve",
                description: "Choose Secp256k1 (Ethereum/Bitcoin) or Ed25519 (Solana)",
                enabled: true,
                completed: false,
            },
            WalletStep {
                icon: "⚙️",
                title: "Configure Threshold Parameters",
                description: "Set participant threshold (e.g., 2-of-3, 3-of-5) for signatures",
                enabled: true,
                completed: false,
            },
            WalletStep {
                icon: "🚀",
                title: "Initialize DKG Process",
                description: "Start Distributed Key Generation with other participants",
                enabled: true,
                completed: false,
            },
        ]
    }
}

impl MockComponent for CreateWalletComponent {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        tracing::debug!("🎨 CreateWalletComponent::view - rendering with selected: {}, focused: {}", 
                       self.selected, self.focused);
        
        // Create professional layout
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(5),    // Header with title and subtitle
                Constraint::Min(0),       // Main content area
                Constraint::Length(4),    // Footer with progress and controls
            ])
            .margin(1)
            .split(area);
        
        // Enhanced header section
        self.render_header(frame, chunks[0]);
        
        // Main content with wallet steps
        self.render_steps(frame, chunks[1]);
        
        // Enhanced footer with progress and controls
        self.render_footer(frame, chunks[2]);
    }
    
    fn query(&self, attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        self.props.get(attr)
    }
    
    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {
        self.props.set(attr, value);
    }
    
    fn state(&self) -> tuirealm::State {
        State::One(StateValue::Usize(self.selected))
    }
    
    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Move(Direction::Up) => {
                if self.selected > 0 {
                    self.selected -= 1;
                } else {
                    self.selected = 3; // Wrap to bottom
                }
                CmdResult::Changed(self.state())
            }
            Cmd::Move(Direction::Down) => {
                if self.selected < 3 {
                    self.selected += 1;
                } else {
                    self.selected = 0; // Wrap to top
                }
                CmdResult::Changed(self.state())
            }
            Cmd::Submit => {
                CmdResult::Submit(self.state())
            }
            _ => CmdResult::None,
        }
    }
}

impl CreateWalletComponent {
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let header_chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(2),
            ])
            .split(area);
        
        // Main title
        let title = Paragraph::new("🏦 MPC Wallet Setup Wizard")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Create New Wallet ")
                    .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            );
        frame.render_widget(title, header_chunks[0]);
        
        // Subtitle
        let subtitle = Paragraph::new("Professional Multi-Party Computation Wallet Creation")
            .style(Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))
            .alignment(Alignment::Center);
        frame.render_widget(subtitle, header_chunks[1]);
    }
    
    fn render_steps(&self, frame: &mut Frame, area: Rect) {
        let steps = self.get_wallet_steps();
        
        let items: Vec<ListItem> = steps
            .iter()
            .enumerate()
            .map(|(i, step)| {
                let is_selected = i == self.selected;
                let is_current = i <= self.selected;
                
                // Create status indicator
                let status_icon = if step.completed {
                    "✅"
                } else if is_selected {
                    "▶️"
                } else if is_current {
                    "🔵"
                } else {
                    "⚪"
                };
                
                // Create step content
                let content = if is_selected {
                    // Show expanded view for selected item
                    format!(
                        "{} {} {}  {}\n      └─ {}",
                        status_icon,
                        step.icon,
                        step.title,
                        if step.enabled { "" } else { "(Coming Soon)" },
                        step.description
                    )
                } else {
                    // Show compact view for non-selected items
                    format!(
                        "{} {} {}",
                        status_icon,
                        step.icon,
                        step.title
                    )
                };
                
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if is_current {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                
                ListItem::new(content).style(style)
            })
            .collect();
        
        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Wallet Creation Steps ")
                    .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(if self.focused {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Gray)
                    })
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            );
        
        let mut list_state = ListState::default();
        list_state.select(Some(self.selected));
        
        frame.render_stateful_widget(list, area, &mut list_state);
    }
    
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer_chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(2),
            ])
            .split(area);
        
        // Progress indicator
        let progress = format!("Progress: Step {} of 4", self.selected + 1);
        let progress_widget = Paragraph::new(progress)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);
        frame.render_widget(progress_widget, footer_chunks[0]);
        
        // Current step info
        let steps = self.get_wallet_steps();
        let current_step = &steps[self.selected];
        let step_info = format!("Current: {} {}", current_step.icon, current_step.title);
        let step_widget = Paragraph::new(step_info)
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(step_widget, footer_chunks[1]);
        
        // Controls
        let controls = if self.focused {
            "🎮 ↑↓ Navigate Steps • Enter: Continue • Esc: Return to Main Menu"
        } else {
            "💡 Press Tab to focus • Professional MPC Wallet Creation"
        };
        
        let controls_widget = Paragraph::new(controls)
            .style(
                Style::default()
                    .fg(if self.focused { Color::Green } else { Color::Gray })
                    .add_modifier(Modifier::ITALIC)
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::DarkGray))
            );
        frame.render_widget(controls_widget, footer_chunks[2]);
    }
}

impl Component<Message, UserEvent> for CreateWalletComponent {
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
            _ => {
                // All key handling is now done at the app level - KISS!
                None
            }
        }
    }
}

impl MpcWalletComponent for CreateWalletComponent {
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