//! Curve Selection Component - Secp256k1 vs Ed25519
//!
//! Professional component explaining cryptographic curve choices

use crate::elm::components::{Id, UserEvent, MpcWalletComponent};
use crate::elm::message::Message;

use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::Event;
use ratatui::layout::{Rect, Constraint, Direction as LayoutDirection, Layout, Alignment};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, BorderType, Paragraph, Wrap};
use tuirealm::{Component, Frame, MockComponent, Props, State, StateValue};

/// Professional curve selection component
#[derive(Debug, Clone)]
pub struct CurveSelectionComponent {
    props: Props,
    selected: usize,
    focused: bool,
}

#[derive(Debug, Clone)]
struct CurveInfo {
    name: &'static str,
    technical_name: &'static str,
    icon: &'static str,
    blockchains: Vec<&'static str>,
    key_size: &'static str,
    signature_size: &'static str,
    security_level: &'static str,
    performance: &'static str,
    features: Vec<&'static str>,
    technical_details: Vec<&'static str>,
    when_to_use: Vec<&'static str>,
}

impl Default for CurveSelectionComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl CurveSelectionComponent {
    pub fn new() -> Self {
        Self {
            props: Props::default(),
            selected: 0,
            focused: false,
        }
    }
    
    pub fn with_selected(selected: usize) -> Self {
        Self {
            props: Props::default(),
            selected,
            focused: false,
        }
    }
    
    fn get_curves(&self) -> Vec<CurveInfo> {
        vec![
            CurveInfo {
                name: "Secp256k1",
                technical_name: "NIST P-256k1 (Koblitz curve)",
                icon: "🔷",
                blockchains: vec![
                    "• Ethereum (ETH)",
                    "• Bitcoin (BTC)",
                    "• BNB Chain",
                    "• Polygon",
                    "• Arbitrum",
                    "• Most EVM chains",
                ],
                key_size: "256 bits (32 bytes)",
                signature_size: "64-71 bytes (DER encoded)",
                security_level: "128-bit security",
                performance: "Moderate speed",
                features: vec![
                    "✓ Industry standard for blockchain",
                    "✓ Hardware wallet support",
                    "✓ Extensive tooling ecosystem",
                    "✓ Battle-tested in production",
                    "✓ Wide exchange support",
                ],
                technical_details: vec![
                    "• Elliptic curve: y² = x³ + 7",
                    "• Prime field: 2²⁵⁶ - 2³² - 977",
                    "• FROST-secp256k1 implementation",
                    "• ECDSA/Schnorr signatures",
                    "• Deterministic k generation",
                ],
                when_to_use: vec![
                    "→ Managing Ethereum assets",
                    "→ Bitcoin operations",
                    "→ DeFi interactions",
                    "→ Cross-chain bridges",
                    "→ Maximum compatibility needed",
                ],
            },
            CurveInfo {
                name: "Ed25519",
                technical_name: "Edwards25519 (Twisted Edwards curve)",
                icon: "🔶",
                blockchains: vec![
                    "• Solana (SOL)",
                    "• Near Protocol",
                    "• Cardano (ADA)",
                    "• Polkadot (DOT)",
                    "• Stellar (XLM)",
                    "• Tezos (XTZ)",
                ],
                key_size: "256 bits (32 bytes)",
                signature_size: "64 bytes (fixed)",
                security_level: "128-bit security",
                performance: "High speed",
                features: vec![
                    "✓ Faster signature generation",
                    "✓ Smaller signatures",
                    "✓ Resistance to side-channels",
                    "✓ No malleability issues",
                    "✓ Simpler implementation",
                ],
                technical_details: vec![
                    "• Curve: -x² + y² = 1 - (121665/121666)x²y²",
                    "• Prime: 2²⁵⁵ - 19",
                    "• FROST-ed25519 implementation",
                    "• EdDSA signatures only",
                    "• Cofactor of 8",
                ],
                when_to_use: vec![
                    "→ Solana ecosystem",
                    "→ High-performance needs",
                    "→ Fixed signature sizes required",
                    "→ Modern blockchain platforms",
                    "→ Lower computational overhead",
                ],
            },
        ]
    }
}

impl MockComponent for CurveSelectionComponent {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(6),   // Header
                Constraint::Min(0),      // Content
                Constraint::Length(4),   // Footer
            ])
            .margin(1)
            .split(area);
        
        // Header
        self.render_header(frame, chunks[0]);
        
        // Main content - split for comparison
        let content_chunks = Layout::default()
            .direction(LayoutDirection::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[1]);
        
        // Render both curves side by side
        let curves = self.get_curves();
        for (i, curve) in curves.iter().enumerate() {
            self.render_curve(frame, content_chunks[i], curve, i == self.selected);
        }
        
        // Footer
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

impl CurveSelectionComponent {
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let header_text = vec![
            "🔐 CRYPTOGRAPHIC CURVE SELECTION (Step 2 of 3)",
            "",
            "Choose the elliptic curve for your MPC wallet",
            "This determines which blockchains you can interact with",
            "⚠️ Cannot be changed after wallet creation",
        ];
        
        let header = Paragraph::new(header_text.join("\n"))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Elliptic Curve Selection ")
                    .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            );
        frame.render_widget(header, area);
    }
    
    fn render_curve(&self, frame: &mut Frame, area: Rect, curve: &CurveInfo, selected: bool) {
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(3),   // Name
                Constraint::Length(2),   // Technical info
                Constraint::Length(8),   // Blockchains
                Constraint::Length(3),   // Performance
                Constraint::Length(7),   // Features
                Constraint::Length(7),   // Technical details
                Constraint::Min(0),      // When to use
            ])
            .margin(1)
            .split(area);
        
        // Name and icon
        let title = Paragraph::new(format!("{} {}", curve.icon, curve.name))
            .style(
                Style::default()
                    .fg(if selected { Color::Yellow } else { Color::White })
                    .add_modifier(Modifier::BOLD)
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);
        
        // Technical name
        let tech_name = Paragraph::new(curve.technical_name)
            .style(Style::default().fg(if selected { Color::Green } else { Color::Gray }))
            .alignment(Alignment::Center);
        frame.render_widget(tech_name, chunks[1]);
        
        // Supported blockchains
        let blockchains = Paragraph::new(format!("⛓️ Blockchains:\n{}", curve.blockchains.join("\n")))
            .style(Style::default().fg(if selected { Color::Cyan } else { Color::DarkGray }))
            .wrap(Wrap { trim: true });
        frame.render_widget(blockchains, chunks[2]);
        
        // Performance info
        let perf_info = format!(
            "📊 {} | {} | {} | Key: {}",
            curve.performance, curve.security_level, curve.signature_size, curve.key_size
        );
        let performance = Paragraph::new(perf_info)
            .style(Style::default().fg(if selected { Color::Magenta } else { Color::DarkGray }))
            .alignment(Alignment::Center);
        frame.render_widget(performance, chunks[3]);
        
        // Features
        let features = Paragraph::new(format!("✨ Features:\n{}", curve.features.join("\n")))
            .style(Style::default().fg(if selected { Color::Green } else { Color::DarkGray }))
            .wrap(Wrap { trim: true });
        frame.render_widget(features, chunks[4]);
        
        // Technical details
        let tech_details = Paragraph::new(format!("🔧 Technical:\n{}", curve.technical_details.join("\n")))
            .style(Style::default().fg(if selected { Color::Blue } else { Color::DarkGray }))
            .wrap(Wrap { trim: true });
        frame.render_widget(tech_details, chunks[5]);
        
        // When to use
        let when_to_use = Paragraph::new(format!("📌 When to Use:\n{}", curve.when_to_use.join("\n")))
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
        frame.render_widget(when_to_use, chunks[6]);
    }
    
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let selected_curve = if self.selected == 0 { "Secp256k1" } else { "Ed25519" };
        let footer_text = vec![
            format!("Selected: {} Curve", selected_curve),
            "".to_string(),
            "← → Switch Between Curves | Enter: Confirm Selection | Esc: Back".to_string(),
            "💡 Most users choose Secp256k1 for Ethereum/Bitcoin compatibility".to_string(),
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

impl Component<Message, UserEvent> for CurveSelectionComponent {
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

impl MpcWalletComponent for CurveSelectionComponent {
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