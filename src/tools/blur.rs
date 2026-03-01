use egui::{Color32, Pos2, Rect};
use image::RgbaImage;

use super::DrawContext;
use crate::theme;

pub fn draw(
    ctx: &DrawContext,
    points: &[Pos2],
    screenshot: &RgbaImage,
    screen_rect: Rect,
    screen_size: (u32, u32),
) {
    if points.len() < 2 {
        return;
    }
    let r = Rect::from_two_pos(points[0], points[1]);
    let block = theme::BLUR_BLOCK_SIZE;
    let img_w = screen_size.0 as f32;
    let img_h = screen_size.1 as f32;
    let scr_w = screen_rect.width();
    let scr_h = screen_rect.height();

    let cols = (r.width() / block).ceil() as usize;
    let rows = (r.height() / block).ceil() as usize;

    for row in 0..rows {
        for col in 0..cols {
            let sx = r.min.x + col as f32 * block;
            let sy = r.min.y + row as f32 * block;
            let sw = block.min(r.max.x - sx);
            let sh = block.min(r.max.y - sy);

            let ix = ((sx - screen_rect.min.x) / scr_w
                * img_w) as u32;
            let iy = ((sy - screen_rect.min.y) / scr_h
                * img_h) as u32;
            let iw = (sw / scr_w * img_w)
                .max(1.0) as u32;
            let ih = (sh / scr_h * img_h)
                .max(1.0) as u32;

            let avg = avg_color(
                screenshot, ix, iy, iw, ih,
            );

            let block_rect = Rect::from_min_size(
                Pos2::new(sx, sy),
                egui::vec2(sw, sh),
            );
            ctx.painter
                .rect_filled(block_rect, 0.0, avg);
        }
    }
}

#[inline]
fn avg_color(
    img: &RgbaImage,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> Color32 {
    let (iw, ih) = img.dimensions();
    let x0 = x.min(iw.saturating_sub(1));
    let y0 = y.min(ih.saturating_sub(1));
    let x1 = (x + w).min(iw);
    let y1 = (y + h).min(ih);

    let raw = img.as_raw();
    let img_w = iw as usize;

    let mut rt: u64 = 0;
    let mut gt: u64 = 0;
    let mut bt: u64 = 0;
    let mut count: u64 = 0;

    for py in y0..y1 {
        for px in x0..x1 {
            let idx =
                (py as usize * img_w + px as usize)
                    * 4;
            rt += raw[idx] as u64;
            gt += raw[idx + 1] as u64;
            bt += raw[idx + 2] as u64;
            count += 1;
        }
    }

    if count == 0 {
        return Color32::from_gray(128);
    }
    Color32::from_rgb(
        (rt / count) as u8,
        (gt / count) as u8,
        (bt / count) as u8,
    )
}
