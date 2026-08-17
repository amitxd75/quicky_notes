//! Theme and palette inspection and override API for plugins.

use crate::plugins::types::ThemeMutation;
use crate::theme::Palette;
use std::sync::{Arc, Mutex};

/// Internal state for ThemeHandle snapshot and mutations.
#[derive(Debug, Clone, Default)]
pub struct ThemeHandleInner {
    pub accent_hex: String,
    pub bg_hex: String,
    pub is_dark: bool,
    pub mutations: Vec<ThemeMutation>,
}

/// Theme handle proxy object exposed to Rhai scripts as `theme`.
#[derive(Debug, Clone, Default)]
pub struct ThemeHandle {
    inner: Arc<Mutex<ThemeHandleInner>>,
}

impl ThemeHandle {
    /// Constructs a ThemeHandle from the active Palette.
    pub fn from_palette(palette: &Palette) -> Self {
        let accent_hex = format!(
            "#{:02X}{:02X}{:02X}",
            palette.accent.r(),
            palette.accent.g(),
            palette.accent.b()
        );
        let bg_hex = format!(
            "#{:02X}{:02X}{:02X}",
            palette.bg.r(),
            palette.bg.g(),
            palette.bg.b()
        );

        // Approximate luminance to determine if dark mode
        let lum = (palette.bg.r() as f32 * 0.299
            + palette.bg.g() as f32 * 0.587
            + palette.bg.b() as f32 * 0.114)
            / 255.0;
        let is_dark = lum < 0.5;

        Self {
            inner: Arc::new(Mutex::new(ThemeHandleInner {
                accent_hex,
                bg_hex,
                is_dark,
                mutations: Vec::new(),
            })),
        }
    }

    /// Returns the current accent color as a hex string (e.g. "#7AA2F7").
    pub fn get_accent(&mut self) -> String {
        self.inner
            .lock()
            .map(|i| i.accent_hex.clone())
            .unwrap_or_default()
    }

    /// Returns the current background color as a hex string.
    pub fn get_bg(&mut self) -> String {
        self.inner
            .lock()
            .map(|i| i.bg_hex.clone())
            .unwrap_or_default()
    }

    /// Sets a temporary accent color override.
    pub fn set_accent(&mut self, hex: String) {
        if let Ok(mut inner) = self.inner.lock() {
            let trimmed = hex.trim().to_string();
            inner.accent_hex = trimmed.clone();
            inner.mutations.push(ThemeMutation::SetAccent(trimmed));
        }
    }

    /// Resets the accent color back to the active theme palette.
    pub fn reset_accent(&mut self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.mutations.push(ThemeMutation::ResetAccent);
        }
    }

    /// Returns true if the active theme is a dark theme.
    pub fn is_dark(&mut self) -> bool {
        self.inner.lock().map(|i| i.is_dark).unwrap_or(true)
    }

    /// Takes queued theme mutations.
    pub fn take_mutations(&self) -> Vec<ThemeMutation> {
        self.inner
            .lock()
            .map(|mut i| std::mem::take(&mut i.mutations))
            .unwrap_or_default()
    }
}
