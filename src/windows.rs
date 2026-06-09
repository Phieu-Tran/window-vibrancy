// Copyright 2024 Phieu-Tran
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(target_os = "windows")]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]

use std::{ffi::c_void, sync::OnceLock};
use windows_sys::core::{BOOL, HRESULT};
use windows_sys::Win32::{Foundation::*, Graphics::Dwm::*, System::LibraryLoader::*};

use crate::{Color, CornerPreference, Effect, Error};

/// All public functions accept `isize` (from raw_window_handle) and cast to HWND internally.
fn h(handle: isize) -> HWND {
    handle as HWND
}

fn hresult(result: HRESULT, function: &'static str) -> Result<(), Error> {
    if result >= 0 {
        Ok(())
    } else {
        Err(Error::WindowsApi {
            function,
            code: result,
        })
    }
}

fn bool_result(result: BOOL, function: &'static str) -> Result<(), Error> {
    if result != 0 {
        Ok(())
    } else {
        Err(Error::WindowsApi {
            function,
            code: unsafe { GetLastError() as i32 },
        })
    }
}

unsafe fn set_dwm_u32(
    hwnd: HWND,
    attribute: DWMWINDOWATTRIBUTE,
    value: u32,
    function: &'static str,
) -> Result<(), Error> {
    hresult(
        DwmSetWindowAttribute(
            hwnd,
            attribute as _,
            &value as *const _ as _,
            std::mem::size_of_val(&value) as u32,
        ),
        function,
    )
}

pub fn apply_blur(handle: isize, color: Option<Color>) -> Result<(), Error> {
    let hwnd = h(handle);
    if is_win7() {
        let bb = DWM_BLURBEHIND {
            dwFlags: DWM_BB_ENABLE,
            fEnable: true.into(),
            hRgnBlur: std::ptr::null_mut(),
            fTransitionOnMaximized: 0,
        };
        unsafe {
            hresult(
                DwmEnableBlurBehindWindow(hwnd, &bb),
                "DwmEnableBlurBehindWindow",
            )?;
        }
    } else if is_swca_supported() {
        unsafe {
            SetWindowCompositionAttribute(hwnd, ACCENT_STATE::ACCENT_ENABLE_BLURBEHIND, color)?
        };
    } else {
        return Err(Error::UnsupportedPlatformVersion(
            "\"apply_blur()\" is only available on Windows 7, Windows 10 v1809 or newer.",
        ));
    }
    Ok(())
}

pub fn clear_blur(handle: isize) -> Result<(), Error> {
    let hwnd = h(handle);
    if is_win7() {
        let bb = DWM_BLURBEHIND {
            dwFlags: DWM_BB_ENABLE,
            fEnable: false.into(),
            hRgnBlur: std::ptr::null_mut(),
            fTransitionOnMaximized: 0,
        };
        unsafe {
            hresult(
                DwmEnableBlurBehindWindow(hwnd, &bb),
                "DwmEnableBlurBehindWindow",
            )?;
        }
    } else if is_swca_supported() {
        unsafe { SetWindowCompositionAttribute(hwnd, ACCENT_STATE::ACCENT_DISABLED, None)? };
    } else {
        return Err(Error::UnsupportedPlatformVersion(
            "\"clear_blur()\" is only available on Windows 7, Windows 10 v1809 or newer.",
        ));
    }
    Ok(())
}

pub fn apply_acrylic(handle: isize, color: Option<Color>) -> Result<(), Error> {
    let hwnd = h(handle);
    if is_backdroptype_supported() {
        unsafe {
            set_dwm_u32(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE,
                DWM_SYSTEMBACKDROP_TYPE::DWMSBT_TRANSIENTWINDOW as u32,
                "DwmSetWindowAttribute(DWMWA_SYSTEMBACKDROP_TYPE)",
            )?;
        }
    } else if is_swca_supported() {
        unsafe {
            SetWindowCompositionAttribute(
                hwnd,
                ACCENT_STATE::ACCENT_ENABLE_ACRYLICBLURBEHIND,
                color,
            )?
        };
    } else {
        return Err(Error::UnsupportedPlatformVersion(
            "\"apply_acrylic()\" is only available on Windows 10 v1809 or newer.",
        ));
    }
    Ok(())
}

pub fn clear_acrylic(handle: isize) -> Result<(), Error> {
    let hwnd = h(handle);
    if is_backdroptype_supported() {
        unsafe {
            set_dwm_u32(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE,
                DWM_SYSTEMBACKDROP_TYPE::DWMSBT_DISABLE as u32,
                "DwmSetWindowAttribute(DWMWA_SYSTEMBACKDROP_TYPE)",
            )?;
        }
    } else if is_swca_supported() {
        unsafe { SetWindowCompositionAttribute(hwnd, ACCENT_STATE::ACCENT_DISABLED, None)? };
    } else {
        return Err(Error::UnsupportedPlatformVersion(
            "\"clear_acrylic()\" is only available on Windows 10 v1809 or newer.",
        ));
    }
    Ok(())
}

pub fn apply_mica(handle: isize, dark: Option<bool>) -> Result<(), Error> {
    let hwnd = h(handle);
    let dark = dark.unwrap_or(false);
    if !is_backdroptype_supported() && !is_undocumented_mica_supported() {
        return Err(Error::UnsupportedPlatformVersion(
            "\"apply_mica()\" is only available on Windows 11.",
        ));
    }

    unsafe {
        set_dwm_u32(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            dark as u32,
            "DwmSetWindowAttribute(DWMWA_USE_IMMERSIVE_DARK_MODE)",
        )?;
    }
    if is_backdroptype_supported() {
        unsafe {
            set_dwm_u32(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE,
                DWM_SYSTEMBACKDROP_TYPE::DWMSBT_MAINWINDOW as u32,
                "DwmSetWindowAttribute(DWMWA_SYSTEMBACKDROP_TYPE)",
            )?;
        }
    } else if is_undocumented_mica_supported() {
        unsafe {
            set_dwm_u32(
                hwnd,
                DWMWA_MICA_EFFECT,
                1,
                "DwmSetWindowAttribute(DWMWA_MICA_EFFECT)",
            )?;
        }
    }
    Ok(())
}

pub fn clear_mica(handle: isize) -> Result<(), Error> {
    let hwnd = h(handle);
    if !is_backdroptype_supported() && !is_undocumented_mica_supported() {
        return Err(Error::UnsupportedPlatformVersion(
            "\"clear_mica()\" is only available on Windows 11.",
        ));
    }

    unsafe {
        set_dwm_u32(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            0,
            "DwmSetWindowAttribute(DWMWA_USE_IMMERSIVE_DARK_MODE)",
        )?;
    }
    if is_backdroptype_supported() {
        unsafe {
            set_dwm_u32(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE,
                DWM_SYSTEMBACKDROP_TYPE::DWMSBT_DISABLE as u32,
                "DwmSetWindowAttribute(DWMWA_SYSTEMBACKDROP_TYPE)",
            )?;
        }
    } else if is_undocumented_mica_supported() {
        unsafe {
            set_dwm_u32(
                hwnd,
                DWMWA_MICA_EFFECT,
                0,
                "DwmSetWindowAttribute(DWMWA_MICA_EFFECT)",
            )?;
        }
    }
    Ok(())
}

pub fn apply_tabbed(handle: isize, dark: Option<bool>) -> Result<(), Error> {
    let hwnd = h(handle);
    let dark = dark.unwrap_or(false);
    if !is_backdroptype_supported() {
        return Err(Error::UnsupportedPlatformVersion(
            "\"apply_tabbed()\" is only available on Windows 11 build 22523+.",
        ));
    }

    unsafe {
        set_dwm_u32(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            dark as u32,
            "DwmSetWindowAttribute(DWMWA_USE_IMMERSIVE_DARK_MODE)",
        )?;
    }
    if is_backdroptype_supported() {
        unsafe {
            set_dwm_u32(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE,
                DWM_SYSTEMBACKDROP_TYPE::DWMSBT_TABBEDWINDOW as u32,
                "DwmSetWindowAttribute(DWMWA_SYSTEMBACKDROP_TYPE)",
            )?;
        }
    }
    Ok(())
}

pub fn clear_tabbed(handle: isize) -> Result<(), Error> {
    let hwnd = h(handle);
    if is_backdroptype_supported() {
        unsafe {
            set_dwm_u32(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE,
                DWM_SYSTEMBACKDROP_TYPE::DWMSBT_DISABLE as u32,
                "DwmSetWindowAttribute(DWMWA_SYSTEMBACKDROP_TYPE)",
            )?;
        }
    } else {
        return Err(Error::UnsupportedPlatformVersion(
            "\"clear_tabbed()\" is only available on Windows 11 build 22523+.",
        ));
    }
    Ok(())
}

// ── Rounded corners ─────────────────────────────────────────────────

pub fn apply_rounded_corners(handle: isize, preference: CornerPreference) -> Result<(), Error> {
    if !is_undocumented_mica_supported() {
        return Err(Error::UnsupportedPlatformVersion(
            "\"apply_rounded_corners()\" is only available on Windows 11.",
        ));
    }
    let hwnd = h(handle);
    unsafe {
        set_dwm_u32(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            preference as u32,
            "DwmSetWindowAttribute(DWMWA_WINDOW_CORNER_PREFERENCE)",
        )?;
    }
    Ok(())
}

// ── Smart helpers ───────────────────────────────────────────────────

pub fn apply_best_effect(handle: isize, dark: Option<bool>) -> Result<Effect, Error> {
    let effect = best_supported_effect();
    match effect {
        Effect::Tabbed => apply_tabbed(handle, dark)?,
        Effect::Mica => apply_mica(handle, dark)?,
        Effect::Acrylic => apply_acrylic(handle, None)?,
        Effect::Blur => apply_blur(handle, None)?,
        Effect::Clear => {}
    }
    Ok(effect)
}

pub fn best_supported_effect() -> Effect {
    let version = current_os_version();
    best_supported_effect_for_version(version.major, version.minor, version.build)
}

pub fn is_effect_supported(effect: Effect) -> bool {
    let version = current_os_version();
    is_effect_supported_for_version(effect, version.major, version.minor, version.build)
}

pub fn clear_all_effects(handle: isize) -> Result<(), Error> {
    let hwnd = h(handle);
    // Reset dark mode
    if is_swca_supported() {
        unsafe {
            set_dwm_u32(
                hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                0,
                "DwmSetWindowAttribute(DWMWA_USE_IMMERSIVE_DARK_MODE)",
            )?;
        }
    }
    // Clear backdrop type (Win11 22523+)
    if is_backdroptype_supported() {
        unsafe {
            set_dwm_u32(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE,
                DWM_SYSTEMBACKDROP_TYPE::DWMSBT_DISABLE as u32,
                "DwmSetWindowAttribute(DWMWA_SYSTEMBACKDROP_TYPE)",
            )?;
        }
    }
    // Clear undocumented Mica (Win11 22000-22522)
    if is_undocumented_mica_supported() {
        unsafe {
            set_dwm_u32(
                hwnd,
                DWMWA_MICA_EFFECT,
                0,
                "DwmSetWindowAttribute(DWMWA_MICA_EFFECT)",
            )?;
        }
    }
    // Clear SWCA-based effects (Win10)
    if is_swca_supported() {
        unsafe { SetWindowCompositionAttribute(hwnd, ACCENT_STATE::ACCENT_DISABLED, None)? };
    }
    // Clear Win7 blur
    if is_win7() {
        let bb = DWM_BLURBEHIND {
            dwFlags: DWM_BB_ENABLE,
            fEnable: false.into(),
            hRgnBlur: std::ptr::null_mut(),
            fTransitionOnMaximized: 0,
        };
        unsafe {
            hresult(
                DwmEnableBlurBehindWindow(hwnd, &bb),
                "DwmEnableBlurBehindWindow",
            )?;
        }
    }
    Ok(())
}

// ── Internal helpers ────────────────────────────────────────────────

fn get_function_impl(library: &str, function: &str) -> FARPROC {
    debug_assert_eq!(library.chars().last(), Some('\0'));
    debug_assert_eq!(function.chars().last(), Some('\0'));

    let mut module = unsafe { GetModuleHandleA(library.as_ptr()) };
    if module.is_null() {
        module = unsafe { LoadLibraryA(library.as_ptr()) };
    }
    if module.is_null() {
        return None;
    }

    unsafe { GetProcAddress(module, function.as_ptr()) }
}

#[repr(C)]
struct ACCENT_POLICY {
    AccentState: u32,
    AccentFlags: u32,
    GradientColor: u32,
    AnimationId: u32,
}

type WINDOWCOMPOSITIONATTRIB = u32;

#[repr(C)]
struct WINDOWCOMPOSITIONATTRIBDATA {
    Attrib: WINDOWCOMPOSITIONATTRIB,
    pvData: *mut c_void,
    cbData: usize,
}

#[derive(PartialEq)]
#[repr(C)]
enum ACCENT_STATE {
    ACCENT_DISABLED = 0,
    ACCENT_ENABLE_BLURBEHIND = 3,
    ACCENT_ENABLE_ACRYLICBLURBEHIND = 4,
}

type SetWindowCompositionAttributeFn =
    unsafe extern "system" fn(HWND, *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;

static SET_WINDOW_COMPOSITION_ATTRIBUTE: OnceLock<Option<SetWindowCompositionAttributeFn>> =
    OnceLock::new();

fn resolve_set_window_composition_attribute() -> Option<SetWindowCompositionAttributeFn> {
    get_function_impl("user32.dll\0", "SetWindowCompositionAttribute\0")
        .map(|f| unsafe { std::mem::transmute::<_, SetWindowCompositionAttributeFn>(f) })
}

unsafe fn SetWindowCompositionAttribute(
    hwnd: HWND,
    accent_state: ACCENT_STATE,
    color: Option<Color>,
) -> Result<(), Error> {
    let set_window_composition_attribute = SET_WINDOW_COMPOSITION_ATTRIBUTE
        .get_or_init(resolve_set_window_composition_attribute)
        .ok_or(Error::MissingFunction("SetWindowCompositionAttribute"))?;

    let mut color = color.unwrap_or_default();
    let is_acrylic = accent_state == ACCENT_STATE::ACCENT_ENABLE_ACRYLICBLURBEHIND;
    if is_acrylic && color.3 == 0 {
        color.3 = 1;
    }

    let mut policy = ACCENT_POLICY {
        AccentState: accent_state as _,
        AccentFlags: if is_acrylic { 0 } else { 2 },
        GradientColor: (color.0 as u32)
            | ((color.1 as u32) << 8)
            | ((color.2 as u32) << 16)
            | ((color.3 as u32) << 24),
        AnimationId: 0,
    };

    let mut data = WINDOWCOMPOSITIONATTRIBDATA {
        Attrib: 0x13,
        pvData: &mut policy as *mut _ as _,
        cbData: std::mem::size_of_val(&policy),
    };

    bool_result(
        set_window_composition_attribute(hwnd, &mut data as *mut _ as _),
        "SetWindowCompositionAttribute",
    )
}

const DWMWA_MICA_EFFECT: DWMWINDOWATTRIBUTE = 1029;
const DWMWA_SYSTEMBACKDROP_TYPE: DWMWINDOWATTRIBUTE = 38;
const DWMWA_WINDOW_CORNER_PREFERENCE: DWMWINDOWATTRIBUTE = 33;

#[allow(unused)]
#[repr(C)]
enum DWM_SYSTEMBACKDROP_TYPE {
    DWMSBT_DISABLE = 1,
    DWMSBT_MAINWINDOW = 2,
    DWMSBT_TRANSIENTWINDOW = 3,
    DWMSBT_TABBEDWINDOW = 4,
}

fn is_win7() -> bool {
    let v = current_os_version();
    is_win7_version(v.major, v.minor)
}

fn is_at_least_build(build: u32) -> bool {
    let v = current_os_version();
    v.build >= build
}

#[derive(Clone, Copy)]
struct OsVersionParts {
    major: u32,
    minor: u32,
    build: u32,
}

fn current_os_version() -> OsVersionParts {
    let v = windows_version::OsVersion::current();
    OsVersionParts {
        major: v.major,
        minor: v.minor,
        build: v.build,
    }
}

fn is_win7_version(major: u32, minor: u32) -> bool {
    major == 6 && minor == 1
}

fn best_supported_effect_for_version(major: u32, minor: u32, build: u32) -> Effect {
    if build >= 22523 {
        Effect::Tabbed
    } else if build >= 22000 {
        Effect::Mica
    } else if build >= 17763 {
        Effect::Acrylic
    } else if is_win7_version(major, minor) {
        Effect::Blur
    } else {
        Effect::Clear
    }
}

fn is_effect_supported_for_version(effect: Effect, major: u32, minor: u32, build: u32) -> bool {
    match effect {
        Effect::Clear => true,
        Effect::Blur => build >= 17763 || is_win7_version(major, minor),
        Effect::Acrylic => build >= 17763,
        Effect::Mica => build >= 22000,
        Effect::Tabbed => build >= 22523,
    }
}

fn is_swca_supported() -> bool {
    is_at_least_build(17763)
}
fn is_undocumented_mica_supported() -> bool {
    is_at_least_build(22000)
}
fn is_backdroptype_supported() -> bool {
    is_at_least_build(22523)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn best_supported_effect_prefers_tabbed_on_new_windows_11() {
        assert_eq!(
            best_supported_effect_for_version(10, 0, 22621),
            Effect::Tabbed
        );
        assert_eq!(
            best_supported_effect_for_version(10, 0, 22523),
            Effect::Tabbed
        );
    }

    #[test]
    fn best_supported_effect_uses_mica_on_early_windows_11() {
        assert_eq!(
            best_supported_effect_for_version(10, 0, 22000),
            Effect::Mica
        );
        assert_eq!(
            best_supported_effect_for_version(10, 0, 22522),
            Effect::Mica
        );
    }

    #[test]
    fn best_supported_effect_uses_acrylic_on_supported_windows_10() {
        assert_eq!(
            best_supported_effect_for_version(10, 0, 17763),
            Effect::Acrylic
        );
        assert_eq!(
            best_supported_effect_for_version(10, 0, 19045),
            Effect::Acrylic
        );
    }

    #[test]
    fn best_supported_effect_uses_blur_only_on_windows_7_legacy_path() {
        assert_eq!(best_supported_effect_for_version(6, 1, 7601), Effect::Blur);
        assert_eq!(best_supported_effect_for_version(6, 2, 9200), Effect::Clear);
    }

    #[test]
    fn effect_support_reports_direct_support_only() {
        assert!(is_effect_supported_for_version(Effect::Blur, 6, 1, 7601));
        assert!(!is_effect_supported_for_version(
            Effect::Acrylic,
            6,
            1,
            7601
        ));
        assert!(is_effect_supported_for_version(
            Effect::Acrylic,
            10,
            0,
            19045
        ));
        assert!(!is_effect_supported_for_version(Effect::Mica, 10, 0, 19045));
        assert!(is_effect_supported_for_version(Effect::Mica, 10, 0, 22000));
        assert!(!is_effect_supported_for_version(
            Effect::Tabbed,
            10,
            0,
            22000
        ));
        assert!(is_effect_supported_for_version(
            Effect::Tabbed,
            10,
            0,
            22621
        ));
        assert!(is_effect_supported_for_version(Effect::Clear, 6, 2, 9200));
    }
}
