use egui::{Pos2, Rect, Stroke};

use super::DrawContext;

pub fn draw(ctx: &DrawContext, points: &[Pos2]) {
    if points.len() < 2 {
        return;
    }
    let r = Rect::from_two_pos(points[0], points[1]);
    let center = r.center();
    let radius = r.size() / 2.0;
    ctx.painter.circle_stroke(
        center,
        radius.x.min(radius.y),
        Stroke::new(ctx.stroke_width, ctx.color),
    );
    if (radius.x - radius.y).abs() > 1.0 {
        let n = 64;
        let pts: Vec<Pos2> = (0..=n)
            .map(|i| {
                let t = i as f32 / n as f32
                    * std::f32::consts::TAU;
                Pos2::new(
                    center.x + radius.x * t.cos(),
                    center.y + radius.y * t.sin(),
                )
            })
            .collect();
        let stroke =
            Stroke::new(ctx.stroke_width, ctx.color);
        for w in pts.windows(2) {
            ctx.painter.line_segment(
                [w[0], w[1]],
                stroke,
            );
        }
    }
}
