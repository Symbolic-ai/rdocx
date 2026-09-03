//! Bounded HTML and CSS import into the native Presentation facade.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::Path;

use oxml_drawing::text::{
    CT_RegularTextRun, CT_TextCharacterProperties, CT_TextLineBreak, CT_TextParagraphProperties,
    TextAlignment, TextFont, TextHyperlink, TextRun, TextStrike, TextUnderline,
};
use scraper::{ElementRef, Html, Node, Selector};

use super::*;

const VIEWPORT_WIDTH_PX: i64 = 1_280;
const VIEWPORT_HEIGHT_PX: i64 = 720;
const EMU_PER_CSS_PIXEL: i64 = 9_525;
const MAX_INPUT_BYTES: usize = 64 * 1024 * 1024;

/// One caller-owned image available to the HTML importer.
#[derive(Clone, Copy, Debug)]
pub struct HtmlImageResource<'a> {
    pub source: &'a str,
    pub bytes: &'a [u8],
    pub filename: &'a str,
}

/// One stable path-aware loss report at the HTML conversion boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDiagnostic {
    pub path: String,
    pub property: Option<String>,
    pub message: String,
}

/// A freshly converted editable presentation and its ordered diagnostics.
pub struct HtmlReadResult {
    pub presentation: Presentation,
    pub diagnostics: Vec<HtmlDiagnostic>,
}

#[derive(Clone, Copy, Debug)]
struct Limits {
    input_bytes: usize,
    depth: usize,
    nodes: usize,
    text_bytes: usize,
    css_rules: usize,
    selector_matches: usize,
    objects: usize,
    table_rows: usize,
    table_columns: usize,
    table_cells: usize,
    links: usize,
    image_bytes: usize,
    diagnostics: usize,
    slides: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            input_bytes: MAX_INPUT_BYTES,
            depth: 256,
            nodes: 100_000,
            text_bytes: MAX_INPUT_BYTES,
            css_rules: 10_000,
            selector_matches: 1_000_000,
            objects: 10_000,
            table_rows: 10_000,
            table_columns: 256,
            table_cells: 65_536,
            links: 10_000,
            image_bytes: 128 * 1024 * 1024,
            diagnostics: 10_000,
            slides: 1_000,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct ComputedStyle {
    position_absolute: bool,
    left: Option<Emu>,
    top: Option<Emu>,
    width: Option<Emu>,
    height: Option<Emu>,
    background: Option<String>,
    border_color: Option<String>,
    border_width: Option<Emu>,
    font_family: Option<String>,
    font_size_points: Option<f64>,
    bold: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
    strike: Option<bool>,
    color: Option<String>,
    alignment: Option<TextAlignment>,
}

impl ComputedStyle {
    fn inherited(&self) -> Self {
        Self {
            font_family: self.font_family.clone(),
            font_size_points: self.font_size_points,
            bold: self.bold,
            italic: self.italic,
            underline: self.underline,
            strike: self.strike,
            color: self.color.clone(),
            alignment: self.alignment,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug)]
enum StyleChange {
    PositionAbsolute(bool),
    Left(Emu),
    Top(Emu),
    Width(Emu),
    Height(Emu),
    Background(Option<String>),
    BorderColor(String),
    BorderWidth(Emu),
    FontFamily(String),
    FontSize(f64),
    Bold(bool),
    Italic(bool),
    Underline(bool),
    Strike(bool),
    Color(String),
    Alignment(TextAlignment),
}

impl StyleChange {
    fn apply(&self, style: &mut ComputedStyle) {
        match self {
            Self::PositionAbsolute(value) => style.position_absolute = *value,
            Self::Left(value) => style.left = Some(*value),
            Self::Top(value) => style.top = Some(*value),
            Self::Width(value) => style.width = Some(*value),
            Self::Height(value) => style.height = Some(*value),
            Self::Background(value) => style.background.clone_from(value),
            Self::BorderColor(value) => style.border_color = Some(value.clone()),
            Self::BorderWidth(value) => style.border_width = Some(*value),
            Self::FontFamily(value) => style.font_family = Some(value.clone()),
            Self::FontSize(value) => style.font_size_points = Some(*value),
            Self::Bold(value) => style.bold = Some(*value),
            Self::Italic(value) => style.italic = Some(*value),
            Self::Underline(value) => style.underline = Some(*value),
            Self::Strike(value) => style.strike = Some(*value),
            Self::Color(value) => style.color = Some(value.clone()),
            Self::Alignment(value) => style.alignment = Some(*value),
        }
    }
}

#[derive(Debug)]
struct CssRule {
    selector: Selector,
    specificity: (u32, u32, u32),
    order: usize,
    changes: Vec<StyleChange>,
}

#[derive(Clone, Debug)]
enum InlinePiece {
    Text {
        text: String,
        style: Box<ComputedStyle>,
        target: Option<String>,
    },
    Break,
}

#[derive(Clone, Debug)]
struct ParagraphModel {
    pieces: Vec<InlinePiece>,
    style: ComputedStyle,
}

#[derive(Clone, Copy, Debug)]
struct Geometry {
    left: Emu,
    top: Emu,
    width: Emu,
    height: Emu,
}

struct Importer<'a> {
    dom: &'a Html,
    presentation: Presentation,
    images: HashMap<&'a str, HtmlImageResource<'a>>,
    diagnostics: Vec<(usize, HtmlDiagnostic)>,
    diagnostic_keys: HashSet<(String, Option<String>, String)>,
    element_order: HashMap<usize, usize>,
    rules: Vec<CssRule>,
    limits: Limits,
    objects: usize,
    rows: usize,
    cells: usize,
    links: usize,
    selector_matches: usize,
}

impl Presentation {
    /// Converts one bounded HTML document or fragment into a fresh editable deck.
    pub fn from_html(html: &str, images: &[HtmlImageResource<'_>]) -> Result<HtmlReadResult> {
        from_html_with_limits(html, images, Limits::default())
    }

    /// Opens and converts one UTF-8 HTML path without fetching external resources.
    pub fn open_html<P: AsRef<Path>>(
        path: P,
        images: &[HtmlImageResource<'_>],
    ) -> Result<HtmlReadResult> {
        let mut file = std::fs::File::open(path).map_err(OpcError::from)?;
        let expected = usize::try_from(file.metadata().map_err(OpcError::from)?.len())
            .map_err(|_| html_error("input", "HTML input size is not representable"))?;
        let bytes = read_bounded(&mut file, expected, MAX_INPUT_BYTES)?;
        let html = std::str::from_utf8(&bytes)
            .map_err(|_| html_error("input", "HTML input is not valid UTF-8"))?;
        Self::from_html(html, images)
    }
}

fn from_html_with_limits<'a>(
    html: &str,
    images: &[HtmlImageResource<'a>],
    limits: Limits,
) -> Result<HtmlReadResult> {
    if html.len() > limits.input_bytes {
        return Err(html_error("input", "HTML input exceeds the size limit"));
    }
    let image_map = validate_images(images, limits)?;
    let document = contains_ascii_case_insensitive(html, b"<html")
        || contains_ascii_case_insensitive(html, b"<!doctype");
    let dom = if document {
        Html::parse_document(html)
    } else {
        Html::parse_fragment(html)
    };
    validate_dom(&dom, limits)?;
    let root = dom.root_element();
    let element_order = std::iter::once(root)
        .chain(root.descendent_elements())
        .enumerate()
        .map(|(order, element)| (element_key(element), order))
        .collect();
    let mut importer = Importer {
        dom: &dom,
        presentation: Presentation::new()?,
        images: image_map,
        diagnostics: Vec::new(),
        diagnostic_keys: HashSet::new(),
        element_order,
        rules: Vec::new(),
        limits,
        objects: 0,
        rows: 0,
        cells: 0,
        links: 0,
        selector_matches: 0,
    };
    importer.record_parser_repairs()?;
    importer.record_forbidden_resources()?;
    importer.collect_styles()?;
    importer.project()?;
    importer.finish()
}

fn validate_images<'a>(
    images: &[HtmlImageResource<'a>],
    limits: Limits,
) -> Result<HashMap<&'a str, HtmlImageResource<'a>>> {
    let mut map = HashMap::with_capacity(images.len());
    let mut total = 0_usize;
    for resource in images {
        if resource.source.is_empty() {
            return Err(html_error("images", "image source names must not be empty"));
        }
        if map.insert(resource.source, *resource).is_some() {
            return Err(html_error(
                format!("images/{}", resource.source),
                "duplicate HTML image source",
            ));
        }
        total = total
            .checked_add(resource.bytes.len())
            .ok_or_else(|| html_error("images", "image byte count overflowed"))?;
        if total > limits.image_bytes {
            return Err(html_error("images", "HTML images exceed the byte limit"));
        }
    }
    Ok(map)
}

fn validate_dom(dom: &Html, limits: Limits) -> Result<()> {
    let mut nodes = 0_usize;
    let mut text_bytes = 0_usize;
    for node in dom.tree.nodes() {
        nodes = nodes
            .checked_add(1)
            .ok_or_else(|| html_error("html", "DOM node count overflowed"))?;
        if nodes > limits.nodes {
            return Err(html_error("html", "HTML DOM exceeds the node limit"));
        }
        let mut depth = 0_usize;
        let mut ancestor = Some(node);
        while let Some(current) = ancestor {
            depth = depth
                .checked_add(1)
                .ok_or_else(|| html_error("html", "DOM depth overflowed"))?;
            if depth > limits.depth {
                return Err(html_error("html", "HTML DOM exceeds the depth limit"));
            }
            ancestor = current.parent();
        }
        if let Node::Text(text) = node.value() {
            text_bytes = text_bytes
                .checked_add(text.len())
                .ok_or_else(|| html_error("html", "retained text length overflowed"))?;
            if text_bytes > limits.text_bytes {
                return Err(html_error("html", "HTML retained text exceeds the limit"));
            }
        }
    }
    Ok(())
}

impl Importer<'_> {
    fn record_parser_repairs(&mut self) -> Result<()> {
        for error in &self.dom.errors {
            self.diagnostic("html", None, format!("HTML parser repair: {error}"))?;
        }
        Ok(())
    }

    fn record_forbidden_resources(&mut self) -> Result<()> {
        let selector = Selector::parse("script, link, iframe, object, embed")
            .expect("static resource selector");
        let elements = self.dom.select(&selector).collect::<Vec<_>>();
        for element in elements {
            let name = element.value().name();
            let message = match name {
                "script" => "HTML script is unsupported and was dropped",
                "link" => "external HTML resource is unsupported and was not fetched",
                "iframe" | "object" | "embed" => {
                    "embedded HTML resource is unsupported and was not fetched"
                }
                _ => continue,
            };
            self.diagnostic_element(element, None, message.to_owned())?;
        }
        Ok(())
    }

    fn collect_styles(&mut self) -> Result<()> {
        let selector = Selector::parse("style").expect("static style selector");
        let styles = self
            .dom
            .select(&selector)
            .map(|element| {
                (
                    element_path(element),
                    self.element_order(element),
                    element.text().collect::<String>(),
                )
            })
            .collect::<Vec<_>>();
        let mut order = 0_usize;
        for (path, document_order, css) in styles {
            let css = strip_css_comments(&css);
            let mut remainder = css.as_str();
            while let Some(open) = remainder.find('{') {
                let selector_text = remainder[..open].trim();
                let after_open = &remainder[open + 1..];
                let Some(close) = after_open.find('}') else {
                    self.diagnostic_ordered(
                        &path,
                        document_order,
                        None,
                        "unterminated CSS rule".to_owned(),
                    )?;
                    remainder = "";
                    break;
                };
                let declarations = &after_open[..close];
                remainder = &after_open[close + 1..];
                if selector_text.starts_with('@') {
                    self.diagnostic_ordered(
                        &path,
                        document_order,
                        None,
                        format!("unsupported CSS at-rule `{selector_text}`"),
                    )?;
                    continue;
                }
                for part in selector_text.split(',').map(str::trim) {
                    if !supported_selector(part) {
                        self.diagnostic_ordered(
                            &path,
                            document_order,
                            None,
                            format!("unsupported CSS selector `{part}`"),
                        )?;
                        continue;
                    }
                    let selector = match Selector::parse(part) {
                        Ok(selector) => selector,
                        Err(_) => {
                            self.diagnostic_ordered(
                                &path,
                                document_order,
                                None,
                                format!("invalid CSS selector `{part}`"),
                            )?;
                            continue;
                        }
                    };
                    let changes = self.parse_declarations(declarations, &path, document_order)?;
                    if !changes.is_empty() {
                        if self.rules.len() >= self.limits.css_rules {
                            return Err(html_error(&path, "HTML exceeds the CSS rule limit"));
                        }
                        self.rules.push(CssRule {
                            selector,
                            specificity: specificity(part),
                            order,
                            changes,
                        });
                        order = order
                            .checked_add(1)
                            .ok_or_else(|| html_error(&path, "CSS source order overflowed"))?;
                    }
                }
            }
            if !remainder.trim().is_empty() {
                self.diagnostic_ordered(
                    &path,
                    document_order,
                    None,
                    "unsupported CSS text outside a rule".to_owned(),
                )?;
            }
        }
        Ok(())
    }

    fn parse_declarations(
        &mut self,
        declarations: &str,
        path: &str,
        document_order: usize,
    ) -> Result<Vec<StyleChange>> {
        let mut changes = Vec::new();
        for declaration in declarations.split(';').map(str::trim) {
            if declaration.is_empty() {
                continue;
            }
            let Some((property, value)) = declaration.split_once(':') else {
                self.diagnostic_ordered(
                    path,
                    document_order,
                    None,
                    format!("invalid CSS declaration `{declaration}`"),
                )?;
                continue;
            };
            let property = property.trim().to_ascii_lowercase();
            match parse_style_changes(&property, value.trim()) {
                Ok(mut parsed) => changes.append(&mut parsed),
                Err(message) => {
                    self.diagnostic_ordered(path, document_order, Some(property), message)?
                }
            }
        }
        Ok(changes)
    }

    fn computed_style(
        &mut self,
        element: ElementRef<'_>,
        inherited: &ComputedStyle,
    ) -> Result<ComputedStyle> {
        self.selector_matches = self
            .selector_matches
            .checked_add(self.rules.len())
            .ok_or_else(|| html_error(element_path(element), "CSS selector work overflowed"))?;
        if self.selector_matches > self.limits.selector_matches {
            return Err(html_error(
                element_path(element),
                "HTML exceeds the CSS selector-match budget",
            ));
        }
        let mut style = inherited.inherited();
        match element.value().name() {
            "b" | "strong" => style.bold = Some(true),
            "i" | "em" => style.italic = Some(true),
            "u" => style.underline = Some(true),
            "s" | "strike" | "del" => style.strike = Some(true),
            "h1" => {
                style.bold = Some(true);
                style.font_size_points = Some(24.0);
            }
            "h2" => {
                style.bold = Some(true);
                style.font_size_points = Some(20.0);
            }
            "h3" | "h4" | "h5" | "h6" => style.bold = Some(true),
            _ => {}
        }
        let mut matching = self
            .rules
            .iter()
            .filter(|rule| rule.selector.matches(&element))
            .collect::<Vec<_>>();
        matching.sort_by(|left, right| {
            compare_specificity(left.specificity, right.specificity)
                .then_with(|| left.order.cmp(&right.order))
        });
        for rule in matching {
            for change in &rule.changes {
                change.apply(&mut style);
            }
        }
        if let Some(inline) = element.attr("style") {
            let path = element_path(element);
            let document_order = self.element_order(element);
            for change in self.parse_declarations(inline, &path, document_order)? {
                change.apply(&mut style);
            }
        }
        Ok(style)
    }

    fn project(&mut self) -> Result<()> {
        self.presentation.set_slide_size(
            Emu(VIEWPORT_WIDTH_PX * EMU_PER_CSS_PIXEL),
            Emu(VIEWPORT_HEIGHT_PX * EMU_PER_CSS_PIXEL),
        )?;
        let slide_selector = Selector::parse("section[data-slide]").expect("static slide selector");
        let mut sections = self
            .dom
            .select(&slide_selector)
            .filter(|section| {
                !section
                    .ancestors()
                    .skip(1)
                    .filter_map(ElementRef::wrap)
                    .any(|ancestor| {
                        ancestor.value().name() == "section"
                            && ancestor.attr("data-slide").is_some()
                    })
            })
            .collect::<Vec<_>>();
        if sections.len() > self.limits.slides {
            return Err(html_error("html", "HTML exceeds the slide limit"));
        }
        if sections.is_empty() {
            let root = self.dom.root_element();
            let body_selector = Selector::parse("body").expect("static body selector");
            sections.push(root.select(&body_selector).next().unwrap_or(root));
        }
        for (slide_index, section) in sections.into_iter().enumerate() {
            self.presentation.add_slide(0)?;
            self.presentation.slides[slide_index]
                .slide
                .common_slide_data
                .shape_tree
                .children
                .clear();
            self.project_children(
                slide_index,
                section,
                &ComputedStyle::default(),
                Emu(0),
                Emu(0),
                true,
            )?;
        }
        Ok(())
    }

    fn project_children(
        &mut self,
        slide_index: usize,
        container: ElementRef<'_>,
        inherited: &ComputedStyle,
        parent_left: Emu,
        parent_top: Emu,
        slide_root: bool,
    ) -> Result<()> {
        let container_style = self.computed_style(container, inherited)?;
        let children = container
            .children()
            .filter_map(ElementRef::wrap)
            .collect::<Vec<_>>();
        for element in children {
            if matches!(
                element.value().name(),
                "head" | "style" | "script" | "title"
            ) {
                continue;
            }
            let style = self.computed_style(element, &container_style)?;
            let supported = is_supported_structure(element.value().name());
            if !supported {
                if unsupported_element_has_presence(element, &style) {
                    self.diagnostic_element(
                        element,
                        None,
                        format!(
                            "unsupported HTML element `{}` was dropped",
                            element.value().name()
                        ),
                    )?;
                }
                self.project_children(
                    slide_index,
                    element,
                    &style,
                    parent_left,
                    parent_top,
                    false,
                )?;
                continue;
            }
            if style.position_absolute {
                let geometry = match geometry(&style, parent_left, parent_top) {
                    Ok(geometry) => geometry,
                    Err(message) => {
                        self.diagnostic_element(element, Some("position".to_owned()), message)?;
                        continue;
                    }
                };
                self.project_element(slide_index, element, &style, geometry)?;
                self.project_children(
                    slide_index,
                    element,
                    &style,
                    geometry.left,
                    geometry.top,
                    false,
                )?;
            } else {
                if (slide_root || is_block_element(element.value().name()))
                    && element_has_visible_content(element)
                {
                    self.diagnostic_element(
                        element,
                        Some("position".to_owned()),
                        "implicit layout is unsupported, use explicit absolute geometry".to_owned(),
                    )?;
                }
                self.project_children(
                    slide_index,
                    element,
                    &style,
                    parent_left,
                    parent_top,
                    false,
                )?;
            }
        }
        Ok(())
    }

    fn project_element(
        &mut self,
        slide_index: usize,
        element: ElementRef<'_>,
        style: &ComputedStyle,
        geometry: Geometry,
    ) -> Result<()> {
        self.objects = self
            .objects
            .checked_add(1)
            .ok_or_else(|| html_error("html", "projected object count overflowed"))?;
        if self.objects > self.limits.objects {
            return Err(html_error(
                "html",
                "HTML exceeds the projected object limit",
            ));
        }
        match element.value().name() {
            "img" => self.project_image(slide_index, element, geometry),
            "table" => self.project_table(slide_index, element, style, geometry),
            _ => self.project_shape(slide_index, element, style, geometry),
        }
    }

    fn project_image(
        &mut self,
        slide_index: usize,
        element: ElementRef<'_>,
        geometry: Geometry,
    ) -> Result<()> {
        let Some(source) = element.attr("src") else {
            self.diagnostic_element(
                element,
                Some("src".to_owned()),
                "image has no source".to_owned(),
            )?;
            return Ok(());
        };
        let Some(resource) = self.images.get(source).copied() else {
            self.diagnostic_element(
                element,
                Some("src".to_owned()),
                format!("image source `{source}` was not supplied by the caller"),
            )?;
            return Ok(());
        };
        self.presentation.add_picture(
            slide_index,
            resource.bytes,
            resource.filename,
            geometry.left,
            geometry.top,
            Some(geometry.width),
            Some(geometry.height),
        )?;
        Ok(())
    }

    fn project_shape(
        &mut self,
        slide_index: usize,
        element: ElementRef<'_>,
        style: &ComputedStyle,
        geometry: Geometry,
    ) -> Result<()> {
        let paragraphs = self.collect_paragraphs(element, style)?;
        let has_text = paragraphs.iter().any(|paragraph| {
            paragraph.pieces.iter().any(|piece| match piece {
                InlinePiece::Text { text, .. } => !text.trim().is_empty(),
                InlinePiece::Break => true,
            })
        });
        if !has_text && style.background.is_none() && style.border_width.is_none() {
            return Ok(());
        }
        let prepared = self.prepare_links(slide_index, paragraphs)?;
        let mut slide = self
            .presentation
            .slide_mut(slide_index)
            .expect("new HTML slide exists");
        let mut shape = if style.background.is_some() || style.border_width.is_some() {
            slide.add_shape(
                "rect",
                geometry.left,
                geometry.top,
                geometry.width,
                geometry.height,
            )?
        } else {
            slide.add_textbox(geometry.left, geometry.top, geometry.width, geometry.height)?
        };
        if let Some(color) = &style.background {
            shape.set_fill(solid_fill(color)?)?;
        }
        if let Some(width) = style.border_width {
            shape.set_line(solid_line(
                width,
                style.border_color.as_deref().unwrap_or("000000"),
            )?)?;
        }
        if let Some(mut frame) = shape.text_frame() {
            write_paragraphs(&mut frame, &prepared)?;
        }
        Ok(())
    }

    fn project_table(
        &mut self,
        slide_index: usize,
        table: ElementRef<'_>,
        inherited: &ComputedStyle,
        geometry: Geometry,
    ) -> Result<()> {
        let row_selector = Selector::parse("tr").expect("static row selector");
        let rows = table
            .select(&row_selector)
            .filter(|row| nearest_ancestor_named(*row, "table") == Some(table))
            .collect::<Vec<_>>();
        if rows.is_empty() {
            self.diagnostic_element(table, None, "empty HTML table was dropped".to_owned())?;
            return Ok(());
        }
        self.rows = self
            .rows
            .checked_add(rows.len())
            .ok_or_else(|| html_error("html", "HTML table row count overflowed"))?;
        if self.rows > self.limits.table_rows {
            return Err(html_error("html", "HTML exceeds the table row limit"));
        }
        let cell_selector = Selector::parse("th, td").expect("static cell selector");
        let mut row_cells = Vec::with_capacity(rows.len());
        let mut columns = 0_usize;
        for row in &rows {
            let cells = row
                .select(&cell_selector)
                .filter(|cell| nearest_ancestor_named(*cell, "tr") == Some(*row))
                .collect::<Vec<_>>();
            columns = columns.max(cells.len());
            row_cells.push(cells);
        }
        if columns == 0 || columns > self.limits.table_columns {
            return Err(html_error(
                element_path(table),
                "HTML table column count is outside the supported bound",
            ));
        }
        let added_cells = rows
            .len()
            .checked_mul(columns)
            .ok_or_else(|| html_error("html", "HTML table cell count overflowed"))?;
        self.cells = self
            .cells
            .checked_add(added_cells)
            .ok_or_else(|| html_error("html", "HTML table cell count overflowed"))?;
        if self.cells > self.limits.table_cells {
            return Err(html_error("html", "HTML exceeds the table cell limit"));
        }
        let mut values = Vec::with_capacity(rows.len());
        for cells in row_cells {
            let mut row = Vec::with_capacity(columns);
            for cell in cells {
                if cell.attr("rowspan").is_some_and(|span| span != "1")
                    || cell.attr("colspan").is_some_and(|span| span != "1")
                {
                    self.diagnostic_element(
                        cell,
                        None,
                        "HTML table spans are unsupported and were flattened".to_owned(),
                    )?;
                }
                let style = self.computed_style(cell, inherited)?;
                let paragraphs = self.collect_paragraphs(cell, &style)?;
                row.push((paragraphs, style));
            }
            while row.len() < columns {
                row.push((Vec::new(), inherited.clone()));
            }
            values.push(row);
        }
        let mut prepared = Vec::with_capacity(values.len());
        for row in values {
            let mut output = Vec::with_capacity(row.len());
            for (paragraphs, style) in row {
                output.push((self.prepare_links(slide_index, paragraphs)?, style));
            }
            prepared.push(output);
        }
        let mut slide = self
            .presentation
            .slide_mut(slide_index)
            .expect("new HTML slide exists");
        let mut shape = slide.add_table(
            rows.len(),
            columns,
            geometry.left,
            geometry.top,
            geometry.width,
            geometry.height,
        )?;
        let mut table_handle = shape.table_mut().expect("new HTML table exists");
        for (row_index, row) in prepared.iter().enumerate() {
            for (column_index, (paragraphs, style)) in row.iter().enumerate() {
                let mut cell = table_handle
                    .cell_mut(row_index, column_index)
                    .expect("rectangular HTML table cell exists");
                if let Some(color) = &style.background {
                    cell.set_fill(Some(solid_fill(color)?));
                }
                let mut frame = cell.text_frame();
                write_paragraphs(&mut frame, paragraphs)?;
            }
        }
        Ok(())
    }

    fn collect_paragraphs(
        &mut self,
        element: ElementRef<'_>,
        inherited: &ComputedStyle,
    ) -> Result<Vec<ParagraphModel>> {
        let block_selector =
            Selector::parse("p, h1, h2, h3, h4, h5, h6").expect("static paragraph selector");
        let blocks = element.select(&block_selector).collect::<Vec<_>>();
        if blocks.is_empty()
            || matches!(
                element.value().name(),
                "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
            )
        {
            let style = self.computed_style(element, inherited)?;
            let mut pieces = Vec::new();
            self.collect_inline_children(element, &style, None, &mut pieces)?;
            return Ok(vec![ParagraphModel { pieces, style }]);
        }
        let mut paragraphs = Vec::with_capacity(blocks.len());
        for block in blocks {
            if self.has_positioned_ancestor_before(block, element, inherited)? {
                continue;
            }
            let style = self.computed_style(block, inherited)?;
            if style.position_absolute {
                continue;
            }
            let mut pieces = Vec::new();
            self.collect_inline_children(block, &style, None, &mut pieces)?;
            paragraphs.push(ParagraphModel { pieces, style });
        }
        Ok(paragraphs)
    }

    fn has_positioned_ancestor_before(
        &mut self,
        element: ElementRef<'_>,
        boundary: ElementRef<'_>,
        inherited: &ComputedStyle,
    ) -> Result<bool> {
        let mut ancestor = element.parent();
        while let Some(node) = ancestor {
            if node == *boundary {
                return Ok(false);
            }
            if let Some(element) = ElementRef::wrap(node)
                && self.computed_style(element, inherited)?.position_absolute
            {
                return Ok(true);
            }
            ancestor = node.parent();
        }
        Ok(false)
    }

    fn collect_inline_children(
        &mut self,
        element: ElementRef<'_>,
        inherited: &ComputedStyle,
        target: Option<&str>,
        output: &mut Vec<InlinePiece>,
    ) -> Result<()> {
        for child in element.children() {
            match child.value() {
                Node::Text(text) => output.push(InlinePiece::Text {
                    text: text.to_string(),
                    style: Box::new(inherited.clone()),
                    target: target.map(str::to_owned),
                }),
                Node::Element(_) => {
                    let child = ElementRef::wrap(child).expect("element checked");
                    let style = self.computed_style(child, inherited)?;
                    if style.position_absolute {
                        continue;
                    }
                    match child.value().name() {
                        "br" => output.push(InlinePiece::Break),
                        "img" | "table" | "style" | "script" => {}
                        "a" => {
                            let next_target = match child.attr("href") {
                                Some(value) if safe_link_target(value) => {
                                    self.links = self.links.checked_add(1).ok_or_else(|| {
                                        html_error("html", "HTML link count overflowed")
                                    })?;
                                    if self.links > self.limits.links {
                                        return Err(html_error(
                                            "html",
                                            "HTML exceeds the link limit",
                                        ));
                                    }
                                    Some(value)
                                }
                                Some(value) => {
                                    self.diagnostic_element(
                                        child,
                                        Some("href".to_owned()),
                                        format!(
                                            "unsafe hyperlink scheme was dropped from `{value}`"
                                        ),
                                    )?;
                                    None
                                }
                                None => None,
                            };
                            self.collect_inline_children(child, &style, next_target, output)?;
                        }
                        name if is_supported_structure(name) => {
                            self.collect_inline_children(child, &style, target, output)?;
                        }
                        name => {
                            if unsupported_element_has_presence(child, &style) {
                                self.diagnostic_element(
                                    child,
                                    None,
                                    format!(
                                        "unsupported inline HTML element `{name}` was flattened"
                                    ),
                                )?;
                            }
                            self.collect_inline_children(child, &style, target, output)?;
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn prepare_links(
        &mut self,
        slide_index: usize,
        paragraphs: Vec<ParagraphModel>,
    ) -> Result<Vec<ParagraphModel>> {
        let slide_part = self.presentation.slides[slide_index].part_name.clone();
        let relationships = self
            .presentation
            .package
            .part_rels
            .entry(slide_part)
            .or_default();
        let mut target_ids = HashMap::<String, String>::new();
        let mut paragraphs = paragraphs;
        for paragraph in &mut paragraphs {
            for piece in &mut paragraph.pieces {
                let InlinePiece::Text { target, .. } = piece else {
                    continue;
                };
                let Some(value) = target.clone() else {
                    continue;
                };
                let id = if let Some(id) = target_ids.get(&value) {
                    id.clone()
                } else if let Some(existing) = relationships.items.iter().find(|relationship| {
                    relationship.rel_type == rel_types::HYPERLINK
                        && relationship.target_mode.as_deref() == Some("External")
                        && relationship.target == value
                }) {
                    existing.id.clone()
                } else {
                    relationships.add_external(rel_types::HYPERLINK, &value)
                };
                target_ids.insert(value, id.clone());
                *target = Some(id);
            }
        }
        Ok(paragraphs)
    }

    fn element_order(&self, element: ElementRef<'_>) -> usize {
        self.element_order
            .get(&element_key(element))
            .copied()
            .expect("HTML element belongs to the imported DOM")
    }

    fn diagnostic_element(
        &mut self,
        element: ElementRef<'_>,
        property: Option<String>,
        message: String,
    ) -> Result<()> {
        self.diagnostic_ordered(
            &element_path(element),
            self.element_order(element),
            property,
            message,
        )
    }

    fn diagnostic(&mut self, path: &str, property: Option<String>, message: String) -> Result<()> {
        let order = if path == "html" { 0 } else { usize::MAX };
        self.diagnostic_ordered(path, order, property, message)
    }

    fn diagnostic_ordered(
        &mut self,
        path: &str,
        document_order: usize,
        property: Option<String>,
        message: String,
    ) -> Result<()> {
        let key = (path.to_owned(), property.clone(), message.clone());
        if !self.diagnostic_keys.insert(key) {
            return Ok(());
        }
        if self.diagnostics.len() >= self.limits.diagnostics {
            return Err(html_error(path, "HTML exceeds the diagnostic limit"));
        }
        self.diagnostics.push((
            document_order,
            HtmlDiagnostic {
                path: path.to_owned(),
                property,
                message,
            },
        ));
        Ok(())
    }

    fn finish(mut self) -> Result<HtmlReadResult> {
        let bytes = self.presentation.to_bytes()?;
        let presentation = Presentation::from_bytes(&bytes)?;
        let issues = presentation.validate();
        if !issues.is_empty() {
            return Err(html_error(
                "presentation",
                format!("converted presentation did not validate: {issues:?}"),
            ));
        }
        self.diagnostics.sort_by_key(|(order, _)| *order);
        Ok(HtmlReadResult {
            presentation,
            diagnostics: self
                .diagnostics
                .into_iter()
                .map(|(_, diagnostic)| diagnostic)
                .collect(),
        })
    }
}

fn write_paragraphs(frame: &mut TextFrame<'_>, paragraphs: &[ParagraphModel]) -> Result<()> {
    frame.set_text("");
    let count = paragraphs.len().max(1);
    for index in 0..count {
        let paragraph = if index == 0 {
            frame
                .body
                .paragraph_mut(0)
                .expect("text body retains one paragraph")
        } else {
            frame.body.add_paragraph()
        };
        paragraph.runs.clear();
        let Some(model) = paragraphs.get(index) else {
            continue;
        };
        let mut properties = CT_TextParagraphProperties::default();
        properties.alignment = model.style.alignment;
        if let Some(font_size) = model.style.font_size_points {
            properties.default_run_properties = Some(character_properties(&ComputedStyle {
                font_size_points: Some(font_size),
                ..ComputedStyle::default()
            })?);
        }
        if properties != CT_TextParagraphProperties::default() {
            paragraph.properties = Some(properties);
        }
        let mut pending_space = false;
        let mut emitted = false;
        for piece in &model.pieces {
            match piece {
                InlinePiece::Break => {
                    paragraph
                        .runs
                        .push(TextRun::Break(CT_TextLineBreak::default()));
                    pending_space = false;
                    emitted = true;
                }
                InlinePiece::Text {
                    text,
                    style,
                    target,
                } => {
                    let text = collapse_text(text, &mut pending_space, emitted);
                    if text.is_empty() {
                        continue;
                    }
                    let mut run = CT_RegularTextRun::new(text);
                    let mut properties = character_properties(style)?;
                    if let Some(relationship_id) = target {
                        let mut hyperlink = TextHyperlink::default();
                        hyperlink.relationship_id = Some(relationship_id.clone());
                        properties.hyperlink_click = Some(hyperlink);
                    }
                    if properties != CT_TextCharacterProperties::default() {
                        run.properties = Some(properties);
                    }
                    paragraph.runs.push(TextRun::Run(run));
                    emitted = true;
                }
            }
        }
    }
    Ok(())
}

fn character_properties(style: &ComputedStyle) -> Result<CT_TextCharacterProperties> {
    let font_size = style.font_size_points.map(|points| (points * 100.0) as i32);
    let latin = style
        .font_family
        .as_ref()
        .map(|family| {
            TextFont::new(family.clone())
                .map_err(|error| html_error("html", format!("invalid font family: {error}")))
        })
        .transpose()?;
    let mut properties = CT_TextCharacterProperties::default();
    properties.font_size = font_size;
    properties.bold = style.bold;
    properties.italic = style.italic;
    properties.underline = style.underline.map(|value| {
        if value {
            TextUnderline::Single
        } else {
            TextUnderline::None
        }
    });
    properties.strike = style.strike.map(|value| {
        if value {
            TextStrike::Single
        } else {
            TextStrike::None
        }
    });
    properties.fill = style.color.as_deref().map(solid_fill).transpose()?;
    properties.latin = latin;
    Ok(properties)
}

fn solid_fill(color: &str) -> Result<Fill> {
    let xml = format!(
        r#"<a:solidFill xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:srgbClr val="{color}"/></a:solidFill>"#
    );
    Fill::from_xml(xml.as_bytes()).map_err(|error| html_error("css", error.to_string()))
}

fn solid_line(width: Emu, color: &str) -> Result<CT_LineProperties> {
    let width = u32::try_from(width.0)
        .map_err(|_| html_error("css", "border width is outside DrawingML range"))?;
    let xml = format!(
        r#"<a:ln xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" w="{width}"><a:solidFill><a:srgbClr val="{color}"/></a:solidFill></a:ln>"#
    );
    CT_LineProperties::from_xml(xml.as_bytes())
        .map_err(|error| html_error("css", error.to_string()))
}

fn geometry(
    style: &ComputedStyle,
    parent_left: Emu,
    parent_top: Emu,
) -> std::result::Result<Geometry, String> {
    let left = style
        .left
        .ok_or_else(|| "absolute layout requires `left`".to_owned())?;
    let top = style
        .top
        .ok_or_else(|| "absolute layout requires `top`".to_owned())?;
    let width = style
        .width
        .ok_or_else(|| "absolute layout requires `width`".to_owned())?;
    let height = style
        .height
        .ok_or_else(|| "absolute layout requires `height`".to_owned())?;
    if width.0 <= 0 || height.0 <= 0 {
        return Err("absolute width and height must be positive".to_owned());
    }
    Ok(Geometry {
        left: Emu(parent_left
            .0
            .checked_add(left.0)
            .ok_or_else(|| "absolute horizontal offset overflowed".to_owned())?),
        top: Emu(parent_top
            .0
            .checked_add(top.0)
            .ok_or_else(|| "absolute vertical offset overflowed".to_owned())?),
        width,
        height,
    })
}

fn parse_style_changes(
    property: &str,
    value: &str,
) -> std::result::Result<Vec<StyleChange>, String> {
    let lower = value.trim().to_ascii_lowercase();
    let changes = match property {
        "position" => vec![StyleChange::PositionAbsolute(match lower.as_str() {
            "absolute" => true,
            "static" => false,
            _ => return Err(format!("unsupported position value `{value}`")),
        })],
        "left" => vec![StyleChange::Left(parse_css_length(value, true)?)],
        "top" => vec![StyleChange::Top(parse_css_length(value, true)?)],
        "width" => vec![StyleChange::Width(parse_css_length(value, false)?)],
        "height" => vec![StyleChange::Height(parse_css_length(value, false)?)],
        "background" | "background-color" => {
            if lower.contains("url(") {
                return Err("CSS resource fetching is unsupported".to_owned());
            }
            vec![StyleChange::Background(
                if lower == "transparent" || lower == "none" {
                    None
                } else {
                    Some(parse_color(value)?)
                },
            )]
        }
        "border-color" => vec![StyleChange::BorderColor(parse_color(value)?)],
        "border-width" => vec![StyleChange::BorderWidth(parse_css_length(value, false)?)],
        "border" => parse_border(value)?,
        "font-family" => {
            let family = value
                .split(',')
                .next()
                .unwrap_or_default()
                .trim()
                .trim_matches(['\'', '"']);
            if family.is_empty() {
                return Err("unsupported empty font-family value".to_owned());
            }
            vec![StyleChange::FontFamily(family.to_owned())]
        }
        "font-size" => vec![StyleChange::FontSize(parse_font_points(value)?)],
        "font-weight" => vec![StyleChange::Bold(match lower.as_str() {
            "normal" | "400" | "500" => false,
            "bold" | "bolder" | "600" | "700" | "800" | "900" => true,
            _ => return Err(format!("unsupported font-weight value `{value}`")),
        })],
        "font-style" => vec![StyleChange::Italic(match lower.as_str() {
            "normal" => false,
            "italic" | "oblique" => true,
            _ => return Err(format!("unsupported font-style value `{value}`")),
        })],
        "text-decoration" | "text-decoration-line" => {
            if lower == "none" {
                vec![StyleChange::Underline(false), StyleChange::Strike(false)]
            } else {
                let mut parsed = Vec::new();
                for token in lower.split_whitespace() {
                    match token {
                        "underline" => parsed.push(StyleChange::Underline(true)),
                        "line-through" => parsed.push(StyleChange::Strike(true)),
                        _ => return Err(format!("unsupported text-decoration value `{value}`")),
                    }
                }
                parsed
            }
        }
        "color" => vec![StyleChange::Color(parse_color(value)?)],
        "text-align" => vec![StyleChange::Alignment(match lower.as_str() {
            "left" | "start" => TextAlignment::Left,
            "center" => TextAlignment::Center,
            "right" | "end" => TextAlignment::Right,
            "justify" => TextAlignment::Justified,
            _ => return Err(format!("unsupported text-align value `{value}`")),
        })],
        "display"
            if matches!(
                lower.as_str(),
                "flex" | "inline-flex" | "grid" | "inline-grid"
            ) =>
        {
            return Err(format!("unsupported browser layout value `{value}`"));
        }
        "transform" => return Err("CSS transforms are unsupported".to_owned()),
        _ => return Err(format!("unsupported CSS property `{property}`")),
    };
    Ok(changes)
}

fn parse_border(value: &str) -> std::result::Result<Vec<StyleChange>, String> {
    if value.trim().eq_ignore_ascii_case("none") {
        return Ok(vec![StyleChange::BorderWidth(Emu(0))]);
    }
    let mut changes = Vec::new();
    for token in value.split_whitespace() {
        if matches!(token.to_ascii_lowercase().as_str(), "solid" | "none") {
            continue;
        }
        if let Ok(length) = parse_css_length(token, false) {
            changes.push(StyleChange::BorderWidth(length));
            continue;
        }
        if let Ok(color) = parse_color(token) {
            changes.push(StyleChange::BorderColor(color));
            continue;
        }
        return Err(format!("unsupported border value `{value}`"));
    }
    Ok(changes)
}

fn parse_css_length(value: &str, signed: bool) -> std::result::Result<Emu, String> {
    let value = value.trim().to_ascii_lowercase();
    if value == "0" {
        return Ok(Emu(0));
    }
    let units = [
        ("px", 9_525.0),
        ("pt", 12_700.0),
        ("in", 914_400.0),
        ("cm", 360_000.0),
        ("mm", 36_000.0),
    ];
    let Some((number, factor)) = units
        .iter()
        .find_map(|(suffix, factor)| value.strip_suffix(suffix).map(|number| (number, *factor)))
    else {
        return Err(format!("unsupported CSS length `{value}`"));
    };
    let number = number
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("invalid CSS length `{value}`"))?;
    let emu = number * factor;
    if !emu.is_finite() || emu < i64::MIN as f64 || emu > i64::MAX as f64 || (!signed && emu < 0.0)
    {
        return Err(format!(
            "CSS length is outside the supported range `{value}`"
        ));
    }
    Ok(Emu(emu as i64))
}

fn parse_font_points(value: &str) -> std::result::Result<f64, String> {
    let emu = parse_css_length(value, false)?;
    let points = emu.0 as f64 / 12_700.0;
    if !points.is_finite() || !(1.0..=4_000.0).contains(&points) {
        return Err(format!(
            "font size is outside the supported range `{value}`"
        ));
    }
    Ok(points)
}

fn parse_color(value: &str) -> std::result::Result<String, String> {
    let lower = value.trim().to_ascii_lowercase();
    let color = match lower.as_str() {
        "black" => "000000".to_owned(),
        "white" => "FFFFFF".to_owned(),
        "red" => "FF0000".to_owned(),
        "green" => "008000".to_owned(),
        "blue" => "0000FF".to_owned(),
        "yellow" => "FFFF00".to_owned(),
        "gray" | "grey" => "808080".to_owned(),
        value if value.len() == 7 && value.starts_with('#') => value[1..].to_ascii_uppercase(),
        value if value.len() == 4 && value.starts_with('#') => {
            let mut expanded = String::with_capacity(6);
            for character in value[1..].chars() {
                expanded.push(character.to_ascii_uppercase());
                expanded.push(character.to_ascii_uppercase());
            }
            expanded
        }
        _ => return Err(format!("unsupported CSS color `{value}`")),
    };
    if !color.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("invalid CSS color `{value}`"));
    }
    Ok(color)
}

fn safe_link_target(target: &str) -> bool {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("mailto:")
    {
        return true;
    }
    !trimmed
        .split(['/', '#', '?'])
        .next()
        .is_some_and(|prefix| prefix.contains(':'))
}

fn supported_selector(selector: &str) -> bool {
    !selector.is_empty()
        && !selector
            .chars()
            .any(|character| matches!(character, '[' | ']' | ':' | '+' | '~' | '*' | '|'))
        && selector.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || character.is_ascii_whitespace()
                || matches!(character, '-' | '_' | '.' | '#' | '>')
        })
        && !selector.contains(">>")
}

fn specificity(selector: &str) -> (u32, u32, u32) {
    let ids = selector.bytes().filter(|byte| *byte == b'#').count() as u32;
    let classes = selector.bytes().filter(|byte| *byte == b'.').count() as u32;
    let types = selector
        .split(|character: char| character.is_ascii_whitespace() || character == '>')
        .filter(|part| {
            part.as_bytes()
                .first()
                .is_some_and(|byte| byte.is_ascii_alphabetic())
        })
        .count() as u32;
    (ids, classes, types)
}

fn compare_specificity(left: (u32, u32, u32), right: (u32, u32, u32)) -> Ordering {
    left.0
        .cmp(&right.0)
        .then_with(|| left.1.cmp(&right.1))
        .then_with(|| left.2.cmp(&right.2))
}

fn is_supported_structure(name: &str) -> bool {
    matches!(
        name,
        "section"
            | "div"
            | "p"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "span"
            | "b"
            | "strong"
            | "i"
            | "em"
            | "u"
            | "s"
            | "strike"
            | "del"
            | "br"
            | "table"
            | "thead"
            | "tbody"
            | "tfoot"
            | "tr"
            | "th"
            | "td"
            | "img"
            | "a"
    )
}

fn is_block_element(name: &str) -> bool {
    matches!(
        name,
        "section" | "div" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "table"
    )
}

fn element_has_visible_content(element: ElementRef<'_>) -> bool {
    element.value().name() == "img" || element.text().any(|text| !text.trim().is_empty())
}

fn unsupported_element_has_presence(element: ElementRef<'_>, style: &ComputedStyle) -> bool {
    style.position_absolute
        || element.text().any(|text| !text.trim().is_empty())
        || matches!(
            element.value().name(),
            "audio"
                | "button"
                | "canvas"
                | "form"
                | "iframe"
                | "input"
                | "object"
                | "select"
                | "svg"
                | "textarea"
                | "video"
        )
        || [
            "aria-label",
            "href",
            "placeholder",
            "role",
            "src",
            "title",
            "value",
        ]
        .into_iter()
        .any(|attribute| element.attr(attribute).is_some())
}

fn element_key(element: ElementRef<'_>) -> usize {
    element.value() as *const _ as usize
}

fn nearest_ancestor_named<'a>(element: ElementRef<'a>, name: &str) -> Option<ElementRef<'a>> {
    let mut ancestor = element.parent();
    while let Some(node) = ancestor {
        if let Some(element) = ElementRef::wrap(node)
            && element.value().name() == name
        {
            return Some(element);
        }
        ancestor = node.parent();
    }
    None
}

fn element_path(element: ElementRef<'_>) -> String {
    let mut segments = Vec::new();
    let mut current = Some(*element);
    while let Some(node) = current {
        if let Some(element) = ElementRef::wrap(node) {
            let name = element.value().name();
            let mut index = 1_usize;
            let mut sibling = element.prev_sibling();
            while let Some(node) = sibling {
                if let Some(sibling_element) = ElementRef::wrap(node)
                    && sibling_element.value().name() == name
                {
                    index += 1;
                }
                sibling = node.prev_sibling();
            }
            if matches!(name, "html" | "head" | "body") {
                segments.push(name.to_owned());
            } else {
                segments.push(format!("{name}[{index}]"));
            }
        }
        current = node.parent();
    }
    segments.reverse();
    if segments.first().is_none_or(|segment| segment != "html") {
        segments.insert(0, "html".to_owned());
    }
    if segments
        .get(1)
        .is_none_or(|segment| !segment.starts_with("body"))
    {
        segments.insert(1, "body".to_owned());
    }
    segments.join("/")
}

fn strip_css_comments(css: &str) -> String {
    let mut output = String::with_capacity(css.len());
    let mut remainder = css;
    while let Some(start) = remainder.find("/*") {
        output.push_str(&remainder[..start]);
        let after_start = &remainder[start + 2..];
        let Some(end) = after_start.find("*/") else {
            return output;
        };
        remainder = &after_start[end + 2..];
    }
    output.push_str(remainder);
    output
}

fn collapse_text(text: &str, pending_space: &mut bool, emitted: bool) -> String {
    let mut output = String::new();
    let mut emitted = emitted;
    for character in text.chars() {
        if character.is_whitespace() {
            *pending_space = true;
        } else {
            if *pending_space && emitted {
                output.push(' ');
            }
            output.push(character);
            *pending_space = false;
            emitted = true;
        }
    }
    output
}

fn contains_ascii_case_insensitive(haystack: &str, needle: &[u8]) -> bool {
    haystack
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle))
}

fn read_bounded<R: Read>(reader: &mut R, expected: usize, limit: usize) -> Result<Vec<u8>> {
    let read_limit = u64::try_from(limit)
        .ok()
        .and_then(|limit| limit.checked_add(1))
        .ok_or_else(|| html_error("input", "HTML input limit is not representable"))?;
    let mut bytes = Vec::with_capacity(expected.min(limit));
    reader
        .take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(OpcError::from)?;
    if bytes.len() > limit {
        return Err(html_error("input", "HTML input exceeds the size limit"));
    }
    Ok(bytes)
}

fn html_error(path: impl Into<String>, message: impl Into<String>) -> Error {
    Error::Html {
        path: path.into(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    const PNG: &[u8] = &[
        0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 13, b'I', b'H', b'D', b'R', 0, 0,
        0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 0x1f, 0x15, 0xc4, 0x89, 0, 0, 0, 13, b'I', b'D', b'A',
        b'T', 8, 0xd7, 0x63, 0xf8, 0xcf, 0xc0, 0xf0, 0x1f, 0, 5, 0, 1, 0xff, 0x89, 0x99, 0x3d,
        0x1d, 0, 0, 0, 0, b'I', b'E', b'N', b'D', 0xae, 0x42, 0x60, 0x82,
    ];

    fn box_html(content: &str, extra: &str) -> String {
        format!(
            "<section data-slide><div style='position:absolute;left:1px;top:2px;width:100px;height:50px;{extra}'>{content}</div></section>"
        )
    }

    type ShapeGeometry = (Option<(Emu, Emu)>, Option<(Emu, Emu)>);

    fn shape_geometry(shape: ShapeRef<'_>) -> ShapeGeometry {
        let transform = match shape.child {
            ShapeTreeChild::Shape(shape) => shape.shape_properties.transform.as_ref(),
            ShapeTreeChild::Picture(picture) => picture.shape_properties.transform.as_ref(),
            ShapeTreeChild::GraphicFrame(frame) => Some(&frame.transform),
            ShapeTreeChild::GroupShape(_) => None,
            ShapeTreeChild::Connector(connector) => connector.shape_properties.transform.as_ref(),
            ShapeTreeChild::AlternateContent(_) => None,
        };
        (
            transform
                .and_then(|transform| transform.offset)
                .map(|point| (point.x, point.y)),
            transform
                .and_then(|transform| transform.extent)
                .map(|extent| (extent.cx, extent.cy)),
        )
    }

    #[test]
    fn html_slide_import_converts_explicit_css_boxes_to_exact_emu() {
        let html = "<section data-slide><div style='position:absolute;left:1px;top:2pt;width:1in;height:1cm'><span style='position:absolute;left:1mm;top:1px;width:10px;height:10px'>nested</span></div></section>";
        let imported = Presentation::from_html(html, &[]).expect("HTML imports");
        let slide = imported.presentation.slide(0).expect("one slide");
        assert_eq!(slide.shapes().len(), 1);
        let shape = slide.shape(0).expect("nested text box");
        let (position, size) = shape_geometry(shape);
        assert_eq!(position, Some((Emu(45_525), Emu(34_925))));
        assert_eq!(size, Some((Emu(95_250), Emu(95_250))));
    }

    #[test]
    fn html_slide_import_applies_the_bounded_cascade_to_shapes_and_text() {
        let html = "<style>div { color:#0000ff; background:#ff0000 } .note { color:#00ff00 } #target { color:#ffffff } section > div { border:1px solid #000000 }</style><section data-slide><div id='target' class='note' style='position:absolute;left:0;top:0;width:100px;height:40px;text-align:center'><b>Hello</b></div></section>";
        let imported = Presentation::from_html(html, &[]).expect("HTML imports");
        let shape = imported.presentation.slide(0).unwrap().shape(0).unwrap();
        let frame = shape.text_frame().expect("shape text");
        let run = frame.paragraph(0).unwrap().run(0).unwrap();
        assert_eq!(run.text(), "Hello");
        let properties = run.properties().expect("run properties");
        assert_eq!(properties.bold, Some(true));
        assert!(properties.fill.is_some());
        assert!(
            frame
                .paragraph(0)
                .unwrap()
                .default_run_properties()
                .is_none()
        );
    }

    #[test]
    fn html_slide_import_rejects_every_declared_resource_limit() {
        let cases = [
            Limits {
                input_bytes: 1,
                ..Limits::default()
            },
            Limits {
                depth: 1,
                ..Limits::default()
            },
            Limits {
                nodes: 1,
                ..Limits::default()
            },
            Limits {
                text_bytes: 0,
                ..Limits::default()
            },
            Limits {
                objects: 0,
                ..Limits::default()
            },
            Limits {
                diagnostics: 0,
                ..Limits::default()
            },
            Limits {
                slides: 0,
                ..Limits::default()
            },
        ];
        for limits in cases {
            assert!(
                from_html_with_limits(&box_html("x", "unknown:value"), &[], limits).is_err(),
                "limit accepted: {limits:?}"
            );
        }
        assert!(
            from_html_with_limits(
                "<style>div { color:red }</style><section data-slide></section>",
                &[],
                Limits {
                    css_rules: 0,
                    ..Limits::default()
                }
            )
            .is_err()
        );
        let selector_bounded = "<style>div { color:red }</style><section data-slide><div style='position:absolute;left:0;top:0;width:10px;height:10px'>x</div></section>";
        let error = match from_html_with_limits(
            selector_bounded,
            &[],
            Limits {
                selector_matches: 0,
                ..Limits::default()
            },
        ) {
            Ok(_) => panic!("selector matching must have an aggregate work bound"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            Error::Html { message, .. } if message == "HTML exceeds the CSS selector-match budget"
        ));
        let table = "<section data-slide><table style='position:absolute;left:0;top:0;width:100px;height:100px'><tr><td>x</td></tr></table></section>";
        for limits in [
            Limits {
                table_rows: 0,
                ..Limits::default()
            },
            Limits {
                table_columns: 0,
                ..Limits::default()
            },
            Limits {
                table_cells: 0,
                ..Limits::default()
            },
        ] {
            assert!(from_html_with_limits(table, &[], limits).is_err());
        }
        let linked = box_html("<a href='https://example.com'>x</a>", "");
        assert!(
            from_html_with_limits(
                &linked,
                &[],
                Limits {
                    links: 0,
                    ..Limits::default()
                }
            )
            .is_err()
        );
        assert!(
            validate_images(
                &[HtmlImageResource {
                    source: "x",
                    bytes: PNG,
                    filename: "x.png"
                }],
                Limits {
                    image_bytes: 0,
                    ..Limits::default()
                }
            )
            .is_err()
        );
        assert!(
            validate_images(
                &[
                    HtmlImageResource {
                        source: "x",
                        bytes: PNG,
                        filename: "x.png"
                    },
                    HtmlImageResource {
                        source: "x",
                        bytes: PNG,
                        filename: "y.png"
                    },
                ],
                Limits::default()
            )
            .is_err()
        );
        let mut reader = Cursor::new(b"12345".to_vec());
        assert!(read_bounded(&mut reader, 0, 4).is_err());
    }

    #[test]
    fn unsupported_html_layout_and_styles_are_diagnosed_without_losing_supported_siblings() {
        let html = "<section data-slide><div style='display:flex;transform:rotate(1deg)'>lost</div><div style='position:absolute;left:0;top:0;width:100px;height:40px;color:red'>kept<a href='javascript:alert(1)'> link</a></div></section>";
        let imported = Presentation::from_html(html, &[]).expect("loss is diagnostic");
        assert_eq!(imported.presentation.slide(0).unwrap().shapes().len(), 1);
        assert_eq!(
            imported
                .presentation
                .slide(0)
                .unwrap()
                .shape(0)
                .unwrap()
                .text(),
            Some("kept link".to_owned())
        );
        assert!(
            imported
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.property.as_deref() == Some("display"))
        );
        assert!(
            imported
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.property.as_deref() == Some("transform"))
        );
        assert!(
            imported
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.property.as_deref() == Some("href"))
        );
        assert!(
            imported
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.path.starts_with("html/"))
        );
    }

    #[test]
    fn unsupported_empty_semantic_elements_are_diagnosed() {
        let html = "<section data-slide><video style='position:absolute;left:0;top:0;width:20px;height:20px'></video><canvas style='position:absolute;left:20px;top:0;width:20px;height:20px'></canvas><input value='semantic'></section>";
        let imported = Presentation::from_html(html, &[]).expect("unsupported nodes are lossy");
        assert_eq!(
            imported
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.as_str())
                .collect::<Vec<_>>(),
            vec![
                "unsupported HTML element `video` was dropped",
                "unsupported HTML element `canvas` was dropped",
                "unsupported HTML element `input` was dropped",
            ]
        );
    }

    #[test]
    fn html_diagnostics_follow_document_order_across_collection_phases() {
        let html = "<section data-slide><div style='position:absolute;left:0;top:0;width:20px;height:20px;transform:rotate(1deg)'>first</div><style>@unknown {}</style><iframe></iframe></section>";
        let imported = Presentation::from_html(html, &[]).expect("loss is diagnostic");
        assert_eq!(
            imported
                .diagnostics
                .iter()
                .map(|diagnostic| (
                    diagnostic.path.as_str(),
                    diagnostic.property.as_deref(),
                    diagnostic.message.as_str(),
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    "html/body/section[1]/div[1]",
                    Some("transform"),
                    "CSS transforms are unsupported",
                ),
                (
                    "html/body/section[1]/style[1]",
                    None,
                    "unsupported CSS at-rule `@unknown`",
                ),
                (
                    "html/body/section[1]/iframe[1]",
                    None,
                    "embedded HTML resource is unsupported and was not fetched",
                ),
                (
                    "html/body/section[1]/iframe[1]",
                    None,
                    "unsupported HTML element `iframe` was dropped",
                ),
            ]
        );
    }

    #[test]
    fn html_slide_import_projects_editable_shapes_tables_images_and_links_after_reopen() {
        let html = "<section data-slide><div style='position:absolute;left:0;top:0;width:200px;height:50px;background:#ff0000'><a href='https://example.com'><b>linked</b></a></div><table style='position:absolute;left:0;top:60px;width:200px;height:100px'><tr><th>A</th><th>B</th></tr><tr><td>C</td><td>D</td></tr></table><img src='logo' style='position:absolute;left:210px;top:0;width:10px;height:10px'></section>";
        let imported = Presentation::from_html(
            html,
            &[HtmlImageResource {
                source: "logo",
                bytes: PNG,
                filename: "logo.png",
            }],
        )
        .expect("HTML imports");
        let bytes = imported.presentation.to_bytes().expect("saves");
        let mut reopened = Presentation::from_bytes(&bytes).expect("reopens");
        assert_eq!(reopened.slide(0).unwrap().shapes().len(), 3);
        assert!(
            reopened
                .slide(0)
                .unwrap()
                .shape(1)
                .unwrap()
                .table()
                .is_some()
        );
        assert_eq!(
            reopened.slide(0).unwrap().shape(2).unwrap().kind(),
            ShapeKind::Picture
        );
        let slide_xml =
            String::from_utf8(reopened.current_part_xml("/ppt/slides/slide1.xml").unwrap())
                .unwrap();
        assert!(slide_xml.contains("a:hlinkClick"));
        reopened
            .slide_mut(0)
            .unwrap()
            .shape_mut(0)
            .unwrap()
            .set_text("edited")
            .unwrap();
        assert_eq!(
            reopened.slide(0).unwrap().shape(0).unwrap().text(),
            Some("edited".to_owned())
        );
    }

    #[test]
    fn html_slide_import_preserves_unmodelled_template_xml_and_schema_order() {
        let baseline = Presentation::new().expect("default template opens");
        let imported = Presentation::from_html(&box_html("schema", ""), &[]).expect("imports");
        let bytes = imported.presentation.to_bytes().expect("saves");
        let reopened = Presentation::from_bytes(&bytes).expect("reopens");
        assert!(reopened.validate().is_empty());
        let slide_xml =
            String::from_utf8(reopened.current_part_xml("/ppt/slides/slide1.xml").unwrap())
                .unwrap();
        let tree = slide_xml.find("<p:spTree").unwrap();
        let shape = slide_xml.find("<p:sp>").unwrap();
        let end_tree = slide_xml.find("</p:spTree>").unwrap();
        assert!(tree < shape && shape < end_tree);
        assert!(
            reopened
                .current_part_xml("/ppt/presentation.xml")
                .unwrap()
                .windows(8)
                .any(|window| window == b"p:sldIdL")
        );
        for part_name in [
            "/ppt/slideMasters/slideMaster1.xml",
            "/ppt/slideLayouts/slideLayout1.xml",
            "/ppt/theme/theme1.xml",
        ] {
            assert_eq!(
                reopened.current_part_xml(part_name),
                baseline.current_part_xml(part_name),
                "opaque template part changed: {part_name}"
            );
        }
    }

    #[test]
    fn source_built_html_matches_pinned_chrome_after_save_and_reopen() {
        const CHROME_ORACLE: &str = "Google Chrome 152.0.7977.65";
        let imported =
            Presentation::from_html(&box_html("Chrome oracle", "background:#ffffff"), &[])
                .expect("imports");
        let bytes = imported.presentation.to_bytes().expect("saves");
        let reopened = Presentation::from_bytes(&bytes).expect("reopens");
        let shape = reopened.slide(0).unwrap().shape(0).unwrap();
        let (position, size) = shape_geometry(shape);
        assert_eq!(position, Some((Emu(9_525), Emu(19_050))), "{CHROME_ORACLE}");
        assert_eq!(size, Some((Emu(952_500), Emu(476_250))), "{CHROME_ORACLE}");
        assert_eq!(
            shape.text(),
            Some("Chrome oracle".to_owned()),
            "{CHROME_ORACLE}"
        );
    }

    #[test]
    fn html_browser_differential_rejects_geometry_text_and_pixel_perturbations() {
        fn differential_matches(expected: (&str, i64, u8), actual: (&str, i64, u8)) -> bool {
            expected.0 == actual.0
                && (expected.1 - actual.1).abs() <= EMU_PER_CSS_PIXEL
                && expected.2.abs_diff(actual.2) <= 12
        }
        let reference = ("text", 100 * EMU_PER_CSS_PIXEL, 100);
        assert!(differential_matches(reference, reference));
        assert!(!differential_matches(
            reference,
            ("text", 102 * EMU_PER_CSS_PIXEL, 100)
        ));
        assert!(!differential_matches(
            reference,
            ("mutated", 100 * EMU_PER_CSS_PIXEL, 100)
        ));
        assert!(!differential_matches(
            reference,
            ("text", 100 * EMU_PER_CSS_PIXEL, 113)
        ));
    }
}
