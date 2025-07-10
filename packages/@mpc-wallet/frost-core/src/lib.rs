// Core FROST implementation shared between WASM and CLI

pub mod traits;
pub mod ed25519;
pub mod secp256k1;
pub mod keystore;
pub mod errors;

// Re-export main types
pub use traits::FrostCurve;
pub use errors::{FrostError, Result};
pub use keystore::{Keystore, KeystoreData};

// Re-export curve implementations
pub use ed25519::Ed25519Curve;
pub use secp256k1::Secp256k1Curve;