//! Border and tab stop types for paragraph formatting.

use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use oxml_core::xml::{
    StrictXmlCompleteness, StrictXmlElement, StrictXmlNode, StrictXmlParsed,
    parse_empty_started_element, parse_reader_element,
};

use crate::error::{OxmlError, Result};
use crate::namespace::W_NS;
#[cfg(test)]
use crate::namespace::matches_local_name;
use crate::raw_xml::NamespaceContext;
use crate::shared::{ST_Border, ST_TabJc, ST_TabLeader};
use crate::units::Twips;

/// A single border edge (top, bottom, left, right, between).
#[derive(Debug, Clone, PartialEq)]
pub struct CT_BorderEdge {
    /// Border style
    pub val: ST_Border,
    /// Border width in eighths of a point
    pub sz: Option<u32>,
    /// Space between border and content in points
    pub space: Option<u32>,
    /// Border color as hex, e.g. "FF0000"
    pub color: Option<String>,
}

impl CT_BorderEdge {
    pub fn new(val: ST_Border) -> Self {
        CT_BorderEdge {
            val,
            sz: None,
            space: None,
            color: None,
        }
    }

    pub fn from_xml_attrs(e: &BytesStart) -> Result<Self> {
        let element = parse_empty_started_element(&NamespaceContext::default(), Some(W_NS), e)?;
        Ok(Self::from_strict_xml(element)?.value)
    }

    pub(crate) fn from_strict_xml(element: StrictXmlElement) -> Result<StrictXmlParsed<Self>> {
        element.parse(|cursor| {
            let val = cursor
                .take_attribute(Some(W_NS), "val")
                .map(|value| ST_Border::from_str(&value))
                .transpose()?
                .unwrap_or(ST_Border::None);
            let sz = cursor
                .take_attribute(Some(W_NS), "sz")
                .map(|value| value.parse())
                .transpose()?;
            let space = cursor
                .take_attribute(Some(W_NS), "space")
                .map(|value| value.parse())
                .transpose()?;
            let color = cursor.take_attribute(Some(W_NS), "color");
            Ok(Self {
                val,
                sz,
                space,
                color,
            })
        })
    }

    pub fn write_xml_attrs(&self, e: &mut BytesStart) {
        let mut buf = itoa::Buffer::new();
        e.push_attribute(("w:val", self.val.to_str()));
        if let Some(sz) = self.sz {
            e.push_attribute(("w:sz", buf.format(sz)));
        }
        if let Some(space) = self.space {
            e.push_attribute(("w:space", buf.format(space)));
        }
        if let Some(ref color) = self.color {
            e.push_attribute(("w:color", color.as_str()));
        }
    }

    /// Write this border edge as an empty element with the given tag name.
    pub fn to_xml<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        tag: &str,
    ) -> crate::error::Result<()> {
        let mut e = BytesStart::new(tag);
        self.write_xml_attrs(&mut e);
        writer.write_event(Event::Empty(e))?;
        Ok(())
    }
}

/// `CT_PBdr` — Paragraph borders (top, bottom, left, right, between, bar).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CT_PBdr {
    #[doc(hidden)]
    pub completeness: StrictXmlCompleteness,
    pub top: Option<CT_BorderEdge>,
    pub bottom: Option<CT_BorderEdge>,
    pub left: Option<CT_BorderEdge>,
    pub right: Option<CT_BorderEdge>,
    pub between: Option<CT_BorderEdge>,
    pub bar: Option<CT_BorderEdge>,
}

impl CT_PBdr {
    pub(crate) fn from_strict_xml(element: StrictXmlElement) -> Result<Self> {
        let mut descendants = Vec::new();
        let parsed = element.parse(|cursor| {
            let mut borders = Self::default();
            for index in 0..cursor.child_slots() {
                let Some(StrictXmlNode::Element(child)) = cursor.child(index) else {
                    continue;
                };
                let target = if child.is_named(Some(W_NS), "top") {
                    Some("top")
                } else if child.is_named(Some(W_NS), "bottom") {
                    Some("bottom")
                } else if child.is_named(Some(W_NS), "left") || child.is_named(Some(W_NS), "start")
                {
                    Some("left")
                } else if child.is_named(Some(W_NS), "right") || child.is_named(Some(W_NS), "end") {
                    Some("right")
                } else if child.is_named(Some(W_NS), "between") {
                    Some("between")
                } else if child.is_named(Some(W_NS), "bar") {
                    Some("bar")
                } else {
                    None
                };
                let Some(target) = target else {
                    continue;
                };
                let child = cursor
                    .take_child(index)
                    .and_then(StrictXmlNode::into_element)
                    .ok_or_else(|| OxmlError::MissingElement("border edge".to_string()))?;
                let parsed_edge = CT_BorderEdge::from_strict_xml(child)?;
                let (edge, leftovers) = parsed_edge.into_parts();
                descendants.push(StrictXmlCompleteness::from_leftovers(leftovers));
                match target {
                    "top" => borders.top = Some(edge),
                    "bottom" => borders.bottom = Some(edge),
                    "left" => borders.left = Some(edge),
                    "right" => borders.right = Some(edge),
                    "between" => borders.between = Some(edge),
                    "bar" => borders.bar = Some(edge),
                    _ => unreachable!(),
                }
            }
            Ok(borders)
        })?;
        let (mut borders, leftovers) = parsed.into_parts();
        borders.completeness = StrictXmlCompleteness::new(leftovers, descendants);
        Ok(borders)
    }

    pub fn has_unmodeled_properties(&self) -> bool {
        !self.completeness.is_complete()
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_context(reader, &NamespaceContext::default())
    }

    pub fn from_xml_with_context(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
    ) -> Result<Self> {
        let element = parse_reader_element(reader, context, Some(W_NS), "pBdr", [])?;
        Self::from_strict_xml(element)
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("w:pBdr")))?;

        if let Some(ref edge) = self.top {
            let mut e = BytesStart::new("w:top");
            edge.write_xml_attrs(&mut e);
            writer.write_event(Event::Empty(e))?;
        }
        if let Some(ref edge) = self.left {
            let mut e = BytesStart::new("w:left");
            edge.write_xml_attrs(&mut e);
            writer.write_event(Event::Empty(e))?;
        }
        if let Some(ref edge) = self.bottom {
            let mut e = BytesStart::new("w:bottom");
            edge.write_xml_attrs(&mut e);
            writer.write_event(Event::Empty(e))?;
        }
        if let Some(ref edge) = self.right {
            let mut e = BytesStart::new("w:right");
            edge.write_xml_attrs(&mut e);
            writer.write_event(Event::Empty(e))?;
        }
        if let Some(ref edge) = self.between {
            let mut e = BytesStart::new("w:between");
            edge.write_xml_attrs(&mut e);
            writer.write_event(Event::Empty(e))?;
        }
        if let Some(ref edge) = self.bar {
            let mut e = BytesStart::new("w:bar");
            edge.write_xml_attrs(&mut e);
            writer.write_event(Event::Empty(e))?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:pBdr")))?;
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.top.is_none()
            && self.bottom.is_none()
            && self.left.is_none()
            && self.right.is_none()
            && self.between.is_none()
            && self.bar.is_none()
    }
}

/// A single tab stop definition.
#[derive(Debug, Clone)]
pub struct CT_TabStop {
    /// Tab stop alignment
    pub val: ST_TabJc,
    /// Position in twips
    pub pos: Twips,
    /// Leader character
    pub leader: Option<ST_TabLeader>,
    /// Original occurrence in a parsed tab collection, or `None` for a new tab.
    ///
    /// This preservation value is ignored by semantic equality. Callers moving
    /// a tab into another `CT_Tabs` collection should set it to `None`.
    pub source_occurrence: Option<usize>,
}

impl PartialEq for CT_TabStop {
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val && self.pos == other.pos && self.leader == other.leader
    }
}

impl CT_TabStop {
    pub fn new(val: ST_TabJc, pos: Twips) -> Self {
        CT_TabStop {
            val,
            pos,
            leader: None,
            source_occurrence: None,
        }
    }

    pub fn from_xml_attrs(e: &BytesStart) -> Result<Self> {
        let element = parse_empty_started_element(&NamespaceContext::default(), Some(W_NS), e)?;
        Ok(Self::from_strict_xml(element)?.value)
    }

    fn from_strict_xml(element: StrictXmlElement) -> Result<StrictXmlParsed<Self>> {
        element.parse(|cursor| {
            let val = cursor
                .take_attribute(Some(W_NS), "val")
                .map(|value| ST_TabJc::from_str(&value))
                .transpose()?
                .unwrap_or(ST_TabJc::Left);
            let pos = cursor
                .take_attribute(Some(W_NS), "pos")
                .map(|value| value.parse().map(Twips))
                .transpose()?
                .unwrap_or(Twips(0));
            let leader = cursor
                .take_attribute(Some(W_NS), "leader")
                .map(|value| ST_TabLeader::from_str(&value))
                .transpose()?;
            Ok(Self {
                val,
                pos,
                leader,
                source_occurrence: None,
            })
        })
    }
}

/// `CT_Tabs` — Collection of tab stop definitions.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CT_Tabs {
    #[doc(hidden)]
    pub completeness: StrictXmlCompleteness,
    pub tabs: Vec<CT_TabStop>,
}

impl CT_Tabs {
    pub(crate) fn from_strict_xml(element: StrictXmlElement) -> Result<Self> {
        let mut descendants = Vec::new();
        let parsed = element.parse(|cursor| {
            let mut tabs = Vec::new();
            for index in 0..cursor.child_slots() {
                let Some(StrictXmlNode::Element(child)) = cursor.child(index) else {
                    continue;
                };
                if !child.is_named(Some(W_NS), "tab") {
                    continue;
                }
                let child = cursor
                    .take_child(index)
                    .and_then(StrictXmlNode::into_element)
                    .ok_or_else(|| OxmlError::MissingElement("tab stop".to_string()))?;
                let parsed_tab = CT_TabStop::from_strict_xml(child)?;
                let (mut tab, leftovers) = parsed_tab.into_parts();
                tab.source_occurrence = Some(tabs.len());
                tabs.push(tab);
                descendants.push(StrictXmlCompleteness::from_leftovers(leftovers));
            }
            Ok(tabs)
        })?;
        let (tabs, leftovers) = parsed.into_parts();
        Ok(Self {
            completeness: StrictXmlCompleteness::new(leftovers, descendants),
            tabs,
        })
    }

    pub fn has_unmodeled_properties(&self) -> bool {
        !self.completeness.is_complete()
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_context(reader, &NamespaceContext::default())
    }

    pub fn from_xml_with_context(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
    ) -> Result<Self> {
        let element = parse_reader_element(reader, context, Some(W_NS), "tabs", [])?;
        Self::from_strict_xml(element)
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        if self.tabs.is_empty() {
            return Ok(());
        }

        writer.write_event(Event::Start(BytesStart::new("w:tabs")))?;

        let mut buf = itoa::Buffer::new();
        for tab in &self.tabs {
            let mut e = BytesStart::new("w:tab");
            e.push_attribute(("w:val", tab.val.to_str()));
            e.push_attribute(("w:pos", buf.format(tab.pos.0)));
            if let Some(leader) = tab.leader {
                e.push_attribute(("w:leader", leader.to_str()));
            }
            writer.write_event(Event::Empty(e))?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:tabs")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_borders(xml: &str) -> CT_PBdr {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if matches_local_name(e.name().as_ref(), b"pBdr") => {
                    break;
                }
                _ => {}
            }
            buf.clear();
        }
        CT_PBdr::from_xml(&mut reader).unwrap()
    }

    #[test]
    fn round_trip_borders() {
        let bdr = CT_PBdr {
            top: Some(CT_BorderEdge {
                val: ST_Border::Single,
                sz: Some(4),
                space: Some(1),
                color: Some("000000".to_string()),
            }),
            bottom: Some(CT_BorderEdge {
                val: ST_Border::Double,
                sz: Some(6),
                space: Some(2),
                color: Some("FF0000".to_string()),
            }),
            ..Default::default()
        };

        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        bdr.to_xml(&mut writer).unwrap();
        let xml = String::from_utf8(output).unwrap();

        let full = xml.to_string();
        let mut reader = Reader::from_str(&full);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        // Skip to pBdr start
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if matches_local_name(e.name().as_ref(), b"pBdr") => {
                    break;
                }
                _ => {}
            }
            buf.clear();
        }
        let parsed = CT_PBdr::from_xml(&mut reader).unwrap();

        assert_eq!(parsed.top.as_ref().unwrap().val, ST_Border::Single);
        assert_eq!(parsed.top.as_ref().unwrap().sz, Some(4));
        assert_eq!(parsed.bottom.as_ref().unwrap().val, ST_Border::Double);
        assert!(parsed.left.is_none());
    }

    #[test]
    fn expanded_border_edge_parses_like_empty_element() {
        let parsed = parse_borders(concat!(
            r#"<w:pBdr><w:bottom w:val="single" w:sz="8" "#,
            r#"w:space="1" w:color="808080"></w:bottom></w:pBdr>"#,
        ));
        let bottom = parsed.bottom.unwrap();

        assert_eq!(bottom.val, ST_Border::Single);
        assert_eq!(bottom.sz, Some(8));
        assert_eq!(bottom.space, Some(1));
        assert_eq!(bottom.color, Some("808080".to_string()));
    }

    #[test]
    fn round_trip_tabs() {
        let tabs = CT_Tabs {
            tabs: vec![
                CT_TabStop {
                    val: ST_TabJc::Left,
                    pos: Twips(720),
                    leader: None,
                    source_occurrence: None,
                },
                CT_TabStop {
                    val: ST_TabJc::Center,
                    pos: Twips(4320),
                    leader: Some(ST_TabLeader::Dot),
                    source_occurrence: None,
                },
                CT_TabStop {
                    val: ST_TabJc::Right,
                    pos: Twips(8640),
                    leader: Some(ST_TabLeader::Hyphen),
                    source_occurrence: None,
                },
            ],
            ..Default::default()
        };

        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        tabs.to_xml(&mut writer).unwrap();
        let xml = String::from_utf8(output).unwrap();

        let mut reader = Reader::from_str(&xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if matches_local_name(e.name().as_ref(), b"tabs") => {
                    break;
                }
                _ => {}
            }
            buf.clear();
        }
        let parsed = CT_Tabs::from_xml(&mut reader).unwrap();

        assert_eq!(parsed.tabs.len(), 3);
        assert_eq!(parsed.tabs[0].val, ST_TabJc::Left);
        assert_eq!(parsed.tabs[0].pos, Twips(720));
        assert_eq!(parsed.tabs[1].val, ST_TabJc::Center);
        assert_eq!(parsed.tabs[1].leader, Some(ST_TabLeader::Dot));
        assert_eq!(parsed.tabs[2].val, ST_TabJc::Right);
        assert_eq!(parsed.tabs[0].source_occurrence, Some(0));
        assert_eq!(parsed.tabs[1].source_occurrence, Some(1));
    }

    #[test]
    fn namespace_aware_tabs_ignore_foreign_same_local_children() {
        let xml = r#"<q:tabs xmlns:q="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:ext="urn:producer"><ext:tab ext:val="right" ext:pos="99"/><q:tab q:val="left" q:pos="720"/></q:tabs>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(_)) => break,
                Ok(Event::Eof) => panic!("missing tabs start"),
                _ => {}
            }
            buf.clear();
        }
        let context = NamespaceContext::new([
            ("q".to_string(), W_NS.to_string()),
            ("ext".to_string(), "urn:producer".to_string()),
        ]);
        let parsed = CT_Tabs::from_xml_with_context(&mut reader, &context).unwrap();
        assert_eq!(parsed.tabs.len(), 1);
        assert_eq!(parsed.tabs[0].pos, Twips(720));
        assert_eq!(parsed.tabs[0].source_occurrence, Some(0));

        let mut constructed = CT_TabStop::new(ST_TabJc::Left, Twips(720));
        assert_eq!(parsed.tabs[0], constructed);
        constructed.source_occurrence = Some(99);
        assert_eq!(parsed.tabs[0], constructed);
    }

    #[test]
    fn namespace_aware_tabs_track_shadows_and_expanded_tab_elements() {
        let xml = r#"<q:tabs xmlns:q="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><foreign xmlns="urn:producer" xmlns:q="urn:producer"><q:tab q:val="right" q:pos="99"/><q:tabs><q:tab q:val="right" q:pos="100"/></q:tabs></foreign><q:tab q:val="left" q:pos="720"></q:tab><q:tab q:val="right" q:pos="1440"/></q:tabs>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(_)) => break,
                Ok(Event::Eof) => panic!("missing tabs start"),
                event => {
                    event.unwrap();
                }
            }
            buf.clear();
        }

        let context = NamespaceContext::new([("q".to_string(), W_NS.to_string())]);
        let parsed = CT_Tabs::from_xml_with_context(&mut reader, &context).unwrap();
        assert_eq!(parsed.tabs.len(), 2);
        assert_eq!(parsed.tabs[0].pos, Twips(720));
        assert_eq!(parsed.tabs[0].source_occurrence, Some(0));
        assert_eq!(parsed.tabs[1].pos, Twips(1440));
        assert_eq!(parsed.tabs[1].source_occurrence, Some(1));

        let default_xml = r#"<tabs xmlns="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><foreign xmlns="urn:producer"><tab val="right" pos="99"/><tabs><tab val="right" pos="100"/></tabs></foreign><tab xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" w:val="center" w:pos="2160"></tab></tabs>"#;
        let mut reader = Reader::from_str(default_xml);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(_)) => break,
                Ok(Event::Eof) => panic!("missing default tabs start"),
                event => {
                    event.unwrap();
                }
            }
            buf.clear();
        }
        let context = NamespaceContext::new([(String::new(), W_NS.to_string())]);
        let parsed = CT_Tabs::from_xml_with_context(&mut reader, &context).unwrap();
        assert_eq!(parsed.tabs.len(), 1);
        assert_eq!(parsed.tabs[0].val, ST_TabJc::Center);
        assert_eq!(parsed.tabs[0].pos, Twips(2160));
        assert_eq!(parsed.tabs[0].source_occurrence, Some(0));
    }

    #[test]
    fn namespace_aware_tabs_reject_deep_distinct_aliases_normally() {
        let mut xml = format!(r#"<q:tabs xmlns:q="{W_NS}">"#);
        for depth in 0..=oxml_core::xml::DEFAULT_MAX_XML_DEPTH {
            xml.push_str(&format!(r#"<n{depth}:unknown xmlns:n{depth}="{W_NS}">"#));
        }
        for depth in (0..=oxml_core::xml::DEFAULT_MAX_XML_DEPTH).rev() {
            xml.push_str(&format!("</n{depth}:unknown>"));
        }
        xml.push_str("</q:tabs>");

        let mut reader = Reader::from_str(&xml);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(_)) => break,
                Ok(Event::Eof) => panic!("missing tabs start"),
                event => {
                    event.unwrap();
                }
            }
            buf.clear();
        }
        let context = NamespaceContext::new([("q".to_string(), W_NS.to_string())]);
        let error = CT_Tabs::from_xml_with_context(&mut reader, &context)
            .expect_err("deep alias nesting must be bounded");
        assert!(error.to_string().contains("depth"));
    }

    #[test]
    fn border_edge_all_styles_round_trip() {
        // Test that all border styles serialize and deserialize correctly
        let styles = [
            ST_Border::None,
            ST_Border::Single,
            ST_Border::Thick,
            ST_Border::Double,
            ST_Border::Dotted,
            ST_Border::Dashed,
            ST_Border::DotDash,
            ST_Border::Wave,
        ];

        for &style in &styles {
            let bdr = CT_PBdr {
                top: Some(CT_BorderEdge {
                    val: style,
                    sz: Some(8),
                    space: Some(0),
                    color: Some("FF00FF".to_string()),
                }),
                ..Default::default()
            };

            let mut output = Vec::new();
            let mut writer = Writer::new(&mut output);
            bdr.to_xml(&mut writer).unwrap();
            let xml = String::from_utf8(output).unwrap();

            let mut reader = Reader::from_str(&xml);
            reader.config_mut().trim_text(true);
            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Start(ref e)) if matches_local_name(e.name().as_ref(), b"pBdr") => {
                        break;
                    }
                    _ => {}
                }
                buf.clear();
            }
            let parsed = CT_PBdr::from_xml(&mut reader).unwrap();
            let top = parsed.top.as_ref().unwrap();
            assert_eq!(
                top.val, style,
                "Border style round-trip failed for {style:?}"
            );
            assert_eq!(top.sz, Some(8));
            assert_eq!(top.color.as_deref(), Some("FF00FF"));
        }
    }
}
