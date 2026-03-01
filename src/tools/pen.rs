use egui::{Pos2, Stroke};

use super::DrawContext;

pub fn draw(ctx: &DrawContext, points: &[Pos2]) {
    if points.len() < 2 {
        return;
    }
    let stroke =
        Stroke::new(ctx.stroke_width, ctx.color);

    if points.len() <= 3 {
        for w in points.windows(2) {
            ctx.painter
                .line_segment([w[0], w[1]], stroke);
        }
        return;
    }

    let smooth = catmull_rom_chain(points, 8);
    for w in smooth.windows(2) {
        ctx.painter
            .line_segment([w[0], w[1]], stroke);
    }
}

fn catmull_rom_chain(
    pts: &[Pos2],
    subdivisions: usize,
) -> Vec<Pos2> {
    let n = pts.len();
    let mut result = Vec::with_capacity(
        (n - 1) * subdivisions + 1,
    );
    result.push(pts[0]);

    for i in 0..n - 1 {
        let p0 = if i == 0 { pts[0] } else { pts[i - 1] };
        let p1 = pts[i];
        let p2 = pts[i + 1];
        let p3 = if i + 2 < n {
            pts[i + 2]
        } else {
            pts[n - 1]
        };

        for s in 1..=subdivisions {
            let t = s as f32 / subdivisions as f32;
            let tt = t * t;
            let ttt = tt * t;

            let x = 0.5
                * ((2.0 * p1.x)
                    + (-p0.x + p2.x) * t
                    + (2.0 * p0.x - 5.0 * p1.x
                        + 4.0 * p2.x
                        - p3.x)
                        * tt
                    + (-p0.x + 3.0 * p1.x
                        - 3.0 * p2.x
                        + p3.x)
                        * ttt);
            let y = 0.5
                * ((2.0 * p1.y)
                    + (-p0.y + p2.y) * t
                    + (2.0 * p0.y - 5.0 * p1.y
                        + 4.0 * p2.y
                        - p3.y)
                        * tt
                    + (-p0.y + 3.0 * p1.y
                        - 3.0 * p2.y
                        + p3.y)
                        * ttt);
            result.push(Pos2::new(x, y));
        }
    }
    result
}
