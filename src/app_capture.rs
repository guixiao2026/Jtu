use std::sync::mpsc;
use std::time::Instant;

use image::RgbaImage;

use crate::capture;
use crate::clipboard;

use crate::app::{AppState, SnapVaultApp};

/// Delay (ms) after hiding the main window before
/// capturing, so the compositor fully removes it.
const CAPTURE_HIDE_DELAY_MS: u64 = 2;

impl SnapVaultApp {
    pub(crate) fn start_capture(
        &mut self,
        ctx: &eframe::egui::Context,
    ) {
        log::info!("start_capture");
        self.capture_start = Some(Instant::now());
        if self.bench_mode {
            let dxgi = self.dxgi.as_ref().unwrap();
            if let Err(e) = dxgi.acquire_frame() {
                log::error!(
                    "Bench acquire failed: {e}"
                );
                self.dxgi = None;
                ctx.send_viewport_cmd(
                    eframe::egui::ViewportCommand::Close,
                );
                return;
            }
            let total_ms = self
                .capture_start
                .map(|s| s.elapsed())
                .unwrap_or_default()
                .as_secs_f64()
                * 1000.0;
            let result = format!("{total_ms:.2}");
            let _ = std::fs::write(
                "snapvault_bench_result.txt",
                &result,
            );
            if let Ok((mut buf, w, h)) =
                dxgi.read_pixels()
            {
                capture::DxgiCapture::bgra_to_rgba(
                    &mut buf,
                );
                if let Some(img) =
                    RgbaImage::from_raw(w, h, buf)
                {
                    let _ = img.save(
                        "snapvault_bench.png",
                    );
                }
            }
            ctx.send_viewport_cmd(
                eframe::egui::ViewportCommand::Close,
            );
            return;
        }

        // Hide main window while capturing
        #[cfg(windows)]
        crate::win_utils::hide_for_capture();

        // DXGI fast path: synchronous capture
        #[cfg(windows)]
        if self.dxgi.is_some() {
            std::thread::sleep(
                std::time::Duration::from_millis(
                    CAPTURE_HIDE_DELAY_MS,
                ),
            );
            let dxgi = self.dxgi.as_ref().unwrap();
            match dxgi.acquire_frame() {
                Ok(()) => {
                    let (w, h) =
                        dxgi.screen_size();
                    log::info!(
                        "DXGI acquired {}x{} \
                         {:.1}ms",
                        w,
                        h,
                        self.capture_start
                            .map(|s| s.elapsed())
                            .unwrap_or_default()
                            .as_secs_f64()
                            * 1000.0
                    );
                    self.run_overlay_dxgi(w, h);
                    return;
                }
                Err(e) => {
                    log::error!(
                        "DXGI acquire: {e}"
                    );
                }
            }
            // Fall through to xcap
        }

        self.state = AppState::WaitingHide;
    }

    pub(crate) fn spawn_capture(
        &mut self,
        ctx: &eframe::egui::Context,
    ) {
        self.state = AppState::Capturing;
        let (tx, rx) = mpsc::channel();
        self.capture_rx = Some(rx);

        let ctx_clone = ctx.clone();
        std::thread::spawn(move || {
            let result =
                capture::capture_primary_monitor();
            let _ = tx.send(result);
            ctx_clone.request_repaint();
        });
    }

    pub(crate) fn poll_capture(
        &mut self,
        _ctx: &eframe::egui::Context,
    ) {
        let Some(ref rx) = self.capture_rx else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(img)) => {
                log::info!(
                    "Captured {}x{}",
                    img.width(),
                    img.height()
                );
                self.capture_rx = None;
                if self.bench_mode {
                    self.bench_finish(&img);
                } else {
                    self.run_native_overlay(&img);
                }
            }
            Ok(Err(e)) => {
                log::error!("Capture failed: {e}");
                self.capture_rx = None;
                self.state = AppState::Main;
                #[cfg(windows)]
                crate::win_utils::reveal_main();
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                log::error!("Capture thread died");
                self.capture_rx = None;
                self.state = AppState::Main;
                #[cfg(windows)]
                crate::win_utils::reveal_main();
            }
        }
    }

    fn bench_finish(&self, img: &RgbaImage) {
        let elapsed = self
            .capture_start
            .map(|s| s.elapsed())
            .unwrap_or_default();
        let ms = elapsed.as_secs_f64() * 1000.0;
        let result = format!("{ms:.2}");
        log::info!("Bench capture: {result}ms");
        let _ = std::fs::write(
            "snapvault_bench_result.txt",
            &result,
        );
        let _ = img.save("snapvault_bench.png");
        std::process::exit(0);
    }

    /// Run Win32 overlay from xcap RGBA fallback.
    fn run_native_overlay(&mut self, img: &RgbaImage) {
        #[cfg(windows)]
        {
            // Convert RGBA → BGRA for new overlay API
            let mut bgra = img.as_raw().to_vec();
            capture::swap_rb_inplace(&mut bgra);
            let w = img.width();
            let h = img.height();

            // Delegate to BGRA path; Copy/Save need
            // the original RGBA image for crop.
            use crate::win_overlay;
            log::info!(
                "enter Win32 overlay (xcap) {}x{}",
                w, h
            );
            let result =
                win_overlay::run_overlay(&bgra, w, h);

            match result.action {
                win_overlay::OverlayAction::Copy => {
                    if let Some((x, y, rw, rh)) =
                        result.selection
                    {
                        let region =
                            capture::capture_region(
                                img, x, y, rw, rh,
                            );
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
                            capture::capture_region(
                                img, x, y, rw, rh,
                            );
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
            log::info!("exit Win32 overlay");
        }

        self.state = AppState::Main;
        #[cfg(windows)]
        crate::win_utils::reveal_main();
    }

    /// DXGI fast path: DXGI writes directly into
    /// DIB memory via callback — zero intermediate
    /// Vec, zero pixel conversion.
    #[cfg(windows)]
    fn run_overlay_dxgi(
        &mut self,
        w: u32,
        h: u32,
    ) {
        use crate::win_overlay;

        log::info!(
            "enter Win32 overlay (DXGI) {}x{}", w, h
        );

        // We need dxgi ref inside the closure, but
        // also need &mut self later. Take dxgi
        // temporarily.
        let Some(dxgi) = self.dxgi.as_ref() else {
            return;
        };
        // read_pixels_into writes staging → DIB bits
        // directly. If it fails, fall back.
        let mut dxgi_ok = true;
        let result = win_overlay::run_overlay_with(
            w,
            h,
            |bits_ptr, bits_len| {
                if let Err(e) =
                    dxgi.read_pixels_into(
                        bits_ptr, bits_len,
                    )
                {
                    log::error!(
                        "DXGI read_into: {e}"
                    );
                    dxgi_ok = false;
                }
            },
        );

        if !dxgi_ok {
            // DIB was not filled — treat as cancel
            self.state = AppState::Main;
            crate::win_utils::reveal_main();
            return;
        }

        // For Copy/Save we need the BGRA data back.
        // read_pixels gives us a fresh copy (only
        // needed on selection, not on cancel).
        match result.action {
            win_overlay::OverlayAction::Copy => {
                if let Some((x, y, rw, rh)) =
                    result.selection
                {
                    self.handle_dxgi_copy(
                        x, y, rw, rh,
                    );
                }
            }
            win_overlay::OverlayAction::Save => {
                if let Some((x, y, rw, rh)) =
                    result.selection
                {
                    self.handle_dxgi_save(
                        x, y, rw, rh,
                    );
                }
            }
            win_overlay::OverlayAction::Cancel => {}
        }

        log::info!("exit Win32 overlay");
        self.state = AppState::Main;
        crate::win_utils::reveal_main();
    }

    /// Read staging texture (still holds the frozen
    /// screenshot) and crop the selected region.
    #[cfg(windows)]
    fn handle_dxgi_copy(
        &self,
        x: u32,
        y: u32,
        rw: u32,
        rh: u32,
    ) {
        let Some(dxgi) = self.dxgi.as_ref() else {
            return;
        };
        match dxgi.read_staging() {
            Ok((buf, w, _h)) => {
                let region =
                    capture::crop_bgra_to_rgba(
                        &buf, w, x, y, rw, rh,
                    );
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
            Err(e) => {
                log::error!("Copy read: {e}");
            }
        }
    }

    #[cfg(windows)]
    fn handle_dxgi_save(
        &self,
        x: u32,
        y: u32,
        rw: u32,
        rh: u32,
    ) {
        let Some(dxgi) = self.dxgi.as_ref() else {
            return;
        };
        match dxgi.read_staging() {
            Ok((buf, w, _h)) => {
                let region =
                    capture::crop_bgra_to_rgba(
                        &buf, w, x, y, rw, rh,
                    );
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
                        log::error!("Save: {e}");
                    }
                }
            }
            Err(e) => {
                log::error!("Save read: {e}");
            }
        }
    }
}
