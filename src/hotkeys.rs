use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};

pub struct HotkeyManager {
    manager: GlobalHotKeyManager,
    capture_hotkey: HotKey,
    active: bool,
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
            active: true,
        })
    }

    pub fn rebind(
        &mut self,
        modifiers: Option<Modifiers>,
        code: Code,
    ) -> Result<(), String> {
        let _ = self
            .manager
            .unregister(self.capture_hotkey);
        let new_hk = HotKey::new(modifiers, code);
        self.manager.register(new_hk).map_err(|e| {
            let _ = self
                .manager
                .register(self.capture_hotkey);
            format!("Register hotkey failed: {e}")
        })?;
        self.capture_hotkey = new_hk;
        self.active = true;
        Ok(())
    }

    pub fn disable(&mut self) {
        let _ = self
            .manager
            .unregister(self.capture_hotkey);
        self.active = false;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn pause(&mut self) {
        let _ = self
            .manager
            .unregister(self.capture_hotkey);
    }

    pub fn resume(&mut self) {
        let _ = self
            .manager
            .register(self.capture_hotkey);
    }

    pub fn hotkey(&self) -> &HotKey {
        &self.capture_hotkey
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

pub fn hotkey_display(hk: &HotKey) -> String {
    let mods = hk.mods;
    let mut parts = Vec::new();
    if mods.contains(Modifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if mods.contains(Modifiers::ALT) {
        parts.push("Alt");
    }
    if mods.contains(Modifiers::SHIFT) {
        parts.push("Shift");
    }
    if mods.contains(Modifiers::META) {
        parts.push("Win");
    }
    parts.push(code_name(hk.key));
    parts.join(" + ")
}

pub fn code_name(code: Code) -> &'static str {
    match code {
        Code::F1 => "F1",
        Code::F2 => "F2",
        Code::F3 => "F3",
        Code::F4 => "F4",
        Code::F5 => "F5",
        Code::F6 => "F6",
        Code::F7 => "F7",
        Code::F8 => "F8",
        Code::F9 => "F9",
        Code::F10 => "F10",
        Code::F11 => "F11",
        Code::F12 => "F12",
        Code::KeyA => "A",
        Code::KeyB => "B",
        Code::KeyC => "C",
        Code::KeyD => "D",
        Code::KeyE => "E",
        Code::KeyF => "F",
        Code::KeyG => "G",
        Code::KeyH => "H",
        Code::KeyI => "I",
        Code::KeyJ => "J",
        Code::KeyK => "K",
        Code::KeyL => "L",
        Code::KeyM => "M",
        Code::KeyN => "N",
        Code::KeyO => "O",
        Code::KeyP => "P",
        Code::KeyQ => "Q",
        Code::KeyR => "R",
        Code::KeyS => "S",
        Code::KeyT => "T",
        Code::KeyU => "U",
        Code::KeyV => "V",
        Code::KeyW => "W",
        Code::KeyX => "X",
        Code::KeyY => "Y",
        Code::KeyZ => "Z",
        Code::Digit0 => "0",
        Code::Digit1 => "1",
        Code::Digit2 => "2",
        Code::Digit3 => "3",
        Code::Digit4 => "4",
        Code::Digit5 => "5",
        Code::Digit6 => "6",
        Code::Digit7 => "7",
        Code::Digit8 => "8",
        Code::Digit9 => "9",
        Code::Space => "Space",
        Code::PrintScreen => "PrtSc",
        Code::Insert => "Insert",
        Code::Delete => "Delete",
        Code::Home => "Home",
        Code::End => "End",
        Code::PageUp => "PgUp",
        Code::PageDown => "PgDn",
        _ => "?",
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        let _ = self
            .manager
            .unregister(self.capture_hotkey);
    }
}
