//! Numbering definitions: `CT_Numbering`, `CT_AbstractNum`, `CT_Num`, `CT_Lvl`.
//!
//! These types represent the content of `numbering.xml`, which defines
//! abstract numbering formats and numbering instances that paragraphs reference.

use std::collections::HashSet;
use std::io::Write;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use oxml_core::xml::{
    StrictXmlCompleteness, StrictXmlCursor, StrictXmlDocument, StrictXmlElement,
    StrictXmlLeftovers, StrictXmlNode, parse_reader_element,
};

use crate::error::{OxmlError, Result};
use crate::namespace::W_NS;
use crate::properties::{CT_PPr, CT_RPr};
use crate::raw_xml::NamespaceContext;
use crate::shared::ST_Jc;
use crate::styles::{CT_Styles, StyleType};

const MAX_NUMBERING_LEVEL: u32 = 8;

fn take_element(
    cursor: &mut StrictXmlCursor,
    index: usize,
    description: &str,
) -> Result<StrictXmlElement> {
    cursor
        .take_child(index)
        .and_then(StrictXmlNode::into_element)
        .ok_or_else(|| OxmlError::MissingElement(description.to_string()))
}

fn required_strict_u32_attr(cursor: &mut StrictXmlCursor, local: &str, path: &str) -> Result<u32> {
    cursor
        .take_attribute(Some(W_NS), local)
        .ok_or_else(|| OxmlError::MissingElement(path.to_string()))?
        .parse()
        .map_err(Into::into)
}

fn required_strict_level_attr(cursor: &mut StrictXmlCursor, path: &str) -> Result<u32> {
    let level = required_strict_u32_attr(cursor, "ilvl", path)?;
    if level <= MAX_NUMBERING_LEVEL {
        Ok(level)
    } else {
        Err(OxmlError::InvalidValue(format!(
            "{path} must be between 0 and {MAX_NUMBERING_LEVEL}, got {level}"
        )))
    }
}

fn unmodeled_element_completeness(element: StrictXmlElement) -> StrictXmlCompleteness {
    StrictXmlCompleteness::from_leftovers(StrictXmlLeftovers {
        attributes: Vec::new(),
        children: vec![StrictXmlNode::Element(Box::new(element))],
    })
}

fn write_extras_at<W: Write>(
    writer: &mut Writer<W>,
    extras: &[(usize, Vec<u8>)],
    index: usize,
) -> Result<()> {
    for (_, raw) in extras.iter().filter(|(position, _)| *position == index) {
        writer.get_mut().write_all(raw)?;
    }
    Ok(())
}

/// `ST_NumberFormat` — Numbering format type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ST_NumberFormat {
    Decimal,
    UpperRoman,
    LowerRoman,
    UpperLetter,
    LowerLetter,
    Ordinal,
    Bullet,
    None,
    /// A valid producer-defined or otherwise unmodelled OOXML format name.
    Other(String),
}

impl ST_NumberFormat {
    pub fn from_str(s: &str) -> Self {
        match s {
            "decimal" => Self::Decimal,
            "upperRoman" => Self::UpperRoman,
            "lowerRoman" => Self::LowerRoman,
            "upperLetter" => Self::UpperLetter,
            "lowerLetter" => Self::LowerLetter,
            "ordinal" => Self::Ordinal,
            "bullet" => Self::Bullet,
            "none" => Self::None,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::Decimal => "decimal",
            Self::UpperRoman => "upperRoman",
            Self::LowerRoman => "lowerRoman",
            Self::UpperLetter => "upperLetter",
            Self::LowerLetter => "lowerLetter",
            Self::Ordinal => "ordinal",
            Self::Bullet => "bullet",
            Self::None => "none",
            Self::Other(value) => value,
        }
    }
}

/// `CT_Lvl` — A single level (0–8) in an abstract numbering definition.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Lvl {
    #[doc(hidden)]
    pub completeness: StrictXmlCompleteness,
    /// Level index (0–8)
    pub ilvl: u32,
    /// Starting number
    pub start: Option<u32>,
    /// Number format
    pub num_fmt: Option<ST_NumberFormat>,
    /// Level text (e.g., "%1.", "%1.%2.", bullet char)
    pub lvl_text: Option<String>,
    /// Level justification
    pub lvl_jc: Option<ST_Jc>,
    /// Paragraph style associated with this numbering level.
    pub p_style: Option<String>,
    /// Paragraph properties for this level (typically indentation)
    pub ppr: Option<CT_PPr>,
    /// Run properties for the numbering symbol
    pub rpr: Option<CT_RPr>,
}

#[allow(non_snake_case)]
impl CT_Lvl {
    pub fn new(ilvl: u32) -> Self {
        CT_Lvl {
            completeness: StrictXmlCompleteness::default(),
            ilvl,
            start: None,
            num_fmt: None,
            lvl_text: None,
            lvl_jc: None,
            p_style: None,
            ppr: None,
            rpr: None,
        }
    }

    fn from_strict_xml(element: StrictXmlElement) -> Result<Self> {
        let mut descendants = Vec::new();
        let parsed = element.parse(|cursor| {
            let ilvl = required_strict_level_attr(cursor, "w:lvl/@w:ilvl")?;
            let mut level = Self::new(ilvl);
            for index in 0..cursor.child_slots() {
                let Some(StrictXmlNode::Element(child)) = cursor.child(index) else {
                    continue;
                };
                let local = [
                    "start", "numFmt", "lvlText", "pStyle", "lvlJc", "pPr", "rPr",
                ]
                .into_iter()
                .find(|local| child.is_named(Some(W_NS), local));
                let Some(local) = local else {
                    continue;
                };
                let child = take_element(cursor, index, local)?;
                let completeness = match local {
                    "pPr" => {
                        let properties = CT_PPr::from_strict_xml(child)?;
                        let completeness = properties.completeness.clone();
                        level.ppr = Some(properties);
                        completeness
                    }
                    "rPr" => {
                        let properties = CT_RPr::from_strict_xml(child)?;
                        let completeness = properties.completeness.clone();
                        level.rpr = Some(properties);
                        completeness
                    }
                    _ => {
                        let parsed_value = child.parse(|cursor| {
                            let value = cursor.take_attribute(Some(W_NS), "val");
                            match local {
                                "start" => {
                                    level.start = value.map(|value| value.parse()).transpose()?;
                                }
                                "numFmt" => {
                                    level.num_fmt =
                                        value.map(|value| ST_NumberFormat::from_str(&value));
                                }
                                "lvlText" => level.lvl_text = value,
                                "pStyle" => level.p_style = value,
                                "lvlJc" => {
                                    level.lvl_jc =
                                        value.map(|value| ST_Jc::from_str(&value)).transpose()?;
                                }
                                _ => unreachable!(),
                            }
                            Ok(())
                        })?;
                        StrictXmlCompleteness::from_leftovers(parsed_value.leftovers)
                    }
                };
                descendants.push(completeness);
            }
            Ok(level)
        })?;
        let (mut level, leftovers) = parsed.into_parts();
        level.completeness = StrictXmlCompleteness::new(leftovers, descendants);
        Ok(level)
    }

    pub fn has_unmodeled_properties(&self) -> bool {
        !self.completeness.is_complete()
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>, ilvl: u32) -> Result<Self> {
        let context = NamespaceContext::default();
        let element = parse_reader_element(
            reader,
            &context,
            Some(W_NS),
            "lvl",
            [("w:ilvl".to_string(), ilvl.to_string())],
        )?;
        Self::from_strict_xml(element)
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut buf = itoa::Buffer::new();
        let mut start = BytesStart::new("w:lvl");
        start.push_attribute(("w:ilvl", buf.format(self.ilvl)));
        writer.write_event(Event::Start(start))?;

        if let Some(s) = self.start {
            let mut e = BytesStart::new("w:start");
            e.push_attribute(("w:val", buf.format(s)));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref fmt) = self.num_fmt {
            let mut e = BytesStart::new("w:numFmt");
            e.push_attribute(("w:val", fmt.to_str()));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref style_id) = self.p_style {
            let mut e = BytesStart::new("w:pStyle");
            e.push_attribute(("w:val", style_id.as_str()));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref text) = self.lvl_text {
            let mut e = BytesStart::new("w:lvlText");
            e.push_attribute(("w:val", text.as_str()));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(jc) = self.lvl_jc {
            let mut e = BytesStart::new("w:lvlJc");
            e.push_attribute(("w:val", jc.to_str()));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref ppr) = self.ppr {
            ppr.to_xml(writer)?;
        }

        if let Some(ref rpr) = self.rpr {
            rpr.to_xml(writer)?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:lvl")))?;
        Ok(())
    }
}

impl Default for CT_Lvl {
    fn default() -> Self {
        Self::new(0)
    }
}

/// `CT_AbstractNum` — An abstract numbering definition with up to 9 levels.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_AbstractNum {
    #[doc(hidden)]
    pub completeness: StrictXmlCompleteness,
    pub abstract_num_id: u32,
    pub levels: Vec<CT_Lvl>,
    /// Optional multi-level type hint
    pub multi_level_type: Option<String>,
    /// Numbering style for which this definition supplies the underlying levels.
    pub style_link: Option<String>,
    /// Numbering style that points to the underlying definition to inherit.
    pub num_style_link: Option<String>,
    /// Still-unmodelled children, positioned among the modelled children.
    pub extra_xml: Vec<(usize, Vec<u8>)>,
}

#[allow(non_snake_case)]
impl CT_AbstractNum {
    pub fn new(id: u32) -> Self {
        CT_AbstractNum {
            completeness: StrictXmlCompleteness::default(),
            abstract_num_id: id,
            levels: Vec::new(),
            multi_level_type: None,
            style_link: None,
            num_style_link: None,
            extra_xml: Vec::new(),
        }
    }

    fn from_strict_xml(element: StrictXmlElement) -> Result<Self> {
        let mut descendants = Vec::new();
        let parsed = element.parse(|cursor| {
            let id = required_strict_u32_attr(
                cursor,
                "abstractNumId",
                "w:abstractNum/@w:abstractNumId",
            )?;
            let mut abstract_num = Self::new(id);
            let mut modeled_index = 0usize;
            for index in 0..cursor.child_slots() {
                let Some(StrictXmlNode::Element(child)) = cursor.child(index) else {
                    continue;
                };
                let kind = if child.is_named(Some(W_NS), "lvl") {
                    Some("lvl")
                } else if child.is_named(Some(W_NS), "multiLevelType") {
                    Some("multiLevelType")
                } else if child.is_named(Some(W_NS), "styleLink")
                    && abstract_num.style_link.is_none()
                {
                    Some("styleLink")
                } else if child.is_named(Some(W_NS), "numStyleLink")
                    && abstract_num.num_style_link.is_none()
                {
                    Some("numStyleLink")
                } else {
                    None
                };
                let child = take_element(cursor, index, "abstract numbering child")?;
                match kind {
                    Some("lvl") => {
                        let level = CT_Lvl::from_strict_xml(child)?;
                        descendants.push(level.completeness.clone());
                        abstract_num.levels.push(level);
                        modeled_index += 1;
                    }
                    Some(local) => {
                        let parsed_value =
                            child.parse(|cursor| Ok(cursor.take_attribute(Some(W_NS), "val")))?;
                        let (value, leftovers) = parsed_value.into_parts();
                        match local {
                            "multiLevelType" => abstract_num.multi_level_type = value,
                            "styleLink" => abstract_num.style_link = value,
                            "numStyleLink" => abstract_num.num_style_link = value,
                            _ => unreachable!(),
                        }
                        descendants.push(StrictXmlCompleteness::from_leftovers(leftovers));
                        modeled_index += 1;
                    }
                    None => {
                        descendants.push(unmodeled_element_completeness(child.clone()));
                        abstract_num
                            .extra_xml
                            .push((modeled_index, child.into_raw_xml().bytes().to_vec()));
                    }
                }
            }
            Ok(abstract_num)
        })?;
        let (mut abstract_num, leftovers) = parsed.into_parts();
        abstract_num.completeness = StrictXmlCompleteness::new(leftovers, descendants);
        Ok(abstract_num)
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>, abstract_num_id: u32) -> Result<Self> {
        let context = NamespaceContext::default();
        let element = parse_reader_element(
            reader,
            &context,
            Some(W_NS),
            "abstractNum",
            [("w:abstractNumId".to_string(), abstract_num_id.to_string())],
        )?;
        Self::from_strict_xml(element)
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut buf = itoa::Buffer::new();
        let mut start = BytesStart::new("w:abstractNum");
        start.push_attribute(("w:abstractNumId", buf.format(self.abstract_num_id)));
        writer.write_event(Event::Start(start))?;

        let mut modelled_index = 0;
        write_extras_at(writer, &self.extra_xml, modelled_index)?;

        if let Some(ref mlt) = self.multi_level_type {
            let mut e = BytesStart::new("w:multiLevelType");
            e.push_attribute(("w:val", mlt.as_str()));
            writer.write_event(Event::Empty(e))?;
            modelled_index += 1;
            write_extras_at(writer, &self.extra_xml, modelled_index)?;
        }

        if let Some(ref style_id) = self.style_link {
            let mut e = BytesStart::new("w:styleLink");
            e.push_attribute(("w:val", style_id.as_str()));
            writer.write_event(Event::Empty(e))?;
            modelled_index += 1;
            write_extras_at(writer, &self.extra_xml, modelled_index)?;
        }

        if let Some(ref style_id) = self.num_style_link {
            let mut e = BytesStart::new("w:numStyleLink");
            e.push_attribute(("w:val", style_id.as_str()));
            writer.write_event(Event::Empty(e))?;
            modelled_index += 1;
            write_extras_at(writer, &self.extra_xml, modelled_index)?;
        }

        for lvl in &self.levels {
            lvl.to_xml(writer)?;
            modelled_index += 1;
            write_extras_at(writer, &self.extra_xml, modelled_index)?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:abstractNum")))?;
        Ok(())
    }
}

impl Default for CT_AbstractNum {
    fn default() -> Self {
        Self::new(0)
    }
}

/// `CT_Num` — A numbering instance that references an abstract numbering definition.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Num {
    #[doc(hidden)]
    pub completeness: StrictXmlCompleteness,
    pub num_id: u32,
    pub abstract_num_id: u32,
    pub level_overrides: Vec<CT_LvlOverride>,
}

/// An instance-specific numbering-level override.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_LvlOverride {
    #[doc(hidden)]
    pub completeness: StrictXmlCompleteness,
    pub ilvl: u32,
    pub start_override: Option<u32>,
    pub level: Option<CT_Lvl>,
}

impl CT_LvlOverride {
    fn from_strict_xml(element: StrictXmlElement) -> Result<Self> {
        let mut descendants = Vec::new();
        let parsed = element.parse(|cursor| {
            let ilvl = required_strict_level_attr(cursor, "w:lvlOverride/@w:ilvl")?;
            let mut value = Self {
                completeness: StrictXmlCompleteness::default(),
                ilvl,
                start_override: None,
                level: None,
            };
            for index in 0..cursor.child_slots() {
                let Some(StrictXmlNode::Element(child)) = cursor.child(index) else {
                    continue;
                };
                if child.is_named(Some(W_NS), "startOverride") && value.start_override.is_none() {
                    let child = take_element(cursor, index, "startOverride")?;
                    let parsed_start = child.parse(|cursor| {
                        cursor
                            .take_attribute(Some(W_NS), "val")
                            .map(|value| value.parse())
                            .transpose()
                            .map_err(Into::into)
                    })?;
                    let (start, leftovers) = parsed_start.into_parts();
                    value.start_override = start;
                    descendants.push(StrictXmlCompleteness::from_leftovers(leftovers));
                } else if child.is_named(Some(W_NS), "lvl") && value.level.is_none() {
                    let child = take_element(cursor, index, "lvl")?;
                    let level = CT_Lvl::from_strict_xml(child)?;
                    descendants.push(level.completeness.clone());
                    value.level = Some(level);
                }
            }
            Ok(value)
        })?;
        let (mut value, leftovers) = parsed.into_parts();
        value.completeness = StrictXmlCompleteness::new(leftovers, descendants);
        Ok(value)
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>, num_id: u32) -> Result<Self> {
        let context = NamespaceContext::default();
        let element = parse_reader_element(
            reader,
            &context,
            Some(W_NS),
            "num",
            [("w:numId".to_string(), num_id.to_string())],
        )?;
        Self::from_strict_xml(element)
    }
    fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut buf = itoa::Buffer::new();
        let mut start = BytesStart::new("w:lvlOverride");
        start.push_attribute(("w:ilvl", buf.format(self.ilvl)));
        writer.write_event(Event::Start(start))?;

        if let Some(value) = self.start_override {
            let mut element = BytesStart::new("w:startOverride");
            element.push_attribute(("w:val", buf.format(value)));
            writer.write_event(Event::Empty(element))?;
        }
        if let Some(ref level) = self.level {
            level.to_xml(writer)?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:lvlOverride")))?;
        Ok(())
    }
}

impl Default for CT_LvlOverride {
    fn default() -> Self {
        Self {
            completeness: StrictXmlCompleteness::default(),
            ilvl: 0,
            start_override: None,
            level: None,
        }
    }
}

#[allow(non_snake_case)]
impl CT_Num {
    fn from_strict_xml(element: StrictXmlElement) -> Result<Self> {
        let mut descendants = Vec::new();
        let parsed = element.parse(|cursor| {
            let num_id = required_strict_u32_attr(cursor, "numId", "w:num/@w:numId")?;
            let mut abstract_num_id = None;
            let mut level_overrides = Vec::new();
            for index in 0..cursor.child_slots() {
                let Some(StrictXmlNode::Element(child)) = cursor.child(index) else {
                    continue;
                };
                if child.is_named(Some(W_NS), "abstractNumId") && abstract_num_id.is_none() {
                    let child = take_element(cursor, index, "abstractNumId")?;
                    let parsed_id = child.parse(|cursor| {
                        cursor
                            .take_attribute(Some(W_NS), "val")
                            .map(|value| value.parse())
                            .transpose()
                            .map_err(Into::into)
                    })?;
                    let (id, leftovers) = parsed_id.into_parts();
                    abstract_num_id = id;
                    descendants.push(StrictXmlCompleteness::from_leftovers(leftovers));
                } else if child.is_named(Some(W_NS), "lvlOverride") {
                    let child = take_element(cursor, index, "lvlOverride")?;
                    let level_override = CT_LvlOverride::from_strict_xml(child)?;
                    descendants.push(level_override.completeness.clone());
                    level_overrides.push(level_override);
                }
            }
            Ok(Self {
                completeness: StrictXmlCompleteness::default(),
                num_id,
                abstract_num_id: abstract_num_id.ok_or_else(|| {
                    OxmlError::MissingElement("w:num/w:abstractNumId/@w:val".to_string())
                })?,
                level_overrides,
            })
        })?;
        let (mut value, leftovers) = parsed.into_parts();
        value.completeness = StrictXmlCompleteness::new(leftovers, descendants);
        Ok(value)
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut buf = itoa::Buffer::new();
        let mut start = BytesStart::new("w:num");
        start.push_attribute(("w:numId", buf.format(self.num_id)));
        writer.write_event(Event::Start(start))?;

        let mut abs_ref = BytesStart::new("w:abstractNumId");
        abs_ref.push_attribute(("w:val", buf.format(self.abstract_num_id)));
        writer.write_event(Event::Empty(abs_ref))?;

        for level_override in &self.level_overrides {
            level_override.to_xml(writer)?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:num")))?;
        Ok(())
    }
}

impl Default for CT_Num {
    fn default() -> Self {
        Self {
            completeness: StrictXmlCompleteness::default(),
            num_id: 0,
            abstract_num_id: 0,
            level_overrides: Vec::new(),
        }
    }
}

/// `CT_Numbering` — Root element of the numbering definitions part.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Numbering {
    #[doc(hidden)]
    pub completeness: StrictXmlCompleteness,
    pub abstract_nums: Vec<CT_AbstractNum>,
    pub nums: Vec<CT_Num>,
}

#[allow(non_snake_case)]
impl CT_Numbering {
    pub fn new() -> Self {
        CT_Numbering {
            completeness: StrictXmlCompleteness::default(),
            abstract_nums: Vec::new(),
            nums: Vec::new(),
        }
    }

    /// Parse from XML bytes (the content of numbering.xml).
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let root = StrictXmlDocument::parse(xml)?.into_root();
        if !root.is_named(Some(W_NS), "numbering") {
            return Err(OxmlError::UnexpectedElement(
                "expected w:numbering".to_string(),
            ));
        }

        let mut descendants = Vec::new();
        let parsed = root.parse(|cursor| {
            let mut numbering = Self::new();
            for index in 0..cursor.child_slots() {
                let Some(StrictXmlNode::Element(child)) = cursor.child(index) else {
                    continue;
                };
                if child.is_named(Some(W_NS), "abstractNum") {
                    let child = take_element(cursor, index, "abstractNum")?;
                    let value = CT_AbstractNum::from_strict_xml(child)?;
                    descendants.push(value.completeness.clone());
                    numbering.abstract_nums.push(value);
                } else if child.is_named(Some(W_NS), "num") {
                    let child = take_element(cursor, index, "num")?;
                    let value = CT_Num::from_strict_xml(child)?;
                    descendants.push(value.completeness.clone());
                    numbering.nums.push(value);
                }
            }
            Ok(numbering)
        })?;
        let (mut numbering, leftovers) = parsed.into_parts();
        numbering.completeness = StrictXmlCompleteness::new(leftovers, descendants);
        Ok(numbering)
    }

    /// Serialize to XML bytes.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

        writer.write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))?;

        let mut start = BytesStart::new("w:numbering");
        start.push_attribute(("xmlns:w", W_NS));
        start.push_attribute((
            "xmlns:r",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
        ));
        writer.write_event(Event::Start(start))?;

        for abs in &self.abstract_nums {
            abs.to_xml(&mut writer)?;
        }

        for num in &self.nums {
            num.to_xml(&mut writer)?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:numbering")))?;

        Ok(writer.into_inner())
    }

    /// Get the next available abstract numbering ID.
    pub fn next_abstract_num_id(&self) -> u32 {
        self.abstract_nums
            .iter()
            .map(|a| a.abstract_num_id)
            .max()
            .map(|m| m + 1)
            .unwrap_or(0)
    }

    /// Get the next available numbering instance ID.
    pub fn next_num_id(&self) -> u32 {
        self.nums
            .iter()
            .map(|n| n.num_id)
            .max()
            .map(|m| m + 1)
            .unwrap_or(1)
    }

    /// Create a bullet list definition and return its numId.
    pub fn add_bullet_list(&mut self) -> u32 {
        self.add_list(&[(ST_NumberFormat::Bullet, Some(1))])
    }

    /// Create a numbered (decimal) list definition and return its numId.
    pub fn add_numbered_list(&mut self) -> u32 {
        self.add_list(&[(ST_NumberFormat::Decimal, Some(1))])
    }

    /// Create a list definition with explicit per-level formats and return
    /// its numId.
    ///
    /// `levels[i]` specifies level `i` as `(format, start)`; a `start` of
    /// `None` defaults to 1 (`start` has no meaning for bullet levels). All
    /// nine levels are always defined so a paragraph referencing a deeper
    /// level than was specified still renders: unspecified levels continue
    /// from the last specified format's family — the bullet-glyph rotation
    /// for bullets, the decimal/letter/roman rotation otherwise — matching
    /// the [`Self::add_bullet_list`] / [`Self::add_numbered_list`] templates.
    ///
    /// An empty `levels` behaves like [`Self::add_numbered_list`].
    pub fn add_list(&mut self, levels: &[(ST_NumberFormat, Option<u32>)]) -> u32 {
        let abs_id = self.next_abstract_num_id();
        let num_id = self.next_num_id();

        let mut abs = CT_AbstractNum::new(abs_id);
        abs.multi_level_type = Some("hybridMultilevel".to_string());

        let mut last_specified = ST_NumberFormat::Decimal;
        for i in 0..9u32 {
            let (num_fmt, start) = match levels.get(i as usize) {
                Some((fmt, start)) => {
                    last_specified = fmt.clone();
                    (fmt.clone(), start.unwrap_or(1))
                }
                None => (level_fill_format(&last_specified, i), 1),
            };

            abs.levels.push(build_level(i, num_fmt, start));
        }

        self.abstract_nums.push(abs);
        self.nums.push(CT_Num {
            num_id,
            abstract_num_id: abs_id,
            level_overrides: Vec::new(),
            ..Default::default()
        });

        num_id
    }

    /// Redefine one level of an existing list definition, for callers that
    /// only learn a deeper level's format when content first reaches it.
    ///
    /// Returns `false` when `num_id` is unknown or `ilvl` is out of range
    /// (levels are 0–8).
    pub fn set_list_level(
        &mut self,
        num_id: u32,
        ilvl: u32,
        num_fmt: ST_NumberFormat,
        start: Option<u32>,
    ) -> bool {
        if ilvl > 8 {
            return false;
        }

        let Some(num) = self.nums.iter().find(|n| n.num_id == num_id) else {
            return false;
        };
        let abstract_num_id = num.abstract_num_id;
        let Some(abs) = self
            .abstract_nums
            .iter_mut()
            .find(|a| a.abstract_num_id == abstract_num_id)
        else {
            return false;
        };

        let level = build_level(ilvl, num_fmt, start.unwrap_or(1));
        match abs.levels.iter_mut().find(|l| l.ilvl == ilvl) {
            Some(existing) => *existing = level,
            None => {
                abs.levels.push(level);
                abs.levels.sort_by_key(|l| l.ilvl);
            }
        }

        true
    }

    /// Look up the abstract numbering definition for a given numId.
    pub fn get_abstract_num_for(&self, num_id: u32) -> Option<&CT_AbstractNum> {
        let num = self.nums.iter().find(|n| n.num_id == num_id)?;
        self.abstract_nums
            .iter()
            .find(|a| a.abstract_num_id == num.abstract_num_id)
    }

    /// Resolve an instance level, applying full-level and start overrides.
    pub fn get_effective_level(
        &self,
        num_id: u32,
        ilvl: u32,
    ) -> Option<EffectiveNumberingLevel<'_>> {
        let num = self.nums.iter().find(|num| num.num_id == num_id)?;
        let abstract_level = self
            .abstract_nums
            .iter()
            .find(|value| value.abstract_num_id == num.abstract_num_id)
            .and_then(|value| value.levels.iter().find(|level| level.ilvl == ilvl));
        let level_override = num.level_overrides.iter().find(|level| level.ilvl == ilvl);
        let level = level_override
            .and_then(|value| value.level.as_ref())
            .or(abstract_level)?;
        let start = level_override
            .and_then(|value| value.start_override)
            .or(level.start)
            .unwrap_or(1);

        Some(EffectiveNumberingLevel {
            level,
            start,
            has_unmodeled_properties: level.has_unmodeled_properties(),
        })
    }

    /// Find the effective level associated with a paragraph style.
    pub fn get_effective_level_for_style(
        &self,
        num_id: u32,
        style_id: &str,
    ) -> Option<EffectiveNumberingLevel<'_>> {
        (0..=8).find_map(|ilvl| {
            let level = self.get_effective_level(num_id, ilvl)?;
            (level.level.p_style.as_deref() == Some(style_id)).then_some(level)
        })
    }

    /// Resolve a numbering level through a `numStyleLink` indirection when needed.
    pub fn get_effective_level_with_styles<'a>(
        &'a self,
        num_id: u32,
        ilvl: u32,
        styles: &CT_Styles,
    ) -> Option<EffectiveNumberingLevel<'a>> {
        self.get_effective_level_with_styles_inner(num_id, ilvl, styles, &mut HashSet::new())
    }

    fn get_effective_level_with_styles_inner<'a>(
        &'a self,
        num_id: u32,
        ilvl: u32,
        styles: &CT_Styles,
        seen: &mut HashSet<u32>,
    ) -> Option<EffectiveNumberingLevel<'a>> {
        if num_id == 0 || !seen.insert(num_id) {
            return None;
        }

        let num = self.nums.iter().find(|value| value.num_id == num_id)?;
        let abstract_num = self
            .abstract_nums
            .iter()
            .find(|value| value.abstract_num_id == num.abstract_num_id)?;
        let level_override = num.level_overrides.iter().find(|value| value.ilvl == ilvl);

        if let Some(level) = level_override.and_then(|value| value.level.as_ref()) {
            let start = level_override
                .and_then(|value| value.start_override)
                .or(level.start)
                .unwrap_or(1);
            return Some(EffectiveNumberingLevel {
                level,
                start,
                has_unmodeled_properties: level.has_unmodeled_properties(),
            });
        }

        let inherited =
            if let Some(level) = abstract_num.levels.iter().find(|level| level.ilvl == ilvl) {
                EffectiveNumberingLevel {
                    level,
                    start: level.start.unwrap_or(1),
                    has_unmodeled_properties: level.has_unmodeled_properties(),
                }
            } else {
                let style_id = abstract_num.num_style_link.as_deref()?;
                let style = styles.get_by_id(style_id)?;
                if style.style_type != StyleType::Numbering {
                    return None;
                }
                let linked_num_id = style
                    .ppr
                    .as_ref()
                    .and_then(|properties| properties.num_id)
                    .filter(|value| *value != 0)?;
                self.get_effective_level_with_styles_inner(linked_num_id, ilvl, styles, seen)?
            };

        Some(EffectiveNumberingLevel {
            level: inherited.level,
            start: level_override
                .and_then(|value| value.start_override)
                .unwrap_or(inherited.start),
            has_unmodeled_properties: inherited.has_unmodeled_properties,
        })
    }

    /// Find a style-associated level, following `numStyleLink` definitions.
    pub fn get_effective_level_for_style_with_styles<'a>(
        &'a self,
        num_id: u32,
        style_id: &str,
        styles: &CT_Styles,
    ) -> Option<EffectiveNumberingLevel<'a>> {
        (0..=8).find_map(|ilvl| {
            let level = self.get_effective_level_with_styles(num_id, ilvl, styles)?;
            (level.level.p_style.as_deref() == Some(style_id)).then_some(level)
        })
    }
}

/// A numbering level after applying its numbering-instance overrides.
#[derive(Debug, Clone, Copy)]
pub struct EffectiveNumberingLevel<'a> {
    pub level: &'a CT_Lvl,
    pub start: u32,
    pub has_unmodeled_properties: bool,
}

impl Default for CT_Numbering {
    fn default() -> Self {
        Self::new()
    }
}

/// Bullet glyph rotation shared by the list templates: • ◦ ▪ repeating.
const BULLET_CHARS: [&str; 9] = [
    "\u{2022}", // bullet •
    "\u{25E6}", // white bullet ◦
    "\u{25AA}", // black small square ▪
    "\u{2022}", // repeat pattern
    "\u{25E6}", "\u{25AA}", "\u{2022}", "\u{25E6}", "\u{25AA}",
];

/// Numeric format rotation shared by the list templates:
/// decimal, lowerLetter, lowerRoman repeating.
const NUMBERED_FORMATS: [ST_NumberFormat; 9] = [
    ST_NumberFormat::Decimal,
    ST_NumberFormat::LowerLetter,
    ST_NumberFormat::LowerRoman,
    ST_NumberFormat::Decimal,
    ST_NumberFormat::LowerLetter,
    ST_NumberFormat::LowerRoman,
    ST_NumberFormat::Decimal,
    ST_NumberFormat::LowerLetter,
    ST_NumberFormat::LowerRoman,
];

/// Template format for an unspecified level, keyed on the last format the
/// caller did specify: bullets stay bullets; anything numeric continues the
/// numbered rotation.
fn level_fill_format(last_specified: &ST_NumberFormat, ilvl: u32) -> ST_NumberFormat {
    match last_specified {
        ST_NumberFormat::Bullet => ST_NumberFormat::Bullet,
        _ => NUMBERED_FORMATS[ilvl as usize % NUMBERED_FORMATS.len()].clone(),
    }
}

/// One level in the shared template shape: bullet glyph or `%N.` text,
/// left-justified, indented 720tw per depth with a 360tw hanging indent.
fn build_level(ilvl: u32, num_fmt: ST_NumberFormat, start: u32) -> CT_Lvl {
    let mut lvl = CT_Lvl::new(ilvl);
    lvl.start = Some(start);
    lvl.num_fmt = Some(num_fmt.clone());
    lvl.lvl_text = Some(match num_fmt {
        ST_NumberFormat::Bullet => BULLET_CHARS[ilvl as usize % BULLET_CHARS.len()].to_string(),
        _ => format!("%{}.", ilvl + 1),
    });
    lvl.lvl_jc = Some(ST_Jc::Left);

    // Standard indentation: 720tw per level
    let indent = (ilvl + 1) as i32 * 720;
    lvl.ppr = Some(CT_PPr {
        ind_left: Some(crate::units::Twips(indent)),
        ind_hanging: Some(crate::units::Twips(360)),
        ..Default::default()
    });

    lvl
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::Twips;

    #[test]
    fn round_trip_numbering() {
        let mut numbering = CT_Numbering::new();
        let num_id = numbering.add_bullet_list();
        assert_eq!(num_id, 1);

        let xml = numbering.to_xml().unwrap();
        let parsed = CT_Numbering::from_xml(&xml).unwrap();

        assert_eq!(parsed.abstract_nums.len(), 1);
        assert_eq!(parsed.nums.len(), 1);
        assert_eq!(parsed.nums[0].num_id, 1);
        assert_eq!(parsed.nums[0].abstract_num_id, 0);

        let abs = &parsed.abstract_nums[0];
        assert_eq!(abs.levels.len(), 9);
        assert_eq!(abs.levels[0].num_fmt, Some(ST_NumberFormat::Bullet));
        assert_eq!(abs.levels[0].lvl_text, Some("\u{2022}".to_string()));
    }

    #[test]
    fn round_trip_numbered_list() {
        let mut numbering = CT_Numbering::new();
        let num_id = numbering.add_numbered_list();
        assert_eq!(num_id, 1);

        let xml = numbering.to_xml().unwrap();
        let parsed = CT_Numbering::from_xml(&xml).unwrap();

        let abs = &parsed.abstract_nums[0];
        assert_eq!(abs.levels[0].num_fmt, Some(ST_NumberFormat::Decimal));
        assert_eq!(abs.levels[0].lvl_text, Some("%1.".to_string()));
        assert_eq!(abs.levels[1].num_fmt, Some(ST_NumberFormat::LowerLetter));
    }

    #[test]
    fn multiple_lists() {
        let mut numbering = CT_Numbering::new();
        let bullet_id = numbering.add_bullet_list();
        let num_id = numbering.add_numbered_list();

        assert_eq!(bullet_id, 1);
        assert_eq!(num_id, 2);

        let xml = numbering.to_xml().unwrap();
        let parsed = CT_Numbering::from_xml(&xml).unwrap();

        assert_eq!(parsed.abstract_nums.len(), 2);
        assert_eq!(parsed.nums.len(), 2);
    }

    #[test]
    fn level_indentation() {
        let mut numbering = CT_Numbering::new();
        numbering.add_bullet_list();

        let abs = &numbering.abstract_nums[0];
        // Level 0: 720tw indent, 360tw hanging
        assert_eq!(
            abs.levels[0].ppr.as_ref().unwrap().ind_left,
            Some(Twips(720))
        );
        assert_eq!(
            abs.levels[0].ppr.as_ref().unwrap().ind_hanging,
            Some(Twips(360))
        );
        // Level 2: 2160tw indent
        assert_eq!(
            abs.levels[2].ppr.as_ref().unwrap().ind_left,
            Some(Twips(2160))
        );
    }

    #[test]
    fn parse_numbering_xml() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:abstractNum w:abstractNumId="0">
    <w:multiLevelType w:val="hybridMultilevel"/>
    <w:lvl w:ilvl="0">
      <w:start w:val="1"/>
      <w:numFmt w:val="decimal"/>
      <w:lvlText w:val="%1."/>
      <w:lvlJc w:val="left"/>
      <w:pPr>
        <w:ind w:left="720" w:hanging="360"/>
      </w:pPr>
    </w:lvl>
    <w:lvl w:ilvl="1">
      <w:start w:val="1"/>
      <w:numFmt w:val="lowerLetter"/>
      <w:lvlText w:val="%2."/>
      <w:lvlJc w:val="left"/>
    </w:lvl>
  </w:abstractNum>
  <w:num w:numId="1">
    <w:abstractNumId w:val="0"/>
  </w:num>
</w:numbering>"#;

        let numbering = CT_Numbering::from_xml(xml).unwrap();
        assert_eq!(numbering.abstract_nums.len(), 1);
        assert_eq!(numbering.nums.len(), 1);

        let abs = &numbering.abstract_nums[0];
        assert_eq!(abs.abstract_num_id, 0);
        assert_eq!(abs.multi_level_type, Some("hybridMultilevel".to_string()));
        assert_eq!(abs.levels.len(), 2);
        assert_eq!(abs.levels[0].start, Some(1));
        assert_eq!(abs.levels[0].num_fmt, Some(ST_NumberFormat::Decimal));
        assert_eq!(abs.levels[0].lvl_text, Some("%1.".to_string()));
        assert_eq!(
            abs.levels[0].ppr.as_ref().unwrap().ind_left,
            Some(Twips(720))
        );
        assert_eq!(abs.levels[1].num_fmt, Some(ST_NumberFormat::LowerLetter));

        let num = &numbering.nums[0];
        assert_eq!(num.num_id, 1);
        assert_eq!(num.abstract_num_id, 0);
    }

    #[test]
    fn preserves_style_associations_overrides_and_unmodelled_formats() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:abstractNum w:abstractNumId="4">
    <w:lvl w:ilvl="1">
      <w:start w:val="2"/>
      <w:numFmt w:val="decimalZero"/>
      <w:pStyle w:val="AssociatedStyle"/>
      <w:lvlText w:val="%2."/>
      <w:pPr><w:ind w:left="1440"/></w:pPr>
    </w:lvl>
    <w:lvl w:ilvl="2">
      <w:numFmt w:val="decimal"/>
      <w:lvlText w:val="%3."/>
    </w:lvl>
  </w:abstractNum>
  <w:num w:numId="8">
    <w:abstractNumId w:val="4"/>
    <w:lvlOverride w:ilvl="1"><w:startOverride w:val="7"/></w:lvlOverride>
    <w:lvlOverride w:ilvl="2">
      <w:lvl w:ilvl="2">
        <w:start w:val="3"/>
        <w:numFmt w:val="chicago"/>
        <w:lvlText w:val="%3)"/>
      </w:lvl>
    </w:lvlOverride>
  </w:num>
</w:numbering>"#;

        let numbering = CT_Numbering::from_xml(xml).unwrap();
        let associated = numbering
            .get_effective_level_for_style(8, "AssociatedStyle")
            .unwrap();
        assert_eq!(associated.level.ilvl, 1);
        assert_eq!(associated.start, 7);
        assert_eq!(associated.level.p_style.as_deref(), Some("AssociatedStyle"));
        assert_eq!(
            associated.level.num_fmt,
            Some(ST_NumberFormat::Other("decimalZero".to_string()))
        );

        let replaced = numbering.get_effective_level(8, 2).unwrap();
        assert_eq!(replaced.start, 3);
        assert_eq!(replaced.level.lvl_text.as_deref(), Some("%3)"));
        assert_eq!(
            replaced.level.num_fmt,
            Some(ST_NumberFormat::Other("chicago".to_string()))
        );

        let round_trip = String::from_utf8(numbering.to_xml().unwrap()).unwrap();
        assert!(round_trip.contains(r#"<w:pStyle w:val="AssociatedStyle"/>"#));
        assert!(round_trip.contains(r#"<w:numFmt w:val="decimalZero"/>"#));
        assert!(round_trip.contains(r#"<w:startOverride w:val="7"/>"#));
        assert!(round_trip.contains(r#"<w:numFmt w:val="chicago"/>"#));
    }

    #[test]
    fn parses_empty_and_expanded_start_overrides() {
        for start_override in [
            r#"<w:startOverride w:val="7"/>"#,
            r#"<w:startOverride w:val="7"></w:startOverride>"#,
        ] {
            let xml = format!(
                r#"<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                  <w:abstractNum w:abstractNumId="4">
                    <w:lvl w:ilvl="0"><w:numFmt w:val="decimal"/></w:lvl>
                  </w:abstractNum>
                  <w:num w:numId="8"><w:abstractNumId w:val="4"/><w:lvlOverride w:ilvl="0">{start_override}</w:lvlOverride></w:num>
                </w:numbering>"#
            );

            let numbering = CT_Numbering::from_xml(xml.as_bytes()).unwrap();
            assert_eq!(numbering.get_effective_level(8, 0).unwrap().start, 7);
        }
    }

    #[test]
    fn preserves_and_resolves_numbering_style_links() {
        let numbering_xml = br#"<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
          <w:abstractNum w:abstractNumId="1">
            <w:nsid w:val="A1B2C3D4"/>
            <w:numStyleLink w:val="LinkedNumbering"/>
          </w:abstractNum>
          <w:abstractNum w:abstractNumId="2">
            <w:styleLink w:val="LinkedNumbering"/>
            <w:lvl w:ilvl="0"><w:numFmt w:val="bullet"/><w:lvlText w:val="-"/></w:lvl>
          </w:abstractNum>
          <w:num w:numId="8"><w:abstractNumId w:val="1"/></w:num>
          <w:num w:numId="9"><w:abstractNumId w:val="2"/></w:num>
        </w:numbering>"#;
        let styles_xml =
            br#"<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
          <w:style w:type="numbering" w:styleId="LinkedNumbering">
            <w:pPr><w:numPr><w:numId w:val="9"/></w:numPr></w:pPr>
          </w:style>
        </w:styles>"#;

        let numbering = CT_Numbering::from_xml(numbering_xml).unwrap();
        let styles = CT_Styles::from_xml(styles_xml).unwrap();
        let level = numbering
            .get_effective_level_with_styles(8, 0, &styles)
            .unwrap();
        assert_eq!(level.level.num_fmt, Some(ST_NumberFormat::Bullet));

        let round_trip = String::from_utf8(numbering.to_xml().unwrap()).unwrap();
        assert!(round_trip.contains(r#"<w:nsid w:val="A1B2C3D4"/>"#));
        assert!(round_trip.contains(r#"<w:numStyleLink w:val="LinkedNumbering"/>"#));
        assert!(round_trip.contains(r#"<w:styleLink w:val="LinkedNumbering"/>"#));
    }

    #[test]
    fn rejects_missing_required_numbering_identifiers() {
        for xml in [
            br#"<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:abstractNum/></w:numbering>"#.as_slice(),
            br#"<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:num><w:abstractNumId w:val="1"/></w:num></w:numbering>"#.as_slice(),
            br#"<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:num w:numId="1"/></w:numbering>"#.as_slice(),
            br#"<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:abstractNum w:abstractNumId="1"><w:lvl/></w:abstractNum></w:numbering>"#.as_slice(),
        ] {
            assert!(matches!(
                CT_Numbering::from_xml(xml),
                Err(OxmlError::MissingElement(_))
            ));
        }
    }

    #[test]
    fn rejects_numbering_levels_above_the_ooxml_limit() {
        for xml in [
            br#"<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:abstractNum w:abstractNumId="1"><w:lvl w:ilvl="9"/></w:abstractNum></w:numbering>"#.as_slice(),
            br#"<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:abstractNum w:abstractNumId="1"/><w:num w:numId="1"><w:abstractNumId w:val="1"/><w:lvlOverride w:ilvl="9"/></w:num></w:numbering>"#.as_slice(),
            br#"<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:abstractNum w:abstractNumId="1"/><w:num w:numId="1"><w:abstractNumId w:val="1"/><w:lvlOverride w:ilvl="0"><w:lvl w:ilvl="9"></w:lvl></w:lvlOverride></w:num></w:numbering>"#.as_slice(),
        ] {
            assert!(matches!(
                CT_Numbering::from_xml(xml),
                Err(OxmlError::InvalidValue(_))
            ));
        }
    }

    #[test]
    fn numbering_levels_report_unmodeled_semantics() {
        let xml = br#"<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
          <w:abstractNum w:abstractNumId="1">
            <w:lvl w:ilvl="0"><w:numFmt w:val="decimal"/><w:lvlRestart w:val="0"/></w:lvl>
            <w:lvl w:ilvl="1" w:vendor="x"><w:numFmt w:val="decimal"/></w:lvl>
            <w:lvl w:ilvl="2"><w:numFmt w:val="decimal" w:vendor="x"/></w:lvl>
            <w:lvl w:ilvl="3"><w:numFmt w:val="decimal"/></w:lvl>
            <w:lvl w:ilvl="4"><w:pPr/><w:rPr/></w:lvl>
          </w:abstractNum>
          <w:num w:numId="1"><w:abstractNumId w:val="1"/></w:num>
        </w:numbering>"#;

        let numbering = CT_Numbering::from_xml(xml).unwrap();
        for level in 0..=2 {
            assert!(
                numbering
                    .get_effective_level(1, level)
                    .unwrap()
                    .has_unmodeled_properties
            );
        }
        assert!(
            !numbering
                .get_effective_level(1, 3)
                .unwrap()
                .has_unmodeled_properties
        );
        assert!(
            !numbering
                .get_effective_level(1, 4)
                .unwrap()
                .has_unmodeled_properties
        );
    }

    #[test]
    fn get_abstract_num_for_lookup() {
        let mut numbering = CT_Numbering::new();
        numbering.add_bullet_list();
        numbering.add_numbered_list();

        let abs = numbering.get_abstract_num_for(2).unwrap();
        assert_eq!(abs.levels[0].num_fmt, Some(ST_NumberFormat::Decimal));

        assert!(numbering.get_abstract_num_for(99).is_none());
    }

    #[test]
    fn add_list_mixed_levels_round_trip() {
        let mut numbering = CT_Numbering::new();
        let num_id = numbering.add_list(&[
            (ST_NumberFormat::Bullet, None),
            (ST_NumberFormat::Decimal, Some(3)),
        ]);
        assert_eq!(num_id, 1);

        let xml = numbering.to_xml().unwrap();
        let parsed = CT_Numbering::from_xml(&xml).unwrap();

        let abs = parsed.get_abstract_num_for(num_id).unwrap();
        assert_eq!(abs.levels.len(), 9);
        assert_eq!(abs.levels[0].num_fmt, Some(ST_NumberFormat::Bullet));
        assert_eq!(abs.levels[0].lvl_text, Some("\u{2022}".to_string()));
        assert_eq!(abs.levels[1].num_fmt, Some(ST_NumberFormat::Decimal));
        assert_eq!(abs.levels[1].lvl_text, Some("%2.".to_string()));
        assert_eq!(abs.levels[1].start, Some(3));
        // Unspecified levels continue the last specified family (numeric).
        assert_eq!(abs.levels[2].num_fmt, Some(ST_NumberFormat::LowerRoman));
        assert_eq!(abs.levels[2].start, Some(1));
    }

    #[test]
    fn add_list_fill_keeps_bullets_for_bullet_lists() {
        let mut numbering = CT_Numbering::new();
        let num_id = numbering.add_list(&[(ST_NumberFormat::Bullet, None)]);

        let abs = numbering.get_abstract_num_for(num_id).unwrap();
        for level in &abs.levels {
            assert_eq!(level.num_fmt, Some(ST_NumberFormat::Bullet));
        }
    }

    #[test]
    fn add_list_delegation_matches_legacy_templates() {
        let mut via_helpers = CT_Numbering::new();
        via_helpers.add_bullet_list();
        via_helpers.add_numbered_list();

        let mut via_add_list = CT_Numbering::new();
        via_add_list.add_list(&[(ST_NumberFormat::Bullet, Some(1))]);
        via_add_list.add_list(&[(ST_NumberFormat::Decimal, Some(1))]);

        assert_eq!(via_helpers.abstract_nums, via_add_list.abstract_nums);
        assert_eq!(via_helpers.nums, via_add_list.nums);
    }

    #[test]
    fn set_list_level_redefines_one_level() {
        let mut numbering = CT_Numbering::new();
        let num_id = numbering.add_list(&[(ST_NumberFormat::Bullet, None)]);

        assert!(numbering.set_list_level(num_id, 1, ST_NumberFormat::Decimal, Some(3)));

        let abs = numbering.get_abstract_num_for(num_id).unwrap();
        assert_eq!(abs.levels[1].num_fmt, Some(ST_NumberFormat::Decimal));
        assert_eq!(abs.levels[1].lvl_text, Some("%2.".to_string()));
        assert_eq!(abs.levels[1].start, Some(3));
        // Neighbors untouched.
        assert_eq!(abs.levels[0].num_fmt, Some(ST_NumberFormat::Bullet));
        assert_eq!(abs.levels[2].num_fmt, Some(ST_NumberFormat::Bullet));
    }

    #[test]
    fn set_list_level_rejects_unknown_targets() {
        let mut numbering = CT_Numbering::new();
        let num_id = numbering.add_list(&[(ST_NumberFormat::Bullet, None)]);

        assert!(!numbering.set_list_level(99, 0, ST_NumberFormat::Decimal, None));
        assert!(!numbering.set_list_level(num_id, 9, ST_NumberFormat::Decimal, None));
    }

    #[test]
    fn numbering_requires_word_namespaces_and_qualified_attributes() {
        let xml = format!(
            r#"<z:numbering xmlns:z="{W_NS}" xmlns:x="urn:foreign">
              <x:abstractNum z:abstractNumId="4"/>
              <z:abstractNum abstractNumId="5"/>
              <z:abstractNum z:abstractNumId="6"/>
            </z:numbering>"#
        );

        assert!(matches!(
            CT_Numbering::from_xml(xml.as_bytes()),
            Err(OxmlError::MissingElement(_))
        ));

        let accepted = format!(
            r#"<z:numbering xmlns:z="{W_NS}"><z:abstractNum z:abstractNumId="6"/></z:numbering>"#
        );
        let numbering = CT_Numbering::from_xml(accepted.as_bytes()).unwrap();
        assert_eq!(numbering.abstract_nums[0].abstract_num_id, 6);
    }

    #[test]
    fn truncated_numbering_is_rejected() {
        let xml = format!(r#"<w:numbering xmlns:w="{W_NS}"><w:abstractNum w:abstractNumId="0">"#);
        assert!(CT_Numbering::from_xml(xml.as_bytes()).is_err());
    }
}
