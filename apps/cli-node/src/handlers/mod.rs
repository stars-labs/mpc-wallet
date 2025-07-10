//! Command handlers for internal commands in the MPC-FROST system
//!
//! This module contains separate handler functions for each InternalCommand type
//! to improve maintainability and readability of the codebase.

pub mod dkg_commands;
pub mod keystore_commands;
pub mod mesh_commands;
pub mod send_commands;
pub mod session_commands;
pub mod signing_commands;
pub mod extension_commands;
// pub mod offline_commands; // Temporarily disabled for browser compatibility focus

pub use dkg_commands::*;
pub use mesh_commands::*;
pub use send_commands::*;
pub use session_commands::*;
pub use signing_commands::*;
// pub use offline_commands::*; // Temporarily disabled

#[cfg(test)]
#[path = "signing_commands_test.rs"]
mod signing_commands_test;

#[cfg(test)]
#[path = "network_test.rs"]
mod network_test;
