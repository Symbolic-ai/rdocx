use std::collections::HashMap;
use std::ops::Range;

use oxml_core::{OxmlError, Result};
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};

use crate::namespace::{NamespaceBindings, R_NS};

#[derive(Debug)]
struct Replacement {
    range: Range<usize>,
    escaped_value: String,
}

/// Rewrite mapped relationship ids inside an XML payload.
///
/// Only attributes in the office document relationships namespace whose
/// decoded values match `rId` followed by ASCII digits are candidates. The
/// original bytes are copied around the replaced attribute values so all
/// other syntax remains exact.
pub fn rewrite_rel_ids(raw: &[u8], map: &HashMap<String, String>) -> Result<Vec<u8>> {
    let mut reader = Reader::from_reader(raw);
    reader.config_mut().check_comments = true;
    let mut buffer = Vec::new();
    let mut scopes = vec![NamespaceBindings::default()];
    let mut replacements = Vec::new();
    let mut depth = 0usize;
    let mut roots = 0usize;
    let mut version = XmlVersion::Implicit1_0;
    let mut declaration_allowed = true;
    let mut seen_declaration = false;
    let mut seen_doctype = false;

    loop {
        let event_start = reader.buffer_position() as usize;
        match reader.read_event_into(&mut buffer)? {
            Event::Decl(declaration) => {
                if !declaration_allowed || seen_declaration {
                    return Err(invalid_xml("XML declaration is not first"));
                }
                version = declaration.xml_version()?;
                seen_declaration = true;
                declaration_allowed = false;
            }
            Event::Start(element) => {
                declaration_allowed = false;
                if depth == 0 {
                    roots += 1;
                    if roots > 1 {
                        return Err(invalid_xml("XML payload has more than one root"));
                    }
                }
                let scope = scopes
                    .last()
                    .expect("the document namespace frame always exists")
                    .with_start(&element)?;
                collect_replacements(
                    raw,
                    event_start,
                    &element,
                    &scope,
                    version,
                    map,
                    &mut replacements,
                )?;
                scopes.push(scope);
                depth += 1;
            }
            Event::Empty(element) => {
                declaration_allowed = false;
                if depth == 0 {
                    roots += 1;
                    if roots > 1 {
                        return Err(invalid_xml("XML payload has more than one root"));
                    }
                }
                let scope = scopes
                    .last()
                    .expect("the document namespace frame always exists")
                    .with_start(&element)?;
                collect_replacements(
                    raw,
                    event_start,
                    &element,
                    &scope,
                    version,
                    map,
                    &mut replacements,
                )?;
            }
            Event::End(_) => {
                declaration_allowed = false;
                if depth == 0 {
                    return Err(invalid_xml("XML payload has an unmatched closing tag"));
                }
                depth -= 1;
                scopes.pop();
            }
            Event::Text(text) => {
                declaration_allowed = false;
                let bytes: &[u8] = text.as_ref();
                if !bytes.iter().all(|byte| byte.is_ascii_whitespace()) && depth == 0 {
                    return Err(invalid_xml("text is not allowed outside the root"));
                }
            }
            Event::CData(_) | Event::GeneralRef(_) => {
                declaration_allowed = false;
                if depth == 0 {
                    return Err(invalid_xml("content is not allowed outside the root"));
                }
            }
            Event::DocType(_) => {
                declaration_allowed = false;
                if depth != 0 || roots != 0 || seen_doctype {
                    return Err(invalid_xml("document type is not in the prolog"));
                }
                seen_doctype = true;
            }
            Event::PI(_) | Event::Comment(_) => {
                declaration_allowed = false;
            }
            Event::Eof => {
                if depth != 0 {
                    return Err(invalid_xml("XML payload ended before its root closed"));
                }
                if roots != 1 {
                    return Err(invalid_xml("XML payload must contain exactly one root"));
                }
                break;
            }
        }
        buffer.clear();
    }

    splice_replacements(raw, replacements)
}

/// Collects every attribute value in the office relationship namespace.
pub fn relationship_ids(raw: &[u8]) -> Result<Vec<String>> {
    let mut reader = Reader::from_reader(raw);
    let mut buffer = Vec::new();
    let mut scopes = vec![NamespaceBindings::default()];
    let mut ids = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) => {
                let parent = scopes
                    .last()
                    .ok_or_else(|| invalid_xml("missing namespace scope"))?;
                let scope = parent.with_start(&element)?;
                collect_relationship_ids(&element, &scope, &mut ids)?;
                scopes.push(scope);
            }
            Event::Empty(element) => {
                let parent = scopes
                    .last()
                    .ok_or_else(|| invalid_xml("missing namespace scope"))?;
                let scope = parent.with_start(&element)?;
                collect_relationship_ids(&element, &scope, &mut ids)?;
            }
            Event::End(_) => {
                if scopes.len() == 1 {
                    return Err(invalid_xml("XML payload has an unmatched closing tag"));
                }
                scopes.pop();
            }
            Event::Eof => {
                if scopes.len() != 1 {
                    return Err(invalid_xml("XML payload ended before its root closed"));
                }
                return Ok(ids);
            }
            _ => {}
        }
        buffer.clear();
    }
}

fn collect_relationship_ids(
    element: &BytesStart<'_>,
    scope: &NamespaceBindings,
    ids: &mut Vec<String>,
) -> Result<()> {
    for attribute in element.attributes() {
        let attribute = attribute?;
        if scope.attribute_uri(attribute.key.as_ref()) == Some(R_NS) {
            let value = attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())?
                .into_owned();
            if !value.is_empty() {
                ids.push(value);
            }
        }
    }
    Ok(())
}

fn collect_replacements(
    raw: &[u8],
    event_start: usize,
    element: &BytesStart<'_>,
    scope: &NamespaceBindings,
    version: XmlVersion,
    map: &HashMap<String, String>,
    replacements: &mut Vec<Replacement>,
) -> Result<()> {
    if raw.get(event_start) != Some(&b'<') {
        return Err(invalid_xml("XML event position does not begin at markup"));
    }
    for attribute in element.attributes() {
        let attribute = attribute?;
        if scope.attribute_uri(attribute.key.as_ref()) != Some(R_NS) {
            continue;
        }
        let decoded = attribute.decoded_and_normalized_value(version, element.decoder())?;
        if !is_numeric_relationship_id(&decoded) {
            continue;
        }
        let Some(target) = map.get(decoded.as_ref()) else {
            continue;
        };
        let value = attribute.value.as_ref();
        let element_address = element.as_ptr() as usize;
        let value_address = value.as_ptr() as usize;
        let relative_start = value_address
            .checked_sub(element_address)
            .filter(|start| {
                start
                    .checked_add(value.len())
                    .is_some_and(|end| end <= element.len())
            })
            .ok_or_else(|| invalid_xml("attribute value is outside its start tag"))?;
        let start = event_start
            .checked_add(1)
            .and_then(|start| start.checked_add(relative_start))
            .ok_or_else(|| invalid_xml("attribute byte range overflowed"))?;
        let end = start
            .checked_add(value.len())
            .ok_or_else(|| invalid_xml("attribute byte range overflowed"))?;
        if raw.get(start..end) != Some(value) {
            return Err(invalid_xml("attribute byte range did not match the source"));
        }
        replacements.push(Replacement {
            range: start..end,
            escaped_value: quick_xml::escape::escape(target).into_owned(),
        });
    }
    Ok(())
}

fn splice_replacements(raw: &[u8], replacements: Vec<Replacement>) -> Result<Vec<u8>> {
    if replacements.is_empty() {
        return Ok(raw.to_vec());
    }
    let added_capacity = replacements.iter().fold(0usize, |capacity, replacement| {
        capacity.saturating_add(
            replacement
                .escaped_value
                .len()
                .saturating_sub(replacement.range.len()),
        )
    });
    let mut rewritten = Vec::with_capacity(raw.len().saturating_add(added_capacity));
    let mut copied_through = 0usize;
    for replacement in replacements {
        if replacement.range.start < copied_through || replacement.range.end > raw.len() {
            return Err(invalid_xml("relationship replacement ranges overlap"));
        }
        rewritten.extend_from_slice(&raw[copied_through..replacement.range.start]);
        rewritten.extend_from_slice(replacement.escaped_value.as_bytes());
        copied_through = replacement.range.end;
    }
    rewritten.extend_from_slice(&raw[copied_through..]);
    Ok(rewritten)
}

fn is_numeric_relationship_id(value: &str) -> bool {
    value.strip_prefix("rId").is_some_and(|digits| {
        !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
    })
}

fn invalid_xml(message: &str) -> OxmlError {
    OxmlError::InvalidValue(message.to_owned())
}
