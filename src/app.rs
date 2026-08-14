//! Core Quicky Notes application controller and event handler.

use crate::note::Note;
use crate::storage::AppData;
use crate::theme::{self, ACCENT_EMERALD, ACCENT_PURPLE};
use crate::ui;
use eframe::egui::{
    self, Color32, FontId, Margin, RichText, Rounding, Stroke, Ui, ViewportCommand,
};
use std::time::{Duration, Instant};

/// Main application state for Quicky Notes.
pub struct QuickyNotesApp {
    /// Saved data container (notes, active note ID, settings).
    pub data: AppData,

    /// Live query string entered in search drawer.
    pub search_query: String,

    /// Highlighted index in search result list.
    pub search_selected_idx: usize,

    /// ID of note title currently undergoing inline rename.
    pub editing_title_id: Option<String>,

    /// Temporary title buffer during inline rename.
    pub temp_title_input: String,

    /// Whether the Options & Settings drawer is open.
    pub show_options: bool,

    /// Whether the Search & Browse drawer is open.
    pub show_search: bool,

    /// Trigger flag to request focus on the search input field.
    pub focus_search: bool,

    /// Trigger flag to request focus on the main text editor.
    pub focus_editor: bool,

    /// Status bar notification text and creation timestamp.
    pub status_msg: Option<(String, Instant)>,

    /// Last auto-save timestamp.
    pub last_auto_save: Instant,

    /// Last wallpaper color polling timestamp.
    pub last_wallpaper_check: Instant,

    /// Cached wallpaper colors for change detection.
    pub last_wallpaper_colors: Option<(Color32, Color32, Color32)>,

    /// ID of note tab pending close confirmation.
    pub confirm_close_id: Option<String>,

    /// Whether unsaved modifications exist.
    pub is_dirty: bool,
}

impl QuickyNotesApp {
    /// Legacy constructor.
    #[allow(dead_code)]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let data = AppData::load();
        Self::new_with_data(cc, data)
    }

    /// Primary constructor using pre-loaded AppData.
    pub fn new_with_data(cc: &eframe::CreationContext<'_>, data: AppData) -> Self {
        theme::setup_glassmorphism_theme(
            &cc.egui_ctx,
            data.settings.opacity,
            data.settings.theme_mode,
        );
        crate::font::apply_system_font(&cc.egui_ctx, &data.settings.selected_font);
        let initial_colors = theme::get_wallpaper_colors();

        if data.settings.always_on_top {
            cc.egui_ctx
                .send_viewport_cmd(ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));
        }

        // Restore saved window dimensions on startup for Hyprland Lua
        let w = data.settings.window_width as u32;
        let h = data.settings.window_height as u32;
        let _ = std::process::Command::new("hyprctl")
            .arg("dispatch")
            .arg(format!("hl.dsp.window.resize({{ x = {}, y = {} }})", w, h))
            .spawn();

        Self {
            data,
            search_query: String::new(),
            search_selected_idx: 0,
            editing_title_id: None,
            temp_title_input: String::new(),
            show_options: false,
            show_search: false,
            focus_search: false,
            focus_editor: true,
            status_msg: Some(("Quicky Notes ready".to_string(), Instant::now())),
            last_auto_save: Instant::now(),
            last_wallpaper_check: Instant::now(),
            last_wallpaper_colors: initial_colors,
            confirm_close_id: None,
            is_dirty: false,
        }
    }

    /// Prompts close confirmation modal if note has content, or closes immediately if empty.
    pub fn prompt_close_note(&mut self, id: &str) {
        if let Some(note) = self.data.notes.iter().find(|n| n.id == id) {
            if note.content.trim().is_empty() {
                self.close_note(id);
            } else {
                self.confirm_close_id = Some(id.to_string());
            }
        }
    }

    /// Sets status bar notification text.
    pub fn set_status(&mut self, text: impl Into<String>) {
        self.status_msg = Some((text.into(), Instant::now()));
    }

    /// Returns mutable reference to active note.
    pub fn active_note_mut(&mut self) -> Option<&mut Note> {
        let active_id = self.data.active_note_id.clone()?;
        self.data.notes.iter_mut().find(|n| n.id == active_id)
    }

    /// Returns immutable reference to active note.
    pub fn active_note(&self) -> Option<&Note> {
        let active_id = self.data.active_note_id.as_ref()?;
        self.data.notes.iter().find(|n| n.id == *active_id)
    }

    /// Creates a new note tab.
    pub fn create_new_note(&mut self) {
        let count = self.data.notes.len() + 1;
        let id = format!("note-{}", chrono::Local::now().timestamp_millis());
        let title = format!("note_{}.txt", count);
        let note = Note::new(id.clone(), title);
        self.data.notes.push(note);
        self.data.active_note_id = Some(id);
        self.focus_editor = true;
        self.is_dirty = true;
        self.set_status("New tab created");
    }

    /// Closes note tab by ID.
    pub fn close_note(&mut self, id: &str) {
        if self.data.notes.len() <= 1 {
            if let Some(note) = self.data.notes.first_mut() {
                note.title = "untitled.txt".to_string();
                note.content.clear();
                note.update_timestamp();
            }
            self.set_status("Tab cleared");
            self.is_dirty = true;
            return;
        }

        if let Some(pos) = self.data.notes.iter().position(|n| n.id == id) {
            self.data.notes.remove(pos);
            if self.data.active_note_id.as_deref() == Some(id) {
                let next_idx = pos.saturating_sub(1);
                self.data.active_note_id = self.data.notes.get(next_idx).map(|n| n.id.clone());
            }
            self.is_dirty = true;
            self.set_status("Tab closed");
        }
    }

    /// Exports active note to ~/Documents or current working directory.
    pub fn export_active_note(&mut self) {
        if let Some(note) = self.active_note() {
            let filename = if note.title.trim().is_empty() {
                "quicky_note.txt".to_string()
            } else if note.title.ends_with(".txt") || note.title.ends_with(".md") {
                note.title.clone()
            } else {
                format!("{}.txt", note.title)
            };

            let path = directories::UserDirs::new()
                .and_then(|u| u.document_dir().map(|d| d.join(&filename)))
                .unwrap_or_else(|| std::path::PathBuf::from(&filename));

            if std::fs::write(&path, &note.content).is_ok() {
                self.set_status(format!("Exported to {}", filename));
            } else {
                self.set_status("Export failed");
            }
        }
    }

    /// Evaluates global keyboard shortcuts.
    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        let ctrl_shift_tab = ctx.input_mut(|i| {
            i.consume_key(
                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                egui::Key::Tab,
            )
        });
        let ctrl_tab = ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Tab));

        if (ctrl_tab || ctrl_shift_tab) && !self.data.notes.is_empty() {
            let current_idx = self
                .data
                .notes
                .iter()
                .position(|n| Some(&n.id) == self.data.active_note_id.as_ref())
                .unwrap_or(0);

            let next_idx = if ctrl_shift_tab {
                if current_idx == 0 {
                    self.data.notes.len() - 1
                } else {
                    current_idx - 1
                }
            } else {
                (current_idx + 1) % self.data.notes.len()
            };

            self.data.active_note_id = Some(self.data.notes[next_idx].id.clone());
            self.focus_editor = true;
        }

        ctx.input(|i| {
            // Ctrl + N: New note tab
            if i.modifiers.ctrl && i.key_pressed(egui::Key::N) {
                self.create_new_note();
            }

            // Ctrl + W: Close active note tab
            if i.modifiers.ctrl
                && i.key_pressed(egui::Key::W)
                && let Some(id) = self.data.active_note_id.clone()
            {
                self.prompt_close_note(&id);
            }

            // Ctrl + S: Save to disk
            if i.modifiers.ctrl && i.key_pressed(egui::Key::S) && self.data.save().is_ok() {
                self.is_dirty = false;
                self.set_status("Saved to disk ✓");
            }

            // Ctrl + K: Search notes modal
            if i.modifiers.ctrl && i.key_pressed(egui::Key::K) {
                self.show_search = !self.show_search;
                self.show_options = false;
                if self.show_search {
                    self.focus_search = true;
                } else {
                    self.focus_editor = true;
                }
            }

            // Ctrl + ,: Options modal
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Comma) {
                self.show_options = !self.show_options;
                self.show_search = false;
                if !self.show_options {
                    self.focus_editor = true;
                }
            }

            // Ctrl + Shift + E: Export note
            if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::E) {
                self.export_active_note();
            }

            // Ctrl + Shift + T: Toggle Always on Top
            if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::T) {
                self.data.settings.always_on_top = !self.data.settings.always_on_top;
                let level = if self.data.settings.always_on_top {
                    egui::WindowLevel::AlwaysOnTop
                } else {
                    egui::WindowLevel::Normal
                };
                ctx.send_viewport_cmd(ViewportCommand::WindowLevel(level));
                self.is_dirty = true;
            }

            // Ctrl + + / Ctrl + =: Increase font size
            if i.modifiers.ctrl
                && (i.key_pressed(egui::Key::Equals) || i.key_pressed(egui::Key::Plus))
            {
                self.data.settings.font_size = (self.data.settings.font_size + 1.0).min(32.0);
                self.is_dirty = true;
                let _ = self.data.save();
                self.set_status(format!("Font size: {:.0}pt", self.data.settings.font_size));
            }

            // Ctrl + -: Decrease font size
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Minus) {
                self.data.settings.font_size = (self.data.settings.font_size - 1.0).max(8.0);
                self.is_dirty = true;
                let _ = self.data.save();
                self.set_status(format!("Font size: {:.0}pt", self.data.settings.font_size));
            }

            // Ctrl + 1..=9 / Ctrl + 0: Switch to tab by index
            if i.modifiers.ctrl {
                let num_keys = [
                    (egui::Key::Num1, 0),
                    (egui::Key::Num2, 1),
                    (egui::Key::Num3, 2),
                    (egui::Key::Num4, 3),
                    (egui::Key::Num5, 4),
                    (egui::Key::Num6, 5),
                    (egui::Key::Num7, 6),
                    (egui::Key::Num8, 7),
                    (egui::Key::Num9, 8),
                ];
                for (key, idx) in num_keys {
                    if i.key_pressed(key)
                        && let Some(note) = self.data.notes.get(idx)
                    {
                        self.data.active_note_id = Some(note.id.clone());
                        self.focus_editor = true;
                    }
                }
                if i.key_pressed(egui::Key::Num0)
                    && let Some(note) = self.data.notes.last()
                {
                    self.data.active_note_id = Some(note.id.clone());
                    self.focus_editor = true;
                }
            }

            // ArrowUp, ArrowDown, Enter inside search drawer
            if self.show_search && !self.data.notes.is_empty() {
                let query = self.search_query.trim().to_lowercase();
                let filtered_count = self
                    .data
                    .notes
                    .iter()
                    .filter(|n| {
                        query.is_empty()
                            || n.title.to_lowercase().contains(&query)
                            || n.content.to_lowercase().contains(&query)
                    })
                    .count();

                if filtered_count > 0 {
                    if self.search_selected_idx >= filtered_count {
                        self.search_selected_idx = 0;
                    }
                    if i.key_pressed(egui::Key::ArrowDown) {
                        self.search_selected_idx = (self.search_selected_idx + 1) % filtered_count;
                    }
                    if i.key_pressed(egui::Key::ArrowUp) {
                        if self.search_selected_idx == 0 {
                            self.search_selected_idx = filtered_count - 1;
                        } else {
                            self.search_selected_idx -= 1;
                        }
                    }
                    if i.key_pressed(egui::Key::Enter) {
                        let filtered_notes: Vec<_> = self
                            .data
                            .notes
                            .iter()
                            .filter(|n| {
                                query.is_empty()
                                    || n.title.to_lowercase().contains(&query)
                                    || n.content.to_lowercase().contains(&query)
                            })
                            .collect();
                        if let Some(note) = filtered_notes.get(self.search_selected_idx) {
                            self.data.active_note_id = Some(note.id.clone());
                            self.show_search = false;
                            self.focus_editor = true;
                        }
                    }
                }
            }

            // Escape: Close modals if open, else close app window
            if i.key_pressed(egui::Key::Escape) {
                if self.confirm_close_id.is_some() {
                    self.confirm_close_id = None;
                } else if self.show_options || self.show_search {
                    self.show_options = false;
                    self.show_search = false;
                    self.focus_editor = true;
                } else {
                    self.save_if_dirty();
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
            }
        });
    }

    /// Saves notes and settings if dirty flag is set.
    fn save_if_dirty(&mut self) {
        if self.is_dirty && self.data.save().is_ok() {
            self.is_dirty = false;
            self.set_status("Auto-saved");
        }
    }

    /// Renders the main glass editor box containing header, workspace, and status bar.
    fn render_main_editor(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        theme::glass_editor_frame(self.data.settings.opacity, self.data.settings.theme_mode).show(
            ui,
            |ui| {
                ui.vertical(|ui| {
                    // 1. Sleek Header Bar
                    ui::header::render_header(self, ctx, ui);

                    // Divider line below header
                    ui::draw_horizontal_divider(ui);

                    // 2. Body Area (Editor vs Options Drawer vs Search Drawer)
                    let font_size = self.data.settings.font_size;
                    let is_monospace = self.data.settings.monospace_font;
                    let mut content_changed = false;
                    let should_focus = self.focus_editor;

                    egui::Frame::none()
                        .inner_margin(Margin::symmetric(14.0, 10.0))
                        .show(ui, |ui| {
                            if self.show_options {
                                ui::options_drawer::render_options_drawer(self, ctx, ui);
                            } else if self.show_search {
                                ui::search_drawer::render_search_drawer(self, ctx, ui);
                            } else if let Some(note) = self.active_note_mut() {
                                let editor_height = ui.available_height() - 34.0;
                                egui::ScrollArea::vertical()
                                    .id_salt("editor_scroll_area")
                                    .auto_shrink([false, false])
                                    .max_height(editor_height)
                                    .show(ui, |ui| {
                                        ui.horizontal_top(|ui| {
                                            // Line numbers column (chunked Label rendering for exact font matching and 100,000+ lines)
                                            let line_count =
                                                note.content.split('\n').count().max(1);
                                            let line_font = if is_monospace {
                                                FontId::monospace(font_size)
                                            } else {
                                                FontId::proportional(font_size)
                                            };

                                            ui.vertical(|ui| {
                                                let chunk_size = 300;
                                                for chunk_start in
                                                    (1..=line_count).step_by(chunk_size)
                                                {
                                                    let chunk_end = (chunk_start + chunk_size - 1)
                                                        .min(line_count);
                                                    let chunk_str = (chunk_start..=chunk_end)
                                                        .map(|n| n.to_string())
                                                        .collect::<Vec<_>>()
                                                        .join("\n");

                                                    ui.add(egui::Label::new(
                                                        RichText::new(&chunk_str)
                                                            .font(line_font.clone())
                                                            .color(Color32::from_gray(110)),
                                                    ));
                                                }
                                            });

                                            ui.add_space(8.0);

                                            // Vertical line separator
                                            let line_height = font_size * 1.3 * line_count as f32;
                                            let (_, rect) = ui.allocate_space(egui::vec2(
                                                1.0,
                                                line_height.max(editor_height),
                                            ));
                                            ui.painter().line_segment(
                                                [rect.min, egui::pos2(rect.min.x, rect.max.y)],
                                                Stroke::new(
                                                    1.0_f32,
                                                    Color32::from_rgba_unmultiplied(
                                                        80, 50, 110, 60,
                                                    ),
                                                ),
                                            );

                                            ui.add_space(8.0);

                                            // Multiline text editor
                                            let font = if is_monospace {
                                                FontId::monospace(font_size)
                                            } else {
                                                FontId::proportional(font_size)
                                            };

                                            let text_edit =
                                                egui::TextEdit::multiline(&mut note.content)
                                                    .id(egui::Id::new(format!(
                                                        "editor_{}",
                                                        note.id
                                                    )))
                                                    .font(font)
                                                    .desired_width(f32::INFINITY)
                                                    .lock_focus(true)
                                                    .frame(false);

                                            let response = ui.add(text_edit);
                                            if should_focus {
                                                response.request_focus();
                                            }
                                            if response.changed() {
                                                note.update_timestamp();
                                                content_changed = true;
                                            }
                                        });
                                    });
                            } else {
                                ui.centered_and_justified(|ui| {
                                    ui.label(
                                        RichText::new("No note open. Press '+' to create one.")
                                            .color(Color32::GRAY),
                                    );
                                });
                            }
                        });

                    if should_focus {
                        self.focus_editor = false;
                    }

                    if content_changed {
                        self.is_dirty = true;
                    }

                    // Divider line above status bar
                    ui::draw_horizontal_divider(ui);

                    // 3. Status Bar at bottom of Editor Box
                    egui::Frame::none()
                        .inner_margin(Margin::symmetric(14.0, 6.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Left: Green status dot ● Saved
                                ui.label(RichText::new("●").size(10.0).color(ACCENT_EMERALD));
                                ui.label(
                                    RichText::new("Saved")
                                        .size(12.0)
                                        .color(Color32::from_gray(210)),
                                );

                                // Right stats: Words: X | Characters: Y | Ln A, Col B | UTF-8
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.spacing_mut().item_spacing.x = 14.0;

                                        ui.label(
                                            RichText::new("UTF-8")
                                                .size(12.0)
                                                .color(Color32::from_gray(170)),
                                        );

                                        if let Some(note) = self.active_note() {
                                            let line_count =
                                                note.content.split('\n').count().max(1);
                                            let last_line_len = note
                                                .content
                                                .split('\n')
                                                .next_back()
                                                .unwrap_or("")
                                                .chars()
                                                .count()
                                                + 1;

                                            ui.label(
                                                RichText::new(format!(
                                                    "Ln {}, Col {}",
                                                    line_count, last_line_len
                                                ))
                                                .size(12.0)
                                                .color(Color32::from_gray(170)),
                                            );
                                            ui.label(
                                                RichText::new(format!(
                                                    "Characters: {}",
                                                    note.char_count()
                                                ))
                                                .size(12.0)
                                                .color(Color32::from_gray(170)),
                                            );
                                            ui.label(
                                                RichText::new(format!(
                                                    "Words: {}",
                                                    note.word_count()
                                                ))
                                                .size(12.0)
                                                .color(Color32::from_gray(170)),
                                            );
                                        }
                                    },
                                );
                            });
                        });
                });
            },
        );
    }
    /// Opens a native file dialog (Zenity/Kdialog) to import any text file from Dolphin.
    pub fn open_file_dialog(&mut self) {
        let output = std::process::Command::new("zenity")
            .arg("--file-selection")
            .output()
            .or_else(|_| {
                std::process::Command::new("kdialog")
                    .arg("--getopenfilename")
                    .output()
            });

        if let Ok(out) = output
            && out.status.success()
        {
            let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let path = std::path::Path::new(&path_str);
            if path.exists()
                && path.is_file()
                && let Ok(content) = std::fs::read_to_string(path)
            {
                let name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "imported.txt".to_string());

                let id = format!("note-{}", chrono::Local::now().timestamp_millis());
                let mut note = Note::new(id.clone(), name.clone());
                note.content = content;
                self.data.notes.push(note);
                self.data.active_note_id = Some(id);
                self.focus_editor = true;
                self.is_dirty = true;
                self.set_status(format!("Opened {}", name));
            }
        }
    }

    /// Handles drag-and-dropped files, file URIs (file://), or raw text snippets into the window.
    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input_mut(|i| std::mem::take(&mut i.raw.dropped_files));
        if dropped.is_empty() {
            return;
        }

        let mut files_to_open: Vec<(String, String)> = Vec::new();

        for file in dropped {
            if let Some(path) = &file.path
                && let Ok(content) = std::fs::read_to_string(path)
            {
                let name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "dropped.txt".to_string());
                files_to_open.push((name, content));
                continue;
            }

            if let Some(bytes) = &file.bytes {
                self.process_dropped_bytes(bytes, &mut files_to_open);
            }
        }

        // Open collected dropped files as new tabs
        for (name, content) in files_to_open {
            if let Some(existing) = self
                .data
                .notes
                .iter()
                .find(|n| n.title == name && n.content == content)
            {
                self.data.active_note_id = Some(existing.id.clone());
            } else {
                let id = format!("note-{}", chrono::Local::now().timestamp_millis());
                let mut note = Note::new(id.clone(), name.clone());
                note.content = content;
                self.data.notes.push(note);
                self.data.active_note_id = Some(id);
                self.is_dirty = true;
            }
            self.focus_editor = true;
            self.set_status(format!("Opened {}", name));
        }
    }

    /// Helper to process byte payload from Dolphin / Wayland dropped files or file:// URI strings
    fn process_dropped_bytes(&self, bytes: &[u8], out: &mut Vec<(String, String)>) {
        let raw_text = String::from_utf8_lossy(bytes);
        let mut opened_any = false;

        for line in raw_text.lines() {
            let line = line
                .trim_matches(|c: char| c == '\0' || c == '\r' || c == '\n' || c.is_whitespace());
            if line.is_empty() {
                continue;
            }

            let unquoted = line.trim_matches('"').trim_matches('\'');
            let decoded = url_decode(unquoted);

            let path_str = if decoded.starts_with("file://") {
                decoded.trim_start_matches("file://")
            } else if unquoted.starts_with("file://") {
                unquoted.trim_start_matches("file://")
            } else if decoded.starts_with("file:") {
                decoded.trim_start_matches("file:")
            } else {
                decoded.as_str()
            };

            let path = std::path::Path::new(path_str);
            if path.exists()
                && path.is_file()
                && let Ok(content) = std::fs::read_to_string(path)
            {
                let name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "dropped.txt".to_string());
                out.push((name, content));
                opened_any = true;
                continue;
            }
        }

        if !opened_any && !raw_text.trim().is_empty() {
            out.push(("dropped_snippet.txt".to_string(), raw_text.to_string()));
        }
    }
}

/// Decodes percent-encoded URL strings (e.g. `%20` -> ` `).
fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(val) = u8::from_str_radix(&s[i + 1..i + 3], 16)
        {
            result.push(val as char);
            i += 3;
            continue;
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

impl eframe::App for QuickyNotesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_dropped_files(ctx);
        self.handle_keyboard_shortcuts(ctx);

        if ctx.input(|i| i.viewport().close_requested()) {
            self.save_if_dirty();
        }

        if self.last_auto_save.elapsed()
            > Duration::from_secs(self.data.settings.auto_save_seconds as u64)
        {
            self.save_if_dirty();
            self.last_auto_save = Instant::now();
        }

        // Real-time wallpaper theme sync (polls Caelestia/Pywal every 1s)
        if self.data.settings.theme_mode == theme::ThemeMode::WallpaperSync
            && self.last_wallpaper_check.elapsed() > Duration::from_secs(1)
        {
            self.last_wallpaper_check = Instant::now();
            if theme::check_wallpaper_color_change(&mut self.last_wallpaper_colors) {
                theme::setup_glassmorphism_theme(
                    ctx,
                    self.data.settings.opacity,
                    theme::ThemeMode::WallpaperSync,
                );
                ctx.request_repaint();
            }
        }

        // Clean main window panel with integrated drawers
        egui::CentralPanel::default()
            .frame(egui::Frame::none().inner_margin(Margin::same(2.0)))
            .show(ctx, |ui| {
                self.render_main_editor(ctx, ui);
            });

        // Render Close Tab Confirmation Modal as a robust foreground overlay
        if let Some(close_id) = self.confirm_close_id.clone() {
            let note_title = self
                .data
                .notes
                .iter()
                .find(|n| n.id == close_id)
                .map(|n| {
                    if n.title.trim().is_empty() {
                        "untitled.txt".to_string()
                    } else {
                        n.title.clone()
                    }
                })
                .unwrap_or_else(|| "note".to_string());

            let palette = theme::get_palette(self.data.settings.theme_mode);

            egui::Area::new(egui::Id::new("confirm_close_modal"))
                .order(egui::Order::Foreground)
                .fixed_pos(egui::Pos2::ZERO)
                .show(ctx, |ui| {
                    let screen_rect = ctx.screen_rect();

                    // Dimmed backdrop mask
                    ui.painter().rect_filled(
                        screen_rect,
                        Rounding::ZERO,
                        Color32::from_black_alpha(170),
                    );

                    let modal_size = egui::vec2(340.0, 140.0);
                    let modal_rect = egui::Rect::from_center_size(screen_rect.center(), modal_size);

                    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(modal_rect));

                    theme::glass_card_frame(
                        self.data.settings.opacity,
                        self.data.settings.theme_mode,
                    )
                    .show(&mut child_ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.spacing_mut().item_spacing.y = 10.0;

                            ui.label(
                                RichText::new("⚠️ Close Note Tab?")
                                    .font(FontId::proportional(15.0))
                                    .strong()
                                    .color(theme::ACCENT_AMBER),
                            );

                            ui.label(
                                RichText::new(format!(
                                    "Are you sure you want to close '{}'?",
                                    note_title
                                ))
                                .font(FontId::proportional(13.0))
                                .color(Color32::from_gray(220)),
                            );

                            ui.add_space(6.0);

                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 12.0;

                                let cancel_btn = ui.add(
                                    egui::Button::new(
                                        RichText::new("Cancel (Esc)")
                                            .font(FontId::proportional(12.5))
                                            .color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_rgba_unmultiplied(
                                        palette.card.r(),
                                        palette.card.g(),
                                        palette.card.b(),
                                        220,
                                    ))
                                    .stroke(Stroke::new(1.0_f32, Color32::from_gray(120)))
                                    .rounding(Rounding::same(8.0))
                                    .min_size(egui::vec2(100.0, 32.0)),
                                );

                                if cancel_btn.clicked()
                                    || ctx.input(|i| i.key_pressed(egui::Key::Escape))
                                {
                                    self.confirm_close_id = None;
                                    ctx.request_repaint();
                                }

                                let confirm_btn = ui.add(
                                    egui::Button::new(
                                        RichText::new("🗑 Close Tab")
                                            .font(FontId::proportional(12.5))
                                            .strong()
                                            .color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_rgba_unmultiplied(180, 45, 60, 240))
                                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(239, 68, 68)))
                                    .rounding(Rounding::same(8.0))
                                    .min_size(egui::vec2(110.0, 32.0)),
                                );

                                if confirm_btn.clicked()
                                    || ctx.input(|i| i.key_pressed(egui::Key::Enter))
                                {
                                    self.close_note(&close_id);
                                    self.confirm_close_id = None;
                                    ctx.request_repaint();
                                }
                            });
                        });
                    });
                });
        }

        // Render drag-and-drop hover overlay if files are being dragged over window
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            egui::Area::new(egui::Id::new("drop_overlay"))
                .order(egui::Order::Foreground)
                .fixed_pos(egui::Pos2::ZERO)
                .show(ctx, |ui| {
                    let rect = ctx.screen_rect();
                    ui.allocate_rect(rect, egui::Sense::hover());
                    ui.painter().rect_filled(
                        rect,
                        Rounding::same(12.0),
                        Color32::from_rgba_unmultiplied(18, 12, 28, 220),
                    );
                    ui.painter().rect_stroke(
                        rect.shrink(8.0),
                        Rounding::same(10.0),
                        Stroke::new(2.0_f32, ACCENT_PURPLE),
                    );
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "📥 Drop text file to open as a new tab",
                        FontId::proportional(18.0),
                        Color32::WHITE,
                    );
                });
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if self.is_dirty {
            let _ = self.data.save();
        }
    }
}
