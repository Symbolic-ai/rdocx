//! Regression tests for previously-fixed defects.
//!
//! Each test names the failure it locks down, so a reintroduction is obvious
//! from the test name alone rather than from a diff.

use std::collections::HashMap;

use rdocx::{Document, Length, RunPosition, RunRange};

const CUSTOM_XML_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml";
const CUSTOM_XML_PROPS_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXmlProps";

fn document_with_content_controls(document_xml: &str) -> Document {
    document_with_bound_content_controls(document_xml, None)
}

fn document_with_bound_content_controls(document_xml: &str, custom_xml: Option<&str>) -> Document {
    let mut seed = Document::new();
    let bytes = seed.to_bytes().unwrap();
    let mut package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(bytes)).unwrap();
    package.set_part("/word/document.xml", document_xml.as_bytes().to_vec());

    if let Some(custom_xml) = custom_xml {
        package.set_part("/customXml/item1.xml", custom_xml.as_bytes().to_vec());
        package.set_part(
            "/customXml/itemProps1.xml",
            br#"<?xml version="1.0" encoding="UTF-8" standalone="no"?><ds:datastoreItem ds:itemID="{11111111-1111-1111-1111-111111111111}" xmlns:ds="http://schemas.openxmlformats.org/officeDocument/2006/customXml"><ds:schemaRefs/></ds:datastoreItem>"#
                .to_vec(),
        );
        package
            .get_or_create_part_rels("/word/document.xml")
            .add(CUSTOM_XML_REL_TYPE, "../customXml/item1.xml");
        package
            .get_or_create_part_rels("/customXml/item1.xml")
            .add(CUSTOM_XML_PROPS_REL_TYPE, "itemProps1.xml");
        package.content_types.add_override(
            "/customXml/itemProps1.xml",
            "application/vnd.openxmlformats-officedocument.customXmlProperties+xml",
        );
    }

    let mut output = std::io::Cursor::new(Vec::new());
    package.write_to(&mut output).unwrap();
    Document::from_bytes(output.get_ref()).unwrap()
}

fn wrap_word_body(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}</w:body></w:document>"#
    )
}

#[test]
fn tag_precedes_alias_and_each_control_updates_once() {
    let xml = wrap_word_body(
        r#"<w:sdt><w:sdtPr><w:tag w:val="customer"/><w:alias w:val="shared"/><w:text/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>one</w:t></w:r></w:p></w:sdtContent></w:sdt><w:sdt><w:sdtPr><w:alias w:val="shared"/><w:text/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>two</w:t></w:r></w:p></w:sdtContent></w:sdt><w:sdt><w:sdtPr><w:tag w:val="missing"/><w:alias w:val="shared"/><w:text/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>three</w:t></w:r></w:p></w:sdtContent></w:sdt>"#,
    );
    let mut document = document_with_content_controls(&xml);
    let values = HashMap::from([
        ("customer".to_string(), "tag value".to_string()),
        ("shared".to_string(), "alias value".to_string()),
    ]);

    assert_eq!(document.bind_content_controls(&values).unwrap(), 3);
    let controls = document.content_controls();
    assert_eq!(
        controls
            .iter()
            .map(|control| control.text())
            .collect::<Vec<_>>(),
        ["tag value", "alias value", "alias value"]
    );
    assert_eq!(controls[0].tag(), Some("customer"));
    assert_eq!(controls[0].alias(), Some("shared"));
    assert_eq!(controls[0].id(), None);
    assert_eq!(
        controls[0].control_type(),
        Some(rdocx_oxml::SdtType::PlainText)
    );
    assert_eq!(document.content_controls_by_tag("customer").len(), 1);
    assert_eq!(document.content_controls_by_alias("shared").len(), 3);

    let mut alias_document = document_with_content_controls(&xml);
    assert_eq!(
        alias_document
            .set_content_control_value_by_alias("shared", "direct alias")
            .unwrap(),
        3
    );
    assert!(
        alias_document
            .content_controls()
            .iter()
            .all(|control| control.text() == "direct alias")
    );
}

#[test]
fn a_control_map_updates_every_matching_display_value() {
    let xml = wrap_word_body(
        r#"<w:sdt><w:sdtPr><w:tag w:val="outer"/><w:text/></w:sdtPr><w:sdtContent><w:p><w:r><w:rPr><w:b/></w:rPr><w:t>old outer</w:t><w:tab/><w:br/></w:r></w:p><w:sdt><w:sdtPr><w:tag w:val="inner"/><w:text/><w:temporary w:val="1"/></w:sdtPr><w:sdtContent><w:p><w:r><w:rPr><w:i/></w:rPr><w:t>old inner</w:t></w:r></w:p></w:sdtContent></w:sdt><w:sdt><w:sdtPr><w:tag w:val="unrelated-control"/><w:text/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>nested untouched</w:t></w:r></w:p></w:sdtContent></w:sdt></w:sdtContent></w:sdt><w:p><w:r><w:t>unrelated</w:t></w:r></w:p>"#,
    );
    let mut document = document_with_content_controls(&xml);
    let values = HashMap::from([
        ("outer".to_string(), "new outer".to_string()),
        ("inner".to_string(), "new inner".to_string()),
    ]);

    assert_eq!(document.bind_content_controls(&values).unwrap(), 2);
    let bytes = document.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(bytes)).unwrap();
    let saved = std::str::from_utf8(package.get_part("/word/document.xml").unwrap()).unwrap();
    assert!(saved.contains("<w:b/>"), "{saved}");
    assert!(saved.contains("<w:t>new outer</w:t>"), "{saved}");
    assert!(saved.contains("<w:i/>"), "{saved}");
    assert!(saved.contains("<w:t>new inner</w:t>"), "{saved}");
    assert!(saved.contains("<w:temporary w:val=\"1\"/>"), "{saved}");
    assert!(saved.contains("<w:t>nested untouched</w:t>"), "{saved}");
    assert!(saved.contains("<w:t>unrelated</w:t>"), "{saved}");
    assert!(!saved.contains("<w:tab"), "{saved}");
    assert!(!saved.contains("<w:br"), "{saved}");
    let controls = document.content_controls();
    assert_eq!(controls[0].text(), "new outer");
    assert_eq!(controls[1].text(), "new inner");
    assert_eq!(controls[2].text(), "nested untouched");
}

#[test]
fn a_bound_custom_xml_value_updates_the_part_and_display_text_atomically() {
    let xml = wrap_word_body(
        r#"<w:sdt><w:sdtPr><w:tag w:val="customer"/><w:dataBinding w:storeItemID="{11111111-1111-1111-1111-111111111111}" w:xpath="/c:root/c:customer[2]/c:name" w:prefixMappings="xmlns:c='urn:customer'"/><w:text/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>old display</w:t></w:r></w:p></w:sdtContent></w:sdt>"#,
    );
    let custom_xml = r#"<?producer keep?><c:root xmlns:c="urn:customer" keep="same"><c:customer code="A"><c:name>First</c:name></c:customer><c:customer xmlns:c="urn:other"><c:name>shadow</c:name></c:customer><!--marker--><c:customer code="B"><c:name old="1">Old &amp; text</c:name><c:other>untouched</c:other></c:customer></c:root>"#;
    let mut document = document_with_bound_content_controls(&xml, Some(custom_xml));

    assert_eq!(
        document
            .set_content_control_value_by_tag("customer", "Ada & Co")
            .unwrap(),
        1
    );
    let bytes = document.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(&bytes)).unwrap();
    assert_eq!(
        package.get_part("/customXml/item1.xml").unwrap(),
        br#"<?producer keep?><c:root xmlns:c="urn:customer" keep="same"><c:customer code="A"><c:name>First</c:name></c:customer><c:customer xmlns:c="urn:other"><c:name>shadow</c:name></c:customer><!--marker--><c:customer code="B"><c:name old="1">Ada &amp; Co</c:name><c:other>untouched</c:other></c:customer></c:root>"#
    );

    let reopened = Document::from_bytes(&bytes).unwrap();
    assert_eq!(reopened.content_controls()[0].text(), "Ada & Co");

    let document_xml =
        std::str::from_utf8(package.get_part("/word/document.xml").unwrap()).unwrap();
    let tag = document_xml.find("<w:tag").unwrap();
    let binding = document_xml.find("<w:dataBinding").unwrap();
    let control_type = document_xml.find("<w:text/>").unwrap();
    assert!(tag < binding && binding < control_type);
}

#[test]
fn an_invalid_binding_changes_neither_document_nor_custom_xml() {
    let cases = [
        (
            "{22222222-2222-2222-2222-222222222222}",
            "/c:root/c:name",
            "xmlns:c='urn:customer'",
        ),
        (
            "{11111111-1111-1111-1111-111111111111}",
            "//c:name",
            "xmlns:c='urn:customer'",
        ),
        (
            "{11111111-1111-1111-1111-111111111111}",
            "/c:root/c:name",
            "xmlns:c='urn:customer'",
        ),
        (
            "{11111111-1111-1111-1111-111111111111}",
            "/c:root/c:name[1]",
            "xmlns:c=urn:customer",
        ),
    ];
    let custom_xml =
        r#"<c:root xmlns:c="urn:customer"><c:name>one</c:name><c:name>two</c:name></c:root>"#;

    for (store_item_id, xpath, prefix_mappings) in cases {
        let xml = wrap_word_body(&format!(
            r#"<w:sdt><w:sdtPr><w:tag w:val="customer"/><w:dataBinding w:storeItemID="{{11111111-1111-1111-1111-111111111111}}" w:xpath="/c:root/c:name[1]" w:prefixMappings="xmlns:c='urn:customer'"/><w:text/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>valid unchanged</w:t></w:r></w:p></w:sdtContent></w:sdt><w:sdt><w:sdtPr><w:tag w:val="customer"/><w:dataBinding w:storeItemID="{store_item_id}" w:xpath="{xpath}" w:prefixMappings="{prefix_mappings}"/><w:text/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>invalid unchanged</w:t></w:r></w:p></w:sdtContent></w:sdt>"#
        ));
        let mut document = document_with_bound_content_controls(&xml, Some(custom_xml));
        let before = document.to_bytes().unwrap();

        assert!(
            document
                .set_content_control_value_by_tag("customer", "changed")
                .is_err(),
            "invalid binding unexpectedly succeeded for {store_item_id} {xpath}"
        );
        assert_eq!(document.to_bytes().unwrap(), before);
    }

    let xml = wrap_word_body(
        r#"<w:sdt><w:sdtPr><w:tag w:val="customer"/><w:dataBinding w:storeItemID="{11111111-1111-1111-1111-111111111111}" w:xpath="/c:root/c:name[1]" w:prefixMappings="xmlns:c='urn:customer'"/><w:text/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>unchanged</w:t></w:r></w:p></w:sdtContent></w:sdt>"#,
    );
    let invalid_properties = [
        br#"<ds:datastoreItem bad:itemID="{11111111-1111-1111-1111-111111111111}" xmlns:ds="http://schemas.openxmlformats.org/officeDocument/2006/customXml" xmlns:bad="urn:producer"><ds:schemaRefs/></ds:datastoreItem>"#
            .as_slice(),
        br#"<bad:wrapper xmlns:bad="urn:producer" xmlns:ds="http://schemas.openxmlformats.org/officeDocument/2006/customXml"><ds:datastoreItem ds:itemID="{11111111-1111-1111-1111-111111111111}"/></bad:wrapper>"#
            .as_slice(),
    ];
    for properties_xml in invalid_properties {
        let mut document = document_with_bound_content_controls(&xml, Some(custom_xml));
        let bytes = document.to_bytes().unwrap();
        let mut package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(bytes)).unwrap();
        package.set_part("/customXml/itemProps1.xml", properties_xml.to_vec());
        let mut output = std::io::Cursor::new(Vec::new());
        package.write_to(&mut output).unwrap();
        let mut document = Document::from_bytes(output.get_ref()).unwrap();
        let before = document.to_bytes().unwrap();
        assert!(
            document
                .set_content_control_value_by_tag("customer", "changed")
                .is_err()
        );
        assert_eq!(document.to_bytes().unwrap(), before);
    }
}

#[test]
fn run_ranges_reject_reverse_missing_and_nonparagraph_positions() {
    let mut document = Document::new();
    document.add_paragraph("first");
    document.add_table(1, 1);
    document.add_paragraph("second");
    let before = document.to_bytes().unwrap();

    let invalid = [
        RunRange {
            start: RunPosition {
                body_index: 2,
                run_index: 0,
            },
            end: RunPosition {
                body_index: 0,
                run_index: 1,
            },
        },
        RunRange {
            start: RunPosition {
                body_index: 99,
                run_index: 0,
            },
            end: RunPosition {
                body_index: 99,
                run_index: 0,
            },
        },
        RunRange {
            start: RunPosition {
                body_index: 1,
                run_index: 0,
            },
            end: RunPosition {
                body_index: 1,
                run_index: 0,
            },
        },
    ];

    for range in invalid {
        assert!(
            document
                .add_comment(range, "Ada", Some("AL"), "review")
                .is_err()
        );
    }
    assert_eq!(document.to_bytes().unwrap(), before);
}

#[test]
fn a_ranged_comment_reply_and_resolution_keep_one_intact_thread() {
    let mut document = Document::new();
    let mut paragraph = document.add_paragraph("");
    paragraph.add_run("first ");
    paragraph.add_run("second");

    let comment_id = document
        .add_comment(
            RunRange {
                start: RunPosition {
                    body_index: 0,
                    run_index: 0,
                },
                end: RunPosition {
                    body_index: 0,
                    run_index: 2,
                },
            },
            "Ada",
            Some("AL"),
            "Please review",
        )
        .unwrap();
    let reply_id = document.reply_to(comment_id, "Ben", "Looks good").unwrap();
    assert!(document.resolve_comment(comment_id, true).unwrap());

    let bytes = document.to_bytes().unwrap();
    let reopened = Document::from_bytes(&bytes).unwrap();
    let comments = reopened.comments();
    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].id(), comment_id);
    assert_eq!(comments[0].text(), "Please review");
    assert!(comments[0].resolved());
    assert_eq!(comments[1].id(), reply_id);
    assert_eq!(comments[1].text(), "Looks good");
    assert_eq!(comments[1].parent_id(), Some(comment_id));
}

#[test]
fn removing_a_comment_removes_only_its_anchors_and_thread_metadata() {
    let mut document = Document::new();
    let mut paragraph = document.add_paragraph("");
    paragraph.add_run("left ");
    paragraph.add_run("middle ");
    paragraph.add_run("right");
    let first = document
        .add_comment(
            RunRange {
                start: RunPosition {
                    body_index: 0,
                    run_index: 0,
                },
                end: RunPosition {
                    body_index: 0,
                    run_index: 1,
                },
            },
            "Ada",
            None,
            "remove me",
        )
        .unwrap();
    let second = document
        .add_comment(
            RunRange {
                start: RunPosition {
                    body_index: 0,
                    run_index: 2,
                },
                end: RunPosition {
                    body_index: 0,
                    run_index: 3,
                },
            },
            "Ben",
            None,
            "keep me",
        )
        .unwrap();

    let bytes = document.to_bytes().unwrap();
    let mut package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(bytes)).unwrap();
    let extension = String::from_utf8(
        package
            .get_part("/word/commentsExtended.xml")
            .unwrap()
            .to_vec(),
    )
    .unwrap()
    .replace(
        "<w15:commentsEx ",
        "<w15:commentsEx xmlns:ext=\"urn:producer\" ",
    )
    .replace(
        "</w15:commentsEx>",
        "<ext:kept token=\"same\"/></w15:commentsEx>",
    );
    package.set_part("/word/commentsExtended.xml", extension.into_bytes());
    let mut seeded = std::io::Cursor::new(Vec::new());
    package.write_to(&mut seeded).unwrap();

    let mut reopened = Document::from_bytes(seeded.get_ref()).unwrap();
    assert!(reopened.remove_comment(first).unwrap());
    let saved = reopened.to_bytes().unwrap();
    let saved_package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(saved)).unwrap();
    let document_xml = String::from_utf8(
        saved_package
            .get_part("/word/document.xml")
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    let extension_xml = String::from_utf8(
        saved_package
            .get_part("/word/commentsExtended.xml")
            .unwrap()
            .to_vec(),
    )
    .unwrap();

    assert_eq!(reopened.text(), "left middle right\n");
    assert_eq!(reopened.comments().len(), 1);
    assert_eq!(reopened.comments()[0].id(), second);
    assert!(!document_xml.contains(&format!("w:id=\"{first}\"")));
    assert!(document_xml.contains(&format!("w:id=\"{second}\"")));
    assert!(extension_xml.contains("<ext:kept token=\"same\"/>"));
}

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

/// Two documents built the same way must produce identical deterministic PDFs.
///
/// The PDF writer keyed its prepared fonts, its font references and its glyph
/// to Unicode table on hashed maps, and iterated all three to write the file.
/// Iteration order differs between map instances, so the same document written
/// twice differed in its `/Font` dictionary order, its font object order and
/// its ToUnicode CMap line order. Nothing about the rendered page changed and
/// nothing failed, which is why it survived until the hash harness went looking.
///
/// Two separately built documents are the point. One document written twice
/// reuses the same map instances and cannot see this.
#[test]
fn two_identical_documents_produce_identical_deterministic_pdfs() {
    let build = || {
        let mut doc = Document::new();
        doc.add_paragraph("A heading in the document");
        let mut styled = doc.add_paragraph("");
        styled.add_run("bold text").bold(true);
        styled.add_run(" and italic text").italic(true);
        styled.add_run(" and plain text");
        doc.add_paragraph("A closing paragraph with enough words to shape.");
        doc
    };

    let first = build()
        .to_pdf_deterministic()
        .expect("deterministic PDF rendering should succeed");
    let second = build()
        .to_pdf_deterministic()
        .expect("deterministic PDF rendering should succeed");

    assert!(first.starts_with(b"%PDF-"));
    assert_eq!(
        first, second,
        "the same document produced two different PDFs"
    );
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

#[test]
fn a_bookmark_inserted_over_a_range_is_listed_with_its_text() {
    let mut document = Document::new();
    document.add_paragraph("before ").add_run("inside");
    document.add_paragraph(" across ").add_run("after");

    let id = document
        .add_bookmark(
            "destination",
            RunRange {
                start: RunPosition {
                    body_index: 0,
                    run_index: 1,
                },
                end: RunPosition {
                    body_index: 1,
                    run_index: 1,
                },
            },
        )
        .expect("valid bookmark range");

    let bookmarks = document.bookmarks();
    assert_eq!(bookmarks.len(), 1);
    assert_eq!(bookmarks[0].id(), Some(id));
    assert_eq!(bookmarks[0].name(), Some("destination"));
    assert_eq!(bookmarks[0].text(), "inside\n across ");
    assert_eq!(bookmarks[0].issue(), None);

    let reopened = Document::from_bytes(&document.to_bytes().unwrap()).unwrap();
    assert_eq!(reopened.bookmarks()[0].text(), "inside\n across ");
}

#[test]
fn ref_and_pageref_resolve_to_the_bookmark_text_and_final_page() {
    let mut document = Document::new();
    document.add_paragraph("field-placeholder");
    document
        .add_paragraph("bookmarked text")
        .page_break_before(true);
    document
        .add_bookmark(
            "destination",
            RunRange {
                start: RunPosition {
                    body_index: 1,
                    run_index: 0,
                },
                end: RunPosition {
                    body_index: 1,
                    run_index: 1,
                },
            },
        )
        .unwrap();

    let bytes = document.to_bytes().unwrap();
    let mut package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(bytes)).unwrap();
    let original_xml =
        String::from_utf8(package.get_part("/word/document.xml").unwrap().to_vec()).unwrap();
    let xml = original_xml.replace(
            "<w:r>\n        <w:t>field-placeholder</w:t>\n      </w:r>",
            r#"<w:fldSimple w:instr=" REF destination "><w:r><w:t>cached</w:t></w:r></w:fldSimple><w:r><w:t> page </w:t></w:r><w:fldSimple w:instr=" PAGEREF destination "><w:r><w:t>cached-page</w:t></w:r></w:fldSimple>"#,
        );
    assert_ne!(xml, original_xml, "{original_xml}");
    package.set_part("/word/document.xml", xml.into_bytes());
    let mut rewritten = std::io::Cursor::new(Vec::new());
    package.write_to(&mut rewritten).unwrap();

    let reopened = Document::from_bytes(rewritten.get_ref()).unwrap();
    let page = reopened.layout_page(0).unwrap().unwrap();
    let text = page
        .elements
        .iter()
        .filter_map(|element| match element {
            oxml_layout::PositionedElement::Text(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect::<String>();
    assert!(text.contains("bookmarked text"), "{text}");
    assert!(text.contains("page 2"), "{text}");
}

#[test]
fn missing_and_duplicate_bookmark_targets_fail_without_mutation() {
    let mut document = Document::new();
    document.add_paragraph("one");
    let range = RunRange {
        start: RunPosition {
            body_index: 0,
            run_index: 0,
        },
        end: RunPosition {
            body_index: 0,
            run_index: 1,
        },
    };
    document.add_bookmark("destination", range).unwrap();
    let before = document.to_bytes().unwrap();

    assert!(document.add_bookmark("destination", range).is_err());
    assert!(document.add_bookmark("_TocReserved", range).is_err());
    assert!(
        document
            .add_bookmark(
                "invalid",
                RunRange {
                    start: RunPosition {
                        body_index: 0,
                        run_index: 2,
                    },
                    end: range.end,
                },
            )
            .is_err()
    );
    assert_eq!(document.to_bytes().unwrap(), before);
}

#[test]
fn malformed_and_unmatched_bookmark_markers_are_reported_without_loss() {
    let mut document = Document::new();
    document.add_paragraph("content");
    let bytes = document.to_bytes().unwrap();
    let mut package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(bytes)).unwrap();
    let original_xml =
        String::from_utf8(package.get_part("/word/document.xml").unwrap().to_vec()).unwrap();
    let xml = original_xml
        .replace(
            "<w:r>\n        <w:t>content</w:t>\n      </w:r>",
            r#"<w:bookmarkStart w:name="missing-id"/><w:r><w:t>content</w:t></w:r><w:bookmarkEnd w:id="9"/>"#,
        );
    assert_ne!(xml, original_xml);
    package.set_part("/word/document.xml", xml.into_bytes());
    let mut rewritten = std::io::Cursor::new(Vec::new());
    package.write_to(&mut rewritten).unwrap();

    let mut reopened = Document::from_bytes(rewritten.get_ref()).unwrap();
    let bookmarks = reopened.bookmarks();
    assert_eq!(bookmarks.len(), 2);
    assert!(bookmarks.iter().all(|bookmark| bookmark.issue().is_some()));
    assert!(bookmarks.iter().any(|bookmark| bookmark.id().is_none()));
    assert!(bookmarks.iter().any(|bookmark| bookmark.id() == Some(9)));

    let round_trip = reopened.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(round_trip)).unwrap();
    let round_trip_xml =
        String::from_utf8(package.get_part("/word/document.xml").unwrap().to_vec()).unwrap();
    assert!(round_trip_xml.contains(r#"<w:bookmarkStart w:name="missing-id"/>"#));
    assert!(round_trip_xml.contains(r#"<w:bookmarkEnd w:id="9"/>"#));
}
