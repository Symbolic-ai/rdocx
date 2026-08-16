//! Layout engine orchestrator: ties all phases together.

use rdocx_oxml::document::{BodyContent, CT_SectPr};
use rdocx_oxml::header_footer::HdrFtrType;
use rdocx_oxml::properties::CT_PPr;
use rdocx_oxml::shared::ST_HighlightColor;
use rdocx_oxml::styles::CT_Styles;
use rdocx_oxml::text::{BreakType, CT_P, FieldType, RunContent};

use crate::block::{self, LayoutBlock, ParagraphBlock};
use crate::convert;
use crate::input::{LayoutInput, MediaRegistry};
use crate::paginator::{self, HeaderFooterContent, PageGeometry};
use crate::style_resolver::{self, NumberingState};
use crate::table;
use oxml_layout::{
    Color, DocumentMetadata, FieldKind, FontManager, GlyphRun, InlineItem, LayoutResult, LineItem,
    PageFrame, Point, PositionedElement, Rect, Result, TextSegment, break_into_lines,
};

/// The layout engine.
pub struct Engine {
    font_manager: FontManager,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            font_manager: FontManager::new(),
        }
    }

    /// Create an engine that resolves fonts without system font discovery.
    pub fn new_deterministic() -> Result<Self> {
        Ok(Engine {
            font_manager: FontManager::new_deterministic()?,
        })
    }

    /// Lay out the entire document.
    pub fn layout(&mut self, input: &LayoutInput) -> Result<LayoutResult> {
        // Load user-provided / DOCX-embedded fonts (highest priority)
        if !input.fonts.is_empty() {
            self.font_manager.load_additional_fonts(&input.fonts);
        }

        let styles = &input.styles;
        let mut num_state = NumberingState::new();
        let media = MediaRegistry::new(&input.images);

        // Get final section properties (body-level sectPr)
        let final_sect_pr = input
            .document
            .body
            .sect_pr
            .as_ref()
            .cloned()
            .unwrap_or_else(CT_SectPr::default_letter);

        // Build sections: each section has blocks + geometry + header/footer
        let mut sections: Vec<paginator::Section> = Vec::new();
        let mut current_blocks: Vec<LayoutBlock> = Vec::new();
        let mut current_sect_pr: Option<CT_SectPr> = None; // Will be set from paragraph sect_pr

        for content in &input.document.body.content {
            match content {
                BodyContent::Paragraph(para) => {
                    // Check if this paragraph ends a section (has sect_pr)
                    let para_sect_pr = para.properties.as_ref().and_then(|p| p.sect_pr.clone());

                    let sect_pr_for_layout = para_sect_pr
                        .as_ref()
                        .or(current_sect_pr.as_ref())
                        .unwrap_or(&final_sect_pr);
                    let geometry = sect_pr_to_geometry(sect_pr_for_layout);

                    let mut para_block = layout_paragraph(
                        para,
                        geometry.content_width(),
                        styles,
                        input,
                        &media,
                        &mut self.font_manager,
                        &mut num_state,
                    )?;

                    // Detect heading style for outline generation
                    if let Some(level) = detect_heading_level(para, styles) {
                        para_block.heading_level = Some(level);
                        para_block.heading_text = Some(para.text());
                    }

                    current_blocks.push(LayoutBlock::Paragraph(para_block));

                    // If this paragraph has sect_pr, it ends a section
                    if let Some(sect_pr) = para_sect_pr {
                        let geometry = sect_pr_to_geometry(&sect_pr);
                        let header_footer = layout_header_footer(
                            &sect_pr,
                            input,
                            styles,
                            &media,
                            &mut self.font_manager,
                            &mut num_state,
                        )?;
                        let title_pg = sect_pr.title_pg.unwrap_or(false);
                        sections.push(paginator::Section {
                            blocks: std::mem::take(&mut current_blocks),
                            geometry,
                            header_footer,
                            title_pg,
                        });
                        current_sect_pr = Some(sect_pr);
                    }
                }
                BodyContent::Table(tbl) => {
                    let sect_pr_for_layout = current_sect_pr.as_ref().unwrap_or(&final_sect_pr);
                    let geometry = sect_pr_to_geometry(sect_pr_for_layout);

                    let table_block = table::layout_table(
                        tbl,
                        geometry.content_width(),
                        styles,
                        input,
                        &media,
                        &mut self.font_manager,
                        &mut num_state,
                    )?;
                    current_blocks.push(LayoutBlock::Table(table_block));
                }
                _ => {} // Skip RawXml elements during layout
            }
        }

        // Remaining blocks belong to the final section
        let final_geometry = sect_pr_to_geometry(&final_sect_pr);
        let final_hf = layout_header_footer(
            &final_sect_pr,
            input,
            styles,
            &media,
            &mut self.font_manager,
            &mut num_state,
        )?;
        let final_title_pg = final_sect_pr.title_pg.unwrap_or(false);
        sections.push(paginator::Section {
            blocks: current_blocks,
            geometry: final_geometry,
            header_footer: final_hf,
            title_pg: final_title_pg,
        });

        // Paginate across all sections
        let (mut pages, outlines) =
            paginator::paginate_sections(&sections, &self.font_manager, &media);

        // Post-pagination pass: substitute field placeholders
        let total_pages = pages.len();
        for page in &mut pages {
            let page_num = page.page_number;
            substitute_fields(
                &mut page.elements,
                page_num,
                total_pages,
                &mut self.font_manager,
            );
        }

        // Post-pagination pass: apply page background color
        apply_page_background(&mut pages, input);

        // Post-pagination pass: render footnotes at page bottoms
        if input.footnotes.is_some() || input.endnotes.is_some() {
            render_page_footnotes(
                &mut pages,
                input,
                styles,
                &final_geometry,
                &media,
                &mut self.font_manager,
                &mut num_state,
            )?;
        }

        // Collect font data
        let fonts = self.font_manager.all_font_data();

        // Convert core properties to document metadata
        let metadata = input.core_properties.as_ref().map(|cp| DocumentMetadata {
            title: cp.title.clone(),
            author: cp.creator.clone(),
            subject: cp.subject.clone(),
            keywords: cp.keywords.clone(),
            creator: Some("rdocx".to_string()),
        });

        Ok(LayoutResult::new(pages, fonts, metadata, outlines))
    }
}

/// Apply page background color from `w:background` element to all pages.
fn apply_page_background(pages: &mut [PageFrame], input: &LayoutInput) {
    let bg_xml = match &input.document.background_xml {
        Some(xml) => xml,
        None => return,
    };

    // Parse w:color attribute from background XML
    let xml_str = std::str::from_utf8(bg_xml).unwrap_or("");
    let color = extract_background_color(xml_str);
    let color = match color {
        Some(c) => c,
        None => return,
    };

    // Insert a full-page FilledRect at position 0 on every page (renders underneath everything)
    for page in pages.iter_mut() {
        page.elements.insert(
            0,
            PositionedElement::FilledRect {
                rect: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: page.width,
                    height: page.height,
                },
                color,
            },
        );
    }
}

/// Extract the background color hex from w:background XML.
fn extract_background_color(xml: &str) -> Option<Color> {
    // Look for w:color="RRGGBB" or color="RRGGBB"
    for attr in ["w:color=\"", "color=\""] {
        if let Some(start) = xml.find(attr) {
            let val_start = start + attr.len();
            if let Some(end) = xml[val_start..].find('"') {
                let hex = &xml[val_start..val_start + end];
                if hex.len() == 6 && hex != "auto" {
                    return Some(Color::from_hex(hex));
                }
            }
        }
    }
    None
}

/// Replace field placeholder GlyphRuns with actual values.
fn substitute_fields(
    elements: &mut [PositionedElement],
    page_number: usize,
    total_pages: usize,
    fm: &mut FontManager,
) {
    for element in elements.iter_mut() {
        if let PositionedElement::Text(run) = element
            && let Some(fk) = run.field_kind
        {
            let value = match fk {
                FieldKind::Page => page_number.to_string(),
                FieldKind::NumPages => total_pages.to_string(),
            };
            // Re-shape the text with the actual value
            if let Ok(shaped) = fm.shape_text(run.font_id, &value, run.font_size) {
                run.text = value;
                run.glyph_ids = shaped.glyph_ids;
                run.advances = shaped.advances;
            }
        }
    }
}

/// Horizontal space reserved for the note marker, to the left of note text.
///
/// Note lines are both broken and drawn against this, so the two agree. Break
/// at the full content width and draw one indent in, and every line overruns
/// the right margin by exactly this much.
const FOOTNOTE_INDENT: f64 = 12.0;

/// Render footnote/endnote content at the bottom of each page.
///
/// For each page, collects footnote IDs from glyph runs, then
/// renders a separator line and the footnote text in a smaller font.
fn render_page_footnotes(
    pages: &mut [PageFrame],
    input: &LayoutInput,
    styles: &CT_Styles,
    geometry: &paginator::PageGeometry,
    media: &MediaRegistry,
    fm: &mut FontManager,
    num_state: &mut NumberingState,
) -> Result<()> {
    let footnote_font_size = 8.0; // Standard footnote font size
    let separator_offset = 6.0; // Space above separator
    let separator_width_frac = 0.33; // Separator is 1/3 of content width

    for page in pages.iter_mut() {
        // Collect footnote IDs referenced on this page (in order, deduplicated)
        let mut footnote_ids: Vec<i32> = Vec::new();
        for element in &page.elements {
            if let PositionedElement::Text(run) = element
                && let Some(fn_id) = run.footnote_id
                && !footnote_ids.contains(&fn_id)
            {
                footnote_ids.push(fn_id);
            }
        }

        if footnote_ids.is_empty() {
            continue;
        }

        // Find the footnote paragraphs to render
        let mut footnote_blocks: Vec<(i32, Vec<block::ParagraphBlock>)> = Vec::new();
        for &fn_id in &footnote_ids {
            // Check footnotes first, then endnotes
            let paragraphs = input
                .footnotes
                .as_ref()
                .and_then(|fns| fns.get_by_id(fn_id))
                .or_else(|| input.endnotes.as_ref().and_then(|ens| ens.get_by_id(fn_id)));

            if let Some(footnote) = paragraphs {
                let mut fn_blocks = Vec::new();
                for para in &footnote.paragraphs {
                    if let Ok(pb) = layout_paragraph(
                        para,
                        geometry.content_width() - FOOTNOTE_INDENT,
                        styles,
                        input,
                        media,
                        fm,
                        num_state,
                    ) {
                        fn_blocks.push(pb);
                    }
                }
                footnote_blocks.push((fn_id, fn_blocks));
            }
        }

        if footnote_blocks.is_empty() {
            continue;
        }

        // Calculate total footnote height
        let total_fn_height: f64 = footnote_blocks
            .iter()
            .flat_map(|(_, blocks)| blocks.iter())
            .map(|b| b.content_height())
            .sum();

        // Position footnotes at page bottom, above bottom margin
        let footnote_area_top =
            page.height - geometry.margin_bottom - total_fn_height - separator_offset;

        // Draw separator line
        let sep_y = footnote_area_top;
        let sep_width = geometry.content_width() * separator_width_frac;
        page.elements.push(PositionedElement::Line {
            start: Point {
                x: geometry.margin_left,
                y: sep_y,
            },
            end: Point {
                x: geometry.margin_left + sep_width,
                y: sep_y,
            },
            width: 0.5,
            color: Color::BLACK,
            dash_pattern: None,
        });

        // Render each footnote
        let mut cursor_y = sep_y + separator_offset;
        for (fn_id, blocks) in &footnote_blocks {
            for pb in blocks {
                let baseline_y = cursor_y + pb.lines.first().map(|l| l.ascent).unwrap_or(0.0);

                // Render the footnote number marker as superscript
                let marker_text = fn_id.to_string();
                let marker_size = footnote_font_size * 0.58;
                if let Ok(font_id) = fm.resolve_font(Some("serif"), false, false)
                    && let Ok(shaped) = fm.shape_text(font_id, &marker_text, marker_size)
                {
                    page.elements.push(PositionedElement::Text(GlyphRun {
                        origin: Point {
                            x: geometry.margin_left,
                            y: baseline_y - footnote_font_size * 0.33,
                        },
                        font_id,
                        font_size: marker_size,
                        glyph_ids: shaped.glyph_ids,
                        advances: shaped.advances,
                        text: marker_text,
                        color: Color::BLACK,
                        bold: false,
                        italic: false,
                        field_kind: None,
                        footnote_id: None,
                    }));
                }

                // Render footnote paragraph lines
                let indent = FOOTNOTE_INDENT;
                for line in &pb.lines {
                    let line_baseline = cursor_y + line.ascent;
                    // Advance across the line the way the body path does. A
                    // fixed origin stacks every segment of a multi-run note on
                    // top of itself.
                    let mut x = geometry.margin_left + indent;
                    for item in &line.items {
                        // A tab or an inline image inside a note is not drawn
                        // yet, but it still occupies width. Skipping its
                        // advance would pull everything after it left.
                        let (seg, advance) = match item {
                            LineItem::Text(seg) | LineItem::Marker(seg) => (Some(seg), seg.width),
                            LineItem::Tab { width, .. } | LineItem::Image { width, .. } => {
                                (None, *width)
                            }
                        };
                        if let Some(seg) = seg {
                            page.elements.push(PositionedElement::Text(GlyphRun {
                                origin: Point {
                                    x,
                                    y: line_baseline - seg.baseline_offset,
                                },
                                font_id: seg.font_id,
                                font_size: seg.font_size,
                                glyph_ids: seg.glyph_ids.clone(),
                                advances: seg.advances.clone(),
                                text: seg.text.clone(),
                                color: seg.color,
                                bold: seg.bold,
                                italic: seg.italic,
                                field_kind: None,
                                footnote_id: None,
                            }));
                        }
                        x += advance;
                    }
                    cursor_y += line.height;
                }
            }
        }
    }

    Ok(())
}

/// Detect if a paragraph has a heading style, returning the level (1-9).
fn detect_heading_level(para: &CT_P, styles: &CT_Styles) -> Option<u32> {
    let style_id = para.properties.as_ref()?.style_id.as_deref()?;
    // Check if style ID matches "Heading1" .. "Heading9"
    if let Some(rest) = style_id.strip_prefix("Heading") {
        return rest.parse::<u32>().ok().filter(|n| (1..=9).contains(n));
    }
    // Also check style name in the styles definitions
    if let Some(style_def) = styles.get_by_id(style_id)
        && let Some(ref name) = style_def.name
        && let Some(rest) = name.strip_prefix("heading ")
    {
        return rest.parse::<u32>().ok().filter(|n| (1..=9).contains(n));
    }
    None
}

/// Lay out a single paragraph into a ParagraphBlock.
pub fn layout_paragraph(
    para: &CT_P,
    available_width: f64,
    styles: &CT_Styles,
    input: &LayoutInput,
    media: &MediaRegistry,
    fm: &mut FontManager,
    num_state: &mut NumberingState,
) -> Result<ParagraphBlock> {
    // Resolve paragraph properties
    let para_style_id = para.properties.as_ref().and_then(|p| p.style_id.as_deref());

    let resolved_ppr = style_resolver::resolve_paragraph_properties(para_style_id, styles);

    let mut effective_ppr = resolved_ppr;

    // A numbering level carries paragraph properties of its own, mainly the
    // indentation for that level. They sit between the style and direct
    // formatting, so merge them before the direct properties rather than
    // after. Without this every level of a list draws at the same indent.
    let direct_ppr = para.properties.as_ref();
    let list_num_id = direct_ppr.and_then(|p| p.num_id).or(effective_ppr.num_id);
    let list_ilvl = direct_ppr
        .and_then(|p| p.num_ilvl)
        .or(effective_ppr.num_ilvl)
        .unwrap_or(0);
    if let (Some(num_id), Some(numbering)) = (list_num_id, input.numbering.as_ref())
        && let Some(lvl_ppr) =
            style_resolver::level_paragraph_properties(num_id, list_ilvl, numbering)
    {
        merge_direct_ppr(&mut effective_ppr, lvl_ppr);
    }

    // Merge direct paragraph properties
    if let Some(direct_ppr) = direct_ppr {
        merge_direct_ppr(&mut effective_ppr, direct_ppr);
    }

    // Convert paragraph properties to layout values
    let space_before = effective_ppr.space_before.map(|t| t.to_pt()).unwrap_or(0.0);
    let space_after = effective_ppr.space_after.map(|t| t.to_pt()).unwrap_or(0.0);
    let ind_left = effective_ppr.ind_left.map(|t| t.to_pt()).unwrap_or(0.0);
    let ind_right = effective_ppr.ind_right.map(|t| t.to_pt()).unwrap_or(0.0);
    let keep_next = effective_ppr.keep_next.unwrap_or(false);
    let keep_lines = effective_ppr.keep_lines.unwrap_or(false);
    let page_break_before = effective_ppr.page_break_before.unwrap_or(false);
    let widow_control = effective_ppr.widow_control.unwrap_or(true);
    let jc = convert::alignment(effective_ppr.jc);

    // Parse shading color
    let shading = effective_ppr
        .shading
        .as_ref()
        .and_then(|shd| shd.fill.as_ref())
        .filter(|f| f != &"auto")
        .map(|f| Color::from_hex(f));

    // Convert runs to inline items
    let mut inline_items = Vec::new();

    // Handle numbering marker
    if let (Some(num_id), Some(numbering)) = (effective_ppr.num_id, input.numbering.as_ref()) {
        let ilvl = effective_ppr.num_ilvl.unwrap_or(0);
        if let Some(marker) = style_resolver::generate_marker(num_id, ilvl, numbering, num_state) {
            // Shape the marker text
            let marker_rpr = marker.marker_rpr;
            let marker_font_size = marker_rpr.sz.map(|hp| hp.to_pt()).unwrap_or_else(|| {
                style_resolver::resolve_run_properties(para_style_id, None, styles)
                    .sz
                    .map(|hp| hp.to_pt())
                    .unwrap_or(11.0)
            });
            let marker_bold = marker_rpr.bold.unwrap_or(false);
            let marker_italic = marker_rpr.italic.unwrap_or(false);
            let marker_font_family = marker_rpr.font_ascii.as_deref();

            // Bullet glyphs are not in every font either, so the marker gets
            // the same coverage check as body text.
            if let Ok(font_id) = fm.resolve_font_for_text(
                marker_font_family,
                marker_bold,
                marker_italic,
                &marker.marker_text,
            ) && let Ok(shaped) = fm.shape_text(font_id, &marker.marker_text, marker_font_size)
            {
                let metrics = fm.metrics(font_id, marker_font_size)?;
                let color = marker_rpr
                    .color
                    .as_ref()
                    .map(|c| Color::from_hex(c))
                    .unwrap_or(Color::BLACK);

                inline_items.push(InlineItem::Marker(TextSegment {
                    text: marker.marker_text,
                    font_id,
                    font_size: marker_font_size,
                    glyph_ids: shaped.glyph_ids,
                    advances: shaped.advances,
                    width: shaped.width,
                    ascent: metrics.ascent,
                    descent: metrics.descent,
                    line_gap: 0.0,
                    color,
                    bold: marker_bold,
                    italic: marker_italic,
                    underline: None,
                    strike: false,
                    dstrike: false,
                    highlight: None,
                    baseline_offset: 0.0,
                    hyperlink_url: None,
                    field_kind: None,
                    footnote_id: None,
                }));

                // Add a space/tab after the marker
                inline_items.push(InlineItem::Tab);
            }
        }
    }

    // Build hyperlink URL map: run index → URL
    let mut run_hyperlink_url: std::collections::HashMap<usize, String> =
        std::collections::HashMap::new();
    for hl in &para.hyperlinks {
        if let Some(ref rel_id) = hl.rel_id
            && let Some(url) = input.hyperlink_urls.get(rel_id)
        {
            for run_idx in hl.run_start..hl.run_end {
                run_hyperlink_url.insert(run_idx, url.clone());
            }
        }
    }

    // Process runs
    for (run_idx, run) in para.runs.iter().enumerate() {
        let current_hyperlink_url = run_hyperlink_url.get(&run_idx).cloned();

        let run_style_id = run.properties.as_ref().and_then(|p| p.style_id.as_deref());

        let resolved_rpr =
            style_resolver::resolve_run_properties(para_style_id, run_style_id, styles);

        // Merge direct run properties
        let mut effective_rpr = resolved_rpr;
        if let Some(ref direct_rpr) = run.properties {
            effective_rpr.merge_from(direct_rpr);
        }

        // Skip hidden text
        if effective_rpr.vanish == Some(true) {
            continue;
        }

        let mut font_size = effective_rpr.sz.map(|hp| hp.to_pt()).unwrap_or(11.0);
        let bold = effective_rpr.bold.unwrap_or(false);
        let italic = effective_rpr.italic.unwrap_or(false);

        // Resolve font family: theme font takes priority when no explicit font is set
        let font_family = resolve_font_family(&effective_rpr, input.theme.as_ref());

        // Resolve color: theme color takes priority over literal color value
        let color = resolve_run_color(&effective_rpr, input.theme.as_ref());

        // Decoration properties
        let underline = convert::underline(effective_rpr.underline);
        let strike = effective_rpr.strike.unwrap_or(false);
        let dstrike = effective_rpr.dstrike.unwrap_or(false);
        let highlight = effective_rpr.highlight.and_then(highlight_to_color);

        // Superscript/subscript handling
        let mut baseline_offset = 0.0;
        if let Some(ref va) = effective_rpr.vert_align {
            match va.as_str() {
                "superscript" => {
                    // Reduce font size to ~58% and raise baseline
                    let original_size = font_size;
                    font_size *= 0.58;
                    baseline_offset = original_size * 0.33; // raise by 1/3 of original size
                }
                "subscript" => {
                    // Reduce font size to ~58% and lower baseline
                    let original_size = font_size;
                    font_size *= 0.58;
                    baseline_offset = -(original_size * 0.14); // lower
                }
                _ => {}
            }
        }

        // Position offset (in half-points, positive=raise)
        if let Some(pos) = effective_rpr.position {
            baseline_offset += pos as f64 / 2.0; // half-points to points
        }

        // Resolved against the run's own text, so a family without glyphs for
        // this script is replaced by one that has them.
        let font_id =
            fm.resolve_font_for_text(font_family.as_deref(), bold, italic, &run.text())?;
        let metrics = fm.metrics(font_id, font_size)?;

        for content in &run.content {
            match content {
                RunContent::Text(ct_text) => {
                    let text = if effective_rpr.caps == Some(true) {
                        ct_text.text.to_uppercase()
                    } else {
                        ct_text.text.clone()
                    };

                    if text.is_empty() {
                        continue;
                    }

                    let mut shaped = fm.shape_text(font_id, &text, font_size)?;

                    // Apply character spacing from run properties (in twips)
                    if let Some(spacing) = effective_rpr.spacing {
                        let extra = spacing.to_pt();
                        for advance in &mut shaped.advances {
                            *advance += extra;
                        }
                        shaped.width += extra * shaped.advances.len() as f64;
                    }

                    inline_items.extend(convert::text_segments(TextSegment {
                        text,
                        font_id,
                        font_size,
                        glyph_ids: shaped.glyph_ids,
                        advances: shaped.advances,
                        width: shaped.width,
                        ascent: metrics.ascent,
                        descent: metrics.descent,
                        line_gap: 0.0,
                        color,
                        bold,
                        italic,
                        underline,
                        strike,
                        dstrike,
                        highlight,
                        baseline_offset,
                        hyperlink_url: current_hyperlink_url.clone(),
                        field_kind: None,
                        footnote_id: None,
                    }));
                }
                RunContent::Tab => {
                    inline_items.push(InlineItem::Tab);
                }
                RunContent::Break(bt) => match bt {
                    BreakType::Line => inline_items.push(InlineItem::LineBreak),
                    BreakType::Page => inline_items.push(InlineItem::PageBreak),
                    BreakType::Column => inline_items.push(InlineItem::ColumnBreak),
                },
                RunContent::Drawing(drawing) => {
                    if let Some(ref inline) = drawing.inline {
                        let width = inline.extent_cx.to_pt();
                        let height = inline.extent_cy.to_pt();
                        inline_items.push(InlineItem::Image {
                            width,
                            height,
                            media_id: media.id_for_relationship(&inline.embed_id),
                        });
                    }
                }
                RunContent::Field { field_type } => {
                    // Shape a placeholder ("99") for estimated width
                    let placeholder = "99";
                    let fk = match field_type {
                        FieldType::Page => FieldKind::Page,
                        FieldType::NumPages => FieldKind::NumPages,
                        FieldType::Other(_) => continue, // skip unsupported fields
                    };
                    let shaped = fm.shape_text(font_id, placeholder, font_size)?;
                    inline_items.push(InlineItem::Text(TextSegment {
                        text: placeholder.to_string(),
                        font_id,
                        font_size,
                        glyph_ids: shaped.glyph_ids,
                        advances: shaped.advances,
                        width: shaped.width,
                        ascent: metrics.ascent,
                        descent: metrics.descent,
                        line_gap: 0.0,
                        color,
                        bold,
                        italic,
                        underline: None,
                        strike: false,
                        dstrike: false,
                        highlight: None,
                        baseline_offset,
                        hyperlink_url: None,
                        field_kind: Some(fk),
                        footnote_id: None,
                    }));
                }
                RunContent::FootnoteRef { id } | RunContent::EndnoteRef { id } => {
                    // Render as superscript number
                    let marker = id.to_string();
                    let sup_size = font_size * 0.58;
                    let sup_offset = font_size * 0.33; // raise baseline
                    let shaped = fm.shape_text(font_id, &marker, sup_size)?;
                    let sup_metrics = fm.metrics(font_id, sup_size)?;
                    inline_items.push(InlineItem::Text(TextSegment {
                        text: marker,
                        font_id,
                        font_size: sup_size,
                        glyph_ids: shaped.glyph_ids,
                        advances: shaped.advances,
                        width: shaped.width,
                        ascent: sup_metrics.ascent,
                        descent: sup_metrics.descent,
                        line_gap: 0.0,
                        color,
                        bold,
                        italic,
                        underline: None,
                        strike: false,
                        dstrike: false,
                        highlight: None,
                        baseline_offset: sup_offset,
                        hyperlink_url: None,
                        field_kind: None,
                        footnote_id: Some(*id),
                    }));
                }
            }
        }
    }

    // Line breaking
    let line_params = convert::line_break_params(&effective_ppr, available_width);

    let mut lines = break_into_lines(&inline_items, &line_params, fm)?;
    convert::restore_word_line_heights(&mut lines, &effective_ppr);

    let mut result = block::build_paragraph_block(
        lines,
        space_before,
        space_after,
        effective_ppr.borders,
        shading,
        ind_left,
        ind_right,
        jc,
        keep_next,
        keep_lines,
        page_break_before,
        widow_control,
    );
    result.anchored = collect_anchored_drawings(para, styles, input, media, fm, num_state)?;
    Ok(result)
}

/// Collect the floating drawings anchored to a paragraph.
///
/// The offsets stay paired with the frame they are measured from. Resolving
/// them here is not possible: a paragraph-relative offset needs the laid-out
/// position of the paragraph, which only the paginator knows.
///
/// A shape's text box is laid out here rather than later, because breaking it
/// into lines needs the font manager.
fn collect_anchored_drawings(
    para: &CT_P,
    styles: &CT_Styles,
    input: &LayoutInput,
    media: &MediaRegistry,
    fm: &mut FontManager,
    num_state: &mut NumberingState,
) -> Result<Vec<block::AnchoredDrawing>> {
    let mut out = Vec::new();

    // Drawings written plainly, and drawings recovered from an
    // mc:AlternateContent block, are both anchored the same way.
    for run in &para.runs {
        let plain = run.content.iter().filter_map(|rc| match rc {
            RunContent::Drawing(d) => Some(d),
            _ => None,
        });
        for drawing in plain.chain(run.alt_drawings.iter()) {
            let Some(anchor) = drawing.anchor.as_ref() else {
                continue;
            };

            // A picture also carries a pic:spPr, so a parsed shape alone does
            // not mean this is a shape. An embed id is what makes it a
            // picture, and that takes precedence.
            let shape = if anchor.embed_id.is_empty() {
                anchor.shape.as_ref()
            } else {
                None
            };

            let content = match shape {
                Some(shape) => {
                    // A shape's text box wraps at the shape width.
                    let mut text = Vec::new();
                    for p in &shape.text {
                        text.push(layout_paragraph(
                            p,
                            anchor.extent_cx.to_pt(),
                            styles,
                            input,
                            media,
                            fm,
                            num_state,
                        )?);
                    }
                    block::AnchoredContent::Shape {
                        preset: block::ShapePreset::from_prst(shape.preset.as_deref()),
                        fill: shape.solid_fill.as_deref().map(Color::from_hex),
                        text,
                    }
                }
                None if anchor.embed_id.is_empty() => continue,
                None => block::AnchoredContent::Image {
                    media_id: media.id_for_relationship(&anchor.embed_id),
                },
            };

            out.push(block::AnchoredDrawing {
                behind_doc: anchor.behind_doc,
                rel_h: anchor.pos_h_relative_from,
                off_h: anchor.pos_h_offset.to_pt(),
                rel_v: anchor.pos_v_relative_from,
                off_v: anchor.pos_v_offset.to_pt(),
                width: anchor.extent_cx.to_pt(),
                height: anchor.extent_cy.to_pt(),
                content,
            });
        }
    }
    Ok(out)
}

/// Merge direct paragraph properties (only fields explicitly set in the XML).
fn merge_direct_ppr(effective: &mut CT_PPr, direct: &CT_PPr) {
    // Don't merge style_id — that was already used for resolution
    if direct.jc.is_some() {
        effective.jc = direct.jc;
    }
    if direct.space_before.is_some() {
        effective.space_before = direct.space_before;
    }
    if direct.space_after.is_some() {
        effective.space_after = direct.space_after;
    }
    if direct.line_spacing.is_some() {
        effective.line_spacing = direct.line_spacing;
    }
    if direct.line_rule.is_some() {
        effective.line_rule = direct.line_rule.clone();
    }
    if direct.ind_left.is_some() {
        effective.ind_left = direct.ind_left;
    }
    if direct.ind_right.is_some() {
        effective.ind_right = direct.ind_right;
    }
    if direct.ind_first_line.is_some() {
        effective.ind_first_line = direct.ind_first_line;
    }
    if direct.ind_hanging.is_some() {
        effective.ind_hanging = direct.ind_hanging;
    }
    if direct.keep_next.is_some() {
        effective.keep_next = direct.keep_next;
    }
    if direct.keep_lines.is_some() {
        effective.keep_lines = direct.keep_lines;
    }
    if direct.page_break_before.is_some() {
        effective.page_break_before = direct.page_break_before;
    }
    if direct.widow_control.is_some() {
        effective.widow_control = direct.widow_control;
    }
    if direct.borders.is_some() {
        effective.borders = direct.borders.clone();
    }
    if direct.tabs.is_some() {
        effective.tabs = direct.tabs.clone();
    }
    if direct.shading.is_some() {
        effective.shading = direct.shading.clone();
    }
    if direct.num_id.is_some() {
        effective.num_id = direct.num_id;
    }
    if direct.num_ilvl.is_some() {
        effective.num_ilvl = direct.num_ilvl;
    }
}

/// Convert section properties to page geometry.
fn sect_pr_to_geometry(sect_pr: &CT_SectPr) -> PageGeometry {
    PageGeometry {
        page_width: sect_pr.page_width.map(|t| t.to_pt()).unwrap_or(612.0),
        page_height: sect_pr.page_height.map(|t| t.to_pt()).unwrap_or(792.0),
        margin_top: sect_pr.margin_top.map(|t| t.to_pt()).unwrap_or(72.0),
        margin_right: sect_pr.margin_right.map(|t| t.to_pt()).unwrap_or(72.0),
        margin_bottom: sect_pr.margin_bottom.map(|t| t.to_pt()).unwrap_or(72.0),
        margin_left: sect_pr.margin_left.map(|t| t.to_pt()).unwrap_or(72.0),
        header_distance: sect_pr.header_distance.map(|t| t.to_pt()).unwrap_or(36.0),
        footer_distance: sect_pr.footer_distance.map(|t| t.to_pt()).unwrap_or(36.0),
    }
}

/// Lay out header and footer content (both Default and First-page).
fn layout_header_footer(
    sect_pr: &CT_SectPr,
    input: &LayoutInput,
    styles: &CT_Styles,
    media: &MediaRegistry,
    fm: &mut FontManager,
    num_state: &mut NumberingState,
) -> Result<Option<HeaderFooterContent>> {
    let mut has_content = false;
    let mut header_blocks = Vec::new();
    let mut footer_blocks = Vec::new();
    let mut first_header_blocks = Vec::new();
    let mut first_footer_blocks = Vec::new();

    let geometry = sect_pr_to_geometry(sect_pr);
    let width = geometry.content_width();

    for href in &sect_pr.header_refs {
        let target_blocks = match href.hdr_ftr_type {
            HdrFtrType::Default => &mut header_blocks,
            HdrFtrType::First => &mut first_header_blocks,
            _ => continue, // skip Even for now
        };
        if let Some(hdr) = input.headers.get(&href.rel_id) {
            for para in &hdr.paragraphs {
                let block = layout_paragraph(para, width, styles, input, media, fm, num_state)?;
                target_blocks.push(block);
            }
            has_content = true;
        }
    }

    for fref in &sect_pr.footer_refs {
        let target_blocks = match fref.hdr_ftr_type {
            HdrFtrType::Default => &mut footer_blocks,
            HdrFtrType::First => &mut first_footer_blocks,
            _ => continue, // skip Even for now
        };
        if let Some(ftr) = input.footers.get(&fref.rel_id) {
            for para in &ftr.paragraphs {
                let block = layout_paragraph(para, width, styles, input, media, fm, num_state)?;
                target_blocks.push(block);
            }
            has_content = true;
        }
    }

    if has_content {
        Ok(Some(HeaderFooterContent {
            header_blocks,
            footer_blocks,
            first_header_blocks,
            first_footer_blocks,
        }))
    } else {
        Ok(None)
    }
}

/// Resolve the effective font family for a run, considering theme fonts.
///
/// Priority: explicit font_ascii > theme font > None (use default).
fn resolve_font_family(
    rpr: &rdocx_oxml::properties::CT_RPr,
    theme: Option<&rdocx_oxml::theme::Theme>,
) -> Option<String> {
    // Explicit font name takes priority
    if rpr.font_ascii.is_some() {
        return rpr.font_ascii.clone();
    }

    // Resolve theme font reference
    if let (Some(theme_ref), Some(theme)) = (&rpr.font_ascii_theme, theme) {
        let font = match theme_ref.as_str() {
            "majorAscii" | "majorHAnsi" | "majorBidi" | "majorEastAsia" => {
                theme.major_font.as_deref()
            }
            "minorAscii" | "minorHAnsi" | "minorBidi" | "minorEastAsia" => {
                theme.minor_font.as_deref()
            }
            _ => None,
        };
        if let Some(f) = font {
            return Some(f.to_string());
        }
    }

    None
}

/// Resolve the effective color for a run, considering theme colors.
///
/// Priority: literal color (non-auto) > theme color > black.
fn resolve_run_color(
    rpr: &rdocx_oxml::properties::CT_RPr,
    theme: Option<&rdocx_oxml::theme::Theme>,
) -> Color {
    // If theme color is specified, resolve it from the theme
    if let Some(ref theme_name) = rpr.color_theme
        && let Some(theme) = theme
        && let Some(hex) = theme.colors.get(theme_name)
    {
        return Color::from_hex(hex);
    }

    // Fall back to literal color value
    rpr.color
        .as_ref()
        .filter(|c| c.as_str() != "auto")
        .map(|c| Color::from_hex(c))
        .unwrap_or(Color::BLACK)
}

/// Convert a highlight color enum to an RGBA Color.
fn highlight_to_color(h: ST_HighlightColor) -> Option<Color> {
    match h {
        ST_HighlightColor::None => None,
        ST_HighlightColor::Black => Some(Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }),
        ST_HighlightColor::Blue => Some(Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }),
        ST_HighlightColor::Cyan => Some(Color {
            r: 0.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }),
        ST_HighlightColor::DarkBlue => Some(Color {
            r: 0.0,
            g: 0.0,
            b: 0.545,
            a: 1.0,
        }),
        ST_HighlightColor::DarkCyan => Some(Color {
            r: 0.0,
            g: 0.545,
            b: 0.545,
            a: 1.0,
        }),
        ST_HighlightColor::DarkGray => Some(Color {
            r: 0.663,
            g: 0.663,
            b: 0.663,
            a: 1.0,
        }),
        ST_HighlightColor::DarkGreen => Some(Color {
            r: 0.0,
            g: 0.392,
            b: 0.0,
            a: 1.0,
        }),
        ST_HighlightColor::DarkMagenta => Some(Color {
            r: 0.545,
            g: 0.0,
            b: 0.545,
            a: 1.0,
        }),
        ST_HighlightColor::DarkRed => Some(Color {
            r: 0.545,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }),
        ST_HighlightColor::DarkYellow => Some(Color {
            r: 0.545,
            g: 0.545,
            b: 0.0,
            a: 1.0,
        }),
        ST_HighlightColor::Green => Some(Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }),
        ST_HighlightColor::LightGray => Some(Color {
            r: 0.827,
            g: 0.827,
            b: 0.827,
            a: 1.0,
        }),
        ST_HighlightColor::Magenta => Some(Color {
            r: 1.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }),
        ST_HighlightColor::Red => Some(Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }),
        ST_HighlightColor::White => Some(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }),
        ST_HighlightColor::Yellow => Some(Color {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::ImageData;
    use oxml_layout::MediaId;
    use std::collections::HashMap;

    fn make_input_with_text(text: &str) -> LayoutInput {
        let mut doc = rdocx_oxml::document::CT_Document::new();
        let mut p = CT_P::new();
        p.add_run(text);
        doc.body.add_paragraph(p);

        LayoutInput {
            document: doc,
            styles: CT_Styles::new_default(),
            numbering: None,
            headers: HashMap::new(),
            footers: HashMap::new(),
            images: HashMap::new(),
            core_properties: None,
            hyperlink_urls: HashMap::new(),
            footnotes: None,
            endnotes: None,
            theme: None,
            fonts: Vec::new(),
        }
    }

    #[test]
    fn layout_simple_document() {
        let input = make_input_with_text("Hello World");
        let result = Engine::new().layout(&input);
        // On systems without fonts, this may fail — that's OK
        if let Ok(result) = result {
            assert!(!result.pages.is_empty());
            assert_eq!(result.pages[0].page_number, 1);
            assert!((result.pages[0].width - 612.0).abs() < 0.01);
        }
    }

    #[test]
    fn layout_empty_document() {
        let mut doc = rdocx_oxml::document::CT_Document::new();
        doc.body.add_paragraph(CT_P::new());

        let input = LayoutInput {
            document: doc,
            styles: CT_Styles::new_default(),
            numbering: None,
            headers: HashMap::new(),
            footers: HashMap::new(),
            images: HashMap::new(),
            core_properties: None,
            hyperlink_urls: HashMap::new(),
            footnotes: None,
            endnotes: None,
            theme: None,
            fonts: Vec::new(),
        };

        let result = Engine::new().layout(&input);
        if let Ok(result) = result {
            assert_eq!(result.pages.len(), 1);
        }
    }

    #[test]
    fn empty_shapeless_anchor_keeps_the_pre_cutover_omission() {
        let input = make_input_with_text("");
        let mut paragraph = CT_P::new();
        paragraph.add_run("").content = vec![RunContent::Drawing(
            rdocx_oxml::drawing::CT_Drawing::anchor(rdocx_oxml::drawing::CT_Anchor::background(
                "", 914_400, 914_400,
            )),
        )];
        let mut font_manager = FontManager::new();
        let mut numbering_state = NumberingState::new();
        let media = MediaRegistry::new(&input.images);

        let anchored = collect_anchored_drawings(
            &paragraph,
            &input.styles,
            &input,
            &media,
            &mut font_manager,
            &mut numbering_state,
        )
        .expect("empty shapeless anchor collection should succeed");

        assert!(anchored.is_empty());
    }

    #[test]
    fn colliding_media_ids_keep_inline_and_anchored_image_bytes_distinct() {
        let mut input = make_input_with_text("");
        input.images.insert(
            "rIdInline".to_string(),
            ImageData {
                data: vec![1, 2, 3],
                content_type: "image/png".to_string(),
            },
        );
        input.images.insert(
            "rIdAnchor".to_string(),
            ImageData {
                data: vec![4, 5, 6],
                content_type: "image/jpeg".to_string(),
            },
        );

        let media = MediaRegistry::with_hasher(&input.images, |_| MediaId(7));
        let inline_id = media.id_for_relationship("rIdInline");
        let anchor_id = media.id_for_relationship("rIdAnchor");
        assert_ne!(inline_id, anchor_id);

        let line = oxml_layout::LayoutLine {
            items: vec![LineItem::Image {
                width: 12.0,
                height: 10.0,
                media_id: inline_id,
            }],
            width: 12.0,
            ascent: 10.0,
            descent: 0.0,
            line_gap: 0.0,
            height: 10.0,
            indent_left: 0.0,
            available_width: 468.0,
            is_last: true,
        };
        let mut paragraph = block::build_paragraph_block(
            vec![line],
            0.0,
            0.0,
            None,
            None,
            0.0,
            0.0,
            None,
            false,
            false,
            false,
            true,
        );
        paragraph.anchored.push(block::AnchoredDrawing {
            behind_doc: false,
            rel_h: rdocx_oxml::drawing::ST_RelativeFromH::Page,
            off_h: 20.0,
            rel_v: rdocx_oxml::drawing::ST_RelativeFromV::Page,
            off_v: 20.0,
            width: 12.0,
            height: 10.0,
            content: block::AnchoredContent::Image {
                media_id: anchor_id,
            },
        });
        let sections = [paginator::Section {
            blocks: vec![LayoutBlock::Paragraph(paragraph)],
            geometry: PageGeometry::default(),
            header_footer: None,
            title_pg: false,
        }];

        let (pages, _) = paginator::paginate_sections(&sections, &FontManager::new(), &media);
        let images = pages[0]
            .elements
            .iter()
            .filter_map(|element| match element {
                PositionedElement::Image {
                    data,
                    content_type,
                    media_id,
                    ..
                } => Some((data.as_slice(), content_type.as_str(), *media_id)),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(images.contains(&(b"\x01\x02\x03".as_slice(), "image/png", inline_id)));
        assert!(images.contains(&(b"\x04\x05\x06".as_slice(), "image/jpeg", anchor_id)));
    }

    #[test]
    fn layout_with_heading_style() {
        let mut doc = rdocx_oxml::document::CT_Document::new();
        let mut p = CT_P::new();
        p.properties = Some(CT_PPr {
            style_id: Some("Heading1".to_string()),
            ..Default::default()
        });
        p.add_run("Chapter 1");
        doc.body.add_paragraph(p);

        let input = LayoutInput {
            document: doc,
            styles: CT_Styles::new_default(),
            numbering: None,
            headers: HashMap::new(),
            footers: HashMap::new(),
            images: HashMap::new(),
            core_properties: None,
            hyperlink_urls: HashMap::new(),
            footnotes: None,
            endnotes: None,
            theme: None,
            fonts: Vec::new(),
        };

        let result = Engine::new().layout(&input);
        if let Ok(result) = result {
            assert!(!result.pages.is_empty());
            // Should produce one outline entry for Heading1
            assert_eq!(result.outlines.len(), 1);
            assert_eq!(result.outlines[0].title, "Chapter 1");
            assert_eq!(result.outlines[0].level, 1);
            assert_eq!(result.outlines[0].page_index, 0);
        }
    }

    #[test]
    fn layout_nested_headings_produce_outlines() {
        let mut doc = rdocx_oxml::document::CT_Document::new();

        // H1
        let mut h1 = CT_P::new();
        h1.properties = Some(CT_PPr {
            style_id: Some("Heading1".to_string()),
            ..Default::default()
        });
        h1.add_run("Chapter 1");
        doc.body.add_paragraph(h1);

        // H2 under H1
        let mut h2 = CT_P::new();
        h2.properties = Some(CT_PPr {
            style_id: Some("Heading2".to_string()),
            ..Default::default()
        });
        h2.add_run("Section 1.1");
        doc.body.add_paragraph(h2);

        // Another H1
        let mut h1b = CT_P::new();
        h1b.properties = Some(CT_PPr {
            style_id: Some("Heading1".to_string()),
            ..Default::default()
        });
        h1b.add_run("Chapter 2");
        doc.body.add_paragraph(h1b);

        let input = LayoutInput {
            document: doc,
            styles: CT_Styles::new_default(),
            numbering: None,
            headers: HashMap::new(),
            footers: HashMap::new(),
            images: HashMap::new(),
            core_properties: None,
            hyperlink_urls: HashMap::new(),
            footnotes: None,
            endnotes: None,
            theme: None,
            fonts: Vec::new(),
        };

        let result = Engine::new().layout(&input);
        if let Ok(result) = result {
            assert_eq!(result.outlines.len(), 3);
            assert_eq!(result.outlines[0].level, 1);
            assert_eq!(result.outlines[0].title, "Chapter 1");
            assert_eq!(result.outlines[1].level, 2);
            assert_eq!(result.outlines[1].title, "Section 1.1");
            assert_eq!(result.outlines[2].level, 1);
            assert_eq!(result.outlines[2].title, "Chapter 2");
        }
    }

    #[test]
    fn sect_pr_geometry_conversion() {
        let sect = CT_SectPr::default_letter();
        let geom = sect_pr_to_geometry(&sect);
        assert!((geom.page_width - 612.0).abs() < 0.01);
        assert!((geom.page_height - 792.0).abs() < 0.01);
        assert!((geom.margin_top - 72.0).abs() < 0.01);
        assert!((geom.content_width() - 468.0).abs() < 0.01);
    }

    #[test]
    fn sect_pr_a4_geometry() {
        let sect = CT_SectPr::default_a4();
        let geom = sect_pr_to_geometry(&sect);
        // A4: 210mm = 595.3pt, 297mm = 841.9pt
        assert!((geom.page_width - 595.3).abs() < 0.5);
        assert!((geom.page_height - 841.9).abs() < 0.5);
    }

    // F-X013a, footnote line advance.

    /// Build a document whose single body paragraph references footnote 1, and
    /// whose footnote 1 is one paragraph made of `note_runs` separate runs.
    fn make_input_with_footnote(note_runs: &[&str]) -> LayoutInput {
        use rdocx_oxml::footnotes::{CT_Footnote, CT_Footnotes};
        use rdocx_oxml::text::CT_R;

        let mut doc = rdocx_oxml::document::CT_Document::new();
        let mut body = CT_P::new();
        body.add_run("Body text carrying a note");
        let mut marker_run = CT_R::new("");
        marker_run.content = vec![RunContent::FootnoteRef { id: 1 }];
        body.runs.push(marker_run);
        doc.body.add_paragraph(body);

        let mut note = CT_P::new();
        for text in note_runs {
            note.add_run(text);
        }

        LayoutInput {
            document: doc,
            styles: CT_Styles::new_default(),
            numbering: None,
            headers: HashMap::new(),
            footers: HashMap::new(),
            images: HashMap::new(),
            core_properties: None,
            hyperlink_urls: HashMap::new(),
            footnotes: Some(CT_Footnotes {
                footnotes: vec![CT_Footnote {
                    id: 1,
                    paragraphs: vec![note],
                }],
            }),
            endnotes: None,
            theme: None,
            fonts: Vec::new(),
        }
    }

    /// The x origin of every glyph run sitting below the footnote separator,
    /// in the order the renderer emitted them. The first is the note marker.
    fn footnote_glyph_x(page: &oxml_layout::output::PageFrame) -> Vec<f64> {
        let separator_y = page
            .elements
            .iter()
            .find_map(|element| match element {
                PositionedElement::Line { start, .. } => Some(start.y),
                _ => None,
            })
            .expect("a page with a footnote draws a separator line");

        page.elements
            .iter()
            .filter_map(|element| match element {
                PositionedElement::Text(run) if run.origin.y > separator_y => Some(run.origin.x),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_multi_segment_footnote_does_not_stack_its_segments_at_one_x() {
        let input = make_input_with_footnote(&["Alpha", "Beta", "Gamma"]);
        let mut engine = Engine::new();
        let output = engine.layout(&input).expect("layout succeeds");
        let xs = footnote_glyph_x(&output.pages[0]);

        assert!(
            xs.len() >= 4,
            "expected a marker and three note segments, got {xs:?}"
        );
        for pair in xs.windows(2) {
            assert!(
                pair[1] > pair[0],
                "footnote segments must advance, got {xs:?}"
            );
        }
    }

    #[test]
    fn a_single_segment_footnote_keeps_its_original_position() {
        let input = make_input_with_footnote(&["Solitary"]);
        let mut engine = Engine::new();
        let output = engine.layout(&input).expect("layout succeeds");
        let xs = footnote_glyph_x(&output.pages[0]);
        let geometry = sect_pr_to_geometry(&CT_SectPr::default_letter());

        // The marker sits at the left margin, the single segment one indent in.
        assert_eq!(xs.len(), 2, "expected a marker and one segment, got {xs:?}");
        assert!(
            (xs[0] - geometry.margin_left).abs() < 0.01,
            "marker at {xs:?}"
        );
        assert!(
            (xs[1] - (geometry.margin_left + 12.0)).abs() < 0.01,
            "segment at {xs:?}"
        );
    }

    #[test]
    fn a_long_footnote_does_not_overrun_the_right_margin() {
        // Long enough to wrap, which is what exposes a break width that
        // disagrees with the indent the note is drawn at.
        let long = "In paged media, footnotes are usually displayed at the \
                    bottom of the text. However, in ebooks, a better paradigm \
                    is to make them clickable endnotes that the reader can \
                    browse at leisure, which this sentence exists to force.";
        let input = make_input_with_footnote(&[long]);
        let mut engine = Engine::new();
        let output = engine.layout(&input).expect("layout succeeds");
        let page = &output.pages[0];
        let geometry = sect_pr_to_geometry(&CT_SectPr::default_letter());
        let right_margin = geometry.page_width - geometry.margin_right;

        let separator_y = page
            .elements
            .iter()
            .find_map(|element| match element {
                PositionedElement::Line { start, .. } => Some(start.y),
                _ => None,
            })
            .expect("a page with a footnote draws a separator line");

        let mut wrapped = false;
        let mut first_y = None;
        for element in &page.elements {
            let PositionedElement::Text(run) = element else {
                continue;
            };
            if run.origin.y <= separator_y {
                continue;
            }
            let first = *first_y.get_or_insert(run.origin.y);
            if run.origin.y > first + 0.01 {
                wrapped = true;
            }
            let right_edge = run.origin.x + run.advances.iter().sum::<f64>();
            assert!(
                right_edge <= right_margin + 0.01,
                "note text reaches {right_edge}, past the right margin {right_margin}"
            );
        }
        assert!(wrapped, "the note must wrap for this test to mean anything");
    }

    #[test]
    fn a_tab_inside_a_footnote_still_advances_the_text_after_it() {
        use rdocx_oxml::footnotes::{CT_Footnote, CT_Footnotes};
        use rdocx_oxml::text::CT_R;

        // Two notes differing only by a tab between their runs. The tab is not
        // drawn, but it occupies width, so the run after it must shift right.
        let build = |with_tab: bool| {
            let mut doc = rdocx_oxml::document::CT_Document::new();
            let mut body = CT_P::new();
            body.add_run("Body");
            let mut marker_run = CT_R::new("");
            marker_run.content = vec![RunContent::FootnoteRef { id: 1 }];
            body.runs.push(marker_run);
            doc.body.add_paragraph(body);

            let mut note = CT_P::new();
            note.add_run("Alpha");
            if with_tab {
                let mut tab_run = CT_R::new("");
                tab_run.content = vec![RunContent::Tab];
                note.runs.push(tab_run);
            }
            note.add_run("Beta");

            LayoutInput {
                document: doc,
                styles: CT_Styles::new_default(),
                numbering: None,
                headers: HashMap::new(),
                footers: HashMap::new(),
                images: HashMap::new(),
                core_properties: None,
                hyperlink_urls: HashMap::new(),
                footnotes: Some(CT_Footnotes {
                    footnotes: vec![CT_Footnote {
                        id: 1,
                        paragraphs: vec![note],
                    }],
                }),
                endnotes: None,
                theme: None,
                fonts: Vec::new(),
            }
        };

        let mut engine = Engine::new();
        let plain = engine.layout(&build(false)).expect("layout succeeds");
        let tabbed = engine.layout(&build(true)).expect("layout succeeds");

        let plain_x = footnote_glyph_x(&plain.pages[0]);
        let tabbed_x = footnote_glyph_x(&tabbed.pages[0]);

        // Marker and both runs are drawn in each case. The tab draws nothing.
        assert_eq!(plain_x.len(), 3, "plain note glyphs {plain_x:?}");
        assert_eq!(tabbed_x.len(), 3, "tabbed note glyphs {tabbed_x:?}");
        assert!(
            tabbed_x[2] > plain_x[2] + 1.0,
            "the run after a tab must shift right, plain {plain_x:?} tabbed {tabbed_x:?}"
        );
    }

    #[test]
    fn footnote_segment_advance_matches_body_segment_advance() {
        let input = make_input_with_footnote(&["Alpha", "Beta", "Gamma"]);
        let mut engine = Engine::new();
        let output = engine.layout(&input).expect("layout succeeds");
        let page = &output.pages[0];

        let separator_y = page
            .elements
            .iter()
            .find_map(|element| match element {
                PositionedElement::Line { start, .. } => Some(start.y),
                _ => None,
            })
            .expect("a page with a footnote draws a separator line");

        // Gaps between consecutive note segments must equal the width of the
        // segment that precedes them, which is what the body path advances by.
        let notes: Vec<&GlyphRun> = page
            .elements
            .iter()
            .filter_map(|element| match element {
                PositionedElement::Text(run) if run.origin.y > separator_y => Some(run),
                _ => None,
            })
            .skip(1) // the marker, which is positioned independently
            .collect();

        assert_eq!(notes.len(), 3, "expected three note segments");
        for pair in notes.windows(2) {
            let advance: f64 = pair[0].advances.iter().sum();
            let gap = pair[1].origin.x - pair[0].origin.x;
            assert!(
                (gap - advance).abs() < 0.01,
                "gap {gap} should equal preceding segment advance {advance}"
            );
        }
    }
}
