//! Individual note model, text statistic utilities, invariant validation, and .qn binary format.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

static NOTE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generates a globally unique note ID combining nanosecond timestamps with a monotonic atomic counter.
pub fn generate_unique_note_id() -> String {
    let count = NOTE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = chrono::Local::now().timestamp_nanos_opt().unwrap_or(0);
    format!("note-{}-{}", nanos, count)
}

/// Maximum title length to prevent UI overflow or excessive allocation.
pub const MAX_NOTE_TITLE_LEN: usize = 128;

/// Default fallback title for unnamed notes.
pub const DEFAULT_NOTE_TITLE: &str = "untitled.qn";

/// Standard ISO-like timestamp format used for note timestamps.
pub const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// Magic byte signature for Quicky Notes binary bundle format (v1).
pub const QN_MAGIC: &[u8; 6] = b"QNOTE\x01";

/// Header overhead size for .qn binary containers (6 bytes magic + 4 bytes metadata length).
pub const QN_HEADER_OVERHEAD: usize = 10;

/// Maximum allowable .qn binary file size (25 MB) to prevent memory exhaustion.
pub const MAX_QN_FILE_SIZE: usize = 25 * 1024 * 1024;

/// Maximum allowable size for a single attachment (10 MB).
pub const MAX_ATTACHMENT_SIZE: usize = 10 * 1024 * 1024;

/// Maximum cumulative size for all attachments in a single note (50 MB).
pub const MAX_TOTAL_ATTACHMENTS_SIZE: usize = 50 * 1024 * 1024;

/// Maximum number of attachments permitted per note.
pub const MAX_ATTACHMENTS_PER_NOTE: usize = 50;

/// Embedded raw image attachment inside a note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteAttachment {
    /// Unique identifier within this note (e.g. `img_1`).
    pub id: String,

    /// Display or original filename (e.g. `screenshot.png`).
    pub name: String,

    /// MIME type (e.g. `image/png`, `image/jpeg`, `image/webp`).
    pub mime_type: String,

    /// Raw binary payload (PNG, JPEG, WebP, GIF, BMP).
    pub data: Vec<u8>,
}

impl NoteAttachment {
    /// Detects MIME type from filename extension.
    pub fn detect_mime(name: &str) -> &'static str {
        let lower = name.to_lowercase();
        if lower.ends_with(".png") {
            "image/png"
        } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
            "image/jpeg"
        } else if lower.ends_with(".webp") {
            "image/webp"
        } else if lower.ends_with(".gif") {
            "image/gif"
        } else if lower.ends_with(".bmp") {
            "image/bmp"
        } else if lower.ends_with(".svg") {
            "image/svg+xml"
        } else {
            "application/octet-stream"
        }
    }

    /// Formats data size into a human-readable string (e.g. `142 KB`).
    pub fn formatted_size(&self) -> String {
        let bytes = self.data.len();
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
        }
    }
}

/// Metadata descriptor for an embedded image in the .qn header.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct QnImageDescriptor {
    id: String,
    name: String,
    mime_type: String,
    offset: u32,
    length: u32,
}

/// Metadata JSON payload stored in the .qn binary container.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct QnMetaHeader {
    version: u32,
    id: String,
    title: String,
    content: String,
    created_at: String,
    updated_at: String,
    pinned: bool,
    color_tag: Option<String>,
    images: Vec<QnImageDescriptor>,
}

/// Structured error for .qn binary decoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QnDecodeError {
    TooShort,
    InvalidMagic,
    CorruptMetadata(String),
    TruncatedPayload,
    ExceedsLimit(String),
}

impl fmt::Display for QnDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort => write!(f, "File is too short to be a valid .qn container"),
            Self::InvalidMagic => write!(f, "Invalid magic signature in .qn header"),
            Self::CorruptMetadata(msg) => write!(f, "Corrupt .qn metadata: {}", msg),
            Self::TruncatedPayload => write!(f, "Truncated image payload in .qn file"),
            Self::ExceedsLimit(msg) => write!(f, "Payload exceeds size limit: {}", msg),
        }
    }
}

impl std::error::Error for QnDecodeError {}

/// Represents an individual note tab in Quicky Notes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    /// Unique identifier for the note (must not be empty).
    pub id: String,

    /// Display title of the note tab (e.g. `ideas.qn`).
    pub title: String,

    /// Text contents of the note editor.
    pub content: String,

    /// Formatted timestamp string of note creation (`YYYY-MM-DD HH:MM:SS`).
    pub created_at: String,

    /// Formatted timestamp string of last modification (`YYYY-MM-DD HH:MM:SS`).
    pub updated_at: String,

    /// Whether this note tab is pinned to the front of the tab bar.
    pub pinned: bool,

    /// Optional color tag hex string for categorization.
    pub color_tag: Option<String>,

    /// Optional filesystem path if this note is linked directly to a file on disk.
    #[serde(default)]
    pub file_path: Option<String>,

    /// Timestamp of the linked disk file when last read or written.
    #[serde(skip)]
    pub last_disk_mtime: Option<std::time::SystemTime>,

    /// Unsaved modifications in the local editor buffer.
    #[serde(skip)]
    pub is_dirty: bool,

    /// Flag indicating external disk changes occurred while in-memory edits were unsaved.
    #[serde(skip)]
    pub has_disk_conflict: bool,

    /// Embedded image attachments bundled inside this note.
    #[serde(default)]
    pub attachments: Vec<NoteAttachment>,
}

impl Note {
    /// Creates a new note with a validated ID and title.
    ///
    /// # Invariants
    /// - `id` is verified non-empty (empty IDs are replaced with timestamp-based fallbacks).
    /// - `title` is sanitized (trimmed, clamped in length, falling back to `DEFAULT_NOTE_TITLE` if blank).
    pub fn new(id: String, title: String) -> Self {
        let id = if id.trim().is_empty() {
            format!("note-{}", chrono::Local::now().timestamp_millis())
        } else {
            id
        };

        let sanitized_title = Self::sanitize_title(&title);
        let now = chrono::Local::now().format(TIMESTAMP_FORMAT).to_string();

        Self {
            id,
            title: sanitized_title,
            content: String::new(),
            created_at: now.clone(),
            updated_at: now,
            pinned: false,
            color_tag: None,
            file_path: None,
            last_disk_mtime: None,
            is_dirty: false,
            has_disk_conflict: false,
            attachments: Vec::new(),
        }
    }

    /// Sanitizes and clamps note title to default maximum length.
    pub fn sanitize_title(title: &str) -> String {
        Self::sanitize_title_with_len(title, MAX_NOTE_TITLE_LEN)
    }

    /// Sanitizes and clamps note title to a custom maximum length.
    pub fn sanitize_title_with_len(title: &str, max_len: usize) -> String {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            DEFAULT_NOTE_TITLE.to_string()
        } else if trimmed.len() > max_len {
            let mut end = max_len;
            while !trimmed.is_char_boundary(end) && end > 0 {
                end -= 1;
            }
            trimmed[..end].to_string()
        } else {
            trimmed.to_string()
        }
    }

    /// Validates and repairs internal invariants after deserialization.
    pub fn validate_and_repair(&mut self, fallback_id: &str) {
        if self.id.trim().is_empty() {
            self.id = fallback_id.to_string();
        }
        self.title = Self::sanitize_title(&self.title);

        if self.created_at.is_empty() {
            self.created_at = chrono::Local::now().format(TIMESTAMP_FORMAT).to_string();
        }
        if self.updated_at.is_empty() {
            self.updated_at = self.created_at.clone();
        }
    }

    /// Computes (word_count, char_count, line_count) in a single fast pass.
    pub fn compute_stats(&self) -> (usize, usize, usize) {
        let mut words = 0;
        let mut chars = 0;
        let mut lines = 1;
        let mut in_word = false;

        for c in self.content.chars() {
            chars += 1;
            if c == '\n' {
                lines += 1;
                in_word = false;
            } else if c.is_whitespace() {
                in_word = false;
            } else if !in_word {
                in_word = true;
                words += 1;
            }
        }

        (words, chars, lines)
    }

    /// Returns estimated word count of the content.
    #[inline]
    #[allow(dead_code)]
    pub fn word_count(&self) -> usize {
        self.compute_stats().0
    }

    /// Returns total character count of the content.
    #[inline]
    #[allow(dead_code)]
    pub fn char_count(&self) -> usize {
        self.compute_stats().1
    }

    /// Returns total number of lines (minimum 1).
    #[inline]
    #[allow(dead_code)]
    pub fn line_count(&self) -> usize {
        self.compute_stats().2
    }

    /// Updates the modification timestamp to current local time.
    pub fn update_timestamp(&mut self) {
        self.updated_at = chrono::Local::now().format(TIMESTAMP_FORMAT).to_string();
    }

    /// Formats updated timestamp safely for display in status and search lists.
    ///
    /// Extracts `HH:MM` from `YYYY-MM-DD HH:MM:SS` without risking UTF-8 panics.
    /// Safety: byte slicing is safe here because `chrono::Local::now().format("%Y-%m-%d %H:%M:%S")`
    /// produces exclusively ASCII characters. The `is_char_boundary` checks provide defense-in-depth.
    pub fn display_time(&self) -> String {
        let text = self.updated_at.trim();
        if text.len() >= 16 && text.is_char_boundary(11) && text.is_char_boundary(16) {
            text[11..16].to_string()
        } else {
            text.to_string()
        }
    }

    /// Returns whether this note supports Markdown preview and rendering.
    #[inline]
    pub fn is_markdown(&self) -> bool {
        let name = self.file_path.as_deref().unwrap_or(&self.title);
        let lower = name.to_lowercase();
        if lower.ends_with(".qn")
            || lower.ends_with(".qnote")
            || lower.ends_with(".md")
            || lower.ends_with(".markdown")
            || lower.ends_with(".txt")
        {
            return true;
        }
        if self.file_path.is_none() {
            return true;
        }
        !lower.ends_with(".rs")
            && !lower.ends_with(".py")
            && !lower.ends_with(".js")
            && !lower.ends_with(".ts")
            && !lower.ends_with(".json")
            && !lower.ends_with(".toml")
            && !lower.ends_with(".yaml")
            && !lower.ends_with(".yml")
            && !lower.ends_with(".cpp")
            && !lower.ends_with(".c")
            && !lower.ends_with(".h")
            && !lower.ends_with(".sh")
            && !lower.ends_with(".go")
            && !lower.ends_with(".java")
    }

    /// Returns whether this note is specifically a Quicky Notes bundled binary file (.qn or .qnote).
    #[inline]
    pub fn is_qn(&self) -> bool {
        let lower = self.title.to_lowercase();
        lower.ends_with(".qn") || lower.ends_with(".qnote")
    }

    /// Returns whether this note is linked to an external file on the local filesystem.
    #[inline]
    pub fn is_linked_file(&self) -> bool {
        self.file_path.is_some()
    }

    /// Returns the character count of the note's content.
    pub fn char_len(&self) -> usize {
        self.content.chars().count()
    }

    /// Safely extracts a substring using character offsets without UTF-8 boundary panics.
    pub fn char_slice(&self, start_char: usize, end_char: usize) -> String {
        let total_chars = self.char_len();
        let s = start_char.min(total_chars);
        let e = end_char.min(total_chars).max(s);
        self.content.chars().skip(s).take(e - s).collect()
    }

    /// Safely replaces a character range with new text without UTF-8 boundary panics.
    pub fn replace_char_range(&mut self, start_char: usize, end_char: usize, replacement: &str) {
        let total_chars = self.char_len();
        let s = start_char.min(total_chars);
        let e = end_char.min(total_chars).max(s);

        let before: String = self.content.chars().take(s).collect();
        let after: String = self.content.chars().skip(e).collect();

        let mut new_content = String::with_capacity(before.len() + replacement.len() + after.len());
        new_content.push_str(&before);
        new_content.push_str(replacement);
        new_content.push_str(&after);

        self.content = new_content;
        self.update_timestamp();
    }

    /// Safely deletes a character range.
    pub fn delete_char_range(&mut self, start_char: usize, end_char: usize) {
        self.replace_char_range(start_char, end_char, "");
    }

    /// Safely inserts text at a character offset.
    #[allow(dead_code)]
    pub fn insert_at_char(&mut self, char_pos: usize, text: &str) {
        self.replace_char_range(char_pos, char_pos, text);
    }

    // --- Image Attachment Operations ---

    /// Adds an image attachment to this note and returns its unique ID.
    pub fn add_attachment(&mut self, name: &str, mime_type: &str, data: Vec<u8>) -> String {
        let id = format!(
            "img_{}_{}",
            chrono::Local::now().timestamp_millis(),
            self.attachments.len() + 1
        );
        let attachment = NoteAttachment {
            id: id.clone(),
            name: name.to_string(),
            mime_type: if mime_type.is_empty() {
                NoteAttachment::detect_mime(name).to_string()
            } else {
                mime_type.to_string()
            },
            data,
        };
        self.attachments.push(attachment);
        self.update_timestamp();
        id
    }

    /// Removes an image attachment by ID and returns true if removed.
    pub fn remove_attachment(&mut self, id: &str) -> bool {
        if let Some(pos) = self.attachments.iter().position(|a| a.id == id) {
            self.attachments.remove(pos);
            self.update_timestamp();
            true
        } else {
            false
        }
    }

    /// Retrieves an image attachment by ID.
    pub fn get_attachment(&self, id: &str) -> Option<&NoteAttachment> {
        self.attachments.iter().find(|a| a.id == id)
    }

    /// Retrieves an image attachment by name or ID.
    pub fn get_attachment_by_name_or_id(&self, name_or_id: &str) -> Option<&NoteAttachment> {
        let trimmed = name_or_id.trim();
        self.attachments
            .iter()
            .find(|a| a.id == trimmed || a.name.eq_ignore_ascii_case(trimmed))
    }

    // --- .qn Binary Encoding & Decoding ---

    /// Encodes this note and all its embedded image attachments into the `.qn` binary format.
    pub fn encode_qn_binary(&self) -> Vec<u8> {
        let mut image_descriptors = Vec::with_capacity(self.attachments.len());
        let mut current_offset: u32 = 0;

        for att in &self.attachments {
            let length = att.data.len() as u32;
            image_descriptors.push(QnImageDescriptor {
                id: att.id.clone(),
                name: att.name.clone(),
                mime_type: att.mime_type.clone(),
                offset: current_offset,
                length,
            });
            current_offset += length;
        }

        let meta = QnMetaHeader {
            version: 1,
            id: self.id.clone(),
            title: self.title.clone(),
            content: self.content.clone(),
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone(),
            pinned: self.pinned,
            color_tag: self.color_tag.clone(),
            images: image_descriptors,
        };

        let meta_json = serde_json::to_vec(&meta).unwrap_or_default();
        let meta_len = meta_json.len() as u32;

        let total_capacity = 6 + 4 + meta_json.len() + current_offset as usize;
        let mut buffer = Vec::with_capacity(total_capacity);

        // 1. Magic Bytes (6 bytes)
        buffer.extend_from_slice(QN_MAGIC);

        // 2. Metadata Length (4 bytes, little-endian)
        buffer.extend_from_slice(&meta_len.to_le_bytes());

        // 3. Metadata JSON Payload
        buffer.extend_from_slice(&meta_json);

        // 4. Raw Image Payloads
        for att in &self.attachments {
            buffer.extend_from_slice(&att.data);
        }

        buffer
    }

    /// Decodes a `.qn` binary payload into a `Note` using default system resource limits.
    pub fn decode_qn_binary(bytes: &[u8]) -> Result<Self, QnDecodeError> {
        Self::decode_qn_binary_with_limits(
            bytes,
            MAX_QN_FILE_SIZE,
            MAX_ATTACHMENT_SIZE,
            MAX_TOTAL_ATTACHMENTS_SIZE,
            MAX_ATTACHMENTS_PER_NOTE,
        )
    }

    /// Decodes a `.qn` binary payload into a `Note` with custom resource limits.
    pub fn decode_qn_binary_with_limits(
        bytes: &[u8],
        max_file_size: usize,
        max_att_size: usize,
        max_total_att_size: usize,
        max_att_count: usize,
    ) -> Result<Self, QnDecodeError> {
        if bytes.len() > max_file_size {
            return Err(QnDecodeError::ExceedsLimit(format!(
                "Payload size {} exceeds maximum allowable .qn size {}",
                bytes.len(),
                max_file_size
            )));
        }

        if bytes.len() < QN_HEADER_OVERHEAD {
            return Err(QnDecodeError::TooShort);
        }

        if &bytes[0..6] != QN_MAGIC {
            return Err(QnDecodeError::InvalidMagic);
        }

        let meta_len_bytes: [u8; 4] = bytes[6..10]
            .try_into()
            .map_err(|_| QnDecodeError::TooShort)?;
        let meta_len = u32::from_le_bytes(meta_len_bytes) as usize;

        if meta_len > max_file_size || bytes.len() < QN_HEADER_OVERHEAD + meta_len {
            return Err(QnDecodeError::TooShort);
        }

        let meta_slice = &bytes[QN_HEADER_OVERHEAD..QN_HEADER_OVERHEAD + meta_len];
        let meta: QnMetaHeader = serde_json::from_slice(meta_slice)
            .map_err(|e| QnDecodeError::CorruptMetadata(e.to_string()))?;

        if meta.images.len() > max_att_count {
            return Err(QnDecodeError::ExceedsLimit(format!(
                "Attachment count {} exceeds maximum allowable count {}",
                meta.images.len(),
                max_att_count
            )));
        }

        let payload_start = QN_HEADER_OVERHEAD + meta_len;
        let payload_slice = &bytes[payload_start..];

        let mut total_att_bytes: usize = 0;
        let mut attachments = Vec::with_capacity(meta.images.len());
        for img_desc in meta.images {
            let length = img_desc.length as usize;
            if length > max_att_size {
                return Err(QnDecodeError::ExceedsLimit(format!(
                    "Attachment '{}' size {} exceeds max single attachment size {}",
                    img_desc.name, length, max_att_size
                )));
            }

            total_att_bytes = total_att_bytes.saturating_add(length);
            if total_att_bytes > max_total_att_size {
                return Err(QnDecodeError::ExceedsLimit(format!(
                    "Cumulative attachments size exceeds maximum allowable limit {}",
                    max_total_att_size
                )));
            }

            let start = img_desc.offset as usize;
            let end = start + length;

            if end > payload_slice.len() {
                return Err(QnDecodeError::TruncatedPayload);
            }

            attachments.push(NoteAttachment {
                id: img_desc.id,
                name: img_desc.name,
                mime_type: img_desc.mime_type,
                data: payload_slice[start..end].to_vec(),
            });
        }

        let mut note = Note {
            id: meta.id,
            title: meta.title,
            content: meta.content,
            created_at: meta.created_at,
            updated_at: meta.updated_at,
            pinned: meta.pinned,
            color_tag: meta.color_tag,
            file_path: None,
            last_disk_mtime: None,
            is_dirty: false,
            has_disk_conflict: false,
            attachments,
        };

        note.validate_and_repair("note-1");
        Ok(note)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_unicode_char_operations() {
        let mut note = Note::new("u1".to_string(), "unicode.qn".to_string());
        note.content = "// ✨ Quicky Notes — Developer Scratchpad\n// Try AI Copilot".to_string();

        assert_eq!(note.char_slice(5, 11), "Quicky");
        note.replace_char_range(5, 11, "Fast");
        assert!(note.content.starts_with("// ✨ Fast Notes"));

        note.delete_char_range(0, 5);
        assert!(note.content.starts_with("Fast Notes"));

        note.insert_at_char(0, "🚀 ");
        assert!(note.content.starts_with("🚀 Fast Notes"));
    }

    #[test]
    fn test_qn_binary_encode_decode_roundtrip() {
        let mut note = Note::new("roundtrip-1".to_string(), "embedded_images.qn".to_string());
        note.content = "# My Architecture\n\n![Diagram](attachment:img_1)\n".to_string();
        note.pinned = true;
        note.color_tag = Some("#38bdf8".to_string());

        let fake_png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x01, 0x02];
        let fake_jpg = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46];

        let id1 = note.add_attachment("diagram.png", "image/png", fake_png.clone());
        let id2 = note.add_attachment("photo.jpg", "image/jpeg", fake_jpg.clone());

        assert_eq!(note.attachments.len(), 2);

        // Encode to binary
        let binary_bytes = note.encode_qn_binary();
        assert!(binary_bytes.starts_with(QN_MAGIC));

        // Decode from binary
        let decoded = Note::decode_qn_binary(&binary_bytes).expect("Failed to decode .qn binary");
        assert_eq!(decoded.id, "roundtrip-1");
        assert_eq!(decoded.title, "embedded_images.qn");
        assert_eq!(decoded.content, note.content);
        assert!(decoded.pinned);
        assert_eq!(decoded.color_tag, Some("#38bdf8".to_string()));

        assert_eq!(decoded.attachments.len(), 2);
        assert_eq!(decoded.attachments[0].id, id1);
        assert_eq!(decoded.attachments[0].data, fake_png);
        assert_eq!(decoded.attachments[1].id, id2);
        assert_eq!(decoded.attachments[1].data, fake_jpg);
    }

    #[test]
    fn test_attachment_add_and_remove() {
        let mut note = Note::new("att-1".to_string(), "test.qn".to_string());
        let id = note.add_attachment("test.png", "image/png", vec![1, 2, 3]);
        assert_eq!(note.attachments.len(), 1);
        assert!(note.get_attachment(&id).is_some());

        let removed = note.remove_attachment(&id);
        assert!(removed);
        assert_eq!(note.attachments.len(), 0);
        assert!(note.get_attachment(&id).is_none());
    }

    #[test]
    fn test_qn_decode_resource_limits() {
        // Test oversized payload rejection
        let oversized = vec![0u8; MAX_QN_FILE_SIZE + 100];
        let res = Note::decode_qn_binary(&oversized);
        assert!(matches!(res, Err(QnDecodeError::ExceedsLimit(_))));

        // Test invalid magic signature
        let corrupt = b"CORRUPT_BYTES";
        let res2 = Note::decode_qn_binary(corrupt);
        assert_eq!(res2, Err(QnDecodeError::InvalidMagic));
    }
}
