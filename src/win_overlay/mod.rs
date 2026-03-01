#![cfg(windows)]
//! Win32 native overlay — GDI BitBlt, zero flash.
//! Replaces egui viewport overlay entirely.

mod gdi;
mod toolbar;

use std::ptr;

use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use gdi::*;
use toolbar::*;

/// Result from the overlay interaction.
pub enum OverlayAction {
    Cancel,
    Copy,
    Save,
}

/// Selection rectangle (screen coords).
pub struct OverlayResult {
    pub action: OverlayAction,
    /// (x, y, w, h) if user made a selection
    pub selection: Option<(u32, u32, u32, u32)>,
}

// ── internal state ──────────────────────────────

pub(self) struct State {
    pub width: i32,
    pub height: i32,
    // Pre-built DCs with bitmaps
    pub hdc_orig: HDC,
    pub hdc_dark: HDC,
    pub hdc_compose: HDC,
    hbmp_orig: HBITMAP,
    hbmp_dark: HBITMAP,
    hbmp_compose: HBITMAP,
    // Selection
    pub selecting: bool,
    pub sel_start: (i32, i32),
    pub sel_end: (i32, i32),
    pub has_selection: bool,
    // Cursor position (for magnifier)
    pub cursor_x: i32,
    pub cursor_y: i32,
    // Output
    pub action: Option<OverlayAction>,
}

// ── wide string helper ──────────────────────────

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16()
        .chain(std::iter::once(0))
        .collect()
}

// ── normalized selection rect ───────────────────

pub(self) fn sel_rect(s: &State) -> RECT {
    let x1 = s.sel_start.0.min(s.sel_end.0);
    let y1 = s.sel_start.1.min(s.sel_end.1);
    let x2 = s.sel_start.0.max(s.sel_end.0);
    let y2 = s.sel_start.1.max(s.sel_end.1);
    RECT {
        left: x1,
        top: y1,
        right: x2,
        bottom: y2,
    }
}

// ── window procedure ────────────────────────────

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let st_ptr = GetWindowLongPtrW(hwnd, 0)
        as *mut State;
    if st_ptr.is_null() {
        return DefWindowProcW(
            hwnd, msg, wparam, lparam,
        );
    }
    let st = &mut *st_ptr;

    match msg {
        WM_PAINT => {
            paint(hwnd, st);
            0
        }
        WM_ERASEBKGND => 1, // we handle it
        WM_LBUTTONDOWN => {
            let x =
                (lparam & 0xFFFF) as i16 as i32;
            let y = ((lparam >> 16) & 0xFFFF)
                as i16 as i32;

            // Check toolbar clicks
            if st.has_selection && !st.selecting {
                let r = sel_rect(st);
                let btns = toolbar_rects(&r);
                if pt_in_rect(&btns.copy_r, x, y) {
                    st.action =
                        Some(OverlayAction::Copy);
                    PostQuitMessage(0);
                    return 0;
                }
                if pt_in_rect(&btns.save_r, x, y) {
                    st.action =
                        Some(OverlayAction::Save);
                    PostQuitMessage(0);
                    return 0;
                }
                if pt_in_rect(
                    &btns.cancel_r,
                    x,
                    y,
                ) {
                    st.action =
                        Some(OverlayAction::Cancel);
                    PostQuitMessage(0);
                    return 0;
                }
            }

            // Start new selection
            st.selecting = true;
            st.has_selection = false;
            st.sel_start = (x, y);
            st.sel_end = (x, y);
            SetCapture(hwnd);
            InvalidateRect(
                hwnd,
                ptr::null(),
                0,
            );
            0
        }
        WM_MOUSEMOVE => {
            let x =
                (lparam & 0xFFFF) as i16 as i32;
            let y = ((lparam >> 16) & 0xFFFF)
                as i16 as i32;
            st.cursor_x = x;
            st.cursor_y = y;
            if st.selecting {
                st.sel_end = (x, y);
            }
            InvalidateRect(
                hwnd,
                ptr::null(),
                0,
            );
            0
        }
        WM_LBUTTONUP => {
            if st.selecting {
                st.selecting = false;
                ReleaseCapture();
                let x = (lparam & 0xFFFF)
                    as i16 as i32;
                let y = ((lparam >> 16) & 0xFFFF)
                    as i16 as i32;
                st.sel_end = (x, y);
                let r = sel_rect(st);
                if r.right - r.left > 3
                    && r.bottom - r.top > 3
                {
                    st.has_selection = true;
                }
                InvalidateRect(
                    hwnd,
                    ptr::null(),
                    0,
                );
            }
            0
        }
        WM_KEYDOWN => {
            let vk = wparam as u32;
            let ctrl = GetAsyncKeyState(
                VK_CONTROL as i32,
            ) < 0;
            match vk {
                0x1B => {
                    // VK_ESCAPE
                    st.action =
                        Some(OverlayAction::Cancel);
                    PostQuitMessage(0);
                }
                0x0D => {
                    // VK_RETURN
                    if st.has_selection {
                        st.action =
                            Some(OverlayAction::Copy);
                        PostQuitMessage(0);
                    }
                }
                0x43 if ctrl => {
                    // Ctrl+C
                    if st.has_selection {
                        st.action =
                            Some(OverlayAction::Copy);
                        PostQuitMessage(0);
                    }
                }
                0x53 if ctrl => {
                    // Ctrl+S
                    if st.has_selection {
                        st.action =
                            Some(OverlayAction::Save);
                        PostQuitMessage(0);
                    }
                }
                k if (k == VK_LEFT as u32
                    || k == VK_RIGHT as u32
                    || k == VK_UP as u32
                    || k == VK_DOWN as u32)
                    && (!st.has_selection
                        || st.selecting) =>
                {
                    let mut pt: POINT =
                        std::mem::zeroed();
                    GetCursorPos(&mut pt);
                    if k == VK_LEFT as u32 {
                        pt.x -= 1;
                    } else if k == VK_RIGHT as u32 {
                        pt.x += 1;
                    } else if k == VK_UP as u32 {
                        pt.y -= 1;
                    } else {
                        pt.y += 1;
                    }
                    SetCursorPos(pt.x, pt.y);
                    // Update internal state
                    st.cursor_x = pt.x;
                    st.cursor_y = pt.y;
                    if st.selecting {
                        st.sel_end =
                            (pt.x, pt.y);
                    }
                    InvalidateRect(
                        hwnd,
                        ptr::null(),
                        0,
                    );
                }
                _ => {}
            }
            0
        }
        WM_SETCURSOR => {
            let cursor =
                LoadCursorW(0 as HINSTANCE, IDC_CROSS);
            SetCursor(cursor);
            1
        }
        _ => DefWindowProcW(
            hwnd, msg, wparam, lparam,
        ),
    }
}

// ── public entry point ──────────────────────────

/// Show overlay with BGRA data from a slice.
/// Used by xcap fallback path.
pub fn run_overlay(
    bgra: &[u8],
    width: u32,
    height: u32,
) -> OverlayResult {
    let w = width as i32;
    let h = height as i32;
    unsafe {
        // Orig: memcpy bgra → DIB
        let (hdc_orig, hbmp_orig) =
            create_dib(w, h, bgra);
        // Dark: BitBlt orig → dark, then darken
        // in-place via raw pointer (zero alloc)
        let (hdc_dark, hbmp_dark, dark_bits) =
            create_dib_uninit(w, h);
        BitBlt(
            hdc_dark, 0, 0, w, h,
            hdc_orig, 0, 0, SRCCOPY,
        );
        let total = (w as usize) * (h as usize) * 4;
        darken_inplace(dark_bits, total);
        let (hdc_compose, hbmp_compose) =
            create_empty_dib(w, h);
        run_overlay_inner(
            w, h, hdc_orig, hdc_dark, hdc_compose,
            hbmp_orig, hbmp_dark, hbmp_compose,
        )
    }
}

/// Show overlay, letting caller fill the orig DIB
/// directly via raw pointer (zero intermediate Vec).
/// `fill_fn(bits_ptr, byte_len)` must write BGRA data.
pub fn run_overlay_with<F>(
    width: u32,
    height: u32,
    fill_fn: F,
) -> OverlayResult
where
    F: FnOnce(*mut u8, usize),
{
    let w = width as i32;
    let h = height as i32;
    let total = (w as usize) * (h as usize) * 4;
    unsafe {
        // Orig: caller writes directly into DIB bits
        let (hdc_orig, hbmp_orig, orig_bits) =
            create_dib_uninit(w, h);
        fill_fn(orig_bits, total);
        // Dark: BitBlt + darken in-place
        let (hdc_dark, hbmp_dark, dark_bits) =
            create_dib_uninit(w, h);
        BitBlt(
            hdc_dark, 0, 0, w, h,
            hdc_orig, 0, 0, SRCCOPY,
        );
        darken_inplace(dark_bits, total);
        let (hdc_compose, hbmp_compose) =
            create_empty_dib(w, h);
        run_overlay_inner(
            w, h, hdc_orig, hdc_dark, hdc_compose,
            hbmp_orig, hbmp_dark, hbmp_compose,
        )
    }
}

unsafe fn run_overlay_inner(
    w: i32,
    h: i32,
    hdc_orig: HDC,
    hdc_dark: HDC,
    hdc_compose: HDC,
    hbmp_orig: HBITMAP,
    hbmp_dark: HBITMAP,
    hbmp_compose: HBITMAP,
) -> OverlayResult {
    let mut state = State {
        width: w,
        height: h,
        hdc_orig,
        hdc_dark,
        hdc_compose,
        hbmp_orig,
        hbmp_dark,
        hbmp_compose,
        selecting: false,
        sel_start: (0, 0),
        sel_end: (0, 0),
        has_selection: false,
        cursor_x: 0,
        cursor_y: 0,
        action: None,
    };

    // Register window class
    let class_name = wide("SnapVaultOverlay");
    let hinstance = 0 as HINSTANCE;
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>()
            as u32,
        style: 0,
        lpfnWndProc: Some(wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: std::mem::size_of::<
            *mut State,
        >() as i32,
        hInstance: hinstance,
        hIcon: 0 as HICON,
        hCursor: LoadCursorW(
            0 as HINSTANCE,
            IDC_CROSS,
        ),
        hbrBackground: 0 as HBRUSH,
        lpszMenuName: ptr::null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: 0 as HICON,
    };
    RegisterClassExW(&wc);

    // Create fullscreen window
    let hwnd = CreateWindowExW(
        WS_EX_TOPMOST,
        class_name.as_ptr(),
        wide("SnapVault Overlay").as_ptr(),
        WS_POPUP | WS_VISIBLE,
        0,
        0,
        w,
        h,
        0 as HWND,
        0 as HMENU,
        hinstance,
        ptr::null(),
    );

    // Store state pointer
    SetWindowLongPtrW(
        hwnd,
        0,
        &mut state as *mut State as isize,
    );

    // Force first paint
    InvalidateRect(hwnd, ptr::null(), 0);
    UpdateWindow(hwnd);
    SetForegroundWindow(hwnd);

    // Message loop
    let mut msg: MSG = std::mem::zeroed();
    while GetMessageW(
        &mut msg,
        0 as HWND,
        0,
        0,
    ) > 0
    {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    // Get result
    let selection = if state.has_selection {
        let r = sel_rect(&state);
        let sx = r.left.max(0) as u32;
        let sy = r.top.max(0) as u32;
        let sw =
            (r.right - r.left).max(0) as u32;
        let sh =
            (r.bottom - r.top).max(0) as u32;
        Some((sx, sy, sw, sh))
    } else {
        None
    };

    // Cleanup
    DestroyWindow(hwnd);
    UnregisterClassW(
        class_name.as_ptr(),
        hinstance,
    );
    DeleteDC(hdc_orig);
    DeleteDC(hdc_dark);
    DeleteDC(hdc_compose);
    DeleteObject(hbmp_orig as _);
    DeleteObject(hbmp_dark as _);
    DeleteObject(hbmp_compose as _);

    let action = state
        .action
        .unwrap_or(OverlayAction::Cancel);

    OverlayResult { action, selection }
}
