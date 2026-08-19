//! Collapsible folder explorer sidebar, directory scanning, and recursive tree rendering.

use crate::theme::Palette;
use eframe::egui::{self, Color32, CornerRadius, FontId, Margin, RichText, Stroke, Ui};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Maximum directory depth to scan recursively to ensure high frame rates.
pub const MAX_SCAN_DEPTH: usize = 6;

/// Default folder sidebar width in pixels.
pub const DEFAULT_SIDEBAR_WIDTH: f32 = 210.0;
pub const MIN_SIDEBAR_WIDTH: f32 = 140.0;
pub const MAX_SIDEBAR_WIDTH: f32 = 360.0;

/// Directory names ignored during scanning to prevent UI stutter and excess disk I/O.
pub const IGNORED_DIRECTORY_NAMES: &[&str] = &[
    "target",
    "node_modules",
    "__pycache__",
    ".git",
    ".hg",
    ".svn",
    ".cache",
    "dist",
    "build",
];

/// Represents a file or folder node in the directory tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<FolderNode>,
    pub extension: Option<String>,
}

impl FolderNode {
    /// Recursively scans filesystem path to construct FolderNode hierarchy with default depth limit.
    pub fn scan_path(path: &Path, depth: usize) -> Option<Self> {
        Self::scan_path_with_depth(path, depth, MAX_SCAN_DEPTH)
    }

    /// Recursively scans filesystem path to construct FolderNode hierarchy with configurable depth limit.
    pub fn scan_path_with_depth(path: &Path, depth: usize, max_depth: usize) -> Option<Self> {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        if path.is_file() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase());
            return Some(Self {
                name,
                path: path.to_path_buf(),
                is_dir: false,
                children: Vec::new(),
                extension: ext,
            });
        }

        if !path.is_dir() {
            return None;
        }

        let mut children = Vec::new();

        if depth < max_depth
            && let Ok(entries) = fs::read_dir(path)
        {
            let mut dir_entries = Vec::new();
            for entry in entries.flatten() {
                let entry_path = entry.path();
                let file_name = entry.file_name().to_string_lossy().to_string();

                // Skip heavy or hidden directories by default to preserve responsiveness
                if file_name.starts_with('.')
                    || IGNORED_DIRECTORY_NAMES.contains(&file_name.as_str())
                {
                    continue;
                }

                if let Some(child_node) =
                    Self::scan_path_with_depth(&entry_path, depth + 1, max_depth)
                {
                    dir_entries.push(child_node);
                }
            }

            // Sort: Directories first alphabetically, then files alphabetically
            dir_entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            });

            children = dir_entries;
        }

        Some(Self {
            name,
            path: path.to_path_buf(),
            is_dir: true,
            children,
            extension: None,
        })
    }

    /// Returns true if this node or any child matches the filter query.
    pub fn matches_filter(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        if self.name.to_lowercase().contains(query) {
            return true;
        }
        if self.is_dir {
            return self.children.iter().any(|c| c.matches_filter(query));
        }
        false
    }
}

/// Active folder workspace state containing directory tree and UI expansion tracking.
#[derive(Debug, Clone, PartialEq)]
pub struct FolderWorkspace {
    /// Absolute root path of the opened folder.
    pub root_path: PathBuf,

    /// Display name of the root folder.
    pub root_name: String,

    /// Cached root directory node.
    pub root_node: FolderNode,

    /// Set of currently expanded directory paths.
    pub expanded_dirs: HashSet<PathBuf>,

    /// Live query string to filter directory tree files.
    pub search_filter: String,

    /// User adjustable sidebar width.
    pub sidebar_width: f32,
}

impl FolderWorkspace {
    /// Opens a folder path, scans its directory tree with default depth, and initializes workspace state.
    pub fn open(path: &Path) -> Option<Self> {
        Self::open_with_depth(path, MAX_SCAN_DEPTH)
    }

    /// Opens a folder path with configurable scan depth.
    pub fn open_with_depth(path: &Path, max_depth: usize) -> Option<Self> {
        let canonical = fs::canonicalize(path)
            .ok()
            .unwrap_or_else(|| path.to_path_buf());
        if !canonical.is_dir() {
            return None;
        }

        let root_name = canonical
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| canonical.to_string_lossy().to_string());

        let root_node = FolderNode::scan_path_with_depth(&canonical, 0, max_depth)?;

        let mut expanded_dirs = HashSet::new();
        expanded_dirs.insert(canonical.clone());

        Some(Self {
            root_path: canonical,
            root_name,
            root_node,
            expanded_dirs,
            search_filter: String::new(),
            sidebar_width: DEFAULT_SIDEBAR_WIDTH,
        })
    }

    /// Rescans directory tree from disk to refresh newly created/deleted files.
    pub fn refresh(&mut self) {
        self.refresh_with_depth(MAX_SCAN_DEPTH);
    }

    /// Rescans directory tree from disk with configurable depth.
    pub fn refresh_with_depth(&mut self, max_depth: usize) {
        if let Some(new_root) = FolderNode::scan_path_with_depth(&self.root_path, 0, max_depth) {
            self.root_node = new_root;
        }
    }

    /// Toggles expanded state for a directory path.
    pub fn toggle_dir(&mut self, path: &Path) {
        if self.expanded_dirs.contains(path) {
            self.expanded_dirs.remove(path);
        } else {
            self.expanded_dirs.insert(path.to_path_buf());
        }
    }

    /// Expands all scanned directories in the workspace.
    pub fn expand_all(&mut self) {
        fn collect_dirs(node: &FolderNode, dirs: &mut HashSet<PathBuf>) {
            if node.is_dir {
                dirs.insert(node.path.clone());
                for child in &node.children {
                    collect_dirs(child, dirs);
                }
            }
        }
        collect_dirs(&self.root_node, &mut self.expanded_dirs);
    }

    /// Collapses all directories except the root.
    pub fn collapse_all(&mut self) {
        self.expanded_dirs.clear();
        self.expanded_dirs.insert(self.root_path.clone());
    }
}

/// File extension icon helper with accurate visual iconography and syntax colors.
pub fn get_file_icon(ext: Option<&str>) -> (&'static str, Color32) {
    match ext {
        Some("rs") => ("🦀", Color32::from_rgb(235, 102, 63)),
        Some("py") => ("🐍", Color32::from_rgb(56, 189, 248)),
        Some("sh" | "bash" | "zsh") => ("📜", Color32::from_rgb(74, 222, 128)),
        Some("md" | "markdown" | "qn" | "qnote") => ("📝", Color32::from_rgb(45, 212, 191)),
        Some("toml" | "json" | "yaml" | "yml" | "lock") => ("⚙", Color32::from_rgb(251, 191, 36)),
        Some("js" | "ts" | "jsx" | "tsx") => ("🌐", Color32::from_rgb(96, 165, 250)),
        Some("html" | "css" | "scss") => ("🎨", Color32::from_rgb(244, 114, 182)),
        Some("png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "svg") => {
            ("🖼", Color32::from_rgb(167, 139, 250))
        }
        _ => ("📄", Color32::from_gray(180)),
    }
}

/// Action produced by clicking items in the folder tree.
pub enum FolderTreeAction {
    OpenFile(PathBuf),
    CloseWorkspace,
}

/// Renders the folder explorer sidebar.
pub fn render_folder_sidebar(
    workspace: &mut FolderWorkspace,
    active_file_path: Option<&str>,
    palette: &Palette,
    ui: &mut Ui,
    height: f32,
    ui_font_size: f32,
) -> Option<FolderTreeAction> {
    let mut triggered_action = None;

    let header_font_size = (ui_font_size + 0.5).clamp(11.0, 26.0);
    let btn_font_size = ui_font_size.clamp(10.0, 24.0);
    let filter_font_size = (ui_font_size - 0.5).clamp(10.0, 22.0);

    let frame_margin = Margin::symmetric(6, 6);
    let inner_height = (height - (frame_margin.top + frame_margin.bottom) as f32).max(20.0);

    egui::Frame::NONE
        .fill(Color32::from_rgba_unmultiplied(
            palette.bg.r(),
            palette.bg.g(),
            palette.bg.b(),
            200,
        ))
        .stroke(Stroke::new(1.0, Palette::with_alpha(palette.border, 70)))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(frame_margin)
        .show(ui, |ui| {
            ui.set_width(workspace.sidebar_width);
            ui.set_height(inner_height);
            ui.set_min_height(inner_height);
            ui.set_max_height(inner_height);

            ui.vertical(|ui| {
                // Top header of Folder Explorer
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.label(
                        RichText::new("📂")
                            .font(FontId::proportional(header_font_size + 1.0))
                            .color(Color32::from_rgb(245, 158, 11)),
                    );
                    ui.label(
                        RichText::new(&workspace.root_name)
                            .font(FontId::proportional(header_font_size))
                            .strong()
                            .color(Color32::WHITE),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Close Folder button
                        let btn_close = ui.add(
                            egui::Button::new(
                                RichText::new("×").font(FontId::proportional(btn_font_size + 2.0)),
                            )
                            .frame(false),
                        );
                        if btn_close.on_hover_text("Close folder workspace").clicked() {
                            triggered_action = Some(FolderTreeAction::CloseWorkspace);
                        }

                        // Collapse All button
                        let btn_collapse = ui.add(
                            egui::Button::new(
                                RichText::new("⇲").font(FontId::proportional(btn_font_size)),
                            )
                            .frame(false),
                        );
                        if btn_collapse.on_hover_text("Collapse all folders").clicked() {
                            workspace.collapse_all();
                        }

                        // Refresh button
                        let btn_refresh = ui.add(
                            egui::Button::new(
                                RichText::new("🔄").font(FontId::proportional(btn_font_size)),
                            )
                            .frame(false),
                        );
                        if btn_refresh.on_hover_text("Refresh folder tree").clicked() {
                            workspace.refresh();
                        }
                    });
                });

                ui.add_space(4.0);

                // Quick filter input
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("🔍")
                            .font(FontId::proportional(filter_font_size - 1.0))
                            .color(Color32::from_gray(140)),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut workspace.search_filter)
                            .hint_text("Filter files...")
                            .font(FontId::proportional(filter_font_size))
                            .desired_width(ui.available_width()),
                    );
                });

                ui.add_space(4.0);
                crate::ui::draw_horizontal_divider(ui);
                ui.add_space(4.0);

                // Recursive file tree scroll area with explicit height bounds
                let scroll_height = ui.available_height().max(80.0);
                egui::ScrollArea::vertical()
                    .id_salt("folder_tree_scroll")
                    .auto_shrink([false, false])
                    .max_height(scroll_height)
                    .min_scrolled_height(scroll_height)
                    .show(ui, |ui| {
                        let query = workspace.search_filter.trim().to_lowercase();
                        let mut toggled_path = None;
                        let mut opened_file = None;

                        render_node_children(
                            &workspace.root_node.children,
                            &workspace.expanded_dirs,
                            &query,
                            active_file_path,
                            0,
                            palette,
                            ui,
                            ui_font_size,
                            &mut toggled_path,
                            &mut opened_file,
                        );

                        if let Some(path) = toggled_path {
                            workspace.toggle_dir(&path);
                        }
                        if let Some(file_path) = opened_file {
                            triggered_action = Some(FolderTreeAction::OpenFile(file_path));
                        }
                    });
            });
        });

    triggered_action
}

/// Recursively renders children of a directory node with full-row interactive responsiveness.
#[allow(clippy::too_many_arguments)]
fn render_node_children(
    nodes: &[FolderNode],
    expanded_dirs: &HashSet<PathBuf>,
    filter_query: &str,
    active_file_path: Option<&str>,
    depth: usize,
    palette: &Palette,
    ui: &mut Ui,
    ui_font_size: f32,
    toggled_path: &mut Option<PathBuf>,
    opened_file: &mut Option<PathBuf>,
) {
    let item_font_size = ui_font_size.clamp(11.0, 24.0);
    let icon_font_size = (ui_font_size + 1.0).clamp(12.0, 26.0);
    let arrow_font_size = (ui_font_size - 0.5).clamp(10.0, 22.0);
    let item_height = (ui_font_size * 1.85).clamp(24.0, 52.0);
    let indent_step = (ui_font_size * 1.15).clamp(14.0, 32.0);
    let icon_offset = (ui_font_size * 1.0).clamp(12.0, 28.0);
    let text_offset = (ui_font_size * 2.35).clamp(28.0, 60.0);

    let indent = (depth as f32) * indent_step;

    for node in nodes {
        if !node.matches_filter(filter_query) {
            continue;
        }

        let is_dir = node.is_dir;
        let is_expanded =
            is_dir && (expanded_dirs.contains(&node.path) || !filter_query.is_empty());
        let node_path_str = node.path.to_string_lossy();
        let is_active = !is_dir && active_file_path == Some(&*node_path_str);

        let row_size = egui::vec2(ui.available_width(), item_height);
        let (row_rect, row_resp) = ui.allocate_exact_size(row_size, egui::Sense::click());

        // Background hover / active styling
        if is_active {
            ui.painter().rect_filled(
                row_rect,
                CornerRadius::same(4),
                Palette::with_alpha(palette.accent, 45),
            );
            ui.painter().rect_stroke(
                row_rect,
                CornerRadius::same(4),
                Stroke::new(1.0, Palette::with_alpha(palette.accent, 140)),
                egui::StrokeKind::Inside,
            );
        } else if row_resp.hovered() {
            ui.painter().rect_filled(
                row_rect,
                CornerRadius::same(4),
                Palette::with_alpha(palette.accent, 25),
            );
        }

        // Draw guideline ticks for depth > 0
        if depth > 0 {
            for d in 0..depth {
                let guide_x = row_rect.min.x + (d as f32 * indent_step) + (indent_step * 0.5);
                ui.painter().line_segment(
                    [
                        egui::pos2(guide_x, row_rect.min.y),
                        egui::pos2(guide_x, row_rect.max.y),
                    ],
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 20)),
                );
            }
        }

        // Render contents directly via painter (prevents accidental text selection from blocking folder clicks)
        let x_start = row_rect.min.x + indent + 4.0;
        let y_center = row_rect.center().y;

        if is_dir {
            let arrow = if is_expanded { "▾" } else { "▸" };
            let folder_icon = if is_expanded { "📂" } else { "📁" };

            let arrow_color = if is_expanded {
                palette.accent
            } else {
                Color32::from_gray(150)
            };

            ui.painter().text(
                egui::pos2(x_start, y_center),
                egui::Align2::LEFT_CENTER,
                arrow,
                FontId::proportional(arrow_font_size),
                arrow_color,
            );

            ui.painter().text(
                egui::pos2(x_start + icon_offset, y_center),
                egui::Align2::LEFT_CENTER,
                folder_icon,
                FontId::proportional(icon_font_size),
                Color32::from_rgb(245, 158, 11),
            );

            let name_color = if is_expanded {
                palette.accent
            } else if row_resp.hovered() {
                Color32::WHITE
            } else {
                Color32::from_gray(210)
            };

            ui.painter().text(
                egui::pos2(x_start + text_offset, y_center),
                egui::Align2::LEFT_CENTER,
                &node.name,
                FontId::proportional(item_font_size),
                name_color,
            );
        } else {
            let (icon, icon_color) = get_file_icon(node.extension.as_deref());

            let final_icon_color = if is_active {
                palette.accent
            } else {
                icon_color
            };

            ui.painter().text(
                egui::pos2(x_start + icon_offset, y_center),
                egui::Align2::LEFT_CENTER,
                icon,
                FontId::proportional(icon_font_size),
                final_icon_color,
            );

            let name_color = if is_active {
                palette.accent
            } else if row_resp.hovered() {
                Color32::WHITE
            } else {
                Color32::from_gray(190)
            };

            ui.painter().text(
                egui::pos2(x_start + text_offset, y_center),
                egui::Align2::LEFT_CENTER,
                &node.name,
                FontId::proportional(item_font_size),
                name_color,
            );
        }

        if is_dir && (row_resp.clicked() || row_resp.double_clicked()) {
            *toggled_path = Some(node.path.clone());
        } else if !is_dir && (row_resp.clicked() || row_resp.double_clicked()) {
            *opened_file = Some(node.path.clone());
        }

        // RMB (Right Mouse Button) Context Menu for direct file/folder operations
        row_resp.context_menu(|ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            if !is_dir {
                if ui.button("📥 Open File").clicked() {
                    *opened_file = Some(node.path.clone());
                    ui.close();
                }
                ui.separator();
                if ui.button("📋 Copy Full Path").clicked() {
                    ui.ctx().copy_text(node.path.to_string_lossy().to_string());
                    ui.close();
                }
                if ui.button("📋 Copy File Name").clicked() {
                    ui.ctx().copy_text(node.name.clone());
                    ui.close();
                }
                if ui.button("📂 Reveal in File Manager").clicked() {
                    if let Some(parent) = node.path.parent() {
                        crate::ui::drag_drop::safe_open_folder(parent);
                    }
                    ui.close();
                }
            } else {
                let toggle_label = if is_expanded {
                    "📁 Collapse Folder"
                } else {
                    "📂 Expand Folder"
                };
                if ui.button(toggle_label).clicked() {
                    *toggled_path = Some(node.path.clone());
                    ui.close();
                }
                ui.separator();
                if ui.button("📋 Copy Folder Path").clicked() {
                    ui.ctx().copy_text(node.path.to_string_lossy().to_string());
                    ui.close();
                }
                if ui.button("📂 Reveal Folder in File Manager").clicked() {
                    crate::ui::drag_drop::safe_open_folder(&node.path);
                    ui.close();
                }
            }
        });

        if !is_dir {
            row_resp.on_hover_text(format!(
                "Click or Right-click to open:\n{}",
                node.path.display()
            ));
        }

        // If directory is expanded, render its children
        if is_dir && is_expanded {
            render_node_children(
                &node.children,
                expanded_dirs,
                filter_query,
                active_file_path,
                depth + 1,
                palette,
                ui,
                ui_font_size,
                toggled_path,
                opened_file,
            );
        }
    }
}
