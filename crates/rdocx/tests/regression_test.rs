//! Regression tests for previously-fixed defects.
//!
//! Each test names the failure it locks down, so a reintroduction is obvious
//! from the test name alone rather than from a diff.

use std::collections::{BTreeMap, HashMap};

use rdocx::{
    Document, FieldDateTime, FieldEvaluationContext, FieldOutcome, Length, RenderOptions,
    RevisionView, RunPosition, RunRange,
};
use rdocx_oxml::CT_Document;
use rdocx_oxml::document::{BodyContent, CT_Body};

fn compatibility_page_elements(
    elements: &[oxml_layout::PositionedElement],
) -> Vec<&oxml_layout::PositionedElement> {
    fn collect<'a>(
        elements: &'a [oxml_layout::PositionedElement],
        output: &mut Vec<&'a oxml_layout::PositionedElement>,
    ) {
        for element in elements {
            match element {
                oxml_layout::PositionedElement::MarkedContent { children, .. } => {
                    collect(children, output);
                }
                other => output.push(other),
            }
        }
    }

    let mut output = Vec::new();
    collect(elements, &mut output);
    output
}

fn clear_compatibility_sources(elements: &mut [oxml_layout::PositionedElement]) {
    for element in elements {
        match element {
            oxml_layout::PositionedElement::Text(run) => run.source = None,
            oxml_layout::PositionedElement::MarkedContent { children, .. } => {
                clear_compatibility_sources(children);
            }
            _ => {}
        }
    }
}

fn remove_compatibility_empty_text(elements: &mut Vec<oxml_layout::PositionedElement>) {
    for element in elements.iter_mut() {
        if let oxml_layout::PositionedElement::MarkedContent { children, .. } = element {
            remove_compatibility_empty_text(children);
        }
    }
    elements.retain(
        |element| !matches!(element, oxml_layout::PositionedElement::Text(run) if run.text.is_empty()),
    );
}

fn document_with_settings(settings_xml: &[u8], target: &str) -> Document {
    let mut seed = Document::new();
    let bytes = seed.to_bytes().unwrap();
    let mut package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(bytes)).unwrap();
    let part_name = oxml_opc::OpcPackage::resolve_rel_target("/word/document.xml", target);
    package.set_part(&part_name, settings_xml.to_vec());
    package.content_types.add_override(
        &part_name,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml",
    );
    package
        .get_or_create_part_rels("/word/document.xml")
        .add(oxml_opc::relationship::rel_types::SETTINGS, target);
    let mut output = std::io::Cursor::new(Vec::new());
    package.write_to(&mut output).unwrap();
    Document::from_bytes(output.get_ref()).unwrap()
}

#[test]
fn each_document_protection_mode_is_reported_with_its_recorded_hash() {
    for (mode, expected) in [
        ("readOnly", rdocx::ProtectionMode::ReadOnly),
        ("comments", rdocx::ProtectionMode::Comments),
        ("trackedChanges", rdocx::ProtectionMode::TrackedChanges),
        ("forms", rdocx::ProtectionMode::Forms),
    ] {
        let settings = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><q:settings xmlns:q="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><q:documentProtection q:edit="{mode}" q:enforcement="1" q:cryptProviderType="rsaAES" q:cryptAlgorithmClass="hash" q:cryptAlgorithmType="typeAny" q:cryptAlgorithmSid="14" q:cryptSpinCount="100000" q:hash="HASH-{mode}" q:salt="SALT-{mode}"/></q:settings>"#
        );
        let mut document = document_with_settings(settings.as_bytes(), "settings.xml");
        let protection = document.document_protection().unwrap();
        assert_eq!(protection.mode, expected);
        assert_eq!(protection.enforcement, Some(true));
        assert_eq!(protection.algorithm_sid, Some(14));
        assert_eq!(protection.spin_count, Some(100_000));
        assert_eq!(
            protection.hash.as_deref(),
            Some(format!("HASH-{mode}").as_str())
        );
        assert_eq!(
            protection.salt.as_deref(),
            Some(format!("SALT-{mode}").as_str())
        );

        let saved = document.to_bytes().unwrap();
        let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(saved)).unwrap();
        assert_eq!(
            package.get_part("/word/settings.xml").unwrap(),
            settings.as_bytes()
        );
    }
}

#[test]
fn malformed_document_protection_remains_opaque_and_unreported() {
    for attributes in [
        r#"q:edit="unsupported" q:cryptSpinCount="100000""#,
        r#"q:edit="forms" q:cryptSpinCount="many""#,
        r#"q:edit="forms" q:cryptAlgorithmSid="SHA-512""#,
        r#"q:edit="forms" q:cryptAlgorithmClass="future""#,
    ] {
        let settings = format!(
            r#"<?xml version="1.0"?><q:settings xmlns:q="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:p="urn:producer"><p:before/><q:documentProtection {attributes}/><p:after/></q:settings>"#
        );
        let mut document = document_with_settings(settings.as_bytes(), "settings.xml");
        assert!(document.document_protection().is_none());

        let saved = document.to_bytes().unwrap();
        let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(saved)).unwrap();
        assert_eq!(
            package.get_part("/word/settings.xml").unwrap(),
            settings.as_bytes()
        );
    }
}

const CUSTOM_XML_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml";
const CUSTOM_XML_PROPS_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXmlProps";
const WORD_REVISION_ORACLE: &str = "Microsoft Word 16.104 build 16.104.25121423";
const WORD_FIELD_ORACLE: &str = "Microsoft Word 16.104 build 16.104.25121423";
const WORD_FIELD_ORACLE_ENVIRONMENT: &str =
    "locale=en-US; calendar=Gregorian; decimal=.; grouping=,; timezone=UTC";
const WORD_FIELD_ORACLE_INPUT: &str = "F-161-readable-field-matrix-v1";

fn document_with_field_parts(
    document_xml: &str,
    settings_xml: Option<&str>,
    custom_properties_xml: Option<&str>,
) -> Document {
    let mut seed = Document::new();
    let bytes = seed.to_bytes().unwrap();
    let mut package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(bytes)).unwrap();
    package.set_part("/word/document.xml", document_xml.as_bytes().to_vec());
    if let Some(settings_xml) = settings_xml {
        package.set_part("/word/settings.xml", settings_xml.as_bytes().to_vec());
        package.content_types.add_override(
            "/word/settings.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml",
        );
        package
            .get_or_create_part_rels("/word/document.xml")
            .add(oxml_opc::relationship::rel_types::SETTINGS, "settings.xml");
    }
    if let Some(custom_properties_xml) = custom_properties_xml {
        package.set_part(
            "/metadata/producer-properties.xml",
            custom_properties_xml.as_bytes().to_vec(),
        );
        package.content_types.add_override(
            "/metadata/producer-properties.xml",
            oxml_opc::content_types::CUSTOM_PROPERTIES,
        );
        package.package_rels.add(
            oxml_opc::relationship::rel_types::CUSTOM_PROPERTIES,
            "metadata/producer-properties.xml",
        );
    }
    let mut output = std::io::Cursor::new(Vec::new());
    package.write_to(&mut output).unwrap();
    Document::from_bytes(output.get_ref()).unwrap()
}

#[test]
fn every_supported_field_matches_the_pinned_word_result() {
    assert_eq!(
        WORD_FIELD_ORACLE,
        "Microsoft Word 16.104 build 16.104.25121423"
    );
    assert_eq!(
        WORD_FIELD_ORACLE_ENVIRONMENT,
        "locale=en-US; calendar=Gregorian; decimal=.; grouping=,; timezone=UTC"
    );
    assert_eq!(WORD_FIELD_ORACLE_INPUT, "F-161-readable-field-matrix-v1");

    let body = r#"
        <w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:bookmarkStart w:id="7" w:name="destination"/><w:r><w:t>Introduction</w:t></w:r><w:bookmarkEnd w:id="7"/></w:p>
        <w:p><w:fldSimple w:instr="IF &quot;10&quot; &gt;= &quot;2&quot; &quot;yes&quot; &quot;no&quot;"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="REF destination"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="PAGEREF destination"><w:r><w:t>1</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="SEQ Figure"><w:r><w:t>0</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="DOCPROPERTY Title"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="DOCVARIABLE Region"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="STYLEREF &quot;Heading 1&quot;"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="INCLUDETEXT &quot;chapter.docx&quot;"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="DATE \@ &quot;MMMM d, yyyy&quot;"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="TIME \@ &quot;h:mm AM/PM&quot;"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="FILENAME"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="AUTHOR"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="MERGEFIELD Name \* Upper"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="PAGE"><w:r><w:t>1</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="NUMPAGES"><w:r><w:t>1</w:t></w:r></w:fldSimple></w:p>
    "#;
    let settings = r#"<q:settings xmlns:q="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><q:docVars><q:docVar q:name="Region" q:val="West"/></q:docVars></q:settings>"#;
    let mut document = document_with_field_parts(&wrap_word_body(body), Some(settings), None);
    document.set_title("Oracle title");
    document.set_author("Ada Lovelace");
    let context = FieldEvaluationContext {
        now: Some(FieldDateTime {
            year: 2025,
            month: 12,
            day: 14,
            hour: 21,
            minute: 7,
            second: 5,
        }),
        file_name: Some("report.docx".to_owned()),
        file_path: Some("/templates/report.docx".to_owned()),
        merge_fields: BTreeMap::from([("Name".to_owned(), "Grace".to_owned())]),
        included_text: BTreeMap::from([("chapter.docx".to_owned(), "Included chapter".to_owned())]),
    };
    let actual = document
        .evaluate_fields(&context)
        .unwrap()
        .into_iter()
        .map(|evaluation| evaluation.outcome)
        .collect::<Vec<_>>();
    let expected = vec![
        FieldOutcome::Resolved("yes".to_owned()),
        FieldOutcome::Resolved("Introduction".to_owned()),
        FieldOutcome::DeferredPagination,
        FieldOutcome::Resolved("1".to_owned()),
        FieldOutcome::Resolved("Oracle title".to_owned()),
        FieldOutcome::Resolved("West".to_owned()),
        FieldOutcome::Resolved("Introduction".to_owned()),
        FieldOutcome::Resolved("Included chapter".to_owned()),
        FieldOutcome::Resolved("December 14, 2025".to_owned()),
        FieldOutcome::Resolved("9:07 PM".to_owned()),
        FieldOutcome::Resolved("report.docx".to_owned()),
        FieldOutcome::Resolved("Ada Lovelace".to_owned()),
        FieldOutcome::Resolved("GRACE".to_owned()),
        FieldOutcome::DeferredPagination,
        FieldOutcome::DeferredPagination,
    ];
    assert_eq!(actual, expected, "{WORD_FIELD_ORACLE_INPUT}");
}

#[test]
fn document_properties_variables_and_author_use_package_values() {
    let body = r#"
        <w:p><w:fldSimple w:instr="DOCPROPERTY ClientTier"><w:r><w:t>stored tier</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="DOCVARIABLE Region"><w:r><w:t>stored region</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="AUTHOR"><w:r><w:t>stored author</w:t></w:r></w:fldSimple></w:p>
    "#;
    let settings = r#"<q:settings xmlns:q="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><q:docVars><q:docVar q:name="Region" q:val="North"/></q:docVars></q:settings>"#;
    let custom = r#"<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/custom-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes"><property fmtid="{D5CDD505-2E9C-101B-9397-08002B2CF9AE}" pid="2" name="ClientTier"><vt:lpwstr>Gold</vt:lpwstr></property></Properties>"#;
    let mut document =
        document_with_field_parts(&wrap_word_body(body), Some(settings), Some(custom));
    document.set_author("Package Author");
    let before = document.to_bytes().unwrap();
    let outcomes = document
        .evaluate_fields(&FieldEvaluationContext::default())
        .unwrap()
        .into_iter()
        .map(|evaluation| evaluation.outcome)
        .collect::<Vec<_>>();
    assert_eq!(
        outcomes,
        [
            FieldOutcome::Resolved("Gold".to_owned()),
            FieldOutcome::Resolved("North".to_owned()),
            FieldOutcome::Resolved("Package Author".to_owned()),
        ]
    );
    assert_eq!(document.to_bytes().unwrap(), before);
}

#[test]
fn resolved_values_drive_date_pictures_and_empty_merge_omits_affixes() {
    let body = r#"
        <w:p><w:fldSimple w:instr="DOCPROPERTY EventDate \@ &quot;MMMM d, yyyy&quot;"><w:r><w:t>stored property</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="MERGEFIELD MergeDate \@ &quot;yyyy-MM-dd&quot;"><w:r><w:t>stored merge date</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="MERGEFIELD Empty \b &quot;Dear &quot; \f &quot;!&quot;"><w:r><w:t>stored empty</w:t></w:r></w:fldSimple></w:p>
    "#;
    let custom = r#"<p:Properties xmlns:p="http://schemas.openxmlformats.org/officeDocument/2006/custom-properties" xmlns:v="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes"><p:property fmtid="{D5CDD505-2E9C-101B-9397-08002B2CF9AE}" pid="2" name="EventDate"><v:filetime>2025-12-14T21:07:05Z</v:filetime></p:property></p:Properties>"#;
    let document = document_with_field_parts(&wrap_word_body(body), None, Some(custom));
    let context = FieldEvaluationContext {
        now: Some(FieldDateTime {
            year: 1999,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
        }),
        merge_fields: BTreeMap::from([
            ("MergeDate".to_owned(), "2026-01-02".to_owned()),
            ("Empty".to_owned(), String::new()),
        ]),
        ..FieldEvaluationContext::default()
    };
    assert_eq!(
        document
            .evaluate_fields(&context)
            .unwrap()
            .into_iter()
            .map(|result| result.outcome)
            .collect::<Vec<_>>(),
        [
            FieldOutcome::Resolved("December 14, 2025".to_owned()),
            FieldOutcome::Resolved("2026-01-02".to_owned()),
            FieldOutcome::Resolved(String::new()),
        ]
    );
}

#[test]
fn foreign_custom_property_lookalikes_are_preserved_but_not_evaluated() {
    let body = r#"<w:p><w:fldSimple w:instr="DOCPROPERTY ClientTier"><w:r><w:t>stored tier</w:t></w:r></w:fldSimple></w:p>"#;
    let foreign = r#"<x:Properties xmlns:x="urn:not-custom-properties" xmlns:v="urn:not-variant-types"><x:property x:fmtid="foreign" x:pid="2" x:name="ClientTier"><v:lpwstr>Injected</v:lpwstr></x:property></x:Properties>"#;
    let mut document = document_with_field_parts(&wrap_word_body(body), None, Some(foreign));
    assert!(matches!(
        document
            .evaluate_fields(&FieldEvaluationContext::default())
            .unwrap()[0]
            .outcome,
        FieldOutcome::KeepStored { .. }
    ));
    let saved = document.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(saved)).unwrap();
    assert_eq!(
        package.get_part("/metadata/producer-properties.xml"),
        Some(foreign.as_bytes())
    );
}

#[test]
fn nested_switch_and_positional_fields_keep_source_order_indices() {
    let body = concat!(
        r#"<w:p>"#,
        r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r>"#,
        r#"<w:r><w:instrText xml:space="preserve">MERGEFIELD \b </w:instrText></w:r>"#,
        r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r>"#,
        r#"<w:r><w:instrText>AUTHOR</w:instrText></w:r>"#,
        r#"<w:r><w:fldChar w:fldCharType="separate"/></w:r>"#,
        r#"<w:r><w:t>stored author</w:t></w:r>"#,
        r#"<w:r><w:fldChar w:fldCharType="end"/></w:r>"#,
        r#"<w:r><w:instrText xml:space="preserve"> </w:instrText></w:r>"#,
        r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r>"#,
        r#"<w:r><w:instrText>MERGEFIELD Name</w:instrText></w:r>"#,
        r#"<w:r><w:fldChar w:fldCharType="separate"/></w:r>"#,
        r#"<w:r><w:t>stored name</w:t></w:r>"#,
        r#"<w:r><w:fldChar w:fldCharType="end"/></w:r>"#,
        r#"<w:r><w:fldChar w:fldCharType="separate"/></w:r>"#,
        r#"<w:r><w:t>stored outer</w:t></w:r>"#,
        r#"<w:r><w:fldChar w:fldCharType="end"/></w:r>"#,
        r#"</w:p>"#,
    );
    let mut document = document_with_field_parts(&wrap_word_body(body), None, None);
    document.set_author("Package Author");
    let context = FieldEvaluationContext {
        merge_fields: BTreeMap::from([("Name".to_owned(), "Ada".to_owned())]),
        ..FieldEvaluationContext::default()
    };
    let evaluations = document.evaluate_fields(&context).unwrap();
    assert_eq!(
        evaluations
            .iter()
            .map(|evaluation| (evaluation.field_index, evaluation.instruction.as_str()))
            .collect::<Vec<_>>(),
        [(0, r"MERGEFIELD \b"), (1, "AUTHOR"), (2, "MERGEFIELD Name"),]
    );
    assert_eq!(
        evaluations[1].outcome,
        FieldOutcome::Resolved("Package Author".to_owned())
    );
    assert_eq!(
        evaluations[2].outcome,
        FieldOutcome::Resolved("Ada".to_owned())
    );

    assert_eq!(document.update_fields(&context).unwrap(), 3);
    let updated = document.evaluate_fields(&context).unwrap();
    assert_eq!(updated[0].cached_result, "stored outer");
    assert_eq!(updated[1].cached_result, "Package Author");
    assert_eq!(updated[2].cached_result, "Ada");
    let reopened = Document::from_bytes(&document.to_bytes().unwrap()).unwrap();
    let reopened = reopened.evaluate_fields(&context).unwrap();
    assert_eq!(reopened[1].cached_result, "Package Author");
    assert_eq!(reopened[2].cached_result, "Ada");
}

#[test]
fn nested_if_operand_keeps_its_position_after_cache_updates() {
    let body = concat!(
        r#"<w:p><w:bookmarkStart w:id="7" w:name="destination"/><w:r><w:t>x</w:t></w:r><w:bookmarkEnd w:id="7"/></w:p>"#,
        r#"<w:p>"#,
        r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r>"#,
        r#"<w:r><w:instrText xml:space="preserve">IF </w:instrText></w:r>"#,
        r#"<w:r><w:fldChar w:fldCharType="begin" w:dirty="1"/></w:r>"#,
        r#"<w:r><w:instrText>REF destination</w:instrText></w:r>"#,
        r#"<w:r><w:fldChar w:fldCharType="separate"/></w:r>"#,
        r#"<w:r><w:t>old reference</w:t></w:r>"#,
        r#"<w:r><w:fldChar w:fldCharType="end"/></w:r>"#,
        r#"<w:r><w:instrText xml:space="preserve"> = &quot;x&quot; &quot;yes&quot; &quot;no&quot;</w:instrText></w:r>"#,
        r#"<w:r><w:fldChar w:fldCharType="separate"/></w:r>"#,
        r#"<w:r><w:t>old outcome</w:t></w:r>"#,
        r#"<w:r><w:fldChar w:fldCharType="end"/></w:r>"#,
        r#"</w:p>"#,
    );
    let context = FieldEvaluationContext::default();
    let mut document = document_with_field_parts(&wrap_word_body(body), None, None);
    let initial = document.evaluate_fields(&context).unwrap();
    assert_eq!(initial[0].outcome, FieldOutcome::Resolved("yes".to_owned()));
    assert_eq!(initial[1].outcome, FieldOutcome::Resolved("x".to_owned()));

    assert_eq!(document.update_fields(&context).unwrap(), 2);
    let saved = document.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(&saved)).unwrap();
    let xml = std::str::from_utf8(package.get_part("/word/document.xml").unwrap()).unwrap();
    assert!(
        xml.find("REF destination").unwrap() < xml.find(" = &quot;x&quot;").unwrap(),
        "{xml}"
    );

    let reopened = Document::from_bytes(&saved).unwrap();
    let reopened = reopened.evaluate_fields(&context).unwrap();
    assert_eq!(reopened[0].cached_result, "yes");
    assert_eq!(
        reopened[0].outcome,
        FieldOutcome::Resolved("yes".to_owned())
    );
    assert_eq!(reopened[1].cached_result, "x");
}

#[test]
fn missing_context_and_unsupported_fields_keep_their_cached_display() {
    let body = r#"
        <w:p><w:fldSimple w:instr="DATE"><w:r><w:t>stored date</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="UNKNOWN"><w:r><w:t>stored unknown</w:t></w:r></w:fldSimple></w:p>
    "#;
    let document = document_with_field_parts(&wrap_word_body(body), None, None);
    let results = document
        .evaluate_fields(&FieldEvaluationContext::default())
        .unwrap();
    assert_eq!(results[0].cached_result, "stored date");
    assert_eq!(
        results[0].outcome,
        FieldOutcome::KeepStored {
            diagnostic: "DATE requires an explicit date and time, stored display retained"
                .to_owned(),
        }
    );
    assert_eq!(results[1].cached_result, "stored unknown");
    assert_eq!(
        results[1].outcome,
        FieldOutcome::KeepStored {
            diagnostic: "field UNKNOWN is unsupported, stored display retained".to_owned(),
        }
    );
}

#[test]
fn field_update_policies_produce_the_expected_result_cache_and_dirty_flag() {
    let body = concat!(
        r#"<w:p><w:fldSimple w:instr="MERGEFIELD Name" w:dirty="1"><w:r><w:t>stored name</w:t></w:r></w:fldSimple></w:p>"#,
        r#"<w:p><w:fldSimple w:instr="PAGE"><w:r><w:t>7</w:t></w:r></w:fldSimple></w:p>"#,
    );
    let context = FieldEvaluationContext {
        merge_fields: BTreeMap::from([("Name".to_owned(), "Ada".to_owned())]),
        ..FieldEvaluationContext::default()
    };

    let mut on_demand = document_with_field_parts(&wrap_word_body(body), None, None);
    assert_eq!(on_demand.update_fields(&context).unwrap(), 2);
    let updated = document_xml(&mut on_demand);
    assert!(updated.contains(r#"w:instr="MERGEFIELD Name" w:dirty="0""#));
    assert!(updated.contains("<w:t>Ada</w:t>"));
    assert!(updated.contains(r#"w:instr="PAGE" w:dirty="1""#));
    assert!(updated.contains("<w:t>7</w:t>"));

    let mut on_save = document_with_field_parts(&wrap_word_body(body), None, None);
    let bytes = on_save.to_bytes_with_field_updates(&context).unwrap();
    let reopened = Document::from_bytes(&bytes).unwrap();
    let outcomes = reopened.evaluate_fields(&context).unwrap();
    assert_eq!(outcomes[0].cached_result, "Ada");
    assert_eq!(outcomes[1].cached_result, "7");

    let mut file_save = document_with_field_parts(&wrap_word_body(body), None, None);
    let path = std::env::temp_dir().join(format!(
        "rdocx-f162-field-update-{}.docx",
        std::process::id()
    ));
    file_save.save_with_field_updates(&path, &context).unwrap();
    let reopened = Document::open(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    let outcomes = reopened.evaluate_fields(&context).unwrap();
    assert_eq!(outcomes[0].cached_result, "Ada");
    assert_eq!(outcomes[1].cached_result, "7");
}

#[test]
fn unsupported_fields_keep_their_cached_result_when_updates_run() {
    let body = concat!(
        r#"<w:p><w:fldSimple w:instr="UNKNOWN"><w:r><w:t>producer display</w:t></w:r></w:fldSimple></w:p>"#,
        r#"<w:p><w:fldSimple w:instr="DATE"><w:r><w:t>stored date</w:t></w:r></w:fldSimple></w:p>"#,
    );
    let mut document = document_with_field_parts(&wrap_word_body(body), None, None);

    assert_eq!(
        document
            .update_fields(&FieldEvaluationContext::default())
            .unwrap(),
        2
    );
    let xml = document_xml(&mut document);
    assert!(xml.contains("<w:t>producer display</w:t>"));
    assert!(xml.contains("<w:t>stored date</w:t>"));
    assert_eq!(xml.matches(r#"w:dirty="1""#).count(), 2);
}

#[test]
fn ordinary_save_leaves_cached_field_results_and_dirty_flags_alone() {
    let body = concat!(
        r#"<w:p xmlns:q="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><q:fldSimple q:instr="MERGEFIELD Name" q:dirty="on" data="producer"><q:r><q:t>stored</q:t><q:producer/></q:r></q:fldSimple></w:p>"#,
        r#"<w:p><w:fldSimple w:instr="PAGE" w:dirty="off"><w:r><w:t>9</w:t></w:r></w:fldSimple></w:p>"#,
    );
    let mut document = document_with_field_parts(&wrap_word_body(body), None, None);

    let xml = document_xml(&mut document);
    assert!(xml.contains(r#"q:dirty="on" data="producer""#));
    assert!(xml.contains("<q:t>stored</q:t><q:producer/>"));
    assert!(xml.contains(r#"w:dirty="off""#));
    assert!(xml.contains("<w:t>9</w:t>"));
}

#[test]
fn field_update_failure_leaves_document_bytes_unchanged() {
    let body = r#"<w:p><w:fldSimple w:instr="MERGEFIELD Name"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>"#;
    let mut document = document_with_field_parts(&wrap_word_body(body), None, None);
    let before = document.to_bytes().unwrap();
    let context = FieldEvaluationContext {
        merge_fields: BTreeMap::from([("Name".to_owned(), "invalid\0xml".to_owned())]),
        ..FieldEvaluationContext::default()
    };

    assert!(document.update_fields(&context).is_err());
    assert_eq!(document.to_bytes().unwrap(), before);
}

#[test]
fn typed_story_fields_have_stable_order_and_physical_parts_are_evaluated_once() {
    let body = r#"
        <w:p><w:fldSimple w:instr="MERGEFIELD Main"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:tbl><w:tblPr/><w:tblGrid/><w:sdt><w:sdtPr><w:richText/></w:sdtPr><w:sdtContent><w:tr><w:tc><w:p><w:fldSimple w:instr="MERGEFIELD TableControl"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p></w:tc></w:tr></w:sdtContent></w:sdt><w:tr><w:tc><w:tcPr/><w:sdt><w:sdtPr><w:richText/></w:sdtPr><w:sdtContent><w:p><w:fldSimple w:instr="MERGEFIELD CellControl"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p></w:sdtContent></w:sdt><w:p><w:fldSimple w:instr="MERGEFIELD Table"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p></w:tc></w:tr></w:tbl>
        <w:sdt><w:sdtPr><w:richText/></w:sdtPr><w:sdtContent><w:p><w:fldSimple w:instr="MERGEFIELD Control"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p></w:sdtContent></w:sdt>
    "#;
    let mut document = document_with_field_parts(&wrap_word_body(body), None, None);
    let mut package =
        oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(document.to_bytes().unwrap()))
            .unwrap();
    let header = r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:fldSimple w:instr="MERGEFIELD Header"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p></w:hdr>"#;
    let earlier_header = r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:fldSimple w:instr="MERGEFIELD EarlierHeader"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p></w:hdr>"#;
    let orphan_header = r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:fldSimple w:instr="MERGEFIELD Orphan"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p></w:hdr>"#;
    let footer = r#"<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:fldSimple w:instr="MERGEFIELD Footer"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p></w:ftr>"#;
    let footnotes = r#"<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:footnote w:id="2"><w:p><w:fldSimple w:instr="MERGEFIELD Footnote"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p></w:footnote></w:footnotes>"#;
    let endnotes = r#"<w:endnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:endnote w:id="2"><w:p><w:fldSimple w:instr="MERGEFIELD Endnote"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p></w:endnote></w:endnotes>"#;
    package.set_part("/word/header1.xml", header.as_bytes().to_vec());
    package.set_part("/word/header2.xml", earlier_header.as_bytes().to_vec());
    package.set_part("/word/orphan-header.xml", orphan_header.as_bytes().to_vec());
    package.set_part("/word/footer1.xml", footer.as_bytes().to_vec());
    package.set_part("/word/footnotes.xml", footnotes.as_bytes().to_vec());
    package.set_part("/word/notes/end-stream.xml", endnotes.as_bytes().to_vec());
    let (header_id, duplicate_header_id, earlier_header_id, footer_id) = {
        let relationships = package.get_or_create_part_rels("/word/document.xml");
        let header_id = relationships.add(oxml_opc::relationship::rel_types::HEADER, "header1.xml");
        let duplicate_header_id =
            relationships.add(oxml_opc::relationship::rel_types::HEADER, "header1.xml");
        let earlier_header_id =
            relationships.add(oxml_opc::relationship::rel_types::HEADER, "header2.xml");
        relationships.add(
            oxml_opc::relationship::rel_types::HEADER,
            "orphan-header.xml",
        );
        let footer_id = relationships.add(oxml_opc::relationship::rel_types::FOOTER, "footer1.xml");
        relationships.add(
            oxml_opc::relationship::rel_types::FOOTNOTES,
            "footnotes.xml",
        );
        relationships.add(
            oxml_opc::relationship::rel_types::ENDNOTES,
            "notes/end-stream.xml",
        );
        (header_id, duplicate_header_id, earlier_header_id, footer_id)
    };
    package.set_part(
        "/word/document.xml",
        format!(
            r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body>{body}<w:p><w:pPr><w:sectPr><w:headerReference w:type="default" r:id="{earlier_header_id}"/></w:sectPr></w:pPr></w:p><w:sectPr><w:headerReference w:type="default" r:id="{header_id}"/><w:headerReference w:type="even" r:id="{duplicate_header_id}"/><w:footerReference w:type="default" r:id="{footer_id}"/></w:sectPr></w:body></w:document>"#
        )
        .into_bytes(),
    );
    let mut bytes = std::io::Cursor::new(Vec::new());
    package.write_to(&mut bytes).unwrap();
    let mut document = Document::from_bytes(bytes.get_ref()).unwrap();
    let names = [
        "Main",
        "TableControl",
        "CellControl",
        "Table",
        "Control",
        "EarlierHeader",
        "Header",
        "Footer",
        "Footnote",
        "Endnote",
    ];
    let context = FieldEvaluationContext {
        merge_fields: names
            .iter()
            .map(|name| ((*name).to_owned(), format!("{name} value")))
            .collect(),
        ..FieldEvaluationContext::default()
    };
    let results = document.evaluate_fields(&context).unwrap();
    assert_eq!(
        results
            .iter()
            .map(|result| result.field_index)
            .collect::<Vec<_>>(),
        (0..10).collect::<Vec<_>>()
    );
    assert_eq!(
        results
            .into_iter()
            .map(|result| result.outcome)
            .collect::<Vec<_>>(),
        names
            .iter()
            .map(|name| FieldOutcome::Resolved(format!("{name} value")))
            .collect::<Vec<_>>()
    );

    assert_eq!(document.update_fields(&context).unwrap(), 10);
    let saved = document.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(saved)).unwrap();
    for (part_name, expected_values) in [
        ("/word/document.xml", &names[..5]),
        ("/word/header2.xml", &names[5..6]),
        ("/word/header1.xml", &names[6..7]),
        ("/word/footer1.xml", &names[7..8]),
        ("/word/footnotes.xml", &names[8..9]),
        ("/word/notes/end-stream.xml", &names[9..10]),
    ] {
        let xml = String::from_utf8(package.get_part(part_name).unwrap().to_vec()).unwrap();
        for name in expected_values {
            assert!(xml.contains(&format!("<w:t>{name} value</w:t>")), "{xml}");
        }
    }
    let orphan = String::from_utf8(
        package
            .get_part("/word/orphan-header.xml")
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(orphan.contains("<w:t>stored</w:t>"));
}

#[test]
fn package_story_updates_preserve_aliased_and_unmodelled_part_boundaries() {
    let mut document = document_with_field_parts(&wrap_word_body(""), None, None);
    let mut package =
        oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(document.to_bytes().unwrap()))
            .unwrap();
    let word_namespace = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let header = format!(
        r#"<q:hdr xmlns:q="{word_namespace}" xmlns:x="urn:producer"><x:headerBefore data="A"/><q:p data="header-paragraph"><q:fldSimple q:instr="MERGEFIELD Header" q:dirty="on" x:token="header"><q:r><q:rPr><q:b/></q:rPr><q:t>stored header</q:t><x:insideHeader/></q:r></q:fldSimple></q:p><x:headerAfter data="B"/></q:hdr>"#,
    );
    let footer = format!(
        r#"<q:ftr xmlns:q="{word_namespace}" xmlns:x="urn:producer"><x:footerBefore/><q:p><q:fldSimple q:instr="MERGEFIELD Footer"><q:r><q:t>stored footer</q:t></q:r></q:fldSimple></q:p><x:footerAfter/></q:ftr>"#,
    );
    let endnotes = format!(
        r#"<q:endnotes xmlns:q="{word_namespace}" xmlns:x="urn:producer"><x:rootBefore/><q:endnote q:id="2" x:token="note"><x:noteBefore/><q:p data="endnote-paragraph"><q:fldSimple q:instr="MERGEFIELD Endnote"><q:r><q:rPr><q:i/></q:rPr><q:t>stored endnote</q:t><x:insideEndnote/></q:r></q:fldSimple></q:p><x:noteAfter/></q:endnote><x:rootAfter/></q:endnotes>"#,
    );
    package.set_part("/word/header-preserve.xml", header.as_bytes().to_vec());
    package.set_part("/word/footer-preserve.xml", footer.as_bytes().to_vec());
    package.set_part("/word/endnotes-preserve.xml", endnotes.as_bytes().to_vec());
    let (header_id, footer_id) = {
        let relationships = package.get_or_create_part_rels("/word/document.xml");
        let header_id = relationships.add(
            oxml_opc::relationship::rel_types::HEADER,
            "header-preserve.xml",
        );
        let footer_id = relationships.add(
            oxml_opc::relationship::rel_types::FOOTER,
            "footer-preserve.xml",
        );
        relationships.add(
            oxml_opc::relationship::rel_types::ENDNOTES,
            "endnotes-preserve.xml",
        );
        (header_id, footer_id)
    };
    package.set_part(
        "/word/document.xml",
        format!(
            r#"<?xml version="1.0"?><w:document xmlns:w="{word_namespace}" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body><w:sectPr><w:headerReference w:type="default" r:id="{header_id}"/><w:footerReference w:type="default" r:id="{footer_id}"/></w:sectPr></w:body></w:document>"#,
        )
        .into_bytes(),
    );
    let mut bytes = std::io::Cursor::new(Vec::new());
    package.write_to(&mut bytes).unwrap();
    let mut document = Document::from_bytes(bytes.get_ref()).unwrap();
    let context = FieldEvaluationContext {
        merge_fields: BTreeMap::from([
            ("Header".to_owned(), "updated header".to_owned()),
            ("Footer".to_owned(), "updated footer".to_owned()),
            ("Endnote".to_owned(), "updated endnote".to_owned()),
        ]),
        ..FieldEvaluationContext::default()
    };

    assert_eq!(document.update_fields(&context).unwrap(), 3);
    let saved = document.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(saved)).unwrap();
    let header =
        std::str::from_utf8(package.get_part("/word/header-preserve.xml").unwrap()).unwrap();
    let footer =
        std::str::from_utf8(package.get_part("/word/footer-preserve.xml").unwrap()).unwrap();
    let endnotes =
        std::str::from_utf8(package.get_part("/word/endnotes-preserve.xml").unwrap()).unwrap();

    assert!(header.contains(r#"<x:headerBefore data="A"/>"#), "{header}");
    assert!(header.contains(r#"x:token="header""#), "{header}");
    assert!(header.contains(r#"<q:rPr><q:b/></q:rPr>"#), "{header}");
    assert!(header.contains("<x:insideHeader/>"), "{header}");
    assert!(header.contains("updated header"), "{header}");
    assert!(
        header.find("<x:headerBefore").unwrap()
            < header.find(r#"<q:p data="header-paragraph""#).unwrap()
            && header.find(r#"<q:p data="header-paragraph""#).unwrap()
                < header.find("<x:headerAfter").unwrap(),
        "{header}"
    );
    assert!(footer.contains("updated footer"), "{footer}");
    assert!(
        footer.find("<x:footerBefore").unwrap() < footer.find("<q:p>").unwrap()
            && footer.find("<q:p>").unwrap() < footer.find("<x:footerAfter").unwrap(),
        "{footer}"
    );
    for preserved in [
        "<x:rootBefore/>",
        r#"x:token="note""#,
        "<x:noteBefore/>",
        r#"<q:rPr><q:i/></q:rPr>"#,
        "<x:insideEndnote/>",
        "<x:noteAfter/>",
        "<x:rootAfter/>",
    ] {
        assert!(
            endnotes.contains(preserved),
            "missing {preserved}: {endnotes}"
        );
    }
    assert!(endnotes.contains("updated endnote"), "{endnotes}");
    assert!(
        endnotes.find("<x:rootBefore").unwrap() < endnotes.find(r#"<q:endnote q:id="2""#).unwrap()
            && endnotes.find("<x:noteBefore").unwrap()
                < endnotes.find(r#"<q:p data="endnote-paragraph""#).unwrap()
            && endnotes.find(r#"<q:p data="endnote-paragraph""#).unwrap()
                < endnotes.find("<x:noteAfter").unwrap()
            && endnotes.find(r#"<q:endnote q:id="2""#).unwrap()
                < endnotes.find("<x:rootAfter").unwrap(),
        "{endnotes}"
    );
}

#[test]
fn pretty_printed_complex_package_field_updates_in_place() {
    let mut document = document_with_field_parts(&wrap_word_body(""), None, None);
    let mut package =
        oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(document.to_bytes().unwrap()))
            .unwrap();
    let word_namespace = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let header = format!(
        r#"<q:hdr xmlns:q="{word_namespace}" xmlns:x="urn:producer">
  <x:before/>
  <q:p>
    <q:r><q:fldChar q:fldCharType="begin" q:dirty="on"/></q:r>
    <q:r><q:instrText>MERGEFIELD Pretty</q:instrText></q:r>
    <q:r><q:fldChar q:fldCharType="separate"/></q:r>
    <q:r><q:rPr><q:b/></q:rPr><q:t>stored pretty</q:t><x:inside/></q:r>
    <q:r><q:fldChar q:fldCharType="end"/></q:r>
  </q:p>
  <x:after/>
</q:hdr>"#,
    );
    package.set_part("/word/header-pretty.xml", header.as_bytes().to_vec());
    let header_id = package.get_or_create_part_rels("/word/document.xml").add(
        oxml_opc::relationship::rel_types::HEADER,
        "header-pretty.xml",
    );
    package.set_part(
        "/word/document.xml",
        format!(
            r#"<?xml version="1.0"?><w:document xmlns:w="{word_namespace}" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body><w:sectPr><w:headerReference w:type="default" r:id="{header_id}"/></w:sectPr></w:body></w:document>"#,
        )
        .into_bytes(),
    );
    let mut bytes = std::io::Cursor::new(Vec::new());
    package.write_to(&mut bytes).unwrap();
    let mut document = Document::from_bytes(bytes.get_ref()).unwrap();
    let context = FieldEvaluationContext {
        merge_fields: BTreeMap::from([("Pretty".to_owned(), "updated pretty".to_owned())]),
        ..FieldEvaluationContext::default()
    };

    assert_eq!(document.update_fields(&context).unwrap(), 1);
    let saved = document.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(saved)).unwrap();
    let header = std::str::from_utf8(package.get_part("/word/header-pretty.xml").unwrap()).unwrap();
    assert!(header.contains("updated pretty"), "{header}");
    assert!(header.contains("\n    <q:r>"), "{header}");
    assert!(header.contains("<q:rPr><q:b/></q:rPr>"), "{header}");
    assert!(header.contains("<x:inside/>"), "{header}");
    assert!(
        header.find("<x:before").unwrap() < header.find("updated pretty").unwrap()
            && header.find("updated pretty").unwrap() < header.find("<x:after").unwrap(),
        "{header}"
    );
}

#[test]
fn complex_field_updates_preserve_inter_run_comments_and_instructions() {
    let mut seed = document_with_field_parts(&wrap_word_body(""), None, None);
    let mut package =
        oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(seed.to_bytes().unwrap())).unwrap();
    let word_namespace = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let header_field = concat!(
        r#"<q:r><q:fldChar q:fldCharType="begin" q:dirty="on"/></q:r>"#,
        "<!-- header-before-instruction -->",
        r#"<q:r><q:instrText>MERGEFIELD Producer</q:instrText></q:r>"#,
        "<?producer header-before-separator?>",
        r#"<q:r><q:fldChar q:fldCharType="separate"/></q:r>"#,
        "<!-- header-before-result -->",
        r#"<q:r><q:t>stored header</q:t></q:r>"#,
        "<?producer header-before-end?>",
        r#"<q:r><q:fldChar q:fldCharType="end"/></q:r>"#,
    );
    let header = format!(r#"<q:hdr xmlns:q="{word_namespace}"><q:p>{header_field}</q:p></q:hdr>"#,);
    package.set_part("/word/header-events.xml", header.into_bytes());
    let header_id = package.get_or_create_part_rels("/word/document.xml").add(
        oxml_opc::relationship::rel_types::HEADER,
        "header-events.xml",
    );
    let main_field = concat!(
        r#"<w:r><w:fldChar w:fldCharType="begin" w:dirty="on"/></w:r>"#,
        "<!-- main-before-instruction -->",
        r#"<w:r><w:instrText>MERGEFIELD Producer</w:instrText></w:r>"#,
        "<?producer main-before-separator?>",
        r#"<w:r><w:fldChar w:fldCharType="separate"/></w:r>"#,
        "<!-- main-before-result -->",
        r#"<w:r><w:t>stored main</w:t></w:r>"#,
        "<?producer main-before-end?>",
        r#"<w:r><w:fldChar w:fldCharType="end"/></w:r>"#,
    );
    package.set_part(
        "/word/document.xml",
        format!(
            r#"<?xml version="1.0"?><w:document xmlns:w="{word_namespace}" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body><w:p>{main_field}</w:p><w:sectPr><w:headerReference w:type="default" r:id="{header_id}"/></w:sectPr></w:body></w:document>"#,
        )
        .into_bytes(),
    );
    let mut bytes = std::io::Cursor::new(Vec::new());
    package.write_to(&mut bytes).unwrap();
    let mut document = Document::from_bytes(bytes.get_ref()).unwrap();
    let context = FieldEvaluationContext {
        merge_fields: BTreeMap::from([("Producer".to_owned(), "updated producer".to_owned())]),
        ..FieldEvaluationContext::default()
    };

    assert_eq!(document.update_fields(&context).unwrap(), 2);
    let saved = document.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(saved)).unwrap();
    let main = std::str::from_utf8(package.get_part("/word/document.xml").unwrap()).unwrap();
    let header = std::str::from_utf8(package.get_part("/word/header-events.xml").unwrap()).unwrap();
    for preserved in [
        "<!-- main-before-instruction -->",
        "<?producer main-before-separator?>",
        "<!-- main-before-result -->",
        "<?producer main-before-end?>",
    ] {
        assert!(main.contains(preserved), "missing {preserved}: {main}");
    }
    for preserved in [
        "<!-- header-before-instruction -->",
        "<?producer header-before-separator?>",
        "<!-- header-before-result -->",
        "<?producer header-before-end?>",
    ] {
        assert!(header.contains(preserved), "missing {preserved}: {header}");
    }
    assert_eq!(main.matches("updated producer").count(), 1, "{main}");
    assert_eq!(header.matches("updated producer").count(), 1, "{header}");
}

#[test]
fn hyperlink_field_trivia_survives_package_update() {
    let mut seed = document_with_field_parts(&wrap_word_body(""), None, None);
    let mut package =
        oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(seed.to_bytes().unwrap())).unwrap();
    let word_namespace = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let header = format!(
        concat!(
            r#"<q:hdr xmlns:q="{0}"><q:p><q:hyperlink q:anchor="destination">"#,
            "\n  ",
            r#"<q:r><q:fldChar q:fldCharType="begin" q:dirty="on"/></q:r>"#,
            "\n  <!-- hyperlink-package-before-instruction -->",
            r#"<q:r><q:instrText>MERGEFIELD Link</q:instrText></q:r>"#,
            "<?producer hyperlink-package-before-separator?>",
            r#"<q:r><q:fldChar q:fldCharType="separate"/></q:r>"#,
            "\n  <!-- hyperlink-package-before-result -->",
            r#"<q:r><q:t>stored link</q:t></q:r>"#,
            "<?producer hyperlink-package-before-end?>",
            r#"<q:r><q:fldChar q:fldCharType="end"/></q:r>"#,
            "\n",
            r#"</q:hyperlink></q:p></q:hdr>"#,
        ),
        word_namespace,
    );
    package.set_part("/word/header-hyperlink-events.xml", header.into_bytes());
    let header_id = package.get_or_create_part_rels("/word/document.xml").add(
        oxml_opc::relationship::rel_types::HEADER,
        "header-hyperlink-events.xml",
    );
    package.set_part(
        "/word/document.xml",
        format!(
            r#"<?xml version="1.0"?><w:document xmlns:w="{word_namespace}" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body><w:sectPr><w:headerReference w:type="default" r:id="{header_id}"/></w:sectPr></w:body></w:document>"#,
        )
        .into_bytes(),
    );
    let mut bytes = std::io::Cursor::new(Vec::new());
    package.write_to(&mut bytes).unwrap();
    let mut document = Document::from_bytes(bytes.get_ref()).unwrap();
    let context = FieldEvaluationContext {
        merge_fields: BTreeMap::from([("Link".to_owned(), "updated link".to_owned())]),
        ..FieldEvaluationContext::default()
    };

    assert_eq!(document.update_fields(&context).unwrap(), 1);
    let saved = document.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(saved)).unwrap();
    let header = std::str::from_utf8(
        package
            .get_part("/word/header-hyperlink-events.xml")
            .unwrap(),
    )
    .unwrap();
    for preserved in [
        "<!-- hyperlink-package-before-instruction -->",
        "<?producer hyperlink-package-before-separator?>",
        "<!-- hyperlink-package-before-result -->",
        "<?producer hyperlink-package-before-end?>",
    ] {
        assert!(header.contains(preserved), "missing {preserved}: {header}");
    }
    assert_eq!(header.matches("updated link").count(), 1, "{header}");
    assert!(header.contains("<q:hyperlink"), "{header}");
}

#[test]
fn identical_aliased_same_run_nested_fields_update_and_reopen() {
    let word_namespace = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let nested = concat!(
        r#"<q:fldChar q:fldCharType="begin" q:dirty="on"/>"#,
        r#"<q:instrText>MERGEFIELD Same</q:instrText>"#,
        r#"<q:fldChar q:fldCharType="separate"/>"#,
        r#"<q:t>stored same run</q:t>"#,
        r#"<q:fldChar q:fldCharType="end"/>"#,
    );
    let body = format!(
        concat!(
            r#"<w:p><q:r xmlns:q="{0}" xmlns:x="urn:producer"><x:before/>"#,
            r#"<q:fldChar q:fldCharType="begin" q:dirty="on"/><q:instrText xml:space="preserve">IF </q:instrText>"#,
            "{1}",
            r#"<q:instrText xml:space="preserve"> = </q:instrText>"#,
            "{1}",
            r#"<q:instrText xml:space="preserve"> &quot;yes&quot; &quot;no&quot;</q:instrText>"#,
            r#"<q:fldChar q:fldCharType="separate"/><q:t>stored outer</q:t>"#,
            r#"<q:fldChar q:fldCharType="end"/><x:after/></q:r></w:p>"#,
        ),
        word_namespace, nested,
    );
    let context = FieldEvaluationContext {
        merge_fields: BTreeMap::from([("Same".to_owned(), "updated same run".to_owned())]),
        ..FieldEvaluationContext::default()
    };
    let mut document = document_with_field_parts(&wrap_word_body(&body), None, None);

    assert_eq!(document.update_fields(&context).unwrap(), 3);
    let saved = document.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(&saved)).unwrap();
    let xml = std::str::from_utf8(package.get_part("/word/document.xml").unwrap()).unwrap();
    assert_eq!(xml.matches("updated same run").count(), 2, "{xml}");
    assert!(!xml.contains("stored same run"), "{xml}");
    assert!(xml.contains("<x:before/>"), "{xml}");
    assert!(xml.contains("<x:after/>"), "{xml}");

    let reopened = Document::from_bytes(&saved).unwrap();
    let evaluations = reopened.evaluate_fields(&context).unwrap();
    assert_eq!(evaluations.len(), 3);
    assert_eq!(evaluations[0].cached_result, "yes");
    assert_eq!(evaluations[1].cached_result, "updated same run");
    assert_eq!(evaluations[2].cached_result, "updated same run");
}

#[test]
fn shared_boundary_run_nested_fields_update_in_order_and_reopen() {
    let word_namespace = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let body = format!(
        concat!(
            r#"<w:p><q:r xmlns:q="{0}"><q:fldChar q:fldCharType="begin" q:dirty="on"/><q:instrText xml:space="preserve">IF </q:instrText></q:r>"#,
            r#"<q:r xmlns:q="{0}"><q:fldChar q:fldCharType="begin" q:dirty="on"/><q:instrText>MERGEFIELD First</q:instrText></q:r>"#,
            r#"<q:r xmlns:q="{0}" xmlns:x="urn:producer"><q:fldChar q:fldCharType="separate"/><q:t>stored first</q:t><q:fldChar q:fldCharType="end"/><x:between/><q:instrText xml:space="preserve"> = </q:instrText><q:fldChar q:fldCharType="begin" q:dirty="on"/></q:r>"#,
            r#"<q:r xmlns:q="{0}"><q:instrText>MERGEFIELD Second</q:instrText><q:fldChar q:fldCharType="separate"/><q:t>stored second</q:t><q:fldChar q:fldCharType="end"/><q:instrText xml:space="preserve"> &quot;yes&quot; &quot;no&quot;</q:instrText></q:r>"#,
            r#"<q:r xmlns:q="{0}"><q:fldChar q:fldCharType="separate"/><q:t>stored outer</q:t><q:fldChar q:fldCharType="end"/></q:r></w:p>"#,
        ),
        word_namespace,
    );
    let context = FieldEvaluationContext {
        merge_fields: BTreeMap::from([
            ("First".to_owned(), "updated first boundary".to_owned()),
            ("Second".to_owned(), "updated second boundary".to_owned()),
        ]),
        ..FieldEvaluationContext::default()
    };
    let mut document = document_with_field_parts(&wrap_word_body(&body), None, None);

    assert_eq!(document.update_fields(&context).unwrap(), 3);
    let saved = document.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(&saved)).unwrap();
    let xml = std::str::from_utf8(package.get_part("/word/document.xml").unwrap()).unwrap();
    assert_eq!(xml.matches("updated first boundary").count(), 1, "{xml}");
    assert_eq!(xml.matches("updated second boundary").count(), 1, "{xml}");
    assert!(xml.contains("<x:between/>"), "{xml}");
    assert!(
        xml.find("updated first boundary").unwrap() < xml.find("<x:between/>").unwrap()
            && xml.find("<x:between/>").unwrap() < xml.find("updated second boundary").unwrap(),
        "{xml}"
    );

    let reopened = Document::from_bytes(&saved).unwrap();
    let evaluations = reopened.evaluate_fields(&context).unwrap();
    assert_eq!(evaluations.len(), 3);
    assert_eq!(evaluations[0].cached_result, "no");
    assert_eq!(evaluations[1].cached_result, "updated first boundary");
    assert_eq!(evaluations[2].cached_result, "updated second boundary");
}

#[test]
fn package_field_patching_skips_opaque_lookalikes_and_maps_identical_aliases() {
    let mut document = document_with_field_parts(&wrap_word_body(""), None, None);
    let mut package =
        oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(document.to_bytes().unwrap()))
            .unwrap();
    let word_namespace = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let field =
        r#"<q:fldSimple q:instr="MERGEFIELD Same"><q:r><q:t>stored same</q:t></q:r></q:fldSimple>"#;
    let header = format!(
        r#"<q:hdr xmlns:q="{word_namespace}" xmlns:x="urn:producer"><x:opaque>{field}</x:opaque><q:p data="first"><x:innerOpaque>{field}</x:innerOpaque>{field}</q:p><q:p data="second">{field}</q:p></q:hdr>"#,
    );
    package.set_part("/word/header-lookalike.xml", header.as_bytes().to_vec());
    let header_id = package.get_or_create_part_rels("/word/document.xml").add(
        oxml_opc::relationship::rel_types::HEADER,
        "header-lookalike.xml",
    );
    package.set_part(
        "/word/document.xml",
        format!(
            r#"<?xml version="1.0"?><w:document xmlns:w="{word_namespace}" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body><w:sectPr><w:headerReference w:type="default" r:id="{header_id}"/></w:sectPr></w:body></w:document>"#,
        )
        .into_bytes(),
    );
    let mut bytes = std::io::Cursor::new(Vec::new());
    package.write_to(&mut bytes).unwrap();
    let mut document = Document::from_bytes(bytes.get_ref()).unwrap();
    let context = FieldEvaluationContext {
        merge_fields: BTreeMap::from([("Same".to_owned(), "updated same".to_owned())]),
        ..FieldEvaluationContext::default()
    };

    assert_eq!(document.update_fields(&context).unwrap(), 2);
    let saved = document.to_bytes().unwrap();
    let package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(saved)).unwrap();
    let header =
        std::str::from_utf8(package.get_part("/word/header-lookalike.xml").unwrap()).unwrap();
    assert!(
        header.contains(&format!(r#"<x:opaque>{field}</x:opaque>"#)),
        "{header}"
    );
    assert!(
        header.contains(&format!(r#"<x:innerOpaque>{field}</x:innerOpaque>"#)),
        "{header}"
    );
    assert_eq!(header.matches("updated same").count(), 2, "{header}");
    assert_eq!(header.matches("stored same").count(), 2, "{header}");
    assert!(
        header.find(r#"<q:p data="first""#).unwrap()
            < header.find(r#"<q:p data="second""#).unwrap(),
        "{header}"
    );
}

#[test]
fn styleref_searches_the_approved_direction_and_scope() {
    let body = r#"
        <w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>First heading</w:t></w:r></w:p>
        <w:p><w:fldSimple w:instr="STYLEREF &quot;Heading 1&quot;"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="STYLEREF &quot;Heading 1&quot; \p"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Last heading</w:t></w:r></w:p>
        <w:p><w:fldSimple w:instr="STYLEREF &quot;Heading 1&quot; \l"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
    "#;
    let document = document_with_field_parts(&wrap_word_body(body), None, None);
    let outcomes = document
        .evaluate_fields(&FieldEvaluationContext::default())
        .unwrap()
        .into_iter()
        .map(|result| result.outcome)
        .collect::<Vec<_>>();
    assert_eq!(
        outcomes,
        [
            FieldOutcome::Resolved("First heading".to_owned()),
            FieldOutcome::Resolved("First heading above".to_owned()),
            FieldOutcome::Resolved("Last heading".to_owned()),
        ]
    );
}

#[test]
fn date_time_filename_mergefield_and_includetext_use_only_explicit_context() {
    let body = r#"
        <w:p><w:fldSimple w:instr="DATE \@ &quot;yyyy-MM-dd&quot;"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="TIME \@ &quot;HH:mm:ss&quot;"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="FILENAME \p"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="MERGEFIELD Name \b &quot;Dear &quot; \f &quot;!&quot; \m \v"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="INCLUDETEXT &quot;chapter.docx&quot;"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="INCLUDETEXT &quot;chapter.docx&quot; &quot;summary&quot; \!"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
    "#;
    let document = document_with_field_parts(&wrap_word_body(body), None, None);
    let context = FieldEvaluationContext {
        now: Some(FieldDateTime {
            year: 2025,
            month: 12,
            day: 14,
            hour: 21,
            minute: 7,
            second: 5,
        }),
        file_name: Some("report.docx".to_owned()),
        file_path: Some("/safe/input/report.docx".to_owned()),
        merge_fields: BTreeMap::from([("Name".to_owned(), "Ada".to_owned())]),
        included_text: BTreeMap::from([
            ("chapter.docx".to_owned(), "Whole chapter".to_owned()),
            (
                "chapter.docx#summary".to_owned(),
                "Chapter summary".to_owned(),
            ),
        ]),
    };
    assert_eq!(
        document
            .evaluate_fields(&context)
            .unwrap()
            .into_iter()
            .map(|result| result.outcome)
            .collect::<Vec<_>>(),
        [
            FieldOutcome::Resolved("2025-12-14".to_owned()),
            FieldOutcome::Resolved("21:07:05".to_owned()),
            FieldOutcome::Resolved("/safe/input/report.docx".to_owned()),
            FieldOutcome::Resolved("Dear Ada!".to_owned()),
            FieldOutcome::Resolved("Whole chapter".to_owned()),
            FieldOutcome::Resolved("Chapter summary".to_owned()),
        ]
    );
}

#[test]
fn both_revision_views_render_and_accepted_matches_resolved_document() {
    let xml = wrap_word_body(
        r#"<w:p><w:r><w:t>ordinary </w:t></w:r><w:ins w:id="1" w:author="Ada"><w:r><w:t>inserted </w:t></w:r></w:ins><w:del w:id="2" w:author="Ben"><w:r><w:delText>deleted </w:delText></w:r></w:del><w:moveFrom w:id="3" w:author="Cy"><w:r><w:t>old </w:t></w:r></w:moveFrom><w:moveTo w:id="4" w:author="Dee"><w:r><w:t>moved</w:t></w:r></w:moveTo></w:p>"#,
    );
    let document = document_with_content_controls(&xml);
    let accepted = document
        .render_page_to_png_deterministic_with_options(
            0,
            150.0,
            RenderOptions {
                revision_view: RevisionView::Accepted,
            },
        )
        .unwrap()
        .expect("accepted page");
    let tracked = document
        .render_page_to_png_deterministic_with_options(
            0,
            150.0,
            RenderOptions {
                revision_view: RevisionView::Tracked,
            },
        )
        .unwrap()
        .expect("tracked page");
    assert_ne!(accepted, tracked);

    let mut resolved = document_with_content_controls(&xml);
    assert_eq!(resolved.accept_all().unwrap(), 4);
    let resolved = resolved
        .render_page_to_png_deterministic(0, 150.0)
        .unwrap()
        .expect("resolved page");
    assert_eq!(accepted, resolved);
}

#[test]
fn default_render_methods_keep_the_accepted_view() {
    let xml = wrap_word_body(
        r#"<w:p><w:r><w:t>ordinary </w:t></w:r><w:ins w:id="1" w:author="Ada"><w:r><w:t>accepted</w:t></w:r></w:ins><w:del w:id="2" w:author="Ben"><w:r><w:delText>omitted</w:delText></w:r></w:del></w:p>"#,
    );
    let document = document_with_content_controls(&xml);
    let options = RenderOptions::default();

    assert_eq!(
        document.to_pdf().unwrap(),
        document.to_pdf_with_options(options).unwrap()
    );
    assert_eq!(
        document.to_pdf_deterministic().unwrap(),
        document.to_pdf_deterministic_with_options(options).unwrap()
    );
    assert_eq!(
        document.render_page_to_png(0, 96.0).unwrap(),
        document
            .render_page_to_png_with_options(0, 96.0, options)
            .unwrap()
    );
    assert_eq!(
        document.render_page_to_png_deterministic(0, 96.0).unwrap(),
        document
            .render_page_to_png_deterministic_with_options(0, 96.0, options)
            .unwrap()
    );
    assert_eq!(
        document.render_all_pages(96.0).unwrap(),
        document
            .render_all_pages_with_options(96.0, options)
            .unwrap()
    );
    assert_eq!(
        format!("{:?}", document.layout_page(0).unwrap()),
        format!(
            "{:?}",
            document.layout_page_with_options(0, options).unwrap()
        )
    );
}

#[test]
fn downstream_renderers_can_traverse_the_complete_public_layout_result() {
    let mut document = Document::new();
    document.add_paragraph("public layout integration");

    let result = document.layout().expect("public layout should succeed");
    let mut saw_resolvable_run = false;
    for page in &result.layout.pages {
        oxml_layout::walk(&page.elements, &mut |element, _| {
            if let oxml_layout::PositionedElement::Text(run) = element {
                let font = result
                    .layout
                    .fonts
                    .iter()
                    .find(|font| font.id == run.font_id)
                    .expect("glyph run font should resolve");
                assert!(!font.data.is_empty());
                if let Some(source) = run.source {
                    assert!(result.source_node(source.node).is_some());
                }
                saw_resolvable_run = true;
            }
        });
    }
    assert!(saw_resolvable_run);
}

#[test]
fn caller_font_layout_options_select_the_tracked_revision_projection() {
    let xml = wrap_word_body(
        r#"<w:p><w:r><w:t>ordinary </w:t></w:r><w:ins w:id="1" w:author="Ada"><w:r><w:t>accepted</w:t></w:r></w:ins><w:del w:id="2" w:author="Ben"><w:r><w:delText> omitted</w:delText></w:r></w:del></w:p>"#,
    );
    let document = document_with_content_controls(&xml);
    let (family, bytes) = oxml_layout::bundled_fonts::bundled_font_data()[0];
    let accepted = document
        .layout_with_fonts(&[(family, bytes)])
        .expect("accepted caller-font layout should succeed");
    let tracked = document
        .layout_with_fonts_and_options(
            &[(family, bytes)],
            RenderOptions {
                revision_view: RevisionView::Tracked,
            },
        )
        .expect("tracked caller-font layout should succeed");

    let visible_text = |result: &rdocx_layout::WordLayoutResult| {
        let mut text = String::new();
        for page in &result.layout.pages {
            oxml_layout::walk(&page.elements, &mut |element, _| {
                if let oxml_layout::PositionedElement::Text(run) = element {
                    text.push_str(&run.text);
                }
            });
        }
        text
    };

    let accepted_text = visible_text(&accepted);
    let tracked_text = visible_text(&tracked);
    assert_eq!(accepted.revision_view, RevisionView::Accepted);
    assert_eq!(tracked.revision_view, RevisionView::Tracked);
    assert!(accepted_text.contains("ordinary accepted"));
    assert!(!accepted_text.contains("omitted"));
    assert!(tracked_text.contains("ordinary accepted omitted"));
}

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
fn contextual_cleanup_preserves_foreign_property_shells() {
    let xml = wrap_word_body(
        r#"<w:p><w:r><x:rPr xmlns:x="urn:foreign-property"><w:del w:id="91" w:author="lookalike"/></x:rPr><w:t>kept</w:t></w:r><w:ins w:id="93" w:author="Ada"><w:r><w:t> accepted</w:t></w:r></w:ins></w:p>"#,
    );
    let mut document = document_with_content_controls(&xml);

    assert_eq!(document.accept_all().unwrap(), 1);
    let saved = document_xml(&mut document);
    assert!(saved.contains(r#"<x:rPr xmlns:x="urn:foreign-property">"#));
    assert!(saved.contains(r#"w:id="91" w:author="lookalike""#));
    assert!(saved.contains("<w:t>kept</w:t>"));
    assert!(saved.contains("<w:t> accepted</w:t>"));
}

#[test]
fn table_cleanup_retains_rows_owned_by_content_controls() {
    let xml = wrap_word_body(
        r#"<w:tbl><w:tblPr/><w:tblGrid/><w:sdt><w:sdtPr><w:tag w:val="retained-row"/></w:sdtPr><w:sdtContent><w:tr><w:tc><w:p><w:r><w:t>retained</w:t></w:r></w:p></w:tc></w:tr></w:sdtContent></w:sdt><w:tr><w:trPr><w:del w:id="92" w:author="Ada"/></w:trPr><w:tc><w:p><w:r><w:t>deleted</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
    );
    let mut document = document_with_content_controls(&xml);

    assert_eq!(document.accept_all().unwrap(), 1);
    let saved = document_xml(&mut document);
    assert!(saved.contains(r#"w:val="retained-row""#), "{saved}");
    assert!(saved.contains("<w:t>retained</w:t>"), "{saved}");
    assert!(!saved.contains("<w:t>deleted</w:t>"), "{saved}");
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
fn duplicate_property_revisions_round_trip_and_resolve_one_identity_at_a_time() {
    let xml = wrap_word_body(
        r#"<w:p><w:pPr><w:numPr><w:ins w:id="501" w:author="Ada"/><x:ins xmlns:x="urn:producer" x:mark="num"/><w:ins w:id="502" w:author="Ada"/></w:numPr><w:pPrChange w:id="101" w:author="Ada"><w:pPr><w:jc w:val="left"/></w:pPr></w:pPrChange><x:pPrChange xmlns:x="urn:producer" x:mark="paragraph"/><w:pPrChange w:id="102" w:author="Ada"><w:pPr><w:jc w:val="right"/></w:pPr></w:pPrChange></w:pPr><w:r><w:rPr><w:rPrChange w:id="201" w:author="Ada"><w:rPr><w:b/></w:rPr></w:rPrChange><x:rPrChange xmlns:x="urn:producer" x:mark="run"/><w:rPrChange w:id="202" w:author="Ada"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:t>x</w:t></w:r></w:p><w:tbl><w:tblPr><w:tblPrChange w:id="301" w:author="Ada"><w:tblPr><w:jc w:val="left"/></w:tblPr></w:tblPrChange><x:tblPrChange xmlns:x="urn:producer" x:mark="table"/><w:tblPrChange w:id="302" w:author="Ada"><w:tblPr><w:jc w:val="right"/></w:tblPr></w:tblPrChange></w:tblPr><w:tblGrid/><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl><w:sectPr><w:sectPrChange w:id="401" w:author="Ada"><w:sectPr><w:titlePg/></w:sectPr></w:sectPrChange><x:sectPrChange xmlns:x="urn:producer" x:mark="section"/><w:sectPrChange w:id="402" w:author="Ada"><w:sectPr><w:pgSz w:w="12240"/></w:sectPr></w:sectPrChange></w:sectPr>"#,
    );
    let mut document = document_with_content_controls(&xml);
    let sorted_ids = |document: &Document| {
        let mut ids = document
            .revisions()
            .iter()
            .map(|revision| revision.id())
            .collect::<Vec<_>>();
        ids.sort_unstable();
        ids
    };
    assert_eq!(sorted_ids(&document), [102, 202, 302, 402, 502]);

    let initial = document_xml(&mut document);
    for id in [101, 102, 201, 202, 301, 302, 401, 402, 501, 502] {
        assert_eq!(initial.matches(&format!(r#"w:id="{id}""#)).count(), 1);
    }
    for mark in ["num", "paragraph", "run", "table", "section"] {
        assert_eq!(initial.matches(&format!(r#"x:mark="{mark}""#)).count(), 1);
    }

    for id in [102, 202, 302, 402, 502] {
        assert_eq!(document.accept_revision_id(id).unwrap(), 1);
    }
    assert_eq!(sorted_ids(&document), [101, 201, 301, 401, 501]);
    let staged = document_xml(&mut document);
    for id in [101, 201, 301, 401, 501] {
        assert_eq!(staged.matches(&format!(r#"w:id="{id}""#)).count(), 1);
    }
    for id in [102, 202, 302, 402, 502] {
        assert!(!staged.contains(&format!(r#"w:id="{id}""#)));
    }
    for mark in ["num", "paragraph", "run", "table", "section"] {
        assert_eq!(staged.matches(&format!(r#"x:mark="{mark}""#)).count(), 1);
    }

    let mut reopened = Document::from_bytes(&document.to_bytes().unwrap()).unwrap();
    assert_eq!(sorted_ids(&reopened), [101, 201, 301, 401, 501]);
    for id in [101, 201, 301, 401, 501] {
        assert_eq!(reopened.accept_revision_id(id).unwrap(), 1);
    }
    assert!(reopened.revisions().is_empty());

    let mut rejected = document_with_content_controls(&xml);
    for id in [102, 202, 302, 402, 502] {
        assert_eq!(rejected.reject_revision_id(id).unwrap(), 1, "id {id}");
    }
    assert_eq!(sorted_ids(&rejected), [101, 201, 301, 401, 501]);
    let rejected_xml = document_xml(&mut rejected);
    for id in [101, 201, 301, 401, 501] {
        assert_eq!(rejected_xml.matches(&format!(r#"w:id="{id}""#)).count(), 1);
    }
    for mark in ["num", "paragraph", "run", "table", "section"] {
        assert_eq!(
            rejected_xml.matches(&format!(r#"x:mark="{mark}""#)).count(),
            1
        );
    }

    let mut rejected = Document::from_bytes(&rejected.to_bytes().unwrap()).unwrap();
    assert_eq!(sorted_ids(&rejected), [101, 201, 301, 401, 501]);
    for id in [101, 201, 301, 401, 501] {
        assert_eq!(rejected.reject_revision_id(id).unwrap(), 1);
    }
    assert!(rejected.revisions().is_empty());
    let rejected_xml = document_xml(&mut rejected);
    for mark in ["num", "paragraph", "run", "table", "section"] {
        assert_eq!(
            rejected_xml.matches(&format!(r#"x:mark="{mark}""#)).count(),
            1
        );
    }

    let mut rejected_all = document_with_content_controls(&xml);
    assert_eq!(rejected_all.reject_all().unwrap(), 10);
    assert!(rejected_all.revisions().is_empty());
    let rejected_all_xml = document_xml(&mut rejected_all);
    for mark in ["num", "paragraph", "run", "table", "section"] {
        assert_eq!(
            rejected_all_xml
                .matches(&format!(r#"x:mark="{mark}""#))
                .count(),
            1
        );
    }
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

    let raw_property_body =
        r#"<w:p><w:r><w:rPr><w:rStyle><w:opaque/></w:rStyle></w:rPr><w:t>A</w:t></w:r></w:p>"#;
    let mut formatted = document_with_content_controls(&wrap_word_body(raw_property_body));
    formatted
        .paragraph_mut(0)
        .unwrap()
        .run_mut(0)
        .unwrap()
        .set_bold(true);
    let formatted_xml = document_xml(&mut formatted);
    let formatted_positions = [
        formatted_xml.find("<w:rPr>").unwrap(),
        formatted_xml.find("<w:rStyle>").unwrap(),
        formatted_xml.find("<w:b/>").unwrap(),
        formatted_xml.find("<w:t>A</w:t>").unwrap(),
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
fn content_control_display_insertion_keeps_trailing_raw_after_the_value() {
    let xml = wrap_word_body(
        r#"<w:sdt><w:sdtPr><w:tag w:val="customer"/><w:text/></w:sdtPr><w:sdtContent><w:p><w:r><w:before/><w:tab/><w:after/></w:r></w:p></w:sdtContent></w:sdt>"#,
    );
    let mut document = document_with_content_controls(&xml);

    assert_eq!(
        document
            .set_content_control_value_by_tag("customer", "Ada")
            .unwrap(),
        1
    );
    let output = document_xml(&mut document);
    let positions = [
        output.find("<w:before/>").unwrap(),
        output.find("<w:t>Ada</w:t>").unwrap(),
        output.find("<w:after/>").unwrap(),
    ];
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));

    let mut reopened = Document::from_bytes(&document.to_bytes().unwrap()).unwrap();
    let reopened_xml = document_xml(&mut reopened);
    assert_eq!(reopened_xml.matches("<w:before/>").count(), 1);
    assert_eq!(reopened_xml.matches("<w:after/>").count(), 1);
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
    let text = compatibility_page_elements(&page.elements)
        .into_iter()
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

#[test]
fn a_nested_loop_and_conditional_generate_the_expected_document() {
    let body = r#"
        <w:p><w:r><w:t>{% for group in groups %}</w:t></w:r></w:p>
        <w:p><w:r><w:t>Group {{ group.name }}</w:t></w:r></w:p>
        <w:p><w:r><w:t>{% if group.visible %}</w:t></w:r></w:p>
        <w:p><w:r><w:t>Visible {{ group.name }}</w:t></w:r></w:p>
        <w:p><w:r><w:t>{% endif %}</w:t></w:r></w:p>
        <w:tbl>
          <w:tblPr/><w:tblGrid><w:gridCol w:w="2400"/></w:tblGrid>
          <w:tr><w:tc><w:p><w:r><w:t>{% for item in group.items %}</w:t></w:r></w:p></w:tc></w:tr>
          <w:tr><w:tc><w:p><w:r><w:t>{{ group.name }}:{{ item.label }}</w:t></w:r></w:p></w:tc></w:tr>
          <w:tr><w:tc><w:p><w:r><w:t>{% endfor %}</w:t></w:r></w:p></w:tc></w:tr>
        </w:tbl>
        <w:p><w:r><w:t>{% endfor %}</w:t></w:r></w:p>
        <w:p><w:r><w:t>Root {{ title }}</w:t></w:r></w:p>
        <w:sectPr/>
    "#;
    let mut document = document_with_content_controls(&wrap_word_body(body));
    let data = serde_json::json!({
        "title": "Summary",
        "groups": [
            {
                "name": "Alpha",
                "visible": true,
                "items": [{"label": "one"}, {"label": "two"}]
            },
            {
                "name": "Beta",
                "visible": false,
                "items": [{"label": "three"}]
            }
        ]
    });

    assert_eq!(document.render_template(&data).unwrap(), 10);
    let body = body_from_document(&mut document);
    let paragraphs = body
        .content
        .iter()
        .filter_map(|content| match content {
            BodyContent::Paragraph(paragraph) => Some(paragraph.text()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        paragraphs,
        ["Group Alpha", "Visible Alpha", "Group Beta", "Root Summary"]
    );
    let row_text = body
        .tables()
        .flat_map(|table| &table.rows)
        .map(|row| {
            row.cells
                .iter()
                .flat_map(|cell| &cell.content)
                .filter_map(|content| match content {
                    rdocx_oxml::table::CellContent::Paragraph(paragraph) => Some(paragraph.text()),
                    _ => None,
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>();
    assert_eq!(row_text, ["Alpha:one", "Alpha:two", "Beta:three"]);
}

#[test]
fn three_template_rows_over_ten_records_produce_thirty_preserved_rows() {
    let body = r#"
        <w:tbl>
          <w:tblPr>
            <w:tblStyle w:val="BandedRows"/>
            <w:tblLook w:val="04A0" w:firstRow="1" w:noHBand="0"/>
          </w:tblPr>
          <w:tblGrid>
            <w:gridCol w:w="1800"/><w:gridCol w:w="1800"/><w:gridCol w:w="1800"/>
          </w:tblGrid>
          <w:tr><w:tc><w:p><w:r><w:t>{% for record in records %}</w:t></w:r></w:p></w:tc></w:tr>
          <w:tr>
            <w:trPr><w:cnfStyle w:val="100000000000"/></w:trPr>
            <w:tc><w:tcPr><w:gridSpan w:val="2"/><w:vMerge w:val="restart"/></w:tcPr><w:p><w:r><w:t>A {{ record.id }}</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>top</w:t></w:r></w:p></w:tc>
          </w:tr>
          <w:tr>
            <w:tc><w:tcPr><w:gridSpan w:val="2"/><w:vMerge/></w:tcPr><w:p><w:r><w:t>B {{ record.id }}</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>middle</w:t></w:r></w:p></w:tc>
          </w:tr>
          <w:tr>
            <w:tc><w:tcPr><w:gridSpan w:val="2"/><w:vMerge/></w:tcPr><w:p><w:r><w:t>C {{ record.id }}</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>bottom</w:t></w:r></w:p></w:tc>
          </w:tr>
          <w:tr><w:tc><w:p><w:r><w:t>{% endfor %}</w:t></w:r></w:p></w:tc></w:tr>
        </w:tbl>
        <w:sectPr/>
    "#;
    let mut document = document_with_content_controls(&wrap_word_body(body));
    let records = (0..10)
        .map(|id| serde_json::json!({"id": id}))
        .collect::<Vec<_>>();

    assert_eq!(
        document
            .render_template(&serde_json::json!({"records": records}))
            .unwrap(),
        30
    );
    let body = body_from_document(&mut document);
    let table = body.tables().next().unwrap();
    assert_eq!(table.rows.len(), 30);
    assert_eq!(table.grid.as_ref().unwrap().columns.len(), 3);
    let properties = table.properties.as_ref().unwrap();
    assert_eq!(properties.style_id.as_deref(), Some("BandedRows"));
    let look = properties.look.as_ref().unwrap();
    assert_eq!(look.first_row, Some(true));
    assert_eq!(look.no_h_band, Some(false));

    for (index, row) in table.rows.iter().enumerate() {
        let cell_properties = row.cells[0].properties.as_ref().unwrap();
        assert_eq!(cell_properties.grid_span, Some(2));
        assert_eq!(
            cell_properties.v_merge,
            Some(if index % 3 == 0 {
                rdocx_oxml::table::VMerge::Restart
            } else {
                rdocx_oxml::table::VMerge::Continue
            })
        );
        let prefix = ["A", "B", "C"][index % 3];
        assert_eq!(row.cells[0].text(), format!("{prefix} {}", index / 3));
    }

    let invalid_list = r#"
        <w:p><w:r><w:t>{% for item in items %}</w:t></w:r></w:p>
        <w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="99"/></w:numPr></w:pPr><w:r><w:t>{{ item }}</w:t></w:r></w:p>
        <w:p><w:r><w:t>{% endfor %}</w:t></w:r></w:p>
        <w:sectPr/>
    "#;
    let mut invalid = document_with_content_controls(&wrap_word_body(invalid_list));
    let before = invalid.to_bytes().unwrap();
    assert!(
        invalid
            .render_template(&serde_json::json!({"items": ["value"]}))
            .is_err()
    );
    assert_eq!(invalid.to_bytes().unwrap(), before);
}

#[test]
fn repeated_numbered_items_keep_one_continuous_sequence() {
    let mut document = Document::new();
    document.add_paragraph("{% for item in items %}");
    document.add_numbered_list_item("Item {{ item.name }}", 2);
    document.add_paragraph("Note {{ item.name }}");
    document.add_paragraph("{% endfor %}");
    let before = document.to_bytes().unwrap();
    let before_package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(before)).unwrap();
    let numbering_before = before_package
        .get_part("/word/numbering.xml")
        .unwrap()
        .to_vec();

    assert_eq!(
        document
            .render_template(&serde_json::json!({
                "items": [{"name": "one"}, {"name": "two"}, {"name": "three"}]
            }))
            .unwrap(),
        6
    );
    let saved = document.to_bytes().unwrap();
    let saved_package = oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(&saved)).unwrap();
    assert_eq!(
        saved_package.get_part("/word/numbering.xml").unwrap(),
        numbering_before
    );
    let reopened = Document::from_bytes(&saved).unwrap();
    let paragraphs = reopened.paragraphs();
    assert_eq!(
        paragraphs
            .iter()
            .map(|paragraph| paragraph.text())
            .collect::<Vec<_>>(),
        [
            "Item one",
            "Note one",
            "Item two",
            "Note two",
            "Item three",
            "Note three"
        ]
    );
    let numbering = paragraphs
        .iter()
        .step_by(2)
        .map(|paragraph| paragraph.numbering().unwrap())
        .collect::<Vec<_>>();
    assert!(numbering.iter().all(|value| *value == numbering[0]));
    assert_eq!(numbering[0].1, 2);

    let invalid_body = r#"
        <w:p><w:r><w:t>{% for item in items %}</w:t></w:r></w:p>
        <w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="99"/></w:numPr></w:pPr><w:r><w:t>{{ item }}</w:t></w:r></w:p>
        <w:p><w:r><w:t>{% endfor %}</w:t></w:r></w:p>
        <w:sectPr/>
    "#;
    let mut invalid = document_with_content_controls(&wrap_word_body(invalid_body));
    let before = invalid.to_bytes().unwrap();
    assert!(
        invalid
            .render_template(&serde_json::json!({"items": ["value"]}))
            .is_err()
    );
    assert_eq!(invalid.to_bytes().unwrap(), before);
}

fn mail_merge_record(values: &[(&str, &str)]) -> BTreeMap<String, String> {
    values
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect()
}

fn document_with_mail_merge_header() -> Document {
    let mut seed = Document::new();
    let mut package =
        oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(seed.to_bytes().unwrap())).unwrap();
    let header = r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:tbl xmlns:q="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><q:tblPr/><q:tblGrid/><q:tr><q:tc><q:p><q:fldSimple q:instr="MERGEFIELD &quot;Full Name&quot;"><q:r><q:t>stored table header</q:t></q:r></q:fldSimple></q:p></q:tc></q:tr></w:tbl><w:sdt><w:sdtPr><w:tag w:val="merge"/></w:sdtPr><w:sdtContent><w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText> MERGEFIELD Name </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>stored control header</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p></w:sdtContent></w:sdt></w:hdr>"#;
    package.set_part("/word/header1.xml", header.as_bytes().to_vec());
    package.content_types.add_override(
        "/word/header1.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
    );
    let header_id = package
        .get_or_create_part_rels("/word/document.xml")
        .add(oxml_opc::relationship::rel_types::HEADER, "header1.xml");
    package.set_part(
        "/word/document.xml",
        format!(
            r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body><w:p><w:r><w:t>body</w:t></w:r></w:p><w:sectPr><w:headerReference w:type="default" r:id="{header_id}"/></w:sectPr></w:body></w:document>"#
        )
        .into_bytes(),
    );
    let mut bytes = std::io::Cursor::new(Vec::new());
    package.write_to(&mut bytes).unwrap();
    Document::from_bytes(bytes.get_ref()).unwrap()
}

#[test]
fn a_fixture_record_set_produces_separate_and_sectioned_documents() {
    let body = r#"
        <w:p><w:fldSimple w:instr="MERGEFIELD Name \* Upper"><w:r><w:t>stored name</w:t></w:r></w:fldSimple></w:p>
        <w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="2400"/></w:tblGrid><w:tr><w:tc><w:p><w:fldSimple w:instr="MERGEFIELD City"><w:r><w:t>stored city</w:t></w:r></w:fldSimple></w:p></w:tc></w:tr></w:tbl>
        <w:sdt><w:sdtPr><w:tag w:val="role"/></w:sdtPr><w:sdtContent><w:p><w:fldSimple w:instr="MERGEFIELD Role"><w:r><w:t>stored role</w:t></w:r></w:fldSimple></w:p></w:sdtContent></w:sdt>
        <w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>
    "#;
    let document = document_with_content_controls(&wrap_word_body(body));
    let records = vec![
        mail_merge_record(&[("Name", "Ada"), ("City", "London"), ("Role", "Engineer")]),
        mail_merge_record(&[("Name", "Grace"), ("Role", "Admiral")]),
    ];

    let mut separate = document.mail_merge(&records).unwrap();
    assert_eq!(separate.len(), 2);
    let first = document_xml(&mut separate[0]);
    let second = document_xml(&mut separate[1]);
    assert!(first.contains(">ADA<"), "{first}");
    assert!(first.contains(">London<"), "{first}");
    assert!(first.contains(">Engineer<"), "{first}");
    assert!(second.contains(">GRACE<"), "{second}");
    assert!(second.contains(">Admiral<"), "{second}");
    assert!(!second.contains("stored city"), "{second}");

    let mut sectioned = document.mail_merge_sections(&records).unwrap();
    let xml = document_xml(&mut sectioned);
    let ada = xml.find(">ADA<").unwrap();
    let grace = xml.find(">GRACE<").unwrap();
    assert!(ada < grace, "{xml}");
    assert!(!xml.contains("stored city"), "{xml}");
    assert_eq!(xml.matches(r#"<w:type w:val="nextPage"/>"#).count(), 1);
}

#[test]
fn mail_merge_preserves_switches_and_general_field_policy() {
    let body = r#"
        <w:p><w:fldSimple w:instr="MERGEFIELD Name \* Upper"><w:r><w:t>stored name</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="MERGEFIELD Missing"><w:r><w:t>stored missing</w:t></w:r></w:fldSimple></w:p>
        <w:sectPr/>
    "#;
    let mut document = document_with_field_parts(&wrap_word_body(body), None, None);
    let mut merged = document
        .mail_merge(&[mail_merge_record(&[("Name", "Ada")])])
        .unwrap()
        .pop()
        .unwrap();
    let merged_xml = document_xml(&mut merged);
    assert!(merged_xml.contains(r#"MERGEFIELD Name \* Upper"#));
    assert!(merged_xml.contains(">ADA<"), "{merged_xml}");
    assert!(!merged_xml.contains("stored missing"), "{merged_xml}");

    let outcomes = document
        .evaluate_fields(&FieldEvaluationContext::default())
        .unwrap();
    assert!(matches!(
        &outcomes[0].outcome,
        FieldOutcome::KeepStored { .. }
    ));
    assert!(matches!(
        &outcomes[1].outcome,
        FieldOutcome::KeepStored { .. }
    ));
    assert_eq!(
        document
            .update_fields(&FieldEvaluationContext::default())
            .unwrap(),
        2
    );
    let ordinary_xml = document_xml(&mut document);
    assert!(ordinary_xml.contains("stored name"), "{ordinary_xml}");
    assert!(ordinary_xml.contains("stored missing"), "{ordinary_xml}");
}

#[test]
fn sectioned_mail_merge_preserves_section_properties_and_unmodelled_xml() {
    const PRODUCER_BODY: &str = r#"<x:producer xmlns:x="urn:producer" mark="kept"/>"#;
    const PRODUCER_SECTION: &str = r#"<x:section xmlns:x="urn:producer" mark="kept"/>"#;
    let body = r#"
        <w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:fldSimple w:instr="MERGEFIELD Name"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="2400"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>fixed</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
        <x:producer xmlns:x="urn:producer" mark="kept"/>
        <w:sectPr><w:pgSz w:w="11906" w:h="16838"/><x:section xmlns:x="urn:producer" mark="kept"/></w:sectPr>
    "#;
    let document = document_with_field_parts(&wrap_word_body(body), None, None);
    let records = vec![
        mail_merge_record(&[("Name", "one")]),
        mail_merge_record(&[("Name", "two")]),
    ];
    let mut merged = document.mail_merge_sections(&records).unwrap();
    let bytes = merged.to_bytes().unwrap();
    let mut reopened = Document::from_bytes(&bytes).unwrap();
    let body = body_from_document(&mut reopened);

    assert_eq!(body.tables().count(), 2);
    assert_eq!(
        body.content
            .iter()
            .filter(|content| matches!(content, BodyContent::Paragraph(paragraph) if paragraph.properties.as_ref().and_then(|properties| properties.sect_pr.as_ref()).is_some()))
            .count(),
        1
    );
    let section_break = body.content.iter().find_map(|content| match content {
        BodyContent::Paragraph(paragraph) => paragraph
            .properties
            .as_ref()
            .and_then(|properties| properties.sect_pr.as_ref()),
        _ => None,
    });
    assert_eq!(
        section_break.unwrap().section_type,
        Some(rdocx_oxml::shared::ST_SectionType::NextPage)
    );
    assert_eq!(body.sect_pr.as_ref().unwrap().page_width.unwrap().0, 11906);

    let xml = document_xml(&mut reopened);
    assert_eq!(xml.matches(PRODUCER_BODY).count(), 2, "{xml}");
    assert_eq!(xml.matches(PRODUCER_SECTION).count(), 2, "{xml}");
    assert!(xml.rfind("<w:sectPr>").unwrap() < xml.rfind("</w:body>").unwrap());
}

#[test]
fn sectioned_mail_merge_remaps_record_local_body_identities() {
    let body = r#"
        <w:p><w:bookmarkStart w:id="7" w:name="Target"/><w:fldSimple w:instr="MERGEFIELD Name"><w:r><w:t>stored target</w:t></w:r></w:fldSimple><w:bookmarkEnd w:id="7"/></w:p>
        <w:p><w:fldSimple w:instr="REF Target"><w:r><w:t>stored reference</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:fldSimple w:instr="REF MailMerge1"><w:r><w:t>intentionally unresolved</w:t></w:r></w:fldSimple></w:p>
        <w:p><w:hyperlink w:anchor="MailMerge2"><w:r><w:t>unresolved anchor</w:t></w:r></w:hyperlink></w:p>
        <w:sdt><w:sdtPr><w:id w:val="9"/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>control</w:t></w:r></w:p></w:sdtContent></w:sdt>
        <w:p><w:r><w:drawing><wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><wp:extent cx="1" cy="1"/><wp:docPr id="11" name="Picture"/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic><pic:nvPicPr><pic:cNvPr id="12" name="Picture"/><pic:cNvPicPr/></pic:nvPicPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>
        <w:sectPr/>
    "#;
    let document = document_with_field_parts(&wrap_word_body(body), None, None);
    let records = [
        mail_merge_record(&[("Name", "one")]),
        mail_merge_record(&[("Name", "two")]),
    ];
    let mut merged = document.mail_merge_sections(&records).unwrap();

    let bookmarks = merged.bookmarks();
    assert_eq!(bookmarks.len(), 2);
    assert!(bookmarks.iter().all(|bookmark| bookmark.issue().is_none()));
    assert_ne!(bookmarks[0].id(), bookmarks[1].id());
    assert_ne!(bookmarks[0].name(), bookmarks[1].name());

    let evaluations = merged
        .evaluate_fields(&FieldEvaluationContext::default())
        .unwrap();
    assert!(evaluations.iter().any(|evaluation| {
        evaluation.instruction == "REF MailMerge1"
            && matches!(evaluation.outcome, FieldOutcome::KeepStored { .. })
    }));
    assert!(
        evaluations
            .iter()
            .filter(|evaluation| {
                matches!(
                    evaluation.instruction.as_str(),
                    "REF Target" | "REF MailMerge3"
                )
            })
            .all(|evaluation| matches!(evaluation.outcome, FieldOutcome::Resolved(_)))
    );
    let xml = document_xml(&mut merged);
    assert!(xml.contains(r#"w:instr="REF Target""#), "{xml}");
    assert!(xml.contains(r#"w:instr="REF MailMerge3""#), "{xml}");
    assert_eq!(
        xml.matches(r#"w:instr="REF MailMerge1""#).count(),
        2,
        "{xml}"
    );
    assert_eq!(xml.matches(r#"w:anchor="MailMerge2""#).count(), 2, "{xml}");
    assert!(!xml.contains(r#"w:name="MailMerge1""#), "{xml}");
    assert!(!xml.contains(r#"w:name="MailMerge2""#), "{xml}");
    assert_eq!(xml.matches(r#"w:val="9""#).count(), 1, "{xml}");
    assert_eq!(xml.matches(r#"wp:docPr id="11""#).count(), 1, "{xml}");
    assert_eq!(xml.matches(r#"pic:cNvPr id="12""#).count(), 1, "{xml}");
}

#[test]
fn sectioned_mail_merge_remaps_references_inside_preserved_body_xml() {
    let body = r#"
        <w:customXml>
          <w:p><w:bookmarkStart w:id="21" w:name="RawTarget"/><w:r><w:t>raw target</w:t></w:r><w:bookmarkEnd w:id="21"/></w:p>
          <w:p><w:fldSimple w:instr="REF RawTarget"><w:r><w:t>stored ref</w:t></w:r></w:fldSimple></w:p>
          <w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText>PAGE</w:instrText><w:instrText>REF RawTarget</w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>1</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>
          <w:p><w:hyperlink w:anchor="RawTarget"><w:r><w:t>raw link</w:t></w:r></w:hyperlink></w:p>
          <w:p><w:fldSimple w:instr="REF MailMerge1"><w:r><w:t>unresolved</w:t></w:r></w:fldSimple></w:p>
        </w:customXml>
        <w:sectPr/>
    "#;
    let document = document_with_field_parts(&wrap_word_body(body), None, None);
    let records = [mail_merge_record(&[]), mail_merge_record(&[])];
    let mut merged = document.mail_merge_sections(&records).unwrap();
    let xml = document_xml(&mut merged);

    assert_eq!(xml.matches(r#"w:name="RawTarget""#).count(), 1, "{xml}");
    assert_eq!(xml.matches(r#"w:name="MailMerge2""#).count(), 1, "{xml}");
    assert!(xml.contains(r#"w:instr="REF MailMerge2""#), "{xml}");
    assert!(xml.contains("PAGEREF MailMerge2"), "{xml}");
    assert!(xml.contains(r#"w:anchor="MailMerge2""#), "{xml}");
    assert_eq!(
        xml.matches(r#"w:instr="REF MailMerge1""#).count(),
        2,
        "{xml}"
    );
    assert!(!xml.contains(r#"w:name="MailMerge1""#), "{xml}");
}

#[test]
fn sectioned_mail_merge_correlates_entity_escaped_bookmark_names() {
    let body = r#"
        <w:p><w:bookmarkStart w:id="7" w:name="A&amp;B"/><w:r><w:t>target</w:t></w:r><w:bookmarkEnd w:id="7"/></w:p>
        <w:p><w:fldSimple w:instr="REF A&amp;B"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p>
        <w:sectPr/>
    "#;
    let document = document_with_field_parts(&wrap_word_body(body), None, None);
    let mut merged = document
        .mail_merge_sections(&[mail_merge_record(&[]), mail_merge_record(&[])])
        .unwrap();
    let evaluations = merged
        .evaluate_fields(&FieldEvaluationContext::default())
        .unwrap();
    assert!(
        evaluations
            .iter()
            .filter(|evaluation| evaluation.instruction.starts_with("REF "))
            .all(|evaluation| matches!(evaluation.outcome, FieldOutcome::Resolved(_)))
    );
    let xml = document_xml(&mut merged);
    assert_eq!(xml.matches(r#"w:name="A&amp;B""#).count(), 1, "{xml}");
    assert_eq!(xml.matches(r#"w:name="MailMerge1""#).count(), 1, "{xml}");
    assert!(xml.contains(r#"w:instr="REF MailMerge1""#), "{xml}");
}

#[test]
fn sectioned_mail_merge_ignores_foreign_same_local_name_attributes() {
    let mut seed = Document::new();
    let mut package =
        oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(seed.to_bytes().unwrap())).unwrap();
    let header = r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:x="urn:producer"><w:p><w:fldSimple x:instr="MERGEFIELD Stable" w:instr="MERGEFIELD Vary"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p></w:hdr>"#;
    package.set_part("/word/header-foreign.xml", header.as_bytes().to_vec());
    package.content_types.add_override(
        "/word/header-foreign.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
    );
    let header_id = package.get_or_create_part_rels("/word/document.xml").add(
        oxml_opc::relationship::rel_types::HEADER,
        "header-foreign.xml",
    );
    package.set_part(
        "/word/document.xml",
        format!(
            r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body><w:customXml xmlns:x="urn:producer"><w:p><w:bookmarkStart x:id="701" w:id="7" x:name="Foreign" w:name="Target"/><w:r><w:t>target</w:t></w:r><w:bookmarkEnd x:id="702" w:id="7"/></w:p><w:p><w:fldSimple x:instr="REF Foreign" w:instr="REF Target"><w:r><w:t>stored ref</w:t></w:r></w:fldSimple></w:p><w:sdt><w:sdtPr><w:id x:val="900" w:val="9"/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>control</w:t></w:r></w:p></w:sdtContent></w:sdt><w:p><w:r><w:drawing><wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><wp:extent cx="1" cy="1"/><wp:docPr x:id="1100" id="11" name="Picture"/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic><pic:nvPicPr><pic:cNvPr x:id="1200" id="12" name="Picture"/><pic:cNvPicPr/></pic:nvPicPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p></w:customXml><w:sectPr><w:headerReference w:type="default" r:id="{header_id}"/></w:sectPr></w:body></w:document>"#
        )
        .into_bytes(),
    );
    let mut bytes = std::io::Cursor::new(Vec::new());
    package.write_to(&mut bytes).unwrap();
    let document = Document::from_bytes(bytes.get_ref()).unwrap();

    assert!(
        document
            .mail_merge_sections(&[
                mail_merge_record(&[("Stable", "same"), ("Vary", "one")]),
                mail_merge_record(&[("Stable", "same"), ("Vary", "two")]),
            ])
            .is_err()
    );

    let mut merged = document
        .mail_merge_sections(&[
            mail_merge_record(&[("Stable", "same"), ("Vary", "same")]),
            mail_merge_record(&[("Stable", "same"), ("Vary", "same")]),
        ])
        .unwrap();
    let xml = document_xml(&mut merged);
    assert_eq!(xml.matches(r#"x:id="701""#).count(), 2, "{xml}");
    assert_eq!(xml.matches(r#"x:id="702""#).count(), 2, "{xml}");
    assert_eq!(xml.matches(r#"x:name="Foreign""#).count(), 2, "{xml}");
    assert_eq!(xml.matches(r#"x:val="900""#).count(), 2, "{xml}");
    assert_eq!(xml.matches(r#"x:id="1100""#).count(), 2, "{xml}");
    assert_eq!(xml.matches(r#"x:id="1200""#).count(), 2, "{xml}");
    assert_eq!(xml.matches(r#"w:name="Target""#).count(), 1, "{xml}");
    assert!(xml.contains(r#"w:name="MailMerge1""#), "{xml}");
    assert_eq!(xml.matches(r#"w:instr="REF Target""#).count(), 1, "{xml}");
    assert!(xml.contains(r#"w:instr="REF MailMerge1""#), "{xml}");
    assert_eq!(xml.matches(r#"w:val="9""#).count(), 1, "{xml}");
    assert_eq!(
        xml.matches(r#"wp:docPr x:id="1100" id="11""#).count(),
        1,
        "{xml}"
    );
    assert_eq!(
        xml.matches(r#"pic:cNvPr x:id="1200" id="12""#).count(),
        1,
        "{xml}"
    );
}

#[test]
fn sectioned_mail_merge_scans_header_references_in_block_content_controls() {
    let mut seed = Document::new();
    let mut package =
        oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(seed.to_bytes().unwrap())).unwrap();
    let header = r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:fldSimple w:instr="MERGEFIELD Vary"><w:r><w:t>stored nested header</w:t></w:r></w:fldSimple></w:p></w:hdr>"#;
    package.set_part(
        "/word/sections/nested-header.xml",
        header.as_bytes().to_vec(),
    );
    package.content_types.add_override(
        "/word/sections/nested-header.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
    );
    let header_id = package.get_or_create_part_rels("/word/document.xml").add(
        oxml_opc::relationship::rel_types::HEADER,
        "sections/nested-header.xml",
    );
    package.set_part(
        "/word/document.xml",
        format!(
            r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body><w:sdt><w:sdtPr><w:tag w:val="nested-section"/></w:sdtPr><w:sdtContent><w:p><w:pPr><w:sectPr><w:headerReference w:type="default" r:id="{header_id}"/></w:sectPr></w:pPr><w:r><w:t>nested section</w:t></w:r></w:p></w:sdtContent></w:sdt><w:sectPr/></w:body></w:document>"#
        )
        .into_bytes(),
    );
    let mut bytes = std::io::Cursor::new(Vec::new());
    package.write_to(&mut bytes).unwrap();
    let mut document = Document::from_bytes(bytes.get_ref()).unwrap();

    assert!(
        document
            .evaluate_fields(&FieldEvaluationContext::default())
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        document
            .update_fields(&FieldEvaluationContext::default())
            .unwrap(),
        0
    );
    assert!(
        document
            .mail_merge_sections(&[
                mail_merge_record(&[("Vary", "one")]),
                mail_merge_record(&[("Vary", "two")]),
            ])
            .is_err()
    );
    assert!(
        document
            .mail_merge_sections(&[
                mail_merge_record(&[("Vary", "same")]),
                mail_merge_record(&[("Vary", "same")]),
            ])
            .is_ok()
    );
}

#[test]
fn mail_merge_uses_the_relationship_resolved_footnotes_part() {
    const RAW_TABLE: &str = r#"<w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:p><w:r><w:t>producer table</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
    let mut document = document_with_field_parts(
        &wrap_word_body(r#"<w:p><w:r><w:t>body</w:t></w:r></w:p><w:sectPr/>"#),
        None,
        None,
    );
    let mut package =
        oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(document.to_bytes().unwrap()))
            .unwrap();
    let footnotes = format!(
        r#"<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:footnote w:id="2"><w:p><w:fldSimple w:instr="MERGEFIELD Note"><w:r><w:t>stored note</w:t></w:r></w:fldSimple></w:p>{RAW_TABLE}</w:footnote></w:footnotes>"#
    );
    package.set_part("/word/notes/producer-footnotes.xml", footnotes.into_bytes());
    package.content_types.add_override(
        "/word/notes/producer-footnotes.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
    );
    package.get_or_create_part_rels("/word/document.xml").add(
        oxml_opc::relationship::rel_types::FOOTNOTES,
        "notes/producer-footnotes.xml",
    );
    let mut bytes = std::io::Cursor::new(Vec::new());
    package.write_to(&mut bytes).unwrap();
    let document = Document::from_bytes(bytes.get_ref()).unwrap();
    let records = [
        mail_merge_record(&[("Note", "one")]),
        mail_merge_record(&[("Note", "two")]),
    ];

    for (mut output, expected) in document
        .mail_merge(&records)
        .unwrap()
        .into_iter()
        .zip(["one", "two"])
    {
        let package =
            oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(output.to_bytes().unwrap()))
                .unwrap();
        let producer = std::str::from_utf8(
            package
                .get_part("/word/notes/producer-footnotes.xml")
                .unwrap(),
        )
        .unwrap();
        assert!(producer.contains(&format!(">{expected}<")), "{producer}");
        assert!(producer.contains(RAW_TABLE), "{producer}");
        assert!(package.get_part("/word/footnotes.xml").is_none());
    }
    assert!(document.mail_merge_sections(&records).is_err());
    let mut sectioned = document
        .mail_merge_sections(&[
            mail_merge_record(&[("Note", "same")]),
            mail_merge_record(&[("Note", "same")]),
        ])
        .unwrap();
    let package =
        oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(sectioned.to_bytes().unwrap()))
            .unwrap();
    let producer = std::str::from_utf8(
        package
            .get_part("/word/notes/producer-footnotes.xml")
            .unwrap(),
    )
    .unwrap();
    assert!(producer.contains(RAW_TABLE), "{producer}");
}

#[test]
fn a_failed_record_leaves_the_source_and_outputs_uncommitted() {
    let body = r#"<w:p><w:fldSimple w:instr="MERGEFIELD Name"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p><w:sectPr/>"#;
    let mut document = document_with_field_parts(&wrap_word_body(body), None, None);
    let before = document.to_bytes().unwrap();
    let records = vec![
        mail_merge_record(&[("Name", "valid")]),
        mail_merge_record(&[("Name", "invalid\u{000b}value")]),
    ];

    assert!(document.mail_merge(&records).is_err());
    assert!(document.mail_merge_sections(&records).is_err());
    assert_eq!(document.to_bytes().unwrap(), before);
}

#[test]
fn empty_and_single_record_merges_have_stable_boundaries() {
    let body = r#"<w:p><w:fldSimple w:instr="MERGEFIELD Name"><w:r><w:t>stored</w:t></w:r></w:fldSimple></w:p><w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>"#;
    let document = document_with_field_parts(&wrap_word_body(body), None, None);
    assert!(document.mail_merge(&[]).is_err());
    assert!(document.mail_merge_sections(&[]).is_err());

    let mut single = document
        .mail_merge_sections(&[mail_merge_record(&[("Name", "only")])])
        .unwrap();
    let xml = document_xml(&mut single);
    assert!(xml.contains(">only<"), "{xml}");
    assert!(!xml.contains(r#"<w:type w:val="nextPage"/>"#), "{xml}");
    let body = body_from_document(&mut single);
    assert!(body.sect_pr.is_some());
    assert!(body.content.iter().all(|content| {
        !matches!(content, BodyContent::Paragraph(paragraph) if paragraph.properties.as_ref().and_then(|properties| properties.sect_pr.as_ref()).is_some())
    }));

    let mut document = document_with_mail_merge_header();
    let package =
        oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(document.to_bytes().unwrap()))
            .unwrap();
    let header_before = package.get_part("/word/header1.xml").unwrap().to_vec();
    assert!(
        document
            .evaluate_fields(&FieldEvaluationContext::default())
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        document
            .update_fields(&FieldEvaluationContext::default())
            .unwrap(),
        0
    );
    let package =
        oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(document.to_bytes().unwrap()))
            .unwrap();
    assert_eq!(
        package.get_part("/word/header1.xml").unwrap(),
        header_before
    );
    let varying = vec![
        mail_merge_record(&[("Name", "same"), ("Full Name", "one")]),
        mail_merge_record(&[("Name", "same"), ("Full Name", "two")]),
    ];
    let mut separate = document.mail_merge(&varying).unwrap();
    for output in &mut separate {
        let package =
            oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(output.to_bytes().unwrap()))
                .unwrap();
        let header = std::str::from_utf8(package.get_part("/word/header1.xml").unwrap()).unwrap();
        assert!(header.contains("stored table header"), "{header}");
        assert!(header.contains("stored control header"), "{header}");
        assert!(!header.contains(">one<"), "{header}");
        assert!(!header.contains(">two<"), "{header}");
    }
    assert!(document.mail_merge_sections(&varying).is_err());
    assert!(
        document
            .mail_merge_sections(&[
                mail_merge_record(&[("Name", "one"), ("Full Name", "same")]),
                mail_merge_record(&[("Name", "two"), ("Full Name", "same")]),
            ])
            .is_err()
    );
    assert!(
        document
            .mail_merge_sections(&[
                mail_merge_record(&[("Name", "same"), ("Full Name", "same")]),
                mail_merge_record(&[("Name", "same"), ("Full Name", "same")]),
            ])
            .is_ok()
    );
}

#[test]
fn repeated_content_produces_a_deterministic_comparison() {
    let original_xml = wrap_word_body(
        r#"<w:p><w:r><w:t>repeat</w:t></w:r><w:r><w:t>repeat</w:t></w:r></w:p><w:p><w:r><w:t>repeat</w:t></w:r></w:p><w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:p><w:r><w:t>repeat row</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>repeat row</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
    );
    let edited_xml = wrap_word_body(
        r#"<w:p><w:r><w:t>repeat</w:t></w:r><w:r><w:t>changed</w:t></w:r></w:p><w:p><w:r><w:t>repeat</w:t></w:r></w:p><w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:p><w:r><w:t>repeat row</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>changed row</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
    );
    let edited = document_with_content_controls(&edited_xml);
    let mut first = document_with_content_controls(&original_xml);
    let mut second = document_with_content_controls(&original_xml);

    assert_eq!(
        first
            .compare(&edited, "Ada", "2026-08-21T09:30:00Z")
            .unwrap(),
        second
            .compare(&edited, "Ada", "2026-08-21T09:30:00Z")
            .unwrap()
    );
    assert_eq!(first.to_bytes().unwrap(), second.to_bytes().unwrap());
}

#[test]
fn comparison_metadata_is_escaped_and_ids_do_not_collide() {
    let original_xml = wrap_word_body(
        r#"<w:p><w:bookmarkStart w:id="0" w:name="kept"/><w:r><w:t>old</w:t></w:r><w:bookmarkEnd w:id="0"/></w:p>"#,
    );
    let edited_xml = wrap_word_body(
        r#"<w:p><w:bookmarkStart w:id="0" w:name="kept"/><w:r><w:t>new</w:t></w:r><w:bookmarkEnd w:id="0"/></w:p>"#,
    );
    let edited = document_with_content_controls(&edited_xml);
    let mut document = document_with_content_controls(&original_xml);
    document
        .compare(&edited, "Ada & \"Bob\"", "2026-08-21T09:30:00+01:00")
        .unwrap();

    let xml = document_xml(&mut document);
    assert!(xml.contains(r#"w:author="Ada &amp; &quot;Bob&quot;""#));
    assert!(
        document
            .revisions()
            .iter()
            .all(|revision| revision.id() != 0)
    );
    let before = document.to_bytes().unwrap();
    assert!(document.compare(&edited, "Ada", "not-a-date").is_err());
    assert_eq!(document.to_bytes().unwrap(), before);
}

#[test]
fn accepting_a_comparison_reproduces_the_edited_body_exactly() {
    let original_xml = wrap_word_body(
        r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="7"/></w:numPr></w:pPr><w:r><w:t>old body</w:t></w:r></w:p><w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:p><w:r><w:t>old cell</w:t></w:r></w:p><w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:p><w:r><w:t>old nested</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/></w:tc></w:tr></w:tbl><w:sdt><w:sdtPr><w:tag w:val="scope"/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>old control</w:t></w:r></w:p></w:sdtContent></w:sdt>"#,
    );
    let edited_xml = wrap_word_body(
        r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="1"/><w:numId w:val="7"/></w:numPr></w:pPr><w:r><w:t>new body</w:t></w:r></w:p><w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:p><w:r><w:t>new cell</w:t></w:r></w:p><w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:p><w:r><w:t>new nested</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/></w:tc></w:tr></w:tbl><w:sdt><w:sdtPr><w:tag w:val="scope"/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>new control</w:t></w:r></w:p><w:p><w:r><w:t>added control child</w:t></w:r></w:p></w:sdtContent></w:sdt>"#,
    );
    let edited = document_with_content_controls(&edited_xml);
    let mut compared = document_with_content_controls(&original_xml);
    compared
        .compare(&edited, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    assert!(!compared.revisions().is_empty());
    compared.accept_all().unwrap();
    assert!(
        compared
            .compare(&edited, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );
    assert!(compared.revisions().is_empty());
}

#[test]
fn rejecting_a_comparison_reproduces_the_original_body_exactly() {
    let original_xml = wrap_word_body(
        r#"<w:p><w:r><w:t>one</w:t></w:r><w:r><w:t>two</w:t></w:r></w:p><w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:p><w:r><w:t>row</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
    );
    let edited_xml = wrap_word_body(
        r#"<w:p><w:r><w:t>one</w:t></w:r><w:r><w:t>changed</w:t></w:r></w:p><w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:p><w:r><w:t>changed row</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
    );
    let original = document_with_content_controls(&original_xml);
    let edited = document_with_content_controls(&edited_xml);
    let mut compared = document_with_content_controls(&original_xml);
    compared
        .compare(&edited, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    compared.reject_all().unwrap();
    assert!(
        compared
            .compare(&original, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );
    assert!(compared.revisions().is_empty());
}

#[test]
fn formatting_only_changes_report_diagnostics_without_revisions() {
    let original_xml =
        wrap_word_body(r#"<w:p><w:r><w:rPr><w:b/></w:rPr><w:t>same</w:t></w:r></w:p>"#);
    let edited_xml =
        wrap_word_body(r#"<w:p><w:r><w:rPr><w:i/></w:rPr><w:t>same</w:t></w:r></w:p>"#);
    let edited = document_with_content_controls(&edited_xml);
    let mut compared = document_with_content_controls(&original_xml);
    let before = body_from_document(&mut compared);
    let diagnostics = compared
        .compare(&edited, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();

    assert_eq!(
        diagnostics,
        vec![rdocx::ComparisonDiagnostic {
            location: "body/paragraph[0]/run[0]".to_owned(),
            message: "formatting differs and the original formatting was retained".to_owned(),
        }]
    );
    assert!(compared.revisions().is_empty());
    assert_eq!(body_from_document(&mut compared), before);
}

#[test]
fn a_failed_comparison_leaves_the_original_package_unchanged() {
    let existing_xml = wrap_word_body(
        r#"<w:p><w:ins w:id="4" w:author="prior"><w:r><w:t>tracked</w:t></w:r></w:ins></w:p>"#,
    );
    let edited = Document::new();
    let mut document = document_with_content_controls(&existing_xml);
    let before = document.to_bytes().unwrap();

    assert!(
        document
            .compare(&edited, "Ada", "2026-08-21T09:30:00Z")
            .is_err()
    );
    assert_eq!(document.to_bytes().unwrap(), before);
}

#[test]
fn comparison_preserves_unmodelled_xml_byte_for_byte() {
    let body_raw = r#"<x:bodyOpaque xmlns:x="urn:comparison-body" x:value="keep"/>"#;
    let paragraph_raw =
        r#"<x:paragraphOpaque xmlns:x="urn:comparison-paragraph"><x:nested/></x:paragraphOpaque>"#;
    let table_raw = r#"<x:tableOpaque xmlns:x="urn:comparison-table" x:value="keep"/>"#;
    let cell_raw = r#"<x:cellOpaque xmlns:x="urn:comparison-cell" x:value="keep"/>"#;
    let control_raw =
        r#"<x:controlOpaque xmlns:x="urn:comparison-control"><x:nested/></x:controlOpaque>"#;
    let body = |value: &str| {
        wrap_word_body(&format!(
            r#"{body_raw}<w:p><w:r><w:t>{value} body</w:t>{paragraph_raw}</w:r></w:p><w:tbl><w:tblPr/><w:tblGrid/>{table_raw}<w:tr><w:tc>{cell_raw}<w:p><w:r><w:t>{value} cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:sdt><w:sdtPr><w:tag w:val="raw-scope"/></w:sdtPr><w:sdtContent>{control_raw}<w:p><w:r><w:t>{value} control</w:t></w:r></w:p></w:sdtContent></w:sdt>"#,
        ))
    };
    let original_xml = body("old");
    let edited_xml = body("new");
    let edited = document_with_content_controls(&edited_xml);
    let mut document = document_with_content_controls(&original_xml);
    document
        .compare(&edited, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();

    let xml = document_xml(&mut document);
    for raw in [body_raw, table_raw, cell_raw, control_raw] {
        assert_eq!(xml.matches(raw).count(), 1, "{xml}");
    }
    assert_eq!(xml.matches(paragraph_raw).count(), 2, "{xml}");
    let mut reopened = Document::from_bytes(&document.to_bytes().unwrap()).unwrap();
    let reopened_xml = document_xml(&mut reopened);
    for raw in [body_raw, table_raw, cell_raw, control_raw] {
        assert_eq!(reopened_xml.matches(raw).count(), 1, "{reopened_xml}");
    }
    assert_eq!(reopened_xml.matches(paragraph_raw).count(), 2);
}

#[test]
fn inserted_and_deleted_body_blocks_resolve_without_empty_containers() {
    let original_xml = wrap_word_body(
        r#"<w:p><w:r><w:t>first</w:t></w:r></w:p><w:p><w:r><w:t>last</w:t></w:r></w:p>"#,
    );
    let edited_xml = wrap_word_body(
        r#"<w:p><w:r><w:t>first</w:t></w:r></w:p><w:p><w:r><w:t>inserted</w:t></w:r></w:p><w:p><w:r><w:t>last</w:t></w:r></w:p>"#,
    );
    let original = document_with_content_controls(&original_xml);
    let edited = document_with_content_controls(&edited_xml);
    let mut inserted = document_with_content_controls(&original_xml);
    inserted
        .compare(&edited, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    let mut accepted = Document::from_bytes(&inserted.to_bytes().unwrap()).unwrap();
    accepted.accept_all().unwrap();
    let diagnostics = accepted
        .compare(&edited, "postcondition", "2026-08-21T09:31:00Z")
        .unwrap();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let mut rejected = Document::from_bytes(&inserted.to_bytes().unwrap()).unwrap();
    rejected.reject_all().unwrap();
    let diagnostics = rejected
        .compare(&original, "postcondition", "2026-08-21T09:31:00Z")
        .unwrap();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");

    let empty_xml = wrap_word_body(r#"<w:p><w:r><w:t>anchor</w:t></w:r></w:p>"#);
    let table_xml = wrap_word_body(
        r#"<w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:p><w:r><w:t>only row</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p><w:r><w:t>anchor</w:t></w:r></w:p>"#,
    );
    let empty = document_with_content_controls(&empty_xml);
    let table = document_with_content_controls(&table_xml);
    let mut inserted_table = document_with_content_controls(&empty_xml);
    inserted_table
        .compare(&table, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    inserted_table.reject_all().unwrap();
    assert!(
        inserted_table
            .compare(&empty, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );

    let final_xml = wrap_word_body(
        r#"<w:p><w:r><w:t>first</w:t></w:r></w:p><w:p><w:r><w:t>final</w:t></w:r></w:p>"#,
    );
    let final_document = document_with_content_controls(&final_xml);
    let mut inserted_final = document_with_content_controls(&empty_xml);
    inserted_final
        .compare(&final_document, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    inserted_final.reject_all().unwrap();
    assert!(
        inserted_final
            .compare(&empty, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn comparison_revises_nested_control_content_without_replacing_its_shell() {
    let body = |value: &str, paragraph_tag: &str| {
        wrap_word_body(&format!(
            r#"<w:p><w:sdt><w:sdtPr><w:tag w:val="{paragraph_tag}"/></w:sdtPr><w:sdtContent><w:r><w:t>{value} paragraph control</w:t></w:r></w:sdtContent></w:sdt></w:p><w:tbl><w:tblPr/><w:tblGrid/><w:sdt><w:sdtPr><w:tag w:val="table-control"/></w:sdtPr><w:sdtContent><w:tr><w:tc><w:p><w:r><w:t>{value} table control</w:t></w:r></w:p></w:tc></w:tr></w:sdtContent></w:sdt><w:tr><w:sdt><w:sdtPr><w:tag w:val="row-control"/></w:sdtPr><w:sdtContent><w:tc><w:p><w:r><w:t>{value} row control</w:t></w:r></w:p></w:tc></w:sdtContent></w:sdt><w:tc><w:sdt><w:sdtPr><w:tag w:val="cell-control"/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>{value} cell control</w:t></w:r></w:p></w:sdtContent></w:sdt></w:tc></w:tr></w:tbl>"#,
        ))
    };
    let original_xml = body("old", "paragraph-control");
    let edited_xml = body("new", "paragraph-control");
    let original = document_with_content_controls(&original_xml);
    let edited = document_with_content_controls(&edited_xml);
    let mut compared = document_with_content_controls(&original_xml);
    compared
        .compare(&edited, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    let tracked_xml = document_xml(&mut compared);
    for tag in [
        "paragraph-control",
        "table-control",
        "row-control",
        "cell-control",
    ] {
        assert_eq!(tracked_xml.matches(&format!(r#"w:val="{tag}""#)).count(), 1);
    }
    let mut accepted = Document::from_bytes(&compared.to_bytes().unwrap()).unwrap();
    accepted.accept_all().unwrap();
    let diagnostics = accepted
        .compare(&edited, "postcondition", "2026-08-21T09:31:00Z")
        .unwrap();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let mut rejected = Document::from_bytes(&compared.to_bytes().unwrap()).unwrap();
    rejected.reject_all().unwrap();
    let diagnostics = rejected
        .compare(&original, "postcondition", "2026-08-21T09:31:00Z")
        .unwrap();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");

    let changed_shell_xml = body("old", "changed-shell");
    let changed_shell = document_with_content_controls(&changed_shell_xml);
    let mut unchanged = document_with_content_controls(&original_xml);
    let before = unchanged.to_bytes().unwrap();
    assert!(
        unchanged
            .compare(&changed_shell, "Ada", "2026-08-21T09:30:00Z")
            .is_err()
    );
    assert_eq!(unchanged.to_bytes().unwrap(), before);
}

#[test]
fn comparison_preserves_content_control_whitespace_slots() {
    let first_slot = "\r\n\t \t\r\n";
    let second_slot = "\n \t \n";
    let body = |value: &str| {
        wrap_word_body(&format!(
            r#"<w:sdt><w:sdtPr><w:tag w:val="pretty"/></w:sdtPr><w:sdtContent>{first_slot}<w:p><w:r><w:t>{value}</w:t></w:r></w:p>{second_slot}</w:sdtContent></w:sdt>"#,
        ))
    };
    let original_xml = body("old");
    let edited_xml = body("new");
    let edited = document_with_content_controls(&edited_xml);
    let mut compared = document_with_content_controls(&original_xml);

    compared
        .compare(&edited, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    let tracked = document_xml(&mut compared);
    assert!(tracked.contains(first_slot), "{tracked:?}");
    assert!(tracked.contains(second_slot), "{tracked:?}");

    let mut accepted = Document::from_bytes(&compared.to_bytes().unwrap()).unwrap();
    accepted.accept_all().unwrap();
    assert!(
        accepted
            .compare(&edited, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn comparison_deletes_text_without_corrupting_tabs() {
    let original_xml =
        wrap_word_body(r#"<w:p><w:r><w:t>old</w:t><w:tab/><w:t>tail</w:t></w:r></w:p>"#);
    let edited_xml =
        wrap_word_body(r#"<w:p><w:r><w:t>new</w:t><w:tab/><w:t>tail</w:t></w:r></w:p>"#);
    let original = document_with_content_controls(&original_xml);
    let edited = document_with_content_controls(&edited_xml);
    let mut compared = document_with_content_controls(&original_xml);

    compared
        .compare(&edited, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    let tracked = document_xml(&mut compared);
    assert!(tracked.contains("<w:tab/>"), "{tracked}");
    assert!(!tracked.contains("delTextab"), "{tracked}");

    let mut accepted = Document::from_bytes(&compared.to_bytes().unwrap()).unwrap();
    accepted.accept_all().unwrap();
    assert!(
        accepted
            .compare(&edited, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );
    let mut rejected = Document::from_bytes(&compared.to_bytes().unwrap()).unwrap();
    rejected.reject_all().unwrap();
    assert!(
        rejected
            .compare(&original, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn comparison_reports_formatting_inside_matched_table_rows() {
    let original_xml = wrap_word_body(
        r#"<w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:tcPr><w:shd w:fill="FF0000"/></w:tcPr><w:p><w:pPr><w:jc w:val="left"/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>same</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
    );
    let edited_xml = wrap_word_body(
        r#"<w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:tcPr><w:shd w:fill="0000FF"/></w:tcPr><w:p><w:pPr><w:jc w:val="right"/></w:pPr><w:r><w:rPr><w:i/></w:rPr><w:t>same</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
    );
    let edited = document_with_content_controls(&edited_xml);
    let mut compared = document_with_content_controls(&original_xml);

    let diagnostics = compared
        .compare(&edited, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.location.as_str())
            .collect::<Vec<_>>(),
        vec![
            "body/table[0]/row[0]/cell[0]",
            "body/table[0]/row[0]/cell[0]/content[0]",
            "body/table[0]/row[0]/cell[0]/content[0]/run[0]",
        ]
    );
    assert!(compared.revisions().is_empty());
}

#[test]
fn comparison_replaces_paragraphs_and_tables_before_an_anchor() {
    let paragraph_xml = wrap_word_body(
        r#"<w:p><w:r><w:t>old paragraph</w:t></w:r></w:p><w:p><w:r><w:t>anchor</w:t></w:r></w:p>"#,
    );
    let table_xml = wrap_word_body(
        r#"<w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:p><w:r><w:t>new table</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p><w:r><w:t>anchor</w:t></w:r></w:p>"#,
    );
    let paragraph = document_with_content_controls(&paragraph_xml);
    let table = document_with_content_controls(&table_xml);

    let mut paragraph_to_table = document_with_content_controls(&paragraph_xml);
    paragraph_to_table
        .compare(&table, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    let mut accepted = Document::from_bytes(&paragraph_to_table.to_bytes().unwrap()).unwrap();
    accepted.accept_all().unwrap();
    assert!(
        accepted
            .compare(&table, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );
    let mut rejected = Document::from_bytes(&paragraph_to_table.to_bytes().unwrap()).unwrap();
    rejected.reject_all().unwrap();
    assert!(
        rejected
            .compare(&paragraph, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );

    let mut table_to_paragraph = document_with_content_controls(&table_xml);
    table_to_paragraph
        .compare(&paragraph, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    let mut accepted = Document::from_bytes(&table_to_paragraph.to_bytes().unwrap()).unwrap();
    accepted.accept_all().unwrap();
    assert!(
        accepted
            .compare(&paragraph, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );
    let mut rejected = Document::from_bytes(&table_to_paragraph.to_bytes().unwrap()).unwrap();
    rejected.reject_all().unwrap();
    assert!(
        rejected
            .compare(&table, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn comparison_preserves_unrelated_modeled_fields() {
    let field =
        r#"<w:fldSimple w:instr="AUTHOR"><w:r><w:t>stored author</w:t></w:r></w:fldSimple>"#;
    let original_xml = wrap_word_body(&format!(
        r#"<w:p>{field}<w:r><w:t>old text</w:t></w:r></w:p>"#
    ));
    let edited_xml = wrap_word_body(&format!(
        r#"<w:p>{field}<w:r><w:t>new text</w:t></w:r></w:p>"#
    ));
    let original = document_with_content_controls(&original_xml);
    let edited = document_with_content_controls(&edited_xml);
    let mut compared = document_with_content_controls(&original_xml);

    compared
        .compare(&edited, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    assert_eq!(
        document_xml(&mut compared).matches("<w:fldSimple").count(),
        1
    );

    let mut accepted = Document::from_bytes(&compared.to_bytes().unwrap()).unwrap();
    accepted.accept_all().unwrap();
    assert!(
        accepted
            .compare(&edited, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        document_xml(&mut accepted).matches("<w:fldSimple").count(),
        1
    );

    let mut rejected = Document::from_bytes(&compared.to_bytes().unwrap()).unwrap();
    rejected.reject_all().unwrap();
    assert!(
        rejected
            .compare(&original, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        document_xml(&mut rejected).matches("<w:fldSimple").count(),
        1
    );
}

#[test]
fn final_paragraph_markers_stay_outside_formatted_run_properties() {
    let anchor = r#"<w:p><w:r><w:rPr><w:b/></w:rPr><w:t>anchor</w:t></w:r></w:p>"#;
    let added = r#"<w:p><w:r><w:t>added</w:t></w:r></w:p>"#;
    let original_xml = wrap_word_body(anchor);
    let edited_xml = wrap_word_body(&format!("{anchor}{added}"));
    let original = document_with_content_controls(&original_xml);
    let edited = document_with_content_controls(&edited_xml);
    let mut compared = document_with_content_controls(&original_xml);

    compared
        .compare(&edited, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    let tracked = document_xml(&mut compared);
    let marker = tracked.find("<w:ins").unwrap();
    let properties_start = tracked.find("<w:pPr").unwrap();
    let properties_end = tracked.find("</w:pPr>").unwrap();
    let first_run = tracked.find("<w:r>").unwrap();
    assert!(
        properties_start < marker && marker < properties_end,
        "{tracked}"
    );
    assert!(properties_end < first_run, "{tracked}");
    assert_eq!(tracked.matches("<w:b/>").count(), 1, "{tracked}");
    let mut rejected = Document::from_bytes(&compared.to_bytes().unwrap()).unwrap();
    rejected.reject_all().unwrap();
    assert!(
        rejected
            .compare(&original, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );

    let mut deletion = document_with_content_controls(&edited_xml);
    deletion
        .compare(&original, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    let tracked = document_xml(&mut deletion);
    let marker = tracked.find("<w:del").unwrap();
    let properties_start = tracked.find("<w:pPr").unwrap();
    let properties_end = tracked.find("</w:pPr>").unwrap();
    let first_run = tracked.find("<w:r>").unwrap();
    assert!(
        properties_start < marker && marker < properties_end,
        "{tracked}"
    );
    assert!(properties_end < first_run, "{tracked}");
    let mut accepted = Document::from_bytes(&deletion.to_bytes().unwrap()).unwrap();
    accepted.accept_all().unwrap();
    assert!(
        accepted
            .compare(&original, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn whole_row_markers_target_the_outer_row_properties() {
    let anchor_xml = wrap_word_body(r#"<w:p><w:r><w:t>anchor</w:t></w:r></w:p>"#);
    let table_xml = wrap_word_body(
        r#"<w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:trPr><w:cantSplit/></w:trPr><w:tc><w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:trPr><w:tblHeader/></w:trPr><w:tc><w:p><w:r><w:t>nested</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/></w:tc></w:tr></w:tbl><w:p><w:r><w:t>anchor</w:t></w:r></w:p>"#,
    );
    let anchor = document_with_content_controls(&anchor_xml);
    let table = document_with_content_controls(&table_xml);
    let mut compared = document_with_content_controls(&anchor_xml);

    compared
        .compare(&table, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    let tracked = document_xml(&mut compared);
    let marker = tracked.find("<w:ins").unwrap();
    let outer_cell = tracked.find("<w:tc>").unwrap();
    let nested_properties = tracked.find("<w:tblHeader/>").unwrap();
    assert!(marker < outer_cell, "{tracked}");
    assert!(marker < nested_properties, "{tracked}");
    assert_eq!(tracked.matches("<w:ins").count(), 1, "{tracked}");

    let mut rejected = Document::from_bytes(&compared.to_bytes().unwrap()).unwrap();
    rejected.reject_all().unwrap();
    assert!(
        rejected
            .compare(&anchor, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );

    let mut deletion = document_with_content_controls(&table_xml);
    deletion
        .compare(&anchor, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    let tracked = document_xml(&mut deletion);
    assert!(tracked.find("<w:del").unwrap() < tracked.find("<w:tc>").unwrap());
    let mut accepted = Document::from_bytes(&deletion.to_bytes().unwrap()).unwrap();
    accepted.accept_all().unwrap();
    assert!(
        accepted
            .compare(&anchor, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn whole_table_marking_includes_control_owned_rows() {
    let anchor_xml = wrap_word_body(r#"<w:p><w:r><w:t>anchor</w:t></w:r></w:p>"#);
    let table_xml = wrap_word_body(
        r#"<w:tbl><w:tblPr/><w:tblGrid/><w:sdt><w:sdtPr><w:tag w:val="owned-row"/></w:sdtPr><w:sdtContent><w:tr><w:tc><w:p><w:r><w:t>same row</w:t></w:r></w:p></w:tc></w:tr></w:sdtContent></w:sdt><w:tr><w:tc><w:p><w:r><w:t>same row</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p><w:r><w:t>anchor</w:t></w:r></w:p>"#,
    );
    let anchor = document_with_content_controls(&anchor_xml);
    let table = document_with_content_controls(&table_xml);

    let mut insertion = document_with_content_controls(&anchor_xml);
    insertion
        .compare(&table, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    let tracked = document_xml(&mut insertion);
    assert_eq!(tracked.matches(r#"<w:trPr><w:ins"#).count(), 2, "{tracked}");
    let mut rejected = Document::from_bytes(&insertion.to_bytes().unwrap()).unwrap();
    rejected.reject_all().unwrap();
    assert!(
        rejected
            .compare(&anchor, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );

    let mut deletion = document_with_content_controls(&table_xml);
    deletion
        .compare(&anchor, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    let tracked = document_xml(&mut deletion);
    assert_eq!(tracked.matches(r#"<w:trPr><w:del"#).count(), 2, "{tracked}");
    let mut accepted = Document::from_bytes(&deletion.to_bytes().unwrap()).unwrap();
    accepted.accept_all().unwrap();
    assert!(
        accepted
            .compare(&anchor, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn table_row_insertions_stay_inside_table_schema_boundaries() {
    let original_xml = wrap_word_body(
        r#"<w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:p><w:r><w:t>anchor</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
    );
    let edited_xmls = [
        wrap_word_body(
            r#"<w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:p><w:r><w:t>inserted</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>anchor</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        ),
        wrap_word_body(
            r#"<w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:p><w:r><w:t>anchor</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>inserted</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        ),
    ];
    let original = document_with_content_controls(&original_xml);

    for edited_xml in edited_xmls {
        let edited = document_with_content_controls(&edited_xml);
        let mut compared = document_with_content_controls(&original_xml);
        compared
            .compare(&edited, "Ada", "2026-08-21T09:30:00Z")
            .unwrap();
        let tracked = document_xml(&mut compared);
        let table_start = tracked.find("<w:tbl>").unwrap();
        let table_end = tracked.find("</w:tbl>").unwrap();
        let inserted_row = tracked
            .find("<w:ins")
            .unwrap_or_else(|| panic!("{tracked}"));
        assert!(
            table_start < inserted_row && inserted_row < table_end,
            "{tracked}"
        );

        let mut accepted = Document::from_bytes(&compared.to_bytes().unwrap()).unwrap();
        accepted.accept_all().unwrap();
        assert!(
            accepted
                .compare(&edited, "postcondition", "2026-08-21T09:31:00Z")
                .unwrap()
                .is_empty()
        );
        let mut rejected = Document::from_bytes(&compared.to_bytes().unwrap()).unwrap();
        rejected.reject_all().unwrap();
        assert!(
            rejected
                .compare(&original, "postcondition", "2026-08-21T09:31:00Z")
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn comparison_adds_and_removes_numbering_without_a_property_owner() {
    let plain_xml = wrap_word_body(r#"<w:p><w:r><w:t>item</w:t></w:r></w:p>"#);
    let numbered_xml = wrap_word_body(
        r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="2"/><w:numId w:val="7"/></w:numPr></w:pPr><w:r><w:t>item</w:t></w:r></w:p>"#,
    );
    let plain = document_with_content_controls(&plain_xml);
    let numbered = document_with_content_controls(&numbered_xml);

    let mut addition = document_with_content_controls(&plain_xml);
    addition
        .compare(&numbered, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    let addition_xml = document_xml(&mut addition);
    assert!(addition_xml.contains("<w:pPrChange"), "{addition_xml}");
    let mut accepted = Document::from_bytes(&addition.to_bytes().unwrap()).unwrap();
    accepted.accept_all().unwrap();
    assert!(
        accepted
            .compare(&numbered, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );
    let mut rejected = Document::from_bytes(&addition.to_bytes().unwrap()).unwrap();
    rejected.reject_all().unwrap();
    assert!(
        rejected
            .compare(&plain, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );

    let mut removal = document_with_content_controls(&numbered_xml);
    removal
        .compare(&plain, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    let removal_xml = document_xml(&mut removal);
    assert!(removal_xml.contains("<w:pPrChange"), "{removal_xml}");
    let mut accepted = Document::from_bytes(&removal.to_bytes().unwrap()).unwrap();
    accepted.accept_all().unwrap();
    assert!(
        accepted
            .compare(&plain, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );
    let mut rejected = Document::from_bytes(&removal.to_bytes().unwrap()).unwrap();
    rejected.reject_all().unwrap();
    assert!(
        rejected
            .compare(&numbered, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn numbering_changes_preserve_unrelated_paragraph_properties() {
    let unnumbered_xml = wrap_word_body(
        r#"<w:p><w:pPr><w:keepNext/><w:jc w:val="center"/></w:pPr><w:r><w:t>item</w:t></w:r></w:p>"#,
    );
    let numbered_xml = wrap_word_body(
        r#"<w:p><w:pPr><w:keepNext/><w:numPr><w:ilvl w:val="1"/><w:numId w:val="9"/></w:numPr><w:jc w:val="center"/></w:pPr><w:r><w:t>item</w:t></w:r></w:p>"#,
    );

    for (original_xml, edited_xml) in [
        (&unnumbered_xml, &numbered_xml),
        (&numbered_xml, &unnumbered_xml),
    ] {
        let original = document_with_content_controls(original_xml);
        let edited = document_with_content_controls(edited_xml);
        let mut compared = document_with_content_controls(original_xml);
        compared
            .compare(&edited, "Ada", "2026-08-21T09:30:00Z")
            .unwrap();

        let mut accepted = Document::from_bytes(&compared.to_bytes().unwrap()).unwrap();
        accepted.accept_all().unwrap();
        let accepted_xml = document_xml(&mut accepted);
        assert!(accepted_xml.contains("<w:keepNext"), "{accepted_xml}");
        assert!(
            accepted_xml.contains(r#"<w:jc w:val="center"/>"#),
            "{accepted_xml}"
        );
        assert!(
            accepted
                .compare(&edited, "postcondition", "2026-08-21T09:31:00Z")
                .unwrap()
                .is_empty()
        );

        let mut rejected = Document::from_bytes(&compared.to_bytes().unwrap()).unwrap();
        rejected.reject_all().unwrap();
        let rejected_xml = document_xml(&mut rejected);
        assert!(rejected_xml.contains("<w:keepNext"), "{rejected_xml}");
        assert!(
            rejected_xml.contains(r#"<w:jc w:val="center"/>"#),
            "{rejected_xml}"
        );
        assert!(
            rejected
                .compare(&original, "postcondition", "2026-08-21T09:31:00Z")
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn resolving_empty_paragraph_property_changes_cleans_only_empty_owners() {
    let empty_xml = wrap_word_body(
        r#"<w:p><w:pPr><w:pPrChange w:id="1" w:author="Ada"><w:pPr/></w:pPrChange></w:pPr><w:r><w:t>item</w:t></w:r></w:p>"#,
    );
    let retained_xml = wrap_word_body(
        r#"<w:p><w:pPr><w:keepNext/><w:pPrChange w:id="1" w:author="Ada"><w:pPr><w:keepNext/></w:pPr></w:pPrChange></w:pPr><w:r><w:t>item</w:t></w:r></w:p>"#,
    );

    for accept in [true, false] {
        let mut empty = document_with_content_controls(&empty_xml);
        if accept {
            empty.accept_all().unwrap();
        } else {
            empty.reject_all().unwrap();
        }
        let empty = document_xml(&mut empty);
        assert!(!empty.contains("<w:pPr"), "{empty}");

        let mut retained = document_with_content_controls(&retained_xml);
        if accept {
            retained.accept_all().unwrap();
        } else {
            retained.reject_all().unwrap();
        }
        let retained = document_xml(&mut retained);
        assert_eq!(retained.matches("<w:pPr>").count(), 1, "{retained}");
        assert_eq!(retained.matches("<w:keepNext").count(), 1, "{retained}");
        assert!(!retained.contains("<w:pPrChange"), "{retained}");
    }
}

#[test]
fn rejecting_an_attributed_empty_prior_paragraph_owner_preserves_it_exactly() {
    let prior = format!(
        r#"<old:pPr xmlns:old="{}" xmlns:ext="urn:producer" ext:flag="keep" ext:mode="exact"/>"#,
        rdocx_oxml::namespace::W_NS
    );
    let tracked_xml = wrap_word_body(&format!(
        r#"<w:p><w:pPr><w:pPrChange w:id="1" w:author="Ada">{prior}</w:pPrChange></w:pPr><w:r><w:t>item</w:t></w:r></w:p>"#,
    ));
    let mut tracked = document_with_content_controls(&tracked_xml);

    let mut accepted = Document::from_bytes(&tracked.to_bytes().unwrap()).unwrap();
    accepted.accept_all().unwrap();
    let accepted = document_xml(&mut accepted);
    assert!(!accepted.contains("<w:pPr"), "{accepted}");
    assert!(!accepted.contains("ext:flag"), "{accepted}");

    let mut rejected = Document::from_bytes(&tracked.to_bytes().unwrap()).unwrap();
    rejected.reject_all().unwrap();
    let rejected = document_xml(&mut rejected);
    assert!(rejected.contains(&prior), "{rejected}");
    assert!(!rejected.contains("<w:pPrChange"), "{rejected}");

    let tracked_xml = wrap_word_body(
        r#"<w:p><w:pPr><w:pPrChange xmlns:ext="urn:inherited" w:id="2" w:author="Ada"><w:pPr ext:flag="keep"/></w:pPrChange></w:pPr><w:r><w:t>item</w:t></w:r></w:p>"#,
    );
    let mut tracked = document_with_content_controls(&tracked_xml);
    let mut rejected = Document::from_bytes(&tracked.to_bytes().unwrap()).unwrap();
    rejected.reject_all().unwrap();
    let rejected = document_xml(&mut rejected);
    assert!(rejected.contains("<w:pPr"), "{rejected}");
    assert!(
        rejected.contains(r#"xmlns:ext="urn:inherited""#),
        "{rejected}"
    );
    assert!(rejected.contains(r#"ext:flag="keep""#), "{rejected}");
    assert!(!rejected.contains("<w:pPrChange"), "{rejected}");
}

#[test]
fn control_block_replacement_keeps_whitespace_before_the_replacement() {
    let before = "\r\n\t  \t\r\n";
    let between = "\n\t \n";
    let control = |first: &str| {
        wrap_word_body(&format!(
            r#"<w:sdt><w:sdtPr><w:tag w:val="block-replacement"/></w:sdtPr><w:sdtContent>{before}{first}{between}<w:p><w:r><w:t>anchor</w:t></w:r></w:p></w:sdtContent></w:sdt>"#,
        ))
    };
    let paragraph_xml = control(r#"<w:p><w:r><w:t>old paragraph</w:t></w:r></w:p>"#);
    let table_xml = control(
        r#"<w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:p><w:r><w:t>new table</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
    );
    let paragraph = document_with_content_controls(&paragraph_xml);
    let table = document_with_content_controls(&table_xml);
    let mut compared = document_with_content_controls(&paragraph_xml);

    compared
        .compare(&table, "Ada", "2026-08-21T09:30:00Z")
        .unwrap();
    let mut accepted = Document::from_bytes(&compared.to_bytes().unwrap()).unwrap();
    accepted.accept_all().unwrap();
    let accepted_xml = document_xml(&mut accepted);
    assert!(accepted_xml.contains(before), "{accepted_xml:?}");
    assert!(accepted_xml.contains(between), "{accepted_xml:?}");
    assert!(accepted_xml.find(before).unwrap() < accepted_xml.find("<w:tbl>").unwrap());
    assert!(accepted_xml.find("</w:tbl>").unwrap() < accepted_xml.find(between).unwrap());
    assert!(
        accepted
            .compare(&table, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );

    let mut rejected = Document::from_bytes(&compared.to_bytes().unwrap()).unwrap();
    rejected.reject_all().unwrap();
    let rejected_xml = document_xml(&mut rejected);
    assert!(rejected_xml.find(before).unwrap() < rejected_xml.find("<w:p>").unwrap());
    assert!(
        rejected
            .compare(&paragraph, "postcondition", "2026-08-21T09:31:00Z")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn fixed_break_runs_match_pdf_and_raster_backends() {
    let mut document = Document::new();
    document.add_paragraph(&"financial planning ttf-parser double  spaces allocated ".repeat(12));
    let (family, bytes) = oxml_layout::bundled_fonts::bundled_font_data()[0];
    let layout = document
        .layout_with_fonts(&[(family, bytes)])
        .expect("caller-font deterministic layout");
    let mut fonts =
        oxml_layout::FontManager::new_with_fonts(vec![(family.to_owned(), bytes.to_vec())]);
    for run in layout.layout.pages.iter().flat_map(|page| {
        page.elements.iter().filter_map(|element| match element {
            oxml_layout::PositionedElement::Text(run) if run.source.is_some() => Some(run),
            _ => None,
        })
    }) {
        let font_family = layout
            .layout
            .fonts
            .iter()
            .find(|font| font.id == run.font_id)
            .expect("run font is in result")
            .family
            .clone();
        let font_id = fonts
            .resolve_font(Some(&font_family), run.bold, run.italic)
            .expect("caller run font resolves");
        let independently_shaped = fonts
            .shape_text(font_id, &run.text, run.font_size)
            .expect("emitted chunk reshapes");
        assert_eq!(run.glyph_ids, independently_shaped.glyph_ids);
        assert_eq!(run.advances, independently_shaped.advances);
    }

    let direct_pdf = oxml_pdf::render_to_pdf(&layout.layout);
    let direct_png = oxml_pdf::render_page_to_png(&layout.layout, 0, 96.0)
        .expect("raster backend renders first page");
    assert_eq!(
        document
            .to_pdf_with_fonts(&[(family, bytes)])
            .expect("PDF facade"),
        direct_pdf
    );
    assert!(!direct_png.is_empty());
}

fn empty_story_layout_input() -> rdocx_layout::LayoutInput {
    use rdocx_oxml::footnotes::{CT_Footnote, CT_Footnotes, NoteType};
    use rdocx_oxml::header_footer::CT_HdrFtr;

    let document = CT_Document::from_xml(
        br#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body><w:p/><w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:p><w:pPr/></w:p></w:tc></w:tr></w:tbl><w:p><w:r><w:footnoteReference w:id="4"/><w:endnoteReference w:id="9"/></w:r></w:p><w:sectPr><w:headerReference w:type="default" r:id="rIdHeader"/><w:footerReference w:type="default" r:id="rIdFooter"/></w:sectPr></w:body></w:document>"#,
    )
    .expect("empty story document parses");
    let mut header = CT_HdrFtr::new();
    header.paragraphs.push(rdocx_oxml::text::CT_P::new());
    let mut footer = CT_HdrFtr::new();
    footer.paragraphs.push(rdocx_oxml::text::CT_P::new());
    let empty_note = |id| CT_Footnote {
        id,
        note_type: NoteType::Normal,
        paragraphs: vec![rdocx_oxml::text::CT_P::new()],
    };

    rdocx_layout::LayoutInput {
        document,
        styles: rdocx_oxml::styles::CT_Styles::new_default(),
        numbering: None,
        headers: HashMap::from([("rIdHeader".to_owned(), header)]),
        footers: HashMap::from([("rIdFooter".to_owned(), footer)]),
        images: HashMap::new(),
        charts: HashMap::new(),
        chart_theme: oxml_drawing::theme::CT_OfficeStyleSheet::office_default(),
        chart_color_map: oxml_drawing::color::ColorMap::default(),
        core_properties: None,
        hyperlink_urls: HashMap::new(),
        footnotes: Some(CT_Footnotes {
            footnotes: vec![empty_note(4)],
        }),
        endnotes: Some(CT_Footnotes {
            footnotes: vec![empty_note(9)],
        }),
        theme: None,
        fonts: Vec::new(),
        revision_view: rdocx_layout::RevisionView::Accepted,
    }
}

#[test]
fn empty_word_stories_emit_one_attributed_zero_width_segment() {
    use rdocx_layout::{WordSourcePath, WordStory};

    let result =
        rdocx_layout::layout_document_deterministic_with_provenance(&empty_story_layout_input())
            .expect("empty Word stories lay out");
    let attributed = result
        .layout
        .pages
        .iter()
        .flat_map(|page| compatibility_page_elements(&page.elements))
        .filter_map(|element| match element {
            oxml_layout::PositionedElement::Text(run) if run.text.is_empty() => Some(run),
            _ => None,
        })
        .collect::<Vec<_>>();
    let actual = attributed
        .iter()
        .filter_map(|run| run.source)
        .filter_map(|source| result.source_node(source.node))
        .collect::<Vec<_>>();
    assert_eq!(attributed.len(), 6, "attributed empty stories: {actual:?}");
    let expected = [
        WordSourcePath {
            story: WordStory::Document,
            children: vec![0],
        },
        WordSourcePath {
            story: WordStory::Document,
            children: vec![1, 0, 0, 0],
        },
        WordSourcePath {
            story: WordStory::Header {
                relationship_id: "rIdHeader".to_owned(),
            },
            children: vec![0],
        },
        WordSourcePath {
            story: WordStory::Footer {
                relationship_id: "rIdFooter".to_owned(),
            },
            children: vec![0],
        },
        WordSourcePath {
            story: WordStory::Footnote { id: 4 },
            children: vec![0],
        },
        WordSourcePath {
            story: WordStory::Endnote { id: 9 },
            children: vec![0],
        },
    ];
    for path in expected {
        let matching = attributed
            .iter()
            .filter(|run| {
                run.source.is_some_and(|source| {
                    source.char_start == 0
                        && source.char_end == 0
                        && result.source_node(source.node) == Some(&path)
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1, "missing or duplicate caret for {path:?}");
        assert_eq!(matching[0].advances.iter().sum::<f64>(), 0.0);
        assert_eq!(matching[0].advances, Vec::<f64>::new());
        assert_eq!(matching[0].glyph_ids, Vec::<u16>::new());
    }
}

#[test]
fn empty_paragraph_uses_resolved_default_metrics() {
    use rdocx_oxml::properties::{CT_PPr, CT_RPr};
    use rdocx_oxml::units::HalfPoint;

    let mut input = empty_story_layout_input();
    input.document = CT_Document::from_xml(
        br#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p/><w:p/></w:body></w:document>"#,
    )
    .expect("metric document parses");
    input.styles.styles[0].rpr = Some(CT_RPr {
        font_ascii: Some("Caladea".to_owned()),
        font_hansi: Some("Caladea".to_owned()),
        sz: Some(HalfPoint(28)),
        ..Default::default()
    });
    let BodyContent::Paragraph(direct) = &mut input.document.body.content[0] else {
        panic!("direct paragraph");
    };
    direct.properties = Some(CT_PPr {
        rpr: Some(CT_RPr {
            font_ascii: Some("Carlito".to_owned()),
            font_hansi: Some("Carlito".to_owned()),
            sz: Some(HalfPoint(36)),
            ..Default::default()
        }),
        ..Default::default()
    });

    let media = rdocx_layout::MediaRegistry::new(&input.images);
    let mut font_manager =
        oxml_layout::FontManager::new_deterministic().expect("deterministic metric fonts load");
    let mut numbering = rdocx_layout::style_resolver::NumberingState::new();
    let mut diagnostics = Vec::new();
    for index in 0..2 {
        let BodyContent::Paragraph(paragraph) = &input.document.body.content[index] else {
            panic!("metric paragraph");
        };
        let block = rdocx_layout::engine::layout_paragraph(
            paragraph,
            400.0,
            &input.styles,
            &input,
            &media,
            &mut font_manager,
            &mut numbering,
            &mut diagnostics,
        )
        .expect("metric paragraph lays out");
        let oxml_layout::LineItem::Text(segment) = &block.lines[0].items[0] else {
            panic!("empty paragraph carrier");
        };
        assert_eq!(segment.width, 0.0);
        let resolved = font_manager
            .metrics(segment.font_id, segment.font_size)
            .expect("carrier metrics resolve");
        assert_eq!(segment.ascent, resolved.ascent);
        assert_eq!(segment.descent, resolved.descent);
    }

    let result = rdocx_layout::layout_document_deterministic_with_provenance(&input)
        .expect("empty metric paragraphs lay out");
    let empty_runs = result
        .layout
        .pages
        .iter()
        .flat_map(|page| compatibility_page_elements(&page.elements))
        .filter_map(|element| match element {
            oxml_layout::PositionedElement::Text(run) if run.text.is_empty() => Some(run),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(empty_runs.len(), 2);
    let metric = |children: &[usize]| {
        let run = empty_runs
            .iter()
            .find(|run| {
                run.source.is_some_and(|source| {
                    matches!(
                        result.source_node(source.node),
                        Some(rdocx_layout::WordSourcePath {
                            story: rdocx_layout::WordStory::Document,
                            children: actual,
                        }) if actual == children
                    )
                })
            })
            .expect("empty paragraph run");
        let family = result
            .layout
            .fonts
            .iter()
            .find(|font| font.id == run.font_id)
            .expect("caret font retained")
            .family
            .as_str();
        (family, run.font_size)
    };
    assert_eq!(metric(&[0]), ("Carlito", 18.0));
    assert_eq!(metric(&[1]), ("Caladea", 14.0));
}

#[test]
fn empty_segment_is_backend_invisible_and_layout_compatible() {
    let mut input = empty_story_layout_input();
    input.document = CT_Document::from_xml(
        br#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:pPr><w:rPr><w:u w:val="double"/><w:highlight w:val="yellow"/><w:strike/></w:rPr></w:pPr></w:p><w:p><w:r><w:t>visible</w:t></w:r></w:p></w:body></w:document>"#,
    )
    .expect("compatibility document parses");
    let ordinary = rdocx_layout::layout_document_deterministic(&input).expect("ordinary layout");
    let mut attributed = rdocx_layout::layout_document_deterministic_with_provenance(&input)
        .expect("attributed layout")
        .into_layout_result();
    assert!(
        attributed
            .pages
            .iter()
            .flat_map(|page| compatibility_page_elements(&page.elements))
            .any(
                |element| matches!(element, oxml_layout::PositionedElement::Text(run)
            if run.text == "visible" && !run.glyph_ids.is_empty())
            )
    );
    for page in &mut attributed.pages {
        clear_compatibility_sources(&mut std::sync::Arc::make_mut(page).elements);
    }
    assert_eq!(format!("{ordinary:?}"), format!("{attributed:?}"));

    let mut without_empty = attributed.clone();
    for page in &mut without_empty.pages {
        remove_compatibility_empty_text(&mut std::sync::Arc::make_mut(page).elements);
    }
    assert_eq!(
        oxml_pdf::render_to_pdf(&attributed),
        oxml_pdf::render_to_pdf(&without_empty)
    );
    assert_eq!(
        oxml_pdf::render_page_to_png(&attributed, 0, 96.0),
        oxml_pdf::render_page_to_png(&without_empty, 0, 96.0)
    );
}
