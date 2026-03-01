use egui::Pos2;

use super::DrawContext;

pub fn draw(
    ctx: &DrawContext,
    points: &[Pos2],
    text: &str,
) {
    if points.is_empty() || text.is_empty() {
        return;
    }
    ctx.painter.text(
        points[0],
        egui::Align2::LEFT_TOP,
        text,
        egui::FontId::proportional(
            ctx.stroke_width * 5.0,
        ),
        ctx.color,
    );
}
