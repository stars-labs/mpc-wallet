//! Styled Main Menu Component using tui-realm-stdlib
//!
//! A better-looking main menu using the stdlib components

use crate::elm::components::UserEvent;
use crate::elm::message::Message;
use crate::elm::model::Screen;

use tui_realm_stdlib::List;
use tuirealm::{Component, Event, MockComponent, NoUserEvent};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, Color, Style, TextModifiers, TextSpan};
use ratatui::layout::Rect;

/// Styled Main Menu using stdlib List
pub struct StyledMainMenu {
    component: List,
    items: Vec<MenuItem>,
}

#[derive(Debug, Clone)]
struct MenuItem {
    label: String,
    description: String,
    screen: Screen,
}

impl Default for StyledMainMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl StyledMainMenu {
    pub fn new() -> Self {
        let items = vec![
            MenuItem {
                label: "🆕 Create New Wallet".to_string(),
                description: "Start DKG to create a new MPC wallet".to_string(),
                screen: Screen::CreateWallet(Default::default()),
            },
            MenuItem {
                label: "📁 Manage Wallets".to_string(),
                description: "View and manage existing wallets".to_string(),
                screen: Screen::ManageWallets,
            },
            MenuItem {
                label: "🔗 Join Session".to_string(),
                description: "Join an existing DKG or signing session".to_string(),
                screen: Screen::JoinSession,
            },
            MenuItem {
                label: "✍️ Sign Transaction".to_string(),
                description: "Sign a transaction with selected wallet".to_string(),
                screen: Screen::ManageWallets,
            },
            MenuItem {
                label: "⚙️ Settings".to_string(),
                description: "Configure network and security settings".to_string(),
                screen: Screen::Settings,
            },
            MenuItem {
                label: "🚪 Exit".to_string(),
                description: "Exit the application".to_string(),
                screen: Screen::About,
            },
        ];
        
        // Create the list component with nice styling
        let rows: Vec<Vec<TextSpan>> = items.iter()
            .map(|item| vec![TextSpan::new(item.label.clone())])
            .collect();
        
        // Build props manually since ListPropsBuilder might not exist
        let mut props = tuirealm::props::Props::default();
        props.set(tuirealm::props::Attribute::Title, 
            tuirealm::props::AttrValue::Title((String::from("🎮 MPC Wallet Terminal Interface"), Alignment::Center)));
        props.set(tuirealm::props::Attribute::Borders, 
            tuirealm::props::AttrValue::Borders(
                Borders::default()
                    .modifiers(tuirealm::props::BorderSides::ALL)
                    .style(BorderType::Rounded)
                    .color(Color::Cyan)
            ));
        props.set(tuirealm::props::Attribute::HighlightedStr, 
            tuirealm::props::AttrValue::String(String::from("► ")));
        props.set(tuirealm::props::Attribute::FocusStyle, 
            tuirealm::props::AttrValue::Style(Style::default().fg(Color::Yellow).add_modifier(TextModifiers::BOLD)));
        props.set(tuirealm::props::Attribute::Style, 
            tuirealm::props::AttrValue::Style(Style::default().fg(Color::Gray)));
        props.set(tuirealm::props::Attribute::Rows, 
            tuirealm::props::AttrValue::Table(rows));
        
        let component = List::new(props);
        
        Self {
            component,
            items,
        }
    }
    
    pub fn set_selected(&mut self, index: usize) {
        // Update the component's selected index
        self.component.attr(
            tuirealm::Attribute::Value,
            tuirealm::AttrValue::Payload(tuirealm::props::PropPayload::One(tuirealm::props::PropValue::Usize(index)))
        );
    }
}

impl Component<Message, UserEvent> for StyledMainMenu {
    fn on(&mut self, event: Event<UserEvent>) -> Option<Message> {
        // Convert our event to NoUserEvent for the List component
        let stdlib_event = match event.clone() {
            Event::Keyboard(key) => Event::<NoUserEvent>::Keyboard(key),
            Event::WindowResize(w, h) => Event::<NoUserEvent>::WindowResize(w, h),
            Event::FocusGained => Event::<NoUserEvent>::FocusGained,
            Event::FocusLost => Event::<NoUserEvent>::FocusLost,
            Event::Paste(s) => Event::<NoUserEvent>::Paste(s),
            Event::Key(k) => Event::<NoUserEvent>::Key(k),
            Event::Mouse(m) => Event::<NoUserEvent>::Mouse(m),
            Event::Tick => Event::<NoUserEvent>::Tick,
            Event::User(_) => Event::<NoUserEvent>::None,
            Event::None => Event::<NoUserEvent>::None,
        };
        
        // Let the List component handle the event for navigation
        let _ = self.component.on(stdlib_event);
        
        // Then handle our specific actions
        match event {
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => {
                // Get the selected index from the component
                if let Some(idx) = self.component.state().unwrap_one().unwrap_usize() {
                    if let Some(item) = self.items.get(idx) {
                        if item.label.contains("Exit") {
                            return Some(Message::Quit);
                        } else {
                            return Some(Message::Navigate(item.screen.clone()));
                        }
                    }
                }
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Esc,
                modifiers: KeyModifiers::NONE,
            }) => {
                Some(Message::NavigateBack)
            }
            _ => None
        }
    }
}

impl MockComponent for StyledMainMenu {
    fn view(&mut self, frame: &mut tuirealm::Frame, area: Rect) {
        // Render the list component
        self.component.view(frame, area);
        
        // Add description at the bottom
        if let Some(idx) = self.component.state().unwrap_one().unwrap_usize() {
            if let Some(item) = self.items.get(idx) {
                let desc_area = Rect {
                    x: area.x + 2,
                    y: area.y + area.height.saturating_sub(2),
                    width: area.width.saturating_sub(4),
                    height: 1,
                };
                
                use ratatui::widgets::Paragraph;
                use ratatui::style::{Color as RatatuiColor, Style as RatatuiStyle};
                
                let description = Paragraph::new(item.description.as_str())
                    .style(RatatuiStyle::default().fg(RatatuiColor::DarkGray));
                
                frame.render_widget(description, desc_area);
            }
        }
    }

    fn query(&self, attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        self.component.query(attr)
    }
    
    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {
        self.component.attr(attr, value);
    }
    
    fn state(&self) -> tuirealm::State {
        self.component.state()
    }
    
    fn perform(&mut self, cmd: tuirealm::command::Cmd) -> tuirealm::command::CmdResult {
        self.component.perform(cmd)
    }
}