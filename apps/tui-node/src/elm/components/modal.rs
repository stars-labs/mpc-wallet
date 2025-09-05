//! Modal Component - Displays modal dialogs

use crate::elm::components::{Id, UserEvent, MpcWalletComponent};
use crate::elm::message::Message;
use crate::elm::model::Modal;
use tuirealm::{Component, Event, Frame, MockComponent, Props, State};
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

impl MockComponent for ModalComponent {
    fn view(&mut self, _frame: &mut Frame, _area: Rect) {
        // Modal rendering will be implemented
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

impl Component<Message, UserEvent> for ModalComponent {
    fn on(&mut self, _event: Event<UserEvent>) -> Option<Message> {
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