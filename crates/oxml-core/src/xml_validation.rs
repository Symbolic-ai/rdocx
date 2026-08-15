//! Strict validation for complete OOXML XML parts.

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::{OxmlError, Result};

/// Validate that `xml` is one complete, well-formed XML document.
///
/// `quick-xml` is intentionally a streaming tokenizer and will otherwise
/// accept multiple top-level elements. Attribute iteration is also required
/// to surface duplicate and malformed attributes before semantic parsers run.
pub fn validate_document(xml: &[u8]) -> Result<()> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut depth = 0usize;
    let mut saw_root = false;
    let mut root_closed = false;
    let mut saw_declaration = false;
    let mut saw_doctype = false;
    let mut buffer = Vec::new();

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element)) => {
                validate_attributes(&element)?;
                if depth == 0 {
                    begin_root(&mut saw_root, root_closed)?;
                }
                depth = depth.checked_add(1).ok_or_else(|| {
                    OxmlError::InvalidValue("XML nesting depth overflow".to_string())
                })?;
            }
            Ok(Event::Empty(element)) => {
                validate_attributes(&element)?;
                if depth == 0 {
                    begin_root(&mut saw_root, root_closed)?;
                    root_closed = true;
                }
            }
            Ok(Event::End(_)) => {
                depth = depth.checked_sub(1).ok_or_else(|| {
                    OxmlError::UnexpectedElement("closing element outside root".to_string())
                })?;
                if depth == 0 {
                    root_closed = true;
                }
            }
            Ok(Event::Text(text)) if depth == 0 => {
                if !text.as_ref().iter().all(u8::is_ascii_whitespace) {
                    return Err(OxmlError::UnexpectedElement(
                        "non-whitespace text outside root".to_string(),
                    ));
                }
            }
            Ok(Event::CData(_)) if depth == 0 => {
                return Err(OxmlError::UnexpectedElement(
                    "CDATA outside root".to_string(),
                ));
            }
            Ok(Event::Decl(_)) => {
                if depth != 0 || saw_root || saw_declaration {
                    return Err(OxmlError::UnexpectedElement(
                        "misplaced XML declaration".to_string(),
                    ));
                }
                saw_declaration = true;
            }
            Ok(Event::DocType(_)) => {
                if depth != 0 || saw_root || saw_doctype {
                    return Err(OxmlError::UnexpectedElement(
                        "misplaced document type".to_string(),
                    ));
                }
                saw_doctype = true;
            }
            Ok(Event::Eof) => break,
            Ok(Event::GeneralRef(_)) if depth == 0 => {
                return Err(OxmlError::UnexpectedElement(
                    "entity reference outside root".to_string(),
                ));
            }
            Ok(_) => {}
            Err(error) => return Err(error.into()),
        }
        buffer.clear();
    }

    if !saw_root {
        return Err(OxmlError::MissingElement("XML root".to_string()));
    }
    if depth != 0 || !root_closed {
        return Err(OxmlError::MissingElement("closing XML root".to_string()));
    }
    Ok(())
}

fn validate_attributes(element: &BytesStart<'_>) -> Result<()> {
    for attribute in element.attributes() {
        attribute?;
    }
    Ok(())
}

fn begin_root(saw_root: &mut bool, root_closed: bool) -> Result<()> {
    if *saw_root || root_closed {
        return Err(OxmlError::UnexpectedElement(
            "multiple XML roots".to_string(),
        ));
    }
    *saw_root = true;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_multiple_roots_and_duplicate_attributes() {
        for xml in [
            br#"<root/><second/>"#.as_slice(),
            br#"<root value="one" value="two"/>"#.as_slice(),
        ] {
            assert!(validate_document(xml).is_err());
        }
    }

    #[test]
    fn accepts_legal_prolog_and_epilog() {
        validate_document(br#"<?xml version="1.0"?><!--before--><root></root><?after ok?>"#)
            .unwrap();
    }
}
