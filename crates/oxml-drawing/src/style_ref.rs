use std::fmt;
use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_core::xml::{get_attr, local_name};
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::color::{ColorChoice, ColorError};
use crate::order::OrderedRawChildren;

/// Errors produced while parsing, writing, or classifying style references.
#[derive(Debug)]
pub enum StyleReferenceError {
    Xml(OxmlError),
    Color(ColorError),
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
    NotFillReference,
}

impl fmt::Display for StyleReferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml(error) => error.fmt(formatter),
            Self::Color(error) => error.fmt(formatter),
            Self::UnexpectedElement(element) => {
                write!(
                    formatter,
                    "unexpected DrawingML style-reference element: {element}"
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
            Self::NotFillReference => write!(formatter, "style reference is not a fillRef"),
        }
    }
}

impl std::error::Error for StyleReferenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml(error) => Some(error),
            Self::Color(error) => Some(error),
            _ => None,
        }
    }
}

impl From<OxmlError> for StyleReferenceError {
    fn from(error: OxmlError) -> Self {
        Self::Xml(error)
    }
}

impl From<ColorError> for StyleReferenceError {
    fn from(error: ColorError) -> Self {
        Self::Color(error)
    }
}

pub type Result<T> = std::result::Result<T, StyleReferenceError>;

/// The normal or background format-scheme list selected by `fillRef@idx`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FillStyleSelection {
    FillStyle(u32),
    BackgroundFillStyle(u32),
}

impl FillStyleSelection {
    /// Classifies a fill reference without performing theme lookup.
    pub const fn from_index(index: u32) -> Self {
        if index > 1000 {
            Self::BackgroundFillStyle(index - 1000)
        } else {
            Self::FillStyle(index)
        }
    }
}

/// The index and colour carried by `lnRef`, `fillRef`, or `effectRef`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StyleMatrixReference {
    pub index: u32,
    pub color: Option<ColorChoice>,
    raw_children: OrderedRawChildren,
}

impl StyleMatrixReference {
    pub fn new(index: u32, color: ColorChoice) -> Self {
        Self {
            index,
            color: Some(color),
            raw_children: OrderedRawChildren::default(),
        }
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }
}

/// The theme font collection selected by `fontRef@idx`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontCollectionIndex {
    Major,
    Minor,
    None,
}

impl FontCollectionIndex {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "major" => Some(Self::Major),
            "minor" => Some(Self::Minor),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Major => "major",
            Self::Minor => "minor",
            Self::None => "none",
        }
    }
}

/// The font collection and colour carried by `fontRef`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontReference {
    pub index: FontCollectionIndex,
    pub color: Option<ColorChoice>,
    raw_children: OrderedRawChildren,
}

impl FontReference {
    pub fn new(index: FontCollectionIndex, color: ColorChoice) -> Self {
        Self {
            index,
            color: Some(color),
            raw_children: OrderedRawChildren::default(),
        }
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }
}

/// One of the four style-reference forms carried by a shape style.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StyleReference {
    Line(StyleMatrixReference),
    Fill(StyleMatrixReference),
    Effect(StyleMatrixReference),
    Font(FontReference),
}

impl StyleReference {
    /// Parses one complete style-reference element with any namespace prefix.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) => {
                    let kind = ReferenceKind::parse(element.name().as_ref())?;
                    return Self::from_element(&mut reader, &element, kind);
                }
                Event::Empty(element) => {
                    let kind = ReferenceKind::parse(element.name().as_ref())?;
                    let index = kind.parse_index(&element)?;
                    return kind.build(index, None, OrderedRawChildren::default());
                }
                Event::Eof => {
                    return Err(StyleReferenceError::Xml(OxmlError::MissingElement(
                        "DrawingML style reference".to_owned(),
                    )));
                }
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_element(
        reader: &mut Reader<&[u8]>,
        start: &BytesStart<'_>,
        kind: ReferenceKind,
    ) -> Result<Self> {
        let index = kind.parse_index(start)?;
        let mut color = None;
        let mut has_color = false;
        let mut raw_children = OrderedRawChildren::default();
        let mut boundary = 0;
        let mut buffer = Vec::new();

        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element)
                    if is_modelled_color(element.name().as_ref()) && !has_color =>
                {
                    color = Some(ColorChoice::from_xml(reader, &element)?);
                    has_color = true;
                    boundary = 1;
                }
                Event::Empty(element)
                    if is_modelled_color(element.name().as_ref()) && !has_color =>
                {
                    color = Some(ColorChoice::from_empty_xml(&element)?);
                    has_color = true;
                    boundary = 1;
                }
                Event::Start(element) => {
                    let occupies_color = is_any_color(element.name().as_ref()) && !has_color;
                    raw_children.push(boundary, capture_element(reader, &element)?);
                    if occupies_color {
                        has_color = true;
                        boundary = 1;
                    }
                }
                Event::Empty(element) => {
                    let occupies_color = is_any_color(element.name().as_ref()) && !has_color;
                    raw_children.push(boundary, capture_empty_element(&element)?);
                    if occupies_color {
                        has_color = true;
                        boundary = 1;
                    }
                }
                Event::End(element)
                    if local_name(element.name().as_ref()) == kind.element_name().as_bytes() =>
                {
                    break;
                }
                Event::Eof => return Err(missing_end(kind.element_name())),
                _ => {}
            }
            buffer.clear();
        }

        kind.build(index, color, raw_children)
    }

    /// Returns the checked format-scheme selection for a fill reference.
    pub fn fill_style_selection(&self) -> Result<FillStyleSelection> {
        match self {
            Self::Fill(reference) => Ok(FillStyleSelection::from_index(reference.index)),
            _ => Err(StyleReferenceError::NotFillReference),
        }
    }

    /// Writes the reference with the fixed `a:` prefix.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        self.write_xml(&mut writer)?;
        Ok(writer.into_inner())
    }

    /// Writes the reference into an existing XML writer.
    pub fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        match self {
            Self::Line(reference) => write_matrix_reference(writer, "a:lnRef", reference),
            Self::Fill(reference) => write_matrix_reference(writer, "a:fillRef", reference),
            Self::Effect(reference) => write_matrix_reference(writer, "a:effectRef", reference),
            Self::Font(reference) => write_font_reference(writer, reference),
        }
    }
}

#[derive(Clone, Copy)]
enum ReferenceKind {
    Line,
    Fill,
    Effect,
    Font,
}

impl ReferenceKind {
    fn parse(name: &[u8]) -> Result<Self> {
        match local_name(name) {
            b"lnRef" => Ok(Self::Line),
            b"fillRef" => Ok(Self::Fill),
            b"effectRef" => Ok(Self::Effect),
            b"fontRef" => Ok(Self::Font),
            _ => Err(StyleReferenceError::UnexpectedElement(
                String::from_utf8_lossy(name).into_owned(),
            )),
        }
    }

    const fn element_name(self) -> &'static str {
        match self {
            Self::Line => "lnRef",
            Self::Fill => "fillRef",
            Self::Effect => "effectRef",
            Self::Font => "fontRef",
        }
    }

    fn parse_index(self, start: &BytesStart<'_>) -> Result<ReferenceIndex> {
        let value =
            get_attr(start, b"idx").ok_or_else(|| StyleReferenceError::MissingAttribute {
                element: self.element_name().to_owned(),
                attribute: "idx".to_owned(),
            })?;
        if matches!(self, Self::Font) {
            return FontCollectionIndex::parse(&value)
                .map(ReferenceIndex::Font)
                .ok_or_else(|| invalid_attribute(self.element_name(), "idx", value));
        }
        let index = value
            .parse::<u32>()
            .map_err(|_| invalid_attribute(self.element_name(), "idx", value.clone()))?;
        Ok(ReferenceIndex::Matrix(index))
    }

    fn build(
        self,
        index: ReferenceIndex,
        color: Option<ColorChoice>,
        raw_children: OrderedRawChildren,
    ) -> Result<StyleReference> {
        let reference = match (self, index) {
            (Self::Line, ReferenceIndex::Matrix(index)) => {
                StyleReference::Line(StyleMatrixReference {
                    index,
                    color,
                    raw_children,
                })
            }
            (Self::Fill, ReferenceIndex::Matrix(index)) => {
                StyleReference::Fill(StyleMatrixReference {
                    index,
                    color,
                    raw_children,
                })
            }
            (Self::Effect, ReferenceIndex::Matrix(index)) => {
                StyleReference::Effect(StyleMatrixReference {
                    index,
                    color,
                    raw_children,
                })
            }
            (Self::Font, ReferenceIndex::Font(index)) => StyleReference::Font(FontReference {
                index,
                color,
                raw_children,
            }),
            _ => {
                return Err(StyleReferenceError::InvalidAttribute {
                    element: self.element_name().to_owned(),
                    attribute: "idx".to_owned(),
                    value: "index kind mismatch".to_owned(),
                });
            }
        };
        Ok(reference)
    }
}

enum ReferenceIndex {
    Matrix(u32),
    Font(FontCollectionIndex),
}

fn write_matrix_reference<W: Write>(
    writer: &mut Writer<W>,
    tag: &str,
    reference: &StyleMatrixReference,
) -> Result<()> {
    write_reference(
        writer,
        tag,
        reference.index.to_string(),
        reference.color.as_ref(),
        &reference.raw_children,
    )
}

fn write_font_reference<W: Write>(writer: &mut Writer<W>, reference: &FontReference) -> Result<()> {
    write_reference(
        writer,
        "a:fontRef",
        reference.index.as_str().to_owned(),
        reference.color.as_ref(),
        &reference.raw_children,
    )
}

fn write_reference<W: Write>(
    writer: &mut Writer<W>,
    tag: &str,
    index: String,
    color: Option<&ColorChoice>,
    raw_children: &OrderedRawChildren,
) -> Result<()> {
    let mut start = BytesStart::new(tag);
    start.push_attribute(("idx", index.as_str()));
    if color.is_none() && raw_children.is_empty() {
        writer
            .write_event(Event::Empty(start))
            .map_err(OxmlError::from)?;
        return Ok(());
    }
    writer
        .write_event(Event::Start(start))
        .map_err(OxmlError::from)?;
    emit_raw(writer, raw_children.at(0))?;
    if let Some(color) = color {
        color.to_xml(writer)?;
    }
    emit_raw(writer, raw_children.at(1))?;
    writer
        .write_event(Event::End(BytesEnd::new(tag)))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn is_modelled_color(name: &[u8]) -> bool {
    matches!(
        local_name(name),
        b"srgbClr" | b"schemeClr" | b"sysClr" | b"prstClr"
    )
}

fn is_any_color(name: &[u8]) -> bool {
    matches!(
        local_name(name),
        b"scrgbClr" | b"srgbClr" | b"hslClr" | b"sysClr" | b"schemeClr" | b"prstClr"
    )
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

fn invalid_attribute(element: &str, attribute: &str, value: String) -> StyleReferenceError {
    StyleReferenceError::InvalidAttribute {
        element: element.to_owned(),
        attribute: attribute.to_owned(),
        value,
    }
}

fn missing_end(element: &str) -> StyleReferenceError {
    StyleReferenceError::Xml(OxmlError::MissingElement(format!(
        "closing DrawingML {element}"
    )))
}

#[cfg(test)]
mod tests {
    use std::panic;

    use super::{FillStyleSelection, StyleReference};
    use crate::shape_props::CT_ShapeProperties;

    #[test]
    fn fill_ref_1001_resolves_to_background_fill_style_1() {
        let reference = StyleReference::from_xml(
            br#"<q:fillRef idx="1001"><q:schemeClr val="phClr"/></q:fillRef>"#,
        )
        .unwrap();
        assert_eq!(
            reference.fill_style_selection().unwrap(),
            FillStyleSelection::BackgroundFillStyle(1)
        );
    }

    #[test]
    fn all_four_style_reference_forms_round_trip() {
        let cases: &[(&[u8], &[u8])] = &[
            (
                br#"<q:lnRef idx="2"><x:before/><q:schemeClr val="accent1"/><x:after/></q:lnRef>"#,
                br#"<a:lnRef idx="2"><x:before/><a:schemeClr val="accent1"/><x:after/></a:lnRef>"#,
            ),
            (
                br#"<q:fillRef idx="1001"><q:srgbClr val="102030"/></q:fillRef>"#,
                br#"<a:fillRef idx="1001"><a:srgbClr val="102030"/></a:fillRef>"#,
            ),
            (
                br#"<q:effectRef idx="3"><q:prstClr val="black"/></q:effectRef>"#,
                br#"<a:effectRef idx="3"><a:prstClr val="black"/></a:effectRef>"#,
            ),
            (
                br#"<q:fontRef idx="minor"><q:sysClr val="windowText" lastClr="000000"/></q:fontRef>"#,
                br#"<a:fontRef idx="minor"><a:sysClr val="windowText" lastClr="000000"/></a:fontRef>"#,
            ),
        ];

        for (xml, expected) in cases {
            let parsed = StyleReference::from_xml(xml).unwrap();
            let written = parsed.to_xml().unwrap();
            assert_eq!(&written, expected);
            assert_eq!(StyleReference::from_xml(&written).unwrap(), parsed);
        }

        let raw_color = br#"<q:fillRef idx="4"><q:hslClr hue="0" sat="0" lum="0"><x:kept/></q:hslClr></q:fillRef>"#;
        assert_eq!(
            StyleReference::from_xml(raw_color).unwrap().to_xml().unwrap(),
            br#"<a:fillRef idx="4"><q:hslClr hue="0" sat="0" lum="0"><x:kept/></q:hslClr></a:fillRef>"#
        );
    }

    #[test]
    fn zero_indices_and_colourless_style_references_round_trip() {
        let cases: &[(&[u8], &[u8])] = &[
            (br#"<q:lnRef idx="0"/>"#, br#"<a:lnRef idx="0"/>"#),
            (br#"<q:fillRef idx="0"/>"#, br#"<a:fillRef idx="0"/>"#),
            (br#"<q:effectRef idx="0"/>"#, br#"<a:effectRef idx="0"/>"#),
            (
                br#"<q:fontRef idx="minor"/>"#,
                br#"<a:fontRef idx="minor"/>"#,
            ),
        ];

        for (xml, expected) in cases {
            let parsed = StyleReference::from_xml(xml).unwrap();
            let written = parsed.to_xml().unwrap();
            assert_eq!(&written, expected);
            assert_eq!(StyleReference::from_xml(&written).unwrap(), parsed);
        }

        let fill = StyleReference::from_xml(cases[1].0).unwrap();
        assert_eq!(
            fill.fill_style_selection().unwrap(),
            FillStyleSelection::FillStyle(0)
        );
    }

    #[test]
    fn malformed_shape_and_style_references_return_errors_without_panicking() {
        let style_cases: &[&[u8]] = &[
            br#"<q:otherRef idx="1"><q:schemeClr val="accent1"/></q:otherRef>"#,
            br#"<q:lnRef><q:schemeClr val="accent1"/></q:lnRef>"#,
            br#"<q:fillRef idx="4294967296"><q:schemeClr val="accent1"/></q:fillRef>"#,
            br#"<q:effectRef idx="wide"><q:schemeClr val="accent1"/></q:effectRef>"#,
            br#"<q:fontRef idx="body"><q:schemeClr val="accent1"/></q:fontRef>"#,
            br#"<q:lnRef idx="1"><q:srgbClr val="not-rgb"/></q:lnRef>"#,
        ];
        for xml in style_cases {
            let result = panic::catch_unwind(|| StyleReference::from_xml(xml));
            assert!(result.is_ok(), "style-reference parser panicked");
            assert!(result.unwrap().is_err(), "malformed style reference parsed");
        }

        let shape_cases: &[&[u8]] = &[
            br#"<q:notSpPr/>"#,
            br#"<q:spPr><q:xfrm rot="bad"/></q:spPr>"#,
            br#"<q:spPr><q:custGeom/></q:spPr>"#,
            br#"<q:spPr><q:ln w="20116801"/></q:spPr>"#,
            br#"<q:spPr><q:solidFill><q:srgbClr val="not-rgb"/></q:solidFill></q:spPr>"#,
        ];
        for xml in shape_cases {
            let result = panic::catch_unwind(|| CT_ShapeProperties::from_xml(xml));
            assert!(result.is_ok(), "shape-properties parser panicked");
            assert!(
                result.unwrap().is_err(),
                "malformed shape properties parsed"
            );
        }
    }
}
