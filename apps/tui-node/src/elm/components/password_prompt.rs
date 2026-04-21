//! Password Prompt Component — captures the wallet-encryption password
//! before DKG starts.
//!
//! **Substep 1.2 state**: placeholder only. Renders a WIP notice so the
//! user can visually confirm the mount branch works and Esc actually
//! returns to the previous screen. The real two-field input + validation
//! lands in Substep 1.3.
//!
//! Modeled after `wallet_detail.rs` — same minimal `Component` /
//! `AppComponent` / `MpcWalletComponent` triad the other leaf screens
//! use.

use crate::elm::components::{Id, MpcWalletComponent, UserEvent};
use crate::elm::message::Message;
use ratatui::layout::{Alignment, Rect};
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, Key, KeyEvent};
use tuirealm::props::Props;
use tuirealm::ratatui::Frame;
use tuirealm::state::State;

#[derive(Debug, Clone, Default)]
pub struct PasswordPromptComponent {
    props: Props,
    focused: bool,
}

impl PasswordPromptComponent {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for PasswordPromptComponent {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        use ratatui::style::{Color, Style};
        use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

        let body = [
            "",
            "🔐  Password Prompt (WIP)",
            "",
            "Substep 1.2 placeholder — the two-field input + validation",
            "lands in Substep 1.3.",
            "",
            "Press Esc to go back.",
        ]
        .join("\n");

        let widget = Paragraph::new(body)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .title(" Set Wallet Password ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Yellow)),
            );

        frame.render_widget(widget, area);
    }

    fn query<'a>(
        &'a self,
        attr: tuirealm::props::Attribute,
    ) -> Option<tuirealm::props::QueryResult<'a>> {
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

impl AppComponent<Message, UserEvent> for PasswordPromptComponent {
    fn on(&mut self, event: &Event<UserEvent>) -> Option<Message> {
        // Esc is the only interaction the placeholder handles. Actual input
        // handling (typing, Tab between fields, Enter to submit) moves in
        // in Substep 1.3 once the real input widget is in place.
        match event {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Message::NavigateBack),
            _ => None,
        }
    }
}

impl MpcWalletComponent for PasswordPromptComponent {
    fn id(&self) -> Id {
        Id::PasswordPrompt
    }

    fn is_visible(&self) -> bool {
        true
    }

    fn on_focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}
