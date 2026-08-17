//! Regression tests for previously-fixed defects.
//!
//! Each test names the failure it locks down, so a reintroduction is obvious
//! from the test name alone rather than from a diff.

use std::collections::HashMap;

use rdocx::{Document, Length, RunPosition, RunRange};
use rdocx_oxml::CT_Document;
use rdocx_oxml::document::CT_Body;

const CUSTOM_XML_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml";
const CUSTOM_XML_PROPS_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXmlProps";
const WORD_REVISION_ORACLE: &str = "Microsoft Word 16.104 build 16.104.25121423";

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

fn body_from_document(document: &mut Document) -> CT_Body {
    let package =
        oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(document.to_bytes().unwrap()))
            .unwrap();
    CT_Document::from_xml(package.get_part("/word/document.xml").unwrap())
        .unwrap()
        .body
}

fn document_xml(document: &mut Document) -> String {
    let package =
        oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(document.to_bytes().unwrap()))
            .unwrap();
    String::from_utf8(package.get_part("/word/document.xml").unwrap().to_vec()).unwrap()
}

fn document_with_comment_paragraphs(paragraphs: &str) -> (Document, i32) {
    let mut document = Document::new();
    document.add_paragraph("seed");
    let comment_id = document
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
            "remove",
        )
        .unwrap();
    let bytes = document.to_bytes().unwrap();
    let mut package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(bytes)).unwrap();
    let mut xml =
        String::from_utf8(package.get_part("/word/document.xml").unwrap().to_vec()).unwrap();
    let start = xml.find("<w:p>").unwrap();
    let end = start + xml[start..].find("</w:p>").unwrap() + "</w:p>".len();
    xml.replace_range(start..end, paragraphs);
    package.set_part("/word/document.xml", xml.into_bytes());
    let mut output = std::io::Cursor::new(Vec::new());
    package.write_to(&mut output).unwrap();
    (Document::from_bytes(output.get_ref()).unwrap(), comment_id)
}

fn body_from_xml(body: &str) -> CT_Body {
    CT_Document::from_xml(wrap_word_body(body).as_bytes())
        .unwrap()
        .body
}

#[test]
fn accepting_every_revision_matches_word_normalized_body_xml() {
    let xml = wrap_word_body(
        r#"<w:p><w:pPr><w:jc w:val="center"/><w:pPrChange w:id="5" w:author="Ada"><w:pPr><w:jc w:val="left"/></w:pPr></w:pPrChange></w:pPr><w:r><w:rPr><w:b/><w:rPrChange w:id="6" w:author="Ada"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:t>kept</w:t></w:r><w:ins w:id="1" w:author="Ada"><w:r><w:t> inserted</w:t></w:r></w:ins><w:del w:id="2" w:author="Ada"><w:r><w:delText> deleted</w:delText></w:r></w:del><w:moveFrom w:id="3" w:author="Ada"><w:r><w:t> old-place</w:t></w:r></w:moveFrom><w:moveTo w:id="4" w:author="Ada"><w:r><w:t> moved</w:t></w:r></w:moveTo></w:p><w:tbl><w:tblPr><w:jc w:val="center"/><w:tblPrChange w:id="7" w:author="Ada"><w:tblPr><w:jc w:val="right"/></w:tblPr></w:tblPrChange></w:tblPr><w:tblGrid/><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl><w:sectPr><w:titlePg/><w:sectPrChange w:id="8" w:author="Ada"><w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:sectPrChange></w:sectPr>"#,
    );
    let mut document = document_with_content_controls(&xml);

    assert_eq!(document.accept_all().unwrap(), 8);
    assert!(document.revisions().is_empty());
    let expected = body_from_xml(
        r#"<w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>kept</w:t></w:r><w:r><w:t> inserted</w:t></w:r><w:r><w:t> moved</w:t></w:r></w:p><w:tbl><w:tblPr><w:jc w:val="center"/></w:tblPr><w:tblGrid/><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl><w:sectPr><w:titlePg/></w:sectPr>"#,
    );
    assert_eq!(
        body_from_document(&mut document),
        expected,
        "normalized body must match the pinned {WORD_REVISION_ORACLE} oracle"
    );
}

#[test]
fn rejecting_insertions_and_deletions_restores_the_recorded_content() {
    let xml = wrap_word_body(
        r#"<w:p><w:pPr><w:jc w:val="center"/><w:pPrChange w:id="5" w:author="Ada"><w:pPr><w:jc w:val="left"/></w:pPr></w:pPrChange></w:pPr><w:r><w:rPr><w:b/><w:rPrChange w:id="6" w:author="Ada"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:t>kept</w:t></w:r><w:ins w:id="1" w:author="Ada"><w:r><w:t> inserted</w:t></w:r></w:ins><w:del w:id="2" w:author="Ada"><w:r><w:delText> deleted</w:delText></w:r></w:del><w:moveFrom w:id="3" w:author="Ada"><w:r><w:t> old-place</w:t></w:r></w:moveFrom><w:moveTo w:id="4" w:author="Ada"><w:r><w:t> moved</w:t></w:r></w:moveTo></w:p><w:tbl><w:tblPr><w:jc w:val="center"/><w:tblPrChange w:id="7" w:author="Ada"><w:tblPr><w:jc w:val="right"/></w:tblPr></w:tblPrChange></w:tblPr><w:tblGrid/><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl><w:sectPr><w:titlePg/><w:sectPrChange w:id="8" w:author="Ada"><w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:sectPrChange></w:sectPr>"#,
    );
    let mut document = document_with_content_controls(&xml);

    assert_eq!(document.reject_all().unwrap(), 8);
    assert!(document.revisions().is_empty());
    let expected = body_from_xml(
        r#"<w:p><w:pPr><w:jc w:val="left"/></w:pPr><w:r><w:rPr><w:i/></w:rPr><w:t>kept</w:t></w:r><w:r><w:t> deleted</w:t></w:r><w:r><w:t> old-place</w:t></w:r></w:p><w:tbl><w:tblPr><w:jc w:val="right"/></w:tblPr><w:tblGrid/><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl><w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>"#,
    );
    assert_eq!(body_from_document(&mut document), expected);
}

#[test]
fn scoped_revision_actions_change_only_matching_revisions() {
    let xml = wrap_word_body(
        r#"<w:p><w:ins w:id="7" w:author="Ada" w:date="2026-08-17T09:00:00+01:00"><w:r><w:t>A</w:t></w:r></w:ins><w:ins w:id="7" w:author="Ben" w:date="2026-08-17T09:00:00Z"><w:r><w:t>B</w:t></w:r></w:ins><w:del w:id="8" w:author="Ada"><w:r><w:delText>C</w:delText></w:r></w:del></w:p>"#,
    );

    let mut by_author = document_with_content_controls(&xml);
    assert_eq!(by_author.accept_revisions_by_author("Ada").unwrap(), 2);
    assert!(document_xml(&mut by_author).contains(
        r#"<w:ins w:id="7" w:author="Ben" w:date="2026-08-17T09:00:00Z"><w:r><w:t>B</w:t></w:r></w:ins>"#
    ));
    assert_eq!(
        by_author
            .revisions()
            .iter()
            .map(|revision| revision.author())
            .collect::<Vec<_>>(),
        vec!["Ben"]
    );

    let mut by_id = document_with_content_controls(&xml);
    assert_eq!(by_id.reject_revision_id(7).unwrap(), 2);
    assert_eq!(
        by_id
            .revisions()
            .iter()
            .map(|revision| revision.id())
            .collect::<Vec<_>>(),
        vec![8]
    );

    let mut by_date = document_with_content_controls(&xml);
    assert_eq!(
        by_date
            .accept_revisions_in_date_range("2026-08-17T08:00:00Z", "2026-08-17T08:00:00Z")
            .unwrap(),
        1
    );
    assert_eq!(
        by_date
            .revisions()
            .iter()
            .map(|revision| revision.author())
            .collect::<Vec<_>>(),
        vec!["Ben", "Ada"]
    );

    let mut by_lowercase_date = document_with_content_controls(&xml);
    assert_eq!(
        by_lowercase_date
            .reject_revisions_in_date_range("2026-08-17t08:00:00z", "2026-08-17t08:00:00z",)
            .unwrap(),
        1
    );
}

#[test]
fn unmodelled_revision_lookalikes_remain_byte_identical() {
    let xml = wrap_word_body(
        r#"<w:customXml><w:ins w:id="99" w:author="raw"><w:r><w:t>opaque</w:t></w:r></w:ins></w:customXml><w:p><w:r><w:t>typed</w:t></w:r></w:p>"#,
    );
    let mut document = document_with_content_controls(&xml);
    let before = document.to_bytes().unwrap();

    assert_eq!(document.accept_all().unwrap(), 0);
    assert_eq!(document.to_bytes().unwrap(), before);
}

#[test]
fn unwrapped_revisions_keep_wrapper_namespace_bindings() {
    let xml = wrap_word_body(
        r#"<w:p><w:ins xmlns:vendor="urn:vendor" w:id="1" w:author="Ada"><w:r><w:t>kept</w:t><vendor:opaque vendor:value="yes"/></w:r></w:ins></w:p>"#,
    );
    let mut document = document_with_content_controls(&xml);

    assert_eq!(document.accept_all().unwrap(), 1);
    let saved = document_xml(&mut document);
    assert!(saved.contains("xmlns:vendor=\"urn:vendor\""));
    assert!(saved.contains("<vendor:opaque"));
    assert!(saved.contains("vendor:value=\"yes\""));
}

#[test]
fn rejected_property_changes_keep_owner_namespace_bindings() {
    let xml = wrap_word_body(
        r#"<w:p><w:r><w:rPr xmlns:vendor="urn:vendor"><w:b/><w:rPrChange w:id="1" w:author="Ada"><w:rPr><vendor:rPrChange vendor:value="prior"/></w:rPr></w:rPrChange></w:rPr><w:t>text</w:t></w:r></w:p>"#,
    );
    let mut document = document_with_content_controls(&xml);

    assert_eq!(document.reject_all().unwrap(), 1);
    let saved = document_xml(&mut document);
    assert!(saved.contains("xmlns:vendor=\"urn:vendor\""));
    assert!(saved.contains("<vendor:rPrChange"));
    assert!(saved.contains("vendor:value=\"prior\""));
}

#[test]
fn invalid_date_ranges_leave_the_document_unchanged() {
    let xml = wrap_word_body(
        r#"<w:p><w:ins w:id="1" w:author="Ada" w:date="2026-08-17T09:00:00Z"><w:r><w:t>A</w:t></w:r></w:ins></w:p>"#,
    );
    let mut document = document_with_content_controls(&xml);
    let before = document.to_bytes().unwrap();

    assert!(
        document
            .accept_revisions_in_date_range("not-a-date", "2026-08-17T09:00:00Z")
            .is_err()
    );
    assert_eq!(document.to_bytes().unwrap(), before);
    assert!(
        document
            .reject_revisions_in_date_range("2026-08-18T09:00:00Z", "2026-08-17T09:00:00Z")
            .is_err()
    );
    assert_eq!(document.to_bytes().unwrap(), before);

    for malformed in [
        "2026-08-17T-1:00:00Z",
        "2026-8-17T09:00:00Z",
        "2026-08-17T9:00:00Z",
        "9223372036854775807-08-17T09:00:00Z",
        "2026-08-17T12:00:60Z",
        "2016-12-31T23:59:60Z",
    ] {
        assert!(
            document
                .accept_revisions_in_date_range(malformed, "2026-08-17T09:00:00Z")
                .is_err(),
            "{malformed} must not parse as RFC 3339"
        );
        assert_eq!(document.to_bytes().unwrap(), before);
    }
}

#[test]
fn contextual_row_markers_keep_or_remove_the_owning_row() {
    let xml = wrap_word_body(
        r#"<w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:trPr><w:ins w:id="1" w:author="Ada"/></w:trPr><w:tc><w:p><w:r><w:t>inserted</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:trPr><w:del w:id="2" w:author="Ada"/></w:trPr><w:tc><w:p><w:r><w:t>deleted</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
    );

    let mut accepted = document_with_content_controls(&xml);
    assert_eq!(accepted.accept_all().unwrap(), 2);
    assert!(accepted.text().contains("inserted"));
    assert!(!accepted.text().contains("deleted"));

    let mut rejected = document_with_content_controls(&xml);
    assert_eq!(rejected.reject_all().unwrap(), 2);
    assert!(!rejected.text().contains("inserted"));
    assert!(rejected.text().contains("deleted"));

    let conflicting_xml = wrap_word_body(
        r#"<w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:trPr><w:ins w:id="3" w:author="Ada"/><w:del w:id="4" w:author="Ada"/></w:trPr><w:tc><w:p><w:r><w:t>ambiguous</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
    );
    let mut conflicting = document_with_content_controls(&conflicting_xml);
    assert_eq!(conflicting.accept_all().unwrap(), 2);
    assert!(!conflicting.text().contains("ambiguous"));
}

#[test]
fn contextual_paragraph_markers_merge_the_adjacent_paragraphs() {
    let xml = wrap_word_body(
        r#"<w:p><w:pPr><w:rPr><w:del w:id="1" w:author="Ada"/></w:rPr></w:pPr><w:r><w:t>first</w:t></w:r></w:p><w:p><w:pPr><w:jc w:val="right"/></w:pPr><w:r><w:t> second</w:t></w:r></w:p>"#,
    );
    let mut accepted = document_with_content_controls(&xml);
    assert_eq!(accepted.accept_all().unwrap(), 1);
    assert_eq!(accepted.paragraph_count(), 1);
    assert_eq!(accepted.text(), "first second\n");
    assert_eq!(
        body_from_document(&mut accepted),
        body_from_xml(
            r#"<w:p><w:pPr><w:jc w:val="right"/></w:pPr><w:r><w:t>first</w:t></w:r><w:r><w:t> second</w:t></w:r></w:p>"#,
        )
    );

    let insertion_xml = wrap_word_body(
        r#"<w:p><w:pPr><w:rPr><w:ins w:id="2" w:author="Ada"/></w:rPr></w:pPr><w:r><w:t>first</w:t></w:r></w:p><w:p><w:r><w:t> second</w:t></w:r></w:p>"#,
    );
    let mut rejected = document_with_content_controls(&insertion_xml);
    assert_eq!(rejected.reject_all().unwrap(), 1);
    assert_eq!(rejected.paragraph_count(), 1);
    assert_eq!(rejected.text(), "first second\n");

    let chained_xml = wrap_word_body(
        r#"<w:p><w:pPr><w:rPr><w:del w:id="3" w:author="Ada"/></w:rPr></w:pPr><w:r><w:t>one</w:t></w:r></w:p><w:p><w:pPr><w:rPr><w:del w:id="4" w:author="Ada"/></w:rPr></w:pPr><w:r><w:t> two</w:t></w:r></w:p><w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:t> three</w:t></w:r></w:p>"#,
    );
    let mut chained = document_with_content_controls(&chained_xml);
    assert_eq!(chained.accept_all().unwrap(), 2);
    assert_eq!(chained.paragraph_count(), 1);
    assert_eq!(chained.text(), "one two three\n");
    assert_eq!(
        body_from_document(&mut chained),
        body_from_xml(
            r#"<w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:t>one</w:t></w:r><w:r><w:t> two</w:t></w:r><w:r><w:t> three</w:t></w:r></w:p>"#,
        )
    );
}

#[test]
fn malformed_selected_property_changes_fail_atomically() {
    let xml = wrap_word_body(
        r#"<w:p><w:pPr><w:jc w:val="center"/><w:pPrChange w:id="1" w:author="Ada"/></w:pPr><w:r><w:t>text</w:t></w:r></w:p>"#,
    );
    let mut document = document_with_content_controls(&xml);
    let before = document.to_bytes().unwrap();
    let normal_layout = document.layout_page(0).unwrap();
    let deterministic_layout = document.to_pdf_deterministic().unwrap();

    assert!(document.reject_all().is_err());
    assert_eq!(document.to_bytes().unwrap(), before);
    assert_eq!(document.revisions().len(), 1);
    assert_eq!(
        format!("{:?}", document.layout_page(0).unwrap()),
        format!("{normal_layout:?}")
    );
    assert_eq!(
        document.to_pdf_deterministic().unwrap(),
        deterministic_layout
    );

    let wrong_prior_xml = wrap_word_body(
        r#"<w:p><w:r><w:rPr><w:b/><w:rPrChange w:id="2" w:author="Ada"><w:pPr/></w:rPrChange></w:rPr><w:t>text</w:t></w:r></w:p>"#,
    );
    let mut wrong_prior = document_with_content_controls(&wrong_prior_xml);
    let wrong_prior_before = wrong_prior.to_bytes().unwrap();
    assert!(wrong_prior.reject_all().is_err());
    assert_eq!(wrong_prior.to_bytes().unwrap(), wrong_prior_before);

    let extra_prior_xml = wrap_word_body(
        r#"<w:p><w:r><w:rPr><w:b/><w:rPrChange w:id="5" w:author="Ada"><w:rPr><w:i/></w:rPr><w:rPr/></w:rPrChange></w:rPr><w:t>text</w:t></w:r></w:p>"#,
    );
    let mut extra_prior = document_with_content_controls(&extra_prior_xml);
    let extra_prior_before = extra_prior.to_bytes().unwrap();
    assert!(extra_prior.accept_all().is_err());
    assert_eq!(extra_prior.to_bytes().unwrap(), extra_prior_before);

    let hidden_xml = wrap_word_body(
        r#"<w:p><w:ins w:id="3" w:author="Ada"><w:r><w:rPr><w:rPrChange w:id="4" w:author="Ada"/></w:rPr><w:t>hidden</w:t></w:r></w:ins></w:p>"#,
    );
    let mut hidden = document_with_content_controls(&hidden_xml);
    let hidden_before = hidden.to_bytes().unwrap();
    assert!(hidden.reject_all().is_err());
    assert_eq!(hidden.to_bytes().unwrap(), hidden_before);
}

#[test]
fn nested_selected_revisions_resolve_inside_out_and_count_once() {
    let xml = wrap_word_body(
        r#"<w:p><w:ins w:id="1" w:author="Ada"><w:r><w:rPr><w:b/><w:rPrChange w:id="2" w:author="Ada"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:t>nested</w:t></w:r></w:ins></w:p>"#,
    );
    let mut accepted = document_with_content_controls(&xml);

    assert_eq!(accepted.accept_all().unwrap(), 2);
    assert!(accepted.revisions().is_empty());
    assert_eq!(accepted.text(), "nested\n");
}

#[test]
fn hyperlink_nested_revisions_resolve_inside_out_when_scoped() {
    let xml = wrap_word_body(
        r#"<w:p><w:hyperlink r:id="rId5"><w:r><w:t xml:space="preserve">before </w:t></w:r><w:ins w:id="11" w:author="Ada"><w:del w:id="12" w:author="Ben"><w:r><w:delText>nested</w:delText></w:r></w:del></w:ins><w:r><w:t xml:space="preserve"> after</w:t></w:r></w:hyperlink></w:p>"#,
    );
    let mut document = document_with_content_controls(&xml);

    assert_eq!(document.reject_revision_id(12).unwrap(), 1);
    assert_eq!(
        document
            .revisions()
            .iter()
            .map(|revision| revision.id())
            .collect::<Vec<_>>(),
        [11]
    );
    let staged = document.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(staged)).unwrap();
    let document_xml =
        std::str::from_utf8(package.get_part("/word/document.xml").unwrap()).unwrap();
    assert!(document_xml.contains(r#"<w:ins w:id="11" w:author="Ada">"#));
    assert!(document_xml.contains("<w:t>nested</w:t>"));
    assert!(!document_xml.contains(r#"w:id="12""#));

    assert_eq!(document.accept_revision_id(11).unwrap(), 1);
    assert!(document.revisions().is_empty());
    assert_eq!(document.text(), "before nested after\n");
}

#[test]
fn targetless_revision_only_hyperlinks_keep_sibling_order_when_resolved() {
    let xml = wrap_word_body(
        r#"<w:p><w:hyperlink><w:ins w:id="21" w:author="Ada"><w:r><w:t>H</w:t></w:r></w:ins></w:hyperlink><w:del w:id="22" w:author="Ben"><w:r><w:delText>B</w:delText></w:r></w:del><w:ins w:id="23" w:author="Cy"><w:r><w:t>C</w:t></w:r></w:ins><w:hyperlink><w:del w:id="24" w:author="Dee"><w:r><w:delText>D</w:delText></w:r></w:del></w:hyperlink></w:p>"#,
    );
    let mut document = document_with_content_controls(&xml);

    assert_eq!(
        document
            .revisions()
            .iter()
            .map(|revision| revision.id())
            .collect::<Vec<_>>(),
        [21, 22, 23, 24]
    );
    assert_eq!(document.accept_revision_id(21).unwrap(), 1);
    assert_eq!(document.reject_revision_id(24).unwrap(), 1);
    assert_eq!(
        document
            .revisions()
            .iter()
            .map(|revision| revision.id())
            .collect::<Vec<_>>(),
        [22, 23]
    );

    let staged = document.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(staged)).unwrap();
    let document_xml =
        std::str::from_utf8(package.get_part("/word/document.xml").unwrap()).unwrap();
    let positions = [
        document_xml.find("<w:t>H</w:t>").unwrap(),
        document_xml.find(r#"w:id="22""#).unwrap(),
        document_xml.find(r#"w:id="23""#).unwrap(),
        document_xml.find("<w:t>D</w:t>").unwrap(),
    ];
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));

    assert_eq!(document.accept_all().unwrap(), 2);
    assert!(document.revisions().is_empty());
    assert_eq!(document.text(), "HCD\n");
}

#[test]
fn resolving_a_modeled_hyperlink_keeps_unreported_raw_children() {
    let malformed = r#"<w:ins w:id="bad"><w:r><w:t>raw revision</w:t></w:r></w:ins>"#;
    let foreign = r#"<x:opaque xmlns:x="urn:opaque" x:flag="1"/>"#;
    let xml = wrap_word_body(&format!(
        r#"<w:p><w:hyperlink r:id="rId5"><w:r><w:t>before</w:t></w:r>{malformed}<w:ins w:id="31" w:author="Ada"><w:r><w:t>reported</w:t></w:r></w:ins>{foreign}<w:r><w:t>after</w:t></w:r></w:hyperlink></w:p>"#
    ));
    let mut document = document_with_content_controls(&xml);

    assert_eq!(
        document
            .revisions()
            .iter()
            .map(|revision| revision.id())
            .collect::<Vec<_>>(),
        [31]
    );
    assert_eq!(document.accept_revision_id(31).unwrap(), 1);
    assert!(document.revisions().is_empty());
    assert_eq!(document.text(), "beforereportedafter\n");

    let staged = document.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(staged)).unwrap();
    let document_xml =
        std::str::from_utf8(package.get_part("/word/document.xml").unwrap()).unwrap();
    assert!(document_xml.contains(malformed));
    assert!(document_xml.contains(foreign));
    let positions = [
        document_xml.find("<w:t>before</w:t>").unwrap(),
        document_xml.find(malformed).unwrap(),
        document_xml.find("<w:t>reported</w:t>").unwrap(),
        document_xml.find(foreign).unwrap(),
        document_xml.find("<w:t>after</w:t>").unwrap(),
    ];
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn malformed_revision_wrappers_are_opaque_to_every_resolution_scope() {
    let malformed = r#"<w:ins w:id="bad"><w:del w:id="51" w:author="Ada" w:date="2026-08-17T10:00:00Z"><w:r><w:delText>hidden</w:delText></w:r></w:del></w:ins>"#;
    let xml = wrap_word_body(&format!(
        r#"<w:p><w:hyperlink>{malformed}</w:hyperlink></w:p>"#
    ));
    let assert_opaque =
        |mut document: Document, action: fn(&mut Document) -> rdocx::Result<usize>| {
            assert!(document.revisions().is_empty());
            let before = document_xml(&mut document);
            assert_eq!(action(&mut document).unwrap(), 0);
            assert_eq!(document_xml(&mut document), before);
            assert!(before.contains(malformed));
        };

    assert_opaque(document_with_content_controls(&xml), Document::accept_all);
    assert_opaque(document_with_content_controls(&xml), Document::reject_all);
    assert_opaque(document_with_content_controls(&xml), |document| {
        document.accept_revision_id(51)
    });
    assert_opaque(document_with_content_controls(&xml), |document| {
        document.reject_revision_id(51)
    });
    assert_opaque(document_with_content_controls(&xml), |document| {
        document.accept_revisions_by_author("Ada")
    });
    assert_opaque(document_with_content_controls(&xml), |document| {
        document.reject_revisions_by_author("Ada")
    });
    assert_opaque(document_with_content_controls(&xml), |document| {
        document.accept_revisions_in_date_range("2026-08-17T09:00:00Z", "2026-08-17T11:00:00Z")
    });
    assert_opaque(document_with_content_controls(&xml), |document| {
        document.reject_revisions_in_date_range("2026-08-17T09:00:00Z", "2026-08-17T11:00:00Z")
    });
}

#[test]
fn comment_removal_remaps_and_retains_hyperlink_owned_revision_content() {
    let raw_solo = r#"<x:solo xmlns:x="urn:opaque"/>"#;
    let raw_middle = r#"<x:middle xmlns:x="urn:opaque"/>"#;
    let paragraphs = format!(
        r#"<w:p><w:commentRangeStart w:id="0"/><w:commentRangeEnd w:id="0"/><w:hyperlink><w:r><w:commentReference w:id="0"/></w:r><w:ins w:id="41" w:author="Ada"><w:r><w:t>solo</w:t></w:r></w:ins>{raw_solo}</w:hyperlink></w:p><w:p><w:hyperlink><w:r><w:commentReference w:id="0"/></w:r><w:r><w:t>before</w:t></w:r><w:ins w:id="42" w:author="Ada"><w:r><w:t>middle</w:t></w:r></w:ins>{raw_middle}<w:r><w:t>after</w:t></w:r></w:hyperlink></w:p>"#
    );
    let (mut document, comment_id) = document_with_comment_paragraphs(&paragraphs);

    assert_eq!(
        document
            .revisions()
            .iter()
            .map(|revision| revision.id())
            .collect::<Vec<_>>(),
        [41, 42]
    );
    assert!(document.remove_comment(comment_id).unwrap());
    let removed = document_xml(&mut document);
    assert!(!removed.contains("commentReference"));
    for raw in [raw_solo, raw_middle] {
        assert!(removed.contains(raw), "missing {raw} in {removed}");
    }
    let positions = [
        removed.find("<w:t>before</w:t>").unwrap(),
        removed.find(r#"w:id="42""#).unwrap(),
        removed.find(raw_middle).unwrap(),
        removed.find("<w:t>after</w:t>").unwrap(),
    ];
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));

    let reopened = Document::from_bytes(&document.to_bytes().unwrap()).unwrap();
    assert_eq!(
        reopened
            .revisions()
            .iter()
            .map(|revision| revision.id())
            .collect::<Vec<_>>(),
        [41, 42]
    );
    let mut reopened = reopened;
    assert_eq!(reopened.accept_revision_id(41).unwrap(), 1);
    assert_eq!(reopened.accept_revision_id(42).unwrap(), 1);
    assert!(reopened.revisions().is_empty());
    let resolved = document_xml(&mut reopened);
    assert!(resolved.contains(raw_solo));
    assert!(resolved.contains(raw_middle));
    assert_eq!(reopened.text(), "solo\nbeforemiddleafter\n");
}

#[test]
fn run_mutations_keep_raw_children_at_live_boundaries() {
    let body = r#"<w:p><w:r><w:before/><w:t>A</w:t><w:between/><w:t>B</w:t><w:after/></w:r></w:p>"#;

    let mut replaced = document_with_content_controls(&wrap_word_body(body));
    replaced
        .paragraph_mut(0)
        .unwrap()
        .run_mut(0)
        .unwrap()
        .set_text("replacement");
    let replaced_xml = document_xml(&mut replaced);
    let replaced_positions = [
        replaced_xml.find("<w:before/>").unwrap(),
        replaced_xml.find("<w:t>replacement</w:t>").unwrap(),
        replaced_xml.find("<w:between/>").unwrap(),
        replaced_xml.find("<w:after/>").unwrap(),
    ];
    assert!(replaced_positions.windows(2).all(|pair| pair[0] < pair[1]));

    let mut appended = document_with_content_controls(&wrap_word_body(body));
    appended
        .paragraph_mut(0)
        .unwrap()
        .run_mut(0)
        .unwrap()
        .add_text("C");
    let appended_xml = document_xml(&mut appended);
    let appended_positions = [
        appended_xml.find("<w:before/>").unwrap(),
        appended_xml.find("<w:t>A</w:t>").unwrap(),
        appended_xml.find("<w:between/>").unwrap(),
        appended_xml.find("<w:t>B</w:t>").unwrap(),
        appended_xml.find("<w:after/>").unwrap(),
        appended_xml.find("<w:t>C</w:t>").unwrap(),
    ];
    assert!(appended_positions.windows(2).all(|pair| pair[0] < pair[1]));

    let mut formatted = document_with_content_controls(&wrap_word_body(body));
    formatted
        .paragraph_mut(0)
        .unwrap()
        .run_mut(0)
        .unwrap()
        .set_bold(true);
    let formatted_xml = document_xml(&mut formatted);
    let formatted_positions = [
        formatted_xml.find("<w:rPr>").unwrap(),
        formatted_xml.find("<w:before/>").unwrap(),
        formatted_xml.find("<w:t>A</w:t>").unwrap(),
        formatted_xml.find("<w:between/>").unwrap(),
        formatted_xml.find("<w:t>B</w:t>").unwrap(),
        formatted_xml.find("<w:after/>").unwrap(),
    ];
    assert!(formatted_positions.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn comment_removal_remaps_direct_and_control_run_raw_boundaries() {
    let paragraphs = r#"<w:p><w:r><w:directBefore/><w:t>A</w:t><w:directMiddle/><w:commentReference w:id="0"/><w:directAfter/><w:t>B</w:t><w:directLast/></w:r></w:p><w:p><w:sdt><w:sdtPr/><w:sdtContent><w:r><w:controlBefore/><w:t>C</w:t><w:controlMiddle/><w:commentReference w:id="0"/><w:controlAfter/><w:t>D</w:t><w:controlLast/></w:r></w:sdtContent></w:sdt></w:p>"#;
    let (mut document, comment_id) = document_with_comment_paragraphs(paragraphs);

    assert!(document.remove_comment(comment_id).unwrap());
    let output = document_xml(&mut document);
    assert!(!output.contains("commentReference"));
    for ordered in [
        [
            "<w:directBefore/>",
            "<w:t>A</w:t>",
            "<w:directMiddle/>",
            "<w:directAfter/>",
            "<w:t>B</w:t>",
            "<w:directLast/>",
        ],
        [
            "<w:controlBefore/>",
            "<w:t>C</w:t>",
            "<w:controlMiddle/>",
            "<w:controlAfter/>",
            "<w:t>D</w:t>",
            "<w:controlLast/>",
        ],
    ] {
        let positions = ordered
            .iter()
            .map(|needle| output.find(needle).unwrap())
            .collect::<Vec<_>>();
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    }

    let mut reopened = Document::from_bytes(&document.to_bytes().unwrap()).unwrap();
    let reopened_xml = document_xml(&mut reopened);
    for raw in [
        "<w:directBefore/>",
        "<w:directMiddle/>",
        "<w:directAfter/>",
        "<w:directLast/>",
        "<w:controlBefore/>",
        "<w:controlMiddle/>",
        "<w:controlAfter/>",
        "<w:controlLast/>",
    ] {
        assert_eq!(reopened_xml.matches(raw).count(), 1, "raw child {raw}");
    }
}

#[test]
fn comment_insertion_keeps_content_at_the_hyperlink_end_boundary() {
    let raw = r#"<x:end xmlns:x="urn:opaque"/>"#;
    let xml = wrap_word_body(&format!(
        r#"<w:p><w:hyperlink><w:r><w:t>one</w:t></w:r><w:r><w:t>two</w:t></w:r><w:ins w:id="43" w:author="Ada"><w:r><w:t>end</w:t></w:r></w:ins>{raw}</w:hyperlink></w:p>"#
    ));
    let mut document = document_with_content_controls(&xml);
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
            None,
            "review",
        )
        .unwrap();

    let inserted = document_xml(&mut document);
    let revision = inserted.find(r#"w:id="43""#).unwrap();
    let raw_position = inserted.find(raw).unwrap();
    let hyperlink_end = inserted.find("</w:hyperlink>").unwrap();
    let reference = inserted.find("<w:commentReference").unwrap();
    assert!(revision < raw_position && raw_position < hyperlink_end && hyperlink_end < reference);

    assert!(document.remove_comment(comment_id).unwrap());
    let removed = document_xml(&mut document);
    assert!(removed.contains(r#"w:id="43""#));
    assert!(removed.contains(raw));
    assert_eq!(document.accept_revision_id(43).unwrap(), 1);
    assert_eq!(document.text(), "onetwoend\n");
}

#[test]
fn hyperlink_relationship_ids_use_expanded_names_and_safe_output_prefixes() {
    let relationship_namespace =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    let xml = wrap_word_body(&format!(
        r#"<w:p><w:hyperlink xmlns:r="urn:foreign" xmlns:rel="{relationship_namespace}" xmlns:x="urn:other" r:id="wrong" rel:id="right" x:id="other"><w:r><w:t>one</w:t></w:r></w:hyperlink><w:hyperlink xmlns:r="urn:foreign" r:id="still-wrong"><w:r><w:t>two</w:t></w:r></w:hyperlink></w:p>"#
    ));
    let mut document = document_with_content_controls(&xml);

    let paragraph = document.paragraph(0).unwrap();
    let spans = paragraph.hyperlink_spans();
    assert_eq!(spans[0].2, Some("right"));
    assert_eq!(spans[1].2, None);
    let output = document_xml(&mut document);
    assert!(output.contains(r#"xmlns:r="urn:foreign""#));
    assert!(output.contains(r#"r:id="wrong""#));
    assert!(output.contains(r#"r:id="still-wrong""#));
    assert!(output.contains(r#"x:id="other""#));
    assert!(output.contains(
        r#"xmlns:rdocxR="http://schemas.openxmlformats.org/officeDocument/2006/relationships""#
    ));
    assert!(output.contains(r#"rdocxR:id="right""#));

    let reopened = Document::from_bytes(&document.to_bytes().unwrap()).unwrap();
    let paragraph = reopened.paragraph(0).unwrap();
    let spans = paragraph.hyperlink_spans();
    assert_eq!(spans[0].2, Some("right"));
    assert_eq!(spans[1].2, None);
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
fn comment_mutations_keep_bookmarks_and_run_controls_at_their_boundaries() {
    let xml = wrap_word_body(
        r#"<w:p><w:bookmarkStart w:id="4" w:name="destination"/><w:r><w:t>left</w:t></w:r><w:sdt><w:sdtPr><w:tag w:val="run-control"/></w:sdtPr><w:sdtContent><w:r><w:t>wrapped</w:t></w:r></w:sdtContent></w:sdt><w:r><w:t>right</w:t></w:r><w:bookmarkEnd w:id="4"/></w:p>"#,
    );
    let mut document = document_with_content_controls(&xml);
    assert_eq!(document.bookmarks()[0].range().unwrap().end.run_index, 2);

    let comment_id = document
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
            "Review",
        )
        .unwrap();
    assert_eq!(document.bookmarks()[0].range().unwrap().end.run_index, 3);
    assert_eq!(document.content_controls()[0].text(), "wrapped");

    let bytes = document.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(bytes)).unwrap();
    let document_xml =
        String::from_utf8(package.get_part("/word/document.xml").unwrap().to_vec()).unwrap();
    let end = document_xml.find("<w:commentRangeEnd").unwrap();
    let reference = document_xml.find("<w:commentReference").unwrap();
    let control = document_xml.find("<w:sdt>").unwrap();
    let right = document_xml.find("<w:t>right</w:t>").unwrap();
    let bookmark_end = document_xml.find("<w:bookmarkEnd").unwrap();
    assert!(end < reference && reference < control && control < right && right < bookmark_end);

    let mut reopened = Document::from_bytes(
        &document
            .to_bytes()
            .expect("serialize document with comment boundaries"),
    )
    .unwrap();
    assert!(reopened.remove_comment(comment_id).unwrap());
    assert_eq!(reopened.bookmarks()[0].range().unwrap().end.run_index, 2);
    assert_eq!(reopened.content_controls()[0].text(), "wrapped");
    let saved = reopened.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(saved)).unwrap();
    let document_xml =
        String::from_utf8(package.get_part("/word/document.xml").unwrap().to_vec()).unwrap();
    assert!(!document_xml.contains("commentRange"));
    assert!(!document_xml.contains("commentReference"));
    let left = document_xml.find("<w:t>left</w:t>").unwrap();
    let control = document_xml.find("<w:sdt>").unwrap();
    let right = document_xml.find("<w:t>right</w:t>").unwrap();
    let bookmark_end = document_xml.find("<w:bookmarkEnd").unwrap();
    assert!(left < control && control < right && right < bookmark_end);
}

#[test]
fn removing_a_comment_recurses_through_every_typed_control_placement() {
    let mut document = Document::new();
    document.add_paragraph("anchor");
    let comment_id = document
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
            "Nested",
        )
        .unwrap();
    let bytes = document.to_bytes().unwrap();
    let mut package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(bytes)).unwrap();
    let mut document_xml =
        String::from_utf8(package.get_part("/word/document.xml").unwrap().to_vec()).unwrap();

    let reference = document_xml.find("<w:commentReference").unwrap();
    let reference_run_start = document_xml[..reference].rfind("<w:r").unwrap();
    let reference_run_end =
        reference + document_xml[reference..].find("</w:r>").unwrap() + "</w:r>".len();
    let reference_run = document_xml[reference_run_start..reference_run_end].to_owned();
    document_xml.replace_range(
        reference_run_start..reference_run_end,
        &format!("<w:sdt><w:sdtContent>{reference_run}</w:sdtContent></w:sdt>"),
    );

    let paragraph_start = document_xml.find("<w:p").unwrap();
    let paragraph_end =
        paragraph_start + document_xml[paragraph_start..].find("</w:p>").unwrap() + "</w:p>".len();
    let paragraph = document_xml[paragraph_start..paragraph_end].to_owned();
    let nested = format!(
        "<w:sdt><w:sdtContent><w:sdt><w:sdtContent><w:tbl><w:sdt><w:sdtContent><w:tr><w:sdt><w:sdtContent><w:tc><w:sdt><w:sdtContent>{paragraph}</w:sdtContent></w:sdt></w:tc></w:sdtContent></w:sdt></w:tr></w:sdtContent></w:sdt></w:tbl></w:sdtContent></w:sdt></w:sdtContent></w:sdt>"
    );
    document_xml.replace_range(paragraph_start..paragraph_end, &nested);
    package.set_part("/word/document.xml", document_xml.into_bytes());
    let mut output = std::io::Cursor::new(Vec::new());
    package.write_to(&mut output).unwrap();

    let mut reopened = Document::from_bytes(output.get_ref()).unwrap();
    assert_eq!(reopened.comments().len(), 1);
    assert!(reopened.remove_comment(comment_id).unwrap());
    assert!(reopened.comments().is_empty());
    assert_eq!(reopened.paragraphs()[0].text(), "anchor");
    let saved = reopened.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(saved)).unwrap();
    let document_xml =
        String::from_utf8(package.get_part("/word/document.xml").unwrap().to_vec()).unwrap();
    assert!(!document_xml.contains("commentRange"));
    assert!(!document_xml.contains("commentReference"));
}

#[test]
fn mutable_document_indexes_match_recursive_immutable_indexes() {
    let xml = wrap_word_body(
        r#"<w:p><w:r><w:t>one</w:t></w:r></w:p><w:sdt><w:sdtContent><w:p><w:r><w:t>wrapped</w:t></w:r></w:p></w:sdtContent></w:sdt><w:p><w:r><w:t>three</w:t></w:r></w:p><w:sdt><w:sdtContent><w:tbl><w:tr><w:tc><w:p><w:r><w:t>table cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:sdtContent></w:sdt><w:tbl><w:tr><w:tc><w:p><w:r><w:t>direct table</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
    );
    let mut document = document_with_content_controls(&xml);
    assert_eq!(
        document
            .paragraphs()
            .iter()
            .map(|paragraph| paragraph.text())
            .collect::<Vec<_>>(),
        ["one", "wrapped", "three", "table cell"]
    );
    assert_eq!(document.tables().len(), 2);

    document
        .paragraph_mut(1)
        .expect("wrapped paragraph uses immutable index order")
        .add_run(" changed");
    document
        .paragraph_mut(3)
        .expect("deep wrapped paragraph uses immutable index order")
        .add_run(" deep");
    document
        .table_mut(0)
        .expect("wrapped table uses immutable index order")
        .set_style("WrappedTable");

    assert_eq!(document.paragraph(1).unwrap().text(), "wrapped changed");
    assert_eq!(document.paragraph(3).unwrap().text(), "table cell deep");
    assert_eq!(document.table(0).unwrap().style_id(), Some("WrappedTable"));
    assert_eq!(document.table(1).unwrap().style_id(), None);
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
