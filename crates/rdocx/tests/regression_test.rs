//! Regression tests for previously-fixed defects.
//!
//! Each test names the failure it locks down, so a reintroduction is obvious
//! from the test name alone rather than from a diff.

use std::collections::HashMap;

use rdocx::{Document, Length};

#[test]
fn mislabelled_jpeg_uses_sniffed_package_metadata() {
    let jpeg = [0xff, 0xd8, 0xff, 0xd9];
    let mut seed = Document::new();
    let seed_bytes = seed.to_bytes().unwrap();
    let mut seed_package =
        oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(seed_bytes)).unwrap();
    seed_package
        .content_types
        .add_default("jpeg", "application/octet-stream");
    let mut reopened_bytes = std::io::Cursor::new(Vec::new());
    seed_package.write_to(&mut reopened_bytes).unwrap();
    let mut document = Document::from_bytes(&reopened_bytes.into_inner()).unwrap();

    document.add_picture(
        &jpeg,
        "mislabelled.png",
        Length::inches(1.0),
        Length::inches(1.0),
    );

    let bytes = document.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(bytes)).unwrap();
    let part_name = "/word/media/image1.jpeg";

    assert_eq!(package.get_part(part_name), Some(jpeg.as_slice()));
    assert_eq!(
        package.content_types.content_type_for(part_name),
        Some("image/jpeg")
    );

    let image_relationship = package
        .get_part_rels("/word/document.xml")
        .and_then(|relationships| {
            relationships.get_by_type(oxml_opc::relationship::rel_types::IMAGE)
        })
        .expect("document should relate the sniffed image part");
    assert_eq!(image_relationship.target, "media/image1.jpeg");
}

#[test]
fn next_image_name_uses_the_highest_existing_index_not_the_part_count() {
    let cases: &[(&[&str], &str)] = &[
        (
            &["/word/media/image1.png", "/word/media/image5.png"],
            "/word/media/image6.png",
        ),
        (
            &[
                "/word/media/image1.png",
                "/word/media/image2.png",
                "/word/media/image4.png",
            ],
            "/word/media/image5.png",
        ),
    ];

    for (existing_names, expected_name) in cases {
        let mut seed = Document::new();
        let seed_bytes = seed.to_bytes().unwrap();
        let mut package =
            oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(seed_bytes)).unwrap();
        for name in *existing_names {
            package.set_part(name, vec![0xAA]);
        }

        let mut package_bytes = std::io::Cursor::new(Vec::new());
        package.write_to(&mut package_bytes).unwrap();
        let mut reopened = Document::from_bytes(&package_bytes.into_inner()).unwrap();
        reopened.add_picture(
            &[0x11, 0x22, 0x33],
            "added.png",
            Length::inches(1.0),
            Length::inches(1.0),
        );

        let saved = reopened.to_bytes().unwrap();
        let saved_package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(saved)).unwrap();
        assert_eq!(
            saved_package.get_part(expected_name),
            Some([0x11, 0x22, 0x33].as_slice()),
            "existing names: {existing_names:?}"
        );
    }
}

#[test]
fn malformed_media_names_do_not_change_the_highest_image_index() {
    let mut seed = Document::new();
    let seed_bytes = seed.to_bytes().unwrap();
    let mut package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(seed_bytes)).unwrap();
    for name in [
        "/word/media/image4.png",
        "/word/media/image.png",
        "/word/media/imagezero.png",
        "/word/media/image-7.png",
        "/word/media/image0.png",
        "/word/media/images99.png",
        "/ppt/media/image99.png",
    ] {
        package.set_part(name, vec![0xAA]);
    }

    let mut package_bytes = std::io::Cursor::new(Vec::new());
    package.write_to(&mut package_bytes).unwrap();
    let mut reopened = Document::from_bytes(&package_bytes.into_inner()).unwrap();
    reopened.add_picture(
        &[0x44, 0x55, 0x66],
        "added.png",
        Length::inches(1.0),
        Length::inches(1.0),
    );

    let saved = reopened.to_bytes().unwrap();
    let saved_package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(saved)).unwrap();
    assert_eq!(
        saved_package.get_part("/word/media/image5.png"),
        Some([0x44, 0x55, 0x66].as_slice())
    );
}

#[test]
fn occupied_max_image_suffix_wraps_to_a_free_low_number() {
    let mut document = Document::new();
    let bytes = document.to_bytes().unwrap();
    let mut package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(bytes)).unwrap();
    let lower_name = format!("/word/media/image{}.png", usize::MAX - 1);
    let max_name = format!("/word/media/image{}.png", usize::MAX);

    package.set_part("/word/media/image1.png", vec![0xaa]);
    package.set_part(&lower_name, vec![0xbb]);
    package.set_part(&max_name, vec![0xbb]);

    let mut input = std::io::Cursor::new(Vec::new());
    package.write_to(&mut input).unwrap();
    let mut reopened = Document::from_bytes(&input.into_inner()).unwrap();
    reopened.add_picture(
        &[0x44, 0x55, 0x66],
        "added.png",
        Length::inches(1.0),
        Length::inches(1.0),
    );

    let saved = reopened.to_bytes().unwrap();
    let saved_package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(saved)).unwrap();

    assert_eq!(
        saved_package.get_part("/word/media/image2.png"),
        Some([0x44, 0x55, 0x66].as_slice())
    );
    assert!(saved_package.get_part("/word/media/image0.png").is_none());
    assert_eq!(
        saved_package.get_part("/word/media/image1.png"),
        Some([0xaa].as_slice())
    );
    assert_eq!(saved_package.get_part(&lower_name), Some([0xbb].as_slice()));
    assert_eq!(saved_package.get_part(&max_name), Some([0xbb].as_slice()));
}

#[test]
fn max_minus_one_allocates_max_then_wraps_safely() {
    let mut document = Document::new();
    let bytes = document.to_bytes().unwrap();
    let mut package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(bytes)).unwrap();
    let lower_name = format!("/word/media/image{}.png", usize::MAX - 1);
    let max_name = format!("/word/media/image{}.png", usize::MAX);

    package.set_part(&lower_name, vec![0xaa]);

    let mut input = std::io::Cursor::new(Vec::new());
    package.write_to(&mut input).unwrap();
    let mut reopened = Document::from_bytes(&input.into_inner()).unwrap();
    reopened.add_picture(
        &[0x11, 0x22, 0x33],
        "first.png",
        Length::inches(1.0),
        Length::inches(1.0),
    );
    reopened.add_picture(
        &[0x44, 0x55, 0x66],
        "second.png",
        Length::inches(1.0),
        Length::inches(1.0),
    );

    let saved = reopened.to_bytes().unwrap();
    let saved_package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(saved)).unwrap();

    assert_eq!(saved_package.get_part(&lower_name), Some([0xaa].as_slice()));
    assert_eq!(
        saved_package.get_part(&max_name),
        Some([0x11, 0x22, 0x33].as_slice())
    );
    assert_eq!(
        saved_package.get_part("/word/media/image1.png"),
        Some([0x44, 0x55, 0x66].as_slice())
    );
    assert!(saved_package.get_part("/word/media/image0.png").is_none());
}

/// A replacement that contains the placeholder used to restart the search from
/// offset 0 and match its own output forever.
#[test]
fn replacement_containing_the_placeholder_terminates() {
    let mut doc = Document::new();
    doc.add_paragraph("Hello NAME!");

    let count = doc.replace_text("NAME", "NAME Smith");

    assert_eq!(count, 1);
    assert_eq!(doc.paragraphs()[0].text(), "Hello NAME Smith!");
}

#[test]
fn repeated_placeholders_are_all_replaced() {
    let mut doc = Document::new();
    doc.add_paragraph("a X b X c X d");

    let count = doc.replace_text("X", "Y");

    assert_eq!(count, 3);
    assert_eq!(doc.paragraphs()[0].text(), "a Y b Y c Y d");
}

#[test]
fn overlapping_replacement_does_not_rescan_its_own_output() {
    let mut doc = Document::new();
    doc.add_paragraph("aaa");

    let count = doc.replace_text("a", "aa");

    assert_eq!(count, 3, "each source 'a' should be replaced exactly once");
    assert_eq!(doc.paragraphs()[0].text(), "aaaaaa");
}

/// The regex path had the same non-termination hazard, plus one for patterns
/// that can match the empty string.
#[test]
fn regex_replacement_containing_the_pattern_terminates() {
    let mut doc = Document::new();
    doc.add_paragraph("value: 42");

    let count = doc.replace_regex(r"\d+", "[$0]").unwrap();

    assert_eq!(count, 1);
    assert_eq!(doc.paragraphs()[0].text(), "value: [42]");
}

#[test]
fn zero_width_regex_match_terminates() {
    let mut doc = Document::new();
    doc.add_paragraph("abc");

    // `x*` matches the empty string at every position.
    let count = doc.replace_regex("x*", "-").unwrap();

    assert!(count <= 4, "should not loop indefinitely, got {count}");
}

/// `9360 / cols` panicked when a caller asked for a zero-column table.
#[test]
fn zero_column_tables_do_not_panic() {
    let mut doc = Document::new();

    let table = doc.add_table(2, 0);
    assert_eq!(table.row_count(), 2);

    doc.insert_table(0, 1, 0);

    let mut with_cell = doc.add_table(1, 1);
    let mut cell = with_cell.cell(0, 0).unwrap();
    cell.add_table(1, 0);
}

/// Styles were always written to `/word/styles.xml`, so a document without a
/// styles part gained an orphan that Word would ignore.
#[test]
fn styles_part_is_reachable_after_save() {
    let mut doc = Document::new();
    doc.add_paragraph("Body");
    let bytes = doc.to_bytes().unwrap();

    let pkg = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(bytes)).unwrap();

    let rels = pkg
        .get_part_rels("/word/document.xml")
        .expect("document should have relationships");
    let styles_rel = rels
        .get_by_type(oxml_opc::relationship::rel_types::STYLES)
        .expect("styles relationship must exist");
    let target = oxml_opc::OpcPackage::resolve_rel_target("/word/document.xml", &styles_rel.target);

    assert!(
        pkg.get_part(&target).is_some(),
        "styles relationship must point at a part that exists"
    );
    assert!(
        pkg.content_types.content_type_for(&target).is_some(),
        "styles part needs a content type"
    );
}

/// Adding a list twice must not produce two numbering relationships.
#[test]
fn numbering_relationship_is_added_once() {
    let mut doc = Document::new();
    doc.add_bullet_list_item("first", 0);
    doc.add_numbered_list_item("second", 0);
    let bytes = doc.to_bytes().unwrap();

    let pkg = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(bytes)).unwrap();
    let rels = pkg.get_part_rels("/word/document.xml").unwrap();
    let numbering_rels = rels.get_all_by_type(oxml_opc::relationship::rel_types::NUMBERING);

    assert_eq!(
        numbering_rels.len(),
        1,
        "expected exactly one numbering rel"
    );
}

/// Saving the same document twice must produce identical bytes.
#[test]
fn saving_is_reproducible() {
    let mut doc = Document::new();
    doc.add_paragraph("Reproducible");
    for i in 0..25 {
        doc.add_picture(
            &[0u8, 1, 2, 3, i],
            &format!("img{i}.png"),
            Length::inches(1.0),
            Length::inches(1.0),
        );
    }

    let first = doc.to_bytes().unwrap();
    let second = doc.to_bytes().unwrap();

    assert_eq!(first, second);
}

/// Batching must produce the same result as replacing one field at a time.
#[test]
fn batch_replacement_matches_sequential() {
    let build = || {
        let mut doc = Document::new();
        doc.add_paragraph("Dear {{name}}, your order {{order}} ships {{date}}.");
        doc
    };

    let mut sequential = build();
    let mut n = sequential.replace_text("{{name}}", "Ada");
    n += sequential.replace_text("{{order}}", "A-1");
    n += sequential.replace_text("{{date}}", "Friday");

    let mut batched = build();
    let map = HashMap::from([
        ("{{name}}", "Ada"),
        ("{{order}}", "A-1"),
        ("{{date}}", "Friday"),
    ]);
    let batch_count = batched.replace_all(&map);

    assert_eq!(n, batch_count);
    assert_eq!(
        sequential.paragraphs()[0].text(),
        batched.paragraphs()[0].text()
    );
    assert_eq!(
        batched.paragraphs()[0].text(),
        "Dear Ada, your order A-1 ships Friday."
    );
}

/// Entity references must survive a parse/serialise round trip.
#[test]
fn xml_entities_round_trip() {
    let mut doc = Document::new();
    doc.add_paragraph("Ampersand & <angle> \"quote\" 'apos'");
    doc.set_title("Title & Co. <tagged>");

    let bytes = doc.to_bytes().unwrap();
    let reopened = Document::from_bytes(&bytes).unwrap();

    assert_eq!(
        reopened.paragraphs()[0].text(),
        "Ampersand & <angle> \"quote\" 'apos'"
    );
    assert_eq!(reopened.title(), Some("Title & Co. <tagged>"));
}

/// A crafted font name or colour must not be able to break out of the `style`
/// attribute it is written into.
#[test]
fn hostile_run_properties_cannot_inject_html() {
    let mut doc = Document::new();
    {
        let mut para = doc.add_paragraph("");
        para.add_run("text")
            .font("Arial\" onmouseover=\"alert(1)")
            .color("red;} body{display:none} .x{");
    }

    let html = doc.to_html_fragment();

    assert!(!html.contains("onmouseover"), "attribute injection: {html}");
    assert!(!html.contains("display:none"), "css injection: {html}");
    assert!(html.contains("text"));
}
