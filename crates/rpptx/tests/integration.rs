use std::collections::{BTreeMap, HashSet};
use std::fmt::Write as _;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

use oxml_drawing::color::ColorMap;
use oxml_drawing::theme::CT_OfficeStyleSheet;
use oxml_opc::relationship::rel_types;
use oxml_opc::{OpcPackage, content_types};
use rpptx::{Angle, CT_LineProperties, Emu, Error, Fill, Presentation, ShapeKind, ShapeRef};
use rpptx_layout::{
    FlattenedItem, ResolveCtx, ResolvedContent, ResolvedSlide, ResolvedTextBody, ResolvedTextRun,
};
use rpptx_oxml::notes_parts::{CT_NotesMaster, CT_NotesSlide};
use rpptx_oxml::placeholder::PhType;
use rpptx_oxml::presentation::CT_Presentation;
use rpptx_oxml::shape_tree::ShapeTreeChild;
use rpptx_oxml::slide_parts::{CT_Slide, CT_SlideLayout, CT_SlideMaster, ColorMapOverrideKind};

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
const LAYOUT_PART: &str = "/custom/layouts/validation.xml";
const POWERPOINT_VERSION: &str = "16.104";
const POWERPOINT_BUILD: &str = "16.104.25121423";
const POWERPOINT_APP_BUILD: &str = "1214";
const MODELLED_CONTENT_TYPES: [&str; 7] = [
    content_types::PRESENTATION,
    content_types::SLIDE,
    content_types::SLIDE_LAYOUT,
    content_types::SLIDE_MASTER,
    content_types::NOTES_SLIDE,
    content_types::NOTES_MASTER,
    content_types::THEME,
];
const VISUAL_DECKS: [&str; 5] = [
    "WithMaster.pptx",
    "backgrounds.pptx",
    "placeholder-layout-color.pptx",
    "bug58144-headers-footers-2007.pptx",
    "60810.pptx",
];
const POWERPOINT_VISUAL_ACCEPTANCE: &str = r#"application	Microsoft PowerPoint 16.104	bundle 16.104.25121423
deck	/Users/atulsharma/Documents/projects/tensorbee/rdocx/corpus/pptx/WithMaster.pptx
deck	/Users/atulsharma/Documents/projects/tensorbee/rdocx/corpus/pptx/backgrounds.pptx
deck	/Users/atulsharma/Documents/projects/tensorbee/rdocx/corpus/pptx/placeholder-layout-color.pptx
deck	/Users/atulsharma/Documents/projects/tensorbee/rdocx/corpus/pptx/bug58144-headers-footers-2007.pptx
native_pdf	/private/tmp/F088-WithMaster.pdf
native_pdf	/private/tmp/F088-backgrounds.pdf
native_pdf	/private/tmp/F088-placeholder-layout-color.pdf
native_pdf	/private/tmp/F088-headers-footers.pdf
rendered_pages	/private/tmp/F088-pdf-render
normalized_evidence	/private/tmp/F-088-normalized-visual.txt
result	clean	opened and exported without repair, clipping, or prompt text
WithMaster	master artwork once, slide footer once on slide 2, no latent date or number
backgrounds	distinct solid, gradient, picture, and texture visuals
placeholder-layout-color	exact cyan on black, RGBA #00FFFF
bug58144-headers-footers-2007	Slide footer once
"#;

#[test]
fn shape_mutation_setters_survive_save_and_reload() {
    let mut presentation = Presentation::from_bytes(&mutation_fixture_bytes()).unwrap();
    let mut slide = presentation.slide_mut(0).unwrap();
    let mut shape = slide.shape_mut(0).unwrap();
    shape.set_position(Emu(101), Emu(202)).unwrap();
    shape.set_size(Emu(303), Emu(404)).unwrap();
    shape.set_rotation(Angle(5_400_000)).unwrap();
    shape.set_name("A & B \"quoted\"").unwrap();
    shape
        .set_fill(
            Fill::from_xml(
                br#"<a:noFill xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"/>"#,
            )
            .unwrap(),
        )
        .unwrap();
    shape
        .set_line(
            CT_LineProperties::from_xml(br#"<a:ln xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" w="12700"/>"#).unwrap(),
        )
        .unwrap();
    shape.set_adjust_value("adj", 25_000.0).unwrap();

    let bytes = presentation.to_bytes().unwrap();
    let reopened = Presentation::from_bytes(&bytes).unwrap();
    assert_eq!(
        reopened.slide(0).unwrap().shape(0).unwrap().kind(),
        ShapeKind::Shape
    );
    let package = open_opc(&bytes, "F-109 setter gate");
    let slide = CT_Slide::from_xml(package.get_part(SLIDE_TWO_PART).unwrap()).unwrap();
    let ShapeTreeChild::Shape(shape) = &slide.common_slide_data.shape_tree.children[0] else {
        panic!("expected ordinary shape");
    };
    let transform = shape.shape_properties.transform.as_ref().unwrap();
    assert_eq!(transform.offset.unwrap().x, Emu(101));
    assert_eq!(transform.offset.unwrap().y, Emu(202));
    assert_eq!(transform.extent.unwrap().cx, Emu(303));
    assert_eq!(transform.extent.unwrap().cy, Emu(404));
    assert_eq!(transform.rotation, Angle(5_400_000));
    assert!(shape.shape_properties.fill.is_some());
    assert_eq!(
        shape.shape_properties.line.as_ref().unwrap().width,
        Some(12_700)
    );
    let adjustments = shape
        .shape_properties
        .preset_geometry
        .as_ref()
        .unwrap()
        .adjust_values();
    assert_eq!(adjustments.len(), 1);
    assert_eq!(adjustments[0].name, "adj");
    assert_eq!(
        adjustments[0].args[0],
        oxml_drawing::geometry::GuideOperand::Literal(25_000.0)
    );
    let xml = String::from_utf8(package.get_part(SLIDE_TWO_PART).unwrap().to_vec()).unwrap();
    assert!(xml.contains("name=\"A &amp; B &quot;quoted&quot;\""));
}

#[test]
fn shape_mutation_preserves_unmodelled_xml_and_schema_order() {
    let mut presentation = Presentation::from_bytes(&mutation_fixture_bytes()).unwrap();
    let mut slide = presentation.slide_mut(0).unwrap();
    let mut shape = slide.shape_mut(0).unwrap();
    shape.set_position(Emu(1), Emu(2)).unwrap();
    shape
        .set_fill(
            Fill::from_xml(
                br#"<a:noFill xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"/>"#,
            )
            .unwrap(),
        )
        .unwrap();
    shape
        .set_line(
            CT_LineProperties::from_xml(
                br#"<a:ln xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"/>"#,
            )
            .unwrap(),
        )
        .unwrap();

    let bytes = presentation.to_bytes().unwrap();
    let package = open_opc(&bytes, "F-109 preservation");
    let xml = String::from_utf8(package.get_part(SLIDE_TWO_PART).unwrap().to_vec()).unwrap();
    assert!(xml.contains("producer=\"kept\""));
    assert!(xml.contains("<ext:nv xmlns:ext=\"urn:ext\">one &amp; two</ext:nv>"));
    assert!(xml.contains("<ext:before xmlns:ext=\"urn:ext\"/>"));
    assert!(xml.contains("<ext:after xmlns:ext=\"urn:ext\"/>"));
    let transform = xml.find("<a:xfrm").unwrap();
    let geometry = xml.find("<a:prstGeom").unwrap();
    let fill = xml.find("<a:noFill").unwrap();
    let line = xml.find("<a:ln").unwrap();
    assert!(transform < geometry && geometry < fill && fill < line);
    Presentation::from_bytes(&bytes).expect("schema-ordered output reparses");
}

#[test]
fn shape_mutation_handles_nested_group_children() {
    let mut presentation = Presentation::from_bytes(&fixture_bytes()).unwrap();
    let mut slide = presentation.slide_mut(0).unwrap();
    let mut group = slide.shape_mut(3).unwrap();
    assert_eq!(group.kind(), ShapeKind::Group);
    assert!(group.child_mut(2).is_none());
    let mut nested = group.child_mut(0).unwrap();
    nested.set_position(Emu(41), Emu(42)).unwrap();
    nested.set_name("nested changed").unwrap();

    let bytes = presentation.to_bytes().unwrap();
    let package = open_opc(&bytes, "F-109 nested group");
    let slide = CT_Slide::from_xml(package.get_part(SLIDE_TWO_PART).unwrap()).unwrap();
    let ShapeTreeChild::GroupShape(group) = &slide.common_slide_data.shape_tree.children[3] else {
        panic!("expected group");
    };
    assert_eq!(group.children.len(), 2);
    assert_eq!(
        group
            .children
            .iter()
            .map(ShapeTreeChild::non_visual_id)
            .collect::<Vec<_>>(),
        vec![Some(7), Some(8)]
    );
    let ShapeTreeChild::Shape(nested) = &group.children[0] else {
        panic!("expected nested shape");
    };
    assert_eq!(
        nested
            .shape_properties
            .transform
            .as_ref()
            .unwrap()
            .offset
            .unwrap()
            .x,
        Emu(41)
    );
    let ShapeTreeChild::Shape(sibling) = &group.children[1] else {
        panic!("expected sibling shape");
    };
    assert_eq!(nested.text_body.as_ref().unwrap().plain_text(), "nested");
    assert_eq!(sibling.text_body.as_ref().unwrap().plain_text(), "sibling");
    assert!(sibling.shape_properties.transform.is_none());
    let xml = String::from_utf8(package.get_part(SLIDE_TWO_PART).unwrap().to_vec()).unwrap();
    let changed = xml.find("id=\"7\" name=\"nested changed\"").unwrap();
    let unchanged = xml.find("id=\"8\" name=\"nested sibling\"").unwrap();
    assert!(changed < unchanged);
    assert_eq!(slide.common_slide_data.shape_tree.children.len(), 6);
}

#[test]
fn inserted_group_transform_precedes_preserved_group_properties() {
    let mut presentation = Presentation::from_bytes(&mutation_fixture_bytes()).unwrap();
    let mut slide = presentation.slide_mut(0).unwrap();
    let mut group = slide.shape_mut(3).unwrap();
    group.set_position(Emu(111), Emu(222)).unwrap();
    group.set_size(Emu(333), Emu(444)).unwrap();
    group.set_rotation(Angle(2_700_000)).unwrap();

    let bytes = presentation.to_bytes().unwrap();
    let package = open_opc(&bytes, "F-109 group transform insertion");
    let xml = String::from_utf8(package.get_part(SLIDE_TWO_PART).unwrap().to_vec()).unwrap();
    let group_start = xml.find("<p:grpSp>").unwrap();
    let transform = xml[group_start..].find("<a:xfrm").unwrap() + group_start;
    let preserved = xml[group_start..]
        .find("<a:solidFill><a:srgbClr val=\"112233\"/></a:solidFill>")
        .unwrap()
        + group_start;
    assert!(transform < preserved);

    let slide = CT_Slide::from_xml(package.get_part(SLIDE_TWO_PART).unwrap()).unwrap();
    let ShapeTreeChild::GroupShape(group) = &slide.common_slide_data.shape_tree.children[3] else {
        panic!("expected group");
    };
    let transform = group.group_transform().unwrap();
    assert_eq!(transform.offset.unwrap().x, Emu(111));
    assert_eq!(transform.offset.unwrap().y, Emu(222));
    assert_eq!(transform.extent.unwrap().cx, Emu(333));
    assert_eq!(transform.extent.unwrap().cy, Emu(444));
    assert_eq!(transform.rotation, Angle(2_700_000));
    let group_xml = String::from_utf8(group.to_xml().unwrap()).unwrap();
    assert!(group_xml.contains("<a:xfrm rot=\"2700000\">"));
    assert!(group_xml.contains("<a:solidFill><a:srgbClr val=\"112233\"/></a:solidFill>"));
}

#[test]
fn alternate_content_fallback_is_not_mutable() {
    let original = fixture_bytes();
    let original_package = open_opc(&original, "F-109 original AlternateContent");
    let original_slide =
        CT_Slide::from_xml(original_package.get_part(SLIDE_TWO_PART).unwrap()).unwrap();
    let ShapeTreeChild::AlternateContent(original_alternate) =
        &original_slide.common_slide_data.shape_tree.children[5]
    else {
        panic!("expected AlternateContent");
    };

    let mut presentation = Presentation::from_bytes(&original).unwrap();
    let mut slide = presentation.slide_mut(0).unwrap();
    let mut alternate = slide.shape_mut(5).unwrap();
    assert!(alternate.child_mut(0).is_none());
    assert!(matches!(
        alternate.set_position(Emu(1), Emu(2)),
        Err(Error::UnsupportedShapeMutation {
            shape_kind: ShapeKind::AlternateContent,
            ..
        })
    ));

    let bytes = presentation.to_bytes().unwrap();
    let saved_package = open_opc(&bytes, "F-109 saved AlternateContent");
    let saved_slide = CT_Slide::from_xml(saved_package.get_part(SLIDE_TWO_PART).unwrap()).unwrap();
    let ShapeTreeChild::AlternateContent(saved_alternate) =
        &saved_slide.common_slide_data.shape_tree.children[5]
    else {
        panic!("expected AlternateContent");
    };
    assert_eq!(saved_alternate.raw_xml(), original_alternate.raw_xml());
}

#[test]
fn shape_mutation_indices_and_kinds_are_total() {
    let mut presentation = Presentation::from_bytes(&fixture_bytes()).unwrap();
    assert!(presentation.slide_mut(2).is_none());
    let mut slide = presentation.slide_mut(0).unwrap();
    assert!(slide.shape_mut(6).is_none());
    for index in 0..5 {
        slide
            .shape_mut(index)
            .unwrap()
            .set_position(Emu(index as i64), Emu(index as i64 + 10))
            .unwrap();
    }
    let mut picture = slide.shape_mut(1).unwrap();
    assert!(picture.child_mut(0).is_none());
    picture
        .set_fill(
            Fill::from_xml(
                br#"<a:noFill xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"/>"#,
            )
            .unwrap(),
        )
        .unwrap();
    picture
        .set_line(
            CT_LineProperties::from_xml(
                br#"<a:ln xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"/>"#,
            )
            .unwrap(),
        )
        .unwrap();
    let mut frame = slide.shape_mut(2).unwrap();
    assert!(matches!(
        frame.set_fill(
            Fill::from_xml(
                br#"<a:noFill xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"/>"#
            )
            .unwrap()
        ),
        Err(Error::UnsupportedShapeMutation {
            shape_kind: ShapeKind::GraphicFrame,
            ..
        })
    ));
    let mut shape = slide.shape_mut(0).unwrap();
    assert!(matches!(
        shape.set_adjust_value("adj", f64::NAN),
        Err(Error::NonFiniteAdjustmentValue { .. })
    ));
    assert!(matches!(
        shape.set_adjust_value("adj", 1.0),
        Err(Error::UnsupportedShapeMutation {
            operation: "set adjustment",
            ..
        })
    ));
    Presentation::from_bytes(&presentation.to_bytes().unwrap())
        .expect("all supported transform variants reparse");
}

#[test]
fn shape_name_mutation_escapes_xml_and_preserves_children() {
    let mut presentation = Presentation::from_bytes(&mutation_fixture_bytes()).unwrap();
    let mut slide = presentation.slide_mut(0).unwrap();
    for index in 0..5 {
        slide
            .shape_mut(index)
            .unwrap()
            .set_name(&format!("A & B \"quoted\" {index}"))
            .unwrap();
    }

    let package = open_opc(&presentation.to_bytes().unwrap(), "F-109 name mutation");
    let xml = String::from_utf8(package.get_part(SLIDE_TWO_PART).unwrap().to_vec()).unwrap();
    for index in 0..5 {
        assert!(xml.contains(&format!("name=\"A &amp; B &quot;quoted&quot; {index}\"")));
    }
    assert!(xml.contains("<ext:nv xmlns:ext=\"urn:ext\">one &amp; two</ext:nv>"));
}

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
fn three_added_slides_have_unique_ids_and_reopen() {
    let mut presentation = Presentation::new().expect("open bundled template");
    assert!(presentation.layout_count() > 0);
    assert!(presentation.layout_name(0).is_some());
    for _ in 0..3 {
        presentation.add_slide(0).expect("add slide");
    }

    let ids = presentation
        .slides()
        .map(|slide| slide.id())
        .collect::<HashSet<_>>();
    assert_eq!(ids.len(), 3);
    assert!(ids.iter().all(|id| *id >= 256));
    let bytes = presentation.to_bytes().expect("serialize three-slide deck");
    assert_eq!(
        Presentation::from_bytes(&bytes)
            .expect("reopen three-slide deck")
            .len(),
        3
    );

    if let Some(path) = std::env::var_os("RPPTX_ACCEPTANCE_OUTPUT") {
        fs::write(path, bytes).expect("write native acceptance deck");
    }
}

#[test]
fn all_pinned_corpus_decks_validate_cleanly() {
    let Some(paths) = corpus_paths() else {
        return;
    };
    for path in paths {
        let deck = deck_name(&path);
        let bytes = fs::read(&path).unwrap_or_else(|error| panic!("{deck}: {error}"));
        let presentation = Presentation::from_bytes(&bytes)
            .unwrap_or_else(|error| panic!("{deck}: open presentation: {error}"));
        let issues = presentation.validate();
        assert!(issues.is_empty(), "{deck}: {issues:?}");
    }
}

#[test]
fn save_writes_the_same_bytes_as_to_bytes() {
    let presentation = Presentation::new().expect("open bundled template");
    let output = std::env::temp_dir().join(format!("rpptx-f108-save-{}.pptx", std::process::id()));
    presentation.save(&output).expect("save presentation");
    assert_eq!(
        fs::read(&output).expect("read saved deck"),
        presentation.to_bytes().unwrap()
    );
    fs::remove_file(output).expect("remove saved deck");
}

#[test]
#[ignore = "requires pinned Microsoft PowerPoint"]
fn three_added_slides_open_in_powerpoint_without_repair() {
    assert_powerpoint_build();
    let mut presentation = Presentation::new().expect("open bundled template");
    for _ in 0..3 {
        presentation.add_slide(0).expect("add slide");
    }
    let output = std::env::temp_dir().join(format!(
        "rpptx-f107-three-slides-{}.pptx",
        std::process::id()
    ));
    fs::write(&output, presentation.to_bytes().expect("serialize deck"))
        .expect("write native acceptance deck");
    let name = output.file_name().unwrap().to_string_lossy();
    let path = output.to_string_lossy();
    let script = format!(
        "with timeout of 120 seconds\ntell application \"Microsoft PowerPoint\"\nset previousStartUpDialog to start up dialog\ntry\nif (Version as text) is not \"{POWERPOINT_VERSION}\" then error \"PowerPoint version mismatch\"\nif (build as text) is not \"{POWERPOINT_APP_BUILD}\" then error \"PowerPoint build mismatch\"\nset start up dialog to false\nset deckPath to \"{path}\"\nopen my POSIX file deckPath\nset checkedDeck to presentation \"{name}\"\nif (count of slides of checkedDeck) is not 3 then error \"slide count mismatch\"\nclose checkedDeck saving no\nset start up dialog to previousStartUpDialog\non error errorMessage number errorNumber\ntry\nclose checkedDeck saving no\nend try\nset start up dialog to previousStartUpDialog\nerror errorMessage number errorNumber\nend try\nend tell\nend timeout\n"
    );
    let result = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .expect("launch PowerPoint acceptance script");
    fs::remove_file(&output).expect("remove native acceptance deck");
    assert!(
        result.status.success(),
        "PowerPoint no-repair open failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
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
    assert!(manifest.contains("default = [\"default-template\"]"));
    assert!(manifest.contains("default-template = []"));
}

#[test]
#[cfg(feature = "default-template")]
fn new_presentation_uses_the_bundled_zero_slide_template() {
    let presentation = Presentation::new().expect("open bundled presentation template");
    assert!(presentation.is_empty());

    let bytes = presentation.to_bytes().expect("serialize new presentation");
    if let Some(path) = std::env::var_os("RDOCX_F105_SAVE_PATH") {
        fs::write(&path, &bytes)
            .unwrap_or_else(|error| panic!("write {}: {error}", Path::new(&path).display()));
    }
    let reopened = Presentation::from_bytes(&bytes).expect("reopen new presentation");
    assert!(reopened.is_empty());
}

#[test]
#[cfg(feature = "default-template")]
fn bundled_template_has_the_documented_part_graph() {
    let asset = workspace_root().join("crates/rpptx/assets/default.pptx");
    let bytes = fs::read(&asset)
        .unwrap_or_else(|error| panic!("read bundled template {}: {error}", asset.display()));
    let package = open_opc(&bytes, "default.pptx");
    let presentation_part = package
        .main_document_part()
        .expect("bundled template main presentation part");
    let presentation = CT_Presentation::from_xml(
        package
            .get_part(&presentation_part)
            .expect("bundled template presentation XML"),
    )
    .expect("parse bundled template presentation XML");

    assert!(presentation.slide_ids.is_empty());
    assert_eq!(presentation.slide_master_ids.len(), 1);
    let slide_size = presentation
        .slide_size
        .expect("bundled template slide size");
    assert_eq!((slide_size.cx.0, slide_size.cy.0), (12_192_000, 6_858_000));

    let count = |content_type: &str| {
        package
            .content_types
            .overrides
            .values()
            .filter(|actual| actual.as_str() == content_type)
            .count()
    };
    assert_eq!(count(content_types::SLIDE_MASTER), 1);
    assert_eq!(count(content_types::SLIDE_LAYOUT), 11);
    assert!(count(content_types::THEME) >= 1);
    assert_eq!(count(content_types::PRES_PROPS), 1);
    assert_eq!(count(content_types::VIEW_PROPS), 1);
    assert_eq!(count(content_types::TABLE_STYLES), 1);
    assert_eq!(count(content_types::NOTES_MASTER), 1);
    assert_eq!(count(content_types::SLIDE), 0);

    for (part_name, content_type) in &package.content_types.overrides {
        if content_type == content_types::THEME {
            CT_OfficeStyleSheet::from_xml(
                package
                    .get_part(part_name)
                    .unwrap_or_else(|| panic!("missing bundled theme {part_name}")),
            )
            .unwrap_or_else(|error| panic!("parse bundled theme {part_name}: {error}"));
        }
    }

    let presentation_relationships = package
        .get_part_rels(&presentation_part)
        .expect("bundled template presentation relationships");
    for relationship_type in [
        rel_types::SLIDE_MASTER,
        rel_types::PRES_PROPS,
        rel_types::VIEW_PROPS,
        rel_types::TABLE_STYLES,
        rel_types::NOTES_MASTER,
    ] {
        assert!(
            presentation_relationships
                .get_by_type(relationship_type)
                .is_some(),
            "bundled template lacks relationship type {relationship_type}"
        );
    }

    let table_styles = package
        .parts
        .iter()
        .find(|(part_name, _)| {
            package.content_types.content_type_for(part_name) == Some(content_types::TABLE_STYLES)
        })
        .map(|(_, bytes)| bytes)
        .expect("bundled template table styles");
    assert!(
        table_styles
            .windows(b"{5C22544A-7EE6-4342-B048-85BDC9FD1C3A}".len())
            .any(|window| window == b"{5C22544A-7EE6-4342-B048-85BDC9FD1C3A}")
    );
}

#[test]
fn presentation_resolves_ordered_slides_and_notes() {
    let presentation = Presentation::from_bytes(&fixture_bytes()).unwrap();
    assert_eq!(presentation.len(), 2);
    assert!(!presentation.is_empty());
    let slides = presentation.slides().collect::<Vec<_>>();
    assert_eq!(slides[0].id(), 900);
    assert_eq!(slides[0].name(), Some("Second in storage, first in order"));
    assert_eq!(
        slides[0].text(),
        "plain\nA\tB\nC\tD\nnested\nsibling\nfallback"
    );
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
    assert_eq!(group.child_count(), 2);
    assert!(group.child(2).is_none());
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
        shapes[3].children().nth(1).unwrap().text().as_deref(),
        Some("sibling")
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

#[test]
fn resolved_visual_dump_matches_python_pptx_1_0_2() {
    let Some(decks) = resolve_visual_decks() else {
        return;
    };
    let rust = normalized_visual_oracle_records(&decks);
    let paths = VISUAL_DECKS
        .iter()
        .map(|name| corpus_dir().join(name))
        .collect::<Vec<_>>();
    let mut command = Command::new("uv");
    command.args([
        "run",
        "--with",
        "python-pptx==1.0.2",
        "python",
        "-c",
        PYTHON_VISUAL_ORACLE,
    ]);
    command.args(&paths);
    let output = command
        .output()
        .expect("run pinned visual python-pptx oracle");
    assert!(
        output.status.success(),
        "visual python-pptx oracle failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let python = String::from_utf8(output.stdout).unwrap();
    assert_eq!(rust, python, "normalized resolved visual records");

    let cyan_deck = decks
        .iter()
        .find(|deck| deck.name == "placeholder-layout-color.pptx")
        .expect("cyan placeholder deck");
    let cyan_fills = cyan_deck.slides[0]
        .resolved
        .shapes
        .iter()
        .filter(|shape| resolved_content_text(&shape.content) == "CYAN TEXT")
        .flat_map(|shape| resolved_run_fills(&shape.content))
        .collect::<Vec<_>>();
    assert_eq!(
        cyan_fills,
        ["Solid(Color { r: 0.0, g: 1.0, b: 1.0, a: 1.0 })"],
        "placeholder run must resolve to exact cyan RGBA"
    );

    let with_master = decks
        .iter()
        .find(|deck| deck.name == "WithMaster.pptx")
        .expect("latent-placeholder deck");
    let first_text = with_master.slides[0]
        .resolved
        .shapes
        .iter()
        .map(|shape| resolved_content_text(&shape.content))
        .collect::<Vec<_>>();
    let second_text = with_master.slides[1]
        .resolved
        .shapes
        .iter()
        .map(|shape| resolved_content_text(&shape.content))
        .collect::<Vec<_>>();
    for text in [&first_text, &second_text] {
        assert!(!text.iter().any(|value| value == "9/26/2011"));
        assert!(!text.iter().any(|value| value == "‹#›"));
    }
    assert_eq!(
        second_text
            .iter()
            .filter(|value| value.as_str() == "Footer from the master slide")
            .count(),
        1,
        "occupied slide footer must resolve exactly once"
    );

    let header_footer = decks
        .iter()
        .find(|deck| deck.name == "bug58144-headers-footers-2007.pptx")
        .expect("header-footer policy deck");
    let header_footer_text = header_footer.slides[0]
        .resolved
        .shapes
        .iter()
        .map(|shape| resolved_content_text(&shape.content))
        .collect::<Vec<_>>();
    assert_eq!(
        header_footer_text
            .iter()
            .filter(|value| value.as_str() == "Slide footer")
            .count(),
        1,
        "enabled slide footer must remain visible exactly once"
    );

    if let Some(path) = std::env::var_os("RDOCX_PPTX_VISUAL_EVIDENCE") {
        let path = PathBuf::from(path);
        fs::write(&path, normalized_resolved_evidence(&decks))
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        eprintln!("normalized visual evidence: {}", path.display());
    }
}

#[test]
fn visual_decks_resolve_in_expected_draw_order() {
    let Some(decks) = resolve_visual_decks() else {
        return;
    };
    let records = normalized_visual_oracle_records(&decks);
    assert!(records.contains("This text comes from the Master Slide"));
    assert!(records.contains("First page title"));
    let master_text = records
        .find("This text comes from the Master Slide")
        .unwrap();
    let slide_text = records.find("First page title").unwrap();
    assert!(
        master_text < slide_text,
        "layout artwork must precede slide content"
    );
}

#[test]
fn visual_decks_never_emit_template_prompt_text() {
    let Some(decks) = resolve_visual_decks() else {
        return;
    };
    let evidence = normalized_resolved_evidence(&decks);
    for prompt in ["Click to edit", "Click to add"] {
        assert!(
            !evidence.contains(prompt),
            "template prompt leaked: {prompt}"
        );
    }
}

#[test]
fn visual_decks_emit_each_master_logo_once() {
    let Some(decks) = resolve_visual_decks() else {
        return;
    };
    let deck = decks
        .iter()
        .find(|deck| deck.name == "60810.pptx")
        .expect("master-logo deck");
    for (slide_index, slide) in deck.slides.iter().enumerate() {
        let logos = slide
            .resolved
            .shapes
            .iter()
            .filter(|shape| {
                shape.unsupported == Some("image media pending relationship resolution")
                    && (shape.bounds.x - 611.751).abs() < 0.001
                    && (shape.bounds.y - 27.000).abs() < 0.001
                    && (shape.bounds.width - 101.732).abs() < 0.001
                    && (shape.bounds.height - 20.312).abs() < 0.001
            })
            .count();
        let expected = usize::from(!matches!(slide_index, 2 | 3));
        assert_eq!(
            logos, expected,
            "60810.pptx slide {slide_index} master logo"
        );
    }
}

#[test]
fn selected_visual_decks_reviewed_once_in_powerpoint() {
    assert!(POWERPOINT_VISUAL_ACCEPTANCE.contains("bundle 16.104.25121423"));
    assert_eq!(
        POWERPOINT_VISUAL_ACCEPTANCE
            .lines()
            .filter(|line| line.starts_with("deck\t"))
            .count(),
        4
    );
    assert!(POWERPOINT_VISUAL_ACCEPTANCE.contains("result\tclean"));
    assert!(POWERPOINT_VISUAL_ACCEPTANCE.contains("RGBA #00FFFF"));
    assert!(POWERPOINT_VISUAL_ACCEPTANCE.contains("slide footer once on slide 2"));
}

struct ResolvedVisualDeck {
    name: &'static str,
    slides: Vec<ResolvedVisualSlide>,
}

struct ResolvedVisualSlide {
    resolved: ResolvedSlide,
    shape_metadata: Vec<ResolvedVisualShapeMetadata>,
}

struct ResolvedVisualShapeMetadata {
    kind: &'static str,
    is_latent: bool,
}

fn resolve_visual_decks() -> Option<Vec<ResolvedVisualDeck>> {
    let corpus = corpus_dir();
    if !corpus.is_dir() {
        assert!(
            std::env::var_os("RDOCX_PPTX_CORPUS_REQUIRED").is_none(),
            "required visual corpus is missing at {}",
            corpus.display()
        );
        eprintln!(
            "visual differential skipped because {} is absent",
            corpus.display()
        );
        return None;
    }
    Some(resolve_visual_decks_from_corpus(&corpus))
}

fn resolve_visual_decks_from_corpus(corpus: &Path) -> Vec<ResolvedVisualDeck> {
    VISUAL_DECKS
        .iter()
        .map(|&name| {
            let path = corpus.join(name);
            assert!(
                path.is_file(),
                "missing selected visual deck {}",
                path.display()
            );
            let package = OpcPackage::open(&path)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            let presentation_part = package
                .main_document_part()
                .unwrap_or_else(|| panic!("{}: no presentation part", path.display()));
            let presentation = CT_Presentation::from_xml(visual_part(&package, &presentation_part))
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            let default_text_style = presentation.default_text_style.unwrap_or_default();
            let size = presentation.slide_size.map_or((720.0, 540.0), |size| {
                (size.cx.0 as f64 / 12_700.0, size.cy.0 as f64 / 12_700.0)
            });
            let presentation_rels = package
                .get_part_rels(&presentation_part)
                .unwrap_or_else(|| panic!("{}: no presentation relationships", path.display()));
            let slides = presentation
                .slide_ids
                .iter()
                .map(|slide_id| {
                    let slide_rel = presentation_rels
                        .get_by_id(&slide_id.relationship_id)
                        .unwrap_or_else(|| {
                            panic!(
                                "{}: missing slide relationship {}",
                                path.display(),
                                slide_id.relationship_id
                            )
                        });
                    assert_eq!(slide_rel.rel_type, rel_types::SLIDE, "{}", path.display());
                    let slide_part =
                        OpcPackage::resolve_rel_target(&presentation_part, &slide_rel.target);
                    let layout_part =
                        related_visual_part(&package, &slide_part, rel_types::SLIDE_LAYOUT, &path);
                    let master_part =
                        related_visual_part(&package, &layout_part, rel_types::SLIDE_MASTER, &path);
                    let theme_part =
                        related_visual_part(&package, &master_part, rel_types::THEME, &path);
                    let slide = CT_Slide::from_xml(visual_part(&package, &slide_part))
                        .unwrap_or_else(|error| panic!("{} {slide_part}: {error}", path.display()));
                    let layout = CT_SlideLayout::from_xml(visual_part(&package, &layout_part))
                        .unwrap_or_else(|error| {
                            panic!("{} {layout_part}: {error}", path.display())
                        });
                    let master = CT_SlideMaster::from_xml(visual_part(&package, &master_part))
                        .unwrap_or_else(|error| {
                            panic!("{} {master_part}: {error}", path.display())
                        });
                    let theme = CT_OfficeStyleSheet::from_xml(visual_part(&package, &theme_part))
                        .unwrap_or_else(|error| panic!("{} {theme_part}: {error}", path.display()));
                    let color_map = effective_visual_color_map(&master, &layout, &slide);
                    let context = ResolveCtx::new(
                        &theme,
                        color_map,
                        &master,
                        &layout,
                        &slide,
                        &default_text_style,
                    );
                    let shape_metadata = context
                        .flatten()
                        .into_iter()
                        .filter_map(|item| match item {
                            FlattenedItem::Background(_) => None,
                            FlattenedItem::Shape { child, .. } => {
                                flattened_child_has_bounds(&context, child).then(|| {
                                    ResolvedVisualShapeMetadata {
                                        kind: flattened_child_kind(child),
                                        is_latent: flattened_child_is_latent(child),
                                    }
                                })
                            }
                        })
                        .collect::<Vec<_>>();
                    let resolved = context
                        .resolve_slide(size)
                        .unwrap_or_else(|error| panic!("{} {slide_part}: {error}", path.display()));
                    assert_eq!(
                        shape_metadata.len(),
                        resolved.shapes.len(),
                        "{} {slide_part}: flattened and resolved shape counts",
                        path.display()
                    );
                    ResolvedVisualSlide {
                        resolved,
                        shape_metadata,
                    }
                })
                .collect();
            ResolvedVisualDeck { name, slides }
        })
        .collect()
}

fn flattened_child_kind(child: &ShapeTreeChild) -> &'static str {
    match child {
        ShapeTreeChild::Shape(_) => "Shape",
        ShapeTreeChild::Picture(_) => "Picture",
        ShapeTreeChild::GraphicFrame(_) => "GraphicFrame",
        ShapeTreeChild::Connector(_) => "Connector",
        ShapeTreeChild::GroupShape(_) => "Group",
        ShapeTreeChild::AlternateContent(_) => "AlternateContent",
    }
}

fn flattened_child_has_bounds(context: &ResolveCtx<'_>, child: &ShapeTreeChild) -> bool {
    match child {
        ShapeTreeChild::Shape(shape) => context
            .effective_xfrm(shape)
            .as_ref()
            .is_some_and(transform_has_bounds),
        ShapeTreeChild::Picture(picture) => picture
            .shape_properties
            .transform
            .as_ref()
            .is_some_and(transform_has_bounds),
        ShapeTreeChild::GraphicFrame(frame) => transform_has_bounds(&frame.transform),
        ShapeTreeChild::Connector(connector) => connector
            .shape_properties
            .transform
            .as_ref()
            .is_some_and(transform_has_bounds),
        ShapeTreeChild::GroupShape(_) | ShapeTreeChild::AlternateContent(_) => false,
    }
}

fn transform_has_bounds(transform: &oxml_drawing::xfrm::CT_Transform2D) -> bool {
    transform.offset.is_some() && transform.extent.is_some()
}

fn flattened_child_is_latent(child: &ShapeTreeChild) -> bool {
    let placeholder = match child {
        ShapeTreeChild::Shape(shape) => shape.placeholder.as_ref(),
        ShapeTreeChild::Picture(picture) => picture.placeholder.as_ref(),
        _ => None,
    };
    placeholder.is_some_and(|placeholder| {
        matches!(
            placeholder.effective_type(),
            PhType::DateTime | PhType::Footer | PhType::SlideNumber
        )
    })
}

fn visual_part<'a>(package: &'a OpcPackage, part_name: &str) -> &'a [u8] {
    package
        .get_part(part_name)
        .unwrap_or_else(|| panic!("missing package part {part_name}"))
}

fn related_visual_part(
    package: &OpcPackage,
    source_part: &str,
    relationship_type: &str,
    deck: &Path,
) -> String {
    let relationship = package
        .get_part_rels(source_part)
        .and_then(|relationships| relationships.get_by_type(relationship_type))
        .unwrap_or_else(|| {
            panic!(
                "{} {source_part}: missing relationship {relationship_type}",
                deck.display()
            )
        });
    OpcPackage::resolve_rel_target(source_part, &relationship.target)
}

fn effective_visual_color_map(
    master: &CT_SlideMaster,
    layout: &CT_SlideLayout,
    slide: &CT_Slide,
) -> ColorMap {
    for override_value in [
        slide.color_map_override.as_ref(),
        layout.color_map_override.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        if let ColorMapOverrideKind::Override(map) = &override_value.kind {
            return map.clone();
        }
    }
    master.color_map.clone()
}

fn normalized_visual_oracle_records(decks: &[ResolvedVisualDeck]) -> String {
    let mut output = String::new();
    for deck in decks {
        writeln!(output, "deck\t{}", deck.name).unwrap();
        for (slide_index, visual_slide) in deck.slides.iter().enumerate() {
            let slide = &visual_slide.resolved;
            writeln!(
                output,
                "slide\t{slide_index}\t{:.3}\t{:.3}",
                slide.size.0, slide.size.1
            )
            .unwrap();
            for (shape, metadata) in slide.shapes.iter().zip(&visual_slide.shape_metadata) {
                // python-pptx exposes header and footer placeholders as raw
                // collection members rather than applying the effective p:hf
                // policy. Their policy is asserted by the Rust regressions.
                if metadata.is_latent {
                    continue;
                }
                let bounds = shape.group_transform.transform_rect_bbox(shape.bounds);
                writeln!(
                    output,
                    "shape\t{slide_index}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{}",
                    metadata.kind,
                    bounds.x,
                    bounds.y,
                    bounds.width,
                    bounds.height,
                    escape_record(&resolved_content_text(&shape.content))
                )
                .unwrap();
            }
        }
    }
    output
}

fn normalized_resolved_evidence(decks: &[ResolvedVisualDeck]) -> String {
    let mut output = String::new();
    for deck in decks {
        writeln!(output, "deck\t{}", deck.name).unwrap();
        for (slide_index, visual_slide) in deck.slides.iter().enumerate() {
            let slide = &visual_slide.resolved;
            writeln!(
                output,
                "slide\t{slide_index}\tsize={:.3}x{:.3}\tbackground={:?}\tdiagnostics={:?}",
                slide.size.0, slide.size.1, slide.background, slide.diagnostics
            )
            .unwrap();
            for (shape_index, (shape, metadata)) in slide
                .shapes
                .iter()
                .zip(&visual_slide.shape_metadata)
                .enumerate()
            {
                writeln!(
                    output,
                    "shape\t{slide_index}.{shape_index}\tkind={}\tbounds={:.3},{:.3},{:.3},{:.3}\ttransform={:?}\tfill={:?}\tline={:?}\trun_fills={:?}\ttext={}\tunsupported={}",
                    metadata.kind,
                    shape.bounds.x,
                    shape.bounds.y,
                    shape.bounds.width,
                    shape.bounds.height,
                    shape.group_transform,
                    shape.fill,
                    shape.line,
                    resolved_run_fills(&shape.content),
                    escape_record(&resolved_content_text(&shape.content)),
                    shape.unsupported.unwrap_or("-")
                )
                .unwrap();
            }
        }
    }
    output
}

fn resolved_run_fills(content: &ResolvedContent) -> Vec<String> {
    let mut fills = Vec::new();
    let mut collect_body = |body: &ResolvedTextBody| {
        for paragraph in &body.paragraphs {
            for run in &paragraph.runs {
                let style = match run {
                    ResolvedTextRun::Text { style, .. } | ResolvedTextRun::Field { style, .. } => {
                        style
                    }
                    ResolvedTextRun::Break => continue,
                };
                if let Some(fill) = &style.fill {
                    fills.push(format!("{fill:?}"));
                }
            }
        }
    };
    match content {
        ResolvedContent::Text(body) => collect_body(body),
        ResolvedContent::Table(table) => {
            for body in table
                .rows
                .iter()
                .flat_map(|row| row.cells.iter())
                .filter_map(|cell| cell.text.as_ref())
            {
                collect_body(body);
            }
        }
        ResolvedContent::None | ResolvedContent::Image(_) => {}
    }
    fills
}

fn resolved_content_text(content: &ResolvedContent) -> String {
    match content {
        ResolvedContent::Text(body) => resolved_text(body),
        ResolvedContent::Table(table) => table
            .rows
            .iter()
            .map(|row| {
                row.cells
                    .iter()
                    .map(|cell| cell.text.as_ref().map(resolved_text).unwrap_or_default())
                    .collect::<Vec<_>>()
                    .join("\t")
            })
            .collect::<Vec<_>>()
            .join("\n"),
        ResolvedContent::None | ResolvedContent::Image(_) => String::new(),
    }
}

fn resolved_text(body: &ResolvedTextBody) -> String {
    body.paragraphs
        .iter()
        .map(|paragraph| {
            paragraph
                .runs
                .iter()
                .map(|run| match run {
                    ResolvedTextRun::Text { text, .. } | ResolvedTextRun::Field { text, .. } => {
                        text.as_str()
                    }
                    ResolvedTextRun::Break => "\n",
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_record(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
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

fn assert_powerpoint_build() {
    let app = "/Applications/Microsoft PowerPoint.app/Contents/Info.plist";
    let version = Command::new("/usr/libexec/PlistBuddy")
        .args(["-c", "Print :CFBundleShortVersionString", app])
        .output()
        .expect("read PowerPoint version");
    let build = Command::new("/usr/libexec/PlistBuddy")
        .args(["-c", "Print :CFBundleVersion", app])
        .output()
        .expect("read PowerPoint build");
    assert!(version.status.success());
    assert!(build.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap().trim(),
        POWERPOINT_VERSION
    );
    assert_eq!(
        String::from_utf8(build.stdout).unwrap().trim(),
        POWERPOINT_BUILD
    );
}

fn fixture_bytes() -> Vec<u8> {
    package_bytes(fixture_package())
}

fn mutation_fixture_bytes() -> Vec<u8> {
    let mut package = fixture_package();
    let original = String::from_utf8(package.get_part(SLIDE_TWO_PART).unwrap().to_vec()).unwrap();
    let with_non_visual = original.replacen(
        "<p:cNvPr/>",
        r#"<p:cNvPr id="2" name="old"><ext:nv xmlns:ext="urn:ext">one &amp; two</ext:nv></p:cNvPr>"#,
        1,
    );
    let with_shape_properties = with_non_visual.replacen(
        "<p:spPr/>",
        r#"<p:spPr producer="kept"><ext:before xmlns:ext="urn:ext"/><a:prstGeom prst="roundRect"><a:avLst/></a:prstGeom><ext:after xmlns:ext="urn:ext"/></p:spPr>"#,
        1,
    );
    let with_picture_name = with_shape_properties.replacen(
        "<p:pic><p:nvPicPr><p:cNvPr/>",
        "<p:pic><p:nvPicPr><p:cNvPr id=\"3\" name=\"picture\"/>",
        1,
    );
    let with_frame_name = with_picture_name.replacen(
        "<p:graphicFrame><p:nvGraphicFramePr/>",
        "<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id=\"4\" name=\"frame\"/></p:nvGraphicFramePr>",
        1,
    );
    let with_group_name = with_frame_name.replacen(
        "<p:grpSp><p:nvGrpSpPr/>",
        "<p:grpSp><p:nvGrpSpPr><p:cNvPr id=\"5\" name=\"group\"/></p:nvGrpSpPr>",
        1,
    );
    let with_group_property = with_group_name.replacen(
        "</p:nvGrpSpPr><p:grpSpPr/>",
        "</p:nvGrpSpPr><p:grpSpPr><a:solidFill><a:srgbClr val=\"112233\"/></a:solidFill></p:grpSpPr>",
        1,
    );
    let complete = with_group_property.replacen(
        "<p:cxnSp><p:nvCxnSpPr><p:cNvPr/>",
        "<p:cxnSp><p:nvCxnSpPr><p:cNvPr id=\"6\" name=\"connector\"/>",
        1,
    );
    assert_ne!(complete, original);
    package.set_part(SLIDE_TWO_PART, complete.into_bytes());
    package_bytes(package)
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
    package.set_part(LAYOUT_PART, simple_layout_xml());
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
    package
        .content_types
        .add_override(LAYOUT_PART, content_types::SLIDE_LAYOUT);
    package
        .content_types
        .add_default("bin", "application/octet-stream");
    let relationships = package.get_or_create_part_rels(PRESENTATION_PART);
    relationships.add_with_id("ordered-first", rel_types::SLIDE, "slides/second.xml");
    relationships.add_with_id("ordered-second", rel_types::SLIDE, "slides/first.xml");
    package.get_or_create_part_rels(SLIDE_TWO_PART).add_with_id(
        "notes-link",
        rel_types::NOTES_SLIDE,
        "../notes/speaker.xml",
    );
    package
        .get_or_create_part_rels(SLIDE_ONE_PART)
        .add(rel_types::SLIDE_LAYOUT, "../layouts/validation.xml");
    package
        .get_or_create_part_rels(SLIDE_TWO_PART)
        .add(rel_types::SLIDE_LAYOUT, "../layouts/validation.xml");
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

fn simple_layout_xml() -> Vec<u8> {
    format!(
        r#"<p:sldLayout xmlns:p="{P_NS}"><p:cSld><p:spTree><p:nvGrpSpPr/><p:grpSpPr/></p:spTree></p:cSld></p:sldLayout>"#
    )
    .into_bytes()
}

fn rich_slide_xml() -> Vec<u8> {
    let text_shape = |text: &str| {
        format!(
            r#"<p:sp><p:nvSpPr><p:cNvPr/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/><p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>{text}</a:t></a:r></a:p></p:txBody></p:sp>"#
        )
    };
    let identified_text_shape = |id: u32, name: &str, text: &str| {
        format!(
            r#"<p:sp><p:nvSpPr><p:cNvPr id="{id}" name="{name}"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/><p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>{text}</a:t></a:r></a:p></p:txBody></p:sp>"#
        )
    };
    let picture = r#"<p:pic><p:nvPicPr><p:cNvPr/><p:cNvPicPr/><p:nvPr/></p:nvPicPr><p:blipFill/><p:spPr/></p:pic>"#;
    let table = r#"<p:graphicFrame><p:nvGraphicFramePr/><p:xfrm/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/table"><a:tbl><a:tblGrid><a:gridCol w="100"/><a:gridCol w="100"/></a:tblGrid><a:tr h="100"><a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>A</a:t></a:r></a:p></a:txBody><a:tcPr/></a:tc><a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>B</a:t></a:r></a:p></a:txBody><a:tcPr/></a:tc></a:tr><a:tr h="100"><a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>C</a:t></a:r></a:p></a:txBody><a:tcPr/></a:tc><a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>D</a:t></a:r></a:p></a:txBody><a:tcPr/></a:tc></a:tr></a:tbl></a:graphicData></a:graphic></p:graphicFrame>"#;
    let group = format!(
        "<p:grpSp><p:nvGrpSpPr/><p:grpSpPr/>{}{}</p:grpSp>",
        identified_text_shape(7, "nested original", "nested"),
        identified_text_shape(8, "nested sibling", "sibling")
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

const PYTHON_VISUAL_ORACLE: &str = r#"
from pathlib import Path
import sys
import pptx
from pptx import Presentation
from pptx.enum.shapes import MSO_SHAPE_TYPE, PP_PLACEHOLDER

assert pptx.__version__ == "1.0.2", pptx.__version__

def escape(value):
    return value.replace("\\", "\\\\").replace("\t", "\\t").replace("\r", "\\r").replace("\n", "\\n")

def shape_text(shape):
    if getattr(shape, "has_table", False):
        return "\n".join("\t".join(cell.text.replace("\v", "\n") for cell in row.cells) for row in shape.table.rows)
    if getattr(shape, "has_text_frame", False):
        return shape.text.replace("\v", "\n")
    return ""

def shape_kind(shape):
    local = shape._element.tag.rsplit("}", 1)[-1]
    return {
        "sp": "Shape",
        "pic": "Picture",
        "graphicFrame": "GraphicFrame",
        "cxnSp": "Connector",
    }[local]

IDENTITY = (1.0, 0.0, 0.0, 1.0, 0.0, 0.0)

def then(first, second):
    a, b, c, d, e, f = first
    g, h, i, j, k, l = second
    return (
        g * a + i * b,
        h * a + j * b,
        g * c + i * d,
        h * c + j * d,
        g * e + i * f + k,
        h * e + j * f + l,
    )

def rotate_about(degrees, cx, cy):
    import math
    radians = math.radians(degrees)
    sine, cosine = math.sin(radians), math.cos(radians)
    return (
        cosine,
        sine,
        -sine,
        cosine,
        cx - cosine * cx + sine * cy,
        cy - sine * cx - cosine * cy,
    )

def group_mapping(group):
    xfrm = group._element.grpSpPr.xfrm
    width, height = group.width, group.height
    child_width, child_height = xfrm.chExt.cx, xfrm.chExt.cy
    scale_x = 1.0 if child_width == 0 else width / child_width
    scale_y = 1.0 if child_height == 0 else height / child_height
    mapping = (
        scale_x,
        0.0,
        0.0,
        scale_y,
        group.left - xfrm.chOff.x * scale_x,
        group.top - xfrm.chOff.y * scale_y,
    )
    center_x = group.left + width / 2.0
    center_y = group.top + height / 2.0
    mapping = then(mapping, rotate_about(group.rotation, center_x, center_y))
    if xfrm.flipH or xfrm.flipV:
        mapping = then(
            mapping,
            (
                -1.0 if xfrm.flipH else 1.0,
                0.0,
                0.0,
                -1.0 if xfrm.flipV else 1.0,
                2.0 * center_x if xfrm.flipH else 0.0,
                2.0 * center_y if xfrm.flipV else 0.0,
            ),
        )
    return mapping

def mapped_bounds(shape, mapping):
    a, b, c, d, e, f = mapping
    corners = (
        (shape.left, shape.top),
        (shape.left + shape.width, shape.top),
        (shape.left, shape.top + shape.height),
        (shape.left + shape.width, shape.top + shape.height),
    )
    points = tuple((a * x + c * y + e, b * x + d * y + f) for x, y in corners)
    xs, ys = tuple(point[0] for point in points), tuple(point[1] for point in points)
    return min(xs), min(ys), max(xs) - min(xs), max(ys) - min(ys)

def leaves(shapes, parent_mapping=IDENTITY):
    for shape in shapes:
        if shape.shape_type == MSO_SHAPE_TYPE.GROUP:
            mapping = then(group_mapping(shape), parent_mapping)
            yield from leaves(shape.shapes, mapping)
        elif not (
            shape.is_placeholder
            and shape.placeholder_format.type
            in (PP_PLACEHOLDER.DATE, PP_PLACEHOLDER.FOOTER, PP_PLACEHOLDER.SLIDE_NUMBER)
        ):
            yield shape, parent_mapping

def visible_shapes(slide):
    layout = slide.slide_layout
    master = layout.slide_master
    if layout._element.get("showMasterSp") != "0":
        yield from leaves(shape for shape in master.shapes if not shape.is_placeholder)
    if slide._element.get("showMasterSp") != "0":
        yield from leaves(shape for shape in layout.shapes if not shape.is_placeholder)
    yield from leaves(slide.shapes)

for filename in sys.argv[1:]:
    path = Path(filename)
    presentation = Presentation(path)
    print("deck\t" + path.name)
    for slide_index, slide in enumerate(presentation.slides):
        print(f"slide\t{slide_index}\t{presentation.slide_width / 12700:.3f}\t{presentation.slide_height / 12700:.3f}")
        for shape, mapping in visible_shapes(slide):
            bounds = mapped_bounds(shape, mapping)
            if any(value is None for value in bounds):
                continue
            print(
                f"shape\t{slide_index}\t{shape_kind(shape)}\t{bounds[0] / 12700:.3f}\t{bounds[1] / 12700:.3f}\t"
                f"{bounds[2] / 12700:.3f}\t{bounds[3] / 12700:.3f}\t{escape(shape_text(shape))}"
            )
"#;
