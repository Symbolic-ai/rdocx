use std::fmt;
use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_core::units::{Angle, Emu};
use oxml_core::xml::{get_attr, local_name, matches_local_name};
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::order::OrderedRawChildren;

/// Errors produced while parsing, writing, or resolving DrawingML transforms.
#[derive(Debug)]
pub enum TransformError {
    Xml(OxmlError),
    UnexpectedElement(String),
    MissingAttribute {
        element: String,
        attribute: String,
    },
    InvalidAttribute {
        element: String,
        attribute: String,
        value: String,
    },
    ZeroChildExtent {
        axis: &'static str,
    },
    NonFiniteMatrix,
}

impl fmt::Display for TransformError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml(error) => error.fmt(formatter),
            Self::UnexpectedElement(element) => {
                write!(
                    formatter,
                    "unexpected DrawingML transform element: {element}"
                )
            }
            Self::MissingAttribute { element, attribute } => {
                write!(formatter, "DrawingML {element} requires @{attribute}")
            }
            Self::InvalidAttribute {
                element,
                attribute,
                value,
            } => write!(
                formatter,
                "DrawingML {element} has invalid @{attribute}: {value}"
            ),
            Self::ZeroChildExtent { axis } => {
                write!(
                    formatter,
                    "DrawingML child extent is zero on the {axis} axis"
                )
            }
            Self::NonFiniteMatrix => write!(formatter, "DrawingML transform matrix is not finite"),
        }
    }
}

impl std::error::Error for TransformError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml(error) => Some(error),
            _ => None,
        }
    }
}

impl From<OxmlError> for TransformError {
    fn from(error: OxmlError) -> Self {
        Self::Xml(error)
    }
}

pub type Result<T> = std::result::Result<T, TransformError>;

/// A DrawingML two-dimensional coordinate pair.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CT_Point2D {
    pub x: Emu,
    pub y: Emu,
}

/// A DrawingML width and height pair.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CT_PositiveSize2D {
    pub cx: Emu,
    pub cy: Emu,
}

/// The offset, extent, rotation, and flips carried by `a:xfrm`.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CT_Transform2D {
    pub offset: Option<CT_Point2D>,
    pub extent: Option<CT_PositiveSize2D>,
    pub child_offset: Option<CT_Point2D>,
    pub child_extent: Option<CT_PositiveSize2D>,
    pub rotation: Angle,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    raw_children: OrderedRawChildren,
}

impl CT_Transform2D {
    /// Parses one complete `a:xfrm` element with any namespace prefix.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) if matches_local_name(element.name().as_ref(), b"xfrm") => {
                    return Self::from_element(&mut reader, &element);
                }
                Event::Empty(element) if matches_local_name(element.name().as_ref(), b"xfrm") => {
                    return Self::from_empty_element(&element);
                }
                Event::Start(element) | Event::Empty(element) => {
                    return Err(TransformError::UnexpectedElement(
                        String::from_utf8_lossy(element.name().as_ref()).into_owned(),
                    ));
                }
                Event::Eof => {
                    return Err(TransformError::Xml(OxmlError::MissingElement(
                        "a:xfrm".to_owned(),
                    )));
                }
                _ => {}
            }
            buffer.clear();
        }
    }

    /// Parses an `a:xfrm` after the caller has consumed its start event.
    pub fn from_element(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut transform = Self::from_empty_element(start)?;
        transform.read_children(reader)?;
        Ok(transform)
    }

    /// Parses a self-closing `a:xfrm` element.
    pub fn from_empty_element(start: &BytesStart<'_>) -> Result<Self> {
        if !matches_local_name(start.name().as_ref(), b"xfrm") {
            return Err(TransformError::UnexpectedElement(
                String::from_utf8_lossy(start.name().as_ref()).into_owned(),
            ));
        }
        Ok(Self {
            rotation: optional_i32(start, b"rot")?.map_or(Angle::default(), Angle),
            flip_horizontal: optional_bool(start, b"flipH")?.unwrap_or(false),
            flip_vertical: optional_bool(start, b"flipV")?.unwrap_or(false),
            ..Self::default()
        })
    }

    /// Writes this transform with the canonical `a:` prefix and schema order.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        self.write_xml(&mut writer)?;
        Ok(writer.into_inner())
    }

    /// Writes this transform into an existing XML writer.
    pub fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut start = BytesStart::new("a:xfrm");
        let rotation = (self.rotation.0 != 0).then(|| self.rotation.0.to_string());
        if let Some(rotation) = rotation.as_deref() {
            start.push_attribute(("rot", rotation));
        }
        if self.flip_horizontal {
            start.push_attribute(("flipH", "1"));
        }
        if self.flip_vertical {
            start.push_attribute(("flipV", "1"));
        }

        if self.offset.is_none()
            && self.extent.is_none()
            && self.child_offset.is_none()
            && self.child_extent.is_none()
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
        if let Some(offset) = self.offset {
            write_point(writer, "a:off", offset)?;
        }
        emit_raw(writer, self.raw_children.at(1))?;
        if let Some(extent) = self.extent {
            write_size(writer, "a:ext", extent)?;
        }
        emit_raw(writer, self.raw_children.at(2))?;
        if let Some(offset) = self.child_offset {
            write_point(writer, "a:chOff", offset)?;
        }
        emit_raw(writer, self.raw_children.at(3))?;
        if let Some(extent) = self.child_extent {
            write_size(writer, "a:chExt", extent)?;
        }
        emit_raw(writer, self.raw_children.at(4))?;
        writer
            .write_event(Event::End(BytesEnd::new("a:xfrm")))
            .map_err(OxmlError::from)?;
        Ok(())
    }

    /// Returns the affine coefficients in PDF matrix order `a, b, c, d, e, f`.
    pub fn matrix(&self) -> Result<[f64; 6]> {
        let offset = self.offset.unwrap_or_default();
        let extent = self.extent.unwrap_or_default();
        let child_offset = self.child_offset.unwrap_or_default();
        let scale = match self.child_extent {
            Some(child_extent) => {
                if child_extent.cx.0 == 0 {
                    return Err(TransformError::ZeroChildExtent { axis: "x" });
                }
                if child_extent.cy.0 == 0 {
                    return Err(TransformError::ZeroChildExtent { axis: "y" });
                }
                [
                    extent.cx.0 as f64 / child_extent.cx.0 as f64,
                    extent.cy.0 as f64 / child_extent.cy.0 as f64,
                ]
            }
            None => [1.0, 1.0],
        };

        let centre_x = offset.x.0 as f64 + extent.cx.0 as f64 / 2.0;
        let centre_y = offset.y.0 as f64 + extent.cy.0 as f64 / 2.0;
        let mut matrix = affine_identity();
        matrix = affine_then(
            matrix,
            affine_translation(-(child_offset.x.0 as f64), -(child_offset.y.0 as f64)),
        );
        matrix = affine_then(matrix, affine_scale(scale[0], scale[1]));
        matrix = affine_then(
            matrix,
            affine_translation(offset.x.0 as f64, offset.y.0 as f64),
        );
        matrix = affine_then(
            matrix,
            affine_rotation_about(self.rotation.to_degrees(), centre_x, centre_y),
        );
        matrix = affine_then(
            matrix,
            affine_scale_about(
                if self.flip_horizontal { -1.0 } else { 1.0 },
                if self.flip_vertical { -1.0 } else { 1.0 },
                centre_x,
                centre_y,
            ),
        );

        if matrix.into_iter().all(f64::is_finite) {
            Ok(matrix)
        } else {
            Err(TransformError::NonFiniteMatrix)
        }
    }

    /// Returns raw, not-yet-modelled children grouped by schema boundary.
    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }

    fn read_children(&mut self, reader: &mut Reader<&[u8]>) -> Result<()> {
        let mut boundary = 0;
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) => {
                    let slot = schema_slot(element.name().as_ref());
                    let raw = capture_element(reader, &element)?;
                    if let Some(slot) = slot.filter(|_| is_explicit_empty_element(&raw)) {
                        if self.set_modelled(slot, &element)? {
                            boundary = boundary.max(slot);
                        } else {
                            self.raw_children.push(boundary, raw);
                        }
                    } else {
                        self.raw_children.push(boundary, raw);
                    }
                }
                Event::Empty(element) => {
                    if let Some(slot) = schema_slot(element.name().as_ref()) {
                        if self.set_modelled(slot, &element)? {
                            boundary = boundary.max(slot);
                        } else {
                            self.raw_children
                                .push(boundary, capture_empty_element(&element)?);
                        }
                    } else {
                        self.raw_children
                            .push(boundary, capture_empty_element(&element)?);
                    }
                }
                Event::End(element) if matches_local_name(element.name().as_ref(), b"xfrm") => {
                    break;
                }
                Event::Eof => {
                    return Err(TransformError::Xml(OxmlError::MissingElement(
                        "closing a:xfrm".to_owned(),
                    )));
                }
                _ => {}
            }
            buffer.clear();
        }
        Ok(())
    }

    fn set_modelled(&mut self, slot: usize, element: &BytesStart<'_>) -> Result<bool> {
        match slot {
            1 if self.offset.is_none() => self.offset = Some(parse_point(element)?),
            2 if self.extent.is_none() => self.extent = Some(parse_size(element)?),
            3 if self.child_offset.is_none() => self.child_offset = Some(parse_point(element)?),
            4 if self.child_extent.is_none() => self.child_extent = Some(parse_size(element)?),
            1..=4 => return Ok(false),
            _ => unreachable!("schema slots are limited to one through four"),
        }
        Ok(true)
    }
}

fn schema_slot(name: &[u8]) -> Option<usize> {
    match local_name(name) {
        b"off" => Some(1),
        b"ext" => Some(2),
        b"chOff" => Some(3),
        b"chExt" => Some(4),
        _ => None,
    }
}

fn parse_point(element: &BytesStart<'_>) -> Result<CT_Point2D> {
    Ok(CT_Point2D {
        x: Emu(required_i64(element, b"x")?),
        y: Emu(required_i64(element, b"y")?),
    })
}

fn parse_size(element: &BytesStart<'_>) -> Result<CT_PositiveSize2D> {
    let cx = required_i64(element, b"cx")?;
    let cy = required_i64(element, b"cy")?;
    if cx < 0 {
        return Err(invalid_attribute(element, b"cx", cx.to_string()));
    }
    if cy < 0 {
        return Err(invalid_attribute(element, b"cy", cy.to_string()));
    }
    Ok(CT_PositiveSize2D {
        cx: Emu(cx),
        cy: Emu(cy),
    })
}

fn required_i64(element: &BytesStart<'_>, attribute: &[u8]) -> Result<i64> {
    let value = get_attr(element, attribute).ok_or_else(|| TransformError::MissingAttribute {
        element: element_local_name(element),
        attribute: String::from_utf8_lossy(attribute).into_owned(),
    })?;
    value
        .parse()
        .map_err(|_| invalid_attribute(element, attribute, value))
}

fn optional_i32(element: &BytesStart<'_>, attribute: &[u8]) -> Result<Option<i32>> {
    get_attr(element, attribute)
        .map(|value| {
            value
                .parse()
                .map_err(|_| invalid_attribute(element, attribute, value))
        })
        .transpose()
}

fn optional_bool(element: &BytesStart<'_>, attribute: &[u8]) -> Result<Option<bool>> {
    get_attr(element, attribute)
        .map(|value| match value.as_str() {
            "1" | "true" => Ok(true),
            "0" | "false" => Ok(false),
            _ => Err(invalid_attribute(element, attribute, value)),
        })
        .transpose()
}

fn invalid_attribute(element: &BytesStart<'_>, attribute: &[u8], value: String) -> TransformError {
    TransformError::InvalidAttribute {
        element: element_local_name(element),
        attribute: String::from_utf8_lossy(attribute).into_owned(),
        value,
    }
}

fn element_local_name(element: &BytesStart<'_>) -> String {
    String::from_utf8_lossy(local_name(element.name().as_ref())).into_owned()
}

fn write_point<W: Write>(writer: &mut Writer<W>, tag: &str, point: CT_Point2D) -> Result<()> {
    let x = point.x.0.to_string();
    let y = point.y.0.to_string();
    let mut element = BytesStart::new(tag);
    element.push_attribute(("x", x.as_str()));
    element.push_attribute(("y", y.as_str()));
    writer
        .write_event(Event::Empty(element))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn write_size<W: Write>(writer: &mut Writer<W>, tag: &str, size: CT_PositiveSize2D) -> Result<()> {
    let cx = size.cx.0.to_string();
    let cy = size.cy.0.to_string();
    let mut element = BytesStart::new(tag);
    element.push_attribute(("cx", cx.as_str()));
    element.push_attribute(("cy", cy.as_str()));
    writer
        .write_event(Event::Empty(element))
        .map_err(OxmlError::from)?;
    Ok(())
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

fn is_explicit_empty_element(xml: &[u8]) -> bool {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    if !matches!(reader.read_event_into(&mut buffer), Ok(Event::Start(_))) {
        return false;
    }
    loop {
        buffer.clear();
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Text(text)) if is_xml_whitespace(text.as_ref()) => {}
            Ok(Event::CData(text)) if is_xml_whitespace(text.as_ref()) => {}
            Ok(Event::Comment(_) | Event::PI(_)) => {}
            Ok(Event::End(_)) => {
                buffer.clear();
                return matches!(reader.read_event_into(&mut buffer), Ok(Event::Eof));
            }
            _ => return false,
        }
    }
}

fn is_xml_whitespace(bytes: &[u8]) -> bool {
    bytes
        .iter()
        .all(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r'))
}

const fn affine_identity() -> [f64; 6] {
    [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
}

const fn affine_translation(x: f64, y: f64) -> [f64; 6] {
    [1.0, 0.0, 0.0, 1.0, x, y]
}

const fn affine_scale(x: f64, y: f64) -> [f64; 6] {
    [x, 0.0, 0.0, y, 0.0, 0.0]
}

fn affine_rotation_about(degrees: f64, centre_x: f64, centre_y: f64) -> [f64; 6] {
    let (sin, cos) = degrees.to_radians().sin_cos();
    [
        cos,
        sin,
        -sin,
        cos,
        centre_x - cos * centre_x + sin * centre_y,
        centre_y - sin * centre_x - cos * centre_y,
    ]
}

const fn affine_scale_about(x: f64, y: f64, centre_x: f64, centre_y: f64) -> [f64; 6] {
    [x, 0.0, 0.0, y, centre_x * (1.0 - x), centre_y * (1.0 - y)]
}

const fn affine_then(first: [f64; 6], next: [f64; 6]) -> [f64; 6] {
    [
        next[0] * first[0] + next[2] * first[1],
        next[1] * first[0] + next[3] * first[1],
        next[0] * first[2] + next[2] * first[3],
        next[1] * first[2] + next[3] * first[3],
        next[0] * first[4] + next[2] * first[5] + next[4],
        next[1] * first[4] + next[3] * first[5] + next[5],
    ]
}

#[cfg(test)]
mod tests {
    use super::{CT_Transform2D, TransformError};

    const EPSILON: f64 = 1.0e-10;

    #[test]
    fn nested_group_transform_composes_to_the_hand_computed_matrix() {
        let transform = CT_Transform2D::from_xml(
            br#"<a:xfrm rot="5400000" flipH="1" flipV="1"><a:off x="100" y="200"/><a:ext cx="400" cy="200"/><a:chOff x="10" y="20"/><a:chExt cx="200" cy="100"/></a:xfrm>"#,
        )
        .unwrap();

        let actual = transform.matrix().unwrap();
        let expected = [0.0, -2.0, 2.0, 0.0, 160.0, 520.0];
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!((actual - expected).abs() < EPSILON);
        }
    }

    #[test]
    fn transform_reads_any_prefix_and_writes_fixed_a_prefix_in_schema_order() {
        let transform = CT_Transform2D::from_xml(
            br#"<p:xfrm rot="-2700000" flipH="true" flipV="1"><p:chExt cx="70" cy="80"/><p:off x="-10" y="20"/><p:chOff x="50" y="60"/><p:ext cx="30" cy="40"/></p:xfrm>"#,
        )
        .unwrap();

        assert_eq!(
            transform.to_xml().unwrap(),
            br#"<a:xfrm rot="-2700000" flipH="1" flipV="1"><a:off x="-10" y="20"/><a:ext cx="30" cy="40"/><a:chOff x="50" y="60"/><a:chExt cx="70" cy="80"/></a:xfrm>"#
        );
    }

    #[test]
    fn unknown_transform_children_round_trip_at_their_original_boundaries() {
        let transform = CT_Transform2D::from_xml(
            br#"<z:xfrm><x:before x:id="1"/><z:off x="1" y="2"/><x:middle>one &amp; two</x:middle><z:ext cx="3" cy="4"/><x:after><!--kept--></x:after></z:xfrm>"#,
        )
        .unwrap();

        assert_eq!(
            transform.to_xml().unwrap(),
            br#"<a:xfrm><x:before x:id="1"/><a:off x="1" y="2"/><x:middle>one &amp; two</x:middle><a:ext cx="3" cy="4"/><x:after><!--kept--></x:after></a:xfrm>"#
        );
    }

    #[test]
    fn zero_child_extent_returns_a_transform_error_instead_of_non_finite_coefficients() {
        let transform = CT_Transform2D::from_xml(
            br#"<a:xfrm><a:off x="1" y="2"/><a:ext cx="3" cy="4"/><a:chExt cx="0" cy="5"/></a:xfrm>"#,
        )
        .unwrap();

        assert!(matches!(
            transform.matrix(),
            Err(TransformError::ZeroChildExtent { axis: "x" })
        ));
    }
}
