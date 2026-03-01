mod annotation;
mod app;
mod app_capture;
mod capture;
mod clipboard;
mod hotkeys;
mod overlay;
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

    let bench_mode = std::env::args()
        .any(|a| a == "--bench");

    if bench_mode {
        #[cfg(windows)]
        {
            bench::run_bench();
        }
        return Ok(());
    }

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

#[cfg(windows)]
mod bench {
    use std::time::Instant;

    use image::RgbaImage;
    use windows_sys::Win32::UI::{
        Input::KeyboardAndMouse::{
            RegisterHotKey, MOD_SHIFT,
        },
        WindowsAndMessaging::{
            GetMessageW, MSG, WM_HOTKEY,
        },
    };

    use crate::capture::DxgiCapture;

    const BENCH_HOTKEY_ID: i32 = 1;
    const VK_F2: u32 = 0x71;

    pub fn run_bench() {
        log::info!("Bench mode: direct capture");

        let dxgi = match DxgiCapture::new() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("DXGI init failed: {e}");
                return;
            }
        };

        // Warm up: do one throwaway capture
        let _ = dxgi.acquire_frame();
        let _ = dxgi.read_pixels();

        let ok = unsafe {
            RegisterHotKey(
                std::ptr::null_mut(),
                BENCH_HOTKEY_ID,
                MOD_SHIFT,
                VK_F2,
            )
        };
        if ok == 0 {
            eprintln!("RegisterHotKey failed");
            return;
        }
        log::info!("Waiting for Shift+F2...");

        // Block until hotkey
        let mut msg: MSG = unsafe {
            std::mem::zeroed()
        };
        loop {
            let ret = unsafe {
                GetMessageW(
                    &mut msg,
                    std::ptr::null_mut(),
                    0,
                    0,
                )
            };
            if ret <= 0 {
                break;
            }
            if msg.message == WM_HOTKEY {
                let start = Instant::now();
                match dxgi.acquire_frame() {
                    Ok(()) => {
                        let ms = start
                            .elapsed()
                            .as_secs_f64()
                            * 1000.0;
                        let result =
                            format!("{ms:.2}");
                        let _ = std::fs::write(
                            "snapvault_bench_result\
                             .txt",
                            &result,
                        );
                        log::info!(
                            "Bench: {result}ms"
                        );
                        if let Ok((
                            mut buf, w, h,
                        )) = dxgi.read_pixels()
                        {
                            DxgiCapture::bgra_to_rgba(
                                &mut buf,
                            );
                            if let Some(img) =
                                RgbaImage::from_raw(
                                    w, h, buf,
                                )
                            {
                                let _ = img.save(
                                    "snapvault_bench\
                                     .png",
                                );
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Acquire failed: {e}"
                        );
                    }
                }
                break;
            }
        }
    }
}
