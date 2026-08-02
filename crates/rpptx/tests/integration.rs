use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

use oxml_drawing::theme::CT_OfficeStyleSheet;
use oxml_opc::relationship::rel_types;
use oxml_opc::{OpcPackage, content_types};
use rpptx::{Error, Presentation, ShapeKind, ShapeRef};
use rpptx_oxml::notes_parts::{CT_NotesMaster, CT_NotesSlide};
use rpptx_oxml::presentation::CT_Presentation;
use rpptx_oxml::slide_parts::{CT_Slide, CT_SlideLayout, CT_SlideMaster};

#[path = "../examples/dump_deck.rs"]
mod dump_deck;

const P_NS: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";
const A_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
const R_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const MC_NS: &str = "http://schemas.openxmlformats.org/markup-compatibility/2006";
const PRESENTATION_PART: &str = "/custom/presentation-main.xml";
const SLIDE_ONE_PART: &str = "/custom/slides/first.xml";
const SLIDE_TWO_PART: &str = "/custom/slides/second.xml";
const NOTES_PART: &str = "/custom/notes/speaker.xml";
const MODELLED_CONTENT_TYPES: [&str; 7] = [
    content_types::PRESENTATION,
    content_types::SLIDE,
    content_types::SLIDE_LAYOUT,
    content_types::SLIDE_MASTER,
    content_types::NOTES_SLIDE,
    content_types::NOTES_MASTER,
    content_types::THEME,
];

#[test]
fn all_corpus_modelled_parts_reparse_structurally() {
    let Some(paths) = corpus_paths() else {
        return;
    };
    let mut coverage = BTreeMap::new();
    for path in paths {
        let deck = deck_name(&path);
        let bytes = fs::read(&path).unwrap_or_else(|error| panic!("{deck}: {error}"));
        let package = open_opc(&bytes, deck);
        for (content_type, count) in modelled_part_bytes(&package, deck).1 {
            *coverage.entry(content_type).or_insert(0) += count;
        }
    }
    for content_type in MODELLED_CONTENT_TYPES {
        assert!(
            coverage.get(content_type).copied().unwrap_or(0) > 0,
            "corpus has no modelled part with content type {content_type}"
        );
    }
}

#[test]
fn all_modelled_corpus_packages_match_expected_parts() {
    let Some(paths) = corpus_paths() else {
        return;
    };
    let save_dir = std::env::var_os("RDOCX_PPTX_SAVE_DIR").map(PathBuf::from);
    if let Some(directory) = &save_dir {
        fs::create_dir_all(directory)
            .unwrap_or_else(|error| panic!("{}: {error}", directory.display()));
    }

    let mut saved_paths = Vec::new();
    for path in paths {
        let deck = deck_name(&path);
        let original_bytes = fs::read(&path).unwrap_or_else(|error| panic!("{deck}: {error}"));
        let original = open_opc(&original_bytes, deck);
        let (rewritten, _) = modelled_part_bytes(&original, deck);

        let mut expected = original.clone();
        for (part_name, bytes) in &rewritten {
            expected.set_part(part_name, bytes.clone());
        }
        let expected = reopen_written_package(&expected, deck, "expected package");

        let facade_bytes = Presentation::from_bytes(&original_bytes)
            .unwrap_or_else(|error| panic!("{deck}: open facade: {error}"))
            .to_bytes()
            .unwrap_or_else(|error| panic!("{deck}: save facade: {error}"));
        let mut actual = open_opc(&facade_bytes, deck);
        for (part_name, bytes) in &rewritten {
            let content_type = original
                .content_types
                .overrides
                .get(part_name)
                .map(String::as_str)
                .unwrap_or_else(|| panic!("{deck} {part_name}: missing content-type override"));
            if matches!(
                content_type,
                content_types::SLIDE_LAYOUT
                    | content_types::SLIDE_MASTER
                    | content_types::NOTES_MASTER
                    | content_types::THEME
            ) {
                actual.set_part(part_name, bytes.clone());
            }
        }
        let actual_bytes = write_package(&actual, deck, "modelled package");
        let actual = open_opc(&actual_bytes, deck);
        assert_packages_equal(&expected, &actual, deck);

        if let Some(directory) = &save_dir {
            let saved_path = directory.join(deck);
            fs::write(&saved_path, &actual_bytes)
                .unwrap_or_else(|error| panic!("{}: {error}", saved_path.display()));
            saved_paths.push(saved_path);
        }
    }

    saved_paths.sort();
    for path in saved_paths {
        eprintln!(
            "sha256\t{}\t{}",
            sha256(&path),
            path.file_name().unwrap().to_string_lossy()
        );
    }
}

#[test]
fn facade_saved_corpus_reopens_with_the_same_read_surface() {
    let Some(paths) = corpus_paths() else {
        return;
    };
    for path in paths {
        let deck = deck_name(&path);
        let bytes = fs::read(&path).unwrap_or_else(|error| panic!("{deck}: {error}"));
        let before = Presentation::from_bytes(&bytes)
            .unwrap_or_else(|error| panic!("{deck}: open facade: {error}"));
        let snapshot = read_surface(&before);
        let saved = before
            .to_bytes()
            .unwrap_or_else(|error| panic!("{deck}: save facade: {error}"));
        let after = Presentation::from_bytes(&saved)
            .unwrap_or_else(|error| panic!("{deck}: reopen facade: {error}"));
        assert_eq!(snapshot, read_surface(&after), "{deck}: read surface");
    }
}

#[test]
fn rpptx_is_an_unpublished_workspace_member() {
    let workspace = include_str!("../../../Cargo.toml");
    let manifest = include_str!("../Cargo.toml");
    assert!(workspace.contains("\"crates/rpptx\""));
    assert!(workspace.contains("rpptx = { path = \"crates/rpptx\", version = \"0.0.0\" }"));
    assert!(manifest.contains("name = \"rpptx\""));
    assert!(manifest.contains("version = \"0.0.0\""));
    assert!(manifest.contains("publish = false"));
}

#[test]
fn presentation_resolves_ordered_slides_and_notes() {
    let presentation = Presentation::from_bytes(&fixture_bytes()).unwrap();
    assert_eq!(presentation.len(), 2);
    assert!(!presentation.is_empty());
    let slides = presentation.slides().collect::<Vec<_>>();
    assert_eq!(slides[0].id(), 900);
    assert_eq!(slides[0].name(), Some("Second in storage, first in order"));
    assert_eq!(slides[0].text(), "plain\nA\tB\nC\tD\nnested\nfallback");
    assert_eq!(slides[0].notes_text().as_deref(), Some("speaker\nnote"));
    assert_eq!(slides[1].id(), 256);
    assert_eq!(slides[1].name(), Some("First in storage, second in order"));
    assert_eq!(slides[1].notes_text(), None);
}

#[test]
fn package_root_targets_with_dot_segments_resolve_to_their_parts() {
    for target in [
        "./custom/presentation-main.xml",
        "../../custom/presentation-main.xml",
    ] {
        let mut package = fixture_package();
        package
            .package_rels
            .items
            .iter_mut()
            .find(|relationship| relationship.rel_type == rel_types::DOCUMENT)
            .unwrap()
            .target = target.to_owned();

        let presentation = open_package(package).unwrap();
        assert_eq!(presentation.len(), 2, "target {target}");
        assert_eq!(presentation.slide(0).unwrap().id(), 900, "target {target}");
    }
}

#[test]
fn indexed_read_access_is_total() {
    let presentation = Presentation::from_bytes(&fixture_bytes()).unwrap();
    assert!(presentation.slide(2).is_none());
    let slide = presentation.slide(0).unwrap();
    assert!(slide.shape(6).is_none());
    let picture = slide.shape(1).unwrap();
    assert_eq!(picture.child_count(), 0);
    assert!(picture.child(0).is_none());
    assert_eq!(picture.children().len(), 0);
    let group = slide.shape(3).unwrap();
    assert_eq!(group.child_count(), 1);
    assert!(group.child(1).is_none());
    let alternate = slide.shape(5).unwrap();
    assert_eq!(alternate.child_count(), 1);
    assert!(alternate.child(1).is_none());

    let empty = Presentation::from_bytes(&package_bytes(empty_package())).unwrap();
    assert!(empty.is_empty());
    assert!(empty.slide(0).is_none());
    assert_eq!(empty.slides().len(), 0);
}

#[test]
fn shape_refs_cover_the_typed_shape_tree() {
    let presentation = Presentation::from_bytes(&fixture_bytes()).unwrap();
    let slide = presentation.slide(0).unwrap();
    let shapes = slide.shapes().collect::<Vec<_>>();
    assert_eq!(
        shapes.iter().map(|shape| shape.kind()).collect::<Vec<_>>(),
        vec![
            ShapeKind::Shape,
            ShapeKind::Picture,
            ShapeKind::GraphicFrame,
            ShapeKind::Group,
            ShapeKind::Connector,
            ShapeKind::AlternateContent,
        ]
    );
    assert_eq!(shapes[0].text().as_deref(), Some("plain"));
    assert_eq!(shapes[1].text(), None);
    assert_eq!(shapes[2].text().as_deref(), Some("A\tB\nC\tD"));
    assert_eq!(
        shapes[3].children().next().unwrap().text().as_deref(),
        Some("nested")
    );
    assert_eq!(
        shapes[5].children().next().unwrap().text().as_deref(),
        Some("fallback")
    );
}

#[test]
fn broken_presentation_graph_returns_contextual_errors() {
    let mut missing = fixture_package();
    missing.parts.remove(SLIDE_TWO_PART);
    assert!(matches!(
        open_package(missing),
        Err(Error::MissingPart { part_name }) if part_name == SLIDE_TWO_PART
    ));

    let mut wrong_type = fixture_package();
    wrong_type
        .get_or_create_part_rels(PRESENTATION_PART)
        .add_with_id(
            "ordered-first",
            rel_types::SLIDE_LAYOUT,
            "slides/second.xml",
        );
    assert!(matches!(
        open_package(wrong_type),
        Err(Error::WrongRelationshipType { relationship_id, .. })
            if relationship_id == "ordered-first"
    ));

    let mut external = fixture_package();
    let relationships = external.get_or_create_part_rels(PRESENTATION_PART);
    let relationship = relationships
        .items
        .iter_mut()
        .find(|relationship| relationship.id == "ordered-first")
        .unwrap();
    relationship.target_mode = Some("External".to_owned());
    assert!(matches!(
        open_package(external),
        Err(Error::ExternalRelationship { relationship_id, .. })
            if relationship_id == "ordered-first"
    ));

    let mut duplicate_notes = fixture_package();
    duplicate_notes
        .get_or_create_part_rels(SLIDE_TWO_PART)
        .add(rel_types::NOTES_SLIDE, "../notes/other.xml");
    assert!(matches!(
        open_package(duplicate_notes),
        Err(Error::DuplicateNotesSlides { count: 2, .. })
    ));

    let mut malformed = fixture_package();
    malformed.set_part(SLIDE_TWO_PART, b"<p:sld".to_vec());
    assert!(matches!(
        open_package(malformed),
        Err(Error::MalformedPart { part_name, .. }) if part_name == SLIDE_TWO_PART
    ));
}

#[test]
fn facade_bytes_reopen_with_the_same_read_model() {
    let original = fixture_bytes();
    let presentation = Presentation::from_bytes(&original).unwrap();
    let saved = presentation.to_bytes().unwrap();
    let reopened = Presentation::from_bytes(&saved).unwrap();
    assert_eq!(presentation.len(), reopened.len());
    for (before, after) in presentation.slides().zip(reopened.slides()) {
        assert_eq!(before.id(), after.id());
        assert_eq!(before.name(), after.name());
        assert_eq!(before.text(), after.text());
        assert_eq!(before.notes_text(), after.notes_text());
    }

    let original_package = OpcPackage::from_reader(Cursor::new(original)).unwrap();
    let saved_package = OpcPackage::from_reader(Cursor::new(saved)).unwrap();
    assert_eq!(
        original_package.get_part("/custom/opaque/data.bin"),
        saved_package.get_part("/custom/opaque/data.bin")
    );
    assert_eq!(
        original_package.get_part("/custom/opaque/raw.xml"),
        saved_package.get_part("/custom/opaque/raw.xml")
    );
}

#[test]
fn dump_deck_matches_python_pptx_1_0_2_for_the_corpus() {
    let corpus = corpus_dir();
    if !corpus.is_dir() {
        assert!(
            std::env::var_os("RDOCX_PPTX_CORPUS_REQUIRED").is_none(),
            "required corpus is missing at {}",
            corpus.display()
        );
        eprintln!(
            "corpus differential skipped because {} is absent",
            corpus.display()
        );
        return;
    }
    let entries = manifest_entries();
    assert_eq!(entries.len(), 50);
    let paths = entries
        .iter()
        .map(|entry| corpus.join(entry))
        .collect::<Vec<_>>();
    assert!(paths.iter().all(|path| path.is_file()));

    let mut command = Command::new("uv");
    command.args([
        "run",
        "--with",
        "python-pptx==1.0.2",
        "python",
        "-c",
        PYTHON_ORACLE,
    ]);
    command.args(&paths);
    let output = command.output().expect("run pinned python-pptx oracle");
    assert!(
        output.status.success(),
        "python-pptx oracle failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let oracle = String::from_utf8(output.stdout).unwrap();

    let mut rust = String::new();
    for path in &paths {
        rust.push_str("deck\t");
        rust.push_str(path.file_name().unwrap().to_str().unwrap());
        rust.push('\n');
        rust.push_str(&dump_deck::records_for_path(path).unwrap());
        rust.push('\n');
    }
    if rust != oracle {
        let rust_lines = rust.lines().collect::<Vec<_>>();
        let oracle_lines = oracle.lines().collect::<Vec<_>>();
        let mismatch = rust_lines
            .iter()
            .zip(&oracle_lines)
            .position(|(rust, oracle)| rust != oracle)
            .unwrap_or_else(|| rust_lines.len().min(oracle_lines.len()));
        panic!(
            "normalized output differs at line {}:\nrpptx: {}\npython-pptx: {}\nline counts: {} and {}",
            mismatch + 1,
            rust_lines.get(mismatch).copied().unwrap_or("<missing>"),
            oracle_lines.get(mismatch).copied().unwrap_or("<missing>"),
            rust_lines.len(),
            oracle_lines.len(),
        );
    }
}

fn corpus_paths() -> Option<Vec<PathBuf>> {
    let corpus = corpus_dir();
    if !corpus.is_dir() {
        assert!(
            std::env::var_os("RDOCX_PPTX_CORPUS_REQUIRED").is_none(),
            "required corpus is missing at {}",
            corpus.display()
        );
        eprintln!(
            "corpus round-trip skipped because {} is absent",
            corpus.display()
        );
        return None;
    }
    let entries = manifest_entries();
    assert_eq!(
        entries.len(),
        50,
        "the corpus manifest must contain 50 decks"
    );
    let mut paths = entries
        .into_iter()
        .map(|entry| corpus.join(entry))
        .collect::<Vec<_>>();
    paths.sort();
    assert!(
        paths.iter().all(|path| path.is_file()),
        "every corpus manifest entry must exist under {}",
        corpus.display()
    );
    Some(paths)
}

fn deck_name(path: &Path) -> &str {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_else(|| panic!("non-UTF-8 corpus filename: {}", path.display()))
}

fn open_opc(bytes: &[u8], deck: &str) -> OpcPackage {
    OpcPackage::from_reader(Cursor::new(bytes))
        .unwrap_or_else(|error| panic!("{deck}: open OPC package: {error}"))
}

fn modelled_part_bytes(
    package: &OpcPackage,
    deck: &str,
) -> (BTreeMap<String, Vec<u8>>, BTreeMap<&'static str, usize>) {
    let mut overrides = package.content_types.overrides.iter().collect::<Vec<_>>();
    overrides.sort_by_key(|(part_name, _)| *part_name);
    let mut rewritten = BTreeMap::new();
    let mut coverage = BTreeMap::new();
    for (part_name, content_type) in overrides {
        let Some(canonical_type) = MODELLED_CONTENT_TYPES
            .iter()
            .copied()
            .find(|candidate| *candidate == content_type)
        else {
            continue;
        };
        let Some(bytes) = package.get_part(part_name) else {
            panic!("{deck} {part_name}: content-type override has no package part");
        };
        let serialised = serialise_modelled_part(content_type, bytes, deck, part_name);
        if let Some(serialised) = serialised {
            *coverage.entry(canonical_type).or_insert(0) += 1;
            rewritten.insert(part_name.clone(), serialised);
        }
    }
    (rewritten, coverage)
}

fn serialise_modelled_part(
    content_type: &str,
    xml: &[u8],
    deck: &str,
    part_name: &str,
) -> Option<Vec<u8>> {
    let context = || format!("{deck} {part_name}");
    match content_type {
        content_types::PRESENTATION => {
            let parsed = CT_Presentation::from_xml(xml)
                .unwrap_or_else(|error| panic!("{}: parse presentation: {error}", context()));
            let serialised = parsed
                .to_xml()
                .unwrap_or_else(|error| panic!("{}: serialise presentation: {error}", context()));
            let reparsed = CT_Presentation::from_xml(&serialised)
                .unwrap_or_else(|error| panic!("{}: reparse presentation: {error}", context()));
            assert_eq!(parsed, reparsed, "{}: presentation structure", context());
            Some(serialised)
        }
        content_types::SLIDE => {
            let parsed = CT_Slide::from_xml(xml)
                .unwrap_or_else(|error| panic!("{}: parse slide: {error}", context()));
            let serialised = parsed
                .to_xml()
                .unwrap_or_else(|error| panic!("{}: serialise slide: {error}", context()));
            let reparsed = CT_Slide::from_xml(&serialised)
                .unwrap_or_else(|error| panic!("{}: reparse slide: {error}", context()));
            assert_eq!(parsed, reparsed, "{}: slide structure", context());
            Some(serialised)
        }
        content_types::SLIDE_LAYOUT => {
            let parsed = CT_SlideLayout::from_xml(xml)
                .unwrap_or_else(|error| panic!("{}: parse slide layout: {error}", context()));
            let serialised = parsed
                .to_xml()
                .unwrap_or_else(|error| panic!("{}: serialise slide layout: {error}", context()));
            let reparsed = CT_SlideLayout::from_xml(&serialised)
                .unwrap_or_else(|error| panic!("{}: reparse slide layout: {error}", context()));
            assert_eq!(parsed, reparsed, "{}: slide layout structure", context());
            Some(serialised)
        }
        content_types::SLIDE_MASTER => {
            let parsed = CT_SlideMaster::from_xml(xml)
                .unwrap_or_else(|error| panic!("{}: parse slide master: {error}", context()));
            let serialised = parsed
                .to_xml()
                .unwrap_or_else(|error| panic!("{}: serialise slide master: {error}", context()));
            let reparsed = CT_SlideMaster::from_xml(&serialised)
                .unwrap_or_else(|error| panic!("{}: reparse slide master: {error}", context()));
            assert_eq!(parsed, reparsed, "{}: slide master structure", context());
            Some(serialised)
        }
        content_types::NOTES_SLIDE => {
            let parsed = CT_NotesSlide::from_xml(xml)
                .unwrap_or_else(|error| panic!("{}: parse notes slide: {error}", context()));
            let serialised = parsed
                .to_xml()
                .unwrap_or_else(|error| panic!("{}: serialise notes slide: {error}", context()));
            let reparsed = CT_NotesSlide::from_xml(&serialised)
                .unwrap_or_else(|error| panic!("{}: reparse notes slide: {error}", context()));
            assert_eq!(parsed, reparsed, "{}: notes slide structure", context());
            Some(serialised)
        }
        content_types::NOTES_MASTER => {
            let parsed = CT_NotesMaster::from_xml(xml)
                .unwrap_or_else(|error| panic!("{}: parse notes master: {error}", context()));
            let serialised = parsed
                .to_xml()
                .unwrap_or_else(|error| panic!("{}: serialise notes master: {error}", context()));
            let reparsed = CT_NotesMaster::from_xml(&serialised)
                .unwrap_or_else(|error| panic!("{}: reparse notes master: {error}", context()));
            assert_eq!(parsed, reparsed, "{}: notes master structure", context());
            Some(serialised)
        }
        content_types::THEME => {
            let parsed = CT_OfficeStyleSheet::from_xml(xml)
                .unwrap_or_else(|error| panic!("{}: parse theme: {error}", context()));
            let serialised = parsed
                .to_xml()
                .unwrap_or_else(|error| panic!("{}: serialise theme: {error}", context()));
            let reparsed = CT_OfficeStyleSheet::from_xml(&serialised)
                .unwrap_or_else(|error| panic!("{}: reparse theme: {error}", context()));
            assert_eq!(parsed, reparsed, "{}: theme structure", context());
            Some(serialised)
        }
        _ => None,
    }
}

fn reopen_written_package(package: &OpcPackage, deck: &str, action: &str) -> OpcPackage {
    open_opc(&write_package(package, deck, action), deck)
}

fn write_package(package: &OpcPackage, deck: &str, action: &str) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    package
        .write_to(&mut output)
        .unwrap_or_else(|error| panic!("{deck}: {action}: {error}"));
    output.into_inner()
}

fn assert_packages_equal(expected: &OpcPackage, actual: &OpcPackage, deck: &str) {
    assert_eq!(
        expected.content_types.defaults, actual.content_types.defaults,
        "{deck}: content-type defaults"
    );
    assert_eq!(
        expected.content_types.overrides, actual.content_types.overrides,
        "{deck}: content-type overrides"
    );
    assert_eq!(
        expected.package_rels.items, actual.package_rels.items,
        "{deck}: package relationships"
    );
    let mut expected_relationship_parts = expected.part_rels.keys().collect::<Vec<_>>();
    let mut actual_relationship_parts = actual.part_rels.keys().collect::<Vec<_>>();
    expected_relationship_parts.sort();
    actual_relationship_parts.sort();
    assert_eq!(
        expected_relationship_parts, actual_relationship_parts,
        "{deck}: relationship part names"
    );
    for part_name in expected_relationship_parts {
        assert_eq!(
            expected.part_rels[part_name].items, actual.part_rels[part_name].items,
            "{deck} {part_name}: relationships"
        );
    }
    assert_eq!(
        expected.parts.len(),
        actual.parts.len(),
        "{deck}: part count"
    );
    let mut expected_part_names = expected.parts.keys().collect::<Vec<_>>();
    let mut actual_part_names = actual.parts.keys().collect::<Vec<_>>();
    expected_part_names.sort();
    actual_part_names.sort();
    assert_eq!(expected_part_names, actual_part_names, "{deck}: part names");
    for part_name in expected_part_names {
        assert_eq!(
            expected.parts[part_name], actual.parts[part_name],
            "{deck} {part_name}: part bytes"
        );
    }
}

fn read_surface(presentation: &Presentation) -> String {
    let mut output = String::new();
    for (slide_index, slide) in presentation.slides().enumerate() {
        writeln!(
            output,
            "slide\t{slide_index}\t{}\t{:?}\t{:?}\t{:?}",
            slide.id(),
            slide.name(),
            slide.text(),
            slide.notes_text()
        )
        .unwrap();
        for (shape_index, shape) in slide.shapes().enumerate() {
            append_shape_surface(&mut output, shape, &shape_index.to_string());
        }
    }
    output
}

fn append_shape_surface(output: &mut String, shape: ShapeRef<'_>, path: &str) {
    writeln!(
        output,
        "shape\t{path}\t{:?}\t{:?}",
        shape.kind(),
        shape.text()
    )
    .unwrap();
    for (child_index, child) in shape.children().enumerate() {
        append_shape_surface(output, child, &format!("{path}.{child_index}"));
    }
}

fn sha256(path: &Path) -> String {
    let output = Command::new("shasum")
        .args(["-a", "256"])
        .arg(path)
        .output()
        .unwrap_or_else(|error| panic!("{}: run shasum: {error}", path.display()));
    assert!(
        output.status.success(),
        "{}: shasum failed: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned()
}

fn open_package(package: OpcPackage) -> Result<Presentation, Error> {
    Presentation::from_bytes(&package_bytes(package))
}

fn fixture_bytes() -> Vec<u8> {
    package_bytes(fixture_package())
}

fn package_bytes(package: OpcPackage) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    package.write_to(&mut output).unwrap();
    output.into_inner()
}

fn empty_package() -> OpcPackage {
    let mut package = OpcPackage::with_main_part(
        PRESENTATION_PART.trim_start_matches('/'),
        content_types::PRESENTATION,
    );
    package.set_part(PRESENTATION_PART, presentation_xml(&[]));
    package
}

fn fixture_package() -> OpcPackage {
    let mut package = OpcPackage::with_main_part(
        PRESENTATION_PART.trim_start_matches('/'),
        content_types::PRESENTATION,
    );
    package.set_part(
        PRESENTATION_PART,
        presentation_xml(&[(900, "ordered-first"), (256, "ordered-second")]),
    );
    package.set_part(SLIDE_ONE_PART, simple_slide_xml());
    package.set_part(SLIDE_TWO_PART, rich_slide_xml());
    package.set_part(NOTES_PART, notes_xml());
    package.set_part("/custom/opaque/data.bin", vec![0, 1, 2, 255]);
    package.set_part(
        "/custom/opaque/raw.xml",
        b"<producer:raw xmlns:producer=\"urn:producer\">exact</producer:raw>".to_vec(),
    );
    package
        .content_types
        .add_override(SLIDE_ONE_PART, content_types::SLIDE);
    package
        .content_types
        .add_override(SLIDE_TWO_PART, content_types::SLIDE);
    package
        .content_types
        .add_override(NOTES_PART, content_types::NOTES_SLIDE);
    let relationships = package.get_or_create_part_rels(PRESENTATION_PART);
    relationships.add_with_id("ordered-first", rel_types::SLIDE, "slides/second.xml");
    relationships.add_with_id("ordered-second", rel_types::SLIDE, "slides/first.xml");
    package.get_or_create_part_rels(SLIDE_TWO_PART).add_with_id(
        "notes-link",
        rel_types::NOTES_SLIDE,
        "../notes/speaker.xml",
    );
    package
}

fn presentation_xml(slides: &[(u32, &str)]) -> Vec<u8> {
    let slide_ids = slides
        .iter()
        .map(|(id, relationship)| format!(r#"<p:sldId id="{id}" r:id="{relationship}"/>"#))
        .collect::<String>();
    format!(
        r#"<p:presentation xmlns:p="{P_NS}" xmlns:r="{R_NS}"><p:sldIdLst>{slide_ids}</p:sldIdLst><p:notesSz cx="6858000" cy="9144000"/></p:presentation>"#
    )
    .into_bytes()
}

fn simple_slide_xml() -> Vec<u8> {
    format!(
        r#"<p:sld xmlns:p="{P_NS}" xmlns:a="{A_NS}"><p:cSld name="First in storage, second in order"><p:spTree><p:nvGrpSpPr/><p:grpSpPr/></p:spTree></p:cSld></p:sld>"#
    )
    .into_bytes()
}

fn rich_slide_xml() -> Vec<u8> {
    let text_shape = |text: &str| {
        format!(
            r#"<p:sp><p:nvSpPr><p:cNvPr/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/><p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>{text}</a:t></a:r></a:p></p:txBody></p:sp>"#
        )
    };
    let picture = r#"<p:pic><p:nvPicPr><p:cNvPr/><p:cNvPicPr/><p:nvPr/></p:nvPicPr><p:blipFill/><p:spPr/></p:pic>"#;
    let table = r#"<p:graphicFrame><p:nvGraphicFramePr/><p:xfrm/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/table"><a:tbl><a:tblGrid><a:gridCol w="100"/><a:gridCol w="100"/></a:tblGrid><a:tr h="100"><a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>A</a:t></a:r></a:p></a:txBody><a:tcPr/></a:tc><a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>B</a:t></a:r></a:p></a:txBody><a:tcPr/></a:tc></a:tr><a:tr h="100"><a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>C</a:t></a:r></a:p></a:txBody><a:tcPr/></a:tc><a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>D</a:t></a:r></a:p></a:txBody><a:tcPr/></a:tc></a:tr></a:tbl></a:graphicData></a:graphic></p:graphicFrame>"#;
    let group = format!(
        "<p:grpSp><p:nvGrpSpPr/><p:grpSpPr/>{}</p:grpSp>",
        text_shape("nested")
    );
    let connector = r#"<p:cxnSp><p:nvCxnSpPr><p:cNvPr/><p:cNvCxnSpPr/><p:nvPr/></p:nvCxnSpPr><p:spPr/></p:cxnSp>"#;
    let alternate = format!(
        r#"<mc:AlternateContent><mc:Choice Requires="p14"><p:sp/></mc:Choice><mc:Fallback>{}</mc:Fallback></mc:AlternateContent>"#,
        text_shape("fallback")
    );
    format!(
        r#"<p:sld xmlns:p="{P_NS}" xmlns:a="{A_NS}" xmlns:mc="{MC_NS}"><p:cSld name="Second in storage, first in order"><p:spTree><p:nvGrpSpPr/><p:grpSpPr/>{}{picture}{table}{group}{connector}{alternate}</p:spTree></p:cSld></p:sld>"#,
        text_shape("plain")
    )
    .into_bytes()
}

fn notes_xml() -> Vec<u8> {
    format!(
        r#"<p:notes xmlns:p="{P_NS}" xmlns:a="{A_NS}"><p:cSld><p:spTree><p:nvGrpSpPr/><p:grpSpPr/><p:sp><p:nvSpPr><p:cNvPr/><p:cNvSpPr/><p:nvPr><p:ph/></p:nvPr></p:nvSpPr><p:spPr/><p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>speaker</a:t></a:r><a:br/><a:r><a:t>note</a:t></a:r></a:p></p:txBody></p:sp></p:spTree></p:cSld></p:notes>"#
    )
    .into_bytes()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
        .to_path_buf()
}

fn corpus_dir() -> PathBuf {
    std::env::var_os("RDOCX_PPTX_CORPUS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root().join("corpus/pptx"))
}

fn manifest_entries() -> Vec<&'static str> {
    include_str!("../../../scripts/pptx-corpus-manifest.tsv")
        .lines()
        .skip(1)
        .map(|line| line.split('\t').next().unwrap())
        .collect()
}

const PYTHON_ORACLE: &str = r#"
from pathlib import Path
import sys
import pptx
from pptx import Presentation
from pptx.shapes.shapetree import SlideShapeFactory

assert pptx.__version__ == "1.0.2", pptx.__version__
P = "http://schemas.openxmlformats.org/presentationml/2006/main"
MC = "http://schemas.openxmlformats.org/markup-compatibility/2006"
KINDS = {
    (P, "sp"): "Shape",
    (P, "pic"): "Picture",
    (P, "graphicFrame"): "GraphicFrame",
    (P, "grpSp"): "Group",
    (P, "cxnSp"): "Connector",
    (MC, "AlternateContent"): "AlternateContent",
}

def qname(element):
    tag = element.tag
    if not isinstance(tag, str) or not tag.startswith("{"):
        return (None, tag)
    namespace, local = tag[1:].split("}", 1)
    return (namespace, local)

def escape(value):
    return value.replace("\\", "\\\\").replace("\t", "\\t").replace("\r", "\\r").replace("\n", "\\n")

def option(value):
    return "-" if value is None else "+" + escape(value)

def normalized_text(value):
    return value.replace("\v", "\n")

def shape_text(element, shape_collection):
    kind = KINDS.get(qname(element))
    if kind == "Shape":
        if not any(qname(child) == (P, "txBody") for child in element):
            return None
        shape = SlideShapeFactory(element, shape_collection)
        return normalized_text(shape.text)
    if kind == "GraphicFrame":
        shape = SlideShapeFactory(element, shape_collection)
        if not shape.has_table:
            return None
        return "\n".join("\t".join(normalized_text(cell.text) for cell in row.cells) for row in shape.table.rows)
    return None

def shape_children(element):
    kind = KINDS.get(qname(element))
    if kind == "Group":
        return [child for child in element if qname(child) in KINDS]
    if kind == "AlternateContent":
        fallback = next((child for child in element if qname(child) == (MC, "Fallback")), None)
        return [] if fallback is None else [child for child in fallback if qname(child) in KINDS]
    return []

def append_shape(lines, slide_index, path, element, shape_collection):
    kind = KINDS[qname(element)]
    lines.append(f"shape\t{slide_index}\t{path}\t{kind}\t{option(shape_text(element, shape_collection))}")
    for index, child in enumerate(shape_children(element)):
        append_shape(lines, slide_index, f"{path}.{index}", child, shape_collection)

for filename in sys.argv[1:]:
    path = Path(filename)
    print("deck\t" + path.name)
    presentation = Presentation(path)
    for slide_index, slide in enumerate(presentation.slides):
        top_level = [child for child in slide.shapes._spTree if qname(child) in KINDS]
        shape_lines = []
        shape_texts = []
        def collect_text(element):
            value = shape_text(element, slide.shapes)
            if value:
                shape_texts.append(value)
            for child in shape_children(element):
                collect_text(child)
        for element in top_level:
            collect_text(element)
        notes = None
        if slide.has_notes_slide:
            frame = slide.notes_slide.notes_text_frame
            notes = "" if frame is None else normalized_text(frame.text)
        print(f"slide\t{slide_index}\t{slide.slide_id}\t{option(slide.name or None)}\t{escape(chr(10).join(shape_texts))}\t{option(notes)}")
        for shape_index, element in enumerate(top_level):
            append_shape(shape_lines, slide_index, str(shape_index), element, slide.shapes)
        print("\n".join(shape_lines))
"#;
