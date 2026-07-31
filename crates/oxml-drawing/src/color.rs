use std::fmt;
use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_core::units::{Angle, Percent1000};
use oxml_core::xml::{get_attr, local_name, matches_local_name};
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::order::OrderedRawChildren;

/// Errors produced while parsing or writing DrawingML colours.
#[derive(Debug)]
pub enum ColorError {
    Xml(OxmlError),
    InvalidRgb(String),
    UnresolvedColor(String),
    MissingAttribute { element: String, attribute: String },
    UnexpectedElement(String),
    InvalidTransformValue { element: String, value: String },
}

impl fmt::Display for ColorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml(error) => error.fmt(formatter),
            Self::InvalidRgb(value) => write!(
                formatter,
                "DrawingML RGB colour must be exactly six hexadecimal digits: {value}"
            ),
            Self::UnresolvedColor(value) => {
                write!(formatter, "no concrete colour is available for: {value}")
            }
            Self::MissingAttribute { element, attribute } => {
                write!(formatter, "DrawingML {element} requires @{attribute}")
            }
            Self::UnexpectedElement(element) => {
                write!(formatter, "unexpected DrawingML colour element: {element}")
            }
            Self::InvalidTransformValue { element, value } => {
                write!(formatter, "DrawingML {element} has invalid @val: {value}")
            }
        }
    }
}

impl std::error::Error for ColorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml(error) => Some(error),
            _ => None,
        }
    }
}

impl From<OxmlError> for ColorError {
    fn from(error: OxmlError) -> Self {
        Self::Xml(error)
    }
}

pub type Result<T> = std::result::Result<T, ColorError>;

/// A validated DrawingML sRGB colour.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RgbColor([u8; 3]);

impl RgbColor {
    /// Creates a colour from its red, green, and blue components.
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self([red, green, blue])
    }

    /// Parses an `RRGGBB` DrawingML colour value.
    pub fn parse(value: &str) -> Result<Self> {
        if value.len() != 6 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ColorError::InvalidRgb(value.to_owned()));
        }

        Ok(Self([
            parse_component(&value[0..2], value)?,
            parse_component(&value[2..4], value)?,
            parse_component(&value[4..6], value)?,
        ]))
    }

    /// Returns the red, green, and blue components.
    pub const fn components(self) -> [u8; 3] {
        self.0
    }
}

impl fmt::Display for RgbColor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:02X}{:02X}{:02X}",
            self.0[0], self.0[1], self.0[2]
        )
    }
}

/// One DrawingML colour transform.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorTransform {
    Tint(Percent1000),
    Shade(Percent1000),
    Complement,
    Inverse,
    Gray,
    Alpha(Percent1000),
    AlphaOffset(Percent1000),
    AlphaModulation(Percent1000),
    Hue(Angle),
    HueOffset(Angle),
    HueModulation(Percent1000),
    Saturation(Percent1000),
    SaturationOffset(Percent1000),
    SaturationModulation(Percent1000),
    Luminance(Percent1000),
    LuminanceOffset(Percent1000),
    LuminanceModulation(Percent1000),
    Red(Percent1000),
    RedOffset(Percent1000),
    RedModulation(Percent1000),
    Green(Percent1000),
    GreenOffset(Percent1000),
    GreenModulation(Percent1000),
    Blue(Percent1000),
    BlueOffset(Percent1000),
    BlueModulation(Percent1000),
    Gamma,
    InverseGamma,
}

impl ColorTransform {
    fn from_xml(element: &BytesStart<'_>) -> Result<Option<Self>> {
        let qualified_name = element.name();
        let name = local_name(qualified_name.as_ref());
        let transform = match name {
            b"tint" => Self::Tint(parse_percent(element)?),
            b"shade" => Self::Shade(parse_percent(element)?),
            b"comp" => Self::Complement,
            b"inv" => Self::Inverse,
            b"gray" => Self::Gray,
            b"alpha" => Self::Alpha(parse_percent(element)?),
            b"alphaOff" => Self::AlphaOffset(parse_percent(element)?),
            b"alphaMod" => Self::AlphaModulation(parse_percent(element)?),
            b"hue" => Self::Hue(parse_angle(element)?),
            b"hueOff" => Self::HueOffset(parse_angle(element)?),
            b"hueMod" => Self::HueModulation(parse_percent(element)?),
            b"sat" => Self::Saturation(parse_percent(element)?),
            b"satOff" => Self::SaturationOffset(parse_percent(element)?),
            b"satMod" => Self::SaturationModulation(parse_percent(element)?),
            b"lum" => Self::Luminance(parse_percent(element)?),
            b"lumOff" => Self::LuminanceOffset(parse_percent(element)?),
            b"lumMod" => Self::LuminanceModulation(parse_percent(element)?),
            b"red" => Self::Red(parse_percent(element)?),
            b"redOff" => Self::RedOffset(parse_percent(element)?),
            b"redMod" => Self::RedModulation(parse_percent(element)?),
            b"green" => Self::Green(parse_percent(element)?),
            b"greenOff" => Self::GreenOffset(parse_percent(element)?),
            b"greenMod" => Self::GreenModulation(parse_percent(element)?),
            b"blue" => Self::Blue(parse_percent(element)?),
            b"blueOff" => Self::BlueOffset(parse_percent(element)?),
            b"blueMod" => Self::BlueModulation(parse_percent(element)?),
            b"gamma" => Self::Gamma,
            b"invGamma" => Self::InverseGamma,
            _ => return Ok(None),
        };
        Ok(Some(transform))
    }

    fn to_xml<W: Write>(self, writer: &mut Writer<W>) -> Result<()> {
        let (name, value) = match self {
            Self::Tint(value) => ("a:tint", Some(value.0)),
            Self::Shade(value) => ("a:shade", Some(value.0)),
            Self::Complement => ("a:comp", None),
            Self::Inverse => ("a:inv", None),
            Self::Gray => ("a:gray", None),
            Self::Alpha(value) => ("a:alpha", Some(value.0)),
            Self::AlphaOffset(value) => ("a:alphaOff", Some(value.0)),
            Self::AlphaModulation(value) => ("a:alphaMod", Some(value.0)),
            Self::Hue(value) => ("a:hue", Some(value.0)),
            Self::HueOffset(value) => ("a:hueOff", Some(value.0)),
            Self::HueModulation(value) => ("a:hueMod", Some(value.0)),
            Self::Saturation(value) => ("a:sat", Some(value.0)),
            Self::SaturationOffset(value) => ("a:satOff", Some(value.0)),
            Self::SaturationModulation(value) => ("a:satMod", Some(value.0)),
            Self::Luminance(value) => ("a:lum", Some(value.0)),
            Self::LuminanceOffset(value) => ("a:lumOff", Some(value.0)),
            Self::LuminanceModulation(value) => ("a:lumMod", Some(value.0)),
            Self::Red(value) => ("a:red", Some(value.0)),
            Self::RedOffset(value) => ("a:redOff", Some(value.0)),
            Self::RedModulation(value) => ("a:redMod", Some(value.0)),
            Self::Green(value) => ("a:green", Some(value.0)),
            Self::GreenOffset(value) => ("a:greenOff", Some(value.0)),
            Self::GreenModulation(value) => ("a:greenMod", Some(value.0)),
            Self::Blue(value) => ("a:blue", Some(value.0)),
            Self::BlueOffset(value) => ("a:blueOff", Some(value.0)),
            Self::BlueModulation(value) => ("a:blueMod", Some(value.0)),
            Self::Gamma => ("a:gamma", None),
            Self::InverseGamma => ("a:invGamma", None),
        };
        let mut element = BytesStart::new(name);
        let value_string = value.map(|raw| raw.to_string());
        if let Some(value) = value_string.as_deref() {
            element.push_attribute(("val", value));
        }
        writer
            .write_event(Event::Empty(element))
            .map_err(OxmlError::from)?;
        Ok(())
    }
}

/// A concrete colour after its DrawingML transform stack is applied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl ResolvedColor {
    pub const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub const fn rgba(self) -> [u8; 4] {
        [self.red, self.green, self.blue, self.alpha]
    }
}

/// One of the twelve semantic slots selected by a DrawingML colour map.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ColorMapSlot {
    Background1,
    Text1,
    Background2,
    Text2,
    Accent1,
    Accent2,
    Accent3,
    Accent4,
    Accent5,
    Accent6,
    Hyperlink,
    FollowedHyperlink,
}

impl ColorMapSlot {
    fn from_value(value: &str) -> Option<Self> {
        match value {
            "bg1" => Some(Self::Background1),
            "tx1" => Some(Self::Text1),
            "bg2" => Some(Self::Background2),
            "tx2" => Some(Self::Text2),
            "accent1" => Some(Self::Accent1),
            "accent2" => Some(Self::Accent2),
            "accent3" => Some(Self::Accent3),
            "accent4" => Some(Self::Accent4),
            "accent5" => Some(Self::Accent5),
            "accent6" => Some(Self::Accent6),
            "hlink" => Some(Self::Hyperlink),
            "folHlink" => Some(Self::FollowedHyperlink),
            _ => None,
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Background1 => 0,
            Self::Text1 => 1,
            Self::Background2 => 2,
            Self::Text2 => 3,
            Self::Accent1 => 4,
            Self::Accent2 => 5,
            Self::Accent3 => 6,
            Self::Accent4 => 7,
            Self::Accent5 => 8,
            Self::Accent6 => 9,
            Self::Hyperlink => 10,
            Self::FollowedHyperlink => 11,
        }
    }
}

/// One of the twelve concrete slots in a DrawingML theme colour scheme.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ThemeColorSlot {
    Dark1,
    Light1,
    Dark2,
    Light2,
    Accent1,
    Accent2,
    Accent3,
    Accent4,
    Accent5,
    Accent6,
    Hyperlink,
    FollowedHyperlink,
}

impl ThemeColorSlot {
    /// Returns the DrawingML theme slot name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dark1 => "dk1",
            Self::Light1 => "lt1",
            Self::Dark2 => "dk2",
            Self::Light2 => "lt2",
            Self::Accent1 => "accent1",
            Self::Accent2 => "accent2",
            Self::Accent3 => "accent3",
            Self::Accent4 => "accent4",
            Self::Accent5 => "accent5",
            Self::Accent6 => "accent6",
            Self::Hyperlink => "hlink",
            Self::FollowedHyperlink => "folHlink",
        }
    }
}

/// The twelve master-controlled mappings applied before theme colour lookup.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColorMap {
    slots: [ThemeColorSlot; 12],
}

impl ColorMap {
    /// Creates a colour map from parsed master values in schema attribute order.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        background1: ThemeColorSlot,
        text1: ThemeColorSlot,
        background2: ThemeColorSlot,
        text2: ThemeColorSlot,
        accent1: ThemeColorSlot,
        accent2: ThemeColorSlot,
        accent3: ThemeColorSlot,
        accent4: ThemeColorSlot,
        accent5: ThemeColorSlot,
        accent6: ThemeColorSlot,
        hyperlink: ThemeColorSlot,
        followed_hyperlink: ThemeColorSlot,
    ) -> Self {
        Self {
            slots: [
                background1,
                text1,
                background2,
                text2,
                accent1,
                accent2,
                accent3,
                accent4,
                accent5,
                accent6,
                hyperlink,
                followed_hyperlink,
            ],
        }
    }

    /// Returns the concrete theme slot selected for one semantic map slot.
    pub const fn theme_slot(&self, slot: ColorMapSlot) -> ThemeColorSlot {
        self.slots[slot.index()]
    }

    /// Returns a copy with only the named layout or slide overrides replaced.
    pub fn with_overrides(&self, overrides: &[(ColorMapSlot, ThemeColorSlot)]) -> Self {
        let mut resolved = self.clone();
        for (source, destination) in overrides {
            resolved.slots[source.index()] = *destination;
        }
        resolved
    }

    fn mapped_name<'a>(&self, value: &'a str) -> &'a str {
        ColorMapSlot::from_value(value)
            .map(|slot| self.theme_slot(slot).as_str())
            .unwrap_or(value)
    }
}

impl Default for ColorMap {
    fn default() -> Self {
        Self::new(
            ThemeColorSlot::Light1,
            ThemeColorSlot::Dark1,
            ThemeColorSlot::Light2,
            ThemeColorSlot::Dark2,
            ThemeColorSlot::Accent1,
            ThemeColorSlot::Accent2,
            ThemeColorSlot::Accent3,
            ThemeColorSlot::Accent4,
            ThemeColorSlot::Accent5,
            ThemeColorSlot::Accent6,
            ThemeColorSlot::Hyperlink,
            ThemeColorSlot::FollowedHyperlink,
        )
    }
}

/// One of the four DrawingML colour choice elements.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ColorChoice {
    Srgb {
        value: RgbColor,
        transforms: Vec<ColorTransform>,
        raw_children: OrderedRawChildren,
    },
    Scheme {
        value: String,
        transforms: Vec<ColorTransform>,
        raw_children: OrderedRawChildren,
    },
    System {
        value: String,
        last_color: Option<RgbColor>,
        transforms: Vec<ColorTransform>,
        raw_children: OrderedRawChildren,
    },
    Preset {
        value: String,
        transforms: Vec<ColorTransform>,
        raw_children: OrderedRawChildren,
    },
}

impl ColorChoice {
    /// Parses a colour after the caller has consumed its start event.
    pub fn from_xml(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let qualified_name = start.name();
        let element_name = local_name(qualified_name.as_ref());
        let (transforms, raw_children) = capture_children(reader, element_name)?;
        Self::from_parts(start, transforms, raw_children)
    }

    /// Parses a colour from a self-closing element.
    pub fn from_empty_xml(start: &BytesStart<'_>) -> Result<Self> {
        Self::from_parts(start, Vec::new(), OrderedRawChildren::default())
    }

    fn from_parts(
        start: &BytesStart<'_>,
        transforms: Vec<ColorTransform>,
        raw_children: OrderedRawChildren,
    ) -> Result<Self> {
        let qualified_name = start.name();
        let element_name = local_name(qualified_name.as_ref());
        let value = required_attr(start, b"val")?;
        match element_name {
            b"srgbClr" => Ok(Self::Srgb {
                value: RgbColor::parse(&value)?,
                transforms,
                raw_children,
            }),
            b"schemeClr" => Ok(Self::Scheme {
                value,
                transforms,
                raw_children,
            }),
            b"sysClr" => Ok(Self::System {
                value,
                last_color: get_attr(start, b"lastClr")
                    .map(|last| RgbColor::parse(&last))
                    .transpose()?,
                transforms,
                raw_children,
            }),
            b"prstClr" => Ok(Self::Preset {
                value,
                transforms,
                raw_children,
            }),
            _ => Err(ColorError::UnexpectedElement(
                String::from_utf8_lossy(start.name().as_ref()).into_owned(),
            )),
        }
    }

    /// Writes this nested colour element with the canonical `a:` prefix.
    pub fn to_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let (tag, value, last_color, transforms, raw_children) = match self {
            Self::Srgb {
                value,
                transforms,
                raw_children,
            } => (
                "a:srgbClr",
                value.to_string(),
                None,
                transforms,
                raw_children,
            ),
            Self::Scheme {
                value,
                transforms,
                raw_children,
            } => ("a:schemeClr", value.clone(), None, transforms, raw_children),
            Self::System {
                value,
                last_color,
                transforms,
                raw_children,
            } => (
                "a:sysClr",
                value.clone(),
                last_color.map(|colour| colour.to_string()),
                transforms,
                raw_children,
            ),
            Self::Preset {
                value,
                transforms,
                raw_children,
            } => ("a:prstClr", value.clone(), None, transforms, raw_children),
        };

        let mut start = BytesStart::new(tag);
        start.push_attribute(("val", value.as_str()));
        if let Some(last_color) = last_color.as_deref() {
            start.push_attribute(("lastClr", last_color));
        }

        if raw_children.is_empty() && transforms.is_empty() {
            writer
                .write_event(Event::Empty(start))
                .map_err(OxmlError::from)?;
            return Ok(());
        }

        writer
            .write_event(Event::Start(start))
            .map_err(OxmlError::from)?;
        for boundary in 0..=transforms.len() {
            for raw in raw_children.at(boundary) {
                writer.get_mut().write_all(raw).map_err(OxmlError::from)?;
            }
            if let Some(transform) = transforms.get(boundary) {
                transform.to_xml(writer)?;
            }
        }
        writer
            .write_event(Event::End(BytesEnd::new(tag)))
            .map_err(OxmlError::from)?;
        Ok(())
    }

    /// Returns raw, not-yet-modelled children in document order.
    pub fn raw_children(&self) -> &OrderedRawChildren {
        match self {
            Self::Srgb { raw_children, .. }
            | Self::Scheme { raw_children, .. }
            | Self::System { raw_children, .. }
            | Self::Preset { raw_children, .. } => raw_children,
        }
    }

    /// Returns modelled transforms in document order.
    pub fn transforms(&self) -> &[ColorTransform] {
        match self {
            Self::Srgb { transforms, .. }
            | Self::Scheme { transforms, .. }
            | Self::System { transforms, .. }
            | Self::Preset { transforms, .. } => transforms,
        }
    }
}

/// Resolves a colour through the master map, lookup table, and transform stack.
///
/// Theme slots, system colour names, and preset colour names share the concrete
/// lookup table. Only scheme colours pass through `color_map`. A system colour
/// falls back to its `lastClr` value when its name is absent from the lookup.
pub fn resolve_color(
    choice: &ColorChoice,
    color_map: &ColorMap,
    lookup: &[(&str, RgbColor)],
) -> Result<ResolvedColor> {
    let find = |name: &str| {
        lookup
            .iter()
            .find_map(|(candidate, colour)| (*candidate == name).then_some(*colour))
    };
    let (base, transforms) = match choice {
        ColorChoice::Srgb {
            value, transforms, ..
        } => (*value, transforms.as_slice()),
        ColorChoice::Scheme {
            value, transforms, ..
        } => {
            let mapped = color_map.mapped_name(value);
            (
                find(mapped).ok_or_else(|| ColorError::UnresolvedColor(mapped.to_owned()))?,
                transforms.as_slice(),
            )
        }
        ColorChoice::System {
            value,
            last_color,
            transforms,
            ..
        } => (
            find(value)
                .or(*last_color)
                .ok_or_else(|| ColorError::UnresolvedColor(value.clone()))?,
            transforms.as_slice(),
        ),
        ColorChoice::Preset {
            value, transforms, ..
        } => (
            find(value).ok_or_else(|| ColorError::UnresolvedColor(value.clone()))?,
            transforms.as_slice(),
        ),
    };
    Ok(apply_color_transforms(base, transforms))
}

fn required_attr(element: &BytesStart<'_>, name: &[u8]) -> Result<String> {
    get_attr(element, name).ok_or_else(|| ColorError::MissingAttribute {
        element: String::from_utf8_lossy(local_name(element.name().as_ref())).into_owned(),
        attribute: String::from_utf8_lossy(name).into_owned(),
    })
}

fn parse_component(component: &str, full_value: &str) -> Result<u8> {
    u8::from_str_radix(component, 16).map_err(|_| ColorError::InvalidRgb(full_value.to_owned()))
}

fn parse_percent(element: &BytesStart<'_>) -> Result<Percent1000> {
    parse_transform_i32(element).map(Percent1000)
}

fn parse_angle(element: &BytesStart<'_>) -> Result<Angle> {
    parse_transform_i32(element).map(Angle)
}

fn parse_transform_i32(element: &BytesStart<'_>) -> Result<i32> {
    let value = required_attr(element, b"val")?;
    value
        .parse()
        .map_err(|_| ColorError::InvalidTransformValue {
            element: String::from_utf8_lossy(local_name(element.name().as_ref())).into_owned(),
            value,
        })
}

fn capture_children(
    reader: &mut Reader<&[u8]>,
    end_name: &[u8],
) -> Result<(Vec<ColorTransform>, OrderedRawChildren)> {
    let mut transforms = Vec::new();
    let mut raw_children = OrderedRawChildren::default();
    let mut buffer = Vec::new();

    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) => {
                let transform = ColorTransform::from_xml(&element)?;
                let raw = capture_element(reader, &element)?;
                if let Some(transform) = transform.filter(|_| is_explicit_empty_element(&raw)) {
                    transforms.push(transform);
                } else {
                    raw_children.push(transforms.len(), raw);
                }
            }
            Event::Empty(element) => {
                if let Some(transform) = ColorTransform::from_xml(&element)? {
                    transforms.push(transform);
                } else {
                    raw_children.push(transforms.len(), capture_empty_element(&element)?);
                }
            }
            Event::End(element) if matches_local_name(element.name().as_ref(), end_name) => break,
            Event::Eof => {
                return Err(ColorError::Xml(OxmlError::MissingElement(format!(
                    "closing {} colour element",
                    String::from_utf8_lossy(end_name)
                ))));
            }
            _ => {}
        }
        buffer.clear();
    }

    Ok((transforms, raw_children))
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

/// Applies a DrawingML transform stack from left to right.
pub fn apply_color_transforms(colour: RgbColor, transforms: &[ColorTransform]) -> ResolvedColor {
    let [red, green, blue] = colour.components();
    let mut current = WorkingColor {
        red: red as f64 / 255.0,
        green: green as f64 / 255.0,
        blue: blue as f64 / 255.0,
        alpha: 1.0,
    };
    for transform in transforms {
        current.apply(*transform);
    }
    current.resolved()
}

/// Applies spec-correct linear-gamma tint and shade percentages.
pub fn apply_tint_shade_pct(
    hex: &str,
    tint: Option<Percent1000>,
    shade: Option<Percent1000>,
) -> String {
    let Ok(colour) = RgbColor::parse(hex) else {
        return hex.to_owned();
    };
    let mut transforms = Vec::with_capacity(2);
    if let Some(value) = tint {
        transforms.push(ColorTransform::Tint(value));
    }
    if let Some(value) = shade {
        transforms.push(ColorTransform::Shade(value));
    }
    let resolved = apply_color_transforms(colour, &transforms);
    RgbColor::new(resolved.red, resolved.green, resolved.blue).to_string()
}

/// Applies DrawingML HSL luminance modulation followed by offset.
pub fn apply_lum_mod_off(
    hex: &str,
    lum_mod: Option<Percent1000>,
    lum_off: Option<Percent1000>,
) -> String {
    let Ok(colour) = RgbColor::parse(hex) else {
        return hex.to_owned();
    };
    let mut transforms = Vec::with_capacity(2);
    if let Some(value) = lum_mod {
        transforms.push(ColorTransform::LuminanceModulation(value));
    }
    if let Some(value) = lum_off {
        transforms.push(ColorTransform::LuminanceOffset(value));
    }
    let resolved = apply_color_transforms(colour, &transforms);
    RgbColor::new(resolved.red, resolved.green, resolved.blue).to_string()
}

/// Converts one sRGB channel to linear light.
pub fn srgb_to_linear(channel: f64) -> f64 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

/// Converts one linear-light channel to sRGB.
pub fn linear_to_srgb(channel: f64) -> f64 {
    if channel <= 0.003_130_8 {
        channel * 12.92
    } else {
        1.055 * channel.powf(1.0 / 2.4) - 0.055
    }
}

#[derive(Clone, Copy)]
struct WorkingColor {
    red: f64,
    green: f64,
    blue: f64,
    alpha: f64,
}

impl WorkingColor {
    fn apply(&mut self, transform: ColorTransform) {
        match transform {
            ColorTransform::Tint(value) => {
                let amount = percent(value);
                self.map_linear(|channel| channel * amount + 1.0 - amount);
            }
            ColorTransform::Shade(value) => {
                let amount = percent(value);
                self.map_linear(|channel| channel * amount);
            }
            ColorTransform::Complement => {
                self.map_hsl(|hue, sat, lum| ((hue + 0.5).rem_euclid(1.0), sat, lum))
            }
            ColorTransform::Inverse => {
                self.map_linear(|channel| 1.0 - channel);
            }
            ColorTransform::Gray => {
                let luminance = 0.2126 * self.red + 0.7152 * self.green + 0.0722 * self.blue;
                self.red = luminance;
                self.green = luminance;
                self.blue = luminance;
            }
            ColorTransform::Alpha(value) => self.alpha = percent(value),
            ColorTransform::AlphaOffset(value) => self.alpha += percent(value),
            ColorTransform::AlphaModulation(value) => self.alpha *= percent(value),
            ColorTransform::Hue(value) => {
                let hue = angle_turns(value);
                self.map_hsl(|_, sat, lum| (hue, sat, lum));
            }
            ColorTransform::HueOffset(value) => {
                let offset = angle_turns(value);
                self.map_hsl(|hue, sat, lum| (hue + offset, sat, lum));
            }
            ColorTransform::HueModulation(value) => {
                let amount = percent(value);
                self.map_hsl(|hue, sat, lum| (hue * amount, sat, lum));
            }
            ColorTransform::Saturation(value) => {
                let value = percent(value);
                self.map_hsl(|hue, _, lum| (hue, value, lum));
            }
            ColorTransform::SaturationOffset(value) => {
                let offset = percent(value);
                self.map_hsl(|hue, sat, lum| (hue, sat + offset, lum));
            }
            ColorTransform::SaturationModulation(value) => {
                let amount = percent(value);
                self.map_hsl(|hue, sat, lum| (hue, sat * amount, lum));
            }
            ColorTransform::Luminance(value) => {
                let value = percent(value);
                self.map_hsl(|hue, sat, _| (hue, sat, value));
            }
            ColorTransform::LuminanceOffset(value) => {
                let offset = percent(value);
                self.map_hsl(|hue, sat, lum| (hue, sat, lum + offset));
            }
            ColorTransform::LuminanceModulation(value) => {
                let amount = percent(value);
                self.map_hsl(|hue, sat, lum| (hue, sat, lum * amount));
            }
            ColorTransform::Red(value) => self.red = linear_to_srgb(percent(value)),
            ColorTransform::RedOffset(value) => {
                self.red = linear_to_srgb(srgb_to_linear(self.red) + percent(value));
            }
            ColorTransform::RedModulation(value) => {
                self.red = linear_to_srgb(srgb_to_linear(self.red) * percent(value));
            }
            ColorTransform::Green(value) => self.green = linear_to_srgb(percent(value)),
            ColorTransform::GreenOffset(value) => {
                self.green = linear_to_srgb(srgb_to_linear(self.green) + percent(value));
            }
            ColorTransform::GreenModulation(value) => {
                self.green = linear_to_srgb(srgb_to_linear(self.green) * percent(value));
            }
            ColorTransform::Blue(value) => self.blue = linear_to_srgb(percent(value)),
            ColorTransform::BlueOffset(value) => {
                self.blue = linear_to_srgb(srgb_to_linear(self.blue) + percent(value));
            }
            ColorTransform::BlueModulation(value) => {
                self.blue = linear_to_srgb(srgb_to_linear(self.blue) * percent(value));
            }
            ColorTransform::Gamma => {
                self.red = linear_to_srgb(self.red);
                self.green = linear_to_srgb(self.green);
                self.blue = linear_to_srgb(self.blue);
            }
            ColorTransform::InverseGamma => {
                self.red = srgb_to_linear(self.red);
                self.green = srgb_to_linear(self.green);
                self.blue = srgb_to_linear(self.blue);
            }
        }
        self.clamp();
    }

    fn map_linear(&mut self, transform: impl Fn(f64) -> f64) {
        self.red = linear_to_srgb(transform(srgb_to_linear(self.red)));
        self.green = linear_to_srgb(transform(srgb_to_linear(self.green)));
        self.blue = linear_to_srgb(transform(srgb_to_linear(self.blue)));
    }

    fn map_hsl(&mut self, transform: impl Fn(f64, f64, f64) -> (f64, f64, f64)) {
        let (hue, sat, lum) = rgb_to_hsl(self.red, self.green, self.blue);
        let (hue, sat, lum) = transform(hue, sat, lum);
        (self.red, self.green, self.blue) = hsl_to_rgb(
            hue.rem_euclid(1.0),
            sat.clamp(0.0, 1.0),
            lum.clamp(0.0, 1.0),
        );
    }

    fn clamp(&mut self) {
        self.red = self.red.clamp(0.0, 1.0);
        self.green = self.green.clamp(0.0, 1.0);
        self.blue = self.blue.clamp(0.0, 1.0);
        self.alpha = self.alpha.clamp(0.0, 1.0);
    }

    fn resolved(self) -> ResolvedColor {
        let alpha = channel_byte(self.alpha);
        if alpha == 0 {
            return ResolvedColor::new(0, 0, 0, 0);
        }
        let red = alpha_quantized_channel(channel_byte(self.red), alpha);
        let green = alpha_quantized_channel(channel_byte(self.green), alpha);
        let blue = alpha_quantized_channel(channel_byte(self.blue), alpha);
        ResolvedColor::new(red, green, blue, alpha)
    }
}

fn percent(value: Percent1000) -> f64 {
    value.to_fraction()
}

fn angle_turns(value: Angle) -> f64 {
    value.to_degrees() / 360.0
}

fn channel_byte(channel: f64) -> u8 {
    (channel.clamp(0.0, 1.0) * 255.0 + 1e-9).round() as u8
}

fn alpha_quantized_channel(channel: u8, alpha: u8) -> u8 {
    if alpha == u8::MAX {
        return channel;
    }
    let channel = u16::from(channel);
    let alpha = u16::from(alpha);
    let premultiplied = (channel * alpha + 127) / 255;
    ((premultiplied * 255 + alpha / 2) / alpha) as u8
}

fn rgb_to_hsl(red: f64, green: f64, blue: f64) -> (f64, f64, f64) {
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let lum = (max + min) / 2.0;
    if (max - min).abs() <= f64::EPSILON {
        return (0.0, 0.0, lum);
    }

    let delta = max - min;
    let sat = if lum > 0.5 {
        delta / (2.0 - max - min)
    } else {
        delta / (max + min)
    };
    let hue = if (max - red).abs() <= f64::EPSILON {
        (green - blue) / delta + if green < blue { 6.0 } else { 0.0 }
    } else if (max - green).abs() <= f64::EPSILON {
        (blue - red) / delta + 2.0
    } else {
        (red - green) / delta + 4.0
    } / 6.0;
    (hue, sat, lum)
}

fn hsl_to_rgb(hue: f64, sat: f64, lum: f64) -> (f64, f64, f64) {
    if sat <= f64::EPSILON {
        return (lum, lum, lum);
    }
    let q = if lum < 0.5 {
        lum * (1.0 + sat)
    } else {
        lum + sat - lum * sat
    };
    let p = 2.0 * lum - q;
    (
        hue_to_rgb(p, q, hue + 1.0 / 3.0),
        hue_to_rgb(p, q, hue),
        hue_to_rgb(p, q, hue - 1.0 / 3.0),
    )
}

fn hue_to_rgb(p: f64, q: f64, hue: f64) -> f64 {
    let hue = hue.rem_euclid(1.0);
    if hue < 1.0 / 6.0 {
        p + (q - p) * 6.0 * hue
    } else if hue < 0.5 {
        q
    } else if hue < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - hue) * 6.0
    } else {
        p
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use oxml_core::units::{Angle, Percent1000};
    use oxml_opc::OpcPackage;
    use oxml_opc::relationship::rel_types;
    use quick_xml::events::Event;

    use super::{
        ColorChoice, ColorError, ColorMap, ColorMapSlot, ColorTransform, ResolvedColor, RgbColor,
        ThemeColorSlot, apply_color_transforms, linear_to_srgb, resolve_color, srgb_to_linear,
    };
    use crate::order::OrderedRawChildren;
    use quick_xml::{Reader, Writer};

    const POWERPOINT_ORACLE_VERSION: &str = "16.104";
    const POWERPOINT_ORACLE_BUILD: &str = "16.104.25121423";
    const POWERPOINT_ORACLE_APP_BUILD: &str = "1214";

    struct OracleCase {
        name: &'static str,
        input: RgbColor,
        transforms: &'static [ColorTransform],
        expected: [u8; 4],
    }

    const ORACLE_CASES: &[OracleCase] = &[
        oracle_case(
            "single_tint",
            0x1F497D,
            &[ColorTransform::Tint(Percent1000(62_000))],
            [167, 174, 189, 255],
        ),
        oracle_case(
            "single_shade",
            0xEEECE1,
            &[ColorTransform::Shade(Percent1000(58_000))],
            [187, 185, 176, 255],
        ),
        oracle_case(
            "single_comp",
            0x4F81BD,
            &[ColorTransform::Complement],
            [189, 139, 79, 255],
        ),
        oracle_case(
            "single_inv",
            0xC0504D,
            &[ColorTransform::Inverse],
            [183, 246, 246, 255],
        ),
        oracle_case(
            "single_gray",
            0x9BBB59,
            &[ColorTransform::Gray],
            [173, 173, 173, 255],
        ),
        oracle_case(
            "single_alpha",
            0x8064A2,
            &[ColorTransform::Alpha(Percent1000(47_000))],
            [128, 100, 162, 120],
        ),
        oracle_case(
            "single_alpha_off",
            0x4BACC6,
            &[ColorTransform::AlphaOffset(Percent1000(-30_000))],
            [76, 172, 198, 179],
        ),
        oracle_case(
            "single_alpha_mod",
            0xF79646,
            &[ColorTransform::AlphaModulation(Percent1000(43_000))],
            [248, 151, 70, 110],
        ),
        oracle_case(
            "single_hue",
            0x1F497D,
            &[ColorTransform::Hue(Angle(9_000_000))],
            [31, 125, 78, 255],
        ),
        oracle_case(
            "single_hue_off",
            0xEEECE1,
            &[ColorTransform::HueOffset(Angle(-3_000_000))],
            [238, 225, 225, 255],
        ),
        oracle_case(
            "single_hue_mod",
            0x4F81BD,
            &[ColorTransform::HueModulation(Percent1000(55_000))],
            [85, 189, 79, 255],
        ),
        oracle_case(
            "single_sat",
            0xC0504D,
            &[ColorTransform::Saturation(Percent1000(72_000))],
            [221, 52, 48, 255],
        ),
        oracle_case(
            "single_sat_off",
            0x9BBB59,
            &[ColorTransform::SaturationOffset(Percent1000(-25_000))],
            [145, 158, 118, 255],
        ),
        oracle_case(
            "single_sat_mod",
            0x8064A2,
            &[ColorTransform::SaturationModulation(Percent1000(45_000))],
            [130, 117, 145, 255],
        ),
        oracle_case(
            "single_lum",
            0x4BACC6,
            &[ColorTransform::Luminance(Percent1000(65_000))],
            [119, 192, 212, 255],
        ),
        oracle_case(
            "single_lum_off",
            0xF79646,
            &[ColorTransform::LuminanceOffset(Percent1000(20_000))],
            [251, 205, 168, 255],
        ),
        oracle_case(
            "single_lum_mod",
            0x1F497D,
            &[ColorTransform::LuminanceModulation(Percent1000(55_000))],
            [17, 40, 69, 255],
        ),
        oracle_case(
            "single_red",
            0xEEECE1,
            &[ColorTransform::Red(Percent1000(20_000))],
            [124, 236, 225, 255],
        ),
        oracle_case(
            "single_red_off",
            0x4F81BD,
            &[ColorTransform::RedOffset(Percent1000(35_000))],
            [175, 129, 189, 255],
        ),
        oracle_case(
            "single_red_mod",
            0xC0504D,
            &[ColorTransform::RedModulation(Percent1000(40_000))],
            [127, 80, 77, 255],
        ),
        oracle_case(
            "single_green",
            0x9BBB59,
            &[ColorTransform::Green(Percent1000(70_000))],
            [155, 218, 89, 255],
        ),
        oracle_case(
            "single_green_off",
            0x8064A2,
            &[ColorTransform::GreenOffset(Percent1000(-20_000))],
            [128, 0, 162, 255],
        ),
        oracle_case(
            "single_green_mod",
            0x4BACC6,
            &[ColorTransform::GreenModulation(Percent1000(140_000))],
            [75, 200, 198, 255],
        ),
        oracle_case(
            "single_blue",
            0xF79646,
            &[ColorTransform::Blue(Percent1000(85_000))],
            [247, 150, 237, 255],
        ),
        oracle_case(
            "single_blue_off",
            0x1F497D,
            &[ColorTransform::BlueOffset(Percent1000(25_000))],
            [31, 73, 180, 255],
        ),
        oracle_case(
            "single_blue_mod",
            0xEEECE1,
            &[ColorTransform::BlueModulation(Percent1000(35_000))],
            [238, 236, 140, 255],
        ),
        oracle_case(
            "single_gamma",
            0x4F81BD,
            &[ColorTransform::Gamma],
            [151, 189, 223, 255],
        ),
        oracle_case(
            "single_inv_gamma",
            0xC0504D,
            &[ColorTransform::InverseGamma],
            [134, 20, 19, 255],
        ),
        oracle_case(
            "stack_red_off_then_mod",
            0x9BBB59,
            &[
                ColorTransform::RedOffset(Percent1000(40_000)),
                ColorTransform::RedModulation(Percent1000(50_000)),
            ],
            [163, 187, 89, 255],
        ),
        oracle_case(
            "stack_red_mod_then_off",
            0x8064A2,
            &[
                ColorTransform::RedModulation(Percent1000(50_000)),
                ColorTransform::RedOffset(Percent1000(40_000)),
            ],
            [189, 100, 162, 255],
        ),
        oracle_case(
            "stack_alpha_clamp_high",
            0x4BACC6,
            &[
                ColorTransform::Alpha(Percent1000(75_000)),
                ColorTransform::AlphaOffset(Percent1000(80_000)),
            ],
            [75, 172, 198, 255],
        ),
        oracle_case(
            "stack_alpha_clamp_low",
            0xF79646,
            &[
                ColorTransform::Alpha(Percent1000(25_000)),
                ColorTransform::AlphaOffset(Percent1000(-80_000)),
            ],
            [0, 0, 0, 0],
        ),
        oracle_case(
            "stack_lum_mod_then_off",
            0x1F497D,
            &[
                ColorTransform::LuminanceModulation(Percent1000(60_000)),
                ColorTransform::LuminanceOffset(Percent1000(25_000)),
            ],
            [44, 103, 177, 255],
        ),
        oracle_case(
            "stack_lum_off_then_mod",
            0xEEECE1,
            &[
                ColorTransform::LuminanceOffset(Percent1000(25_000)),
                ColorTransform::LuminanceModulation(Percent1000(60_000)),
            ],
            [153, 153, 153, 255],
        ),
        oracle_case(
            "stack_hsl",
            0x4F81BD,
            &[
                ColorTransform::HueOffset(Angle(4_200_000)),
                ColorTransform::SaturationModulation(Percent1000(65_000)),
                ColorTransform::LuminanceModulation(Percent1000(80_000)),
            ],
            [121, 76, 139, 255],
        ),
        oracle_case(
            "stack_tint_then_shade",
            0xC0504D,
            &[
                ColorTransform::Tint(Percent1000(60_000)),
                ColorTransform::Shade(Percent1000(70_000)),
            ],
            [188, 152, 151, 255],
        ),
        oracle_case(
            "stack_shade_then_tint",
            0x9BBB59,
            &[
                ColorTransform::Shade(Percent1000(70_000)),
                ColorTransform::Tint(Percent1000(60_000)),
            ],
            [194, 205, 177, 255],
        ),
        oracle_case(
            "stack_linear_tint",
            0x8064A2,
            &[
                ColorTransform::InverseGamma,
                ColorTransform::Tint(Percent1000(55_000)),
                ColorTransform::Gamma,
            ],
            [220, 219, 223, 255],
        ),
        oracle_case(
            "stack_gamma_shade",
            0x4BACC6,
            &[
                ColorTransform::Gamma,
                ColorTransform::Shade(Percent1000(55_000)),
                ColorTransform::InverseGamma,
            ],
            [41, 95, 109, 255],
        ),
        oracle_case(
            "stack_comp_hue_gray",
            0xF79646,
            &[
                ColorTransform::Complement,
                ColorTransform::HueOffset(Angle(-2_400_000)),
                ColorTransform::Gray,
            ],
            [207, 207, 207, 255],
        ),
    ];

    const fn oracle_case(
        name: &'static str,
        input: u32,
        transforms: &'static [ColorTransform],
        expected: [u8; 4],
    ) -> OracleCase {
        OracleCase {
            name,
            input: RgbColor::new(
                ((input >> 16) & 0xff) as u8,
                ((input >> 8) & 0xff) as u8,
                (input & 0xff) as u8,
            ),
            transforms,
            expected,
        }
    }

    fn parse(xml: &[u8]) -> ColorChoice {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        match reader.read_event_into(&mut buffer).unwrap() {
            Event::Start(element) => ColorChoice::from_xml(&mut reader, &element).unwrap(),
            Event::Empty(element) => ColorChoice::from_empty_xml(&element).unwrap(),
            event => panic!("expected colour element, got {event:?}"),
        }
    }

    fn write(colour: &ColorChoice) -> Vec<u8> {
        let mut writer = Writer::new(Vec::new());
        colour.to_xml(&mut writer).unwrap();
        writer.into_inner()
    }

    #[test]
    fn srgb_colour_parses_and_round_trips() {
        let colour = parse(br#"<x:srgbClr val="12ABef"/>"#);
        assert_eq!(
            colour,
            ColorChoice::Srgb {
                value: RgbColor::new(0x12, 0xAB, 0xEF),
                transforms: Vec::new(),
                raw_children: OrderedRawChildren::default(),
            }
        );
        assert_eq!(write(&colour), br#"<a:srgbClr val="12ABEF"/>"#);
    }

    #[test]
    fn scheme_colour_parses_and_round_trips() {
        let colour = parse(br#"<x:schemeClr val="accent2"/>"#);
        assert_eq!(write(&colour), br#"<a:schemeClr val="accent2"/>"#);
    }

    #[test]
    fn system_colour_uses_and_preserves_last_colour() {
        let colour = parse(br#"<x:sysClr val="windowText" lastClr="102030"/>"#);
        assert_eq!(
            write(&colour),
            br#"<a:sysClr val="windowText" lastClr="102030"/>"#
        );
    }

    #[test]
    fn system_colour_without_last_colour_round_trips() {
        let colour = parse(br#"<x:sysClr val="windowText"/>"#);
        assert_eq!(write(&colour), br#"<a:sysClr val="windowText"/>"#);
    }

    #[test]
    fn preset_colour_parses_and_round_trips() {
        let colour = parse(br#"<x:prstClr val="aliceBlue"/>"#);
        assert_eq!(write(&colour), br#"<a:prstClr val="aliceBlue"/>"#);
    }

    #[test]
    fn unknown_colour_children_are_preserved_in_place() {
        let input = br#"<x:schemeClr val="accent2"><z:first z:id="1"/><z:second><z:leaf>one &amp; two</z:leaf></z:second></x:schemeClr>"#;
        let colour = parse(input);

        assert_eq!(
            colour.raw_children().at(0).collect::<Vec<_>>(),
            vec![
                br#"<z:first z:id="1"/>"#.as_slice(),
                br#"<z:second><z:leaf>one &amp; two</z:leaf></z:second>"#.as_slice(),
            ]
        );
        assert_eq!(
            write(&colour),
            br#"<a:schemeClr val="accent2"><z:first z:id="1"/><z:second><z:leaf>one &amp; two</z:leaf></z:second></a:schemeClr>"#
        );
    }

    #[test]
    fn malformed_srgb_values_are_rejected() {
        assert!(matches!(
            RgbColor::parse("12345"),
            Err(ColorError::InvalidRgb(value)) if value == "12345"
        ));
        assert!(matches!(
            RgbColor::parse("GG0000"),
            Err(ColorError::InvalidRgb(value)) if value == "GG0000"
        ));
    }

    #[test]
    fn malformed_system_fallback_is_rejected() {
        let xml = br#"<a:sysClr val="window" lastClr="12345"/>"#;
        let mut reader = Reader::from_reader(xml.as_slice());
        let mut buffer = Vec::new();
        let Event::Empty(element) = reader.read_event_into(&mut buffer).unwrap() else {
            panic!("expected empty system colour");
        };
        assert!(ColorChoice::from_empty_xml(&element).is_err());
    }

    #[test]
    fn standard_colour_map_uses_office_theme_slots() {
        let map = ColorMap::default();
        let expected = [
            (ColorMapSlot::Background1, ThemeColorSlot::Light1),
            (ColorMapSlot::Text1, ThemeColorSlot::Dark1),
            (ColorMapSlot::Background2, ThemeColorSlot::Light2),
            (ColorMapSlot::Text2, ThemeColorSlot::Dark2),
            (ColorMapSlot::Accent1, ThemeColorSlot::Accent1),
            (ColorMapSlot::Accent2, ThemeColorSlot::Accent2),
            (ColorMapSlot::Accent3, ThemeColorSlot::Accent3),
            (ColorMapSlot::Accent4, ThemeColorSlot::Accent4),
            (ColorMapSlot::Accent5, ThemeColorSlot::Accent5),
            (ColorMapSlot::Accent6, ThemeColorSlot::Accent6),
            (ColorMapSlot::Hyperlink, ThemeColorSlot::Hyperlink),
            (
                ColorMapSlot::FollowedHyperlink,
                ThemeColorSlot::FollowedHyperlink,
            ),
        ];

        for (source, destination) in expected {
            assert_eq!(map.theme_slot(source), destination);
        }
    }

    #[test]
    fn dark_master_colour_map_inverts_background_and_text() {
        let map = ColorMap::default().with_overrides(&[
            (ColorMapSlot::Background1, ThemeColorSlot::Dark1),
            (ColorMapSlot::Text1, ThemeColorSlot::Light1),
        ]);
        let theme = [
            ("dk1", RgbColor::new(0x1F, 0x49, 0x7D)),
            ("lt1", RgbColor::new(0xEE, 0xDD, 0xCC)),
        ];

        assert_eq!(
            resolve_color(
                &parse(br#"<a:schemeClr val="bg1"><a:tint val="62000"/></a:schemeClr>"#),
                &map,
                &theme,
            )
            .unwrap(),
            ResolvedColor::new(167, 174, 189, 255)
        );
        assert_eq!(
            resolve_color(&parse(br#"<a:schemeClr val="tx1"/>"#), &map, &theme).unwrap(),
            ResolvedColor::new(0xEE, 0xDD, 0xCC, 255)
        );
    }

    #[test]
    fn colour_map_override_wins_before_theme_lookup() {
        let master = ColorMap::new(
            ThemeColorSlot::Dark2,
            ThemeColorSlot::Light2,
            ThemeColorSlot::Accent3,
            ThemeColorSlot::Accent4,
            ThemeColorSlot::Accent5,
            ThemeColorSlot::Accent6,
            ThemeColorSlot::Accent1,
            ThemeColorSlot::Accent2,
            ThemeColorSlot::Dark1,
            ThemeColorSlot::Light1,
            ThemeColorSlot::FollowedHyperlink,
            ThemeColorSlot::Hyperlink,
        );
        let map = master.with_overrides(&[(ColorMapSlot::Background1, ThemeColorSlot::Accent6)]);
        let expected = ColorMap::new(
            ThemeColorSlot::Accent6,
            ThemeColorSlot::Light2,
            ThemeColorSlot::Accent3,
            ThemeColorSlot::Accent4,
            ThemeColorSlot::Accent5,
            ThemeColorSlot::Accent6,
            ThemeColorSlot::Accent1,
            ThemeColorSlot::Accent2,
            ThemeColorSlot::Dark1,
            ThemeColorSlot::Light1,
            ThemeColorSlot::FollowedHyperlink,
            ThemeColorSlot::Hyperlink,
        );

        assert_eq!(map, expected);
        assert_eq!(
            master.theme_slot(ColorMapSlot::Background1),
            ThemeColorSlot::Dark2
        );
    }

    #[test]
    fn direct_colours_bypass_the_master_colour_map() {
        let standard = ColorMap::default();
        let dark = standard.with_overrides(&[
            (ColorMapSlot::Background1, ThemeColorSlot::Dark1),
            (ColorMapSlot::Text1, ThemeColorSlot::Light1),
        ]);
        let lookup = [
            ("windowText", RgbColor::new(0x10, 0x20, 0x30)),
            ("aliceBlue", RgbColor::new(0xF0, 0xF8, 0xFF)),
        ];
        let direct = [
            (
                parse(br#"<a:srgbClr val="EEECE1"><a:shade val="58000"/></a:srgbClr>"#),
                ResolvedColor::new(187, 185, 176, 255),
            ),
            (
                parse(br#"<a:sysClr val="windowText" lastClr="FFFFFF"/>"#),
                ResolvedColor::new(0x10, 0x20, 0x30, 255),
            ),
            (
                parse(br#"<a:sysClr val="missing" lastClr="AABBCC"/>"#),
                ResolvedColor::new(0xAA, 0xBB, 0xCC, 255),
            ),
            (
                parse(br#"<a:prstClr val="aliceBlue"/>"#),
                ResolvedColor::new(0xF0, 0xF8, 0xFF, 255),
            ),
        ];

        for (colour, expected) in direct {
            assert_eq!(
                resolve_color(&colour, &standard, &lookup).unwrap(),
                expected
            );
            assert_eq!(resolve_color(&colour, &dark, &lookup).unwrap(), expected);
        }
    }

    #[test]
    fn powerpoint_colour_transform_oracle_matches_all_forty_pairs() {
        assert_eq!(POWERPOINT_ORACLE_VERSION, "16.104");
        assert_eq!(POWERPOINT_ORACLE_BUILD, "16.104.25121423");
        assert_eq!(ORACLE_CASES.len(), 40);

        for case in ORACLE_CASES {
            assert_eq!(
                apply_color_transforms(case.input, case.transforms).rgba(),
                case.expected,
                "PowerPoint colour oracle disagreement for {}",
                case.name
            );
        }
    }

    #[test]
    fn colour_transforms_apply_in_document_order() {
        let base = RgbColor::new(0x33, 0x66, 0x99);
        let forward = apply_color_transforms(
            base,
            &[
                ColorTransform::RedOffset(Percent1000(40_000)),
                ColorTransform::RedModulation(Percent1000(50_000)),
            ],
        );
        let reversed = apply_color_transforms(
            base,
            &[
                ColorTransform::RedModulation(Percent1000(50_000)),
                ColorTransform::RedOffset(Percent1000(40_000)),
            ],
        );

        assert_eq!(forward.red, 128);
        assert_eq!(reversed.red, 173);
        assert_ne!(forward, reversed);
    }

    #[test]
    fn linear_gamma_round_trip_preserves_channel_endpoints() {
        assert_eq!(srgb_to_linear(0.0), 0.0);
        assert_eq!(srgb_to_linear(1.0), 1.0);
        assert_eq!(linear_to_srgb(0.0), 0.0);
        assert!((linear_to_srgb(1.0) - 1.0).abs() < f64::EPSILON);
        for channel in [0.01, 0.18, 0.5, 0.75] {
            assert!((linear_to_srgb(srgb_to_linear(channel)) - channel).abs() < 1e-12);
        }
    }

    #[test]
    fn alpha_transforms_clamp_to_the_valid_range() {
        let colour = apply_color_transforms(
            RgbColor::new(0x12, 0x34, 0x56),
            &[
                ColorTransform::Alpha(Percent1000(75_000)),
                ColorTransform::AlphaModulation(Percent1000(50_000)),
                ColorTransform::AlphaOffset(Percent1000(80_000)),
            ],
        );

        assert_eq!(colour, ResolvedColor::new(0x12, 0x34, 0x56, 255));
    }

    #[test]
    fn known_and_unknown_transform_children_keep_document_order() {
        let input = br#"<x:srgbClr val="336699"><z:before z:id="1"/><x:tint val="65000"/><z:middle><z:leaf/></z:middle><x:hueOff val="5400000"/><z:after z:id="3"/></x:srgbClr>"#;
        let colour = parse(input);

        assert_eq!(
            colour.transforms(),
            &[
                ColorTransform::Tint(Percent1000(65_000)),
                ColorTransform::HueOffset(Angle(5_400_000)),
            ]
        );
        assert_eq!(
            write(&colour),
            br#"<a:srgbClr val="336699"><z:before z:id="1"/><a:tint val="65000"/><z:middle><z:leaf/></z:middle><a:hueOff val="5400000"/><z:after z:id="3"/></a:srgbClr>"#
        );
    }

    #[test]
    fn nonempty_known_transform_preserves_its_nested_xml_verbatim() {
        let input = br#"<x:srgbClr val="336699"><x:tint val="65000"><z:extension z:id="1"/></x:tint></x:srgbClr>"#;
        let colour = parse(input);

        assert!(colour.transforms().is_empty());
        assert_eq!(
            write(&colour),
            br#"<a:srgbClr val="336699"><x:tint val="65000"><z:extension z:id="1"/></x:tint></a:srgbClr>"#
        );
    }

    #[test]
    fn explicit_empty_transform_pair_is_modelled_and_canonicalised() {
        let input = br#"<x:srgbClr val="336699"><x:tint val="65000"></x:tint></x:srgbClr>"#;
        let colour = parse(input);

        assert_eq!(
            colour.transforms(),
            &[ColorTransform::Tint(Percent1000(65_000))]
        );
        assert_eq!(
            write(&colour),
            br#"<a:srgbClr val="336699"><a:tint val="65000"/></a:srgbClr>"#
        );
    }

    #[test]
    fn partially_transparent_rgba_matches_powerpoint_png_quantization() {
        let offset = apply_color_transforms(
            RgbColor::new(0x4B, 0xAC, 0xC6),
            &[ColorTransform::AlphaOffset(Percent1000(-30_000))],
        );
        let modulation = apply_color_transforms(
            RgbColor::new(0xF7, 0x96, 0x46),
            &[ColorTransform::AlphaModulation(Percent1000(43_000))],
        );

        assert_eq!(offset.rgba(), [76, 172, 198, 179]);
        assert_eq!(modulation.rgba(), [248, 151, 70, 110]);
    }

    #[test]
    #[ignore = "requires RDOCX_POWERPOINT_ORACLE_SHELL, pinned Microsoft PowerPoint, and native shape clipboard PNGs"]
    fn generate_powerpoint_colour_transform_oracle() {
        assert_powerpoint_build();
        let supplied_shell = PathBuf::from(
            std::env::var_os("RDOCX_POWERPOINT_ORACLE_SHELL").expect(
                "RDOCX_POWERPOINT_ORACLE_SHELL must name a PowerPoint-authored PPTX with one blank slide and one shape named probe_shape",
            ),
        );
        let output_dir = std::env::temp_dir().join(format!(
            "rdocx-f055-powerpoint-oracle-{}",
            std::process::id()
        ));
        fs::create_dir_all(&output_dir).unwrap();
        let shell_path = output_dir.join("powerpoint-native-shell.pptx");
        let deck_path = output_dir.join("colour-transform-oracle.pptx");
        fs::write(&shell_path, fs::read(&supplied_shell).unwrap()).unwrap();
        validate_powerpoint_shell(&shell_path);
        inject_oracle_transforms(&shell_path, &deck_path);
        validate_powerpoint_deck(&deck_path);
        export_oracle_shapes(&deck_path, &output_dir);

        for case in ORACLE_CASES {
            let rgba = sample_uniform_centre(&output_dir.join(format!("{}.png", case.name)));
            let implementation = apply_color_transforms(case.input, case.transforms).rgba();
            println!(
                "{}: PowerPoint {rgba:?}, implementation {implementation:?}",
                case.name
            );
        }
        println!("oracle artefacts: {}", output_dir.display());
    }

    fn assert_powerpoint_build() {
        let app = "/Applications/Microsoft PowerPoint.app/Contents/Info.plist";
        let version = Command::new("/usr/libexec/PlistBuddy")
            .args(["-c", "Print :CFBundleShortVersionString", app])
            .output()
            .unwrap();
        let build = Command::new("/usr/libexec/PlistBuddy")
            .args(["-c", "Print :CFBundleVersion", app])
            .output()
            .unwrap();
        assert!(version.status.success());
        assert!(build.status.success());
        assert_eq!(
            String::from_utf8(version.stdout).unwrap().trim(),
            POWERPOINT_ORACLE_VERSION
        );
        assert_eq!(
            String::from_utf8(build.stdout).unwrap().trim(),
            POWERPOINT_ORACLE_BUILD
        );
    }

    fn validate_powerpoint_shell(path: &Path) {
        let path = path.to_string_lossy();
        let name = Path::new(path.as_ref())
            .file_name()
            .unwrap()
            .to_string_lossy();
        let mut script = powerpoint_script_start(120);
        script.push_str(&format!(
            "set deckPath to \"{path}\"\nopen my POSIX file deckPath\nset shellDeck to presentation \"{name}\"\nif (full name of shellDeck) is not deckPath then error \"oracle shell exact path mismatch\"\nif (count of slides of shellDeck) is not 1 then error \"oracle shell slide count mismatch\"\nif (count of shapes of slide 1 of shellDeck) is not 1 then error \"oracle shell shape count mismatch\"\nif (name of shape 1 of slide 1 of shellDeck) is not \"probe_shape\" then error \"oracle shell lacks probe_shape\"\nclose shellDeck saving no\n"
        ));
        script.push_str(&powerpoint_script_finish("shellDeck"));
        run_powerpoint_script(&script, "PowerPoint shell validation");
    }

    fn validate_powerpoint_deck(path: &Path) {
        let path = path.to_string_lossy();
        let name = Path::new(path.as_ref())
            .file_name()
            .unwrap()
            .to_string_lossy();
        let mut script = powerpoint_script_start(120);
        script.push_str(&format!(
            "set deckPath to \"{path}\"\nopen my POSIX file deckPath\nset checkedDeck to presentation \"{name}\"\nif (full name of checkedDeck) is not deckPath then error \"oracle deck exact path mismatch\"\nif (count of slides of checkedDeck) is not 1 then error \"oracle deck slide count mismatch\"\nif (count of shapes of slide 1 of checkedDeck) is not 40 then error \"oracle deck shape count mismatch\"\nset oracleShapeNames to name of every shape of slide 1 of checkedDeck\n"
        ));
        for case in ORACLE_CASES {
            script.push_str(&format!(
                "if oracleShapeNames does not contain \"{}\" then error \"missing oracle shape {}\"\n",
                case.name, case.name
            ));
        }
        script.push_str("close checkedDeck saving no\n");
        script.push_str(&powerpoint_script_finish("checkedDeck"));
        run_powerpoint_script(&script, "PowerPoint deck validation");
    }

    fn inject_oracle_transforms(shell_path: &Path, deck_path: &Path) {
        let mut package = OpcPackage::open(shell_path).unwrap();
        let presentation_part = package.main_document_part().unwrap();
        let slide_target = package
            .get_part_rels(&presentation_part)
            .unwrap()
            .get_by_type(rel_types::SLIDE)
            .unwrap()
            .target
            .clone();
        let slide_part = OpcPackage::resolve_rel_target(&presentation_part, &slide_target);
        let mut slide_xml =
            String::from_utf8(package.get_part(&slide_part).unwrap().to_vec()).unwrap();

        let name_marker = "name=\"probe_shape\"";
        let name_index = slide_xml.find(name_marker).unwrap();
        let shape_start = slide_xml[..name_index].rfind("<p:sp>").unwrap();
        let shape_end =
            name_index + slide_xml[name_index..].find("</p:sp>").unwrap() + "</p:sp>".len();
        let shape_template = &slide_xml[shape_start..shape_end];
        let mut generated_shapes = String::new();

        for (index, case) in ORACLE_CASES.iter().enumerate() {
            let mut shape = shape_template.replacen(
                "id=\"2\" name=\"probe_shape\"",
                &format!("id=\"{}\" name=\"{}\"", index + 2, case.name),
                1,
            );
            let mut transform_writer = Writer::new(Vec::new());
            for transform in case.transforms {
                transform.to_xml(&mut transform_writer).unwrap();
            }
            let transform_xml = String::from_utf8(transform_writer.into_inner()).unwrap();
            let replacement = format!(
                "<a:srgbClr val=\"{}\">{transform_xml}</a:srgbClr>",
                case.input
            );
            shape = shape.replacen("<a:srgbClr val=\"1F497D\"/>", &replacement, 1);
            assert!(shape.contains(&format!("name=\"{}\"", case.name)));
            assert!(shape.contains(&replacement));
            generated_shapes.push_str(&shape);
        }
        slide_xml.replace_range(shape_start..shape_end, &generated_shapes);

        package.set_part(&slide_part, slide_xml.into_bytes());
        package.save(deck_path).unwrap();
    }

    fn powerpoint_script_start(timeout_seconds: u32) -> String {
        format!(
            "with timeout of {timeout_seconds} seconds\ntell application \"Microsoft PowerPoint\"\nset previousStartUpDialog to start up dialog\ntry\nif (Version as text) is not \"{POWERPOINT_ORACLE_VERSION}\" then error \"PowerPoint version mismatch: \" & (Version as text)\nif (build as text) is not \"{POWERPOINT_ORACLE_APP_BUILD}\" then error \"PowerPoint application build mismatch: \" & (build as text)\nset start up dialog to false\n"
        )
    }

    fn powerpoint_script_finish(deck_variable: &str) -> String {
        format!(
            "set start up dialog to previousStartUpDialog\non error errorMessage number errorNumber\ntry\nclose {deck_variable} saving no\nend try\nset start up dialog to previousStartUpDialog\nerror errorMessage number errorNumber\nend try\nend tell\nend timeout\n"
        )
    }

    fn run_powerpoint_script(script: &str, action: &str) {
        let result = Command::new("osascript")
            .args(["-e", script])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{action} failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }

    fn export_oracle_shapes(deck_path: &Path, output_dir: &Path) {
        let deck = deck_path.to_string_lossy();
        let name = Path::new(deck.as_ref())
            .file_name()
            .unwrap()
            .to_string_lossy();
        let mut script = powerpoint_script_start(600);
        script.push_str(&format!(
            "set deckPath to \"{deck}\"\nopen my POSIX file deckPath\nset oracleDeck to presentation \"{name}\"\nif (full name of oracleDeck) is not deckPath then error \"oracle export exact path mismatch\"\n"
        ));
        for case in ORACLE_CASES {
            let output = output_dir.join(format!("{}.png", case.name));
            script.push_str(&format!(
                "set oracleShape to shape \"{}\" of slide 1 of oracleDeck\ncopy shape oracleShape\ndelay 0.5\ntell me\nset pngData to the clipboard as «class PNGf»\nset outputFile to open for access (POSIX file \"{}\") with write permission\nset eof outputFile to 0\nwrite pngData to outputFile\nclose access outputFile\nend tell\n",
                case.name,
                output.to_string_lossy()
            ));
        }
        script.push_str("close oracleDeck saving no\n");
        script.push_str(&powerpoint_script_finish("oracleDeck"));
        run_powerpoint_script(&script, "PowerPoint direct shape clipboard render");
    }

    fn sample_uniform_centre(path: &Path) -> [u8; 4] {
        let rgb = run_pngtopnm(path, false);
        let alpha = run_pngtopnm(path, true);
        assert_eq!((rgb.width, rgb.height), (alpha.width, alpha.height));
        let centre_x = rgb.width / 2;
        let centre_y = rgb.height / 2;
        let mut sample = None;
        for y in centre_y - 2..=centre_y + 2 {
            for x in centre_x - 2..=centre_x + 2 {
                let rgb_offset = (y * rgb.width + x) * 3;
                let alpha_offset = y * alpha.width + x;
                let pixel = [
                    rgb.data[rgb_offset],
                    rgb.data[rgb_offset + 1],
                    rgb.data[rgb_offset + 2],
                    alpha.data[alpha_offset],
                ];
                assert_eq!(
                    *sample.get_or_insert(pixel),
                    pixel,
                    "non-uniform 5 by 5 centre block in {}",
                    path.display()
                );
            }
        }
        sample.unwrap()
    }

    struct NetpbmImage {
        width: usize,
        height: usize,
        data: Vec<u8>,
    }

    fn run_pngtopnm(path: &Path, alpha: bool) -> NetpbmImage {
        let mut command = Command::new("pngtopnm");
        if alpha {
            command.arg("-alpha");
        }
        let output = command.arg(path).output().unwrap();
        assert!(
            output.status.success(),
            "pngtopnm failed for {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        parse_netpbm(output.stdout, if alpha { b'5' } else { b'6' })
    }

    fn parse_netpbm(bytes: Vec<u8>, expected_kind: u8) -> NetpbmImage {
        let mut cursor = 0;
        let magic = netpbm_token(&bytes, &mut cursor);
        assert_eq!(magic, [b'P', expected_kind]);
        let width = parse_netpbm_usize(netpbm_token(&bytes, &mut cursor));
        let height = parse_netpbm_usize(netpbm_token(&bytes, &mut cursor));
        assert_eq!(parse_netpbm_usize(netpbm_token(&bytes, &mut cursor)), 255);
        assert!(bytes[cursor].is_ascii_whitespace());
        cursor += 1;
        let channels = if expected_kind == b'6' { 3 } else { 1 };
        assert_eq!(bytes.len() - cursor, width * height * channels);
        NetpbmImage {
            width,
            height,
            data: bytes[cursor..].to_vec(),
        }
    }

    fn netpbm_token<'a>(bytes: &'a [u8], cursor: &mut usize) -> &'a [u8] {
        loop {
            while bytes[*cursor].is_ascii_whitespace() {
                *cursor += 1;
            }
            if bytes[*cursor] != b'#' {
                break;
            }
            while bytes[*cursor] != b'\n' {
                *cursor += 1;
            }
        }
        let start = *cursor;
        while *cursor < bytes.len() && !bytes[*cursor].is_ascii_whitespace() {
            *cursor += 1;
        }
        &bytes[start..*cursor]
    }

    fn parse_netpbm_usize(token: &[u8]) -> usize {
        std::str::from_utf8(token).unwrap().parse().unwrap()
    }

    #[test]
    fn all_twenty_eight_transform_elements_parse_and_write_with_fixed_prefixes() {
        let input = br#"<x:srgbClr val="123456"><x:tint val="10000"/><x:shade val="20000"/><x:comp/><x:inv/><x:gray/><x:alpha val="30000"/><x:alphaOff val="-40000"/><x:alphaMod val="50000"/><x:hue val="60000"/><x:hueOff val="-120000"/><x:hueMod val="60000"/><x:sat val="70000"/><x:satOff val="-80000"/><x:satMod val="90000"/><x:lum val="100000"/><x:lumOff val="-10000"/><x:lumMod val="100000"/><x:red val="12000"/><x:redOff val="-13000"/><x:redMod val="40000"/><x:green val="15000"/><x:greenOff val="-16000"/><x:greenMod val="70000"/><x:blue val="18000"/><x:blueOff val="-19000"/><x:blueMod val="100000"/><x:gamma/><x:invGamma/></x:srgbClr>"#;
        let colour = parse(input);

        assert_eq!(colour.transforms().len(), 28);
        let output = String::from_utf8(write(&colour)).unwrap();
        assert!(!output.contains("<x:"));
        assert!(output.contains("<a:hue val=\"60000\"/>"));
        assert!(output.contains("<a:hueOff val=\"-120000\"/>"));
        assert!(output.contains("<a:invGamma/>"));
    }

    #[test]
    fn whitespace_and_comments_in_empty_transform_pairs_are_modelled() {
        let input = br#"<x:srgbClr val="336699"><x:tint val="65000">
            <!-- formatting only -->
        </x:tint></x:srgbClr>"#;
        let colour = parse(input);

        assert_eq!(
            colour.transforms(),
            &[ColorTransform::Tint(Percent1000(65_000))]
        );
        assert_eq!(
            write(&colour),
            br#"<a:srgbClr val="336699"><a:tint val="65000"/></a:srgbClr>"#
        );
    }
}
