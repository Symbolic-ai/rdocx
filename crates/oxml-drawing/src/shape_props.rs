use std::fmt;
use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_core::xml::{local_name, matches_local_name};
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer, XmlVersion};

use crate::effect::{
    CT_EffectList, EffectError, raw_contains_placeholder_color, raw_is_effect_dag,
};
use crate::fill::{Fill, FillError};
use crate::geometry::{CT_CustomGeometry2D, GeometryError};
use crate::line::{CT_LineProperties, LineError};
use crate::order::OrderedRawChildren;
use crate::xfrm::{CT_Transform2D, TransformError};

/// Errors produced while parsing or writing DrawingML shape properties.
#[derive(Debug)]
pub enum ShapePropertiesError {
    Xml(OxmlError),
    Transform(TransformError),
    Geometry(GeometryError),
    Fill(FillError),
    Line(LineError),
    Effect(EffectError),
    UnexpectedElement(String),
}

impl fmt::Display for ShapePropertiesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml(error) => error.fmt(formatter),
            Self::Transform(error) => error.fmt(formatter),
            Self::Geometry(error) => error.fmt(formatter),
            Self::Fill(error) => error.fmt(formatter),
            Self::Line(error) => error.fmt(formatter),
            Self::Effect(error) => error.fmt(formatter),
            Self::UnexpectedElement(element) => {
                write!(
                    formatter,
                    "unexpected DrawingML shape-properties element: {element}"
                )
            }
        }
    }
}

impl std::error::Error for ShapePropertiesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml(error) => Some(error),
            Self::Transform(error) => Some(error),
            Self::Geometry(error) => Some(error),
            Self::Fill(error) => Some(error),
            Self::Line(error) => Some(error),
            Self::Effect(error) => Some(error),
            Self::UnexpectedElement(_) => None,
        }
    }
}

impl From<OxmlError> for ShapePropertiesError {
    fn from(error: OxmlError) -> Self {
        Self::Xml(error)
    }
}

impl From<TransformError> for ShapePropertiesError {
    fn from(error: TransformError) -> Self {
        Self::Transform(error)
    }
}

impl From<GeometryError> for ShapePropertiesError {
    fn from(error: GeometryError) -> Self {
        Self::Geometry(error)
    }
}

impl From<FillError> for ShapePropertiesError {
    fn from(error: FillError) -> Self {
        Self::Fill(error)
    }
}

impl From<LineError> for ShapePropertiesError {
    fn from(error: LineError) -> Self {
        Self::Line(error)
    }
}

impl From<EffectError> for ShapePropertiesError {
    fn from(error: EffectError) -> Self {
        Self::Effect(error)
    }
}

pub type Result<T> = std::result::Result<T, ShapePropertiesError>;

/// The modelled children of one DrawingML `a:spPr` element.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CT_ShapeProperties {
    pub transform: Option<CT_Transform2D>,
    pub custom_geometry: Option<CT_CustomGeometry2D>,
    pub fill: Option<Fill>,
    pub line: Option<CT_LineProperties>,
    pub effects: Option<CT_EffectList>,
    raw_attributes: Vec<(String, String)>,
    raw_children: OrderedRawChildren,
}

impl CT_ShapeProperties {
    /// Parses one complete `a:spPr` element with any namespace prefix.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) if matches_local_name(element.name().as_ref(), b"spPr") => {
                    return Self::from_element(&mut reader, &element);
                }
                Event::Empty(element) if matches_local_name(element.name().as_ref(), b"spPr") => {
                    return Self::from_start(&element);
                }
                Event::Start(element) | Event::Empty(element) => {
                    return Err(ShapePropertiesError::UnexpectedElement(element_name(
                        &element,
                    )));
                }
                Event::Eof => {
                    return Err(ShapePropertiesError::Xml(OxmlError::MissingElement(
                        "DrawingML shape properties".to_owned(),
                    )));
                }
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_start(start: &BytesStart<'_>) -> Result<Self> {
        Ok(Self {
            raw_attributes: capture_raw_attributes(start)?,
            ..Self::default()
        })
    }

    fn from_element(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut properties = Self::from_start(start)?;
        let mut boundary = 0;
        let mut buffer = Vec::new();

        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) => {
                    let name = local_name(element.name().as_ref()).to_vec();
                    let raw = capture_element(reader, &element)?;
                    if properties.parse_modelled(&name, &raw)? {
                        boundary = boundary.max(modelled_boundary(&name));
                    } else {
                        properties.raw_children.push(boundary, raw);
                        boundary = boundary.max(raw_boundary_after(&name));
                    }
                }
                Event::Empty(element) => {
                    let name = local_name(element.name().as_ref()).to_vec();
                    let raw = capture_empty_element(&element)?;
                    if properties.parse_modelled(&name, &raw)? {
                        boundary = boundary.max(modelled_boundary(&name));
                    } else {
                        properties.raw_children.push(boundary, raw);
                        boundary = boundary.max(raw_boundary_after(&name));
                    }
                }
                Event::End(element) if matches_local_name(element.name().as_ref(), b"spPr") => {
                    break;
                }
                Event::Eof => return Err(missing_end()),
                _ => {}
            }
            buffer.clear();
        }
        Ok(properties)
    }

    fn parse_modelled(&mut self, name: &[u8], raw: &[u8]) -> Result<bool> {
        match name {
            b"xfrm" if self.transform.is_none() => {
                self.transform = Some(CT_Transform2D::from_xml(raw)?);
            }
            b"custGeom" if self.custom_geometry.is_none() => {
                self.custom_geometry = Some(CT_CustomGeometry2D::from_xml(raw)?);
            }
            name if is_fill(name) && self.fill.is_none() => {
                self.fill = Some(Fill::from_xml(raw)?);
            }
            b"ln" if self.line.is_none() => {
                self.line = Some(CT_LineProperties::from_xml(raw)?);
            }
            b"effectLst" if self.effects.is_none() => {
                self.effects = Some(CT_EffectList::from_xml(raw)?);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// Writes shape properties with the fixed `a:` prefix and schema order.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        self.write_xml(&mut writer)?;
        Ok(writer.into_inner())
    }

    /// Writes shape properties into an existing XML writer.
    pub fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        self.write_xml_as(writer, "a:spPr")
    }

    /// Writes shape properties under the caller's required root name.
    pub fn write_xml_as<W: Write>(&self, writer: &mut Writer<W>, name: &str) -> Result<()> {
        let mut start = BytesStart::new(name);
        push_raw_attributes(&mut start, &self.raw_attributes);
        if self.transform.is_none()
            && self.custom_geometry.is_none()
            && self.fill.is_none()
            && self.line.is_none()
            && self.effects.is_none()
            && self.raw_children.is_empty()
        {
            writer
                .write_event(Event::Empty(start))
                .map_err(OxmlError::from)?;
            return Ok(());
        }

        writer
            .write_event(Event::Start(start))
            .map_err(OxmlError::from)?;
        emit_raw(writer, self.raw_children.at(0))?;
        if let Some(transform) = &self.transform {
            transform.write_xml(writer)?;
        }
        emit_raw(writer, self.raw_children.at(1))?;
        if let Some(geometry) = &self.custom_geometry {
            writer
                .get_mut()
                .write_all(&geometry.to_xml()?)
                .map_err(OxmlError::from)?;
        }
        emit_raw(writer, self.raw_children.at(2))?;
        if let Some(fill) = &self.fill {
            fill.write_xml(writer)?;
        }
        emit_raw(writer, self.raw_children.at(3))?;
        if let Some(line) = &self.line {
            line.write_xml(writer)?;
        }
        emit_raw(writer, self.raw_children.at(4))?;
        if let Some(effects) = &self.effects {
            effects.write_xml(writer)?;
        }
        for boundary in 5..=8 {
            emit_raw(writer, self.raw_children.at(boundary))?;
        }
        writer
            .write_event(Event::End(BytesEnd::new(name)))
            .map_err(OxmlError::from)?;
        Ok(())
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }

    /// Returns whether an opaque effect DAG is present instead of a typed list.
    pub fn has_unmodelled_effect(&self) -> bool {
        (0..=8).any(|boundary| self.raw_children.at(boundary).any(raw_is_effect_dag))
    }

    /// Reports a placeholder colour inside an opaque effect DAG.
    pub fn has_unmodelled_effect_placeholder_color(&self) -> bool {
        (0..=8).any(|boundary| {
            self.raw_children
                .at(boundary)
                .any(|xml| raw_is_effect_dag(xml) && raw_contains_placeholder_color(xml))
        })
    }
}

fn is_fill(name: &[u8]) -> bool {
    matches!(
        name,
        b"noFill" | b"solidFill" | b"gradFill" | b"pattFill" | b"blipFill"
    )
}

fn modelled_boundary(name: &[u8]) -> usize {
    match name {
        b"xfrm" => 1,
        b"custGeom" => 2,
        name if is_fill(name) => 3,
        b"ln" => 4,
        b"effectLst" => 5,
        _ => 0,
    }
}

fn raw_boundary_after(name: &[u8]) -> usize {
    match name {
        b"xfrm" => 1,
        b"custGeom" | b"prstGeom" => 2,
        b"noFill" | b"solidFill" | b"gradFill" | b"blipFill" | b"pattFill" | b"grpFill" => 3,
        b"ln" => 4,
        b"effectLst" | b"effectDag" => 5,
        b"scene3d" => 6,
        b"sp3d" => 7,
        b"extLst" => 8,
        _ => 0,
    }
}

fn emit_raw<'a, W: Write>(
    writer: &mut Writer<W>,
    children: impl Iterator<Item = &'a [u8]>,
) -> Result<()> {
    for child in children {
        writer.get_mut().write_all(child).map_err(OxmlError::from)?;
    }
    Ok(())
}

fn capture_raw_attributes(start: &BytesStart<'_>) -> Result<Vec<(String, String)>> {
    let mut raw = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        let name = std::str::from_utf8(attribute.key.as_ref())
            .map_err(OxmlError::from)?
            .to_owned();
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())
            .map_err(OxmlError::from)?
            .into_owned();
        raw.push((name, value));
    }
    Ok(raw)
}

fn push_raw_attributes(start: &mut BytesStart<'_>, attributes: &[(String, String)]) {
    for (name, value) in attributes {
        start.push_attribute((name.as_str(), value.as_str()));
    }
}

fn element_name(element: &BytesStart<'_>) -> String {
    String::from_utf8_lossy(element.name().as_ref()).into_owned()
}

fn missing_end() -> ShapePropertiesError {
    ShapePropertiesError::Xml(OxmlError::MissingElement(
        "closing DrawingML shape properties".to_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::CT_ShapeProperties;

    #[test]
    fn shape_properties_round_trip_in_schema_order() {
        let xml = br#"<z:spPr><x:before/><z:xfrm rot="60000"><z:off x="1" y="2"/></z:xfrm><x:afterXfrm/><z:custGeom><z:pathLst><z:path/></z:pathLst></z:custGeom><x:afterGeom/><z:solidFill><z:srgbClr val="112233"/></z:solidFill><x:afterFill/><z:ln w="12700"><z:noFill/></z:ln><x:afterLine/><z:effectLst><z:glow rad="40000"><z:srgbClr val="445566"/></z:glow></z:effectLst><x:afterEffects/></z:spPr>"#;
        let properties = CT_ShapeProperties::from_xml(xml).unwrap();
        assert!(properties.transform.is_some());
        assert!(properties.custom_geometry.is_some());
        assert!(properties.fill.is_some());
        assert!(properties.line.is_some());
        assert!(properties.effects.is_some());

        let written = properties.to_xml().unwrap();
        assert_eq!(written, br#"<a:spPr><x:before/><a:xfrm rot="60000"><a:off x="1" y="2"/></a:xfrm><x:afterXfrm/><a:custGeom><a:pathLst><a:path/></a:pathLst></a:custGeom><x:afterGeom/><a:solidFill><a:srgbClr val="112233"/></a:solidFill><x:afterFill/><a:ln w="12700"><a:noFill/></a:ln><x:afterLine/><a:effectLst><z:glow rad="40000"><z:srgbClr val="445566"/></z:glow></a:effectLst><x:afterEffects/></a:spPr>"#);
        assert_eq!(CT_ShapeProperties::from_xml(&written).unwrap(), properties);
    }

    #[test]
    fn shape_property_root_attributes_round_trip_without_loss() {
        let properties =
            CT_ShapeProperties::from_xml(br#"<q:spPr bwMode="gray" x:future="keep &amp; stay"/>"#)
                .unwrap();
        assert_eq!(
            properties.to_xml().unwrap(),
            br#"<a:spPr bwMode="gray" x:future="keep &amp; stay"/>"#
        );
    }
}
