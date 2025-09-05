//! Main Menu Component
//!
//! The main menu component provides the primary navigation interface for the application.

use crate::elm::components::{Id, UserEvent, MpcWalletComponent};
use crate::elm::message::Message;
use crate::elm::model::Screen;

use tuirealm::command::{Cmd, CmdResult};
use tuirealm::event::{Event, Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Color, Style, TextModifiers};
use ratatui::layout::Rect;
use ratatui::widgets::{Block, BorderType as TuiBorderType, Borders as TuiBorders, List, ListItem, ListState};
use tuirealm::{Component, Frame, MockComponent, Props, State, StateValue};

/// Main menu component
#[derive(Debug, Clone)]
pub struct MainMenu {
    props: Props,
    items: Vec<MenuItem>,
    selected: usize,
    focused: bool,
}

impl MainMenu {
    /// Update the selected index
    pub fn set_selected(&mut self, index: usize) {
        self.selected = index.min(self.items.len().saturating_sub(1));
    }
}

#[derive(Debug, Clone)]
struct MenuItem {
    label: String,
    description: String,
    screen: Screen,
    enabled: bool,
}

impl Default for MainMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl MainMenu {
    pub fn new() -> Self {
        let items = vec![
            MenuItem {
                label: "Create New Wallet".to_string(),
                description: "Start DKG to create a new MPC wallet".to_string(),
                screen: Screen::CreateWallet(Default::default()),
                enabled: true,
            },
            MenuItem {
                label: "Manage Wallets".to_string(),
                description: "View and manage existing wallets".to_string(),
                screen: Screen::ManageWallets,
                enabled: true,
            },
            MenuItem {
                label: "Join Session".to_string(),
                description: "Join an existing DKG or signing session".to_string(),
                screen: Screen::JoinSession,
                enabled: true,
            },
            MenuItem {
                label: "Sign Transaction".to_string(),
                description: "Sign a transaction with selected wallet".to_string(),
                screen: Screen::ManageWallets, // Will navigate to wallet selection first
                enabled: true,
            },
            MenuItem {
                label: "Settings".to_string(),
                description: "Configure network and security settings".to_string(),
                screen: Screen::Settings,
                enabled: true,
            },
            MenuItem {
                label: "Exit".to_string(),
                description: "Exit the application".to_string(),
                screen: Screen::About, // Not used, handled separately
                enabled: true,
            },
        ];
        
        let mut props = Props::default();
        props.set(tuirealm::props::Attribute::Title, tuirealm::props::AttrValue::String("MPC Wallet - Main Menu".to_string()));
        // Set borders - tuirealm doesn't have Borders::ALL, so we use default
        
        Self {
            props,
            items,
            selected: 0,
            focused: false,
        }
    }
    
    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }
    
    fn move_down(&mut self) {
        if self.selected < self.items.len() - 1 {
            self.selected += 1;
        }
    }
    
    fn select_current(&self) -> Option<Message> {
        if let Some(item) = self.items.get(self.selected) {
            if item.enabled {
                // Special handling for Exit
                if item.label == "Exit" {
                    Some(Message::Quit)
                } else {
                    Some(Message::Navigate(item.screen.clone()))
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl MockComponent for MainMenu {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        // Create list items with styling
        let items: Vec<ListItem> = self.items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.selected {
                    if self.focused {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(TextModifiers::BOLD)
                    } else {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(TextModifiers::BOLD)
                    }
                } else if !item.enabled {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::Gray)
                };
                
                let prefix = if i == self.selected { "► " } else { "  " };
                let text = format!("{}{}", prefix, item.label);
                
                ListItem::new(text).style(style)
            })
            .collect();
        
        // Create the list widget
        let mut list_state = ListState::default();
        list_state.select(Some(self.selected));
        
        let list = List::new(items)
            .block(
                Block::default()
                    .title("MPC Wallet Terminal Interface")
                    .borders(TuiBorders::ALL)
                    .border_type(TuiBorderType::Rounded)
                    .border_style(if self.focused {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::Gray)
                    })
            )
            .highlight_style(Style::default().bg(Color::DarkGray));
        
        frame.render_stateful_widget(list, area, &mut list_state);
        
        // Render description at the bottom
        if let Some(item) = self.items.get(self.selected) {
            let desc_area = Rect {
                x: area.x + 2,
                y: area.y + area.height - 3,
                width: area.width - 4,
                height: 1,
            };
            
            let description = ratatui::widgets::Paragraph::new(item.description.as_str())
                .style(Style::default().fg(Color::DarkGray));
            
            frame.render_widget(description, desc_area);
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

impl Component<Message, UserEvent> for MainMenu {
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
                code: Key::Char('q'),
                modifiers: KeyModifiers::CONTROL,
            }) => {
                Some(Message::Quit)
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

impl MpcWalletComponent for MainMenu {
    fn id(&self) -> Id {
        Id::MainMenu
    }
    
    fn is_visible(&self) -> bool {
        true
    }
    
    fn on_focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}