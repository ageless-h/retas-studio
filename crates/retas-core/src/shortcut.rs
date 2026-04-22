use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Space, Enter, Escape, Tab, Backspace, Delete, Insert,
    Home, End, PageUp, PageDown,
    Up, Down, Left, Right,
    Plus, Minus, Equal, BracketLeft, BracketRight,
    Semicolon, Quote, Comma, Period, Slash, Backslash,
    Grave, CapsLock,
    Unknown,
}

impl KeyCode {
    pub fn from_key(key: &str) -> Self {
        match key.to_lowercase().as_str() {
            "a" => KeyCode::A, "b" => KeyCode::B, "c" => KeyCode::C,
            "d" => KeyCode::D, "e" => KeyCode::E, "f" => KeyCode::F,
            "g" => KeyCode::G, "h" => KeyCode::H, "i" => KeyCode::I,
            "j" => KeyCode::J, "k" => KeyCode::K, "l" => KeyCode::L,
            "m" => KeyCode::M, "n" => KeyCode::N, "o" => KeyCode::O,
            "p" => KeyCode::P, "q" => KeyCode::Q, "r" => KeyCode::R,
            "s" => KeyCode::S, "t" => KeyCode::T, "u" => KeyCode::U,
            "v" => KeyCode::V, "w" => KeyCode::W, "x" => KeyCode::X,
            "y" => KeyCode::Y, "z" => KeyCode::Z,
            "0" => KeyCode::Num0, "1" => KeyCode::Num1, "2" => KeyCode::Num2,
            "3" => KeyCode::Num3, "4" => KeyCode::Num4, "5" => KeyCode::Num5,
            "6" => KeyCode::Num6, "7" => KeyCode::Num7, "8" => KeyCode::Num8,
            "9" => KeyCode::Num9,
            "f1" => KeyCode::F1, "f2" => KeyCode::F2, "f3" => KeyCode::F3,
            "f4" => KeyCode::F4, "f5" => KeyCode::F5, "f6" => KeyCode::F6,
            "f7" => KeyCode::F7, "f8" => KeyCode::F8, "f9" => KeyCode::F9,
            "f10" => KeyCode::F10, "f11" => KeyCode::F11, "f12" => KeyCode::F12,
            "space" => KeyCode::Space, "enter" => KeyCode::Enter,
            "escape" | "esc" => KeyCode::Escape, "tab" => KeyCode::Tab,
            "backspace" => KeyCode::Backspace, "delete" => KeyCode::Delete,
            "insert" => KeyCode::Insert, "home" => KeyCode::Home,
            "end" => KeyCode::End, "pageup" => KeyCode::PageUp,
            "pagedown" => KeyCode::PageDown,
            "up" => KeyCode::Up, "down" => KeyCode::Down,
            "left" => KeyCode::Left, "right" => KeyCode::Right,
            "plus" | "=" | "equal" => KeyCode::Plus,
            "minus" | "-" => KeyCode::Minus,
            "[" => KeyCode::BracketLeft, "]" => KeyCode::BracketRight,
            ";" => KeyCode::Semicolon, "'" => KeyCode::Quote,
            "," => KeyCode::Comma, "." => KeyCode::Period,
            "/" => KeyCode::Slash, "\\" => KeyCode::Backslash,
            "`" => KeyCode::Grave, "capslock" => KeyCode::CapsLock,
            _ => KeyCode::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

impl Default for KeyModifiers {
    fn default() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        }
    }
}

impl KeyModifiers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub fn alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn meta(mut self) -> Self {
        self.meta = true;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Shortcut {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
}

impl Shortcut {
    pub fn new(key: KeyCode) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::default(),
        }
    }

    pub fn with_modifiers(key: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { key, modifiers }
    }

    pub fn ctrl(key: KeyCode) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::new().ctrl(),
        }
    }

    pub fn alt(key: KeyCode) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::new().alt(),
        }
    }

    pub fn shift(key: KeyCode) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::new().shift(),
        }
    }

    pub fn ctrl_shift(key: KeyCode) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::new().ctrl().shift(),
        }
    }

    pub fn matches(&self, key: KeyCode, modifiers: &KeyModifiers) -> bool {
        self.key == key && 
        self.modifiers.ctrl == modifiers.ctrl &&
        self.modifiers.alt == modifiers.alt &&
        self.modifiers.shift == modifiers.shift &&
        self.modifiers.meta == modifiers.meta
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShortcutAction {
    Undo,
    Redo,
    Save,
    SaveAs,
    Open,
    New,
    Cut,
    Copy,
    Paste,
    SelectAll,
    DeselectAll,
    Delete,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    Pan,
    Brush,
    Eraser,
    Fill,
    Eyedropper,
    Pen,
    Text,
    Move,
    Select,
    Lasso,
    MagicWand,
    NewLayer,
    DeleteLayer,
    DuplicateLayer,
    MergeDown,
    LayerUp,
    LayerDown,
    PlayPause,
    NextFrame,
    PrevFrame,
    FirstFrame,
    LastFrame,
    ToggleOnionSkin,
    ToggleGrid,
    ToggleRulers,
    Fullscreen,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutBinding {
    pub shortcut: Shortcut,
    pub action: ShortcutAction,
    pub description: String,
}

impl ShortcutBinding {
    pub fn new(shortcut: Shortcut, action: ShortcutAction, description: impl Into<String>) -> Self {
        Self {
            shortcut,
            action,
            description: description.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutManager {
    bindings: HashMap<Shortcut, ShortcutAction>,
    descriptions: HashMap<ShortcutAction, String>,
}

impl ShortcutManager {
    pub fn new() -> Self {
        let mut manager = Self {
            bindings: HashMap::new(),
            descriptions: HashMap::new(),
        };
        manager.load_defaults();
        manager
    }

    fn load_defaults(&mut self) {
        self.bind(Shortcut::ctrl(KeyCode::Z), ShortcutAction::Undo, "Undo");
        self.bind(Shortcut::ctrl_shift(KeyCode::Z), ShortcutAction::Redo, "Redo");
        self.bind(Shortcut::ctrl(KeyCode::Y), ShortcutAction::Redo, "Redo (alternate)");
        
        self.bind(Shortcut::ctrl(KeyCode::S), ShortcutAction::Save, "Save");
        self.bind(Shortcut::ctrl_shift(KeyCode::S), ShortcutAction::SaveAs, "Save As");
        self.bind(Shortcut::ctrl(KeyCode::O), ShortcutAction::Open, "Open");
        self.bind(Shortcut::ctrl(KeyCode::N), ShortcutAction::New, "New Document");
        
        self.bind(Shortcut::ctrl(KeyCode::X), ShortcutAction::Cut, "Cut");
        self.bind(Shortcut::ctrl(KeyCode::C), ShortcutAction::Copy, "Copy");
        self.bind(Shortcut::ctrl(KeyCode::V), ShortcutAction::Paste, "Paste");
        self.bind(Shortcut::ctrl(KeyCode::A), ShortcutAction::SelectAll, "Select All");
        self.bind(Shortcut::ctrl_shift(KeyCode::A), ShortcutAction::DeselectAll, "Deselect All");
        self.bind(Shortcut::new(KeyCode::Delete), ShortcutAction::Delete, "Delete");
        
        self.bind(Shortcut::ctrl(KeyCode::Plus), ShortcutAction::ZoomIn, "Zoom In");
        self.bind(Shortcut::ctrl(KeyCode::Minus), ShortcutAction::ZoomOut, "Zoom Out");
        self.bind(Shortcut::ctrl(KeyCode::Num0), ShortcutAction::ZoomReset, "Zoom Reset");
        
        self.bind(Shortcut::new(KeyCode::B), ShortcutAction::Brush, "Brush Tool");
        self.bind(Shortcut::new(KeyCode::E), ShortcutAction::Eraser, "Eraser Tool");
        self.bind(Shortcut::new(KeyCode::G), ShortcutAction::Fill, "Fill Tool");
        self.bind(Shortcut::new(KeyCode::I), ShortcutAction::Eyedropper, "Eyedropper Tool");
        self.bind(Shortcut::new(KeyCode::P), ShortcutAction::Pen, "Pen Tool");
        self.bind(Shortcut::new(KeyCode::T), ShortcutAction::Text, "Text Tool");
        self.bind(Shortcut::new(KeyCode::V), ShortcutAction::Move, "Move Tool");
        self.bind(Shortcut::new(KeyCode::M), ShortcutAction::Select, "Select Tool");
        self.bind(Shortcut::new(KeyCode::L), ShortcutAction::Lasso, "Lasso Tool");
        self.bind(Shortcut::new(KeyCode::W), ShortcutAction::MagicWand, "Magic Wand Tool");
        
        self.bind(Shortcut::ctrl_shift(KeyCode::N), ShortcutAction::NewLayer, "New Layer");
        self.bind(Shortcut::ctrl(KeyCode::Delete), ShortcutAction::DeleteLayer, "Delete Layer");
        self.bind(Shortcut::ctrl(KeyCode::D), ShortcutAction::DuplicateLayer, "Duplicate Layer");
        self.bind(Shortcut::ctrl(KeyCode::E), ShortcutAction::MergeDown, "Merge Down");
        self.bind(Shortcut::new(KeyCode::BracketLeft), ShortcutAction::LayerUp, "Move Layer Up");
        self.bind(Shortcut::new(KeyCode::BracketRight), ShortcutAction::LayerDown, "Move Layer Down");
        
        self.bind(Shortcut::new(KeyCode::Space), ShortcutAction::PlayPause, "Play/Pause");
        self.bind(Shortcut::new(KeyCode::Right), ShortcutAction::NextFrame, "Next Frame");
        self.bind(Shortcut::new(KeyCode::Left), ShortcutAction::PrevFrame, "Previous Frame");
        self.bind(Shortcut::new(KeyCode::Home), ShortcutAction::FirstFrame, "First Frame");
        self.bind(Shortcut::new(KeyCode::End), ShortcutAction::LastFrame, "Last Frame");
        self.bind(Shortcut::new(KeyCode::O), ShortcutAction::ToggleOnionSkin, "Toggle Onion Skin");
        
        self.bind(Shortcut::ctrl(KeyCode::Grave), ShortcutAction::ToggleGrid, "Toggle Grid");
        self.bind(Shortcut::ctrl_shift(KeyCode::R), ShortcutAction::ToggleRulers, "Toggle Rulers");
        self.bind(Shortcut::new(KeyCode::F11), ShortcutAction::Fullscreen, "Fullscreen");
    }

    pub fn bind(&mut self, shortcut: Shortcut, action: ShortcutAction, description: &str) {
        self.bindings.insert(shortcut, action.clone());
        self.descriptions.insert(action, description.to_string());
    }

    pub fn unbind(&mut self, shortcut: &Shortcut) {
        if let Some(action) = self.bindings.remove(shortcut) {
            self.descriptions.remove(&action);
        }
    }

    pub fn lookup(&self, key: KeyCode, modifiers: &KeyModifiers) -> Option<&ShortcutAction> {
        let shortcut = Shortcut {
            key,
            modifiers: *modifiers,
        };
        self.bindings.get(&shortcut)
    }

    pub fn get_description(&self, action: &ShortcutAction) -> Option<&str> {
        self.descriptions.get(action).map(|s| s.as_str())
    }

    pub fn get_all_bindings(&self) -> Vec<ShortcutBinding> {
        self.bindings
            .iter()
            .map(|(shortcut, action)| ShortcutBinding {
                shortcut: *shortcut,
                action: action.clone(),
                description: self.descriptions.get(action).cloned().unwrap_or_default(),
            })
            .collect()
    }

    pub fn get_bindings_for_action(&self, action: &ShortcutAction) -> Vec<Shortcut> {
        self.bindings
            .iter()
            .filter(|(_, a)| **a == *action)
            .map(|(s, _)| *s)
            .collect()
    }
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}
