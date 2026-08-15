//! Paragraph properties (`CT_PPr`) and run properties (`CT_RPr`).

use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use oxml_core::xml::{
    StrictXmlCompleteness, StrictXmlCursor, StrictXmlElement, StrictXmlNode, StrictXmlParsed,
    parse_empty_started_element, parse_reader_element,
};

use crate::borders::{CT_PBdr, CT_Tabs};
use crate::document::CT_SectPr;
use crate::error::{OxmlError, Result};
use crate::namespace::W_NS;
#[cfg(test)]
use crate::namespace::matches_local_name;
use crate::raw_xml::NamespaceContext;
use crate::shared::{ST_HighlightColor, ST_Jc, ST_OnOff, ST_Underline};
use crate::units::{HalfPoint, Twips};

/// `CT_Shd` — Shading/background fill.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Shd {
    /// Shading pattern (e.g. "clear", "solid", "horzStripe")
    pub val: String,
    /// Foreground color hex
    pub color: Option<String>,
    /// Background fill color hex
    pub fill: Option<String>,
}

impl CT_Shd {
    pub fn from_xml_attrs(e: &BytesStart) -> Result<Self> {
        let element = parse_empty_started_element(&NamespaceContext::default(), Some(W_NS), e)?;
        Ok(Self::from_strict_xml(element)?.value)
    }

    pub(crate) fn from_strict_xml(element: StrictXmlElement) -> Result<StrictXmlParsed<Self>> {
        element.parse(|cursor| {
            Ok(Self {
                val: cursor
                    .take_attribute(Some(W_NS), "val")
                    .unwrap_or_else(|| "clear".to_string()),
                color: cursor.take_attribute(Some(W_NS), "color"),
                fill: cursor.take_attribute(Some(W_NS), "fill"),
            })
        })
    }

    pub fn write_xml<W: std::io::Write>(&self, writer: &mut Writer<W>, tag: &str) -> Result<()> {
        let mut e = BytesStart::new(tag);
        e.push_attribute(("w:val", self.val.as_str()));
        if let Some(ref c) = self.color {
            e.push_attribute(("w:color", c.as_str()));
        }
        if let Some(ref f) = self.fill {
            e.push_attribute(("w:fill", f.as_str()));
        }
        writer.write_event(Event::Empty(e))?;
        Ok(())
    }
}

/// `CT_PPr` — Paragraph properties.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CT_PPr {
    #[doc(hidden)]
    pub completeness: StrictXmlCompleteness,
    /// Paragraph style ID (pStyle)
    pub style_id: Option<String>,
    /// Justification (jc)
    pub jc: Option<ST_Jc>,
    /// Space before paragraph in twips (spacing/@w:before)
    pub space_before: Option<Twips>,
    /// Space after paragraph in twips (spacing/@w:after)
    pub space_after: Option<Twips>,
    /// Line spacing in twips (spacing/@w:line)
    pub line_spacing: Option<Twips>,
    /// Line spacing rule (spacing/@w:lineRule): "auto", "exact", "atLeast"
    pub line_rule: Option<String>,
    /// Space before auto-spacing (spacing/@w:beforeAutospacing)
    pub before_autospacing: Option<bool>,
    /// Space after auto-spacing (spacing/@w:afterAutospacing)
    pub after_autospacing: Option<bool>,
    /// Left indentation in twips (ind/@w:left)
    pub ind_left: Option<Twips>,
    /// Right indentation in twips (ind/@w:right)
    pub ind_right: Option<Twips>,
    /// First line indent in twips (ind/@w:firstLine)
    pub ind_first_line: Option<Twips>,
    /// Hanging indent in twips (ind/@w:hanging)
    pub ind_hanging: Option<Twips>,
    /// Keep with next paragraph (keepNext)
    pub keep_next: Option<bool>,
    /// Keep lines together (keepLines)
    pub keep_lines: Option<bool>,
    /// Page break before (pageBreakBefore)
    pub page_break_before: Option<bool>,
    /// Widow/orphan control (widowControl)
    pub widow_control: Option<bool>,
    /// Suppress auto-hyphens (suppressAutoHyphens)
    pub suppress_auto_hyphens: Option<bool>,
    /// Outline level 0-8 (outlineLvl)
    pub outline_lvl: Option<u32>,
    /// Paragraph borders (pBdr)
    pub borders: Option<CT_PBdr>,
    /// Tab stops (tabs)
    pub tabs: Option<CT_Tabs>,
    /// Paragraph shading (shd)
    pub shading: Option<CT_Shd>,
    /// Run properties for the paragraph mark (rPr)
    pub rpr: Option<CT_RPr>,
    /// Numbering level (numPr/ilvl)
    pub num_ilvl: Option<u32>,
    /// Numbering ID (numPr/numId)
    pub num_id: Option<u32>,
    /// Section properties embedded in paragraph (section break)
    pub sect_pr: Option<CT_SectPr>,
}

#[allow(non_snake_case)]
impl CT_PPr {
    pub(crate) fn from_strict_xml(element: StrictXmlElement) -> Result<Self> {
        let mut descendants = Vec::new();
        let parsed = element.parse(|cursor| {
            let mut properties = Self::default();
            for index in 0..cursor.child_slots() {
                let Some(StrictXmlNode::Element(child)) = cursor.child(index) else {
                    continue;
                };
                let recognized = [
                    "rPr",
                    "numPr",
                    "pBdr",
                    "tabs",
                    "sectPr",
                    "pStyle",
                    "jc",
                    "spacing",
                    "ind",
                    "keepNext",
                    "keepLines",
                    "pageBreakBefore",
                    "widowControl",
                    "suppressAutoHyphens",
                    "outlineLvl",
                    "shd",
                ]
                .into_iter()
                .find(|local| child.is_named(Some(W_NS), local));
                let Some(local) = recognized else {
                    continue;
                };
                let child = take_element_child(cursor, index, local)?;
                let completeness = match local {
                    "rPr" => {
                        let run_properties = CT_RPr::from_strict_xml(child)?;
                        let completeness = run_properties.completeness.clone();
                        properties.rpr = Some(run_properties);
                        completeness
                    }
                    "numPr" => properties.parse_strict_numbering(child)?,
                    "pBdr" => {
                        let borders = CT_PBdr::from_strict_xml(child)?;
                        let completeness = borders.completeness.clone();
                        properties.borders = Some(borders);
                        completeness
                    }
                    "tabs" => {
                        let tabs = CT_Tabs::from_strict_xml(child)?;
                        let completeness = tabs.completeness.clone();
                        properties.tabs = Some(tabs);
                        completeness
                    }
                    "sectPr" => {
                        let section = CT_SectPr::from_strict_xml(child)?;
                        let completeness = section.completeness.clone();
                        properties.sect_pr = Some(section);
                        completeness
                    }
                    _ => properties.parse_strict_property(local, child)?,
                };
                descendants.push(completeness);
            }
            Ok(properties)
        })?;
        let (mut properties, leftovers) = parsed.into_parts();
        properties.completeness = StrictXmlCompleteness::new(leftovers, descendants);
        Ok(properties)
    }

    fn parse_strict_property(
        &mut self,
        local: &str,
        element: StrictXmlElement,
    ) -> Result<StrictXmlCompleteness> {
        if local == "shd" {
            let parsed = CT_Shd::from_strict_xml(element)?;
            let (shading, leftovers) = parsed.into_parts();
            self.shading = Some(shading);
            return Ok(StrictXmlCompleteness::from_leftovers(leftovers));
        }

        let parsed = element.parse(|cursor| {
            match local {
                "pStyle" => self.style_id = cursor.take_attribute(Some(W_NS), "val"),
                "jc" => {
                    self.jc = cursor
                        .take_attribute(Some(W_NS), "val")
                        .map(|value| ST_Jc::from_str(&value))
                        .transpose()?;
                }
                "spacing" => {
                    self.space_before = take_twips_attribute(cursor, "before")?;
                    self.space_after = take_twips_attribute(cursor, "after")?;
                    self.line_spacing = take_twips_attribute(cursor, "line")?;
                    self.line_rule = cursor.take_attribute(Some(W_NS), "lineRule");
                    self.before_autospacing = cursor
                        .take_attribute(Some(W_NS), "beforeAutospacing")
                        .map(|value| value == "1" || value == "true");
                    self.after_autospacing = cursor
                        .take_attribute(Some(W_NS), "afterAutospacing")
                        .map(|value| value == "1" || value == "true");
                }
                "ind" => {
                    self.ind_left = take_twips_attribute(cursor, "left")?
                        .or(take_twips_attribute(cursor, "start")?);
                    self.ind_right = take_twips_attribute(cursor, "right")?
                        .or(take_twips_attribute(cursor, "end")?);
                    self.ind_first_line = take_twips_attribute(cursor, "firstLine")?;
                    self.ind_hanging = take_twips_attribute(cursor, "hanging")?;
                }
                "keepNext" => self.keep_next = Some(parse_strict_toggle(cursor)),
                "keepLines" => self.keep_lines = Some(parse_strict_toggle(cursor)),
                "pageBreakBefore" => {
                    self.page_break_before = Some(parse_strict_toggle(cursor));
                }
                "widowControl" => self.widow_control = Some(parse_strict_toggle(cursor)),
                "suppressAutoHyphens" => {
                    self.suppress_auto_hyphens = Some(parse_strict_toggle(cursor));
                }
                "outlineLvl" => {
                    self.outline_lvl = cursor
                        .take_attribute(Some(W_NS), "val")
                        .map(|value| value.parse())
                        .transpose()?;
                }
                _ => unreachable!(),
            }
            Ok(())
        })?;
        Ok(StrictXmlCompleteness::from_leftovers(parsed.leftovers))
    }

    fn parse_strict_numbering(
        &mut self,
        element: StrictXmlElement,
    ) -> Result<StrictXmlCompleteness> {
        let mut descendants = Vec::new();
        let parsed = element.parse(|cursor| {
            for index in 0..cursor.child_slots() {
                let Some(StrictXmlNode::Element(child)) = cursor.child(index) else {
                    continue;
                };
                let local = if child.is_named(Some(W_NS), "ilvl") {
                    Some("ilvl")
                } else if child.is_named(Some(W_NS), "numId") {
                    Some("numId")
                } else {
                    None
                };
                let Some(local) = local else {
                    continue;
                };
                let child = take_element_child(cursor, index, local)?;
                let parsed_value = child.parse(|cursor| {
                    cursor
                        .take_attribute(Some(W_NS), "val")
                        .map(|value| value.parse())
                        .transpose()
                        .map_err(Into::into)
                })?;
                let (value, leftovers) = parsed_value.into_parts();
                if local == "ilvl" {
                    self.num_ilvl = value;
                } else {
                    self.num_id = value;
                }
                descendants.push(StrictXmlCompleteness::from_leftovers(leftovers));
            }
            Ok(())
        })?;
        Ok(StrictXmlCompleteness::new(parsed.leftovers, descendants))
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
        let element = parse_reader_element(reader, context, Some(W_NS), "pPr", [])?;
        Self::from_strict_xml(element)
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        if self.is_empty() {
            return Ok(());
        }

        let mut buf = itoa::Buffer::new();
        writer.write_event(Event::Start(BytesStart::new("w:pPr")))?;

        if let Some(ref style_id) = self.style_id {
            let mut e = BytesStart::new("w:pStyle");
            e.push_attribute(("w:val", style_id.as_str()));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(keep_next) = self.keep_next {
            write_toggle(writer, "w:keepNext", keep_next)?;
        }
        if let Some(keep_lines) = self.keep_lines {
            write_toggle(writer, "w:keepLines", keep_lines)?;
        }
        if let Some(page_break) = self.page_break_before {
            write_toggle(writer, "w:pageBreakBefore", page_break)?;
        }
        if let Some(widow) = self.widow_control {
            write_toggle(writer, "w:widowControl", widow)?;
        }
        if let Some(suppress) = self.suppress_auto_hyphens {
            write_toggle(writer, "w:suppressAutoHyphens", suppress)?;
        }

        // pBdr
        if let Some(ref borders) = self.borders
            && !borders.is_empty()
        {
            borders.to_xml(writer)?;
        }

        // shd
        if let Some(ref shd) = self.shading {
            shd.write_xml(writer, "w:shd")?;
        }

        // tabs
        if let Some(ref tabs) = self.tabs {
            tabs.to_xml(writer)?;
        }

        // numPr
        if self.num_id.is_some() || self.num_ilvl.is_some() {
            writer.write_event(Event::Start(BytesStart::new("w:numPr")))?;
            if let Some(ilvl) = self.num_ilvl {
                let mut e = BytesStart::new("w:ilvl");
                e.push_attribute(("w:val", buf.format(ilvl)));
                writer.write_event(Event::Empty(e))?;
            }
            if let Some(num_id) = self.num_id {
                let mut e = BytesStart::new("w:numId");
                e.push_attribute(("w:val", buf.format(num_id)));
                writer.write_event(Event::Empty(e))?;
            }
            writer.write_event(Event::End(BytesEnd::new("w:numPr")))?;
        }

        // spacing
        if self.space_before.is_some()
            || self.space_after.is_some()
            || self.line_spacing.is_some()
            || self.before_autospacing.is_some()
            || self.after_autospacing.is_some()
        {
            let mut e = BytesStart::new("w:spacing");
            if let Some(before) = self.space_before {
                e.push_attribute(("w:before", buf.format(before.0)));
            }
            if let Some(after) = self.space_after {
                e.push_attribute(("w:after", buf.format(after.0)));
            }
            if let Some(line) = self.line_spacing {
                e.push_attribute(("w:line", buf.format(line.0)));
            }
            if let Some(ref rule) = self.line_rule {
                e.push_attribute(("w:lineRule", rule.as_str()));
            }
            if let Some(ba) = self.before_autospacing {
                e.push_attribute(("w:beforeAutospacing", if ba { "1" } else { "0" }));
            }
            if let Some(aa) = self.after_autospacing {
                e.push_attribute(("w:afterAutospacing", if aa { "1" } else { "0" }));
            }
            writer.write_event(Event::Empty(e))?;
        }

        // ind
        if self.ind_left.is_some()
            || self.ind_right.is_some()
            || self.ind_first_line.is_some()
            || self.ind_hanging.is_some()
        {
            let mut e = BytesStart::new("w:ind");
            if let Some(left) = self.ind_left {
                e.push_attribute(("w:left", buf.format(left.0)));
            }
            if let Some(right) = self.ind_right {
                e.push_attribute(("w:right", buf.format(right.0)));
            }
            if let Some(fl) = self.ind_first_line {
                e.push_attribute(("w:firstLine", buf.format(fl.0)));
            }
            if let Some(hang) = self.ind_hanging {
                e.push_attribute(("w:hanging", buf.format(hang.0)));
            }
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(jc) = self.jc {
            let mut e = BytesStart::new("w:jc");
            e.push_attribute(("w:val", jc.to_str()));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(lvl) = self.outline_lvl {
            let mut e = BytesStart::new("w:outlineLvl");
            e.push_attribute(("w:val", buf.format(lvl)));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref rpr) = self.rpr {
            rpr.to_xml(writer)?;
        }

        if let Some(ref sect) = self.sect_pr {
            sect.to_xml(writer)?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:pPr")))?;
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.style_id.is_none()
            && self.jc.is_none()
            && self.space_before.is_none()
            && self.space_after.is_none()
            && self.line_spacing.is_none()
            && self.before_autospacing.is_none()
            && self.after_autospacing.is_none()
            && self.ind_left.is_none()
            && self.ind_right.is_none()
            && self.ind_first_line.is_none()
            && self.ind_hanging.is_none()
            && self.keep_next.is_none()
            && self.keep_lines.is_none()
            && self.page_break_before.is_none()
            && self.widow_control.is_none()
            && self.suppress_auto_hyphens.is_none()
            && self.outline_lvl.is_none()
            && self.borders.is_none()
            && self.tabs.is_none()
            && self.shading.is_none()
            && self.rpr.is_none()
            && self.num_id.is_none()
            && self.num_ilvl.is_none()
            && self.sect_pr.is_none()
    }

    /// Merge another CT_PPr into this one (non-None fields override).
    /// Used for style inheritance.
    pub fn merge_from(&mut self, other: &CT_PPr) {
        self.completeness.extend(other.completeness.clone());
        if other.style_id.is_some() {
            self.style_id = other.style_id.clone();
        }
        if other.jc.is_some() {
            self.jc = other.jc;
        }
        if other.space_before.is_some() {
            self.space_before = other.space_before;
        }
        if other.space_after.is_some() {
            self.space_after = other.space_after;
        }
        if other.line_spacing.is_some() {
            self.line_spacing = other.line_spacing;
        }
        if other.line_rule.is_some() {
            self.line_rule = other.line_rule.clone();
        }
        if other.before_autospacing.is_some() {
            self.before_autospacing = other.before_autospacing;
        }
        if other.after_autospacing.is_some() {
            self.after_autospacing = other.after_autospacing;
        }
        if other.ind_left.is_some() {
            self.ind_left = other.ind_left;
        }
        if other.ind_right.is_some() {
            self.ind_right = other.ind_right;
        }
        if other.ind_first_line.is_some() {
            self.ind_first_line = other.ind_first_line;
        }
        if other.ind_hanging.is_some() {
            self.ind_hanging = other.ind_hanging;
        }
        if other.keep_next.is_some() {
            self.keep_next = other.keep_next;
        }
        if other.keep_lines.is_some() {
            self.keep_lines = other.keep_lines;
        }
        if other.page_break_before.is_some() {
            self.page_break_before = other.page_break_before;
        }
        if other.widow_control.is_some() {
            self.widow_control = other.widow_control;
        }
        if other.suppress_auto_hyphens.is_some() {
            self.suppress_auto_hyphens = other.suppress_auto_hyphens;
        }
        if other.outline_lvl.is_some() {
            self.outline_lvl = other.outline_lvl;
        }
        if other.borders.is_some() {
            self.borders = other.borders.clone();
        }
        if other.tabs.is_some() {
            self.tabs = other.tabs.clone();
        }
        if other.shading.is_some() {
            self.shading = other.shading.clone();
        }
        if other.num_ilvl.is_some() {
            self.num_ilvl = other.num_ilvl;
        }
        if other.num_id.is_some() {
            self.num_id = other.num_id;
        }
    }
}

/// `CT_RPr` — Run properties.
#[derive(Debug, Clone, Default, PartialEq)]
#[allow(non_snake_case)]
pub struct CT_RPr {
    #[doc(hidden)]
    pub completeness: StrictXmlCompleteness,
    /// Character style ID (rStyle)
    pub style_id: Option<String>,
    /// Font name for ASCII range (rFonts/@w:ascii)
    pub font_ascii: Option<String>,
    /// Font name for high-ANSI range (rFonts/@w:hAnsi)
    pub font_hansi: Option<String>,
    /// Font name for East Asian text (rFonts/@w:eastAsia)
    pub font_east_asia: Option<String>,
    /// Font name for complex script (rFonts/@w:cs)
    pub font_cs: Option<String>,
    /// Theme font for ASCII range (rFonts/@w:asciiTheme), e.g. "minorHAnsi", "majorHAnsi"
    pub font_ascii_theme: Option<String>,
    /// Theme font for hAnsi range (rFonts/@w:hAnsiTheme)
    pub font_hansi_theme: Option<String>,
    /// Bold (b)
    pub bold: Option<bool>,
    /// Bold complex script (bCs)
    pub bold_cs: Option<bool>,
    /// Italic (i)
    pub italic: Option<bool>,
    /// Italic complex script (iCs)
    pub italic_cs: Option<bool>,
    /// Underline type (u)
    pub underline: Option<ST_Underline>,
    /// Strikethrough (strike)
    pub strike: Option<bool>,
    /// Double strikethrough (dstrike)
    pub dstrike: Option<bool>,
    /// Font size in half-points (sz)
    pub sz: Option<HalfPoint>,
    /// Complex-script font size in half-points (szCs)
    pub sz_cs: Option<HalfPoint>,
    /// Text color as hex string, e.g. "FF0000" (color/@w:val)
    pub color: Option<String>,
    /// Color theme reference (color/@w:themeColor)
    pub color_theme: Option<String>,
    /// Highlight color (highlight)
    pub highlight: Option<ST_HighlightColor>,
    /// All caps (caps)
    pub caps: Option<bool>,
    /// Small caps (smallCaps)
    pub small_caps: Option<bool>,
    /// Superscript/subscript (vertAlign)
    pub vert_align: Option<String>,
    /// Character spacing in twips (spacing/@w:val)
    pub spacing: Option<Twips>,
    /// Character width scale in percent (w/@w:val)
    pub width_scale: Option<u32>,
    /// Text position (raised/lowered) in half-points (position/@w:val)
    pub position: Option<i32>,
    /// Run shading (shd)
    pub shading: Option<CT_Shd>,
    /// Vanish/hidden text (vanish)
    pub vanish: Option<bool>,
}

#[allow(non_snake_case)]
impl CT_RPr {
    pub(crate) fn from_strict_xml(element: StrictXmlElement) -> Result<Self> {
        let mut descendants = Vec::new();
        let parsed = element.parse(|cursor| {
            let mut properties = Self::default();
            for index in 0..cursor.child_slots() {
                let Some(StrictXmlNode::Element(child)) = cursor.child(index) else {
                    continue;
                };
                let recognized = [
                    "rStyle",
                    "rFonts",
                    "b",
                    "bCs",
                    "i",
                    "iCs",
                    "u",
                    "strike",
                    "dstrike",
                    "sz",
                    "szCs",
                    "color",
                    "highlight",
                    "caps",
                    "smallCaps",
                    "vertAlign",
                    "spacing",
                    "w",
                    "position",
                    "shd",
                    "vanish",
                ]
                .into_iter()
                .find(|local| child.is_named(Some(W_NS), local));
                let Some(local) = recognized else {
                    continue;
                };
                let child = take_element_child(cursor, index, local)?;
                let completeness = properties.parse_strict_property(local, child)?;
                descendants.push(completeness);
            }
            Ok(properties)
        })?;
        let (mut properties, leftovers) = parsed.into_parts();
        properties.completeness = StrictXmlCompleteness::new(leftovers, descendants);
        Ok(properties)
    }

    fn parse_strict_property(
        &mut self,
        local: &str,
        element: StrictXmlElement,
    ) -> Result<StrictXmlCompleteness> {
        if local == "shd" {
            let parsed = CT_Shd::from_strict_xml(element)?;
            let (shading, leftovers) = parsed.into_parts();
            self.shading = Some(shading);
            return Ok(StrictXmlCompleteness::from_leftovers(leftovers));
        }

        let parsed = element.parse(|cursor| {
            match local {
                "rStyle" => self.style_id = cursor.take_attribute(Some(W_NS), "val"),
                "rFonts" => {
                    self.font_ascii = cursor.take_attribute(Some(W_NS), "ascii");
                    self.font_hansi = cursor.take_attribute(Some(W_NS), "hAnsi");
                    self.font_east_asia = cursor.take_attribute(Some(W_NS), "eastAsia");
                    self.font_cs = cursor.take_attribute(Some(W_NS), "cs");
                    self.font_ascii_theme = cursor.take_attribute(Some(W_NS), "asciiTheme");
                    self.font_hansi_theme = cursor.take_attribute(Some(W_NS), "hAnsiTheme");
                }
                "b" => self.bold = Some(parse_strict_toggle(cursor)),
                "bCs" => self.bold_cs = Some(parse_strict_toggle(cursor)),
                "i" => self.italic = Some(parse_strict_toggle(cursor)),
                "iCs" => self.italic_cs = Some(parse_strict_toggle(cursor)),
                "u" => {
                    self.underline = Some(
                        cursor
                            .take_attribute(Some(W_NS), "val")
                            .map(|value| ST_Underline::from_str(&value))
                            .transpose()?
                            .unwrap_or(ST_Underline::Single),
                    );
                }
                "strike" => self.strike = Some(parse_strict_toggle(cursor)),
                "dstrike" => self.dstrike = Some(parse_strict_toggle(cursor)),
                "sz" => {
                    self.sz = cursor
                        .take_attribute(Some(W_NS), "val")
                        .map(|value| value.parse().map(HalfPoint))
                        .transpose()?;
                }
                "szCs" => {
                    self.sz_cs = cursor
                        .take_attribute(Some(W_NS), "val")
                        .map(|value| value.parse().map(HalfPoint))
                        .transpose()?;
                }
                "color" => {
                    self.color = cursor.take_attribute(Some(W_NS), "val");
                    self.color_theme = cursor.take_attribute(Some(W_NS), "themeColor");
                }
                "highlight" => {
                    self.highlight = cursor
                        .take_attribute(Some(W_NS), "val")
                        .map(|value| ST_HighlightColor::from_str(&value))
                        .transpose()?;
                }
                "caps" => self.caps = Some(parse_strict_toggle(cursor)),
                "smallCaps" => self.small_caps = Some(parse_strict_toggle(cursor)),
                "vertAlign" => self.vert_align = cursor.take_attribute(Some(W_NS), "val"),
                "spacing" => {
                    self.spacing = cursor
                        .take_attribute(Some(W_NS), "val")
                        .map(|value| value.parse().map(Twips))
                        .transpose()?;
                }
                "w" => {
                    self.width_scale = cursor
                        .take_attribute(Some(W_NS), "val")
                        .map(|value| value.parse())
                        .transpose()?;
                }
                "position" => {
                    self.position = cursor
                        .take_attribute(Some(W_NS), "val")
                        .map(|value| value.parse())
                        .transpose()?;
                }
                "vanish" => self.vanish = Some(parse_strict_toggle(cursor)),
                _ => unreachable!(),
            }
            Ok(())
        })?;
        Ok(StrictXmlCompleteness::from_leftovers(parsed.leftovers))
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
        let element = parse_reader_element(reader, context, Some(W_NS), "rPr", [])?;
        Self::from_strict_xml(element)
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        if self.is_empty() {
            return Ok(());
        }

        let mut buf = itoa::Buffer::new();
        writer.write_event(Event::Start(BytesStart::new("w:rPr")))?;

        if let Some(ref style_id) = self.style_id {
            let mut e = BytesStart::new("w:rStyle");
            e.push_attribute(("w:val", style_id.as_str()));
            writer.write_event(Event::Empty(e))?;
        }

        // rFonts
        if self.font_ascii.is_some()
            || self.font_hansi.is_some()
            || self.font_east_asia.is_some()
            || self.font_cs.is_some()
            || self.font_ascii_theme.is_some()
            || self.font_hansi_theme.is_some()
        {
            let mut e = BytesStart::new("w:rFonts");
            if let Some(ref f) = self.font_ascii {
                e.push_attribute(("w:ascii", f.as_str()));
            }
            if let Some(ref f) = self.font_hansi {
                e.push_attribute(("w:hAnsi", f.as_str()));
            }
            if let Some(ref f) = self.font_east_asia {
                e.push_attribute(("w:eastAsia", f.as_str()));
            }
            if let Some(ref f) = self.font_cs {
                e.push_attribute(("w:cs", f.as_str()));
            }
            if let Some(ref f) = self.font_ascii_theme {
                e.push_attribute(("w:asciiTheme", f.as_str()));
            }
            if let Some(ref f) = self.font_hansi_theme {
                e.push_attribute(("w:hAnsiTheme", f.as_str()));
            }
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(bold) = self.bold {
            write_toggle(writer, "w:b", bold)?;
        }
        if let Some(bold_cs) = self.bold_cs {
            write_toggle(writer, "w:bCs", bold_cs)?;
        }
        if let Some(italic) = self.italic {
            write_toggle(writer, "w:i", italic)?;
        }
        if let Some(italic_cs) = self.italic_cs {
            write_toggle(writer, "w:iCs", italic_cs)?;
        }
        if let Some(caps) = self.caps {
            write_toggle(writer, "w:caps", caps)?;
        }
        if let Some(small_caps) = self.small_caps {
            write_toggle(writer, "w:smallCaps", small_caps)?;
        }
        if let Some(vanish) = self.vanish {
            write_toggle(writer, "w:vanish", vanish)?;
        }
        if let Some(strike) = self.strike {
            write_toggle(writer, "w:strike", strike)?;
        }
        if let Some(dstrike) = self.dstrike {
            write_toggle(writer, "w:dstrike", dstrike)?;
        }

        if let Some(ref color) = self.color {
            let mut e = BytesStart::new("w:color");
            e.push_attribute(("w:val", color.as_str()));
            if let Some(ref tc) = self.color_theme {
                e.push_attribute(("w:themeColor", tc.as_str()));
            }
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref spacing) = self.spacing {
            let mut e = BytesStart::new("w:spacing");
            e.push_attribute(("w:val", buf.format(spacing.0)));
            writer.write_event(Event::Empty(e))?;
        }
        if let Some(ws) = self.width_scale {
            let mut e = BytesStart::new("w:w");
            e.push_attribute(("w:val", buf.format(ws)));
            writer.write_event(Event::Empty(e))?;
        }
        if let Some(pos) = self.position {
            let mut e = BytesStart::new("w:position");
            e.push_attribute(("w:val", buf.format(pos)));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref sz) = self.sz {
            let mut e = BytesStart::new("w:sz");
            e.push_attribute(("w:val", buf.format(sz.0)));
            writer.write_event(Event::Empty(e))?;
        }
        if let Some(ref sz_cs) = self.sz_cs {
            let mut e = BytesStart::new("w:szCs");
            e.push_attribute(("w:val", buf.format(sz_cs.0)));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(underline) = self.underline {
            let mut e = BytesStart::new("w:u");
            e.push_attribute(("w:val", underline.to_str()));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref highlight) = self.highlight {
            let mut e = BytesStart::new("w:highlight");
            e.push_attribute(("w:val", highlight.to_str()));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref vert_align) = self.vert_align {
            let mut e = BytesStart::new("w:vertAlign");
            e.push_attribute(("w:val", vert_align.as_str()));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref shd) = self.shading {
            shd.write_xml(writer, "w:shd")?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:rPr")))?;
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.style_id.is_none()
            && self.font_ascii.is_none()
            && self.font_hansi.is_none()
            && self.font_east_asia.is_none()
            && self.font_cs.is_none()
            && self.font_ascii_theme.is_none()
            && self.font_hansi_theme.is_none()
            && self.bold.is_none()
            && self.bold_cs.is_none()
            && self.italic.is_none()
            && self.italic_cs.is_none()
            && self.underline.is_none()
            && self.strike.is_none()
            && self.dstrike.is_none()
            && self.sz.is_none()
            && self.sz_cs.is_none()
            && self.color.is_none()
            && self.color_theme.is_none()
            && self.highlight.is_none()
            && self.caps.is_none()
            && self.small_caps.is_none()
            && self.vert_align.is_none()
            && self.spacing.is_none()
            && self.width_scale.is_none()
            && self.position.is_none()
            && self.shading.is_none()
            && self.vanish.is_none()
    }

    /// Merge another CT_RPr into this one (non-None fields override).
    /// Used for style inheritance.
    pub fn merge_from(&mut self, other: &CT_RPr) {
        self.completeness.extend(other.completeness.clone());
        if other.style_id.is_some() {
            self.style_id = other.style_id.clone();
        }
        if other.font_ascii.is_some() {
            self.font_ascii = other.font_ascii.clone();
        }
        if other.font_hansi.is_some() {
            self.font_hansi = other.font_hansi.clone();
        }
        if other.font_east_asia.is_some() {
            self.font_east_asia = other.font_east_asia.clone();
        }
        if other.font_cs.is_some() {
            self.font_cs = other.font_cs.clone();
        }
        if other.font_ascii_theme.is_some() {
            self.font_ascii_theme = other.font_ascii_theme.clone();
        }
        if other.font_hansi_theme.is_some() {
            self.font_hansi_theme = other.font_hansi_theme.clone();
        }
        if other.bold.is_some() {
            self.bold = other.bold;
        }
        if other.bold_cs.is_some() {
            self.bold_cs = other.bold_cs;
        }
        if other.italic.is_some() {
            self.italic = other.italic;
        }
        if other.italic_cs.is_some() {
            self.italic_cs = other.italic_cs;
        }
        if other.underline.is_some() {
            self.underline = other.underline;
        }
        if other.strike.is_some() {
            self.strike = other.strike;
        }
        if other.dstrike.is_some() {
            self.dstrike = other.dstrike;
        }
        if other.sz.is_some() {
            self.sz = other.sz;
        }
        if other.sz_cs.is_some() {
            self.sz_cs = other.sz_cs;
        }
        if other.color.is_some() {
            self.color = other.color.clone();
        }
        if other.color_theme.is_some() {
            self.color_theme = other.color_theme.clone();
        }
        if other.highlight.is_some() {
            self.highlight = other.highlight;
        }
        if other.caps.is_some() {
            self.caps = other.caps;
        }
        if other.small_caps.is_some() {
            self.small_caps = other.small_caps;
        }
        if other.vert_align.is_some() {
            self.vert_align = other.vert_align.clone();
        }
        if other.spacing.is_some() {
            self.spacing = other.spacing;
        }
        if other.width_scale.is_some() {
            self.width_scale = other.width_scale;
        }
        if other.position.is_some() {
            self.position = other.position;
        }
        if other.shading.is_some() {
            self.shading = other.shading.clone();
        }
        if other.vanish.is_some() {
            self.vanish = other.vanish;
        }
    }
}

/// Parse a toggle element (like `<w:b/>` or `<w:b w:val="false"/>`).

fn parse_strict_toggle(cursor: &mut StrictXmlCursor) -> bool {
    let value = cursor.take_attribute(Some(W_NS), "val");
    ST_OnOff::from_str_or_default(value.as_deref()).is_on()
}

fn take_twips_attribute(cursor: &mut StrictXmlCursor, local: &str) -> Result<Option<Twips>> {
    cursor
        .take_attribute(Some(W_NS), local)
        .map(|value| value.parse().map(Twips))
        .transpose()
        .map_err(Into::into)
}

fn take_element_child(
    cursor: &mut StrictXmlCursor,
    index: usize,
    local: &str,
) -> Result<StrictXmlElement> {
    cursor
        .take_child(index)
        .and_then(StrictXmlNode::into_element)
        .ok_or_else(|| OxmlError::MissingElement(format!("w:{local}")))
}

/// Write a toggle element.
pub(crate) fn write_toggle<W: std::io::Write>(
    writer: &mut Writer<W>,
    tag: &str,
    value: bool,
) -> Result<()> {
    if value {
        writer.write_event(Event::Empty(BytesStart::new(tag)))?;
    } else {
        let mut e = BytesStart::new(tag);
        e.push_attribute(("w:val", "false"));
        writer.write_event(Event::Empty(e))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ST_Border;

    fn parse_ppr(xml: &str) -> CT_PPr {
        let full = format!("<w:pPr>{xml}</w:pPr>");
        let mut reader = Reader::from_str(&full);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if matches_local_name(e.name().as_ref(), b"pPr") => break,
                _ => {}
            }
            buf.clear();
        }
        CT_PPr::from_xml(&mut reader).unwrap()
    }

    fn parse_rpr(xml: &str) -> CT_RPr {
        let full = format!("<w:rPr>{xml}</w:rPr>");
        let mut reader = Reader::from_str(&full);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if matches_local_name(e.name().as_ref(), b"rPr") => break,
                _ => {}
            }
            buf.clear();
        }
        CT_RPr::from_xml(&mut reader).unwrap()
    }

    #[test]
    fn parse_basic_ppr() {
        let ppr = parse_ppr(r#"<w:pStyle w:val="Heading1"/><w:jc w:val="center"/>"#);
        assert_eq!(ppr.style_id, Some("Heading1".to_string()));
        assert_eq!(ppr.jc, Some(ST_Jc::Center));
    }

    #[test]
    fn parse_spacing() {
        let ppr = parse_ppr(r#"<w:spacing w:before="240" w:after="120" w:line="360"/>"#);
        assert_eq!(ppr.space_before, Some(Twips(240)));
        assert_eq!(ppr.space_after, Some(Twips(120)));
        assert_eq!(ppr.line_spacing, Some(Twips(360)));
    }

    #[test]
    fn parse_borders() {
        let ppr = parse_ppr(
            r#"<w:pBdr><w:top w:val="single" w:sz="4" w:space="1" w:color="000000"/><w:bottom w:val="double" w:sz="6"/></w:pBdr>"#,
        );
        let borders = ppr.borders.unwrap();
        assert_eq!(borders.top.as_ref().unwrap().val, ST_Border::Single);
        assert_eq!(borders.top.as_ref().unwrap().sz, Some(4));
        assert_eq!(borders.bottom.as_ref().unwrap().val, ST_Border::Double);
        assert!(borders.left.is_none());
    }

    #[test]
    fn parse_tabs() {
        let ppr = parse_ppr(
            r#"<w:tabs><w:tab w:val="left" w:pos="720"/><w:tab w:val="right" w:pos="8640" w:leader="dot"/></w:tabs>"#,
        );
        let tabs = ppr.tabs.unwrap();
        assert_eq!(tabs.tabs.len(), 2);
        assert_eq!(tabs.tabs[0].pos, Twips(720));
        assert_eq!(tabs.tabs[1].leader, Some(crate::shared::ST_TabLeader::Dot));
    }

    #[test]
    fn parse_shading() {
        let ppr = parse_ppr(r#"<w:shd w:val="clear" w:fill="FFFF00"/>"#);
        let shd = ppr.shading.unwrap();
        assert_eq!(shd.val, "clear");
        assert_eq!(shd.fill, Some("FFFF00".to_string()));
    }

    #[test]
    fn parse_basic_rpr() {
        let rpr = parse_rpr(r#"<w:b/><w:i/><w:sz w:val="24"/><w:color w:val="FF0000"/>"#);
        assert_eq!(rpr.bold, Some(true));
        assert_eq!(rpr.italic, Some(true));
        assert_eq!(rpr.sz, Some(HalfPoint(24)));
        assert_eq!(rpr.color, Some("FF0000".to_string()));
    }

    #[test]
    fn expanded_numbering_and_hidden_properties_parse_like_empty_elements() {
        let ppr = parse_ppr(concat!(
            r#"<w:numPr><w:ilvl w:val="2"></w:ilvl>"#,
            r#"<w:numId w:val="7"></w:numId></w:numPr>"#,
        ));
        let rpr = parse_rpr(r#"<w:vanish></w:vanish>"#);

        assert_eq!(ppr.num_ilvl, Some(2));
        assert_eq!(ppr.num_id, Some(7));
        assert_eq!(rpr.vanish, Some(true));
    }

    #[test]
    fn unmodeled_word_properties_are_observable() {
        let ppr = parse_ppr(r#"<w:contextualSpacing/><x:ignored xmlns:x="urn:foreign"/>"#);
        let rpr = parse_rpr(r#"<w:rtl/><x:ignored xmlns:x="urn:foreign"/>"#);

        assert!(ppr.has_unmodeled_properties());
        assert!(rpr.has_unmodeled_properties());
    }

    #[test]
    fn extension_properties_make_the_semantic_container_incomplete() {
        let ppr = parse_ppr(r#"<x:paragraphEffect xmlns:x="urn:foreign"/>"#);
        let rpr = parse_rpr(
            r#"<w14:textOutline xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"/>"#,
        );

        assert!(ppr.has_unmodeled_properties());
        assert!(rpr.has_unmodeled_properties());
    }

    #[test]
    fn unmodeled_attributes_and_nested_properties_propagate_to_the_container() {
        let ppr = parse_ppr(concat!(
            r#"<w:spacing w:before="240" w:beforeLines="2"/>"#,
            r#"<w:pBdr><w:top w:val="single" w:shadow="true"/></w:pBdr>"#,
        ));
        let rpr = parse_rpr(r#"<w:color w:val="FF0000" w:themeTint="80"/>"#);

        assert_eq!(ppr.space_before, Some(Twips(240)));
        assert!(ppr.borders.as_ref().unwrap().top.is_some());
        assert!(ppr.has_unmodeled_properties());
        assert_eq!(rpr.color, Some("FF0000".to_string()));
        assert!(rpr.has_unmodeled_properties());
    }

    #[test]
    fn parse_rpr_spacing() {
        let rpr = parse_rpr(r#"<w:spacing w:val="20"/><w:w w:val="150"/><w:position w:val="-4"/>"#);
        assert_eq!(rpr.spacing, Some(Twips(20)));
        assert_eq!(rpr.width_scale, Some(150));
        assert_eq!(rpr.position, Some(-4));
    }

    #[test]
    fn round_trip_rpr() {
        let original = CT_RPr {
            bold: Some(true),
            italic: Some(true),
            sz: Some(HalfPoint(24)),
            color: Some("FF0000".to_string()),
            underline: Some(ST_Underline::Single),
            spacing: Some(Twips(20)),
            ..Default::default()
        };

        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        original.to_xml(&mut writer).unwrap();
        let xml = String::from_utf8(output).unwrap();

        let inner = xml
            .strip_prefix("<w:rPr>")
            .unwrap()
            .strip_suffix("</w:rPr>")
            .unwrap();
        let parsed = parse_rpr(inner);
        assert_eq!(parsed.bold, original.bold);
        assert_eq!(parsed.italic, original.italic);
        assert_eq!(parsed.sz, original.sz);
        assert_eq!(parsed.color, original.color);
        assert_eq!(parsed.underline, original.underline);
        assert_eq!(parsed.spacing, original.spacing);
    }

    #[test]
    fn merge_ppr() {
        let mut base = CT_PPr {
            jc: Some(ST_Jc::Left),
            space_after: Some(Twips(200)),
            ..Default::default()
        };
        let override_ppr = CT_PPr {
            jc: Some(ST_Jc::Center),
            space_before: Some(Twips(120)),
            ..Default::default()
        };
        base.merge_from(&override_ppr);
        assert_eq!(base.jc, Some(ST_Jc::Center)); // overridden
        assert_eq!(base.space_after, Some(Twips(200))); // kept
        assert_eq!(base.space_before, Some(Twips(120))); // added
    }

    #[test]
    fn merge_rpr() {
        let mut base = CT_RPr {
            bold: Some(true),
            sz: Some(HalfPoint(24)),
            ..Default::default()
        };
        let override_rpr = CT_RPr {
            sz: Some(HalfPoint(28)),
            italic: Some(true),
            ..Default::default()
        };
        base.merge_from(&override_rpr);
        assert_eq!(base.bold, Some(true)); // kept
        assert_eq!(base.sz, Some(HalfPoint(28))); // overridden
        assert_eq!(base.italic, Some(true)); // added
    }
}
