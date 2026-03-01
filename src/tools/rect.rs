use egui::{Pos2, Rect, Stroke, StrokeKind};

use super::DrawContext;

pub fn draw(ctx: &DrawContext, points: &[Pos2]) {
    if points.len() < 2 {
        return;
    }
    let r = Rect::from_two_pos(points[0], points[1]);
    ctx.painter.rect_stroke(
        r,
        0.0,
        Stroke::new(ctx.stroke_width, ctx.color),
        StrokeKind::Outside,
    );
}
