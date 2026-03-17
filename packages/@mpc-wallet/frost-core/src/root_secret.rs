//! Root secret module for unified multi-curve key derivation.
//!
//! A single 32-byte root secret is used to deterministically derive
//! curve-specific RNG seeds via HKDF. This ensures that one DKG session
//! can produce key packages for both ed25519 and secp256k1 from the
//! same entropy source.

use crate::errors::{FrostError, Result};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use rand::RngCore;
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;
use sha2::Sha256;

const ROOT_SECRET_LEN: usize = 32;

/// A 32-byte root secret from which curve-specific DKG randomness is derived.
#[derive(Clone)]
pub struct RootSecret([u8; ROOT_SECRET_LEN]);

impl RootSecret {
    /// Generate a new root secret from OS randomness.
    pub fn generate() -> Self {
        let mut bytes = [0u8; ROOT_SECRET_LEN];
        OsRng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    /// Create a root secret from raw bytes.
    pub fn from_bytes(bytes: [u8; ROOT_SECRET_LEN]) -> Self {
        Self(bytes)
    }

    /// Get the raw bytes of the root secret.
    pub fn as_bytes(&self) -> &[u8; ROOT_SECRET_LEN] {
        &self.0
    }

    /// Derive a deterministic ChaCha20Rng for a specific curve.
    ///
    /// Uses HKDF-SHA256 with the curve name as info to produce a 32-byte
    /// seed, which is then used to create a ChaCha20Rng.
    pub fn derive_rng(&self, curve_tag: &str) -> Result<ChaCha20Rng> {
        let hk = Hkdf::<Sha256>::new(None, &self.0);
        let mut seed = [0u8; 32];
        hk.expand(curve_tag.as_bytes(), &mut seed)
            .map_err(|e| FrostError::DkgError(format!("HKDF expand failed: {}", e)))?;
        Ok(ChaCha20Rng::from_seed(seed))
    }

    /// Derive a deterministic RNG for the ed25519 curve DKG.
    pub fn derive_ed25519_rng(&self) -> Result<ChaCha20Rng> {
        self.derive_rng("frost-dkg/ed25519")
    }

    /// Derive a deterministic RNG for the secp256k1 curve DKG.
    pub fn derive_secp256k1_rng(&self) -> Result<ChaCha20Rng> {
        self.derive_rng("frost-dkg/secp256k1")
    }
}

impl Drop for RootSecret {
    fn drop(&mut self) {
        // Zeroize on drop for security
        self.0.fill(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_derivation() {
        let secret = RootSecret::from_bytes([42u8; 32]);
        let mut rng1 = secret.derive_ed25519_rng().unwrap();
        let mut rng2 = secret.derive_ed25519_rng().unwrap();

        let mut buf1 = [0u8; 32];
        let mut buf2 = [0u8; 32];
        rng1.fill_bytes(&mut buf1);
        rng2.fill_bytes(&mut buf2);

        assert_eq!(buf1, buf2, "Same root secret must produce same RNG output");
    }

    #[test]
    fn test_different_curves_produce_different_rngs() {
        let secret = RootSecret::from_bytes([42u8; 32]);
        let mut ed_rng = secret.derive_ed25519_rng().unwrap();
        let mut secp_rng = secret.derive_secp256k1_rng().unwrap();

        let mut buf_ed = [0u8; 32];
        let mut buf_secp = [0u8; 32];
        ed_rng.fill_bytes(&mut buf_ed);
        secp_rng.fill_bytes(&mut buf_secp);

        assert_ne!(buf_ed, buf_secp, "Different curves must produce different RNG output");
    }
}
