//! Footnote and endnote elements: `CT_Footnotes`, `CT_Footnote`.

use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};

use oxml_core::xml::{StrictXmlDocument, StrictXmlNode};

use crate::error::{OxmlError, Result};
use crate::namespace::W_NS;
use crate::text::CT_P;

/// A single footnote or endnote.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Footnote {
    /// Footnote ID (matches w:footnoteReference w:id in the document body).
    pub id: i32,
    /// Paragraphs making up the footnote content.
    pub paragraphs: Vec<CT_P>,
}

/// Collection of footnotes parsed from `word/footnotes.xml`.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Footnotes {
    pub footnotes: Vec<CT_Footnote>,
}

#[allow(non_snake_case)]
impl CT_Footnotes {
    pub fn new() -> Self {
        CT_Footnotes {
            footnotes: Vec::new(),
        }
    }

    /// Get a footnote by its ID.
    pub fn get_by_id(&self, id: i32) -> Option<&CT_Footnote> {
        self.footnotes.iter().find(|f| f.id == id)
    }

    /// Parse from XML bytes (the content of footnotes.xml).
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let root = StrictXmlDocument::parse(xml)?.into_root();
        if !root.is_named(Some(W_NS), "footnotes") && !root.is_named(Some(W_NS), "endnotes") {
            return Err(OxmlError::UnexpectedElement(
                "expected w:footnotes or w:endnotes".to_string(),
            ));
        }
        let mut footnotes = Vec::new();
        for child in root.children() {
            let StrictXmlNode::Element(note) = child else {
                continue;
            };
            if !note.is_named(Some(W_NS), "footnote") && !note.is_named(Some(W_NS), "endnote") {
                continue;
            }
            let id = note
                .attribute(Some(W_NS), "id")
                .map(str::parse)
                .transpose()?
                .unwrap_or(0);
            if id <= 0 {
                continue;
            }
            let paragraphs = note
                .children()
                .iter()
                .filter_map(StrictXmlNode::as_element)
                .filter(|element| element.is_named(Some(W_NS), "p"))
                .cloned()
                .map(CT_P::from_strict_xml)
                .collect::<Result<Vec<_>>>()?;
            footnotes.push(CT_Footnote { id, paragraphs });
        }

        Ok(CT_Footnotes { footnotes })
    }

    /// Serialize to XML bytes as footnotes.
    pub fn to_xml_footnotes(&self) -> Result<Vec<u8>> {
        self.to_xml_root("w:footnotes", "w:footnote")
    }

    /// Serialize to XML bytes as endnotes.
    pub fn to_xml_endnotes(&self) -> Result<Vec<u8>> {
        self.to_xml_root("w:endnotes", "w:endnote")
    }

    fn to_xml_root(&self, root_tag: &str, item_tag: &str) -> Result<Vec<u8>> {
        let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

        writer.write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))?;

        let mut start = BytesStart::new(root_tag);
        start.push_attribute(("xmlns:w", W_NS));
        start.push_attribute((
            "xmlns:r",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
        ));
        writer.write_event(Event::Start(start))?;

        let mut buf = itoa::Buffer::new();
        for footnote in &self.footnotes {
            let mut fn_start = BytesStart::new(item_tag);
            fn_start.push_attribute(("w:id", buf.format(footnote.id)));
            writer.write_event(Event::Start(fn_start))?;

            for p in &footnote.paragraphs {
                p.to_xml(&mut writer)?;
            }

            writer.write_event(Event::End(BytesEnd::new(item_tag)))?;
        }

        writer.write_event(Event::End(BytesEnd::new(root_tag)))?;

        Ok(writer.into_inner())
    }
}

impl Default for CT_Footnotes {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_footnotes_xml() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
        <w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
            <w:footnote w:id="0">
                <w:p><w:r><w:t>separator</w:t></w:r></w:p>
            </w:footnote>
            <w:footnote w:id="1">
                <w:p><w:r><w:t>First footnote text.</w:t></w:r></w:p>
            </w:footnote>
            <w:footnote w:id="2">
                <w:p><w:r><w:t>Second footnote.</w:t></w:r></w:p>
                <w:p><w:r><w:t>With two paragraphs.</w:t></w:r></w:p>
            </w:footnote>
        </w:footnotes>"#;

        let footnotes = CT_Footnotes::from_xml(xml).unwrap();
        // id=0 (separator) is skipped
        assert_eq!(footnotes.footnotes.len(), 2);
        assert_eq!(footnotes.footnotes[0].id, 1);
        assert_eq!(footnotes.footnotes[0].paragraphs.len(), 1);
        assert_eq!(
            footnotes.footnotes[0].paragraphs[0].text(),
            "First footnote text."
        );
        assert_eq!(footnotes.footnotes[1].id, 2);
        assert_eq!(footnotes.footnotes[1].paragraphs.len(), 2);
    }

    #[test]
    fn parse_endnotes_xml() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
        <w:endnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
            <w:endnote w:id="0">
                <w:p><w:r><w:t>separator</w:t></w:r></w:p>
            </w:endnote>
            <w:endnote w:id="1">
                <w:p><w:r><w:t>An endnote.</w:t></w:r></w:p>
            </w:endnote>
        </w:endnotes>"#;

        let endnotes = CT_Footnotes::from_xml(xml).unwrap();
        assert_eq!(endnotes.footnotes.len(), 1);
        assert_eq!(endnotes.footnotes[0].id, 1);
    }

    #[test]
    fn aliased_footnote_paragraph_properties_keep_root_scope() {
        let xml = format!(
            r#"<q:footnotes xmlns:q="{W_NS}" xmlns:ext="urn:producer"><ext:footnote ext:id="9"><ext:p/></ext:footnote><q:footnote q:id="1"><ext:p><ext:pPr><ext:jc ext:val="right"/></ext:pPr></ext:p><q:p><q:pPr><ext:jc ext:val="right"/><q:jc q:val="center"/></q:pPr><q:r><q:t>Note</q:t></q:r></q:p></q:footnote></q:footnotes>"#
        );
        let footnotes = CT_Footnotes::from_xml(xml.as_bytes()).unwrap();
        assert_eq!(footnotes.footnotes.len(), 1);
        assert_eq!(footnotes.footnotes[0].paragraphs.len(), 1);
        let paragraph = &footnotes.footnotes[0].paragraphs[0];
        assert_eq!(paragraph.text(), "Note");
        assert_eq!(
            paragraph.properties.as_ref().unwrap().jc,
            Some(crate::shared::ST_Jc::Center)
        );
    }

    #[test]
    fn default_namespace_footnote_properties_keep_root_scope() {
        let xml = format!(
            r#"<footnotes xmlns="{W_NS}" xmlns:w="{W_NS}" xmlns:ext="urn:producer"><ext:footnote ext:id="9"><ext:p/></ext:footnote><footnote w:id="1"><ext:p><ext:pPr><ext:jc ext:val="right"/></ext:pPr></ext:p><p><pPr><ext:jc ext:val="right"/><jc w:val="center"/></pPr><r><t>Note</t></r></p></footnote></footnotes>"#
        );
        let footnotes = CT_Footnotes::from_xml(xml.as_bytes()).unwrap();
        assert_eq!(footnotes.footnotes.len(), 1);
        assert_eq!(footnotes.footnotes[0].paragraphs.len(), 1);
        let paragraph = &footnotes.footnotes[0].paragraphs[0];
        assert_eq!(paragraph.text(), "Note");
        assert_eq!(
            paragraph.properties.as_ref().unwrap().jc,
            Some(crate::shared::ST_Jc::Center)
        );
    }

    #[test]
    fn get_footnote_by_id() {
        let footnotes = CT_Footnotes {
            footnotes: vec![
                CT_Footnote {
                    id: 1,
                    paragraphs: vec![],
                },
                CT_Footnote {
                    id: 2,
                    paragraphs: vec![],
                },
            ],
        };
        assert!(footnotes.get_by_id(1).is_some());
        assert!(footnotes.get_by_id(2).is_some());
        assert!(footnotes.get_by_id(3).is_none());
    }

    #[test]
    fn round_trip_footnotes() {
        let mut fn1_para = CT_P::new();
        fn1_para.add_run("First footnote.");

        let mut fn2_para = CT_P::new();
        fn2_para.add_run("Second footnote.");

        let footnotes = CT_Footnotes {
            footnotes: vec![
                CT_Footnote {
                    id: 1,
                    paragraphs: vec![fn1_para],
                },
                CT_Footnote {
                    id: 2,
                    paragraphs: vec![fn2_para],
                },
            ],
        };

        let xml = footnotes.to_xml_footnotes().unwrap();
        let parsed = CT_Footnotes::from_xml(&xml).unwrap();
        assert_eq!(parsed.footnotes.len(), 2);
        assert_eq!(parsed.footnotes[0].id, 1);
        assert_eq!(parsed.footnotes[0].paragraphs[0].text(), "First footnote.");
        assert_eq!(parsed.footnotes[1].id, 2);
        assert_eq!(parsed.footnotes[1].paragraphs[0].text(), "Second footnote.");
    }
}
