use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_core::units::Emu;
use oxml_drawing::color::{ColorMapSlot, ThemeColorSlot};
use oxml_drawing::shape_props::CT_ShapeProperties;
use oxml_drawing::table::CT_Table;
use oxml_drawing::text::CT_TextBody;
use oxml_drawing::xfrm::CT_Transform2D;
use oxml_opc::relationship::rel_types;
use oxml_opc::{OpcPackage, content_types};
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer, XmlVersion};
use rpptx_oxml::PRESENTATION_PART;
use rpptx_oxml::connector::CT_ConnectionShape;
use rpptx_oxml::graphic_frame::{CT_GraphicFrame, GraphicDataPayload};
use rpptx_oxml::namespace::{MC_NS, P_NS, P_PREFIX, R_NS, R_PREFIX};
use rpptx_oxml::notes_parts::{CT_NotesMaster, CT_NotesSlide};
use rpptx_oxml::picture::CT_Picture;
use rpptx_oxml::placeholder::{CT_Placeholder, PhType, PlaceholderKey};
use rpptx_oxml::presentation::{CT_Presentation, CT_SlideId};
use rpptx_oxml::relmap::{rewrite_exact_rel_ids, rewrite_rel_ids};
use rpptx_oxml::shape_tree::{
    CT_Shape, CT_ShapeTree, ShapeIdAllocator, ShapeTreeChild, rewrite_shape_ids,
};
use rpptx_oxml::slide_parts::{
    BackgroundRendering, CT_Slide, CT_SlideLayout, CT_SlideMaster, ColorMapOverrideKind,
};

const MANIFEST: &str = include_str!("../../../scripts/pptx-corpus-manifest.tsv");
const EXPECTED_DECKS: usize = 50;
type DrawingElements = (Vec<Vec<u8>>, Vec<Vec<u8>>);

#[test]
fn slide_size_mutation_preserves_kind_and_unmodelled_xml() {
    let xml = format!(
        r#"<q:presentation xmlns:q="{P_NS}" xmlns:x="urn:f115"><x:before/><q:sldSz cy="6858000" type="screen16x9" x:keep="yes" cx="12192000"><x:size> A &amp; B </x:size></q:sldSz><x:between/><q:notesSz cx="6858000" cy="9144000"/></q:presentation>"#
    );
    let mut presentation = CT_Presentation::from_xml(xml.as_bytes()).unwrap();

    presentation
        .set_slide_size(Emu(10_000_001), Emu(5_000_003))
        .unwrap();

    let written = presentation.to_xml().unwrap();
    let text = String::from_utf8(written.clone()).unwrap();
    assert!(
        text.contains(r#"<p:sldSz cx="10000001" cy="5000003" type="screen16x9" x:keep="yes">"#)
    );
    assert!(text.contains("<x:size> A &amp; B </x:size>"));
    assert_order(
        &written,
        &["<x:before", "<p:sldSz", "<x:between", "<p:notesSz"],
    );
    let reparsed = CT_Presentation::from_xml(&written).unwrap();
    let size = reparsed.slide_size.unwrap();
    assert_eq!((size.cx, size.cy), (Emu(10_000_001), Emu(5_000_003)));
    assert_eq!(size.kind.as_deref(), Some("screen16x9"));
}

#[test]
fn hidden_flag_uses_inverse_show_semantics() {
    let slide_xml = |show: &str| {
        format!(
            r#"<q:sld xmlns:q="{P_NS}" xmlns:x="urn:f115"{show} x:keep="root"><q:cSld><q:spTree><q:nvGrpSpPr/><q:grpSpPr/></q:spTree></q:cSld><x:tail/></q:sld>"#
        )
    };

    let missing = CT_Slide::from_xml(slide_xml("").as_bytes()).unwrap();
    let shown = CT_Slide::from_xml(slide_xml(r#" show="true""#).as_bytes()).unwrap();
    let hidden = CT_Slide::from_xml(slide_xml(r#" show="false""#).as_bytes()).unwrap();
    assert!(!missing.hidden());
    assert!(!shown.hidden());
    assert!(hidden.hidden());

    let mut slide = missing;
    slide.set_hidden(true);
    let hidden_xml = String::from_utf8(slide.to_xml().unwrap()).unwrap();
    assert!(hidden_xml.contains(r#" show="0""#));
    assert!(hidden_xml.contains(r#"x:keep="root""#));
    assert!(hidden_xml.contains("<x:tail/>"));
    slide.set_hidden(false);
    let shown_xml = String::from_utf8(slide.to_xml().unwrap()).unwrap();
    assert!(shown_xml.contains(r#" show="1""#));
}

#[test]
fn background_set_and_clear_preserve_theme_references_and_raw_xml() {
    let reference_xml = format!(
        r#"<q:sld xmlns:q="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:f115"><q:cSld><q:bg x:keep="theme"><q:bgRef idx="1001"><a:schemeClr val="bg1"/></q:bgRef><x:bgTail/></q:bg><q:spTree><q:nvGrpSpPr/><q:grpSpPr/></q:spTree></q:cSld></q:sld>"#
    );
    let mut reference = CT_Slide::from_xml(reference_xml.as_bytes()).unwrap();
    let reference_before = reference
        .common_slide_data
        .background
        .as_ref()
        .unwrap()
        .raw_xml()
        .to_vec();
    let fill = oxml_drawing::fill::Fill::from_xml(
        br#"<z:solidFill xmlns:z="http://schemas.openxmlformats.org/drawingml/2006/main"><z:srgbClr val="102030"/></z:solidFill>"#,
    )
    .unwrap();
    assert!(reference.set_background(fill.clone()).is_err());
    reference.clear_background();
    assert_eq!(
        reference.common_slide_data.background.unwrap().raw_xml(),
        reference_before
    );

    let direct_xml = format!(
        r#"<q:sld xmlns:q="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:f115"><q:cSld><x:before/><q:bg x:keep="container"><!--before-properties--><q:bgPr shadeToTitle="1" x:keep="properties"><x:before-fill/><!--before-fill--><a:solidFill><a:srgbClr val="ABCDEF"/></a:solidFill><!--after-fill--><a:effectLst><a:outerShdw blurRad="40000"><a:srgbClr val="010203"/></a:outerShdw></a:effectLst><x:after-fill/></q:bgPr><!--after-properties--><x:bg-extension/></q:bg><q:spTree><q:nvGrpSpPr/><q:grpSpPr/></q:spTree><x:after/></q:cSld></q:sld>"#
    );
    let mut direct = CT_Slide::from_xml(direct_xml.as_bytes()).unwrap();
    direct.set_background(fill).unwrap();
    let expected_background = br#"<q:bg x:keep="container"><!--before-properties--><q:bgPr shadeToTitle="1" x:keep="properties"><x:before-fill/><!--before-fill--><a:solidFill><a:srgbClr val="102030"/></a:solidFill><!--after-fill--><a:effectLst><a:outerShdw blurRad="40000"><a:srgbClr val="010203"/></a:outerShdw></a:effectLst><x:after-fill/></q:bgPr><!--after-properties--><x:bg-extension/></q:bg>"#;
    assert_eq!(
        direct
            .common_slide_data
            .background
            .as_ref()
            .unwrap()
            .raw_xml(),
        expected_background
    );
    let written = direct.to_xml().unwrap();
    assert!(
        written
            .windows(expected_background.len())
            .any(|window| window == expected_background)
    );
    assert_order(
        &written,
        &["<x:before/>", "<q:bg", "<p:spTree", "<x:after/>"],
    );
    direct.clear_background();
    let cleared = String::from_utf8(direct.to_xml().unwrap()).unwrap();
    assert!(!cleared.contains("<p:bg"));
    assert!(cleared.contains("<x:before/>"));
    assert!(cleared.contains("<x:after/>"));
}

#[test]
fn fresh_shape_ids_preserve_other_bytes_and_rewrite_connector_endpoints() {
    let xml = format!(
        r#"<p:sld xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:mc="{MC_NS}"><p:cSld producer="kept"><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name="root"/></p:nvGrpSpPr><p:grpSpPr/><p:sp><p:nvSpPr><p:cNvPr id="2" name="one"><x:raw xmlns:x="urn:f114"> A &amp; B </x:raw></p:cNvPr></p:nvSpPr><p:spPr/></p:sp><p:cxnSp><p:nvCxnSpPr><p:cNvPr id="7" name="link"/><p:cNvCxnSpPr><a:stCxn id="2" idx="0"/><a:endCxn id="9" idx="1"/></p:cNvCxnSpPr></p:nvCxnSpPr><p:spPr/></p:cxnSp><p:sp><p:nvSpPr><p:cNvPr id="9" name="two"/></p:nvSpPr><p:spPr/></p:sp><mc:AlternateContent><mc:Choice Requires="p14"><p:sp><p:nvSpPr><p:cNvPr id="20" name="choice"/></p:nvSpPr></p:sp><p:cxnSp><p:nvCxnSpPr><p:cNvPr id="21" name="choice link"/><p:cNvCxnSpPr><a:stCxn id="20" idx="0"/><a:endCxn id="20" idx="1"/></p:cNvCxnSpPr></p:nvCxnSpPr></p:cxnSp></mc:Choice><mc:Fallback><p:sp><p:nvSpPr><p:cNvPr id="22" name="fallback"/></p:nvSpPr></p:sp><p:cxnSp><p:nvCxnSpPr><p:cNvPr id="23" name="fallback link"/><p:cNvCxnSpPr><a:stCxn id="22" idx="0"/><a:endCxn id="22" idx="1"/></p:cNvCxnSpPr></p:nvCxnSpPr></p:cxnSp></mc:Fallback></mc:AlternateContent></p:spTree></p:cSld></p:sld>"#
    );

    let rewritten = String::from_utf8(rewrite_shape_ids(xml.as_bytes()).unwrap()).unwrap();

    assert!(rewritten.contains(r#"<p:cNvPr id="1" name="root"/>"#));
    assert!(rewritten.contains(
        r#"<p:cNvPr id="3" name="one"><x:raw xmlns:x="urn:f114"> A &amp; B </x:raw></p:cNvPr>"#
    ));
    assert!(rewritten.contains(r#"<p:cNvPr id="4" name="link"/>"#));
    assert!(rewritten.contains(r#"<p:cNvPr id="5" name="two"/>"#));
    assert!(rewritten.contains(r#"<a:stCxn id="3" idx="0"/><a:endCxn id="5" idx="1"/>"#));
    assert!(rewritten.contains(r#"<p:cNvPr id="6" name="choice"/>"#));
    assert!(rewritten.contains(r#"<p:cNvPr id="8" name="choice link"/>"#));
    assert!(rewritten.contains(r#"<a:stCxn id="6" idx="0"/><a:endCxn id="6" idx="1"/>"#));
    assert!(rewritten.contains(r#"<p:cNvPr id="10" name="fallback"/>"#));
    assert!(rewritten.contains(r#"<p:cNvPr id="11" name="fallback link"/>"#));
    assert!(rewritten.contains(r#"<a:stCxn id="10" idx="0"/><a:endCxn id="10" idx="1"/>"#));
    assert!(rewritten.contains(r#"<p:cSld producer="kept">"#));
}

#[test]
fn slide_id_list_raw_children_follow_surviving_ids_after_collection_edits() {
    let xml = format!(
        r#"<p:presentation xmlns:p="{P_NS}" xmlns:r="{R_NS}" xmlns:x="urn:f114"><p:sldIdLst><x:before/><p:sldId id="256" r:id="rId1"/><x:between-one-two/><p:sldId id="257" r:id="rId2"/><x:between-two-three/><p:sldId id="258" r:id="rId3"/><x:after/></p:sldIdLst><p:notesSz cx="1" cy="1"/></p:presentation>"#
    );
    let mut presentation = CT_Presentation::from_xml(xml.as_bytes()).unwrap();
    let third = presentation.slide_ids.remove(2);
    presentation.slide_ids.insert(0, third);
    presentation.slide_ids.remove(1);
    presentation
        .slide_ids
        .insert(1, CT_SlideId::new(300, "rId4").unwrap());

    let written = String::from_utf8(presentation.to_xml().unwrap()).unwrap();

    let third = written.find(r#"r:id="rId3""#).unwrap();
    let inserted = written.find(r#"r:id="rId4""#).unwrap();
    let between = written.find("<x:between-one-two/>").unwrap();
    let second = written.find(r#"r:id="rId2""#).unwrap();
    assert!(third < inserted && inserted < between && between < second);
    assert!(written.contains("<x:before/>"));
    assert!(written.contains("<x:between-two-three/>"));
    assert!(written.contains("<x:after/>"));
    assert!(!written.contains(r#"r:id="rId1""#));
}

#[test]
fn custom_show_removal_preserves_unrelated_presentation_xml() {
    let xml = format!(
        r#"<p:presentation xmlns:p="{P_NS}" xmlns:r="{R_NS}" xmlns:q="urn:f114" q:producer="kept"><p:sldIdLst><p:sldId id="256" r:id="gone"/><p:sldId id="257" r:id="kept"/></p:sldIdLst><p:custShowLst q:raw=" A &amp; B "><p:custShow name="one" id="1"><p:sldLst><p:sld r:id="gone"/><q:marker/><p:sld r:id="kept"/></p:sldLst></p:custShow><p:custShow name="two" id="2"><p:sldLst><p:sld r:id="gone"></p:sld></p:sldLst></p:custShow></p:custShowLst><p:notesSz cx="1" cy="1"/></p:presentation>"#
    );
    let mut presentation = CT_Presentation::from_xml(xml.as_bytes()).unwrap();

    presentation.remove_custom_show_slide("gone").unwrap();
    let written = String::from_utf8(presentation.to_xml().unwrap()).unwrap();

    assert_eq!(written.matches(r#"r:id="gone""#).count(), 1);
    assert_eq!(written.matches(r#"r:id="kept""#).count(), 2);
    assert!(written.contains(r#"<p:custShowLst q:raw=" A &amp; B ">"#));
    assert!(written.contains("<q:marker/>"));
}

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
fn rpptx_oxml_is_an_explicit_publication_candidate() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("name = \"rpptx-oxml\""));
    assert!(manifest.contains("version = \"0.3.0\""));
    assert!(manifest.contains("publish = true"));
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
fn every_corpus_drawingml_table_round_trips_structurally() {
    let Some(corpus) = require_or_skip_corpus() else {
        return;
    };
    verify_fetched_corpus();
    let mut coverage = TableCoverage::default();
    for entry in manifest_entries() {
        let package = OpcPackage::open(corpus.join(entry.path)).unwrap();
        for (part, xml) in &package.parts {
            if !part.ends_with(".xml") {
                continue;
            }
            for extracted in drawingml_tables(xml)
                .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path))
            {
                let table = CT_Table::from_xml_with_inherited_namespaces(
                    &extracted.xml,
                    &extracted.inherited_namespaces,
                )
                .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path));
                let written = table.to_xml().unwrap();
                assert_eq!(
                    table,
                    CT_Table::from_xml(&written)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path))
                );
                coverage.add(&table);
            }
        }
    }
    assert_eq!(coverage.tables, 26);
    assert_eq!(coverage.rows, 152);
    assert_eq!(coverage.cells, 724);
    assert_eq!(coverage.merge_origins, 21);
    assert_eq!(coverage.horizontal_continuations, 126);
    assert_eq!(coverage.vertical_continuations, 9);
    eprintln!("DrawingML table corpus gate checked {coverage:?}");
}

#[derive(Debug, Default)]
struct TableCoverage {
    tables: usize,
    rows: usize,
    cells: usize,
    merge_origins: usize,
    horizontal_continuations: usize,
    vertical_continuations: usize,
}

impl TableCoverage {
    fn add(&mut self, table: &CT_Table) {
        self.tables += 1;
        self.rows += table.rows.len();
        for row in &table.rows {
            self.cells += row.cells.len();
            for cell in &row.cells {
                self.merge_origins += usize::from(cell.row_span > 1 || cell.grid_span > 1);
                self.horizontal_continuations += usize::from(cell.horizontal_merge);
                self.vertical_continuations += usize::from(cell.vertical_merge);
            }
        }
    }
}

struct ExtractedTable {
    xml: Vec<u8>,
    inherited_namespaces: Vec<(String, String)>,
}

fn drawingml_tables(xml: &[u8]) -> Result<Vec<ExtractedTable>, String> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut tables = Vec::new();
    let mut namespace_frames = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element)) => {
                let local_declarations = namespace_declarations(&element)?;
                if local_name(element.name().as_ref()) == b"tbl"
                    && element_namespace(
                        element.name().as_ref(),
                        &namespace_frames,
                        &local_declarations,
                    ) == Some(oxml_drawing::namespace::A_NS)
                {
                    tables.push(ExtractedTable {
                        xml: capture_element(&mut reader, &element)
                            .map_err(|error| format!("table scan failed: {error}"))?,
                        inherited_namespaces: effective_namespaces(&namespace_frames),
                    });
                } else {
                    namespace_frames.push(local_declarations);
                }
            }
            Ok(Event::Empty(element)) => {
                let local_declarations = namespace_declarations(&element)?;
                if local_name(element.name().as_ref()) == b"tbl"
                    && element_namespace(
                        element.name().as_ref(),
                        &namespace_frames,
                        &local_declarations,
                    ) == Some(oxml_drawing::namespace::A_NS)
                {
                    tables.push(ExtractedTable {
                        xml: capture_empty_element(&element)
                            .map_err(|error| format!("table scan failed: {error}"))?,
                        inherited_namespaces: effective_namespaces(&namespace_frames),
                    });
                }
            }
            Ok(Event::End(_)) => {
                namespace_frames
                    .pop()
                    .ok_or_else(|| "namespace stack underflow".to_owned())?;
            }
            Ok(Event::Eof) => return Ok(tables),
            Ok(_) => {}
            Err(error) => return Err(format!("XML scan failed: {error}")),
        }
        buffer.clear();
    }
}

fn namespace_declarations(element: &BytesStart<'_>) -> Result<Vec<(String, String)>, String> {
    let mut declarations = Vec::new();
    for attribute in element.attributes() {
        let attribute = attribute.map_err(|error| format!("namespace scan failed: {error}"))?;
        let key = attribute.key.as_ref();
        let prefix = if key == b"xmlns" {
            Some("")
        } else {
            key.strip_prefix(b"xmlns:")
                .map(|value| std::str::from_utf8(value))
                .transpose()
                .map_err(|error| format!("namespace prefix is not UTF-8: {error}"))?
        };
        if let Some(prefix) = prefix {
            let uri = attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())
                .map_err(|error| format!("namespace URI decode failed: {error}"))?
                .into_owned();
            declarations.push((prefix.to_owned(), uri));
        }
    }
    Ok(declarations)
}

fn element_namespace<'a>(
    name: &[u8],
    frames: &'a [Vec<(String, String)>],
    local_declarations: &'a [(String, String)],
) -> Option<&'a str> {
    let prefix = name
        .iter()
        .position(|byte| *byte == b':')
        .map_or(&[][..], |position| &name[..position]);
    let prefix = std::str::from_utf8(prefix).ok()?;
    local_declarations
        .iter()
        .rev()
        .chain(frames.iter().rev().flat_map(|frame| frame.iter().rev()))
        .find(|(candidate, _)| candidate == prefix)
        .map(|(_, uri)| uri.as_str())
}

fn effective_namespaces(frames: &[Vec<(String, String)>]) -> Vec<(String, String)> {
    let mut effective = Vec::new();
    for (prefix, uri) in frames.iter().flatten() {
        if let Some((_, existing_uri)) = effective
            .iter_mut()
            .find(|(existing_prefix, _)| existing_prefix == prefix)
        {
            *existing_uri = uri.clone();
        } else {
            effective.push((prefix.clone(), uri.clone()));
        }
    }
    effective
}

#[test]
fn drawingml_table_extraction_carries_ancestor_namespaces() {
    let xml = format!(
        r#"<p:sld xmlns:p="{P_NS}" xmlns:d="{}" xmlns:x="urn:producer" xmlns:a="urn:ancestor-producer"><p:cSld><d:tbl xmlns:a="{}"><d:tblGrid><d:gridCol w="100"/></d:tblGrid><d:tr h="200"><d:tc><x:extension/></d:tc></d:tr></d:tbl><x:tbl/></p:cSld></p:sld>"#,
        oxml_drawing::namespace::A_NS,
        oxml_drawing::namespace::A_NS
    );
    let extracted = drawingml_tables(xml.as_bytes()).unwrap();
    assert_eq!(extracted.len(), 1);
    let table = CT_Table::from_xml_with_inherited_namespaces(
        &extracted[0].xml,
        &extracted[0].inherited_namespaces,
    )
    .unwrap();
    let written = table.to_xml().unwrap();
    let text = std::str::from_utf8(&written).unwrap();
    assert!(text.contains(r#"xmlns:x="urn:producer""#));
    assert!(text.contains("<x:extension/>"));
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
fn slide_ids_preserve_order_and_defer_semantic_validation() {
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

    for accepted_for_facade_validation in [
        r#"<p:sldId id="255" r:id="rId1"/>"#,
        r#"<p:sldId id="2147483648" r:id="rId1"/>"#,
    ] {
        let xml = format!(
            r#"<p:presentation xmlns:p="{P_NS}" xmlns:r="{R_NS}"><p:sldIdLst>{accepted_for_facade_validation}</p:sldIdLst><p:notesSz cx="6858000" cy="9144000"/></p:presentation>"#
        );
        assert!(CT_Presentation::from_xml(xml.as_bytes()).is_ok());
    }

    let invalid = format!(
        r#"<p:presentation xmlns:p="{P_NS}" xmlns:r="{R_NS}"><p:sldIdLst><p:sldId id="not-an-integer" r:id="rId1"/></p:sldIdLst><p:notesSz cx="6858000" cy="9144000"/></p:presentation>"#
    );
    assert!(CT_Presentation::from_xml(invalid.as_bytes()).is_err());

    let duplicate = format!(
        r#"<p:presentation xmlns:p="{P_NS}" xmlns:r="{R_NS}"><p:sldIdLst><p:sldId id="256" r:id="rId1"/><p:sldId id="256" r:id="rId2"/></p:sldIdLst><p:notesSz cx="6858000" cy="9144000"/></p:presentation>"#
    );
    assert!(CT_Presentation::from_xml(duplicate.as_bytes()).is_ok());

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
fn background_projection_preserves_the_source_subtree_verbatim() {
    let properties = format!(
        r#"<q:sld xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer"><q:cSld><q:bg x:keep="background"><q:bgPr shadeToTitle="1"><x:before/><d:gradFill rotWithShape="1"><d:gsLst><d:gs pos="0"><d:srgbClr val="102030"/></d:gs><d:gs pos="100000"><d:srgbClr val="D0E0F0"/></d:gs></d:gsLst><d:lin ang="0"/></d:gradFill><d:effectLst><x:effect/></d:effectLst><x:after/></q:bgPr></q:bg><q:spTree><q:nvGrpSpPr/><q:grpSpPr/></q:spTree></q:cSld></q:sld>"#
    );
    let expected = r#"<q:bg x:keep="background"><q:bgPr shadeToTitle="1"><x:before/><d:gradFill rotWithShape="1"><d:gsLst><d:gs pos="0"><d:srgbClr val="102030"/></d:gs><d:gs pos="100000"><d:srgbClr val="D0E0F0"/></d:gs></d:gsLst><d:lin ang="0"/></d:gradFill><d:effectLst><x:effect/></d:effectLst><x:after/></q:bgPr></q:bg>"#;
    let parsed = CT_Slide::from_xml(properties.as_bytes()).unwrap();
    let background = parsed.common_slide_data.background.as_ref().unwrap();
    assert_eq!(background.raw_xml(), expected.as_bytes());
    assert!(matches!(
        background.rendering(),
        BackgroundRendering::Properties(Some(fill))
            if matches!(fill.as_ref(), oxml_drawing::fill::Fill::Gradient(_))
    ));
    let written = parsed.to_xml().unwrap();
    assert!(
        written
            .windows(expected.len())
            .any(|window| window == expected.as_bytes())
    );
    assert_order(&written, &["<q:bg ", "<p:spTree"]);
    assert_eq!(parsed, CT_Slide::from_xml(&written).unwrap());

    let reference = format!(
        r#"<q:sld xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer"><q:cSld><q:bg><q:bgRef idx="1001"><x:before/><d:schemeClr val="accent2"><d:tint val="25000"/></d:schemeClr><x:after/></q:bgRef></q:bg><q:spTree><q:nvGrpSpPr/><q:grpSpPr/></q:spTree></q:cSld></q:sld>"#
    );
    let expected = r#"<q:bg><q:bgRef idx="1001"><x:before/><d:schemeClr val="accent2"><d:tint val="25000"/></d:schemeClr><x:after/></q:bgRef></q:bg>"#;
    let parsed = CT_Slide::from_xml(reference.as_bytes()).unwrap();
    let background = parsed.common_slide_data.background.as_ref().unwrap();
    assert_eq!(background.raw_xml(), expected.as_bytes());
    assert!(matches!(
        background.rendering(),
        BackgroundRendering::Reference {
            index: 1001,
            color: Some(_)
        }
    ));
    let written = parsed.to_xml().unwrap();
    assert!(
        written
            .windows(expected.len())
            .any(|window| window == expected.as_bytes())
    );
    assert_eq!(parsed, CT_Slide::from_xml(&written).unwrap());

    let wrong_namespace = format!(
        r#"<p:sld xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer"><p:cSld><p:bg><p:bgPr><x:gradFill/></p:bgPr></p:bg><p:spTree><p:nvGrpSpPr/><p:grpSpPr/></p:spTree></p:cSld></p:sld>"#
    );
    let parsed = CT_Slide::from_xml(wrong_namespace.as_bytes()).unwrap();
    assert!(matches!(
        parsed
            .common_slide_data
            .background
            .as_ref()
            .unwrap()
            .rendering(),
        BackgroundRendering::Properties(None)
    ));
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

    let default_text_descendant = format!(
        r#"<q:presentation xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main"><q:notesSz cx="6858000" cy="9144000"/><q:defaultTextStyle><d:lvl1pPr><d:defRPr xmlns:a="urn:producer"><a:raw/></d:defRPr></d:lvl1pPr></q:defaultTextStyle></q:presentation>"#
    );
    assert!(CT_Presentation::from_xml(default_text_descendant.as_bytes()).is_err());

    let master_text_level = format!(
        r#"<q:sldMaster xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main"><q:cSld><q:spTree><q:nvGrpSpPr/><q:grpSpPr/></q:spTree></q:cSld><q:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/><q:txStyles><q:titleStyle><d:lvl1pPr xmlns:a="urn:producer"><a:raw/></d:lvl1pPr></q:titleStyle><q:bodyStyle/><q:otherStyle/></q:txStyles></q:sldMaster>"#
    );
    assert!(CT_SlideMaster::from_xml(master_text_level.as_bytes()).is_err());

    let master_text_descendant = format!(
        r#"<q:sldMaster xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main"><q:cSld><q:spTree><q:nvGrpSpPr/><q:grpSpPr/></q:spTree></q:cSld><q:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/><q:txStyles><q:titleStyle><d:lvl1pPr><d:defRPr xmlns:a="urn:producer"><a:raw/></d:defRPr></d:lvl1pPr></q:titleStyle><q:bodyStyle/><q:otherStyle/></q:txStyles></q:sldMaster>"#
    );
    assert!(CT_SlideMaster::from_xml(master_text_descendant.as_bytes()).is_err());

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
        r#"<q:spTree xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main"><q:nvGrpSpPr marker="kept"/><q:grpSpPr><d:xfrm><d:off x="10" y="20"/><d:ext cx="30" cy="40"/><d:chOff x="1" y="2"/><d:chExt cx="3" cy="4"/></d:xfrm><d:solidFill><d:srgbClr val="112233"/></d:solidFill></q:grpSpPr><q:sp><q:nvSpPr><q:cNvPr/><q:cNvSpPr/><q:nvPr/></q:nvSpPr><q:spPr/></q:sp></q:spTree>"#
    );
    let parsed = CT_ShapeTree::from_xml(valid.as_bytes()).unwrap();
    let transform = parsed.group_transform().unwrap();
    assert_eq!(transform.offset.unwrap().x.0, 10);
    assert_eq!(transform.child_extent.unwrap().cy.0, 4);
    let written = parsed.to_xml().unwrap();
    assert_order(&written, &["<p:nvGrpSpPr", "<p:grpSpPr", "<p:sp xmlns"]);
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

    let tree_xml = format!(
        r#"<p:spTree xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer"><p:nvGrpSpPr/><p:grpSpPr/><p:graphicFrame><p:nvGraphicFramePr/><p:xfrm/><a:graphic><a:graphicData uri="urn:producer"><x:data name="mc"/></a:graphicData></a:graphic></p:graphicFrame></p:spTree>"#
    );
    let tree = CT_ShapeTree::from_xml(tree_xml.as_bytes()).unwrap();
    assert_eq!(
        tree,
        CT_ShapeTree::from_xml(&tree.to_xml().unwrap()).unwrap()
    );
}

#[test]
fn all_six_child_variants_keep_document_order() {
    let payloads = [
        r#"<p:sp marker="shape"><p:nvSpPr><p:cNvPr/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/></p:sp>"#,
        r#"<p:pic marker="picture"><p:nvPicPr><p:cNvPr/><p:cNvPicPr/><p:nvPr/></p:nvPicPr><p:blipFill/><p:spPr/></p:pic>"#,
        r#"<p:graphicFrame marker="frame"><p:nvGraphicFramePr/><p:xfrm/><a:graphic><a:graphicData uri="urn:frame"><x:data/></a:graphicData></a:graphic></p:graphicFrame>"#,
        r#"<p:cxnSp marker="connector"><p:nvCxnSpPr><p:cNvPr id="2" name="Connector"/><p:cNvCxnSpPr/><p:nvPr/></p:nvCxnSpPr><p:spPr/></p:cxnSp>"#,
        r#"<mc:AlternateContent marker="alternate"><mc:Choice Requires="p14"><p:sp/></mc:Choice></mc:AlternateContent>"#,
    ];
    let xml = format!(
        r#"<p:spTree xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:x="urn:producer"><p:nvGrpSpPr/><p:grpSpPr/>{}<x:between/>{}<p:grpSp><p:nvGrpSpPr/><p:grpSpPr/><p:sp marker="nested"><p:nvSpPr><p:cNvPr/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/></p:sp></p:grpSp>{}{}{}</p:spTree>"#,
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
    assert!(std::str::from_utf8(&written).unwrap().contains("<x:data/>"));
    let alternate_content = payloads[4].as_bytes();
    assert!(
        written
            .windows(alternate_content.len())
            .any(|window| window == alternate_content)
    );
    assert_eq!(parsed, CT_ShapeTree::from_xml(&written).unwrap());
}

#[test]
fn alternate_content_selects_fallback_without_parsing_choices() {
    let raw = r#"<mc:AlternateContent marker="selected"><mc:Choice Requires="p14"><p:sp/></mc:Choice><mc:Fallback><p:sp marker="fallback-shape"><p:nvSpPr><p:cNvPr/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/></p:sp><x:ignored/><p:pic marker="fallback-picture"><p:nvPicPr><p:cNvPr/><p:cNvPicPr/><p:nvPr/></p:nvPicPr><p:blipFill/><p:spPr/></p:pic><p:cxnSp marker="fallback-connector"><p:nvCxnSpPr><p:cNvPr/><p:cNvCxnSpPr/><p:nvPr/></p:nvCxnSpPr><p:spPr/></p:cxnSp></mc:Fallback></mc:AlternateContent>"#;
    let tree = alternate_content_tree(raw.as_bytes(), &[]).unwrap();
    let ShapeTreeChild::AlternateContent(alternate) = &tree.children[0] else {
        panic!("expected typed AlternateContent");
    };
    let selected = alternate.selected_fallback().unwrap();
    assert_eq!(selected.len(), 3);
    assert!(matches!(selected[0], ShapeTreeChild::Shape(_)));
    assert!(matches!(selected[1], ShapeTreeChild::Picture(_)));
    assert!(matches!(selected[2], ShapeTreeChild::Connector(_)));
}

#[test]
fn alternate_content_without_fallback_preserves_choices_and_selects_none() {
    let raw = r#"<mc:AlternateContent marker="choice-only"><mc:Choice Requires="p14"><p:sp/></mc:Choice></mc:AlternateContent>"#;
    let tree = alternate_content_tree(raw.as_bytes(), &[]).unwrap();
    let ShapeTreeChild::AlternateContent(alternate) = &tree.children[0] else {
        panic!("expected typed AlternateContent");
    };
    assert_eq!(alternate.raw_xml(), raw.as_bytes());
    assert_eq!(alternate.selected_fallback(), None);
    assert!(
        tree.to_xml()
            .unwrap()
            .windows(raw.len())
            .any(|part| part == raw.as_bytes())
    );
}

#[test]
fn alternate_content_resolves_branch_namespaces_and_rejects_duplicate_fallbacks() {
    let raw = r#"<m:AlternateContent><m:Choice Requires="q"><m:Fallback><q:sp/></m:Fallback></m:Choice><x:Fallback><q:sp/></x:Fallback><m:Fallback><q:sp marker="alias"><q:nvSpPr><q:cNvPr/><q:cNvSpPr/><q:nvPr/></q:nvSpPr><q:spPr/></q:sp></m:Fallback></m:AlternateContent>"#;
    let inherited = [
        (
            "m",
            "http://schemas.openxmlformats.org/markup-compatibility/2006",
        ),
        ("q", P_NS),
        ("x", "urn:not-mc"),
    ];
    let tree = alternate_content_tree(raw.as_bytes(), &inherited).unwrap();
    let ShapeTreeChild::AlternateContent(alternate) = &tree.children[0] else {
        panic!("expected typed AlternateContent");
    };
    let selected = alternate.selected_fallback().unwrap();
    assert_eq!(selected.len(), 1);
    assert!(matches!(selected[0], ShapeTreeChild::Shape(_)));

    let empty = r#"<m:AlternateContent><m:Fallback/></m:AlternateContent>"#;
    let empty_tree = alternate_content_tree(empty.as_bytes(), &inherited).unwrap();
    let ShapeTreeChild::AlternateContent(empty_alternate) = &empty_tree.children[0] else {
        panic!("expected empty typed AlternateContent fallback");
    };
    assert_eq!(empty_alternate.selected_fallback(), Some(&[][..]));

    let duplicate = r#"<m:AlternateContent><m:Fallback/><m:Fallback/></m:AlternateContent>"#;
    assert!(alternate_content_tree(duplicate.as_bytes(), &inherited).is_err());
}

#[test]
fn alternate_content_fallback_selection_does_not_change_stored_alternatives() {
    let raw = r#"<mc:AlternateContent x:marker="a &amp; b">
  <?producer keep?>
  <!-- choice stays opaque -->
  <mc:Choice Requires="p14"><p:sp x:data="&quot;raw&quot;"/></mc:Choice>
  <mc:Fallback><x:before/><p:sp marker="selected"><p:nvSpPr><p:cNvPr/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/></p:sp><x:after/></mc:Fallback>
</mc:AlternateContent>"#;
    let tree = alternate_content_tree(raw.as_bytes(), &[]).unwrap();
    let ShapeTreeChild::AlternateContent(alternate) = &tree.children[0] else {
        panic!("expected typed AlternateContent");
    };
    assert_eq!(alternate.raw_xml(), raw.as_bytes());
    assert_eq!(alternate.selected_fallback().unwrap().len(), 1);
    let written = tree.to_xml().unwrap();
    assert!(
        written
            .windows(raw.len())
            .any(|part| part == raw.as_bytes())
    );
    let reparsed = CT_ShapeTree::from_xml(&written).unwrap();
    let ShapeTreeChild::AlternateContent(reparsed_alternate) = &reparsed.children[0] else {
        panic!("expected typed AlternateContent after round-trip");
    };
    assert_eq!(reparsed_alternate.raw_xml(), raw.as_bytes());
}

#[test]
fn alternate_content_fallback_keeps_shape_tree_order_inside_nested_groups() {
    let raw = r#"<mc:AlternateContent marker="alternate"><mc:Fallback><p:sp marker="first"><p:nvSpPr><p:cNvPr/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/></p:sp><p:grpSp marker="second"><p:nvGrpSpPr/><p:grpSpPr/><p:pic marker="nested-picture"><p:nvPicPr><p:cNvPr/><p:cNvPicPr/><p:nvPr/></p:nvPicPr><p:blipFill/><p:spPr/></p:pic></p:grpSp><p:cxnSp marker="third"><p:nvCxnSpPr><p:cNvPr/><p:cNvCxnSpPr/><p:nvPr/></p:nvCxnSpPr><p:spPr/></p:cxnSp></mc:Fallback></mc:AlternateContent>"#;
    let outer = format!(
        r#"<p:spTree xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"><p:nvGrpSpPr/><p:grpSpPr/><p:grpSp marker="outer"><p:nvGrpSpPr/><p:grpSpPr/>{raw}</p:grpSp><p:sp marker="after"><p:nvSpPr><p:cNvPr/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/></p:sp></p:spTree>"#
    );
    let tree = CT_ShapeTree::from_xml(outer.as_bytes()).unwrap();
    let ShapeTreeChild::GroupShape(group) = &tree.children[0] else {
        panic!("expected outer group");
    };
    let ShapeTreeChild::AlternateContent(alternate) = &group.children[0] else {
        panic!("expected AlternateContent inside group");
    };
    let selected = alternate.selected_fallback().unwrap();
    assert!(matches!(selected[0], ShapeTreeChild::Shape(_)));
    let ShapeTreeChild::GroupShape(selected_group) = &selected[1] else {
        panic!("expected selected nested group");
    };
    assert!(matches!(
        selected_group.children[0],
        ShapeTreeChild::Picture(_)
    ));
    assert!(matches!(selected[2], ShapeTreeChild::Connector(_)));
    assert!(matches!(tree.children[1], ShapeTreeChild::Shape(_)));
    let written = tree.to_xml().unwrap();
    assert_order(
        &written,
        &[
            "marker=\"outer\"",
            "marker=\"alternate\"",
            "marker=\"after\"",
        ],
    );
    assert_eq!(tree, CT_ShapeTree::from_xml(&written).unwrap());
}

#[test]
fn every_corpus_alternate_content_subtree_round_trips_byte_identically() {
    let Some(corpus) = require_or_skip_corpus() else {
        return;
    };
    verify_fetched_corpus();
    let mut coverage = 0usize;
    for entry in manifest_entries() {
        let package = OpcPackage::open(corpus.join(entry.path)).unwrap();
        for (part, xml) in &package.parts {
            if !part.ends_with(".xml") {
                continue;
            }
            for alternate in alternate_content_subtrees(xml)
                .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path))
            {
                let inherited: Vec<_> = alternate
                    .inherited_namespaces
                    .iter()
                    .map(|(prefix, uri)| (prefix.as_str(), uri.as_str()))
                    .collect();
                let tree = alternate_content_tree(&alternate.xml, &inherited)
                    .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path));
                let ShapeTreeChild::AlternateContent(model) = &tree.children[0] else {
                    panic!("{} {part}: expected typed AlternateContent", entry.path);
                };
                assert_eq!(model.raw_xml(), alternate.xml);
                let written = tree.to_xml().unwrap();
                assert!(
                    written
                        .windows(alternate.xml.len())
                        .any(|part| part == alternate.xml)
                );
                assert_eq!(tree, CT_ShapeTree::from_xml(&written).unwrap());
                coverage += 1;
            }
        }
    }
    assert!(
        coverage > 0,
        "the pinned corpus contained no MC AlternateContent"
    );
    eprintln!("AlternateContent corpus gate checked {coverage} subtrees");
}

fn alternate_content_tree(
    raw: &[u8],
    inherited: &[(&str, &str)],
) -> Result<CT_ShapeTree, oxml_core::OxmlError> {
    let mut attributes = vec![
        ("xmlns:p".to_owned(), P_NS.to_owned()),
        (
            "xmlns:a".to_owned(),
            oxml_drawing::namespace::A_NS.to_owned(),
        ),
        ("xmlns:r".to_owned(), R_NS.to_owned()),
        ("xmlns:mc".to_owned(), MC_NS.to_owned()),
    ];
    for (prefix, uri) in inherited {
        if matches!(*prefix, "p" | "a" | "r" | "mc") {
            continue;
        }
        let name = if prefix.is_empty() {
            "xmlns".to_owned()
        } else {
            format!("xmlns:{prefix}")
        };
        attributes.push((name, (*uri).to_owned()));
    }

    let mut writer = Writer::new(Vec::new());
    let mut root = BytesStart::new("p:spTree");
    for (name, value) in &attributes {
        root.push_attribute((name.as_str(), value.as_str()));
    }
    writer.write_event(Event::Start(root))?;
    writer.write_event(Event::Empty(BytesStart::new("p:nvGrpSpPr")))?;
    writer.write_event(Event::Empty(BytesStart::new("p:grpSpPr")))?;
    writer.get_mut().extend_from_slice(raw);
    writer.write_event(Event::End(BytesEnd::new("p:spTree")))?;
    CT_ShapeTree::from_xml(&writer.into_inner())
}

struct ExtractedAlternateContent {
    xml: Vec<u8>,
    inherited_namespaces: Vec<(String, String)>,
}

fn alternate_content_subtrees(xml: &[u8]) -> Result<Vec<ExtractedAlternateContent>, String> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut alternatives = Vec::new();
    let mut namespace_frames = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element)) => {
                let local_declarations = namespace_declarations(&element)?;
                if local_name(element.name().as_ref()) == b"AlternateContent"
                    && element_namespace(
                        element.name().as_ref(),
                        &namespace_frames,
                        &local_declarations,
                    ) == Some(MC_NS)
                {
                    alternatives.push(ExtractedAlternateContent {
                        xml: capture_element(&mut reader, &element)
                            .map_err(|error| format!("AlternateContent scan failed: {error}"))?,
                        inherited_namespaces: effective_namespaces(&namespace_frames),
                    });
                } else {
                    namespace_frames.push(local_declarations);
                }
            }
            Ok(Event::Empty(element)) => {
                let local_declarations = namespace_declarations(&element)?;
                if local_name(element.name().as_ref()) == b"AlternateContent"
                    && element_namespace(
                        element.name().as_ref(),
                        &namespace_frames,
                        &local_declarations,
                    ) == Some(MC_NS)
                {
                    alternatives.push(ExtractedAlternateContent {
                        xml: capture_empty_element(&element)
                            .map_err(|error| format!("AlternateContent scan failed: {error}"))?,
                        inherited_namespaces: effective_namespaces(&namespace_frames),
                    });
                }
            }
            Ok(Event::End(_)) => {
                namespace_frames
                    .pop()
                    .ok_or_else(|| "namespace stack underflow".to_owned())?;
            }
            Ok(Event::Eof) => return Ok(alternatives),
            Ok(_) => {}
            Err(error) => return Err(format!("XML scan failed: {error}")),
        }
        buffer.clear();
    }
}

#[test]
fn connector_with_start_and_end_connections_round_trips() {
    let xml = format!(
        r#"<p:cxnSp xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><p:nvCxnSpPr><p:cNvPr id="2" name="Connector 1"/><p:cNvCxnSpPr><a:stCxn id="3" idx="0"/><a:endCxn id="4" idx="1"/></p:cNvCxnSpPr><p:nvPr/></p:nvCxnSpPr><p:spPr/></p:cxnSp>"#
    );
    let parsed = CT_ConnectionShape::from_xml(xml.as_bytes()).unwrap();
    assert_eq!(parsed.start_connection.as_ref().unwrap().id, 3);
    assert_eq!(parsed.start_connection.as_ref().unwrap().idx, 0);
    assert_eq!(parsed.end_connection.as_ref().unwrap().id, 4);
    assert_eq!(parsed.end_connection.as_ref().unwrap().idx, 1);
    assert_eq!(
        parsed,
        CT_ConnectionShape::from_xml(&parsed.to_xml().unwrap()).unwrap()
    );
}

#[test]
fn connector_reads_any_prefix_and_writes_fixed_prefixes_in_schema_order() {
    let xml = format!(
        r#"<q:cxnSp xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main"><q:nvCxnSpPr><q:cNvPr id="2" name="Connector"/><q:cNvCxnSpPr><d:stCxn id="3" idx="2"/><d:endCxn id="4" idx="1"/></q:cNvCxnSpPr><q:nvPr/></q:nvCxnSpPr><q:spPr/></q:cxnSp>"#
    );
    let written = CT_ConnectionShape::from_xml(xml.as_bytes())
        .unwrap()
        .to_xml()
        .unwrap();
    assert_order(
        &written,
        &[
            "<p:nvCxnSpPr",
            "<p:cNvPr",
            "<p:cNvCxnSpPr",
            "<a:stCxn",
            "<a:endCxn",
            "<p:nvPr",
            "<p:spPr",
        ],
    );
}

#[test]
fn connector_requires_non_visual_and_shape_properties_in_schema_order() {
    let documents = [
        format!(r#"<p:cxnSp xmlns:p="{P_NS}"><p:spPr/></p:cxnSp>"#),
        format!(
            r#"<p:cxnSp xmlns:p="{P_NS}"><p:nvCxnSpPr><p:cNvPr/><p:cNvCxnSpPr/><p:nvPr/></p:nvCxnSpPr></p:cxnSp>"#
        ),
        format!(
            r#"<p:cxnSp xmlns:p="{P_NS}"><p:spPr/><p:nvCxnSpPr><p:cNvPr/><p:cNvCxnSpPr/><p:nvPr/></p:nvCxnSpPr></p:cxnSp>"#
        ),
        format!(
            r#"<p:cxnSp xmlns:p="{P_NS}"><p:nvCxnSpPr><p:cNvCxnSpPr/><p:cNvPr/><p:nvPr/></p:nvCxnSpPr><p:spPr/></p:cxnSp>"#
        ),
        format!(
            r#"<p:cxnSp xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><p:nvCxnSpPr><p:cNvPr/><p:cNvCxnSpPr><a:endCxn id="4" idx="1"/><a:stCxn id="3" idx="0"/></p:cNvCxnSpPr><p:nvPr/></p:nvCxnSpPr><p:spPr/></p:cxnSp>"#
        ),
    ];
    for document in documents {
        assert!(
            CT_ConnectionShape::from_xml(document.as_bytes()).is_err(),
            "{document}"
        );
    }

    for endpoints in [
        "".to_owned(),
        r#"<a:stCxn id="3" idx="0"/>"#.to_owned(),
        r#"<a:endCxn id="4" idx="1"/>"#.to_owned(),
    ] {
        let document = format!(
            r#"<p:cxnSp xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><p:nvCxnSpPr><p:cNvPr/><p:cNvCxnSpPr>{endpoints}</p:cNvCxnSpPr><p:nvPr/></p:nvCxnSpPr><p:spPr/></p:cxnSp>"#
        );
        CT_ConnectionShape::from_xml(document.as_bytes()).unwrap();
    }
}

#[test]
fn connector_connection_attributes_cannot_be_shadowed_by_qualified_names() {
    let valid = format!(
        r#"<p:cxnSp xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:lookalike"><p:nvCxnSpPr><p:cNvPr/><p:cNvCxnSpPr><a:stCxn x:id="99" x:idx="98" id="3" idx="2"/></p:cNvCxnSpPr><p:nvPr/></p:nvCxnSpPr><p:spPr/></p:cxnSp>"#
    );
    let parsed = CT_ConnectionShape::from_xml(valid.as_bytes()).unwrap();
    assert_eq!(parsed.start_connection.as_ref().unwrap().id, 3);
    assert_eq!(parsed.start_connection.as_ref().unwrap().idx, 2);
    let written = String::from_utf8(parsed.to_xml().unwrap()).unwrap();
    assert!(written.contains(r#"x:id="99" x:idx="98""#));

    let missing_unqualified = format!(
        r#"<p:cxnSp xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:lookalike"><p:nvCxnSpPr><p:cNvPr/><p:cNvCxnSpPr><a:endCxn x:id="97" x:idx="96"/></p:cNvCxnSpPr><p:nvPr/></p:nvCxnSpPr><p:spPr/></p:cxnSp>"#
    );
    assert!(CT_ConnectionShape::from_xml(missing_unqualified.as_bytes()).is_err());
}

#[test]
fn connector_preserves_unknown_children_in_their_schema_slots() {
    let xml = format!(
        r#"<p:cxnSp xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer" x:root="kept"><x:before/><p:nvCxnSpPr x:nv="kept"><x:nv-before/><p:cNvPr id="2" name="Connector"><x:drawing/></p:cNvPr><x:nv-middle/><p:cNvCxnSpPr x:connector="kept"><x:locks/><a:stCxn id="3" idx="0" x:endpoint="kept"><x:payload/></a:stCxn><x:between/><a:endCxn id="4" idx="1"/><x:extension/></p:cNvCxnSpPr><x:nv-after/><p:nvPr><x:application/></p:nvPr></p:nvCxnSpPr><x:middle/><p:spPr><a:prstGeom prst="line"><a:avLst/></a:prstGeom></p:spPr><x:after/><p:style><x:style/></p:style><p:extLst><x:extension/></p:extLst></p:cxnSp>"#
    );
    let mut parsed = CT_ConnectionShape::from_xml(xml.as_bytes()).unwrap();
    parsed.start_connection.as_mut().unwrap().id = 30;
    parsed.end_connection.as_mut().unwrap().idx = 10;
    let written = parsed.to_xml().unwrap();
    let text = std::str::from_utf8(&written).unwrap();
    for preserved in [
        r#"<x:before/>"#,
        r#"<x:drawing/>"#,
        r#"<x:locks/>"#,
        r#"<x:payload/>"#,
        r#"<x:between/>"#,
        r#"<x:application/>"#,
        r#"<x:after/>"#,
        r#"<p:style><x:style/></p:style>"#,
        r#"<p:extLst><x:extension/></p:extLst>"#,
    ] {
        assert!(text.contains(preserved), "missing {preserved}: {text}");
    }
    assert!(text.contains(r#"<a:stCxn id="30" idx="0""#));
    assert!(text.contains(r#"<a:endCxn id="4" idx="10""#));
    assert_eq!(parsed, CT_ConnectionShape::from_xml(&written).unwrap());
}

#[derive(Clone, Copy, Default)]
struct ConnectorCoverage {
    total: usize,
    starts: usize,
    ends: usize,
    nested: usize,
}

impl ConnectorCoverage {
    fn add(&mut self, other: Self) {
        self.total += other.total;
        self.starts += other.starts;
        self.ends += other.ends;
        self.nested += other.nested;
    }
}

fn verify_connectors(
    children: &[ShapeTreeChild],
    nested: bool,
    deck: &str,
    part: &str,
) -> ConnectorCoverage {
    let mut coverage = ConnectorCoverage::default();
    for child in children {
        match child {
            ShapeTreeChild::Connector(connector) => {
                let written = connector.to_xml().unwrap();
                assert_eq!(
                    connector,
                    &CT_ConnectionShape::from_xml(&written)
                        .unwrap_or_else(|error| panic!("{deck} {part}: {error}"))
                );
                coverage.total += 1;
                coverage.starts += usize::from(connector.start_connection.is_some());
                coverage.ends += usize::from(connector.end_connection.is_some());
                coverage.nested += usize::from(nested);
            }
            ShapeTreeChild::GroupShape(group) => {
                coverage.add(verify_connectors(&group.children, true, deck, part));
            }
            _ => {}
        }
    }
    coverage
}

#[test]
fn every_corpus_connector_round_trips_structurally() {
    let Some(corpus) = require_or_skip_corpus() else {
        return;
    };
    verify_fetched_corpus();
    let mut coverage = ConnectorCoverage::default();
    for entry in manifest_entries() {
        let package = OpcPackage::open(corpus.join(entry.path)).unwrap();
        for (part, content_type) in &package.content_types.overrides {
            let Some(xml) = package.get_part(part) else {
                continue;
            };
            let children = match content_type.as_str() {
                content_types::SLIDE => {
                    let parsed = CT_Slide::from_xml(xml)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path));
                    parsed.common_slide_data.shape_tree.children
                }
                content_types::SLIDE_LAYOUT => {
                    let parsed = CT_SlideLayout::from_xml(xml)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path));
                    parsed.common_slide_data.shape_tree.children
                }
                content_types::SLIDE_MASTER => {
                    let parsed = CT_SlideMaster::from_xml(xml)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path));
                    parsed.common_slide_data.shape_tree.children
                }
                _ => continue,
            };
            coverage.add(verify_connectors(&children, false, entry.path, part));
        }
    }
    assert!(coverage.total > 0);
    assert!(coverage.starts > 0);
    assert!(coverage.ends > 0);
    assert!(coverage.nested > 0);
    eprintln!(
        "Connector corpus gate checked {} connectors, {} starts, {} ends, and {} nested connectors",
        coverage.total, coverage.starts, coverage.ends, coverage.nested
    );
}

#[test]
fn nested_group_shape_tree_round_trips_with_tree_shape_preserved() {
    let xml = format!(
        r#"<p:sld xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><p:cSld><p:spTree><p:nvGrpSpPr/><p:grpSpPr/><p:grpSp><p:nvGrpSpPr/><p:grpSpPr><a:xfrm><a:off x="100" y="200"/><a:ext cx="300" cy="400"/><a:chOff x="10" y="20"/><a:chExt cx="30" cy="40"/></a:xfrm></p:grpSpPr><p:sp marker="outer"><p:nvSpPr><p:cNvPr/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/></p:sp><p:grpSp><p:nvGrpSpPr/><p:grpSpPr/><p:pic marker="inner"><p:nvPicPr><p:cNvPr/><p:cNvPicPr/><p:nvPr/></p:nvPicPr><p:blipFill/><p:spPr/></p:pic></p:grpSp></p:grpSp></p:spTree></p:cSld></p:sld>"#
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

#[test]
fn cropped_picture_round_trips_with_relationship_and_source_rectangle() {
    let xml = format!(
        r#"<p:pic xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="{R_NS}"><p:nvPicPr><p:cNvPr id="7" name="Photo"/><p:cNvPicPr/><p:nvPr/></p:nvPicPr><p:blipFill dpi="144"><a:blip r:embed="rId7"/><a:srcRect l="1000" t="2000" r="3000" b="4000"/><a:stretch><a:fillRect/></a:stretch></p:blipFill><p:spPr><a:xfrm><a:off x="1" y="2"/><a:ext cx="3" cy="4"/></a:xfrm></p:spPr></p:pic>"#
    );
    let picture = CT_Picture::from_xml(xml.as_bytes()).unwrap();
    let blip_fill = picture.blip_fill.as_ref().unwrap();
    assert_eq!(
        blip_fill.blip.as_ref().unwrap().embed.as_deref(),
        Some("rId7")
    );
    let crop = blip_fill.source_rect.as_ref().unwrap();
    assert_eq!(crop.left.unwrap().0, 1000);
    assert_eq!(crop.top.unwrap().0, 2000);
    assert_eq!(crop.right.unwrap().0, 3000);
    assert_eq!(crop.bottom.unwrap().0, 4000);
    let written = picture.to_xml().unwrap();
    assert_eq!(picture, CT_Picture::from_xml(&written).unwrap());
}

#[test]
fn picture_reads_any_prefix_and_writes_fixed_prefixes_in_schema_order() {
    let xml = format!(
        r#"<q:pic xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:rel="{R_NS}" xmlns:x="urn:producer"><q:nvPicPr><q:cNvPr id="1" name="Picture"/><q:cNvPicPr/><q:nvPr><q:ph type="pic" idx="3"/></q:nvPr></q:nvPicPr><q:blipFill x:fill-marker="kept"><d:blip rel:embed="rId1"/></q:blipFill><q:spPr><d:solidFill><d:srgbClr val="112233"/></d:solidFill></q:spPr></q:pic>"#
    );
    let picture = CT_Picture::from_xml(xml.as_bytes()).unwrap();
    assert_eq!(picture.placeholder.as_ref().unwrap().idx, Some(3));
    let written = picture.to_xml().unwrap();
    let text = std::str::from_utf8(&written).unwrap();
    assert!(text.starts_with(&format!(r#"<p:pic xmlns:p="{P_NS}""#)));
    assert!(text.contains(&format!(r#"<a:blip xmlns:r="{R_NS}" r:embed="rId1"/>"#)));
    assert!(text.contains(r#"<p:blipFill x:fill-marker="kept">"#));
    assert_order(&written, &["<p:nvPicPr", "<p:blipFill", "<p:spPr"]);
    assert_eq!(picture, CT_Picture::from_xml(&written).unwrap());
}

#[test]
fn picture_requires_non_visual_and_shape_properties_in_schema_order() {
    let invalid = [
        format!(r#"<p:pic xmlns:p="{P_NS}"><p:spPr/></p:pic>"#),
        format!(
            r#"<p:pic xmlns:p="{P_NS}"><p:nvPicPr><p:cNvPr/><p:cNvPicPr/><p:nvPr/></p:nvPicPr></p:pic>"#
        ),
        format!(
            r#"<p:pic xmlns:p="{P_NS}"><p:spPr/><p:nvPicPr><p:cNvPr/><p:cNvPicPr/><p:nvPr/></p:nvPicPr></p:pic>"#
        ),
        format!(
            r#"<p:pic xmlns:p="{P_NS}"><p:nvPicPr><p:cNvPicPr/><p:cNvPr/><p:nvPr/></p:nvPicPr><p:spPr/></p:pic>"#
        ),
        format!(
            r#"<p:pic xmlns:p="{P_NS}"><p:nvPicPr><p:cNvPr/><p:cNvPicPr/><p:nvPr/></p:nvPicPr><p:spPr/><p:blipFill/></p:pic>"#
        ),
        format!(
            r#"<p:pic xmlns:p="{P_NS}" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:x="urn:producer"><p:nvPicPr><p:cNvPr/><p:cNvPicPr/><p:nvPr/></p:nvPicPr><mc:AlternateContent><mc:Choice Requires="x"><x:not-a-fill/></mc:Choice></mc:AlternateContent><p:spPr/></p:pic>"#
        ),
    ];
    for xml in invalid {
        assert!(CT_Picture::from_xml(xml.as_bytes()).is_err(), "{xml}");
    }
}

#[test]
fn picture_preserves_unknown_children_and_alternate_blip_choice_verbatim() {
    let xml = format!(
        r#"<q:pic xmlns:q="{P_NS}" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:p14="urn:p14" xmlns:x="urn:producer" x:marker="kept"><q:nvPicPr><q:cNvPr/><q:cNvPicPr/><q:nvPr><x:before/><q:ph type="pic"/><x:after/></q:nvPr></q:nvPicPr><mc:AlternateContent><mc:Choice Requires="p14"><q:blipFill x:opaque="yes"/></mc:Choice></mc:AlternateContent><q:spPr/><x:after-properties/><q:style><x:style-data/></q:style><q:extLst><x:extension/></q:extLst></q:pic>"#
    );
    let picture = CT_Picture::from_xml(xml.as_bytes()).unwrap();
    assert!(picture.blip_fill.is_none());
    assert!(picture.placeholder.is_some());
    let written = picture.to_xml().unwrap();
    let text = std::str::from_utf8(&written).unwrap();
    assert!(text.contains(r#"x:marker="kept""#));
    assert!(text.contains(r#"<mc:AlternateContent><mc:Choice Requires="p14"><q:blipFill x:opaque="yes"/></mc:Choice></mc:AlternateContent>"#));
    assert_order(
        &written,
        &[
            "<p:nvPicPr",
            "<mc:AlternateContent",
            "<p:spPr",
            "<x:after-properties",
            "<q:style",
            "<q:extLst",
        ],
    );
    assert_eq!(picture, CT_Picture::from_xml(&written).unwrap());
}

#[test]
fn qualified_picture_relationship_attributes_are_not_shadowed() {
    let xml = format!(
        r#"<p:pic xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="{R_NS}" xmlns:x="urn:not-relationships"><p:nvPicPr><p:cNvPr/><p:cNvPicPr/><p:nvPr/></p:nvPicPr><p:blipFill><a:blip r:embed="rId1" x:embed="shadow"/></p:blipFill><p:spPr/></p:pic>"#
    );
    assert!(CT_Picture::from_xml(xml.as_bytes()).is_err());
}

#[test]
fn every_corpus_picture_round_trips_structurally() {
    let Some(corpus) = require_or_skip_corpus() else {
        return;
    };
    verify_fetched_corpus();
    let mut coverage = PictureCoverage::default();
    let mut raw_pictures = 0usize;
    for entry in manifest_entries() {
        let package = OpcPackage::open(corpus.join(entry.path)).unwrap();
        for (part, content_type) in &package.content_types.overrides {
            let Some(xml) = package.get_part(part) else {
                continue;
            };
            let part_coverage = match content_type.as_str() {
                content_types::SLIDE => {
                    let parsed = CT_Slide::from_xml(xml).unwrap();
                    let result = verify_pictures(
                        &parsed.common_slide_data.shape_tree.children,
                        false,
                        entry.path,
                        part,
                    );
                    assert_eq!(
                        parsed,
                        CT_Slide::from_xml(&parsed.to_xml().unwrap()).unwrap()
                    );
                    result
                }
                content_types::SLIDE_LAYOUT => {
                    let parsed = CT_SlideLayout::from_xml(xml).unwrap();
                    let result = verify_pictures(
                        &parsed.common_slide_data.shape_tree.children,
                        false,
                        entry.path,
                        part,
                    );
                    assert_eq!(
                        parsed,
                        CT_SlideLayout::from_xml(&parsed.to_xml().unwrap()).unwrap()
                    );
                    result
                }
                content_types::SLIDE_MASTER => {
                    let parsed = CT_SlideMaster::from_xml(xml).unwrap();
                    let result = verify_pictures(
                        &parsed.common_slide_data.shape_tree.children,
                        false,
                        entry.path,
                        part,
                    );
                    assert_eq!(
                        parsed,
                        CT_SlideMaster::from_xml(&parsed.to_xml().unwrap()).unwrap()
                    );
                    result
                }
                _ => continue,
            };
            let part_raw_pictures = count_picture_elements(xml);
            assert_eq!(
                part_coverage.typed + part_coverage.opaque,
                part_raw_pictures,
                "{} {part}: every picture must be typed or inside a preserved opaque child",
                entry.path
            );
            raw_pictures += part_raw_pictures;
            coverage.add(part_coverage);
        }
    }
    assert_eq!(raw_pictures, 240);
    assert_eq!(coverage.typed + coverage.opaque, raw_pictures);
    assert_eq!(coverage.cropped + coverage.opaque_cropped, 198);
    assert_eq!(coverage.nested, 98);
    assert_eq!(coverage.placeholders, 18);
    assert_eq!(coverage.alternate_blip, 1);
    assert!(coverage.cropped > 0);
    assert!(coverage.embedded > 0);
    assert!(coverage.linked > 0);
    assert!(coverage.nested > 0);
    assert!(coverage.placeholders > 0);
    assert!(coverage.alternate_blip > 0);
    eprintln!("Picture corpus gate checked {raw_pictures} p:pic elements: {coverage:?}");
}

#[derive(Debug, Default)]
struct PictureCoverage {
    typed: usize,
    opaque: usize,
    opaque_cropped: usize,
    cropped: usize,
    embedded: usize,
    linked: usize,
    nested: usize,
    placeholders: usize,
    alternate_blip: usize,
}

impl PictureCoverage {
    fn add(&mut self, other: Self) {
        self.typed += other.typed;
        self.opaque += other.opaque;
        self.opaque_cropped += other.opaque_cropped;
        self.cropped += other.cropped;
        self.embedded += other.embedded;
        self.linked += other.linked;
        self.nested += other.nested;
        self.placeholders += other.placeholders;
        self.alternate_blip += other.alternate_blip;
    }
}

fn verify_pictures(
    children: &[ShapeTreeChild],
    nested: bool,
    deck: &str,
    part: &str,
) -> PictureCoverage {
    let mut coverage = PictureCoverage::default();
    for child in children {
        match child {
            ShapeTreeChild::Picture(picture) => {
                let written = picture.to_xml().unwrap();
                assert_eq!(
                    picture,
                    &CT_Picture::from_xml(&written)
                        .unwrap_or_else(|error| panic!("{deck} {part}: {error}"))
                );
                coverage.typed += 1;
                coverage.nested += usize::from(nested);
                coverage.placeholders += usize::from(picture.placeholder.is_some());
                coverage.alternate_blip += usize::from(picture.blip_fill.is_none());
                if let Some(fill) = &picture.blip_fill {
                    coverage.cropped += usize::from(fill.source_rect.is_some());
                    if let Some(blip) = &fill.blip {
                        coverage.embedded += usize::from(blip.embed.is_some());
                        coverage.linked += usize::from(blip.link.is_some());
                    }
                }
            }
            ShapeTreeChild::GroupShape(group) => {
                coverage.add(verify_pictures(&group.children, true, deck, part));
            }
            ShapeTreeChild::GraphicFrame(frame) => {
                let xml = frame.to_xml().unwrap();
                coverage.opaque += count_picture_elements(&xml);
                coverage.opaque_cropped += count_pictures_with_descendant(&xml, b"srcRect");
            }
            ShapeTreeChild::Connector(connector) => {
                let xml = connector.to_xml().unwrap();
                coverage.opaque += count_picture_elements(&xml);
                coverage.opaque_cropped += count_pictures_with_descendant(&xml, b"srcRect");
            }
            ShapeTreeChild::AlternateContent(alternate) => {
                coverage.opaque += count_picture_elements(alternate.raw_xml());
                coverage.opaque_cropped +=
                    count_pictures_with_descendant(alternate.raw_xml(), b"srcRect");
            }
            _ => {}
        }
    }
    coverage
}

fn count_picture_elements(xml: &[u8]) -> usize {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut count = 0usize;
    loop {
        match reader.read_event_into(&mut buffer).unwrap() {
            Event::Start(start) | Event::Empty(start)
                if local_name(start.name().as_ref()) == b"pic" =>
            {
                count += 1;
            }
            Event::Eof => return count,
            _ => {}
        }
        buffer.clear();
    }
}

fn count_pictures_with_descendant(xml: &[u8], descendant: &[u8]) -> usize {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut picture_matches = Vec::new();
    let mut count = 0usize;
    loop {
        match reader.read_event_into(&mut buffer).unwrap() {
            Event::Start(start) if local_name(start.name().as_ref()) == b"pic" => {
                picture_matches.push(false);
            }
            Event::Start(start) | Event::Empty(start)
                if local_name(start.name().as_ref()) == descendant =>
            {
                if let Some(matches) = picture_matches.last_mut() {
                    *matches = true;
                }
            }
            Event::End(end) if local_name(end.name().as_ref()) == b"pic" => {
                count += usize::from(picture_matches.pop().unwrap());
            }
            Event::Eof => return count,
            _ => {}
        }
        buffer.clear();
    }
}

#[test]
fn placeholders_match_by_idx_before_type() {
    let title = PlaceholderKey {
        ph_type: PhType::Title,
        idx: Some(7),
    };
    let body = PlaceholderKey {
        ph_type: PhType::Body,
        idx: Some(7),
    };
    let same_type_other_idx = PlaceholderKey {
        ph_type: PhType::Title,
        idx: Some(8),
    };
    assert!(title.matches(&body));
    assert!(!title.matches(&same_type_other_idx));
}

#[test]
fn placeholders_match_by_type_when_either_idx_is_absent() {
    let indexed = PlaceholderKey {
        ph_type: PhType::Chart,
        idx: Some(3),
    };
    let unindexed = PlaceholderKey {
        ph_type: PhType::Chart,
        idx: None,
    };
    let other_type = PlaceholderKey {
        ph_type: PhType::Table,
        idx: None,
    };
    assert!(indexed.matches(&unindexed));
    assert!(!indexed.matches(&other_type));
}

#[test]
fn absent_placeholder_type_defaults_to_body() {
    let xml = format!(r#"<q:ph xmlns:q="{P_NS}" idx="4"/>"#);
    let placeholder = CT_Placeholder::from_xml(xml.as_bytes()).unwrap();
    assert_eq!(placeholder.ph_type, None);
    assert_eq!(placeholder.effective_type(), PhType::Body);
    assert_eq!(placeholder.key().idx, Some(4));
}

#[test]
fn title_and_centered_title_are_equivalent_placeholders() {
    let title = PlaceholderKey {
        ph_type: PhType::Title,
        idx: None,
    };
    let centered = PlaceholderKey {
        ph_type: PhType::CenteredTitle,
        idx: None,
    };
    assert!(title.matches(&centered));
    assert!(centered.matches(&title));
}

#[test]
fn body_subtitle_and_object_are_equivalent_placeholders() {
    let keys = [PhType::Body, PhType::Subtitle, PhType::Object]
        .map(|ph_type| PlaceholderKey { ph_type, idx: None });
    for left in &keys {
        for right in &keys {
            assert!(left.matches(right));
        }
    }
}

#[test]
fn placeholder_attributes_and_unknown_children_round_trip_in_place() {
    let xml = format!(
        r#"<q:ph xmlns:q="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer" type="vertBody" orient="vert" sz="half" idx="4294967295" hasCustomPrompt="1" x:marker="kept"><x:before/><a:opaque/><q:extLst><q:ext uri="raw"/></q:extLst></q:ph>"#
    );
    let placeholder = CT_Placeholder::from_xml(xml.as_bytes()).unwrap();
    assert_eq!(placeholder.ph_type, Some(PhType::VerticalBody));
    assert_eq!(placeholder.idx, Some(u32::MAX));
    let written = placeholder.to_xml().unwrap();
    let text = std::str::from_utf8(&written).unwrap();
    assert!(text.starts_with(&format!(r#"<p:ph xmlns:p="{P_NS}""#)));
    assert!(text.contains(r#"orient="vert""#));
    assert!(text.contains(r#"sz="half""#));
    assert!(text.contains(r#"hasCustomPrompt="1""#));
    assert!(text.contains(r#"x:marker="kept""#));
    assert!(text.contains(r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main""#));
    assert!(text.contains("<x:before/><a:opaque/><q:extLst><q:ext uri=\"raw\"/></q:extLst>"));
    assert_eq!(placeholder, CT_Placeholder::from_xml(&written).unwrap());
}

#[test]
fn typed_shape_placeholder_round_trips_inside_nested_groups() {
    let xml = format!(
        r#"<q:spTree xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer"><q:nvGrpSpPr/><q:grpSpPr/><q:grpSp><q:nvGrpSpPr/><q:grpSpPr/><q:sp marker="shape"><q:nvSpPr x:keep="yes"><q:cNvPr id="2" name="Title"/><x:between/><q:cNvSpPr/><q:nvPr><x:before/><q:ph type="ctrTitle" idx="5"/><x:after/></q:nvPr></q:nvSpPr><q:spPr/><q:txBody><d:bodyPr/><x:raw/><d:p/></q:txBody></q:sp></q:grpSp></q:spTree>"#
    );
    let tree = CT_ShapeTree::from_xml(xml.as_bytes()).unwrap();
    let ShapeTreeChild::GroupShape(group) = &tree.children[0] else {
        panic!("expected group");
    };
    let ShapeTreeChild::Shape(shape) = &group.children[0] else {
        panic!("expected typed shape");
    };
    let placeholder = shape.placeholder.as_ref().unwrap();
    assert_eq!(placeholder.ph_type, Some(PhType::CenteredTitle));
    assert_eq!(placeholder.idx, Some(5));
    let standalone_shape = shape.to_xml().unwrap();
    let standalone_text = std::str::from_utf8(&standalone_shape).unwrap();
    assert!(standalone_text.contains(&format!(r#"xmlns:q="{P_NS}""#)));
    assert!(standalone_text.contains(r#"xmlns:x="urn:producer""#));
    assert_eq!(shape, &CT_Shape::from_xml(&standalone_shape).unwrap());
    let standalone_placeholder = placeholder.to_xml().unwrap();
    assert_eq!(
        placeholder,
        &CT_Placeholder::from_xml(&standalone_placeholder).unwrap()
    );
    let written = tree.to_xml().unwrap();
    let text = std::str::from_utf8(&written).unwrap();
    assert!(text.contains("<p:sp marker=\"shape\""));
    assert!(text.contains("<p:nvSpPr x:keep=\"yes\">"));
    assert_order(&written, &["<x:before/>", "<p:ph", "<x:after/>"]);
    assert_eq!(tree, CT_ShapeTree::from_xml(&written).unwrap());
}

#[test]
fn ordinary_shape_properties_round_trip_in_schema_order() {
    let xml = format!(
        r#"<q:sp xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer"><q:nvSpPr><q:cNvPr id="7" name="Shape 7"/><q:cNvSpPr/><q:nvPr/></q:nvSpPr><q:spPr x:marker="kept"><x:before/><d:xfrm><d:off x="10" y="20"/><d:ext cx="30" cy="40"/></d:xfrm><x:between/><d:solidFill><d:srgbClr val="112233"/></d:solidFill><x:after/></q:spPr><x:between-shape/><q:style><x:style-payload/><d:lnRef idx="0"/><d:fillRef idx="0"/><d:effectRef idx="0"/><d:fontRef idx="none"/></q:style><q:txBody><d:bodyPr/><d:p/></q:txBody></q:sp>"#
    );
    let shape = CT_Shape::from_xml(xml.as_bytes()).unwrap();
    let properties: &CT_ShapeProperties = &shape.shape_properties;
    let transform = properties.transform.as_ref().unwrap();
    assert_eq!(transform.offset.unwrap().x.0, 10);
    assert!(properties.fill.is_some());

    let written = shape.to_xml().unwrap();
    let text = std::str::from_utf8(&written).unwrap();
    assert!(text.contains(r#"<p:spPr x:marker="kept">"#));
    assert!(text.contains("<x:before/>"));
    assert!(text.contains("<x:between/>"));
    assert!(text.contains("<x:after/>"));
    assert_order(
        &written,
        &[
            "<p:spPr",
            "<x:before/>",
            "<a:xfrm",
            "<x:between/>",
            "<a:solidFill",
            "<x:after/>",
            "<x:between-shape/>",
            "<p:style",
            "<p:txBody",
        ],
    );
    assert_eq!(shape, CT_Shape::from_xml(&written).unwrap());
}

#[test]
fn every_corpus_shape_placeholder_round_trips_structurally() {
    let Some(corpus) = require_or_skip_corpus() else {
        return;
    };
    verify_fetched_corpus();
    let mut placeholders = 0usize;
    for entry in manifest_entries() {
        let package = OpcPackage::open(corpus.join(entry.path)).unwrap();
        for (part, content_type) in &package.content_types.overrides {
            let Some(xml) = package.get_part(part) else {
                continue;
            };
            let children = match content_type.as_str() {
                content_types::SLIDE => {
                    &CT_Slide::from_xml(xml)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path))
                        .common_slide_data
                        .shape_tree
                        .children
                }
                content_types::SLIDE_LAYOUT => {
                    &CT_SlideLayout::from_xml(xml)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path))
                        .common_slide_data
                        .shape_tree
                        .children
                }
                content_types::SLIDE_MASTER => {
                    &CT_SlideMaster::from_xml(xml)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path))
                        .common_slide_data
                        .shape_tree
                        .children
                }
                _ => continue,
            };
            placeholders += verify_shape_placeholders(children, entry.path, part);
        }
    }
    assert!(placeholders > 0);
    eprintln!("Placeholder corpus gate checked {placeholders} typed p:ph elements");
}

fn verify_shape_placeholders(children: &[ShapeTreeChild], deck: &str, part: &str) -> usize {
    children
        .iter()
        .map(|child| match child {
            ShapeTreeChild::Shape(shape) => {
                let written = shape.to_xml().unwrap();
                assert_eq!(
                    shape,
                    &CT_Shape::from_xml(&written)
                        .unwrap_or_else(|error| panic!("{deck} {part}: {error}"))
                );
                usize::from(shape.placeholder.is_some())
            }
            ShapeTreeChild::GroupShape(group) => {
                verify_shape_placeholders(&group.children, deck, part)
            }
            _ => 0,
        })
        .sum()
}

#[test]
fn graphic_data_uri_dispatch_recognises_table_chart_smartart_and_ole() {
    let table =
        r#"<a:tbl><a:tblGrid><a:gridCol w="100"/></a:tblGrid><a:tr h="200"><a:tc/></a:tr></a:tbl>"#;
    let chart = r#"<c:chart xmlns:c="urn:chart" r:id="rId1"/>"#;
    let smartart = r#"<dgm:relIds xmlns:dgm="urn:diagram" r:dm="rId2"/>"#;
    let ole = r#"<mc:AlternateContent><mc:Choice Requires="p14"><p:oleObj name="object"/></mc:Choice></mc:AlternateContent>"#;

    let table_frame = CT_GraphicFrame::from_xml(
        graphic_frame_fixture(
            "http://schemas.openxmlformats.org/drawingml/2006/table",
            table,
        )
        .as_bytes(),
    )
    .unwrap();
    let GraphicDataPayload::Table(table) = table_frame.graphic_data.payload() else {
        panic!("table URI did not select the typed table branch");
    };
    assert_eq!(table.grid.columns.len(), 1);
    assert_eq!(table.rows.len(), 1);

    for (uri, payload, expected) in [
        (
            "http://schemas.openxmlformats.org/drawingml/2006/chart",
            chart,
            "chart",
        ),
        (
            "http://schemas.openxmlformats.org/drawingml/2006/diagram",
            smartart,
            "smartart",
        ),
        (
            "http://schemas.openxmlformats.org/presentationml/2006/ole",
            ole,
            "ole",
        ),
        ("urn:producer:graphic-data", "<x:payload/>", "other"),
    ] {
        let frame =
            CT_GraphicFrame::from_xml(graphic_frame_fixture(uri, payload).as_bytes()).unwrap();
        let raw = match (frame.graphic_data.payload(), expected) {
            (GraphicDataPayload::Chart(raw), "chart")
            | (GraphicDataPayload::SmartArt(raw), "smartart")
            | (GraphicDataPayload::Other(raw), "other") => raw,
            (GraphicDataPayload::Ole { raw, .. }, "ole") => raw,
            (actual, _) => panic!("{uri} selected the wrong branch: {actual:?}"),
        };
        assert_eq!(raw, payload.as_bytes());
        assert_eq!(
            frame,
            CT_GraphicFrame::from_xml(&frame.to_xml().unwrap()).unwrap()
        );
    }
}

#[test]
fn shape_ids_ignore_foreign_non_visual_elements() {
    let xml = br#"<p:spTree xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:extension"><p:nvGrpSpPr><p:cNvPr id="1"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><p:grpSp><p:nvGrpSpPr><x:cNvPr id="99"/><p:cNvPr id="4"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><p:graphicFrame><p:nvGraphicFramePr><x:cNvPr id="98"/><p:cNvPr id="6"/><p:cNvGraphicFramePr/><p:nvPr/></p:nvGraphicFramePr><p:xfrm><a:off x="1" y="2"/><a:ext cx="3" cy="4"/></p:xfrm><a:graphic><a:graphicData uri="urn:producer"><x:payload/></a:graphicData></a:graphic></p:graphicFrame></p:grpSp></p:spTree>"#;
    let tree = CT_ShapeTree::from_xml(xml).unwrap();
    let ShapeTreeChild::GroupShape(group) = &tree.children[0] else {
        panic!("expected group shape");
    };

    assert_eq!(tree.children[0].non_visual_id(), Some(4));
    assert_eq!(group.children[0].non_visual_id(), Some(6));
    let mut allocator = ShapeIdAllocator::scan(&tree);
    assert_eq!(allocator.allocate(), 2);
    assert_eq!(allocator.allocate(), 3);
    assert_eq!(allocator.allocate(), 5);
}

#[test]
fn ole_payload_projects_optional_fallback_picture_without_rewriting_raw_bytes() {
    let payload = r#"<mc:AlternateContent><mc:Choice Requires="p"><p:oleObj name="object"><p:embed/></p:oleObj></mc:Choice><mc:Fallback><p:oleObj name="object"><p:embed/><p:pic><p:nvPicPr><p:cNvPr/><p:cNvPicPr/><p:nvPr/></p:nvPicPr><p:blipFill><a:blip r:embed="rId7"/><a:stretch><a:fillRect/></a:stretch></p:blipFill><p:spPr><a:xfrm><a:off x="11" y="22"/><a:ext cx="33" cy="44"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr></p:pic></p:oleObj></mc:Fallback></mc:AlternateContent>"#;
    let frame = CT_GraphicFrame::from_xml(
        graphic_frame_fixture(
            "http://schemas.openxmlformats.org/presentationml/2006/ole",
            payload,
        )
        .as_bytes(),
    )
    .unwrap();
    let GraphicDataPayload::Ole { raw, preview } = frame.graphic_data.payload() else {
        panic!("OLE URI did not select the OLE branch");
    };

    assert_eq!(raw, payload.as_bytes());
    let preview = preview
        .as_ref()
        .expect("fallback picture was not projected");
    assert_eq!(
        preview
            .blip_fill
            .as_ref()
            .and_then(|fill| fill.blip.as_ref())
            .and_then(|blip| blip.embed.as_deref()),
        Some("rId7")
    );
    let transform = preview
        .shape_properties
        .transform
        .as_ref()
        .expect("fallback picture transform was not projected");
    assert_eq!(transform.offset.as_ref().map(|offset| offset.x.0), Some(11));
    assert_eq!(
        transform.extent.as_ref().map(|extents| extents.cx.0),
        Some(33)
    );
    assert_eq!(
        frame,
        CT_GraphicFrame::from_xml(&frame.to_xml().unwrap()).unwrap()
    );

    let absent = CT_GraphicFrame::from_xml(
        graphic_frame_fixture(
            "http://schemas.openxmlformats.org/presentationml/2006/ole",
            r#"<p:oleObj name="object"><p:embed/></p:oleObj>"#,
        )
        .as_bytes(),
    )
    .unwrap();
    assert!(matches!(
        absent.graphic_data.payload(),
        GraphicDataPayload::Ole { preview: None, .. }
    ));
}

#[test]
fn graphic_frame_requires_children_in_schema_order_and_writes_fixed_prefixes() {
    let xml = format!(
        r#"<q:graphicFrame xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:c="urn:chart"><q:nvGraphicFramePr><q:cNvPr id="2" name="Chart"/><q:cNvGraphicFramePr/><q:nvPr/></q:nvGraphicFramePr><q:xfrm rot="60000"><d:off x="10" y="20"/><d:ext cx="30" cy="40"/></q:xfrm><d:graphic><d:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart/></d:graphicData></d:graphic></q:graphicFrame>"#
    );
    let frame = CT_GraphicFrame::from_xml(xml.as_bytes()).unwrap();
    assert_eq!(frame.transform.offset.unwrap().x.0, 10);
    let written = frame.to_xml().unwrap();
    let text = std::str::from_utf8(&written).unwrap();
    assert!(text.starts_with("<p:graphicFrame xmlns:p="));
    assert_order(
        &written,
        &[
            "<p:nvGraphicFramePr",
            "<p:xfrm",
            "<a:off",
            "<a:graphic",
            "<a:graphicData",
        ],
    );
    assert_eq!(frame, CT_GraphicFrame::from_xml(&written).unwrap());

    for invalid in [
        format!(
            r#"<p:graphicFrame xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><p:xfrm/><a:graphic><a:graphicData uri="urn:x"><a:x/></a:graphicData></a:graphic></p:graphicFrame>"#
        ),
        format!(
            r#"<p:graphicFrame xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><p:nvGraphicFramePr/><a:graphic><a:graphicData uri="urn:x"><a:x/></a:graphicData></a:graphic><p:xfrm/></p:graphicFrame>"#
        ),
        format!(
            r#"<p:graphicFrame xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><p:nvGraphicFramePr/><p:xfrm/></p:graphicFrame>"#
        ),
        format!(
            r#"<p:graphicFrame xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><p:nvGraphicFramePr/><p:xfrm/><p:xfrm/><a:graphic><a:graphicData uri="urn:x"><a:x/></a:graphicData></a:graphic></p:graphicFrame>"#
        ),
    ] {
        assert!(
            CT_GraphicFrame::from_xml(invalid.as_bytes()).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn table_graphic_frame_constructor_writes_the_canonical_shell() {
    let table = CT_Table::new(2, 3, Emu(302), Emu(201)).unwrap();
    let frame =
        CT_GraphicFrame::new_table(7, "Table & 7", CT_Transform2D::default(), table).unwrap();
    let written = frame.to_xml().unwrap();
    let xml = String::from_utf8(written.clone()).unwrap();
    assert!(xml.contains(r#"<p:cNvPr id="7" name="Table &amp; 7"/>"#));
    assert!(xml.contains("<p:cNvGraphicFramePr/>"));
    assert!(xml.contains("<p:nvPr/>"));
    assert!(xml.contains(
        r#"<a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/table">"#
    ));
    assert!(xml.find("<p:nvGraphicFramePr>").unwrap() < xml.find("<p:xfrm").unwrap());
    assert!(xml.find("<p:xfrm").unwrap() < xml.find("<a:graphic>").unwrap());
    let reparsed = CT_GraphicFrame::from_xml(&written).unwrap();
    assert_eq!(reparsed, frame);
    assert!(matches!(
        reparsed.graphic_data.payload(),
        GraphicDataPayload::Table(_)
    ));

    let mut invalid_table = CT_Table::new(1, 1, Emu(100), Emu(100)).unwrap();
    invalid_table.rows.clear();
    let error =
        CT_GraphicFrame::new_table(8, "Invalid table", CT_Transform2D::default(), invalid_table)
            .unwrap_err();
    assert!(error.to_string().contains("at least one a:tr"));
}

#[test]
fn chart_relationship_projection_ignores_unqualified_and_foreign_ids() {
    for payload in [
        r#"<c:chart xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" id="plain"/>"#,
        r#"<c:chart xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:x="urn:foreign" x:id="foreign"/>"#,
        r#"<c:chart xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" r:id=""/>"#,
    ] {
        let frame = CT_GraphicFrame::from_xml(
            graphic_frame_fixture(
                "http://schemas.openxmlformats.org/drawingml/2006/chart",
                payload,
            )
            .as_bytes(),
        )
        .unwrap();
        assert_eq!(frame.chart_relationship_id(), None);
    }
}

#[test]
fn authored_chart_graphic_frame_round_trips() {
    let mut frame =
        CT_GraphicFrame::new_chart(9, "Chart & 9", CT_Transform2D::default(), "rId7").unwrap();
    assert!(frame.graphic_data.table_mut().is_none());
    assert_eq!(frame.chart_relationship_id(), Some("rId7"));
    let written = frame.to_xml().unwrap();
    let xml = String::from_utf8(written.clone()).unwrap();
    assert!(xml.contains(r#"<p:cNvPr id="9" name="Chart &amp; 9"/>"#));
    assert!(xml.contains(
        r#"<a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/chart">"#
    ));
    assert!(xml.contains(
        r#"<c:chart xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" r:id="rId7"/>"#
    ));
    assert_order(
        &written,
        &[
            "<p:nvGraphicFramePr",
            "<p:xfrm",
            "<a:graphic",
            "<a:graphicData",
            "<c:chart",
        ],
    );
    let reparsed = CT_GraphicFrame::from_xml(&written).unwrap();
    assert_eq!(reparsed.chart_relationship_id(), Some("rId7"));
    assert_eq!(reparsed, frame);
    assert!(matches!(
        reparsed.graphic_data.payload(),
        GraphicDataPayload::Chart(_)
    ));
}

#[test]
fn chart_choice_and_picture_fallback_remain_byte_preserved() {
    let raw = format!(
        r#"<mc:AlternateContent xmlns:mc="{MC_NS}" xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:z="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:rel="{R_NS}" marker="kept"><mc:Choice Requires="z"><q:graphicFrame><q:nvGraphicFramePr/><q:xfrm><d:off x="1" y="2"/><d:ext cx="127000" cy="254000"/></q:xfrm><d:graphic><d:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/chart"><z:chart rel:id="chartAlias"/></d:graphicData></d:graphic></q:graphicFrame></mc:Choice><mc:Fallback><q:pic><q:nvPicPr><q:cNvPr/><q:cNvPicPr/><q:nvPr/></q:nvPicPr><q:blipFill><d:blip rel:embed="previewAlias"/><d:stretch><d:fillRect/></d:stretch></q:blipFill><q:spPr/></q:pic></mc:Fallback></mc:AlternateContent>"#
    );
    let tree_xml = format!(
        r#"<q:spTree xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:mc="{MC_NS}" xmlns:z="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:rel="{R_NS}"><q:nvGrpSpPr/><q:grpSpPr/>{raw}</q:spTree>"#
    );
    let tree = rpptx_oxml::shape_tree::CT_ShapeTree::from_xml(tree_xml.as_bytes()).unwrap();
    let ShapeTreeChild::AlternateContent(alternate) = &tree.children[0] else {
        panic!("expected chart alternate content");
    };

    assert_eq!(
        alternate
            .chart_choice()
            .and_then(CT_GraphicFrame::chart_relationship_id),
        Some("chartAlias")
    );
    assert!(alternate.picture_fallback().is_some());
    assert_eq!(alternate.raw_xml(), raw.as_bytes());

    let written = tree.to_xml().unwrap();
    assert!(
        written
            .windows(raw.len())
            .any(|window| window == raw.as_bytes()),
        "the preserved alternate-content subtree must remain byte-identical"
    );
}

#[test]
fn malformed_non_chart_choice_remains_opaque() {
    let raw = format!(
        r#"<mc:AlternateContent xmlns:mc="{MC_NS}" xmlns:p="{P_NS}"><mc:Choice Requires="x"><p:graphicFrame><p:xfrm/></p:graphicFrame></mc:Choice><mc:Fallback/></mc:AlternateContent>"#
    );
    let tree_xml = format!(
        r#"<p:spTree xmlns:p="{P_NS}" xmlns:mc="{MC_NS}"><p:nvGrpSpPr/><p:grpSpPr/>{raw}</p:spTree>"#
    );

    let tree = rpptx_oxml::shape_tree::CT_ShapeTree::from_xml(tree_xml.as_bytes()).unwrap();
    let ShapeTreeChild::AlternateContent(alternate) = &tree.children[0] else {
        panic!("expected preserved alternate content");
    };
    assert!(alternate.chart_choice().is_none());
    assert_eq!(alternate.raw_xml(), raw.as_bytes());
}

#[test]
fn non_chart_choice_with_descendant_chart_uri_remains_opaque() {
    let raw = format!(
        r#"<mc:AlternateContent xmlns:mc="{MC_NS}" xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer"><mc:Choice Requires="x"><p:graphicFrame><p:nvGraphicFramePr/><p:xfrm/><a:graphic><a:graphicData uri="urn:producer"><x:extension><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/chart"/></x:extension></a:graphicData></a:graphic></p:graphicFrame></mc:Choice><mc:Fallback/></mc:AlternateContent>"#
    );
    let tree_xml = format!(
        r#"<p:spTree xmlns:p="{P_NS}" xmlns:mc="{MC_NS}"><p:nvGrpSpPr/><p:grpSpPr/>{raw}</p:spTree>"#
    );

    let tree = rpptx_oxml::shape_tree::CT_ShapeTree::from_xml(tree_xml.as_bytes()).unwrap();
    let ShapeTreeChild::AlternateContent(alternate) = &tree.children[0] else {
        panic!("expected preserved alternate content");
    };
    assert!(alternate.chart_choice().is_none());
    assert_eq!(alternate.raw_xml(), raw.as_bytes());
}

#[test]
fn graphic_frame_preserves_unknown_payload_and_extension_xml_byte_for_byte() {
    let payload = r#"<u:payload xmlns:u="urn:unknown" marker="a &amp; b"><u:data><!--kept--></u:data></u:payload>"#;
    let xml = format!(
        r#"<p:graphicFrame xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer" x:frame="kept"><x:before/><!--frame-kept--><p:nvGraphicFramePr x:nv="kept"><!--nv-kept--><x:nv-child/><?nv kept?></p:nvGraphicFramePr><x:between-nv-and-xfrm/><p:xfrm/><x:between-xfrm-and-graphic/><a:graphic x:graphic="kept"><x:graphic-before/><!--graphic-kept--><a:graphicData uri="urn:producer:data" x:data="kept"><!--data-before-->{payload}<?data kept?><x:payload-after/></a:graphicData><x:graphic-after/></a:graphic><x:before-ext/><p:extLst><p:ext uri="kept"><x:extension/></p:ext></p:extLst><x:after/></p:graphicFrame>"#
    );
    let frame = CT_GraphicFrame::from_xml(xml.as_bytes()).unwrap();
    let GraphicDataPayload::Other(actual) = frame.graphic_data.payload() else {
        panic!("unknown URI did not select the opaque branch");
    };
    assert_eq!(actual, payload.as_bytes());
    let written = frame.to_xml().unwrap();
    let text = std::str::from_utf8(&written).unwrap();
    for raw in [
        "<x:before/>",
        "<!--frame-kept-->",
        "<!--nv-kept-->",
        "<x:nv-child/>",
        "<?nv kept?>",
        "<x:between-nv-and-xfrm/>",
        "<x:between-xfrm-and-graphic/>",
        "<x:graphic-before/>",
        "<!--graphic-kept-->",
        "<!--data-before-->",
        payload,
        "<?data kept?>",
        "<x:payload-after/>",
        "<x:graphic-after/>",
        "<x:before-ext/>",
        "<p:extLst><p:ext uri=\"kept\"><x:extension/></p:ext></p:extLst>",
        "<x:after/>",
    ] {
        assert!(text.contains(raw), "missing {raw}: {text}");
    }
    assert_order(
        &written,
        &[
            "<x:before/>",
            "<p:nvGraphicFramePr",
            "<x:between-nv-and-xfrm/>",
            "<p:xfrm",
            "<x:between-xfrm-and-graphic/>",
            "<a:graphic",
            "<x:before-ext/>",
            "<p:extLst",
            "<x:after/>",
        ],
    );
    assert_eq!(frame, CT_GraphicFrame::from_xml(&written).unwrap());
}

#[test]
fn empty_namespace_shadow_does_not_hide_later_inherited_binding() {
    let xml = format!(
        r#"<p:spTree xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:outer"><p:nvGrpSpPr/><p:grpSpPr/><p:graphicFrame><p:nvGraphicFramePr/><p:xfrm/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/table"><a:tbl><a:tblPr><x:shadow xmlns:x="urn:inner"/></a:tblPr><a:tblGrid><a:gridCol w="100"/></a:tblGrid><a:tr h="200"><a:tc><x:needed/><a:tcPr/></a:tc></a:tr></a:tbl></a:graphicData></a:graphic></p:graphicFrame></p:spTree>"#
    );
    let tree = CT_ShapeTree::from_xml(xml.as_bytes()).unwrap();
    let ShapeTreeChild::GraphicFrame(frame) = &tree.children[0] else {
        panic!("shape tree did not retain the graphic frame");
    };
    let frame_xml = frame.to_xml().unwrap();
    let frame_text = std::str::from_utf8(&frame_xml).unwrap();
    assert!(frame_text.contains(r#"xmlns:x="urn:outer""#));
    assert!(frame_text.contains(r#"<x:shadow xmlns:x="urn:inner"/>"#));
    assert!(frame_text.contains("<x:needed/>"));

    let GraphicDataPayload::Table(table) = frame.graphic_data.payload() else {
        panic!("table URI did not select the typed table branch");
    };
    let table_xml = table.to_xml().unwrap();
    let table_text = std::str::from_utf8(&table_xml).unwrap();
    assert!(table_text.contains(r#"xmlns:x="urn:outer""#));
    assert!(table_text.contains("<x:needed/>"));
}

#[test]
fn every_corpus_graphic_frame_round_trips_structurally() {
    let Some(corpus) = require_or_skip_corpus() else {
        return;
    };
    verify_fetched_corpus();
    let mut coverage = GraphicFrameCoverage::default();
    for entry in manifest_entries() {
        let package = OpcPackage::open(corpus.join(entry.path)).unwrap();
        for (part, content_type) in &package.content_types.overrides {
            let Some(xml) = package.get_part(part) else {
                continue;
            };
            let children = match content_type.as_str() {
                content_types::SLIDE => {
                    &CT_Slide::from_xml(xml)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path))
                        .common_slide_data
                        .shape_tree
                        .children
                }
                content_types::SLIDE_LAYOUT => {
                    &CT_SlideLayout::from_xml(xml)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path))
                        .common_slide_data
                        .shape_tree
                        .children
                }
                content_types::SLIDE_MASTER => {
                    &CT_SlideMaster::from_xml(xml)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path))
                        .common_slide_data
                        .shape_tree
                        .children
                }
                _ => continue,
            };
            verify_graphic_frames(children, entry.path, part, &mut coverage);
        }
    }
    assert_eq!(coverage.tables, 26);
    assert_eq!(coverage.charts, 26);
    assert_eq!(coverage.smartart, 18);
    assert_eq!(coverage.ole, 16);
    assert_eq!(coverage.other, 0);
    assert_eq!(coverage.total(), 86);
    eprintln!("Graphic-frame corpus gate checked {coverage:?}");
}

#[test]
fn text_body_plain_text_preserves_run_field_break_and_paragraph_order() {
    let body = CT_TextBody::from_xml(
        br#"<a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>first</a:t></a:r><a:br/><a:fld id="{14E3C6B4-82AF-4BA3-9737-6C8DF2E2A225}" type="datetime"><a:t>field</a:t></a:fld><a:r><a:t> last</a:t></a:r></a:p><a:p><a:r><a:t>second</a:t></a:r></a:p></a:txBody>"#,
    )
    .unwrap();
    assert_eq!(body.plain_text(), "first\nfield last\nsecond");
}

#[test]
fn notes_slide_extracts_only_body_placeholder_text() {
    let xml = format!(
        r#"<p:notes xmlns:p="{P_NS}" xmlns:a="{}"><p:cSld><p:spTree><p:nvGrpSpPr/><p:grpSpPr/>
<p:sp><p:nvSpPr><p:cNvPr/><p:cNvSpPr/><p:nvPr><p:ph type="sldImg"/></p:nvPr></p:nvSpPr><p:spPr/><p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>slide image</a:t></a:r></a:p></p:txBody></p:sp>
<p:sp><p:nvSpPr><p:cNvPr/><p:cNvSpPr/><p:nvPr><p:ph type="sldNum"/></p:nvPr></p:nvSpPr><p:spPr/><p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:fld id="{{14E3C6B4-82AF-4BA3-9737-6C8DF2E2A225}}"><a:t>12</a:t></a:fld></a:p></p:txBody></p:sp>
<p:sp><p:nvSpPr><p:cNvPr/><p:cNvSpPr/><p:nvPr><p:ph type="ftr"/></p:nvPr></p:nvSpPr><p:spPr/><p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>footer</a:t></a:r></a:p></p:txBody></p:sp>
<p:sp><p:nvSpPr><p:cNvPr/><p:cNvSpPr/><p:nvPr><p:ph/></p:nvPr></p:nvSpPr><p:spPr/><p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>speaker</a:t></a:r><a:br/><a:r><a:t>note</a:t></a:r></a:p></p:txBody></p:sp>
</p:spTree></p:cSld></p:notes>"#,
        oxml_drawing::namespace::A_NS
    );
    let notes = CT_NotesSlide::from_xml(xml.as_bytes()).unwrap();
    assert_eq!(notes.notes_text(), "speaker\nnote");
    assert_eq!(
        notes,
        CT_NotesSlide::from_xml(&notes.to_xml().unwrap()).unwrap()
    );
}

#[test]
fn notes_parts_read_any_prefix_write_fixed_prefixes_and_schema_order() {
    let color_map = r#"bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink""#;
    let slide_xml = format!(
        r#"<q:notes xmlns:q="{P_NS}" xmlns:d="{}"><q:cSld><q:spTree><q:nvGrpSpPr/><q:grpSpPr/></q:spTree></q:cSld><q:clrMapOvr><d:masterClrMapping/></q:clrMapOvr><q:extLst><q:ext uri="slide"/></q:extLst></q:notes>"#,
        oxml_drawing::namespace::A_NS
    );
    let slide = CT_NotesSlide::from_xml(slide_xml.as_bytes()).unwrap();
    let slide_written = slide.to_xml().unwrap();
    assert_order(&slide_written, &["<p:cSld", "<p:clrMapOvr", "<q:extLst"]);
    assert_eq!(slide, CT_NotesSlide::from_xml(&slide_written).unwrap());

    let master_xml = format!(
        r#"<q:notesMaster xmlns:q="{P_NS}" xmlns:d="{}"><q:cSld><q:spTree><q:nvGrpSpPr/><q:grpSpPr/></q:spTree></q:cSld><q:clrMap {color_map}/><q:hf hdr="1"/><q:notesStyle><d:lvl1pPr/></q:notesStyle><q:extLst><q:ext uri="master"/></q:extLst></q:notesMaster>"#,
        oxml_drawing::namespace::A_NS
    );
    let master = CT_NotesMaster::from_xml(master_xml.as_bytes()).unwrap();
    let master_written = master.to_xml().unwrap();
    assert_order(
        &master_written,
        &[
            "<p:cSld",
            "<p:clrMap",
            "<q:hf",
            "<p:notesStyle",
            "<q:extLst",
        ],
    );
    assert_eq!(master, CT_NotesMaster::from_xml(&master_written).unwrap());

    let out_of_order = format!(
        r#"<p:notesMaster xmlns:p="{P_NS}"><p:clrMap {color_map}/><p:cSld><p:spTree><p:nvGrpSpPr/><p:grpSpPr/></p:spTree></p:cSld><p:notesStyle/></p:notesMaster>"#
    );
    assert!(CT_NotesMaster::from_xml(out_of_order.as_bytes()).is_err());
}

#[test]
fn notes_master_without_notes_style_round_trips_with_extension_in_schema_order() {
    let color_map = r#"bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink""#;
    for header_footer in ["", r#"<q:hf hdr="1"/>"#] {
        let xml = format!(
            r#"<q:notesMaster xmlns:q="{P_NS}" xmlns:x="urn:producer"><q:cSld><q:spTree><q:nvGrpSpPr/><q:grpSpPr/></q:spTree></q:cSld><q:clrMap {color_map}/>{header_footer}<x:between/><q:extLst><x:payload/></q:extLst></q:notesMaster>"#
        );
        let parsed = CT_NotesMaster::from_xml(xml.as_bytes()).unwrap();
        assert!(parsed.notes_style.is_none());
        let written = parsed.to_xml().unwrap();
        let mut expected = vec!["<p:cSld", "<p:clrMap"];
        if !header_footer.is_empty() {
            expected.push("<q:hf");
        }
        expected.extend(["<x:between", "<q:extLst"]);
        assert_order(&written, &expected);
        assert_eq!(
            parsed,
            CT_NotesMaster::from_xml(&written).unwrap(),
            "{header_footer}"
        );
    }
}

#[test]
fn notes_parts_preserve_unmodelled_children_in_their_schema_slots() {
    let producer_before =
        r#"<x:before marker="a &amp; b"><x:data/><!--kept--><?producer value?></x:before>"#;
    let producer_between = r#"<x:between value="2"/>"#;
    let producer_after = r#"<x:after value="3"/>"#;
    let xml = format!(
        r#"<q:notes xmlns:q="{P_NS}" xmlns:d="{}" xmlns:x="urn:producer" x:root="kept">{producer_before}<q:cSld><q:spTree><q:nvGrpSpPr/><q:grpSpPr/></q:spTree></q:cSld>{producer_between}<q:clrMapOvr><d:masterClrMapping/></q:clrMapOvr>{producer_after}<q:extLst><x:payload/></q:extLst></q:notes>"#,
        oxml_drawing::namespace::A_NS
    );
    let parsed = CT_NotesSlide::from_xml(xml.as_bytes()).unwrap();
    let written = parsed.to_xml().unwrap();
    let written_text = std::str::from_utf8(&written).unwrap();
    assert!(written_text.contains(r#"xmlns:x="urn:producer""#));
    assert!(written_text.contains(r#"x:root="kept""#));
    for raw in [producer_before, producer_between, producer_after] {
        assert!(
            written
                .windows(raw.len())
                .any(|window| window == raw.as_bytes())
        );
    }
    assert_order(
        &written,
        &[
            producer_before,
            "<p:cSld",
            producer_between,
            "<p:clrMapOvr",
            producer_after,
        ],
    );
}

#[test]
fn every_corpus_notes_slide_and_master_round_trips_structurally() {
    let Some(corpus) = require_or_skip_corpus() else {
        return;
    };
    verify_fetched_corpus();
    let mut notes_slides = 0usize;
    let mut notes_masters = 0usize;
    let mut nonempty_notes = 0usize;
    for entry in manifest_entries() {
        let package = OpcPackage::open(corpus.join(entry.path)).unwrap();
        for (part, content_type) in &package.content_types.overrides {
            let Some(xml) = package.get_part(part) else {
                continue;
            };
            match content_type.as_str() {
                content_types::NOTES_SLIDE => {
                    let parsed = CT_NotesSlide::from_xml(xml)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path));
                    nonempty_notes += usize::from(!parsed.notes_text().is_empty());
                    assert_eq!(
                        parsed,
                        CT_NotesSlide::from_xml(&parsed.to_xml().unwrap()).unwrap(),
                        "{} {part}",
                        entry.path
                    );
                    notes_slides += 1;
                }
                content_types::NOTES_MASTER => {
                    let parsed = CT_NotesMaster::from_xml(xml)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path));
                    assert_eq!(
                        parsed,
                        CT_NotesMaster::from_xml(&parsed.to_xml().unwrap()).unwrap(),
                        "{} {part}",
                        entry.path
                    );
                    notes_masters += 1;
                }
                _ => {}
            }
        }
    }
    assert!(notes_slides > 0);
    assert!(notes_masters > 0);
    assert!(nonempty_notes > 0);
    eprintln!(
        "Notes corpus gate checked {notes_slides} notes slides, {notes_masters} notes masters, and {nonempty_notes} nonempty note bodies"
    );
}

#[test]
fn corpus_notes_relationships_are_complete() {
    let Some(corpus) = require_or_skip_corpus() else {
        return;
    };
    verify_fetched_corpus();
    let mut notes_slides = 0usize;
    let mut notes_masters = 0usize;
    for entry in manifest_entries() {
        let package = OpcPackage::open(corpus.join(entry.path)).unwrap();
        for (part, content_type) in &package.content_types.overrides {
            if !package.parts.contains_key(part) {
                continue;
            }
            let relationships = package.get_part_rels(part);
            let count = |relationship_type: &str| {
                relationships.map_or(0, |items| {
                    items
                        .items
                        .iter()
                        .filter(|relationship| relationship.rel_type == relationship_type)
                        .count()
                })
            };
            match content_type.as_str() {
                content_types::NOTES_SLIDE => {
                    assert_eq!(count(rel_types::NOTES_MASTER), 1, "{} {part}", entry.path);
                    assert_eq!(count(rel_types::SLIDE), 1, "{} {part}", entry.path);
                    notes_slides += 1;
                }
                content_types::NOTES_MASTER => {
                    assert_eq!(count(rel_types::THEME), 1, "{} {part}", entry.path);
                    notes_masters += 1;
                }
                content_types::SLIDE => {
                    assert!(count(rel_types::NOTES_SLIDE) <= 1, "{} {part}", entry.path);
                }
                _ => {}
            }
        }
    }
    assert!(notes_slides > 0);
    assert!(notes_masters > 0);
}

const TABLE_GRAPHIC_DATA_URI: &str = "http://schemas.openxmlformats.org/drawingml/2006/table";

fn graphic_frame_fixture(uri: &str, payload: &str) -> String {
    format!(
        r#"<p:graphicFrame xmlns:p="{P_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="{R_NS}" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:x="urn:producer"><p:nvGraphicFramePr/><p:xfrm><a:off x="1" y="2"/><a:ext cx="3" cy="4"/></p:xfrm><a:graphic><a:graphicData uri="{uri}">{payload}</a:graphicData></a:graphic></p:graphicFrame>"#
    )
}

#[derive(Debug, Default)]
struct GraphicFrameCoverage {
    tables: usize,
    charts: usize,
    smartart: usize,
    ole: usize,
    other: usize,
}

impl GraphicFrameCoverage {
    const fn total(&self) -> usize {
        self.tables + self.charts + self.smartart + self.ole + self.other
    }
}

fn verify_graphic_frames(
    children: &[ShapeTreeChild],
    deck: &str,
    part: &str,
    coverage: &mut GraphicFrameCoverage,
) {
    for child in children {
        match child {
            ShapeTreeChild::GraphicFrame(frame) => {
                let written = frame.to_xml().unwrap();
                assert_eq!(
                    frame.as_ref(),
                    &CT_GraphicFrame::from_xml(&written)
                        .unwrap_or_else(|error| panic!("{deck} {part}: {error}")),
                    "{deck} {part}: graphic frame changed"
                );
                match frame.graphic_data.payload() {
                    GraphicDataPayload::Table(table) => {
                        assert_eq!(frame.graphic_data.uri, TABLE_GRAPHIC_DATA_URI);
                        assert_eq!(
                            table.as_ref(),
                            &CT_Table::from_xml(&table.to_xml().unwrap()).unwrap()
                        );
                        coverage.tables += 1;
                    }
                    GraphicDataPayload::Chart(_) => coverage.charts += 1,
                    GraphicDataPayload::SmartArt(_) => coverage.smartart += 1,
                    GraphicDataPayload::Ole { .. } => coverage.ole += 1,
                    GraphicDataPayload::Other(_) => coverage.other += 1,
                }
            }
            ShapeTreeChild::GroupShape(group) => {
                verify_graphic_frames(&group.children, deck, part, coverage);
            }
            _ => {}
        }
    }
}

#[test]
fn rewrites_relationship_attributes_and_no_other_bytes() {
    let raw = format!(
        r#"<x:payload xmlns:x="urn:producer" xmlns:r="{R_NS}" r:embed="rId1" x:keep="rId1"><x:item r:link='rId2'/><x:item r:dm="rId3"/></x:payload>"#
    );
    let map = HashMap::from([
        ("rId1".to_owned(), "rId11".to_owned()),
        ("rId2".to_owned(), "rId12".to_owned()),
        ("rId3".to_owned(), "rId13".to_owned()),
    ]);

    let rewritten = rewrite_rel_ids(raw.as_bytes(), &map).unwrap();

    assert_eq!(
        rewritten,
        format!(
            r#"<x:payload xmlns:x="urn:producer" xmlns:r="{R_NS}" r:embed="rId11" x:keep="rId1"><x:item r:link='rId12'/><x:item r:dm="rId13"/></x:payload>"#
        )
        .into_bytes()
    );
}

#[test]
fn relationship_namespace_aliases_and_shadowing_are_respected() {
    let raw = format!(
        r#"<p:root xmlns:p="urn:producer" xmlns:rel="{R_NS}" xmlns:r="{R_NS}" rel:embed="rId1"><p:item xmlns:r="urn:shadow" r:link="rId2" rel:dm="rId3"/><p:item r:id="rId4" r:lo="rId&#52;"/></p:root>"#
    );
    let map = HashMap::from([
        ("rId1".to_owned(), "rId11".to_owned()),
        ("rId2".to_owned(), "rId12".to_owned()),
        ("rId3".to_owned(), "rId13".to_owned()),
        ("rId4".to_owned(), "rId14".to_owned()),
    ]);

    let rewritten = rewrite_rel_ids(raw.as_bytes(), &map).unwrap();

    assert_eq!(
        rewritten,
        format!(
            r#"<p:root xmlns:p="urn:producer" xmlns:rel="{R_NS}" xmlns:r="{R_NS}" rel:embed="rId11"><p:item xmlns:r="urn:shadow" r:link="rId2" rel:dm="rId13"/><p:item r:id="rId14" r:lo="rId14"/></p:root>"#
        )
        .into_bytes()
    );
}

#[test]
fn unmapped_and_non_numeric_relationship_values_are_unchanged() {
    let raw = format!(
        r#"<x:payload xmlns:x="urn:producer" xmlns:r="{R_NS}" r:id="rId1" r:embed="rIdABC" r:link="rId" r:dm="rId12x" id="rId2" x:id="rId3"/>"#
    );
    let map = HashMap::from([
        ("rId2".to_owned(), "rId20".to_owned()),
        ("rId3".to_owned(), "rId30".to_owned()),
        ("rIdABC".to_owned(), "rId40".to_owned()),
    ]);

    assert_eq!(
        rewrite_rel_ids(raw.as_bytes(), &map).unwrap(),
        raw.as_bytes()
    );
}

#[test]
fn exact_relationship_rewrite_changes_only_named_nonnumeric_ids() {
    let raw = format!(
        r#"<x:payload xmlns:x="urn:producer" xmlns:r="{R_NS}" r:id="slide-link" syntax=" A &amp; B "><x:other r:id='other-link'/><x:foreign x:id="slide-link"/></x:payload>"#
    );
    let map = HashMap::from([("slide-link".to_owned(), "rId9".to_owned())]);

    let rewritten = rewrite_exact_rel_ids(raw.as_bytes(), &map).unwrap();

    assert_eq!(
        rewritten,
        format!(
            r#"<x:payload xmlns:x="urn:producer" xmlns:r="{R_NS}" r:id="rId9" syntax=" A &amp; B "><x:other r:id='other-link'/><x:foreign x:id="slide-link"/></x:payload>"#
        )
        .into_bytes()
    );
}

#[test]
fn comments_processing_instructions_and_attribute_syntax_are_preserved() {
    let raw = format!(
        "<?producer keep?><x:payload  xmlns:x='urn:producer' xmlns:r=\"{R_NS}\"\n r:embed = 'rId1' x:keep=\"a &amp; b\"><!-- keep --><x:item r:link=\"rId2\" /></x:payload>"
    );
    let map = HashMap::from([
        ("rId1".to_owned(), "rId&amp;'\"".to_owned()),
        ("rId2".to_owned(), "rId22".to_owned()),
    ]);

    let rewritten = rewrite_rel_ids(raw.as_bytes(), &map).unwrap();

    assert_eq!(
        rewritten,
        format!(
            "<?producer keep?><x:payload  xmlns:x='urn:producer' xmlns:r=\"{R_NS}\"\n r:embed = 'rId&amp;amp;&apos;&quot;' x:keep=\"a &amp; b\"><!-- keep --><x:item r:link=\"rId22\" /></x:payload>"
        )
        .into_bytes()
    );
}

#[test]
fn malformed_preserved_xml_returns_an_error() {
    for raw in [
        br#"<x:payload xmlns:x="urn:producer">"#.as_slice(),
        br#"<x:payload xmlns:x="urn:producer"><x:item></x:payload>"#.as_slice(),
        br#"<x:payload xmlns:x="urn:producer" bad="unterminated/>"#.as_slice(),
    ] {
        assert!(rewrite_rel_ids(raw, &HashMap::new()).is_err());
    }
}

#[test]
fn every_corpus_preserved_payload_is_identity_with_an_empty_map() {
    let Some(corpus) = require_or_skip_corpus() else {
        return;
    };
    verify_fetched_corpus();
    let mut xml_parts = 0usize;
    for entry in manifest_entries() {
        let package = OpcPackage::open(corpus.join(entry.path)).unwrap();
        for (part, raw) in &package.parts {
            if !part.ends_with(".xml") {
                continue;
            }
            assert_eq!(
                rewrite_rel_ids(raw, &HashMap::new())
                    .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path)),
                *raw,
                "{} {part}",
                entry.path
            );
            xml_parts += 1;
        }
    }
    assert!(xml_parts > 0);
}

#[test]
fn visibility_and_header_footer_inputs_round_trip_in_schema_order() {
    let slide_xml = format!(
        r#"<q:sld xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer" showMasterSp="false" x:keep="slide"><q:cSld><q:spTree><q:nvGrpSpPr/><q:grpSpPr/></q:spTree></q:cSld><q:clrMapOvr><d:masterClrMapping/></q:clrMapOvr><q:extLst/></q:sld>"#
    );
    let layout_xml = format!(
        r#"<q:sldLayout xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer" showMasterSp="0" x:keep="layout"><q:cSld><q:spTree><q:nvGrpSpPr/><q:grpSpPr/></q:spTree></q:cSld><q:clrMapOvr><d:masterClrMapping/></q:clrMapOvr><x:beforeHf/><q:hf sldNum="true" hdr="false" ftr="0" dt="1" x:keep="hf"><x:child/></q:hf><q:timing/><q:transition/><q:extLst/></q:sldLayout>"#
    );
    let master_xml = format!(
        r#"<q:sldMaster xmlns:q="{P_NS}" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer"><q:cSld><q:spTree><q:nvGrpSpPr/><q:grpSpPr/></q:spTree></q:cSld><q:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/><q:sldLayoutIdLst/><q:timing/><q:hf sldNum="0" ftr="1" dt="false" x:keep="master"><x:child/></q:hf><x:afterHf/><q:txStyles><q:titleStyle/><q:bodyStyle/><q:otherStyle/></q:txStyles><q:extLst/></q:sldMaster>"#
    );

    let slide = CT_Slide::from_xml(slide_xml.as_bytes()).unwrap();
    let layout = CT_SlideLayout::from_xml(layout_xml.as_bytes()).unwrap();
    let master = CT_SlideMaster::from_xml(master_xml.as_bytes()).unwrap();

    assert_eq!(slide.show_master_shapes, Some(false));
    assert_eq!(layout.show_master_shapes, Some(false));
    let layout_hf = layout.header_footer.as_ref().unwrap();
    assert_eq!(layout_hf.slide_number, Some(true));
    assert_eq!(layout_hf.header, Some(false));
    assert_eq!(layout_hf.footer, Some(false));
    assert_eq!(layout_hf.date_time, Some(true));
    let master_hf = master.header_footer.as_ref().unwrap();
    assert_eq!(master_hf.slide_number, Some(false));
    assert_eq!(master_hf.footer, Some(true));
    assert_eq!(master_hf.date_time, Some(false));

    let written_slide = slide.to_xml().unwrap();
    let written_layout = layout.to_xml().unwrap();
    let written_master = master.to_xml().unwrap();
    let slide_text = std::str::from_utf8(&written_slide).unwrap();
    let layout_text = std::str::from_utf8(&written_layout).unwrap();
    let master_text = std::str::from_utf8(&written_master).unwrap();

    assert!(slide_text.starts_with("<p:sld "));
    assert!(slide_text.contains("showMasterSp=\"0\""));
    assert!(slide_text.contains("x:keep=\"slide\""));
    assert!(layout_text.contains(
        "<p:hf sldNum=\"1\" hdr=\"0\" ftr=\"0\" dt=\"1\" x:keep=\"hf\"><x:child/></p:hf>"
    ));
    assert!(
        master_text
            .contains("<p:hf sldNum=\"0\" ftr=\"1\" dt=\"0\" x:keep=\"master\"><x:child/></p:hf>")
    );
    assert!(!layout_text.contains("<q:hf"));
    assert!(!master_text.contains("<q:hf"));
    assert_order(
        &written_layout,
        &[
            "<p:clrMapOvr",
            "<x:beforeHf",
            "<p:hf",
            "<q:timing",
            "<q:transition",
            "<q:extLst",
        ],
    );
    assert_order(
        &written_master,
        &[
            "<p:clrMap",
            "<q:sldLayoutIdLst",
            "<q:timing",
            "<p:hf",
            "<x:afterHf",
            "<p:txStyles",
            "<q:extLst",
        ],
    );
    assert_eq!(slide, CT_Slide::from_xml(&written_slide).unwrap());
    assert_eq!(layout, CT_SlideLayout::from_xml(&written_layout).unwrap());
    assert_eq!(master, CT_SlideMaster::from_xml(&written_master).unwrap());
}

#[test]
fn all_corpus_slide_layout_and_master_parts_reparse_after_visibility_typing() {
    let Some(corpus) = require_or_skip_corpus() else {
        return;
    };
    verify_fetched_corpus();
    let mut decks = 0usize;
    let mut counts = [0usize; 3];
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
                    let _ = parsed.show_master_shapes;
                    assert_eq!(
                        parsed,
                        CT_Slide::from_xml(&parsed.to_xml().unwrap()).unwrap()
                    );
                    counts[0] += 1;
                }
                content_types::SLIDE_LAYOUT => {
                    let parsed = CT_SlideLayout::from_xml(xml)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path));
                    let _ = (parsed.show_master_shapes, parsed.header_footer.as_ref());
                    assert_eq!(
                        parsed,
                        CT_SlideLayout::from_xml(&parsed.to_xml().unwrap()).unwrap()
                    );
                    counts[1] += 1;
                }
                content_types::SLIDE_MASTER => {
                    let parsed = CT_SlideMaster::from_xml(xml)
                        .unwrap_or_else(|error| panic!("{} {part}: {error}", entry.path));
                    let _ = parsed.header_footer.as_ref();
                    assert_eq!(
                        parsed,
                        CT_SlideMaster::from_xml(&parsed.to_xml().unwrap()).unwrap()
                    );
                    counts[2] += 1;
                }
                _ => {}
            }
        }
        decks += 1;
    }
    assert_eq!(decks, EXPECTED_DECKS);
    assert!(
        counts.iter().all(|count| *count > 0),
        "part counts: {counts:?}"
    );
}
