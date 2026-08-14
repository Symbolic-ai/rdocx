//! Table elements: `CT_Tbl`, `CT_Row`, `CT_Tc` and related types.

use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::borders::CT_BorderEdge;
use crate::error::{OxmlError, Result};
#[cfg(test)]
use crate::namespace::matches_local_name;
use crate::namespace::{
    has_unmodeled_attributes, matches_word_attribute, matches_word_element, matches_word_name,
};
use crate::properties::{CT_Shd, get_val_attr_with_context};
use crate::raw_xml::{
    NamespaceContext, RawXml, capture_element, capture_empty_element, capture_raw_element,
    capture_raw_empty_element,
};
#[cfg(test)]
use crate::shared::ST_Border;
use crate::shared::ST_Jc;
use crate::text::CT_P;
use crate::units::Twips;

const MAX_MODEL_DEPTH: usize = 32;

fn model_depth_error() -> OxmlError {
    OxmlError::InvalidValue(format!(
        "recognized model nesting exceeds {MAX_MODEL_DEPTH} levels"
    ))
}

/// Write any captured raw XML that belongs immediately before position `pos`.
///
/// Table children we do not model are stored as `(position, raw)` pairs so
/// they can be put back where they were found, the same way `CT_P` handles
/// its own unknown children.
fn write_extras_at<W: std::io::Write>(
    writer: &mut Writer<W>,
    extra_xml: &[(usize, Vec<u8>)],
    pos: usize,
) -> Result<()> {
    for (at, raw) in extra_xml {
        if *at == pos {
            writer.get_mut().write_all(raw)?;
        }
    }
    Ok(())
}

// ---- Table border types ----

/// `CT_TblBorders` — Table-level borders.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CT_TblBorders {
    pub top: Option<CT_BorderEdge>,
    pub bottom: Option<CT_BorderEdge>,
    pub left: Option<CT_BorderEdge>,
    pub right: Option<CT_BorderEdge>,
    pub inside_h: Option<CT_BorderEdge>,
    pub inside_v: Option<CT_BorderEdge>,
}

impl CT_TblBorders {
    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_context(reader, &NamespaceContext::default())
    }

    pub fn from_xml_with_context(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
    ) -> Result<Self> {
        Self::from_xml_with_context_and_completeness(reader, context).map(|(borders, _)| borders)
    }

    fn from_xml_with_context_and_completeness(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
    ) -> Result<(Self, bool)> {
        let mut borders = CT_TblBorders::default();
        let mut has_unmodeled_properties = false;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) => {
                    if !Self::parse_edge(e, context, &mut borders)? {
                        has_unmodeled_properties = true;
                    }
                }
                Ok(Event::Start(ref e)) => {
                    if !Self::parse_edge(e, context, &mut borders)? {
                        has_unmodeled_properties = true;
                    }
                    reader.read_to_end_into(e.name(), &mut Vec::new())?;
                }
                Ok(Event::End(ref e))
                    if matches_word_name(e.name().as_ref(), context, b"tblBorders")
                        || matches_word_name(e.name().as_ref(), context, b"tcBorders") =>
                {
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(OxmlError::MissingElement(
                        "closing table borders".to_string(),
                    ));
                }
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok((borders, has_unmodeled_properties))
    }

    fn parse_edge(
        e: &BytesStart<'_>,
        context: &NamespaceContext,
        borders: &mut Self,
    ) -> Result<bool> {
        let modeled = matches_word_element(e, context, b"top")
            || matches_word_element(e, context, b"bottom")
            || matches_word_element(e, context, b"left")
            || matches_word_element(e, context, b"start")
            || matches_word_element(e, context, b"right")
            || matches_word_element(e, context, b"end")
            || matches_word_element(e, context, b"insideH")
            || matches_word_element(e, context, b"insideV");
        if !modeled {
            return Ok(false);
        }
        if matches_word_element(e, context, b"top") {
            let edge = CT_BorderEdge::from_xml_attrs_with_context(e, context)?;
            borders.top = Some(edge);
        } else if matches_word_element(e, context, b"bottom") {
            let edge = CT_BorderEdge::from_xml_attrs_with_context(e, context)?;
            borders.bottom = Some(edge);
        } else if matches_word_element(e, context, b"left")
            || matches_word_element(e, context, b"start")
        {
            let edge = CT_BorderEdge::from_xml_attrs_with_context(e, context)?;
            borders.left = Some(edge);
        } else if matches_word_element(e, context, b"right")
            || matches_word_element(e, context, b"end")
        {
            let edge = CT_BorderEdge::from_xml_attrs_with_context(e, context)?;
            borders.right = Some(edge);
        } else if matches_word_element(e, context, b"insideH") {
            let edge = CT_BorderEdge::from_xml_attrs_with_context(e, context)?;
            borders.inside_h = Some(edge);
        } else if matches_word_element(e, context, b"insideV") {
            let edge = CT_BorderEdge::from_xml_attrs_with_context(e, context)?;
            borders.inside_v = Some(edge);
        }
        Ok(!has_unmodeled_attributes(
            e,
            context,
            &[b"val", b"sz", b"space", b"color"],
            &[],
        )?)
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>, tag: &str) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new(tag)))?;
        if let Some(ref e) = self.top {
            e.to_xml(writer, "w:top")?;
        }
        if let Some(ref e) = self.left {
            e.to_xml(writer, "w:left")?;
        }
        if let Some(ref e) = self.bottom {
            e.to_xml(writer, "w:bottom")?;
        }
        if let Some(ref e) = self.right {
            e.to_xml(writer, "w:right")?;
        }
        if let Some(ref e) = self.inside_h {
            e.to_xml(writer, "w:insideH")?;
        }
        if let Some(ref e) = self.inside_v {
            e.to_xml(writer, "w:insideV")?;
        }
        writer.write_event(Event::End(BytesEnd::new(tag)))?;
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.top.is_none()
            && self.bottom.is_none()
            && self.left.is_none()
            && self.right.is_none()
            && self.inside_h.is_none()
            && self.inside_v.is_none()
    }
}

/// Table cell margin (a single edge width).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CT_TblCellMar {
    pub top: Option<Twips>,
    pub bottom: Option<Twips>,
    pub left: Option<Twips>,
    pub right: Option<Twips>,
}

impl CT_TblCellMar {
    fn parse_edge(e: &BytesStart, context: &NamespaceContext) -> Result<(Option<Twips>, bool)> {
        let element_context = context.with_element(e);
        let mut width = None;
        let mut width_type_is_modeled = true;
        for attr in e.attributes() {
            let attr = attr?;
            if matches_word_attribute(attr.key.as_ref(), &element_context, b"w") {
                let val: i32 = std::str::from_utf8(&attr.value)?.parse()?;
                width = Some(Twips(val));
            } else if matches_word_attribute(attr.key.as_ref(), &element_context, b"type") {
                width_type_is_modeled = std::str::from_utf8(&attr.value)? == "dxa";
            }
        }
        let has_unmodeled_properties =
            has_unmodeled_attributes(e, context, &[b"w", b"type"], &[])? || !width_type_is_modeled;
        Ok((width, has_unmodeled_properties))
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_context_and_completeness(reader, &NamespaceContext::default())
            .map(|(margins, _)| margins)
    }

    fn from_xml_with_context_and_completeness(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
    ) -> Result<(Self, bool)> {
        let mut mar = CT_TblCellMar::default();
        let mut has_unmodeled_properties = false;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) => {
                    if matches_word_element(e, context, b"top") {
                        let (value, unmodeled) = Self::parse_edge(e, context)?;
                        mar.top = value;
                        has_unmodeled_properties |= unmodeled;
                    } else if matches_word_element(e, context, b"bottom") {
                        let (value, unmodeled) = Self::parse_edge(e, context)?;
                        mar.bottom = value;
                        has_unmodeled_properties |= unmodeled;
                    } else if matches_word_element(e, context, b"left")
                        || matches_word_element(e, context, b"start")
                    {
                        let (value, unmodeled) = Self::parse_edge(e, context)?;
                        mar.left = value;
                        has_unmodeled_properties |= unmodeled;
                    } else if matches_word_element(e, context, b"right")
                        || matches_word_element(e, context, b"end")
                    {
                        let (value, unmodeled) = Self::parse_edge(e, context)?;
                        mar.right = value;
                        has_unmodeled_properties |= unmodeled;
                    } else {
                        has_unmodeled_properties = true;
                    }
                }
                Ok(Event::Start(ref e)) => {
                    if matches_word_element(e, context, b"top")
                        || matches_word_element(e, context, b"bottom")
                        || matches_word_element(e, context, b"left")
                        || matches_word_element(e, context, b"start")
                        || matches_word_element(e, context, b"right")
                        || matches_word_element(e, context, b"end")
                    {
                        let (value, unmodeled) = Self::parse_edge(e, context)?;
                        if matches_word_element(e, context, b"top") {
                            mar.top = value;
                        } else if matches_word_element(e, context, b"bottom") {
                            mar.bottom = value;
                        } else if matches_word_element(e, context, b"left")
                            || matches_word_element(e, context, b"start")
                        {
                            mar.left = value;
                        } else {
                            mar.right = value;
                        }
                        has_unmodeled_properties |= unmodeled;
                    } else {
                        has_unmodeled_properties = true;
                    }
                    reader.read_to_end_into(e.name(), &mut Vec::new())?;
                }
                Ok(Event::End(ref e))
                    if matches_word_name(e.name().as_ref(), context, b"tblCellMar") =>
                {
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(OxmlError::MissingElement(
                        "closing w:tblCellMar".to_string(),
                    ));
                }
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok((mar, has_unmodeled_properties))
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("w:tblCellMar")))?;

        fn write_edge<W: std::io::Write>(
            writer: &mut Writer<W>,
            tag: &str,
            val: Twips,
        ) -> Result<()> {
            let mut buf = itoa::Buffer::new();
            let mut e = BytesStart::new(tag);
            e.push_attribute(("w:w", buf.format(val.0)));
            e.push_attribute(("w:type", "dxa"));
            writer.write_event(Event::Empty(e))?;
            Ok(())
        }

        if let Some(t) = self.top {
            write_edge(writer, "w:top", t)?;
        }
        if let Some(l) = self.left {
            write_edge(writer, "w:left", l)?;
        }
        if let Some(b) = self.bottom {
            write_edge(writer, "w:bottom", b)?;
        }
        if let Some(r) = self.right {
            write_edge(writer, "w:right", r)?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:tblCellMar")))?;
        Ok(())
    }
}

// ---- Table width ----

/// Table width specification.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_TblWidth {
    /// Width value
    pub w: i32,
    /// Width type: "dxa" (twips), "pct" (50ths of a percent), "auto", "nil"
    pub width_type: String,
}

impl CT_TblWidth {
    pub fn dxa(twips: i32) -> Self {
        CT_TblWidth {
            w: twips,
            width_type: "dxa".to_string(),
        }
    }

    pub fn pct(fiftieths: i32) -> Self {
        CT_TblWidth {
            w: fiftieths,
            width_type: "pct".to_string(),
        }
    }

    pub fn auto() -> Self {
        CT_TblWidth {
            w: 0,
            width_type: "auto".to_string(),
        }
    }

    pub fn from_xml_attrs(e: &BytesStart) -> Result<Self> {
        Self::from_xml_attrs_with_context(e, &NamespaceContext::default())
    }

    fn from_xml_attrs_with_context(
        e: &BytesStart,
        parent_context: &NamespaceContext,
    ) -> Result<Self> {
        let context = parent_context.with_element(e);
        let mut w = 0;
        let mut width_type = "dxa".to_string();

        for attr in e.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            let val = std::str::from_utf8(&attr.value)?;
            if matches_word_attribute(key, &context, b"w") {
                w = val.parse().unwrap_or(0);
            } else if matches_word_attribute(key, &context, b"type") {
                width_type = val.to_string();
            }
        }

        Ok(CT_TblWidth { w, width_type })
    }

    pub fn write_xml<W: std::io::Write>(&self, writer: &mut Writer<W>, tag: &str) -> Result<()> {
        let mut buf = itoa::Buffer::new();
        let mut e = BytesStart::new(tag);
        e.push_attribute(("w:w", buf.format(self.w)));
        e.push_attribute(("w:type", self.width_type.as_str()));
        writer.write_event(Event::Empty(e))?;
        Ok(())
    }
}

// ---- Table grid column ----

/// `CT_TblGridCol` — A column definition in the table grid.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_TblGridCol {
    /// Column width in twips
    pub width: Twips,
}

// ---- Table properties ----

/// `CT_TblPr` — Table properties.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CT_TblPr {
    /// Whether this group contained a Word property the reader does not model.
    pub has_unmodeled_properties: bool,
    /// Table style ID
    pub style_id: Option<String>,
    /// Table width
    pub width: Option<CT_TblWidth>,
    /// Table alignment
    pub jc: Option<ST_Jc>,
    /// Table borders
    pub borders: Option<CT_TblBorders>,
    /// Default cell margins
    pub cell_margin: Option<CT_TblCellMar>,
    /// Table layout: "fixed" or "autofit"
    pub layout: Option<String>,
    /// Table indent from left margin
    pub indent: Option<CT_TblWidth>,
    /// Table shading/background
    pub shading: Option<CT_Shd>,
    /// Which parts of the table style's conditional formatting apply.
    pub look: Option<CT_TblLook>,
}

/// `w:tblLook` — which parts of a table style's conditional formatting apply.
///
/// The style reference in `w:tblStyle` says *which* style to use. This says
/// which of its conditional parts to turn on: header row emphasis, banding,
/// first-column formatting. Dropping it leaves the style name intact and the
/// table rendered with base formatting only, which reads as the style having
/// been lost.
///
/// `w:val` is a legacy bitmask carrying the same information. Both are kept,
/// because writers disagree about which one to emit and readers disagree about
/// which one to trust.
#[derive(Debug, Clone, PartialEq, Default)]
#[allow(non_snake_case)]
pub struct CT_TblLook {
    /// Legacy bitmask form, e.g. "04A0".
    pub val: Option<String>,
    pub first_row: Option<bool>,
    pub last_row: Option<bool>,
    pub first_column: Option<bool>,
    pub last_column: Option<bool>,
    pub no_h_band: Option<bool>,
    pub no_v_band: Option<bool>,
}

/// Read an OOXML boolean attribute, which may be written as 1/0 or true/false.
fn parse_ooxml_bool(value: &str) -> Option<bool> {
    match value {
        "1" | "true" | "on" => Some(true),
        "0" | "false" | "off" => Some(false),
        _ => None,
    }
}

fn ooxml_bool_str(value: bool) -> &'static str {
    if value { "1" } else { "0" }
}

fn parse_toggle_element(
    element: &BytesStart<'_>,
    context: &NamespaceContext,
    name: &str,
) -> Result<bool> {
    let Some(value) = get_val_attr_with_context(element, context)? else {
        return Ok(true);
    };
    parse_ooxml_bool(&value)
        .ok_or_else(|| OxmlError::InvalidValue(format!("invalid w:{name} toggle value: {value}")))
}

#[allow(non_snake_case)]
impl CT_TblLook {
    pub fn from_xml_attrs(e: &BytesStart) -> Result<Self> {
        Self::from_xml_attrs_with_context(e, &NamespaceContext::default())
    }

    fn from_xml_attrs_with_context(
        e: &BytesStart,
        parent_context: &NamespaceContext,
    ) -> Result<Self> {
        let context = parent_context.with_element(e);
        let mut look = CT_TblLook::default();
        for attr in e.attributes().flatten() {
            let value = std::str::from_utf8(&attr.value)?;
            let key = attr.key.as_ref();
            if matches_word_attribute(key, &context, b"val") {
                look.val = Some(value.to_string());
            } else if matches_word_attribute(key, &context, b"firstRow") {
                look.first_row = parse_ooxml_bool(value);
            } else if matches_word_attribute(key, &context, b"lastRow") {
                look.last_row = parse_ooxml_bool(value);
            } else if matches_word_attribute(key, &context, b"firstColumn") {
                look.first_column = parse_ooxml_bool(value);
            } else if matches_word_attribute(key, &context, b"lastColumn") {
                look.last_column = parse_ooxml_bool(value);
            } else if matches_word_attribute(key, &context, b"noHBand") {
                look.no_h_band = parse_ooxml_bool(value);
            } else if matches_word_attribute(key, &context, b"noVBand") {
                look.no_v_band = parse_ooxml_bool(value);
            }
        }
        Ok(look)
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut e = BytesStart::new("w:tblLook");
        if let Some(ref val) = self.val {
            e.push_attribute(("w:val", val.as_str()));
        }
        for (name, value) in [
            ("w:firstRow", self.first_row),
            ("w:lastRow", self.last_row),
            ("w:firstColumn", self.first_column),
            ("w:lastColumn", self.last_column),
            ("w:noHBand", self.no_h_band),
            ("w:noVBand", self.no_v_band),
        ] {
            if let Some(value) = value {
                e.push_attribute((name, ooxml_bool_str(value)));
            }
        }
        writer.write_event(Event::Empty(e))?;
        Ok(())
    }
}

#[allow(non_snake_case)]
impl CT_TblPr {
    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_context(reader, &NamespaceContext::default())
    }

    pub fn from_xml_with_context(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
    ) -> Result<Self> {
        let mut pr = CT_TblPr::default();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) => {
                    if matches_word_element(e, context, b"tblBorders") {
                        pr.has_unmodeled_properties |=
                            has_unmodeled_attributes(e, context, &[], &[])?;
                        pr.borders = Some(CT_TblBorders::default());
                    } else if matches_word_element(e, context, b"tblCellMar") {
                        pr.has_unmodeled_properties |=
                            has_unmodeled_attributes(e, context, &[], &[])?;
                        pr.cell_margin = Some(CT_TblCellMar::default());
                    } else if !Self::parse_property_element(e, context, &mut pr)? {
                        pr.has_unmodeled_properties = true;
                    }
                }
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    if matches_word_element(e, context, b"tblBorders") {
                        pr.has_unmodeled_properties |=
                            has_unmodeled_attributes(e, context, &[], &[])?;
                        let child_context = context.with_element(e);
                        let (borders, has_unmodeled_properties) =
                            CT_TblBorders::from_xml_with_context_and_completeness(
                                reader,
                                &child_context,
                            )?;
                        pr.has_unmodeled_properties |= has_unmodeled_properties;
                        pr.borders = Some(borders);
                    } else if matches_word_element(e, context, b"tblCellMar") {
                        pr.has_unmodeled_properties |=
                            has_unmodeled_attributes(e, context, &[], &[])?;
                        let child_context = context.with_element(e);
                        let (cell_margin, has_unmodeled_properties) =
                            CT_TblCellMar::from_xml_with_context_and_completeness(
                                reader,
                                &child_context,
                            )?;
                        pr.has_unmodeled_properties |= has_unmodeled_properties;
                        pr.cell_margin = Some(cell_margin);
                    } else {
                        if !Self::parse_property_element(e, context, &mut pr)? {
                            pr.has_unmodeled_properties = true;
                        }
                        reader.read_to_end_into(name, &mut Vec::new())?;
                    }
                }
                Ok(Event::End(ref e))
                    if matches_word_name(e.name().as_ref(), context, b"tblPr") =>
                {
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(OxmlError::MissingElement("closing w:tblPr".to_string()));
                }
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(pr)
    }

    fn parse_property_element(
        e: &BytesStart<'_>,
        context: &NamespaceContext,
        pr: &mut Self,
    ) -> Result<bool> {
        let element_context = context.with_element(e);
        let allowed_word_attributes: &[&[u8]];
        if matches_word_element(e, context, b"tblStyle") {
            pr.style_id = get_val_attr_with_context(e, &element_context)?;
            allowed_word_attributes = &[b"val"];
        } else if matches_word_element(e, context, b"tblW") {
            pr.width = Some(CT_TblWidth::from_xml_attrs_with_context(e, context)?);
            allowed_word_attributes = &[b"w", b"type"];
        } else if matches_word_element(e, context, b"jc") {
            if let Some(val) = get_val_attr_with_context(e, &element_context)? {
                pr.jc = Some(ST_Jc::from_str(&val)?);
            }
            allowed_word_attributes = &[b"val"];
        } else if matches_word_element(e, context, b"tblLayout") {
            for attribute in e.attributes() {
                let attribute = attribute?;
                if matches_word_attribute(attribute.key.as_ref(), &element_context, b"type") {
                    pr.layout = Some(std::str::from_utf8(&attribute.value)?.to_string());
                    break;
                }
            }
            allowed_word_attributes = &[b"type"];
        } else if matches_word_element(e, context, b"tblInd") {
            pr.indent = Some(CT_TblWidth::from_xml_attrs_with_context(e, context)?);
            allowed_word_attributes = &[b"w", b"type"];
        } else if matches_word_element(e, context, b"shd") {
            pr.shading = Some(CT_Shd::from_xml_attrs_with_context(e, context)?);
            allowed_word_attributes = &[b"val", b"color", b"fill"];
        } else if matches_word_element(e, context, b"tblLook") {
            pr.look = Some(CT_TblLook::from_xml_attrs_with_context(e, context)?);
            allowed_word_attributes = &[
                b"val",
                b"firstRow",
                b"lastRow",
                b"firstColumn",
                b"lastColumn",
                b"noHBand",
                b"noVBand",
            ];
        } else {
            return Ok(false);
        }
        pr.has_unmodeled_properties |=
            has_unmodeled_attributes(e, context, allowed_word_attributes, &[])?;
        Ok(true)
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("w:tblPr")))?;

        if let Some(ref style_id) = self.style_id {
            let mut e = BytesStart::new("w:tblStyle");
            e.push_attribute(("w:val", style_id.as_str()));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref width) = self.width {
            width.write_xml(writer, "w:tblW")?;
        }

        if let Some(jc) = self.jc {
            let mut e = BytesStart::new("w:jc");
            e.push_attribute(("w:val", jc.to_str()));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref indent) = self.indent {
            indent.write_xml(writer, "w:tblInd")?;
        }

        if let Some(ref borders) = self.borders
            && !borders.is_empty()
        {
            borders.to_xml(writer, "w:tblBorders")?;
        }

        if let Some(ref shd) = self.shading {
            shd.write_xml(writer, "w:shd")?;
        }

        if let Some(ref layout) = self.layout {
            let mut e = BytesStart::new("w:tblLayout");
            e.push_attribute(("w:type", layout.as_str()));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref cell_margin) = self.cell_margin {
            cell_margin.to_xml(writer)?;
        }

        if let Some(ref look) = self.look {
            look.to_xml(writer)?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:tblPr")))?;
        Ok(())
    }
}

// ---- Table grid ----

/// `CT_TblGrid` — Defines the column structure of a table.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CT_TblGrid {
    pub columns: Vec<CT_TblGridCol>,
    /// Whether the grid contains children or attributes the semantic model does not expose.
    pub has_unmodeled_properties: bool,
}

#[allow(non_snake_case)]
impl CT_TblGrid {
    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_context(reader, &NamespaceContext::default())
    }

    pub fn from_xml_with_context(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
    ) -> Result<Self> {
        let mut grid = CT_TblGrid::default();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) => {
                    if !Self::parse_column(e, context, &mut grid)? {
                        grid.has_unmodeled_properties = true;
                    }
                }
                Ok(Event::Start(ref e)) => {
                    if !Self::parse_column(e, context, &mut grid)? {
                        grid.has_unmodeled_properties = true;
                    }
                    reader.read_to_end_into(e.name(), &mut Vec::new())?;
                }
                Ok(Event::End(ref e))
                    if matches_word_name(e.name().as_ref(), context, b"tblGrid") =>
                {
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(OxmlError::MissingElement("closing w:tblGrid".to_string()));
                }
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(grid)
    }

    fn parse_column(
        e: &BytesStart<'_>,
        context: &NamespaceContext,
        grid: &mut CT_TblGrid,
    ) -> Result<bool> {
        if !matches_word_element(e, context, b"gridCol") {
            return Ok(false);
        }
        let element_context = context.with_element(e);
        let mut width = Twips(0);
        for attr in e.attributes() {
            let attr = attr?;
            if matches_word_attribute(attr.key.as_ref(), &element_context, b"w") {
                width = Twips(std::str::from_utf8(&attr.value)?.parse()?);
            } else if matches_word_attribute(attr.key.as_ref(), &element_context, b"type")
                && attr.value.as_ref() != b"dxa"
            {
                grid.has_unmodeled_properties = true;
            }
        }
        grid.has_unmodeled_properties |=
            has_unmodeled_attributes(e, context, &[b"w", b"type"], &[])?;
        grid.columns.push(CT_TblGridCol { width });
        Ok(true)
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut buf = itoa::Buffer::new();
        writer.write_event(Event::Start(BytesStart::new("w:tblGrid")))?;

        for col in &self.columns {
            let mut e = BytesStart::new("w:gridCol");
            e.push_attribute(("w:w", buf.format(col.width.0)));
            writer.write_event(Event::Empty(e))?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:tblGrid")))?;
        Ok(())
    }
}

// ---- Row properties ----

/// Vertical merge state for a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VMerge {
    /// Start of a vertical merge group
    Restart,
    /// Continuation of the merge group above
    Continue,
}

fn parse_merge_element(
    element: &BytesStart<'_>,
    context: &NamespaceContext,
    name: &str,
) -> Result<VMerge> {
    match get_val_attr_with_context(element, context)?.as_deref() {
        Some("restart") => Ok(VMerge::Restart),
        None | Some("continue") => Ok(VMerge::Continue),
        Some(value) => Err(OxmlError::InvalidValue(format!(
            "invalid w:{name} value: {value}"
        ))),
    }
}

fn write_merge_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    name: &str,
    merge: &VMerge,
) -> Result<()> {
    let mut element = BytesStart::new(name);
    if matches!(merge, VMerge::Restart) {
        element.push_attribute(("w:val", "restart"));
    }
    writer.write_event(Event::Empty(element))?;
    Ok(())
}

/// `CT_TrPr` — Table row properties.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CT_TrPr {
    /// Whether this group contained a Word property the reader does not model.
    pub has_unmodeled_properties: bool,
    /// Row height in twips
    pub height: Option<Twips>,
    /// Row height rule: "exact" or "atLeast"
    pub height_rule: Option<String>,
    /// Repeat as header row on each page
    pub header: Option<bool>,
    /// Row alignment
    pub jc: Option<ST_Jc>,
    /// Allow row to break across pages
    pub cant_split: Option<bool>,
    /// Number of table grid columns omitted before the first cell.
    pub grid_before: Option<u32>,
    /// Number of table grid columns omitted after the last cell.
    pub grid_after: Option<u32>,
    /// `w:cnfStyle` — which conditional parts of the table style this row is.
    ///
    /// Word writes this alongside `w:tblLook` and needs both to reproduce a
    /// styled table. Dropping it loses the header-row and banding emphasis.
    pub cnf_style: Option<String>,
}

#[allow(non_snake_case)]
impl CT_TrPr {
    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_context(reader, &NamespaceContext::default())
    }

    pub fn from_xml_with_context(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
    ) -> Result<Self> {
        let mut pr = CT_TrPr::default();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) => {
                    if !Self::parse_property_element(e, context, &mut pr)? {
                        pr.has_unmodeled_properties = true;
                    }
                }
                Ok(Event::Start(ref e)) => {
                    if !Self::parse_property_element(e, context, &mut pr)? {
                        pr.has_unmodeled_properties = true;
                    }
                    reader.read_to_end_into(e.name(), &mut Vec::new())?;
                }
                Ok(Event::End(ref e)) if matches_word_name(e.name().as_ref(), context, b"trPr") => {
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(OxmlError::MissingElement("closing w:trPr".to_string()));
                }
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(pr)
    }

    fn parse_property_element(
        e: &BytesStart<'_>,
        context: &NamespaceContext,
        pr: &mut CT_TrPr,
    ) -> Result<bool> {
        let element_context = context.with_element(e);
        let allowed_word_attributes: &[&[u8]];
        if matches_word_element(e, context, b"trHeight") {
            for attr in e.attributes() {
                let attr = attr?;
                let key = attr.key.as_ref();
                let val = std::str::from_utf8(&attr.value)?;
                if matches_word_attribute(key, &element_context, b"val") {
                    pr.height = Some(Twips(val.parse()?));
                } else if matches_word_attribute(key, &element_context, b"hRule") {
                    pr.height_rule = Some(val.to_string());
                }
            }
            allowed_word_attributes = &[b"val", b"hRule"];
        } else if matches_word_element(e, context, b"tblHeader") {
            pr.header = Some(parse_toggle_element(e, &element_context, "tblHeader")?);
            allowed_word_attributes = &[b"val"];
        } else if matches_word_element(e, context, b"jc") {
            if let Some(val) = get_val_attr_with_context(e, &element_context)? {
                pr.jc = Some(ST_Jc::from_str(&val)?);
            }
            allowed_word_attributes = &[b"val"];
        } else if matches_word_element(e, context, b"cnfStyle") {
            pr.cnf_style = get_val_attr_with_context(e, &element_context)?;
            allowed_word_attributes = &[b"val"];
        } else if matches_word_element(e, context, b"cantSplit") {
            pr.cant_split = Some(parse_toggle_element(e, &element_context, "cantSplit")?);
            allowed_word_attributes = &[b"val"];
        } else if matches_word_element(e, context, b"gridBefore")
            && let Some(val) = get_val_attr_with_context(e, &element_context)?
        {
            pr.grid_before = Some(val.parse()?);
            allowed_word_attributes = &[b"val"];
        } else if matches_word_element(e, context, b"gridAfter")
            && let Some(val) = get_val_attr_with_context(e, &element_context)?
        {
            pr.grid_after = Some(val.parse()?);
            allowed_word_attributes = &[b"val"];
        } else if matches_word_element(e, context, b"gridBefore")
            || matches_word_element(e, context, b"gridAfter")
        {
            allowed_word_attributes = &[b"val"];
        } else if !matches_word_element(e, context, b"gridAfter") {
            return Ok(false);
        } else {
            allowed_word_attributes = &[b"val"];
        }
        pr.has_unmodeled_properties |=
            has_unmodeled_attributes(e, context, allowed_word_attributes, &[])?;
        Ok(true)
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        if self.is_empty() {
            return Ok(());
        }

        writer.write_event(Event::Start(BytesStart::new("w:trPr")))?;

        // cnfStyle comes first in the schema sequence for both trPr and tcPr.
        if let Some(ref cnf) = self.cnf_style {
            let mut e = BytesStart::new("w:cnfStyle");
            e.push_attribute(("w:val", cnf.as_str()));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(grid_before) = self.grid_before {
            let mut buf = itoa::Buffer::new();
            let mut e = BytesStart::new("w:gridBefore");
            e.push_attribute(("w:val", buf.format(grid_before)));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(grid_after) = self.grid_after {
            let mut buf = itoa::Buffer::new();
            let mut e = BytesStart::new("w:gridAfter");
            e.push_attribute(("w:val", buf.format(grid_after)));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(cant_split) = self.cant_split {
            let mut element = BytesStart::new("w:cantSplit");
            if !cant_split {
                element.push_attribute(("w:val", "0"));
            }
            writer.write_event(Event::Empty(element))?;
        }

        if let Some(height) = self.height {
            let mut buf = itoa::Buffer::new();
            let mut e = BytesStart::new("w:trHeight");
            e.push_attribute(("w:val", buf.format(height.0)));
            if let Some(ref rule) = self.height_rule {
                e.push_attribute(("w:hRule", rule.as_str()));
            }
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(header) = self.header {
            let mut element = BytesStart::new("w:tblHeader");
            if !header {
                element.push_attribute(("w:val", "0"));
            }
            writer.write_event(Event::Empty(element))?;
        }

        if let Some(jc) = self.jc {
            let mut e = BytesStart::new("w:jc");
            e.push_attribute(("w:val", jc.to_str()));
            writer.write_event(Event::Empty(e))?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:trPr")))?;
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.height.is_none()
            && self.header.is_none()
            && self.jc.is_none()
            && self.cant_split.is_none()
            && self.grid_before.is_none()
            && self.grid_after.is_none()
            && self.cnf_style.is_none()
    }
}

// ---- Cell properties ----

/// Vertical alignment within a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ST_VerticalJc {
    Top,
    Center,
    Bottom,
}

impl ST_VerticalJc {
    pub fn from_str(s: &str) -> Self {
        match s {
            "center" => Self::Center,
            "bottom" => Self::Bottom,
            _ => Self::Top,
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Center => "center",
            Self::Bottom => "bottom",
        }
    }
}

/// `CT_TcPr` — Table cell properties.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CT_TcPr {
    /// Whether this group contained a Word property the reader does not model.
    pub has_unmodeled_properties: bool,
    /// Cell width
    pub width: Option<CT_TblWidth>,
    /// Horizontal merge (number of grid columns spanned)
    pub grid_span: Option<u32>,
    /// Legacy horizontal merge state.
    pub h_merge: Option<VMerge>,
    /// Vertical merge
    pub v_merge: Option<VMerge>,
    /// Cell borders
    pub borders: Option<CT_TblBorders>,
    /// Cell shading
    pub shading: Option<CT_Shd>,
    /// Vertical alignment
    pub v_align: Option<ST_VerticalJc>,
    /// No-wrap text
    pub no_wrap: Option<bool>,
    /// Text direction
    pub text_direction: Option<String>,
    /// `w:cnfStyle` — which conditional parts of the table style this cell is.
    pub cnf_style: Option<String>,
}

#[allow(non_snake_case)]
impl CT_TcPr {
    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_context(reader, &NamespaceContext::default())
    }

    pub fn from_xml_with_context(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
    ) -> Result<Self> {
        let mut pr = CT_TcPr::default();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) => {
                    if matches_word_element(e, context, b"tcBorders") {
                        pr.has_unmodeled_properties |=
                            has_unmodeled_attributes(e, context, &[], &[])?;
                        pr.borders = Some(CT_TblBorders::default());
                    } else if !Self::parse_property_element(e, context, &mut pr)? {
                        pr.has_unmodeled_properties = true;
                    }
                }
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    if matches_word_element(e, context, b"tcBorders") {
                        pr.has_unmodeled_properties |=
                            has_unmodeled_attributes(e, context, &[], &[])?;
                        let child_context = context.with_element(e);
                        let (borders, has_unmodeled_properties) =
                            CT_TblBorders::from_xml_with_context_and_completeness(
                                reader,
                                &child_context,
                            )?;
                        pr.has_unmodeled_properties |= has_unmodeled_properties;
                        pr.borders = Some(borders);
                    } else {
                        if !Self::parse_property_element(e, context, &mut pr)? {
                            pr.has_unmodeled_properties = true;
                        }
                        reader.read_to_end_into(name, &mut Vec::new())?;
                    }
                }
                Ok(Event::End(ref e)) if matches_word_name(e.name().as_ref(), context, b"tcPr") => {
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(OxmlError::MissingElement("closing w:tcPr".to_string()));
                }
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(pr)
    }

    fn parse_property_element(
        e: &BytesStart<'_>,
        context: &NamespaceContext,
        pr: &mut CT_TcPr,
    ) -> Result<bool> {
        let element_context = context.with_element(e);
        let allowed_word_attributes: &[&[u8]];
        if matches_word_element(e, context, b"tcW") {
            pr.width = Some(CT_TblWidth::from_xml_attrs_with_context(e, context)?);
            allowed_word_attributes = &[b"w", b"type"];
        } else if matches_word_element(e, context, b"gridSpan") {
            if let Some(val) = get_val_attr_with_context(e, &element_context)? {
                let span = val.parse()?;
                if span == 0 {
                    return Err(OxmlError::InvalidValue(
                        "w:gridSpan must be greater than zero".to_string(),
                    ));
                }
                pr.grid_span = Some(span);
            }
            allowed_word_attributes = &[b"val"];
        } else if matches_word_element(e, context, b"hMerge") {
            pr.h_merge = Some(parse_merge_element(e, &element_context, "hMerge")?);
            allowed_word_attributes = &[b"val"];
        } else if matches_word_element(e, context, b"vMerge") {
            pr.v_merge = Some(parse_merge_element(e, &element_context, "vMerge")?);
            allowed_word_attributes = &[b"val"];
        } else if matches_word_element(e, context, b"vAlign") {
            if let Some(val) = get_val_attr_with_context(e, &element_context)? {
                pr.v_align = Some(ST_VerticalJc::from_str(&val));
            }
            allowed_word_attributes = &[b"val"];
        } else if matches_word_element(e, context, b"shd") {
            pr.shading = Some(CT_Shd::from_xml_attrs_with_context(e, context)?);
            allowed_word_attributes = &[b"val", b"color", b"fill"];
        } else if matches_word_element(e, context, b"cnfStyle") {
            pr.cnf_style = get_val_attr_with_context(e, &element_context)?;
            allowed_word_attributes = &[b"val"];
        } else if matches_word_element(e, context, b"noWrap") {
            pr.no_wrap = Some(parse_toggle_element(e, &element_context, "noWrap")?);
            allowed_word_attributes = &[b"val"];
        } else if matches_word_element(e, context, b"textDirection")
            && let Some(val) = get_val_attr_with_context(e, &element_context)?
        {
            pr.text_direction = Some(val);
            allowed_word_attributes = &[b"val"];
        } else if matches_word_element(e, context, b"textDirection") {
            allowed_word_attributes = &[b"val"];
        } else if !matches_word_element(e, context, b"textDirection") {
            return Ok(false);
        } else {
            allowed_word_attributes = &[b"val"];
        }
        pr.has_unmodeled_properties |=
            has_unmodeled_attributes(e, context, allowed_word_attributes, &[])?;
        Ok(true)
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        if self.is_empty() {
            return Ok(());
        }

        writer.write_event(Event::Start(BytesStart::new("w:tcPr")))?;

        if let Some(ref cnf) = self.cnf_style {
            let mut e = BytesStart::new("w:cnfStyle");
            e.push_attribute(("w:val", cnf.as_str()));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref width) = self.width {
            width.write_xml(writer, "w:tcW")?;
        }

        if let Some(grid_span) = self.grid_span
            && grid_span > 1
        {
            let mut buf = itoa::Buffer::new();
            let mut e = BytesStart::new("w:gridSpan");
            e.push_attribute(("w:val", buf.format(grid_span)));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref merge) = self.h_merge {
            write_merge_element(writer, "w:hMerge", merge)?;
        }

        if let Some(ref vm) = self.v_merge {
            write_merge_element(writer, "w:vMerge", vm)?;
        }

        if let Some(ref borders) = self.borders
            && !borders.is_empty()
        {
            borders.to_xml(writer, "w:tcBorders")?;
        }

        if let Some(ref shd) = self.shading {
            shd.write_xml(writer, "w:shd")?;
        }

        if let Some(no_wrap) = self.no_wrap {
            let mut element = BytesStart::new("w:noWrap");
            if !no_wrap {
                element.push_attribute(("w:val", "0"));
            }
            writer.write_event(Event::Empty(element))?;
        }

        if let Some(ref va) = self.v_align {
            let mut e = BytesStart::new("w:vAlign");
            e.push_attribute(("w:val", va.to_str()));
            writer.write_event(Event::Empty(e))?;
        }

        if let Some(ref td) = self.text_direction {
            let mut e = BytesStart::new("w:textDirection");
            e.push_attribute(("w:val", td.as_str()));
            writer.write_event(Event::Empty(e))?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:tcPr")))?;
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.width.is_none()
            && self.grid_span.is_none()
            && self.h_merge.is_none()
            && self.v_merge.is_none()
            && self.borders.is_none()
            && self.shading.is_none()
            && self.v_align.is_none()
            && self.no_wrap.is_none()
            && self.text_direction.is_none()
            && self.cnf_style.is_none()
    }
}

// ---- Table cell ----

/// Content that can appear inside a table cell.
#[derive(Debug, Clone, PartialEq)]
pub enum CellContent {
    /// A paragraph.
    Paragraph(CT_P),
    /// A nested table.
    Table(CT_Tbl),
    /// A child the reader does not model, retained in its exact source order.
    Unsupported(RawXml),
}

/// `CT_Tc` — A table cell containing paragraphs and possibly nested tables.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Tc {
    pub properties: Option<CT_TcPr>,
    /// Cell content (paragraphs and nested tables).
    pub content: Vec<CellContent>,
}

#[allow(non_snake_case)]
impl CT_Tc {
    pub fn new() -> Self {
        CT_Tc {
            properties: None,
            // OOXML requires at least one paragraph per cell
            content: vec![CellContent::Paragraph(CT_P::new())],
        }
    }

    /// Get all paragraphs in this cell (excludes nested tables).
    pub fn paragraphs(&self) -> Vec<&CT_P> {
        self.content
            .iter()
            .filter_map(|c| match c {
                CellContent::Paragraph(p) => Some(p),
                CellContent::Table(_) | CellContent::Unsupported(_) => None,
            })
            .collect()
    }

    /// Get mutable reference to paragraphs (backward compatibility).
    pub fn paragraphs_mut(&mut self) -> Vec<&mut CT_P> {
        self.content
            .iter_mut()
            .filter_map(|c| match c {
                CellContent::Paragraph(p) => Some(p),
                CellContent::Table(_) | CellContent::Unsupported(_) => None,
            })
            .collect()
    }

    pub fn text(&self) -> String {
        self.paragraphs()
            .iter()
            .map(|p| p.text())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_context(reader, &NamespaceContext::default())
    }

    pub fn from_xml_with_context(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
    ) -> Result<Self> {
        Self::from_xml_with_context_at_depth(reader, context, 1)
    }

    fn from_xml_with_context_at_depth(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
        model_depth: usize,
    ) -> Result<Self> {
        if model_depth > MAX_MODEL_DEPTH {
            return Err(model_depth_error());
        }
        let mut properties = None;
        let mut content = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    if matches_word_element(e, context, b"tcPr") {
                        let child_context = context.with_element(e);
                        properties = Some(CT_TcPr::from_xml_with_context(reader, &child_context)?);
                    } else if matches_word_element(e, context, b"p") {
                        let child_context = context.with_element(e);
                        content.push(CellContent::Paragraph(CT_P::from_xml_with_context(
                            reader,
                            &child_context,
                        )?));
                    } else if matches_word_element(e, context, b"tbl") {
                        let child_context = context.with_element(e);
                        content.push(CellContent::Table(CT_Tbl::from_xml_with_context_at_depth(
                            reader,
                            &child_context,
                            model_depth + 1,
                        )?));
                    } else {
                        // Content controls (w:sdt), bookmarks and revision
                        // marks live here. Keep them verbatim rather than
                        // dropping the subtree, which used to delete every
                        // paragraph wrapped in a content control.
                        content.push(CellContent::Unsupported(capture_raw_element(
                            reader, e, context,
                        )?));
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    if matches_word_element(e, context, b"tcPr") {
                        properties = Some(CT_TcPr::default());
                    } else if matches_word_element(e, context, b"p") {
                        content.push(CellContent::Paragraph(CT_P::new()));
                    } else if matches_word_element(e, context, b"tbl") {
                        content.push(CellContent::Table(CT_Tbl::new()));
                    } else {
                        content.push(CellContent::Unsupported(capture_raw_empty_element(
                            e, context,
                        )?));
                    }
                }
                Ok(Event::End(ref e)) if matches_word_name(e.name().as_ref(), context, b"tc") => {
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(OxmlError::MissingElement("closing w:tc".to_string()));
                }
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(CT_Tc {
            properties,
            content,
        })
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        self.to_xml_with_context(writer, &NamespaceContext::default())
    }

    pub(crate) fn to_xml_with_context<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        context: &NamespaceContext,
    ) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("w:tc")))?;

        if let Some(ref props) = self.properties {
            props.to_xml(writer)?;
        }

        for item in &self.content {
            match item {
                CellContent::Paragraph(p) => p.to_xml_with_context(writer, context)?,
                CellContent::Table(tbl) => tbl.to_xml_with_context(writer, context)?,
                CellContent::Unsupported(raw) => {
                    raw.write_to_with_context(writer.get_mut(), context)?
                }
            }
        }

        writer.write_event(Event::End(BytesEnd::new("w:tc")))?;
        Ok(())
    }
}

impl Default for CT_Tc {
    fn default() -> Self {
        Self::new()
    }
}

// ---- Table row ----

/// `CT_Row` — A table row containing cells.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Row {
    pub properties: Option<CT_TrPr>,
    pub cells: Vec<CT_Tc>,
    /// Raw XML for children we do not model, tagged with the cell index they
    /// appeared before so they can be written back in place.
    pub extra_xml: Vec<(usize, Vec<u8>)>,
}

#[allow(non_snake_case)]
impl CT_Row {
    pub fn new() -> Self {
        CT_Row {
            properties: None,
            cells: Vec::new(),
            extra_xml: Vec::new(),
        }
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_context(reader, &NamespaceContext::default())
    }

    pub fn from_xml_with_context(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
    ) -> Result<Self> {
        Self::from_xml_with_context_at_depth(reader, context, 1)
    }

    fn from_xml_with_context_at_depth(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
        model_depth: usize,
    ) -> Result<Self> {
        let mut properties = None;
        let mut cells = Vec::new();
        let mut extra_xml = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    if matches_word_element(e, context, b"trPr") {
                        let child_context = context.with_element(e);
                        properties = Some(CT_TrPr::from_xml_with_context(reader, &child_context)?);
                    } else if matches_word_element(e, context, b"tc") {
                        let child_context = context.with_element(e);
                        cells.push(CT_Tc::from_xml_with_context_at_depth(
                            reader,
                            &child_context,
                            model_depth,
                        )?);
                    } else {
                        // A cell wrapped in a content control used to be
                        // dropped here, leaving a row with no cells at all.
                        extra_xml.push((cells.len(), capture_element(reader, e)?));
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    if matches_word_element(e, context, b"trPr") {
                        properties = Some(CT_TrPr::default());
                    } else if matches_word_element(e, context, b"tc") {
                        cells.push(CT_Tc {
                            properties: None,
                            content: Vec::new(),
                        });
                    } else {
                        extra_xml.push((cells.len(), capture_empty_element(e)?));
                    }
                }
                Ok(Event::End(ref e)) if matches_word_name(e.name().as_ref(), context, b"tr") => {
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(OxmlError::MissingElement("closing w:tr".to_string()));
                }
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(CT_Row {
            properties,
            cells,
            extra_xml,
        })
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        self.to_xml_with_context(writer, &NamespaceContext::default())
    }

    pub(crate) fn to_xml_with_context<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        context: &NamespaceContext,
    ) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("w:tr")))?;

        if let Some(ref props) = self.properties {
            props.to_xml(writer)?;
        }

        for (idx, cell) in self.cells.iter().enumerate() {
            write_extras_at(writer, &self.extra_xml, idx)?;
            cell.to_xml_with_context(writer, context)?;
        }
        write_extras_at(writer, &self.extra_xml, self.cells.len())?;

        writer.write_event(Event::End(BytesEnd::new("w:tr")))?;
        Ok(())
    }
}

impl Default for CT_Row {
    fn default() -> Self {
        Self::new()
    }
}

// ---- Table ----

/// `CT_Tbl` — A table element containing rows.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Tbl {
    pub properties: Option<CT_TblPr>,
    pub grid: Option<CT_TblGrid>,
    pub rows: Vec<CT_Row>,
    /// Raw XML for children we do not model, tagged with the row index they
    /// appeared before so they can be written back in place.
    pub extra_xml: Vec<(usize, Vec<u8>)>,
}

#[allow(non_snake_case)]
impl CT_Tbl {
    pub fn new() -> Self {
        CT_Tbl {
            properties: None,
            grid: None,
            rows: Vec::new(),
            extra_xml: Vec::new(),
        }
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_context(reader, &NamespaceContext::default())
    }

    pub fn from_xml_with_context(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
    ) -> Result<Self> {
        Self::from_xml_with_context_at_depth(reader, context, 1)
    }

    fn from_xml_with_context_at_depth(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
        model_depth: usize,
    ) -> Result<Self> {
        if model_depth > MAX_MODEL_DEPTH {
            return Err(model_depth_error());
        }
        let mut properties = None;
        let mut grid = None;
        let mut rows = Vec::new();
        let mut extra_xml = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    if matches_word_element(e, context, b"tblPr") {
                        let child_context = context.with_element(e);
                        properties = Some(CT_TblPr::from_xml_with_context(reader, &child_context)?);
                    } else if matches_word_element(e, context, b"tblGrid") {
                        let child_context = context.with_element(e);
                        grid = Some(CT_TblGrid::from_xml_with_context(reader, &child_context)?);
                    } else if matches_word_element(e, context, b"tr") {
                        let child_context = context.with_element(e);
                        rows.push(CT_Row::from_xml_with_context_at_depth(
                            reader,
                            &child_context,
                            model_depth,
                        )?);
                    } else {
                        // Rows wrapped in a content control used to be dropped
                        // here, which silently deleted whole tables.
                        extra_xml.push((rows.len(), capture_element(reader, e)?));
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    if matches_word_element(e, context, b"tblPr") {
                        properties = Some(CT_TblPr::default());
                    } else if matches_word_element(e, context, b"tblGrid") {
                        grid = Some(CT_TblGrid::default());
                    } else if matches_word_element(e, context, b"tr") {
                        rows.push(CT_Row::new());
                    } else {
                        extra_xml.push((rows.len(), capture_empty_element(e)?));
                    }
                }
                Ok(Event::End(ref e)) if matches_word_name(e.name().as_ref(), context, b"tbl") => {
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(OxmlError::MissingElement("closing w:tbl".to_string()));
                }
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(CT_Tbl {
            properties,
            grid,
            rows,
            extra_xml,
        })
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        self.to_xml_with_context(writer, &NamespaceContext::default())
    }

    pub(crate) fn to_xml_with_context<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        context: &NamespaceContext,
    ) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("w:tbl")))?;

        if let Some(ref props) = self.properties {
            props.to_xml(writer)?;
        }

        if let Some(ref grid) = self.grid {
            grid.to_xml(writer)?;
        }

        for (idx, row) in self.rows.iter().enumerate() {
            write_extras_at(writer, &self.extra_xml, idx)?;
            row.to_xml_with_context(writer, context)?;
        }
        write_extras_at(writer, &self.extra_xml, self.rows.len())?;

        writer.write_event(Event::End(BytesEnd::new("w:tbl")))?;
        Ok(())
    }
}

impl Default for CT_Tbl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::namespace::W_NS;

    fn parse_table(xml: &str) -> CT_Tbl {
        let full = format!("<w:tbl>{xml}</w:tbl>");
        let mut reader = Reader::from_str(&full);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if matches_local_name(e.name().as_ref(), b"tbl") => break,
                _ => {}
            }
            buf.clear();
        }
        CT_Tbl::from_xml(&mut reader).unwrap()
    }

    #[test]
    fn parse_simple_table() {
        let tbl = parse_table(
            r#"<w:tblPr><w:tblW w:w="5000" w:type="dxa"/></w:tblPr>
               <w:tblGrid><w:gridCol w:w="2500"/><w:gridCol w:w="2500"/></w:tblGrid>
               <w:tr>
                 <w:tc><w:p><w:r><w:t>A1</w:t></w:r></w:p></w:tc>
                 <w:tc><w:p><w:r><w:t>B1</w:t></w:r></w:p></w:tc>
               </w:tr>
               <w:tr>
                 <w:tc><w:p><w:r><w:t>A2</w:t></w:r></w:p></w:tc>
                 <w:tc><w:p><w:r><w:t>B2</w:t></w:r></w:p></w:tc>
               </w:tr>"#,
        );
        assert_eq!(tbl.rows.len(), 2);
        assert_eq!(tbl.rows[0].cells.len(), 2);
        assert_eq!(tbl.rows[0].cells[0].text(), "A1");
        assert_eq!(tbl.rows[1].cells[1].text(), "B2");

        let grid = tbl.grid.unwrap();
        assert_eq!(grid.columns.len(), 2);
        assert_eq!(grid.columns[0].width, Twips(2500));

        let pr = tbl.properties.unwrap();
        assert_eq!(pr.width.as_ref().unwrap().w, 5000);
    }

    #[test]
    fn parse_cell_merge() {
        let tbl = parse_table(
            r#"<w:tblGrid><w:gridCol w:w="2500"/><w:gridCol w:w="2500"/></w:tblGrid>
               <w:tr>
                 <w:tc>
                   <w:tcPr><w:gridSpan w:val="2"/></w:tcPr>
                   <w:p><w:r><w:t>Merged</w:t></w:r></w:p>
                 </w:tc>
               </w:tr>
               <w:tr>
                 <w:tc>
                   <w:tcPr><w:vMerge w:val="restart"/></w:tcPr>
                   <w:p><w:r><w:t>VM Start</w:t></w:r></w:p>
                 </w:tc>
                 <w:tc><w:p/></w:tc>
               </w:tr>
               <w:tr>
                 <w:tc>
                   <w:tcPr><w:vMerge/></w:tcPr>
                   <w:p/>
                 </w:tc>
                 <w:tc><w:p/></w:tc>
               </w:tr>"#,
        );

        // First row: horizontal merge
        assert_eq!(
            tbl.rows[0].cells[0].properties.as_ref().unwrap().grid_span,
            Some(2)
        );

        // Second row: vertical merge start
        assert_eq!(
            tbl.rows[1].cells[0].properties.as_ref().unwrap().v_merge,
            Some(VMerge::Restart)
        );

        // Third row: vertical merge continue
        assert_eq!(
            tbl.rows[2].cells[0].properties.as_ref().unwrap().v_merge,
            Some(VMerge::Continue)
        );
    }

    #[test]
    fn expanded_cell_merge_properties_parse_like_empty_elements() {
        let tbl = parse_table(
            r#"<w:tblGrid><w:gridCol w:w="100"/><w:gridCol w:w="100"/><w:gridCol w:w="100"/></w:tblGrid>
               <w:tr><w:tc><w:tcPr>
                 <w:gridSpan w:val="2"></w:gridSpan>
                 <w:vMerge w:val="restart"></w:vMerge>
               </w:tcPr><w:p/></w:tc><w:tc><w:p/></w:tc></w:tr>
               <w:tr><w:tc><w:tcPr><w:vMerge></w:vMerge></w:tcPr><w:p/></w:tc></w:tr>"#,
        );

        let merged = tbl.rows[0].cells[0].properties.as_ref().unwrap();
        let continued = tbl.rows[1].cells[0].properties.as_ref().unwrap();
        assert_eq!(merged.grid_span, Some(2));
        assert_eq!(merged.v_merge, Some(VMerge::Restart));
        assert_eq!(continued.v_merge, Some(VMerge::Continue));
    }

    #[test]
    fn expanded_grid_columns_parse_like_empty_elements() {
        let tbl = parse_table(concat!(
            r#"<w:tblGrid><w:gridCol w:w="100"></w:gridCol>"#,
            r#"<w:gridCol w:w="200"/></w:tblGrid>"#,
            r#"<w:tr><w:tc><w:p/></w:tc><w:tc><w:p/></w:tc></w:tr>"#,
        ));
        let widths: Vec<_> = tbl
            .grid
            .unwrap()
            .columns
            .into_iter()
            .map(|column| column.width)
            .collect();
        assert_eq!(widths, vec![Twips(100), Twips(200)]);
    }

    #[test]
    fn table_grid_reports_unmodeled_children_and_attributes() {
        let table = parse_table(concat!(
            r#"<w:tblGrid><w:gridCol w:w="100" w:vendor="x"/>"#,
            r#"<w:tblGridChange/></w:tblGrid>"#,
            r#"<w:tr><w:tc><w:p/></w:tc></w:tr>"#,
        ));

        let grid = table.grid.unwrap();
        assert_eq!(grid.columns.len(), 1);
        assert!(grid.has_unmodeled_properties);

        let table = parse_table(concat!(
            r#"<w:tblGrid><w:gridCol w:w="100" w:type="dxa"/></w:tblGrid>"#,
            r#"<w:tr><w:tc><w:p/></w:tc></w:tr>"#,
        ));
        assert!(!table.grid.unwrap().has_unmodeled_properties);
    }

    #[test]
    fn invalid_vertical_merge_value_is_rejected() {
        let full = concat!(
            r#"<w:tbl><w:tblGrid><w:gridCol w:w="100"/></w:tblGrid>"#,
            r#"<w:tr><w:tc><w:tcPr><w:vMerge w:val="vendor"/></w:tcPr>"#,
            r#"<w:p/></w:tc></w:tr></w:tbl>"#,
        );
        let mut reader = Reader::from_str(full);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if matches_local_name(e.name().as_ref(), b"tbl") => break,
                _ => {}
            }
            buf.clear();
        }
        assert!(matches!(
            CT_Tbl::from_xml(&mut reader),
            Err(OxmlError::InvalidValue(_))
        ));
    }

    #[test]
    fn parse_table_borders() {
        let tbl = parse_table(
            r#"<w:tblPr>
                 <w:tblBorders>
                   <w:top w:val="single" w:sz="4" w:color="000000"/>
                   <w:bottom w:val="single" w:sz="4" w:color="000000"/>
                   <w:left w:val="single" w:sz="4" w:color="000000"/>
                   <w:right w:val="single" w:sz="4" w:color="000000"/>
                   <w:insideH w:val="single" w:sz="4" w:color="000000"/>
                   <w:insideV w:val="single" w:sz="4" w:color="000000"/>
                 </w:tblBorders>
               </w:tblPr>
               <w:tblGrid><w:gridCol w:w="5000"/></w:tblGrid>
               <w:tr><w:tc><w:p/></w:tc></w:tr>"#,
        );

        let borders = tbl.properties.unwrap().borders.unwrap();
        assert_eq!(borders.top.unwrap().val, ST_Border::Single);
        assert_eq!(borders.inside_h.unwrap().val, ST_Border::Single);
        assert_eq!(borders.inside_v.unwrap().val, ST_Border::Single);
    }

    #[test]
    fn parse_cell_shading() {
        let tbl = parse_table(
            r#"<w:tblGrid><w:gridCol w:w="5000"/></w:tblGrid>
               <w:tr>
                 <w:tc>
                   <w:tcPr><w:shd w:val="clear" w:fill="FFFF00"/></w:tcPr>
                   <w:p/>
                 </w:tc>
               </w:tr>"#,
        );

        let shd = tbl.rows[0].cells[0]
            .properties
            .as_ref()
            .unwrap()
            .shading
            .as_ref()
            .unwrap();
        assert_eq!(shd.fill, Some("FFFF00".to_string()));
    }

    #[test]
    fn parse_row_properties() {
        let tbl = parse_table(
            r#"<w:tblGrid><w:gridCol w:w="5000"/></w:tblGrid>
               <w:tr>
                 <w:trPr>
                   <w:trHeight w:val="720" w:hRule="exact"/>
                   <w:tblHeader/>
                 </w:trPr>
                 <w:tc><w:p/></w:tc>
               </w:tr>"#,
        );

        let tr_pr = tbl.rows[0].properties.as_ref().unwrap();
        assert_eq!(tr_pr.height, Some(Twips(720)));
        assert_eq!(tr_pr.height_rule, Some("exact".to_string()));
        assert_eq!(tr_pr.header, Some(true));
    }

    #[test]
    fn round_trip_table() {
        let mut tbl = CT_Tbl::new();
        tbl.properties = Some(CT_TblPr {
            width: Some(CT_TblWidth::dxa(9000)),
            borders: Some(CT_TblBorders {
                top: Some(CT_BorderEdge {
                    val: ST_Border::Single,
                    sz: Some(4),
                    space: Some(0),
                    color: Some("000000".to_string()),
                }),
                bottom: Some(CT_BorderEdge {
                    val: ST_Border::Single,
                    sz: Some(4),
                    space: Some(0),
                    color: Some("000000".to_string()),
                }),
                ..Default::default()
            }),
            ..Default::default()
        });
        tbl.grid = Some(CT_TblGrid {
            columns: vec![
                CT_TblGridCol { width: Twips(4500) },
                CT_TblGridCol { width: Twips(4500) },
            ],
            ..Default::default()
        });

        let mut row = CT_Row::new();
        let mut cell1 = CT_Tc::new();
        cell1.paragraphs_mut()[0].add_run("Hello");
        let mut cell2 = CT_Tc::new();
        cell2.paragraphs_mut()[0].add_run("World");
        row.cells.push(cell1);
        row.cells.push(cell2);
        tbl.rows.push(row);

        // Serialize
        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        tbl.to_xml(&mut writer).unwrap();
        let xml = String::from_utf8(output).unwrap();

        // Parse back
        let parsed = parse_table(
            xml.strip_prefix("<w:tbl>")
                .unwrap()
                .strip_suffix("</w:tbl>")
                .unwrap(),
        );

        assert_eq!(parsed.rows.len(), 1);
        assert_eq!(parsed.rows[0].cells.len(), 2);
        assert_eq!(parsed.rows[0].cells[0].text(), "Hello");
        assert_eq!(parsed.rows[0].cells[1].text(), "World");

        let grid = parsed.grid.unwrap();
        assert_eq!(grid.columns.len(), 2);
        assert_eq!(grid.columns[0].width, Twips(4500));

        let borders = parsed.properties.unwrap().borders.unwrap();
        assert!(borders.top.is_some());
        assert!(borders.bottom.is_some());
    }

    #[test]
    fn nested_table_xml_round_trip() {
        use crate::text::CT_P;

        // Build a cell containing a paragraph + a nested table
        let mut outer_cell = CT_Tc::new();
        outer_cell.paragraphs_mut()[0].add_run("Before table");

        let mut nested_tbl = CT_Tbl::new();
        nested_tbl.grid = Some(CT_TblGrid {
            columns: vec![CT_TblGridCol { width: Twips(2000) }],
            ..Default::default()
        });
        let mut nested_row = CT_Row::new();
        let mut nested_cell = CT_Tc::new();
        nested_cell.paragraphs_mut()[0].add_run("Nested content");
        nested_row.cells.push(nested_cell);
        nested_tbl.rows.push(nested_row);

        outer_cell.content.push(CellContent::Table(nested_tbl));

        let mut after = CT_P::new();
        after.add_run("After table");
        outer_cell.content.push(CellContent::Paragraph(after));

        // Serialize
        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        outer_cell.to_xml(&mut writer).unwrap();
        let xml = String::from_utf8(output).unwrap();

        // Should contain nested <w:tbl>
        assert!(xml.contains("<w:tbl>"));
        assert!(xml.contains("Nested content"));

        // Parse back
        let inner_xml = xml
            .strip_prefix("<w:tc>")
            .unwrap()
            .strip_suffix("</w:tc>")
            .unwrap();
        let full_xml = format!(
            "<w:tc xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">{inner_xml}</w:tc>"
        );
        let mut reader = Reader::from_str(&full_xml);
        reader.config_mut().trim_text(true);
        // Skip start tag
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) if e.local_name().as_ref() == b"tc" => break,
                _ => {}
            }
        }
        let parsed = CT_Tc::from_xml(&mut reader).unwrap();

        // Check structure: 2 paragraphs + 1 nested table
        assert_eq!(parsed.paragraphs().len(), 2);
        assert_eq!(parsed.paragraphs()[0].text(), "Before table");
        assert_eq!(parsed.paragraphs()[1].text(), "After table");

        // Check nested table
        let tables: Vec<_> = parsed
            .content
            .iter()
            .filter_map(|c| match c {
                CellContent::Table(t) => Some(t),
                _ => None,
            })
            .collect();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].rows.len(), 1);
        assert_eq!(tables[0].rows[0].cells[0].text(), "Nested content");
    }

    #[test]
    fn paragraphs_method_backward_compat() {
        let mut cell = CT_Tc::new();
        // Cell starts with one empty paragraph
        assert_eq!(cell.paragraphs().len(), 1);

        // Add a run to existing paragraph
        cell.paragraphs_mut()[0].add_run("First");

        // Add a nested table (should not appear in paragraphs())
        let nested = CT_Tbl::new();
        cell.content.push(CellContent::Table(nested));

        // Add another paragraph
        let mut p = CT_P::new();
        p.add_run("Second");
        cell.content.push(CellContent::Paragraph(p));

        // paragraphs() should return only the 2 CT_P items
        assert_eq!(cell.paragraphs().len(), 2);
        assert_eq!(cell.paragraphs()[0].text(), "First");
        assert_eq!(cell.paragraphs()[1].text(), "Second");

        // text() should concat paragraph text with newline separator
        assert_eq!(cell.text(), "First\nSecond");
    }

    /// Serialize a table and return the XML, for the fidelity tests below.
    fn table_to_xml(tbl: &CT_Tbl) -> String {
        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        tbl.to_xml(&mut writer).unwrap();
        String::from_utf8(output).unwrap()
    }

    /// Table children we do not model must survive a read and write cycle.
    ///
    /// These used to be dropped, which silently deleted whole rows, cells and
    /// paragraphs whenever they were wrapped in a content control, and lost
    /// the bookmarks that cross references and a table of figures rely on.
    #[test]
    fn unknown_table_children_round_trip() {
        const GRID: &str = r#"<w:tblGrid><w:gridCol w:w="4675"/></w:tblGrid>"#;

        for (label, inner) in [
            (
                "row wrapped in a content control",
                format!(
                    r#"{GRID}<w:sdt><w:sdtContent><w:tr><w:tc><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc></w:tr></w:sdtContent></w:sdt>"#
                ),
            ),
            (
                "cell wrapped in a content control",
                format!(
                    r#"{GRID}<w:tr><w:sdt><w:sdtContent><w:tc><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc></w:sdtContent></w:sdt></w:tr>"#
                ),
            ),
            (
                "paragraph wrapped in a content control",
                format!(
                    r#"{GRID}<w:tr><w:tc><w:sdt><w:sdtContent><w:p><w:r><w:t>x</w:t></w:r></w:p></w:sdtContent></w:sdt></w:tc></w:tr>"#
                ),
            ),
            (
                "bookmark at table level",
                format!(
                    r#"{GRID}<w:bookmarkStart w:id="1" w:name="b"/><w:tr><w:tc><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc></w:tr>"#
                ),
            ),
            (
                "bookmark at row level",
                format!(
                    r#"{GRID}<w:tr><w:bookmarkStart w:id="1" w:name="b"/><w:tc><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc></w:tr>"#
                ),
            ),
        ] {
            let tbl = parse_table(&inner);
            let xml = table_to_xml(&tbl);
            assert_eq!(
                xml,
                format!("<w:tbl>{inner}</w:tbl>"),
                "{label} was not preserved"
            );
        }
    }

    #[test]
    fn row_grid_offsets_round_trip_in_schema_order() {
        let inner = concat!(
            r#"<w:tblGrid><w:gridCol w:w="100"/><w:gridCol w:w="100"/></w:tblGrid>"#,
            r#"<w:tr><w:trPr><w:gridBefore w:val="2"/><w:gridAfter w:val="1"/>"#,
            r#"<w:cantSplit/><w:trHeight w:val="240"/><w:tblHeader/><w:jc w:val="center"/>"#,
            r#"</w:trPr><w:tc><w:p/></w:tc></w:tr>"#,
        );
        let table = parse_table(inner);
        let properties = table.rows[0].properties.as_ref().unwrap();

        assert_eq!(properties.grid_before, Some(2));
        assert_eq!(properties.grid_after, Some(1));
        let xml = table_to_xml(&table);
        assert!(
            xml.contains(concat!(
                r#"<w:trPr><w:gridBefore w:val="2"/><w:gridAfter w:val="1"/>"#,
                r#"<w:cantSplit/><w:trHeight w:val="240"/><w:tblHeader/>"#,
                r#"<w:jc w:val="center"/></w:trPr>"#,
            )),
            "{xml}"
        );
    }

    #[test]
    fn expanded_row_grid_offsets_parse_like_empty_elements() {
        let inner = concat!(
            r#"<w:tblGrid><w:gridCol w:w="100"/></w:tblGrid>"#,
            r#"<w:tr><w:trPr><w:gridBefore w:val="2"></w:gridBefore>"#,
            r#"<w:gridAfter w:val="1"></w:gridAfter></w:trPr>"#,
            r#"<w:tc><w:p/></w:tc></w:tr>"#,
        );
        let table = parse_table(inner);
        let properties = table.rows[0].properties.as_ref().unwrap();

        assert_eq!(properties.grid_before, Some(2));
        assert_eq!(properties.grid_after, Some(1));
    }

    /// A styled table must keep the markup that says which conditional parts
    /// of its style apply.
    ///
    /// `w:tblStyle` alone is not enough. `w:tblLook` and `w:cnfStyle` are what
    /// turn on the header row, banding and first column formatting, so losing
    /// them leaves the style name intact and the table drawn with base
    /// formatting only, which reads as the style having been lost.
    #[test]
    fn table_style_conditional_formatting_round_trips() {
        let inner = concat!(
            r#"<w:tblPr><w:tblStyle w:val="GridTable4-Accent1"/>"#,
            r#"<w:tblLook w:val="04A0" w:firstRow="1" w:lastRow="0" w:firstColumn="1" w:lastColumn="0" w:noHBand="0" w:noVBand="1"/>"#,
            r#"</w:tblPr><w:tblGrid><w:gridCol w:w="4675"/></w:tblGrid>"#,
            r#"<w:tr><w:trPr><w:cnfStyle w:val="100000000000"/></w:trPr>"#,
            r#"<w:tc><w:tcPr><w:cnfStyle w:val="001000000000"/></w:tcPr><w:p/></w:tc>"#,
            r#"</w:tr>"#,
        );
        let tbl = parse_table(inner);

        let look = tbl
            .properties
            .as_ref()
            .and_then(|p| p.look.as_ref())
            .expect("tblLook should be parsed");
        assert_eq!(look.val.as_deref(), Some("04A0"));
        assert_eq!(look.first_row, Some(true));
        assert_eq!(look.last_row, Some(false));
        assert_eq!(look.first_column, Some(true));
        assert_eq!(look.no_v_band, Some(true));

        assert_eq!(
            tbl.rows[0]
                .properties
                .as_ref()
                .and_then(|p| p.cnf_style.as_deref()),
            Some("100000000000")
        );
        assert_eq!(
            tbl.rows[0].cells[0]
                .properties
                .as_ref()
                .and_then(|p| p.cnf_style.as_deref()),
            Some("001000000000")
        );

        assert_eq!(
            table_to_xml(&tbl),
            format!("<w:tbl>{inner}</w:tbl>").replace("<w:p/>", "<w:p></w:p>"),
            "the whole thing must survive a write"
        );
    }

    /// A row or cell carrying only cnfStyle is not empty.
    ///
    /// Both types skip writing their properties when every field is unset, so
    /// a new field that is not in that check is parsed and then silently
    /// dropped on the way out.
    #[test]
    fn properties_holding_only_cnf_style_are_still_written() {
        let tbl = parse_table(concat!(
            r#"<w:tblGrid><w:gridCol w:w="100"/></w:tblGrid>"#,
            r#"<w:tr><w:trPr><w:cnfStyle w:val="100000000000"/></w:trPr>"#,
            r#"<w:tc><w:tcPr><w:cnfStyle w:val="001000000000"/></w:tcPr><w:p/></w:tc></w:tr>"#,
        ));
        let xml = table_to_xml(&tbl);
        assert!(
            xml.contains(r#"<w:trPr><w:cnfStyle w:val="100000000000"/></w:trPr>"#),
            "{xml}"
        );
        assert!(
            xml.contains(r#"<w:tcPr><w:cnfStyle w:val="001000000000"/></w:tcPr>"#),
            "{xml}"
        );
    }

    /// OOXML booleans come in both spellings.
    #[test]
    fn tbl_look_accepts_either_boolean_spelling() {
        let tbl = parse_table(concat!(
            r#"<w:tblPr><w:tblLook w:firstRow="true" w:lastRow="false" w:noVBand="1"/></w:tblPr>"#,
            r#"<w:tblGrid><w:gridCol w:w="100"/></w:tblGrid>"#,
        ));
        let look = tbl
            .properties
            .as_ref()
            .and_then(|p| p.look.as_ref())
            .unwrap();
        assert_eq!(look.first_row, Some(true));
        assert_eq!(look.last_row, Some(false));
        assert_eq!(look.no_v_band, Some(true));
    }

    #[test]
    fn table_layout_reads_the_schema_type_attribute() {
        let table = parse_table(concat!(
            r#"<w:tblPr><w:tblLayout w:type="fixed"/></w:tblPr>"#,
            r#"<w:tblGrid><w:gridCol w:w="100"/></w:tblGrid>"#,
        ));

        assert_eq!(table.properties.unwrap().layout.as_deref(), Some("fixed"));
    }

    #[test]
    fn unmodeled_table_row_and_cell_properties_are_observable() {
        let table = parse_table(concat!(
            r#"<w:tblPr><w:bidiVisual/></w:tblPr>"#,
            r#"<w:tblGrid><w:gridCol w:w="100"/></w:tblGrid>"#,
            r#"<w:tr><w:trPr><w:tblCellSpacing/></w:trPr>"#,
            r#"<w:tc><w:tcPr><w:fitText/></w:tcPr><w:p/></w:tc></w:tr>"#,
        ));

        assert!(table.properties.as_ref().unwrap().has_unmodeled_properties);
        assert!(
            table.rows[0]
                .properties
                .as_ref()
                .unwrap()
                .has_unmodeled_properties
        );
        assert!(
            table.rows[0].cells[0]
                .properties
                .as_ref()
                .unwrap()
                .has_unmodeled_properties
        );
    }

    #[test]
    fn extension_attributes_and_nested_cell_borders_make_properties_incomplete() {
        let table = parse_table(concat!(
            r#"<w:tblPr><w:tblW w:w="100" w:type="dxa" w14:foo="1" xmlns:w14="urn:w14"/></w:tblPr>"#,
            r#"<w:tblGrid><w:gridCol w:w="100"/></w:tblGrid>"#,
            r#"<w:tr><w:trPr><w:trHeight w:val="240" w14:foo="1" xmlns:w14="urn:w14"/></w:trPr>"#,
            r#"<w:tc><w:tcPr><w:tcBorders><w:tl2br w:val="single"/></w:tcBorders>"#,
            r#"</w:tcPr><w:p/></w:tc></w:tr>"#,
        ));

        assert!(table.properties.as_ref().unwrap().has_unmodeled_properties);
        assert!(
            table.rows[0]
                .properties
                .as_ref()
                .unwrap()
                .has_unmodeled_properties
        );
        let cell_properties = table.rows[0].cells[0].properties.as_ref().unwrap();
        assert!(cell_properties.has_unmodeled_properties);
        assert!(cell_properties.borders.as_ref().unwrap().is_empty());
    }

    #[test]
    fn foreign_extension_property_children_are_observable() {
        let table = parse_table(concat!(
            r#"<w:tblPr><x:effect xmlns:x="urn:foreign"/></w:tblPr>"#,
            r#"<w:tblGrid><w:gridCol w:w="100"/></w:tblGrid>"#,
            r#"<w:tr><w:trPr><x:effect xmlns:x="urn:foreign"/></w:trPr>"#,
            r#"<w:tc><w:tcPr><x:effect xmlns:x="urn:foreign"/></w:tcPr><w:p/></w:tc>"#,
            r#"</w:tr>"#,
        ));

        assert!(table.properties.as_ref().unwrap().has_unmodeled_properties);
        assert!(
            table.rows[0]
                .properties
                .as_ref()
                .unwrap()
                .has_unmodeled_properties
        );
        assert!(
            table.rows[0].cells[0]
                .properties
                .as_ref()
                .unwrap()
                .has_unmodeled_properties
        );
    }

    #[test]
    fn recognized_table_nesting_is_bounded() {
        let mut xml = String::new();
        for _ in 0..=MAX_MODEL_DEPTH {
            xml.push_str("<w:tbl><w:tr><w:tc>");
        }
        xml.push_str("<w:p/>");
        for _ in 0..=MAX_MODEL_DEPTH {
            xml.push_str("</w:tc></w:tr></w:tbl>");
        }

        let mut reader = Reader::from_str(&xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(ref element))
                    if matches_local_name(element.name().as_ref(), b"tbl") =>
                {
                    break;
                }
                Ok(Event::Eof) => panic!("missing table root"),
                Ok(_) => {}
                Err(error) => panic!("failed before table root: {error}"),
            }
            buffer.clear();
        }

        let error = CT_Tbl::from_xml(&mut reader).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("recognized model nesting exceeds 32 levels"),
            "{error}"
        );
    }

    /// A self-closing tblPr or tblGrid must not be captured as extra XML.
    /// Both have a fixed position ahead of the rows, and extras are written
    /// from the row positions, so capturing them would reorder the children.
    #[test]
    fn self_closing_table_properties_are_not_reordered() {
        let tbl = parse_table(
            r#"<w:tblPr/><w:tblGrid><w:gridCol w:w="100"/></w:tblGrid><w:tr><w:tc><w:p/></w:tc></w:tr>"#,
        );
        assert!(tbl.extra_xml.is_empty(), "tblPr must not be captured");
        let xml = table_to_xml(&tbl);
        assert!(
            !xml.contains("</w:tr><w:tblPr/>"),
            "tblPr must never follow the rows: {xml}"
        );
    }

    #[test]
    fn table_cell_dispatch_respects_namespaces_and_empty_elements() {
        let table = parse_table(concat!(
            r#"<w:tblPr/><w:tblGrid/>"#,
            r#"<x:tr xmlns:x="urn:foreign"/><x:row xmlns:x="urn:foreign"/>"#,
            r#"<w:tr><w:trPr/><x:tc xmlns:x="urn:foreign"/><w:tc><w:tcPr/>"#,
            r#"<x:p xmlns:x="urn:foreign"/><w:p/><w:tbl/>"#,
            r#"</w:tc><w:tc/></w:tr>"#,
        ));

        assert!(table.properties.is_some());
        assert!(table.grid.is_some());
        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.extra_xml.len(), 2);
        assert_eq!(table.rows[0].cells.len(), 2);
        assert_eq!(table.rows[0].extra_xml.len(), 1);
        assert!(matches!(
            table.rows[0].cells[0].content[0],
            CellContent::Unsupported(_)
        ));
        assert!(matches!(
            table.rows[0].cells[0].content[1],
            CellContent::Paragraph(_)
        ));
        assert!(matches!(
            table.rows[0].cells[0].content[2],
            CellContent::Table(_)
        ));

        let xml = table_to_xml(&table);
        let foreign_tr = xml.find("<x:tr").unwrap();
        let foreign_row = xml.find("<x:row").unwrap();
        let word_row = xml.find("<w:tr>").unwrap();
        assert!(foreign_tr < foreign_row && foreign_row < word_row, "{xml}");
    }

    #[test]
    fn table_toggles_preserve_explicit_false() {
        let table = parse_table(concat!(
            r#"<w:tblGrid><w:gridCol w:w="100"/></w:tblGrid>"#,
            r#"<w:tr><w:trPr><w:tblHeader w:val="false"/><w:cantSplit w:val="0"/></w:trPr>"#,
            r#"<w:tc><w:tcPr><w:noWrap w:val="off"/></w:tcPr><w:p/></w:tc></w:tr>"#,
        ));

        let row = table.rows[0].properties.as_ref().unwrap();
        let cell = table.rows[0].cells[0].properties.as_ref().unwrap();
        assert_eq!(row.header, Some(false));
        assert_eq!(row.cant_split, Some(false));
        assert_eq!(cell.no_wrap, Some(false));
    }

    #[test]
    fn horizontal_merge_is_exposed_and_zero_grid_span_is_rejected() {
        let table = parse_table(concat!(
            r#"<w:tblGrid><w:gridCol w:w="100"/></w:tblGrid>"#,
            r#"<w:tr><w:tc><w:tcPr><w:hMerge w:val="restart"/></w:tcPr><w:p/></w:tc></w:tr>"#,
        ));
        assert_eq!(
            table.rows[0].cells[0].properties.as_ref().unwrap().h_merge,
            Some(VMerge::Restart)
        );

        let full = format!(
            r#"<w:tbl xmlns:w="{W_NS}"><w:tblGrid><w:gridCol w:w="100"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:gridSpan w:val="0"/></w:tcPr><w:p/></w:tc></w:tr></w:tbl>"#
        );
        let mut reader = Reader::from_str(&full);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref element))
                    if matches_local_name(element.name().as_ref(), b"tbl") =>
                {
                    break;
                }
                _ => buf.clear(),
            }
        }
        assert!(matches!(
            CT_Tbl::from_xml(&mut reader),
            Err(OxmlError::InvalidValue(_))
        ));
    }
}
