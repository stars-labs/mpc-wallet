//! Create Wallet Component
//!
//! Component for the wallet creation flow

use crate::elm::components::{Id, UserEvent, MpcWalletComponent};
use crate::elm::message::Message;

use tuirealm::command::{Cmd, CmdResult};
use tuirealm::event::Event;
use ratatui::layout::{Rect, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, BorderType, List, ListItem, ListState, Paragraph};
use tuirealm::{Component, Frame, MockComponent, Props, State, StateValue};

/// Create Wallet component
#[derive(Debug, Clone)]
pub struct CreateWalletComponent {
    props: Props,
    selected: usize,
    focused: bool,
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
}

impl MockComponent for CreateWalletComponent {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        tracing::debug!("🎨 CreateWalletComponent::view - rendering with selected: {}, focused: {}", 
                       self.selected, self.focused);
        
        // Split the area into title and options
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);
        
        // Title
        let title = Paragraph::new("Create New MPC Wallet")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(title, chunks[0]);
        
        // Options
        let options = vec![
            "1. Choose Mode (Online/Offline)",
            "2. Select Curve (Secp256k1/Ed25519)",
            "3. Configure Threshold",
            "4. Start DKG Process",
        ];
        
        let items: Vec<ListItem> = options
            .iter()
            .enumerate()
            .map(|(i, opt)| {
                let style = if i == self.selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };
                
                let prefix = if i == self.selected { "► " } else { "  " };
                ListItem::new(format!("{}{}", prefix, opt)).style(style)
            })
            .collect();
        
        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Wallet Creation Steps ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(if self.focused {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::Gray)
                    })
            );
        
        let mut list_state = ListState::default();
        list_state.select(Some(self.selected));
        
        frame.render_stateful_widget(list, chunks[1], &mut list_state);
        
        // Help text
        let help = Paragraph::new("↑↓ Navigate • Enter Select • Esc Back to Menu")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(help, chunks[2]);
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
                if self.selected > 0 {
                    self.selected -= 1;
                } else {
                    self.selected = 3;
                }
                CmdResult::Changed(self.state())
            }
            Cmd::Move(tuirealm::command::Direction::Down) => {
                if self.selected < 3 {
                    self.selected += 1;
                } else {
                    self.selected = 0;
                }
                CmdResult::Changed(self.state())
            }
            Cmd::Submit => CmdResult::Submit(self.state()),
            _ => CmdResult::None,
        }
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