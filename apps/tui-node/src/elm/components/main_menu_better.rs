//! Better-looking Main Menu Component
//!
//! Enhanced main menu with better visual styling

use crate::elm::components::{Id, UserEvent, MpcWalletComponent};
use crate::elm::message::Message;
use crate::elm::model::Screen;

use tuirealm::command::{Cmd, CmdResult};
use tuirealm::event::{Event, Key, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Padding};
use tuirealm::{Component, Frame, MockComponent, Props, State, StateValue};

/// Better-looking main menu component
#[derive(Debug, Clone)]
pub struct BetterMainMenu {
    props: Props,
    items: Vec<MenuItem>,
    selected: usize,
    focused: bool,
}

#[derive(Debug, Clone)]
struct MenuItem {
    icon: &'static str,
    label: String,
    description: String,
    screen: Screen,
    enabled: bool,
}

impl Default for BetterMainMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl BetterMainMenu {
    pub fn new() -> Self {
        Self::with_wallet_count(0)
    }
    
    pub fn with_wallet_count(wallet_count: usize) -> Self {
        let mut items = vec![
            MenuItem {
                icon: "🆕",
                label: "Create New Wallet".to_string(),
                description: "Start DKG to create a new MPC wallet".to_string(),
                screen: Screen::CreateWallet(Default::default()),
                enabled: true,
            },
            MenuItem {
                icon: "🔗",
                label: "Join Session".to_string(),
                description: "Join an existing DKG or signing session".to_string(),
                screen: Screen::JoinSession,
                enabled: true,
            },
        ];
        
        // Only show these options if wallets exist
        if wallet_count > 0 {
            items.push(MenuItem {
                icon: "📁",
                label: "Manage Wallets".to_string(),
                description: format!("View and manage {} existing wallet{}", wallet_count, if wallet_count == 1 { "" } else { "s" }),
                screen: Screen::ManageWallets,
                enabled: true,
            });
            
            items.push(MenuItem {
                icon: "✍️",
                label: "Sign Transaction".to_string(),
                description: "Sign a transaction with selected wallet".to_string(),
                screen: Screen::ManageWallets, // Will navigate to wallet selection first
                enabled: true,
            });
        }
        
        // Always show Settings and Exit
        items.push(MenuItem {
            icon: "⚙️",
            label: "Settings".to_string(),
            description: "Configure network and security settings".to_string(),
            screen: Screen::Settings,
            enabled: true,
        });
        
        items.push(MenuItem {
            icon: "🚪",
            label: "Exit".to_string(),
            description: "Exit the application".to_string(),
            screen: Screen::About, // Not used, handled separately
            enabled: true,
        });
        
        let props = Props::default();
        
        Self {
            props,
            items,
            selected: 0,
            focused: false,
        }
    }
    
    /// Update the selected index
    pub fn set_selected(&mut self, index: usize) {
        self.selected = index.min(self.items.len().saturating_sub(1));
    }
    
    fn move_up(&mut self) {
        let old_selected = self.selected;
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            // Wrap around to bottom
            self.selected = self.items.len().saturating_sub(1);
        }
        tracing::debug!("MainMenu::move_up: {} -> {}", old_selected, self.selected);
    }
    
    fn move_down(&mut self) {
        let old_selected = self.selected;
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
        } else {
            // Wrap around to top
            self.selected = 0;
        }
        tracing::debug!("MainMenu::move_down: {} -> {}", old_selected, self.selected);
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

impl MockComponent for BetterMainMenu {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        use ratatui::style::{Color as RatatuiColor, Modifier, Style as RatatuiStyle};
        
        tracing::debug!("🎨 MainMenu render - selected: {}, focused: {}", self.selected, self.focused);
        
        // Create list items with better styling
        let items: Vec<ListItem> = self.items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.selected {
                    if self.focused {
                        RatatuiStyle::default()
                            .fg(RatatuiColor::Yellow)
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::UNDERLINED)
                    } else {
                        RatatuiStyle::default()
                            .fg(RatatuiColor::White)
                            .add_modifier(Modifier::BOLD)
                    }
                } else if !item.enabled {
                    RatatuiStyle::default().fg(RatatuiColor::DarkGray)
                } else {
                    RatatuiStyle::default().fg(RatatuiColor::Gray)
                };
                
                let prefix = if i == self.selected { 
                    "  ► " 
                } else { 
                    "    " 
                };
                
                let text = format!("{}{} {}", prefix, item.icon, item.label);
                
                ListItem::new(text).style(style)
            })
            .collect();
        
        // Create the list widget with better styling
        let mut list_state = ListState::default();
        list_state.select(Some(self.selected));
        
        let list = List::new(items)
            .block(
                Block::default()
                    .title(" 🎮 MPC Wallet Terminal Interface ")
                    .title_style(RatatuiStyle::default()
                        .fg(RatatuiColor::Cyan)
                        .add_modifier(Modifier::BOLD))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(if self.focused {
                        RatatuiStyle::default().fg(RatatuiColor::Cyan)
                    } else {
                        RatatuiStyle::default().fg(RatatuiColor::Gray)
                    })
                    .padding(Padding::symmetric(1, 1))
            )
            .highlight_style(RatatuiStyle::default()
                .bg(RatatuiColor::Rgb(40, 40, 40)));
        
        frame.render_stateful_widget(list, area, &mut list_state);
        
        // Render description at the bottom
        if let Some(item) = self.items.get(self.selected) {
            let desc_area = Rect {
                x: area.x + 6,
                y: area.y + area.height - 3,
                width: area.width - 8,
                height: 1,
            };
            
            use ratatui::widgets::Paragraph;
            
            let description = Paragraph::new(format!("💡 {}", item.description))
                .style(RatatuiStyle::default()
                    .fg(RatatuiColor::Rgb(150, 150, 150))
                    .add_modifier(Modifier::ITALIC));
            
            frame.render_widget(description, desc_area);
        }
        
        // Render help text at the very bottom
        let help_area = Rect {
            x: area.x + 6,
            y: area.y + area.height - 2,
            width: area.width - 8,
            height: 1,
        };
        
        use ratatui::widgets::Paragraph;
        let help = Paragraph::new("↑↓ Navigate  •  Enter Select  •  Esc Back  •  Ctrl+Q Exit")
            .style(RatatuiStyle::default()
                .fg(RatatuiColor::DarkGray)
                .add_modifier(Modifier::DIM));
        
        frame.render_widget(help, help_area);
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

impl Component<Message, UserEvent> for BetterMainMenu {
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

impl MpcWalletComponent for BetterMainMenu {
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