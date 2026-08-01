use std::collections::HashSet;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_drawing::shape_props::CT_ShapeProperties;
use oxml_drawing::text::CT_TextBody;
use oxml_opc::OpcPackage;
use quick_xml::Reader;
use quick_xml::events::Event;
use rpptx_oxml::PRESENTATION_PART;
use rpptx_oxml::namespace::{P_NS, P_PREFIX, R_NS, R_PREFIX};
use rpptx_oxml::presentation::CT_Presentation;

const MANIFEST: &str = include_str!("../../../scripts/pptx-corpus-manifest.tsv");
const EXPECTED_DECKS: usize = 50;
type DrawingElements = (Vec<Vec<u8>>, Vec<Vec<u8>>);

#[derive(Debug)]
struct ManifestEntry<'a> {
    path: &'a str,
    producer: &'a str,
    digest: &'a str,
    url: &'a str,
}

fn manifest_entries() -> Vec<ManifestEntry<'static>> {
    let mut lines = MANIFEST.lines();
    assert_eq!(
        lines.next(),
        Some("path\tproducer\tsha256\turl"),
        "unexpected corpus manifest header"
    );
    lines
        .enumerate()
        .map(|(index, line)| {
            let fields: Vec<_> = line.split('\t').collect();
            assert_eq!(fields.len(), 4, "manifest line {}", index + 2);
            ManifestEntry {
                path: fields[0],
                producer: fields[1],
                digest: fields[2],
                url: fields[3],
            }
        })
        .collect()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn corpus_dir() -> PathBuf {
    workspace_root().join("corpus/pptx")
}

fn require_or_skip_corpus() -> Option<PathBuf> {
    let corpus = corpus_dir();
    if corpus.is_dir() {
        return Some(corpus);
    }
    assert_ne!(
        std::env::var_os("RDOCX_PPTX_CORPUS_REQUIRED").as_deref(),
        Some(std::ffi::OsStr::new("1")),
        "the pinned corpus is required but {} does not exist",
        corpus.display()
    );
    eprintln!(
        "corpus gate skipped because {} is absent, run scripts/fetch_pptx_corpus.py",
        corpus.display()
    );
    None
}

fn verify_fetched_corpus() {
    let status = Command::new("python3")
        .arg(workspace_root().join("scripts/fetch_pptx_corpus.py"))
        .arg("--check")
        .status()
        .expect("run corpus verifier");
    assert!(status.success(), "pinned corpus verification failed");
}

#[test]
fn rpptx_oxml_is_an_unpublished_workspace_member() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("name = \"rpptx-oxml\""));
    assert!(manifest.contains("version = \"0.0.0\""));
    assert!(manifest.contains("publish = false"));
    assert_eq!(
        P_NS,
        "http://schemas.openxmlformats.org/presentationml/2006/main"
    );
    assert_eq!(P_PREFIX, "p");
    assert_eq!(
        R_NS,
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
    );
    assert_eq!(R_PREFIX, "r");
    assert_eq!(PRESENTATION_PART, "/ppt/presentation.xml");
}

#[test]
fn corpus_manifest_is_complete_and_verified() {
    let entries = manifest_entries();
    assert_eq!(entries.len(), EXPECTED_DECKS);
    let mut paths = HashSet::new();
    let mut urls = HashSet::new();
    let mut producers = HashSet::new();
    for entry in &entries {
        assert!(paths.insert(entry.path), "duplicate path {}", entry.path);
        assert!(urls.insert(entry.url), "duplicate URL {}", entry.url);
        assert!(
            entry.path.ends_with(".pptx"),
            "non-pptx path {}",
            entry.path
        );
        assert!(
            !entry.path.contains('/'),
            "nested corpus path {}",
            entry.path
        );
        assert!(
            !entry.producer.is_empty(),
            "missing producer for {}",
            entry.path
        );
        assert!(
            entry.digest.len() == 64 && entry.digest.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "invalid SHA-256 for {}",
            entry.path
        );
        assert!(
            entry.url.starts_with("https://"),
            "non-HTTPS URL for {}",
            entry.path
        );
        producers.insert(entry.producer);
    }
    for expected in [
        "microsoft-powerpoint",
        "google-slides",
        "keynote-export",
        "libreoffice-impress",
    ] {
        assert!(producers.contains(expected), "missing producer {expected}");
    }
    if corpus_dir().is_dir() {
        verify_fetched_corpus();
    }
}

#[test]
fn code_built_presentation_package_round_trips_without_external_corpus() {
    let mut package = OpcPackage::with_main_part(
        "ppt/presentation.xml",
        oxml_opc::content_types::PRESENTATION,
    );
    package.set_part(PRESENTATION_PART, br#"<p:presentation/>"#.to_vec());
    let mut bytes = Cursor::new(Vec::new());
    package.write_to(&mut bytes).unwrap();
    bytes.set_position(0);
    let reopened = OpcPackage::from_reader(bytes).unwrap();
    assert_eq!(
        reopened.main_document_part().as_deref(),
        Some(PRESENTATION_PART)
    );
    assert_eq!(
        reopened.get_part(PRESENTATION_PART),
        Some(br#"<p:presentation/>"#.as_slice())
    );
}

#[test]
fn all_corpus_decks_round_trip_opaquely() {
    let Some(corpus) = require_or_skip_corpus() else {
        return;
    };
    verify_fetched_corpus();
    for entry in manifest_entries() {
        let path = corpus.join(entry.path);
        let original = OpcPackage::open(&path)
            .unwrap_or_else(|error| panic!("{}: open failed: {error}", entry.path));
        assert_eq!(
            original.main_document_part().as_deref(),
            Some(PRESENTATION_PART),
            "{}: wrong presentation main part",
            entry.path
        );
        let mut canonical = Cursor::new(Vec::new());
        original
            .write_to(&mut canonical)
            .unwrap_or_else(|error| panic!("{}: canonical save failed: {error}", entry.path));
        canonical.set_position(0);
        let saved = OpcPackage::from_reader(canonical)
            .unwrap_or_else(|error| panic!("{}: reopen failed: {error}", entry.path));
        assert_package_parts_equal(entry.path, &original, &saved);
    }
}

fn assert_package_parts_equal(deck: &str, original: &OpcPackage, saved: &OpcPackage) {
    assert_eq!(
        original.content_types.defaults, saved.content_types.defaults,
        "{deck}: content-type defaults changed"
    );
    assert_eq!(
        original.content_types.overrides, saved.content_types.overrides,
        "{deck}: content-type overrides changed"
    );
    assert_eq!(
        original.package_rels.items, saved.package_rels.items,
        "{deck}: package relationships changed"
    );
    assert_eq!(
        original.parts.len(),
        saved.parts.len(),
        "{deck}: part count changed"
    );
    for (part, bytes) in &original.parts {
        assert_eq!(
            saved.parts.get(part),
            Some(bytes),
            "{deck}: part {part} changed"
        );
    }
    assert_eq!(
        original.part_rels.len(),
        saved.part_rels.len(),
        "{deck}: relationship-part count changed"
    );
    for (part, relationships) in &original.part_rels {
        let saved_relationships = saved
            .part_rels
            .get(part)
            .unwrap_or_else(|| panic!("{deck}: relationship part {part} is missing"));
        assert_eq!(
            relationships.items, saved_relationships.items,
            "{deck}: relationships for {part} changed"
        );
    }
}

#[test]
fn carried_m7_drawingml_gate_passes_for_the_corpus() {
    let Some(corpus) = require_or_skip_corpus() else {
        return;
    };
    verify_fetched_corpus();
    let mut text_body_count = 0usize;
    let mut shape_properties_count = 0usize;
    for entry in manifest_entries() {
        let package = OpcPackage::open(corpus.join(entry.path))
            .unwrap_or_else(|error| panic!("{}: open failed: {error}", entry.path));
        for (part, bytes) in &package.parts {
            if !part.ends_with(".xml") {
                continue;
            }
            let (text_bodies, shape_properties) = drawing_elements(bytes)
                .unwrap_or_else(|error| panic!("{} {part}: XML scan failed: {error}", entry.path));
            for xml in text_bodies {
                let parsed = CT_TextBody::from_xml(&xml).unwrap_or_else(|error| {
                    panic!("{} {part}: txBody parse failed: {error}", entry.path)
                });
                let written = parsed.to_xml().unwrap_or_else(|error| {
                    panic!("{} {part}: txBody write failed: {error}", entry.path)
                });
                let reparsed = CT_TextBody::from_xml(&written).unwrap_or_else(|error| {
                    panic!(
                        "{} {part}: written txBody parse failed: {error}",
                        entry.path
                    )
                });
                assert_eq!(parsed, reparsed, "{} {part}: txBody changed", entry.path);
                text_body_count += 1;
            }
            for xml in shape_properties {
                let parsed = CT_ShapeProperties::from_xml(&xml).unwrap_or_else(|error| {
                    panic!("{} {part}: spPr parse failed: {error}", entry.path)
                });
                let written = parsed.to_xml().unwrap_or_else(|error| {
                    panic!("{} {part}: spPr write failed: {error}", entry.path)
                });
                let reparsed = CT_ShapeProperties::from_xml(&written).unwrap_or_else(|error| {
                    panic!("{} {part}: written spPr parse failed: {error}", entry.path)
                });
                assert_eq!(parsed, reparsed, "{} {part}: spPr changed", entry.path);
                shape_properties_count += 1;
            }
        }
    }
    assert!(
        text_body_count > 0,
        "the corpus contained no txBody elements"
    );
    assert!(
        shape_properties_count > 0,
        "the corpus contained no spPr elements"
    );
    eprintln!(
        "DrawingML corpus gate checked {text_body_count} txBody and {shape_properties_count} spPr elements"
    );
}

fn drawing_elements(xml: &[u8]) -> Result<DrawingElements, String> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut text_bodies = Vec::new();
    let mut shape_properties = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element)) if local_name(element.name().as_ref()) == b"txBody" => {
                text_bodies.push(
                    capture_element(&mut reader, &element).map_err(|error| error.to_string())?,
                );
            }
            Ok(Event::Empty(element)) if local_name(element.name().as_ref()) == b"txBody" => {
                text_bodies
                    .push(capture_empty_element(&element).map_err(|error| error.to_string())?);
            }
            Ok(Event::Start(element)) if local_name(element.name().as_ref()) == b"spPr" => {
                shape_properties.push(
                    capture_element(&mut reader, &element).map_err(|error| error.to_string())?,
                );
            }
            Ok(Event::Empty(element)) if local_name(element.name().as_ref()) == b"spPr" => {
                shape_properties
                    .push(capture_empty_element(&element).map_err(|error| error.to_string())?);
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => return Err(error.to_string()),
        }
        buffer.clear();
    }
    Ok((text_bodies, shape_properties))
}

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

#[test]
fn presentation_reads_any_prefix_and_writes_fixed_prefixes_in_schema_order() {
    let xml = format!(
        r#"<q:presentation xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:rel="{R_NS}" xmlns:v="urn:producer" marker="yes"><q:sldMasterIdLst><q:sldMasterId id="2147483648" rel:id="rId1"/></q:sldMasterIdLst><q:notesMasterIdLst><q:notesMasterId rel:id="rIdNotes"/></q:notesMasterIdLst><v:sldIdLst><v:sldId id="not-a-slide"/></v:sldIdLst><q:sldIdLst keep="list"><q:before/><q:sldId id="256" rel:id="rId2" v:id="producer-id"/><q:between/><q:sldId id="300" rel:id="rId3"/><q:after/></q:sldIdLst><q:sldSz cx="12192000" cy="6858000" type="screen16x9"/><q:notesSz cx="6858000" cy="9144000"/><q:custom value="a &amp; b"><q:n/></q:custom><q:defaultTextStyle/><q:extLst><q:ext uri="raw"/></q:extLst></q:presentation>"#
    );
    let parsed = CT_Presentation::from_xml(xml.as_bytes()).unwrap();
    assert_eq!(parsed.slide_master_ids.len(), 1);
    assert_eq!(parsed.slide_master_ids[0].id, Some(2_147_483_648));
    assert_eq!(parsed.slide_master_ids[0].relationship_id, "rId1");
    assert_eq!(
        parsed
            .slide_ids
            .iter()
            .map(|slide| (slide.id, slide.relationship_id.as_str()))
            .collect::<Vec<_>>(),
        vec![(256, "rId2"), (300, "rId3")]
    );
    assert_eq!(parsed.slide_size.as_ref().unwrap().cx.0, 12_192_000);
    assert_eq!(parsed.notes_size.cy.0, 9_144_000);
    assert!(parsed.default_text_style.is_some());

    let written = parsed.to_xml().unwrap();
    let written_text = std::str::from_utf8(&written).unwrap();
    assert!(written_text.starts_with(&format!(
        r#"<p:presentation xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="{R_NS}""#
    )));
    assert!(written_text.contains(r#"<p:sldMasterId id="2147483648" r:id="rId1"/>"#));
    assert!(written_text.contains(r#"<p:sldId id="256" r:id="rId2" v:id="producer-id"/>"#));
    assert!(written_text.contains(r#"<p:defaultTextStyle/>"#));
    assert!(written_text.contains(r#"<q:custom value="a &amp; b"><q:n/></q:custom>"#));
    assert!(written_text.contains(
        r#"<q:notesMasterIdLst><q:notesMasterId rel:id="rIdNotes"/></q:notesMasterIdLst>"#
    ));
    assert!(written_text.contains(r#"<v:sldIdLst><v:sldId id="not-a-slide"/></v:sldIdLst>"#));
    assert!(written_text.contains(
        r#"<q:before/><p:sldId id="256" r:id="rId2" v:id="producer-id"/><q:between/><p:sldId id="300" r:id="rId3"/><q:after/>"#
    ));
    let master = written_text.find("<p:sldMasterIdLst").unwrap();
    let notes_master = written_text.find("<q:notesMasterIdLst").unwrap();
    let slides = written_text.find("<p:sldIdLst").unwrap();
    let slide_size = written_text.find("<p:sldSz").unwrap();
    let notes_size = written_text.find("<p:notesSz").unwrap();
    let defaults = written_text.find("<p:defaultTextStyle").unwrap();
    let extensions = written_text.find("<q:extLst").unwrap();
    assert!(
        master < notes_master
            && notes_master < slides
            && slides < slide_size
            && slide_size < notes_size
            && notes_size < defaults
            && defaults < extensions
    );
    assert_eq!(parsed, CT_Presentation::from_xml(&written).unwrap());
}

#[test]
fn slide_ids_preserve_order_and_enforce_powerpoint_bounds() {
    let valid = format!(
        r#"<p:presentation xmlns:p="{P_NS}" xmlns:r="{R_NS}"><p:sldIdLst><p:sldId id="2147483647" r:id="last"/><p:sldId id="256" r:id="first"/></p:sldIdLst><p:notesSz cx="6858000" cy="9144000"/></p:presentation>"#
    );
    let parsed = CT_Presentation::from_xml(valid.as_bytes()).unwrap();
    assert_eq!(
        parsed
            .slide_ids
            .iter()
            .map(|slide| slide.id)
            .collect::<Vec<_>>(),
        vec![2_147_483_647, 256]
    );

    for invalid in [
        r#"<p:sldId id="255" r:id="rId1"/>"#,
        r#"<p:sldId id="2147483648" r:id="rId1"/>"#,
        r#"<p:sldId id="not-an-integer" r:id="rId1"/>"#,
    ] {
        let xml = format!(
            r#"<p:presentation xmlns:p="{P_NS}" xmlns:r="{R_NS}"><p:sldIdLst>{invalid}</p:sldIdLst><p:notesSz cx="6858000" cy="9144000"/></p:presentation>"#
        );
        assert!(
            CT_Presentation::from_xml(xml.as_bytes()).is_err(),
            "{invalid}"
        );
    }

    let duplicate = format!(
        r#"<p:presentation xmlns:p="{P_NS}" xmlns:r="{R_NS}"><p:sldIdLst><p:sldId id="256" r:id="rId1"/><p:sldId id="256" r:id="rId2"/></p:sldIdLst><p:notesSz cx="6858000" cy="9144000"/></p:presentation>"#
    );
    assert!(CT_Presentation::from_xml(duplicate.as_bytes()).is_err());

    for invalid_size in [
        r#"<p:notesSz cy="9144000"/>"#,
        r#"<p:notesSz cx="broken" cy="9144000"/>"#,
        r#"<p:notesSz cx="0" cy="9144000"/>"#,
    ] {
        let xml = format!(r#"<p:presentation xmlns:p="{P_NS}">{invalid_size}</p:presentation>"#);
        assert!(
            CT_Presentation::from_xml(xml.as_bytes()).is_err(),
            "{invalid_size}"
        );
    }

    let wrong_root_namespace =
        br#"<x:presentation xmlns:x="urn:not-presentationml"><x:notesSz cx="1" cy="1"/></x:presentation>"#;
    assert!(CT_Presentation::from_xml(wrong_root_namespace).is_err());

    let conflicting_fixed_prefix = format!(
        r#"<q:presentation xmlns:q="{P_NS}" xmlns:p="urn:producer"><q:notesSz cx="6858000" cy="9144000"/></q:presentation>"#
    );
    assert!(CT_Presentation::from_xml(conflicting_fixed_prefix.as_bytes()).is_err());

    let nested_conflicting_prefix = format!(
        r#"<q:presentation xmlns:q="{P_NS}" xmlns:rel="{R_NS}"><q:sldIdLst xmlns:r="urn:producer"><q:sldId id="256" rel:id="rId1"/></q:sldIdLst><q:notesSz cx="6858000" cy="9144000"/></q:presentation>"#
    );
    assert!(CT_Presentation::from_xml(nested_conflicting_prefix.as_bytes()).is_err());
}

#[test]
fn zero_slide_template_round_trips() {
    let xml = format!(
        r#"<p:presentation xmlns:p="{P_NS}"><p:sldIdLst/><p:sldSz cx="12192000" cy="6858000"/><p:notesSz cx="6858000" cy="9144000"/></p:presentation>"#
    );
    let parsed = CT_Presentation::from_xml(xml.as_bytes()).unwrap();
    assert!(parsed.slide_ids.is_empty());
    let written = parsed.to_xml().unwrap();
    assert!(
        std::str::from_utf8(&written)
            .unwrap()
            .contains("<p:sldIdLst/>")
    );
    assert_eq!(parsed, CT_Presentation::from_xml(&written).unwrap());
}

#[test]
fn every_corpus_presentation_part_round_trips_structurally() {
    let Some(corpus) = require_or_skip_corpus() else {
        return;
    };
    verify_fetched_corpus();
    let mut presentation_count = 0usize;
    for entry in manifest_entries() {
        let package = OpcPackage::open(corpus.join(entry.path))
            .unwrap_or_else(|error| panic!("{}: open failed: {error}", entry.path));
        let xml = package
            .get_part(PRESENTATION_PART)
            .unwrap_or_else(|| panic!("{}: presentation part missing", entry.path));
        let parsed = CT_Presentation::from_xml(xml)
            .unwrap_or_else(|error| panic!("{}: presentation parse failed: {error}", entry.path));
        let written = parsed
            .to_xml()
            .unwrap_or_else(|error| panic!("{}: presentation write failed: {error}", entry.path));
        let reparsed = CT_Presentation::from_xml(&written).unwrap_or_else(|error| {
            panic!("{}: written presentation parse failed: {error}", entry.path)
        });
        assert_eq!(
            parsed, reparsed,
            "{}: presentation changed structurally",
            entry.path
        );
        presentation_count += 1;
    }
    assert_eq!(presentation_count, EXPECTED_DECKS);
    eprintln!("PresentationML corpus gate checked {presentation_count} presentation parts");
}
