//! Numbering definitions: `CT_Numbering`, `CT_AbstractNum`, `CT_Num`, `CT_Lvl`.
//!
//! These types represent the content of `numbering.xml`, which defines
//! abstract numbering formats and numbering instances that paragraphs reference.

use std::collections::HashSet;
use std::io::Write;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::{OxmlError, Result};
use crate::namespace::{W_NS, matches_local_name};
use crate::properties::{CT_PPr, CT_RPr, get_val_attr};
use crate::raw_xml::{capture_element, capture_empty_element};
use crate::shared::ST_Jc;
use crate::styles::{CT_Styles, StyleType};

fn required_u32_attr(element: &BytesStart<'_>, local: &[u8], path: &str) -> Result<u32> {
    for attribute in element.attributes() {
        let attribute = attribute?;
        if matches_local_name(attribute.key.as_ref(), local) {
            return Ok(std::str::from_utf8(&attribute.value)?.parse()?);
        }
    }
    Err(OxmlError::MissingElement(path.to_string()))
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

    pub fn from_xml(reader: &mut Reader<&[u8]>, ilvl: u32) -> Result<Self> {
        let mut lvl = CT_Lvl::new(ilvl);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    if matches_local_name(name.as_ref(), b"pPr") {
                        lvl.ppr = Some(CT_PPr::from_xml(reader)?);
                    } else if matches_local_name(name.as_ref(), b"rPr") {
                        lvl.rpr = Some(CT_RPr::from_xml(reader)?);
                    } else {
                        reader.read_to_end_into(name, &mut Vec::new())?;
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    if matches_local_name(name.as_ref(), b"start") {
                        if let Some(val) = get_val_attr(e)? {
                            lvl.start = Some(val.parse()?);
                        }
                    } else if matches_local_name(name.as_ref(), b"numFmt") {
                        if let Some(val) = get_val_attr(e)? {
                            lvl.num_fmt = Some(ST_NumberFormat::from_str(&val));
                        }
                    } else if matches_local_name(name.as_ref(), b"lvlText") {
                        lvl.lvl_text = get_val_attr(e)?;
                    } else if matches_local_name(name.as_ref(), b"pStyle") {
                        lvl.p_style = get_val_attr(e)?;
                    } else if matches_local_name(name.as_ref(), b"lvlJc")
                        && let Some(val) = get_val_attr(e)?
                    {
                        lvl.lvl_jc = Some(ST_Jc::from_str(&val)?);
                    }
                }
                Ok(Event::End(ref e)) if matches_local_name(e.name().as_ref(), b"lvl") => {
                    break;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(lvl)
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

/// `CT_AbstractNum` — An abstract numbering definition with up to 9 levels.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_AbstractNum {
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
            abstract_num_id: id,
            levels: Vec::new(),
            multi_level_type: None,
            style_link: None,
            num_style_link: None,
            extra_xml: Vec::new(),
        }
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>, abstract_num_id: u32) -> Result<Self> {
        let mut abs = CT_AbstractNum::new(abstract_num_id);
        let mut modelled_index = 0;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    if matches_local_name(name.as_ref(), b"lvl") {
                        let ilvl = required_u32_attr(e, b"ilvl", "w:lvl/@w:ilvl")?;
                        abs.levels.push(CT_Lvl::from_xml(reader, ilvl)?);
                        modelled_index += 1;
                    } else {
                        abs.extra_xml
                            .push((modelled_index, capture_element(reader, e)?));
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    if matches_local_name(name.as_ref(), b"multiLevelType") {
                        abs.multi_level_type = get_val_attr(e)?;
                        modelled_index += 1;
                    } else if matches_local_name(name.as_ref(), b"styleLink")
                        && abs.style_link.is_none()
                    {
                        abs.style_link = get_val_attr(e)?;
                        modelled_index += 1;
                    } else if matches_local_name(name.as_ref(), b"numStyleLink")
                        && abs.num_style_link.is_none()
                    {
                        abs.num_style_link = get_val_attr(e)?;
                        modelled_index += 1;
                    } else if matches_local_name(name.as_ref(), b"lvl") {
                        let ilvl = required_u32_attr(e, b"ilvl", "w:lvl/@w:ilvl")?;
                        abs.levels.push(CT_Lvl::new(ilvl));
                        modelled_index += 1;
                    } else {
                        abs.extra_xml
                            .push((modelled_index, capture_empty_element(e)?));
                    }
                }
                Ok(Event::End(ref e)) if matches_local_name(e.name().as_ref(), b"abstractNum") => {
                    break;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(abs)
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

/// `CT_Num` — A numbering instance that references an abstract numbering definition.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Num {
    pub num_id: u32,
    pub abstract_num_id: u32,
    pub level_overrides: Vec<CT_LvlOverride>,
}

/// An instance-specific numbering-level override.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_LvlOverride {
    pub ilvl: u32,
    pub start_override: Option<u32>,
    pub level: Option<CT_Lvl>,
}

impl CT_LvlOverride {
    fn from_xml(reader: &mut Reader<&[u8]>, ilvl: u32) -> Result<Self> {
        let mut value = Self {
            ilvl,
            start_override: None,
            level: None,
        };
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if matches_local_name(e.name().as_ref(), b"lvl") => {
                    let level_index = required_u32_attr(e, b"ilvl", "w:lvl/@w:ilvl")?;
                    value.level = Some(CT_Lvl::from_xml(reader, level_index)?);
                }
                Ok(Event::Start(ref e)) => {
                    reader.read_to_end_into(e.name(), &mut Vec::new())?;
                }
                Ok(Event::Empty(ref e))
                    if matches_local_name(e.name().as_ref(), b"startOverride") =>
                {
                    if let Some(start) = get_val_attr(e)? {
                        value.start_override = Some(start.parse()?);
                    }
                }
                Ok(Event::End(ref e)) if matches_local_name(e.name().as_ref(), b"lvlOverride") => {
                    break;
                }
                Ok(Event::Eof) => break,
                Err(error) => return Err(error.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(value)
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

#[allow(non_snake_case)]
impl CT_Num {
    pub fn from_xml(reader: &mut Reader<&[u8]>, num_id: u32) -> Result<Self> {
        let mut abstract_num_id = None;
        let mut level_overrides = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) => {
                    if matches_local_name(e.name().as_ref(), b"abstractNumId")
                        && let Some(val) = get_val_attr(e)?
                    {
                        abstract_num_id = Some(val.parse()?);
                    }
                }
                Ok(Event::Start(ref e)) => {
                    if matches_local_name(e.name().as_ref(), b"lvlOverride") {
                        let ilvl = required_u32_attr(e, b"ilvl", "w:lvlOverride/@w:ilvl")?;
                        level_overrides.push(CT_LvlOverride::from_xml(reader, ilvl)?);
                    } else {
                        reader.read_to_end_into(e.name(), &mut Vec::new())?;
                    }
                }
                Ok(Event::End(ref e)) if matches_local_name(e.name().as_ref(), b"num") => {
                    break;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(CT_Num {
            num_id,
            abstract_num_id: abstract_num_id.ok_or_else(|| {
                OxmlError::MissingElement("w:num/w:abstractNumId/@w:val".to_string())
            })?,
            level_overrides,
        })
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

/// `CT_Numbering` — Root element of the numbering definitions part.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Numbering {
    pub abstract_nums: Vec<CT_AbstractNum>,
    pub nums: Vec<CT_Num>,
}

#[allow(non_snake_case)]
impl CT_Numbering {
    pub fn new() -> Self {
        CT_Numbering {
            abstract_nums: Vec::new(),
            nums: Vec::new(),
        }
    }

    /// Parse from XML bytes (the content of numbering.xml).
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        reader.config_mut().trim_text(true);

        let mut abstract_nums = Vec::new();
        let mut nums = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    if matches_local_name(name.as_ref(), b"abstractNum") {
                        let id = required_u32_attr(
                            e,
                            b"abstractNumId",
                            "w:abstractNum/@w:abstractNumId",
                        )?;
                        abstract_nums.push(CT_AbstractNum::from_xml(&mut reader, id)?);
                    } else if matches_local_name(name.as_ref(), b"num") {
                        let id = required_u32_attr(e, b"numId", "w:num/@w:numId")?;
                        nums.push(CT_Num::from_xml(&mut reader, id)?);
                    } else if matches_local_name(name.as_ref(), b"numbering") {
                        // root element, continue
                    } else {
                        reader.read_to_end_into(name, &mut Vec::new())?;
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    if matches_local_name(e.name().as_ref(), b"abstractNum") {
                        let id = required_u32_attr(
                            e,
                            b"abstractNumId",
                            "w:abstractNum/@w:abstractNumId",
                        )?;
                        abstract_nums.push(CT_AbstractNum::new(id));
                    } else if matches_local_name(e.name().as_ref(), b"num") {
                        required_u32_attr(e, b"numId", "w:num/@w:numId")?;
                        return Err(OxmlError::MissingElement(
                            "w:num/w:abstractNumId/@w:val".to_string(),
                        ));
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(CT_Numbering {
            abstract_nums,
            nums,
        })
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

        Some(EffectiveNumberingLevel { level, start })
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
            return Some(EffectiveNumberingLevel { level, start });
        }

        let inherited =
            if let Some(level) = abstract_num.levels.iter().find(|level| level.ilvl == ilvl) {
                EffectiveNumberingLevel {
                    level,
                    start: level.start.unwrap_or(1),
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
}
