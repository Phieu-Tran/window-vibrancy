# window-vibrancy

[![License](https://img.shields.io/badge/License-Apache%202.0%20%2F%20MIT-blue.svg)](LICENSE)

Windows vibrancy effects library — Mica, Mica Alt, Acrylic, Blur, rounded corners, and smart OS-version fallback.

Built for [ClipPaste](https://github.com/Phieu-Tran/ClipPaste). Windows-only fork of [tauri-apps/window-vibrancy](https://github.com/tauri-apps/window-vibrancy).

## Features

| Function | Windows Version | Notes |
|:---------|:---------------:|:------|
| `apply_blur` / `clear_blur` | 7, 10 v1809+, 11 | May lag on Win11 22621+ during resize |
| `apply_acrylic` / `clear_acrylic` | 10 v1809+, 11 | May lag on Win10 v1903+ and Win11 22000 |
| `apply_mica` / `clear_mica` | 11 | Falls back to Acrylic on Win10 |
| `apply_tabbed` / `clear_tabbed` | 11 build 22523+ | Falls back to Mica, then Acrylic |
| `apply_rounded_corners` | 11 | Native DWM rounded corners |
| `apply_best_effect` | 7+ | Auto-detects OS, applies best effect |
| `switch_effect` | 7+ | Flicker-free effect switching |
| `clear_all_effects` | 7+ | Clears any active effect |

## Usage

```rust
use window_vibrancy::{apply_mica, apply_rounded_corners, CornerPreference};

// Apply Mica with dark mode (falls back to Acrylic on Win10)
apply_mica(&window, Some(true)).unwrap();

// Native rounded corners on Win11
apply_rounded_corners(&window, CornerPreference::Round).unwrap();
```

### Smart effect switching (no flicker)

```rust
use window_vibrancy::{switch_effect, Effect};

// Switch from any effect to Mica Alt — clears old effect first
switch_effect(&window, Effect::Tabbed, Some(true), None).unwrap();
```

### Auto-detect best effect

```rust
use window_vibrancy::apply_best_effect;

let applied = apply_best_effect(&window, Some(true)).unwrap();
println!("Applied: {:?}", applied); // e.g. Effect::Tabbed on Win11
```

## Tauri Integration

```rust
// In your Tauri setup:
use window_vibrancy::{apply_mica, apply_rounded_corners, CornerPreference};

let window = app.get_webview_window("main").unwrap();
apply_mica(&window, Some(true)).unwrap();
apply_rounded_corners(&window, CornerPreference::Round).unwrap();
```

Don't forget to set in your frontend:
```css
html, body { background: transparent; }
```

And in `tauri.conf.json`:
```json
{ "windows": [{ "transparent": true }] }
```

## Types

### `Effect`
```rust
pub enum Effect {
    Blur,     // Windows 7 / 10 v1809+
    Acrylic,  // Windows 10 v1809+
    Mica,     // Windows 11
    Tabbed,   // Windows 11 build 22523+ (Mica Alt)
    Clear,    // No effect
}
```

### `CornerPreference`
```rust
pub enum CornerPreference {
    Default = 0,    // System default
    Square = 1,     // No rounding
    Round = 2,      // Standard rounded corners
    RoundSmall = 3, // Small rounded corners
}
```

## License

Apache-2.0 OR MIT
