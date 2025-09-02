//! Theme and color system for the TUI
//! Provides semantic colors and visual indicators for better UX

use ratatui::style::{Color, Modifier, Style};

/// Semantic color system for consistent visual language
pub struct ColorTheme {
    // Status colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub pending: Color,
    
    // UI colors
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub muted: Color,
    pub background: Color,
    pub surface: Color,
    
    // Semantic colors for specific features
    pub network: Color,
    pub crypto: Color,
    pub wallet: Color,
    pub security: Color,
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self {
            // Status colors
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,
            pending: Color::LightYellow,
            
            // UI colors
            primary: Color::White,
            secondary: Color::Gray,
            accent: Color::Cyan,
            muted: Color::DarkGray,
            background: Color::Black,
            surface: Color::Rgb(24, 24, 24),
            
            // Feature-specific colors
            network: Color::LightCyan,
            crypto: Color::LightMagenta,
            wallet: Color::LightBlue,
            security: Color::Magenta,
        }
    }
}

impl ColorTheme {
    /// Get a high contrast theme for better accessibility
    pub fn high_contrast() -> Self {
        Self {
            success: Color::LightGreen,
            warning: Color::LightYellow,
            error: Color::LightRed,
            info: Color::LightBlue,
            pending: Color::Yellow,
            
            primary: Color::White,
            secondary: Color::Rgb(192, 192, 192),  // Light gray
            accent: Color::LightCyan,
            muted: Color::Gray,
            background: Color::Black,
            surface: Color::Rgb(32, 32, 32),
            
            network: Color::Cyan,
            crypto: Color::Magenta,
            wallet: Color::Blue,
            security: Color::LightMagenta,
        }
    }
}

/// Visual indicators for status representation
pub struct StatusIndicators;

impl StatusIndicators {
    // Connection status
    pub const CONNECTED: &'static str = "🟢";
    pub const CONNECTING: &'static str = "🟡";
    pub const DISCONNECTED: &'static str = "🔴";
    pub const UNKNOWN: &'static str = "⚪";
    
    // Operation status
    pub const SUCCESS: &'static str = "✓";
    pub const IN_PROGRESS: &'static str = "⏳";
    pub const FAILED: &'static str = "✗";
    pub const WARNING: &'static str = "⚠";
    pub const INFO: &'static str = "ℹ";
    
    // Network indicators
    pub const NETWORK: &'static str = "🌐";
    pub const MESH_READY: &'static str = "🔗";
    pub const MESH_PARTIAL: &'static str = "⛓";
    
    // Security indicators
    pub const LOCKED: &'static str = "🔒";
    pub const UNLOCKED: &'static str = "🔓";
    pub const KEY: &'static str = "🔑";
    pub const SHIELD: &'static str = "🛡";
    
    // Wallet indicators
    pub const WALLET: &'static str = "💼";
    pub const HOT_WALLET: &'static str = "🔥";
    pub const COLD_WALLET: &'static str = "❄️";
    
    // Navigation
    pub const ARROW_RIGHT: &'static str = "→";
    pub const ARROW_LEFT: &'static str = "←";
    pub const ARROW_UP: &'static str = "↑";
    pub const ARROW_DOWN: &'static str = "↓";
    
    // ASCII fallbacks for limited terminals
    pub const ASCII_SUCCESS: &'static str = "[OK]";
    pub const ASCII_IN_PROGRESS: &'static str = "[..]";
    pub const ASCII_FAILED: &'static str = "[X]";
    pub const ASCII_WARNING: &'static str = "[!]";
    pub const ASCII_INFO: &'static str = "[i]";
    pub const ASCII_CONNECTED: &'static str = "[+]";
    pub const ASCII_DISCONNECTED: &'static str = "[-]";
}

/// Helper functions for applying themed styles
pub struct ThemedStyles {
    pub theme: ColorTheme,
}

impl Default for ThemedStyles {
    fn default() -> Self {
        Self {
            theme: ColorTheme::default(),
        }
    }
}

impl ThemedStyles {
    pub fn success(&self) -> Style {
        Style::default().fg(self.theme.success)
    }
    
    pub fn warning(&self) -> Style {
        Style::default().fg(self.theme.warning)
    }
    
    pub fn error(&self) -> Style {
        Style::default().fg(self.theme.error).add_modifier(Modifier::BOLD)
    }
    
    pub fn info(&self) -> Style {
        Style::default().fg(self.theme.info)
    }
    
    pub fn primary(&self) -> Style {
        Style::default().fg(self.theme.primary)
    }
    
    pub fn secondary(&self) -> Style {
        Style::default().fg(self.theme.secondary)
    }
    
    pub fn accent(&self) -> Style {
        Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD)
    }
    
    pub fn muted(&self) -> Style {
        Style::default().fg(self.theme.muted)
    }
    
    pub fn highlight(&self) -> Style {
        Style::default()
            .bg(self.theme.accent)
            .fg(self.theme.background)
            .add_modifier(Modifier::BOLD)
    }
    
    pub fn network_status(&self, connected: bool) -> Style {
        if connected {
            self.success()
        } else {
            self.error()
        }
    }
    
    pub fn dkg_status(&self, in_progress: bool) -> Style {
        if in_progress {
            Style::default().fg(self.theme.warning).add_modifier(Modifier::SLOW_BLINK)
        } else {
            self.success()
        }
    }
}

/// Format status with appropriate indicator and color
pub fn format_status_with_indicator(status: &str, is_success: bool, use_ascii: bool) -> String {
    let indicator = if use_ascii {
        if is_success {
            StatusIndicators::ASCII_SUCCESS
        } else {
            StatusIndicators::ASCII_FAILED
        }
    } else {
        if is_success {
            StatusIndicators::SUCCESS
        } else {
            StatusIndicators::FAILED
        }
    };
    
    format!("{} {}", indicator, status)
}

/// Get connection status indicator
pub fn get_connection_indicator(connected: bool, use_ascii: bool) -> &'static str {
    if use_ascii {
        if connected {
            StatusIndicators::ASCII_CONNECTED
        } else {
            StatusIndicators::ASCII_DISCONNECTED
        }
    } else {
        if connected {
            StatusIndicators::CONNECTED
        } else {
            StatusIndicators::DISCONNECTED
        }
    }
}

/// Progress bar generator
pub fn create_progress_bar(current: usize, total: usize, width: usize) -> String {
    if total == 0 {
        return "─".repeat(width);
    }
    
    let percentage = (current as f32 / total as f32 * 100.0) as usize;
    let filled = (current as f32 / total as f32 * width as f32) as usize;
    let empty = width.saturating_sub(filled);
    
    format!(
        "{}{}  {}% ({}/{})",
        "█".repeat(filled),
        "░".repeat(empty),
        percentage,
        current,
        total
    )
}

/// Create a styled header with indicators
pub fn create_status_header(
    title: &str,
    status: Option<(&str, bool)>,
    use_ascii: bool,
) -> String {
    match status {
        Some((status_text, is_good)) => {
            let indicator = if use_ascii {
                if is_good {
                    StatusIndicators::ASCII_SUCCESS
                } else {
                    StatusIndicators::ASCII_WARNING
                }
            } else {
                if is_good {
                    StatusIndicators::SUCCESS
                } else {
                    StatusIndicators::WARNING
                }
            };
            format!("{} {} {}", title, indicator, status_text)
        }
        None => title.to_string(),
    }
}