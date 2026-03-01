use egui::{Color32, Pos2};

#[derive(Clone, Copy, PartialEq)]
pub enum ToolKind {
    Select,
    Rect,
    Ellipse,
    Arrow,
    Pen,
    Text,
    Blur,
    Eraser,
}

#[derive(Clone)]
pub struct Annotation {
    pub kind: ToolKind,
    pub color: Color32,
    pub stroke_width: f32,
    pub points: Vec<Pos2>,
    pub text: String,
}

pub struct AnnotationHistory {
    annotations: Vec<Annotation>,
    undo_stack: Vec<Annotation>,
}

impl AnnotationHistory {
    pub fn new() -> Self {
        Self {
            annotations: Vec::new(),
            undo_stack: Vec::new(),
        }
    }

    pub fn push(&mut self, ann: Annotation) {
        self.annotations.push(ann);
        self.undo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(ann) = self.annotations.pop() {
            self.undo_stack.push(ann);
        }
    }

    pub fn redo(&mut self) {
        if let Some(ann) = self.undo_stack.pop() {
            self.annotations.push(ann);
        }
    }

    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    pub fn remove_at_pos(
        &mut self,
        pos: Pos2,
        threshold: f32,
    ) -> bool {
        let idx = self.annotations.iter().rposition(|ann| {
            ann.points.iter().any(|p| {
                p.distance(pos) < threshold
            })
        });
        if let Some(i) = idx {
            let removed = self.annotations.remove(i);
            self.undo_stack.push(removed);
            true
        } else {
            false
        }
    }
}
