use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use oxml_opc::OpcPackage;
use oxml_opc::relationship::rel_types;
use rpptx::{CT_TextCharacterProperties, Emu, Presentation};

static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

struct TempWorkspace {
    path: PathBuf,
}

impl TempWorkspace {
    fn new(label: &str) -> Self {
        let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("rpptx-cli-{label}-{}-{id}", std::process::id()));
        fs::create_dir_all(&path).expect("create temporary workspace");
        Self { path }
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rpptx"))
        .args(args)
        .output()
        .expect("run rpptx CLI")
}

fn write_deck(path: &Path, texts: &[&str]) {
    let mut presentation = Presentation::new().expect("open bundled template");
    for text in texts {
        presentation.add_slide(6).expect("add blank slide");
        presentation
            .slide_mut(presentation.len() - 1)
            .unwrap()
            .add_textbox(Emu(100_000), Emu(100_000), Emu(3_000_000), Emu(800_000))
            .expect("add text box")
            .set_text(text)
            .expect("set slide text");
    }
    presentation.save(path).expect("write fixture deck");
}

fn corpus_dir() -> PathBuf {
    std::env::var_os("RDOCX_PPTX_CORPUS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/pptx"))
}

#[test]
fn validate_rejects_corruption_and_accepts_the_pinned_corpus() {
    let temp = TempWorkspace::new("validate");
    let valid = temp.path.join("valid.pptx");
    write_deck(&valid, &["valid"]);
    let mut package = OpcPackage::open(&valid).expect("open fixture package");
    let slide = package
        .content_types
        .overrides
        .iter()
        .find_map(|(part, content_type)| {
            (content_type
                == "application/vnd.openxmlformats-officedocument.presentationml.slide+xml")
                .then_some(part.clone())
        })
        .expect("fixture slide part");
    package
        .get_or_create_part_rels(&slide)
        .add(rel_types::IMAGE, "../media/missing.png");
    let corrupt = temp.path.join("corrupt.pptx");
    package.save(&corrupt).expect("write corrupted deck");
    assert!(
        !cli(&["validate", corrupt.to_str().unwrap()])
            .status
            .success()
    );

    let corpus = corpus_dir();
    assert!(
        corpus.is_dir(),
        "required corpus missing at {}",
        corpus.display()
    );
    let entries = include_str!("../../../scripts/pptx-corpus-manifest.tsv")
        .lines()
        .skip(1)
        .map(|line| line.split('\t').next().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 50);
    for entry in entries {
        let deck = corpus.join(entry);
        assert!(
            deck.is_file(),
            "missing pinned corpus deck {}",
            deck.display()
        );
        let output = cli(&["validate", deck.to_str().unwrap()]);
        assert!(
            output.status.success(),
            "validate failed for {}: {}",
            deck.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn inspect_and_text_report_presentation_order() {
    let temp = TempWorkspace::new("inspect-text");
    let deck = temp.path.join("ordered.pptx");
    write_deck(&deck, &["first slide", "second slide"]);
    let mut presentation = Presentation::open(&deck).unwrap();
    let core = presentation.core_properties_mut();
    core.title = Some("Quarterly deck".to_owned());
    core.creator = Some("A. Presenter".to_owned());
    core.subject = Some("Plain inspect metadata".to_owned());
    core.description = Some("Metadata command regression".to_owned());
    core.keywords = Some("slides, metadata".to_owned());
    core.last_modified_by = Some("F-144".to_owned());
    core.created = Some("2026-08-13T10:00:00Z".to_owned());
    core.modified = Some("2026-08-13T11:00:00Z".to_owned());
    presentation.save(&deck).unwrap();
    let inspected = cli(&["inspect", deck.to_str().unwrap(), "--json"]);
    assert!(inspected.status.success());
    let value: serde_json::Value = serde_json::from_slice(&inspected.stdout).unwrap();
    assert_eq!(value["schema"], 1);
    assert_eq!(value["slides"], 2);
    assert_eq!(value["slide_details"][0]["shapes"], 1);

    let inspected = cli(&["inspect", deck.to_str().unwrap()]);
    assert!(inspected.status.success());
    let inspected = String::from_utf8(inspected.stdout).unwrap();
    for expected in [
        "Title: Quarterly deck",
        "Creator: A. Presenter",
        "Subject: Plain inspect metadata",
        "Description: Metadata command regression",
        "Keywords: slides, metadata",
        "Last modified by: F-144",
        "Created: 2026-08-13T10:00:00Z",
        "Modified: 2026-08-13T11:00:00Z",
    ] {
        assert!(inspected.contains(expected), "missing {expected:?}");
    }

    let text = cli(&["text", deck.to_str().unwrap()]);
    assert!(text.status.success());
    assert_eq!(
        String::from_utf8(text.stdout).unwrap(),
        "first slide\nsecond slide\n"
    );

    let help = cli(&["--help"]);
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).unwrap();
    for command in [
        "inspect", "text", "convert", "diff", "replace", "validate", "render",
    ] {
        assert!(help.contains(command), "missing command {command}");
    }
    assert!(!help.contains("thumbnail"));
    assert!(!help.contains("outline"));
}

#[test]
fn convert_rejects_png_rasters_above_the_pixel_budget() {
    let temp = TempWorkspace::new("convert-dpi-bound");
    let deck = temp.path.join("bounded.pptx");
    let output = temp.path.join("bounded.png");
    write_deck(&deck, &["bounded"]);

    let result = cli(&[
        "convert",
        deck.to_str().unwrap(),
        "--to",
        "png",
        "--output",
        output.to_str().unwrap(),
        "--dpi",
        "330",
    ]);

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("8,000,000 pixel limit"));
    assert!(!output.exists());
}

#[test]
fn render_rejects_png_rasters_above_the_pixel_budget() {
    let temp = TempWorkspace::new("render-dpi-bound");
    let deck = temp.path.join("bounded.pptx");
    let output = temp.path.join("rendered");
    write_deck(&deck, &["bounded"]);

    let result = cli(&[
        "render",
        deck.to_str().unwrap(),
        "--output",
        output.to_str().unwrap(),
        "--dpi",
        "330",
    ]);

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("8,000,000 pixel limit"));
    assert!(!output.join("bounded_slide1.png").exists());
}

#[test]
fn convert_rejects_zero_slide_png_without_creating_output() {
    let temp = TempWorkspace::new("convert-empty");
    let deck = temp.path.join("empty.pptx");
    let output = temp.path.join("empty.png");
    let presentation = Presentation::new().unwrap();
    assert!(presentation.is_empty());
    presentation.save(&deck).unwrap();

    let result = cli(&[
        "convert",
        deck.to_str().unwrap(),
        "--to",
        "png",
        "--output",
        output.to_str().unwrap(),
    ]);

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("no slides"));
    assert!(result.stdout.is_empty());
    assert!(!output.exists());
}

#[test]
fn convert_and_render_write_deterministic_pdf_and_png_outputs() {
    let temp = TempWorkspace::new("render");
    let deck = temp.path.join("rendered.pptx");
    write_deck(&deck, &["one", "two"]);
    let pdf = temp.path.join("rendered.pdf");
    let converted = cli(&["convert", deck.to_str().unwrap(), "--to", "pdf"]);
    assert!(converted.status.success());
    assert!(fs::read(&pdf).unwrap().starts_with(b"%PDF-"));

    let converted = cli(&["convert", deck.to_str().unwrap(), "--to", "png"]);
    assert!(converted.status.success());
    assert!(
        fs::read(temp.path.join("rendered_001.png"))
            .unwrap()
            .starts_with(b"\x89PNG")
    );
    assert!(
        fs::read(temp.path.join("rendered_002.png"))
            .unwrap()
            .starts_with(b"\x89PNG")
    );

    let output_dir = temp.path.join("selected");
    let rendered = cli(&[
        "render",
        deck.to_str().unwrap(),
        "--output",
        output_dir.to_str().unwrap(),
        "--slide",
        "2",
    ]);
    assert!(rendered.status.success());
    assert!(
        fs::read(output_dir.join("rendered_slide2.png"))
            .unwrap()
            .starts_with(b"\x89PNG")
    );
    assert!(!output_dir.join("rendered_slide1.png").exists());
}

#[test]
fn convert_streams_crafted_multi_slide_pngs_with_one_based_names() {
    let temp = TempWorkspace::new("convert-stream");
    let deck = temp.path.join("streamed.pptx");
    let texts = vec!["streamed"; 24];
    write_deck(&deck, &texts);

    let converted = cli(&[
        "convert",
        deck.to_str().unwrap(),
        "--to",
        "png",
        "--dpi",
        "72",
    ]);

    assert!(converted.status.success());
    let stdout = String::from_utf8(converted.stdout).unwrap();
    for one_based in 1..=24 {
        let output = temp.path.join(format!("streamed_{one_based:03}.png"));
        assert!(fs::read(output).unwrap().starts_with(b"\x89PNG"));
        assert!(stdout.contains(&format!("Slide {one_based} ->")));
    }
    let commands = include_str!("../src/commands.rs");
    assert!(commands.contains("render_page_to_png"));
    assert!(
        !commands.contains("render_all_pages"),
        "convert must not retain every encoded PNG"
    );
}

#[test]
fn diff_reports_slide_text_lcs_changes() {
    let temp = TempWorkspace::new("diff");
    let before = temp.path.join("before.pptx");
    let after = temp.path.join("after.pptx");
    write_deck(&before, &["same", "removed"]);
    write_deck(&after, &["same", "added"]);
    let output = cli(&["diff", before.to_str().unwrap(), after.to_str().unwrap()]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("- [2] removed"));
    assert!(stdout.contains("+ [2] added"));
}

#[test]
fn diff_rejects_large_repeated_slide_matrix_above_cell_budget() {
    let temp = TempWorkspace::new("diff-bound");
    let deck = temp.path.join("repeated.pptx");
    let repeated = vec!["repeat"; 1_000];
    write_deck(&deck, &repeated);

    let output = cli(&["diff", deck.to_str().unwrap(), deck.to_str().unwrap()]);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("1,000,000 LCS cell limit"));
}

#[test]
fn replacement_preserves_formatting_and_opaque_parts() {
    let temp = TempWorkspace::new("replace");
    let source = temp.path.join("source.pptx");
    let mut presentation = Presentation::new().unwrap();
    presentation.add_slide(6).unwrap();
    let mut slide = presentation.slide_mut(0).unwrap();
    let mut shape = slide
        .add_textbox(Emu(1), Emu(2), Emu(3_000_000), Emu(800_000))
        .unwrap();
    shape.set_text("pre old").unwrap();
    let mut frame = shape.text_frame().unwrap();
    let mut paragraph = frame.paragraph_mut(0).unwrap();
    let mut first_properties = CT_TextCharacterProperties::default();
    first_properties.bold = Some(true);
    paragraph
        .run_mut(0)
        .unwrap()
        .set_properties(first_properties.clone());
    let mut second = paragraph.add_run(" needle post");
    let mut second_properties = CT_TextCharacterProperties::default();
    second_properties.italic = Some(true);
    second.set_properties(second_properties.clone());
    let mut package =
        OpcPackage::from_reader(Cursor::new(presentation.to_bytes().unwrap())).unwrap();
    package.set_part("/custom/opaque.bin", b"opaque bytes".to_vec());
    package
        .content_types
        .add_default("bin", "application/octet-stream");
    package.save(&source).unwrap();

    let output = temp.path.join("replaced.pptx");
    let result = cli(&[
        "replace",
        source.to_str().unwrap(),
        "--placeholder",
        "old needle",
        "--value",
        "new",
        "--output",
        output.to_str().unwrap(),
    ]);
    assert!(result.status.success());
    let reopened = Presentation::open(&output).unwrap();
    let paragraph = reopened
        .slide(0)
        .unwrap()
        .shape(0)
        .unwrap()
        .text_frame()
        .unwrap()
        .paragraph(0)
        .unwrap();
    assert_eq!(paragraph.text(), "pre new post");
    assert_eq!(
        paragraph.run(0).unwrap().properties(),
        Some(&first_properties)
    );
    assert_eq!(
        paragraph.run(1).unwrap().properties(),
        Some(&second_properties)
    );
    let package = OpcPackage::open(&output).unwrap();
    assert_eq!(
        package.get_part("/custom/opaque.bin"),
        Some(b"opaque bytes".as_slice())
    );
}
