//! The HTML and Markdown emitters treat the source document as untrusted.
//!
//! A DOCX is an attacker-supplied file in most deployments, so nothing it
//! carries may become markup or a script trigger in the converted output.

use std::collections::HashMap;

use rdocx_html::{HtmlInput, HtmlOptions, ImageData, to_html_fragment, to_markdown};
use rdocx_oxml::document::CT_Document;
use rdocx_oxml::properties::{CT_RPr, CT_Shd};
use rdocx_oxml::styles::CT_Styles;
use rdocx_oxml::text::{CT_Hyperlink, CT_P, CT_R, ParagraphChild};

fn input_from(doc: CT_Document, hyperlink_urls: HashMap<String, String>) -> HtmlInput {
    HtmlInput {
        document: doc,
        styles: CT_Styles::new_default(),
        numbering: None,
        images: HashMap::new(),
        hyperlink_urls,
    }
}

fn linked_paragraph(url: &str) -> HtmlInput {
    let mut doc = CT_Document::new();
    let mut p = CT_P::new();
    let mut hyperlink = CT_Hyperlink::new(Some("rId7".to_string()), None);
    hyperlink.add_run("click me");
    p.add_hyperlink(hyperlink);
    doc.body.add_paragraph(p);

    input_from(doc, HashMap::from([("rId7".to_string(), url.to_string())]))
}

#[test]
fn dangerous_url_schemes_are_dropped() {
    for url in [
        "javascript:alert(1)",
        "JaVaScRiPt:alert(1)",
        "  javascript:alert(1)",
        "data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==",
        "vbscript:msgbox(1)",
    ] {
        let input = linked_paragraph(url);

        let html = to_html_fragment(&input, &HtmlOptions::default());
        let markdown = to_markdown(&input);

        assert!(!html.contains("<a "), "emitted a link for {url}: {html}");
        assert!(!markdown.contains(url), "markdown kept {url}: {markdown}");
        // The text itself must still render.
        assert!(html.contains("click me"));
        assert!(markdown.contains("click me"));
    }
}

#[test]
fn ordinary_url_schemes_still_link() {
    for url in [
        "https://example.com/a?b=c",
        "http://example.com",
        "mailto:someone@example.com",
        "#internal-anchor",
        "relative/page.html",
    ] {
        let input = linked_paragraph(url);
        let html = to_html_fragment(&input, &HtmlOptions::default());
        assert!(html.contains("<a href="), "no link for {url}: {html}");
    }
}

#[test]
fn font_names_cannot_escape_the_style_attribute() {
    let mut doc = CT_Document::new();
    let mut p = CT_P::new();
    let mut run = CT_R::new("styled");
    run.properties = Some(CT_RPr {
        font_ascii: Some("Arial\" onmouseover=\"alert(1)".to_string()),
        ..Default::default()
    });
    p.content.push(ParagraphChild::Run(run));
    doc.body.add_paragraph(p);

    let html = to_html_fragment(&input_from(doc, HashMap::new()), &HtmlOptions::default());

    assert!(!html.contains("onmouseover"), "{html}");
    assert!(html.contains("styled"));
}

#[test]
fn colours_must_be_hex_to_reach_the_output() {
    let mut doc = CT_Document::new();
    let mut p = CT_P::new();
    let mut run = CT_R::new("shaded");
    run.properties = Some(CT_RPr {
        color: Some("red;} body{display:none} .x{".to_string()),
        shading: Some(CT_Shd {
            val: "clear".to_string(),
            color: None,
            fill: Some("\"><script>alert(1)</script>".to_string()),
        }),
        ..Default::default()
    });
    p.content.push(ParagraphChild::Run(run));
    doc.body.add_paragraph(p);

    let html = to_html_fragment(&input_from(doc, HashMap::new()), &HtmlOptions::default());

    assert!(!html.contains("<script"), "{html}");
    assert!(!html.contains("display:none"), "{html}");
    assert!(html.contains("shaded"));
}

#[test]
fn valid_colours_are_preserved() {
    let mut doc = CT_Document::new();
    let mut p = CT_P::new();
    let mut run = CT_R::new("red text");
    run.properties = Some(CT_RPr {
        color: Some("FF0000".to_string()),
        ..Default::default()
    });
    p.content.push(ParagraphChild::Run(run));
    doc.body.add_paragraph(p);

    let html = to_html_fragment(&input_from(doc, HashMap::new()), &HtmlOptions::default());

    assert!(html.contains("color:#FF0000"), "{html}");
}

#[test]
fn image_content_type_is_escaped() {
    let mut doc = CT_Document::new();
    let mut p = CT_P::new();
    p.add_run("caption");
    doc.body.add_paragraph(p);

    let mut input = input_from(doc, HashMap::new());
    input.images.insert(
        "rId1".to_string(),
        ImageData {
            data: vec![1, 2, 3],
            content_type: "image/png\"><script>alert(1)</script>".to_string(),
        },
    );

    let html = to_html_fragment(&input, &HtmlOptions::default());
    assert!(!html.contains("<script"), "{html}");
}

#[test]
fn text_content_is_escaped() {
    let mut doc = CT_Document::new();
    let mut p = CT_P::new();
    p.add_run("<script>alert('xss')</script> & more");
    doc.body.add_paragraph(p);

    let html = to_html_fragment(&input_from(doc, HashMap::new()), &HtmlOptions::default());

    assert!(!html.contains("<script"), "{html}");
    assert!(html.contains("&lt;script&gt;"), "{html}");
    assert!(html.contains("&amp; more"), "{html}");
}
