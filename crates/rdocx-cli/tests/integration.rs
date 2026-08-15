use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use oxml_opc::OpcPackage;
use oxml_opc::relationship::rel_types;
use rdocx::Document;
use serde_json::json;

static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

struct TempWorkspace {
    path: PathBuf,
}

impl TempWorkspace {
    fn new(label: &str) -> Self {
        let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("rdocx-cli-{label}-{}-{id}", std::process::id()));
        fs::create_dir(&path).expect("create temporary workspace");
        Self { path }
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rdocx"))
        .args(args)
        .output()
        .expect("run rdocx CLI")
}

fn assert_success(output: &Output, command: &str) {
    assert!(
        output.status.success(),
        "{command} failed with {}: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{command} wrote stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn fixture_document(paragraphs: &[&str]) -> Document {
    let mut document = Document::new();
    document.set_title("CLI fixture");
    document.set_author("rdocx tests");
    document.set_subject("command integration");
    document.set_keywords("cli,test");
    for text in paragraphs {
        document.add_paragraph(text);
    }
    document
}

fn write_document(path: &Path, paragraphs: &[&str]) {
    fixture_document(paragraphs)
        .save(path)
        .expect("write DOCX fixture");
}

fn path_text(path: &Path) -> &str {
    path.to_str().expect("temporary path is UTF-8")
}

fn png_dimensions(bytes: &[u8]) -> (u32, u32) {
    assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert_eq!(&bytes[12..16], b"IHDR");
    (
        u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
        u32::from_be_bytes(bytes[20..24].try_into().unwrap()),
    )
}

#[test]
fn inspect_json_uses_the_shared_schema() {
    let temp = TempWorkspace::new("inspect");
    let input = temp.path.join("structure.docx");
    let mut document = fixture_document(&["First", "Second"]);
    {
        let mut table = document.add_table(1, 2);
        table.cell(0, 0).unwrap().set_text("Left");
        table.cell(0, 1).unwrap().set_text("Right");
    }
    document.save(&input).unwrap();

    let output = cli(&["inspect", path_text(&input), "--json"]);
    assert_success(&output, "inspect");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(
        value,
        json!({
            "schema": 1,
            "file": path_text(&input),
            "paragraphs": 2,
            "tables": 1,
            "content_elements": 3,
            "metadata": {
                "title": "CLI fixture",
                "author": "rdocx tests",
                "subject": "command integration",
                "keywords": "cli,test",
            },
            "styles_used": [],
        })
    );
}

#[test]
fn text_prints_body_and_table_content_in_document_order() {
    let temp = TempWorkspace::new("text");
    let input = temp.path.join("content.docx");
    let mut document = fixture_document(&["Body first"]);
    {
        let mut table = document.add_table(1, 2);
        table.cell(0, 0).unwrap().set_text("Left cell");
        table.cell(0, 1).unwrap().set_text("Right cell");
    }
    document.add_paragraph("Body second");
    document.save(&input).unwrap();

    let output = cli(&["text", path_text(&input)]);
    assert_success(&output, "text");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Body first\nLeft cell\tRight cell\t\nBody second\n"
    );
}

#[test]
fn convert_writes_valid_formats_and_uses_the_shared_default_output() {
    let temp = TempWorkspace::new("convert");
    let input = temp.path.join("source.docx");
    write_document(&input, &["Converted content"]);

    let markdown = input.with_extension("md");
    let output = cli(&["convert", path_text(&input), "--to", "md"]);
    assert_success(&output, "convert markdown");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("Written to {}\n", markdown.display())
    );
    assert_eq!(
        fs::read_to_string(&markdown).unwrap(),
        "Converted content\n\n"
    );

    let html = temp.path.join("converted.html");
    let output = cli(&[
        "convert",
        path_text(&input),
        "--to",
        "html",
        "--output",
        path_text(&html),
    ]);
    assert_success(&output, "convert HTML");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("Written to {}\n", html.display())
    );
    let html_text = fs::read_to_string(html).unwrap();
    assert!(html_text.starts_with("<!DOCTYPE html>\n<html>"));
    assert!(html_text.contains("<p>Converted content</p>"));

    let pdf = temp.path.join("converted.pdf");
    let output = cli(&[
        "convert",
        path_text(&input),
        "--to",
        "pdf",
        "--output",
        path_text(&pdf),
    ]);
    assert_success(&output, "convert PDF");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("Written to {}\n", pdf.display())
    );
    let pdf_bytes = fs::read(pdf).unwrap();
    assert!(pdf_bytes.starts_with(b"%PDF-"));
    assert!(pdf_bytes.ends_with(b"%%EOF"));

    let png = temp.path.join("converted.png");
    let output = cli(&[
        "convert",
        path_text(&input),
        "--to",
        "png",
        "--dpi",
        "24",
        "--output",
        path_text(&png),
    ]);
    assert_success(&output, "convert PNG");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("Written to {}\n", png.display())
    );
    let dimensions = png_dimensions(&fs::read(png).unwrap());
    assert!(dimensions.0 > 0 && dimensions.1 > 0);
}

#[test]
fn diff_reports_changed_paragraphs_without_using_exit_status_as_a_verdict() {
    let temp = TempWorkspace::new("diff");
    let before = temp.path.join("before.docx");
    let after = temp.path.join("after.docx");
    write_document(&before, &["Same", "Old text"]);
    write_document(&after, &["Same", "New text"]);

    let output = cli(&["diff", path_text(&before), path_text(&after)]);
    assert_success(&output, "diff");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "--- {} (2 paragraphs, 0 tables)\n+++ {} (2 paragraphs, 0 tables)\n\n- [2] Old text\n+ [2] New text\n\n2 paragraph(s) differ.\n",
            before.display(),
            after.display()
        )
    );
}

#[test]
fn replace_writes_a_reopenable_document_and_reports_the_exact_count() {
    let temp = TempWorkspace::new("replace");
    let input = temp.path.join("template.docx");
    let replaced = temp.path.join("replaced.docx");
    write_document(&input, &["Hello {{name}}, {{name}} again"]);

    let output = cli(&[
        "replace",
        path_text(&input),
        "--placeholder",
        "{{name}}",
        "--value",
        "Reader",
        "--output",
        path_text(&replaced),
    ]);
    assert_success(&output, "replace");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "Replaced 2 occurrence(s) of \"{{{{name}}}}\" -> \"Reader\"\nWritten to {}\n",
            replaced.display()
        )
    );

    let reopened = Document::open(&replaced).unwrap();
    assert_eq!(
        reopened.paragraph(0).unwrap().text(),
        "Hello Reader, Reader again"
    );
    assert_eq!(
        Document::open(input).unwrap().paragraph(0).unwrap().text(),
        "Hello {{name}}, {{name}} again"
    );
}

#[test]
fn validate_exit_status_is_a_verdict() {
    let temp = TempWorkspace::new("validate");
    let valid = temp.path.join("valid.docx");
    let corrupt = temp.path.join("corrupt.docx");
    write_document(&valid, &["Valid content"]);

    let output = cli(&["validate", path_text(&valid)]);
    assert_success(&output, "validate valid document");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("OK — no issues found in {}\n", valid.display())
    );

    let mut package = OpcPackage::open(&valid).unwrap();
    let document_part = package.main_document_part().unwrap();
    let relationship_id = package
        .get_or_create_part_rels(&document_part)
        .add(rel_types::IMAGE, "media/missing.png");
    package.save(&corrupt).unwrap();

    let output = cli(&["validate", path_text(&corrupt)]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "1 error(s) in {}:\n  1. relationship {relationship_id} points at missing part /word/media/missing.png\n",
            corrupt.display()
        )
    );
}

#[test]
fn render_uses_the_bundled_font_deterministic_path() {
    let temp = TempWorkspace::new("render");
    let input = temp.path.join("visible.docx");
    let first_dir = temp.path.join("first");
    let second_dir = temp.path.join("second");
    let mut document = fixture_document(&[]);
    document
        .add_paragraph("")
        .add_run("Visible deterministic text")
        .set_font("Helvetica Neue");
    document.save(&input).unwrap();
    let expected = Document::open(&input)
        .unwrap()
        .render_page_to_png_deterministic(0, 24.0)
        .unwrap()
        .unwrap();

    let selected_output = cli(&[
        "render",
        path_text(&input),
        "--output-dir",
        path_text(&first_dir),
        "--dpi",
        "24",
        "--page",
        "0",
    ]);
    assert_success(&selected_output, "render selected page");
    let selected_png = first_dir.join("visible_page1.png");
    let selected_bytes = fs::read(&selected_png).unwrap();
    assert_eq!(
        String::from_utf8(selected_output.stdout).unwrap(),
        format!(
            "Page 1 -> {} ({} bytes)\n",
            selected_png.display(),
            selected_bytes.len()
        )
    );
    let dimensions = png_dimensions(&selected_bytes);
    assert!(dimensions.0 > 0 && dimensions.1 > 0);
    assert_eq!(selected_bytes, expected);

    let all_output = cli(&[
        "render",
        path_text(&input),
        "--output-dir",
        path_text(&second_dir),
        "--dpi",
        "24",
    ]);
    assert_success(&all_output, "render all pages");
    let all_png = second_dir.join("visible_page1.png");
    let all_bytes = fs::read(&all_png).unwrap();
    assert_eq!(
        String::from_utf8(all_output.stdout).unwrap(),
        format!(
            "Page 1 -> {} ({} bytes)\nRendered 1 page(s) at 24 DPI\n",
            all_png.display(),
            all_bytes.len()
        )
    );
    assert_eq!(all_bytes, expected);

    assert_eq!(fs::read(selected_png).unwrap(), fs::read(all_png).unwrap());
}
