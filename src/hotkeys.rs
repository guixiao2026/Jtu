use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};

pub struct HotkeyManager {
    manager: GlobalHotKeyManager,
    capture_hotkey: HotKey,
}

pub enum HotkeyEvent {
    Capture,
    None,
}

impl HotkeyManager {
    pub fn new() -> Result<Self, String> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| {
                format!("Hotkey manager: {e}")
            })?;

        let capture_hotkey = HotKey::new(
            Some(Modifiers::SHIFT),
            Code::F2,
        );

        match manager.register(capture_hotkey) {
            Ok(_) => {
                log::info!(
                    "Hotkey Shift+F2 registered OK"
                );
            }
            Err(e) => {
                let msg = format!(
                    "Register hotkey failed: {e}"
                );
                log::error!("{msg}");
                return Err(msg);
            }
        }

        Ok(Self {
            manager,
            capture_hotkey,
        })
    }

    pub fn poll_event(&self) -> HotkeyEvent {
        let receiver = GlobalHotKeyEvent::receiver();
        while let Ok(event) = receiver.try_recv() {
            if event.state() == HotKeyState::Pressed
                && event.id()
                    == self.capture_hotkey.id()
            {
                return HotkeyEvent::Capture;
            }
        }
        HotkeyEvent::None
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        let _ = self
            .manager
            .unregister(self.capture_hotkey);
    }
}
