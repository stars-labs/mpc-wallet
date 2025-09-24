//! Offline DKG Process Component
//!
//! Comprehensive component for air-gapped DKG with manual SD card coordination

use crate::elm::components::{Id, UserEvent, MpcWalletComponent};
use crate::elm::message::Message;

use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::Event;
use ratatui::layout::{Rect, Constraint, Direction as LayoutDirection, Layout, Alignment};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, BorderType, Paragraph, List, ListItem, Wrap, Gauge};
use tuirealm::{Component, Frame, MockComponent, Props, State, StateValue};

/// Offline DKG process component with detailed steps
#[derive(Debug, Clone)]
pub struct OfflineDKGProcessComponent {
    props: Props,
    current_step: usize,
    role: ParticipantRole,
    focused: bool,
    round: DKGRound,
    participants_ready: Vec<String>,
    total_participants: usize,
    threshold: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParticipantRole {
    Coordinator,
    Participant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DKGRound {
    Setup,           // Initial setup and parameter distribution
    Round1,          // Commitment generation
    Round2,          // Share distribution
    Finalization,    // Final key assembly
    Complete,        // DKG complete
}

#[derive(Debug, Clone)]
struct DKGStep {
    round: DKGRound,
    #[allow(dead_code)]
    step_number: usize,
    title: &'static str,
    description: Vec<&'static str>,
    coordinator_actions: Vec<&'static str>,
    participant_actions: Vec<&'static str>,
    verification_steps: Vec<&'static str>,
    data_format: &'static str,
    estimated_time: &'static str,
    security_notes: Vec<&'static str>,
}

impl Default for OfflineDKGProcessComponent {
    fn default() -> Self {
        Self::new(ParticipantRole::Participant, 3, 2)
    }
}

impl OfflineDKGProcessComponent {
    pub fn new(role: ParticipantRole, total_participants: usize, threshold: usize) -> Self {
        Self {
            props: Props::default(),
            current_step: 0,
            role,
            focused: false,
            round: DKGRound::Setup,
            participants_ready: Vec::new(),
            total_participants,
            threshold,
        }
    }
    
    fn get_dkg_steps(&self) -> Vec<DKGStep> {
        vec![
            // =================== SETUP PHASE ===================
            DKGStep {
                round: DKGRound::Setup,
                step_number: 1,
                title: "🔧 Initial Setup & Parameter Distribution",
                description: vec![
                    "Coordinator creates the DKG session with parameters",
                    "All participants verify they're in offline/air-gapped mode",
                    "Coordinator distributes session parameters via SD card",
                ],
                coordinator_actions: vec![
                    "1. ✅ Verify all network interfaces are disabled",
                    "2. 📝 Create session with threshold parameters",
                    "3. 🆔 Generate unique session ID and participant IDs",
                    "4. 💾 Export session package to SD card:",
                    "   • Session ID: [Generated at runtime]",
                    "   • Participants: 3, Threshold: 2",
                    "   • Curve: Secp256k1",
                    "   • Participant IDs: [P1, P2, P3]",
                    "5. 📦 Physically deliver SD card to each participant",
                ],
                participant_actions: vec![
                    "1. ✅ Disable ALL network interfaces (WiFi, Ethernet, Bluetooth)",
                    "2. ⏳ Wait for coordinator's SD card",
                    "3. 💾 Import session parameters from SD card",
                    "4. ✔️ Verify parameters match expectations",
                    "5. 🔑 Note your assigned participant ID",
                ],
                verification_steps: vec![
                    "• Verify session ID matches across all participants",
                    "• Confirm threshold parameters (2-of-3)",
                    "• Check participant count matches expected",
                    "• Verify curve type is correct",
                    "• Ensure all machines are air-gapped",
                ],
                data_format: "JSON file: session_params.json",
                estimated_time: "15-30 minutes",
                security_notes: vec![
                    "⚠️ CRITICAL: Verify network disconnection before proceeding",
                    "🔒 Use write-protected SD cards when possible",
                    "👁️ Maintain physical control of SD cards at all times",
                ],
            },
            
            // =================== ROUND 1 ===================
            DKGStep {
                round: DKGRound::Round1,
                step_number: 2,
                title: "📤 Round 1: Commitment Generation & Exchange",
                description: vec![
                    "Each participant generates cryptographic commitments",
                    "Commitments are exported to SD cards",
                    "Coordinator collects and redistributes all commitments",
                ],
                coordinator_actions: vec![
                    "1. 🔑 Generate your own Round 1 commitment",
                    "2. 💾 Export commitment to: round1_P1_commitment.json",
                    "3. 📦 Collect SD cards from all participants",
                    "4. ✔️ Verify all commitments are valid",
                    "5. 📂 Create aggregated commitment package:",
                    "   • All participant commitments",
                    "   • Timestamp and round identifier",
                    "   • Checksum for integrity",
                    "6. 💾 Copy package to SD cards for each participant",
                    "7. 📦 Distribute SD cards back to participants",
                ],
                participant_actions: vec![
                    "1. 🔑 Generate Round 1 commitment",
                    "2. 💾 Export to: round1_P[ID]_commitment.json",
                    "3. 📤 Deliver SD card to coordinator",
                    "4. ⏳ Wait for aggregated commitments",
                    "5. 💾 Import all commitments from coordinator's SD card",
                    "6. ✔️ Verify commitment integrity",
                ],
                verification_steps: vec![
                    "• Each commitment file has correct participant ID",
                    "• All commitments use same session ID",
                    "• Cryptographic signatures are valid",
                    "• No duplicate or missing commitments",
                    "• File checksums match",
                ],
                data_format: "JSON files: round1_*_commitment.json",
                estimated_time: "30-45 minutes",
                security_notes: vec![
                    "⚠️ Never share your private polynomial",
                    "🔒 Commitments are public but must be authentic",
                    "✅ Verify participant IDs match setup phase",
                ],
            },
            
            // =================== ROUND 2 ===================
            DKGStep {
                round: DKGRound::Round2,
                step_number: 3,
                title: "🔐 Round 2: Encrypted Share Distribution",
                description: vec![
                    "Each participant creates encrypted shares for others",
                    "Shares are distributed via coordinator",
                    "Each participant verifies received shares",
                ],
                coordinator_actions: vec![
                    "1. 🔐 Generate encrypted shares for other participants",
                    "2. 📁 Create share files:",
                    "   • round2_P1_shares_for_P2.enc",
                    "   • round2_P1_shares_for_P3.enc",
                    "3. 💾 Export your shares to SD card",
                    "4. 📦 Collect SD cards from all participants",
                    "5. 🗂️ Organize shares by recipient:",
                    "   • For P1: [shares from P2, P3]",
                    "   • For P2: [shares from P1, P3]",
                    "   • For P3: [shares from P1, P2]",
                    "6. 💾 Create personalized SD cards per participant",
                    "7. 📦 Securely deliver correct SD card to each participant",
                    "8. 🔥 Securely destroy any temporary copies",
                ],
                participant_actions: vec![
                    "1. 🔐 Generate encrypted shares for each other participant",
                    "2. 💾 Export shares with clear naming:",
                    "   • round2_P[MY_ID]_shares_for_P[THEIR_ID].enc",
                    "3. 📤 Deliver SD card to coordinator",
                    "4. ⏳ Wait for your personalized share package",
                    "5. 💾 Import shares meant for you",
                    "6. 🔓 Decrypt shares using your private key",
                    "7. ✔️ Verify share validity using commitments",
                    "8. ⚠️ Report any invalid shares immediately",
                ],
                verification_steps: vec![
                    "• Each share is properly encrypted for recipient",
                    "• Shares match Round 1 commitments",
                    "• No shares are missing or duplicated",
                    "• Decryption succeeds with correct keys",
                    "• Share values are within expected range",
                    "• Complaints process if shares are invalid",
                ],
                data_format: "Encrypted files: round2_*_shares_for_*.enc",
                estimated_time: "45-60 minutes",
                security_notes: vec![
                    "🔥 CRITICAL: Shares are highly sensitive",
                    "🔐 Only recipient can decrypt their shares",
                    "⚠️ Never share unencrypted shares",
                    "🗑️ Securely wipe SD cards after use",
                ],
            },
            
            // =================== FINALIZATION ===================
            DKGStep {
                round: DKGRound::Finalization,
                step_number: 4,
                title: "✨ Finalization: Key Assembly & Verification",
                description: vec![
                    "Participants compute final key shares",
                    "Public key is derived and verified",
                    "Wallet addresses are generated",
                ],
                coordinator_actions: vec![
                    "1. 🔑 Compute your final key share from received shares",
                    "2. 🔢 Calculate public key from all commitments",
                    "3. 📊 Generate verification proof",
                    "4. 💾 Export public data package:",
                    "   • Group public key",
                    "   • Individual public shares",
                    "   • Wallet addresses (ETH, BTC)",
                    "   • Verification proofs",
                    "5. 📦 Distribute verification package to all",
                    "6. ✅ Collect confirmation from each participant",
                    "7. 📄 Create final DKG summary document",
                ],
                participant_actions: vec![
                    "1. 🔑 Compute final key share",
                    "2. 🔒 Securely store private key share:",
                    "   • Encrypted with strong password",
                    "   • Backup to secure location",
                    "   • Never store on networked device",
                    "3. 💾 Import verification package",
                    "4. ✔️ Verify group public key matches",
                    "5. 📊 Test signature generation (optional)",
                    "6. ✅ Send confirmation to coordinator",
                ],
                verification_steps: vec![
                    "• All participants derive same public key",
                    "• Wallet addresses match across participants",
                    "• Test signature can be verified",
                    "• Each participant has valid key share",
                    "• Threshold signatures work correctly",
                ],
                data_format: "JSON: final_wallet_data.json",
                estimated_time: "30-45 minutes",
                security_notes: vec![
                    "✅ Save wallet addresses for future use",
                    "🔒 Key shares must remain air-gapped",
                    "📝 Document participant mapping",
                    "🔥 Destroy temporary files securely",
                ],
            },
            
            // =================== COMPLETE ===================
            DKGStep {
                round: DKGRound::Complete,
                step_number: 5,
                title: "🎉 DKG Complete: Wallet Ready",
                description: vec![
                    "Distributed key generation successful",
                    "Wallet is ready for threshold signing",
                    "Maintain security practices for operations",
                ],
                coordinator_actions: vec![
                    "1. 📄 Distribute final wallet summary:",
                    "   • Wallet ID and addresses",
                    "   • Participant roster",
                    "   • Threshold configuration",
                    "   • Contact information (offline)",
                    "2. 🗓️ Schedule regular key share verification",
                    "3. 📋 Document offline signing procedures",
                    "4. 🔒 Secure all DKG artifacts",
                ],
                participant_actions: vec![
                    "1. 💾 Store key share in secure location",
                    "2. 📝 Record wallet information",
                    "3. 🔒 Maintain air-gap for key storage",
                    "4. ✅ Be ready for signing ceremonies",
                ],
                verification_steps: vec![
                    "• All participants confirmed success",
                    "• Wallet addresses are recorded",
                    "• Backup procedures documented",
                    "• Recovery plan in place",
                ],
                data_format: "Complete wallet package",
                estimated_time: "15 minutes",
                security_notes: vec![
                    "🎯 Success! Your MPC wallet is ready",
                    "🔒 Maintain offline security practices",
                    "📋 Follow same process for signing",
                ],
            },
        ]
    }
    
    fn get_current_step(&self) -> DKGStep {
        let steps = self.get_dkg_steps();
        steps[self.current_step.min(steps.len() - 1)].clone()
    }
    
    fn get_progress_internal(&self) -> f32 {
        let total_steps = 5;
        (self.current_step + 1) as f32 / total_steps as f32
    }
    
    /// Public method for testing
    pub fn get_progress(&self) -> f32 {
        self.get_progress_internal()
    }
}

impl MockComponent for OfflineDKGProcessComponent {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(7),   // Header with progress
                Constraint::Min(0),      // Main content
                Constraint::Length(5),   // Footer with controls
            ])
            .margin(1)
            .split(area);
        
        // Header with progress
        self.render_header(frame, chunks[0]);
        
        // Main content area
        let content_chunks = Layout::default()
            .direction(LayoutDirection::Horizontal)
            .constraints([
                Constraint::Percentage(50),  // Left: Actions
                Constraint::Percentage(50),  // Right: Verification & Notes
            ])
            .split(chunks[1]);
        
        self.render_actions(frame, content_chunks[0]);
        self.render_verification(frame, content_chunks[1]);
        
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
        State::One(StateValue::Usize(self.current_step))
    }
    
    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Move(Direction::Left) => {
                if self.current_step > 0 {
                    self.current_step -= 1;
                    self.round = self.get_current_step().round.clone();
                }
                CmdResult::Changed(self.state())
            }
            Cmd::Move(Direction::Right) => {
                let max_steps = self.get_dkg_steps().len();
                if self.current_step < max_steps - 1 {
                    self.current_step += 1;
                    self.round = self.get_current_step().round.clone();
                }
                CmdResult::Changed(self.state())
            }
            Cmd::Submit => CmdResult::Submit(self.state()),
            _ => CmdResult::None,
        }
    }
}

impl OfflineDKGProcessComponent {
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(2),  // Progress bar
                Constraint::Length(2),  // Current step info
            ])
            .split(area);
        
        // Title
        let role_text = match self.role {
            ParticipantRole::Coordinator => "COORDINATOR",
            ParticipantRole::Participant => "PARTICIPANT",
        };
        
        let title = Paragraph::new(format!(
            "🔒 OFFLINE DKG PROCESS - {} MODE",
            role_text
        ))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Yellow))
                .title(format!(" Air-Gapped {}-of-{} Threshold Setup ", self.threshold, self.total_participants))
                .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        );
        frame.render_widget(title, chunks[0]);
        
        // Progress bar
        let progress = self.get_progress_internal();
        let gauge = Gauge::default()
            .block(Block::default().title("Overall Progress"))
            .gauge_style(Style::default().fg(Color::Green))
            .percent((progress * 100.0) as u16)
            .label(format!("Step {} of 5 - {:.0}% Complete", self.current_step + 1, progress * 100.0));
        frame.render_widget(gauge, chunks[1]);
        
        // Current step info
        let current_step = self.get_current_step();
        let step_info = Paragraph::new(format!(
            "{} | Est. Time: {}",
            current_step.title, current_step.estimated_time
        ))
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
        frame.render_widget(step_info, chunks[2]);
    }
    
    fn render_actions(&self, frame: &mut Frame, area: Rect) {
        let current_step = self.get_current_step();
        
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(4),   // Description
                Constraint::Min(0),      // Actions list
            ])
            .split(area);
        
        // Description
        let desc_text = current_step.description.join("\n");
        let description = Paragraph::new(desc_text)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray))
                    .title(" Step Description ")
            );
        frame.render_widget(description, chunks[0]);
        
        // Actions based on role
        let actions = match self.role {
            ParticipantRole::Coordinator => &current_step.coordinator_actions,
            ParticipantRole::Participant => &current_step.participant_actions,
        };
        
        let action_items: Vec<ListItem> = actions
            .iter()
            .map(|action| {
                let style = if action.starts_with("   ") {
                    Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)
                } else if action.contains("CRITICAL") || action.contains("⚠️") {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                } else if action.contains("✅") || action.contains("✔️") {
                    Style::default().fg(Color::Green)
                } else if action.contains("💾") || action.contains("📦") {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(*action).style(style)
            })
            .collect();
        
        let actions_list = List::new(action_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(
                        if self.role == ParticipantRole::Coordinator {
                            Color::Yellow
                        } else {
                            Color::Cyan
                        }
                    ))
                    .title(format!(" {} Actions ", 
                        if self.role == ParticipantRole::Coordinator {
                            "📋 Coordinator"
                        } else {
                            "👤 Participant"
                        }
                    ))
                    .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            );
        
        frame.render_widget(actions_list, chunks[1]);
    }
    
    fn render_verification(&self, frame: &mut Frame, area: Rect) {
        let current_step = self.get_current_step();
        
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Percentage(40),  // Verification steps
                Constraint::Percentage(30),  // Security notes
                Constraint::Percentage(30),  // Data format & timing
            ])
            .split(area);
        
        // Verification steps
        let verification_items: Vec<ListItem> = current_step.verification_steps
            .iter()
            .map(|step| ListItem::new(*step).style(Style::default().fg(Color::Cyan)))
            .collect();
        
        let verification_list = List::new(verification_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .title(" ✔️ Verification Steps ")
                    .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            );
        frame.render_widget(verification_list, chunks[0]);
        
        // Security notes
        let security_items: Vec<ListItem> = current_step.security_notes
            .iter()
            .map(|note| {
                let style = if note.contains("CRITICAL") || note.contains("⚠️") {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                } else if note.contains("🔒") {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(*note).style(style)
            })
            .collect();
        
        let security_list = List::new(security_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red))
                    .title(" 🔒 Security Notes ")
                    .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            );
        frame.render_widget(security_list, chunks[1]);
        
        // Data format and participant status
        let status_text = format!(
            "📁 Data Format: {}\n\n👥 Participants: {}/{} Ready\n\n⏱️ Estimated Time: {}\n\n💾 SD Card Required: Yes",
            current_step.data_format,
            self.participants_ready.len(),
            self.total_participants,
            current_step.estimated_time
        );
        
        let status = Paragraph::new(status_text)
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(" 📊 Status & Format ")
            );
        frame.render_widget(status, chunks[2]);
    }
    
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer_text = vec![
            format!("Current: Step {} - {}", 
                self.current_step + 1,
                match self.round {
                    DKGRound::Setup => "Setup",
                    DKGRound::Round1 => "Round 1",
                    DKGRound::Round2 => "Round 2",
                    DKGRound::Finalization => "Finalization",
                    DKGRound::Complete => "Complete",
                }
            ),
            "".to_string(),
            "← → Navigate Steps | Enter: Perform Action | E: Export to SD | I: Import from SD | Esc: Back".to_string(),
            format!("💡 {} Mode - Follow {} instructions carefully",
                if self.role == ParticipantRole::Coordinator { "Coordinator" } else { "Participant" },
                if self.role == ParticipantRole::Coordinator { "coordinator" } else { "participant" }
            ),
        ];
        
        let footer = Paragraph::new(footer_text.join("\n"))
            .style(Style::default().fg(Color::Green).add_modifier(Modifier::ITALIC))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::DarkGray))
            );
        frame.render_widget(footer, area);
    }
}

impl Component<Message, UserEvent> for OfflineDKGProcessComponent {
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

impl MpcWalletComponent for OfflineDKGProcessComponent {
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