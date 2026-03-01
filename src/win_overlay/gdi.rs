use std::ptr;

use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;

use super::toolbar::draw_toolbar;
use super::{State, sel_rect};

// ── pixel helpers ───────────────────────────────

/// Darken BGRA pixels in-place via raw pointer.
/// Zero allocation, zero bounds checks.
pub(super) unsafe fn darken_inplace(
    bits: *mut u8,
    len: usize,
) {
    let mut i = 0;
    while i + 4 <= len {
        let p = bits.add(i);
        *p = ((*p as u32 * 100) >> 8) as u8;
        let p1 = p.add(1);
        *p1 = ((*p1 as u32 * 100) >> 8) as u8;
        let p2 = p.add(2);
        *p2 = ((*p2 as u32 * 100) >> 8) as u8;
        // alpha (p+3) unchanged
        i += 4;
    }
}

// ── bitmap helpers ──────────────────────────────

pub(super) fn bitmapinfo_header(
    w: i32,
    h: i32,
) -> BITMAPINFO {
    BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<
                BITMAPINFOHEADER,
            >() as u32,
            biWidth: w,
            biHeight: -h, // top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: 0, // BI_RGB
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [RGBQUAD {
            rgbBlue: 0,
            rgbGreen: 0,
            rgbRed: 0,
            rgbReserved: 0,
        }],
    }
}

pub(super) unsafe fn create_dib(
    w: i32,
    h: i32,
    pixels: &[u8],
) -> (HDC, HBITMAP) {
    let hdc_screen = GetDC(0 as HWND);
    let hdc = CreateCompatibleDC(hdc_screen);
    let bi = bitmapinfo_header(w, h);
    let mut bits: *mut std::ffi::c_void =
        ptr::null_mut();
    let hbmp = CreateDIBSection(
        hdc,
        &bi,
        0, // DIB_RGB_COLORS
        &mut bits,
        0 as HANDLE,
        0,
    );
    if !bits.is_null() {
        ptr::copy_nonoverlapping(
            pixels.as_ptr(),
            bits as *mut u8,
            pixels.len(),
        );
    }
    SelectObject(hdc, hbmp as _);
    ReleaseDC(0 as HWND, hdc_screen);
    (hdc, hbmp)
}

/// Create a DIB section and return the raw bits
/// pointer for direct writing (zero-init).
pub(super) unsafe fn create_dib_uninit(
    w: i32,
    h: i32,
) -> (HDC, HBITMAP, *mut u8) {
    let hdc_screen = GetDC(0 as HWND);
    let hdc = CreateCompatibleDC(hdc_screen);
    let bi = bitmapinfo_header(w, h);
    let mut bits: *mut std::ffi::c_void =
        ptr::null_mut();
    let hbmp = CreateDIBSection(
        hdc,
        &bi,
        0,
        &mut bits,
        0 as HANDLE,
        0,
    );
    SelectObject(hdc, hbmp as _);
    ReleaseDC(0 as HWND, hdc_screen);
    (hdc, hbmp, bits as *mut u8)
}

pub(super) unsafe fn create_empty_dib(
    w: i32,
    h: i32,
) -> (HDC, HBITMAP) {
    let hdc_screen = GetDC(0 as HWND);
    let hdc = CreateCompatibleDC(hdc_screen);
    let hbmp = CreateCompatibleBitmap(
        hdc_screen,
        w,
        h,
    );
    SelectObject(hdc, hbmp as _);
    ReleaseDC(0 as HWND, hdc_screen);
    (hdc, hbmp)
}

// ── paint ───────────────────────────────────────

pub(super) unsafe fn paint(
    hwnd: HWND,
    st: &State,
) {
    let mut ps: PAINTSTRUCT = std::mem::zeroed();
    let hdc = BeginPaint(hwnd, &mut ps);

    // 1) Dark background → compose
    BitBlt(
        st.hdc_compose,
        0,
        0,
        st.width,
        st.height,
        st.hdc_dark,
        0,
        0,
        SRCCOPY,
    );

    if st.has_selection || st.selecting {
        let r = sel_rect(st);
        let sw = r.right - r.left;
        let sh = r.bottom - r.top;
        if sw > 0 && sh > 0 {
            // 2) Bright selection area
            BitBlt(
                st.hdc_compose,
                r.left,
                r.top,
                sw,
                sh,
                st.hdc_orig,
                r.left,
                r.top,
                SRCCOPY,
            );

            // 3) Selection border (white 2px)
            let pen = CreatePen(0, 2, 0x00FFFFFF);
            let brush =
                GetStockObject(5); // NULL_BRUSH
            let old_pen =
                SelectObject(st.hdc_compose, pen as _);
            let old_brush = SelectObject(
                st.hdc_compose,
                brush,
            );
            Rectangle(
                st.hdc_compose,
                r.left,
                r.top,
                r.right,
                r.bottom,
            );
            SelectObject(st.hdc_compose, old_pen);
            SelectObject(st.hdc_compose, old_brush);
            DeleteObject(pen as _);

            // 4) Toolbar buttons
            if st.has_selection && !st.selecting {
                draw_toolbar(st, &r);
            }
        }
    }

    // 5) Magnifier (during selection or before)
    if !st.has_selection || st.selecting {
        draw_magnifier(st);
    }

    // 6) Compose → screen
    BitBlt(
        hdc,
        0,
        0,
        st.width,
        st.height,
        st.hdc_compose,
        0,
        0,
        SRCCOPY,
    );

    EndPaint(hwnd, &ps);
}

// ── magnifier ────────────────────────────────────

const MAG_SIZE: i32 = 240;
const MAG_ZOOM: i32 = 8;
const MAG_OFFSET: i32 = 20;
const MAG_INFO_H: i32 = 32;
const MAG_BORDER: u32 = 0x00FF9640; // BGR for #4096FF

unsafe fn draw_magnifier(st: &State) {
    let hdc = st.hdc_compose;
    let cx = st.cursor_x;
    let cy = st.cursor_y;
    let sample = MAG_SIZE / MAG_ZOOM;

    // Position: bottom-right of cursor, flip if
    // near edge
    let mut mx = cx + MAG_OFFSET;
    let mut my = cy + MAG_OFFSET;
    let total_h = MAG_SIZE + MAG_INFO_H;
    if mx + MAG_SIZE > st.width {
        mx = cx - MAG_SIZE - MAG_OFFSET;
    }
    if my + total_h > st.height {
        my = cy - total_h - MAG_OFFSET;
    }

    // Zoom area from original screenshot
    let src_x = cx - sample / 2;
    let src_y = cy - sample / 2;
    SetStretchBltMode(hdc, 3); // COLORONCOLOR
    StretchBlt(
        hdc,
        mx,
        my,
        MAG_SIZE,
        MAG_SIZE,
        st.hdc_orig,
        src_x,
        src_y,
        sample,
        sample,
        SRCCOPY,
    );

    // Border
    let pen = CreatePen(0, 2, MAG_BORDER);
    let null_br = GetStockObject(5); // NULL_BRUSH
    let old_pen = SelectObject(hdc, pen as _);
    let old_br = SelectObject(hdc, null_br);
    Rectangle(
        hdc,
        mx,
        my,
        mx + MAG_SIZE,
        my + MAG_SIZE,
    );
    SelectObject(hdc, old_pen);
    SelectObject(hdc, old_br);
    DeleteObject(pen as _);

    // Crosshair
    let cross_pen =
        CreatePen(0, 1, 0x00B4FF40); // light blue
    let old_cp = SelectObject(hdc, cross_pen as _);
    let center_x = mx + MAG_SIZE / 2;
    let center_y = my + MAG_SIZE / 2;
    let cell = MAG_SIZE / sample;
    // Horizontal line
    MoveToEx(hdc, mx, center_y, ptr::null_mut());
    LineTo(hdc, mx + MAG_SIZE, center_y);
    // Vertical line
    MoveToEx(hdc, center_x, my, ptr::null_mut());
    LineTo(hdc, center_x, my + MAG_SIZE);
    SelectObject(hdc, old_cp);
    DeleteObject(cross_pen as _);

    // Center pixel highlight box
    let hi_pen = CreatePen(0, 1, 0x00FFFFFF);
    let old_hp = SelectObject(hdc, hi_pen as _);
    let old_hb = SelectObject(hdc, null_br);
    Rectangle(
        hdc,
        center_x - cell / 2,
        center_y - cell / 2,
        center_x + cell / 2 + 1,
        center_y + cell / 2 + 1,
    );
    SelectObject(hdc, old_hp);
    SelectObject(hdc, old_hb);
    DeleteObject(hi_pen as _);

    // Info panel below magnifier
    let info_r = RECT {
        left: mx,
        top: my + MAG_SIZE,
        right: mx + MAG_SIZE,
        bottom: my + MAG_SIZE + MAG_INFO_H,
    };
    let info_bg = CreateSolidBrush(0x00282828);
    FillRect(hdc, &info_r, info_bg);
    DeleteObject(info_bg as _);

    // Border around info panel
    let info_pen = CreatePen(0, 1, MAG_BORDER);
    let old_ip = SelectObject(hdc, info_pen as _);
    let old_ib = SelectObject(hdc, null_br);
    Rectangle(
        hdc,
        info_r.left,
        info_r.top,
        info_r.right,
        info_r.bottom,
    );
    SelectObject(hdc, old_ip);
    SelectObject(hdc, old_ib);
    DeleteObject(info_pen as _);

    // Read pixel color from original screenshot
    let pixel = GetPixel(
        st.hdc_orig,
        cx.clamp(0, st.width - 1),
        cy.clamp(0, st.height - 1),
    );
    let r = (pixel & 0xFF) as u8;
    let g = ((pixel >> 8) & 0xFF) as u8;
    let b = ((pixel >> 16) & 0xFF) as u8;

    // Color swatch
    let swatch_r = RECT {
        left: info_r.left + 4,
        top: info_r.top + 4,
        right: info_r.left + 4 + 14,
        bottom: info_r.top + 4 + 14,
    };
    let swatch_br =
        CreateSolidBrush(pixel & 0x00FFFFFF);
    FillRect(hdc, &swatch_r, swatch_br);
    DeleteObject(swatch_br as _);
    // Swatch border
    let sw_pen = CreatePen(0, 1, 0x00FFFFFF);
    let old_sp = SelectObject(hdc, sw_pen as _);
    let old_sb = SelectObject(hdc, null_br);
    Rectangle(
        hdc,
        swatch_r.left,
        swatch_r.top,
        swatch_r.right,
        swatch_r.bottom,
    );
    SelectObject(hdc, old_sp);
    SelectObject(hdc, old_sb);
    DeleteObject(sw_pen as _);

    // Text: hex color + coordinates
    SetBkMode(hdc, 1); // TRANSPARENT
    SetTextColor(hdc, 0x00FFFFFF);
    let font = CreateFontW(
        13, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 0, 0,
        wide_c("Consolas").as_ptr(),
    );
    let old_font = SelectObject(hdc, font as _);

    let hex_text = format!(
        "#{:02X}{:02X}{:02X}\0", r, g, b
    );
    let hex_bytes: Vec<u16> = hex_text
        .trim_end_matches('\0')
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    TextOutW(
        hdc,
        swatch_r.right + 4,
        info_r.top + 3,
        hex_bytes.as_ptr(),
        (hex_bytes.len() - 1) as i32,
    );

    let coord_text =
        format!("({},{})\0", cx, cy);
    let coord_bytes: Vec<u16> = coord_text
        .trim_end_matches('\0')
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    SetTextColor(hdc, 0x00B4B4B4);
    TextOutW(
        hdc,
        swatch_r.right + 4,
        info_r.top + 16,
        coord_bytes.as_ptr(),
        (coord_bytes.len() - 1) as i32,
    );

    SelectObject(hdc, old_font);
    DeleteObject(font as _);
}

fn wide_c(s: &str) -> Vec<u16> {
    s.encode_utf16()
        .chain(std::iter::once(0))
        .collect()
}
