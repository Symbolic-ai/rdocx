use std::collections::{BTreeMap, HashSet};
use std::fmt::Write as _;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use oxml_drawing::color::ColorMap;
use oxml_drawing::theme::CT_OfficeStyleSheet;
use oxml_layout::{PositionedElement, walk};
use oxml_opc::relationship::rel_types;
use oxml_opc::{OpcPackage, content_types};
use rpptx::{
    Angle, CT_LineProperties, CT_TextCharacterProperties, CT_TextParagraphProperties, ChartData,
    ChartKind, ConnectorType, Emu, Error, Fill, Presentation, ShapeKind, ShapeRef, TextBullet,
    TextBulletCharacter, TextBulletChoice, TextFont,
};
use rpptx_chart::{AxisData, CT_ChartSpace};
use rpptx_layout::{
    FlattenedItem, ResolveCtx, ResolvedContent, ResolvedSlide, ResolvedTextBody, ResolvedTextRun,
};
use rpptx_oxml::graphic_frame::GraphicDataPayload;
use rpptx_oxml::notes_parts::{CT_NotesMaster, CT_NotesSlide};
use rpptx_oxml::placeholder::PhType;
use rpptx_oxml::presentation::CT_Presentation;
use rpptx_oxml::shape_tree::ShapeTreeChild;
use rpptx_oxml::slide_parts::{
    BackgroundRendering, CT_Slide, CT_SlideLayout, CT_SlideMaster, ColorMapOverrideKind,
};

#[path = "../examples/dump_deck.rs"]
mod dump_deck;
#[allow(dead_code)]
#[path = "../examples/render_deck.rs"]
mod render_deck_example;

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
const KEYNOTE_VERSION: &str = "14.4";
const KEYNOTE_BUILD: &str = "7043.0.93";
const LIBREOFFICE_VERSION: &str = "LibreOffice 26.2.5.2 cd7284b4cbbfeb507e630c1aac019f4157393acb";
const F116_CANDIDATE_PATH: &str = "/private/tmp/rdocx-f116-m11-write-api.pptx";
const F124_CANDIDATE_PATH: &str = "/private/tmp/rdocx-f124-add-chart.pptx";
const F124_ARTIFACT_SHA256: &str =
    "e6e9f7eef1c774d0414c5d0c3f1202da1a28635b5d089e15455b7adc3f66cb00";
const F116_ARTIFACT_SHA256: &str =
    "d36da6e8849eabd4487d2572baea19c3716ee7d0fe03aaa4714a28ce3c41de4f";
const F116_FINAL_TITLES: [&str; 10] = [
    "F-116 slide 10",
    "F-116 slide 02",
    "F-116 slide 03",
    "F-116 slide 04",
    "F-116 slide 05",
    "F-116 slide 06",
    "F-116 slide 07",
    "F-116 slide 08",
    "F-116 slide 08",
    "F-116 slide 09",
];
static F116_TEMP_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn f124_chart_data() -> ChartData {
    ChartData {
        categories: vec!["North".to_owned(), "South".to_owned(), "West".to_owned()],
        series: vec![
            ("Revenue".to_owned(), vec![12.5, 19.0, 14.25]),
            ("Cost".to_owned(), vec![8.0, 11.5, 9.75]),
        ],
        number_format: Some("0.00".to_owned()),
    }
}

fn presentation_with_authored_chart() -> Presentation {
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add chart slide");
    presentation
        .add_chart(
            0,
            ChartKind::Bar,
            Emu(914_400),
            Emu(914_400),
            Emu(7_315_200),
            Emu(4_572_000),
            &f124_chart_data(),
        )
        .expect("add authored chart");
    presentation
}

fn override_color_map(accent1: &str) -> String {
    format!(
        r#"<a:overrideClrMapping bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="{accent1}" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>"#
    )
}

fn authored_chart_with_color_maps(slide_mapping: Option<&str>, layout_mapping: &str) -> Vec<u8> {
    let bytes = presentation_with_authored_chart().to_bytes().unwrap();
    let mut package = open_opc(&bytes, "authored chart color maps");
    let slide_part = package
        .content_types
        .overrides
        .iter()
        .find_map(|(part, content_type)| {
            (content_type == content_types::SLIDE).then_some(part.clone())
        })
        .expect("slide content type override");
    let layout_part = {
        let relationship = package
            .get_part_rels(&slide_part)
            .and_then(|relationships| relationships.get_by_type(rel_types::SLIDE_LAYOUT))
            .expect("slide layout relationship");
        OpcPackage::resolve_rel_target(&slide_part, &relationship.target)
    };
    let master_mapping = "<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>";

    let slide_xml = String::from_utf8(package.get_part(&slide_part).unwrap().to_vec()).unwrap();
    assert!(slide_xml.contains(master_mapping));
    let slide_xml = match slide_mapping {
        Some(mapping) => slide_xml.replace(
            master_mapping,
            &format!("<p:clrMapOvr>{mapping}</p:clrMapOvr>"),
        ),
        None => slide_xml.replace(master_mapping, ""),
    };
    package.set_part(&slide_part, slide_xml.into_bytes());

    let layout_xml = String::from_utf8(package.get_part(&layout_part).unwrap().to_vec()).unwrap();
    assert!(layout_xml.contains(master_mapping));
    let layout_xml = layout_xml.replace(
        master_mapping,
        &format!("<p:clrMapOvr>{layout_mapping}</p:clrMapOvr>"),
    );
    package.set_part(&layout_part, layout_xml.into_bytes());

    write_package(&package, "authored chart color maps", "write color maps")
}

#[test]
fn add_chart_writes_complete_relationship_graph() {
    let bytes = presentation_with_authored_chart()
        .to_bytes()
        .expect("serialize authored chart");
    let package = open_opc(&bytes, "authored chart graph");
    let chart_part = package
        .content_types
        .overrides
        .iter()
        .find_map(|(part, content_type)| {
            (content_type == content_types::CHART).then_some(part.as_str())
        })
        .expect("chart content type override");
    let workbook_part = package
        .content_types
        .overrides
        .iter()
        .find_map(|(part, content_type)| {
            (content_type == content_types::EMBEDDED_WORKBOOK).then_some(part.as_str())
        })
        .expect("workbook content type override");
    let slide_part = package
        .content_types
        .overrides
        .iter()
        .find_map(|(part, content_type)| {
            (content_type == content_types::SLIDE).then_some(part.as_str())
        })
        .expect("slide content type override");
    let chart_relationship = package
        .get_part_rels(slide_part)
        .expect("slide relationships")
        .get_by_type(rel_types::CHART)
        .expect("slide to chart relationship");
    assert_eq!(
        OpcPackage::resolve_rel_target(slide_part, &chart_relationship.target),
        chart_part
    );
    let workbook_relationship = package
        .get_part_rels(chart_part)
        .expect("chart relationships")
        .get_by_type(rel_types::PACKAGE)
        .expect("chart to workbook relationship");
    assert_eq!(
        OpcPackage::resolve_rel_target(chart_part, &workbook_relationship.target),
        workbook_part
    );
    let slide_xml = String::from_utf8(package.get_part(slide_part).unwrap().to_vec()).unwrap();
    assert!(slide_xml.contains(&format!(r#"<c:chart xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:r="{R_NS}" r:id="{}"/>"#, chart_relationship.id)));
    let chart_xml = String::from_utf8(package.get_part(chart_part).unwrap().to_vec()).unwrap();
    assert!(chart_xml.contains(&format!(
        r#"<c:externalData r:id="{}">"#,
        workbook_relationship.id
    )));
    assert!(package.get_part(workbook_part).is_some());
    assert!(
        Presentation::from_bytes(&bytes)
            .unwrap()
            .validate()
            .is_empty()
    );
}

#[test]
fn authored_chart_honors_slide_layout_and_master_color_maps() {
    let baseline = deterministic_render(&presentation_with_authored_chart().to_bytes().unwrap());
    let layout_mapping = override_color_map("accent6");
    let slide_mapping = override_color_map("accent5");

    let master = deterministic_render(&authored_chart_with_color_maps(
        Some("<a:masterClrMapping/>"),
        &layout_mapping,
    ));
    let layout = deterministic_render(&authored_chart_with_color_maps(None, &layout_mapping));
    let slide = deterministic_render(&authored_chart_with_color_maps(
        Some(&slide_mapping),
        &layout_mapping,
    ));

    assert_eq!(master, baseline);
    assert_ne!(layout, baseline);
    assert_ne!(slide, baseline);
    assert_ne!(slide, layout);
}

#[test]
fn add_chart_uses_collision_free_part_numbers() {
    let mut source = Presentation::new().unwrap();
    source.add_slide(0).unwrap();
    let mut package = open_opc(&source.to_bytes().unwrap(), "sparse chart fixture");
    package.set_part("/ppt/charts/chart2.xml", b"occupied chart".to_vec());
    package.set_part(
        "/ppt/embeddings/Workbook7.xlsx",
        b"occupied workbook".to_vec(),
    );
    package
        .content_types
        .add_override("/ppt/charts/chart2.xml", content_types::CHART);
    package.content_types.add_override(
        "/ppt/embeddings/Workbook7.xlsx",
        content_types::EMBEDDED_WORKBOOK,
    );
    let mut bytes = Cursor::new(Vec::new());
    package.write_to(&mut bytes).unwrap();
    let mut presentation = Presentation::from_bytes(&bytes.into_inner()).unwrap();
    presentation
        .add_chart(
            0,
            ChartKind::Line,
            Emu(1),
            Emu(2),
            Emu(3),
            Emu(4),
            &f124_chart_data(),
        )
        .unwrap();
    let package = open_opc(&presentation.to_bytes().unwrap(), "numbered chart");
    assert!(package.parts.contains_key("/ppt/charts/chart3.xml"));
    assert!(package.parts.contains_key("/ppt/embeddings/Workbook8.xlsx"));
    assert_eq!(
        package.get_part("/ppt/charts/chart2.xml"),
        Some(b"occupied chart".as_slice())
    );
}

#[test]
fn add_chart_caches_and_workbook_share_one_source() {
    let bytes = presentation_with_authored_chart().to_bytes().unwrap();
    let package = open_opc(&bytes, "authored chart cache");
    let chart_part = package
        .content_types
        .overrides
        .iter()
        .find_map(|(part, content_type)| {
            (content_type == content_types::CHART).then_some(part.as_str())
        })
        .unwrap();
    let chart = CT_ChartSpace::from_xml(package.get_part(chart_part).unwrap()).unwrap();
    let series = chart.chart.plot_area.series().unwrap();
    assert_eq!(series.len(), 2);
    let expected_series = [
        (
            "Sheet1!$B$1",
            "Revenue",
            "Sheet1!$B$2:$B$4",
            [12.5, 19.0, 14.25],
        ),
        ("Sheet1!$C$1", "Cost", "Sheet1!$C$2:$C$4", [8.0, 11.5, 9.75]),
    ];
    for (actual, (name_formula, name, value_formula, values)) in series.iter().zip(expected_series)
    {
        let actual_name = actual.name.as_ref().unwrap();
        assert_eq!(actual_name.formula, name_formula);
        assert_eq!(actual_name.values, [name]);
        let AxisData::String(categories) = actual.categories.as_ref().unwrap() else {
            panic!("category series must use a string cache");
        };
        assert_eq!(categories.formula, "Sheet1!$A$2:$A$4");
        assert_eq!(categories.values, ["North", "South", "West"]);
        assert_eq!(actual.values.formula, value_formula);
        assert_eq!(actual.values.format_code, "0.00");
        assert_eq!(actual.values.values, values);
    }
    let workbook_relationship = package
        .get_part_rels(chart_part)
        .unwrap()
        .get_by_type(rel_types::PACKAGE)
        .unwrap();
    let workbook_part = OpcPackage::resolve_rel_target(chart_part, &workbook_relationship.target);
    let workbook =
        OpcPackage::from_reader(Cursor::new(package.get_part(&workbook_part).unwrap())).unwrap();
    let worksheet = String::from_utf8(
        workbook
            .get_part("/xl/worksheets/sheet1.xml")
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    let shared_strings =
        String::from_utf8(workbook.get_part("/xl/sharedStrings.xml").unwrap().to_vec()).unwrap();
    let shared_strings = shared_string_values(&shared_strings);
    let expected_cells = BTreeMap::from([
        ("A1", "Category"),
        ("A2", "North"),
        ("A3", "South"),
        ("A4", "West"),
        ("B1", "Revenue"),
        ("B2", "12.5"),
        ("B3", "19"),
        ("B4", "14.25"),
        ("C1", "Cost"),
        ("C2", "8"),
        ("C3", "11.5"),
        ("C4", "9.75"),
    ]);
    let actual_cells = expected_cells
        .keys()
        .map(|address| {
            let (shared, value) = worksheet_cell(&worksheet, address);
            let value = if shared {
                shared_strings[value.parse::<usize>().unwrap()].as_str()
            } else {
                value
            };
            (*address, value)
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(actual_cells, expected_cells);
}

#[test]
fn add_chart_rejects_invalid_data_without_mutation() {
    let invalid = [
        ChartData {
            categories: Vec::new(),
            series: vec![("Revenue".to_owned(), Vec::new())],
            number_format: None,
        },
        ChartData {
            categories: vec!["North".to_owned()],
            series: Vec::new(),
            number_format: None,
        },
        ChartData {
            categories: vec!["North".to_owned(), "South".to_owned()],
            series: vec![("Revenue".to_owned(), vec![12.5])],
            number_format: None,
        },
        ChartData {
            categories: vec!["North".to_owned()],
            series: vec![("Revenue".to_owned(), vec![f64::NAN])],
            number_format: None,
        },
        ChartData {
            categories: vec!["North".to_owned()],
            series: vec![("Revenue".to_owned(), vec![12.5])],
            number_format: Some(String::new()),
        },
    ];
    for data in invalid {
        let mut presentation = Presentation::new().unwrap();
        presentation.add_slide(0).unwrap();
        let before = presentation.to_bytes().unwrap();
        assert!(
            presentation
                .add_chart(0, ChartKind::Bar, Emu(1), Emu(2), Emu(3), Emu(4), &data,)
                .is_err()
        );
        assert_eq!(presentation.to_bytes().unwrap(), before);
    }

    let mut presentation = Presentation::new().unwrap();
    presentation.add_slide(0).unwrap();
    let before = presentation.to_bytes().unwrap();
    assert!(
        presentation
            .add_chart(
                0,
                ChartKind::Pie,
                Emu(1),
                Emu(2),
                Emu(3),
                Emu(4),
                &f124_chart_data(),
            )
            .is_err()
    );
    assert_eq!(presentation.to_bytes().unwrap(), before);

    for (width, height) in [
        (Emu(0), Emu(1)),
        (Emu(-1), Emu(1)),
        (Emu(1), Emu(0)),
        (Emu(1), Emu(-1)),
    ] {
        assert!(
            presentation
                .add_chart(
                    0,
                    ChartKind::Bar,
                    Emu(1),
                    Emu(2),
                    width,
                    height,
                    &f124_chart_data(),
                )
                .is_err()
        );
        assert_eq!(presentation.to_bytes().unwrap(), before);
    }

    let scatter = ChartData {
        categories: vec!["not numeric".to_owned()],
        series: vec![("Revenue".to_owned(), vec![12.5])],
        number_format: None,
    };
    assert!(
        presentation
            .add_chart(
                0,
                ChartKind::Scatter,
                Emu(1),
                Emu(2),
                Emu(3),
                Emu(4),
                &scatter,
            )
            .is_err()
    );
    assert_eq!(presentation.to_bytes().unwrap(), before);
}

#[test]
fn add_chart_authors_each_supported_family() {
    let one_series = ChartData {
        categories: vec!["1".to_owned(), "2".to_owned(), "3".to_owned()],
        series: vec![("Revenue".to_owned(), vec![12.5, 19.0, 14.25])],
        number_format: None,
    };
    for (kind, plot_element) in [
        (ChartKind::Bar, "c:barChart"),
        (ChartKind::Line, "c:lineChart"),
        (ChartKind::Pie, "c:pieChart"),
        (ChartKind::Doughnut, "c:doughnutChart"),
        (ChartKind::Area, "c:areaChart"),
        (ChartKind::Scatter, "c:scatterChart"),
        (ChartKind::Radar, "c:radarChart"),
    ] {
        let mut presentation = Presentation::new().unwrap();
        presentation.add_slide(0).unwrap();
        presentation
            .add_chart(0, kind, Emu(1), Emu(2), Emu(3), Emu(4), &one_series)
            .unwrap();
        let bytes = presentation.to_bytes().unwrap();
        let package = open_opc(&bytes, "supported authored chart");
        let chart = package
            .content_types
            .overrides
            .iter()
            .find_map(|(part, content_type)| {
                (content_type == content_types::CHART).then(|| package.get_part(part).unwrap())
            })
            .unwrap();
        let chart = String::from_utf8(chart.to_vec()).unwrap();
        assert!(
            chart.contains(&format!("<{plot_element}>")),
            "{kind:?}: {chart}"
        );
        assert!(
            Presentation::from_bytes(&bytes)
                .unwrap()
                .validate()
                .is_empty()
        );
    }
}

#[test]
fn authored_chart_relationship_enters_presentation_renderer() {
    let bytes = presentation_with_authored_chart()
        .to_bytes()
        .expect("serialize authored chart for rendering");
    let rendered = render_presentation_package(&bytes);

    let mut paths = 0;
    let mut text_runs = 0;
    walk(
        &rendered.pages[0].elements,
        &mut |element, _| match element {
            PositionedElement::Path(path) => {
                let path_bounds = path.path.bounds().expect("nonempty authored chart path");
                assert!(path_bounds.x.is_finite());
                assert!(path_bounds.y.is_finite());
                assert!(path_bounds.width.is_finite());
                assert!(path_bounds.height.is_finite());
                paths += 1;
            }
            PositionedElement::Text(run) => {
                assert!(run.origin.x.is_finite());
                assert!(run.origin.y.is_finite());
                assert!(run.font_size.is_finite());
                assert!(run.advances.iter().all(|advance| advance.is_finite()));
                assert!(!run.text.is_empty());
                text_runs += 1;
            }
            PositionedElement::Line {
                start, end, width, ..
            } => {
                assert!(start.x.is_finite());
                assert!(start.y.is_finite());
                assert!(end.x.is_finite());
                assert!(end.y.is_finite());
                assert!(width.is_finite());
            }
            _ => {}
        },
    );
    assert!(paths >= 6, "expected authored bars and annotation paths");
    assert!(text_runs >= 6, "expected authored axis and legend labels");
}

#[test]
fn supported_and_fallback_charts_render_deterministically() {
    let supported = presentation_with_authored_chart().to_bytes().unwrap();
    let preview = valid_one_pixel_png();
    let fallback =
        presentation_with_three_dimensional_chart_fallback(Some((&preview, "image/png")));
    let missing = presentation_with_three_dimensional_chart_fallback(None);
    let malformed =
        presentation_with_three_dimensional_chart_fallback(Some((b"not a PNG", "image/png")));
    let mut corrupt_png = valid_one_pixel_png();
    corrupt_png[41] = 0;
    let corrupt =
        presentation_with_three_dimensional_chart_fallback(Some((&corrupt_png, "image/png")));
    let mismatched =
        presentation_with_three_dimensional_chart_fallback(Some((&preview, "image/jpeg")));
    let jpeg = valid_template_jpeg();
    let jpeg_fallback =
        presentation_with_three_dimensional_chart_fallback(Some((&jpeg, "image/jpeg")));
    let sof = jpeg
        .windows(2)
        .position(|window| matches!(window, [0xff, 0xc0] | [0xff, 0xc2]))
        .unwrap();
    let sof_length = usize::from(u16::from_be_bytes([jpeg[sof + 2], jpeg[sof + 3]]));
    let corrupt_jpeg = jpeg[..sof + 2 + sof_length].to_vec();
    assert!(oxml_media::probe(&corrupt_jpeg).is_some());
    let corrupt_jpeg_fallback =
        presentation_with_three_dimensional_chart_fallback(Some((&corrupt_jpeg, "image/jpeg")));
    let mismatched_jpeg =
        presentation_with_three_dimensional_chart_fallback(Some((&jpeg, "image/png")));
    let inflation_bomb = png_inflation_bomb();
    assert!(oxml_media::probe(&inflation_bomb).is_some());
    let inflation_bomb_fallback =
        presentation_with_three_dimensional_chart_fallback(Some((&inflation_bomb, "image/png")));
    let oversized = png_header(u32::MAX, u32::MAX);
    let oversized_fallback =
        presentation_with_three_dimensional_chart_fallback(Some((&oversized, "image/png")));
    let sparse = sparse_preview_png();
    let sparse_fallback =
        presentation_with_three_dimensional_chart_fallback(Some((&sparse, "image/png")));
    let grayscale_jpeg = valid_grayscale_jpeg();
    assert_eq!(oxml_media::probe(&grayscale_jpeg).unwrap().channels, 1);
    let grayscale_jpeg_fallback =
        presentation_with_three_dimensional_chart_fallback(Some((&grayscale_jpeg, "image/jpeg")));
    let cmyk_jpeg = valid_cmyk_jpeg();
    assert_eq!(oxml_media::probe(&cmyk_jpeg).unwrap().channels, 4);
    let cmyk_jpeg_fallback =
        presentation_with_three_dimensional_chart_fallback(Some((&cmyk_jpeg, "image/jpeg")));

    for bytes in [
        &supported,
        &fallback,
        &missing,
        &malformed,
        &corrupt,
        &mismatched,
        &jpeg_fallback,
        &corrupt_jpeg_fallback,
        &mismatched_jpeg,
        &inflation_bomb_fallback,
        &oversized_fallback,
        &sparse_fallback,
        &grayscale_jpeg_fallback,
        &cmyk_jpeg_fallback,
    ] {
        let first = render_presentation_package(bytes);
        let second = render_presentation_package(bytes);
        assert_eq!(format!("{first:?}"), format!("{second:?}"));
        assert_eq!(
            oxml_pdf::render_page_to_png(&first, 0, 96.0).unwrap(),
            oxml_pdf::render_page_to_png(&second, 0, 96.0).unwrap()
        );
    }

    let rendered_fallback = render_presentation_package(&fallback);
    let mut cached_images = Vec::new();
    walk(&rendered_fallback.pages[0].elements, &mut |element, _| {
        if let PositionedElement::Image { data, .. } = element {
            cached_images.push(data.clone());
        }
    });
    assert_eq!(cached_images, vec![preview]);

    let rendered_jpeg = render_presentation_package(&jpeg_fallback);
    let mut cached_jpegs = Vec::new();
    walk(&rendered_jpeg.pages[0].elements, &mut |element, _| {
        if let PositionedElement::Image { data, .. } = element {
            cached_jpegs.push(data.clone());
        }
    });
    assert_eq!(cached_jpegs, vec![jpeg]);

    let rendered_sparse = render_presentation_package(&sparse_fallback);
    let mut cached_sparse_images = Vec::new();
    walk(&rendered_sparse.pages[0].elements, &mut |element, _| {
        if let PositionedElement::Image { data, .. } = element {
            cached_sparse_images.push(data.clone());
        }
    });
    assert_eq!(cached_sparse_images, vec![sparse]);

    for bytes in [
        &missing,
        &malformed,
        &corrupt,
        &mismatched,
        &corrupt_jpeg_fallback,
        &mismatched_jpeg,
        &inflation_bomb_fallback,
        &oversized_fallback,
        &grayscale_jpeg_fallback,
        &cmyk_jpeg_fallback,
    ] {
        let rendered = render_presentation_package(bytes);
        let mut labels = Vec::new();
        walk(&rendered.pages[0].elements, &mut |element, _| {
            if let PositionedElement::Text(run) = element {
                labels.push(run.text.clone());
            }
        });
        assert!(labels.iter().any(|label| label == "Unsupported chart"));
    }
}

#[test]
#[ignore = "manual PowerPoint open and Edit Data acceptance"]
fn created_chart_opens_and_edit_data_matches_source() {
    let bytes = presentation_with_authored_chart().to_bytes().unwrap();
    fs::write(F124_CANDIDATE_PATH, &bytes).unwrap();
    let sha = sha256(Path::new(F124_CANDIDATE_PATH));
    eprintln!("F-124 PowerPoint candidate: {F124_CANDIDATE_PATH}, SHA-256 {sha}");
    assert_eq!(sha, F124_ARTIFACT_SHA256);
    assert!(open_opc(&bytes, "F-124 candidate").parts.len() > 1);
}

fn shared_string_values(xml: &str) -> Vec<String> {
    xml.split("<t>")
        .skip(1)
        .map(|tail| tail.split_once("</t>").unwrap().0.to_owned())
        .collect()
}

fn worksheet_cell<'a>(xml: &'a str, address: &str) -> (bool, &'a str) {
    let marker = format!(r#"<c r="{address}""#);
    let cell = xml
        .split_once(&marker)
        .unwrap_or_else(|| panic!("missing worksheet cell {address}"))
        .1;
    let (attributes, cell) = cell.split_once('>').unwrap();
    let cell = cell.split_once("</c>").unwrap().0;
    let value = cell
        .split_once("<v>")
        .and_then(|(_, tail)| tail.split_once("</v>"))
        .map(|(value, _)| value)
        .unwrap_or_else(|| panic!("missing worksheet value {address}"));
    (attributes.contains(r#"t="s""#), value)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CrossViewerObservation {
    Clean {
        opened_or_imported: bool,
        observed_slide_count: usize,
        no_repair_or_conversion_error: bool,
        export_or_close_result: &'static str,
    },
    Pending {
        reason: &'static str,
    },
}

#[derive(Clone, Copy, Debug)]
struct CrossViewerEvidence {
    application: &'static str,
    version_or_date: Option<&'static str>,
    build: Option<&'static str>,
    input_sha256: &'static str,
    observation: CrossViewerObservation,
}

const CROSS_VIEWER_ACCEPTANCE: [CrossViewerEvidence; 4] = [
    CrossViewerEvidence {
        application: "Microsoft PowerPoint",
        version_or_date: Some(POWERPOINT_VERSION),
        build: Some("Info.plist 16.104.25121423, AppleScript 1214"),
        input_sha256: F116_ARTIFACT_SHA256,
        observation: CrossViewerObservation::Clean {
            opened_or_imported: true,
            observed_slide_count: 10,
            no_repair_or_conversion_error: true,
            export_or_close_result: "closed without saving",
        },
    },
    CrossViewerEvidence {
        application: "Apple Keynote",
        version_or_date: Some(KEYNOTE_VERSION),
        build: Some(KEYNOTE_BUILD),
        input_sha256: F116_ARTIFACT_SHA256,
        observation: CrossViewerObservation::Clean {
            opened_or_imported: true,
            observed_slide_count: 10,
            no_repair_or_conversion_error: true,
            export_or_close_result: "user confirmed clean open and closed the presentation",
        },
    },
    CrossViewerEvidence {
        application: "Google Slides",
        version_or_date: Some("accepted 2026-08-09"),
        build: Some("Google Chrome 151.0.7922.76, build 7922.76"),
        input_sha256: F116_ARTIFACT_SHA256,
        observation: CrossViewerObservation::Clean {
            opened_or_imported: true,
            observed_slide_count: 10,
            no_repair_or_conversion_error: true,
            export_or_close_result: "Microsoft PowerPoint download started once",
        },
    },
    CrossViewerEvidence {
        application: "LibreOffice Impress",
        version_or_date: Some("26.2.5.2"),
        build: Some("cd7284b4cbbfeb507e630c1aac019f4157393acb"),
        input_sha256: F116_ARTIFACT_SHA256,
        observation: CrossViewerObservation::Clean {
            opened_or_imported: true,
            observed_slide_count: 10,
            no_repair_or_conversion_error: true,
            export_or_close_result: "hidden-slide PDF export succeeded with ten pages",
        },
    },
];

#[test]
fn ten_slide_write_api_deck_validates_and_reopens() {
    let presentation = build_f116_ten_slide_deck();
    assert_f116_deck_structure(&presentation);
    let bytes = presentation.to_bytes().expect("serialize F-116 deck");
    let candidate_path = f116_temp_path("validation", "pptx");
    fs::write(&candidate_path, &bytes).expect("write temporary F-116 candidate");
    let candidate_sha = sha256(&candidate_path);
    fs::remove_file(&candidate_path).expect("remove temporary F-116 candidate");
    eprintln!("F-116 candidate SHA-256: {candidate_sha}");
    assert_eq!(candidate_sha, F116_ARTIFACT_SHA256);

    let reopened = Presentation::from_bytes(&bytes).expect("reopen F-116 deck");
    assert_eq!(reopened.len(), 10);
    assert_f116_deck_structure(&reopened);

    let package = open_opc(&bytes, "F-116 ten-slide candidate");
    let presentation_part = package.main_document_part().unwrap();
    let slide_relationships = package
        .get_part_rels(&presentation_part)
        .unwrap()
        .get_all_by_type(rel_types::SLIDE);
    assert_eq!(slide_relationships.len(), 10);
    let image_scopes = package
        .part_rels
        .values()
        .filter(|relationships| {
            relationships
                .items
                .iter()
                .any(|relationship| relationship.rel_type == rel_types::IMAGE)
        })
        .count();
    assert_eq!(image_scopes, 3);
    assert_eq!(
        package
            .parts
            .keys()
            .filter(|part| part.starts_with("/ppt/media/"))
            .count(),
        1
    );
}

#[test]
fn ten_slide_write_api_deck_saves_as_presentation_and_show() {
    let presentation = build_f116_ten_slide_deck();
    assert!(presentation.validate().is_empty());
    let ordinary_path = f116_temp_path("ordinary-save", "pptx");
    presentation
        .save(&ordinary_path)
        .expect("save F-116 presentation");
    let ordinary_bytes = fs::read(&ordinary_path).expect("read F-116 presentation");
    fs::remove_file(&ordinary_path).expect("remove temporary F-116 presentation");
    let ordinary = open_opc(&ordinary_bytes, "F-116 presentation package");
    let ordinary_main = ordinary.main_document_part().unwrap();
    assert_eq!(
        ordinary.content_types.content_type_for(&ordinary_main),
        Some(content_types::PRESENTATION)
    );

    let show_path = f116_temp_path("slideshow-save", "ppsx");
    presentation
        .save_as_show(&show_path)
        .expect("save F-116 slideshow");
    let show_bytes = fs::read(&show_path).expect("read F-116 slideshow");
    fs::remove_file(&show_path).expect("remove temporary F-116 slideshow");
    let show = open_opc(&show_bytes, "F-116 slideshow package");
    let show_main = show.main_document_part().unwrap();
    assert_eq!(
        show.content_types.content_type_for(&show_main),
        Some(content_types::SLIDESHOW)
    );
    let reopened = Presentation::from_bytes(&show_bytes).expect("reopen F-116 slideshow");
    assert_f116_deck_structure(&reopened);
}

#[test]
#[ignore = "requires pinned PowerPoint, Keynote, Google Slides, and LibreOffice"]
fn generated_ten_slide_write_api_deck_opens_clean_in_all_four_viewers() {
    let path = write_f116_candidate();
    assert_eq!(sha256(&path), F116_ARTIFACT_SHA256);
    assert_f116_powerpoint_acceptance(&path);
    assert_f116_libreoffice_acceptance(&path);
    for row in CROSS_VIEWER_ACCEPTANCE {
        assert_clean_cross_viewer_evidence(row).unwrap_or_else(|error| panic!("{error}"));
    }
}

#[test]
fn cross_viewer_acceptance_evidence_is_complete_and_bound_to_one_artifact() {
    assert_eq!(CROSS_VIEWER_ACCEPTANCE.len(), 4);
    assert_eq!(
        CROSS_VIEWER_ACCEPTANCE
            .iter()
            .map(|row| row.application)
            .collect::<HashSet<_>>(),
        HashSet::from([
            "Microsoft PowerPoint",
            "Apple Keynote",
            "Google Slides",
            "LibreOffice Impress",
        ])
    );
    let candidate = write_f116_temporary_candidate("evidence");
    let actual_sha = sha256(&candidate);
    fs::remove_file(&candidate).expect("remove temporary F-116 evidence candidate");
    assert_eq!(actual_sha, F116_ARTIFACT_SHA256);
    for row in CROSS_VIEWER_ACCEPTANCE {
        assert_eq!(row.input_sha256, F116_ARTIFACT_SHA256);
        if matches!(row.observation, CrossViewerObservation::Clean { .. }) {
            assert_clean_cross_viewer_evidence(row).unwrap();
        }
    }
    let pending = CROSS_VIEWER_ACCEPTANCE
        .iter()
        .filter_map(|row| match row.observation {
            CrossViewerObservation::Pending { reason } => {
                assert!(!reason.is_empty());
                Some(row.application)
            }
            CrossViewerObservation::Clean { .. } => None,
        })
        .collect::<Vec<_>>();
    assert!(pending.is_empty(), "pending viewer evidence: {pending:?}");
}

#[test]
fn pending_cross_viewer_evidence_cannot_pass_as_clean() {
    let pending = CrossViewerEvidence {
        application: "pending regression fixture",
        version_or_date: Some("observed"),
        build: Some("observed"),
        input_sha256: F116_ARTIFACT_SHA256,
        observation: CrossViewerObservation::Pending {
            reason: "acceptance operation remains incomplete",
        },
    };
    assert!(
        assert_clean_cross_viewer_evidence(pending).is_err(),
        "pending evidence passed as clean"
    );
}

#[test]
fn clean_cross_viewer_evidence_requires_each_positive_operation() {
    let clean = CrossViewerEvidence {
        application: "regression fixture",
        version_or_date: Some("observed"),
        build: Some("observed"),
        input_sha256: F116_ARTIFACT_SHA256,
        observation: CrossViewerObservation::Clean {
            opened_or_imported: true,
            observed_slide_count: 10,
            no_repair_or_conversion_error: true,
            export_or_close_result: "closed or exported successfully",
        },
    };
    assert!(assert_clean_cross_viewer_evidence(clean).is_ok());

    for incomplete in [
        CrossViewerObservation::Clean {
            opened_or_imported: false,
            observed_slide_count: 10,
            no_repair_or_conversion_error: true,
            export_or_close_result: "closed or exported successfully",
        },
        CrossViewerObservation::Clean {
            opened_or_imported: true,
            observed_slide_count: 0,
            no_repair_or_conversion_error: true,
            export_or_close_result: "closed or exported successfully",
        },
        CrossViewerObservation::Clean {
            opened_or_imported: true,
            observed_slide_count: 10,
            no_repair_or_conversion_error: false,
            export_or_close_result: "closed or exported successfully",
        },
        CrossViewerObservation::Clean {
            opened_or_imported: true,
            observed_slide_count: 10,
            no_repair_or_conversion_error: true,
            export_or_close_result: "",
        },
        CrossViewerObservation::Clean {
            opened_or_imported: true,
            observed_slide_count: 10,
            no_repair_or_conversion_error: true,
            export_or_close_result: " \t\n",
        },
    ] {
        assert!(
            assert_clean_cross_viewer_evidence(CrossViewerEvidence {
                observation: incomplete,
                ..clean
            })
            .is_err()
        );
    }
}

#[test]
fn clean_cross_viewer_evidence_rejects_missing_or_blank_metadata() {
    let clean = CrossViewerEvidence {
        application: "regression fixture",
        version_or_date: Some("observed"),
        build: Some("observed"),
        input_sha256: F116_ARTIFACT_SHA256,
        observation: CrossViewerObservation::Clean {
            opened_or_imported: true,
            observed_slide_count: 10,
            no_repair_or_conversion_error: true,
            export_or_close_result: "closed or exported successfully",
        },
    };

    for version_or_date in [None, Some(""), Some(" \t\n")] {
        assert!(
            assert_clean_cross_viewer_evidence(CrossViewerEvidence {
                version_or_date,
                ..clean
            })
            .is_err()
        );
    }
    for build in [None, Some(""), Some(" \t\n")] {
        assert!(
            assert_clean_cross_viewer_evidence(CrossViewerEvidence { build, ..clean }).is_err()
        );
    }
}

#[test]
fn slide_and_presentation_properties_round_trip() {
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add slide");
    presentation
        .set_slide_size(Emu(10_000_001), Emu(5_000_003))
        .expect("set slide size");
    let fill = Fill::from_xml(br#"<a:solidFill><a:srgbClr val="345678"/></a:solidFill>"#).unwrap();
    let mut slide = presentation.slide_mut(0).unwrap();
    slide.set_hidden(true);
    slide.set_background(fill).expect("set direct background");
    presentation.core_properties_mut().title = Some("F-115 properties".to_owned());

    let bytes = presentation.to_bytes().expect("serialize properties");
    let package = open_opc(&bytes, "property round trip");
    let slide_part = package
        .content_types
        .overrides
        .iter()
        .find_map(|(part_name, content_type)| {
            (content_type == content_types::SLIDE).then_some(part_name)
        })
        .unwrap();
    let slide = CT_Slide::from_xml(package.get_part(slide_part).unwrap()).unwrap();
    let BackgroundRendering::Properties(Some(background_fill)) = slide
        .common_slide_data
        .background
        .as_ref()
        .unwrap()
        .rendering()
    else {
        panic!("expected a direct slide background fill");
    };
    assert_eq!(
        background_fill.to_xml().unwrap(),
        br#"<a:solidFill><a:srgbClr val="345678"/></a:solidFill>"#
    );
    let reopened = Presentation::from_bytes(&bytes).expect("reopen properties");

    assert_eq!(
        reopened.slide_size(),
        Some((Emu(10_000_001), Emu(5_000_003)))
    );
    assert!(reopened.slide(0).unwrap().hidden());
    assert!(reopened.slide(0).unwrap().has_explicit_background());
    assert_eq!(
        reopened
            .core_properties()
            .and_then(|props| props.title.as_deref()),
        Some("F-115 properties")
    );
}

#[test]
fn core_properties_are_loaded_lazily_and_written_with_valid_graph() {
    const CORE_PART: &str = "/metadata/nonstandard-core.xml";
    let core_xml = br#"<?xml version="1.0"?><cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title> Original &amp; exact </dc:title><!--producer--></cp:coreProperties>"#;
    let mut package = fixture_package();
    package.set_part(CORE_PART, core_xml.to_vec());
    package
        .content_types
        .add_override(CORE_PART, content_types::CORE_PROPERTIES);
    package.package_rels.add_with_id(
        "core-link",
        rel_types::CORE_PROPERTIES,
        "metadata/nonstandard-core.xml",
    );
    let mut presentation = Presentation::from_bytes(&package_bytes(package)).unwrap();

    assert_eq!(
        presentation
            .core_properties()
            .and_then(|props| props.title.as_deref()),
        Some(" Original & exact ")
    );
    let immutable = open_opc(
        &presentation.to_bytes().unwrap(),
        "immutable core properties",
    );
    assert_eq!(immutable.get_part(CORE_PART).unwrap(), core_xml);

    presentation.core_properties_mut().creator = Some("F-115".to_owned());
    let mutated = open_opc(&presentation.to_bytes().unwrap(), "mutated core properties");
    assert_eq!(
        mutated.content_types.content_type_for(CORE_PART),
        Some(content_types::CORE_PROPERTIES)
    );
    let relationship = mutated
        .package_rels
        .get_by_type(rel_types::CORE_PROPERTIES)
        .unwrap();
    assert_eq!(
        OpcPackage::resolve_rel_target("/", &relationship.target),
        CORE_PART
    );
    assert_eq!(
        oxml_core::core_properties::CoreProperties::from_xml(mutated.get_part(CORE_PART).unwrap())
            .unwrap()
            .creator
            .as_deref(),
        Some("F-115")
    );

    let mut absent = Presentation::from_bytes(&package_bytes(fixture_package())).unwrap();
    assert!(absent.core_properties().is_none());
    absent.core_properties_mut().subject = Some("created".to_owned());
    let created = open_opc(&absent.to_bytes().unwrap(), "created core properties");
    let relationship = created
        .package_rels
        .get_by_type(rel_types::CORE_PROPERTIES)
        .unwrap();
    let part = OpcPackage::resolve_rel_target("/", &relationship.target);
    assert_eq!(part, "/docProps/core.xml");
    assert_eq!(
        created.content_types.content_type_for(&part),
        Some(content_types::CORE_PROPERTIES)
    );
    let created_xml = created.get_part(&part).expect("created core part");
    assert_eq!(
        oxml_core::core_properties::CoreProperties::from_xml(created_xml)
            .unwrap()
            .subject
            .as_deref(),
        Some("created")
    );
}

#[test]
fn occupied_conventional_core_part_is_not_overwritten_without_relationship() {
    const OCCUPIED_CORE_PART: &str = "/docProps/core.xml";
    let occupied = b"producer-owned bytes outside the core relationship graph";
    let mut package = fixture_package();
    package.set_part(OCCUPIED_CORE_PART, occupied.to_vec());
    let source_bytes = package_bytes(package);
    let mut presentation = Presentation::from_bytes(&source_bytes).unwrap();
    presentation.core_properties_mut().subject = Some("must not overwrite".to_owned());

    let error = presentation.to_bytes().unwrap_err();
    assert!(matches!(
        error,
        Error::CorePropertiesPartCollision { ref part_name }
            if part_name == OCCUPIED_CORE_PART
    ));

    let output = std::env::temp_dir().join(format!(
        "rpptx-f115-core-collision-{}.pptx",
        std::process::id()
    ));
    fs::write(&output, b"existing output").unwrap();
    assert!(presentation.save(&output).is_err());
    assert_eq!(fs::read(&output).unwrap(), b"existing output");
    fs::remove_file(&output).unwrap();

    let source = open_opc(&source_bytes, "occupied core source");
    assert_eq!(source.get_part(OCCUPIED_CORE_PART).unwrap(), occupied);
    assert!(
        source
            .package_rels
            .get_by_type(rel_types::CORE_PROPERTIES)
            .is_none()
    );
}

#[test]
fn save_as_show_changes_only_the_main_content_type() {
    let presentation = Presentation::from_bytes(&package_bytes(fixture_package())).unwrap();
    let ordinary_bytes = presentation.to_bytes().unwrap();
    let output = std::env::temp_dir().join(format!("rpptx-f115-show-{}.ppsx", std::process::id()));
    presentation.save_as_show(&output).expect("save slideshow");
    let show_bytes = fs::read(&output).expect("read slideshow");
    fs::remove_file(&output).expect("remove slideshow");

    let ordinary = open_opc(&ordinary_bytes, "ordinary presentation");
    let show = open_opc(&show_bytes, "slideshow presentation");
    assert_eq!(
        show.content_types.content_type_for(PRESENTATION_PART),
        Some(content_types::SLIDESHOW)
    );
    assert_eq!(ordinary.parts, show.parts);
    assert_eq!(ordinary.package_rels.items, show.package_rels.items);
    assert_eq!(ordinary.part_rels.len(), show.part_rels.len());
    for (part_name, relationships) in &ordinary.part_rels {
        assert_eq!(
            relationships.items,
            show.part_rels.get(part_name).unwrap().items,
            "{part_name}: relationship scope"
        );
    }
    let mut ordinary_overrides = ordinary.content_types.overrides;
    ordinary_overrides.insert(
        PRESENTATION_PART.to_owned(),
        content_types::SLIDESHOW.to_owned(),
    );
    assert_eq!(ordinary_overrides, show.content_types.overrides);
    assert_eq!(ordinary.content_types.defaults, show.content_types.defaults);
    assert_eq!(presentation.to_bytes().unwrap(), ordinary_bytes);
    assert_eq!(
        Presentation::from_bytes(&show_bytes).unwrap().len(),
        presentation.len()
    );
}

#[test]
fn invalid_slide_size_does_not_mutate_the_presentation() {
    let mut presentation = Presentation::new().expect("open bundled template");
    let before = presentation.to_bytes().unwrap();

    assert!(presentation.set_slide_size(Emu(0), Emu(5_000_000)).is_err());
    assert!(
        presentation
            .set_slide_size(Emu(10_000_000), Emu(-1))
            .is_err()
    );

    assert_eq!(presentation.to_bytes().unwrap(), before);
}

#[test]
fn invalid_slide_collection_indices_do_not_mutate_the_presentation() {
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add slide");
    let before = presentation.to_bytes().expect("serialize baseline");

    assert!(presentation.remove_slide(1).is_err());
    assert!(presentation.move_slide(1, 0).is_err());
    assert!(presentation.move_slide(0, 1).is_err());
    assert!(presentation.duplicate_slide(1).is_err());
    assert_eq!(presentation.to_bytes().unwrap(), before);
}

#[test]
fn move_slide_reorders_the_slide_id_list_without_rewriting_relationships() {
    let mut package = fixture_package();
    let marked = String::from_utf8(package.get_part(PRESENTATION_PART).unwrap().to_vec())
        .unwrap()
        .replacen(
            r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships""#,
            r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:x="urn:f114""#,
            1,
        )
        .replacen(
            r#"r:id="ordered-first"/>"#,
            r#"r:id="ordered-first"/><x:between producer="kept"/>"#,
            1,
        );
    package.set_part(PRESENTATION_PART, marked.into_bytes());
    let before_relationships = package
        .get_part_rels(PRESENTATION_PART)
        .unwrap()
        .items
        .clone();
    let mut presentation = Presentation::from_bytes(&package_bytes(package)).unwrap();
    let before = presentation
        .slides()
        .map(|slide| slide.id())
        .collect::<Vec<_>>();

    presentation.move_slide(0, 1).expect("move first slide");

    let after = presentation
        .slides()
        .map(|slide| slide.id())
        .collect::<Vec<_>>();
    assert_eq!(after, vec![before[1], before[0]]);
    let bytes = presentation.to_bytes().unwrap();
    let package = open_opc(&bytes, "moved slide package");
    assert_eq!(
        package.get_part_rels(PRESENTATION_PART).unwrap().items,
        before_relationships
    );
    let presentation_xml =
        String::from_utf8(package.get_part(PRESENTATION_PART).unwrap().to_vec()).unwrap();
    assert!(
        presentation_xml
            .find("<x:between producer=\"kept\"/>")
            .unwrap()
            < presentation_xml.find(r#"r:id="ordered-second""#).unwrap()
    );
    let reopened = Presentation::from_bytes(&bytes).unwrap();
    assert_eq!(
        reopened
            .slides()
            .map(|slide| slide.id())
            .collect::<Vec<_>>(),
        after
    );
}

#[test]
fn remove_slide_removes_its_part_relationship_notes_and_custom_show_entries() {
    let mut package = fixture_package();
    let with_custom_show = String::from_utf8(package.get_part(PRESENTATION_PART).unwrap().to_vec())
        .unwrap()
        .replacen(
            "<p:notesSz",
            r#"<p:custShowLst xmlns:x="urn:f114" x:producer="kept"><p:custShow name="review" id="1"><p:sldLst><p:sld r:id="ordered-first"/><x:marker/><p:sld r:id="ordered-second"/></p:sldLst></p:custShow></p:custShowLst><p:notesSz"#,
            1,
        );
    package.set_part(PRESENTATION_PART, with_custom_show.into_bytes());
    let mut presentation = Presentation::from_bytes(&package_bytes(package)).unwrap();
    assert!(presentation.slide(0).unwrap().notes_text().is_some());

    presentation
        .remove_slide(0)
        .expect("remove slide with notes");

    let bytes = presentation.to_bytes().unwrap();
    let package = open_opc(&bytes, "removed slide package");
    assert!(!package.parts.contains_key(SLIDE_TWO_PART));
    assert!(!package.parts.contains_key(NOTES_PART));
    assert!(!package.part_rels.contains_key(SLIDE_TWO_PART));
    assert!(!package.part_rels.contains_key(NOTES_PART));
    assert!(!package.content_types.overrides.contains_key(SLIDE_TWO_PART));
    assert!(!package.content_types.overrides.contains_key(NOTES_PART));
    assert!(
        package
            .get_part_rels(PRESENTATION_PART)
            .unwrap()
            .get_by_id("ordered-first")
            .is_none()
    );
    let presentation_xml =
        String::from_utf8(package.get_part(PRESENTATION_PART).unwrap().to_vec()).unwrap();
    assert_eq!(
        presentation_xml.matches(r#"r:id="ordered-first""#).count(),
        0
    );
    assert!(presentation_xml.contains("<x:marker/>"));
    assert!(presentation_xml.contains(r#"r:id="ordered-second""#));
    let reopened = Presentation::from_bytes(&bytes).unwrap();
    assert_eq!(reopened.len(), 1);
    assert!(reopened.validate().is_empty());
}

#[test]
fn removing_shared_and_last_image_users_prunes_only_new_orphans() {
    let png = png_header(4, 3);
    let mut presentation = Presentation::new().expect("open bundled template");
    for _ in 0..2 {
        presentation.add_slide(0).expect("add slide");
    }
    for index in 0..2 {
        presentation
            .add_picture(index, &png, "shared.png", Emu(0), Emu(0), None, None)
            .unwrap();
    }

    presentation.remove_slide(0).expect("remove one image user");
    let shared = open_opc(&presentation.to_bytes().unwrap(), "shared image remains");
    assert_eq!(
        shared
            .parts
            .iter()
            .filter(|(part, bytes)| part.starts_with("/ppt/media/") && *bytes == &png)
            .count(),
        1
    );

    presentation
        .remove_slide(0)
        .expect("remove last image user");
    let pruned = open_opc(&presentation.to_bytes().unwrap(), "last image pruned");
    assert_eq!(
        pruned
            .parts
            .iter()
            .filter(|(part, bytes)| part.starts_with("/ppt/media/") && *bytes == &png)
            .count(),
        0
    );
}

#[test]
fn removing_last_slide_image_preserves_a_package_root_relationship_target() {
    let png = png_header(4, 3);
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add slide");
    presentation
        .add_picture(0, &png, "root-shared.png", Emu(0), Emu(0), None, None)
        .unwrap();
    let mut package = open_opc(&presentation.to_bytes().unwrap(), "root media fixture");
    let media_part = package
        .parts
        .keys()
        .find(|part| part.starts_with("/ppt/media/"))
        .unwrap()
        .clone();
    package
        .package_rels
        .add(rel_types::THUMBNAIL, media_part.trim_start_matches('/'));
    let mut presentation = Presentation::from_bytes(&package_bytes(package)).unwrap();

    presentation.remove_slide(0).expect("remove final slide");

    let package = open_opc(&presentation.to_bytes().unwrap(), "root media result");
    let relationship = package
        .package_rels
        .get_all_by_type(rel_types::THUMBNAIL)
        .into_iter()
        .find(|relationship| {
            OpcPackage::resolve_rel_target("/", &relationship.target) == media_part
        })
        .unwrap();
    assert_eq!(
        OpcPackage::resolve_rel_target("/", &relationship.target),
        media_part
    );
    assert_eq!(package.get_part(&media_part).unwrap(), png);
    assert!(presentation.validate().is_empty());
}

#[test]
fn duplicated_slides_images_resolve_to_the_new_slides_own_relationships() {
    let png = png_header(4, 3);
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add source slide");
    presentation
        .add_picture(0, &png, "source.png", Emu(0), Emu(0), None, None)
        .unwrap();

    presentation.duplicate_slide(0).expect("duplicate slide");

    let bytes = presentation.to_bytes().unwrap();
    let package = open_opc(&bytes, "duplicated image scopes");
    let presentation_part = package.main_document_part().unwrap();
    let model = CT_Presentation::from_xml(package.get_part(&presentation_part).unwrap()).unwrap();
    let mut image_relationship_ids = Vec::new();
    let mut image_targets = Vec::new();
    for slide_id in &model.slide_ids {
        let slide_relationship = package
            .get_part_rels(&presentation_part)
            .unwrap()
            .get_by_id(&slide_id.relationship_id)
            .unwrap();
        let slide_part =
            OpcPackage::resolve_rel_target(&presentation_part, &slide_relationship.target);
        let slide = CT_Slide::from_xml(package.get_part(&slide_part).unwrap()).unwrap();
        let embed = slide
            .common_slide_data
            .shape_tree
            .children
            .iter()
            .find_map(|child| match child {
                ShapeTreeChild::Picture(picture) => picture
                    .blip_fill
                    .as_ref()
                    .and_then(|fill| fill.blip.as_ref())
                    .and_then(|blip| blip.embed.clone()),
                _ => None,
            })
            .unwrap();
        let image_relationship = package
            .get_part_rels(&slide_part)
            .unwrap()
            .get_by_id(&embed)
            .unwrap();
        assert_eq!(image_relationship.rel_type, rel_types::IMAGE);
        image_relationship_ids.push(embed);
        image_targets.push(OpcPackage::resolve_rel_target(
            &slide_part,
            &image_relationship.target,
        ));
    }
    assert_eq!(image_relationship_ids.len(), 2);
    assert_eq!(image_targets[0], image_targets[1]);
    assert_eq!(package.get_part(&image_targets[0]).unwrap(), png);
    assert_eq!(
        package
            .parts
            .keys()
            .filter(|part| part.starts_with("/ppt/media/"))
            .count(),
        1
    );
    let reopened = Presentation::from_bytes(&bytes).unwrap();
    assert_eq!(reopened.len(), 2);
    assert!(reopened.validate().is_empty());
}

#[test]
fn duplicate_slide_rewrites_typed_and_preserved_relationship_ids_without_other_byte_changes() {
    let mut package = open_opc(&mutation_fixture_bytes(), "relationship rewrite fixture");
    let image_part = "/custom/media/preserved.png";
    let image = png_header(3, 2);
    package.set_part(image_part, image.clone());
    package.content_types.add_default("png", "image/png");
    package.get_or_create_part_rels(SLIDE_TWO_PART).add_with_id(
        "rId7",
        rel_types::IMAGE,
        "../media/preserved.png",
    );
    let marked = String::from_utf8(package.get_part(SLIDE_TWO_PART).unwrap().to_vec())
        .unwrap()
        .replacen(
            "<p:blipFill/>",
            &format!(
                r#"<p:blipFill><a:blip xmlns:r="{R_NS}" r:embed="rId7"><x:effect xmlns:x="urn:f114"> A &amp; B </x:effect></a:blip></p:blipFill>"#
            ),
            1,
        )
        .replacen(
            "</p:cSld>",
            &format!(
                r#"<x:preserved xmlns:x="urn:f114" xmlns:r="{R_NS}" r:id="rId7" syntax=" A &amp; B "/></p:cSld>"#
            ),
            1,
        );
    package.set_part(SLIDE_TWO_PART, marked.into_bytes());
    let mut presentation = Presentation::from_bytes(&package_bytes(package)).unwrap();
    let source_text = presentation.slide(0).unwrap().text();

    presentation
        .duplicate_slide(0)
        .expect("duplicate marked slide");

    assert_eq!(presentation.slide(1).unwrap().text(), source_text);
    let bytes = presentation.to_bytes().unwrap();
    let package = open_opc(&bytes, "relationship rewrite result");
    let model = CT_Presentation::from_xml(package.get_part(PRESENTATION_PART).unwrap()).unwrap();
    let duplicate_relationship = package
        .get_part_rels(PRESENTATION_PART)
        .unwrap()
        .get_by_id(&model.slide_ids[1].relationship_id)
        .unwrap();
    let duplicate_part =
        OpcPackage::resolve_rel_target(PRESENTATION_PART, &duplicate_relationship.target);
    let duplicate_xml =
        String::from_utf8(package.get_part(&duplicate_part).unwrap().to_vec()).unwrap();
    assert!(duplicate_xml.contains(r#"r:embed="rId2""#));
    assert!(duplicate_xml.contains(r#"r:id="rId2" syntax=" A &amp; B "/>"#));
    assert!(duplicate_xml.contains(r#"<x:effect xmlns:x="urn:f114"> A &amp; B </x:effect>"#));
    assert!(!duplicate_xml.contains("rId7"));
    let image_relationship = package
        .get_part_rels(&duplicate_part)
        .unwrap()
        .get_by_id("rId2")
        .unwrap();
    let duplicate_image =
        OpcPackage::resolve_rel_target(&duplicate_part, &image_relationship.target);
    assert_eq!(package.get_part(&duplicate_image).unwrap(), image);
    assert!(Presentation::from_bytes(&bytes).is_ok());
}

#[test]
fn duplicate_slide_assigns_fresh_shape_ids_and_rewrites_connector_endpoints() {
    let mut presentation = Presentation::from_bytes(&mutation_fixture_bytes()).unwrap();

    presentation
        .duplicate_slide(0)
        .expect("duplicate connected shapes");

    let package = open_opc(
        &presentation.to_bytes().unwrap(),
        "duplicated connector ids",
    );
    let model = CT_Presentation::from_xml(package.get_part(PRESENTATION_PART).unwrap()).unwrap();
    let slide_parts = model
        .slide_ids
        .iter()
        .map(|slide_id| {
            let relationship = package
                .get_part_rels(PRESENTATION_PART)
                .unwrap()
                .get_by_id(&slide_id.relationship_id)
                .unwrap();
            OpcPackage::resolve_rel_target(PRESENTATION_PART, &relationship.target)
        })
        .collect::<Vec<_>>();
    let source = String::from_utf8(package.get_part(&slide_parts[0]).unwrap().to_vec()).unwrap();
    let duplicate = String::from_utf8(package.get_part(&slide_parts[1]).unwrap().to_vec()).unwrap();
    assert!(source.contains(r#"<a:stCxn id="2" idx="0"/><a:endCxn id="3" idx="1"/>"#));
    assert!(duplicate.contains(r#"<a:stCxn id="9" idx="0"/><a:endCxn id="10" idx="1"/>"#));
    assert!(source.contains(r#"<p:cNvPr id="20" name="choice shape"/>"#));
    assert!(source.contains(r#"<a:stCxn id="20" idx="0"/><a:endCxn id="20" idx="1"/>"#));
    assert!(source.contains(r#"<p:cNvPr id="22" name="fallback shape"/>"#));
    assert!(source.contains(r#"<a:stCxn id="22" idx="0"/><a:endCxn id="22" idx="1"/>"#));
    assert!(duplicate.contains(r#"<p:cNvPr id="16" name="choice shape"/>"#));
    assert!(duplicate.contains(r#"<p:cNvPr id="17" name="choice connector"/>"#));
    assert!(duplicate.contains(r#"<a:stCxn id="16" idx="0"/><a:endCxn id="16" idx="1"/>"#));
    assert!(duplicate.contains(r#"<p:cNvPr id="18" name="fallback shape"/>"#));
    assert!(duplicate.contains(r#"<p:cNvPr id="19" name="fallback connector"/>"#));
    assert!(duplicate.contains(r#"<a:stCxn id="18" idx="0"/><a:endCxn id="18" idx="1"/>"#));
    for stale_id in [20, 21, 22, 23] {
        assert!(!duplicate.contains(&format!(r#"id="{stale_id}""#)));
    }
    let duplicate_slide = CT_Slide::from_xml(duplicate.as_bytes()).unwrap();
    let connector = duplicate_slide
        .common_slide_data
        .shape_tree
        .children
        .iter()
        .find_map(|child| match child {
            ShapeTreeChild::Connector(connector) => Some(connector),
            _ => None,
        })
        .unwrap();
    assert_eq!(connector.start_connection.as_ref().unwrap().id, 9);
    assert_eq!(connector.end_connection.as_ref().unwrap().id, 10);
    assert_eq!(
        duplicate_slide
            .common_slide_data
            .shape_tree
            .children
            .iter()
            .find_map(|child| matches!(child, ShapeTreeChild::Connector(_))
                .then(|| child.non_visual_id()))
            .flatten(),
        Some(15)
    );
    assert!(!duplicate.contains(r#"<p:cNvPr id="2""#));
    assert!(presentation.validate().is_empty());
}

#[test]
fn duplicate_slide_copies_notes_with_a_fresh_back_relationship() {
    let mut presentation = Presentation::from_bytes(&fixture_bytes()).unwrap();
    let notes = presentation.slide(0).unwrap().notes_text();

    presentation
        .duplicate_slide(0)
        .expect("duplicate slide with notes");

    assert_eq!(presentation.slide(1).unwrap().notes_text(), notes);
    let bytes = presentation.to_bytes().unwrap();
    let package = open_opc(&bytes, "duplicated notes scope");
    let model = CT_Presentation::from_xml(package.get_part(PRESENTATION_PART).unwrap()).unwrap();
    let mut notes_parts = Vec::new();
    for slide_id in model.slide_ids.iter().take(2) {
        let slide_relationship = package
            .get_part_rels(PRESENTATION_PART)
            .unwrap()
            .get_by_id(&slide_id.relationship_id)
            .unwrap();
        let slide_part =
            OpcPackage::resolve_rel_target(PRESENTATION_PART, &slide_relationship.target);
        let notes_relationship = package
            .get_part_rels(&slide_part)
            .unwrap()
            .items
            .iter()
            .find(|relationship| relationship.rel_type == rel_types::NOTES_SLIDE)
            .unwrap();
        let notes_part = OpcPackage::resolve_rel_target(&slide_part, &notes_relationship.target);
        let back_relationship = package
            .get_part_rels(&notes_part)
            .unwrap()
            .items
            .iter()
            .find(|relationship| relationship.rel_type == rel_types::SLIDE)
            .unwrap();
        assert_eq!(
            OpcPackage::resolve_rel_target(&notes_part, &back_relationship.target),
            slide_part
        );
        notes_parts.push(notes_part);
    }
    assert_ne!(notes_parts[0], notes_parts[1]);
    assert!(Presentation::from_bytes(&bytes).is_ok());
}

#[test]
fn duplicate_slide_adds_a_missing_notes_back_relationship() {
    let mut package = fixture_package();
    package
        .get_or_create_part_rels(NOTES_PART)
        .items
        .retain(|relationship| relationship.rel_type != rel_types::SLIDE);

    assert_notes_back_relationship_is_normalized(package, &HashSet::new());
}

#[test]
fn duplicate_slide_collapses_multiple_notes_back_relationships() {
    let mut package = fixture_package();
    package.get_or_create_part_rels(NOTES_PART).add_with_id(
        "second-slide-link",
        rel_types::SLIDE,
        "../slides/first.xml",
    );
    let source_ids = package
        .get_part_rels(NOTES_PART)
        .unwrap()
        .items
        .iter()
        .filter(|relationship| relationship.rel_type == rel_types::SLIDE)
        .map(|relationship| relationship.id.clone())
        .collect();

    assert_notes_back_relationship_is_normalized(package, &source_ids);
}

#[test]
fn duplicate_slide_replaces_an_external_notes_back_relationship() {
    let mut package = fixture_package();
    let relationship = package
        .get_or_create_part_rels(NOTES_PART)
        .items
        .iter_mut()
        .find(|relationship| relationship.rel_type == rel_types::SLIDE)
        .unwrap();
    relationship.target = "https://example.com/source-slide".to_owned();
    relationship.target_mode = Some("External".to_owned());
    let source_ids = HashSet::from([relationship.id.clone()]);

    assert_notes_back_relationship_is_normalized(package, &source_ids);
}

#[test]
fn duplicate_slide_rewrites_preserved_nonnumeric_notes_back_references() {
    let mut package = fixture_package();
    let marked_notes = String::from_utf8(package.get_part(NOTES_PART).unwrap().to_vec())
        .unwrap()
        .replacen(
            "</p:cSld>",
            &format!(
                r#"<x:back xmlns:x="urn:f114" xmlns:r="{R_NS}" r:id="slide-link" syntax=" A &amp; B "/><x:other xmlns:x="urn:f114" xmlns:r="{R_NS}" r:id="other-link" syntax=" untouched "/></p:cSld>"#
            ),
            1,
        );
    package.set_part(NOTES_PART, marked_notes.into_bytes());
    package.get_or_create_part_rels(NOTES_PART).add_with_id(
        "other-link",
        "urn:f114:unrelated",
        "../opaque/data.bin",
    );
    let mut presentation = Presentation::from_bytes(&package_bytes(package)).unwrap();

    presentation.duplicate_slide(0).expect("duplicate slide");
    let saved = presentation.to_bytes().unwrap();
    let reopened = Presentation::from_bytes(&saved).unwrap();
    let resaved = reopened.to_bytes().unwrap();
    let package = open_opc(&resaved, "nonnumeric notes back reference result");
    let model = CT_Presentation::from_xml(package.get_part(PRESENTATION_PART).unwrap()).unwrap();
    let duplicate_relationship = package
        .get_part_rels(PRESENTATION_PART)
        .unwrap()
        .get_by_id(&model.slide_ids[1].relationship_id)
        .unwrap();
    let duplicate_slide =
        OpcPackage::resolve_rel_target(PRESENTATION_PART, &duplicate_relationship.target);
    let notes_relationship = package
        .get_part_rels(&duplicate_slide)
        .unwrap()
        .items
        .iter()
        .find(|relationship| relationship.rel_type == rel_types::NOTES_SLIDE)
        .unwrap();
    let duplicate_notes =
        OpcPackage::resolve_rel_target(&duplicate_slide, &notes_relationship.target);
    let notes_relationships = package.get_part_rels(&duplicate_notes).unwrap();
    let back_relationships = notes_relationships
        .items
        .iter()
        .filter(|relationship| relationship.rel_type == rel_types::SLIDE)
        .collect::<Vec<_>>();
    assert_eq!(back_relationships.len(), 1);
    let back_relationship = back_relationships[0];
    assert_ne!(back_relationship.id, "slide-link");
    assert_eq!(back_relationship.target_mode, None);
    assert_eq!(
        OpcPackage::resolve_rel_target(&duplicate_notes, &back_relationship.target),
        duplicate_slide
    );
    let duplicate_xml =
        String::from_utf8(package.get_part(&duplicate_notes).unwrap().to_vec()).unwrap();
    assert!(duplicate_xml.contains(&format!(
        r#"r:id="{}" syntax=" A &amp; B "/>"#,
        back_relationship.id
    )));
    assert!(duplicate_xml.contains(
        r#"<x:other xmlns:x="urn:f114" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" r:id="other-link" syntax=" untouched "/>"#
    ));
    assert!(!duplicate_xml.contains(r#"r:id="slide-link""#));
    let unrelated = notes_relationships.get_by_id("other-link").unwrap();
    assert_eq!(unrelated.rel_type, "urn:f114:unrelated");
    assert_eq!(unrelated.target_mode, None);
    assert_eq!(
        OpcPackage::resolve_rel_target(&duplicate_notes, &unrelated.target),
        "/custom/opaque/data.bin"
    );
    assert!(reopened.validate().is_empty());
}

#[test]
fn remove_move_and_duplicate_round_trip_with_clean_validation() {
    let mut presentation = Presentation::new().expect("open bundled template");
    for _ in 0..3 {
        presentation.add_slide(0).expect("add slide");
    }

    presentation
        .duplicate_slide(1)
        .expect("duplicate middle slide");
    presentation.move_slide(3, 0).expect("move duplicate first");
    presentation.remove_slide(2).expect("remove one slide");

    let reopened = Presentation::from_bytes(&presentation.to_bytes().unwrap()).unwrap();
    assert_eq!(reopened.len(), 3);
    assert!(reopened.validate().is_empty());
}

#[test]
fn merge_then_split_restores_the_original_grid() {
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add slide");
    {
        let mut slide = presentation.slide_mut(0).expect("borrow slide");
        let mut shape = slide
            .add_table(2, 3, Emu(10), Emu(20), Emu(302), Emu(201))
            .expect("add table");
        let mut table = shape.table_mut().expect("table handle");
        let widths = (0..3)
            .map(|column| table.column_width(column).unwrap())
            .collect::<Vec<_>>();
        table.cell_mut(0, 0).unwrap().merge_to(1, 2).unwrap();
        table.cell_mut(0, 0).unwrap().split().unwrap();
        assert_eq!(
            (0..3)
                .map(|column| table.column_width(column).unwrap())
                .collect::<Vec<_>>(),
            widths
        );
    }

    let saved = presentation.to_bytes().expect("save presentation");
    let reopened = Presentation::from_bytes(&saved).expect("reopen presentation");
    let slide = reopened.slide(0).unwrap();
    let table = slide.shapes().last().unwrap().table().unwrap();
    assert_eq!(table.row_count(), 2);
    assert_eq!(table.column_count(), 3);
    assert!((0..2).all(|row| (0..3).all(|column| {
        let cell = table.cell(row, column).unwrap();
        !cell.is_merge_origin()
            && !cell.is_spanned()
            && cell.span_height() == 1
            && cell.span_width() == 1
    })));
    assert_eq!(
        (0..3)
            .map(|column| table.column_width(column).unwrap())
            .collect::<Vec<_>>(),
        vec![Emu(100), Emu(100), Emu(102)]
    );

    let package = open_opc(&saved, "F-113 merge then split");
    let slide = CT_Slide::from_xml(package.get_part("/ppt/slides/slide1.xml").unwrap()).unwrap();
    let ShapeTreeChild::GraphicFrame(frame) =
        slide.common_slide_data.shape_tree.children.last().unwrap()
    else {
        panic!("expected table graphic frame");
    };
    let GraphicDataPayload::Table(table) = frame.graphic_data.payload() else {
        panic!("expected table payload");
    };
    assert_eq!(
        table.rows.iter().map(|row| row.height).collect::<Vec<_>>(),
        vec![Emu(100), Emu(101)]
    );
    assert!(table.rows.iter().all(|row| row.cells.len() == 3));
}

#[test]
fn add_table_round_trips_cells_formatting_banding_and_widths() {
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add slide");
    let fill = Fill::from_xml(br#"<a:solidFill><a:srgbClr val="112233"/></a:solidFill>"#).unwrap();
    {
        let mut slide = presentation.slide_mut(0).unwrap();
        let mut shape = slide
            .add_table(2, 2, Emu(10), Emu(20), Emu(201), Emu(101))
            .expect("add table");
        let mut table = shape.table_mut().unwrap();
        table.set_first_row(true);
        table.set_last_row(true);
        table.set_first_column(true);
        table.set_last_column(true);
        table.set_horizontal_banding(true);
        table.set_vertical_banding(true);
        table.set_column_width(1, Emu(150)).unwrap();
        let mut cell = table.cell_mut(0, 0).unwrap();
        cell.set_text("formatted");
        cell.text_frame()
            .add_paragraph()
            .set_text("second paragraph");
        cell.set_fill(Some(fill.clone()));
        cell.set_margins(Some(Emu(10)), Some(Emu(20)), Some(Emu(30)), Some(Emu(40)));
    }

    let saved = presentation.to_bytes().unwrap();
    let reopened = Presentation::from_bytes(&saved).unwrap();
    let slide = reopened.slide(0).unwrap();
    let table = slide.shapes().last().unwrap().table().unwrap();
    assert_eq!(table.row_count(), 2);
    assert_eq!(table.column_count(), 2);
    assert_eq!(table.column_width(0), Some(Emu(100)));
    assert_eq!(table.column_width(1), Some(Emu(150)));
    assert!(table.first_row());
    assert!(table.last_row());
    assert!(table.first_column());
    assert!(table.last_column());
    assert!(table.horizontal_banding());
    assert!(table.vertical_banding());
    let cell = table.cell(0, 0).unwrap();
    assert_eq!(cell.text(), "formatted\nsecond paragraph");
    assert_eq!(cell.fill(), Some(&fill));
    assert_eq!(
        cell.margins(),
        (Some(Emu(10)), Some(Emu(20)), Some(Emu(30)), Some(Emu(40)),)
    );

    let package = open_opc(&saved, "F-113 table formatting");
    let slide = CT_Slide::from_xml(package.get_part("/ppt/slides/slide1.xml").unwrap()).unwrap();
    let ShapeTreeChild::GraphicFrame(frame) =
        slide.common_slide_data.shape_tree.children.last().unwrap()
    else {
        panic!("expected table graphic frame");
    };
    assert_eq!(frame.transform.extent.unwrap().cx, Emu(250));
    assert_eq!(frame.transform.extent.unwrap().cy, Emu(101));
}

#[test]
fn table_mutation_rejects_invalid_ranges_without_partial_changes() {
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add slide");
    let before = presentation.to_bytes().unwrap();
    let rejected = {
        let mut slide = presentation.slide_mut(0).unwrap();
        matches!(
            slide.add_table(0, 2, Emu(0), Emu(0), Emu(200), Emu(200)),
            Err(Error::InvalidTableMutation { .. })
        )
    };
    assert!(rejected);
    assert_eq!(presentation.to_bytes().unwrap(), before);

    let rejected = {
        let mut slide = presentation.slide_mut(0).unwrap();
        matches!(
            slide.add_table(2, 2, Emu(0), Emu(0), Emu(-1), Emu(200)),
            Err(Error::InvalidTableMutation { .. })
        )
    };
    assert!(rejected);
    assert_eq!(presentation.to_bytes().unwrap(), before);

    presentation
        .slide_mut(0)
        .unwrap()
        .add_table(2, 3, Emu(0), Emu(0), Emu(300), Emu(200))
        .unwrap();
    let table_index = presentation.slide(0).unwrap().shapes().len() - 1;
    let before_invalid_cell = presentation.to_bytes().unwrap();
    {
        let mut slide = presentation.slide_mut(0).unwrap();
        let mut shape = slide.shape_mut(table_index).unwrap();
        let mut table = shape.table_mut().unwrap();
        assert!(table.cell_mut(2, 0).is_none());
        assert!(table.cell_mut(0, 3).is_none());
        assert!(table.cell_mut(0, 0).unwrap().merge_to(2, 0).is_err());
        assert!(table.cell_mut(0, 0).unwrap().merge_to(0, 0).is_err());
    }
    assert_eq!(presentation.to_bytes().unwrap(), before_invalid_cell);

    {
        let mut slide = presentation.slide_mut(0).unwrap();
        let mut shape = slide.shape_mut(table_index).unwrap();
        let mut table = shape.table_mut().unwrap();
        assert!(table.cell_mut(0, 0).unwrap().split().is_err());
        assert!(table.set_column_width(0, Emu(0)).is_err());
        assert!(table.set_column_width(0, Emu(i64::MAX)).is_err());
    }
    assert_eq!(presentation.to_bytes().unwrap(), before_invalid_cell);

    {
        let mut slide = presentation.slide_mut(0).unwrap();
        let mut shape = slide.shape_mut(table_index).unwrap();
        shape
            .table_mut()
            .unwrap()
            .cell_mut(0, 0)
            .unwrap()
            .merge_to(1, 1)
            .unwrap();
    }
    let before_overlap = presentation.to_bytes().unwrap();
    {
        let mut slide = presentation.slide_mut(0).unwrap();
        let mut shape = slide.shape_mut(table_index).unwrap();
        let result = shape
            .table_mut()
            .unwrap()
            .cell_mut(0, 1)
            .unwrap()
            .merge_to(1, 2);
        assert!(matches!(result, Err(Error::InvalidTableMutation { .. })));
    }
    assert_eq!(presentation.to_bytes().unwrap(), before_overlap);
}

#[test]
fn table_mutation_preserves_unmodelled_xml_and_schema_order() {
    let fixture = table_mutation_fixture_bytes();
    let original_package = open_opc(&fixture, "F-113 original table preservation");
    let original_xml = original_package.get_part(SLIDE_TWO_PART).unwrap();
    let original_raw = capture_exact_subtree(original_xml, b"<x:raw-payload", b"</x:raw-payload>");
    let mut presentation = Presentation::from_bytes(&fixture).unwrap();
    {
        let mut slide = presentation.slide_mut(0).unwrap();
        let mut table_shape = slide.shape_mut(2).unwrap();
        let mut table = table_shape.table_mut().unwrap();
        table.set_column_width(0, Emu(125)).unwrap();
    }
    let saved = presentation.to_bytes().unwrap();
    let package = open_opc(&saved, "F-113 table preservation");
    let saved_xml = package.get_part(SLIDE_TWO_PART).unwrap();
    let saved_raw = capture_exact_subtree(saved_xml, b"<x:raw-payload", b"</x:raw-payload>");
    assert_eq!(saved_raw, original_raw, "unsupported table XML bytes");
    let xml = String::from_utf8(saved_xml.to_vec()).unwrap();
    for marker in [
        r#"producer="kept""#,
        r#"<x:before-table/>"#,
        r#"x:grid="kept""#,
        r#"<x:before-column/>"#,
        r#"x:column="kept""#,
        r#"<x:column-child/>"#,
        r#"<x:after-grid/>"#,
        r#"x:row="kept""#,
        r#"<x:before-cell/>"#,
        r#"x:cell="kept""#,
        r#"<x:before-text/>"#,
        r#"x:cell-properties="kept""#,
        r#"<x:cell-properties-child/>"#,
    ] {
        assert!(xml.contains(marker), "missing preserved marker {marker}");
    }
    assert!(xml.contains(r#"<a:gridCol w="125" x:column="kept">"#));
    let table_start = xml.find("<a:tbl ").unwrap();
    let grid_start = xml[table_start..].find("<a:tblGrid").unwrap() + table_start;
    let row_start = xml[grid_start..].find("<a:tr ").unwrap() + grid_start;
    assert!(table_start < grid_start && grid_start < row_start);
    let cell_start = xml[row_start..].find("<a:tc ").unwrap() + row_start;
    let text_start = xml[cell_start..].find("<a:txBody>").unwrap() + cell_start;
    let properties_start = xml[cell_start..].find("<a:tcPr ").unwrap() + cell_start;
    assert!(cell_start < text_start && text_start < properties_start);
}

#[test]
#[ignore = "requires uv and pinned python-pptx 1.0.2"]
fn add_table_matches_pinned_python_pptx_table_semantics() {
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add slide");
    for (index, split) in [false, true].into_iter().enumerate() {
        let mut slide = presentation.slide_mut(0).unwrap();
        let mut shape = slide
            .add_table(
                2,
                3,
                Emu(10),
                Emu(20 + i64::try_from(index).unwrap() * 300),
                Emu(302),
                Emu(201),
            )
            .expect("add table");
        let mut table = shape.table_mut().unwrap();
        for (cell_index, text) in ["A", "B", "C", "D", "E", "F"].into_iter().enumerate() {
            table
                .cell_mut(cell_index / 3, cell_index % 3)
                .unwrap()
                .set_text(text);
        }
        table.cell_mut(1, 1).unwrap().merge_to(0, 0).unwrap();
        if split {
            table.cell_mut(0, 0).unwrap().split().unwrap();
        }
    }

    let output_path = std::env::temp_dir().join(format!(
        "rpptx-f113-python-oracle-{}.pptx",
        std::process::id()
    ));
    fs::write(&output_path, presentation.to_bytes().unwrap()).unwrap();
    let script = r#"
import json
import sys
import pptx
from pptx import Presentation

assert pptx.__version__ == "1.0.2", pptx.__version__

def normalize(shape):
    table = shape.table
    return {
        "frame": [shape.left, shape.top, shape.width, shape.height],
        "columns": [column.width for column in table.columns],
        "rows": [row.height for row in table.rows],
        "flags": [
            table.first_row,
            table.last_row,
            table.first_col,
            table.last_col,
            table.horz_banding,
            table.vert_banding,
        ],
        "cells": [[
            cell.text,
            cell.is_merge_origin,
            cell.is_spanned,
            cell.span_height,
            cell.span_width,
        ] for row in table.rows for cell in row.cells],
    }

ours = Presentation(sys.argv[1])
ours_records = [normalize(shape) for shape in list(ours.slides[0].shapes)[-2:]]

oracle = Presentation()
slide = oracle.slides.add_slide(oracle.slide_layouts[6])
for index, split in enumerate((False, True)):
    shape = slide.shapes.add_table(2, 3, 10, 20 + index * 300, 302, 201)
    table = shape.table
    for cell, text in zip((cell for row in table.rows for cell in row.cells), "ABCDEF"):
        cell.text = text
    table.cell(0, 0).merge(table.cell(1, 1))
    if split:
        table.cell(0, 0).split()
oracle_records = [normalize(shape) for shape in list(slide.shapes)[-2:]]

print(json.dumps(ours_records, sort_keys=True))
print(json.dumps(oracle_records, sort_keys=True))
"#;
    let output = Command::new("uv")
        .args([
            "run",
            "--with",
            "python-pptx==1.0.2",
            "python",
            "-c",
            script,
        ])
        .arg(&output_path)
        .output()
        .expect("run pinned python-pptx table oracle");
    fs::remove_file(&output_path).unwrap();
    assert!(
        output.status.success(),
        "python-pptx oracle failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let records = String::from_utf8(output.stdout).unwrap();
    let mut lines = records.lines();
    let ours = lines.next().unwrap();
    let oracle = lines.next().unwrap();
    assert_eq!(ours, oracle, "normalized table semantics");
    assert!(lines.next().is_none());
}

fn png_header(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
    bytes.extend_from_slice(&[0; 4]);
    bytes
}

fn valid_one_pixel_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x78, 0xda, 0x63, 0xf8,
        0xcf, 0xc0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0xf7, 0x03, 0x41, 0x43, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ]
}

fn png_inflation_bomb() -> Vec<u8> {
    let inflated = vec![0; 1024 * 1024];
    let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&inflated, 6);
    let mut png = png_header(1, 1);
    png.extend_from_slice(&(compressed.len() as u32).to_be_bytes());
    png.extend_from_slice(b"IDAT");
    png.extend_from_slice(&compressed);
    png.extend_from_slice(&[0; 4]);
    png.extend_from_slice(&0_u32.to_be_bytes());
    png.extend_from_slice(b"IEND");
    png.extend_from_slice(&[0; 4]);
    png
}

fn sparse_preview_png() -> Vec<u8> {
    let width = 9_u32;
    let height = 9_u32;
    let mut pixels = Vec::with_capacity((width * height * 4 + height) as usize);
    for row in 0..height {
        pixels.push(0);
        for column in 0..width {
            if row == 0 && column == 0 {
                pixels.extend_from_slice(&[255, 0, 0, 255]);
            } else {
                pixels.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&pixels, 6);
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    append_png_chunk(&mut png, b"IHDR", &ihdr);
    append_png_chunk(&mut png, b"IDAT", &compressed);
    append_png_chunk(&mut png, b"IEND", &[]);
    png
}

fn append_png_chunk(png: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    png.extend_from_slice(&(data.len() as u32).to_be_bytes());
    png.extend_from_slice(kind);
    png.extend_from_slice(data);
    let mut crc = 0xffff_ffff_u32;
    for byte in kind.iter().chain(data) {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    png.extend_from_slice(&(!crc).to_be_bytes());
}

fn valid_template_jpeg() -> Vec<u8> {
    let bytes = Presentation::new().unwrap().to_bytes().unwrap();
    open_opc(&bytes, "bundled template JPEG")
        .get_part("/docProps/thumbnail.jpeg")
        .unwrap()
        .to_vec()
}

fn valid_grayscale_jpeg() -> Vec<u8> {
    decode_base64_fixture(
        "/9j/4AAQSkZJRgABAQAASABIAAD/4QNERXhpZgAATU0AKgAAAAgACQEPAAIAAAAIAAAAegEQAAIAAAAOAAAAggEaAAUAAAABAAAAkAEbAAUAAAABAAAAmAEoAAMAAAABAAIAAAExAAIAAAAMAAAAoAEyAAIAAAAUAAAArIdpAAQAAAABAAAAwIglAAQAAAABAAACegAAAABPbmVQbHVzAE9ORVBMVVMgQTUwMTAAAAAASAAAAAEAAABIAAAAAUdJTVAgMi44LjIyADIwMTk6MTE6MjYgMjI6Mzc6NDQAABuCmgAFAAAAAQAAAgqCnQAFAAAAAQAAAhKIIgADAAAAAQAAAACIJwADAAAAAQBkAACQAAAHAAAABDAyMjCQAwACAAAAFAAAAhqQBAACAAAAFAAAAi6RAQAHAAAABAECAwCSAQAKAAAAAQAAAkKSAgAFAAAAAQAAAkqSAwAKAAAAAQAAAlKSBwADAAAAAQAAAACSCQADAAAAAQAQAACSCgAFAAAAAQAAAlqSkAACAAAABwAAAmKSkQACAAAABwAAAmqSkgACAAAABwAAAnKgAAAHAAAABDAxMDCgAQADAAAAAf//AACgAgAEAAAAAQAAAAGgAwAEAAAAAQAAAAGiFwADAAAAAQACAACjAQAHAAAAAQEAAACkAgADAAAAAQAAAACkAwADAAAAAQAAAACkBQADAAAAAQAYAACkBgADAAAAAQAAAAAAAAAAAAAAAQAABgsAAAARAAAACjIwMTk6MTE6MDkgMTA6NTk6MzYAMjAxOToxMTowOSAxMDo1OTozNgAAAAhHAAAAyAAAAJkAAABkAAAAJQAAAAUAABAHAAAD6DYwMjY5NQAANjAyNjk1AAA2MDI2OTUAAAAIAAEAAgAAAAJOAAAAAAIABQAAAAMAAALgAAMAAgAAAAJFAAAAAAQABQAAAAMAAAL4AAUAAQAAAAEAAAAAAAYABQAAAAEAAAMQAAcABQAAAAMAAAMYAB0AAgAAAAsAAAMwAAAAAAAAADAAAAABAAAAMwAAAAEAAAZnAAAAZAAAAAIAAAABAAAAEgAAAAEAAABoAAAAZAABK6cAAAPoAAAACQAAAAEAAAA7AAAAAQAAACQAAAABMjAxOToxMTowOQAA/+0AeFBob3Rvc2hvcCAzLjAAOEJJTQQEAAAAAAA/HAFaAAMbJUccAgAAAgACHAI/AAYxMDU5MzYcAj4ACDIwMTkxMTA5HAI3AAgyMDE5MTEwORwCPAAGMTA1OTM2ADhCSU0EJQAAAAAAEG0diLVAz066cPvLhCa6YLX/wAALCAABAAEBAREA/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/9sAQwACAgICAgIDAgIDBQMDAwUGBQUFBQYIBgYGBgYICggICAgICAoKCgoKCgoKDAwMDAwMDg4ODg4PDw8PDw8PDw8P/90ABAAB/9oACAEBAAA/APTK/9k=",
    )
}

fn valid_cmyk_jpeg() -> Vec<u8> {
    decode_base64_fixture(
        "/9j/4AAQSkZJRgABAQAASABIAAD/4QEGRXhpZgAATU0AKgAAAAgACAEGAAMAAAABAAIAAAESAAMAAAABAAEAAAEaAAUAAAABAAAAbgEbAAUAAAABAAAAdgEoAAMAAAABAAIAAAExAAIAAAAhAAAAfgEyAAIAAAAUAAAAoIdpAAQAAAABAAAAtAAAAAAAAABIAAAAAQAAAEgAAAABQWRvYmUgUGhvdG9zaG9wIDI0LjYgKE1hY2ludG9zaCkAADIwMjQ6MDM6MjMgMDk6NTY6MDMAAASQAAAHAAAABDAyMzGQBAACAAAAFAAAAOqgAgAEAAAAAQAAAAGgAwAEAAAAAQAAAAEAAAAAMjAyMjowNDoxMiAxMjowNTo1MQD/7QBkUGhvdG9zaG9wIDMuMAA4QklNBAQAAAAAACwcAVoAAxslRxwCAAACAAIcAj4ACDIwMjIwNDEyHAI/AAsxMjA1NTErMDIwMDhCSU0EJQAAAAAAEOtjiDMDJLv+fLJTuujhSZT/7gAOQWRvYmUAZAAAAAAA/8AAFAgAAQABBAERAAIRAQMRAQQRAf/EAB8AAAEFAQEBAQEBAAAAAAAAAAABAgMEBQYHCAkKC//EALUQAAIBAwMCBAMFBQQEAAABfQECAwAEEQUSITFBBhNRYQcicRQygZGhCCNCscEVUtHwJDNicoIJChYXGBkaJSYnKCkqNDU2Nzg5OkNERUZHSElKU1RVVldYWVpjZGVmZ2hpanN0dXZ3eHl6g4SFhoeIiYqSk5SVlpeYmZqio6Slpqeoqaqys7S1tre4ubrCw8TFxsfIycrS09TV1tfY2drh4uPk5ebn6Onq8fLz9PX29/j5+v/EAB8BAAMBAQEBAQEBAQEAAAAAAAABAgMEBQYHCAkKC//EALURAAIBAgQEAwQHBQQEAAECdwABAgMRBAUhMQYSQVEHYXETIjKBCBRCkaGxwQkjM1LwFWJy0QoWJDThJfEXGBkaJicoKSo1Njc4OTpDREVGR0hJSlNUVVZXWFlaY2RlZmdoaWpzdHV2d3h5eoKDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uLj5OXm5+jp6vLz9PX29/j5+v/bAEMAAgICAgICAwICAwUDAwMFBgUFBQUGCAYGBgYGCAoICAgICAgKCgoKCgoKCgwMDAwMDA4ODg4ODw8PDw8PDw8PD//bAEMBAgMDBAQEBwQEBxALCQsQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEP/dAAQAAf/aAA4EAQACEQMRBBEAPwDj6/qA/wA7z+/D/9k=",
    )
}

fn decode_base64_fixture(encoded: &str) -> Vec<u8> {
    let mut output = Vec::with_capacity(encoded.len() * 3 / 4);
    let mut accumulator = 0_u32;
    let mut bits = 0_u32;
    for byte in encoded.bytes().take_while(|byte| *byte != b'=') {
        let value = match byte {
            b'A'..=b'Z' => u32::from(byte - b'A'),
            b'a'..=b'z' => u32::from(byte - b'a') + 26,
            b'0'..=b'9' => u32::from(byte - b'0') + 52,
            b'+' => 62,
            b'/' => 63,
            _ => panic!("invalid fixture base64"),
        };
        accumulator = (accumulator << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push((accumulator >> bits) as u8);
            accumulator &= (1_u32 << bits).wrapping_sub(1);
        }
    }
    output
}

#[test]
fn setting_text_on_placeholder_round_trips_and_renders() {
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add title slide");
    let placeholder_index = presentation
        .slide(0)
        .unwrap()
        .shapes()
        .position(|shape| shape.text().is_some())
        .expect("layout clone supplies a text placeholder");
    presentation
        .slide_mut(0)
        .unwrap()
        .shape_mut(placeholder_index)
        .unwrap()
        .set_text("")
        .unwrap();
    let blank_render = deterministic_render(&presentation.to_bytes().unwrap());
    presentation
        .slide_mut(0)
        .unwrap()
        .shape_mut(placeholder_index)
        .unwrap()
        .set_text("Visible changed placeholder")
        .unwrap();

    let bytes = presentation
        .to_bytes()
        .expect("serialize changed placeholder");
    let reopened = Presentation::from_bytes(&bytes).expect("reopen changed placeholder");
    assert_eq!(
        reopened
            .slide(0)
            .unwrap()
            .shape(placeholder_index)
            .unwrap()
            .text(),
        Some("Visible changed placeholder".to_owned())
    );
    let changed_render = deterministic_render(&bytes);
    assert_ne!(
        changed_render, blank_render,
        "changed text must alter pixels"
    );
}

#[test]
fn clearing_text_preserves_required_paragraph() {
    let mut presentation = Presentation::from_bytes(&mutation_fixture_bytes()).unwrap();
    presentation
        .slide_mut(0)
        .unwrap()
        .shape_mut(0)
        .unwrap()
        .set_text("")
        .unwrap();
    let bytes = presentation.to_bytes().unwrap();
    let reopened = Presentation::from_bytes(&bytes).unwrap();
    assert!(reopened.validate().is_empty());
    let package = open_opc(&bytes, "F-112 empty text");
    let slide = CT_Slide::from_xml(package.get_part(SLIDE_TWO_PART).unwrap()).unwrap();
    let ShapeTreeChild::Shape(shape) = &slide.common_slide_data.shape_tree.children[0] else {
        panic!("expected ordinary shape");
    };
    assert_eq!(shape.text_body.as_ref().unwrap().paragraph_count(), 1);
}

#[test]
fn paragraph_run_font_and_bullet_properties_round_trip() {
    let mut presentation = Presentation::from_bytes(&mutation_fixture_bytes()).unwrap();
    let mut slide = presentation.slide_mut(0).unwrap();
    let mut shape = slide.shape_mut(0).unwrap();
    shape.set_text("first").unwrap();
    let mut frame = shape.text_frame().unwrap();
    let mut paragraph = frame.paragraph_mut(0).unwrap();
    let mut paragraph_properties = CT_TextParagraphProperties::default();
    paragraph_properties.level = Some(2);
    paragraph.set_properties(paragraph_properties);
    paragraph.set_bullet(Some(TextBullet {
        choice: Some(TextBulletChoice::Character(
            TextBulletCharacter::new("•").unwrap(),
        )),
        ..TextBullet::default()
    }));
    let mut run = paragraph.add_run(" second");
    let mut run_properties = CT_TextCharacterProperties::default();
    run_properties.font_size = Some(1_800);
    run_properties.bold = Some(true);
    run.set_properties(run_properties);
    run.set_font(Some(TextFont::new("Carlito").unwrap()));

    let bytes = presentation.to_bytes().unwrap();
    let package = open_opc(&bytes, "F-112 formatting round trip");
    let slide = CT_Slide::from_xml(package.get_part(SLIDE_TWO_PART).unwrap()).unwrap();
    let ShapeTreeChild::Shape(shape) = &slide.common_slide_data.shape_tree.children[0] else {
        panic!("expected ordinary shape");
    };
    let paragraph = &shape.text_body.as_ref().unwrap().paragraphs()[0];
    let properties = paragraph.properties.as_ref().unwrap();
    assert_eq!(properties.level, Some(2));
    assert!(matches!(
        properties.bullet.as_ref().and_then(|bullet| bullet.choice.as_ref()),
        Some(TextBulletChoice::Character(character)) if character.character == "•"
    ));
    let oxml_drawing::text::TextRun::Run(run) = &paragraph.runs[1] else {
        panic!("expected appended regular run");
    };
    let properties = run.properties.as_ref().unwrap();
    assert_eq!(properties.font_size, Some(1_800));
    assert_eq!(properties.bold, Some(true));
    assert_eq!(properties.latin.as_ref().unwrap().typeface, "Carlito");
}

#[test]
fn text_mutation_preserves_placeholder_identity() {
    let mut package = open_opc(&mutation_fixture_bytes(), "F-112 placeholder fixture");
    let original = String::from_utf8(package.get_part(SLIDE_TWO_PART).unwrap().to_vec()).unwrap();
    let with_placeholder = original.replacen(
        "<p:nvPr/>",
        r#"<p:nvPr><p:ph type="body" idx="7"/></p:nvPr>"#,
        1,
    );
    package.set_part(SLIDE_TWO_PART, with_placeholder.into_bytes());
    let mut presentation = Presentation::from_bytes(&package_bytes(package)).unwrap();
    presentation
        .slide_mut(0)
        .unwrap()
        .shape_mut(0)
        .unwrap()
        .set_text("replacement")
        .unwrap();

    let output = open_opc(
        &presentation.to_bytes().unwrap(),
        "F-112 placeholder output",
    );
    let slide = CT_Slide::from_xml(output.get_part(SLIDE_TWO_PART).unwrap()).unwrap();
    let ShapeTreeChild::Shape(shape) = &slide.common_slide_data.shape_tree.children[0] else {
        panic!("expected ordinary shape");
    };
    let placeholder = shape.placeholder.as_ref().unwrap();
    assert_eq!(placeholder.ph_type, Some(PhType::Body));
    assert_eq!(placeholder.idx, Some(7));
}

#[test]
fn text_mutation_preserves_unmodelled_xml_and_schema_order() {
    let mut package = open_opc(&mutation_fixture_bytes(), "F-112 raw fixture");
    let original = String::from_utf8(package.get_part(SLIDE_TWO_PART).unwrap().to_vec()).unwrap();
    let enriched = original.replacen(
        "<a:bodyPr/>",
        r#"<x:before xmlns:x="urn:f112"/><a:bodyPr/><x:after-body xmlns:x="urn:f112"/>"#,
        1,
    ).replacen(
        "<a:r><a:t>plain</a:t></a:r>",
        r#"<mc:AlternateContent xmlns:x="urn:f112"><mc:Choice Requires="x"><x:producer-marker value="kept"/></mc:Choice><mc:Fallback><a:r><a:t>fallback</a:t></a:r></mc:Fallback></mc:AlternateContent><a:fld id="{00000000-0000-0000-0000-000000000001}"><a:t>field</a:t></a:fld><a:br/><x:after-runs xmlns:x="urn:f112"/>"#,
        1,
    );
    package.set_part(SLIDE_TWO_PART, enriched.clone().into_bytes());
    let mut formatted = Presentation::from_bytes(&package_bytes(package)).unwrap();
    let mut slide = formatted.slide_mut(0).unwrap();
    let mut shape = slide.shape_mut(0).unwrap();
    let mut frame = shape.text_frame().unwrap();
    let mut paragraph = frame.paragraph_mut(0).unwrap();
    let mut properties = CT_TextParagraphProperties::default();
    properties.level = Some(1);
    paragraph.set_properties(properties);
    let formatted_output = open_opc(&formatted.to_bytes().unwrap(), "F-112 formatted raw output");
    let formatted_xml =
        String::from_utf8(formatted_output.get_part(SLIDE_TWO_PART).unwrap().to_vec()).unwrap();
    let marker = r#"<mc:AlternateContent xmlns:x="urn:f112"><mc:Choice Requires="x"><x:producer-marker value="kept"/></mc:Choice><mc:Fallback><a:r><a:t>fallback</a:t></a:r></mc:Fallback></mc:AlternateContent>"#;
    let field =
        r#"<a:fld id="{00000000-0000-0000-0000-000000000001}"><a:t>field</a:t></a:fld><a:br/>"#;
    let properties_index = formatted_xml.find("<a:pPr lvl=\"1\"").unwrap();
    let marker_index = formatted_xml.find(marker).unwrap();
    let field_index = formatted_xml.find(field).unwrap();
    assert!(properties_index < marker_index);
    assert!(marker_index < field_index);
    assert!(formatted_xml.contains(field));
    let reparsed_xml = String::from_utf8(
        CT_Slide::from_xml(formatted_output.get_part(SLIDE_TWO_PART).unwrap())
            .unwrap()
            .to_xml()
            .unwrap(),
    )
    .unwrap();
    let properties_index = reparsed_xml.find("<a:pPr lvl=\"1\"").unwrap();
    let marker_index = reparsed_xml.find(marker).unwrap();
    let field_index = reparsed_xml.find(field).unwrap();
    assert!(properties_index < marker_index);
    assert!(marker_index < field_index);

    let mut package = open_opc(&mutation_fixture_bytes(), "F-112 raw replacement fixture");
    package.set_part(SLIDE_TWO_PART, enriched.into_bytes());
    let mut presentation = Presentation::from_bytes(&package_bytes(package)).unwrap();
    presentation
        .slide_mut(0)
        .unwrap()
        .shape_mut(0)
        .unwrap()
        .set_text("ordered")
        .unwrap();

    let output = open_opc(&presentation.to_bytes().unwrap(), "F-112 raw output");
    let xml = String::from_utf8(output.get_part(SLIDE_TWO_PART).unwrap().to_vec()).unwrap();
    for marker in [
        "<x:before ",
        "<x:after-body ",
        "<x:producer-marker ",
        "<x:after-runs ",
    ] {
        assert!(xml.contains(marker), "missing preserved marker {marker}");
    }
    assert!(xml.find("<a:bodyPr").unwrap() < xml.find("<a:p>").unwrap());
    assert!(xml.find("<a:r>").unwrap() < xml.find("<a:t>ordered</a:t>").unwrap());
}

#[test]
fn text_mutation_indices_and_shape_kinds_are_total() {
    let mut presentation = Presentation::from_bytes(&mutation_fixture_bytes()).unwrap();
    let mut slide = presentation.slide_mut(0).unwrap();
    assert!(slide.shape_mut(999).is_none());
    let mut shape = slide.shape_mut(0).unwrap();
    {
        let mut frame = shape.text_frame().unwrap();
        assert!(frame.paragraph_mut(999).is_none());
    }
    let mut picture = slide.shape_mut(1).unwrap();
    assert!(picture.text_frame().is_none());
    assert!(matches!(
        picture.set_text("unsupported"),
        Err(Error::UnsupportedShapeMutation { .. })
    ));
}

#[test]
fn text_frame_handles_append_paragraphs_and_runs_in_order() {
    let mut presentation = Presentation::from_bytes(&mutation_fixture_bytes()).unwrap();
    let mut slide = presentation.slide_mut(0).unwrap();
    let mut shape = slide.shape_mut(0).unwrap();
    shape.set_text("one").unwrap();
    let mut frame = shape.text_frame().unwrap();
    frame.paragraph_mut(0).unwrap().add_run(" two");
    frame.add_paragraph().set_text("three");
    assert_eq!(frame.text(), "one two\nthree");

    let reopened = Presentation::from_bytes(&presentation.to_bytes().unwrap()).unwrap();
    assert_eq!(
        reopened.slide(0).unwrap().shape(0).unwrap().text(),
        Some("one two\nthree".to_owned())
    );
}

#[test]
fn picture_without_explicit_size_uses_native_dimensions() {
    let png = png_header(32, 16);
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add slide");
    let original_count = presentation.slide(0).unwrap().shapes().len();

    let picture = presentation
        .add_picture(0, &png, "image.png", Emu(10), Emu(20), None, None)
        .expect("add native-size picture");
    assert_eq!(picture.kind(), ShapeKind::Picture);

    let bytes = presentation.to_bytes().expect("serialize picture");
    let reopened = Presentation::from_bytes(&bytes).expect("reopen picture");
    assert!(reopened.validate().is_empty());
    let package = open_opc(&bytes, "F-111 native picture");
    let slide = CT_Slide::from_xml(package.get_part("/ppt/slides/slide1.xml").unwrap()).unwrap();
    let ShapeTreeChild::Picture(picture) =
        &slide.common_slide_data.shape_tree.children[original_count]
    else {
        panic!("expected appended picture");
    };
    let transform = picture.shape_properties.transform.as_ref().unwrap();
    assert_eq!(transform.offset.unwrap().x, Emu(10));
    assert_eq!(transform.offset.unwrap().y, Emu(20));
    assert_eq!(transform.extent.unwrap().cx, Emu(406_400));
    assert_eq!(transform.extent.unwrap().cy, Emu(203_200));
}

#[test]
fn picture_one_dimension_preserves_aspect_ratio_with_truncation() {
    let png = png_header(3, 2);
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add slide");
    let original_count = presentation.slide(0).unwrap().shapes().len();
    presentation
        .add_picture(0, &png, "ratio.png", Emu(0), Emu(0), Some(Emu(10)), None)
        .unwrap();
    presentation
        .add_picture(0, &png, "ratio.png", Emu(0), Emu(0), None, Some(Emu(10)))
        .unwrap();

    let package = open_opc(&presentation.to_bytes().unwrap(), "F-111 aspect ratio");
    assert_eq!(
        package
            .get_part_rels("/ppt/slides/slide1.xml")
            .unwrap()
            .get_all_by_type(rel_types::IMAGE)
            .len(),
        1,
        "equal bytes on one slide reuse the slide-scoped relationship"
    );
    let slide = CT_Slide::from_xml(package.get_part("/ppt/slides/slide1.xml").unwrap()).unwrap();
    let pictures = &slide.common_slide_data.shape_tree.children[original_count..];
    let ShapeTreeChild::Picture(width_only) = &pictures[0] else {
        panic!("expected width-only picture");
    };
    let ShapeTreeChild::Picture(height_only) = &pictures[1] else {
        panic!("expected height-only picture");
    };
    let width_only = width_only.shape_properties.transform.as_ref().unwrap();
    let height_only = height_only.shape_properties.transform.as_ref().unwrap();
    assert_eq!(width_only.extent.unwrap().cx, Emu(10));
    assert_eq!(width_only.extent.unwrap().cy, Emu(6));
    assert_eq!(height_only.extent.unwrap().cx, Emu(15));
    assert_eq!(height_only.extent.unwrap().cy, Emu(10));
}

#[test]
fn duplicate_picture_bytes_share_one_media_part_across_slides() {
    let png = png_header(4, 3);
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add first slide");
    presentation.add_slide(0).expect("add second slide");
    for slide_index in 0..2 {
        presentation
            .add_picture(slide_index, &png, "shared.png", Emu(0), Emu(0), None, None)
            .unwrap();
    }

    let bytes = presentation.to_bytes().unwrap();
    let reopened = Presentation::from_bytes(&bytes).unwrap();
    assert!(reopened.validate().is_empty());
    let package = open_opc(&bytes, "F-111 deduplicated media");
    let media = package
        .parts
        .iter()
        .filter(|(part_name, bytes)| part_name.starts_with("/ppt/media/") && *bytes == &png)
        .map(|(part_name, _)| part_name.clone())
        .collect::<Vec<_>>();
    assert_eq!(media.len(), 1);
    for slide_part in ["/ppt/slides/slide1.xml", "/ppt/slides/slide2.xml"] {
        let relationships = package.get_part_rels(slide_part).unwrap();
        let images = relationships.get_all_by_type(rel_types::IMAGE);
        assert_eq!(images.len(), 1);
        assert_eq!(
            OpcPackage::resolve_rel_target(slide_part, &images[0].target),
            media[0]
        );
    }
}

#[test]
fn picture_sniffs_bytes_when_extension_is_misleading() {
    let png = png_header(5, 4);
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add slide");
    presentation
        .add_picture(0, &png, "misleading.jpeg", Emu(0), Emu(0), None, None)
        .unwrap();

    let package = open_opc(&presentation.to_bytes().unwrap(), "F-111 sniffed media");
    let part = package
        .parts
        .iter()
        .find(|(part_name, bytes)| part_name.starts_with("/ppt/media/") && *bytes == &png)
        .map(|(part_name, _)| part_name)
        .unwrap();
    assert!(part.ends_with(".png"));
    assert_eq!(
        package.content_types.content_type_for(part),
        Some("image/png")
    );
}

#[test]
fn picture_with_both_dimensions_does_not_require_native_metadata() {
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="2" height="1"/>"#;
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add slide");
    let original_count = presentation.slide(0).unwrap().shapes().len();
    presentation
        .add_picture(
            0,
            svg,
            "vector.svg",
            Emu(1),
            Emu(2),
            Some(Emu(300)),
            Some(Emu(400)),
        )
        .unwrap();

    let package = open_opc(&presentation.to_bytes().unwrap(), "F-111 explicit size");
    let slide = CT_Slide::from_xml(package.get_part("/ppt/slides/slide1.xml").unwrap()).unwrap();
    let ShapeTreeChild::Picture(picture) =
        &slide.common_slide_data.shape_tree.children[original_count]
    else {
        panic!("expected explicit-size picture");
    };
    let extent = picture
        .shape_properties
        .transform
        .as_ref()
        .unwrap()
        .extent
        .unwrap();
    assert_eq!(extent.cx, Emu(300));
    assert_eq!(extent.cy, Emu(400));
    assert!(package.parts.keys().any(|part| part.ends_with(".svg")));
}

#[test]
fn picture_append_preserves_schema_final_content_and_raw_reserved_ids() {
    const EXTENSION: &str = r#"<p:extLst><p:ext uri="{F110-APPEND-ORDER}"><x:marker xmlns:x="urn:f110" value="kept"/></p:ext></p:extLst>"#;
    let png = png_header(2, 1);
    let mut presentation = Presentation::from_bytes(&append_boundary_fixture_bytes()).unwrap();
    presentation
        .add_picture(0, &png, "ordered.png", Emu(1), Emu(2), None, None)
        .unwrap();

    let package = open_opc(&presentation.to_bytes().unwrap(), "F-111 append boundary");
    let xml = String::from_utf8(package.get_part(SLIDE_TWO_PART).unwrap().to_vec()).unwrap();
    assert!(xml.contains(EXTENSION));
    assert!(xml.rfind("<p:pic>").unwrap() < xml.find(EXTENSION).unwrap());
    let slide = CT_Slide::from_xml(xml.as_bytes()).unwrap();
    let appended = slide.common_slide_data.shape_tree.children.last().unwrap();
    assert_eq!(appended.non_visual_id(), Some(2));
    let ShapeTreeChild::Picture(_) = appended else {
        panic!("expected appended picture");
    };
}

#[test]
fn invalid_picture_input_does_not_mutate_the_presentation() {
    let png = png_header(5, 4);
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add slide");
    let before = presentation.to_bytes().unwrap();

    assert!(matches!(
        presentation.add_picture(1, &png, "image.png", Emu(0), Emu(0), None, None),
        Err(Error::UnknownSlideIndex { index: 1, .. })
    ));
    assert_eq!(presentation.to_bytes().unwrap(), before);

    assert!(matches!(
        presentation.add_picture(0, b"not an image", "image.bin", Emu(0), Emu(0), None, None,),
        Err(Error::UnsupportedPicture { .. })
    ));
    assert_eq!(presentation.to_bytes().unwrap(), before);

    assert!(matches!(
        presentation.add_picture(
            0,
            br#"<svg xmlns="http://www.w3.org/2000/svg"/>"#,
            "image.svg",
            Emu(0),
            Emu(0),
            None,
            None,
        ),
        Err(Error::UnavailablePictureDimensions { .. })
    ));
    assert_eq!(presentation.to_bytes().unwrap(), before);

    assert!(matches!(
        presentation.add_picture(
            0,
            &png_header(1, u32::MAX),
            "too-tall.png",
            Emu(0),
            Emu(0),
            Some(Emu(i64::MAX)),
            None,
        ),
        Err(Error::PictureDimensionOutOfRange { axis: "height", .. })
    ));
    assert_eq!(presentation.to_bytes().unwrap(), before);
}

#[test]
#[ignore = "requires uv and pinned python-pptx 1.0.2"]
fn picture_native_size_matches_python_pptx_1_0_2() {
    let png = valid_one_pixel_png();
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add slide");
    presentation
        .add_picture(0, &png, "pixel.png", Emu(10), Emu(20), None, None)
        .unwrap();
    let output_path = std::env::temp_dir().join(format!(
        "rpptx-f111-python-oracle-{}.pptx",
        std::process::id()
    ));
    fs::write(&output_path, presentation.to_bytes().unwrap()).unwrap();
    let image_hex = png
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let script = r#"
import io
import sys
from pptx import Presentation

ours = Presentation(sys.argv[1])
ours_picture = ours.slides[0].shapes[-1]
print(f"ours\t{ours_picture.shape_type.name}\t{ours_picture.width}\t{ours_picture.height}")

oracle = Presentation()
slide = oracle.slides.add_slide(oracle.slide_layouts[6])
oracle_picture = slide.shapes.add_picture(io.BytesIO(bytes.fromhex(sys.argv[2])), 10, 20)
print(f"oracle\t{oracle_picture.shape_type.name}\t{oracle_picture.width}\t{oracle_picture.height}")
"#;
    let output = Command::new("uv")
        .args([
            "run",
            "--with",
            "python-pptx==1.0.2",
            "python",
            "-c",
            script,
        ])
        .arg(&output_path)
        .arg(image_hex)
        .output()
        .expect("run pinned python-pptx picture oracle");
    fs::remove_file(&output_path).unwrap();
    assert!(
        output.status.success(),
        "python-pptx oracle failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "ours\tPICTURE\t12700\t12700\noracle\tPICTURE\t12700\t12700\n"
    );
}

#[test]
#[ignore = "requires pinned Microsoft PowerPoint"]
fn added_picture_validates_and_opens_without_repair() {
    assert_powerpoint_build();
    let png = valid_one_pixel_png();
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add slide");
    presentation
        .add_picture(
            0,
            &png,
            "pixel.png",
            Emu(914_400),
            Emu(914_400),
            Some(Emu(914_400)),
            Some(Emu(914_400)),
        )
        .unwrap();
    assert!(presentation.validate().is_empty());
    let output =
        std::env::temp_dir().join(format!("rpptx-f111-picture-{}.pptx", std::process::id()));
    fs::write(&output, presentation.to_bytes().expect("serialize deck"))
        .expect("write native acceptance deck");
    let name = output.file_name().unwrap().to_string_lossy();
    let path = output.to_string_lossy();
    let script = format!(
        "with timeout of 120 seconds\ntell application \"Microsoft PowerPoint\"\nset previousStartUpDialog to start up dialog\ntry\nif (Version as text) is not \"{POWERPOINT_VERSION}\" then error \"PowerPoint version mismatch\"\nif (build as text) is not \"{POWERPOINT_APP_BUILD}\" then error \"PowerPoint build mismatch\"\nset start up dialog to false\nset deckPath to \"{path}\"\nopen my POSIX file deckPath\nset checkedDeck to presentation \"{name}\"\nif (count of slides of checkedDeck) is not 1 then error \"slide count mismatch\"\nclose checkedDeck saving no\nset start up dialog to previousStartUpDialog\non error errorMessage number errorNumber\ntry\nclose checkedDeck saving no\nend try\nset start up dialog to previousStartUpDialog\nerror errorMessage number errorNumber\nend try\nend tell\nend timeout\n"
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
fn four_appended_shapes_have_unique_ids_and_reopen() {
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add slide");
    let original_count = presentation.slide(0).unwrap().shapes().len();
    let mut slide = presentation.slide_mut(0).unwrap();
    assert_eq!(
        slide
            .add_textbox(Emu(10), Emu(20), Emu(300), Emu(400))
            .unwrap()
            .kind(),
        ShapeKind::Shape
    );
    assert_eq!(
        slide
            .add_shape("triangle", Emu(30), Emu(40), Emu(500), Emu(600))
            .unwrap()
            .kind(),
        ShapeKind::Shape
    );
    assert_eq!(
        slide
            .add_connector(ConnectorType::Elbow, Emu(700), Emu(800), Emu(100), Emu(200),)
            .unwrap()
            .kind(),
        ShapeKind::Connector
    );
    assert_eq!(slide.add_group_shape().unwrap().kind(), ShapeKind::Group);

    let bytes = presentation
        .to_bytes()
        .expect("serialize constructed shapes");
    let reopened = Presentation::from_bytes(&bytes).expect("reopen constructed shapes");
    assert!(reopened.validate().is_empty());
    let reopened_slide = reopened.slide(0).unwrap();
    let shapes = reopened_slide.shapes().collect::<Vec<_>>();
    assert_eq!(shapes.len(), original_count + 4);
    assert_eq!(shapes[original_count].kind(), ShapeKind::Shape);
    assert_eq!(shapes[original_count + 1].kind(), ShapeKind::Shape);
    assert_eq!(shapes[original_count + 2].kind(), ShapeKind::Connector);
    assert_eq!(shapes[original_count + 3].kind(), ShapeKind::Group);

    let package = open_opc(&bytes, "F-110 constructor reopen");
    let slide = CT_Slide::from_xml(package.get_part("/ppt/slides/slide1.xml").unwrap()).unwrap();
    let appended = &slide.common_slide_data.shape_tree.children[original_count..];
    let ids = appended
        .iter()
        .map(|child| child.non_visual_id().unwrap())
        .collect::<HashSet<_>>();
    assert_eq!(ids.len(), 4);

    let ShapeTreeChild::Shape(textbox) = &appended[0] else {
        panic!("expected textbox shape");
    };
    let transform = textbox.shape_properties.transform.as_ref().unwrap();
    assert_eq!(transform.offset.unwrap().x, Emu(10));
    assert_eq!(transform.offset.unwrap().y, Emu(20));
    assert_eq!(transform.extent.unwrap().cx, Emu(300));
    assert_eq!(transform.extent.unwrap().cy, Emu(400));
    assert_eq!(
        textbox
            .shape_properties
            .preset_geometry
            .as_ref()
            .unwrap()
            .preset,
        "rect"
    );
    assert!(textbox.shape_properties.fill.is_some());
    let text_body = textbox.text_body.as_ref().unwrap();
    assert!(text_body.has_list_style());
    assert_eq!(text_body.paragraph_count(), 1);
    assert!(text_body.paragraphs()[0].runs.is_empty());
    let textbox_xml = String::from_utf8(textbox.to_xml().unwrap()).unwrap();
    assert!(textbox_xml.contains("<p:cNvSpPr txBox=\"1\"/>"));
    assert!(textbox_xml.contains("<a:noFill/>"));

    let ShapeTreeChild::Shape(shape) = &appended[1] else {
        panic!("expected ordinary shape");
    };
    let transform = shape.shape_properties.transform.as_ref().unwrap();
    assert_eq!(transform.offset.unwrap().x, Emu(30));
    assert_eq!(transform.offset.unwrap().y, Emu(40));
    assert_eq!(transform.extent.unwrap().cx, Emu(500));
    assert_eq!(transform.extent.unwrap().cy, Emu(600));
    assert_eq!(
        shape
            .shape_properties
            .preset_geometry
            .as_ref()
            .unwrap()
            .preset,
        "triangle"
    );
    assert!(shape.shape_properties.fill.is_none());
    let text_body = shape.text_body.as_ref().unwrap();
    assert!(text_body.has_list_style());
    assert_eq!(text_body.paragraph_count(), 1);
    assert!(text_body.paragraphs()[0].runs.is_empty());

    let ShapeTreeChild::Connector(connector) = &appended[2] else {
        panic!("expected connector");
    };
    let transform = connector.shape_properties.transform.as_ref().unwrap();
    assert_eq!(transform.offset.unwrap().x, Emu(100));
    assert_eq!(transform.offset.unwrap().y, Emu(200));
    assert_eq!(transform.extent.unwrap().cx, Emu(600));
    assert_eq!(transform.extent.unwrap().cy, Emu(600));
    assert!(transform.flip_horizontal);
    assert!(transform.flip_vertical);
    assert_eq!(
        connector
            .shape_properties
            .preset_geometry
            .as_ref()
            .unwrap()
            .preset,
        "bentConnector3"
    );

    let ShapeTreeChild::GroupShape(group) = &appended[3] else {
        panic!("expected empty group");
    };
    assert!(group.children.is_empty());
    assert!(group.group_transform().is_none());
    let group_xml = String::from_utf8(group.to_xml().unwrap()).unwrap();
    assert!(group_xml.contains("<p:cNvGrpSpPr/>"));
    assert!(group_xml.contains("<p:nvPr/>"));
    assert!(group_xml.contains("<p:grpSpPr/>"));
}

#[test]
fn unknown_preset_does_not_mutate_the_slide() {
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add slide");
    let before = presentation.to_bytes().unwrap();

    let error = match presentation.slide_mut(0).unwrap().add_shape(
        "not-a-preset",
        Emu(1),
        Emu(2),
        Emu(3),
        Emu(4),
    ) {
        Ok(_) => panic!("unknown preset was accepted"),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        Error::InvalidShapeMutation {
            operation: "add shape",
            ref message,
        } if message.contains("unknown DrawingML preset geometry: not-a-preset")
    ));
    assert_eq!(presentation.to_bytes().unwrap(), before);
}

#[test]
fn every_constructor_appends_before_preserved_shape_tree_extensions() {
    const EXTENSION: &str = r#"<p:extLst><p:ext uri="{F110-APPEND-ORDER}"><x:marker xmlns:x="urn:f110" value="kept"/></p:ext></p:extLst>"#;

    for constructor in 0..4 {
        let mut presentation = Presentation::from_bytes(&append_boundary_fixture_bytes()).unwrap();
        let original_count = presentation.slide(0).unwrap().shapes().len();
        let mut slide = presentation.slide_mut(0).unwrap();
        let expected_tag = match constructor {
            0 => {
                slide.add_textbox(Emu(1), Emu(2), Emu(3), Emu(4)).unwrap();
                "<p:sp>"
            }
            1 => {
                slide
                    .add_shape("triangle", Emu(1), Emu(2), Emu(3), Emu(4))
                    .unwrap();
                "<p:sp>"
            }
            2 => {
                slide
                    .add_connector(ConnectorType::Straight, Emu(1), Emu(2), Emu(3), Emu(4))
                    .unwrap();
                "<p:cxnSp>"
            }
            3 => {
                slide.add_group_shape().unwrap();
                "<p:grpSp>"
            }
            _ => unreachable!(),
        };

        let bytes = presentation.to_bytes().unwrap();
        let package = open_opc(&bytes, "F-110 append boundary");
        let xml = String::from_utf8(package.get_part(SLIDE_TWO_PART).unwrap().to_vec()).unwrap();
        assert!(xml.contains(EXTENSION));
        assert!(xml.rfind(expected_tag).unwrap() < xml.find(EXTENSION).unwrap());
        let reparsed = CT_Slide::from_xml(xml.as_bytes()).unwrap();
        assert_eq!(
            reparsed.common_slide_data.shape_tree.children.len(),
            original_count + 1
        );
    }
}

#[test]
fn constructor_names_are_deterministic_from_allocated_ids() {
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add slide");
    let original_count = presentation.slide(0).unwrap().shapes().len();
    let mut slide = presentation.slide_mut(0).unwrap();
    slide.add_textbox(Emu(1), Emu(2), Emu(3), Emu(4)).unwrap();
    slide
        .add_shape("rect", Emu(5), Emu(6), Emu(7), Emu(8))
        .unwrap();
    slide
        .add_connector(ConnectorType::Straight, Emu(9), Emu(10), Emu(11), Emu(12))
        .unwrap();
    slide.add_group_shape().unwrap();

    let bytes = presentation.to_bytes().unwrap();
    let package = open_opc(&bytes, "F-110 deterministic names");
    let xml =
        String::from_utf8(package.get_part("/ppt/slides/slide1.xml").unwrap().to_vec()).unwrap();
    let slide = CT_Slide::from_xml(xml.as_bytes()).unwrap();
    let appended = &slide.common_slide_data.shape_tree.children[original_count..];
    for (child, prefix) in appended
        .iter()
        .zip(["TextBox", "Shape", "Connector", "Group"])
    {
        let id = child.non_visual_id().unwrap();
        assert!(xml.contains(&format!("name=\"{prefix} {id}\"")));
    }
}

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
#[ignore = "requires pinned Microsoft PowerPoint"]
fn all_shape_constructors_open_in_powerpoint_without_repair() {
    assert_powerpoint_build();
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation.add_slide(0).expect("add slide");
    let mut package = open_opc(
        &presentation.to_bytes().unwrap(),
        "F-110 native append boundary",
    );
    let original =
        String::from_utf8(package.get_part("/ppt/slides/slide1.xml").unwrap().to_vec()).unwrap();
    let with_extension = original.replacen(
        "</p:spTree>",
        r#"<p:extLst><p:ext uri="{F110-NATIVE-APPEND-ORDER}"><x:marker xmlns:x="urn:f110" value="kept"/></p:ext></p:extLst></p:spTree>"#,
        1,
    );
    assert_ne!(with_extension, original);
    package.set_part("/ppt/slides/slide1.xml", with_extension.into_bytes());
    let mut presentation = Presentation::from_bytes(&package_bytes(package)).unwrap();
    let mut slide = presentation.slide_mut(0).unwrap();
    slide
        .add_textbox(Emu(914_400), Emu(914_400), Emu(2_000_000), Emu(900_000))
        .unwrap();
    slide
        .add_shape(
            "triangle",
            Emu(3_000_000),
            Emu(914_400),
            Emu(2_000_000),
            Emu(2_000_000),
        )
        .unwrap();
    slide
        .add_connector(
            ConnectorType::Curve,
            Emu(1_000_000),
            Emu(4_000_000),
            Emu(5_000_000),
            Emu(3_000_000),
        )
        .unwrap();
    slide.add_group_shape().unwrap();
    let output = std::env::temp_dir().join(format!(
        "rpptx-f110-shape-constructors-{}.pptx",
        std::process::id()
    ));
    fs::write(&output, presentation.to_bytes().expect("serialize deck"))
        .expect("write native acceptance deck");
    let name = output.file_name().unwrap().to_string_lossy();
    let path = output.to_string_lossy();
    let script = format!(
        "with timeout of 120 seconds\ntell application \"Microsoft PowerPoint\"\nset previousStartUpDialog to start up dialog\ntry\nif (Version as text) is not \"{POWERPOINT_VERSION}\" then error \"PowerPoint version mismatch\"\nif (build as text) is not \"{POWERPOINT_APP_BUILD}\" then error \"PowerPoint build mismatch\"\nset start up dialog to false\nset deckPath to \"{path}\"\nopen my POSIX file deckPath\nset checkedDeck to presentation \"{name}\"\nif (count of slides of checkedDeck) is not 1 then error \"slide count mismatch\"\nclose checkedDeck saving no\nset start up dialog to previousStartUpDialog\non error errorMessage number errorNumber\ntry\nclose checkedDeck saving no\nend try\nset start up dialog to previousStartUpDialog\nerror errorMessage number errorNumber\nend try\nend tell\nend timeout\n"
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
fn rpptx_is_an_explicit_publication_candidate() {
    let workspace = include_str!("../../../Cargo.toml");
    let manifest = include_str!("../Cargo.toml");
    assert!(workspace.contains("\"crates/rpptx\""));
    assert!(workspace.contains("rpptx = { path = \"crates/rpptx\", version = \"0.1.2\" }"));
    assert!(manifest.contains("name = \"rpptx\""));
    assert!(manifest.contains("version = \"0.1.2\""));
    assert!(manifest.contains("publish = true"));
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
        ResolvedContent::None | ResolvedContent::Image(_) | ResolvedContent::Group(_) => {}
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
        ResolvedContent::Group(group) => {
            let mut text = Vec::new();
            walk(&group.children, &mut |element, _| {
                if let PositionedElement::Text(run) = element {
                    text.push(run.text.clone());
                }
            });
            text.join("\n")
        }
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

fn deterministic_render(bytes: &[u8]) -> Vec<u8> {
    let layout = render_presentation_package(bytes);
    oxml_pdf::render_page_to_png(&layout, 0, 72.0).unwrap()
}

fn render_presentation_package(bytes: &[u8]) -> oxml_layout::LayoutResult {
    let package = open_opc(bytes, "F-112 deterministic render");
    render_deck_example::render_package(&package, Path::new("F-128 integration package"))
        .unwrap()
        .layout
}

fn presentation_with_three_dimensional_chart_fallback(preview: Option<(&[u8], &str)>) -> Vec<u8> {
    let bytes = presentation_with_authored_chart().to_bytes().unwrap();
    let mut package = open_opc(&bytes, "three-dimensional chart fallback");
    let chart_part = package
        .content_types
        .overrides
        .iter()
        .find_map(|(part, content_type)| {
            (content_type == content_types::CHART).then_some(part.clone())
        })
        .unwrap();
    package.set_part(
        &chart_part,
        br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart><c:plotArea><c:bar3DChart><c:barDir val="col"/></c:bar3DChart></c:plotArea></c:chart></c:chartSpace>"#.to_vec(),
    );
    let slide_part = package
        .content_types
        .overrides
        .iter()
        .find_map(|(part, content_type)| {
            (content_type == content_types::SLIDE).then_some(part.clone())
        })
        .unwrap();
    let preview_target = "../media/f128-chart-preview.png";
    let preview_id = package
        .get_or_create_part_rels(&slide_part)
        .add(rel_types::IMAGE, preview_target);
    let slide_xml = String::from_utf8(package.get_part(&slide_part).unwrap().to_vec()).unwrap();
    let frame_start = slide_xml.find("<p:graphicFrame").unwrap();
    let frame_close = "</p:graphicFrame>";
    let frame_end =
        frame_start + slide_xml[frame_start..].find(frame_close).unwrap() + frame_close.len();
    let frame = &slide_xml[frame_start..frame_end];
    let fallback_picture = format!(
        r#"<p:pic><p:nvPicPr><p:cNvPr/><p:cNvPicPr/><p:nvPr/></p:nvPicPr><p:blipFill><a:blip xmlns:r="{R_NS}" r:embed="{preview_id}"/><a:stretch><a:fillRect/></a:stretch></p:blipFill><p:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="1" cy="1"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr></p:pic>"#
    );
    let alternate = format!(
        r#"<mc:AlternateContent xmlns:mc="{MC_NS}"><mc:Choice xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" Requires="c">{frame}</mc:Choice><mc:Fallback>{fallback_picture}</mc:Fallback></mc:AlternateContent>"#
    );
    package.set_part(
        &slide_part,
        format!(
            "{}{}{}",
            &slide_xml[..frame_start],
            alternate,
            &slide_xml[frame_end..]
        )
        .into_bytes(),
    );
    if let Some((preview, content_type)) = preview {
        let preview_part = OpcPackage::resolve_rel_target(&slide_part, preview_target);
        package.set_part(&preview_part, preview.to_vec());
        package
            .content_types
            .add_override(&preview_part, content_type);
    }
    let mut output = Cursor::new(Vec::new());
    package.write_to(&mut output).unwrap();
    output.into_inner()
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

fn build_f116_ten_slide_deck() -> Presentation {
    let mut presentation = Presentation::new().expect("open bundled template");
    presentation
        .set_slide_size(Emu(12_800_000), Emu(7_200_000))
        .expect("set F-116 slide size");
    let core = presentation.core_properties_mut();
    core.title = Some("rdocx M11 cross-viewer acceptance".to_owned());
    core.creator = Some("rdocx deterministic acceptance generator".to_owned());
    core.subject = Some("F-107 through F-115".to_owned());

    for number in 1..=10 {
        presentation.add_slide(0).expect("add F-116 slide");
        let mut slide = presentation.slide_mut(number - 1).unwrap();
        slide
            .add_textbox(Emu(450_000), Emu(220_000), Emu(5_600_000), Emu(650_000))
            .expect("add slide label")
            .set_text(&format!("F-116 slide {number:02}"))
            .expect("set slide label");
    }

    let blue_fill = Fill::from_xml(
        br#"<a:solidFill xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:srgbClr val="2F5597"/></a:solidFill>"#,
    )
    .unwrap();
    let green_fill = Fill::from_xml(
        br#"<a:solidFill xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:srgbClr val="70AD47"/></a:solidFill>"#,
    )
    .unwrap();
    let line = CT_LineProperties::from_xml(
        br#"<a:ln xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" w="25400"><a:solidFill><a:srgbClr val="203864"/></a:solidFill></a:ln>"#,
    )
    .unwrap();

    {
        let mut slide = presentation.slide_mut(1).unwrap();
        let mut shape = slide
            .add_shape(
                "roundRect",
                Emu(900_000),
                Emu(1_250_000),
                Emu(3_200_000),
                Emu(1_650_000),
            )
            .expect("add mutable shape");
        shape.set_position(Emu(1_000_000), Emu(1_350_000)).unwrap();
        shape.set_size(Emu(3_400_000), Emu(1_800_000)).unwrap();
        shape.set_rotation(Angle(600_000)).unwrap();
        shape.set_name("M11 mutable shape").unwrap();
        shape.set_fill(blue_fill.clone()).unwrap();
        shape.set_line(line).unwrap();
        shape.set_adjust_value("adj", 30_000.0).unwrap();
        shape
            .set_text("Position, size, rotation, fill, line, and adjustment")
            .unwrap();
    }

    {
        let mut slide = presentation.slide_mut(2).unwrap();
        slide
            .add_textbox(Emu(700_000), Emu(1_200_000), Emu(2_900_000), Emu(800_000))
            .unwrap()
            .set_text("Textbox constructor")
            .unwrap();
        slide
            .add_shape(
                "triangle",
                Emu(4_000_000),
                Emu(1_200_000),
                Emu(1_800_000),
                Emu(1_800_000),
            )
            .unwrap();
        slide
            .add_connector(
                ConnectorType::Straight,
                Emu(900_000),
                Emu(3_600_000),
                Emu(3_000_000),
                Emu(4_300_000),
            )
            .unwrap();
        slide
            .add_connector(
                ConnectorType::Elbow,
                Emu(3_300_000),
                Emu(3_600_000),
                Emu(5_400_000),
                Emu(4_300_000),
            )
            .unwrap();
        slide
            .add_connector(
                ConnectorType::Curve,
                Emu(5_700_000),
                Emu(3_600_000),
                Emu(7_800_000),
                Emu(4_300_000),
            )
            .unwrap();
        slide.add_group_shape().unwrap();
    }

    let png = valid_one_pixel_png();
    presentation
        .add_picture(
            3,
            &png,
            "f116-pixel.png",
            Emu(1_000_000),
            Emu(1_300_000),
            Some(Emu(2_000_000)),
            Some(Emu(2_000_000)),
        )
        .expect("add F-116 picture");

    {
        let mut slide = presentation.slide_mut(4).unwrap();
        let mut shape = slide
            .add_textbox(Emu(800_000), Emu(1_250_000), Emu(7_000_000), Emu(2_600_000))
            .unwrap();
        shape.set_text("Formatted paragraph").unwrap();
        let mut frame = shape.text_frame().unwrap();
        let mut paragraph = frame.paragraph_mut(0).unwrap();
        let mut paragraph_properties = CT_TextParagraphProperties::default();
        paragraph_properties.level = Some(1);
        paragraph.set_properties(paragraph_properties);
        paragraph.set_bullet(Some(TextBullet {
            choice: Some(TextBulletChoice::Character(
                TextBulletCharacter::new("•").unwrap(),
            )),
            ..TextBullet::default()
        }));
        let mut run = paragraph.add_run(" with a bold Carlito run");
        let mut run_properties = CT_TextCharacterProperties::default();
        run_properties.font_size = Some(2_000);
        run_properties.bold = Some(true);
        run.set_properties(run_properties);
        run.set_font(Some(TextFont::new("Carlito").unwrap()));
        frame
            .add_paragraph()
            .set_text("Second paragraph exercises structural append");
    }

    {
        let mut slide = presentation.slide_mut(5).unwrap();
        let mut shape = slide
            .add_table(
                3,
                3,
                Emu(750_000),
                Emu(1_250_000),
                Emu(7_500_000),
                Emu(3_600_000),
            )
            .expect("add F-116 table");
        let mut table = shape.table_mut().unwrap();
        table.set_first_row(true);
        table.set_last_column(true);
        table.set_horizontal_banding(true);
        table.set_vertical_banding(true);
        table.set_column_width(2, Emu(2_700_000)).unwrap();
        for row in 0..3 {
            for column in 0..3 {
                table.cell_mut(row, column).unwrap().set_text(&format!(
                    "R{}C{}",
                    row + 1,
                    column + 1
                ));
            }
        }
        let mut first = table.cell_mut(0, 0).unwrap();
        first.set_fill(Some(green_fill.clone()));
        first.set_margins(
            Some(Emu(72_000)),
            Some(Emu(72_000)),
            Some(Emu(36_000)),
            Some(Emu(36_000)),
        );
        first.merge_to(1, 1).unwrap();
        table.cell_mut(2, 1).unwrap().merge_to(2, 2).unwrap();
        table.cell_mut(2, 1).unwrap().split().unwrap();
    }

    {
        let mut slide = presentation.slide_mut(6).unwrap();
        slide.set_hidden(true);
        slide
            .set_background(green_fill.clone())
            .expect("set F-116 background");
    }

    presentation
        .add_picture(
            7,
            &png,
            "f116-reused-pixel.png",
            Emu(1_000_000),
            Emu(1_300_000),
            Some(Emu(1_500_000)),
            Some(Emu(1_500_000)),
        )
        .expect("add duplicated-slide picture");

    {
        let mut slide = presentation.slide_mut(8).unwrap();
        slide.set_background(blue_fill).unwrap();
        slide.clear_background();
    }

    presentation
        .duplicate_slide(7)
        .expect("duplicate relationship-bearing slide");
    presentation
        .remove_slide(0)
        .expect("remove collection slide");
    presentation
        .move_slide(9, 0)
        .expect("move final slide first");
    presentation
}

fn assert_f116_deck_structure(presentation: &Presentation) {
    assert_eq!(presentation.len(), 10);
    assert!(
        presentation.validate().is_empty(),
        "F-116 validation issues: {:?}",
        presentation.validate()
    );
    assert_eq!(
        presentation.slide_size(),
        Some((Emu(12_800_000), Emu(7_200_000)))
    );
    assert_eq!(
        presentation
            .core_properties()
            .and_then(|properties| properties.title.as_deref()),
        Some("rdocx M11 cross-viewer acceptance")
    );
    for (slide, expected_title) in presentation.slides().zip(F116_FINAL_TITLES) {
        assert!(
            slide.text().contains(expected_title),
            "slide {} lacks {expected_title:?}",
            slide.id()
        );
    }
    assert_eq!(
        presentation
            .slides()
            .map(|slide| slide.id())
            .collect::<HashSet<_>>()
            .len(),
        10
    );
    assert!(presentation.slide(6).unwrap().hidden());
    assert!(presentation.slide(6).unwrap().has_explicit_background());
    let constructor_kinds = presentation
        .slide(2)
        .unwrap()
        .shapes()
        .map(|shape| shape.kind())
        .collect::<Vec<_>>();
    for kind in [ShapeKind::Shape, ShapeKind::Connector, ShapeKind::Group] {
        assert!(constructor_kinds.contains(&kind));
    }
    assert!(
        presentation
            .slide(5)
            .unwrap()
            .shapes()
            .any(|shape| shape.table().is_some())
    );
    assert_eq!(
        presentation
            .slides()
            .map(|slide| {
                slide
                    .shapes()
                    .filter(|shape| shape.kind() == ShapeKind::Picture)
                    .count()
            })
            .sum::<usize>(),
        3
    );
}

fn write_f116_candidate() -> PathBuf {
    let presentation = build_f116_ten_slide_deck();
    assert_f116_deck_structure(&presentation);
    let path = PathBuf::from(F116_CANDIDATE_PATH);
    presentation
        .save(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    path
}

fn write_f116_temporary_candidate(label: &str) -> PathBuf {
    let presentation = build_f116_ten_slide_deck();
    assert_f116_deck_structure(&presentation);
    let path = f116_temp_path(label, "pptx");
    presentation
        .save(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    path
}

fn f116_temp_path(label: &str, extension: &str) -> PathBuf {
    let ordinal = F116_TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "rdocx-f116-{label}-{}-{ordinal}.{extension}",
        std::process::id()
    ))
}

fn assert_clean_cross_viewer_evidence(row: CrossViewerEvidence) -> Result<(), String> {
    let CrossViewerObservation::Clean {
        opened_or_imported,
        observed_slide_count,
        no_repair_or_conversion_error,
        export_or_close_result,
    } = row.observation
    else {
        return Err(format!("{} evidence is pending", row.application));
    };
    if row
        .version_or_date
        .is_none_or(|value| value.trim().is_empty())
    {
        return Err(format!(
            "{} lacks a nonblank version or acceptance date",
            row.application
        ));
    }
    if row.build.is_none_or(|value| value.trim().is_empty()) {
        return Err(format!("{} lacks a nonblank build", row.application));
    }
    if !opened_or_imported {
        return Err(format!(
            "{} lacks a positive open or import",
            row.application
        ));
    }
    if observed_slide_count != 10 {
        return Err(format!(
            "{} observed {observed_slide_count} slides instead of 10",
            row.application
        ));
    }
    if !no_repair_or_conversion_error {
        return Err(format!(
            "{} observed a repair or conversion error",
            row.application
        ));
    }
    if export_or_close_result.trim().is_empty() {
        return Err(format!(
            "{} lacks an export or close result",
            row.application
        ));
    }
    Ok(())
}

fn assert_f116_powerpoint_acceptance(path: &Path) {
    assert_powerpoint_build();
    let name = path.file_name().unwrap().to_string_lossy();
    let path = path.to_string_lossy();
    let script = format!(
        "with timeout of 120 seconds\ntell application \"Microsoft PowerPoint\"\nset previousStartUpDialog to start up dialog\ntry\nif (Version as text) is not \"{POWERPOINT_VERSION}\" then error \"PowerPoint version mismatch\"\nif (build as text) is not \"{POWERPOINT_APP_BUILD}\" then error \"PowerPoint build mismatch\"\nset start up dialog to false\nset deckPath to \"{path}\"\nopen my POSIX file deckPath\nset checkedDeck to presentation \"{name}\"\nif (count of slides of checkedDeck) is not 10 then error \"slide count mismatch\"\nclose checkedDeck saving no\nset start up dialog to previousStartUpDialog\non error errorMessage number errorNumber\ntry\nclose checkedDeck saving no\nend try\nset start up dialog to previousStartUpDialog\nerror errorMessage number errorNumber\nend try\nend tell\nend timeout\n"
    );
    let result = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .expect("launch PowerPoint F-116 acceptance script");
    assert!(
        result.status.success(),
        "PowerPoint F-116 acceptance failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

fn assert_f116_libreoffice_acceptance(path: &Path) {
    let version = Command::new("soffice")
        .arg("--version")
        .output()
        .expect("read LibreOffice version");
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap().trim(),
        LIBREOFFICE_VERSION
    );

    let output_dir = PathBuf::from(format!(
        "/private/tmp/rdocx-f116-libreoffice-{}",
        std::process::id()
    ));
    let profile_dir = PathBuf::from(format!(
        "/private/tmp/rdocx-f116-libreoffice-profile-{}",
        std::process::id()
    ));
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).expect("remove stale F-116 LibreOffice output");
    }
    if profile_dir.exists() {
        fs::remove_dir_all(&profile_dir).expect("remove stale F-116 LibreOffice profile");
    }
    fs::create_dir_all(&output_dir).expect("create F-116 LibreOffice output");
    let profile = format!("-env:UserInstallation=file://{}", profile_dir.display());
    let filter =
        r#"pdf:impress_pdf_Export:{"ExportHiddenSlides":{"type":"boolean","value":"true"}}"#;
    let result = Command::new("soffice")
        .args(["--headless", &profile, "--convert-to", filter, "--outdir"])
        .arg(&output_dir)
        .arg(path)
        .output()
        .expect("run LibreOffice F-116 acceptance");
    assert!(
        result.status.success(),
        "LibreOffice F-116 import or export failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let pdf = output_dir.join("rdocx-f116-m11-write-api.pdf");
    assert!(
        pdf.is_file(),
        "LibreOffice did not create {}",
        pdf.display()
    );
    let info = Command::new("pdfinfo")
        .arg(&pdf)
        .output()
        .expect("inspect LibreOffice F-116 PDF");
    assert!(
        info.status.success(),
        "pdfinfo failed: {}",
        String::from_utf8_lossy(&info.stderr)
    );
    let info = String::from_utf8(info.stdout).unwrap();
    let pages = info
        .lines()
        .find_map(|line| line.strip_prefix("Pages:").map(str::trim))
        .and_then(|value| value.parse::<usize>().ok());
    assert_eq!(pages, Some(10));
    fs::remove_dir_all(&output_dir).expect("remove F-116 LibreOffice output");
    if profile_dir.exists() {
        fs::remove_dir_all(&profile_dir).expect("remove F-116 LibreOffice profile");
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
    let complete = complete.replacen(
        "<p:cNvCxnSpPr/>",
        r#"<p:cNvCxnSpPr><a:stCxn id="2" idx="0"/><a:endCxn id="3" idx="1"/></p:cNvCxnSpPr>"#,
        1,
    );
    let complete = complete
        .replacen(
            r#"<mc:Choice Requires="p14"><p:sp/></mc:Choice>"#,
            r#"<mc:Choice Requires="p14"><p:sp><p:nvSpPr><p:cNvPr id="20" name="choice shape"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/></p:sp><p:cxnSp><p:nvCxnSpPr><p:cNvPr id="21" name="choice connector"/><p:cNvCxnSpPr><a:stCxn id="20" idx="0"/><a:endCxn id="20" idx="1"/></p:cNvCxnSpPr><p:nvPr/></p:nvCxnSpPr><p:spPr/></p:cxnSp></mc:Choice>"#,
            1,
        )
        .replacen(
            "<mc:Fallback>",
            r#"<mc:Fallback><p:sp><p:nvSpPr><p:cNvPr id="22" name="fallback shape"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/></p:sp><p:cxnSp><p:nvCxnSpPr><p:cNvPr id="23" name="fallback connector"/><p:cNvCxnSpPr><a:stCxn id="22" idx="0"/><a:endCxn id="22" idx="1"/></p:cNvCxnSpPr><p:nvPr/></p:nvCxnSpPr><p:spPr/></p:cxnSp>"#,
            1,
        );
    assert_ne!(complete, original);
    package.set_part(SLIDE_TWO_PART, complete.into_bytes());
    package_bytes(package)
}

fn table_mutation_fixture_bytes() -> Vec<u8> {
    let mut package = fixture_package();
    let original = String::from_utf8(package.get_part(SLIDE_TWO_PART).unwrap().to_vec()).unwrap();
    let marked = original
        .replacen(
            "<a:tbl>",
            r#"<a:tbl xmlns:x="urn:f113" producer="kept"><x:before-table/><x:raw-payload xmlns:z="urn:f113:attributes" z:second="2" first="1">
  one &amp; two<!-- preserve spacing --><?f113 raw?><x:nested z:value=" A &amp; B "/>
</x:raw-payload>"#,
            1,
        )
        .replacen(
            r#"<a:tblGrid><a:gridCol w="100"/><a:gridCol w="100"/></a:tblGrid>"#,
            r#"<a:tblGrid x:grid="kept"><x:before-column/><a:gridCol w="100" x:column="kept"><x:column-child/></a:gridCol><a:gridCol w="101"/></a:tblGrid><x:after-grid/>"#,
            1,
        )
        .replacen(
            r#"<a:tr h="100"><a:tc><a:txBody>"#,
            r#"<a:tr h="100" x:row="kept"><x:before-cell/><a:tc x:cell="kept"><x:before-text/><a:txBody>"#,
            1,
        )
        .replacen(
            "<a:tcPr/>",
            r#"<a:tcPr x:cell-properties="kept"><x:cell-properties-child/></a:tcPr>"#,
            1,
        );
    assert_ne!(marked, original);
    package.set_part(SLIDE_TWO_PART, marked.into_bytes());
    package_bytes(package)
}

fn append_boundary_fixture_bytes() -> Vec<u8> {
    let mut package = fixture_package();
    let original = String::from_utf8(package.get_part(SLIDE_TWO_PART).unwrap().to_vec()).unwrap();
    let with_extension = original.replacen(
        "</p:spTree>",
        r#"<p:extLst><p:ext uri="{F110-APPEND-ORDER}"><x:marker xmlns:x="urn:f110" value="kept"/></p:ext></p:extLst></p:spTree>"#,
        1,
    );
    assert_ne!(with_extension, original);
    package.set_part(SLIDE_TWO_PART, with_extension.into_bytes());
    package_bytes(package)
}

fn package_bytes(package: OpcPackage) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    package.write_to(&mut output).unwrap();
    output.into_inner()
}

fn assert_notes_back_relationship_is_normalized(
    mut package: OpcPackage,
    source_back_relationship_ids: &HashSet<String>,
) {
    let unrelated_type = "urn:f114:unrelated";
    package
        .get_or_create_part_rels(NOTES_PART)
        .add_external(unrelated_type, "https://example.com/preserved");
    let mut presentation = Presentation::from_bytes(&package_bytes(package)).unwrap();
    presentation.duplicate_slide(0).expect("duplicate slide");

    let bytes = presentation.to_bytes().unwrap();
    let package = open_opc(&bytes, "normalized notes relationships");
    let model = CT_Presentation::from_xml(package.get_part(PRESENTATION_PART).unwrap()).unwrap();
    let duplicate_relationship = package
        .get_part_rels(PRESENTATION_PART)
        .unwrap()
        .get_by_id(&model.slide_ids[1].relationship_id)
        .unwrap();
    let duplicate_slide =
        OpcPackage::resolve_rel_target(PRESENTATION_PART, &duplicate_relationship.target);
    let notes_relationship = package
        .get_part_rels(&duplicate_slide)
        .unwrap()
        .items
        .iter()
        .find(|relationship| relationship.rel_type == rel_types::NOTES_SLIDE)
        .unwrap();
    let duplicate_notes =
        OpcPackage::resolve_rel_target(&duplicate_slide, &notes_relationship.target);
    let notes_relationships = package.get_part_rels(&duplicate_notes).unwrap();
    let back_relationships = notes_relationships
        .items
        .iter()
        .filter(|relationship| relationship.rel_type == rel_types::SLIDE)
        .collect::<Vec<_>>();
    assert_eq!(back_relationships.len(), 1);
    assert_eq!(back_relationships[0].target_mode, None);
    assert_eq!(
        OpcPackage::resolve_rel_target(&duplicate_notes, &back_relationships[0].target),
        duplicate_slide
    );
    assert!(!source_back_relationship_ids.contains(&back_relationships[0].id));
    let preserved = notes_relationships
        .items
        .iter()
        .find(|relationship| relationship.rel_type == unrelated_type)
        .unwrap();
    assert_eq!(preserved.target, "https://example.com/preserved");
    assert_eq!(preserved.target_mode.as_deref(), Some("External"));
    assert!(
        Presentation::from_bytes(&bytes)
            .unwrap()
            .validate()
            .is_empty()
    );
}

fn capture_exact_subtree(xml: &[u8], opening: &[u8], closing: &[u8]) -> Vec<u8> {
    let start = xml
        .windows(opening.len())
        .position(|window| window == opening)
        .expect("opening marker exists");
    let relative_end = xml[start..]
        .windows(closing.len())
        .position(|window| window == closing)
        .expect("closing marker exists");
    let end = start + relative_end + closing.len();
    xml[start..end].to_vec()
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
    package.get_or_create_part_rels(NOTES_PART).add_with_id(
        "slide-link",
        rel_types::SLIDE,
        "../slides/second.xml",
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
