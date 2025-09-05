//! TUI Components using tui-realm
//!
//! This module contains all UI components implemented using the tui-realm framework,
//! following the Elm Architecture pattern.

pub mod main_menu;
pub mod main_menu_better;
pub mod main_menu_professional;
pub mod wallet_list;
pub mod wallet_detail;
pub mod create_wallet;
pub mod create_wallet_styled;
pub mod modal;
pub mod notification;

// Professional wallet creation and join components
pub mod mode_selection;
pub mod curve_selection;
pub mod threshold_config;
pub mod join_session;

// Offline DKG components
pub mod offline_dkg_process;
pub mod sd_card_manager;

// Use the professional-styled versions by default
pub use main_menu_professional::ProfessionalMainMenu as MainMenu;
pub use create_wallet_styled::StyledCreateWalletComponent as CreateWalletComponent;

// Professional wallet creation flow components
pub use mode_selection::ModeSelectionComponent;
pub use curve_selection::CurveSelectionComponent;
pub use threshold_config::ThresholdConfigComponent;
pub use join_session::JoinSessionComponent;

// Offline DKG components
pub use offline_dkg_process::OfflineDKGProcessComponent;
pub use sd_card_manager::SDCardManagerComponent;

// Keep legacy components available for fallback
pub use main_menu_better::BetterMainMenu;
pub use create_wallet::CreateWalletComponent as BasicCreateWalletComponent;
pub use wallet_list::WalletList;
pub use wallet_detail::WalletDetail;
pub use modal::ModalComponent;
pub use notification::NotificationBar;

use tuirealm::Component;

/// Component IDs for the view
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Id {
    MainMenu,
    WalletList,
    WalletDetail,
    CreateWallet,
    Modal,
    NotificationBar,
    InputField,
    SessionList,
    DKGProgress,
    SigningProgress,
}

/// Common trait for all our components
pub trait MpcWalletComponent: Component<crate::elm::message::Message, UserEvent> {
    /// Get the component's ID
    fn id(&self) -> Id;
    
    /// Check if the component should be visible
    fn is_visible(&self) -> bool;
    
    /// Handle focus change
    fn on_focus(&mut self, focused: bool);
}

/// User events that can be sent to components
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserEvent {
    FocusGained,
    FocusLost,
    Selected(usize),
    InputChanged(String),
    ScrollUp,
    ScrollDown,
    Refresh,
}