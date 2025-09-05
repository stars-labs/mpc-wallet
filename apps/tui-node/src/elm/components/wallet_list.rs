//! Wallet List Component
//!
//! Displays the list of available wallets with their metadata.

use crate::elm::components::{Id, UserEvent, MpcWalletComponent};
use crate::elm::message::Message;
use crate::keystore::WalletMetadata;

use tuirealm::command::{Cmd, CmdResult};
use tuirealm::event::{Event, Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, Color, Style, TextModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, BorderType as TuiBorderType, Borders as TuiBorders, List, ListItem, ListState, Paragraph};
use tuirealm::{Component, Frame, MockComponent, Props, State, StateValue};

/// Wallet list component
#[derive(Debug, Clone)]
pub struct WalletList {
    props: Props,
    wallets: Vec<WalletMetadata>,
    selected: usize,
    focused: bool,
    scroll_offset: usize,
}

impl Default for WalletList {
    fn default() -> Self {
        Self::new()
    }
}

impl WalletList {
    pub fn new() -> Self {
        let mut props = Props::default();
        props.set(tuirealm::props::Attribute::Title, tuirealm::props::AttrValue::String("Manage Wallets".to_string()));
        // Set borders - tuirealm doesn't have Borders::ALL, so we use default
        
        Self {
            props,
            wallets: Vec::new(),
            selected: 0,
            focused: false,
            scroll_offset: 0,
        }
    }
    
    pub fn set_wallets(&mut self, wallets: Vec<WalletMetadata>) {
        self.wallets = wallets;
        if self.selected >= self.wallets.len() && !self.wallets.is_empty() {
            self.selected = self.wallets.len() - 1;
        }
    }
    
    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            // Adjust scroll if needed
            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
        }
    }
    
    fn move_down(&mut self) {
        if self.selected < self.wallets.len().saturating_sub(1) {
            self.selected += 1;
            // Adjust scroll if needed
            let visible_height = 10; // Approximate visible items
            if self.selected >= self.scroll_offset + visible_height {
                self.scroll_offset = self.selected - visible_height + 1;
            }
        }
    }
    
    fn select_current(&self) -> Option<Message> {
        if let Some(wallet) = self.wallets.get(self.selected) {
            Some(Message::SelectWallet {
                wallet_id: wallet.session_id.clone(),
            })
        } else {
            None
        }
    }
}

impl MockComponent for WalletList {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),      // List area
                Constraint::Length(4),   // Details area
            ])
            .split(area);
        
        // Render wallet list
        if self.wallets.is_empty() {
            // Show empty state
            let empty_msg = Paragraph::new("No wallets found. Create a new wallet to get started.")
                .block(
                    Block::default()
                        .title("Wallets")
                        .borders(TuiBorders::ALL)
                        .border_type(TuiBorderType::Rounded)
                        .border_style(if self.focused {
                            Style::default().fg(Color::Cyan)
                        } else {
                            Style::default().fg(Color::Gray)
                        })
                )
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            
            frame.render_widget(empty_msg, chunks[0]);
        } else {
            // Create list items
            let items: Vec<ListItem> = self.wallets
                .iter()
                .enumerate()
                .skip(self.scroll_offset)
                .map(|(i, wallet)| {
                    let is_selected = i == self.selected;
                    
                    let style = if is_selected {
                        if self.focused {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(TextModifiers::BOLD)
                        } else {
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(TextModifiers::BOLD)
                        }
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    
                    let prefix = if is_selected { "► " } else { "  " };
                    let text = format!(
                        "{}{} ({}/{}) - {}",
                        prefix,
                        wallet.session_id.chars().take(12).collect::<String>(),
                        wallet.threshold,
                        wallet.total_participants,
                        wallet.curve_type
                    );
                    
                    ListItem::new(text).style(style)
                })
                .collect();
            
            // Create the list widget
            let mut list_state = ListState::default();
            list_state.select(Some(self.selected - self.scroll_offset));
            
            let list = List::new(items)
                .block(
                    Block::default()
                        .title(format!("Wallets ({} total)", self.wallets.len()))
                        .borders(TuiBorders::ALL)
                        .border_type(TuiBorderType::Rounded)
                        .border_style(if self.focused {
                            Style::default().fg(Color::Cyan)
                        } else {
                            Style::default().fg(Color::Gray)
                        })
                )
                .highlight_style(Style::default().bg(Color::DarkGray));
            
            frame.render_stateful_widget(list, chunks[0], &mut list_state);
        }
        
        // Render selected wallet details
        if let Some(wallet) = self.wallets.get(self.selected) {
            let details = vec![
                format!("Created: {}", wallet.created_at),
                format!("Device: {}", wallet.device_id),
                format!("Index: {}/{}", wallet.participant_index, wallet.total_participants),
            ];
            
            let details_text = details.join(" | ");
            
            let details_widget = Paragraph::new(details_text)
                .block(
                    Block::default()
                        .title("Details")
                        .borders(TuiBorders::ALL)
                        .border_type(TuiBorderType::Rounded)
                        .border_style(Style::default().fg(Color::DarkGray))
                )
                .style(Style::default().fg(Color::Gray))
                .wrap(ratatui::widgets::Wrap { trim: true });
            
            frame.render_widget(details_widget, chunks[1]);
        }
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
            Cmd::Move(tuirealm::command::Direction::Up) => {
                self.move_up();
                CmdResult::Changed(self.state())
            }
            Cmd::Move(tuirealm::command::Direction::Down) => {
                self.move_down();
                CmdResult::Changed(self.state())
            }
            Cmd::Submit => CmdResult::Submit(self.state()),
            _ => CmdResult::None,
        }
    }
}

impl Component<Message, UserEvent> for WalletList {
    fn on(&mut self, event: Event<UserEvent>) -> Option<Message> {
        match event {
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.move_up();
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.move_down();
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.select_current()
            }
            Event::Keyboard(KeyEvent {
                code: Key::Esc,
                modifiers: KeyModifiers::NONE,
            }) => {
                // Properly navigate back, not exit!
                Some(Message::NavigateBack)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('d'),
                modifiers: KeyModifiers::NONE,
            }) => {
                // Delete wallet
                if let Some(wallet) = self.wallets.get(self.selected) {
                    Some(Message::DeleteWallet {
                        wallet_id: wallet.session_id.clone(),
                    })
                } else {
                    None
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char('e'),
                modifiers: KeyModifiers::NONE,
            }) => {
                // Export wallet
                if let Some(wallet) = self.wallets.get(self.selected) {
                    Some(Message::ExportWallet {
                        wallet_id: wallet.session_id.clone(),
                    })
                } else {
                    None
                }
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

impl MpcWalletComponent for WalletList {
    fn id(&self) -> Id {
        Id::WalletList
    }
    
    fn is_visible(&self) -> bool {
        true
    }
    
    fn on_focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}