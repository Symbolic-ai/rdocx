//! Font loading, resolution, shaping, and metrics.
//!
//! Uses fontdb for system font discovery, ttf-parser for metrics,
//! and HarfRust for text shaping.

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{LayoutError, Result};
use crate::output::FontId;

/// Key for caching resolved fonts.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FontKey {
    family: String,
    bold: bool,
    italic: bool,
}

/// Metrics for a font at a given size.
#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    /// Ascent in points (positive, above baseline).
    pub ascent: f64,
    /// Descent in points (positive, below baseline).
    pub descent: f64,
    /// Line gap in points.
    pub line_gap: f64,
    /// Units per em.
    pub units_per_em: u16,
}

/// Result of shaping a text string.
#[derive(Debug, Clone)]
pub struct ShapedText {
    /// Glyph IDs from shaping.
    pub glyph_ids: Vec<u16>,
    /// Per-glyph advances in points.
    pub advances: Vec<f64>,
    /// Total width in points.
    pub width: f64,
}

/// Internal record for a loaded font face.
struct LoadedFont {
    id: FontId,
    family: String,
    bold: bool,
    italic: bool,
    data: Arc<Vec<u8>>,
    face_index: u32,
    units_per_em: u16,
    /// Vertical metrics in design units, read once when the face is loaded.
    ascender: i16,
    descender: i16,
    line_gap: i16,
    /// HarfRust's per-face shaping caches. Building these is the expensive
    /// part of shaping, so it happens once per face instead of once per run.
    shaper_data: harfrust::ShaperData,
}

/// Manages font discovery, loading, shaping, and metrics.
pub struct FontManager {
    db: fontdb::Database,
    /// Map from FontKey to loaded font info.
    cache: HashMap<FontKey, usize>,
    /// All loaded fonts.
    fonts: Vec<LoadedFont>,
    /// Next font ID counter.
    next_id: u32,
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FontManager {
    /// Create a new FontManager and load system fonts.
    ///
    /// When the `bundled-fonts` feature is enabled, bundled fonts (Carlito,
    /// Caladea, Liberation) are loaded as fallbacks.
    pub fn new() -> Self {
        let mut db = fontdb::Database::new();

        // Load bundled fonts first (lowest priority fallbacks)
        for (_family, data) in crate::bundled_fonts::bundled_font_data() {
            db.load_font_data(data.to_vec());
        }

        // Then load system fonts
        db.load_system_fonts();

        FontManager {
            db,
            cache: HashMap::new(),
            fonts: Vec::new(),
            next_id: 0,
        }
    }

    /// Create a font manager that loads bundled fonts without discovering
    /// system fonts.
    ///
    /// This mode makes font resolution reproducible across machines. It
    /// requires the `bundled-fonts` feature so callers cannot accidentally
    /// construct an empty deterministic font database.
    pub fn new_deterministic() -> Result<Self> {
        #[cfg(feature = "bundled-fonts")]
        {
            let mut db = fontdb::Database::new();
            for (_family, data) in crate::bundled_fonts::bundled_font_data() {
                db.load_font_data(data.to_vec());
            }

            Ok(FontManager {
                db,
                cache: HashMap::new(),
                fonts: Vec::new(),
                next_id: 0,
            })
        }

        #[cfg(not(feature = "bundled-fonts"))]
        {
            Err(LayoutError::Layout(
                "deterministic font mode requires the 'bundled-fonts' feature".to_string(),
            ))
        }
    }

    /// Load additional font files (user-provided or extracted from DOCX).
    ///
    /// These fonts are loaded AFTER system fonts, so they take the highest
    /// priority in font resolution (fontdb returns the last-loaded match).
    pub fn load_additional_fonts(&mut self, font_files: &[crate::input::FontFile]) {
        for font_file in font_files {
            self.db.load_font_data(font_file.data.clone());
        }
        // Clear the cache since new fonts may affect resolution
        self.cache.clear();
    }

    /// Create a FontManager with user-provided fonts (no system font loading).
    ///
    /// Each entry is `(family_name, font_bytes)`. This is useful in environments
    /// where system fonts are not available, such as WASM.
    pub fn new_with_fonts(fonts: Vec<(String, Vec<u8>)>) -> Self {
        let mut db = fontdb::Database::new();
        for (_name, data) in &fonts {
            db.load_font_data(data.clone());
        }
        FontManager {
            db,
            cache: HashMap::new(),
            fonts: Vec::new(),
            next_id: 0,
        }
    }

    /// Resolve a font by family name, bold, and italic flags.
    /// Returns a FontId. Uses fallback chain if the requested font is not found.
    pub fn resolve_font(
        &mut self,
        family: Option<&str>,
        bold: bool,
        italic: bool,
    ) -> Result<FontId> {
        let family_name = family.unwrap_or("Arial");

        let key = FontKey {
            family: family_name.to_string(),
            bold,
            italic,
        };

        if let Some(&idx) = self.cache.get(&key) {
            return Ok(self.fonts[idx].id);
        }

        // Map common Word font names to metric-compatible alternatives
        let mapped = map_font_name(family_name);

        // Try the requested font, mapped alternatives, then generic fallbacks
        let mut fallbacks: Vec<&str> = Vec::with_capacity(10);
        fallbacks.push(family_name);
        for alt in mapped {
            if *alt != family_name {
                fallbacks.push(alt);
            }
        }
        for generic in &[
            "Carlito",
            "Arial",
            "Liberation Sans",
            "Helvetica",
            "DejaVu Sans",
            "Noto Sans",
        ] {
            if !fallbacks.contains(generic) {
                fallbacks.push(generic);
            }
        }

        let style = if italic {
            fontdb::Style::Italic
        } else {
            fontdb::Style::Normal
        };
        let weight = if bold {
            fontdb::Weight::BOLD
        } else {
            fontdb::Weight::NORMAL
        };

        let mut found_id = None;
        for fallback in &fallbacks {
            let query = fontdb::Query {
                families: &[fontdb::Family::Name(fallback)],
                weight,
                style,
                stretch: fontdb::Stretch::Normal,
            };

            if let Some(id) = self.db.query(&query) {
                found_id = Some(id);
                break;
            }
        }

        // Last resort: try generic families
        if found_id.is_none() {
            for generic_family in &[
                fontdb::Family::SansSerif,
                fontdb::Family::Serif,
                fontdb::Family::Monospace,
            ] {
                let query = fontdb::Query {
                    families: &[*generic_family],
                    weight,
                    style,
                    stretch: fontdb::Stretch::Normal,
                };
                if let Some(id) = self.db.query(&query) {
                    found_id = Some(id);
                    break;
                }
            }
        }

        let db_id = found_id.ok_or_else(|| {
            LayoutError::FontNotFound(format!("No font found for family '{family_name}'"))
        })?;

        let font_id = FontId(self.next_id);
        self.next_id += 1;

        // Load the font data
        let (data, face_index) = self
            .db
            .with_face_data(db_id, |data, idx| (Arc::new(data.to_vec()), idx))
            .ok_or_else(|| LayoutError::FontParse("Failed to load font data".into()))?;

        let (units_per_em, ascender, descender, line_gap) = {
            let face = ttf_parser::Face::parse(&data, face_index)
                .map_err(|e| LayoutError::FontParse(format!("ttf-parser error: {e}")))?;
            (
                face.units_per_em(),
                face.ascender(),
                face.descender(),
                face.line_gap(),
            )
        };

        // Every metric and advance is scaled by size/upem, so a zero here would
        // turn the whole layout into infinities.
        if units_per_em == 0 {
            return Err(LayoutError::FontParse(format!(
                "font '{family_name}' declares zero units per em"
            )));
        }

        let shaper_data = {
            let face = harfrust::FontRef::from_index(&data, face_index)
                .map_err(|e| LayoutError::FontParse(format!("failed to read font face: {e}")))?;
            harfrust::ShaperData::new(&face)
        };

        let actual_family = self
            .db
            .face(db_id)
            .map(|f| {
                f.families
                    .first()
                    .map(|(name, _)| name.clone())
                    .unwrap_or_else(|| family_name.to_string())
            })
            .unwrap_or_else(|| family_name.to_string());

        let idx = self.fonts.len();
        self.fonts.push(LoadedFont {
            id: font_id,
            family: actual_family,
            bold,
            italic,
            data,
            face_index,
            units_per_em,
            ascender,
            descender,
            line_gap,
            shaper_data,
        });
        self.cache.insert(key, idx);

        Ok(font_id)
    }

    /// Get font metrics at a given size in points.
    pub fn metrics(&self, font_id: FontId, size_pt: f64) -> Result<FontMetrics> {
        let font = self.get_font(font_id)?;
        let scale = size_pt / font.units_per_em as f64;

        Ok(FontMetrics {
            ascent: font.ascender as f64 * scale,
            descent: -(font.descender as f64) * scale, // make positive
            line_gap: font.line_gap as f64 * scale,
            units_per_em: font.units_per_em,
        })
    }

    /// Shape a text string using HarfRust. Returns glyph IDs and advances.
    pub fn shape_text(&self, font_id: FontId, text: &str, size_pt: f64) -> Result<ShapedText> {
        // HarfRust cannot derive segment properties from an empty buffer, and
        // there is nothing to shape anyway.
        if text.is_empty() {
            return Ok(ShapedText {
                glyph_ids: Vec::new(),
                advances: Vec::new(),
                width: 0.0,
            });
        }

        let font = self.get_font(font_id)?;

        let face = harfrust::FontRef::from_index(&font.data, font.face_index)
            .map_err(|e| LayoutError::Shaping(format!("failed to read font face: {e}")))?;

        let shaper = font.shaper_data.shaper(&face).build();

        let mut buffer = harfrust::UnicodeBuffer::new();
        buffer.push_str(text);
        // Infer direction, script and language from the text. Unlike rustybuzz,
        // HarfRust does not do this implicitly and panics on an unset direction.
        buffer.guess_segment_properties();

        let output = shaper.shape(buffer, harfrust::ShapeOptions::default());
        let infos = output.glyph_infos();
        let positions = output.glyph_positions();

        let upem = font.units_per_em as f64;
        let scale = size_pt / upem;

        let mut glyph_ids = Vec::with_capacity(infos.len());
        let mut advances = Vec::with_capacity(positions.len());
        let mut total_width = 0.0;

        for (info, pos) in infos.iter().zip(positions.iter()) {
            glyph_ids.push(info.glyph_id as u16);
            let advance = pos.x_advance as f64 * scale;
            advances.push(advance);
            total_width += advance;
        }

        Ok(ShapedText {
            glyph_ids,
            advances,
            width: total_width,
        })
    }

    /// Get font data for PDF embedding.
    pub fn font_data(&self, font_id: FontId) -> Result<crate::output::FontData> {
        let font = self.get_font(font_id)?;
        Ok(crate::output::FontData {
            id: font.id,
            family: font.family.clone(),
            data: (*font.data).clone(),
            face_index: font.face_index,
            bold: font.bold,
            italic: font.italic,
        })
    }

    /// Get all used font data.
    pub fn all_font_data(&self) -> Vec<crate::output::FontData> {
        self.fonts
            .iter()
            .map(|f| crate::output::FontData {
                id: f.id,
                family: f.family.clone(),
                data: (*f.data).clone(),
                face_index: f.face_index,
                bold: f.bold,
                italic: f.italic,
            })
            .collect()
    }

    fn get_font(&self, font_id: FontId) -> Result<&LoadedFont> {
        self.fonts
            .iter()
            .find(|f| f.id == font_id)
            .ok_or_else(|| LayoutError::FontNotFound(format!("FontId({}) not loaded", font_id.0)))
    }
}

/// Map common Word font names to metric-compatible alternatives.
/// Returns a list of candidate names to try (including the original).
///
/// Priority: original font → metric-compatible open-source clone → generic fallback.
/// Carlito is metric-compatible with Calibri, Caladea with Cambria,
/// Liberation Sans/Serif/Mono with Arial/Times New Roman/Courier New.
fn map_font_name(name: &str) -> &[&str] {
    match name {
        "Calibri" => &["Calibri", "Carlito"],
        "Calibri Light" => &["Calibri Light", "Carlito"],
        "Cambria" => &["Cambria", "Caladea"],
        "Cambria Math" => &["Cambria Math", "Cambria", "Caladea"],
        "Arial" => &["Arial", "Liberation Sans", "Helvetica"],
        "Times New Roman" => &["Times New Roman", "Liberation Serif", "Times"],
        "Courier New" => &["Courier New", "Liberation Mono", "Courier"],
        "Consolas" => &["Consolas", "Liberation Mono", "DejaVu Sans Mono"],
        "Segoe UI" => &["Segoe UI", "Carlito", "Liberation Sans"],
        "Tahoma" => &["Tahoma", "Liberation Sans", "Helvetica"],
        "Verdana" => &["Verdana", "Liberation Sans", "DejaVu Sans"],
        "Georgia" => &["Georgia", "Caladea", "Liberation Serif"],
        "Palatino Linotype" => &["Palatino Linotype", "Palatino", "Liberation Serif"],
        "Book Antiqua" => &["Book Antiqua", "Palatino", "Liberation Serif"],
        "Garamond" => &["Garamond", "Caladea", "Liberation Serif"],
        "Trebuchet MS" => &["Trebuchet MS", "Liberation Sans", "DejaVu Sans"],
        "Impact" => &["Impact", "Liberation Sans", "Arial"],
        "Comic Sans MS" => &["Comic Sans MS", "Liberation Sans", "DejaVu Sans"],
        "Symbol" => &["Symbol", "DejaVu Sans"],
        "Wingdings" => &["Wingdings", "Symbol"],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "bundled-fonts")]
    use crate::bundled_fonts::bundled_font_data;

    #[cfg(feature = "bundled-fonts")]
    #[test]
    fn deterministic_font_manager_uses_only_bundled_fonts() {
        let mut fm = FontManager::new_deterministic().expect("bundled fonts should load");

        assert_eq!(fm.db.faces().count(), bundled_font_data().len());
        assert!(fm.resolve_font(Some("Arial"), false, false).is_ok());
    }

    #[cfg(not(feature = "bundled-fonts"))]
    #[test]
    fn deterministic_font_manager_requires_bundled_fonts() {
        match FontManager::new_deterministic() {
            Err(LayoutError::Layout(message)) => assert_eq!(
                message,
                "deterministic font mode requires the 'bundled-fonts' feature"
            ),
            _ => panic!("deterministic mode must reject a build without bundled fonts"),
        }
    }

    #[test]
    fn load_system_font() {
        let mut fm = FontManager::new();
        // Should be able to resolve at least one font via fallback
        let result = fm.resolve_font(None, false, false);
        // On CI or systems without fonts this might fail, so we just check it doesn't panic
        if let Ok(id) = result {
            assert_eq!(id.0, 0);
        }
    }

    #[test]
    fn font_metrics_positive() {
        let mut fm = FontManager::new();
        if let Ok(id) = fm.resolve_font(None, false, false) {
            let metrics = fm.metrics(id, 12.0).unwrap();
            assert!(metrics.ascent > 0.0);
            assert!(metrics.descent > 0.0);
            assert!(metrics.units_per_em > 0);
        }
    }

    #[test]
    fn shape_hello_world() {
        let mut fm = FontManager::new();
        if let Ok(id) = fm.resolve_font(None, false, false) {
            let shaped = fm.shape_text(id, "Hello World", 12.0).unwrap();
            assert!(!shaped.glyph_ids.is_empty());
            assert_eq!(shaped.glyph_ids.len(), shaped.advances.len());
            assert!(shaped.width > 0.0);
        }
    }

    #[test]
    fn font_caching() {
        let mut fm = FontManager::new();
        if let Ok(id1) = fm.resolve_font(Some("Arial"), false, false) {
            let id2 = fm.resolve_font(Some("Arial"), false, false).unwrap();
            assert_eq!(id1, id2);
        }
    }

    #[test]
    fn bold_italic_variants() {
        let mut fm = FontManager::new();
        let regular = fm.resolve_font(None, false, false);
        let bold = fm.resolve_font(None, true, false);
        if let (Ok(r), Ok(b)) = (regular, bold) {
            // Bold should get a different font ID (different variant)
            assert_ne!(r, b);
        }
    }
}
