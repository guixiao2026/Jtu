#![cfg(windows)]

use std::ptr;

use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use global_hotkey::hotkey::{Code, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use tray_icon::menu::{MenuEvent, MenuId};

use crate::hotkeys::HotkeyManager;

// ── Colors ─────────────────────────────────────
const BG: u32 = rgb(30, 30, 30);
const TEXT_DIM: u32 = rgb(160, 160, 160);
const TEXT_BRIGHT: u32 = rgb(230, 230, 230);
const BOX_BG: u32 = rgb(50, 50, 50);
const BOX_BORDER: u32 = rgb(70, 70, 70);
const ACCENT: u32 = rgb(66, 133, 244);
const RECORDING_BG: u32 = rgb(60, 50, 30);
const RECORDING_BORDER: u32 = rgb(255, 180, 0);

const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    r as u32 | ((g as u32) << 8) | ((b as u32) << 16)
}

// ── Layout ─────────────────────────────────────
const WIN_W: i32 = 320;
const WIN_H: i32 = 140;
const LABEL_Y: i32 = 24;
const BOX_LEFT: i32 = 24;
const BOX_TOP: i32 = 52;
const BOX_RIGHT: i32 = WIN_W - 24;
const BOX_BOTTOM: i32 = 92;
const BOX_RADIUS: i32 = 6;
const HINT_Y: i32 = 100;

struct SettingsState {
    recording: bool,
    hk: *mut HotkeyManager,
    hfont_label: HFONT,
    hfont_key: HFONT,
    hfont_hint: HFONT,
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16()
        .chain(std::iter::once(0))
        .collect()
}

/// Returns true if app should quit.
pub fn show_settings<F: Fn()>(
    hk: &mut HotkeyManager,
    on_capture: F,
    quit_id: &MenuId,
    capture_id: &MenuId,
) -> bool {
    unsafe {
        show_settings_inner(
            hk,
            &on_capture,
            quit_id,
            capture_id,
        )
    }
}

unsafe fn make_font(
    size: i32,
    weight: u32,
) -> HFONT {
    CreateFontW(
        size,
        0,
        0,
        0,
        weight as i32,
        0,
        0,
        0,
        0,
        0,
        0,
        CLEARTYPE_QUALITY as u32,
        0,
        wide("Segoe UI").as_ptr(),
    )
}

unsafe fn show_settings_inner(
    hk: &mut HotkeyManager,
    on_capture: &dyn Fn(),
    quit_id: &MenuId,
    capture_id: &MenuId,
) -> bool {
    let hfont_label = make_font(-14, FW_NORMAL);
    let hfont_key = make_font(-18, FW_SEMIBOLD);
    let hfont_hint = make_font(-12, FW_NORMAL);

    let mut state = SettingsState {
        recording: false,
        hk: hk as *mut HotkeyManager,
        hfont_label,
        hfont_key,
        hfont_hint,
    };

    let class_name = wide("JtuSettings");
    let hinstance = GetModuleHandleW(ptr::null());
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>()
            as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: std::mem::size_of::<
            *mut SettingsState,
        >() as i32,
        hInstance: hinstance,
        hIcon: 0 as HICON,
        hCursor: LoadCursorW(
            0 as HINSTANCE,
            IDC_ARROW,
        ),
        hbrBackground: 0 as HBRUSH,
        lpszMenuName: ptr::null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: 0 as HICON,
    };
    RegisterClassExW(&wc);

    let title = wide("Jtu");
    // Calculate window rect for desired client area
    let mut rc = RECT {
        left: 0,
        top: 0,
        right: WIN_W,
        bottom: WIN_H,
    };
    let style =
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU;
    AdjustWindowRectEx(&mut rc, style, 0, 0);
    let w = rc.right - rc.left;
    let h = rc.bottom - rc.top;

    let scr_w = GetSystemMetrics(SM_CXSCREEN);
    let scr_h = GetSystemMetrics(SM_CYSCREEN);
    let x = (scr_w - w) / 2;
    let y = (scr_h - h) / 2;

    let hwnd = CreateWindowExW(
        0,
        class_name.as_ptr(),
        title.as_ptr(),
        style | WS_VISIBLE,
        x,
        y,
        w,
        h,
        0 as HWND,
        0 as HMENU,
        hinstance,
        ptr::null(),
    );

    SetWindowLongPtrW(
        hwnd,
        0,
        &mut state as *mut SettingsState as isize,
    );

    SetForegroundWindow(hwnd);
    InvalidateRect(hwnd, ptr::null(), 1);

    let mut msg: MSG = std::mem::zeroed();
    let mut quit_app = false;

    'outer: loop {
        // Check tray menu events
        if let Ok(ev) =
            MenuEvent::receiver().try_recv()
        {
            if *ev.id() == *quit_id {
                // Fix 1: resume hotkey if recording
                if state.recording {
                    hk.resume();
                }
                quit_app = true;
                break 'outer;
            }
            // Fix 2: handle Screenshot from tray
            if *ev.id() == *capture_id {
                // Fix 3: hide so DXGI won't capture
                ShowWindow(hwnd, SW_HIDE);
                std::thread::sleep(
                    std::time::Duration::from_millis(
                        50,
                    ),
                );
                on_capture();
                ShowWindow(hwnd, SW_SHOW);
                SetForegroundWindow(hwnd);
            }
        }

        // Check hotkey channel when not recording
        // Fix 4: read hotkey_id fresh each iteration
        if !state.recording && hk.is_active() {
            let hotkey_id = hk.hotkey().id();
            let rx =
                GlobalHotKeyEvent::receiver();
            while let Ok(ev) = rx.try_recv() {
                if ev.state()
                    == HotKeyState::Pressed
                    && ev.id() == hotkey_id
                {
                    // Fix 3: hide so DXGI won't capture
                    ShowWindow(hwnd, SW_HIDE);
                    std::thread::sleep(
                        std::time::Duration::from_millis(
                            50,
                        ),
                    );
                    on_capture();
                    ShowWindow(hwnd, SW_SHOW);
                    SetForegroundWindow(hwnd);
                }
            }
        }

        // Process Win32 messages
        while PeekMessageW(
            &mut msg,
            ptr::null_mut(),
            0,
            0,
            PM_REMOVE,
        ) != 0
        {
            if msg.message == WM_QUIT {
                break 'outer;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        std::thread::sleep(
            std::time::Duration::from_millis(8),
        );
    }

    DestroyWindow(hwnd);
    UnregisterClassW(
        class_name.as_ptr(),
        hinstance,
    );
    DeleteObject(hfont_label as _);
    DeleteObject(hfont_key as _);
    DeleteObject(hfont_hint as _);

    quit_app
}

// ── Paint ──────────────────────────────────────

unsafe fn paint(hwnd: HWND, st: &SettingsState) {
    let mut ps: PAINTSTRUCT = std::mem::zeroed();
    let hdc = BeginPaint(hwnd, &mut ps);

    // Double buffer
    let mut cr = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    GetClientRect(hwnd, &mut cr);
    let cw = cr.right;
    let ch = cr.bottom;
    let mem_dc = CreateCompatibleDC(hdc);
    let mem_bmp =
        CreateCompatibleBitmap(hdc, cw, ch);
    let old_bmp = SelectObject(mem_dc, mem_bmp as _);

    // Fill background
    let bg_brush = CreateSolidBrush(BG);
    FillRect(mem_dc, &cr, bg_brush);
    DeleteObject(bg_brush as _);

    SetBkMode(mem_dc, TRANSPARENT as i32);

    // Label: "Screenshot"
    SelectObject(mem_dc, st.hfont_label as _);
    SetTextColor(mem_dc, TEXT_DIM);
    let label = wide("Screenshot");
    let mut lr = RECT {
        left: BOX_LEFT,
        top: LABEL_Y,
        right: BOX_RIGHT,
        bottom: BOX_TOP,
    };
    DrawTextW(
        mem_dc,
        label.as_ptr(),
        label.len() as i32 - 1,
        &mut lr,
        DT_LEFT | DT_SINGLELINE,
    );

    // Hotkey box
    let (box_bg, box_border) = if st.recording {
        (RECORDING_BG, RECORDING_BORDER)
    } else {
        (BOX_BG, BOX_BORDER)
    };
    let box_r = RECT {
        left: BOX_LEFT,
        top: BOX_TOP,
        right: BOX_RIGHT,
        bottom: BOX_BOTTOM,
    };
    let fill_brush = CreateSolidBrush(box_bg);
    let border_pen = CreatePen(PS_SOLID, 1, box_border);
    let old_brush =
        SelectObject(mem_dc, fill_brush as _);
    let old_pen =
        SelectObject(mem_dc, border_pen as _);
    RoundRect(
        mem_dc,
        box_r.left,
        box_r.top,
        box_r.right,
        box_r.bottom,
        BOX_RADIUS,
        BOX_RADIUS,
    );
    SelectObject(mem_dc, old_brush);
    SelectObject(mem_dc, old_pen);
    DeleteObject(fill_brush as _);
    DeleteObject(border_pen as _);

    // Hotkey text inside box
    let hk = &*st.hk;
    let key_text = if st.recording {
        "Press keys...".to_string()
    } else if !hk.is_active() {
        "None".to_string()
    } else {
        crate::hotkeys::hotkey_display(hk.hotkey())
    };
    let wkey = wide(&key_text);
    SelectObject(mem_dc, st.hfont_key as _);
    let text_color = if st.recording {
        RECORDING_BORDER
    } else if !hk.is_active() {
        TEXT_DIM
    } else {
        ACCENT
    };
    SetTextColor(mem_dc, text_color);
    let mut kr = RECT {
        left: box_r.left,
        top: box_r.top,
        right: box_r.right,
        bottom: box_r.bottom,
    };
    DrawTextW(
        mem_dc,
        wkey.as_ptr(),
        wkey.len() as i32 - 1,
        &mut kr,
        DT_CENTER | DT_SINGLELINE | DT_VCENTER,
    );

    // Hint text
    SelectObject(mem_dc, st.hfont_hint as _);
    SetTextColor(mem_dc, TEXT_DIM);
    let hint = if st.recording {
        wide("ESC to cancel")
    } else {
        wide("Click to change  |  ESC to close")
    };
    let mut hr = RECT {
        left: BOX_LEFT,
        top: HINT_Y,
        right: BOX_RIGHT,
        bottom: ch,
    };
    DrawTextW(
        mem_dc,
        hint.as_ptr(),
        hint.len() as i32 - 1,
        &mut hr,
        DT_CENTER | DT_SINGLELINE,
    );

    // Blit to screen
    BitBlt(hdc, 0, 0, cw, ch, mem_dc, 0, 0, SRCCOPY);
    SelectObject(mem_dc, old_bmp);
    DeleteObject(mem_bmp as _);
    DeleteDC(mem_dc);

    EndPaint(hwnd, &ps);
}

// ── WndProc ────────────────────────────────────

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let st_ptr = GetWindowLongPtrW(hwnd, 0)
        as *mut SettingsState;
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
        WM_ERASEBKGND => 1,
        WM_LBUTTONDOWN => {
            let x =
                (lparam & 0xFFFF) as i16 as i32;
            let y = ((lparam >> 16) & 0xFFFF)
                as i16 as i32;
            let in_box = x >= BOX_LEFT
                && x <= BOX_RIGHT
                && y >= BOX_TOP
                && y <= BOX_BOTTOM;
            if in_box && !st.recording {
                st.recording = true;
                let hk = &mut *st.hk;
                hk.pause();
                InvalidateRect(
                    hwnd, ptr::null(), 0,
                );
            }
            0
        }
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            if st.recording {
                let vk = wparam as u32;
                if matches!(
                    vk as u16,
                    VK_SHIFT
                        | VK_CONTROL
                        | VK_MENU
                        | VK_LSHIFT
                        | VK_RSHIFT
                        | VK_LCONTROL
                        | VK_RCONTROL
                        | VK_LMENU
                        | VK_RMENU
                        | VK_RETURN
                        | VK_TAB
                        | VK_BACK
                        | VK_CAPITAL
                        | VK_NUMLOCK
                        | VK_SCROLL
                ) {
                    return 0;
                }

                // ESC → disable hotkey
                if vk == VK_ESCAPE as u32 {
                    st.recording = false;
                    let hk = &mut *st.hk;
                    hk.disable();
                    InvalidateRect(
                        hwnd, ptr::null(), 0,
                    );
                    return 0;
                }

                if let Some(code) = vk_to_code(vk)
                {
                    let mut mods =
                        Modifiers::empty();
                    if GetAsyncKeyState(
                        VK_CONTROL as i32,
                    ) < 0
                    {
                        mods |= Modifiers::CONTROL;
                    }
                    if GetAsyncKeyState(
                        VK_SHIFT as i32,
                    ) < 0
                    {
                        mods |= Modifiers::SHIFT;
                    }
                    if GetAsyncKeyState(
                        VK_MENU as i32,
                    ) < 0
                    {
                        mods |= Modifiers::ALT;
                    }
                    let mods_opt =
                        if mods.is_empty() {
                            None
                        } else {
                            Some(mods)
                        };
                    let hk = &mut *st.hk;
                    match hk.rebind(mods_opt, code) {
                        Ok(()) => {
                            log::info!(
                                "Hotkey rebound OK"
                            );
                        }
                        Err(e) => {
                            log::error!(
                                "Rebind: {e}"
                            );
                        }
                    }
                    st.recording = false;
                    InvalidateRect(
                        hwnd, ptr::null(), 0,
                    );
                }
            } else if wparam as u16 == VK_ESCAPE {
                PostQuitMessage(0);
            }
            0
        }
        WM_CLOSE => {
            if st.recording {
                // Cancel recording, restore hotkey
                let hk = &mut *st.hk;
                hk.resume();
                st.recording = false;
            }
            PostQuitMessage(0);
            0
        }
        _ => {
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }
}

// ── VK → Code mapping ─────────────────────────

fn vk_to_code(vk: u32) -> Option<Code> {
    match vk {
        0x70 => Some(Code::F1),
        0x71 => Some(Code::F2),
        0x72 => Some(Code::F3),
        0x73 => Some(Code::F4),
        0x74 => Some(Code::F5),
        0x75 => Some(Code::F6),
        0x76 => Some(Code::F7),
        0x77 => Some(Code::F8),
        0x78 => Some(Code::F9),
        0x79 => Some(Code::F10),
        0x7A => Some(Code::F11),
        0x7B => Some(Code::F12),
        0x41 => Some(Code::KeyA),
        0x42 => Some(Code::KeyB),
        0x43 => Some(Code::KeyC),
        0x44 => Some(Code::KeyD),
        0x45 => Some(Code::KeyE),
        0x46 => Some(Code::KeyF),
        0x47 => Some(Code::KeyG),
        0x48 => Some(Code::KeyH),
        0x49 => Some(Code::KeyI),
        0x4A => Some(Code::KeyJ),
        0x4B => Some(Code::KeyK),
        0x4C => Some(Code::KeyL),
        0x4D => Some(Code::KeyM),
        0x4E => Some(Code::KeyN),
        0x4F => Some(Code::KeyO),
        0x50 => Some(Code::KeyP),
        0x51 => Some(Code::KeyQ),
        0x52 => Some(Code::KeyR),
        0x53 => Some(Code::KeyS),
        0x54 => Some(Code::KeyT),
        0x55 => Some(Code::KeyU),
        0x56 => Some(Code::KeyV),
        0x57 => Some(Code::KeyW),
        0x58 => Some(Code::KeyX),
        0x59 => Some(Code::KeyY),
        0x5A => Some(Code::KeyZ),
        0x30 => Some(Code::Digit0),
        0x31 => Some(Code::Digit1),
        0x32 => Some(Code::Digit2),
        0x33 => Some(Code::Digit3),
        0x34 => Some(Code::Digit4),
        0x35 => Some(Code::Digit5),
        0x36 => Some(Code::Digit6),
        0x37 => Some(Code::Digit7),
        0x38 => Some(Code::Digit8),
        0x39 => Some(Code::Digit9),
        0x20 => Some(Code::Space),
        0x2C => Some(Code::PrintScreen),
        0x2D => Some(Code::Insert),
        0x2E => Some(Code::Delete),
        0x24 => Some(Code::Home),
        0x23 => Some(Code::End),
        0x21 => Some(Code::PageUp),
        0x22 => Some(Code::PageDown),
        _ => None,
    }
}
