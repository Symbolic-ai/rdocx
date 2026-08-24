//! CLI command implementations.

use std::path::{Path, PathBuf};

use oxml_cli_support::{
    StagedOutputSet, default_output_path, ensure_output_paths_available, json_envelope, parse_range,
};
use oxml_pdf::{RasterFormat, RasterOptions, RasterOutput};
use rpptx::{Presentation, ShapeRef};
use serde_json::json;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const MAX_RASTER_PIXELS: u64 = 8_000_000;
const MAX_LCS_CELLS: usize = 1_000_000;
const THUMBNAIL_WIDTH: f64 = 320.0;

pub fn inspect(file: &Path, as_json: bool) -> Result<()> {
    let presentation = Presentation::open(file)?;
    let size = presentation.slide_size();
    let core = presentation.core_properties();
    let slides = presentation
        .slides()
        .map(|slide| {
            json!({
                "id": slide.id(),
                "name": slide.name(),
                "hidden": slide.hidden(),
                "shapes": slide.shapes().len(),
            })
        })
        .collect::<Vec<_>>();

    if as_json {
        let value = json_envelope(json!({
            "file": file.display().to_string(),
            "slides": presentation.len(),
            "layouts": presentation.layout_count(),
            "slide_size": size.map(|(width, height)| json!({
                "width_emu": width.0,
                "height_emu": height.0,
            })),
            "metadata": {
                "title": core.and_then(|value| value.title.as_deref()),
                "creator": core.and_then(|value| value.creator.as_deref()),
                "subject": core.and_then(|value| value.subject.as_deref()),
                "description": core.and_then(|value| value.description.as_deref()),
                "keywords": core.and_then(|value| value.keywords.as_deref()),
                "last_modified_by": core.and_then(|value| value.last_modified_by.as_deref()),
                "created": core.and_then(|value| value.created.as_deref()),
                "modified": core.and_then(|value| value.modified.as_deref()),
            },
            "slide_details": slides,
        }))?;
        println!("{}", serde_json::to_string_pretty(&value)?);
        return Ok(());
    }

    println!("File: {}", file.display());
    println!("Slides: {}", presentation.len());
    println!("Layouts: {}", presentation.layout_count());
    if let Some((width, height)) = size {
        println!("Slide size: {} x {} EMU", width.0, height.0);
    }
    println!();
    println!("Metadata:");
    if let Some(core) = core {
        if let Some(value) = core.title.as_deref() {
            println!("  Title: {value}");
        }
        if let Some(value) = core.creator.as_deref() {
            println!("  Creator: {value}");
        }
        if let Some(value) = core.subject.as_deref() {
            println!("  Subject: {value}");
        }
        if let Some(value) = core.description.as_deref() {
            println!("  Description: {value}");
        }
        if let Some(value) = core.keywords.as_deref() {
            println!("  Keywords: {value}");
        }
        if let Some(value) = core.last_modified_by.as_deref() {
            println!("  Last modified by: {value}");
        }
        if let Some(value) = core.created.as_deref() {
            println!("  Created: {value}");
        }
        if let Some(value) = core.modified.as_deref() {
            println!("  Modified: {value}");
        }
        if core == &rpptx::CoreProperties::default() {
            println!("  (none)");
        }
    } else {
        println!("  (none)");
    }
    println!();
    for (index, slide) in presentation.slides().enumerate() {
        println!(
            "Slide {}: id={}, name={}, hidden={}, shapes={}",
            index + 1,
            slide.id(),
            slide.name().unwrap_or(""),
            slide.hidden(),
            slide.shapes().len()
        );
    }
    Ok(())
}

pub fn text(file: &Path) -> Result<()> {
    let presentation = Presentation::open(file)?;
    for slide in presentation.slides() {
        println!("{}", slide.text());
    }
    Ok(())
}

pub fn convert(
    file: &Path,
    format: &str,
    output: Option<&Path>,
    dpi: f64,
    slides: Option<&str>,
    quality: u8,
    transparent: bool,
) -> Result<()> {
    validate_dpi(dpi)?;
    let presentation = Presentation::open(file)?;
    let image_format = parse_image_format(format, quality, transparent);
    let extension = match (format, image_format.as_ref()) {
        ("pdf", _) => "pdf",
        (_, Ok((_, extension))) => extension,
        (other, Err(_)) => {
            return Err(format!("Unknown format: {other}. Supported: pdf, png, jpeg, tiff").into());
        }
    };
    let output = output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_path(file, extension));
    match format {
        "pdf" => {
            std::fs::write(&output, presentation.to_pdf_deterministic()?)?;
            println!("Written to {}", output.display());
        }
        "png" | "jpg" | "jpeg" | "tif" | "tiff" => {
            if presentation.is_empty() {
                return Err("cannot convert a presentation with no slides to an image".into());
            }
            let (_, layout) = presentation.render_deterministic()?;
            if layout.pages.len() != presentation.len() {
                return Err(format!(
                    "rendered {} image pages for {} slides",
                    layout.pages.len(),
                    presentation.len()
                )
                .into());
            }
            let selected = selected_zero_based_slides(presentation.len(), slides)?;
            for index in &selected {
                let page = &layout.pages[*index];
                validate_raster_dimensions(page.width, page.height, dpi)?;
            }
            let (format, extension) = image_format?;
            match format {
                RasterFormat::Tiff => {
                    let output_bytes =
                        oxml_pdf::render_pages(&layout, &selected, RasterOptions { dpi, format })?;
                    let RasterOutput::MultiPageTiff(tiff) = output_bytes else {
                        return Err("TIFF render did not produce one stream".into());
                    };
                    stage_and_publish(&[(output.clone(), tiff)])?;
                    println!("Written to {}", output.display());
                }
                RasterFormat::Png { .. } | RasterFormat::Jpeg { .. } => {
                    let output_paths =
                        convert_separate_output_paths(&output, extension, selected.len());
                    ensure_output_paths_available(&output_paths)?;
                    let mut staged = StagedOutputSet::new();
                    let mut rendered = Vec::with_capacity(selected.len());
                    for (one_based, (index, path)) in
                        selected.iter().zip(output_paths.iter()).enumerate()
                    {
                        let image =
                            render_one_raster_page(&layout, *index, RasterOptions { dpi, format })?;
                        staged.stage_bytes(path, &image)?;
                        rendered.push((one_based + 1, path.clone()));
                    }
                    staged.publish()?;
                    for (one_based, path) in rendered {
                        println!("Slide {one_based} -> {}", path.display());
                    }
                }
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}

pub fn diff(file_a: &Path, file_b: &Path) -> Result<()> {
    let presentation_a = Presentation::open(file_a)?;
    let presentation_b = Presentation::open(file_b)?;
    let text_a = presentation_a
        .slides()
        .map(|slide| slide.text())
        .collect::<Vec<_>>();
    let text_b = presentation_b
        .slides()
        .map(|slide| slide.text())
        .collect::<Vec<_>>();
    let common = longest_common_subsequence(&text_a, &text_b)?;
    let mut a = 0;
    let mut b = 0;
    for value in common {
        while text_a.get(a) != Some(&value) {
            println!("- [{}] {}", a + 1, text_a[a]);
            a += 1;
        }
        while text_b.get(b) != Some(&value) {
            println!("+ [{}] {}", b + 1, text_b[b]);
            b += 1;
        }
        a += 1;
        b += 1;
    }
    while a < text_a.len() {
        println!("- [{}] {}", a + 1, text_a[a]);
        a += 1;
    }
    while b < text_b.len() {
        println!("+ [{}] {}", b + 1, text_b[b]);
        b += 1;
    }
    Ok(())
}

fn longest_common_subsequence(a: &[String], b: &[String]) -> Result<Vec<String>> {
    let rows = a
        .len()
        .checked_add(1)
        .ok_or("first slide count is too large to diff")?;
    let columns = b
        .len()
        .checked_add(1)
        .ok_or("second slide count is too large to diff")?;
    let cells = rows
        .checked_mul(columns)
        .ok_or("slide diff LCS matrix size overflowed")?;
    if cells > MAX_LCS_CELLS {
        return Err(format!(
            "slide diff requires {cells} LCS cells, which exceeds the 1,000,000 LCS cell limit"
        )
        .into());
    }
    let mut lengths = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for a_index in 1..=a.len() {
        for b_index in 1..=b.len() {
            lengths[a_index][b_index] = if a[a_index - 1] == b[b_index - 1] {
                lengths[a_index - 1][b_index - 1] + 1
            } else {
                lengths[a_index - 1][b_index].max(lengths[a_index][b_index - 1])
            };
        }
    }
    let mut result = Vec::new();
    let mut a_index = a.len();
    let mut b_index = b.len();
    while a_index > 0 && b_index > 0 {
        if a[a_index - 1] == b[b_index - 1] {
            result.push(a[a_index - 1].clone());
            a_index -= 1;
            b_index -= 1;
        } else if lengths[a_index - 1][b_index] >= lengths[a_index][b_index - 1] {
            a_index -= 1;
        } else {
            b_index -= 1;
        }
    }
    result.reverse();
    Ok(result)
}

pub fn replace(file: &Path, placeholder: &str, value: &str, output: &Path) -> Result<()> {
    let mut presentation = Presentation::open(file)?;
    let count = presentation.replace_text(placeholder, value);
    presentation.save(output)?;
    println!("Replaced {count} occurrence(s)");
    println!("Written to {}", output.display());
    Ok(())
}

pub fn validate(file: &Path) -> Result<bool> {
    let presentation = Presentation::open(file)?;
    let issues = presentation.validate();
    for issue in &issues {
        eprintln!("{issue:?}");
    }
    if issues.is_empty() {
        println!("Validation passed: {}", file.display());
    } else {
        eprintln!("Validation failed with {} issue(s)", issues.len());
    }
    Ok(issues.is_empty())
}

pub fn render(
    file: &Path,
    output: Option<&Path>,
    dpi: f64,
    range: Option<&str>,
    format: &str,
    quality: u8,
    transparent: bool,
) -> Result<()> {
    validate_dpi(dpi)?;
    let presentation = Presentation::open(file)?;
    let (format, extension) = parse_image_format(format, quality, transparent)?;
    let selected = selected_zero_based_slides(presentation.len(), range)?;
    let output = output.unwrap_or_else(|| Path::new("."));
    let stem = file.file_stem().unwrap_or_default().to_string_lossy();
    let (_, layout) = presentation.render_deterministic()?;
    for index in &selected {
        let page = layout
            .pages
            .get(*index)
            .ok_or_else(|| format!("slide {} has no rendered page", index + 1))?;
        validate_raster_dimensions(page.width, page.height, dpi)?;
    }
    match format {
        RasterFormat::Tiff => {
            let output_bytes =
                oxml_pdf::render_pages(&layout, &selected, RasterOptions { dpi, format })?;
            let RasterOutput::MultiPageTiff(tiff) = output_bytes else {
                return Err("TIFF render did not produce one stream".into());
            };
            let path = output.join(format!("{stem}.tiff"));
            std::fs::create_dir_all(output)?;
            stage_and_publish(&[(path.clone(), tiff)])?;
            println!("Written to {}", path.display());
        }
        RasterFormat::Png { .. } | RasterFormat::Jpeg { .. } => {
            let output_paths = selected
                .iter()
                .map(|index| {
                    let one_based = index + 1;
                    output.join(format!("{stem}_slide{one_based}.{extension}"))
                })
                .collect::<Vec<_>>();
            std::fs::create_dir_all(output)?;
            ensure_output_paths_available(&output_paths)?;
            let mut staged = StagedOutputSet::new();
            let mut rendered = Vec::with_capacity(selected.len());
            for (index, path) in selected.iter().zip(output_paths.iter()) {
                let one_based = index + 1;
                let image = render_one_raster_page(&layout, *index, RasterOptions { dpi, format })?;
                staged.stage_bytes(path, &image)?;
                rendered.push((one_based, path.clone()));
            }
            staged.publish()?;
            for (one_based, path) in rendered {
                println!("Slide {one_based} -> {}", path.display());
            }
        }
    }
    Ok(())
}

pub fn thumbnail(file: &Path, output: Option<&Path>) -> Result<()> {
    let presentation = Presentation::open(file)?;
    if presentation.is_empty() {
        return Err("cannot thumbnail a presentation with no slides".into());
    }
    let (_, layout) = presentation.render_deterministic()?;
    let page = layout
        .pages
        .first()
        .ok_or("slide one has no rendered page")?;
    if !page.width.is_finite() || page.width <= 0.0 {
        return Err("slide one has an invalid rendered width".into());
    }
    let dpi = (THUMBNAIL_WIDTH - 1e-9) * 72.0 / page.width;
    validate_raster_dimensions(page.width, page.height, dpi)?;
    let png = oxml_pdf::render_page_to_png(&layout, 0, dpi)
        .ok_or("slide one did not rasterize for thumbnail")?;
    let output = output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_path(file, "png"));
    std::fs::write(&output, png)?;
    println!("Written to {}", output.display());
    Ok(())
}

pub fn outline(file: &Path) -> Result<()> {
    let presentation = Presentation::open(file)?;
    for (index, slide) in presentation.slides().enumerate() {
        let title_shape = slide.title();
        let title = normalize_outline_text(
            &title_shape
                .and_then(|shape| shape.text())
                .unwrap_or_default(),
        );
        if title.is_empty() {
            println!("Slide {}", index + 1);
        } else {
            println!("Slide {}: {title}", index + 1);
        }
        for shape in slide.shapes() {
            if title_shape == Some(shape) {
                continue;
            }
            print_shape_outline(shape);
        }
    }
    Ok(())
}

fn print_shape_outline(shape: ShapeRef<'_>) {
    if let Some(frame) = shape.text_frame() {
        print_frame_outline(frame);
    }
    if let Some(table) = shape.table() {
        for row in 0..table.row_count() {
            for column in 0..table.column_count() {
                let Some(cell) = table.cell(row, column) else {
                    continue;
                };
                if cell.is_spanned() {
                    continue;
                }
                if let Some(frame) = cell.text_frame() {
                    print_frame_outline(frame);
                }
            }
        }
    }
    for child in shape.children() {
        print_shape_outline(child);
    }
}

fn print_frame_outline(frame: rpptx::TextFrameRef<'_>) {
    for index in 0..frame.paragraph_count() {
        let Some(paragraph) = frame.paragraph(index) else {
            continue;
        };
        let text = normalize_outline_text(&paragraph.text());
        if !text.is_empty() {
            println!("{}- {text}", "  ".repeat(paragraph.level() as usize));
        }
    }
}

fn normalize_outline_text(text: &str) -> String {
    text.trim().replace(['\r', '\n', '\u{000b}'], " ")
}

fn validate_dpi(dpi: f64) -> Result<()> {
    if !dpi.is_finite() || dpi <= 0.0 {
        return Err("DPI must be a positive finite number".into());
    }
    Ok(())
}

fn parse_image_format(
    format: &str,
    quality: u8,
    transparent: bool,
) -> Result<(RasterFormat, &'static str)> {
    match format {
        "png" => Ok((
            RasterFormat::Png {
                transparent_background: transparent,
            },
            "png",
        )),
        "jpg" | "jpeg" => {
            if !(1..=100).contains(&quality) {
                return Err(format!("JPEG quality must be 1 through 100, got {quality}").into());
            }
            Ok((RasterFormat::Jpeg { quality }, "jpg"))
        }
        "tif" | "tiff" => Ok((RasterFormat::Tiff, "tiff")),
        other => Err(format!("Unknown image format: {other}. Supported: png, jpeg, tiff").into()),
    }
}

fn selected_zero_based_slides(slide_count: usize, range: Option<&str>) -> Result<Vec<usize>> {
    let selected = match range {
        Some(range) => parse_range(range)?,
        None => (1..=slide_count).collect(),
    };
    if let Some(index) = selected.iter().copied().find(|index| *index > slide_count) {
        return Err(format!("slide {index} is out of range for {slide_count} slides").into());
    }
    Ok(selected.into_iter().map(|index| index - 1).collect())
}

fn convert_separate_output_paths(
    output: &Path,
    extension: &str,
    page_count: usize,
) -> Vec<PathBuf> {
    if page_count == 1 {
        return vec![output.to_path_buf()];
    }
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    let stem = output.file_stem().unwrap_or_default().to_string_lossy();
    (1..=page_count)
        .map(|index| parent.join(format!("{stem}_{index:03}.{extension}")))
        .collect()
}

fn render_one_raster_page(
    layout: &oxml_layout::LayoutResult,
    page_index: usize,
    options: RasterOptions,
) -> Result<Vec<u8>> {
    let output = oxml_pdf::render_pages(layout, &[page_index], options)?;
    let RasterOutput::SeparatePages(mut pages) = output else {
        return Err("single-slide render did not produce a separate image".into());
    };
    if pages.len() != 1 {
        return Err("single-slide render returned the wrong page count".into());
    }
    Ok(pages.remove(0))
}

fn stage_and_publish(outputs: &[(PathBuf, Vec<u8>)]) -> Result<()> {
    let paths = outputs
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    ensure_output_paths_available(&paths)?;
    let mut staged = StagedOutputSet::new();
    for (path, bytes) in outputs {
        staged.stage_bytes(path, bytes)?;
    }
    staged.publish()?;
    Ok(())
}

fn validate_raster_dimensions(width_points: f64, height_points: f64, dpi: f64) -> Result<()> {
    let scale = dpi / 72.0;
    let width = (width_points * scale).ceil();
    let height = (height_points * scale).ceil();
    if !width.is_finite()
        || !height.is_finite()
        || width < 1.0
        || height < 1.0
        || width > u32::MAX as f64
        || height > u32::MAX as f64
    {
        return Err(format!("DPI {dpi} produces invalid raster dimensions").into());
    }
    let width = width as u64;
    let height = height as u64;
    let pixels = width
        .checked_mul(height)
        .ok_or("raster pixel count overflowed")?;
    if pixels > MAX_RASTER_PIXELS {
        return Err(format!(
            "DPI {dpi} produces a {width} x {height} raster, which exceeds the 8,000,000 pixel limit"
        )
        .into());
    }
    Ok(())
}
