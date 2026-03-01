use arboard::{Clipboard, ImageData};
use image::RgbaImage;

pub fn copy_image(
    rgba: &[u8],
    w: usize,
    h: usize,
) -> Result<(), String> {
    let mut cb = Clipboard::new()
        .map_err(|e| format!("Clipboard init: {e}"))?;
    let img_data = ImageData {
        width: w,
        height: h,
        bytes: rgba.into(),
    };
    cb.set_image(img_data)
        .map_err(|e| format!("Clipboard set: {e}"))?;
    Ok(())
}

pub fn copy_rgba_image(img: &RgbaImage) -> Result<(), String> {
    let (w, h) = img.dimensions();
    copy_image(img.as_raw(), w as usize, h as usize)
}
