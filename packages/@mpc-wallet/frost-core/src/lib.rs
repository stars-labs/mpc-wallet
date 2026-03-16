// Core FROST implementation shared between WASM and CLI

pub mod traits;
pub mod ed25519;
pub mod secp256k1;
pub mod keystore;
pub mod errors;
pub mod root_secret;
pub mod unified_dkg;

// Re-export main types
pub use traits::FrostCurve;
pub use errors::{FrostError, Result};
pub use keystore::{Keystore, KeystoreData, MultiCurveKeystoreData};

// Re-export curve implementations
pub use ed25519::Ed25519Curve;
pub use secp256k1::Secp256k1Curve;

// Re-export unified DKG types
pub use root_secret::RootSecret;
pub use unified_dkg::UnifiedDkg;