use std::fmt;
use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_core::units::{Angle, Percent1000};
use oxml_core::xml::{get_attr, local_name, matches_local_name};
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::color::{ColorChoice, ColorError};
use crate::order::OrderedRawChildren;

const MAX_POSITIVE_COORDINATE: i64 = 27_273_042_316_900;
const FULL_CIRCLE_ANGLE: i32 = 21_600_000;
const QUARTER_CIRCLE_ANGLE: i32 = 5_400_000;

/// Errors produced while parsing or writing DrawingML effects.
#[derive(Debug)]
pub enum EffectError {
    Xml(OxmlError),
    Color(ColorError),
    UnexpectedElement(String),
    MissingColor,
    InvalidAttribute {
        element: String,
        attribute: String,
        value: String,
    },
}

impl fmt::Display for EffectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml(error) => error.fmt(formatter),
            Self::Color(error) => error.fmt(formatter),
            Self::UnexpectedElement(element) => {
                write!(formatter, "unexpected DrawingML effect element: {element}")
            }
            Self::MissingColor => write!(formatter, "DrawingML outerShdw requires a colour child"),
            Self::InvalidAttribute {
                element,
                attribute,
                value,
            } => write!(
                formatter,
                "DrawingML {element} has invalid @{attribute}: {value}"
            ),
        }
    }
}

impl std::error::Error for EffectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml(error) => Some(error),
            Self::Color(error) => Some(error),
            _ => None,
        }
    }
}

impl From<OxmlError> for EffectError {
    fn from(error: OxmlError) -> Self {
        Self::Xml(error)
    }
}

impl From<ColorError> for EffectError {
    fn from(error: ColorError) -> Self {
        Self::Color(error)
    }
}

pub type Result<T> = std::result::Result<T, EffectError>;

/// DrawingML rectangle alignment used as the origin of a shadow transform.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RectAlignment {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl RectAlignment {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "tl" => Some(Self::TopLeft),
            "t" => Some(Self::Top),
            "tr" => Some(Self::TopRight),
            "l" => Some(Self::Left),
            "ctr" => Some(Self::Center),
            "r" => Some(Self::Right),
            "bl" => Some(Self::BottomLeft),
            "b" => Some(Self::Bottom),
            "br" => Some(Self::BottomRight),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::TopLeft => "tl",
            Self::Top => "t",
            Self::TopRight => "tr",
            Self::Left => "l",
            Self::Center => "ctr",
            Self::Right => "r",
            Self::BottomLeft => "bl",
            Self::Bottom => "b",
            Self::BottomRight => "br",
        }
    }
}

/// Modelled properties of one DrawingML `a:outerShdw` effect.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CT_OuterShadowEffect {
    pub blur_radius: Option<i64>,
    pub distance: Option<i64>,
    pub direction: Option<Angle>,
    pub scale_x: Option<Percent1000>,
    pub scale_y: Option<Percent1000>,
    pub skew_x: Option<Angle>,
    pub skew_y: Option<Angle>,
    pub alignment: Option<RectAlignment>,
    pub rotate_with_shape: Option<bool>,
    pub color: Option<ColorChoice>,
    raw_children: OrderedRawChildren,
}

impl CT_OuterShadowEffect {
    fn from_start(start: &BytesStart<'_>) -> Result<Self> {
        let shadow = Self {
            blur_radius: optional_parse(start, b"blurRad")?,
            distance: optional_parse(start, b"dist")?,
            direction: optional_parse::<i32>(start, b"dir")?.map(Angle),
            scale_x: optional_parse::<i32>(start, b"sx")?.map(Percent1000),
            scale_y: optional_parse::<i32>(start, b"sy")?.map(Percent1000),
            skew_x: optional_parse::<i32>(start, b"kx")?.map(Angle),
            skew_y: optional_parse::<i32>(start, b"ky")?.map(Angle),
            alignment: optional_enum(start, b"algn", RectAlignment::parse)?,
            rotate_with_shape: optional_bool(start, b"rotWithShape")?,
            color: None,
            raw_children: OrderedRawChildren::default(),
        };
        shadow.validate_attributes()?;
        Ok(shadow)
    }

    fn from_element(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut shadow = Self::from_start(start)?;
        let mut boundary = 0;
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element)
                    if is_color(element.name().as_ref()) && shadow.color.is_none() =>
                {
                    shadow.color = Some(ColorChoice::from_xml(reader, &element)?);
                    boundary = 1;
                }
                Event::Empty(element)
                    if is_color(element.name().as_ref()) && shadow.color.is_none() =>
                {
                    shadow.color = Some(ColorChoice::from_empty_xml(&element)?);
                    boundary = 1;
                }
                Event::Start(element) => shadow
                    .raw_children
                    .push(boundary, capture_element(reader, &element)?),
                Event::Empty(element) => shadow
                    .raw_children
                    .push(boundary, capture_empty_element(&element)?),
                Event::End(element)
                    if matches_local_name(element.name().as_ref(), b"outerShdw") =>
                {
                    break;
                }
                Event::Eof => return Err(missing_end("outerShdw")),
                _ => {}
            }
            buffer.clear();
        }
        if shadow.color.is_none() && shadow.raw_children.is_empty() {
            return Err(EffectError::MissingColor);
        }
        Ok(shadow)
    }

    fn validate_attributes(&self) -> Result<()> {
        if let Some(value) = self.blur_radius {
            validate_coordinate("blurRad", value)?;
        }
        if let Some(value) = self.distance {
            validate_coordinate("dist", value)?;
        }
        if let Some(value) = self.direction {
            validate_direction(value.0)?;
        }
        if let Some(value) = self.skew_x {
            validate_skew("kx", value.0)?;
        }
        if let Some(value) = self.skew_y {
            validate_skew("ky", value.0)?;
        }
        Ok(())
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        self.validate_attributes()?;
        if self.color.is_none() && self.raw_children.is_empty() {
            return Err(EffectError::MissingColor);
        }
        let mut start = BytesStart::new("a:outerShdw");
        let blur_radius = self.blur_radius.map(|value| value.to_string());
        let distance = self.distance.map(|value| value.to_string());
        let direction = self.direction.map(|value| value.0.to_string());
        let scale_x = self.scale_x.map(|value| value.0.to_string());
        let scale_y = self.scale_y.map(|value| value.0.to_string());
        let skew_x = self.skew_x.map(|value| value.0.to_string());
        let skew_y = self.skew_y.map(|value| value.0.to_string());
        for (name, value) in [
            ("blurRad", blur_radius.as_deref()),
            ("dist", distance.as_deref()),
            ("dir", direction.as_deref()),
            ("sx", scale_x.as_deref()),
            ("sy", scale_y.as_deref()),
            ("kx", skew_x.as_deref()),
            ("ky", skew_y.as_deref()),
        ] {
            if let Some(value) = value {
                start.push_attribute((name, value));
            }
        }
        if let Some(alignment) = self.alignment {
            start.push_attribute(("algn", alignment.as_str()));
        }
        if let Some(rotate) = self.rotate_with_shape.map(bool_text) {
            start.push_attribute(("rotWithShape", rotate));
        }

        write_start(writer, start)?;
        emit_raw(writer, self.raw_children.at(0))?;
        if let Some(color) = &self.color {
            color.to_xml(writer)?;
        }
        emit_raw(writer, self.raw_children.at(1))?;
        write_end(writer, "a:outerShdw")
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }
}

/// DrawingML `a:effectLst` with an outer shadow and preserved other effects.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CT_EffectList {
    pub outer_shadow: Option<CT_OuterShadowEffect>,
    raw_children: OrderedRawChildren,
}

impl CT_EffectList {
    /// Parses one complete `a:effectLst` with any namespace prefix.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element)
                    if matches_local_name(element.name().as_ref(), b"effectLst") =>
                {
                    return Self::from_element(&mut reader);
                }
                Event::Empty(element)
                    if matches_local_name(element.name().as_ref(), b"effectLst") =>
                {
                    return Ok(Self::default());
                }
                Event::Start(element) | Event::Empty(element) => {
                    return Err(unexpected(&element));
                }
                Event::Eof => {
                    return Err(EffectError::Xml(OxmlError::MissingElement(
                        "DrawingML effect list".to_owned(),
                    )));
                }
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_element(reader: &mut Reader<&[u8]>) -> Result<Self> {
        let mut effects = Self::default();
        let mut boundary = 0;
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element)
                    if matches_local_name(element.name().as_ref(), b"outerShdw")
                        && effects.outer_shadow.is_none() =>
                {
                    effects.outer_shadow =
                        Some(CT_OuterShadowEffect::from_element(reader, &element)?);
                    boundary = 1;
                }
                Event::Empty(element)
                    if matches_local_name(element.name().as_ref(), b"outerShdw")
                        && effects.outer_shadow.is_none() =>
                {
                    return Err(EffectError::MissingColor);
                }
                Event::Start(element) => effects
                    .raw_children
                    .push(boundary, capture_element(reader, &element)?),
                Event::Empty(element) => effects
                    .raw_children
                    .push(boundary, capture_empty_element(&element)?),
                Event::End(element)
                    if matches_local_name(element.name().as_ref(), b"effectLst") =>
                {
                    break;
                }
                Event::Eof => return Err(missing_end("effectLst")),
                _ => {}
            }
            buffer.clear();
        }
        Ok(effects)
    }

    /// Writes this list with a fixed root prefix and the shadow in schema order.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        self.write_xml(&mut writer)?;
        Ok(writer.into_inner())
    }

    /// Writes this effect list into an existing XML writer.
    pub fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        if self.outer_shadow.is_none() && self.raw_children.is_empty() {
            return write_empty(writer, BytesStart::new("a:effectLst"));
        }
        write_start(writer, BytesStart::new("a:effectLst"))?;
        emit_raw(writer, self.raw_children.at(0))?;
        if let Some(shadow) = &self.outer_shadow {
            shadow.write_xml(writer)?;
        }
        emit_raw(writer, self.raw_children.at(1))?;
        write_end(writer, "a:effectLst")
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }

    /// Reports an unresolved placeholder colour inside an opaque effect child.
    pub fn has_unmodelled_placeholder_color(&self) -> bool {
        self.raw_children
            .at(0)
            .chain(self.raw_children.at(1))
            .any(raw_contains_placeholder_color)
            || self.outer_shadow.as_ref().is_some_and(|shadow| {
                shadow
                    .raw_children()
                    .at(0)
                    .chain(shadow.raw_children().at(1))
                    .any(raw_contains_placeholder_color)
            })
    }
}

pub(crate) fn raw_contains_placeholder_color(xml: &[u8]) -> bool {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element) | Event::Empty(element))
                if matches_local_name(element.name().as_ref(), b"schemeClr")
                    && get_attr(&element, b"val").as_deref() == Some("phClr") =>
            {
                return true;
            }
            Ok(Event::Eof) | Err(_) => return false,
            _ => {}
        }
        buffer.clear();
    }
}

pub(crate) fn raw_is_effect_dag(xml: &[u8]) -> bool {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element) | Event::Empty(element)) => {
                return matches_local_name(element.name().as_ref(), b"effectDag");
            }
            Ok(Event::Eof) | Err(_) => return false,
            _ => {}
        }
        buffer.clear();
    }
}

fn is_color(name: &[u8]) -> bool {
    matches!(
        local_name(name),
        b"srgbClr" | b"schemeClr" | b"sysClr" | b"prstClr"
    )
}

fn validate_coordinate(attribute: &str, value: i64) -> Result<()> {
    if (0..=MAX_POSITIVE_COORDINATE).contains(&value) {
        Ok(())
    } else {
        Err(invalid_value(attribute, value))
    }
}

fn validate_direction(value: i32) -> Result<()> {
    if (0..FULL_CIRCLE_ANGLE).contains(&value) {
        Ok(())
    } else {
        Err(invalid_value("dir", value))
    }
}

fn validate_skew(attribute: &str, value: i32) -> Result<()> {
    if (-QUARTER_CIRCLE_ANGLE..QUARTER_CIRCLE_ANGLE).contains(&value) {
        Ok(())
    } else {
        Err(invalid_value(attribute, value))
    }
}

fn invalid_value(attribute: &str, value: impl ToString) -> EffectError {
    EffectError::InvalidAttribute {
        element: "outerShdw".to_owned(),
        attribute: attribute.to_owned(),
        value: value.to_string(),
    }
}

fn optional_parse<T: std::str::FromStr>(start: &BytesStart<'_>, name: &[u8]) -> Result<Option<T>> {
    get_attr(start, name)
        .map(|value| {
            value
                .parse()
                .map_err(|_| invalid(start, name, value.to_owned()))
        })
        .transpose()
}

fn optional_bool(start: &BytesStart<'_>, name: &[u8]) -> Result<Option<bool>> {
    get_attr(start, name)
        .map(|value| match value.as_str() {
            "1" | "true" => Ok(true),
            "0" | "false" => Ok(false),
            _ => Err(invalid(start, name, value)),
        })
        .transpose()
}

fn optional_enum<T>(
    start: &BytesStart<'_>,
    name: &[u8],
    parse: impl FnOnce(&str) -> Option<T>,
) -> Result<Option<T>> {
    get_attr(start, name)
        .map(|value| parse(&value).ok_or_else(|| invalid(start, name, value)))
        .transpose()
}

fn invalid(start: &BytesStart<'_>, attribute: &[u8], value: String) -> EffectError {
    EffectError::InvalidAttribute {
        element: String::from_utf8_lossy(local_name(start.name().as_ref())).into_owned(),
        attribute: String::from_utf8_lossy(attribute).into_owned(),
        value,
    }
}

fn unexpected(start: &BytesStart<'_>) -> EffectError {
    EffectError::UnexpectedElement(String::from_utf8_lossy(start.name().as_ref()).into_owned())
}

fn missing_end(name: &str) -> EffectError {
    EffectError::Xml(OxmlError::MissingElement(format!("closing a:{name}")))
}

const fn bool_text(value: bool) -> &'static str {
    if value { "1" } else { "0" }
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

fn write_start<W: Write>(writer: &mut Writer<W>, start: BytesStart<'_>) -> Result<()> {
    writer
        .write_event(Event::Start(start))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn write_empty<W: Write>(writer: &mut Writer<W>, start: BytesStart<'_>) -> Result<()> {
    writer
        .write_event(Event::Empty(start))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn write_end<W: Write>(writer: &mut Writer<W>, name: &str) -> Result<()> {
    writer
        .write_event(Event::End(BytesEnd::new(name)))
        .map_err(OxmlError::from)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use oxml_core::units::{Angle, Percent1000};

    use super::{CT_EffectList, EffectError, RectAlignment};
    use crate::color::ColorChoice;

    #[test]
    fn a_shape_with_glow_round_trips_with_glow_intact_as_raw_xml() {
        let xml = br#"<z:effectLst><a:glow rad="63500"><a:srgbClr val="FF0000"><a:alpha val="50000"/></a:srgbClr><!--kept--></a:glow></z:effectLst>"#;

        let written = CT_EffectList::from_xml(xml).unwrap().to_xml().unwrap();

        assert_eq!(written, br#"<a:effectLst><a:glow rad="63500"><a:srgbClr val="FF0000"><a:alpha val="50000"/></a:srgbClr><!--kept--></a:glow></a:effectLst>"#);
    }

    #[test]
    fn outer_shadow_properties_and_colour_round_trip_structurally() {
        let xml = br#"<z:effectLst><z:outerShdw blurRad="50800" dist="38100" dir="2700000" sx="120000" sy="80000" kx="1200000" ky="-600000" algn="br" rotWithShape="1"><z:schemeClr val="accent2"><z:alpha val="60000"/></z:schemeClr></z:outerShdw></z:effectLst>"#;
        let parsed = CT_EffectList::from_xml(xml).unwrap();
        let shadow = parsed.outer_shadow.as_ref().unwrap();

        assert_eq!(shadow.blur_radius, Some(50_800));
        assert_eq!(shadow.distance, Some(38_100));
        assert_eq!(shadow.direction, Some(Angle(2_700_000)));
        assert_eq!(shadow.scale_x, Some(Percent1000(120_000)));
        assert_eq!(shadow.scale_y, Some(Percent1000(80_000)));
        assert_eq!(shadow.skew_x, Some(Angle(1_200_000)));
        assert_eq!(shadow.skew_y, Some(Angle(-600_000)));
        assert_eq!(shadow.alignment, Some(RectAlignment::BottomRight));
        assert_eq!(shadow.rotate_with_shape, Some(true));
        assert!(matches!(shadow.color, Some(ColorChoice::Scheme { .. })));

        let written = parsed.to_xml().unwrap();
        assert_eq!(CT_EffectList::from_xml(&written).unwrap(), parsed);
    }

    #[test]
    fn effect_list_writes_schema_order_and_keeps_raw_effect_positions() {
        let xml = br#"<z:effectLst><x:before/><z:blur rad="10"/><z:glow rad="20"><x:item>one &amp; two</x:item></z:glow><z:innerShdw><z:srgbClr val="010203"/></z:innerShdw><z:outerShdw dist="30"><x:shadowBefore/><z:srgbClr val="AABBCC"/><x:shadowAfter/></z:outerShdw><z:prstShdw prst="shdw1"><z:srgbClr val="040506"/></z:prstShdw><z:reflection blurRad="40"/><x:after/></z:effectLst>"#;

        let written = CT_EffectList::from_xml(xml).unwrap().to_xml().unwrap();

        assert_eq!(written, br#"<a:effectLst><x:before/><z:blur rad="10"/><z:glow rad="20"><x:item>one &amp; two</x:item></z:glow><z:innerShdw><z:srgbClr val="010203"/></z:innerShdw><a:outerShdw dist="30"><x:shadowBefore/><a:srgbClr val="AABBCC"/><x:shadowAfter/></a:outerShdw><z:prstShdw prst="shdw1"><z:srgbClr val="040506"/></z:prstShdw><z:reflection blurRad="40"/><x:after/></a:effectLst>"#);
    }

    #[test]
    fn malformed_outer_shadow_values_return_errors_without_panicking() {
        let cases: &[&[u8]] = &[
            br#"<a:effectLst><a:outerShdw blurRad="-1"><a:srgbClr val="000000"/></a:outerShdw></a:effectLst>"#,
            br#"<a:effectLst><a:outerShdw dist="27273042316901"><a:srgbClr val="000000"/></a:outerShdw></a:effectLst>"#,
            br#"<a:effectLst><a:outerShdw dir="21600000"><a:srgbClr val="000000"/></a:outerShdw></a:effectLst>"#,
            br#"<a:effectLst><a:outerShdw kx="5400000"><a:srgbClr val="000000"/></a:outerShdw></a:effectLst>"#,
            br#"<a:effectLst><a:outerShdw sx="wide"><a:srgbClr val="000000"/></a:outerShdw></a:effectLst>"#,
            br#"<a:effectLst><a:outerShdw algn="middle"><a:srgbClr val="000000"/></a:outerShdw></a:effectLst>"#,
            br#"<a:effectLst><a:outerShdw rotWithShape="maybe"><a:srgbClr val="000000"/></a:outerShdw></a:effectLst>"#,
            br#"<a:effectLst><a:outerShdw><a:srgbClr val="XYZXYZ"/></a:outerShdw></a:effectLst>"#,
            br#"<a:effectLst><a:outerShdw/></a:effectLst>"#,
        ];
        for xml in cases {
            let result = std::panic::catch_unwind(|| CT_EffectList::from_xml(xml));
            assert!(
                result.is_ok(),
                "effect parser panicked for {}",
                String::from_utf8_lossy(xml)
            );
            assert!(
                result.unwrap().is_err(),
                "malformed outer shadow parsed successfully"
            );
        }
        assert!(matches!(
            CT_EffectList::from_xml(cases[8]),
            Err(EffectError::MissingColor)
        ));
    }
}
