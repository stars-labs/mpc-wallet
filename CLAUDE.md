# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
# Rust (all workspace crates)
cargo build                              # Build all workspace members
cargo test                               # Run all Rust tests
cargo test -p mpc-wallet-frost-core      # Test specific package
cargo test -p tui-node                   # Test TUI node
cargo test test_name                     # Run single test by name
cargo run --example unified_dkg -p mpc-wallet-frost-core  # Run example
cargo run --bin mpc-wallet-tui -p tui-node                # Run TUI app
cargo check                              # Fast type check without codegen

# Browser extension (Bun + WASM)
bun install                              # Install JS dependencies
bun run build:wasm                       # Build WASM bindings (required first)
bun run dev                              # Dev server with hot reload
bun run build                            # Production build
bun test                                 # Run JS/TS tests
bun test path/to/test.ts                 # Run single test file
```

## Architecture

Rust monorepo (edition 2024) with a Bun-managed browser extension. Seven Cargo workspace members:

### Core Library: `packages/@mpc-wallet/frost-core/`
Shared FROST cryptographic implementation used by all Rust targets. Key modules:
- `unified_dkg.rs` — Runs FROST DKG for ed25519 + secp256k1 simultaneously from a single root secret
- `hd_derivation.rs` — BIP-44 style HD key derivation using additive scalar offsets (no extra DKG rounds)
- `traits.rs` — `FrostCurve` trait abstracting over curve operations
- `ed25519.rs` / `secp256k1.rs` — Curve implementations (Solana addresses, Ethereum addresses)
- `keystore.rs` — Encrypted key share storage (PBKDF2 + AES-256-GCM)
- `root_secret.rs` — Root entropy → deterministic per-curve RNGs via HKDF

### Applications
- **`apps/tui-node/`** — Terminal UI (Ratatui) with Elm architecture (`src/elm/` for Model/Update/View). Exposes `lib.rs` so native-node can reuse business logic. Supports online (WebRTC mesh) and offline (SD card air-gap) DKG modes.
- **`apps/native-node/`** — Desktop GUI (Slint) consuming tui-node as a library via `UIProvider` trait
- **`apps/browser-extension/`** — Chrome/Firefox extension (WXT + Svelte 5). Manifest V3 with background worker, popup, offscreen document (WebRTC + WASM), content script.
- **`apps/signal-server/`** — WebRTC signaling: standard WebSocket server + Cloudflare Worker variant

### WASM & Blockchain
- **`packages/@mpc-wallet/core-wasm/`** — Thin `wasm-bindgen` wrapper around frost-core
- **`packages/@mpc-wallet/blockchain/`** — Multi-chain support (solana-sdk, ethers, bitcoin crate)

## Key Patterns

**FROST ciphersuite type names**: `frost_ed25519::Ed25519Sha512` and `frost_secp256k1::Secp256K1Sha256` (note capital K in Secp256K1).

**frost-core internal types**: `SigningShare::new()`, `VerifyingShare::new()`, `VerifyingKey::new()` are `pub(crate)`. To construct these from outside frost-core, use `serialize()` / `deserialize()` round-trips through `Field::serialize`/`Group::serialize`.

**UIProvider trait** (`apps/tui-node/`): Abstracts UI backend so TUI (Ratatui) and native (Slint) share the same DKG/signing/network logic.

**Elm architecture** in TUI: State is `Model`, transitions via `Update`, rendering via `View`. Event-driven through `InternalCommand<C>` enum.

## Dependencies

FROST: `frost-core` 2.2.0, `frost-ed25519` 2.2.0, `frost-secp256k1` 2.2.0 (ZCash implementations).
Crypto: `sha2`, `sha3`, `hmac`, `hkdf`, `aes-gcm`, `argon2`, `k256`, `ed25519-dalek`.
Dev environment: Nix flake (`nix develop`) provides all system deps including graphics libs.

## Workspace Layout

```
Cargo.toml              # Workspace root, resolver = "2"
package.json            # Bun monorepo (browser extension)
flake.nix               # Nix dev environment (Linux + macOS)
apps/
  tui-node/             # Rust binary + library
  native-node/          # Rust binary (Slint GUI)
  signal-server/        # server/ + cloudflare-worker/
  browser-extension/    # WXT + Svelte + TailwindCSS
packages/@mpc-wallet/
  frost-core/           # Core crypto library
  core-wasm/            # WASM bindings
  blockchain/           # Chain integrations
```
