use egui::{Color32, Pos2, Rect, Stroke};

use crate::annotation::{Annotation, ToolKind};
use crate::theme;

use super::OverlayState;

impl OverlayState {
    pub(super) fn show_color_picker(
        &mut self,
        ctx: &egui::Context,
        sel_rect: Rect,
    ) {
        if self.active_tool == ToolKind::Select
            || self.active_tool == ToolKind::Eraser
        {
            return;
        }

        let top_y = sel_rect.max.y + 48.0;
        let center_x = sel_rect.center().x;

        egui::Area::new(egui::Id::new("color_picker"))
            .fixed_pos(Pos2::new(
                center_x - 140.0,
                top_y,
            ))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(theme::TOOLBAR_BG)
                    .corner_radius(
                        theme::TOOLBAR_CORNER_RADIUS,
                    )
                    .inner_margin(
                        theme::TOOLBAR_INNER_MARGIN,
                    )
                    .shadow(egui::Shadow {
                        blur: theme::TOOLBAR_GLOW_BLUR,
                        color: theme::TOOLBAR_GLOW_COLOR,
                        ..Default::default()
                    })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut()
                                .item_spacing
                                .x = 6.0;
                            let r =
                                theme::COLOR_SWATCH_RADIUS;
                            for c in
                                theme::COLOR_PICKER_COLORS
                            {
                                let (rect, resp) =
                                    ui.allocate_exact_size(
                                        egui::vec2(
                                            r * 2.0 + 4.0,
                                            r * 2.0 + 4.0,
                                        ),
                                        egui::Sense::click(),
                                    );
                                let center =
                                    rect.center();
                                ui.painter()
                                    .circle_filled(
                                        center, r, c,
                                    );
                                if self.tool_color == c {
                                    ui.painter()
                                        .circle_stroke(
                                        center,
                                        r + 2.0,
                                        Stroke::new(
                                            theme::COLOR_SWATCH_SELECTED_STROKE,
                                            Color32::WHITE,
                                        ),
                                    );
                                }
                                if resp.clicked() {
                                    self.tool_color = c;
                                }
                            }

                            ui.add_space(4.0);

                            let slider_resp = ui.add(
                                egui::Slider::new(
                                    &mut self
                                        .stroke_width,
                                    1.0..=10.0,
                                )
                                .show_value(false),
                            );
                            let val_text = format!(
                                "{:.0}",
                                self.stroke_width
                            );
                            let val_pos = Pos2::new(
                                slider_resp.rect.max.x
                                    + 4.0,
                                slider_resp
                                    .rect
                                    .center()
                                    .y,
                            );
                            ui.painter().text(
                                val_pos,
                                egui::Align2::LEFT_CENTER,
                                val_text,
                                egui::FontId::proportional(
                                    11.0,
                                ),
                                Color32::from_rgb(
                                    160, 160, 160,
                                ),
                            );
                        });
                    });
            });
    }

    pub(super) fn show_text_input(
        &mut self,
        ctx: &egui::Context,
    ) {
        let Some(pos) = self.text_input_pos else {
            return;
        };

        egui::Area::new(egui::Id::new(
            "text_input_area",
        ))
        .fixed_pos(pos)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(theme::TEXT_INPUT_BG)
                .corner_radius(
                    theme::TEXT_INPUT_CORNER_RADIUS,
                )
                .inner_margin(4.0)
                .show(ui, |ui| {
                    let resp = ui.text_edit_singleline(
                        &mut self.text_input,
                    );
                    resp.request_focus();

                    if ui
                        .input(|i| {
                            i.key_pressed(
                                egui::Key::Enter,
                            )
                        })
                        && !self.text_input.is_empty()
                    {
                        let ann = Annotation {
                            kind: ToolKind::Text,
                            color: self.tool_color,
                            stroke_width: self
                                .stroke_width,
                            points: vec![pos],
                            text: self
                                .text_input
                                .clone(),
                        };
                        self.annotations.push(ann);
                        self.text_input.clear();
                        self.text_input_pos = None;
                    }

                    if ui.input(|i| {
                        i.key_pressed(
                            egui::Key::Escape,
                        )
                    }) {
                        self.text_input.clear();
                        self.text_input_pos = None;
                    }
                });
        });
    }
}
