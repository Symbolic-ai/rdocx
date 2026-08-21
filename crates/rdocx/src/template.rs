//! Scalar template parsing and JSON evaluation.

use std::collections::BTreeMap;

use quick_xml::Reader;
use quick_xml::events::Event;
use rdocx_oxml::content_control::{CT_Sdt, SdtContent};
use rdocx_oxml::document::{BodyContent, CT_Document};
use rdocx_oxml::header_footer::CT_HdrFtr;
use rdocx_oxml::namespace::matches_local_name;
use rdocx_oxml::table::{CT_Row, CT_Tbl, CT_Tc, CellContent};
use rdocx_oxml::text::{CT_P, RunContent};
use serde_json::Value;

use crate::document::Document;
use crate::error::{Error, Result};

#[derive(Debug)]
struct Replacement {
    literal: String,
    value: String,
    occurrences: usize,
}

pub(crate) fn render(document: &mut Document, data: &Value) -> Result<usize> {
    let mut candidate = document.clone_for_template();
    let sources = candidate.template_sources()?;
    let replacements = resolve_replacements(&sources, data)?;
    let expected = replacements
        .iter()
        .map(|replacement| replacement.occurrences)
        .sum();
    if expected == 0 {
        return Ok(0);
    }

    let sentinels = collision_free_sentinels(&replacements, &sources);
    let first_pass = replacements
        .iter()
        .zip(&sentinels)
        .map(|(replacement, sentinel)| (replacement.literal.as_str(), sentinel.as_str()))
        .collect::<Vec<_>>();
    let replaced = candidate.apply_template_pairs(&first_pass);
    if replaced != expected {
        return Err(Error::Other(format!(
            "template changed while staging: discovered {expected} tags but replaced {replaced}"
        )));
    }

    let second_pass = sentinels
        .iter()
        .zip(&replacements)
        .map(|(sentinel, replacement)| (sentinel.as_str(), replacement.value.as_str()))
        .collect::<Vec<_>>();
    let restored = candidate.apply_template_pairs(&second_pass);
    if restored != expected {
        return Err(Error::Other(format!(
            "template staging lost replacements: expected {expected} sentinels but found {restored}"
        )));
    }

    candidate.document.to_xml()?;
    document.commit_template(candidate);
    Ok(expected)
}

fn resolve_replacements(sources: &[String], data: &Value) -> Result<Vec<Replacement>> {
    let mut resolved = BTreeMap::<String, (String, usize)>::new();
    for source in sources {
        scan_source(source, data, &mut resolved)?;
    }
    Ok(resolved
        .into_iter()
        .map(|(literal, (value, occurrences))| Replacement {
            literal,
            value,
            occurrences,
        })
        .collect())
}

fn scan_source(
    source: &str,
    data: &Value,
    resolved: &mut BTreeMap<String, (String, usize)>,
) -> Result<()> {
    let mut cursor = 0;
    while cursor < source.len() {
        let marker =
            next_marker(&source[cursor..]).map(|(offset, marker)| (cursor + offset, marker));
        let Some((start, marker)) = marker else {
            break;
        };
        match marker {
            "{{" => {
                let content_start = start + 2;
                let Some(relative_end) = source[content_start..].find("}}") else {
                    return Err(template_error("unclosed scalar tag"));
                };
                let end = content_start + relative_end;
                let inner = &source[content_start..end];
                if inner.contains("{{") || inner.contains("{%") || inner.contains("%}") {
                    return Err(template_error("nested or mismatched scalar tag"));
                }
                let path = parse_path(inner)?;
                let value = resolve_path(data, &path)?;
                let literal = source[start..end + 2].to_owned();
                let entry = resolved
                    .entry(literal)
                    .or_insert_with(|| (value.clone(), 0));
                entry.1 += 1;
                cursor = end + 2;
            }
            "{%" => {
                let content_start = start + 2;
                let Some(relative_end) = source[content_start..].find("%}") else {
                    return Err(template_error("unclosed control tag"));
                };
                let end = content_start + relative_end;
                let inner = source[content_start..end].trim();
                if inner.is_empty()
                    || inner.contains("{{")
                    || inner.contains("{%")
                    || inner.contains("}}")
                {
                    return Err(template_error("malformed control tag"));
                }
                cursor = end + 2;
            }
            "}}" | "%}" => return Err(template_error("closing template delimiter without opener")),
            _ => unreachable!("next_marker returns known markers"),
        }
    }
    Ok(())
}

fn next_marker(source: &str) -> Option<(usize, &'static str)> {
    ["{{", "{%", "}}", "%}"]
        .into_iter()
        .filter_map(|marker| source.find(marker).map(|offset| (offset, marker)))
        .min_by_key(|(offset, _)| *offset)
}

fn parse_path(inner: &str) -> Result<Vec<&str>> {
    let path = inner.trim();
    if path.is_empty() {
        return Err(template_error("empty scalar path"));
    }
    let components = path.split('.').collect::<Vec<_>>();
    if components.iter().any(|component| {
        component.is_empty()
            || component.chars().any(|character| {
                character.is_whitespace() || matches!(character, '{' | '}' | '%' | '\0')
            })
    }) {
        return Err(template_error("invalid dotted scalar path"));
    }
    Ok(components)
}

fn resolve_path(data: &Value, path: &[&str]) -> Result<String> {
    let display_path = path.join(".");
    let mut value = data;
    for component in path {
        let Value::Object(object) = value else {
            return Err(template_error(&format!(
                "path `{display_path}` crosses a non-object value"
            )));
        };
        let Some(next) = object.get(*component) else {
            return Err(template_error(&format!("missing path `{display_path}`")));
        };
        value = next;
    }
    match value {
        Value::String(value) => Ok(value.clone()),
        Value::Number(value) => Ok(value.to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Null => Ok(String::new()),
        Value::Array(_) | Value::Object(_) => Err(template_error(&format!(
            "path `{display_path}` does not resolve to a scalar value"
        ))),
    }
}

fn template_error(message: &str) -> Error {
    Error::Other(format!("invalid template: {message}"))
}

fn collision_free_sentinels(replacements: &[Replacement], sources: &[String]) -> Vec<String> {
    let mut nonce = 0usize;
    replacements
        .iter()
        .map(|_| {
            loop {
                let sentinel = format!("\u{e000}rdocx-template-{nonce}\u{e001}");
                nonce += 1;
                let collides = sources.iter().any(|source| source.contains(&sentinel))
                    || replacements
                        .iter()
                        .any(|replacement| replacement.value.contains(&sentinel));
                if !collides {
                    break sentinel;
                }
            }
        })
        .collect()
}

pub(crate) fn body_sources(document: &CT_Document) -> Vec<String> {
    let mut sources = Vec::new();
    for content in &document.body.content {
        match content {
            BodyContent::Paragraph(paragraph) => sources.push(paragraph_text(paragraph)),
            BodyContent::Table(table) => collect_table(table, &mut sources),
            BodyContent::ContentControl(_) | BodyContent::RawXml(_) => {}
        }
    }
    sources
}

pub(crate) fn header_footer_sources(header_footer: &CT_HdrFtr) -> Vec<String> {
    header_footer
        .paragraphs
        .iter()
        .map(paragraph_text)
        .collect()
}

fn paragraph_text(paragraph: &CT_P) -> String {
    paragraph
        .runs
        .iter()
        .flat_map(|run| &run.content)
        .filter_map(|content| match content {
            RunContent::Text(text) => Some(text.text.as_str()),
            _ => None,
        })
        .collect()
}

fn collect_table(table: &CT_Tbl, sources: &mut Vec<String>) {
    for (_, _, control) in &table.content_controls {
        collect_control(control, sources);
    }
    for row in &table.rows {
        collect_row(row, sources);
    }
}

fn collect_control(control: &CT_Sdt, sources: &mut Vec<String>) {
    for content in &control.content {
        match content {
            SdtContent::Paragraph(paragraph) => sources.push(paragraph_text(paragraph)),
            SdtContent::Table(table) => collect_table(table, sources),
            SdtContent::Row(row) => collect_row(row, sources),
            SdtContent::Cell(cell) => collect_cell(cell, sources),
            SdtContent::ContentControl(nested) => collect_control(nested, sources),
            SdtContent::Run(_) | SdtContent::RawXml(_) => {}
        }
    }
}

fn collect_row(row: &CT_Row, sources: &mut Vec<String>) {
    for (_, _, control) in &row.content_controls {
        collect_control(control, sources);
    }
    for cell in &row.cells {
        collect_cell(cell, sources);
    }
}

fn collect_cell(cell: &CT_Tc, sources: &mut Vec<String>) {
    for content in &cell.content {
        match content {
            CellContent::Paragraph(paragraph) => sources.push(paragraph_text(paragraph)),
            CellContent::Table(table) => collect_table(table, sources),
            CellContent::ContentControl(control) => collect_control(control, sources),
        }
    }
}

pub(crate) fn text_box_sources(xml: &[u8]) -> Result<Vec<String>> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut sources = Vec::new();
    let mut buffer = Vec::new();
    let mut in_text_box = false;

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(element))
                if matches_local_name(element.name().as_ref(), b"txbxContent") =>
            {
                in_text_box = true;
            }
            Ok(Event::Start(element))
                if in_text_box && matches_local_name(element.name().as_ref(), b"p") =>
            {
                let paragraph = CT_P::from_xml(&mut reader)?;
                sources.push(paragraph_text(&paragraph));
            }
            Ok(Event::Start(element)) if in_text_box => {
                reader
                    .read_to_end_into(element.name(), &mut Vec::new())
                    .map_err(template_xml_error)?;
            }
            Ok(Event::End(element))
                if in_text_box && matches_local_name(element.name().as_ref(), b"txbxContent") =>
            {
                in_text_box = false;
            }
            Ok(_) => {}
            Err(error) => return Err(template_xml_error(error)),
        }
        buffer.clear();
    }

    Ok(sources)
}

pub(crate) fn chart_sources(xml: &[u8]) -> Result<Vec<String>> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut sources = Vec::new();
    let mut buffer = Vec::new();

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(element))
                if matches_local_name(element.name().as_ref(), b"t")
                    || matches_local_name(element.name().as_ref(), b"v") =>
            {
                sources.push(oxml_core::xml_text::read_element_text(
                    &mut reader,
                    element.name(),
                ));
            }
            Ok(_) => {}
            Err(error) => return Err(template_xml_error(error)),
        }
        buffer.clear();
    }

    Ok(sources)
}

fn template_xml_error(error: quick_xml::Error) -> Error {
    Error::Other(format!("invalid template XML: {error}"))
}
