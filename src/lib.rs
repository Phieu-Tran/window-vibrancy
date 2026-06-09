//! # window-vibrancy
//!
//! Windows vibrancy effects — Mica, Mica Alt (Tabbed), Acrylic, Blur, rounded corners,
//! and smart OS-version fallback.
//!
//! Fork of [tauri-apps/window-vibrancy](https://github.com/tauri-apps/window-vibrancy)
//! by Phieu-Tran, trimmed to Windows-only with extra features:
//!
//! - **Rounded corners** via DWM `DWMWA_WINDOW_CORNER_PREFERENCE`
//! - **`apply_best_effect()`** — auto-detect OS version and apply the best available effect
//! - **`switch_effect()`** — clear old effect and apply new one in a single call (no flicker)
//! - **Win10 fallback** — requesting Mica on Win10 gracefully falls back to Acrylic
//!
//! # Example
//!
//! ```no_run
//! use window_vibrancy::{apply_mica, apply_rounded_corners, CornerPreference};
//!
//! # let window: &dyn raw_window_handle::HasWindowHandle = unsafe { std::mem::zeroed() };
//! // Apply Mica with dark mode and rounded corners
//! apply_mica(&window, Some(true)).unwrap();
//! apply_rounded_corners(&window, CornerPreference::Round).unwrap();
//! ```

mod windows;

/// A tuple of RGBA colors. Each value has minimum of 0 and maximum of 255.
pub type Color = (u8, u8, u8, u8);

/// The visual effect to apply to a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// Blur effect (Windows 7 / 10 v1809+)
    Blur,
    /// Acrylic effect (Windows 10 v1809+)
    Acrylic,
    /// Mica effect (Windows 11)
    Mica,
    /// Mica Alt / Tabbed effect (Windows 11 build 22523+)
    Tabbed,
    /// No effect (clear/transparent)
    Clear,
}

/// Corner preference for window rounded corners (Windows 11+).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CornerPreference {
    /// System default
    Default = 0,
    /// No rounding
    Square = 1,
    /// Standard rounded corners
    Round = 2,
    /// Small rounded corners
    RoundSmall = 3,
}

fn get_hwnd(window: &impl raw_window_handle::HasWindowHandle) -> Result<isize, Error> {
    match window.window_handle()?.as_raw() {
        #[cfg(target_os = "windows")]
        raw_window_handle::RawWindowHandle::Win32(handle) => Ok(handle.hwnd.get()),
        _ => Err(Error::UnsupportedPlatform("Only Windows is supported.")),
    }
}

// ── Individual effects ──────────────────────────────────────────────

/// Applies blur effect to window. Works on Windows 7, Windows 10 v1809+.
pub fn apply_blur(
    window: impl raw_window_handle::HasWindowHandle,
    color: Option<Color>,
) -> Result<(), Error> {
    windows::apply_blur(get_hwnd(&window)?, color)
}

/// Clears blur effect.
pub fn clear_blur(window: impl raw_window_handle::HasWindowHandle) -> Result<(), Error> {
    windows::clear_blur(get_hwnd(&window)?)
}

/// Applies acrylic effect. Works on Windows 10 v1809+.
pub fn apply_acrylic(
    window: impl raw_window_handle::HasWindowHandle,
    color: Option<Color>,
) -> Result<(), Error> {
    windows::apply_acrylic(get_hwnd(&window)?, color)
}

/// Clears acrylic effect.
pub fn clear_acrylic(window: impl raw_window_handle::HasWindowHandle) -> Result<(), Error> {
    windows::clear_acrylic(get_hwnd(&window)?)
}

/// Applies Mica effect. Works on Windows 11.
///
/// On Windows 10, falls back to Acrylic automatically.
pub fn apply_mica(
    window: impl raw_window_handle::HasWindowHandle,
    dark: Option<bool>,
) -> Result<(), Error> {
    let hwnd = get_hwnd(&window)?;
    match windows::apply_mica(hwnd, dark) {
        Err(Error::UnsupportedPlatformVersion(_)) => {
            // Fallback: Win10 doesn't support Mica, use Acrylic instead
            windows::apply_acrylic(hwnd, None)
        }
        other => other,
    }
}

/// Clears Mica effect.
pub fn clear_mica(window: impl raw_window_handle::HasWindowHandle) -> Result<(), Error> {
    windows::clear_mica(get_hwnd(&window)?)
}

/// Applies Mica Alt (Tabbed) effect. Works on Windows 11 build 22523+.
///
/// On older Windows, falls back to Mica, then Acrylic.
pub fn apply_tabbed(
    window: impl raw_window_handle::HasWindowHandle,
    dark: Option<bool>,
) -> Result<(), Error> {
    let hwnd = get_hwnd(&window)?;
    match windows::apply_tabbed(hwnd, dark) {
        Err(Error::UnsupportedPlatformVersion(_)) => {
            // Fallback chain: Tabbed -> Mica -> Acrylic
            match windows::apply_mica(hwnd, dark) {
                Err(Error::UnsupportedPlatformVersion(_)) => windows::apply_acrylic(hwnd, None),
                other => other,
            }
        }
        other => other,
    }
}

/// Clears Tabbed effect.
pub fn clear_tabbed(window: impl raw_window_handle::HasWindowHandle) -> Result<(), Error> {
    windows::clear_tabbed(get_hwnd(&window)?)
}

// ── Rounded corners ─────────────────────────────────────────────────

/// Sets the corner preference for a window. Works on Windows 11.
pub fn apply_rounded_corners(
    window: impl raw_window_handle::HasWindowHandle,
    preference: CornerPreference,
) -> Result<(), Error> {
    windows::apply_rounded_corners(get_hwnd(&window)?, preference)
}

// ── Smart helpers ───────────────────────────────────────────────────

/// Applies the best available effect for the current OS version.
///
/// - Windows 11 build 22523+: Tabbed (Mica Alt)
/// - Windows 11 build 22000+: Mica
/// - Windows 10 v1809+: Acrylic
/// - Windows 7: Blur
///
/// Returns the [`Effect`] that was actually applied.
pub fn apply_best_effect(
    window: impl raw_window_handle::HasWindowHandle,
    dark: Option<bool>,
) -> Result<Effect, Error> {
    let hwnd = get_hwnd(&window)?;
    windows::apply_best_effect(hwnd, dark)
}

/// Returns the best effect supported by the current OS without applying it.
pub fn best_supported_effect() -> Effect {
    windows::best_supported_effect()
}

/// Returns whether an effect is directly supported by the current OS.
///
/// This reports direct support only: for example, [`Effect::Mica`] returns `false`
/// on Windows 10 even though [`apply_mica`] can fall back to Acrylic.
pub fn is_effect_supported(effect: Effect) -> bool {
    windows::is_effect_supported(effect)
}

/// Clears any vibrancy effect from the window.
pub fn clear_all_effects(window: impl raw_window_handle::HasWindowHandle) -> Result<(), Error> {
    let hwnd = get_hwnd(&window)?;
    windows::clear_all_effects(hwnd)
}

/// Switches from any current effect to a new one without flicker.
///
/// Clears all existing effects first, then applies the requested one.
pub fn switch_effect(
    window: impl raw_window_handle::HasWindowHandle,
    effect: Effect,
    dark: Option<bool>,
    color: Option<Color>,
) -> Result<(), Error> {
    let hwnd = get_hwnd(&window)?;
    // Clear everything first
    let _ = windows::clear_all_effects(hwnd);
    // Apply the new effect
    match effect {
        Effect::Blur => windows::apply_blur(hwnd, color),
        Effect::Acrylic => windows::apply_acrylic(hwnd, color),
        Effect::Mica => match windows::apply_mica(hwnd, dark) {
            Err(Error::UnsupportedPlatformVersion(_)) => windows::apply_acrylic(hwnd, color),
            other => other,
        },
        Effect::Tabbed => match windows::apply_tabbed(hwnd, dark) {
            Err(Error::UnsupportedPlatformVersion(_)) => match windows::apply_mica(hwnd, dark) {
                Err(Error::UnsupportedPlatformVersion(_)) => windows::apply_acrylic(hwnd, color),
                other => other,
            },
            other => other,
        },
        Effect::Clear => Ok(()), // Already cleared above
    }
}

// ── Error type ──────────────────────────────────────────────────────

#[derive(Debug)]
pub enum Error {
    UnsupportedPlatform(&'static str),
    UnsupportedPlatformVersion(&'static str),
    WindowsApi { function: &'static str, code: i32 },
    MissingFunction(&'static str),
    NoWindowHandle(raw_window_handle::HandleError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnsupportedPlatform(e) | Error::UnsupportedPlatformVersion(e) => {
                write!(f, "{}", e)
            }
            Error::WindowsApi { function, code } => {
                write!(f, "{} failed with code {:#x}", function, code)
            }
            Error::MissingFunction(function) => write!(f, "{} is not available", function),
            Error::NoWindowHandle(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<raw_window_handle::HandleError> for Error {
    fn from(err: raw_window_handle::HandleError) -> Self {
        Error::NoWindowHandle(err)
    }
}
