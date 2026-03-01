pub mod arrow;
pub mod blur;
pub mod ellipse;
pub mod pen;
pub mod rect;
pub mod text;

use egui::{Color32, Painter, Pos2};
use image::RgbaImage;

use crate::annotation::Annotation;

pub struct DrawContext<'a> {
    pub painter: &'a Painter,
    pub color: Color32,
    pub stroke_width: f32,
}

pub fn render_annotation(
    ann: &Annotation,
    painter: &Painter,
    screenshot: &RgbaImage,
    screen_rect: egui::Rect,
    screen_size: (u32, u32),
) {
    let ctx = DrawContext {
        painter,
        color: ann.color,
        stroke_width: ann.stroke_width,
    };
    match ann.kind {
        crate::annotation::ToolKind::Rect => {
            rect::draw(&ctx, &ann.points);
        }
        crate::annotation::ToolKind::Ellipse => {
            ellipse::draw(&ctx, &ann.points);
        }
        crate::annotation::ToolKind::Arrow => {
            arrow::draw(&ctx, &ann.points);
        }
        crate::annotation::ToolKind::Pen => {
            pen::draw(&ctx, &ann.points);
        }
        crate::annotation::ToolKind::Text => {
            text::draw(&ctx, &ann.points, &ann.text);
        }
        crate::annotation::ToolKind::Blur => {
            blur::draw(
                &ctx,
                &ann.points,
                screenshot,
                screen_rect,
                screen_size,
            );
        }
        _ => {}
    }
}

pub fn interpolate_points(
    from: Pos2,
    to: Pos2,
    spacing: f32,
) -> Vec<Pos2> {
    let dist = from.distance(to);
    if dist < spacing {
        return vec![to];
    }
    let steps = (dist / spacing).ceil() as usize;
    (1..=steps)
        .map(|i| {
            let t = i as f32 / steps as f32;
            Pos2::new(
                from.x + (to.x - from.x) * t,
                from.y + (to.y - from.y) * t,
            )
        })
        .collect()
}
