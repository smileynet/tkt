//! Color and symbol support for terminal output.
//!
//! Color mode detection hierarchy:
//! 1. --color=always|never|auto (CLI flag, highest priority)
//! 2. NO_COLOR=1 → force off (https://no-color.org/)
//! 3. TKT_COLOR=1 → force on
//! 4. Auto: color if stdout is a tty
//!
//! Symbol mode:
//! - TKT_ASCII=1 → ASCII-only symbols (✓→[ok], ✗→[err], ⚠→[warn], →→->)
//! - Default: Unicode glyphs

use std::sync::atomic::{AtomicU8, Ordering};

/// Global color mode — set once at startup.
static COLOR_MODE: AtomicU8 = AtomicU8::new(0); // 0=auto, 1=always, 2=never
/// Global ASCII mode — set once at startup.
static ASCII_MODE: AtomicU8 = AtomicU8::new(0); // 0=unicode, 1=ascii

// ANSI codes
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
#[allow(dead_code)]
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

/// Color mode for output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

/// Initialize color state from CLI flag and environment.
/// Call once at startup after parsing args.
pub fn init(cli_color: Option<&str>) {
    let mode = match cli_color {
        Some("always") => ColorMode::Always,
        Some("never") => ColorMode::Never,
        _ => ColorMode::Auto,
    };
    let mode_val = match mode {
        ColorMode::Auto => 0,
        ColorMode::Always => 1,
        ColorMode::Never => 2,
    };
    COLOR_MODE.store(mode_val, Ordering::Relaxed);

    // ASCII mode from env
    let ascii = std::env::var("TKT_ASCII").is_ok_and(|v| v == "1" || v == "true");
    ASCII_MODE.store(if ascii { 1 } else { 0 }, Ordering::Relaxed);
}

/// Whether color is currently enabled for stdout.
/// Auto mode prefers color: enabled unless NO_COLOR is set or stdout is not a tty.
pub fn is_color_enabled() -> bool {
    match COLOR_MODE.load(Ordering::Relaxed) {
        1 => true,  // always
        2 => false, // never
        _ => {
            // auto: color on by default, suppressed only by NO_COLOR or non-tty
            if std::env::var("NO_COLOR").is_ok() {
                return false;
            }
            stdout_is_tty()
        }
    }
}

/// Whether to use ASCII-only symbols.
fn is_ascii() -> bool {
    ASCII_MODE.load(Ordering::Relaxed) == 1
}

/// Check if stdout is a terminal.
fn stdout_is_tty() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

// --- Symbols ---

/// Success symbol: ✓ (green when colored, [ok] when ASCII)
pub fn sym_ok() -> String {
    if is_ascii() {
        if is_color_enabled() {
            format!("{}[ok]{}", GREEN, RESET)
        } else {
            "[ok]".to_string()
        }
    } else if is_color_enabled() {
        format!("{}✓{}", GREEN, RESET)
    } else {
        "✓".to_string()
    }
}

/// Error symbol: ✗ (red when colored, [err] when ASCII)
pub fn sym_err() -> String {
    if is_ascii() {
        if is_color_enabled() {
            format!("{}[err]{}", RED, RESET)
        } else {
            "[err]".to_string()
        }
    } else if is_color_enabled() {
        format!("{}✗{}", RED, RESET)
    } else {
        "✗".to_string()
    }
}

/// Warning symbol: ⚠ (yellow when colored, [warn] when ASCII)
pub fn sym_warn() -> String {
    if is_ascii() {
        if is_color_enabled() {
            format!("{}[warn]{}", YELLOW, RESET)
        } else {
            "[warn]".to_string()
        }
    } else if is_color_enabled() {
        format!("{}⚠{}", YELLOW, RESET)
    } else {
        "⚠".to_string()
    }
}

/// Arrow symbol: → (or -> when ASCII)
pub fn sym_arrow() -> String {
    if is_ascii() {
        "->".to_string()
    } else {
        "→".to_string()
    }
}

/// Bold text (no-op when color disabled)
#[allow(dead_code)]
pub fn bold(text: &str) -> String {
    if is_color_enabled() {
        format!("{}{}{}", BOLD, text, RESET)
    } else {
        text.to_string()
    }
}

/// Green text (no-op when color disabled)
#[allow(dead_code)]
pub fn green(text: &str) -> String {
    if is_color_enabled() {
        format!("{}{}{}", GREEN, text, RESET)
    } else {
        text.to_string()
    }
}

/// Red text (no-op when color disabled)
#[allow(dead_code)]
pub fn red(text: &str) -> String {
    if is_color_enabled() {
        format!("{}{}{}", RED, text, RESET)
    } else {
        text.to_string()
    }
}

/// Yellow text (no-op when color disabled)
#[allow(dead_code)]
pub fn yellow(text: &str) -> String {
    if is_color_enabled() {
        format!("{}{}{}", YELLOW, text, RESET)
    } else {
        text.to_string()
    }
}
