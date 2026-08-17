//! Intelligence and autocomplete engines.

pub mod ai;
pub mod suggest;

pub use ai::{
    AiAction, AiProvider, AiRequest, AiResult, AiSettings, GeneratedTheme, parse_hex_color,
    spawn_ai_request, spawn_generate_theme_request,
};
pub use suggest::SuggestionEngine;
