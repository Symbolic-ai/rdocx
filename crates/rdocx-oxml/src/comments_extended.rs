//! Word comments-extended metadata for replies and resolved threads.

use std::io::Write;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer, XmlVersion};

use crate::error::{OxmlError, Result};
use crate::raw_xml::{capture_element, capture_empty_element};

/// Namespace used by the Microsoft comments-extended extension.
pub const W15_NS: &str = "http://schemas.microsoft.com/office/word/2012/wordml";

/// Thread metadata keyed by the first paragraph of a comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CT_CommentEx {
    pub para_id: String,
    pub para_id_parent: Option<String>,
    pub done: Option<bool>,
    /// Unmodelled attributes retained on the entry.
    pub extra_attributes: Vec<(String, String)>,
}

/// The typed contents of `word/commentsExtended.xml`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CT_CommentsEx {
    pub comments: Vec<CT_CommentEx>,
    /// Namespace declarations and producer attributes from the root.
    pub root_attributes: Vec<(String, String)>,
    /// Unmodelled root children retained at comment boundaries.
    pub extra_xml: Vec<(usize, Vec<u8>)>,
}

impl CT_CommentsEx {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a complete comments-extended part.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        reader.config_mut().trim_text(true);
        let mut comments = Vec::new();
        let mut root_attributes = Vec::new();
        let mut extra_xml = Vec::new();
        let mut prefixes = Vec::new();
        let mut saw_root = false;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref element)) => {
                    let child_prefixes = extension_prefixes_at(element, &prefixes)?;
                    if is_extension_element(element.name().as_ref(), b"commentsEx", &child_prefixes)
                    {
                        root_attributes = capture_attributes(element, &[], &child_prefixes)?;
                        prefixes = child_prefixes;
                        saw_root = true;
                    } else if is_extension_element(
                        element.name().as_ref(),
                        b"commentEx",
                        &child_prefixes,
                    ) {
                        comments.push(parse_comment_ex(element, &child_prefixes)?);
                        reader.read_to_end_into(element.name(), &mut Vec::new())?;
                    } else if saw_root {
                        extra_xml.push((comments.len(), capture_element(&mut reader, element)?));
                    } else {
                        reader.read_to_end_into(element.name(), &mut Vec::new())?;
                    }
                }
                Ok(Event::Empty(ref element)) => {
                    let child_prefixes = extension_prefixes_at(element, &prefixes)?;
                    if is_extension_element(element.name().as_ref(), b"commentsEx", &child_prefixes)
                    {
                        root_attributes = capture_attributes(element, &[], &child_prefixes)?;
                        saw_root = true;
                    } else if is_extension_element(
                        element.name().as_ref(),
                        b"commentEx",
                        &child_prefixes,
                    ) {
                        comments.push(parse_comment_ex(element, &child_prefixes)?);
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
            return Err(OxmlError::MissingElement("commentsEx root".to_owned()));
        }
        Ok(Self {
            comments,
            root_attributes,
            extra_xml,
        })
    }

    /// Serialize with fixed extension prefixes and schema child order.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        writer.write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))?;
        let mut root = BytesStart::new("w15:commentsEx");
        root.push_attribute(("xmlns:w15", W15_NS));
        push_preserved_attributes(&mut root, &self.root_attributes, true);
        writer.write_event(Event::Start(root))?;

        let root_prefixes =
            preserved_extension_prefixes(&self.root_attributes, &["w15".to_owned()]);
        write_raw_at(&mut writer, &self.extra_xml, 0)?;
        for (index, comment) in self.comments.iter().enumerate() {
            write_comment_ex(&mut writer, comment, &root_prefixes)?;
            write_raw_at(&mut writer, &self.extra_xml, index + 1)?;
        }
        writer.write_event(Event::End(BytesEnd::new("w15:commentsEx")))?;
        Ok(writer.into_inner())
    }
}

fn parse_comment_ex(element: &BytesStart<'_>, prefixes: &[String]) -> Result<CT_CommentEx> {
    let para_id = extension_attribute(element, b"paraId", prefixes)?
        .ok_or_else(|| OxmlError::MissingElement("commentEx paraId attribute".to_owned()))?;
    let para_id_parent = extension_attribute(element, b"paraIdParent", prefixes)?;
    let done_value = extension_attribute(element, b"done", prefixes)?;
    let done = done_value.as_deref().and_then(|value| match value {
        "1" | "true" | "on" => Some(true),
        "0" | "false" | "off" => Some(false),
        _ => None,
    });
    let modelled: &[&[u8]] = if done_value.is_none() || done.is_some() {
        &[b"paraId", b"paraIdParent", b"done"]
    } else {
        &[b"paraId", b"paraIdParent"]
    };
    let extra_attributes = capture_attributes(element, modelled, prefixes)?;
    Ok(CT_CommentEx {
        para_id,
        para_id_parent,
        done,
        extra_attributes,
    })
}

fn write_comment_ex<W: Write>(
    writer: &mut Writer<W>,
    comment: &CT_CommentEx,
    inherited_prefixes: &[String],
) -> Result<()> {
    let mut element = BytesStart::new("w15:commentEx");
    element.push_attribute(("w15:paraId", comment.para_id.as_str()));
    if let Some(parent) = &comment.para_id_parent {
        element.push_attribute(("w15:paraIdParent", parent.as_str()));
    }
    if let Some(done) = comment.done {
        element.push_attribute(("w15:done", if done { "1" } else { "0" }));
    }
    let prefixes = preserved_extension_prefixes(&comment.extra_attributes, inherited_prefixes);
    for (name, value) in &comment.extra_attributes {
        if comment.done.is_some() && is_extension_attribute(name.as_bytes(), b"done", &prefixes) {
            continue;
        }
        element.push_attribute((name.as_str(), value.as_str()));
    }
    writer.write_event(Event::Empty(element))?;
    Ok(())
}

fn preserved_extension_prefixes(
    attributes: &[(String, String)],
    inherited: &[String],
) -> Vec<String> {
    let mut prefixes = inherited.to_vec();
    for (name, value) in attributes {
        let prefix = if name == "xmlns" {
            Some("")
        } else {
            name.strip_prefix("xmlns:")
        };
        let Some(prefix) = prefix else {
            continue;
        };
        prefixes.retain(|candidate| candidate != prefix);
        if value == W15_NS {
            prefixes.push(prefix.to_owned());
        }
    }
    prefixes
}

fn extension_prefixes_at(start: &BytesStart<'_>, inherited: &[String]) -> Result<Vec<String>> {
    let mut prefixes = inherited.to_vec();
    for attribute in start.attributes() {
        let attribute = attribute?;
        let name = attribute.key.as_ref();
        let prefix = if name == b"xmlns" {
            b"".as_slice()
        } else if let Some(prefix) = name.strip_prefix(b"xmlns:") {
            prefix
        } else {
            continue;
        };
        let prefix = std::str::from_utf8(prefix)?.to_owned();
        prefixes.retain(|candidate| candidate != &prefix);
        let value =
            attribute.decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())?;
        if value.as_ref() == W15_NS {
            prefixes.push(prefix);
        }
    }
    Ok(prefixes)
}

fn is_extension_element(name: &[u8], local: &[u8], prefixes: &[String]) -> bool {
    let qualified_local = name.rsplit(|byte| *byte == b':').next().unwrap_or(name);
    if qualified_local != local {
        return false;
    }
    match name.iter().position(|byte| *byte == b':') {
        Some(separator) => prefixes
            .iter()
            .any(|prefix| prefix.as_bytes() == &name[..separator]),
        None => prefixes.iter().any(String::is_empty),
    }
}

fn extension_attribute(
    start: &BytesStart<'_>,
    local: &[u8],
    prefixes: &[String],
) -> Result<Option<String>> {
    for attribute in start.attributes() {
        let attribute = attribute?;
        if is_extension_attribute(attribute.key.as_ref(), local, prefixes) {
            return Ok(Some(
                attribute
                    .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())?
                    .into_owned(),
            ));
        }
    }
    Ok(None)
}

fn is_extension_attribute(key: &[u8], local: &[u8], prefixes: &[String]) -> bool {
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
            .any(|local| is_extension_attribute(attribute.key.as_ref(), local, prefixes))
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

fn push_preserved_attributes(
    start: &mut BytesStart<'_>,
    attributes: &[(String, String)],
    root: bool,
) {
    for (name, value) in attributes {
        if root && name == "xmlns:w15" {
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
    fn aliases_round_trip_with_fixed_prefix_and_preserved_children() {
        let xml = br#"<x:commentsEx xmlns:x="http://schemas.microsoft.com/office/word/2012/wordml" xmlns:ext="urn:producer" ext:root="kept"><ext:before/><x:commentEx x:paraId="0000000A" x:paraIdParent="00000009" x:done="true" ext:item="kept"/><ext:after/></x:commentsEx>"#;
        let parsed = CT_CommentsEx::from_xml(xml).unwrap();
        assert_eq!(parsed.comments.len(), 1);
        assert_eq!(parsed.comments[0].para_id, "0000000A");
        assert_eq!(
            parsed.comments[0].para_id_parent.as_deref(),
            Some("00000009")
        );
        assert_eq!(parsed.comments[0].done, Some(true));

        let output = String::from_utf8(parsed.to_xml().unwrap()).unwrap();
        assert!(output.contains("<w15:commentsEx"));
        assert!(output.contains("w15:paraId=\"0000000A\""));
        assert!(output.contains("w15:paraIdParent=\"00000009\""));
        assert!(output.contains("w15:done=\"1\""));
        assert!(output.contains("<ext:before/><w15:commentEx"));
        assert!(output.contains("/><ext:after/>"));
        assert!(output.contains("ext:item=\"kept\""));
    }

    #[test]
    fn unknown_done_value_is_preserved_as_unmodelled_metadata() {
        let xml = br#"<w15:commentsEx xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml"><w15:commentEx w15:paraId="00000001" w15:done="producer-value"/></w15:commentsEx>"#;
        let parsed = CT_CommentsEx::from_xml(xml).unwrap();
        assert_eq!(parsed.comments[0].done, None);
        let output = String::from_utf8(parsed.to_xml().unwrap()).unwrap();
        assert!(output.contains("w15:done=\"producer-value\""));
    }

    #[test]
    fn typed_done_replaces_a_preserved_unknown_done_value() {
        let xml = br#"<x:commentsEx xmlns:x="http://schemas.microsoft.com/office/word/2012/wordml"><x:commentEx x:paraId="00000001" x:done="producer-value"/></x:commentsEx>"#;
        let mut parsed = CT_CommentsEx::from_xml(xml).unwrap();
        parsed.comments[0].done = Some(true);

        let output = String::from_utf8(parsed.to_xml().unwrap()).unwrap();
        assert_eq!(output.matches(":done=").count(), 1);
        assert!(output.contains("w15:done=\"1\""));
        assert!(!output.contains("producer-value"));
    }
}
