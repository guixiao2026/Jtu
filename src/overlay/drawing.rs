use egui::{
    Color32, ColorImage, Pos2, Rect, Stroke,
    StrokeKind, TextureOptions, Vec2,
};

use crate::annotation::Annotation;
use crate::theme;
use crate::tools;

use super::OverlayState;

impl OverlayState {
    pub(super) fn draw_mask(
        &self,
        painter: &egui::Painter,
        screen_rect: Rect,
    ) {
        let mask_color = theme::MASK_COLOR;

        if let Some(sel) = self.selection.rect {
            let top = Rect::from_min_max(
                screen_rect.min,
                Pos2::new(
                    screen_rect.max.x,
                    sel.min.y,
                ),
            );
            let bottom = Rect::from_min_max(
                Pos2::new(
                    screen_rect.min.x,
                    sel.max.y,
                ),
                screen_rect.max,
            );
            let left = Rect::from_min_max(
                Pos2::new(
                    screen_rect.min.x,
                    sel.min.y,
                ),
                Pos2::new(sel.min.x, sel.max.y),
            );
            let right = Rect::from_min_max(
                Pos2::new(sel.max.x, sel.min.y),
                Pos2::new(
                    screen_rect.max.x,
                    sel.max.y,
                ),
            );
            for r in [top, bottom, left, right] {
                painter.rect_filled(
                    r, 0.0, mask_color,
                );
            }
        } else {
            painter.rect_filled(
                screen_rect,
                0.0,
                mask_color,
            );
        }
    }

    pub(super) fn draw_selection(
        &self,
        painter: &egui::Painter,
    ) {
        let Some(sel) = self.selection.rect else {
            return;
        };

        painter.rect_stroke(
            sel,
            0.0,
            Stroke::new(
                theme::SELECTION_BORDER_WIDTH,
                theme::SELECTION_BORDER_COLOR,
            ),
            StrokeKind::Outside,
        );

        for (handle_rect, _kind) in
            self.selection.handle_rects()
        {
            let center = handle_rect.center();
            let r = theme::CONTROL_POINT_RADIUS;
            painter.circle_filled(
                center,
                r + 2.0,
                theme::CONTROL_POINT_GLOW,
            );
            painter.circle_filled(
                center,
                r,
                theme::CONTROL_POINT_FILL,
            );
            painter.circle_stroke(
                center,
                r,
                Stroke::new(
                    theme::CONTROL_POINT_STROKE_WIDTH,
                    theme::CONTROL_POINT_STROKE_COLOR,
                ),
            );
        }

        if let Some((w, h)) =
            self.selection.dimensions()
        {
            let text =
                format!("W: {} px  H: {} px", w, h);
            let font = egui::FontId::proportional(
                theme::SIZE_CAPSULE_FONT_SIZE,
            );
            let galley = painter.layout_no_wrap(
                text,
                font,
                theme::SIZE_CAPSULE_TEXT,
            );
            let text_size = galley.size();
            let capsule_w = text_size.x
                + theme::SIZE_CAPSULE_PADDING_H * 2.0;
            let capsule_h = text_size.y
                + theme::SIZE_CAPSULE_PADDING_V * 2.0;
            let capsule_x = sel.min.x;
            let capsule_y = sel.min.y
                - capsule_h
                - theme::SIZE_CAPSULE_GAP;
            let capsule_rect = Rect::from_min_size(
                Pos2::new(capsule_x, capsule_y),
                egui::vec2(capsule_w, capsule_h),
            );
            painter.rect_filled(
                capsule_rect,
                theme::SIZE_CAPSULE_CORNER_RADIUS,
                theme::SIZE_CAPSULE_BG,
            );
            let text_pos = Pos2::new(
                capsule_x
                    + theme::SIZE_CAPSULE_PADDING_H,
                capsule_y
                    + theme::SIZE_CAPSULE_PADDING_V,
            );
            painter.galley(
                text_pos,
                galley,
                Color32::TRANSPARENT,
            );
        }
    }

    pub(super) fn draw_annotations(
        &self,
        painter: &egui::Painter,
        screen_rect: Rect,
    ) {
        for ann in self.annotations.annotations() {
            tools::render_annotation(
                ann,
                painter,
                &self.screenshot,
                screen_rect,
                self.screen_size,
            );
        }

        if !self.current_points.is_empty()
            && self.is_drawing
        {
            let temp_ann = Annotation {
                kind: self.active_tool,
                color: self.tool_color,
                stroke_width: self.stroke_width,
                points: self.current_points.clone(),
                text: String::new(),
            };
            tools::render_annotation(
                &temp_ann,
                painter,
                &self.screenshot,
                screen_rect,
                self.screen_size,
            );
        }
    }

    pub(super) fn draw_magnifier(
        &self,
        painter: &egui::Painter,
        screen_rect: Rect,
    ) {
        let Some(cursor) = self.cursor_pos else {
            return;
        };
        let sw = self.screen_size.0 as f32;
        let sh = self.screen_size.1 as f32;
        let zoom = theme::MAGNIFIER_ZOOM;
        let mag_size = theme::MAGNIFIER_SIZE;
        let sample_r = (mag_size / zoom / 2.0) as i32;
        let px = (cursor.x * sw / screen_rect.width())
            as i32;
        let py = (cursor.y * sh / screen_rect.height())
            as i32;

        let diameter = (sample_r * 2 + 1) as usize;
        let mut pixels =
            vec![0u8; diameter * diameter * 4];
        let img_w = self.screen_size.0 as i32;
        let img_h = self.screen_size.1 as i32;
        for dy in -sample_r..=sample_r {
            for dx in -sample_r..=sample_r {
                let sx = (px + dx).clamp(0, img_w - 1)
                    as u32;
                let sy = (py + dy).clamp(0, img_h - 1)
                    as u32;
                let p = self.screenshot.get_pixel(sx, sy);
                let idx = ((dy + sample_r) as usize
                    * diameter
                    + (dx + sample_r) as usize)
                    * 4;
                pixels[idx] = p[0];
                pixels[idx + 1] = p[1];
                pixels[idx + 2] = p[2];
                pixels[idx + 3] = 255;
            }
        }

        let color_image =
            ColorImage::from_rgba_unmultiplied(
                [diameter, diameter],
                &pixels,
            );
        let tex = painter.ctx().load_texture(
            "magnifier_tex",
            color_image,
            TextureOptions::NEAREST,
        );

        let info_h = theme::MAGNIFIER_INFO_HEIGHT;
        let total_h = mag_size + info_h;
        let mut mag_x = cursor.x + 20.0;
        let mut mag_y =
            cursor.y + theme::MAGNIFIER_OFFSET_Y;
        if mag_x + mag_size > screen_rect.max.x {
            mag_x = cursor.x - mag_size - 20.0;
        }
        if mag_y + total_h > screen_rect.max.y {
            mag_y = cursor.y - total_h - 20.0;
        }

        let mag_rect = Rect::from_min_size(
            Pos2::new(mag_x, mag_y),
            Vec2::splat(mag_size),
        );
        painter.rect_filled(
            mag_rect,
            0.0,
            theme::MAGNIFIER_BG,
        );
        painter.image(
            tex.id(),
            mag_rect,
            Rect::from_min_max(
                Pos2::ZERO,
                Pos2::new(1.0, 1.0),
            ),
            Color32::WHITE,
        );

        let cross_center = mag_rect.center();
        let cell = mag_size / diameter as f32;
        painter.line_segment(
            [
                Pos2::new(mag_rect.min.x, cross_center.y),
                Pos2::new(mag_rect.max.x, cross_center.y),
            ],
            Stroke::new(
                1.0,
                theme::MAGNIFIER_CROSSHAIR_COLOR,
            ),
        );
        painter.line_segment(
            [
                Pos2::new(cross_center.x, mag_rect.min.y),
                Pos2::new(cross_center.x, mag_rect.max.y),
            ],
            Stroke::new(
                1.0,
                theme::MAGNIFIER_CROSSHAIR_COLOR,
            ),
        );
        let cell_rect = Rect::from_center_size(
            cross_center,
            Vec2::splat(cell),
        );
        painter.rect_stroke(
            cell_rect,
            0.0,
            Stroke::new(
                1.5,
                Color32::WHITE,
            ),
            StrokeKind::Outside,
        );

        painter.rect_stroke(
            mag_rect,
            0.0,
            Stroke::new(
                theme::MAGNIFIER_BORDER_WIDTH,
                theme::MAGNIFIER_BORDER_COLOR,
            ),
            StrokeKind::Outside,
        );

        let cpx = px.clamp(0, img_w - 1) as u32;
        let cpy = py.clamp(0, img_h - 1) as u32;
        let pixel = self.screenshot.get_pixel(cpx, cpy);
        let hex_color = format!(
            "#{:02X}{:02X}{:02X}",
            pixel[0], pixel[1], pixel[2]
        );
        let coord_text = format!(
            "({}, {})", cpx, cpy
        );

        let info_rect = Rect::from_min_size(
            Pos2::new(mag_x, mag_y + mag_size),
            Vec2::new(mag_size, info_h),
        );
        painter.rect_filled(
            info_rect,
            0.0,
            theme::MAGNIFIER_BG,
        );
        painter.rect_stroke(
            info_rect,
            0.0,
            Stroke::new(
                theme::MAGNIFIER_BORDER_WIDTH,
                theme::MAGNIFIER_BORDER_COLOR,
            ),
            StrokeKind::Outside,
        );

        let swatch_size = theme::MAGNIFIER_COLOR_SWATCH;
        let swatch_rect = Rect::from_min_size(
            Pos2::new(
                info_rect.min.x + 6.0,
                info_rect.center().y - swatch_size / 2.0,
            ),
            Vec2::splat(swatch_size),
        );
        painter.rect_filled(
            swatch_rect,
            2.0,
            Color32::from_rgb(
                pixel[0], pixel[1], pixel[2],
            ),
        );
        painter.rect_stroke(
            swatch_rect,
            2.0,
            Stroke::new(1.0, Color32::WHITE),
            StrokeKind::Outside,
        );

        let font = egui::FontId::proportional(
            theme::MAGNIFIER_INFO_FONT_SIZE,
        );
        let text_x = swatch_rect.max.x + 6.0;
        painter.text(
            Pos2::new(text_x, info_rect.min.y + 4.0),
            egui::Align2::LEFT_TOP,
            &hex_color,
            font.clone(),
            Color32::WHITE,
        );
        painter.text(
            Pos2::new(
                text_x,
                info_rect.min.y + 4.0
                    + theme::MAGNIFIER_INFO_FONT_SIZE
                    + 2.0,
            ),
            egui::Align2::LEFT_TOP,
            &coord_text,
            font,
            Color32::from_rgb(180, 180, 180),
        );
    }
}
