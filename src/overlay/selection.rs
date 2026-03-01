use egui::{Pos2, Rect, Vec2};

#[derive(Clone, Copy, PartialEq)]
pub enum DragKind {
    NewSelection,
    MoveSelection,
    ResizeTopLeft,
    ResizeTopRight,
    ResizeBottomLeft,
    ResizeBottomRight,
    ResizeTop,
    ResizeBottom,
    ResizeLeft,
    ResizeRight,
}

#[derive(Clone)]
pub struct Selection {
    pub rect: Option<Rect>,
    pub drag: Option<DragState>,
    pub confirmed: bool,
}

#[derive(Clone)]
pub struct DragState {
    pub kind: DragKind,
    pub start: Pos2,
    pub original_rect: Option<Rect>,
}

const HANDLE_RADIUS: f32 = 5.0;

impl Selection {
    pub fn new() -> Self {
        Self {
            rect: None,
            drag: None,
            confirmed: false,
        }
    }

    pub fn handle_rects(&self) -> Vec<(Rect, DragKind)> {
        let Some(r) = self.rect else {
            return Vec::new();
        };
        let s = HANDLE_RADIUS;
        vec![
            (handle_at(r.left_top(), s), DragKind::ResizeTopLeft),
            (handle_at(r.right_top(), s), DragKind::ResizeTopRight),
            (handle_at(r.left_bottom(), s), DragKind::ResizeBottomLeft),
            (
                handle_at(r.right_bottom(), s),
                DragKind::ResizeBottomRight,
            ),
            (handle_at(r.center_top(), s), DragKind::ResizeTop),
            (handle_at(r.center_bottom(), s), DragKind::ResizeBottom),
            (handle_at(r.left_center(), s), DragKind::ResizeLeft),
            (handle_at(r.right_center(), s), DragKind::ResizeRight),
        ]
    }

    pub fn hit_test(&self, pos: Pos2) -> Option<DragKind> {
        for (handle_rect, kind) in self.handle_rects() {
            if handle_rect.contains(pos) {
                return Some(kind);
            }
        }
        if let Some(r) = self.rect {
            if r.contains(pos) {
                return Some(DragKind::MoveSelection);
            }
        }
        None
    }

    pub fn begin_drag(&mut self, pos: Pos2, kind: DragKind) {
        self.drag = Some(DragState {
            kind,
            start: pos,
            original_rect: self.rect,
        });
    }

    pub fn update_drag(&mut self, current: Pos2) {
        let Some(ref drag) = self.drag else { return };
        match drag.kind {
            DragKind::NewSelection => {
                let r = Rect::from_two_pos(drag.start, current);
                if r.width() > 2.0 && r.height() > 2.0 {
                    self.rect = Some(r);
                }
            }
            DragKind::MoveSelection => {
                if let Some(orig) = drag.original_rect {
                    let delta = current - drag.start;
                    self.rect =
                        Some(orig.translate(delta));
                }
            }
            _ => {
                if let Some(orig) = drag.original_rect {
                    self.rect =
                        Some(resize(orig, drag.kind, current));
                }
            }
        }
    }

    pub fn end_drag(&mut self) {
        if self.drag.is_some() {
            if self.rect.is_some() {
                self.confirmed = true;
            }
            self.drag = None;
        }
    }

    pub fn clear(&mut self) {
        self.rect = None;
        self.drag = None;
        self.confirmed = false;
    }

    pub fn dimensions(&self) -> Option<(u32, u32)> {
        self.rect.map(|r| {
            (r.width() as u32, r.height() as u32)
        })
    }
}

fn handle_at(center: Pos2, radius: f32) -> Rect {
    Rect::from_center_size(
        center,
        Vec2::splat(radius * 2.0),
    )
}

fn resize(
    orig: Rect,
    kind: DragKind,
    pos: Pos2,
) -> Rect {
    let (mut min, mut max) = (orig.min, orig.max);
    match kind {
        DragKind::ResizeTopLeft => {
            min = pos;
        }
        DragKind::ResizeTopRight => {
            min.y = pos.y;
            max.x = pos.x;
        }
        DragKind::ResizeBottomLeft => {
            min.x = pos.x;
            max.y = pos.y;
        }
        DragKind::ResizeBottomRight => {
            max = pos;
        }
        DragKind::ResizeTop => {
            min.y = pos.y;
        }
        DragKind::ResizeBottom => {
            max.y = pos.y;
        }
        DragKind::ResizeLeft => {
            min.x = pos.x;
        }
        DragKind::ResizeRight => {
            max.x = pos.x;
        }
        _ => {}
    }
    Rect::from_min_max(
        Pos2::new(min.x.min(max.x), min.y.min(max.y)),
        Pos2::new(min.x.max(max.x), min.y.max(max.y)),
    )
}
