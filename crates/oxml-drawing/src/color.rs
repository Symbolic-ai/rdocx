use std::fmt;
use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_core::xml::{get_attr, local_name, matches_local_name};
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

/// Errors produced while parsing or writing DrawingML colours.
#[derive(Debug)]
pub enum ColorError {
    Xml(OxmlError),
    InvalidRgb(String),
    MissingAttribute { element: String, attribute: String },
    UnexpectedElement(String),
}

impl fmt::Display for ColorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml(error) => error.fmt(formatter),
            Self::InvalidRgb(value) => write!(
                formatter,
                "DrawingML RGB colour must be exactly six hexadecimal digits: {value}"
            ),
            Self::MissingAttribute { element, attribute } => {
                write!(formatter, "DrawingML {element} requires @{attribute}")
            }
            Self::UnexpectedElement(element) => {
                write!(formatter, "unexpected DrawingML colour element: {element}")
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

/// One of the four DrawingML colour choice elements.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ColorChoice {
    Srgb {
        value: RgbColor,
        raw_children: Vec<Vec<u8>>,
    },
    Scheme {
        value: String,
        raw_children: Vec<Vec<u8>>,
    },
    System {
        value: String,
        last_color: Option<RgbColor>,
        raw_children: Vec<Vec<u8>>,
    },
    Preset {
        value: String,
        raw_children: Vec<Vec<u8>>,
    },
}

impl ColorChoice {
    /// Parses a colour after the caller has consumed its start event.
    pub fn from_xml(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let qualified_name = start.name();
        let element_name = local_name(qualified_name.as_ref());
        let raw_children = capture_children(reader, element_name)?;
        Self::from_parts(start, raw_children)
    }

    /// Parses a colour from a self-closing element.
    pub fn from_empty_xml(start: &BytesStart<'_>) -> Result<Self> {
        Self::from_parts(start, Vec::new())
    }

    fn from_parts(start: &BytesStart<'_>, raw_children: Vec<Vec<u8>>) -> Result<Self> {
        let qualified_name = start.name();
        let element_name = local_name(qualified_name.as_ref());
        let value = required_attr(start, b"val")?;
        match element_name {
            b"srgbClr" => Ok(Self::Srgb {
                value: RgbColor::parse(&value)?,
                raw_children,
            }),
            b"schemeClr" => Ok(Self::Scheme {
                value,
                raw_children,
            }),
            b"sysClr" => Ok(Self::System {
                value,
                last_color: get_attr(start, b"lastClr")
                    .map(|last| RgbColor::parse(&last))
                    .transpose()?,
                raw_children,
            }),
            b"prstClr" => Ok(Self::Preset {
                value,
                raw_children,
            }),
            _ => Err(ColorError::UnexpectedElement(
                String::from_utf8_lossy(start.name().as_ref()).into_owned(),
            )),
        }
    }

    /// Writes this nested colour element with the canonical `a:` prefix.
    pub fn to_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let (tag, value, last_color, raw_children) = match self {
            Self::Srgb {
                value,
                raw_children,
            } => ("a:srgbClr", value.to_string(), None, raw_children),
            Self::Scheme {
                value,
                raw_children,
            } => ("a:schemeClr", value.clone(), None, raw_children),
            Self::System {
                value,
                last_color,
                raw_children,
            } => (
                "a:sysClr",
                value.clone(),
                last_color.map(|colour| colour.to_string()),
                raw_children,
            ),
            Self::Preset {
                value,
                raw_children,
            } => ("a:prstClr", value.clone(), None, raw_children),
        };

        let mut start = BytesStart::new(tag);
        start.push_attribute(("val", value.as_str()));
        if let Some(last_color) = last_color.as_deref() {
            start.push_attribute(("lastClr", last_color));
        }

        if raw_children.is_empty() {
            writer
                .write_event(Event::Empty(start))
                .map_err(OxmlError::from)?;
            return Ok(());
        }

        writer
            .write_event(Event::Start(start))
            .map_err(OxmlError::from)?;
        for raw in raw_children {
            writer.get_mut().write_all(raw).map_err(OxmlError::from)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new(tag)))
            .map_err(OxmlError::from)?;
        Ok(())
    }

    /// Returns raw, not-yet-modelled children in document order.
    pub fn raw_children(&self) -> &[Vec<u8>] {
        match self {
            Self::Srgb { raw_children, .. }
            | Self::Scheme { raw_children, .. }
            | Self::System { raw_children, .. }
            | Self::Preset { raw_children, .. } => raw_children,
        }
    }
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

fn capture_children(reader: &mut Reader<&[u8]>, end_name: &[u8]) -> Result<Vec<Vec<u8>>> {
    let mut raw_children = Vec::new();
    let mut buffer = Vec::new();

    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) => {
                raw_children.push(capture_element(reader, &element)?);
            }
            Event::Empty(element) => {
                raw_children.push(capture_empty_element(&element)?);
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

    Ok(raw_children)
}

#[cfg(test)]
mod tests {
    use quick_xml::events::Event;

    use super::{ColorChoice, ColorError, RgbColor};
    use quick_xml::{Reader, Writer};

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
                raw_children: Vec::new(),
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
            colour.raw_children(),
            &[
                br#"<z:first z:id="1"/>"#.to_vec(),
                br#"<z:second><z:leaf>one &amp; two</z:leaf></z:second>"#.to_vec(),
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
}
