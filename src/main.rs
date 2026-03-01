#![windows_subsystem = "windows"]

mod capture;
mod clipboard;
mod hotkeys;
#[cfg(windows)]
mod settings;
mod tray;
#[cfg(windows)]
mod win_overlay;

use std::time::Instant;

use hotkeys::{HotkeyEvent, HotkeyManager};
use image::RgbaImage;
use tray::{AppTray, TrayEvent};

#[cfg(windows)]
fn pump_messages() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage,
        MSG, PM_REMOVE,
    };
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while PeekMessageW(
            &mut msg,
            std::ptr::null_mut(),
            0,
            0,
            PM_REMOVE,
        ) != 0
        {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

fn main() {
    env_logger::init();

    let _tray = match AppTray::new() {
        Ok(t) => t,
        Err(e) => {
            log::error!("Tray init: {e}");
            return;
        }
    };

    let mut hotkey_manager =
        match HotkeyManager::new() {
            Ok(h) => h,
            Err(e) => {
                log::error!("Hotkey init: {e}");
                return;
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

    log::info!("Jtu started (tray mode)");

    loop {
        // Pump Win32 messages first — hotkey and
        // tray events only appear in channels after
        // DispatchMessageW processes them.
        #[cfg(windows)]
        pump_messages();

        match _tray.poll_event() {
            TrayEvent::Capture => {
                do_capture(
                    &dxgi,
                );
            }
            TrayEvent::Settings => {
                #[cfg(windows)]
                if settings::show_settings(
                    &mut hotkey_manager,
                    || do_capture(&dxgi),
                    _tray.quit_id(),
                    _tray.capture_id(),
                ) {
                    break;
                }
            }
            TrayEvent::Quit => {
                break;
            }
            TrayEvent::None => {}
        }

        // Pump hotkey events
        match hotkey_manager.poll_event() {
            HotkeyEvent::Capture => {
                log::info!("Hotkey capture!");
                do_capture(
                    &dxgi,
                );
            }
            HotkeyEvent::None => {}
        }

        std::thread::sleep(
            std::time::Duration::from_millis(8),
        );
    }
}

fn do_capture(
    dxgi: &Option<capture::DxgiCapture>,
) {
    let start = Instant::now();

    // DXGI fast path
    #[cfg(windows)]
    if let Some(ref dxgi) = dxgi {
        match dxgi.acquire_frame() {
            Ok(()) => {
                let (w, h) = dxgi.screen_size();
                log::info!(
                    "DXGI acquired {}x{} {:.1}ms",
                    w,
                    h,
                    start.elapsed().as_secs_f64()
                        * 1000.0
                );
                run_overlay_dxgi(dxgi, w, h);
                return;
            }
            Err(e) => {
                log::error!("DXGI acquire: {e}");
            }
        }
    }

    // xcap fallback
    match capture::capture_primary_monitor() {
        Ok(img) => {
            log::info!(
                "xcap captured {}x{} {:.1}ms",
                img.width(),
                img.height(),
                start.elapsed().as_secs_f64()
                    * 1000.0
            );
            run_overlay_xcap(&img);
        }
        Err(e) => {
            log::error!("Capture failed: {e}");
        }
    }
}

fn run_overlay_xcap(img: &RgbaImage) {
    #[cfg(windows)]
    {
        let mut bgra = img.as_raw().to_vec();
        capture::swap_rb_inplace(&mut bgra);
        let w = img.width();
        let h = img.height();

        log::info!(
            "enter Win32 overlay (xcap) {}x{}",
            w, h
        );
        let result =
            win_overlay::run_overlay(&bgra, w, h);

        handle_result(&result, |x, y, rw, rh| {
            capture::capture_region(img, x, y, rw, rh)
        });
    }
}

#[cfg(windows)]
fn run_overlay_dxgi(
    dxgi: &capture::DxgiCapture,
    w: u32,
    h: u32,
) {
    log::info!(
        "enter Win32 overlay (DXGI) {}x{}",
        w, h
    );

    let mut dxgi_ok = true;
    let result = win_overlay::run_overlay_with(
        w,
        h,
        |bits_ptr, bits_len| {
            if let Err(e) = dxgi.read_pixels_into(
                bits_ptr, bits_len,
            ) {
                log::error!(
                    "DXGI read_into: {e}"
                );
                dxgi_ok = false;
            }
        },
    );

    if !dxgi_ok {
        return;
    }

    handle_result(&result, |x, y, rw, rh| {
        match dxgi.read_staging() {
            Ok((buf, full_w, _)) => {
                capture::crop_bgra_to_rgba(
                    &buf, full_w, x, y, rw, rh,
                )
            }
            Err(e) => {
                log::error!("Read staging: {e}");
                RgbaImage::new(1, 1)
            }
        }
    });

    log::info!("exit Win32 overlay");
}

fn handle_result<F>(
    result: &win_overlay::OverlayResult,
    get_region: F,
) where
    F: FnOnce(u32, u32, u32, u32) -> RgbaImage,
{
    match result.action {
        win_overlay::OverlayAction::Copy => {
            if let Some((x, y, rw, rh)) =
                result.selection
            {
                let region =
                    get_region(x, y, rw, rh);
                if let Err(e) =
                    clipboard::copy_rgba_image(
                        &region,
                    )
                {
                    log::error!(
                        "Copy failed: {e}"
                    );
                }
            }
        }
        win_overlay::OverlayAction::Save => {
            if let Some((x, y, rw, rh)) =
                result.selection
            {
                let region =
                    get_region(x, y, rw, rh);
                if let Some(path) =
                    rfd::FileDialog::new()
                        .add_filter(
                            "PNG",
                            &["png"],
                        )
                        .set_file_name(
                            "screenshot.png",
                        )
                        .save_file()
                {
                    if let Err(e) =
                        region.save(&path)
                    {
                        log::error!(
                            "Save: {e}"
                        );
                    }
                }
            }
        }
        win_overlay::OverlayAction::Cancel => {}
    }
}
