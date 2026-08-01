use std::collections::HashSet;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_drawing::color::{ColorMapSlot, ThemeColorSlot};
use oxml_drawing::shape_props::CT_ShapeProperties;
use oxml_drawing::text::CT_TextBody;
use oxml_opc::relationship::rel_types;
use oxml_opc::{OpcPackage, content_types};
use quick_xml::Reader;
use quick_xml::events::Event;
use rpptx_oxml::PRESENTATION_PART;
use rpptx_oxml::namespace::{P_NS, P_PREFIX, R_NS, R_PREFIX};
use rpptx_oxml::presentation::CT_Presentation;
use rpptx_oxml::shape_tree::{CT_ShapeTree, ShapeTreeChild};
use rpptx_oxml::slide_parts::{CT_Slide, CT_SlideLayout, CT_SlideMaster, ColorMapOverrideKind};

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

#[test]
fn slide_layout_and_master_write_their_own_schema_order() {
    let slide = format!(
        r#"<p:sld xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer"><x:transition/><p:cSld><p:spTree><p:nvGrpSpPr/><p:grpSpPr/></p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr><p:transition/><p:timing/><p:extLst/></p:sld>"#
    );
    let layout = format!(
        r#"<p:sldLayout xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><p:cSld><p:spTree><p:nvGrpSpPr/><p:grpSpPr/></p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr><p:hf/><p:timing/><p:transition/><p:extLst/></p:sldLayout>"#
    );
    let master = format!(
        r#"<p:sldMaster xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer"><p:cSld><p:spTree><p:nvGrpSpPr/><p:grpSpPr/></p:spTree></p:cSld><p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/><p:sldLayoutIdLst/><p:transition/><p:timing/><p:hf/><x:beforeTextStyles><x:data/></x:beforeTextStyles><p:txStyles><p:titleStyle/><p:bodyStyle/><p:otherStyle/></p:txStyles><p:extLst/></p:sldMaster>"#
    );

    let written_slide = CT_Slide::from_xml(slide.as_bytes())
        .unwrap()
        .to_xml()
        .unwrap();
    let written_layout = CT_SlideLayout::from_xml(layout.as_bytes())
        .unwrap()
        .to_xml()
        .unwrap();
    let written_master = CT_SlideMaster::from_xml(master.as_bytes())
        .unwrap()
        .to_xml()
        .unwrap();
    assert_order(
        &written_slide,
        &[
            "<x:transition",
            "<p:cSld",
            "<p:clrMapOvr",
            "<p:transition",
            "<p:timing",
            "<p:extLst",
        ],
    );
    assert_order(
        &written_layout,
        &[
            "<p:cSld",
            "<p:clrMapOvr",
            "<p:hf",
            "<p:timing",
            "<p:transition",
            "<p:extLst",
        ],
    );
    assert_order(
        &written_master,
        &[
            "<p:cSld",
            "<p:clrMap",
            "<p:sldLayoutIdLst",
            "<p:transition",
            "<p:timing",
            "<p:hf",
            "<x:beforeTextStyles",
            "<p:txStyles",
            "<p:extLst",
        ],
    );
}

#[test]
fn colour_maps_read_any_prefix_and_preserve_extensions() {
    let xml = format!(
        r#"<q:sld xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer"><q:cSld><q:spTree><q:nvGrpSpPr/><q:grpSpPr/></q:spTree></q:cSld><q:clrMapOvr x:keep="yes"><x:before/><d:overrideClrMapping bg1="dk1" tx1="lt1" bg2="dk2" tx2="lt2" accent1="accent6" accent2="accent5" accent3="accent4" accent4="accent3" accent5="accent2" accent6="accent1" hlink="folHlink" folHlink="hlink" x:map="kept"><x:ext/></d:overrideClrMapping><x:after/></q:clrMapOvr></q:sld>"#
    );
    let parsed = CT_Slide::from_xml(xml.as_bytes()).unwrap();
    let override_map = parsed.color_map_override.as_ref().unwrap();
    let ColorMapOverrideKind::Override(map) = &override_map.kind else {
        panic!("expected an override colour map");
    };
    assert_eq!(
        map.theme_slot(ColorMapSlot::Background1),
        ThemeColorSlot::Dark1
    );
    assert_eq!(
        map.theme_slot(ColorMapSlot::Accent1),
        ThemeColorSlot::Accent6
    );
    let written = parsed.to_xml().unwrap();
    let text = std::str::from_utf8(&written).unwrap();
    assert!(text.contains("<p:clrMapOvr x:keep=\"yes\"><x:before/><a:overrideClrMapping"));
    assert!(text.contains("x:map=\"kept\""));
    assert!(text.contains("<x:ext/></a:overrideClrMapping><x:after/>"));
    assert_eq!(parsed, CT_Slide::from_xml(&written).unwrap());
}

#[test]
fn modelled_nested_elements_reject_fixed_prefix_rebinding() {
    let colour_choice = format!(
        r#"<q:sld xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main"><q:cSld><q:spTree><q:nvGrpSpPr/><q:grpSpPr/></q:spTree></q:cSld><q:clrMapOvr><d:overrideClrMapping xmlns:a="urn:producer" bg1="dk1" tx1="lt1" bg2="dk2" tx2="lt2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"><a:raw/></d:overrideClrMapping></q:clrMapOvr></q:sld>"#
    );
    assert!(CT_Slide::from_xml(colour_choice.as_bytes()).is_err());

    let text_style = format!(
        r#"<q:sldMaster xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main"><q:cSld><q:spTree><q:nvGrpSpPr/><q:grpSpPr/></q:spTree></q:cSld><q:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/><q:txStyles><q:titleStyle xmlns:p="urn:producer"><p:raw/></q:titleStyle><q:bodyStyle/><q:otherStyle/></q:txStyles></q:sldMaster>"#
    );
    assert!(CT_SlideMaster::from_xml(text_style.as_bytes()).is_err());

    let default_text_level = format!(
        r#"<q:presentation xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main"><q:notesSz cx="6858000" cy="9144000"/><q:defaultTextStyle><d:lvl1pPr xmlns:a="urn:producer"><a:raw/></d:lvl1pPr></q:defaultTextStyle></q:presentation>"#
    );
    assert!(CT_Presentation::from_xml(default_text_level.as_bytes()).is_err());

    let master_text_level = format!(
        r#"<q:sldMaster xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main"><q:cSld><q:spTree><q:nvGrpSpPr/><q:grpSpPr/></q:spTree></q:cSld><q:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/><q:txStyles><q:titleStyle><d:lvl1pPr xmlns:a="urn:producer"><a:raw/></d:lvl1pPr></q:titleStyle><q:bodyStyle/><q:otherStyle/></q:txStyles></q:sldMaster>"#
    );
    assert!(CT_SlideMaster::from_xml(master_text_level.as_bytes()).is_err());

    let required_shell = format!(
        r#"<q:spTree xmlns:q="{P_NS}"><q:nvGrpSpPr xmlns:p="urn:producer"><p:raw/></q:nvGrpSpPr><q:grpSpPr/></q:spTree>"#
    );
    assert!(CT_ShapeTree::from_xml(required_shell.as_bytes()).is_err());

    let group_transform = format!(
        r#"<q:spTree xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main"><q:nvGrpSpPr/><q:grpSpPr><d:xfrm xmlns:a="urn:producer"><a:raw/></d:xfrm></q:grpSpPr></q:spTree>"#
    );
    assert!(CT_ShapeTree::from_xml(group_transform.as_bytes()).is_err());

    let nested_group = format!(
        r#"<q:spTree xmlns:q="{P_NS}"><q:nvGrpSpPr/><q:grpSpPr/><q:grpSp xmlns:p="urn:producer"><q:nvGrpSpPr/><q:grpSpPr/><p:raw/></q:grpSp></q:spTree>"#
    );
    assert!(CT_ShapeTree::from_xml(nested_group.as_bytes()).is_err());
}

#[test]
fn common_slide_data_uses_the_typed_shape_tree() {
    let producer = r#"<x:producer xmlns:x="urn:producer"><x:data/></x:producer>"#;
    let xml = format!(
        r#"<q:sld xmlns:q="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><q:cSld name="Named"><q:bg><q:bgPr/></q:bg><q:spTree marker="a &amp; b"><q:nvGrpSpPr/><q:grpSpPr/>{producer}</q:spTree><q:extLst/></q:cSld></q:sld>"#
    );
    let parsed = CT_Slide::from_xml(xml.as_bytes()).unwrap();
    assert_eq!(parsed.common_slide_data.name.as_deref(), Some("Named"));
    assert!(parsed.common_slide_data.shape_tree.children.is_empty());
    let written = parsed.to_xml().unwrap();
    assert!(
        written
            .windows(producer.len())
            .any(|window| window == producer.as_bytes())
    );
    assert_eq!(parsed, CT_Slide::from_xml(&written).unwrap());
}

#[test]
fn every_corpus_slide_layout_and_master_round_trips_structurally() {
    let Some(corpus) = require_or_skip_corpus() else {
        return;
    };
    verify_fetched_corpus();
    let mut counts = [0usize; 3];
    for entry in manifest_entries() {
        let package = OpcPackage::open(corpus.join(entry.path)).unwrap();
        for (part, content_type) in &package.content_types.overrides {
            if !package.parts.contains_key(part) {
                continue;
            }
            let xml = package
                .get_part(part)
                .unwrap_or_else(|| panic!("{} {part}: missing part", entry.path));
            match content_type.as_str() {
                content_types::SLIDE => {
                    let parsed = CT_Slide::from_xml(xml)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path));
                    let written = parsed.to_xml().unwrap();
                    assert_eq!(parsed, CT_Slide::from_xml(&written).unwrap());
                    counts[0] += 1;
                }
                content_types::SLIDE_LAYOUT => {
                    let parsed = CT_SlideLayout::from_xml(xml)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path));
                    let written = parsed.to_xml().unwrap();
                    assert_eq!(parsed, CT_SlideLayout::from_xml(&written).unwrap());
                    counts[1] += 1;
                }
                content_types::SLIDE_MASTER => {
                    let parsed = CT_SlideMaster::from_xml(xml)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path));
                    let written = parsed.to_xml().unwrap();
                    assert_eq!(parsed, CT_SlideMaster::from_xml(&written).unwrap());
                    counts[2] += 1;
                }
                _ => {}
            }
        }
    }
    assert!(
        counts.iter().all(|count| *count > 0),
        "part counts: {counts:?}"
    );
    eprintln!("PresentationML corpus gate checked {counts:?} slide, layout, and master parts");
}

#[test]
fn corpus_part_relationship_counts_are_valid() {
    let Some(corpus) = require_or_skip_corpus() else {
        return;
    };
    verify_fetched_corpus();
    for entry in manifest_entries() {
        let package = OpcPackage::open(corpus.join(entry.path)).unwrap();
        for (part, content_type) in &package.content_types.overrides {
            if !package.parts.contains_key(part) {
                continue;
            }
            let required_type = match content_type.as_str() {
                content_types::SLIDE => Some(rel_types::SLIDE_LAYOUT),
                content_types::SLIDE_LAYOUT => Some(rel_types::SLIDE_MASTER),
                content_types::SLIDE_MASTER => Some(rel_types::THEME),
                _ => None,
            };
            let Some(required_type) = required_type else {
                continue;
            };
            let count = package
                .get_part_rels(part)
                .map(|rels| {
                    rels.items
                        .iter()
                        .filter(|rel| rel.rel_type == required_type)
                        .count()
                })
                .unwrap_or(0);
            assert_eq!(count, 1, "{} {part}: {required_type}", entry.path);
        }
    }
}

fn assert_order(xml: &[u8], tags: &[&str]) {
    let text = std::str::from_utf8(xml).unwrap();
    let positions: Vec<_> = tags
        .iter()
        .map(|tag| {
            text.find(tag)
                .unwrap_or_else(|| panic!("missing {tag}: {text}"))
        })
        .collect();
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]), "{text}");
}

#[test]
fn shape_tree_requires_non_visual_and_group_properties_in_order() {
    let valid = format!(
        r#"<q:spTree xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main"><q:nvGrpSpPr marker="kept"/><q:grpSpPr><d:xfrm><d:off x="10" y="20"/><d:ext cx="30" cy="40"/><d:chOff x="1" y="2"/><d:chExt cx="3" cy="4"/></d:xfrm><d:solidFill><d:srgbClr val="112233"/></d:solidFill></q:grpSpPr><q:sp/></q:spTree>"#
    );
    let parsed = CT_ShapeTree::from_xml(valid.as_bytes()).unwrap();
    let transform = parsed.group_transform().unwrap();
    assert_eq!(transform.offset.unwrap().x.0, 10);
    assert_eq!(transform.child_extent.unwrap().cy.0, 4);
    let written = parsed.to_xml().unwrap();
    assert_order(&written, &["<p:nvGrpSpPr", "<p:grpSpPr", "<q:sp"]);
    assert!(std::str::from_utf8(&written).unwrap().contains("<a:xfrm>"));
    assert_eq!(parsed, CT_ShapeTree::from_xml(&written).unwrap());

    for invalid in [
        format!(r#"<p:spTree xmlns:p="{P_NS}"><p:grpSpPr/></p:spTree>"#),
        format!(r#"<p:spTree xmlns:p="{P_NS}"><p:nvGrpSpPr/></p:spTree>"#),
        format!(r#"<p:spTree xmlns:p="{P_NS}"><p:grpSpPr/><p:nvGrpSpPr/></p:spTree>"#),
        format!(
            r#"<p:spTree xmlns:p="{P_NS}"><p:nvGrpSpPr/><p:grpSpPr/><p:nvGrpSpPr/></p:spTree>"#
        ),
        format!(r#"<p:spTree xmlns:p="{P_NS}"><p:nvGrpSpPr/><p:grpSpPr/><p:grpSpPr/></p:spTree>"#),
    ] {
        assert!(
            CT_ShapeTree::from_xml(invalid.as_bytes()).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn all_six_child_variants_keep_document_order() {
    let payloads = [
        r#"<p:sp marker="shape"><p:spPr/></p:sp>"#,
        r#"<p:pic marker="picture"><p:blipFill/></p:pic>"#,
        r#"<p:graphicFrame marker="frame"><a:graphic/></p:graphicFrame>"#,
        r#"<p:cxnSp marker="connector"><p:spPr/></p:cxnSp>"#,
        r#"<mc:AlternateContent marker="alternate"><mc:Choice Requires="p14"><p:sp/></mc:Choice></mc:AlternateContent>"#,
    ];
    let xml = format!(
        r#"<p:spTree xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:x="urn:producer"><p:nvGrpSpPr/><p:grpSpPr/>{}<x:between/>{}<p:grpSp><p:nvGrpSpPr/><p:grpSpPr/><p:sp marker="nested"/></p:grpSp>{}{}{}</p:spTree>"#,
        payloads[0], payloads[1], payloads[2], payloads[3], payloads[4]
    );
    let parsed = CT_ShapeTree::from_xml(xml.as_bytes()).unwrap();
    assert!(matches!(parsed.children[0], ShapeTreeChild::Shape(_)));
    assert!(matches!(parsed.children[1], ShapeTreeChild::Picture(_)));
    assert!(matches!(parsed.children[2], ShapeTreeChild::GroupShape(_)));
    assert!(matches!(
        parsed.children[3],
        ShapeTreeChild::GraphicFrame(_)
    ));
    assert!(matches!(parsed.children[4], ShapeTreeChild::Connector(_)));
    assert!(matches!(
        parsed.children[5],
        ShapeTreeChild::AlternateContent(_)
    ));
    let written = parsed.to_xml().unwrap();
    assert_order(
        &written,
        &[
            "marker=\"shape\"",
            "<x:between",
            "marker=\"picture\"",
            "marker=\"nested\"",
            "marker=\"frame\"",
            "marker=\"connector\"",
            "marker=\"alternate\"",
        ],
    );
    for payload in payloads {
        assert!(
            written
                .windows(payload.len())
                .any(|window| window == payload.as_bytes())
        );
    }
    assert_eq!(parsed, CT_ShapeTree::from_xml(&written).unwrap());
}

#[test]
fn nested_group_shape_tree_round_trips_with_tree_shape_preserved() {
    let xml = format!(
        r#"<p:sld xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><p:cSld><p:spTree><p:nvGrpSpPr/><p:grpSpPr/><p:grpSp><p:nvGrpSpPr/><p:grpSpPr><a:xfrm><a:off x="100" y="200"/><a:ext cx="300" cy="400"/><a:chOff x="10" y="20"/><a:chExt cx="30" cy="40"/></a:xfrm></p:grpSpPr><p:sp marker="outer"/><p:grpSp><p:nvGrpSpPr/><p:grpSpPr/><p:pic marker="inner"/></p:grpSp></p:grpSp></p:spTree></p:cSld></p:sld>"#
    );
    let parsed = CT_Slide::from_xml(xml.as_bytes()).unwrap();
    let ShapeTreeChild::GroupShape(outer) = &parsed.common_slide_data.shape_tree.children[0] else {
        panic!("expected outer group");
    };
    assert_eq!(
        outer.group_transform().unwrap().child_offset.unwrap().x.0,
        10
    );
    assert!(matches!(outer.children[0], ShapeTreeChild::Shape(_)));
    let ShapeTreeChild::GroupShape(inner) = &outer.children[1] else {
        panic!("expected inner group");
    };
    assert!(matches!(inner.children[0], ShapeTreeChild::Picture(_)));
    let written = parsed.to_xml().unwrap();
    let reparsed = CT_Slide::from_xml(&written).unwrap();
    assert_eq!(parsed, reparsed);
    assert_eq!(
        count_groups(&reparsed.common_slide_data.shape_tree.children),
        2
    );
}

#[test]
fn every_corpus_shape_tree_round_trips_structurally() {
    let Some(corpus) = require_or_skip_corpus() else {
        return;
    };
    verify_fetched_corpus();
    let mut trees = 0usize;
    let mut groups = 0usize;
    for entry in manifest_entries() {
        let package = OpcPackage::open(corpus.join(entry.path)).unwrap();
        for (part, content_type) in &package.content_types.overrides {
            let Some(xml) = package.get_part(part) else {
                continue;
            };
            match content_type.as_str() {
                content_types::SLIDE => {
                    let parsed = CT_Slide::from_xml(xml)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path));
                    groups += count_groups(&parsed.common_slide_data.shape_tree.children);
                    assert_eq!(
                        parsed,
                        CT_Slide::from_xml(&parsed.to_xml().unwrap()).unwrap()
                    );
                    trees += 1;
                }
                content_types::SLIDE_LAYOUT => {
                    let parsed = CT_SlideLayout::from_xml(xml)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path));
                    groups += count_groups(&parsed.common_slide_data.shape_tree.children);
                    assert_eq!(
                        parsed,
                        CT_SlideLayout::from_xml(&parsed.to_xml().unwrap()).unwrap()
                    );
                    trees += 1;
                }
                content_types::SLIDE_MASTER => {
                    let parsed = CT_SlideMaster::from_xml(xml)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path));
                    groups += count_groups(&parsed.common_slide_data.shape_tree.children);
                    assert_eq!(
                        parsed,
                        CT_SlideMaster::from_xml(&parsed.to_xml().unwrap()).unwrap()
                    );
                    trees += 1;
                }
                _ => {}
            }
        }
    }
    assert!(trees > 0);
    assert!(groups > 0);
    eprintln!("Shape-tree corpus gate checked {trees} trees and {groups} recursive groups");
}

fn count_groups(children: &[ShapeTreeChild]) -> usize {
    children
        .iter()
        .map(|child| match child {
            ShapeTreeChild::GroupShape(group) => 1 + count_groups(&group.children),
            _ => 0,
        })
        .sum()
}
