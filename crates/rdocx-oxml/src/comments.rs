//! Word comment elements: `CT_Comments` and `CT_Comment`.

use std::io::Write;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer, XmlVersion};

use crate::error::{OxmlError, Result};
use crate::namespace::{W_NS, matches_local_name};
use crate::numbering::word_prefixes_at;
use crate::properties::is_word_element;
use crate::raw_xml::{capture_element, capture_empty_element};
use crate::text::CT_P;

/// One entry in a Word comments part.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Comment {
    pub id: i32,
    pub author: Option<String>,
    pub date: Option<String>,
    pub initials: Option<String>,
    pub paragraphs: Vec<CT_P>,
    /// Unmodelled attributes, including producer namespace declarations.
    pub extra_attributes: Vec<(String, String)>,
    /// Unmodelled children retained at paragraph boundaries.
    pub extra_xml: Vec<(usize, Vec<u8>)>,
}

/// The typed contents of `word/comments.xml`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CT_Comments {
    pub comments: Vec<CT_Comment>,
    /// Namespace declarations and compatibility attributes from the root.
    pub root_attributes: Vec<(String, String)>,
    /// Unmodelled root children retained at comment boundaries.
    pub extra_xml: Vec<(usize, Vec<u8>)>,
}

impl CT_Comments {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a complete comments part.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        reader.config_mut().trim_text(true);
        let mut comments = Vec::new();
        let mut root_attributes = Vec::new();
        let mut extra_xml = Vec::new();
        let mut word_prefixes = Vec::new();
        let mut saw_root = false;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref element)) => {
                    let prefixes = word_prefixes_at(element, &word_prefixes)?;
                    let name = element.name();
                    if is_word_element(name.as_ref(), b"comments", &prefixes) {
                        root_attributes = capture_attributes(element, &[], &prefixes)?;
                        word_prefixes = prefixes;
                        saw_root = true;
                    } else if is_word_element(name.as_ref(), b"comment", &prefixes) {
                        comments.push(parse_comment(&mut reader, element, &prefixes)?);
                    } else if saw_root {
                        extra_xml.push((comments.len(), capture_element(&mut reader, element)?));
                    } else {
                        reader.read_to_end_into(name, &mut Vec::new())?;
                    }
                }
                Ok(Event::Empty(ref element)) => {
                    let prefixes = word_prefixes_at(element, &word_prefixes)?;
                    let name = element.name();
                    if is_word_element(name.as_ref(), b"comments", &prefixes) {
                        root_attributes = capture_attributes(element, &[], &prefixes)?;
                        saw_root = true;
                    } else if is_word_element(name.as_ref(), b"comment", &prefixes) {
                        comments.push(parse_empty_comment(element, &prefixes)?);
                    } else if saw_root {
                        extra_xml.push((comments.len(), capture_empty_element(element)?));
                    }
                }
                Ok(Event::Eof) => break,
                Err(error) => return Err(error.into()),
                _ => {}
            }
            buf.clear();
        }

        if !saw_root {
            return Err(OxmlError::MissingElement("comments root".to_owned()));
        }
        Ok(Self {
            comments,
            root_attributes,
            extra_xml,
        })
    }

    /// Serialize with fixed WordprocessingML prefixes and schema child order.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        writer.write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))?;

        let mut root = BytesStart::new("w:comments");
        root.push_attribute(("xmlns:w", W_NS));
        push_preserved_attributes(&mut root, &self.root_attributes, true);
        writer.write_event(Event::Start(root))?;

        write_raw_at(&mut writer, &self.extra_xml, 0)?;
        for (index, comment) in self.comments.iter().enumerate() {
            write_comment(&mut writer, comment)?;
            write_raw_at(&mut writer, &self.extra_xml, index + 1)?;
        }
        writer.write_event(Event::End(BytesEnd::new("w:comments")))?;
        Ok(writer.into_inner())
    }
}

fn parse_comment(
    reader: &mut Reader<&[u8]>,
    start: &BytesStart<'_>,
    prefixes: &[String],
) -> Result<CT_Comment> {
    let (id, author, date, initials, extra_attributes) = comment_attributes(start, prefixes)?;
    let mut paragraphs = Vec::new();
    let mut extra_xml = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref element)) => {
                let child_prefixes = word_prefixes_at(element, prefixes)?;
                if is_word_element(element.name().as_ref(), b"p", &child_prefixes) {
                    paragraphs.push(CT_P::from_xml_with_prefixes(reader, &child_prefixes)?);
                } else {
                    extra_xml.push((paragraphs.len(), capture_element(reader, element)?));
                }
            }
            Ok(Event::Empty(ref element)) => {
                let child_prefixes = word_prefixes_at(element, prefixes)?;
                if is_word_element(element.name().as_ref(), b"p", &child_prefixes) {
                    paragraphs.push(CT_P::new());
                } else {
                    extra_xml.push((paragraphs.len(), capture_empty_element(element)?));
                }
            }
            Ok(Event::End(ref element))
                if matches_local_name(element.name().as_ref(), b"comment") =>
            {
                break;
            }
            Ok(Event::Eof) => break,
            Err(error) => return Err(error.into()),
            _ => {}
        }
        buf.clear();
    }

    Ok(CT_Comment {
        id,
        author,
        date,
        initials,
        paragraphs,
        extra_attributes,
        extra_xml,
    })
}

fn parse_empty_comment(start: &BytesStart<'_>, prefixes: &[String]) -> Result<CT_Comment> {
    let (id, author, date, initials, extra_attributes) = comment_attributes(start, prefixes)?;
    Ok(CT_Comment {
        id,
        author,
        date,
        initials,
        paragraphs: Vec::new(),
        extra_attributes,
        extra_xml: Vec::new(),
    })
}

type CommentAttributes = (
    i32,
    Option<String>,
    Option<String>,
    Option<String>,
    Vec<(String, String)>,
);

fn comment_attributes(start: &BytesStart<'_>, prefixes: &[String]) -> Result<CommentAttributes> {
    let prefixes = word_prefixes_at(start, prefixes)?;
    let id = required_i32_attribute(start, b"id", &prefixes)?;
    let author = word_attribute(start, b"author", &prefixes)?;
    let date = word_attribute(start, b"date", &prefixes)?;
    let initials = word_attribute(start, b"initials", &prefixes)?;
    let extra_attributes =
        capture_attributes(start, &[b"id", b"author", b"date", b"initials"], &prefixes)?;
    Ok((id, author, date, initials, extra_attributes))
}

fn required_i32_attribute(
    start: &BytesStart<'_>,
    local: &[u8],
    prefixes: &[String],
) -> Result<i32> {
    let value = word_attribute(start, local, prefixes)?.ok_or_else(|| {
        OxmlError::MissingElement(format!(
            "comment {} attribute",
            String::from_utf8_lossy(local)
        ))
    })?;
    value.parse().map_err(OxmlError::from)
}

fn word_attribute(
    start: &BytesStart<'_>,
    local: &[u8],
    prefixes: &[String],
) -> Result<Option<String>> {
    for attribute in start.attributes() {
        let attribute = attribute?;
        if is_word_attribute(attribute.key.as_ref(), local, prefixes) {
            return Ok(Some(
                attribute
                    .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())?
                    .into_owned(),
            ));
        }
    }
    Ok(None)
}

fn is_word_attribute(key: &[u8], local: &[u8], prefixes: &[String]) -> bool {
    let Some(separator) = key.iter().position(|byte| *byte == b':') else {
        return false;
    };
    key.get(separator + 1..) == Some(local)
        && prefixes
            .iter()
            .any(|prefix| prefix.as_bytes() == &key[..separator])
}

fn capture_attributes(
    start: &BytesStart<'_>,
    modelled: &[&[u8]],
    prefixes: &[String],
) -> Result<Vec<(String, String)>> {
    let mut attributes = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute?;
        if modelled
            .iter()
            .any(|local| is_word_attribute(attribute.key.as_ref(), local, prefixes))
        {
            continue;
        }
        let name = std::str::from_utf8(attribute.key.as_ref())?.to_owned();
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())?
            .into_owned();
        attributes.push((name, value));
    }
    Ok(attributes)
}

fn write_comment<W: Write>(writer: &mut Writer<W>, comment: &CT_Comment) -> Result<()> {
    let mut start = BytesStart::new("w:comment");
    if let Some(author) = &comment.author {
        start.push_attribute(("w:author", author.as_str()));
    }
    if let Some(date) = &comment.date {
        start.push_attribute(("w:date", date.as_str()));
    }
    if let Some(initials) = &comment.initials {
        start.push_attribute(("w:initials", initials.as_str()));
    }
    let mut id = itoa::Buffer::new();
    start.push_attribute(("w:id", id.format(comment.id)));
    push_preserved_attributes(&mut start, &comment.extra_attributes, false);
    writer.write_event(Event::Start(start))?;

    write_raw_at(writer, &comment.extra_xml, 0)?;
    for (index, paragraph) in comment.paragraphs.iter().enumerate() {
        paragraph.to_xml(writer)?;
        write_raw_at(writer, &comment.extra_xml, index + 1)?;
    }
    writer.write_event(Event::End(BytesEnd::new("w:comment")))?;
    Ok(())
}

fn push_preserved_attributes(
    start: &mut BytesStart<'_>,
    attributes: &[(String, String)],
    root: bool,
) {
    for (name, value) in attributes {
        if root && name == "xmlns:w" {
            continue;
        }
        start.push_attribute((name.as_str(), value.as_str()));
    }
}

fn write_raw_at<W: Write>(
    writer: &mut Writer<W>,
    extra_xml: &[(usize, Vec<u8>)],
    position: usize,
) -> Result<()> {
    for (at, raw) in extra_xml {
        if *at == position {
            writer.get_mut().write_all(raw)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comments_accept_aliases_and_write_fixed_prefixes_in_schema_order() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<x:comments xmlns:x="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:ext="urn:producer" ext:root="kept"><ext:before/><x:comment ext:item="kept" x:id="7" x:author="Ada" x:date="2026-08-17T10:00:00Z" x:initials="AL"><ext:beforeParagraph/><x:p><x:r><x:t>Review this</x:t></x:r></x:p><ext:afterParagraph/></x:comment><ext:after/></x:comments>"#;

        let comments = CT_Comments::from_xml(xml).expect("aliased comments should parse");
        assert_eq!(comments.comments.len(), 1);
        assert_eq!(comments.comments[0].id, 7);
        assert_eq!(comments.comments[0].author.as_deref(), Some("Ada"));
        assert_eq!(comments.comments[0].initials.as_deref(), Some("AL"));
        assert_eq!(comments.comments[0].paragraphs[0].text(), "Review this");

        let output = String::from_utf8(comments.to_xml().expect("comments should serialize"))
            .expect("comments XML should be UTF-8");
        assert!(output.contains("<w:comments"));
        assert!(output.contains(
            r#"<w:comment w:author="Ada" w:date="2026-08-17T10:00:00Z" w:initials="AL" w:id="7""#
        ));
        assert!(output.contains("<ext:beforeParagraph/><w:p>"));
        assert!(output.contains("</w:p><ext:afterParagraph/>"));
        assert!(output.contains("<ext:before/><w:comment"));
        assert!(output.contains("</w:comment><ext:after/>"));
        assert!(output.contains(r#"ext:root="kept""#));
        assert!(output.contains(r#"ext:item="kept""#));
    }

    #[test]
    fn malformed_comment_id_is_rejected() {
        let xml = br#"<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:comment w:id="not-a-number"><w:p/></w:comment></w:comments>"#;
        assert!(CT_Comments::from_xml(xml).is_err());
    }
}
