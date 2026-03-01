use egui::{Color32, Pos2, Rect, RichText, Stroke, Vec2};

use crate::annotation::ToolKind;
use crate::theme;

#[derive(Clone, Copy, PartialEq)]
pub enum ToolbarAction {
    SelectTool(ToolKind),
    Copy,
    Save,
    Cancel,
    Pin,
    Undo,
    Redo,
}

pub struct Toolbar;

impl Toolbar {
    fn draw_separator(ui: &mut egui::Ui) {
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(
                theme::SEPARATOR_WIDTH + 6.0,
                theme::SEPARATOR_HEIGHT,
            ),
            egui::Sense::hover(),
        );
        let center_x = rect.center().x;
        let top_y =
            rect.center().y - theme::SEPARATOR_HEIGHT / 2.0;
        let bot_y =
            rect.center().y + theme::SEPARATOR_HEIGHT / 2.0;
        ui.painter().line_segment(
            [
                Pos2::new(center_x, top_y),
                Pos2::new(center_x, bot_y),
            ],
            Stroke::new(
                theme::SEPARATOR_WIDTH,
                theme::SEPARATOR_COLOR,
            ),
        );
    }

    fn tool_button(
        ui: &mut egui::Ui,
        icon: &str,
        active: bool,
    ) -> bool {
        let text_color = if active {
            theme::TOOL_ACTIVE_TEXT
        } else {
            theme::TOOL_INACTIVE_TEXT
        };
        let fill = if active {
            theme::TOOL_ACTIVE_BG
        } else {
            Color32::TRANSPARENT
        };
        let rt = RichText::new(icon)
            .size(theme::TOOL_ICON_SIZE)
            .color(text_color);
        let btn = egui::Button::new(rt)
            .fill(fill)
            .corner_radius(theme::TOOL_BTN_CORNER_RADIUS)
            .min_size(Vec2::splat(
                theme::TOOL_BTN_MIN_SIZE,
            ));
        ui.add(btn).clicked()
    }

    pub fn show(
        ctx: &egui::Context,
        selection_rect: Rect,
        active_tool: ToolKind,
        _tool_color: Color32,
        _stroke_width: f32,
    ) -> Option<ToolbarAction> {
        let gap = 8.0;
        let center_x = selection_rect.center().x;
        let top_y = selection_rect.max.y + gap;

        let mut action = None;

        egui::Area::new(egui::Id::new(
            "overlay_toolbar",
        ))
        .fixed_pos(Pos2::new(center_x - 280.0, top_y))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(theme::TOOLBAR_BG)
                .corner_radius(
                    theme::TOOLBAR_CORNER_RADIUS,
                )
                .inner_margin(theme::TOOLBAR_INNER_MARGIN)
                .shadow(egui::Shadow {
                    blur: theme::TOOLBAR_GLOW_BLUR,
                    color: theme::TOOLBAR_GLOW_COLOR,
                    ..Default::default()
                })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x =
                            2.0;

                        let tools = [
                            (ToolKind::Select, "▭"),
                            (ToolKind::Rect, "□"),
                            (ToolKind::Ellipse, "○"),
                            (ToolKind::Arrow, "→"),
                            (ToolKind::Pen, "✎"),
                            (ToolKind::Text, "T"),
                            (ToolKind::Blur, "▓"),
                            (ToolKind::Eraser, "⌫"),
                        ];

                        for (kind, icon) in tools {
                            let is_active =
                                active_tool == kind;
                            if Self::tool_button(
                                ui, icon, is_active,
                            ) {
                                action = Some(
                                    ToolbarAction::SelectTool(
                                        kind,
                                    ),
                                );
                            }
                        }

                        Self::draw_separator(ui);

                        // Undo / Redo
                        let undo_rt = RichText::new("↶")
                            .size(theme::TOOL_ICON_SIZE)
                            .color(
                                theme::TOOL_INACTIVE_TEXT,
                            );
                        let undo_btn =
                            egui::Button::new(undo_rt)
                                .fill(Color32::TRANSPARENT)
                                .corner_radius(
                                    theme::TOOL_BTN_CORNER_RADIUS,
                                )
                                .min_size(Vec2::splat(
                                    theme::TOOL_BTN_MIN_SIZE,
                                ));
                        if ui
                            .add(undo_btn)
                            .on_hover_text("Undo")
                            .clicked()
                        {
                            action = Some(
                                ToolbarAction::Undo,
                            );
                        }

                        let redo_rt = RichText::new("↷")
                            .size(theme::TOOL_ICON_SIZE)
                            .color(
                                theme::TOOL_INACTIVE_TEXT,
                            );
                        let redo_btn =
                            egui::Button::new(redo_rt)
                                .fill(Color32::TRANSPARENT)
                                .corner_radius(
                                    theme::TOOL_BTN_CORNER_RADIUS,
                                )
                                .min_size(Vec2::splat(
                                    theme::TOOL_BTN_MIN_SIZE,
                                ));
                        if ui
                            .add(redo_btn)
                            .on_hover_text("Redo")
                            .clicked()
                        {
                            action = Some(
                                ToolbarAction::Redo,
                            );
                        }

                        Self::draw_separator(ui);

                        // Pin
                        let pin_rt = RichText::new("📌")
                            .size(theme::TOOL_ICON_SIZE)
                            .color(
                                theme::TOOL_INACTIVE_TEXT,
                            );
                        let pin_btn =
                            egui::Button::new(pin_rt)
                                .fill(Color32::TRANSPARENT)
                                .corner_radius(
                                    theme::TOOL_BTN_CORNER_RADIUS,
                                )
                                .min_size(Vec2::splat(
                                    theme::TOOL_BTN_MIN_SIZE,
                                ));
                        if ui
                            .add(pin_btn)
                            .on_hover_text("Pin")
                            .clicked()
                        {
                            action =
                                Some(ToolbarAction::Pin);
                        }

                        // Save
                        let save_rt = RichText::new("💾")
                            .size(theme::TOOL_ICON_SIZE)
                            .color(
                                theme::TOOL_INACTIVE_TEXT,
                            );
                        let save_btn =
                            egui::Button::new(save_rt)
                                .fill(Color32::TRANSPARENT)
                                .corner_radius(
                                    theme::TOOL_BTN_CORNER_RADIUS,
                                )
                                .min_size(Vec2::splat(
                                    theme::TOOL_BTN_MIN_SIZE,
                                ));
                        if ui
                            .add(save_btn)
                            .on_hover_text("Save")
                            .clicked()
                        {
                            action = Some(
                                ToolbarAction::Save,
                            );
                        }

                        // Cancel (red)
                        let cancel_rt =
                            RichText::new("✕")
                                .size(
                                    theme::TOOL_ICON_SIZE,
                                )
                                .color(
                                    theme::CANCEL_TEXT_COLOR,
                                );
                        let cancel_btn =
                            egui::Button::new(cancel_rt)
                                .fill(Color32::TRANSPARENT)
                                .corner_radius(
                                    theme::TOOL_BTN_CORNER_RADIUS,
                                )
                                .min_size(Vec2::splat(
                                    theme::TOOL_BTN_MIN_SIZE,
                                ));
                        if ui
                            .add(cancel_btn)
                            .on_hover_text("Cancel")
                            .clicked()
                        {
                            action = Some(
                                ToolbarAction::Cancel,
                            );
                        }

                        // Copy (blue)
                        let copy_rt =
                            RichText::new("📋")
                                .size(
                                    theme::TOOL_ICON_SIZE,
                                )
                                .color(
                                    theme::COPY_TEXT_COLOR,
                                );
                        let copy_btn =
                            egui::Button::new(copy_rt)
                                .fill(Color32::TRANSPARENT)
                                .corner_radius(
                                    theme::TOOL_BTN_CORNER_RADIUS,
                                )
                                .min_size(Vec2::splat(
                                    theme::TOOL_BTN_MIN_SIZE,
                                ));
                        if ui
                            .add(copy_btn)
                            .on_hover_text("Copy")
                            .clicked()
                        {
                            action = Some(
                                ToolbarAction::Copy,
                            );
                        }
                    });
                });
        });

        action
    }
}
