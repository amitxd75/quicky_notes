//! UI feedback and clipboard interaction API exposed to Rhai scripts.

use crate::app::ToastKind;
use std::sync::{Arc, Mutex};

/// Internal UI side effects buffer.
#[derive(Debug, Clone, Default)]
pub struct UiHandleInner {
    pub toasts: Vec<(String, ToastKind)>,
    pub status_msg: Option<String>,
    pub copy_clipboard: Option<String>,
    pub request_repaint: bool,
}

/// UI proxy object collecting toast notifications and clipboard side effects during script execution.
#[derive(Debug, Clone, Default)]
pub struct UiHandle {
    inner: Arc<Mutex<UiHandleInner>>,
}

impl UiHandle {
    /// Creates a new empty UI handle.
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues an informational toast notification.
    pub fn toast(&mut self, message: String) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.toasts.push((message, ToastKind::Info));
        }
    }

    /// Queues a success toast notification.
    pub fn toast_success(&mut self, message: String) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.toasts.push((message, ToastKind::Success));
        }
    }

    /// Queues an error toast notification.
    pub fn toast_error(&mut self, message: String) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.toasts.push((message, ToastKind::Error));
        }
    }

    /// Queues a warning toast notification.
    pub fn toast_warning(&mut self, message: String) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.toasts.push((message, ToastKind::Warning));
        }
    }

    /// Sets the status bar text.
    pub fn set_status(&mut self, message: String) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.status_msg = Some(message);
        }
    }

    /// Copies text to the system clipboard.
    pub fn copy_to_clipboard(&mut self, text: String) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.copy_clipboard = Some(text);
        }
    }

    /// Reads text from the system clipboard.
    pub fn get_clipboard(&mut self) -> String {
        crate::ui::context_menu::get_clipboard_text()
    }

    /// Requests an egui frame redraw.
    pub fn request_repaint(&mut self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.request_repaint = true;
        }
    }

    /// Takes all queued toast messages.
    pub fn take_toasts(&self) -> Vec<(String, ToastKind)> {
        self.inner
            .lock()
            .map(|mut i| std::mem::take(&mut i.toasts))
            .unwrap_or_default()
    }

    /// Takes the status message if set.
    pub fn take_status_msg(&self) -> Option<String> {
        self.inner
            .lock()
            .map(|mut i| i.status_msg.take())
            .unwrap_or_default()
    }

    /// Takes the copy clipboard string if set.
    pub fn take_copy_clipboard(&self) -> Option<String> {
        self.inner
            .lock()
            .map(|mut i| i.copy_clipboard.take())
            .unwrap_or_default()
    }

    /// Returns true if repaint was requested.
    pub fn is_repaint_requested(&self) -> bool {
        self.inner
            .lock()
            .map(|i| i.request_repaint)
            .unwrap_or(false)
    }
}
