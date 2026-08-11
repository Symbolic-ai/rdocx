//! Render the pinned PresentationML corpus through deterministic fonts.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use oxml_drawing::color::ColorMap;
use oxml_drawing::table::CT_TableStyleList;
use oxml_drawing::theme::CT_OfficeStyleSheet;
use oxml_drawing::xfrm::CT_Transform2D;
use oxml_layout::{Color, FontManager, LayoutResult, MediaId, PageFrame, PositionedElement, Rect};
use oxml_media::{ImageFormat, probe};
use oxml_opc::OpcPackage;
use oxml_opc::relationship::rel_types;
use rpptx_chart::CT_ChartSpace;
use rpptx_layout::{
    ChartResource, FlattenedItem, ResolveCtx, ResolvedContent, ResolvedTextBody, ResolvedTextRun,
    ScopedChartResources, ScopedHyperlinkTargets, ScopedMediaIds,
};
use rpptx_oxml::presentation::CT_Presentation;
use rpptx_oxml::shape_tree::ShapeTreeChild;
use rpptx_oxml::slide_parts::{CT_Slide, CT_SlideLayout, CT_SlideMaster, ColorMapOverrideKind};
use rpptx_render::{MediaData, RenderInput, layout_presentation_with_font_manager};

const DPI: f64 = 150.0;
const EXPECTED_DECKS: usize = 50;
const CORPUS_MANIFEST: &str = include_str!("../../../scripts/pptx-corpus-manifest.tsv");

fn main() {
    if let Err(error) = run() {
        eprintln!("render_deck: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let arguments = env::args_os().skip(1).collect::<Vec<_>>();
    let deck_filter = match arguments.as_slice() {
        [_, _, _] => None,
        [_, _, _, option, deck] if option == "--deck" => deck.to_str(),
        _ => {
            return Err(
                "usage: render_deck CORPUS_DIR OUTPUT_DIR MANIFEST_TSV [--deck NAME]".to_owned(),
            );
        }
    };
    if arguments.len() == 5 && deck_filter.is_none() {
        return Err("deck filter is not valid UTF-8".to_owned());
    }
    let corpus_dir = PathBuf::from(&arguments[0]);
    let output_dir = PathBuf::from(&arguments[1]);
    let manifest_path = PathBuf::from(&arguments[2]);
    let mut entries = corpus_entries()?;
    if let Some(deck_filter) = deck_filter {
        entries.retain(|deck| *deck == deck_filter);
        if entries.len() != 1 {
            return Err(format!(
                "deck filter is not in the corpus manifest: {deck_filter}"
            ));
        }
    }
    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("create {}: {error}", output_dir.display()))?;

    let mut rows = vec![
        "deck\tslide\tsource_shapes\tresolved_shapes\tdropped_shapes\tdiagnostics\toutput"
            .to_owned(),
    ];
    for deck in entries {
        let path = corpus_dir.join(deck);
        let result = catch_unwind(AssertUnwindSafe(|| render_deck(&path, &output_dir)));
        match result {
            Ok(Ok(deck_rows)) => rows.extend(deck_rows),
            Ok(Err(error)) => return Err(error),
            Err(payload) => {
                let message = payload
                    .downcast_ref::<&str>()
                    .copied()
                    .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
                    .unwrap_or("unknown panic");
                return Err(format!("{} panicked: {message}", path.display()));
            }
        }
    }
    fs::write(&manifest_path, rows.join("\n") + "\n")
        .map_err(|error| format!("write {}: {error}", manifest_path.display()))?;
    Ok(())
}

fn corpus_entries() -> Result<Vec<&'static str>, String> {
    let lines = CORPUS_MANIFEST.lines().collect::<Vec<_>>();
    if lines.first().copied() != Some("path\tproducer\tsha256\turl") {
        return Err("invalid corpus manifest header".to_owned());
    }
    let entries = lines[1..]
        .iter()
        .map(|line| {
            line.split('\t')
                .next()
                .ok_or_else(|| "invalid corpus manifest row".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    if entries.len() != EXPECTED_DECKS {
        return Err(format!(
            "corpus manifest has {} decks, expected {EXPECTED_DECKS}",
            entries.len()
        ));
    }
    Ok(entries)
}

fn render_deck(path: &Path, output_root: &Path) -> Result<Vec<String>, String> {
    if !path.is_file() {
        return Err(format!("missing corpus deck {}", path.display()));
    }
    let package = OpcPackage::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let rendered = render_package(&package, path)?;
    if let Some(audit_path) = env::var_os("RPPTX_TEXT_AUDIT") {
        write_text_audit(path, &rendered.input, Path::new(&audit_path))?;
        return Ok(Vec::new());
    }
    let pngs = oxml_pdf::render_all_pages(&rendered.layout, DPI);
    if pngs.len() != rendered.input.slides.len() {
        return Err(format!(
            "{}: rasterized {} pages for {} slides",
            path.display(),
            pngs.len(),
            rendered.input.slides.len()
        ));
    }

    let deck_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("non-UTF-8 deck path {}", path.display()))?;
    let deck_output = output_root.join(deck_name);
    fs::create_dir_all(&deck_output)
        .map_err(|error| format!("create {}: {error}", deck_output.display()))?;
    let mut rows = Vec::with_capacity(pngs.len());
    for (index, png) in pngs.into_iter().enumerate() {
        let output = deck_output.join(format!("slide-{:04}.png", index + 1));
        fs::write(&output, png).map_err(|error| format!("write {}: {error}", output.display()))?;
        let diagnostics = rendered.input.slides[index]
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        rows.push(format!(
            "{}\t{}\t{}\t{}\t0\t{}\t{}",
            escape_tsv(deck_name),
            index + 1,
            rendered.source_counts[index],
            rendered.input.slides[index].shapes.len(),
            escape_tsv(&diagnostics),
            escape_tsv(&output.to_string_lossy())
        ));
    }
    Ok(rows)
}

pub(crate) struct RenderedDeck {
    pub(crate) input: RenderInput,
    pub(crate) layout: LayoutResult,
    source_counts: Vec<usize>,
}

pub(crate) fn render_package(package: &OpcPackage, path: &Path) -> Result<RenderedDeck, String> {
    let presentation_part = package
        .main_document_part()
        .ok_or_else(|| format!("{}: no presentation part", path.display()))?;
    let presentation = CT_Presentation::from_xml(part(package, &presentation_part)?)
        .map_err(|error| format!("{} {presentation_part}: {error}", path.display()))?;
    let default_text_style = presentation.default_text_style.clone().unwrap_or_default();
    let size = presentation
        .slide_size
        .as_ref()
        .map_or((720.0, 540.0), |size| {
            (size.cx.0 as f64 / 12_700.0, size.cy.0 as f64 / 12_700.0)
        });
    let table_styles = table_styles(package, &presentation_part, path)?;
    let presentation_relationships = package
        .get_part_rels(&presentation_part)
        .ok_or_else(|| format!("{}: no presentation relationships", path.display()))?;

    let mut media = HashMap::new();
    let mut font_manager = FontManager::new_deterministic()
        .map_err(|error| format!("deterministic fonts: {error}"))?;
    let mut resolved_slides = Vec::with_capacity(presentation.slide_ids.len());
    let mut source_counts = Vec::with_capacity(presentation.slide_ids.len());
    for slide_id in &presentation.slide_ids {
        let slide_relationship = presentation_relationships
            .get_by_id(&slide_id.relationship_id)
            .ok_or_else(|| {
                format!(
                    "{}: missing slide relationship {}",
                    path.display(),
                    slide_id.relationship_id
                )
            })?;
        if slide_relationship.rel_type != rel_types::SLIDE {
            return Err(format!(
                "{}: relationship {} is not a slide",
                path.display(),
                slide_id.relationship_id
            ));
        }
        let slide_part =
            OpcPackage::resolve_rel_target(&presentation_part, &slide_relationship.target);
        let layout_part = related_part(package, &slide_part, rel_types::SLIDE_LAYOUT, path)?;
        let master_part = related_part(package, &layout_part, rel_types::SLIDE_MASTER, path)?;
        let theme_part = related_part(package, &master_part, rel_types::THEME, path)?;
        let slide = CT_Slide::from_xml(part(package, &slide_part)?)
            .map_err(|error| format!("{} {slide_part}: {error}", path.display()))?;
        let layout = CT_SlideLayout::from_xml(part(package, &layout_part)?)
            .map_err(|error| format!("{} {layout_part}: {error}", path.display()))?;
        let master = CT_SlideMaster::from_xml(part(package, &master_part)?)
            .map_err(|error| format!("{} {master_part}: {error}", path.display()))?;
        let theme = CT_OfficeStyleSheet::from_xml(part(package, &theme_part)?)
            .map_err(|error| format!("{} {theme_part}: {error}", path.display()))?;
        let color_map = effective_color_map(&master, &layout, &slide);
        let mut context = ResolveCtx::new(
            &theme,
            color_map,
            &master,
            &layout,
            &slide,
            &default_text_style,
        );
        if let Some(styles) = table_styles.as_ref() {
            context = context.with_table_styles(styles);
        }
        let source_count = context
            .flatten()
            .into_iter()
            .filter(|item| source_shape_has_bounds(&context, item))
            .count();
        let (slide_media, slide_hyperlinks, slide_charts) = scoped_resources(
            package,
            [&slide_part, &layout_part, &master_part],
            &mut media,
        )?;
        let resolved = context
            .resolve_slide_with_chart_resources(
                size,
                &slide_media,
                &slide_hyperlinks,
                &slide_charts,
                &mut font_manager,
            )
            .map_err(|error| format!("{} {slide_part}: {error}", path.display()))?;
        if source_count != resolved.shapes.len() {
            return Err(format!(
                "{} {slide_part}: dropped bounded source shape, source {source_count}, resolved {}",
                path.display(),
                resolved.shapes.len()
            ));
        }
        source_counts.push(source_count);
        resolved_slides.push(resolved);
    }

    let input = RenderInput {
        slides: resolved_slides,
        media,
        fonts: Vec::new(),
        metadata: None,
    };
    let layout = layout_presentation_with_font_manager(&input, font_manager)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if layout.pages.len() != input.slides.len() {
        return Err(format!(
            "{}: rendered {} pages for {} slides",
            path.display(),
            layout.pages.len(),
            input.slides.len()
        ));
    }
    Ok(RenderedDeck {
        input,
        layout,
        source_counts,
    })
}

fn write_text_audit(path: &Path, input: &RenderInput, output: &Path) -> Result<(), String> {
    let deck = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("non-UTF-8 deck path {}", path.display()))?;
    let mut fonts = FontManager::new_deterministic().map_err(|error| error.to_string())?;
    let mut rows = vec![
        "deck\tslide\tshape\tx\ty\twidth\theight\tanchor\tautofit\tparagraph\tline_spacing\tdeclared_font\tactual_font\tfont_size\tascent\tdescent\tline_gap\ttext"
            .to_owned(),
    ];
    for (slide_index, slide) in input.slides.iter().enumerate() {
        for (shape_index, shape) in slide.shapes.iter().enumerate() {
            let ResolvedContent::Text(body) = &shape.content else {
                continue;
            };
            audit_text_body(
                deck,
                slide_index + 1,
                shape_index + 1,
                shape.bounds,
                body,
                &mut fonts,
                &mut rows,
            )?;
        }
    }
    fs::write(output, rows.join("\n") + "\n")
        .map_err(|error| format!("write {}: {error}", output.display()))
}

fn audit_text_body(
    deck: &str,
    slide: usize,
    shape: usize,
    bounds: oxml_layout::Rect,
    body: &ResolvedTextBody,
    fonts: &mut FontManager,
    rows: &mut Vec<String>,
) -> Result<(), String> {
    for (paragraph_index, paragraph) in body.paragraphs.iter().enumerate() {
        for run in &paragraph.runs {
            let (text, style) = match run {
                ResolvedTextRun::Text { text, style }
                | ResolvedTextRun::Field { text, style, .. } => (text.as_str(), style),
                ResolvedTextRun::Break => continue,
            };
            let declared = style
                .latin_typeface
                .as_deref()
                .or(style.east_asian_typeface.as_deref())
                .or(style.complex_script_typeface.as_deref())
                .or(style.symbol_typeface.as_deref());
            let font_id = fonts
                .resolve_font_for_text(declared, style.bold, style.italic, text)
                .map_err(|error| error.to_string())?;
            let size = style.font_size.unwrap_or(18.0);
            let metrics = fonts
                .metrics(font_id, size)
                .map_err(|error| error.to_string())?;
            let actual = fonts
                .font_data(font_id)
                .map_err(|error| error.to_string())?
                .family;
            rows.push(format!(
                "{}\t{}\t{}\t{:.6}\t{:.6}\t{:.6}\t{:.6}\t{:?}\t{:?}\t{}\t{:?}\t{}\t{}\t{:.6}\t{:.6}\t{:.6}\t{:.6}\t{}",
                escape_tsv(deck),
                slide,
                shape,
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
                body.anchor,
                body.autofit,
                paragraph_index + 1,
                paragraph.line_spacing,
                escape_tsv(declared.unwrap_or("")),
                escape_tsv(&actual),
                size,
                metrics.ascent,
                metrics.descent,
                metrics.line_gap,
                escape_tsv(text),
            ));
        }
    }
    Ok(())
}

fn table_styles(
    package: &OpcPackage,
    presentation_part: &str,
    deck: &Path,
) -> Result<Option<CT_TableStyleList>, String> {
    let Some(relationship) = package
        .get_part_rels(presentation_part)
        .and_then(|relationships| relationships.get_by_type(rel_types::TABLE_STYLES))
    else {
        return Ok(None);
    };
    let target = OpcPackage::resolve_rel_target(presentation_part, &relationship.target);
    CT_TableStyleList::from_xml(part(package, &target)?)
        .map(Some)
        .map_err(|error| format!("{} {target}: {error}", deck.display()))
}

fn scoped_resources(
    package: &OpcPackage,
    parts: [&str; 3],
    deck_media: &mut HashMap<MediaId, MediaData>,
) -> Result<(ScopedMediaIds, ScopedHyperlinkTargets, ScopedChartResources), String> {
    let slide = part_resources(package, parts[0], deck_media)?;
    let layout = part_resources(package, parts[1], deck_media)?;
    let master = part_resources(package, parts[2], deck_media)?;
    Ok((
        ScopedMediaIds {
            slide: slide.media_ids,
            layout: layout.media_ids,
            master: master.media_ids,
            media_content_types: deck_media
                .iter()
                .map(|(media_id, media)| (*media_id, media.content_type.clone()))
                .collect(),
        },
        ScopedHyperlinkTargets {
            slide: slide.hyperlinks,
            layout: layout.hyperlinks,
            master: master.hyperlinks,
        },
        ScopedChartResources {
            slide: slide.charts,
            layout: layout.charts,
            master: master.charts,
        },
    ))
}

struct PartResources {
    media_ids: HashMap<String, MediaId>,
    hyperlinks: HashMap<String, String>,
    charts: HashMap<String, ChartResource>,
}

fn part_resources(
    package: &OpcPackage,
    source_part: &str,
    deck_media: &mut HashMap<MediaId, MediaData>,
) -> Result<PartResources, String> {
    let mut media_ids = HashMap::new();
    let mut hyperlinks = HashMap::new();
    let mut charts = HashMap::new();
    let Some(relationships) = package.get_part_rels(source_part) else {
        return Ok(PartResources {
            media_ids,
            hyperlinks,
            charts,
        });
    };
    for relationship in &relationships.items {
        let external = relationship
            .target_mode
            .as_deref()
            .is_some_and(|mode| mode.eq_ignore_ascii_case("external"));
        if relationship.rel_type == rel_types::HYPERLINK && external {
            hyperlinks.insert(relationship.id.clone(), relationship.target.clone());
        }
        if relationship.rel_type == rel_types::CHART {
            let resource = if external {
                ChartResource::External(relationship.target.clone())
            } else {
                let target = OpcPackage::resolve_rel_target(source_part, &relationship.target);
                match package.get_part(&target) {
                    Some(bytes) => CT_ChartSpace::from_xml(bytes)
                        .map(|chart| ChartResource::Parsed(Box::new(chart)))
                        .unwrap_or_else(|error| ChartResource::Invalid(error.to_string())),
                    None => ChartResource::MissingTarget(target),
                }
            };
            charts.insert(relationship.id.clone(), resource);
        }
        if relationship.rel_type != rel_types::IMAGE || external {
            continue;
        }
        let target = OpcPackage::resolve_rel_target(source_part, &relationship.target);
        let Some(bytes) = package.get_part(&target) else {
            continue;
        };
        let content_type = package
            .content_types
            .content_type_for(&target)
            .unwrap_or("application/octet-stream")
            .to_owned();
        if !renderer_compatible_media(bytes, &content_type) {
            continue;
        }
        let media_id = MediaId::from_bytes(bytes);
        deck_media.entry(media_id).or_insert_with(|| MediaData {
            bytes: bytes.to_vec(),
            content_type,
        });
        media_ids.insert(relationship.id.clone(), media_id);
    }
    Ok(PartResources {
        media_ids,
        hyperlinks,
        charts,
    })
}

fn renderer_compatible_media(bytes: &[u8], content_type: &str) -> bool {
    let format = match ImageFormat::sniff(bytes) {
        Some(ImageFormat::Png) if content_type.eq_ignore_ascii_case("image/png") => {
            ImageFormat::Png
        }
        Some(ImageFormat::Jpeg)
            if content_type.eq_ignore_ascii_case("image/jpeg")
                || content_type.eq_ignore_ascii_case("image/jpg") =>
        {
            ImageFormat::Jpeg
        }
        _ => return false,
    };
    if !media_within_decode_bounds(bytes, format) {
        return false;
    }
    let Some(info) = probe(bytes).filter(|info| info.width_px > 0 && info.height_px > 0) else {
        return false;
    };
    if format == ImageFormat::Jpeg && (info.bit_depth != 8 || info.channels != 3) {
        return false;
    }
    let width = f64::from(info.width_px);
    let height = f64::from(info.height_px);
    let media_id = MediaId::from_bytes(bytes);
    [Color::BLACK, Color::WHITE].into_iter().any(|background| {
        let background_element = PositionedElement::FilledRect {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width,
                height,
            },
            color: background,
        };
        let baseline = LayoutResult::new(
            vec![PageFrame::new(
                1,
                width,
                height,
                vec![background_element.clone()],
            )],
            Vec::new(),
            None,
            Vec::new(),
        );
        let candidate = LayoutResult::new(
            vec![PageFrame::new(
                1,
                width,
                height,
                vec![
                    background_element,
                    PositionedElement::Image {
                        rect: Rect {
                            x: 0.0,
                            y: 0.0,
                            width,
                            height,
                        },
                        data: bytes.to_vec(),
                        content_type: content_type.to_owned(),
                        media_id,
                    },
                ],
            )],
            Vec::new(),
            None,
            Vec::new(),
        );
        oxml_pdf::render_page_to_png(&baseline, 0, 72.0)
            .zip(oxml_pdf::render_page_to_png(&candidate, 0, 72.0))
            .is_some_and(|(baseline, candidate)| baseline != candidate)
    })
}

const MAX_PREVIEW_BYTES: usize = 16 * 1024 * 1024;
const MAX_PREVIEW_DECODED_BYTES: usize = 16 * 1024 * 1024;

fn media_within_decode_bounds(bytes: &[u8], format: ImageFormat) -> bool {
    if bytes.len() > MAX_PREVIEW_BYTES {
        return false;
    }
    let Some(info) = probe(bytes) else {
        return false;
    };
    let Some(pixel_bytes) = usize::try_from(info.width_px)
        .ok()
        .and_then(|width| width.checked_mul(info.height_px as usize))
        .and_then(|pixels| pixels.checked_mul(4))
    else {
        return false;
    };
    if pixel_bytes > MAX_PREVIEW_DECODED_BYTES {
        return false;
    }
    if format != ImageFormat::Png {
        return true;
    }
    let Some(expected) = usize::try_from(info.width_px)
        .ok()
        .and_then(|width| width.checked_mul(info.channels as usize))
        .and_then(|stride| stride.checked_add(1))
        .and_then(|row| row.checked_mul(info.height_px as usize))
        .filter(|expected| *expected <= MAX_PREVIEW_DECODED_BYTES)
    else {
        return false;
    };
    let mut idat = Vec::new();
    let mut offset = 33usize;
    while offset < bytes.len() {
        let Some(length_bytes) = bytes.get(offset..offset.saturating_add(4)) else {
            return false;
        };
        let length = u32::from_be_bytes(length_bytes.try_into().unwrap()) as usize;
        let Some(kind_start) = offset.checked_add(4) else {
            return false;
        };
        let Some(payload_start) = kind_start.checked_add(4) else {
            return false;
        };
        let Some(payload_end) = payload_start.checked_add(length) else {
            return false;
        };
        let Some(chunk_end) = payload_end.checked_add(4) else {
            return false;
        };
        let (Some(kind), Some(payload), Some(_)) = (
            bytes.get(kind_start..payload_start),
            bytes.get(payload_start..payload_end),
            bytes.get(payload_end..chunk_end),
        ) else {
            return false;
        };
        if kind == b"IDAT" {
            if idat.len().saturating_add(payload.len()) > MAX_PREVIEW_BYTES {
                return false;
            }
            idat.extend_from_slice(payload);
        }
        if kind == b"IEND" {
            break;
        }
        offset = chunk_end;
    }
    !idat.is_empty()
        && miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(&idat, expected)
            .is_ok_and(|decoded| decoded.len() >= expected)
}

fn source_shape_has_bounds(context: &ResolveCtx<'_>, item: &FlattenedItem<'_>) -> bool {
    let FlattenedItem::Shape { child, .. } = item else {
        return false;
    };
    match child {
        ShapeTreeChild::Shape(shape) => context
            .effective_xfrm(shape)
            .as_ref()
            .is_some_and(transform_has_bounds),
        ShapeTreeChild::Picture(picture) => context
            .effective_picture_xfrm(picture)
            .as_ref()
            .is_some_and(transform_has_bounds),
        ShapeTreeChild::GraphicFrame(frame) => transform_has_bounds(&frame.transform),
        ShapeTreeChild::Connector(connector) => connector
            .shape_properties
            .transform
            .as_ref()
            .is_some_and(connector_transform_has_bounds),
        ShapeTreeChild::AlternateContent(alternate) => alternate
            .chart_choice()
            .is_some_and(|frame| transform_has_bounds(&frame.transform)),
        ShapeTreeChild::GroupShape(_) => false,
    }
}

fn transform_has_bounds(transform: &CT_Transform2D) -> bool {
    transform
        .extent
        .is_some_and(|extent| extent.cx.0 > 0 && extent.cy.0 > 0)
}

fn connector_transform_has_bounds(transform: &CT_Transform2D) -> bool {
    transform.extent.is_some_and(|extent| {
        extent.cx.0 >= 0 && extent.cy.0 >= 0 && (extent.cx.0 > 0 || extent.cy.0 > 0)
    })
}

fn effective_color_map(
    master: &CT_SlideMaster,
    layout: &CT_SlideLayout,
    slide: &CT_Slide,
) -> ColorMap {
    match slide
        .color_map_override
        .as_ref()
        .or(layout.color_map_override.as_ref())
    {
        Some(override_value) => match &override_value.kind {
            ColorMapOverrideKind::Master => master.color_map.clone(),
            ColorMapOverrideKind::Override(map) => map.clone(),
        },
        None => master.color_map.clone(),
    }
}

fn related_part(
    package: &OpcPackage,
    source_part: &str,
    relationship_type: &str,
    deck: &Path,
) -> Result<String, String> {
    let relationship = package
        .get_part_rels(source_part)
        .and_then(|relationships| relationships.get_by_type(relationship_type))
        .ok_or_else(|| {
            format!(
                "{} {source_part}: missing relationship {relationship_type}",
                deck.display()
            )
        })?;
    Ok(OpcPackage::resolve_rel_target(
        source_part,
        &relationship.target,
    ))
}

fn part<'a>(package: &'a OpcPackage, part_name: &str) -> Result<&'a [u8], String> {
    package
        .get_part(part_name)
        .ok_or_else(|| format!("missing package part {part_name}"))
}

fn escape_tsv(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}
