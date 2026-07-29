//! Helpers for turning quick-xml text events into plain Rust strings.
//!
//! Since quick-xml 0.41 a character or general entity reference (`&amp;`,
//! `&#65;`) is reported as its own [`Event::GeneralRef`] rather than being
//! folded into the surrounding [`Event::Text`]. Any loop that accumulates the
//! text of an element therefore has to handle both events, or entities are
//! silently dropped from the result.
//!
//! [`Event::GeneralRef`]: quick_xml::events::Event::GeneralRef
//! [`Event::Text`]: quick_xml::events::Event::Text

use quick_xml::Reader;
use quick_xml::escape::unescape;
use quick_xml::events::{BytesRef, BytesText, Event};
use quick_xml::name::QName;

/// Decode a [`BytesText`] and resolve any XML entities it still contains.
///
/// Use this for spans produced by [`Reader::read_text`], which returns the raw
/// markup between the tags — entities in it are *not* pre-resolved.
pub(crate) fn decode_escaped(text: &BytesText<'_>) -> String {
    let Ok(raw) = text.decode() else {
        return String::new();
    };
    match unescape(&raw) {
        Ok(unescaped) => unescaped.into_owned(),
        // Malformed or unknown entity: keep the raw text rather than losing it.
        Err(_) => raw.into_owned(),
    }
}

/// Decode an [`Event::Text`] payload, which quick-xml has already stripped of
/// entity references (those arrive separately as [`Event::GeneralRef`]).
///
/// [`Event::GeneralRef`]: quick_xml::events::Event::GeneralRef
/// [`Event::Text`]: quick_xml::events::Event::Text
pub(crate) fn decode_plain(text: &BytesText<'_>) -> String {
    text.decode().map(|c| c.into_owned()).unwrap_or_default()
}

/// Resolve a single entity reference event to the text it stands for.
///
/// Handles numeric references (`&#65;`, `&#x41;`) and the five XML predefined
/// entities. An unresolvable reference is reproduced verbatim (`&name;`) so it
/// survives a round trip instead of vanishing.
pub(crate) fn resolve_entity(entity: &BytesRef<'_>) -> String {
    let Ok(name) = entity.decode() else {
        return String::new();
    };
    match unescape(&format!("&{name};")) {
        Ok(resolved) => resolved.into_owned(),
        Err(_) => format!("&{name};"),
    }
}

/// Read the full text content of the element that `start_name` opened,
/// resolving entity references.
///
/// This is the event-loop counterpart to [`decode_escaped`]: it consumes events
/// up to the matching end tag, concatenating [`Event::Text`], [`Event::CData`]
/// and [`Event::GeneralRef`] payloads.
///
/// [`Event::CData`]: quick_xml::events::Event::CData
/// [`Event::GeneralRef`]: quick_xml::events::Event::GeneralRef
/// [`Event::Text`]: quick_xml::events::Event::Text
pub(crate) fn read_element_text(reader: &mut Reader<&[u8]>, start_name: QName<'_>) -> String {
    let end = start_name.as_ref().to_vec();
    let mut out = String::new();
    let mut buf = Vec::new();
    let mut depth = 1u32;

    // An entity splits the value into several Text events, and trimming each of
    // them individually would eat the spaces on either side of the entity —
    // "Title &amp; Co." would come back as "Title&Co.". Read untrimmed and
    // restore the caller's setting afterwards.
    let (trim_start, trim_end) = {
        let config = reader.config_mut();
        let previous = (config.trim_text_start, config.trim_text_end);
        config.trim_text(false);
        previous
    };

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == end {
                    depth += 1;
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == end {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
            }
            Ok(Event::Text(ref e)) => out.push_str(&decode_plain(e)),
            Ok(Event::CData(ref e)) => {
                if let Ok(decoded) = e.decode() {
                    out.push_str(&decoded);
                }
            }
            Ok(Event::GeneralRef(ref e)) => out.push_str(&resolve_entity(e)),
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    let config = reader.config_mut();
    config.trim_text_start = trim_start;
    config.trim_text_end = trim_end;

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_of(xml: &str) -> String {
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"t" => {
                    return read_element_text(&mut reader, e.name());
                }
                Ok(Event::Eof) => return String::new(),
                _ => {}
            }
        }
    }

    #[test]
    fn entities_survive_text_accumulation() {
        assert_eq!(text_of("<t>a &amp; b</t>"), "a & b");
        assert_eq!(text_of("<t>&lt;tag&gt;</t>"), "<tag>");
        assert_eq!(text_of("<t>&#65;&#x42;</t>"), "AB");
        assert_eq!(text_of("<t>plain</t>"), "plain");
    }

    #[test]
    fn whitespace_around_entities_is_kept_even_when_trimming() {
        let mut reader = Reader::from_str("<t>Title &amp; Co. &lt;tagged&gt;</t>");
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    assert_eq!(
                        read_element_text(&mut reader, e.name()),
                        "Title & Co. <tagged>"
                    );
                    // The caller's trim setting must be restored.
                    assert!(reader.config().trim_text_start);
                    return;
                }
                Ok(Event::Eof) => panic!("no start tag"),
                _ => {}
            }
        }
    }

    #[test]
    fn unknown_entity_is_preserved_verbatim() {
        assert_eq!(text_of("<t>a &nbsp; b</t>"), "a &nbsp; b");
    }

    #[test]
    fn read_text_span_is_unescaped() {
        let xml = "<t>a &amp; b</t>";
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let span = reader.read_text(e.name()).unwrap();
                    assert_eq!(decode_escaped(&span), "a & b");
                    return;
                }
                Ok(Event::Eof) => panic!("no start tag"),
                _ => {}
            }
        }
    }
}
