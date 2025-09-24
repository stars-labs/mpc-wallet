//! FROST DKG Protocol Implementation
//!
//! This module implements the actual FROST Distributed Key Generation protocol
//! using the frost-core library, with WebRTC for message passing between participants.

pub mod dkg;

pub use dkg::{DKGCoordinator, DKGMessage, DKGRoundMessage};