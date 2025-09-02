//! Wallet templates and auto-generation utilities

use chrono::Local;
use rand::Rng;

/// Predefined wallet configuration templates
#[derive(Debug, Clone)]
pub struct WalletTemplate {
    pub name: &'static str,
    pub description: &'static str,
    pub total: u16,
    pub threshold: u16,
    pub security_level: &'static str,
    pub use_case: &'static str,
    pub color: ratatui::style::Color,
}

/// Standard wallet templates for quick setup
pub const WALLET_TEMPLATES: &[WalletTemplate] = &[
    WalletTemplate {
        name: "2-of-3 Standard",
        description: "Balanced security and convenience",
        total: 3,
        threshold: 2,
        security_level: "Standard",
        use_case: "Personal wallet, small team",
        color: ratatui::style::Color::Green,
    },
    WalletTemplate {
        name: "3-of-5 High Security",
        description: "Enhanced security for business use",
        total: 5,
        threshold: 3,
        security_level: "High",
        use_case: "Business operations, treasury",
        color: ratatui::style::Color::Yellow,
    },
    WalletTemplate {
        name: "2-of-2 Dual Control",
        description: "Both parties must approve",
        total: 2,
        threshold: 2,
        security_level: "Maximum",
        use_case: "Partnership, joint custody",
        color: ratatui::style::Color::Red,
    },
    WalletTemplate {
        name: "5-of-9 Enterprise",
        description: "Board-level approvals",
        total: 9,
        threshold: 5,
        security_level: "Enterprise",
        use_case: "Large organization, DAO",
        color: ratatui::style::Color::Magenta,
    },
    WalletTemplate {
        name: "Custom Setup",
        description: "Configure your own threshold",
        total: 0, // Will be set by user
        threshold: 0, // Will be set by user
        security_level: "Custom",
        use_case: "Advanced users",
        color: ratatui::style::Color::Cyan,
    },
];

/// Cool themed prefixes for wallet names
const WALLET_THEMES: &[&str] = &[
    "Phoenix", "Nexus", "Quantum", "Cipher", "Vault", 
    "Shield", "Fortress", "Guardian", "Sentinel", "Apex",
    "Prime", "Atlas", "Cosmos", "Nova", "Zenith",
    "Aurora", "Chronos", "Eclipse", "Horizon", "Nebula",
];

/// Generate a themed wallet name like "MPC-Phoenix-042"
pub fn generate_themed_wallet_name() -> String {
    let mut rng = rand::rng();
    let theme_index = rng.random_range(0..WALLET_THEMES.len());
    let theme = WALLET_THEMES.get(theme_index).unwrap_or(&"Vault");
    let number: u16 = rng.random::<u16>() % 1000;
    format!("MPC-{}-{:03}", theme, number)
}

/// Generate a date-based wallet name like "Wallet-2024-08-16-1430"
pub fn generate_wallet_name() -> String {
    let now = Local::now();
    format!("Wallet-{}", now.format("%Y-%m-%d-%H%M"))
}

/// Generate a simple incremental name like "Wallet-001"
pub fn generate_simple_wallet_name(index: u32) -> String {
    format!("Wallet-{:03}", index)
}

/// Get template by index
pub fn get_template(index: usize) -> Option<&'static WalletTemplate> {
    WALLET_TEMPLATES.get(index)
}

/// Check if template is custom
pub fn is_custom_template(index: usize) -> bool {
    index == WALLET_TEMPLATES.len() - 1
}