//! Mode Selection Component - Online vs Offline
//!
//! Professional component explaining the differences between online and offline modes

use crate::elm::components::{Id, UserEvent, MpcWalletComponent};
use crate::elm::message::Message;

use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::Event;
use ratatui::layout::{Rect, Constraint, Direction as LayoutDirection, Layout, Alignment};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, BorderType, Paragraph, Wrap};
use tuirealm::{Component, Frame, MockComponent, Props, State, StateValue};

/// Professional mode selection component
#[derive(Debug, Clone)]
pub struct ModeSelectionComponent {
    props: Props,
    selected: usize,
    focused: bool,
}

#[derive(Debug, Clone)]
struct OperationMode {
    name: &'static str,
    icon: &'static str,
    security_level: &'static str,
    speed: &'static str,
    requirements: Vec<&'static str>,
    use_cases: Vec<&'static str>,
    pros: Vec<&'static str>,
    cons: Vec<&'static str>,
}

impl Default for ModeSelectionComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl ModeSelectionComponent {
    pub fn new() -> Self {
        Self {
            props: Props::default(),
            selected: 0,
            focused: false,
        }
    }
    
    fn get_modes(&self) -> Vec<OperationMode> {
        vec![
            OperationMode {
                name: "Online Mode (Hot Wallet)",
                icon: "🌐",
                security_level: "High Security",
                speed: "Real-time Operations",
                requirements: vec![
                    "• Active internet connection",
                    "• WebSocket server access (wss://auto-life.tech)",
                    "• WebRTC capability for P2P mesh",
                    "• TLS 1.3 encryption support",
                ],
                use_cases: vec![
                    "• Daily trading operations",
                    "• Quick transaction signing",
                    "• Real-time DKG ceremonies",
                    "• Instant participant coordination",
                ],
                pros: vec![
                    "✅ Instant key generation (< 30 seconds)",
                    "✅ Real-time participant discovery",
                    "✅ Automatic session synchronization",
                    "✅ Live status updates",
                    "✅ Convenient for regular operations",
                ],
                cons: vec![
                    "⚠️ Requires network connectivity",
                    "⚠️ Vulnerable to network-level attacks",
                    "⚠️ Trust in signaling infrastructure needed",
                    "⚠️ Not suitable for highest-value assets",
                ],
            },
            OperationMode {
                name: "Offline Mode (Cold Wallet)",
                icon: "🔒",
                security_level: "Maximum Security",
                speed: "Manual Coordination",
                requirements: vec![
                    "• Air-gapped machines (network disabled)",
                    "• Removable storage media (SD cards/USB)",
                    "• Physical access to all participants",
                    "• Secure channels for data exchange",
                ],
                use_cases: vec![
                    "• Treasury management",
                    "• Cold storage operations",
                    "• High-value asset protection",
                    "• Regulatory compliance requirements",
                ],
                pros: vec![
                    "✅ Complete air-gap protection",
                    "✅ No network attack surface",
                    "✅ Verifiable at each step",
                    "✅ Meets strict compliance standards",
                    "✅ Maximum security for critical assets",
                ],
                cons: vec![
                    "⚠️ Slower coordination (hours/days)",
                    "⚠️ Requires physical media exchange",
                    "⚠️ Manual verification needed",
                    "⚠️ Less convenient for frequent use",
                ],
            },
        ]
    }
}

impl MockComponent for ModeSelectionComponent {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(5),   // Header
                Constraint::Min(0),      // Content
                Constraint::Length(4),   // Footer
            ])
            .margin(1)
            .split(area);
        
        // Header
        self.render_header(frame, chunks[0]);
        
        // Main content - split horizontally for side-by-side comparison
        let content_chunks = Layout::default()
            .direction(LayoutDirection::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[1]);
        
        // Render both modes side by side
        let modes = self.get_modes();
        for (i, mode) in modes.iter().enumerate() {
            self.render_mode(frame, content_chunks[i], mode, i == self.selected);
        }
        
        // Footer with controls
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
            Cmd::Move(Direction::Left) => {
                self.selected = 0;
                CmdResult::Changed(self.state())
            }
            Cmd::Move(Direction::Right) => {
                self.selected = 1;
                CmdResult::Changed(self.state())
            }
            Cmd::Submit => CmdResult::Submit(self.state()),
            _ => CmdResult::None,
        }
    }
}

impl ModeSelectionComponent {
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let header_text = vec![
            "🔐 OPERATION MODE SELECTION",
            "",
            "Choose between Online (Hot) and Offline (Cold) wallet modes",
            "This decision affects security, convenience, and operational workflow",
        ];
        
        let header = Paragraph::new(header_text.join("\n"))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Choose Your Security Model ")
                    .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            );
        frame.render_widget(header, area);
    }
    
    fn render_mode(&self, frame: &mut Frame, area: Rect, mode: &OperationMode, selected: bool) {
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(3),   // Title
                Constraint::Length(2),   // Security & Speed
                Constraint::Length(6),   // Requirements
                Constraint::Length(6),   // Use Cases
                Constraint::Min(0),      // Pros & Cons
            ])
            .margin(1)
            .split(area);
        
        // Title
        let title = Paragraph::new(format!("{} {}", mode.icon, mode.name))
            .style(
                Style::default()
                    .fg(if selected { Color::Yellow } else { Color::White })
                    .add_modifier(Modifier::BOLD)
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);
        
        // Security & Speed badges
        let badges = Paragraph::new(format!("🛡️ {} | ⚡ {}", mode.security_level, mode.speed))
            .style(Style::default().fg(if selected { Color::Green } else { Color::Gray }))
            .alignment(Alignment::Center);
        frame.render_widget(badges, chunks[1]);
        
        // Requirements
        let req_text = format!("📋 Requirements:\n{}", mode.requirements.join("\n"));
        let requirements = Paragraph::new(req_text)
            .style(Style::default().fg(if selected { Color::Cyan } else { Color::DarkGray }))
            .wrap(Wrap { trim: true });
        frame.render_widget(requirements, chunks[2]);
        
        // Use Cases
        let use_case_text = format!("💼 Use Cases:\n{}", mode.use_cases.join("\n"));
        let use_cases = Paragraph::new(use_case_text)
            .style(Style::default().fg(if selected { Color::Magenta } else { Color::DarkGray }))
            .wrap(Wrap { trim: true });
        frame.render_widget(use_cases, chunks[3]);
        
        // Pros & Cons
        let pros_cons = format!(
            "Advantages:\n{}\n\nConsiderations:\n{}",
            mode.pros.join("\n"),
            mode.cons.join("\n")
        );
        let pros_cons_widget = Paragraph::new(pros_cons)
            .style(Style::default().fg(if selected { Color::White } else { Color::DarkGray }))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(if selected { BorderType::Thick } else { BorderType::Rounded })
                    .border_style(
                        Style::default().fg(if selected { Color::Yellow } else { Color::Gray })
                    )
            );
        frame.render_widget(pros_cons_widget, chunks[4]);
    }
    
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let selected_mode = if self.selected == 0 { "Online" } else { "Offline" };
        let footer_text = vec![
            format!("Selected: {} Mode", selected_mode),
            "".to_string(),
            "← → Switch Between Modes | Enter: Confirm Selection | Esc: Back".to_string(),
            "💡 Tip: You can switch modes later, but it requires re-initialization".to_string(),
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

impl Component<Message, UserEvent> for ModeSelectionComponent {
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

impl MpcWalletComponent for ModeSelectionComponent {
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