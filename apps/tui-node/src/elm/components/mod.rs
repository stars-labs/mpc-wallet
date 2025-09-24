//! TUI Components using tui-realm
//!
//! This module contains all UI components implemented using the tui-realm framework,
//! following the Elm Architecture pattern.

// Core UI components
pub mod main_menu;
pub mod create_wallet;
pub mod wallet_list;
pub mod wallet_detail;
pub mod modal;
pub mod notification;

// Professional wallet creation flow components
pub mod mode_selection;
pub mod curve_selection;
pub mod threshold_config;
pub mod join_session;

// DKG components
pub mod dkg_progress;
pub mod offline_dkg_process;
pub mod sd_card_manager;

// Main exports
pub use main_menu::MainMenu;
pub use create_wallet::CreateWalletComponent;
pub use wallet_list::WalletList;
pub use wallet_detail::WalletDetail;
pub use modal::ModalComponent;
pub use notification::NotificationBar;

// Professional wallet creation flow components
pub use mode_selection::ModeSelectionComponent;
pub use curve_selection::CurveSelectionComponent;
pub use threshold_config::ThresholdConfigComponent;
pub use join_session::JoinSessionComponent;

// DKG components
pub use dkg_progress::DKGProgressComponent;
pub use offline_dkg_process::{OfflineDKGProcessComponent, ParticipantRole};
pub use sd_card_manager::SDCardManagerComponent;

use tuirealm::Component;

/// Trait for MPC wallet components
pub trait MpcWalletComponent: Component<crate::elm::message::Message, UserEvent> {
    /// Get the component's ID
    fn id(&self) -> Id;
    
    /// Check if the component should be visible
    fn is_visible(&self) -> bool;
    
    /// Handle focus change
    fn on_focus(&mut self, focused: bool);
}

/// Component IDs for the view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Id {
    MainMenu,
    WalletList,
    WalletDetail,
    CreateWallet,
    Modal, // Alias for ModalDialog
    ModalDialog,
    NotificationBar,
    ModeSelection,
    CurveSelection,
    ThresholdConfig,
    JoinSession,
    OfflineDKGProcess,
    DKGProgress,
    SDCardManager,
}

/// User events emitted by components
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserEvent {
    MenuItemSelected(usize),
    WalletSelected(usize),
    CreateWalletRequested,
    NavigateBack,
    Quit,
    ModalConfirm,
    ModalCancel,
    FocusGained,
    FocusLost,
}