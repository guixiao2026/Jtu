use std::sync::mpsc;
use std::time::Instant;

use eframe::egui::{self, Color32};
use image::RgbaImage;

use crate::capture;
use crate::hotkeys::{HotkeyEvent, HotkeyManager};
use crate::pin::PinnedWindow;
use crate::theme;
use crate::tray::{AppTray, TrayEvent};

pub enum AppState {
    Main,
    WaitingHide,
    Capturing,
}

pub struct SnapVaultApp {
    pub(crate) state: AppState,
    pub(crate) capture_requested: bool,
    tray: Option<AppTray>,
    hotkey_manager: Option<HotkeyManager>,
    pinned_windows: Vec<PinnedWindow>,
    pub(crate) capture_rx:
        Option<mpsc::Receiver<Result<RgbaImage, String>>>,
    pub(crate) capture_start: Option<Instant>,
    pub(crate) bench_mode: bool,
    #[cfg(windows)]
    pub(crate) dxgi: Option<capture::DxgiCapture>,
}

impl SnapVaultApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        bench_mode: bool,
    ) -> Self {
        cc.egui_ctx.set_visuals(
            egui::Visuals::dark(),
        );
        let tray = match AppTray::new() {
            Ok(t) => Some(t),
            Err(e) => {
                log::error!("Tray init: {e}");
                None
            }
        };

        let hotkey_manager =
            match HotkeyManager::new() {
                Ok(h) => Some(h),
                Err(e) => {
                    log::error!("Hotkey init: {e}");
                    None
                }
            };

        #[cfg(windows)]
        let dxgi = match capture::DxgiCapture::new() {
            Ok(d) => Some(d),
            Err(e) => {
                log::error!("DXGI init: {e}");
                None
            }
        };

        Self {
            state: AppState::Main,
            capture_requested: false,
            tray,
            hotkey_manager,
            pinned_windows: Vec::new(),
            capture_rx: None,
            capture_start: None,
            bench_mode,
            #[cfg(windows)]
            dxgi,
        }
    }

    fn poll_events(
        &mut self,
        ctx: &egui::Context,
    ) {
        if let Some(ref tray) = self.tray {
            match tray.poll_event() {
                TrayEvent::Capture => {
                    self.capture_requested = true;
                }
                TrayEvent::Quit => {
                    ctx.send_viewport_cmd(
                        egui::ViewportCommand::Close,
                    );
                }
                _ => {}
            }
        }

        if let Some(ref hk) = self.hotkey_manager {
            match hk.poll_event() {
                HotkeyEvent::Capture => {
                    log::info!(
                        "Hotkey capture triggered!"
                    );
                    if matches!(
                        self.state,
                        AppState::Main
                    ) {
                        self.capture_requested = true;
                    }
                }
                HotkeyEvent::None => {}
            }
        }
    }

    fn show_main_ui(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        let dark_frame = egui::Frame::new()
            .fill(Color32::from_rgb(30, 30, 30));
        egui::CentralPanel::default()
            .frame(dark_frame)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(
                        egui::RichText::new("SnapVault")
                            .size(theme::TITLE_FONT_SIZE)
                            .strong()
                            .color(Color32::WHITE),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(
                            "Screenshot & Clipboard \
                             Manager",
                        )
                        .size(theme::SUBTITLE_FONT_SIZE)
                        .color(theme::SUBTITLE_COLOR),
                    );
                    ui.add_space(24.0);
                    let btn = egui::Button::new(
                        egui::RichText::new(
                            "Screenshot",
                        )
                        .size(15.0)
                        .color(Color32::WHITE),
                    )
                    .fill(theme::SELECTION_BORDER_COLOR)
                    .min_size(egui::vec2(
                        theme::MAIN_BTN_WIDTH,
                        theme::MAIN_BTN_HEIGHT,
                    ))
                    .corner_radius(
                        theme::MAIN_BTN_CORNER_RADIUS,
                    );
                    if ui.add(btn).clicked() {
                        self.capture_requested = true;
                    }
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(
                            "Shift+F2",
                        )
                        .size(12.0)
                        .color(
                            theme::SHORTCUT_HINT_COLOR,
                        ),
                    );
                });
            });
    }
}

impl eframe::App for SnapVaultApp {
    fn update(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
    ) {
        self.poll_events(ctx);

        ctx.request_repaint_after(
            std::time::Duration::from_millis(16),
        );

        if ctx.input(|i| {
            i.key_pressed(egui::Key::Escape)
        }) {
            if let Some(pin) =
                self.pinned_windows.last()
            {
                pin.close();
            }
        }

        if self.capture_requested {
            self.capture_requested = false;
            self.start_capture(ctx);
            return;
        }

        match &self.state {
            AppState::Main => {
                self.show_main_ui(ctx, frame);
            }
            AppState::WaitingHide => {
                self.spawn_capture(ctx);
            }
            AppState::Capturing => {
                self.poll_capture(ctx);
            }
        }

        self.pinned_windows.retain_mut(|pin| {
            pin.show(ctx)
        });
    }
}
