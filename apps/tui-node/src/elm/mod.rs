//! Elm Architecture implementation for the TUI
//! 
//! This module implements the Elm Architecture pattern using tui-realm,
//! providing a clean, functional approach to building the terminal interface
//! with predictable state management.

pub mod model;
pub mod message;
pub mod update;
pub mod command;
pub mod app;
pub mod components;
pub mod provider;

pub use model::{Model, Screen, UIState, WalletState, NetworkState};
pub use message::Message;
pub use update::update;
pub use command::Command;
pub use app::ElmApp;
pub use provider::{UIProvider, NoOpUIProvider, WalletDisplayInfo};