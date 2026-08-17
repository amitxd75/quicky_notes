//! UI views, layouts, headers, status bars, and integrated glass drawers.

pub mod ai_modal;
pub mod context_menu;
pub mod drag_drop;
pub mod editor;
pub mod header;
pub mod image_view;
pub mod markdown;

pub mod options_drawer;
pub mod search_drawer;
pub mod shortcuts;
pub mod syntax;
pub mod toast;

pub use crate::components::divider::horizontal_divider as draw_horizontal_divider;
