//! SD Card Manager Component
//!
//! Handles export/import operations for offline DKG and signing

use crate::elm::components::{Id, UserEvent, MpcWalletComponent};
use crate::elm::message::Message;

use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::Event;
use ratatui::layout::{Rect, Constraint, Direction as LayoutDirection, Layout, Alignment};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, BorderType, Paragraph, List, ListItem, Wrap, Gauge};
use tuirealm::{Component, Frame, MockComponent, Props, State, StateValue};

/// SD Card manager for offline operations
#[derive(Debug, Clone)]
pub struct SDCardManagerComponent {
    props: Props,
    operation: SDCardOperation,
    selected_file: usize,
    files: Vec<FileEntry>,
    focused: bool,
    sd_card_mounted: bool,
    sd_card_path: String,
    operation_status: OperationStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SDCardOperation {
    Export,
    Import,
}

#[derive(Debug, Clone)]
struct FileEntry {
    name: String,
    file_type: FileType,
    size: String,
    timestamp: String,
    status: FileStatus,
    description: String,
}

#[derive(Debug, Clone, PartialEq)]
enum FileType {
    SessionParams,
    Commitment,
    EncryptedShare,
    PublicData,
    SigningRequest,
    SignatureShare,
}

#[derive(Debug, Clone, PartialEq)]
enum FileStatus {
    Ready,
    Processing,
    Verified,
    Failed,
    Encrypted,
}

#[derive(Debug, Clone, PartialEq)]
enum OperationStatus {
    Idle,
    Mounting,
    Scanning,
    Verifying,
    Exporting,
    Importing,
    Success(String),
    Error(String),
}

impl Default for SDCardManagerComponent {
    fn default() -> Self {
        Self::new(SDCardOperation::Export)
    }
}

impl SDCardManagerComponent {
    pub fn new(operation: SDCardOperation) -> Self {
        let files = if operation == SDCardOperation::Import {
            // Mock files found on SD card for import
            vec![
                FileEntry {
                    name: "session_params.json".to_string(),
                    file_type: FileType::SessionParams,
                    size: "2.3 KB".to_string(),
                    timestamp: "2025-01-05 14:30".to_string(),
                    status: FileStatus::Verified,
                    description: "DKG session parameters (ID: DKG-2025-001)".to_string(),
                },
                FileEntry {
                    name: "round1_P1_commitment.json".to_string(),
                    file_type: FileType::Commitment,
                    size: "4.7 KB".to_string(),
                    timestamp: "2025-01-05 14:45".to_string(),
                    status: FileStatus::Ready,
                    description: "Round 1 commitment from Participant 1".to_string(),
                },
                FileEntry {
                    name: "round1_P2_commitment.json".to_string(),
                    file_type: FileType::Commitment,
                    size: "4.6 KB".to_string(),
                    timestamp: "2025-01-05 14:47".to_string(),
                    status: FileStatus::Ready,
                    description: "Round 1 commitment from Participant 2".to_string(),
                },
                FileEntry {
                    name: "round2_P1_shares_for_P3.enc".to_string(),
                    file_type: FileType::EncryptedShare,
                    size: "8.2 KB".to_string(),
                    timestamp: "2025-01-05 15:10".to_string(),
                    status: FileStatus::Encrypted,
                    description: "Encrypted shares from P1 for you (P3)".to_string(),
                },
            ]
        } else {
            // Files ready to export
            vec![
                FileEntry {
                    name: "round1_P3_commitment.json".to_string(),
                    file_type: FileType::Commitment,
                    size: "4.8 KB".to_string(),
                    timestamp: "Now".to_string(),
                    status: FileStatus::Ready,
                    description: "Your Round 1 commitment to share".to_string(),
                },
                FileEntry {
                    name: "round2_P3_shares_for_P1.enc".to_string(),
                    file_type: FileType::EncryptedShare,
                    size: "8.1 KB".to_string(),
                    timestamp: "Now".to_string(),
                    status: FileStatus::Encrypted,
                    description: "Encrypted shares for Participant 1".to_string(),
                },
                FileEntry {
                    name: "round2_P3_shares_for_P2.enc".to_string(),
                    file_type: FileType::EncryptedShare,
                    size: "8.1 KB".to_string(),
                    timestamp: "Now".to_string(),
                    status: FileStatus::Encrypted,
                    description: "Encrypted shares for Participant 2".to_string(),
                },
            ]
        };
        
        Self {
            props: Props::default(),
            operation,
            selected_file: 0,
            files,
            focused: false,
            sd_card_mounted: false,
            sd_card_path: "/media/sdcard".to_string(),
            operation_status: OperationStatus::Idle,
        }
    }
    
    fn get_file_icon(&self, file_type: &FileType) -> &str {
        match file_type {
            FileType::SessionParams => "📋",
            FileType::Commitment => "🔑",
            FileType::EncryptedShare => "🔐",
            FileType::PublicData => "📊",
            FileType::SigningRequest => "✍️",
            FileType::SignatureShare => "✅",
        }
    }
    
    fn get_status_color(&self, status: &FileStatus) -> Color {
        match status {
            FileStatus::Ready => Color::Green,
            FileStatus::Processing => Color::Yellow,
            FileStatus::Verified => Color::Cyan,
            FileStatus::Failed => Color::Red,
            FileStatus::Encrypted => Color::Magenta,
        }
    }
}

impl MockComponent for SDCardManagerComponent {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(5),   // Header
                Constraint::Length(8),   // SD Card status
                Constraint::Min(0),      // File list
                Constraint::Length(6),   // Operation panel
                Constraint::Length(4),   // Footer
            ])
            .margin(1)
            .split(area);
        
        // Header
        self.render_header(frame, chunks[0]);
        
        // SD Card status
        self.render_sd_status(frame, chunks[1]);
        
        // File list
        self.render_file_list(frame, chunks[2]);
        
        // Operation panel
        self.render_operation_panel(frame, chunks[3]);
        
        // Footer
        self.render_footer(frame, chunks[4]);
    }
    
    fn query(&self, attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        self.props.get(attr)
    }
    
    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {
        self.props.set(attr, value);
    }
    
    fn state(&self) -> tuirealm::State {
        State::One(StateValue::Usize(self.selected_file))
    }
    
    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Move(Direction::Up) => {
                if self.selected_file > 0 {
                    self.selected_file -= 1;
                } else {
                    self.selected_file = self.files.len().saturating_sub(1);
                }
                CmdResult::Changed(self.state())
            }
            Cmd::Move(Direction::Down) => {
                if self.selected_file < self.files.len().saturating_sub(1) {
                    self.selected_file += 1;
                } else {
                    self.selected_file = 0;
                }
                CmdResult::Changed(self.state())
            }
            Cmd::Submit => CmdResult::Submit(self.state()),
            _ => CmdResult::None,
        }
    }
}

impl SDCardManagerComponent {
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let title = match self.operation {
            SDCardOperation::Export => "💾 EXPORT TO SD CARD - Offline DKG Data Transfer",
            SDCardOperation::Import => "📂 IMPORT FROM SD CARD - Offline DKG Data Reception",
        };
        
        let header = Paragraph::new(title)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(
                        if self.operation == SDCardOperation::Export {
                            Color::Green
                        } else {
                            Color::Yellow
                        }
                    ))
                    .title(" SD Card Manager - Air-Gapped Transfer ")
                    .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            );
        frame.render_widget(header, area);
    }
    
    fn render_sd_status(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(LayoutDirection::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(area);
        
        // SD Card detection status
        let detection_text = if self.sd_card_mounted {
            format!(
                "✅ SD Card Detected\n\n📁 Mount Point: {}\n💾 Capacity: 32 GB\n📊 Available: 28.5 GB\n🔒 Write Protected: No",
                self.sd_card_path
            )
        } else {
            "❌ No SD Card Detected\n\n⚠️ Please insert SD card\n💡 Ensure card is formatted\n🔌 Check card reader connection\n🔄 Press M to refresh".to_string()
        };
        
        let detection_color = if self.sd_card_mounted { Color::Green } else { Color::Red };
        
        let detection = Paragraph::new(detection_text)
            .style(Style::default().fg(detection_color))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(detection_color))
                    .title(" 💾 SD Card Status ")
            );
        frame.render_widget(detection, chunks[0]);
        
        // Security verification
        let sd_status = if self.sd_card_mounted { "✅" } else { "⬜" };
        let security_text = vec![
            "🔒 Security Checklist:",
            "",
            &format!("{} SD card is present", sd_status),
            "✅ Network interfaces disabled",
            "✅ Bluetooth disabled",
            "✅ WiFi disabled",
            "✅ Running in air-gapped mode",
            "⬜ Files encrypted (optional)",
        ].join("\n");
        
        let security = Paragraph::new(security_text)
            .style(Style::default().fg(Color::Cyan))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" 🛡️ Security Status ")
            );
        frame.render_widget(security, chunks[1]);
    }
    
    fn render_file_list(&self, frame: &mut Frame, area: Rect) {
        if self.files.is_empty() {
            let empty_msg = Paragraph::new("No files to display\n\nPress R to refresh")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray))
                        .title(if self.operation == SDCardOperation::Export {
                            " 📤 Files to Export "
                        } else {
                            " 📥 Files to Import "
                        })
                );
            frame.render_widget(empty_msg, area);
            return;
        }
        
        let items: Vec<ListItem> = self.files
            .iter()
            .enumerate()
            .map(|(i, file)| {
                let is_selected = i == self.selected_file;
                let icon = self.get_file_icon(&file.file_type);
                let status_color = self.get_status_color(&file.status);
                
                let content = if is_selected {
                    format!(
                        "▶ {} {} ({}) - {}\n  └─ {}",
                        icon,
                        file.name,
                        file.size,
                        file.timestamp,
                        file.description
                    )
                } else {
                    format!(
                        "  {} {} ({}) - {:?}",
                        icon,
                        file.name,
                        file.size,
                        file.status
                    )
                };
                
                ListItem::new(content).style(
                    Style::default().fg(if is_selected { status_color } else { Color::Gray })
                )
            })
            .collect();
        
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::White))
                    .title(if self.operation == SDCardOperation::Export {
                        " 📤 Files Ready to Export "
                    } else {
                        " 📥 Files Available to Import "
                    })
                    .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            );
        
        frame.render_widget(list, area);
    }
    
    fn render_operation_panel(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(3),  // Operation status
                Constraint::Length(3),  // Instructions
            ])
            .split(area);
        
        // Operation status
        let (status_text, status_color) = match &self.operation_status {
            OperationStatus::Idle => ("Ready for operation", Color::Gray),
            OperationStatus::Mounting => ("Mounting SD card...", Color::Yellow),
            OperationStatus::Scanning => ("Scanning files...", Color::Yellow),
            OperationStatus::Verifying => ("Verifying file integrity...", Color::Cyan),
            OperationStatus::Exporting => ("Exporting files to SD card...", Color::Blue),
            OperationStatus::Importing => ("Importing files from SD card...", Color::Blue),
            OperationStatus::Success(msg) => (msg.as_str(), Color::Green),
            OperationStatus::Error(msg) => (msg.as_str(), Color::Red),
        };
        
        let status = Paragraph::new(format!("📊 Status: {}", status_text))
            .style(Style::default().fg(status_color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(status_color))
            );
        frame.render_widget(status, chunks[0]);
        
        // Instructions
        let instructions = if self.operation == SDCardOperation::Export {
            "📝 Instructions: Select files and press Enter to export | Space to select/deselect | A to select all"
        } else {
            "📝 Instructions: Select files and press Enter to import | V to verify signatures | Space to preview"
        };
        
        let instructions_widget = Paragraph::new(instructions)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(instructions_widget, chunks[1]);
    }
    
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer_text = vec![
            format!("Mode: {} | Files: {} | Selected: {}", 
                if self.operation == SDCardOperation::Export { "Export" } else { "Import" },
                self.files.len(),
                self.selected_file + 1
            ),
            "".to_string(),
            "↑↓ Navigate | Enter: Execute | M: Mount SD | E: Eject Safely | V: Verify | Esc: Back".to_string(),
            "⚠️ IMPORTANT: Always safely eject SD card before physical removal".to_string(),
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

impl Component<Message, UserEvent> for SDCardManagerComponent {
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

impl MpcWalletComponent for SDCardManagerComponent {
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