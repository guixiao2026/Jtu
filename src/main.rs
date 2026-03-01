mod annotation;
mod app;
mod app_capture;
mod capture;
mod clipboard;
mod hotkeys;
mod pin;
mod theme;
mod tools;
mod tray;
#[cfg(windows)]
mod win_overlay;
#[cfg(windows)]
mod win_utils;

use eframe::egui;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([400.0, 300.0])
        .with_min_inner_size([300.0, 200.0]);
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "SnapVault",
        options,
        Box::new(move |cc| {
            Ok(Box::new(
                app::SnapVaultApp::new(cc, false),
            ))
        }),
    )
}
