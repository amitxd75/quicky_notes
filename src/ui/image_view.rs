//! True-color image view renderer, thread-safe texture cache, and attachment popup manager.
//!
//! # Color Fidelity Invariant
//! All images MUST be rendered with authentic RGB pixel colors and `Color32::WHITE` neutral tint.
//! Never apply theme accent, palette card tint, or wallpaper colors to image textures.

use crate::models::{Note, NoteAttachment};
use crate::theme::Palette;
use eframe::egui::{
    self, Color32, ColorImage, CornerRadius, FontId, Margin, Response, RichText, Sense, Stroke,
    TextureHandle, TextureOptions, Ui, vec2,
};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Maximum width of the image manager popover in pixels.
pub const IMAGE_POPUP_MAX_WIDTH: f32 = 380.0;

/// Maximum width of the thumbnail scroll area in pixels.
pub const THUMBNAIL_SCROLL_MAX_WIDTH: f32 = 360.0;

/// Image preview thumbnail dimension in pixels.
pub const THUMBNAIL_SIZE: egui::Vec2 = egui::vec2(120.0, 90.0);

/// Minimum proportional scaling factor for rendered images.
pub const MIN_IMAGE_FONT_SCALE: f32 = 0.5;

/// Maximum proportional scaling factor for rendered images.
pub const MAX_IMAGE_FONT_SCALE: f32 = 3.0;

/// Minimum display width in pixels for rendered images.
pub const MIN_IMAGE_DISPLAY_WIDTH: f32 = 32.0;

/// Maximum number of loaded GPU textures retained in the LRU cache.
pub const MAX_TEXTURE_CACHE_ENTRIES: usize = 64;

/// Maximum image width or height in pixels.
pub const MAX_IMAGE_DIMENSION: u32 = 8192;

/// Maximum image total pixels (16 Megapixels).
pub const MAX_IMAGE_PIXELS: u64 = 16_777_216;

/// Thread-safe in-memory LRU cache for loaded GPU textures.
struct TextureCache {
    textures: HashMap<String, TextureHandle>,
    order: Vec<String>,
}

impl TextureCache {
    fn insert(&mut self, key: String, handle: TextureHandle) {
        if self.textures.len() >= MAX_TEXTURE_CACHE_ENTRIES
            && !self.textures.contains_key(&key)
            && let Some(oldest) = self.order.first().cloned()
        {
            self.order.remove(0);
            self.textures.remove(&oldest);
        }
        self.order.retain(|k| k != &key);
        self.order.push(key.clone());
        self.textures.insert(key, handle);
    }

    fn touch(&mut self, key: &str) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            let k = self.order.remove(pos);
            self.order.push(k);
        }
    }
}

static TEXTURE_CACHE: OnceLock<Mutex<TextureCache>> = OnceLock::new();

fn get_texture_cache() -> &'static Mutex<TextureCache> {
    TEXTURE_CACHE.get_or_init(|| {
        Mutex::new(TextureCache {
            textures: HashMap::new(),
            order: Vec::new(),
        })
    })
}

/// Retrieves or decodes an image attachment into an `egui::TextureHandle`.
///
/// Images are cached in memory by composite key `"{note_id}:{attachment_id}"`.
pub fn get_or_load_attachment_texture(
    ctx: &egui::Context,
    note_id: &str,
    att: &NoteAttachment,
) -> Option<TextureHandle> {
    let key = format!("{}:{}", note_id, att.id);
    let mut cache = get_texture_cache().lock().unwrap();

    if let Some(tex) = cache.textures.get(&key).cloned() {
        cache.touch(&key);
        return Some(tex);
    }

    match decode_color_image(&att.data) {
        Ok(color_img) => {
            let tex = ctx.load_texture(&key, color_img, TextureOptions::LINEAR);
            cache.insert(key, tex.clone());
            Some(tex)
        }
        Err(err) => {
            eprintln!(
                "Warning: Failed to load texture for attachment '{}': {}",
                att.name, err
            );
            None
        }
    }
}

/// Decodes raw image bytes (PNG, JPEG, WebP, GIF, BMP) into an `egui::ColorImage`.
fn decode_color_image(bytes: &[u8]) -> Result<ColorImage, String> {
    if bytes.is_empty() {
        return Err("Empty image payload".to_string());
    }
    let dyn_img =
        image::load_from_memory(bytes).map_err(|e| format!("Image decode failed: {}", e))?;
    let (w, h) = (dyn_img.width(), dyn_img.height());
    if w > MAX_IMAGE_DIMENSION
        || h > MAX_IMAGE_DIMENSION
        || (w as u64 * h as u64) > MAX_IMAGE_PIXELS
    {
        return Err(format!(
            "Image dimensions {}x{} exceed maximum supported limit",
            w, h
        ));
    }
    let rgba = dyn_img.to_rgba8();
    let size = [w as usize, h as usize];
    let pixels = rgba.into_raw();
    Ok(ColorImage::from_rgba_unmultiplied(size, &pixels))
}

/// Renders an authentic true-color image widget adhering to strict color fidelity and responsive font scaling.
pub fn render_true_color_image(
    ui: &mut Ui,
    texture: &TextureHandle,
    max_width: f32,
    corner_radius: u8,
    font_size: f32,
) -> Response {
    let size = texture.size_vec2();
    let available_w = max_width
        .min(ui.available_width())
        .max(MIN_IMAGE_DISPLAY_WIDTH);

    let aspect = if size.x > 0.0 { size.y / size.x } else { 1.0 };
    let font_scale = (font_size / crate::models::settings::DEFAULT_FONT_SIZE)
        .clamp(MIN_IMAGE_FONT_SCALE, MAX_IMAGE_FONT_SCALE);
    let scaled_w = size.x * font_scale;
    let display_w = scaled_w.min(available_w);
    let display_h = display_w * aspect;

    let img = egui::Image::new(texture)
        .fit_to_exact_size(vec2(display_w, display_h))
        .corner_radius(CornerRadius::same(corner_radius))
        .tint(Color32::WHITE);

    ui.add(img)
}

/// Renders a sleek, non-intrusive pill button in the status bar that opens a floating image manager popup.
pub fn render_attachment_popup_button(
    ui: &mut Ui,
    ctx: &egui::Context,
    note: &mut Note,
    palette: &Palette,

    content_changed: &mut bool,
) {
    let att_count = note.attachments.len();
    if att_count == 0 {
        return;
    }

    let popup_id = ui.make_persistent_id(format!("attach_popup_{}", note.id));
    let mut is_open = ui.data(|d| d.get_temp::<bool>(popup_id).unwrap_or(false));

    let label_text = format!("🖼 {}", att_count);
    let btn_resp = ui
        .add(
            egui::Label::new(
                RichText::new(label_text)
                    .font(FontId::proportional(11.5))
                    .color(if is_open {
                        palette.accent
                    } else {
                        Color32::from_gray(190)
                    }),
            )
            .sense(Sense::click()),
        )
        .on_hover_text(format!(
            "🖼 {} image attachment(s)\nClick to view & manage",
            att_count
        ));

    if btn_resp.clicked() {
        is_open = !is_open;
        ui.data_mut(|d| d.insert_temp(popup_id, is_open));
    }

    if is_open {
        let popup_pos = btn_resp.rect.left_top() + vec2(0.0, -8.0);
        let mut remove_id: Option<String> = None;
        let mut copy_tag: Option<String> = None;
        let mut close_popup = false;

        egui::Area::new(popup_id.with("floating_popup"))
            .order(egui::Order::Foreground)
            .pivot(egui::Align2::LEFT_BOTTOM)
            .fixed_pos(popup_pos)
            .show(ctx, |ui| {
                egui::Frame::NONE
                    .fill(Color32::from_rgba_unmultiplied(
                        palette.bg.r(),
                        palette.bg.g(),
                        palette.bg.b(),
                        248,
                    ))
                    .stroke(Stroke::new(
                        1.0,
                        Palette::with_alpha(palette.accent, 150),
                    ))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.set_max_width(IMAGE_POPUP_MAX_WIDTH);

                        // Header
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("🖼 Attached Images ({})", att_count))
                                    .font(FontId::proportional(12.0))
                                    .strong()
                                    .color(palette.accent),
                            );

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("✕").on_hover_text("Close").clicked() {
                                    close_popup = true;
                                }
                            });
                        });

                        ui.add_space(6.0);

                        // Horizontal thumbnail carousel
                        egui::ScrollArea::horizontal()
                            .id_salt(popup_id.with("scroll"))
                            .max_width(THUMBNAIL_SCROLL_MAX_WIDTH)
                            .show(ui, |ui| {

                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 8.0;

                                    for att in &note.attachments {
                                        egui::Frame::NONE
                                            .fill(Color32::from_rgba_unmultiplied(20, 16, 32, 230))
                                            .stroke(Stroke::new(
                                                1.0,
                                                Palette::with_alpha(palette.border, 120),
                                            ))
                                            .corner_radius(CornerRadius::same(6))
                                            .inner_margin(Margin::symmetric(6, 6))
                                            .show(ui, |ui| {
                                                ui.vertical(|ui| {
                                                    ui.set_width(110.0);

                                                    if let Some(tex) =
                                                        get_or_load_attachment_texture(
                                                            ctx,
                                                            &note.id,
                                                            att,
                                                        )
                                                    {
                                                        let thumb_size = vec2(110.0, 70.0);
                                                        let img = ui.add(
                                                            egui::Image::new(&tex)
                                                                .fit_to_exact_size(thumb_size)
                                                                .corner_radius(CornerRadius::same(
                                                                    4,
                                                                ))
                                                                .tint(Color32::WHITE)
                                                                .sense(Sense::click()),
                                                        );
                                                        if img
                                                            .on_hover_text(format!(
                                                                "Click to copy tag:\n![{}](attachment:{})",
                                                                att.name, att.id
                                                            ))
                                                            .clicked()
                                                        {
                                                            copy_tag = Some(format!(
                                                                "![{}](attachment:{})",
                                                                att.name, att.id
                                                            ));
                                                        }
                                                    } else {
                                                        ui.allocate_space(vec2(110.0, 70.0));
                                                    }

                                                    ui.add_space(3.0);
                                                    ui.label(
                                                        RichText::new(&att.name)
                                                            .font(FontId::proportional(10.5))
                                                            .color(Color32::WHITE)
                                                            .strong(),
                                                    );
                                                    ui.label(
                                                        RichText::new(att.formatted_size())
                                                            .font(FontId::proportional(9.5))
                                                            .color(Color32::from_gray(140)),
                                                    );

                                                    ui.add_space(3.0);
                                                    ui.horizontal(|ui| {
                                                        if ui
                                                            .small_button("📋")
                                                            .on_hover_text("Copy markdown tag")
                                                            .clicked()
                                                        {
                                                            copy_tag = Some(format!(
                                                                "![{}](attachment:{})",
                                                                att.name, att.id
                                                            ));
                                                        }
                                                        if ui
                                                            .button(
                                                                RichText::new("✕")
                                                                    .font(FontId::proportional(
                                                                        10.0,
                                                                    ))
                                                                    .color(Color32::from_rgb(
                                                                        248, 113, 113,
                                                                    )),
                                                            )
                                                            .on_hover_text("Remove attachment")
                                                            .clicked()
                                                        {
                                                            remove_id = Some(att.id.clone());
                                                        }
                                                    });
                                                });
                                            });
                                    }
                                });
                            });
                    });
            });

        if let Some(tag) = copy_tag {
            ctx.copy_text(tag);
        }

        if let Some(id) = remove_id {
            note.remove_attachment(&id);
            *content_changed = true;
        }

        if close_popup {
            ui.data_mut(|d| d.insert_temp(popup_id, false));
        }
    }
}
