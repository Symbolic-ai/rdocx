//! Render the pinned PresentationML corpus through deterministic fonts.

use std::env;
use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use oxml_layout::{FontManager, LayoutResult};
use oxml_opc::OpcPackage;
use rpptx_layout::{ResolvedContent, ResolvedTextBody, ResolvedTextRun};
use rpptx_render::RenderInput;

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
    let mut bytes = std::io::Cursor::new(Vec::new());
    package
        .write_to(&mut bytes)
        .map_err(|error| format!("{}: serialize package for facade: {error}", path.display()))?;
    let presentation = rpptx::Presentation::from_bytes(&bytes.into_inner())
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let (input, layout) = presentation
        .render_deterministic()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let source_counts = input
        .slides
        .iter()
        .map(|slide| slide.shapes.len())
        .collect();
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

fn escape_tsv(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}
