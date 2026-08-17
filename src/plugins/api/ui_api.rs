use crate::app::ToastKind;
use crate::plugins::types::PanelAction;
use std::sync::{Arc, Mutex};

/// Internal UI side effects buffer.
#[derive(Debug, Clone, Default)]
pub struct UiHandleInner {
    pub toasts: Vec<(String, ToastKind)>,
    pub status_msg: Option<String>,
    pub copy_clipboard: Option<String>,
    pub panel_actions: Vec<PanelAction>,
    pub request_repaint: bool,
}

/// UI proxy object collecting toast notifications, clipboard, and panel side effects.
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

    /// Opens the bottom output console panel with a title and content.
    pub fn show_panel(&mut self, title: String, content: String) {
        if let Ok(mut inner) = self.inner.lock() {
            inner
                .panel_actions
                .push(PanelAction::Show { title, content });
        }
    }

    /// Appends new text to the bottom output console panel.
    pub fn append_panel(&mut self, text: String) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.panel_actions.push(PanelAction::Append(text));
        }
    }

    /// Replaces the content in the bottom output console panel.
    pub fn set_panel_content(&mut self, content: String) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.panel_actions.push(PanelAction::SetContent(content));
        }
    }

    /// Clears text in the bottom output console panel.
    pub fn clear_panel(&mut self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.panel_actions.push(PanelAction::Clear);
        }
    }

    /// Closes and hides the bottom output console panel.
    pub fn hide_panel(&mut self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.panel_actions.push(PanelAction::Hide);
        }
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

    /// Takes all queued panel control actions.
    pub fn take_panel_actions(&self) -> Vec<PanelAction> {
        self.inner
            .lock()
            .map(|mut i| std::mem::take(&mut i.panel_actions))
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
