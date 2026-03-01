use image::RgbaImage;
use xcap::Monitor;

pub struct MonitorInfo {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
}

pub fn capture_primary_monitor(
) -> Result<RgbaImage, String> {
    let monitors = Monitor::all().map_err(|e| {
        format!("Failed to enumerate monitors: {e}")
    })?;
    let primary = monitors
        .into_iter()
        .find(|m| {
            m.is_primary().unwrap_or(false)
        })
        .ok_or_else(|| {
            "No primary monitor found".to_string()
        })?;
    primary
        .capture_image()
        .map_err(|e| format!("Failed to capture: {e}"))
}

pub fn capture_all_monitors()
    -> Result<Vec<(MonitorInfo, RgbaImage)>, String>
{
    let monitors = Monitor::all().map_err(|e| {
        format!("Failed to enumerate monitors: {e}")
    })?;
    let mut results = Vec::new();
    for mon in monitors {
        let info = MonitorInfo {
            name: mon.name().unwrap_or_default(),
            x: mon.x().unwrap_or(0),
            y: mon.y().unwrap_or(0),
            width: mon.width().unwrap_or(0),
            height: mon.height().unwrap_or(0),
            scale_factor: mon
                .scale_factor()
                .unwrap_or(1.0),
        };
        let img = mon
            .capture_image()
            .map_err(|e| {
                format!("Capture {}: {e}", info.name)
            })?;
        results.push((info, img));
    }
    Ok(results)
}

pub fn capture_region(
    img: &RgbaImage,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> RgbaImage {
    image::imageops::crop_imm(img, x, y, w, h)
        .to_image()
}

/// In-place RGBA↔BGRA swap. Works in both directions
/// since it just swaps bytes 0↔2 in each 4-byte pixel.
/// Batches 2 pixels per iteration for better throughput.
pub fn swap_rb_inplace(buf: &mut [u8]) {
    let len = buf.len();
    let mut i = 0;
    while i + 8 <= len {
        buf.swap(i, i + 2);
        buf.swap(i + 4, i + 6);
        i += 8;
    }
    if i + 4 <= len {
        buf.swap(i, i + 2);
    }
}

/// Crop a region from a full-screen BGRA buffer
/// and convert to RgbaImage (B↔R swap per pixel).
/// Only processes the selected region — fast even
/// in debug builds.
pub fn crop_bgra_to_rgba(
    bgra: &[u8],
    full_w: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> RgbaImage {
    let stride = (full_w * 4) as usize;
    let row_bytes = (w * 4) as usize;
    let mut out = vec![0u8; (w * h * 4) as usize];
    for row in 0..h as usize {
        let src_off =
            (y as usize + row) * stride
                + (x as usize) * 4;
        let dst_off = row * row_bytes;
        let src = &bgra[src_off..src_off + row_bytes];
        let dst =
            &mut out[dst_off..dst_off + row_bytes];
        dst.copy_from_slice(src);
        swap_rb_inplace(dst);
    }
    RgbaImage::from_raw(w, h, out)
        .expect("crop_bgra_to_rgba: invalid dims")
}

#[cfg(windows)]
mod dxgi;

#[cfg(windows)]
pub use dxgi::DxgiCapture;
