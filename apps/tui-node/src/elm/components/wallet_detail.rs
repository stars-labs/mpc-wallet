//! Wallet Detail Component - Shows detailed wallet information

use crate::elm::components::{Id, UserEvent, MpcWalletComponent};
use crate::elm::message::Message;
use tuirealm::{Component, Event, Frame, MockComponent, Props, State};
use tuirealm::command::{Cmd, CmdResult};
use ratatui::layout::Rect;

#[derive(Debug, Clone)]
pub struct WalletDetail {
    props: Props,
    wallet_id: Option<String>,
    focused: bool,
}

impl Default for WalletDetail {
    fn default() -> Self {
        Self {
            props: Props::default(),
            wallet_id: None,
            focused: false,
        }
    }
}

impl WalletDetail {
    pub fn with_wallet_id(wallet_id: String) -> Self {
        Self {
            props: Props::default(),
            wallet_id: Some(wallet_id),
            focused: false,
        }
    }
}

impl MockComponent for WalletDetail {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        use ratatui::widgets::{Block, Borders, Paragraph};
        use ratatui::style::{Color, Style};
        
        let wallet_info = if let Some(ref id) = self.wallet_id {
            format!("Wallet Details\nID: {}", id)
        } else {
            "No wallet selected".to_string()
        };
        
        let widget = Paragraph::new(wallet_info)
            .block(Block::default()
                .title("Wallet Detail")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray)));
        
        frame.render_widget(widget, area);
    }
    
    fn query(&self, attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        self.props.get(attr)
    }
    
    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {
        self.props.set(attr, value);
    }
    
    fn state(&self) -> State {
        State::None
    }
    
    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }
}

impl Component<Message, UserEvent> for WalletDetail {
    fn on(&mut self, _event: Event<UserEvent>) -> Option<Message> {
        None
    }
}

impl MpcWalletComponent for WalletDetail {
    fn id(&self) -> Id {
        Id::WalletDetail
    }
    
    fn is_visible(&self) -> bool {
        true
    }
    
    fn on_focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}