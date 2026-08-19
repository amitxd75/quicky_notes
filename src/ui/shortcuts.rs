//! Global keyboard shortcuts and key event registry handler.

use crate::app::QuickyNotesApp;
use crate::ui;
use eframe::egui::{self, ViewportCommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Font size scaling step increment in points.
pub const FONT_SIZE_STEP: f32 = 1.0;

/// Identifies all triggerable keyboard shortcut actions in Quicky Notes.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShortcutAction {
    NewNote,
    CloseNote,
    SaveNotes,
    OpenFile,
    OpenFolder,
    ToggleFolderSidebar,
    AttachImage,
    ToggleMarkdown,
    SearchNotes,
    OpenSettings,
    ExportNote,
    ToggleAlwaysOnTop,
    ToggleOutline,
    IncreaseFontSize,
    DecreaseFontSize,
    AiAssist,
    NextTab,
    PrevTab,
    SwitchTab1,
    SwitchTab2,
    SwitchTab3,
    SwitchTab4,
    SwitchTab5,
    SwitchTab6,
    SwitchTab7,
    SwitchTab8,
    SwitchTab9,
    SwitchLastTab,
}

impl ShortcutAction {
    /// Ordered list of all shortcut actions for registry traversal and UI listing.
    pub const ALL: &'static [Self] = &[
        Self::NewNote,
        Self::CloseNote,
        Self::SaveNotes,
        Self::OpenFile,
        Self::OpenFolder,
        Self::ToggleFolderSidebar,
        Self::ToggleOutline,
        Self::AttachImage,
        Self::ToggleMarkdown,
        Self::AiAssist,
        Self::SearchNotes,
        Self::OpenSettings,
        Self::ExportNote,
        Self::ToggleAlwaysOnTop,
        Self::IncreaseFontSize,
        Self::DecreaseFontSize,
        Self::NextTab,
        Self::PrevTab,
        Self::SwitchTab1,
        Self::SwitchTab2,
        Self::SwitchTab3,
        Self::SwitchTab4,
        Self::SwitchTab5,
        Self::SwitchTab6,
        Self::SwitchTab7,
        Self::SwitchTab8,
        Self::SwitchTab9,
        Self::SwitchLastTab,
    ];

    /// Human-readable title for the shortcut action.
    pub fn title(self) -> &'static str {
        match self {
            Self::NewNote => "New Note Tab",
            Self::CloseNote => "Close Active Tab",
            Self::SaveNotes => "Save Notes to Disk",
            Self::OpenFile => "Open / Import File from Disk",
            Self::OpenFolder => "Open Folder Workspace",
            Self::ToggleFolderSidebar => "Toggle Folder Sidebar",
            Self::AttachImage => "Attach Image from Disk",
            Self::ToggleMarkdown => "Toggle Markdown Preview",
            Self::ToggleOutline => "Toggle Document Outline",
            Self::AiAssist => "AI Copilot & Fixer",
            Self::SearchNotes => "Search & Browse Notes",
            Self::OpenSettings => "Open Settings & Preferences",
            Self::ExportNote => "Export Active Note to File",
            Self::ToggleAlwaysOnTop => "Toggle Always on Top",
            Self::IncreaseFontSize => "Increase Font Size",
            Self::DecreaseFontSize => "Decrease Font Size",
            Self::NextTab => "Next Tab",
            Self::PrevTab => "Previous Tab",
            Self::SwitchTab1 => "Switch to Tab 1",
            Self::SwitchTab2 => "Switch to Tab 2",
            Self::SwitchTab3 => "Switch to Tab 3",
            Self::SwitchTab4 => "Switch to Tab 4",
            Self::SwitchTab5 => "Switch to Tab 5",
            Self::SwitchTab6 => "Switch to Tab 6",
            Self::SwitchTab7 => "Switch to Tab 7",
            Self::SwitchTab8 => "Switch to Tab 8",
            Self::SwitchTab9 => "Switch to Tab 9",
            Self::SwitchLastTab => "Switch to Last Tab",
        }
    }

    /// Functional category group for UI categorization.
    pub fn category(self) -> &'static str {
        match self {
            Self::NextTab
            | Self::PrevTab
            | Self::SwitchTab1
            | Self::SwitchTab2
            | Self::SwitchTab3
            | Self::SwitchTab4
            | Self::SwitchTab5
            | Self::SwitchTab6
            | Self::SwitchTab7
            | Self::SwitchTab8
            | Self::SwitchTab9
            | Self::SwitchLastTab => "Tabs & Navigation",

            Self::NewNote
            | Self::CloseNote
            | Self::SaveNotes
            | Self::ExportNote
            | Self::OpenFile
            | Self::OpenFolder
            | Self::AttachImage => "Note Operations",

            Self::ToggleMarkdown
            | Self::ToggleFolderSidebar
            | Self::ToggleOutline
            | Self::AiAssist
            | Self::IncreaseFontSize
            | Self::DecreaseFontSize => "Editor & View",

            Self::SearchNotes | Self::OpenSettings | Self::ToggleAlwaysOnTop => "Window & Modals",
        }
    }

    /// Default keybinding combination for this action.
    pub fn default_binding(self) -> KeyBinding {
        match self {
            Self::NewNote => KeyBinding::ctrl("N"),
            Self::CloseNote => KeyBinding::ctrl("W"),
            Self::SaveNotes => KeyBinding::ctrl("S"),
            Self::OpenFile => KeyBinding::ctrl("O"),
            Self::OpenFolder => KeyBinding::ctrl_shift("O"),
            Self::ToggleFolderSidebar => KeyBinding::ctrl("B"),
            Self::ToggleOutline => KeyBinding::alt("O"),
            Self::AttachImage => KeyBinding::ctrl_shift("I"),
            Self::ToggleMarkdown => KeyBinding::ctrl("P"),
            Self::AiAssist => KeyBinding::ctrl("Enter"),
            Self::SearchNotes => KeyBinding::ctrl("K"),
            Self::OpenSettings => KeyBinding::ctrl(","),
            Self::ExportNote => KeyBinding::ctrl_shift("E"),
            Self::ToggleAlwaysOnTop => KeyBinding::ctrl_shift("T"),
            Self::IncreaseFontSize => KeyBinding::ctrl("="),
            Self::DecreaseFontSize => KeyBinding::ctrl("-"),
            Self::NextTab => KeyBinding::ctrl("Tab"),
            Self::PrevTab => KeyBinding::ctrl_shift("Tab"),
            Self::SwitchTab1 => KeyBinding::ctrl("1"),
            Self::SwitchTab2 => KeyBinding::ctrl("2"),
            Self::SwitchTab3 => KeyBinding::ctrl("3"),
            Self::SwitchTab4 => KeyBinding::ctrl("4"),
            Self::SwitchTab5 => KeyBinding::ctrl("5"),
            Self::SwitchTab6 => KeyBinding::ctrl("6"),
            Self::SwitchTab7 => KeyBinding::ctrl("7"),
            Self::SwitchTab8 => KeyBinding::ctrl("8"),
            Self::SwitchTab9 => KeyBinding::ctrl("9"),
            Self::SwitchLastTab => KeyBinding::ctrl("0"),
        }
    }
}

/// Represents a single customizable key combination.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyBinding {
    /// String identifier for key (e.g. "N", "Tab", ",", "1", "F1", etc.)
    pub key: String,
    #[serde(default)]
    pub ctrl: bool,
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub alt: bool,
}

impl KeyBinding {
    /// Creates a key combination with Ctrl modifier.
    pub fn ctrl(key: &str) -> Self {
        Self {
            key: key.to_string(),
            ctrl: true,
            shift: false,
            alt: false,
        }
    }

    /// Creates a key combination with Ctrl + Shift modifiers.
    pub fn ctrl_shift(key: &str) -> Self {
        Self {
            key: key.to_string(),
            ctrl: true,
            shift: true,
            alt: false,
        }
    }

    /// Creates a key combination with Alt modifier.
    pub fn alt(key: &str) -> Self {
        Self {
            key: key.to_string(),
            ctrl: false,
            shift: false,
            alt: true,
        }
    }

    /// Creates an empty/unbound keybinding.
    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self {
            key: String::new(),
            ctrl: false,
            shift: false,
            alt: false,
        }
    }

    /// Parses a key combination string like "Ctrl+`", "Ctrl+Shift+T", "Alt+F12", "F12".
    pub fn from_string(s: &str) -> Self {
        let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
        let mut ctrl = false;
        let mut shift = false;
        let mut alt = false;
        let mut key = String::new();

        for part in parts {
            match part.to_lowercase().as_str() {
                "ctrl" | "control" => ctrl = true,
                "shift" => shift = true,
                "alt" => alt = true,
                other => {
                    if !other.is_empty() {
                        key = if other == "`"
                            || other == "~"
                            || other == "backquote"
                            || other == "grave"
                            || other == "backtick"
                        {
                            "`".to_string()
                        } else {
                            other.to_uppercase()
                        };
                    }
                }
            }
        }

        Self {
            key,
            ctrl,
            shift,
            alt,
        }
    }

    /// Returns true if this keybinding is unassigned.
    pub fn is_empty(&self) -> bool {
        self.key.trim().is_empty()
    }

    /// Formats the key combination into a human-readable display string (e.g., "Ctrl + Shift + E").
    pub fn to_display_string(&self) -> String {
        if self.is_empty() {
            return "Unassigned".to_string();
        }

        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }

        let key_display = match self.key.as_str() {
            "," => ",",
            "." => ".",
            "=" => "=",
            "+" => "+",
            "-" => "-",
            "Tab" => "Tab",
            "Escape" => "Esc",
            "Enter" => "Enter",
            "Space" => "Space",
            other => other,
        };
        parts.push(key_display);

        parts.join(" + ")
    }

    /// Maps string key to egui::Key enum.
    pub fn to_egui_key(&self) -> Option<egui::Key> {
        match self.key.to_uppercase().as_str() {
            "A" => Some(egui::Key::A),
            "B" => Some(egui::Key::B),
            "C" => Some(egui::Key::C),
            "D" => Some(egui::Key::D),
            "E" => Some(egui::Key::E),
            "F" => Some(egui::Key::F),
            "G" => Some(egui::Key::G),
            "H" => Some(egui::Key::H),
            "I" => Some(egui::Key::I),
            "J" => Some(egui::Key::J),
            "K" => Some(egui::Key::K),
            "L" => Some(egui::Key::L),
            "M" => Some(egui::Key::M),
            "N" => Some(egui::Key::N),
            "O" => Some(egui::Key::O),
            "P" => Some(egui::Key::P),
            "Q" => Some(egui::Key::Q),
            "R" => Some(egui::Key::R),
            "S" => Some(egui::Key::S),
            "T" => Some(egui::Key::T),
            "U" => Some(egui::Key::U),
            "V" => Some(egui::Key::V),
            "W" => Some(egui::Key::W),
            "X" => Some(egui::Key::X),
            "Y" => Some(egui::Key::Y),
            "Z" => Some(egui::Key::Z),
            "0" | "NUM0" => Some(egui::Key::Num0),
            "1" | "NUM1" => Some(egui::Key::Num1),
            "2" | "NUM2" => Some(egui::Key::Num2),
            "3" | "NUM3" => Some(egui::Key::Num3),
            "4" | "NUM4" => Some(egui::Key::Num4),
            "5" | "NUM5" => Some(egui::Key::Num5),
            "6" | "NUM6" => Some(egui::Key::Num6),
            "7" | "NUM7" => Some(egui::Key::Num7),
            "8" | "NUM8" => Some(egui::Key::Num8),
            "9" | "NUM9" => Some(egui::Key::Num9),
            "TAB" => Some(egui::Key::Tab),
            "ENTER" => Some(egui::Key::Enter),
            "ESCAPE" | "ESC" => Some(egui::Key::Escape),
            "SPACE" => Some(egui::Key::Space),
            "," | "COMMA" => Some(egui::Key::Comma),
            "." | "PERIOD" => Some(egui::Key::Period),
            "-" | "MINUS" => Some(egui::Key::Minus),
            "=" | "EQUALS" => Some(egui::Key::Equals),
            "+" | "PLUS" => Some(egui::Key::Plus),
            "[" | "OPENBRACKET" => Some(egui::Key::OpenBracket),
            "]" | "CLOSEBRACKET" => Some(egui::Key::CloseBracket),
            ";" | "SEMICOLON" => Some(egui::Key::Semicolon),
            ":" | "COLON" => Some(egui::Key::Colon),
            "'" | "\"" | "QUOTE" => Some(egui::Key::Quote),
            "/" | "SLASH" => Some(egui::Key::Slash),
            "\\" | "BACKSLASH" => Some(egui::Key::Backslash),
            "?" | "QUESTIONMARK" => Some(egui::Key::Questionmark),
            "F1" => Some(egui::Key::F1),
            "F2" => Some(egui::Key::F2),
            "F3" => Some(egui::Key::F3),
            "F4" => Some(egui::Key::F4),
            "F5" => Some(egui::Key::F5),
            "F6" => Some(egui::Key::F6),
            "F7" => Some(egui::Key::F7),
            "F8" => Some(egui::Key::F8),
            "F9" => Some(egui::Key::F9),
            "F10" => Some(egui::Key::F10),
            "F11" => Some(egui::Key::F11),
            "F12" => Some(egui::Key::F12),
            _ => None,
        }
    }

    /// Constructs a `KeyBinding` from an egui Key and Modifiers.
    pub fn from_egui_key_and_modifiers(
        key: egui::Key,
        modifiers: &egui::Modifiers,
    ) -> Option<Self> {
        let key_str = match key {
            egui::Key::A => "A",
            egui::Key::B => "B",
            egui::Key::C => "C",
            egui::Key::D => "D",
            egui::Key::E => "E",
            egui::Key::F => "F",
            egui::Key::G => "G",
            egui::Key::H => "H",
            egui::Key::I => "I",
            egui::Key::J => "J",
            egui::Key::K => "K",
            egui::Key::L => "L",
            egui::Key::M => "M",
            egui::Key::N => "N",
            egui::Key::O => "O",
            egui::Key::P => "P",
            egui::Key::Q => "Q",
            egui::Key::R => "R",
            egui::Key::S => "S",
            egui::Key::T => "T",
            egui::Key::U => "U",
            egui::Key::V => "V",
            egui::Key::W => "W",
            egui::Key::X => "X",
            egui::Key::Y => "Y",
            egui::Key::Z => "Z",
            egui::Key::Num0 => "0",
            egui::Key::Num1 => "1",
            egui::Key::Num2 => "2",
            egui::Key::Num3 => "3",
            egui::Key::Num4 => "4",
            egui::Key::Num5 => "5",
            egui::Key::Num6 => "6",
            egui::Key::Num7 => "7",
            egui::Key::Num8 => "8",
            egui::Key::Num9 => "9",
            egui::Key::Tab => "Tab",
            egui::Key::Enter => "Enter",
            egui::Key::Escape => "Escape",
            egui::Key::Space => "Space",
            egui::Key::Comma => ",",
            egui::Key::Period => ".",
            egui::Key::Minus => "-",
            egui::Key::Equals => "=",
            egui::Key::Plus => "+",
            egui::Key::OpenBracket => "[",
            egui::Key::CloseBracket => "]",
            egui::Key::Semicolon => ";",
            egui::Key::Colon => ":",
            egui::Key::Quote => "'",
            egui::Key::Slash => "/",
            egui::Key::Backslash => "\\",
            egui::Key::Questionmark => "?",
            egui::Key::F1 => "F1",
            egui::Key::F2 => "F2",
            egui::Key::F3 => "F3",
            egui::Key::F4 => "F4",
            egui::Key::F5 => "F5",
            egui::Key::F6 => "F6",
            egui::Key::F7 => "F7",
            egui::Key::F8 => "F8",
            egui::Key::F9 => "F9",
            egui::Key::F10 => "F10",
            egui::Key::F11 => "F11",
            egui::Key::F12 => "F12",
            _ => return None,
        };

        Some(Self {
            key: key_str.to_string(),
            ctrl: modifiers.ctrl,
            shift: modifiers.shift,
            alt: modifiers.alt,
        })
    }

    /// Evaluates whether the current egui input state matches this keybinding.
    pub fn matches_input(&self, input: &egui::InputState) -> bool {
        if self.is_empty() {
            return false;
        }

        if input.modifiers.ctrl != self.ctrl {
            return false;
        }
        if input.modifiers.shift != self.shift {
            return false;
        }
        if input.modifiers.alt != self.alt {
            return false;
        }

        if let Some(target_key) = self.to_egui_key() {
            if input.key_pressed(target_key) {
                return true;
            }
            // For equals / plus flexibility
            if (target_key == egui::Key::Equals || target_key == egui::Key::Plus)
                && (input.key_pressed(egui::Key::Equals) || input.key_pressed(egui::Key::Plus))
            {
                return true;
            }
        }

        false
    }
}

/// Registry mapping shortcut actions to their configured keybindings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyBindings {
    #[serde(default = "default_keybindings_map")]
    pub bindings: HashMap<ShortcutAction, KeyBinding>,
}

fn default_keybindings_map() -> HashMap<ShortcutAction, KeyBinding> {
    let mut map = HashMap::new();
    for &action in ShortcutAction::ALL {
        map.insert(action, action.default_binding());
    }
    map
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            bindings: default_keybindings_map(),
        }
    }
}

impl KeyBindings {
    /// Retrieves the binding for an action, falling back to its default if unmapped.
    pub fn get(&self, action: ShortcutAction) -> KeyBinding {
        self.bindings
            .get(&action)
            .cloned()
            .unwrap_or_else(|| action.default_binding())
    }

    /// Rebinds a shortcut action to a new key combination.
    pub fn set(&mut self, action: ShortcutAction, binding: KeyBinding) {
        self.bindings.insert(action, binding);
    }

    /// Resets a specific shortcut action to its factory default.
    pub fn reset_action(&mut self, action: ShortcutAction) {
        self.bindings.insert(action, action.default_binding());
    }

    /// Resets all shortcut actions to factory defaults.
    pub fn reset_all(&mut self) {
        self.bindings = default_keybindings_map();
    }

    /// Ensures every defined action exists in the map (e.g. after loading from older settings).
    pub fn ensure_all_actions_present(&mut self) {
        for &action in ShortcutAction::ALL {
            self.bindings
                .entry(action)
                .or_insert_with(|| action.default_binding());
        }
    }

    /// Finds if any other action shares the exact same keybinding.
    pub fn find_conflict(&self, action: ShortcutAction) -> Option<ShortcutAction> {
        let binding = self.get(action);
        if binding.is_empty() {
            return None;
        }
        ShortcutAction::ALL
            .iter()
            .copied()
            .find(|&other_action| other_action != action && self.get(other_action) == binding)
    }
}

/// Evaluates global keyboard shortcuts and handles interactive shortcut recording.
pub fn handle_keyboard_shortcuts(app: &mut QuickyNotesApp, ctx: &egui::Context) {
    // If the AI Copilot modal is open, let the modal handle all keyboard interactions
    if app.ai_modal.is_open {
        return;
    }

    // 1. Handle interactive shortcut recording mode in settings
    if let Some(recording_action) = app.recording_shortcut {
        let captured = ctx.input(|i| {
            // Cancel on Escape
            if i.key_pressed(egui::Key::Escape) && !i.modifiers.ctrl && !i.modifiers.alt {
                return Some(None);
            }

            // Find any non-modifier key pressed
            for &key in &[
                egui::Key::A,
                egui::Key::B,
                egui::Key::C,
                egui::Key::D,
                egui::Key::E,
                egui::Key::F,
                egui::Key::G,
                egui::Key::H,
                egui::Key::I,
                egui::Key::J,
                egui::Key::K,
                egui::Key::L,
                egui::Key::M,
                egui::Key::N,
                egui::Key::O,
                egui::Key::P,
                egui::Key::Q,
                egui::Key::R,
                egui::Key::S,
                egui::Key::T,
                egui::Key::U,
                egui::Key::V,
                egui::Key::W,
                egui::Key::X,
                egui::Key::Y,
                egui::Key::Z,
                egui::Key::Num0,
                egui::Key::Num1,
                egui::Key::Num2,
                egui::Key::Num3,
                egui::Key::Num4,
                egui::Key::Num5,
                egui::Key::Num6,
                egui::Key::Num7,
                egui::Key::Num8,
                egui::Key::Num9,
                egui::Key::Tab,
                egui::Key::Enter,
                egui::Key::Space,
                egui::Key::Comma,
                egui::Key::Period,
                egui::Key::Minus,
                egui::Key::Equals,
                egui::Key::Plus,
                egui::Key::OpenBracket,
                egui::Key::CloseBracket,
                egui::Key::Semicolon,
                egui::Key::Colon,
                egui::Key::Quote,
                egui::Key::Slash,
                egui::Key::Backslash,
                egui::Key::Questionmark,
                egui::Key::F1,
                egui::Key::F2,
                egui::Key::F3,
                egui::Key::F4,
                egui::Key::F5,
                egui::Key::F6,
                egui::Key::F7,
                egui::Key::F8,
                egui::Key::F9,
                egui::Key::F10,
                egui::Key::F11,
                egui::Key::F12,
            ] {
                if i.key_pressed(key)
                    && let Some(binding) =
                        KeyBinding::from_egui_key_and_modifiers(key, &i.modifiers)
                {
                    return Some(Some(binding));
                }
            }
            None
        });

        if let Some(result) = captured {
            if let Some(new_binding) = result {
                app.data
                    .settings
                    .keybindings
                    .set(recording_action, new_binding);
                let display = app
                    .data
                    .settings
                    .keybindings
                    .get(recording_action)
                    .to_display_string();
                app.show_toast(
                    format!("{}: {}", recording_action.title(), display),
                    crate::app::ToastKind::Success,
                );
            }
            app.recording_shortcut = None;
            ctx.request_repaint();
            return;
        }

        // When recording, consume other key inputs so they don't execute actions
        return;
    }

    // 2. Tab Navigation (Next / Prev Tab) with key consumption
    let next_tab_binding = app.data.settings.keybindings.get(ShortcutAction::NextTab);
    let prev_tab_binding = app.data.settings.keybindings.get(ShortcutAction::PrevTab);

    let mut trigger_next_tab = false;
    let mut trigger_prev_tab = false;

    // Check if next/prev tab uses Tab key so we consume it
    if let Some(egui::Key::Tab) = next_tab_binding.to_egui_key() {
        let mut modifiers = egui::Modifiers::NONE;
        if next_tab_binding.ctrl {
            modifiers |= egui::Modifiers::CTRL;
        }
        if next_tab_binding.shift {
            modifiers |= egui::Modifiers::SHIFT;
        }
        if next_tab_binding.alt {
            modifiers |= egui::Modifiers::ALT;
        }
        if ctx.input_mut(|i| i.consume_key(modifiers, egui::Key::Tab)) {
            trigger_next_tab = true;
        }
    }

    if let Some(egui::Key::Tab) = prev_tab_binding.to_egui_key() {
        let mut modifiers = egui::Modifiers::NONE;
        if prev_tab_binding.ctrl {
            modifiers |= egui::Modifiers::CTRL;
        }
        if prev_tab_binding.shift {
            modifiers |= egui::Modifiers::SHIFT;
        }
        if prev_tab_binding.alt {
            modifiers |= egui::Modifiers::ALT;
        }
        if ctx.input_mut(|i| i.consume_key(modifiers, egui::Key::Tab)) {
            trigger_prev_tab = true;
        }
    }

    if !trigger_next_tab && !trigger_prev_tab {
        ctx.input(|i| {
            if next_tab_binding.matches_input(i) {
                trigger_next_tab = true;
            } else if prev_tab_binding.matches_input(i) {
                trigger_prev_tab = true;
            }
        });
    }

    if (trigger_next_tab || trigger_prev_tab) && !app.data.notes.is_empty() {
        let current_idx = app
            .data
            .notes
            .iter()
            .position(|n| Some(n.id.as_str()) == app.data.active_note_id.as_deref())
            .unwrap_or(0);

        let next_idx = if trigger_prev_tab {
            if current_idx == 0 {
                app.data.notes.len() - 1
            } else {
                current_idx - 1
            }
        } else {
            (current_idx + 1) % app.data.notes.len()
        };

        app.data.active_note_id = Some(app.data.notes[next_idx].id.clone());
        app.show_options = false;
        app.show_search = false;
        app.focus_editor = true;
        ctx.request_repaint();
    }

    // 3. Evaluate all other registered global shortcuts
    let kb = &app.data.settings.keybindings;

    let trigger_new_note = ctx.input(|i| kb.get(ShortcutAction::NewNote).matches_input(i));
    let trigger_close_note = ctx.input(|i| kb.get(ShortcutAction::CloseNote).matches_input(i));
    let trigger_save = ctx.input(|i| kb.get(ShortcutAction::SaveNotes).matches_input(i));
    let trigger_open_file = ctx.input(|i| kb.get(ShortcutAction::OpenFile).matches_input(i));
    let trigger_open_folder = ctx.input(|i| kb.get(ShortcutAction::OpenFolder).matches_input(i));
    let trigger_toggle_folder_sidebar =
        ctx.input(|i| kb.get(ShortcutAction::ToggleFolderSidebar).matches_input(i));
    let trigger_toggle_outline =
        ctx.input(|i| kb.get(ShortcutAction::ToggleOutline).matches_input(i));
    let trigger_attach_image = ctx.input(|i| kb.get(ShortcutAction::AttachImage).matches_input(i));
    let trigger_markdown = ctx.input(|i| kb.get(ShortcutAction::ToggleMarkdown).matches_input(i));

    // Consume AI assist key so TextEdit doesn't insert an unwanted newline
    let ai_binding = kb.get(ShortcutAction::AiAssist);
    let mut trigger_ai = false;
    if let Some(key) = ai_binding.to_egui_key() {
        let mut modifiers = egui::Modifiers::NONE;
        if ai_binding.ctrl {
            modifiers |= egui::Modifiers::CTRL;
        }
        if ai_binding.shift {
            modifiers |= egui::Modifiers::SHIFT;
        }
        if ai_binding.alt {
            modifiers |= egui::Modifiers::ALT;
        }
        if ctx.input_mut(|i| i.consume_key(modifiers, key)) {
            trigger_ai = true;
        }
    }
    if !trigger_ai && ctx.input(|i| ai_binding.matches_input(i)) {
        trigger_ai = true;
    }

    let trigger_search = ctx.input(|i| kb.get(ShortcutAction::SearchNotes).matches_input(i));
    let trigger_settings = ctx.input(|i| kb.get(ShortcutAction::OpenSettings).matches_input(i));
    let trigger_export = ctx.input(|i| kb.get(ShortcutAction::ExportNote).matches_input(i));
    let trigger_always_on_top =
        ctx.input(|i| kb.get(ShortcutAction::ToggleAlwaysOnTop).matches_input(i));
    let trigger_font_inc = ctx.input(|i| kb.get(ShortcutAction::IncreaseFontSize).matches_input(i));
    let trigger_font_dec = ctx.input(|i| kb.get(ShortcutAction::DecreaseFontSize).matches_input(i));

    // Direct tab switching 1..9 & LastTab
    let tab_actions = [
        (ShortcutAction::SwitchTab1, 0),
        (ShortcutAction::SwitchTab2, 1),
        (ShortcutAction::SwitchTab3, 2),
        (ShortcutAction::SwitchTab4, 3),
        (ShortcutAction::SwitchTab5, 4),
        (ShortcutAction::SwitchTab6, 5),
        (ShortcutAction::SwitchTab7, 6),
        (ShortcutAction::SwitchTab8, 7),
        (ShortcutAction::SwitchTab9, 8),
    ];

    for (act, idx) in tab_actions {
        if ctx.input(|i| kb.get(act).matches_input(i))
            && let Some(note) = app.data.notes.get(idx)
        {
            app.data.active_note_id = Some(note.id.clone());
            app.show_options = false;
            app.show_search = false;
            app.focus_editor = true;
            ctx.request_repaint();
        }
    }

    if ctx.input(|i| kb.get(ShortcutAction::SwitchLastTab).matches_input(i))
        && let Some(note) = app.data.notes.last()
    {
        app.data.active_note_id = Some(note.id.clone());
        app.show_options = false;
        app.show_search = false;
        app.focus_editor = true;
        ctx.request_repaint();
    }

    if trigger_new_note {
        app.show_options = false;
        app.show_search = false;
        app.create_new_note();
    }

    if trigger_open_file {
        app.show_options = false;
        app.show_search = false;
        ui::drag_drop::open_file_dialog(app);
    }

    if trigger_open_folder {
        app.show_options = false;
        app.show_search = false;
        ui::drag_drop::open_folder_dialog(app);
    }

    if trigger_toggle_folder_sidebar {
        if app.folder_workspace.is_none() {
            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            app.open_folder_workspace(&cwd);
        } else {
            app.show_folder_sidebar = !app.show_folder_sidebar;
        }
        ctx.request_repaint();
    }

    if trigger_toggle_outline {
        app.show_outline = !app.show_outline;
        if app.show_outline {
            app.show_options = false;
            app.show_search = false;
        }
        ctx.request_repaint();
    }

    if trigger_attach_image {
        app.show_options = false;
        app.show_search = false;
        ui::drag_drop::open_image_dialog(app);
    }

    if trigger_close_note && let Some(id) = app.data.active_note_id.clone() {
        app.prompt_close_note(&id);
    }

    if trigger_save {
        app.save_notes_to_disk();
    }

    if trigger_markdown {
        if app.active_note().is_some_and(|n| n.is_markdown()) {
            app.preview_mode = app.preview_mode.next();
            app.set_status(format!("Markdown: {:?}", app.preview_mode));
        } else {
            app.set_status("Markdown preview only available for .md files");
        }
    }

    if trigger_ai {
        app.trigger_ai_assist();
        ctx.request_repaint();
    }

    if trigger_search {
        app.show_search = !app.show_search;
        app.show_options = false;
        if app.show_search {
            app.focus_search = true;
            app.search_selected_idx = 0;
        } else {
            app.focus_editor = true;
        }
        ctx.request_repaint();
    }

    if trigger_settings {
        app.show_options = !app.show_options;
        app.show_search = false;
        if !app.show_options {
            app.focus_editor = true;
        }
        ctx.request_repaint();
    }

    if trigger_export {
        ui::drag_drop::export_active_note(app);
    }

    if trigger_always_on_top {
        app.data.settings.window.always_on_top = !app.data.settings.window.always_on_top;
        let level = if app.data.settings.window.always_on_top {
            egui::WindowLevel::AlwaysOnTop
        } else {
            egui::WindowLevel::Normal
        };
        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(level));
        app.is_dirty = true;
        ctx.request_repaint();
    }

    if trigger_font_inc {
        app.data.settings.editor.font_size = (app.data.settings.editor.font_size + FONT_SIZE_STEP)
            .min(crate::models::settings::MAX_FONT_SIZE);
        app.data.settings.validate_and_clamp();
        app.is_dirty = true;
        let _ = crate::storage::AppData::save_settings_to_path(
            &app.data.settings,
            &crate::storage::AppData::config_path(),
        );
        app.set_status(format!(
            "Editor font size: {:.0}pt (saved)",
            app.data.settings.editor.font_size
        ));
        ctx.request_repaint();
    }

    if trigger_font_dec {
        app.data.settings.editor.font_size = (app.data.settings.editor.font_size - FONT_SIZE_STEP)
            .max(crate::models::settings::MIN_FONT_SIZE);
        app.data.settings.validate_and_clamp();
        app.is_dirty = true;
        let _ = crate::storage::AppData::save_settings_to_path(
            &app.data.settings,
            &crate::storage::AppData::config_path(),
        );
        app.set_status(format!(
            "Editor font size: {:.0}pt (saved)",
            app.data.settings.editor.font_size
        ));
        ctx.request_repaint();
    }

    // 3. Custom Plugin Shortcuts
    if app.data.settings.plugins.enabled {
        let plugin_shortcuts: Vec<_> = app
            .plugin_manager
            .all_shortcuts()
            .into_iter()
            .cloned()
            .collect();
        let mut triggered_plugin_action = None;
        for s in plugin_shortcuts {
            let kb = KeyBinding::from_string(&s.key_combination);
            if ctx.input(|i| kb.matches_input(i)) {
                triggered_plugin_action = Some(s.action_id.clone());
                break;
            }
        }
        if let Some(action_id) = triggered_plugin_action {
            app.dispatch_plugin_shortcut(&action_id);
            ctx.request_repaint();
        }
    }

    // Direct Ctrl+V clipboard handler (prioritizes image screenshot/file paste)
    let is_paste_key = ctx.input_mut(|i| {
        i.consume_key(egui::Modifiers::CTRL, egui::Key::V)
            || i.consume_key(egui::Modifiers::COMMAND, egui::Key::V)
            || ((i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::V))
    });

    if is_paste_key {
        if let Some((name, mime, bytes)) = crate::ui::context_menu::get_clipboard_image() {
            let cursor_range = app.last_cursor_range;
            crate::ui::drag_drop::attach_image_to_active_note_at_cursor(
                app,
                &name,
                mime,
                bytes,
                cursor_range,
            );
            ctx.request_repaint();
        } else {
            let cursor_range = app.last_cursor_range;
            let clip = crate::ui::context_menu::get_clipboard_text();
            if !clip.is_empty()
                && let Some(note) = app.active_note_mut()
            {
                let s = cursor_range.map_or(note.char_len(), |(st, _)| st);
                crate::ui::context_menu::insert_or_replace_text(note, &clip, cursor_range);
                let new_pos = s + clip.chars().count();
                app.last_cursor_range = Some((new_pos, new_pos));
                app.is_dirty = true;
                ctx.request_repaint();
            }
        }
    }

    // 4. Modal and drawer specific key handling
    ctx.input(|i| {
        // ArrowUp, ArrowDown, Enter inside search drawer
        if app.show_search && !app.data.notes.is_empty() {
            let query = app.search_query.trim().to_lowercase();
            let matching_indices: Vec<_> = app
                .data
                .notes
                .iter()
                .enumerate()
                .filter(|(_, n)| {
                    ui::search_drawer::note_matches_query(&n.title, &n.content, &query)
                })
                .map(|(idx, _)| idx)
                .collect();

            let filtered_count = matching_indices.len();

            if filtered_count > 0 {
                if app.search_selected_idx >= filtered_count {
                    app.search_selected_idx = 0;
                }
                if i.key_pressed(egui::Key::ArrowDown) {
                    app.search_selected_idx = (app.search_selected_idx + 1) % filtered_count;
                }
                if i.key_pressed(egui::Key::ArrowUp) {
                    if app.search_selected_idx == 0 {
                        app.search_selected_idx = filtered_count - 1;
                    } else {
                        app.search_selected_idx -= 1;
                    }
                }
                if i.key_pressed(egui::Key::Enter)
                    && let Some(&note_idx) = matching_indices.get(app.search_selected_idx)
                    && let Some(note) = app.data.notes.get(note_idx)
                {
                    app.data.active_note_id = Some(note.id.clone());
                    app.show_search = false;
                    app.focus_editor = true;
                }
            }
        }

        // Escape: Close active overlays, modals, and drawers; clear text selection; refocus editor
        if i.key_pressed(egui::Key::Escape) {
            if app.confirm_close_id.is_some() {
                app.confirm_close_id = None;
            } else if app.ai_modal.is_open {
                app.ai_modal.close();
                app.focus_editor = true;
            } else if app.show_options || app.show_search {
                app.show_options = false;
                app.show_search = false;
                app.focus_editor = true;
            } else if let Some((_start, end)) = app.last_cursor_range
                && _start < end
            {
                // Clear active text selection on Escape
                app.last_cursor_range = Some((end, end));
                app.focus_editor = true;
            } else {
                app.focus_editor = true;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keybinding_display_formatting() {
        let binding = KeyBinding::ctrl("N");
        assert_eq!(binding.to_display_string(), "Ctrl + N");

        let shift_binding = KeyBinding::ctrl_shift("E");
        assert_eq!(shift_binding.to_display_string(), "Ctrl + Shift + E");

        let empty = KeyBinding::empty();
        assert_eq!(empty.to_display_string(), "Unassigned");
    }

    #[test]
    fn test_keybindings_rebind_and_reset() {
        let mut kb = KeyBindings::default();
        let custom = KeyBinding {
            key: "M".to_string(),
            ctrl: true,
            shift: true,
            alt: false,
        };
        kb.set(ShortcutAction::NewNote, custom.clone());
        assert_eq!(kb.get(ShortcutAction::NewNote), custom);

        kb.reset_action(ShortcutAction::NewNote);
        assert_eq!(
            kb.get(ShortcutAction::NewNote),
            ShortcutAction::NewNote.default_binding()
        );
    }

    #[test]
    fn test_keybinding_serialization_roundtrip() {
        let kb = KeyBindings::default();
        let json = serde_json::to_string(&kb).expect("Serialization failed");
        let deserialized: KeyBindings =
            serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(kb, deserialized);
    }

    #[test]
    fn test_keybinding_conflict_detection() {
        let mut kb = KeyBindings::default();
        assert_eq!(kb.find_conflict(ShortcutAction::NewNote), None);

        // Assign same binding as NewNote to CloseNote
        kb.set(ShortcutAction::CloseNote, KeyBinding::ctrl("N"));
        assert_eq!(
            kb.find_conflict(ShortcutAction::NewNote),
            Some(ShortcutAction::CloseNote)
        );
    }
}
