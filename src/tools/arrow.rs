use egui::{
    epaint::PathShape, Color32, Pos2, Stroke, Vec2,
};

use super::DrawContext;
use crate::theme;

pub fn draw(ctx: &DrawContext, points: &[Pos2]) {
    if points.len() < 2 {
        return;
    }
    let from = points[0];
    let to = points[1];
    let dir = (to - from).normalized();
    let head_len = (ctx.stroke_width
        * theme::ARROW_HEAD_LENGTH_FACTOR)
        .max(theme::ARROW_HEAD_MIN_LENGTH);
    let head_width =
        head_len * theme::ARROW_HEAD_WIDTH_RATIO;

    let base = to - dir * head_len;
    let perp = Vec2::new(-dir.y, dir.x);
    let p1 = base + perp * head_width;
    let p2 = base - perp * head_width;

    let stroke =
        Stroke::new(ctx.stroke_width, ctx.color);
    ctx.painter.line_segment([from, base], stroke);

    let triangle =
        PathShape::convex_polygon(
            vec![to, p1, p2],
            ctx.color,
            Stroke::new(0.0, Color32::TRANSPARENT),
        );
    ctx.painter.add(triangle);
}
