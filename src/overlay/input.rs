use egui::{CursorIcon, Rect};

use crate::annotation::{Annotation, ToolKind};

use super::OverlayState;
use super::selection::DragKind;

impl OverlayState {
    pub(super) fn handle_input(
        &mut self,
        ui: &mut egui::Ui,
        draw_rect: Rect,
    ) {
        let response = ui.allocate_rect(
            draw_rect,
            egui::Sense::click_and_drag(),
        );

        if let Some(pos) =
            response.interact_pointer_pos()
        {
            if response.drag_started() {
                let kind =
                    if self.selection.confirmed {
                        self.selection
                            .hit_test(pos)
                            .unwrap_or(
                                DragKind::NewSelection,
                            )
                    } else {
                        DragKind::NewSelection
                    };
                if kind == DragKind::NewSelection {
                    self.selection.confirmed = false;
                }
                self.selection.begin_drag(pos, kind);
            }

            if response.dragged() {
                self.selection.update_drag(pos);
            }
        }

        if response.drag_stopped() {
            self.selection.end_drag();
        }

        if let Some(pos) = response.hover_pos() {
            if self.selection.confirmed {
                let cursor = match self
                    .selection
                    .hit_test(pos)
                {
                    Some(DragKind::MoveSelection) => {
                        CursorIcon::Grabbing
                    }
                    Some(
                        DragKind::ResizeTopLeft
                        | DragKind::ResizeBottomRight,
                    ) => CursorIcon::ResizeNwSe,
                    Some(
                        DragKind::ResizeTopRight
                        | DragKind::ResizeBottomLeft,
                    ) => CursorIcon::ResizeNeSw,
                    Some(
                        DragKind::ResizeTop
                        | DragKind::ResizeBottom,
                    ) => CursorIcon::ResizeVertical,
                    Some(
                        DragKind::ResizeLeft
                        | DragKind::ResizeRight,
                    ) => CursorIcon::ResizeHorizontal,
                    _ => CursorIcon::Crosshair,
                };
                ui.ctx().set_cursor_icon(cursor);
            } else {
                ui.ctx().set_cursor_icon(
                    CursorIcon::Crosshair,
                );
            }
        }
    }

    pub(super) fn handle_tool_input(
        &mut self,
        ui: &mut egui::Ui,
        draw_rect: Rect,
    ) {
        if self.active_tool == ToolKind::Select {
            self.handle_input(ui, draw_rect);
            return;
        }

        let sel_rect = match self.selection.rect {
            Some(r) => r,
            None => return,
        };

        let response = ui.allocate_rect(
            sel_rect,
            egui::Sense::click_and_drag(),
        );

        if self.active_tool == ToolKind::Text {
            if response.clicked() {
                if let Some(pos) =
                    response.interact_pointer_pos()
                {
                    self.text_input_pos = Some(pos);
                }
            }
            return;
        }

        if self.active_tool == ToolKind::Eraser {
            if response.clicked() {
                if let Some(pos) =
                    response.interact_pointer_pos()
                {
                    self.annotations.remove_at_pos(
                        pos,
                        self.stroke_width * 3.0,
                    );
                }
            }
            ui.ctx()
                .set_cursor_icon(CursorIcon::NotAllowed);
            return;
        }

        if let Some(pos) =
            response.interact_pointer_pos()
        {
            if response.drag_started() {
                self.is_drawing = true;
                self.current_points = vec![pos];
            }

            if response.dragged() && self.is_drawing {
                match self.active_tool {
                    ToolKind::Pen => {
                        self.current_points.push(pos);
                    }
                    _ => {
                        if self.current_points.len() < 2
                        {
                            self.current_points
                                .push(pos);
                        } else {
                            self.current_points[1] =
                                pos;
                        }
                    }
                }
            }
        }

        if response.drag_stopped() && self.is_drawing {
            self.is_drawing = false;
            if self.current_points.len() >= 2 {
                let ann = Annotation {
                    kind: self.active_tool,
                    color: self.tool_color,
                    stroke_width: self.stroke_width,
                    points: std::mem::take(
                        &mut self.current_points,
                    ),
                    text: String::new(),
                };
                self.annotations.push(ann);
            }
            self.current_points.clear();
        }

        match self.active_tool {
            ToolKind::Pen => {
                ui.ctx().set_cursor_icon(
                    CursorIcon::Crosshair,
                );
            }
            _ => {
                ui.ctx().set_cursor_icon(
                    CursorIcon::Crosshair,
                );
            }
        }
    }
}
