//! Notification Bar Component - Displays notifications

use crate::elm::components::{Id, UserEvent, MpcWalletComponent};
use crate::elm::message::Message;
use crate::elm::model::Notification;
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::Event;
use tuirealm::ratatui::Frame;
use tuirealm::props::Props;
use tuirealm::state::State;
use tuirealm::command::{Cmd, CmdResult};
use ratatui::layout::Rect;

#[derive(Debug, Clone)]
pub struct NotificationBar {
    props: Props,
    notifications: Vec<Notification>,
    focused: bool,
}

impl Default for NotificationBar {
    fn default() -> Self {
        Self {
            props: Props::default(),
            notifications: Vec::new(),
            focused: false,
        }
    }
}

impl NotificationBar {
    pub fn set_notifications(&mut self, notifications: Vec<Notification>) {
        self.notifications = notifications;
    }
}

impl Component for NotificationBar {
    fn view(&mut self, _frame: &mut Frame, _area: Rect) {
        // Notification rendering will be implemented
    }
    
    fn query<'a>(&'a self, attr: tuirealm::props::Attribute) -> Option<tuirealm::props::QueryResult<'a>> {
        self.props.get_for_query(attr)
    }
    
    fn attr(&mut self, attr: tuirealm::props::Attribute, value: tuirealm::props::AttrValue) {
        self.props.set(attr, value);
    }
    
    fn state(&self) -> State {
        State::None
    }
    
    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::NoChange
    }
}

impl AppComponent<Message, UserEvent> for NotificationBar {
    fn on(&mut self, _event: &Event<UserEvent>) -> Option<Message> {
        None
    }
}

impl MpcWalletComponent for NotificationBar {
    fn id(&self) -> Id {
        Id::NotificationBar
    }
    
    fn is_visible(&self) -> bool {
        !self.notifications.is_empty()
    }
    
    fn on_focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}