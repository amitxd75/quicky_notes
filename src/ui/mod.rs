//! UI views, layouts, headers, status bars, and integrated glass drawers.

pub mod ai_modal;
pub mod context_menu;
pub mod drag_drop;
pub mod editor;
pub mod folder_tree;
pub mod header;
pub mod image_view;
pub mod markdown;

pub mod options;
pub use options as options_drawer;
pub mod search_drawer;
pub mod shortcuts;
pub mod syntax;
pub mod toast;

pub use crate::components::divider::horizontal_divider as draw_horizontal_divider;
use eframe::egui::{self, ViewportCommand};

/// Handles window border and corner hover cursors and native drag resizing for undecorated windows.
pub fn handle_window_resizing(ctx: &egui::Context) {
    let screen = match ctx.input(|i| i.raw.screen_rect) {
        Some(s) => s,
        None => return,
    };
    let border = 7.0_f32;
    let corner = 16.0_f32;

    if let Some(pos) = ctx.input(|i| i.pointer.latest_pos()) {
        if !screen.contains(pos) {
            return;
        }

        let is_left = pos.x <= screen.min.x + border;
        let is_right = pos.x >= screen.max.x - border;
        let is_top = pos.y <= screen.min.y + border;
        let is_bottom = pos.y >= screen.max.y - border;

        let is_corner_left = pos.x <= screen.min.x + corner;
        let is_corner_right = pos.x >= screen.max.x - corner;
        let is_corner_top = pos.y <= screen.min.y + corner;
        let is_corner_bottom = pos.y >= screen.max.y - corner;

        let resize_dir = if is_corner_right && is_corner_bottom {
            Some((
                egui::viewport::ResizeDirection::SouthEast,
                egui::CursorIcon::ResizeNwSe,
            ))
        } else if is_corner_left && is_corner_bottom {
            Some((
                egui::viewport::ResizeDirection::SouthWest,
                egui::CursorIcon::ResizeNeSw,
            ))
        } else if is_corner_right && is_corner_top {
            Some((
                egui::viewport::ResizeDirection::NorthEast,
                egui::CursorIcon::ResizeNeSw,
            ))
        } else if is_corner_left && is_corner_top {
            Some((
                egui::viewport::ResizeDirection::NorthWest,
                egui::CursorIcon::ResizeNwSe,
            ))
        } else if is_bottom {
            Some((
                egui::viewport::ResizeDirection::South,
                egui::CursorIcon::ResizeVertical,
            ))
        } else if is_top {
            Some((
                egui::viewport::ResizeDirection::North,
                egui::CursorIcon::ResizeVertical,
            ))
        } else if is_right {
            Some((
                egui::viewport::ResizeDirection::East,
                egui::CursorIcon::ResizeHorizontal,
            ))
        } else if is_left {
            Some((
                egui::viewport::ResizeDirection::West,
                egui::CursorIcon::ResizeHorizontal,
            ))
        } else {
            None
        };

        if let Some((direction, cursor)) = resize_dir {
            ctx.set_cursor_icon(cursor);
            if ctx.input(|i| i.pointer.any_pressed()) {
                ctx.send_viewport_cmd(ViewportCommand::BeginResize(direction));
            }
        }
    }
}
