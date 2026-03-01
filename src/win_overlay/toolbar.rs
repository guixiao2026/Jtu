use std::ptr;

use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;

use super::State;

// ── toolbar geometry ────────────────────────────

pub(super) const BTN_W: i32 = 40;
pub(super) const BTN_H: i32 = 36;
const BTN_GAP: i32 = 2;
const TB_MARGIN: i32 = 8;

pub(super) struct BtnRect {
    pub copy_r: RECT,
    pub save_r: RECT,
    pub cancel_r: RECT,
}

pub(super) fn toolbar_rects(
    sel: &RECT,
) -> BtnRect {
    let total_w = BTN_W * 3 + BTN_GAP * 2;
    let cx = (sel.left + sel.right) / 2;
    let left = cx - total_w / 2;
    let top = sel.bottom + TB_MARGIN;
    let mk = |i: i32| RECT {
        left: left + i * (BTN_W + BTN_GAP),
        top,
        right: left + i * (BTN_W + BTN_GAP)
            + BTN_W,
        bottom: top + BTN_H,
    };
    BtnRect {
        copy_r: mk(0),
        save_r: mk(1),
        cancel_r: mk(2),
    }
}

pub(super) fn pt_in_rect(
    r: &RECT,
    x: i32,
    y: i32,
) -> bool {
    x >= r.left
        && x < r.right
        && y >= r.top
        && y < r.bottom
}

// ── draw ────────────────────────────────────────

pub(super) unsafe fn draw_toolbar(
    st: &State,
    sel: &RECT,
) {
    let btns = toolbar_rects(sel);
    let hdc = st.hdc_compose;

    // Toolbar background (pill shape)
    let tb = RECT {
        left: btns.copy_r.left - 4,
        top: btns.copy_r.top - 4,
        right: btns.cancel_r.right + 4,
        bottom: btns.cancel_r.bottom + 4,
    };
    let bg_br = CreateSolidBrush(0x00302828);
    FillRect(hdc, &tb, bg_br);
    DeleteObject(bg_br as _);

    // Copy: light blue #F7C34F → BGR 0x004FC3F7
    draw_icon_btn(
        hdc,
        &btns.copy_r,
        0x00403030,
        |h, x, y| {
            draw_icon_copy(h, x, y, 0x00F7C34F);
        },
    );
    // Save: green #84C781 → BGR 0x0081C784
    draw_icon_btn(
        hdc,
        &btns.save_r,
        0x00403030,
        |h, x, y| {
            draw_icon_save(h, x, y, 0x0084C781);
        },
    );
    // Cancel: soft red #9A9AEF → BGR 0x00EF9A9A
    draw_icon_btn(
        hdc,
        &btns.cancel_r,
        0x00403030,
        |h, x, y| {
            draw_icon_cancel(h, x, y, 0x009A9AEF);
        },
    );
}

unsafe fn draw_icon_btn(
    hdc: HDC,
    r: &RECT,
    bg: u32,
    draw_icon: impl FnOnce(HDC, i32, i32),
) {
    let brush = CreateSolidBrush(bg);
    FillRect(hdc, r, brush);
    DeleteObject(brush as _);
    // Center 18x18 icon in button
    let ix = r.left + (BTN_W - 18) / 2;
    let iy = r.top + (BTN_H - 18) / 2;
    draw_icon(hdc, ix, iy);
}

/// Two overlapping documents (copy)
unsafe fn draw_icon_copy(
    hdc: HDC,
    x: i32,
    y: i32,
    col: u32,
) {
    let pen = CreatePen(0, 1, col);
    let null_br = GetStockObject(5); // NULL_BRUSH
    let op = SelectObject(hdc, pen as _);
    let ob = SelectObject(hdc, null_br);
    // back document
    Rectangle(hdc, x + 5, y, x + 17, y + 13);
    // front document
    Rectangle(hdc, x, y + 4, x + 12, y + 17);
    // lines on front doc
    let line_pen = CreatePen(0, 1, col);
    SelectObject(hdc, line_pen as _);
    MoveToEx(hdc, x + 3, y + 8, ptr::null_mut());
    LineTo(hdc, x + 9, y + 8);
    MoveToEx(hdc, x + 3, y + 11, ptr::null_mut());
    LineTo(hdc, x + 9, y + 11);
    MoveToEx(hdc, x + 3, y + 14, ptr::null_mut());
    LineTo(hdc, x + 7, y + 14);
    SelectObject(hdc, op);
    SelectObject(hdc, ob);
    DeleteObject(pen as _);
    DeleteObject(line_pen as _);
}

/// Downward arrow into tray (save)
unsafe fn draw_icon_save(
    hdc: HDC,
    x: i32,
    y: i32,
    col: u32,
) {
    let pen = CreatePen(0, 2, col);
    let op = SelectObject(hdc, pen as _);
    let cx = x + 9;
    // Arrow shaft
    MoveToEx(hdc, cx, y + 1, ptr::null_mut());
    LineTo(hdc, cx, y + 11);
    // Arrow head left
    MoveToEx(hdc, cx - 5, y + 7, ptr::null_mut());
    LineTo(hdc, cx, y + 12);
    // Arrow head right
    MoveToEx(hdc, cx + 5, y + 7, ptr::null_mut());
    LineTo(hdc, cx, y + 12);
    // Tray (U shape)
    MoveToEx(hdc, x + 1, y + 10, ptr::null_mut());
    LineTo(hdc, x + 1, y + 16);
    LineTo(hdc, x + 17, y + 16);
    LineTo(hdc, x + 17, y + 10);
    SelectObject(hdc, op);
    DeleteObject(pen as _);
}

/// X mark (cancel)
unsafe fn draw_icon_cancel(
    hdc: HDC,
    x: i32,
    y: i32,
    col: u32,
) {
    let pen = CreatePen(0, 2, col);
    let op = SelectObject(hdc, pen as _);
    MoveToEx(hdc, x + 3, y + 3, ptr::null_mut());
    LineTo(hdc, x + 16, y + 16);
    MoveToEx(hdc, x + 15, y + 3, ptr::null_mut());
    LineTo(hdc, x + 2, y + 16);
    SelectObject(hdc, op);
    DeleteObject(pen as _);
}
