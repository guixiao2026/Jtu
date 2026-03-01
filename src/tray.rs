use tray_icon::{
    menu::{Menu, MenuEvent, MenuId, MenuItem},
    TrayIcon, TrayIconBuilder,
};

pub struct AppTray {
    _tray: TrayIcon,
    capture_id: MenuId,
    settings_id: MenuId,
    quit_id: MenuId,
}

pub enum TrayEvent {
    Capture,
    Settings,
    Quit,
    None,
}

impl AppTray {
    pub fn new() -> Result<Self, String> {
        let menu = Menu::new();

        let capture =
            MenuItem::new("Screenshot", true, None);
        let settings =
            MenuItem::new("Settings", true, None);
        let quit = MenuItem::new("Quit", true, None);

        let capture_id = capture.id().clone();
        let settings_id = settings.id().clone();
        let quit_id = quit.id().clone();

        menu.append(&capture)
            .map_err(|e| format!("Menu: {e}"))?;
        menu.append(&settings)
            .map_err(|e| format!("Menu: {e}"))?;
        menu.append(&quit)
            .map_err(|e| format!("Menu: {e}"))?;

        let icon = create_default_icon();

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Jtu")
            .with_icon(icon)
            .build()
            .map_err(|e| {
                format!("Tray build: {e}")
            })?;

        Ok(Self {
            _tray: tray,
            capture_id,
            settings_id,
            quit_id,
        })
    }

    pub fn quit_id(&self) -> &MenuId {
        &self.quit_id
    }

    pub fn capture_id(&self) -> &MenuId {
        &self.capture_id
    }

    pub fn poll_event(&self) -> TrayEvent {
        if let Ok(event) =
            MenuEvent::receiver().try_recv()
        {
            let id = event.id();
            if *id == self.capture_id {
                return TrayEvent::Capture;
            }
            if *id == self.settings_id {
                return TrayEvent::Settings;
            }
            if *id == self.quit_id {
                return TrayEvent::Quit;
            }
        }
        TrayEvent::None
    }
}

fn create_default_icon() -> tray_icon::Icon {
    let s = 32u32;
    let mut px = vec![0u8; (s * s * 4) as usize];

    let set = |buf: &mut [u8],
               x: u32,
               y: u32,
               c: [u8; 4]| {
        if x < s && y < s {
            let off = ((y * s + x) * 4) as usize;
            buf[off..off + 4].copy_from_slice(&c);
        }
    };

    // SnapVault blue
    let blue = [66u8, 133, 244, 255];
    // Lighter accent for center dot
    let white = [220u8, 235, 255, 255];

    // ── Corner brackets (viewfinder) ────────
    let m = 3u32; // margin from edge
    let arm = 10u32; // bracket arm length
    let thick = 3u32; // bracket line thickness
    for i in 0..arm {
        for t in 0..thick {
            // top-left
            set(&mut px, m + i, m + t, blue);
            set(&mut px, m + t, m + i, blue);
            // top-right
            set(&mut px, s - 1 - m - i, m + t, blue);
            set(&mut px, s - 1 - m - t, m + i, blue);
            // bottom-left
            set(&mut px, m + i, s - 1 - m - t, blue);
            set(&mut px, m + t, s - 1 - m - i, blue);
            // bottom-right
            let rx = s - 1 - m - i;
            let ry = s - 1 - m - t;
            set(&mut px, rx, ry, blue);
            set(
                &mut px,
                s - 1 - m - t,
                s - 1 - m - i,
                blue,
            );
        }
    }

    // ── Center focus dot (5x5 diamond) ──────
    let c = 15u32; // visual center of 32px
    // 5x5 diamond shape
    let pts: &[(i32, i32)] = &[
        (0, -2),
        (-1, -1),
        (0, -1),
        (1, -1),
        (-2, 0),
        (-1, 0),
        (0, 0),
        (1, 0),
        (2, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
        (0, 2),
    ];
    for &(dx, dy) in pts {
        let x = (c as i32 + dx) as u32;
        let y = (c as i32 + dy) as u32;
        set(&mut px, x, y, white);
    }

    tray_icon::Icon::from_rgba(px, s, s).unwrap()
}
