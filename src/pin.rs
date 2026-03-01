use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use egui::{
    Color32, ColorImage, Pos2, Rect, TextureHandle,
    TextureOptions, ViewportBuilder, ViewportId,
};
use image::RgbaImage;

pub struct PinnedWindow {
    id: ViewportId,
    image: Arc<RgbaImage>,
    title: String,
    closed: Arc<AtomicBool>,
    id_num: u32,
}

static NEXT_PIN_ID: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(1);

impl PinnedWindow {
    pub fn new(image: RgbaImage) -> Self {
        let id_num = NEXT_PIN_ID
            .fetch_add(
                1,
                std::sync::atomic::Ordering::Relaxed,
            );
        let title = format!("Pin #{id_num}");
        Self {
            id: ViewportId::from_hash_of(
                format!("pin_{id_num}"),
            ),
            image: Arc::new(image),
            title,
            closed: Arc::new(AtomicBool::new(false)),
            id_num,
        }
    }

    pub fn close(&self) {
        self.closed.store(true, Ordering::Relaxed);
    }

    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Relaxed)
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
    ) -> bool {
        if self.closed.load(Ordering::Relaxed) {
            return false;
        }

        let (w, h) = (
            self.image.width() as f32,
            self.image.height() as f32,
        );
        let scale = if w > 800.0 || h > 600.0 {
            (800.0 / w).min(600.0 / h)
        } else {
            1.0
        };
        let display_w = w * scale;
        let display_h = h * scale;

        let builder = ViewportBuilder::default()
            .with_title(&self.title)
            .with_inner_size([display_w, display_h])
            .with_always_on_top()
            .with_decorations(true);

        let image_data = Arc::clone(&self.image);
        let closed_flag = self.closed.clone();
        let pin_id_num = self.id_num;

        ctx.show_viewport_deferred(
            self.id,
            builder,
            move |ctx, _class| {
                let tex_id = egui::Id::new((
                    "pin_texture",
                    pin_id_num,
                ));
                let texture: TextureHandle = ctx
                    .memory_mut(|mem| {
                        mem.data
                            .get_temp::<TextureHandle>(
                                tex_id,
                            )
                    })
                    .unwrap_or_else(|| {
                        let color_image =
                            ColorImage::from_rgba_unmultiplied(
                                [
                                    image_data.width()
                                        as usize,
                                    image_data.height()
                                        as usize,
                                ],
                                image_data.as_raw(),
                            );
                        let tex = ctx.load_texture(
                            "pinned",
                            color_image,
                            TextureOptions::LINEAR,
                        );
                        ctx.memory_mut(|mem| {
                            mem.data.insert_temp(
                                tex_id,
                                tex.clone(),
                            );
                        });
                        tex
                    });

                egui::CentralPanel::default().show(
                    ctx,
                    |ui| {
                        let rect = ui
                            .available_rect_before_wrap();
                        ui.painter().image(
                            texture.id(),
                            rect,
                            Rect::from_min_max(
                                Pos2::ZERO,
                                Pos2::new(1.0, 1.0),
                            ),
                            Color32::WHITE,
                        );
                    },
                );

                if ctx.input(|i| {
                    i.key_pressed(egui::Key::Escape)
                        || i.viewport()
                            .close_requested()
                }) {
                    closed_flag
                        .store(true, Ordering::Relaxed);
                    ctx.send_viewport_cmd(
                        egui::ViewportCommand::Close,
                    );
                }
            },
        );

        true
    }
}
