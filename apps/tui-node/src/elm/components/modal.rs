//! Modal Component - Displays modal dialogs

use crate::elm::components::{Id, UserEvent, MpcWalletComponent};
use crate::elm::message::Message;
use crate::elm::model::Modal;
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::Event;
use tuirealm::ratatui::Frame;
use tuirealm::props::Props;
use tuirealm::state::State;
use tuirealm::command::{Cmd, CmdResult};
use ratatui::layout::Rect;

#[derive(Debug, Clone)]
pub struct ModalComponent {
    props: Props,
    modal: Option<Modal>,
    focused: bool,
}

impl Default for ModalComponent {
    fn default() -> Self {
        Self {
            props: Props::default(),
            modal: None,
            focused: false,
        }
    }
}

impl ModalComponent {
    pub fn set_modal(&mut self, modal: Option<Modal>) {
        self.modal = modal;
    }
}

impl Component for ModalComponent {
    fn view(&mut self, _frame: &mut Frame, _area: Rect) {
        // Modal rendering will be implemented
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

impl AppComponent<Message, UserEvent> for ModalComponent {
    fn on(&mut self, _event: &Event<UserEvent>) -> Option<Message> {
        None
    }
}

impl MpcWalletComponent for ModalComponent {
    fn id(&self) -> Id {
        Id::Modal
    }
    
    fn is_visible(&self) -> bool {
        self.modal.is_some()
    }
    
    fn on_focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}