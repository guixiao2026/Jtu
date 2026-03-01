use std::path::Path;
use std::time::Instant;

use eframe::egui;

#[cfg(windows)]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD,
    KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
    VK_F2, VK_LSHIFT, VK_LWIN, VK_S,
};

#[derive(Clone, Copy, PartialEq)]
enum TestTarget {
    SnapVault,
    WinShiftS,
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    /// Waiting for user to click or auto warmup
    Ready,
    /// Warmup countdown (auto mode only)
    Warmup,
    /// Keys sent, timer running, waiting for result
    Running,
    /// Result file detected, test done
    Done,
}

struct LatencyBench {
    timer_start: Option<Instant>,
    auto_target: Option<TestTarget>,
    phase: Phase,
    warmup_start: Instant,
    result_ms: Option<String>,
    screenshot_timer_ms: Option<String>,
    done_at: Option<Instant>,
}

const AUTO_WARMUP_SECS: f64 = 3.0;
const RESULT_FILE: &str = "snapvault_bench_result.txt";

impl LatencyBench {
    fn new(auto_target: Option<TestTarget>) -> Self {
        // Delete old result file
        let _ = std::fs::remove_file(RESULT_FILE);
        Self {
            timer_start: None,
            auto_target,
            phase: if auto_target.is_some() {
                Phase::Warmup
            } else {
                Phase::Ready
            },
            warmup_start: Instant::now(),
            result_ms: None,
            screenshot_timer_ms: None,
            done_at: None,
        }
    }

    #[cfg(windows)]
    fn send_keys(keys: &[(u16, bool)]) {
        let inputs: Vec<INPUT> = keys
            .iter()
            .map(|&(vk, up)| {
                let mut flags = KEYEVENTF_EXTENDEDKEY;
                if up {
                    flags |= KEYEVENTF_KEYUP;
                }
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: vk,
                            wScan: 0,
                            dwFlags: flags,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                }
            })
            .collect();
        unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            );
        }
    }

    #[cfg(windows)]
    fn fire_snapvault(&mut self) {
        self.timer_start = Some(Instant::now());
        self.phase = Phase::Running;
        Self::send_keys(&[
            (VK_LSHIFT, false),
            (VK_F2, false),
            (VK_F2, true),
            (VK_LSHIFT, true),
        ]);
    }

    #[cfg(windows)]
    fn fire_win_shift_s(&mut self) {
        self.timer_start = Some(Instant::now());
        self.phase = Phase::Running;
        Self::send_keys(&[
            (VK_LWIN, false),
            (VK_LSHIFT, false),
            (VK_S as u16, false),
            (VK_S as u16, true),
            (VK_LSHIFT, true),
            (VK_LWIN, true),
        ]);
    }

    #[cfg(not(windows))]
    fn fire_snapvault(&mut self) {
        self.timer_start = Some(Instant::now());
        self.phase = Phase::Running;
        eprintln!(
            "SendInput not available on this OS"
        );
    }

    #[cfg(not(windows))]
    fn fire_win_shift_s(&mut self) {
        self.timer_start = Some(Instant::now());
        self.phase = Phase::Running;
        eprintln!(
            "SendInput not available on this OS"
        );
    }

    fn check_result(&mut self) {
        if self.phase != Phase::Running {
            return;
        }
        if Path::new(RESULT_FILE).exists() {
            if let Ok(content) =
                std::fs::read_to_string(RESULT_FILE)
            {
                let elapsed_ms = self
                    .timer_start
                    .map(|s| {
                        s.elapsed().as_secs_f64()
                            * 1000.0
                    })
                    .unwrap_or(0.0);
                self.screenshot_timer_ms = Some(
                    format!("{elapsed_ms:.0}"),
                );
                self.result_ms =
                    Some(content.trim().to_string());
                self.phase = Phase::Done;
                self.done_at = Some(Instant::now());
            }
        }
    }
}

impl eframe::App for LatencyBench {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        ctx.request_repaint();

        // Auto-fire after warmup
        let warmup_elapsed = self
            .warmup_start
            .elapsed()
            .as_secs_f64();
        if self.phase == Phase::Warmup
            && warmup_elapsed >= AUTO_WARMUP_SECS
        {
            if let Some(target) = self.auto_target {
                match target {
                    TestTarget::SnapVault => {
                        self.fire_snapvault();
                    }
                    TestTarget::WinShiftS => {
                        self.fire_win_shift_s();
                    }
                }
            }
        }

        // Poll for result file
        self.check_result();

        // Auto-close immediately after result
        if self.phase == Phase::Done
            && self.auto_target.is_some()
        {
            ctx.send_viewport_cmd(
                egui::ViewportCommand::Close,
            );
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::WHITE),
            )
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);

                    match self.phase {
                        Phase::Ready => {
                            ui.label(
                                egui::RichText::new(
                                    "Ready",
                                )
                                .size(24.0)
                                .color(
                                    egui::Color32::DARK_GREEN,
                                )
                                .strong(),
                            );
                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new(
                                    "0 ms",
                                )
                                .size(80.0)
                                .color(
                                    egui::Color32::BLACK,
                                )
                                .strong(),
                            );
                            ui.add_space(10.0);

                            let sv = egui::Button::new(
                                egui::RichText::new(
                                    "Test Shift+F2",
                                )
                                .size(18.0),
                            )
                            .min_size(egui::vec2(
                                200.0, 50.0,
                            ));
                            if ui.add(sv).clicked() {
                                self.fire_snapvault();
                            }

                            ui.add_space(10.0);

                            let wss =
                                egui::Button::new(
                                    egui::RichText::new(
                                        "Test \
                                         Win+Shift+S",
                                    )
                                    .size(18.0),
                                )
                                .min_size(egui::vec2(
                                    200.0, 50.0,
                                ));
                            if ui.add(wss).clicked() {
                                self.fire_win_shift_s();
                            }
                        }
                        Phase::Warmup => {
                            let remaining =
                                (AUTO_WARMUP_SECS
                                    - warmup_elapsed)
                                    .max(0.0);
                            ui.label(
                                egui::RichText::new(
                                    format!(
                                        "Warmup: \
                                         {remaining:.1}s"
                                    ),
                                )
                                .size(24.0)
                                .color(
                                    egui::Color32::from_rgb(
                                        200, 150, 0,
                                    ),
                                )
                                .strong(),
                            );
                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new(
                                    "0 ms",
                                )
                                .size(80.0)
                                .color(
                                    egui::Color32::BLACK,
                                )
                                .strong(),
                            );
                        }
                        Phase::Running => {
                            ui.label(
                                egui::RichText::new(
                                    "RUNNING...",
                                )
                                .size(24.0)
                                .color(
                                    egui::Color32::RED,
                                )
                                .strong(),
                            );
                            ui.add_space(10.0);
                            let ms = self
                                .timer_start
                                .map(|s| {
                                    s.elapsed()
                                        .as_secs_f64()
                                        * 1000.0
                                })
                                .unwrap_or(0.0);
                            ui.label(
                                egui::RichText::new(
                                    format!(
                                        "{ms:.0} ms"
                                    ),
                                )
                                .size(80.0)
                                .color(
                                    egui::Color32::BLACK,
                                )
                                .strong(),
                            );
                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new(
                                    "Waiting for \
                                     capture result...",
                                )
                                .size(14.0)
                                .color(
                                    egui::Color32::GRAY,
                                ),
                            );
                            ui.add_space(10.0);
                            let back =
                                egui::Button::new(
                                    egui::RichText::new(
                                        "Back",
                                    )
                                    .size(18.0),
                                )
                                .min_size(egui::vec2(
                                    200.0, 50.0,
                                ));
                            if ui.add(back).clicked()
                            {
                                self.timer_start =
                                    None;
                                self.phase =
                                    Phase::Ready;
                            }
                        }
                        Phase::Done => {
                            ui.label(
                                egui::RichText::new(
                                    "DONE",
                                )
                                .size(24.0)
                                .color(
                                    egui::Color32::DARK_GREEN,
                                )
                                .strong(),
                            );
                            ui.add_space(10.0);
                            let dxgi_ms = self
                                .result_ms
                                .as_deref()
                                .unwrap_or("?");
                            ui.label(
                                egui::RichText::new(
                                    format!(
                                        "DXGI: \
                                         {dxgi_ms} ms"
                                    ),
                                )
                                .size(48.0)
                                .color(
                                    egui::Color32::from_rgb(
                                        0, 120, 0,
                                    ),
                                )
                                .strong(),
                            );
                            ui.add_space(10.0);
                            let e2e = self
                                .screenshot_timer_ms
                                .as_deref()
                                .unwrap_or("?");
                            ui.label(
                                egui::RichText::new(
                                    format!(
                                        "E2E: \
                                         ~{e2e} ms"
                                    ),
                                )
                                .size(24.0)
                                .color(
                                    egui::Color32::DARK_GRAY,
                                ),
                            );
                            ui.add_space(20.0);

                            let reset =
                                egui::Button::new(
                                    egui::RichText::new(
                                        "Reset",
                                    )
                                    .size(18.0),
                                )
                                .min_size(egui::vec2(
                                    200.0, 50.0,
                                ));
                            if ui
                                .add(reset)
                                .clicked()
                            {
                                let _ =
                                    std::fs::remove_file(
                                        RESULT_FILE,
                                    );
                                self.timer_start = None;
                                self.result_ms = None;
                                self.screenshot_timer_ms
                                    = None;
                                self.phase =
                                    Phase::Ready;
                            }
                        }
                    }
                });
            });
    }
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> =
        std::env::args().collect();
    let auto_target = args
        .windows(2)
        .find(|w| w[0] == "--auto")
        .and_then(|w| match w[1].as_str() {
            "snapvault" => {
                Some(TestTarget::SnapVault)
            }
            "wss" => Some(TestTarget::WinShiftS),
            _ => None,
        });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_title("Latency Bench"),
        vsync: false,
        ..Default::default()
    };

    eframe::run_native(
        "Latency Bench",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(LatencyBench::new(auto_target)))
        }),
    )
}
