pub mod selection;
pub mod toolbar;
mod drawing;
mod input;
mod widgets;

use std::time::Instant;

use egui::{
    Color32, ColorImage, Pos2, Rect,
    TextureHandle, TextureOptions,
};
use image::RgbaImage;

use crate::annotation::{
    AnnotationHistory, ToolKind,
};
use crate::theme;

use self::selection::Selection;
use self::toolbar::{Toolbar, ToolbarAction};

pub enum OverlayResult {
    None,
    Cancel,
    Copy,
    Save,
    Pin,
}

pub struct OverlayState {
    pub(self) screenshot: RgbaImage,
    texture: Option<TextureHandle>,
    pub selection: Selection,
    pub(self) screen_size: (u32, u32),
    pub(self) active_tool: ToolKind,
    pub(self) tool_color: Color32,
    pub(self) stroke_width: f32,
    pub(self) annotations: AnnotationHistory,
    pub(self) current_points: Vec<Pos2>,
    pub(self) is_drawing: bool,
    pub(self) text_input: String,
    pub(self) text_input_pos: Option<Pos2>,
    editing: bool,
    hotkey_time: Option<Instant>,
    cursor_pos: Option<Pos2>,
}

impl OverlayState {
    pub fn new(
        screenshot: RgbaImage,
        hotkey_time: Option<Instant>,
    ) -> Self {
        let w = screenshot.width();
        let h = screenshot.height();
        Self {
            screenshot,
            texture: None,
            selection: Selection::new(),
            screen_size: (w, h),
            active_tool: ToolKind::Select,
            tool_color: Color32::RED,
            stroke_width: 3.0,
            annotations: AnnotationHistory::new(),
            current_points: Vec::new(),
            is_drawing: false,
            text_input: String::new(),
            text_input_pos: None,
            editing: false,
            hotkey_time,
            cursor_pos: None,
        }
    }

    pub fn screenshot(&self) -> &RgbaImage {
        &self.screenshot
    }

    pub fn screen_width(&self) -> u32 {
        self.screen_size.0
    }

    pub fn screen_height(&self) -> u32 {
        self.screen_size.1
    }

    pub fn annotations(&self) -> &AnnotationHistory {
        &self.annotations
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
    ) -> OverlayResult {
        let tex = self.ensure_texture(ctx);
        let draw_rect = ctx.screen_rect();
        let mut result = OverlayResult::None;
        let tex_id = tex.id();

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let painter =
                    ui.painter_at(draw_rect);

                painter.image(
                    tex_id,
                    draw_rect,
                    Rect::from_min_max(
                        Pos2::ZERO,
                        Pos2::new(1.0, 1.0),
                    ),
                    Color32::WHITE,
                );

                self.draw_mask(
                    &painter, draw_rect,
                );
                self.draw_selection(&painter);

                self.draw_annotations(
                    &painter,
                    draw_rect,
                );

                self.cursor_pos =
                    ui.input(|i| i.pointer.hover_pos());

                if !self.editing {
                    self.handle_input(
                        ui, draw_rect,
                    );
                    self.draw_magnifier(
                        &painter, draw_rect,
                    );
                } else {
                    self.handle_tool_input(
                        ui, draw_rect,
                    );
                }
            });

        if self.selection.confirmed {
            if !self.editing {
                self.editing = true;
            }
            if let Some(sel_rect) = self.selection.rect {
                if let Some(action) = Toolbar::show(
                    ctx,
                    sel_rect,
                    self.active_tool,
                    self.tool_color,
                    self.stroke_width,
                ) {
                    match action {
                        ToolbarAction::Cancel => {
                            result = OverlayResult::Cancel;
                        }
                        ToolbarAction::Copy => {
                            result = OverlayResult::Copy;
                        }
                        ToolbarAction::Save => {
                            result = OverlayResult::Save;
                        }
                        ToolbarAction::Pin => {
                            result = OverlayResult::Pin;
                        }
                        ToolbarAction::Undo => {
                            self.annotations.undo();
                        }
                        ToolbarAction::Redo => {
                            self.annotations.redo();
                        }
                        ToolbarAction::SelectTool(
                            kind,
                        ) => {
                            self.active_tool = kind;
                        }
                    }
                }

                self.show_color_picker(ctx, sel_rect);
            }
        }

        if self.text_input_pos.is_some() {
            self.show_text_input(ctx);
        }

        if let Some(t) = self.hotkey_time {
            let ms = t.elapsed().as_millis();
            let screen = ctx.screen_rect();
            egui::Area::new(egui::Id::new(
                "latency_timer",
            ))
            .fixed_pos(Pos2::new(
                screen.max.x
                    - theme::LATENCY_TIMER_OFFSET_X,
                theme::LATENCY_TIMER_OFFSET_Y,
            ))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "{}ms", ms
                    ))
                    .size(20.0)
                    .color(Color32::from_rgb(
                        0, 255, 100,
                    ))
                    .strong(),
                );
            });
        }

        result
    }

    fn ensure_texture(
        &mut self,
        ctx: &egui::Context,
    ) -> TextureHandle {
        if let Some(ref tex) = self.texture {
            return tex.clone();
        }
        log::info!(
            "[DIAG] ensure_texture: uploading GPU \
             texture {}x{}",
            self.screen_size.0,
            self.screen_size.1,
        );
        let (w, h) = self.screen_size;
        let color_image =
            ColorImage::from_rgba_unmultiplied(
                [w as usize, h as usize],
                self.screenshot.as_raw(),
            );
        let tex = ctx.load_texture(
            "screenshot",
            color_image,
            TextureOptions::LINEAR,
        );
        log::info!(
            "[DIAG] ensure_texture: upload done"
        );
        self.texture = Some(tex.clone());
        tex
    }
}
