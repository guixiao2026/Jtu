#![cfg(windows)]

use std::sync::atomic::{AtomicIsize, Ordering};

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::Graphics::Dwm::{
    DwmSetWindowAttribute, DWMWA_CLOAK,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowThreadProcessId,
};

static CACHED_HWND: AtomicIsize =
    AtomicIsize::new(0);

unsafe extern "system" fn enum_cb(
    hwnd: HWND,
    lparam: isize,
) -> i32 {
    let target_pid = lparam as u32;
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);
    if pid == target_pid {
        CACHED_HWND
            .store(hwnd as isize, Ordering::Relaxed);
        return 0;
    }
    1
}

fn get_hwnd() -> HWND {
    let cached =
        CACHED_HWND.load(Ordering::Relaxed);
    if cached != 0 {
        return cached as HWND;
    }
    let pid = std::process::id();
    unsafe {
        EnumWindows(
            Some(enum_cb),
            pid as isize,
        );
    }
    let h = CACHED_HWND.load(Ordering::Relaxed);
    if h == 0 {
        log::warn!("[win_utils] HWND not found!");
    } else {
        log::info!(
            "[win_utils] found HWND: {h}"
        );
    }
    h as HWND
}

/// Cloak window via DWM — invisible to user but
/// DWM still composites it (no black flash).
fn cloak(hwnd: HWND) {
    let val: u32 = 1; // TRUE = cloak
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_CLOAK as u32,
            &val as *const u32 as *const _,
            std::mem::size_of::<u32>() as u32,
        );
    }
}

/// Uncloak window — becomes visible.
fn uncloak(hwnd: HWND) {
    let val: u32 = 0; // FALSE = uncloak
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_CLOAK as u32,
            &val as *const u32 as *const _,
            std::mem::size_of::<u32>() as u32,
        );
    }
}

/// Hide window for capture (cloak + keep DWM
/// compositing so swapchain stays valid).
pub fn hide_for_capture() {
    let hwnd = get_hwnd();
    if hwnd.is_null() {
        return;
    }
    log::info!("[win_utils] hide_for_capture");
    cloak(hwnd);
}

/// Uncloak main window after warmup rendering.
pub fn reveal_main() {
    let hwnd = get_hwnd();
    if hwnd.is_null() {
        return;
    }
    log::info!("[win_utils] reveal_main");
    uncloak(hwnd);
}

