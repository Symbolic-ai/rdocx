#![allow(non_camel_case_types)]

use std::fmt;
use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_core::xml::{local_name, matches_local_name};
use oxml_core::xml_text::{decode_plain, resolve_entity};
use oxml_drawing::order::OrderedRawChildren;
use oxml_drawing::shape_props::{CT_ShapeProperties, ShapePropertiesError};
use oxml_drawing::text::{CT_TextBody, TextError};
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer, XmlVersion};

/// The transitional ChartML namespace used by PresentationML packages.
pub const C_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/chart";
/// The fixed ChartML prefix used by writers in this crate.
pub const C_PREFIX: &str = "c";
/// The DrawingML namespace written on a chart-space root.
pub const A_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
/// The office-document relationships namespace written on a chart-space root.
pub const R_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// Errors produced while reading or writing the implemented ChartML core.
#[derive(Debug)]
pub enum ChartError {
    Xml(OxmlError),
    ShapeProperties(ShapePropertiesError),
    Text(TextError),
    UnexpectedElement(String),
    MissingElement(String),
    DuplicateElement(String),
    InvalidAttribute {
        element: String,
        attribute: String,
        value: String,
    },
    InvalidValue {
        element: String,
        value: String,
    },
}

impl fmt::Display for ChartError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml(error) => error.fmt(formatter),
            Self::ShapeProperties(error) => error.fmt(formatter),
            Self::Text(error) => error.fmt(formatter),
            Self::UnexpectedElement(element) => {
                write!(formatter, "unexpected ChartML element: {element}")
            }
            Self::MissingElement(element) => {
                write!(formatter, "ChartML requires {element}")
            }
            Self::DuplicateElement(element) => {
                write!(formatter, "ChartML contains duplicate {element}")
            }
            Self::InvalidAttribute {
                element,
                attribute,
                value,
            } => write!(
                formatter,
                "ChartML {element} has invalid @{attribute}: {value}"
            ),
            Self::InvalidValue { element, value } => {
                write!(formatter, "ChartML {element} has invalid value: {value}")
            }
        }
    }
}

impl std::error::Error for ChartError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml(error) => Some(error),
            Self::ShapeProperties(error) => Some(error),
            Self::Text(error) => Some(error),
            _ => None,
        }
    }
}

impl From<OxmlError> for ChartError {
    fn from(error: OxmlError) -> Self {
        Self::Xml(error)
    }
}

impl From<ShapePropertiesError> for ChartError {
    fn from(error: ShapePropertiesError) -> Self {
        Self::ShapeProperties(error)
    }
}

impl From<TextError> for ChartError {
    fn from(error: TextError) -> Self {
        Self::Text(error)
    }
}

pub type Result<T> = std::result::Result<T, ChartError>;
type NamespaceBindings = Vec<(Vec<u8>, String)>;
type XmlAttributes = Vec<(String, String)>;
type RootAttributes = (XmlAttributes, XmlAttributes);

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ScalarMarkup {
    raw_attributes: XmlAttributes,
    raw_content: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct TextMarkup {
    raw_attributes: XmlAttributes,
    raw_children: OrderedRawChildren,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct PointMarkup {
    raw_attributes: XmlAttributes,
    raw_children: OrderedRawChildren,
    value: TextMarkup,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ReferenceMarkup {
    raw_attributes: XmlAttributes,
    raw_children: OrderedRawChildren,
    formula: TextMarkup,
    cache_attributes: XmlAttributes,
    cache_children: OrderedRawChildren,
    format_code: Option<TextMarkup>,
    point_count: ScalarMarkup,
    declared_point_count: Option<u32>,
    point_indexes: Vec<u32>,
    points: Vec<PointMarkup>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct WrapperMarkup {
    raw_attributes: XmlAttributes,
    raw_children: OrderedRawChildren,
}

/// A formula-backed string cache whose count and point indexes are derived.
#[derive(Clone, Debug, PartialEq)]
pub struct StringRef {
    pub formula: String,
    pub values: Vec<String>,
    markup: ReferenceMarkup,
}

impl StringRef {
    pub fn new(formula: String, values: Vec<String>) -> Result<Self> {
        validate_formula(&formula, "c:strRef/c:f")?;
        validate_point_count(values.len(), "c:strCache")?;
        Ok(Self {
            formula,
            values,
            markup: ReferenceMarkup::default(),
        })
    }

    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        Self::from_xml_with_namespaces(xml, &chart_namespace_defaults())
    }

    fn from_xml_with_namespaces(xml: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let parsed = parse_reference(xml, b"strRef", b"strCache", false, inherited)?;
        let values = parsed.values;
        Ok(Self {
            formula: parsed.formula,
            values,
            markup: parsed.markup,
        })
    }

    pub fn to_xml(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut writer = Writer::new(Vec::new());
        self.write_xml(&mut writer, true)?;
        Ok(writer.into_inner())
    }

    fn validate(&self) -> Result<()> {
        validate_formula(&self.formula, "c:strRef/c:f")?;
        validate_point_count(self.values.len(), "c:strCache")
    }

    fn write_xml(&self, writer: &mut Writer<Vec<u8>>, standalone: bool) -> Result<()> {
        self.validate()?;
        let mut start = BytesStart::new("c:strRef");
        if standalone {
            start.push_attribute(("xmlns:c", C_NS));
        }
        push_attributes(&mut start, &self.markup.raw_attributes);
        writer
            .write_event(Event::Start(start))
            .map_err(OxmlError::from)?;
        emit_raw(writer, self.markup.raw_children.at(0))?;
        write_text(writer, "c:f", &self.formula, &self.markup.formula)?;
        emit_raw(writer, self.markup.raw_children.at(1))?;
        write_string_cache(writer, &self.values, &self.markup)?;
        emit_raw(writer, self.markup.raw_children.at(2))?;
        writer
            .write_event(Event::End(BytesEnd::new("c:strRef")))
            .map_err(OxmlError::from)?;
        Ok(())
    }
}

/// A formula-backed numeric cache whose metadata and points are derived.
#[derive(Clone, Debug, PartialEq)]
pub struct NumericData {
    pub formula: String,
    pub format_code: String,
    pub values: Vec<f64>,
    markup: ReferenceMarkup,
}

impl NumericData {
    pub fn new(formula: String, format_code: String, values: Vec<f64>) -> Result<Self> {
        validate_formula(&formula, "c:numRef/c:f")?;
        validate_format_code(&format_code)?;
        validate_numeric_values(&values)?;
        validate_point_count(values.len(), "c:numCache")?;
        Ok(Self {
            formula,
            format_code,
            values,
            markup: ReferenceMarkup::default(),
        })
    }

    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        Self::from_xml_with_namespaces(xml, &chart_namespace_defaults())
    }

    fn from_xml_with_namespaces(xml: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let parsed = parse_reference(xml, b"numRef", b"numCache", true, inherited)?;
        let format_code = parsed
            .format_code
            .ok_or_else(|| ChartError::MissingElement("c:formatCode".to_owned()))?;
        let mut values = Vec::with_capacity(parsed.values.len());
        for value in parsed.values {
            let number = value.parse::<f64>().map_err(|_| ChartError::InvalidValue {
                element: "c:v".to_owned(),
                value: value.clone(),
            })?;
            if !number.is_finite() {
                return Err(ChartError::InvalidValue {
                    element: "c:v".to_owned(),
                    value,
                });
            }
            values.push(number);
        }
        let data = Self {
            formula: parsed.formula,
            format_code,
            values,
            markup: parsed.markup,
        };
        data.validate()?;
        Ok(data)
    }

    pub fn to_xml(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut writer = Writer::new(Vec::new());
        self.write_xml(&mut writer, true)?;
        Ok(writer.into_inner())
    }

    fn validate(&self) -> Result<()> {
        validate_formula(&self.formula, "c:numRef/c:f")?;
        validate_format_code(&self.format_code)?;
        validate_numeric_values(&self.values)?;
        validate_point_count(self.values.len(), "c:numCache")
    }

    fn write_xml(&self, writer: &mut Writer<Vec<u8>>, standalone: bool) -> Result<()> {
        self.validate()?;
        let mut start = BytesStart::new("c:numRef");
        if standalone {
            start.push_attribute(("xmlns:c", C_NS));
        }
        push_attributes(&mut start, &self.markup.raw_attributes);
        writer
            .write_event(Event::Start(start))
            .map_err(OxmlError::from)?;
        emit_raw(writer, self.markup.raw_children.at(0))?;
        write_text(writer, "c:f", &self.formula, &self.markup.formula)?;
        emit_raw(writer, self.markup.raw_children.at(1))?;
        write_numeric_cache(writer, self, &self.markup)?;
        emit_raw(writer, self.markup.raw_children.at(2))?;
        writer
            .write_event(Event::End(BytesEnd::new("c:numRef")))
            .map_err(OxmlError::from)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AxisData {
    String(StringRef),
    Numeric(NumericData),
}

/// The common formula-backed payload of one ChartML series.
#[derive(Clone, Debug, PartialEq)]
pub struct Series {
    pub index: u32,
    pub order: u32,
    pub name: Option<StringRef>,
    pub categories: Option<AxisData>,
    pub values: NumericData,
    pub bubble_size: Option<NumericData>,
    pub sp_pr: Option<CT_ShapeProperties>,
    index_markup: ScalarMarkup,
    order_markup: ScalarMarkup,
    name_markup: Option<WrapperMarkup>,
    categories_markup: Option<WrapperMarkup>,
    values_markup: WrapperMarkup,
    bubble_size_markup: Option<WrapperMarkup>,
    opaque_name: bool,
    opaque_categories: bool,
    opaque_bubble_size: bool,
    raw_attributes: XmlAttributes,
    namespace_declarations: XmlAttributes,
    raw_children: OrderedRawChildren,
}

impl Series {
    pub fn new(index: u32, order: u32, values: NumericData) -> Self {
        Self {
            index,
            order,
            name: None,
            categories: None,
            values,
            bubble_size: None,
            sp_pr: None,
            index_markup: ScalarMarkup::default(),
            order_markup: ScalarMarkup::default(),
            name_markup: None,
            categories_markup: None,
            values_markup: WrapperMarkup::default(),
            bubble_size_markup: None,
            opaque_name: false,
            opaque_categories: false,
            opaque_bubble_size: false,
            raw_attributes: Vec::new(),
            namespace_declarations: Vec::new(),
            raw_children: OrderedRawChildren::default(),
        }
    }

    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        Self::from_xml_with_namespaces(xml, &chart_namespace_defaults())
    }

    fn from_xml_with_namespaces(xml: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) if matches_local_name(element.name().as_ref(), b"ser") => {
                    chart_root_prefix(&element)?;
                    if !element_is_in_namespace(&element, C_NS, inherited)? {
                        return Err(ChartError::UnexpectedElement(element_name(&element)));
                    }
                    return Self::from_element(&mut reader, &element, inherited);
                }
                Event::Empty(element) if matches_local_name(element.name().as_ref(), b"ser") => {
                    return Err(ChartError::MissingElement("c:idx".to_owned()));
                }
                Event::Start(element) | Event::Empty(element) => {
                    return Err(ChartError::UnexpectedElement(element_name(&element)));
                }
                Event::Eof => return Err(ChartError::MissingElement("c:ser".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_element(
        reader: &mut Reader<&[u8]>,
        start: &BytesStart<'_>,
        inherited: &NamespaceBindings,
    ) -> Result<Self> {
        reject_conflicting_prefix(start, b"a", A_NS)?;
        let namespaces = chart_bindings(inherited, start)?;
        require_fixed_namespace(&namespaces, b"c", C_NS, start)?;
        require_fixed_namespace(&namespaces, b"a", A_NS, start)?;
        require_fixed_namespace(&namespaces, b"r", R_NS, start)?;
        let (raw_attributes, _) =
            capture_fixed_root_attributes(start, &["xmlns:c", "xmlns:a", "xmlns:r"])?;
        let namespace_declarations = standalone_namespace_declarations(&namespaces)?;
        let mut state = SeriesParseState::default();
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) => {
                    let name = chart_child_local(&element, &namespaces)?;
                    let raw = capture_element(reader, &element)?;
                    state.parse_child(name.as_deref().unwrap_or_default(), raw, &namespaces)?;
                }
                Event::Empty(element) => {
                    let name = chart_child_local(&element, &namespaces)?;
                    let raw = capture_empty_element(&element)?;
                    state.parse_child(name.as_deref().unwrap_or_default(), raw, &namespaces)?;
                }
                event @ (Event::Text(_)
                | Event::CData(_)
                | Event::Comment(_)
                | Event::PI(_)
                | Event::GeneralRef(_)) => state.capture_event(capture_event(event)?),
                Event::End(element) if matches_local_name(element.name().as_ref(), b"ser") => {
                    break;
                }
                Event::Eof => return Err(missing_end("c:ser")),
                _ => {}
            }
            buffer.clear();
        }
        state.finish(raw_attributes, namespace_declarations)
    }

    pub fn to_xml(&self) -> Result<Vec<u8>> {
        if self.opaque_name && self.name.is_some() {
            return Err(ChartError::DuplicateElement("c:tx".to_owned()));
        }
        if self.opaque_categories && self.categories.is_some() {
            return Err(ChartError::DuplicateElement("c:cat".to_owned()));
        }
        if self.opaque_bubble_size && self.bubble_size.is_some() {
            return Err(ChartError::DuplicateElement("c:bubbleSize".to_owned()));
        }
        self.values.validate()?;
        if let Some(name) = &self.name {
            name.validate()?;
        }
        if let Some(categories) = &self.categories {
            match categories {
                AxisData::String(reference) => reference.validate()?,
                AxisData::Numeric(reference) => reference.validate()?,
            }
        }
        if let Some(size) = &self.bubble_size {
            size.validate()?;
        }
        let mut writer = Writer::new(Vec::new());
        let mut start = BytesStart::new("c:ser");
        start.push_attribute(("xmlns:c", C_NS));
        start.push_attribute(("xmlns:a", A_NS));
        start.push_attribute(("xmlns:r", R_NS));
        push_attributes(&mut start, &self.namespace_declarations);
        push_attributes(&mut start, &self.raw_attributes);
        writer
            .write_event(Event::Start(start))
            .map_err(OxmlError::from)?;
        emit_raw(&mut writer, self.raw_children.at(0))?;
        write_scalar(
            &mut writer,
            "c:idx",
            &self.index.to_string(),
            Some(&self.index_markup),
        )?;
        emit_raw(&mut writer, self.raw_children.at(1))?;
        write_scalar(
            &mut writer,
            "c:order",
            &self.order.to_string(),
            Some(&self.order_markup),
        )?;
        emit_raw(&mut writer, self.raw_children.at(2))?;
        if let Some(name) = &self.name {
            write_wrapper_start(
                &mut writer,
                "c:tx",
                self.name_markup
                    .as_ref()
                    .unwrap_or(&WrapperMarkup::default()),
            )?;
            name.write_xml(&mut writer, false)?;
            write_wrapper_end(
                &mut writer,
                "c:tx",
                self.name_markup
                    .as_ref()
                    .unwrap_or(&WrapperMarkup::default()),
            )?;
        }
        emit_raw(&mut writer, self.raw_children.at(3))?;
        if let Some(properties) = &self.sp_pr {
            properties.write_xml_as(&mut writer, "c:spPr")?;
        }
        emit_raw(&mut writer, self.raw_children.at(4))?;
        if let Some(categories) = &self.categories {
            let default_markup = WrapperMarkup::default();
            let markup = self.categories_markup.as_ref().unwrap_or(&default_markup);
            write_wrapper_start(&mut writer, "c:cat", markup)?;
            match categories {
                AxisData::String(reference) => reference.write_xml(&mut writer, false)?,
                AxisData::Numeric(reference) => reference.write_xml(&mut writer, false)?,
            }
            write_wrapper_end(&mut writer, "c:cat", markup)?;
        }
        emit_raw(&mut writer, self.raw_children.at(5))?;
        write_wrapper_start(&mut writer, "c:val", &self.values_markup)?;
        self.values.write_xml(&mut writer, false)?;
        write_wrapper_end(&mut writer, "c:val", &self.values_markup)?;
        emit_raw(&mut writer, self.raw_children.at(6))?;
        if let Some(size) = &self.bubble_size {
            let default_markup = WrapperMarkup::default();
            let markup = self.bubble_size_markup.as_ref().unwrap_or(&default_markup);
            write_wrapper_start(&mut writer, "c:bubbleSize", markup)?;
            size.write_xml(&mut writer, false)?;
            write_wrapper_end(&mut writer, "c:bubbleSize", markup)?;
        }
        emit_raw(&mut writer, self.raw_children.at(7))?;
        writer
            .write_event(Event::End(BytesEnd::new("c:ser")))
            .map_err(OxmlError::from)?;
        Ok(writer.into_inner())
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }
}

#[derive(Default)]
struct SeriesParseState {
    index: Option<(u32, ScalarMarkup)>,
    order: Option<(u32, ScalarMarkup)>,
    name: Option<(StringRef, WrapperMarkup)>,
    categories: Option<(AxisData, WrapperMarkup)>,
    values: Option<(NumericData, WrapperMarkup)>,
    bubble_size: Option<(NumericData, WrapperMarkup)>,
    name_seen: bool,
    categories_seen: bool,
    bubble_size_seen: bool,
    opaque_name: bool,
    opaque_categories: bool,
    opaque_bubble_size: bool,
    sp_pr: Option<CT_ShapeProperties>,
    raw_children: OrderedRawChildren,
    boundary: usize,
}

impl SeriesParseState {
    fn capture_event(&mut self, raw: Vec<u8>) {
        self.raw_children.push(self.boundary, raw);
    }

    fn parse_child(
        &mut self,
        name: &[u8],
        raw: Vec<u8>,
        namespaces: &NamespaceBindings,
    ) -> Result<()> {
        match name {
            b"idx" => {
                set_once(&mut self.index, parse_u32_scalar(&raw, "idx")?, "c:idx")?;
                self.boundary = self.boundary.max(1);
            }
            b"order" => {
                set_once(&mut self.order, parse_u32_scalar(&raw, "order")?, "c:order")?;
                self.boundary = self.boundary.max(2);
            }
            b"tx" => {
                mark_once(&mut self.name_seen, "c:tx")?;
                let parsed = parse_wrapper(&raw, b"tx", &[b"strRef"], namespaces)?;
                if let Some((_, reference)) = parsed.choice {
                    self.name = Some((
                        StringRef::from_xml_with_namespaces(&reference, &parsed.namespaces)?,
                        parsed.markup,
                    ));
                } else {
                    self.opaque_name = true;
                    self.raw_children.push(2, raw);
                }
                self.boundary = self.boundary.max(3);
            }
            b"spPr" => {
                reject_conflicting_prefix_in_xml(&raw, b"a", A_NS)?;
                let properties = CT_ShapeProperties::from_xml(&raw)?;
                let mut writer = Writer::new(Vec::new());
                properties.write_xml_as(&mut writer, "c:spPr")?;
                reject_rewritten_foreign_elements(&raw, &writer.into_inner(), namespaces, b"spPr")?;
                set_once(&mut self.sp_pr, properties, "c:spPr")?;
                self.boundary = self.boundary.max(4);
            }
            b"cat" => {
                mark_once(&mut self.categories_seen, "c:cat")?;
                let parsed = parse_wrapper(&raw, b"cat", &[b"strRef", b"numRef"], namespaces)?;
                if let Some((choice, reference)) = parsed.choice {
                    let categories = if choice == b"strRef" {
                        AxisData::String(StringRef::from_xml_with_namespaces(
                            &reference,
                            &parsed.namespaces,
                        )?)
                    } else {
                        AxisData::Numeric(NumericData::from_xml_with_namespaces(
                            &reference,
                            &parsed.namespaces,
                        )?)
                    };
                    self.categories = Some((categories, parsed.markup));
                } else {
                    self.opaque_categories = true;
                    self.raw_children.push(4, raw);
                }
                self.boundary = self.boundary.max(5);
            }
            b"val" => {
                let parsed = parse_wrapper(&raw, b"val", &[b"numRef"], namespaces)?;
                let (_, reference) = parsed
                    .choice
                    .ok_or_else(|| ChartError::MissingElement("c:val/c:numRef".to_owned()))?;
                set_once(
                    &mut self.values,
                    (
                        NumericData::from_xml_with_namespaces(&reference, &parsed.namespaces)?,
                        parsed.markup,
                    ),
                    "c:val",
                )?;
                self.boundary = self.boundary.max(6);
            }
            b"bubbleSize" => {
                mark_once(&mut self.bubble_size_seen, "c:bubbleSize")?;
                let parsed = parse_wrapper(&raw, b"bubbleSize", &[b"numRef"], namespaces)?;
                if let Some((_, reference)) = parsed.choice {
                    self.bubble_size = Some((
                        NumericData::from_xml_with_namespaces(&reference, &parsed.namespaces)?,
                        parsed.markup,
                    ));
                } else {
                    self.opaque_bubble_size = true;
                    self.raw_children.push(6, raw);
                }
                self.boundary = self.boundary.max(7);
            }
            _ => {
                let boundary = series_raw_boundary(name, self.boundary);
                self.raw_children.push(boundary, raw);
                self.boundary = self.boundary.max(boundary);
            }
        }
        Ok(())
    }

    fn finish(
        self,
        raw_attributes: XmlAttributes,
        namespace_declarations: XmlAttributes,
    ) -> Result<Series> {
        let (index, index_markup) = self
            .index
            .ok_or_else(|| ChartError::MissingElement("c:idx".to_owned()))?;
        let (order, order_markup) = self
            .order
            .ok_or_else(|| ChartError::MissingElement("c:order".to_owned()))?;
        let (values, values_markup) = self
            .values
            .ok_or_else(|| ChartError::MissingElement("c:val".to_owned()))?;
        let (name, name_markup) = self
            .name
            .map(|(value, markup)| (Some(value), Some(markup)))
            .unwrap_or((None, None));
        let (categories, categories_markup) = self
            .categories
            .map(|(value, markup)| (Some(value), Some(markup)))
            .unwrap_or((None, None));
        let (bubble_size, bubble_size_markup) = self
            .bubble_size
            .map(|(value, markup)| (Some(value), Some(markup)))
            .unwrap_or((None, None));
        Ok(Series {
            index,
            order,
            name,
            categories,
            values,
            bubble_size,
            sp_pr: self.sp_pr,
            index_markup,
            order_markup,
            name_markup,
            categories_markup,
            values_markup,
            bubble_size_markup,
            opaque_name: self.opaque_name,
            opaque_categories: self.opaque_categories,
            opaque_bubble_size: self.opaque_bubble_size,
            raw_attributes,
            namespace_declarations,
            raw_children: raw_children_in_schema_order(&self.raw_children, 7),
        })
    }
}

fn series_raw_boundary(name: &[u8], current: usize) -> usize {
    match name {
        b"marker" | b"invertIfNegative" | b"pictureOptions" | b"explosion" | b"dPt" | b"dLbls"
        | b"trendline" | b"errBars" => 4,
        b"shape" | b"smooth" => 6,
        b"extLst" => 7,
        _ => current,
    }
}

struct ParsedReference {
    formula: String,
    format_code: Option<String>,
    values: Vec<String>,
    markup: ReferenceMarkup,
}

fn parse_reference(
    xml: &[u8],
    reference_local: &[u8],
    cache_local: &[u8],
    numeric: bool,
    inherited: &NamespaceBindings,
) -> Result<ParsedReference> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element)
                if matches_local_name(element.name().as_ref(), reference_local) =>
            {
                let namespaces = typed_rewrite_bindings(&element, inherited)?;
                let raw_attributes = capture_fixed_attributes(&element, &["xmlns:c"])?;
                let mut formula: Option<(String, TextMarkup)> = None;
                let mut cache: Option<(Option<String>, Vec<String>, CacheMarkup)> = None;
                let mut raw_children = OrderedRawChildren::default();
                let mut boundary = 0usize;
                let mut inner = Vec::new();
                loop {
                    match reader
                        .read_event_into(&mut inner)
                        .map_err(OxmlError::from)?
                    {
                        Event::Start(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_element(&mut reader, &child)?;
                            match name.as_deref().unwrap_or_default() {
                                b"f" => {
                                    set_once(
                                        &mut formula,
                                        parse_text_element(&raw, b"f", &namespaces)?,
                                        "c:f",
                                    )?;
                                    boundary = boundary.max(1);
                                }
                                name if name == cache_local => {
                                    set_once(
                                        &mut cache,
                                        parse_cache(&raw, cache_local, numeric, &namespaces)?,
                                        &format!("c:{}", String::from_utf8_lossy(cache_local)),
                                    )?;
                                    boundary = boundary.max(2);
                                }
                                _ => raw_children.push(boundary, raw),
                            }
                        }
                        Event::Empty(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_empty_element(&child)?;
                            match name.as_deref().unwrap_or_default() {
                                b"f" => {
                                    set_once(
                                        &mut formula,
                                        parse_text_element(&raw, b"f", &namespaces)?,
                                        "c:f",
                                    )?;
                                    boundary = boundary.max(1);
                                }
                                name if name == cache_local => {
                                    set_once(
                                        &mut cache,
                                        parse_cache(&raw, cache_local, numeric, &namespaces)?,
                                        &format!("c:{}", String::from_utf8_lossy(cache_local)),
                                    )?;
                                    boundary = boundary.max(2);
                                }
                                _ => raw_children.push(boundary, raw),
                            }
                        }
                        event @ (Event::Text(_)
                        | Event::CData(_)
                        | Event::Comment(_)
                        | Event::PI(_)
                        | Event::GeneralRef(_)) => {
                            raw_children.push(boundary, capture_event(event)?);
                        }
                        Event::End(end)
                            if matches_local_name(end.name().as_ref(), reference_local) =>
                        {
                            let (formula, formula_markup) = formula
                                .ok_or_else(|| ChartError::MissingElement("c:f".to_owned()))?;
                            validate_formula(&formula, "c:f")?;
                            let (format_code, values, cache_markup) = cache.ok_or_else(|| {
                                ChartError::MissingElement(format!(
                                    "c:{}",
                                    String::from_utf8_lossy(cache_local)
                                ))
                            })?;
                            let markup = ReferenceMarkup {
                                raw_attributes,
                                raw_children: raw_children_in_schema_order(&raw_children, 2),
                                formula: formula_markup,
                                cache_attributes: cache_markup.raw_attributes,
                                cache_children: cache_markup.raw_children,
                                format_code: cache_markup.format_code,
                                point_count: cache_markup.point_count,
                                declared_point_count: cache_markup.declared_point_count,
                                point_indexes: cache_markup.point_indexes,
                                points: cache_markup.points,
                            };
                            return Ok(ParsedReference {
                                formula,
                                format_code,
                                values,
                                markup,
                            });
                        }
                        Event::Eof => {
                            return Err(missing_end(&format!(
                                "c:{}",
                                String::from_utf8_lossy(reference_local)
                            )));
                        }
                        _ => {}
                    }
                    inner.clear();
                }
            }
            Event::Empty(element)
                if matches_local_name(element.name().as_ref(), reference_local) =>
            {
                return Err(ChartError::MissingElement("c:f".to_owned()));
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => {
                return Err(ChartError::MissingElement(format!(
                    "c:{}",
                    String::from_utf8_lossy(reference_local)
                )));
            }
            _ => {}
        }
        buffer.clear();
    }
}

struct CacheMarkup {
    raw_attributes: XmlAttributes,
    raw_children: OrderedRawChildren,
    format_code: Option<TextMarkup>,
    point_count: ScalarMarkup,
    declared_point_count: Option<u32>,
    point_indexes: Vec<u32>,
    points: Vec<PointMarkup>,
}

fn parse_cache(
    xml: &[u8],
    cache_local: &[u8],
    numeric: bool,
    inherited: &NamespaceBindings,
) -> Result<(Option<String>, Vec<String>, CacheMarkup)> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) if matches_local_name(element.name().as_ref(), cache_local) => {
                let namespaces = typed_rewrite_bindings(&element, inherited)?;
                let raw_attributes = capture_attributes(&element)?;
                let mut state = CacheParseState::new(numeric, namespaces);
                let mut inner = Vec::new();
                loop {
                    match reader
                        .read_event_into(&mut inner)
                        .map_err(OxmlError::from)?
                    {
                        Event::Start(child) => {
                            let name = chart_child_local(&child, &state.namespaces)?;
                            let raw = capture_element(&mut reader, &child)?;
                            state.parse_child(name.as_deref().unwrap_or_default(), raw)?;
                        }
                        Event::Empty(child) => {
                            let name = chart_child_local(&child, &state.namespaces)?;
                            let raw = capture_empty_element(&child)?;
                            state.parse_child(name.as_deref().unwrap_or_default(), raw)?;
                        }
                        event @ (Event::Text(_)
                        | Event::CData(_)
                        | Event::Comment(_)
                        | Event::PI(_)
                        | Event::GeneralRef(_)) => {
                            state.capture_event(capture_event(event)?);
                        }
                        Event::End(end) if matches_local_name(end.name().as_ref(), cache_local) => {
                            return state.finish(raw_attributes);
                        }
                        Event::Eof => {
                            return Err(missing_end(&format!(
                                "c:{}",
                                String::from_utf8_lossy(cache_local)
                            )));
                        }
                        _ => {}
                    }
                    inner.clear();
                }
            }
            Event::Empty(element) if matches_local_name(element.name().as_ref(), cache_local) => {
                return Err(ChartError::MissingElement("c:ptCount".to_owned()));
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => {
                return Err(ChartError::MissingElement(format!(
                    "c:{}",
                    String::from_utf8_lossy(cache_local)
                )));
            }
            _ => {}
        }
        buffer.clear();
    }
}

struct CacheParseState {
    numeric: bool,
    namespaces: NamespaceBindings,
    point_base: usize,
    boundary: usize,
    format_code: Option<(String, TextMarkup)>,
    point_count: Option<(u32, ScalarMarkup)>,
    points: Vec<(u32, String, PointMarkup)>,
    raw_children: OrderedRawChildren,
}

impl CacheParseState {
    fn new(numeric: bool, namespaces: NamespaceBindings) -> Self {
        Self {
            numeric,
            namespaces,
            point_base: usize::from(numeric) + 1,
            boundary: 0,
            format_code: None,
            point_count: None,
            points: Vec::new(),
            raw_children: OrderedRawChildren::default(),
        }
    }

    fn capture_event(&mut self, raw: Vec<u8>) {
        self.raw_children.push(self.boundary, raw);
    }

    fn parse_child(&mut self, name: &[u8], raw: Vec<u8>) -> Result<()> {
        match name {
            b"formatCode" if self.numeric => {
                set_once(
                    &mut self.format_code,
                    parse_text_element(&raw, b"formatCode", &self.namespaces)?,
                    "c:formatCode",
                )?;
                self.boundary = self.boundary.max(1);
            }
            b"ptCount" => {
                set_once(
                    &mut self.point_count,
                    parse_u32_scalar(&raw, "ptCount")?,
                    "c:ptCount",
                )?;
                self.boundary = self.boundary.max(self.point_base);
            }
            b"pt" => {
                self.points.push(parse_point(&raw, &self.namespaces)?);
                self.boundary = self.boundary.max(self.point_base + self.points.len());
            }
            _ => self.raw_children.push(self.boundary, raw),
        }
        Ok(())
    }

    fn finish(
        self,
        raw_attributes: XmlAttributes,
    ) -> Result<(Option<String>, Vec<String>, CacheMarkup)> {
        let (declared, point_count_markup) = self
            .point_count
            .ok_or_else(|| ChartError::MissingElement("c:ptCount".to_owned()))?;
        let actual = u32::try_from(self.points.len()).map_err(|_| ChartError::InvalidValue {
            element: "c:ptCount".to_owned(),
            value: self.points.len().to_string(),
        })?;
        if declared < actual {
            return Err(invalid_attribute("ptCount", "val", declared.to_string()));
        }
        let mut values = Vec::with_capacity(self.points.len());
        let mut point_indexes = Vec::with_capacity(self.points.len());
        let mut point_markup = Vec::with_capacity(self.points.len());
        let mut previous = None;
        for (index, value, markup) in self.points {
            if index >= declared || previous.is_some_and(|last| index <= last) {
                return Err(invalid_attribute("pt", "idx", index.to_string()));
            }
            previous = Some(index);
            values.push(value);
            point_indexes.push(index);
            point_markup.push(markup);
        }
        let (format_code, format_markup) = self
            .format_code
            .map(|(value, markup)| (Some(value), Some(markup)))
            .unwrap_or((None, None));
        if self.numeric && format_code.is_none() {
            return Err(ChartError::MissingElement("c:formatCode".to_owned()));
        }
        Ok((
            format_code,
            values,
            CacheMarkup {
                raw_attributes,
                raw_children: raw_children_in_schema_order(
                    &self.raw_children,
                    self.point_base + point_markup.len(),
                ),
                format_code: format_markup,
                point_count: point_count_markup,
                declared_point_count: Some(declared),
                point_indexes,
                points: point_markup,
            },
        ))
    }
}

fn parse_point(xml: &[u8], inherited: &NamespaceBindings) -> Result<(u32, String, PointMarkup)> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) if matches_local_name(element.name().as_ref(), b"pt") => {
                let namespaces = typed_rewrite_bindings(&element, inherited)?;
                let index = required_u32_attribute(&element, "pt", b"idx")?;
                let raw_attributes = capture_attributes_except(&element, b"idx")?;
                let mut value: Option<(String, TextMarkup)> = None;
                let mut raw_children = OrderedRawChildren::default();
                let mut boundary = 0usize;
                let mut inner = Vec::new();
                loop {
                    match reader
                        .read_event_into(&mut inner)
                        .map_err(OxmlError::from)?
                    {
                        Event::Start(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_element(&mut reader, &child)?;
                            if name.as_deref() == Some(b"v") {
                                set_once(
                                    &mut value,
                                    parse_text_element(&raw, b"v", &namespaces)?,
                                    "c:v",
                                )?;
                                boundary = 1;
                            } else {
                                raw_children.push(boundary, raw);
                            }
                        }
                        Event::Empty(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_empty_element(&child)?;
                            if name.as_deref() == Some(b"v") {
                                set_once(
                                    &mut value,
                                    parse_text_element(&raw, b"v", &namespaces)?,
                                    "c:v",
                                )?;
                                boundary = 1;
                            } else {
                                raw_children.push(boundary, raw);
                            }
                        }
                        event @ (Event::Text(_)
                        | Event::CData(_)
                        | Event::Comment(_)
                        | Event::PI(_)
                        | Event::GeneralRef(_)) => {
                            raw_children.push(boundary, capture_event(event)?);
                        }
                        Event::End(end) if matches_local_name(end.name().as_ref(), b"pt") => {
                            let (value, value_markup) = value
                                .ok_or_else(|| ChartError::MissingElement("c:v".to_owned()))?;
                            return Ok((
                                index,
                                value,
                                PointMarkup {
                                    raw_attributes,
                                    raw_children: raw_children_in_schema_order(&raw_children, 1),
                                    value: value_markup,
                                },
                            ));
                        }
                        Event::Eof => return Err(missing_end("c:pt")),
                        _ => {}
                    }
                    inner.clear();
                }
            }
            Event::Empty(element) if matches_local_name(element.name().as_ref(), b"pt") => {
                return Err(ChartError::MissingElement("c:v".to_owned()));
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => return Err(ChartError::MissingElement("c:pt".to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

fn parse_text_element(
    xml: &[u8],
    local: &[u8],
    inherited: &NamespaceBindings,
) -> Result<(String, TextMarkup)> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) if matches_local_name(element.name().as_ref(), local) => {
                typed_rewrite_bindings(&element, inherited)?;
                let raw_attributes = capture_attributes(&element)?;
                let mut value = String::new();
                let mut raw_children = OrderedRawChildren::default();
                let mut seen_text = false;
                let mut inner = Vec::new();
                loop {
                    match reader
                        .read_event_into(&mut inner)
                        .map_err(OxmlError::from)?
                    {
                        Event::Text(text) => {
                            value.push_str(&decode_plain(&text));
                            seen_text = true;
                        }
                        Event::CData(text) => {
                            let decoded =
                                text.decode().map_err(|error| ChartError::InvalidValue {
                                    element: format!("c:{}", String::from_utf8_lossy(local)),
                                    value: error.to_string(),
                                })?;
                            value.push_str(&decoded);
                            seen_text = true;
                        }
                        Event::GeneralRef(reference) => {
                            value.push_str(&resolve_entity(&reference));
                            seen_text = true;
                        }
                        event @ (Event::Comment(_) | Event::PI(_)) => {
                            raw_children.push(usize::from(seen_text), capture_event(event)?);
                        }
                        Event::Start(child) | Event::Empty(child) => {
                            return Err(ChartError::UnexpectedElement(element_name(&child)));
                        }
                        Event::End(end) if matches_local_name(end.name().as_ref(), local) => {
                            return Ok((
                                value,
                                TextMarkup {
                                    raw_attributes,
                                    raw_children: raw_children_in_schema_order(&raw_children, 1),
                                },
                            ));
                        }
                        Event::Eof => {
                            return Err(missing_end(&format!(
                                "c:{}",
                                String::from_utf8_lossy(local)
                            )));
                        }
                        _ => {}
                    }
                    inner.clear();
                }
            }
            Event::Empty(element) if matches_local_name(element.name().as_ref(), local) => {
                typed_rewrite_bindings(&element, inherited)?;
                return Ok((
                    String::new(),
                    TextMarkup {
                        raw_attributes: capture_attributes(&element)?,
                        raw_children: OrderedRawChildren::default(),
                    },
                ));
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => {
                return Err(ChartError::MissingElement(format!(
                    "c:{}",
                    String::from_utf8_lossy(local)
                )));
            }
            _ => {}
        }
        buffer.clear();
    }
}

struct ParsedWrapper {
    choice: Option<(Vec<u8>, Vec<u8>)>,
    markup: WrapperMarkup,
    namespaces: NamespaceBindings,
}

fn parse_wrapper(
    xml: &[u8],
    local: &[u8],
    choices: &[&[u8]],
    inherited: &NamespaceBindings,
) -> Result<ParsedWrapper> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) if matches_local_name(element.name().as_ref(), local) => {
                let namespaces = typed_rewrite_bindings(&element, inherited)?;
                let raw_attributes = capture_attributes(&element)?;
                let mut choice = None;
                let mut raw_children = OrderedRawChildren::default();
                let mut boundary = 0usize;
                let mut inner = Vec::new();
                loop {
                    match reader
                        .read_event_into(&mut inner)
                        .map_err(OxmlError::from)?
                    {
                        Event::Start(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_element(&mut reader, &child)?;
                            if let Some(name) =
                                name.filter(|name| choices.contains(&name.as_slice()))
                            {
                                if choice.is_some() {
                                    return Err(ChartError::DuplicateElement(format!(
                                        "c:{} reference",
                                        String::from_utf8_lossy(local)
                                    )));
                                }
                                choice = Some((name, raw));
                                boundary = 1;
                            } else {
                                raw_children.push(boundary, raw);
                            }
                        }
                        Event::Empty(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_empty_element(&child)?;
                            if let Some(name) =
                                name.filter(|name| choices.contains(&name.as_slice()))
                            {
                                if choice.is_some() {
                                    return Err(ChartError::DuplicateElement(format!(
                                        "c:{} reference",
                                        String::from_utf8_lossy(local)
                                    )));
                                }
                                choice = Some((name, raw));
                                boundary = 1;
                            } else {
                                raw_children.push(boundary, raw);
                            }
                        }
                        event @ (Event::Text(_)
                        | Event::CData(_)
                        | Event::Comment(_)
                        | Event::PI(_)
                        | Event::GeneralRef(_)) => {
                            raw_children.push(boundary, capture_event(event)?);
                        }
                        Event::End(end) if matches_local_name(end.name().as_ref(), local) => {
                            return Ok(ParsedWrapper {
                                choice,
                                markup: WrapperMarkup {
                                    raw_attributes,
                                    raw_children: raw_children_in_schema_order(&raw_children, 1),
                                },
                                namespaces,
                            });
                        }
                        Event::Eof => {
                            return Err(missing_end(&format!(
                                "c:{}",
                                String::from_utf8_lossy(local)
                            )));
                        }
                        _ => {}
                    }
                    inner.clear();
                }
            }
            Event::Empty(element) if matches_local_name(element.name().as_ref(), local) => {
                let namespaces = chart_bindings(inherited, &element)?;
                return Ok(ParsedWrapper {
                    choice: None,
                    markup: WrapperMarkup {
                        raw_attributes: capture_attributes(&element)?,
                        raw_children: OrderedRawChildren::default(),
                    },
                    namespaces,
                });
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => {
                return Err(ChartError::MissingElement(format!(
                    "c:{}",
                    String::from_utf8_lossy(local)
                )));
            }
            _ => {}
        }
        buffer.clear();
    }
}

fn write_string_cache(
    writer: &mut Writer<Vec<u8>>,
    values: &[String],
    markup: &ReferenceMarkup,
) -> Result<()> {
    let (declared_count, indexes) = cache_layout(markup, values.len())?;
    let mut start = BytesStart::new("c:strCache");
    push_attributes(&mut start, &markup.cache_attributes);
    writer
        .write_event(Event::Start(start))
        .map_err(OxmlError::from)?;
    emit_raw(writer, markup.cache_children.at(0))?;
    write_scalar(
        writer,
        "c:ptCount",
        &declared_count.to_string(),
        Some(&markup.point_count),
    )?;
    emit_cache_point_raw(writer, markup, 1, values.len(), 0)?;
    for (position, value) in values.iter().enumerate() {
        let default_markup = PointMarkup::default();
        write_point(
            writer,
            indexes[position],
            value,
            markup.points.get(position).unwrap_or(&default_markup),
        )?;
        emit_cache_point_raw(writer, markup, 1, values.len(), position + 1)?;
    }
    writer
        .write_event(Event::End(BytesEnd::new("c:strCache")))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn write_numeric_cache(
    writer: &mut Writer<Vec<u8>>,
    data: &NumericData,
    markup: &ReferenceMarkup,
) -> Result<()> {
    let (declared_count, indexes) = cache_layout(markup, data.values.len())?;
    let mut start = BytesStart::new("c:numCache");
    push_attributes(&mut start, &markup.cache_attributes);
    writer
        .write_event(Event::Start(start))
        .map_err(OxmlError::from)?;
    emit_raw(writer, markup.cache_children.at(0))?;
    let default_format_markup = TextMarkup::default();
    write_text(
        writer,
        "c:formatCode",
        &data.format_code,
        markup
            .format_code
            .as_ref()
            .unwrap_or(&default_format_markup),
    )?;
    emit_raw(writer, markup.cache_children.at(1))?;
    write_scalar(
        writer,
        "c:ptCount",
        &declared_count.to_string(),
        Some(&markup.point_count),
    )?;
    emit_cache_point_raw(writer, markup, 2, data.values.len(), 0)?;
    for (position, value) in data.values.iter().enumerate() {
        let default_markup = PointMarkup::default();
        write_point(
            writer,
            indexes[position],
            &value.to_string(),
            markup.points.get(position).unwrap_or(&default_markup),
        )?;
        emit_cache_point_raw(writer, markup, 2, data.values.len(), position + 1)?;
    }
    writer
        .write_event(Event::End(BytesEnd::new("c:numCache")))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn emit_cache_point_raw(
    writer: &mut Writer<Vec<u8>>,
    markup: &ReferenceMarkup,
    point_base: usize,
    current_count: usize,
    completed_points: usize,
) -> Result<()> {
    let original_count = markup.points.len();
    if current_count == 0 {
        for boundary in point_base..=point_base + original_count {
            emit_raw(writer, markup.cache_children.at(boundary))?;
        }
    } else if completed_points == 0 {
        if original_count > 0 {
            emit_raw(writer, markup.cache_children.at(point_base))?;
        }
    } else if completed_points < current_count {
        if completed_points < original_count {
            emit_raw(
                writer,
                markup.cache_children.at(point_base + completed_points),
            )?;
        }
    } else {
        let first_tail = if original_count == 0 {
            point_base
        } else {
            point_base + current_count.min(original_count)
        };
        for boundary in first_tail..=point_base + original_count {
            emit_raw(writer, markup.cache_children.at(boundary))?;
        }
    }
    Ok(())
}

fn write_point(
    writer: &mut Writer<Vec<u8>>,
    index: u32,
    value: &str,
    markup: &PointMarkup,
) -> Result<()> {
    let index = index.to_string();
    let mut start = BytesStart::new("c:pt");
    start.push_attribute(("idx", index.as_str()));
    push_attributes(&mut start, &markup.raw_attributes);
    writer
        .write_event(Event::Start(start))
        .map_err(OxmlError::from)?;
    emit_raw(writer, markup.raw_children.at(0))?;
    write_text(writer, "c:v", value, &markup.value)?;
    emit_raw(writer, markup.raw_children.at(1))?;
    writer
        .write_event(Event::End(BytesEnd::new("c:pt")))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn cache_layout(markup: &ReferenceMarkup, value_count: usize) -> Result<(u32, Vec<u32>)> {
    if markup.point_indexes.len() == value_count
        && let Some(declared) = markup.declared_point_count
    {
        return Ok((declared, markup.point_indexes.clone()));
    }
    let declared = u32::try_from(value_count).map_err(|_| ChartError::InvalidValue {
        element: "c:ptCount".to_owned(),
        value: value_count.to_string(),
    })?;
    Ok((declared, (0..declared).collect()))
}

fn write_text(
    writer: &mut Writer<Vec<u8>>,
    tag: &str,
    value: &str,
    markup: &TextMarkup,
) -> Result<()> {
    let mut start = BytesStart::new(tag);
    push_attributes(&mut start, &markup.raw_attributes);
    writer
        .write_event(Event::Start(start))
        .map_err(OxmlError::from)?;
    emit_raw(writer, markup.raw_children.at(0))?;
    writer
        .write_event(Event::Text(BytesText::new(value)))
        .map_err(OxmlError::from)?;
    emit_raw(writer, markup.raw_children.at(1))?;
    writer
        .write_event(Event::End(BytesEnd::new(tag)))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn write_wrapper_start(
    writer: &mut Writer<Vec<u8>>,
    tag: &str,
    markup: &WrapperMarkup,
) -> Result<()> {
    let mut start = BytesStart::new(tag);
    push_attributes(&mut start, &markup.raw_attributes);
    writer
        .write_event(Event::Start(start))
        .map_err(OxmlError::from)?;
    emit_raw(writer, markup.raw_children.at(0))
}

fn write_wrapper_end(
    writer: &mut Writer<Vec<u8>>,
    tag: &str,
    markup: &WrapperMarkup,
) -> Result<()> {
    emit_raw(writer, markup.raw_children.at(1))?;
    writer
        .write_event(Event::End(BytesEnd::new(tag)))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn validate_formula(formula: &str, element: &str) -> Result<()> {
    if formula.trim().is_empty() {
        return Err(ChartError::InvalidValue {
            element: element.to_owned(),
            value: formula.to_owned(),
        });
    }
    Ok(())
}

fn validate_format_code(format_code: &str) -> Result<()> {
    if format_code.is_empty() {
        return Err(ChartError::InvalidValue {
            element: "c:formatCode".to_owned(),
            value: format_code.to_owned(),
        });
    }
    Ok(())
}

fn validate_numeric_values(values: &[f64]) -> Result<()> {
    if let Some(value) = values.iter().find(|value| !value.is_finite()) {
        return Err(ChartError::InvalidValue {
            element: "c:v".to_owned(),
            value: value.to_string(),
        });
    }
    Ok(())
}

fn validate_point_count(count: usize, element: &str) -> Result<()> {
    u32::try_from(count)
        .map(|_| ())
        .map_err(|_| ChartError::InvalidValue {
            element: element.to_owned(),
            value: count.to_string(),
        })
}

fn parse_u32_scalar(xml: &[u8], local: &str) -> Result<(u32, ScalarMarkup)> {
    let (value, markup) = scalar_value(xml, local)?;
    let value = value.ok_or_else(|| invalid_attribute(local, "val", "<missing>".to_owned()))?;
    let parsed = value
        .parse::<u32>()
        .map_err(|_| invalid_attribute(local, "val", value))?;
    Ok((parsed, markup))
}

fn required_u32_attribute(element: &BytesStart<'_>, local: &str, attribute: &[u8]) -> Result<u32> {
    let value = attribute_value(element, attribute)?.ok_or_else(|| {
        invalid_attribute(
            local,
            &String::from_utf8_lossy(attribute),
            "<missing>".to_owned(),
        )
    })?;
    value
        .parse::<u32>()
        .map_err(|_| invalid_attribute(local, &String::from_utf8_lossy(attribute), value))
}

fn chart_root_prefix(element: &BytesStart<'_>) -> Result<Vec<u8>> {
    let name = element.name();
    let qualified = name.as_ref();
    let prefix = qualified
        .iter()
        .position(|byte| *byte == b':')
        .map(|position| qualified[..position].to_vec())
        .unwrap_or_default();
    let declaration = if prefix.is_empty() {
        b"xmlns".to_vec()
    } else {
        let mut declaration = b"xmlns:".to_vec();
        declaration.extend_from_slice(&prefix);
        declaration
    };
    if let Some(namespace) = attribute_value(element, &declaration)?
        && namespace != C_NS
    {
        return Err(invalid_attribute(
            &element_name(element),
            "namespace",
            namespace,
        ));
    }
    reject_conflicting_prefix(element, b"c", C_NS)?;
    Ok(prefix)
}

fn typed_rewrite_bindings(
    element: &BytesStart<'_>,
    inherited: &NamespaceBindings,
) -> Result<NamespaceBindings> {
    chart_root_prefix(element)?;
    if !element_is_in_namespace(element, C_NS, inherited)? {
        return Err(ChartError::UnexpectedElement(element_name(element)));
    }
    let bindings = chart_bindings(inherited, element)?;
    require_fixed_namespace(&bindings, b"c", C_NS, element)?;
    Ok(bindings)
}

fn chart_namespace_defaults() -> NamespaceBindings {
    vec![
        (b"c".to_vec(), C_NS.to_owned()),
        (b"a".to_vec(), A_NS.to_owned()),
        (b"r".to_vec(), R_NS.to_owned()),
    ]
}

fn chart_bindings(
    inherited: &NamespaceBindings,
    element: &BytesStart<'_>,
) -> Result<NamespaceBindings> {
    let mut bindings = chart_namespace_defaults();
    for (prefix, namespace) in inherited
        .iter()
        .cloned()
        .chain(namespace_bindings(element)?)
    {
        upsert_namespace_binding(&mut bindings, prefix, namespace);
    }
    Ok(bindings)
}

fn root_chart_bindings(
    xml: &[u8],
    local: &[u8],
    inherited: &NamespaceBindings,
) -> Result<NamespaceBindings> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) | Event::Empty(element)
                if matches_local_name(element.name().as_ref(), local) =>
            {
                if !element_is_in_namespace(&element, C_NS, inherited)? {
                    return Err(ChartError::UnexpectedElement(element_name(&element)));
                }
                return chart_bindings(inherited, &element);
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => {
                return Err(ChartError::MissingElement(format!(
                    "c:{}",
                    String::from_utf8_lossy(local)
                )));
            }
            _ => {}
        }
        buffer.clear();
    }
}

fn chart_child_local(
    element: &BytesStart<'_>,
    inherited: &NamespaceBindings,
) -> Result<Option<Vec<u8>>> {
    Ok(element_is_in_namespace(element, C_NS, inherited)?
        .then(|| local_name(element.name().as_ref()).to_vec()))
}

fn capture_fixed_root_attributes(
    start: &BytesStart<'_>,
    fixed: &[&str],
) -> Result<(XmlAttributes, XmlAttributes)> {
    let mut attributes = Vec::new();
    let mut namespaces = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        let name = std::str::from_utf8(attribute.key.as_ref())
            .map_err(OxmlError::from)?
            .to_owned();
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())
            .map_err(OxmlError::from)?
            .into_owned();
        if fixed.contains(&name.as_str()) {
            continue;
        }
        if name == "xmlns" || name.starts_with("xmlns:") {
            namespaces.push((name, value));
        } else {
            attributes.push((name, value));
        }
    }
    Ok((attributes, namespaces))
}

fn require_fixed_namespace(
    bindings: &NamespaceBindings,
    prefix: &[u8],
    expected: &str,
    element: &BytesStart<'_>,
) -> Result<()> {
    let actual = bindings
        .iter()
        .find(|(candidate, _)| candidate.as_slice() == prefix)
        .map(|(_, namespace)| namespace.as_str());
    if actual == Some(expected) {
        return Ok(());
    }
    Err(invalid_attribute(
        &element_name(element),
        &format!("xmlns:{}", String::from_utf8_lossy(prefix)),
        actual.unwrap_or("<missing>").to_owned(),
    ))
}

fn standalone_namespace_declarations(bindings: &NamespaceBindings) -> Result<XmlAttributes> {
    bindings
        .iter()
        .filter(|(prefix, _)| prefix != b"c" && prefix != b"a" && prefix != b"r")
        .map(|(prefix, namespace)| {
            let prefix = std::str::from_utf8(prefix).map_err(OxmlError::from)?;
            let name = if prefix.is_empty() {
                "xmlns".to_owned()
            } else {
                format!("xmlns:{prefix}")
            };
            Ok((name, namespace.clone()))
        })
        .collect()
}

fn capture_fixed_attributes(start: &BytesStart<'_>, fixed: &[&str]) -> Result<XmlAttributes> {
    let mut attributes = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        let name = std::str::from_utf8(attribute.key.as_ref())
            .map_err(OxmlError::from)?
            .to_owned();
        if fixed.contains(&name.as_str()) {
            continue;
        }
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())
            .map_err(OxmlError::from)?
            .into_owned();
        attributes.push((name, value));
    }
    Ok(attributes)
}

/// How a chart displays cells whose cached values are blank.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DispBlanksAs {
    #[default]
    Gap,
    Zero,
    Span,
}

impl DispBlanksAs {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "gap" => Some(Self::Gap),
            "zero" => Some(Self::Zero),
            "span" => Some(Self::Span),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Gap => "gap",
            Self::Zero => "zero",
            Self::Span => "span",
        }
    }
}

/// A chart title shell whose current children remain opaque until later stories.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CT_Title {
    raw_attributes: Vec<(String, String)>,
    raw_children: OrderedRawChildren,
}

/// A plot-area shell. F-119 through F-122 replace selected raw slots with types.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CT_PlotArea {
    raw_attributes: Vec<(String, String)>,
    raw_children: OrderedRawChildren,
    namespace_bindings: NamespaceBindings,
}

/// A chart legend shell whose current children remain opaque.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CT_Legend {
    raw_attributes: Vec<(String, String)>,
    raw_children: OrderedRawChildren,
}

impl CT_Title {
    fn from_xml(xml: &[u8]) -> Result<Self> {
        let (raw_attributes, raw_children) = parse_raw_shell(xml, b"title", "c:title")?;
        Ok(Self {
            raw_attributes,
            raw_children,
        })
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        write_raw_shell(writer, "c:title", &self.raw_attributes, &self.raw_children)
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }
}

impl CT_PlotArea {
    fn from_xml_with_namespaces(xml: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let (raw_attributes, raw_children) = parse_raw_shell(xml, b"plotArea", "c:plotArea")?;
        let namespace_bindings = root_chart_bindings(xml, b"plotArea", inherited)?;
        Ok(Self {
            raw_attributes,
            raw_children,
            namespace_bindings,
        })
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        write_raw_shell(
            writer,
            "c:plotArea",
            &self.raw_attributes,
            &self.raw_children,
        )
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }

    /// Parses the common series payloads nested in category-based plot shells.
    pub fn series(&self) -> Result<Vec<Series>> {
        let mut series = Vec::new();
        for raw in self.raw_children.at(0) {
            parse_plot_series(raw, &self.namespace_bindings, &mut series)?;
        }
        Ok(series)
    }
}

fn parse_plot_series(
    xml: &[u8],
    inherited: &NamespaceBindings,
    series: &mut Vec<Series>,
) -> Result<()> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) => {
                if !element_is_in_namespace(&element, C_NS, inherited)? {
                    return Ok(());
                }
                let local = local_name(element.name().as_ref()).to_vec();
                if !is_supported_series_plot(&local) {
                    return Ok(());
                }
                let namespaces = chart_bindings(inherited, &element)?;
                let mut inner = Vec::new();
                loop {
                    match reader
                        .read_event_into(&mut inner)
                        .map_err(OxmlError::from)?
                    {
                        Event::Start(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_element(&mut reader, &child)?;
                            if name.as_deref() == Some(b"ser") {
                                series.push(Series::from_xml_with_namespaces(&raw, &namespaces)?);
                            }
                        }
                        Event::Empty(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            if name.as_deref() == Some(b"ser") {
                                return Err(ChartError::MissingElement("c:ser/c:idx".to_owned()));
                            }
                        }
                        Event::End(end) if matches_local_name(end.name().as_ref(), &local) => {
                            return Ok(());
                        }
                        Event::Eof => {
                            return Err(missing_end(&format!(
                                "c:{}",
                                String::from_utf8_lossy(&local)
                            )));
                        }
                        _ => {}
                    }
                    inner.clear();
                }
            }
            Event::Empty(_) | Event::Eof => return Ok(()),
            _ => {}
        }
        buffer.clear();
    }
}

fn is_supported_series_plot(local: &[u8]) -> bool {
    matches!(
        local,
        b"areaChart"
            | b"area3DChart"
            | b"barChart"
            | b"bar3DChart"
            | b"doughnutChart"
            | b"lineChart"
            | b"line3DChart"
            | b"ofPieChart"
            | b"pieChart"
            | b"pie3DChart"
            | b"radarChart"
            | b"stockChart"
            | b"surfaceChart"
            | b"surface3DChart"
    )
}

impl CT_Legend {
    fn from_xml(xml: &[u8]) -> Result<Self> {
        let (raw_attributes, raw_children) = parse_raw_shell(xml, b"legend", "c:legend")?;
        Ok(Self {
            raw_attributes,
            raw_children,
        })
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        write_raw_shell(writer, "c:legend", &self.raw_attributes, &self.raw_children)
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }
}

fn parse_raw_shell(
    xml: &[u8],
    local: &[u8],
    tag: &str,
) -> Result<(XmlAttributes, OrderedRawChildren)> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) if matches_local_name(element.name().as_ref(), local) => {
                reject_conflicting_prefix(&element, b"c", C_NS)?;
                let raw_attributes = capture_attributes(&element)?;
                let mut raw_children = OrderedRawChildren::default();
                let mut child_buffer = Vec::new();
                loop {
                    match reader
                        .read_event_into(&mut child_buffer)
                        .map_err(OxmlError::from)?
                    {
                        Event::Start(child) => {
                            raw_children.push(0, capture_element(&mut reader, &child)?)
                        }
                        Event::Empty(child) => raw_children.push(0, capture_empty_element(&child)?),
                        event @ (Event::Text(_)
                        | Event::CData(_)
                        | Event::Comment(_)
                        | Event::PI(_)
                        | Event::GeneralRef(_)) => {
                            raw_children.push(0, capture_event(event)?);
                        }
                        Event::End(end) if matches_local_name(end.name().as_ref(), local) => break,
                        Event::Eof => return Err(missing_end(tag)),
                        _ => {}
                    }
                    child_buffer.clear();
                }
                return Ok((raw_attributes, raw_children));
            }
            Event::Empty(element) if matches_local_name(element.name().as_ref(), local) => {
                reject_conflicting_prefix(&element, b"c", C_NS)?;
                return Ok((capture_attributes(&element)?, OrderedRawChildren::default()));
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => return Err(ChartError::MissingElement(tag.to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

fn write_raw_shell<W: Write>(
    writer: &mut Writer<W>,
    tag: &str,
    raw_attributes: &XmlAttributes,
    raw_children: &OrderedRawChildren,
) -> Result<()> {
    let mut start = BytesStart::new(tag);
    push_attributes(&mut start, raw_attributes);
    if raw_children.is_empty() {
        writer
            .write_event(Event::Empty(start))
            .map_err(OxmlError::from)?;
        return Ok(());
    }
    writer
        .write_event(Event::Start(start))
        .map_err(OxmlError::from)?;
    emit_raw(writer, raw_children.at(0))?;
    writer
        .write_event(Event::End(BytesEnd::new(tag)))
        .map_err(OxmlError::from)?;
    Ok(())
}

/// The implemented behavior-bearing core of one `c:chart`.
#[derive(Clone, Debug, PartialEq)]
pub struct CT_Chart {
    pub title: Option<CT_Title>,
    pub auto_title_deleted: bool,
    pub plot_area: CT_PlotArea,
    pub legend: Option<CT_Legend>,
    pub plot_vis_only: bool,
    pub disp_blanks_as: DispBlanksAs,
    auto_title_deleted_markup: Option<ScalarMarkup>,
    plot_vis_only_markup: Option<ScalarMarkup>,
    disp_blanks_as_markup: Option<ScalarMarkup>,
    raw_attributes: Vec<(String, String)>,
    raw_children: OrderedRawChildren,
}

impl CT_Chart {
    fn from_xml_with_namespaces(xml: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) if matches_local_name(element.name().as_ref(), b"chart") => {
                    reject_conflicting_prefix(&element, b"c", C_NS)?;
                    return Self::from_element(&mut reader, &element, inherited);
                }
                Event::Empty(element) if matches_local_name(element.name().as_ref(), b"chart") => {
                    return Err(ChartError::MissingElement("c:plotArea".to_owned()));
                }
                Event::Start(element) | Event::Empty(element) => {
                    return Err(ChartError::UnexpectedElement(element_name(&element)));
                }
                Event::Eof => return Err(ChartError::MissingElement("c:chart".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_element(
        reader: &mut Reader<&[u8]>,
        start: &BytesStart<'_>,
        inherited: &NamespaceBindings,
    ) -> Result<Self> {
        let namespaces = bindings_with_local(inherited, start)?;
        let raw_attributes = capture_attributes(start)?;
        let mut state = ChartParseState::default();
        let mut buffer = Vec::new();

        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) => {
                    let name = if element_is_in_namespace(&element, C_NS, &namespaces)? {
                        local_name(element.name().as_ref()).to_vec()
                    } else {
                        Vec::new()
                    };
                    let raw = capture_element(reader, &element)?;
                    state.parse_child(&name, raw, &namespaces)?;
                }
                Event::Empty(element) => {
                    let name = if element_is_in_namespace(&element, C_NS, &namespaces)? {
                        local_name(element.name().as_ref()).to_vec()
                    } else {
                        Vec::new()
                    };
                    let raw = capture_empty_element(&element)?;
                    state.parse_child(&name, raw, &namespaces)?;
                }
                event @ (Event::Text(_)
                | Event::CData(_)
                | Event::Comment(_)
                | Event::PI(_)
                | Event::GeneralRef(_)) => {
                    state.capture_event(capture_event(event)?);
                }
                Event::End(element) if matches_local_name(element.name().as_ref(), b"chart") => {
                    break;
                }
                Event::Eof => return Err(missing_end("c:chart")),
                _ => {}
            }
            buffer.clear();
        }

        state.finish(raw_attributes)
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut start = BytesStart::new("c:chart");
        push_attributes(&mut start, &self.raw_attributes);
        writer
            .write_event(Event::Start(start))
            .map_err(OxmlError::from)?;
        emit_raw(writer, self.raw_children.at(0))?;
        if let Some(title) = &self.title {
            title.write_xml(writer)?;
        }
        emit_raw(writer, self.raw_children.at(1))?;
        if self.auto_title_deleted_markup.is_some() || self.auto_title_deleted {
            write_scalar(
                writer,
                "c:autoTitleDeleted",
                if self.auto_title_deleted { "1" } else { "0" },
                self.auto_title_deleted_markup.as_ref(),
            )?;
        }
        for raw_boundary in 2..8 {
            emit_raw(writer, self.raw_children.at(raw_boundary))?;
        }
        self.plot_area.write_xml(writer)?;
        emit_raw(writer, self.raw_children.at(8))?;
        if let Some(legend) = &self.legend {
            legend.write_xml(writer)?;
        }
        emit_raw(writer, self.raw_children.at(9))?;
        if self.plot_vis_only_markup.is_some() || !self.plot_vis_only {
            write_scalar(
                writer,
                "c:plotVisOnly",
                if self.plot_vis_only { "1" } else { "0" },
                self.plot_vis_only_markup.as_ref(),
            )?;
        }
        emit_raw(writer, self.raw_children.at(10))?;
        if self.disp_blanks_as_markup.is_some() || self.disp_blanks_as != DispBlanksAs::Gap {
            write_scalar(
                writer,
                "c:dispBlanksAs",
                self.disp_blanks_as.as_str(),
                self.disp_blanks_as_markup.as_ref(),
            )?;
        }
        for raw_boundary in 11..=13 {
            emit_raw(writer, self.raw_children.at(raw_boundary))?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("c:chart")))
            .map_err(OxmlError::from)?;
        Ok(())
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }
}

#[derive(Default)]
struct ChartParseState {
    title: Option<CT_Title>,
    auto_title_deleted: Option<(bool, ScalarMarkup)>,
    plot_area: Option<CT_PlotArea>,
    legend: Option<CT_Legend>,
    plot_vis_only: Option<(bool, ScalarMarkup)>,
    disp_blanks_as: Option<(DispBlanksAs, ScalarMarkup)>,
    raw_children: OrderedRawChildren,
    boundary: usize,
}

impl ChartParseState {
    fn capture_event(&mut self, raw: Vec<u8>) {
        self.raw_children.push(self.boundary, raw);
    }

    fn parse_child(
        &mut self,
        name: &[u8],
        raw: Vec<u8>,
        namespaces: &NamespaceBindings,
    ) -> Result<()> {
        match name {
            b"title" => {
                set_once(&mut self.title, CT_Title::from_xml(&raw)?, "c:title")?;
                self.boundary = self.boundary.max(1);
            }
            b"autoTitleDeleted" => {
                set_once(
                    &mut self.auto_title_deleted,
                    parse_bool_value(&raw, "autoTitleDeleted")?,
                    "c:autoTitleDeleted",
                )?;
                self.boundary = self.boundary.max(2);
            }
            b"plotArea" => {
                set_once(
                    &mut self.plot_area,
                    CT_PlotArea::from_xml_with_namespaces(&raw, namespaces)?,
                    "c:plotArea",
                )?;
                self.boundary = self.boundary.max(8);
            }
            b"legend" => {
                set_once(&mut self.legend, CT_Legend::from_xml(&raw)?, "c:legend")?;
                self.boundary = self.boundary.max(9);
            }
            b"plotVisOnly" => {
                set_once(
                    &mut self.plot_vis_only,
                    parse_bool_value(&raw, "plotVisOnly")?,
                    "c:plotVisOnly",
                )?;
                self.boundary = self.boundary.max(10);
            }
            b"dispBlanksAs" => {
                set_once(
                    &mut self.disp_blanks_as,
                    parse_enum_value(&raw, "dispBlanksAs")?,
                    "c:dispBlanksAs",
                )?;
                self.boundary = self.boundary.max(11);
            }
            _ => {
                self.raw_children
                    .push(chart_raw_slot(name).unwrap_or(self.boundary), raw);
                self.boundary = self.boundary.max(chart_raw_boundary_after(name));
            }
        }
        Ok(())
    }

    fn finish(self, raw_attributes: XmlAttributes) -> Result<CT_Chart> {
        let (auto_title_deleted, auto_title_deleted_markup) = self
            .auto_title_deleted
            .map(|(value, markup)| (value, Some(markup)))
            .unwrap_or((false, None));
        let (plot_vis_only, plot_vis_only_markup) = self
            .plot_vis_only
            .map(|(value, markup)| (value, Some(markup)))
            .unwrap_or((true, None));
        let (disp_blanks_as, disp_blanks_as_markup) = self
            .disp_blanks_as
            .map(|(value, markup)| (value, Some(markup)))
            .unwrap_or((DispBlanksAs::Gap, None));
        Ok(CT_Chart {
            title: self.title,
            auto_title_deleted,
            plot_area: self
                .plot_area
                .ok_or_else(|| ChartError::MissingElement("c:plotArea".to_owned()))?,
            legend: self.legend,
            plot_vis_only,
            disp_blanks_as,
            auto_title_deleted_markup,
            plot_vis_only_markup,
            disp_blanks_as_markup,
            raw_attributes,
            raw_children: raw_children_in_schema_order(&self.raw_children, 13),
        })
    }
}

fn chart_raw_slot(name: &[u8]) -> Option<usize> {
    match name {
        b"pivotFmts" => Some(2),
        b"view3D" => Some(3),
        b"floor" => Some(4),
        b"sideWall" => Some(5),
        b"backWall" => Some(6),
        // Both unmodelled tail children share a slot so producer sibling order
        // remains byte-stable until a later story types either element.
        b"showDLblsOverMax" | b"extLst" => Some(11),
        _ => None,
    }
}

fn chart_raw_boundary_after(name: &[u8]) -> usize {
    match name {
        b"title" => 1,
        b"autoTitleDeleted" => 2,
        b"pivotFmts" => 3,
        b"view3D" => 4,
        b"floor" => 5,
        b"sideWall" => 6,
        b"backWall" => 7,
        b"plotArea" => 8,
        b"legend" => 9,
        b"plotVisOnly" => 10,
        b"dispBlanksAs" => 11,
        b"showDLblsOverMax" | b"extLst" => 11,
        _ => 0,
    }
}

/// One complete `c:chartSpace` part with the F-118 core typed.
#[derive(Clone, Debug, PartialEq)]
pub struct CT_ChartSpace {
    pub chart: CT_Chart,
    pub sp_pr: Option<CT_ShapeProperties>,
    pub tx_pr: Option<CT_TextBody>,
    raw_attributes: Vec<(String, String)>,
    namespace_declarations: Vec<(String, String)>,
    raw_children: OrderedRawChildren,
}

impl CT_ChartSpace {
    /// Parses one complete ChartML chart-space part.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element)
                    if matches_local_name(element.name().as_ref(), b"chartSpace") =>
                {
                    validate_chart_space_namespace(&element)?;
                    return Self::from_element(&mut reader, &element);
                }
                Event::Empty(element)
                    if matches_local_name(element.name().as_ref(), b"chartSpace") =>
                {
                    validate_chart_space_namespace(&element)?;
                    return Err(ChartError::MissingElement("c:chart".to_owned()));
                }
                Event::Start(element) | Event::Empty(element) => {
                    return Err(ChartError::UnexpectedElement(element_name(&element)));
                }
                Event::Eof => return Err(ChartError::MissingElement("c:chartSpace".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_element(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let namespaces = namespace_bindings(start)?;
        let (raw_attributes, namespace_declarations) = capture_root_attributes(start)?;
        let mut state = ChartSpaceParseState::default();
        let mut buffer = Vec::new();

        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) => {
                    let name = if element_is_in_namespace(&element, C_NS, &namespaces)? {
                        local_name(element.name().as_ref()).to_vec()
                    } else {
                        Vec::new()
                    };
                    let raw = capture_element(reader, &element)?;
                    state.parse_child(&name, raw, &namespaces)?;
                }
                Event::Empty(element) => {
                    let name = if element_is_in_namespace(&element, C_NS, &namespaces)? {
                        local_name(element.name().as_ref()).to_vec()
                    } else {
                        Vec::new()
                    };
                    let raw = capture_empty_element(&element)?;
                    state.parse_child(&name, raw, &namespaces)?;
                }
                event @ (Event::Text(_)
                | Event::CData(_)
                | Event::Comment(_)
                | Event::PI(_)
                | Event::GeneralRef(_)) => {
                    state.capture_event(capture_event(event)?);
                }
                Event::End(element)
                    if matches_local_name(element.name().as_ref(), b"chartSpace") =>
                {
                    break;
                }
                Event::Eof => return Err(missing_end("c:chartSpace")),
                _ => {}
            }
            buffer.clear();
        }

        state.finish(raw_attributes, namespace_declarations)
    }

    /// Writes one chart-space part using fixed OOXML prefixes and schema order.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        let mut start = BytesStart::new("c:chartSpace");
        start.push_attribute(("xmlns:c", C_NS));
        start.push_attribute(("xmlns:a", A_NS));
        start.push_attribute(("xmlns:r", R_NS));
        push_attributes(&mut start, &self.namespace_declarations);
        push_attributes(&mut start, &self.raw_attributes);
        writer
            .write_event(Event::Start(start))
            .map_err(OxmlError::from)?;
        for raw_boundary in 0..8 {
            emit_raw(&mut writer, self.raw_children.at(raw_boundary))?;
        }
        self.chart.write_xml(&mut writer)?;
        emit_raw(&mut writer, self.raw_children.at(8))?;
        if let Some(properties) = &self.sp_pr {
            properties.write_xml_as(&mut writer, "c:spPr")?;
        }
        emit_raw(&mut writer, self.raw_children.at(9))?;
        if let Some(text) = &self.tx_pr {
            text.write_xml_as(&mut writer, "c:txPr")?;
        }
        for raw_boundary in 10..=14 {
            emit_raw(&mut writer, self.raw_children.at(raw_boundary))?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("c:chartSpace")))
            .map_err(OxmlError::from)?;
        Ok(writer.into_inner())
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }
}

#[derive(Default)]
struct ChartSpaceParseState {
    chart: Option<CT_Chart>,
    sp_pr: Option<CT_ShapeProperties>,
    tx_pr: Option<CT_TextBody>,
    raw_children: OrderedRawChildren,
    boundary: usize,
}

impl ChartSpaceParseState {
    fn capture_event(&mut self, raw: Vec<u8>) {
        self.raw_children.push(self.boundary, raw);
    }

    fn parse_child(
        &mut self,
        name: &[u8],
        raw: Vec<u8>,
        namespaces: &NamespaceBindings,
    ) -> Result<()> {
        match name {
            b"chart" => {
                set_once(
                    &mut self.chart,
                    CT_Chart::from_xml_with_namespaces(&raw, namespaces)?,
                    "c:chart",
                )?;
                self.boundary = self.boundary.max(8);
            }
            b"spPr" => {
                reject_conflicting_prefix_in_xml(&raw, b"a", A_NS)?;
                let properties = CT_ShapeProperties::from_xml(&raw)?;
                let mut writer = Writer::new(Vec::new());
                properties.write_xml_as(&mut writer, "c:spPr")?;
                reject_rewritten_foreign_elements(&raw, &writer.into_inner(), namespaces, b"spPr")?;
                set_once(&mut self.sp_pr, properties, "c:spPr")?;
                self.boundary = self.boundary.max(9);
            }
            b"txPr" => {
                reject_conflicting_prefix_in_xml(&raw, b"a", A_NS)?;
                let text = CT_TextBody::from_xml_as(&raw, b"txPr")?;
                let mut writer = Writer::new(Vec::new());
                text.write_xml_as(&mut writer, "c:txPr")?;
                reject_rewritten_foreign_elements(&raw, &writer.into_inner(), namespaces, b"txPr")?;
                set_once(&mut self.tx_pr, text, "c:txPr")?;
                self.boundary = self.boundary.max(10);
            }
            _ => {
                self.raw_children
                    .push(chart_space_raw_slot(name).unwrap_or(self.boundary), raw);
                self.boundary = self.boundary.max(chart_space_raw_boundary_after(name));
            }
        }
        Ok(())
    }

    fn finish(
        self,
        raw_attributes: XmlAttributes,
        namespace_declarations: XmlAttributes,
    ) -> Result<CT_ChartSpace> {
        Ok(CT_ChartSpace {
            chart: self
                .chart
                .ok_or_else(|| ChartError::MissingElement("c:chart".to_owned()))?,
            sp_pr: self.sp_pr,
            tx_pr: self.tx_pr,
            raw_attributes,
            namespace_declarations,
            raw_children: raw_children_in_schema_order(&self.raw_children, 14),
        })
    }
}

fn raw_children_in_schema_order(
    raw_children: &OrderedRawChildren,
    final_boundary: usize,
) -> OrderedRawChildren {
    let mut ordered = OrderedRawChildren::default();
    for boundary in 0..=final_boundary {
        for raw in raw_children.at(boundary) {
            ordered.push(boundary, raw.to_vec());
        }
    }
    ordered
}

fn chart_space_raw_slot(name: &[u8]) -> Option<usize> {
    match name {
        b"date1904" => Some(0),
        b"lang" => Some(1),
        b"roundedCorners" => Some(2),
        b"style" => Some(3),
        b"clrMapOvr" => Some(4),
        b"pivotSource" => Some(5),
        b"protection" => Some(6),
        b"externalData" => Some(10),
        b"printSettings" => Some(11),
        b"userShapes" => Some(12),
        b"extLst" => Some(13),
        _ => None,
    }
}

fn chart_space_raw_boundary_after(name: &[u8]) -> usize {
    match name {
        b"date1904" => 1,
        b"lang" => 2,
        b"roundedCorners" => 3,
        b"style" => 4,
        b"clrMapOvr" => 5,
        b"pivotSource" => 6,
        b"protection" => 7,
        b"chart" => 8,
        b"spPr" => 9,
        b"txPr" => 10,
        b"externalData" => 11,
        b"printSettings" => 12,
        b"userShapes" => 13,
        b"extLst" => 14,
        _ => 0,
    }
}

fn parse_bool_value(xml: &[u8], local: &str) -> Result<(bool, ScalarMarkup)> {
    let (value, markup) = scalar_value(xml, local)?;
    let value = match value.as_deref().unwrap_or("1") {
        "1" | "true" => true,
        "0" | "false" => false,
        invalid => return Err(invalid_attribute(local, "val", invalid.to_owned())),
    };
    Ok((value, markup))
}

fn parse_enum_value(xml: &[u8], local: &str) -> Result<(DispBlanksAs, ScalarMarkup)> {
    let (value, markup) = scalar_value(xml, local)?;
    let value = value.ok_or_else(|| invalid_attribute(local, "val", "<missing>".to_owned()))?;
    let parsed =
        DispBlanksAs::parse(&value).ok_or_else(|| invalid_attribute(local, "val", value))?;
    Ok((parsed, markup))
}

fn scalar_value(xml: &[u8], expected: &str) -> Result<(Option<String>, ScalarMarkup)> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Empty(element)
                if matches_local_name(element.name().as_ref(), expected.as_bytes()) =>
            {
                reject_conflicting_prefix(&element, b"c", C_NS)?;
                return Ok((
                    attribute_value(&element, b"val")?,
                    ScalarMarkup {
                        raw_attributes: capture_attributes_except(&element, b"val")?,
                        raw_content: Vec::new(),
                    },
                ));
            }
            Event::Start(element)
                if matches_local_name(element.name().as_ref(), expected.as_bytes()) =>
            {
                reject_conflicting_prefix(&element, b"c", C_NS)?;
                let value = attribute_value(&element, b"val")?;
                let raw_attributes = capture_attributes_except(&element, b"val")?;
                let mut raw_content = Vec::new();
                let mut inner = Vec::new();
                loop {
                    match reader
                        .read_event_into(&mut inner)
                        .map_err(OxmlError::from)?
                    {
                        Event::End(end)
                            if matches_local_name(end.name().as_ref(), expected.as_bytes()) =>
                        {
                            return Ok((
                                value,
                                ScalarMarkup {
                                    raw_attributes,
                                    raw_content,
                                },
                            ));
                        }
                        event @ (Event::Text(_)
                        | Event::CData(_)
                        | Event::Comment(_)
                        | Event::PI(_)
                        | Event::GeneralRef(_)) => raw_content.push(capture_event(event)?),
                        Event::Eof => return Err(missing_end(expected)),
                        _ => return Err(ChartError::UnexpectedElement(expected.to_owned())),
                    }
                    inner.clear();
                }
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => return Err(ChartError::MissingElement(expected.to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

fn set_once<T>(slot: &mut Option<T>, value: T, name: &str) -> Result<()> {
    if slot.is_some() {
        return Err(ChartError::DuplicateElement(name.to_owned()));
    }
    *slot = Some(value);
    Ok(())
}

fn mark_once(seen: &mut bool, name: &str) -> Result<()> {
    if *seen {
        return Err(ChartError::DuplicateElement(name.to_owned()));
    }
    *seen = true;
    Ok(())
}

fn validate_chart_space_namespace(element: &BytesStart<'_>) -> Result<()> {
    let name = element.name();
    let qualified = name.as_ref();
    let declaration = match qualified.iter().position(|byte| *byte == b':') {
        Some(position) => {
            let mut declaration = b"xmlns:".to_vec();
            declaration.extend_from_slice(&qualified[..position]);
            declaration
        }
        None => b"xmlns".to_vec(),
    };
    let namespace = attribute_value(element, &declaration)?
        .ok_or_else(|| invalid_attribute("chartSpace", "namespace", "<missing>".to_owned()))?;
    if namespace != C_NS {
        return Err(invalid_attribute("chartSpace", "namespace", namespace));
    }
    reject_conflicting_prefix(element, b"c", C_NS)?;
    reject_conflicting_prefix(element, b"a", A_NS)?;
    reject_conflicting_prefix(element, b"r", R_NS)
}

fn reject_conflicting_prefix_in_xml(xml: &[u8], prefix: &[u8], namespace: &str) -> Result<()> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) | Event::Empty(element) => {
                return reject_conflicting_prefix(&element, prefix, namespace);
            }
            Event::Eof => return Err(ChartError::MissingElement("XML root".to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

fn reject_rewritten_foreign_elements(
    original: &[u8],
    written: &[u8],
    inherited: &NamespaceBindings,
    root_local: &[u8],
) -> Result<()> {
    let original = foreign_element_regions(original, inherited, root_local)?;
    let mut written_namespaces = inherited.clone();
    written_namespaces.retain(|(prefix, _)| prefix != b"a");
    written_namespaces.push((b"a".to_vec(), A_NS.to_owned()));
    let written = foreign_element_regions(written, &written_namespaces, root_local)?;
    if original != written {
        return Err(invalid_attribute(
            &String::from_utf8_lossy(root_local),
            "namespace",
            "foreign DrawingML content would be converted to typed content".to_owned(),
        ));
    }
    Ok(())
}

fn foreign_element_regions(
    xml: &[u8],
    inherited: &NamespaceBindings,
    root_local: &[u8],
) -> Result<Vec<Vec<u8>>> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let root_namespaces = loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) if matches_local_name(element.name().as_ref(), root_local) => {
                break bindings_with_local(inherited, &element)?;
            }
            Event::Empty(element) if matches_local_name(element.name().as_ref(), root_local) => {
                return Ok(Vec::new());
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => {
                return Err(ChartError::MissingElement(
                    String::from_utf8_lossy(root_local).into_owned(),
                ));
            }
            _ => {}
        }
        buffer.clear();
    };

    let mut namespace_stack = vec![root_namespaces];
    let mut foreign = Vec::new();
    let mut child_buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut child_buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) => {
                let namespaces = namespace_stack.last().expect("root namespace binding");
                if element_is_in_namespace(&element, A_NS, namespaces)? {
                    namespace_stack.push(bindings_with_local(namespaces, &element)?);
                } else {
                    foreign.push(capture_element(&mut reader, &element)?);
                }
            }
            Event::Empty(element) => {
                let namespaces = namespace_stack.last().expect("root namespace binding");
                if !element_is_in_namespace(&element, A_NS, namespaces)? {
                    foreign.push(capture_empty_element(&element)?);
                }
            }
            Event::End(element) if matches_local_name(element.name().as_ref(), root_local) => {
                return Ok(foreign);
            }
            Event::End(_) => {
                namespace_stack.pop();
            }
            Event::Eof => {
                return Err(missing_end(&String::from_utf8_lossy(root_local)));
            }
            _ => {}
        }
        child_buffer.clear();
    }
}

fn reject_conflicting_prefix(
    element: &BytesStart<'_>,
    prefix: &[u8],
    namespace: &str,
) -> Result<()> {
    let mut key = b"xmlns:".to_vec();
    key.extend_from_slice(prefix);
    if let Some(value) = attribute_value(element, &key)?
        && value != namespace
    {
        return Err(invalid_attribute(
            &element_name(element),
            &String::from_utf8_lossy(&key),
            value,
        ));
    }
    Ok(())
}

fn namespace_bindings(element: &BytesStart<'_>) -> Result<NamespaceBindings> {
    let mut bindings = Vec::new();
    for attribute in element.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        let key = attribute.key.as_ref();
        let prefix = if key == b"xmlns" {
            Vec::new()
        } else if let Some(prefix) = key.strip_prefix(b"xmlns:") {
            prefix.to_vec()
        } else {
            continue;
        };
        let namespace = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())
            .map_err(OxmlError::from)?
            .into_owned();
        bindings.push((prefix, namespace));
    }
    Ok(bindings)
}

fn bindings_with_local(
    inherited: &NamespaceBindings,
    element: &BytesStart<'_>,
) -> Result<NamespaceBindings> {
    let mut bindings = inherited.clone();
    for (prefix, namespace) in namespace_bindings(element)? {
        upsert_namespace_binding(&mut bindings, prefix, namespace);
    }
    Ok(bindings)
}

fn upsert_namespace_binding(bindings: &mut NamespaceBindings, prefix: Vec<u8>, namespace: String) {
    if let Some((_, current)) = bindings.iter_mut().find(|(current, _)| current == &prefix) {
        *current = namespace;
    } else {
        bindings.push((prefix, namespace));
    }
}

fn element_is_in_namespace(
    element: &BytesStart<'_>,
    expected: &str,
    inherited: &NamespaceBindings,
) -> Result<bool> {
    let name = element.name();
    let qualified = name.as_ref();
    let prefix = qualified
        .iter()
        .position(|byte| *byte == b':')
        .map(|position| &qualified[..position])
        .unwrap_or_default();
    let local = namespace_bindings(element)?;
    let namespace = local
        .iter()
        .rev()
        .chain(inherited.iter().rev())
        .find(|(candidate, _)| candidate.as_slice() == prefix)
        .map(|(_, namespace)| namespace.as_str());
    Ok(namespace == Some(expected))
}

fn attribute_value(element: &BytesStart<'_>, name: &[u8]) -> Result<Option<String>> {
    for attribute in element.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        if attribute.key.as_ref() == name {
            return Ok(Some(
                attribute
                    .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())
                    .map_err(OxmlError::from)?
                    .into_owned(),
            ));
        }
    }
    Ok(None)
}

fn capture_root_attributes(start: &BytesStart<'_>) -> Result<RootAttributes> {
    let mut attributes = Vec::new();
    let mut namespaces = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        let name = std::str::from_utf8(attribute.key.as_ref())
            .map_err(OxmlError::from)?
            .to_owned();
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())
            .map_err(OxmlError::from)?
            .into_owned();
        if name == "xmlns:c" || name == "xmlns:a" || name == "xmlns:r" {
            continue;
        }
        if name == "xmlns" || name.starts_with("xmlns:") {
            namespaces.push((name, value));
        } else {
            attributes.push((name, value));
        }
    }
    Ok((attributes, namespaces))
}

fn capture_attributes(start: &BytesStart<'_>) -> Result<XmlAttributes> {
    let mut attributes = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        attributes.push((
            std::str::from_utf8(attribute.key.as_ref())
                .map_err(OxmlError::from)?
                .to_owned(),
            attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())
                .map_err(OxmlError::from)?
                .into_owned(),
        ));
    }
    Ok(attributes)
}

fn capture_attributes_except(start: &BytesStart<'_>, excluded: &[u8]) -> Result<XmlAttributes> {
    let mut attributes = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        if attribute.key.as_ref() == excluded {
            continue;
        }
        attributes.push((
            std::str::from_utf8(attribute.key.as_ref())
                .map_err(OxmlError::from)?
                .to_owned(),
            attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())
                .map_err(OxmlError::from)?
                .into_owned(),
        ));
    }
    Ok(attributes)
}

fn push_attributes(start: &mut BytesStart<'_>, attributes: &[(String, String)]) {
    for (name, value) in attributes {
        start.push_attribute((name.as_str(), value.as_str()));
    }
}

fn write_scalar<W: Write>(
    writer: &mut Writer<W>,
    name: &str,
    value: &str,
    markup: Option<&ScalarMarkup>,
) -> Result<()> {
    let mut start = BytesStart::new(name);
    start.push_attribute(("val", value));
    if let Some(markup) = markup {
        push_attributes(&mut start, &markup.raw_attributes);
    }
    let has_content = markup.is_some_and(|markup| !markup.raw_content.is_empty());
    if !has_content {
        writer
            .write_event(Event::Empty(start))
            .map_err(OxmlError::from)?;
        return Ok(());
    }
    writer
        .write_event(Event::Start(start))
        .map_err(OxmlError::from)?;
    if let Some(markup) = markup {
        for raw in &markup.raw_content {
            writer.get_mut().write_all(raw).map_err(OxmlError::from)?;
        }
    }
    writer
        .write_event(Event::End(BytesEnd::new(name)))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn capture_event(event: Event<'_>) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Vec::new());
    writer
        .write_event(event.into_owned())
        .map_err(OxmlError::from)?;
    Ok(writer.into_inner())
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

fn invalid_attribute(element: &str, attribute: &str, value: String) -> ChartError {
    ChartError::InvalidAttribute {
        element: element.to_owned(),
        attribute: attribute.to_owned(),
        value,
    }
}

fn element_name(element: &BytesStart<'_>) -> String {
    String::from_utf8_lossy(element.name().as_ref()).into_owned()
}

fn missing_end(element: &str) -> ChartError {
    ChartError::Xml(OxmlError::MissingElement(format!("closing {element}")))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use oxml_core::raw_xml::{capture_element, capture_empty_element};
    use oxml_opc::OpcPackage;
    use quick_xml::Reader;
    use quick_xml::events::Event;

    use super::{
        AxisData, CT_ChartSpace, CT_ShapeProperties, CT_TextBody, DispBlanksAs, NumericData,
        Series, StringRef, capture_event, local_name,
    };

    const MANIFEST: &str = include_str!("../../../scripts/pptx-corpus-manifest.tsv");
    const EXPECTED_DECKS: usize = 50;

    #[test]
    fn series_formula_and_cache_are_consistent_with_one_source() {
        let values = NumericData::new(
            "'Sales 24'!$B$2:$B$4".to_owned(),
            "0.0".to_owned(),
            vec![4.25, 8.5, 17.0],
        )
        .unwrap();
        let mut series = Series::new(3, 1, values);
        series.categories = Some(AxisData::String(
            StringRef::new(
                "'Sales 24'!$A$2:$A$4".to_owned(),
                vec!["North".to_owned(), "South".to_owned(), "West".to_owned()],
            )
            .unwrap(),
        ));

        let written = String::from_utf8(series.to_xml().unwrap()).unwrap();
        assert!(written.contains("<c:f>&apos;Sales 24&apos;!$B$2:$B$4</c:f>"));
        assert!(written.contains("<c:formatCode>0.0</c:formatCode>"));
        assert_eq!(written.matches("<c:ptCount val=\"3\"").count(), 2);
        for (index, value) in ["4.25", "8.5", "17"].iter().enumerate() {
            assert!(written.contains(&format!("<c:pt idx=\"{index}\"><c:v>{value}</c:v></c:pt>")));
        }
        let reparsed = Series::from_xml(written.as_bytes()).unwrap();
        assert_eq!(reparsed.index, series.index);
        assert_eq!(reparsed.order, series.order);
        match (&reparsed.categories, &series.categories) {
            (Some(AxisData::String(left)), Some(AxisData::String(right))) => {
                assert_eq!(left.formula, right.formula);
                assert_eq!(left.values, right.values);
            }
            _ => panic!("expected string categories"),
        }
        assert_eq!(reparsed.values.formula, series.values.formula);
        assert_eq!(reparsed.values.format_code, series.values.format_code);
        assert_eq!(reparsed.values.values, series.values.values);
    }

    #[test]
    fn string_and_numeric_references_write_fixed_prefixes_in_schema_order() {
        let string_xml = br#"<q:strRef xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart"><q:strCache><q:pt idx="0"><q:v>West</q:v></q:pt><q:ptCount val="1"/></q:strCache><q:f>Sheet1!$A$2</q:f></q:strRef>"#;
        let string_ref = StringRef::from_xml(string_xml).unwrap();
        let written = String::from_utf8(string_ref.to_xml().unwrap()).unwrap();
        assert!(written.starts_with("<c:strRef xmlns:c="));
        assert!(written.find("<c:f>").unwrap() < written.find("<c:strCache>").unwrap());
        assert!(written.find("<c:ptCount").unwrap() < written.find("<c:pt idx=").unwrap());

        let numeric_xml = br#"<q:numRef xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart"><q:numCache><q:ptCount val="2"/><q:pt idx="0"><q:v>1.5</q:v></q:pt><q:pt idx="1"><q:v>2.5</q:v></q:pt><q:formatCode>0.00</q:formatCode></q:numCache><q:f>Sheet1!$B$2:$B$3</q:f></q:numRef>"#;
        let numeric = NumericData::from_xml(numeric_xml).unwrap();
        let written = String::from_utf8(numeric.to_xml().unwrap()).unwrap();
        let positions: Vec<_> = [
            "<c:f>",
            "<c:numCache>",
            "<c:formatCode>",
            "<c:ptCount",
            "<c:pt idx=",
        ]
        .iter()
        .map(|tag| written.find(tag).unwrap())
        .collect();
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(numeric, NumericData::from_xml(written.as_bytes()).unwrap());
    }

    #[test]
    fn mixed_chartml_aliases_resolve_by_namespace_uri() {
        let xml = br#"<q:ser xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="4"/><q:order val="2"/><c:cat><q:strRef><c:f>Sheet1!$A$2:$A$3</c:f><q:strCache><c:ptCount val="2"/><q:pt idx="0"><c:v>North</c:v></q:pt><c:pt idx="1"><q:v>West</q:v></c:pt></q:strCache></q:strRef></c:cat><q:val><c:numRef><q:f>Sheet1!$B$2:$B$3</q:f><c:numCache><q:formatCode>0.0</q:formatCode><c:ptCount val="2"/><q:pt idx="0"><c:v>1.5</c:v></q:pt><c:pt idx="1"><q:v>2.5</q:v></c:pt></c:numCache></c:numRef></q:val></q:ser>"#;
        let parsed = Series::from_xml(xml).unwrap();
        assert_eq!(parsed.index, 4);
        assert_eq!(parsed.order, 2);
        assert_eq!(parsed.values.values, vec![1.5, 2.5]);
        assert!(matches!(parsed.categories, Some(AxisData::String(_))));
        assert_eq!(parsed, Series::from_xml(&parsed.to_xml().unwrap()).unwrap());
    }

    #[test]
    fn plot_area_series_ignores_inherited_foreign_plot_aliases() {
        let xml = br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:x="urn:producer"><c:chart><c:plotArea><x:barChart><x:ser><x:idx val="9"/><x:order val="9"/><x:val><x:numRef><x:f>foreign</x:f><x:numCache><x:formatCode>General</x:formatCode><x:ptCount val="0"/></x:numCache></x:numRef></x:val></x:ser></x:barChart><c:barChart><q:ser><q:idx val="1"/><q:order val="0"/><q:marker><x:data/></q:marker><q:val><q:numRef><q:f>Sheet1!$B$2</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount val="1"/><q:pt idx="0"><q:v>3</q:v></q:pt></q:numCache></q:numRef></q:val></q:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#;
        let chart = CT_ChartSpace::from_xml(xml).unwrap();
        let series = chart.chart.plot_area.series().unwrap();
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].index, 1);
        let written = String::from_utf8(series[0].to_xml().unwrap()).unwrap();
        assert!(
            written.contains(r#"xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart""#)
        );
        assert!(written.contains(r#"xmlns:x="urn:producer""#));
        assert!(written.contains("<q:marker><x:data/></q:marker>"));
        assert_eq!(series[0], Series::from_xml(written.as_bytes()).unwrap());

        let conflicting = br#"<q:chartSpace xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart"><q:chart><q:plotArea><q:barChart xmlns:c="urn:foreign"><q:ser><q:idx val="0"/><q:order val="0"/><q:val><q:numRef><q:f>Sheet1!$A$1</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount val="0"/></q:numCache></q:numRef></q:val></q:ser></q:barChart></q:plotArea></q:chart></q:chartSpace>"#;
        let chart = CT_ChartSpace::from_xml(conflicting).unwrap();
        assert!(chart.chart.plot_area.series().is_err());
    }

    #[test]
    fn malformed_series_and_cache_values_return_errors_without_panicking() {
        let cases: &[&[u8]] = &[
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:order val="0"/><c:val><c:numRef><c:f>S!$A$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:order val="0"/></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:idx val="1"/><c:order val="0"/><c:val><c:numRef><c:f>S!$A$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:order val="0"/><c:val><c:numRef><c:f></c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:order val="0"/><c:val><c:numRef><c:f>S!$A$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/><c:pt idx="0"><c:v>1</c:v></c:pt></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:order val="0"/><c:val><c:numRef><c:f>S!$A$1:$A$2</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="2"/><c:pt idx="0"><c:v>1</c:v></c:pt><c:pt idx="2"><c:v>2</c:v></c:pt></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:order val="0"/><c:val><c:numRef><c:f>S!$A$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="1"/><c:pt idx="0"><c:v>NaN</c:v></c:pt></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="urn:producer"><c:idx val="0"/><c:order val="0"/><a:marker/><c:val><c:numRef><c:f>S!$A$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:x="urn:producer"><c:idx val="0"/><c:order val="0"/><c:spPr><x:solidFill><x:srgbClr val="112233"/></x:solidFill></c:spPr><c:val><c:numRef><c:f>S!$A$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:order val="0"/><c:tx><c:v>opaque</c:v></c:tx><c:tx><c:strRef><c:f>S!$A$1</c:f><c:strCache><c:ptCount val="0"/></c:strCache></c:strRef></c:tx><c:val><c:numRef><c:f>S!$B$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:order val="0"/><c:cat><c:multiLvlStrRef/></c:cat><c:cat><c:numRef><c:f>S!$A$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:cat><c:val><c:numRef><c:f>S!$B$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:order val="0"/><c:val><c:numRef><c:f>S!$B$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val><c:bubbleSize><c:numLit/></c:bubbleSize><c:bubbleSize><c:numRef><c:f>S!$C$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:bubbleSize></c:ser>"#,
            br#"<q:ser xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart"><q:idx val="0"/><q:order val="0"/><q:val xmlns:c="urn:producer"><q:numRef><q:f>S!$A$1</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount val="0"/></q:numCache></q:numRef></q:val></q:ser>"#,
            br#"<q:ser xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart"><q:idx val="0"/><q:order val="0"/><q:val><q:numRef><q:f>S!$A$1</q:f><q:numCache xmlns:c="urn:producer"><q:formatCode>General</q:formatCode><q:ptCount val="0"/></q:numCache></q:numRef></q:val></q:ser>"#,
            br#"<q:ser xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart"><q:idx val="0"/><q:order val="0"/><q:val><q:numRef><q:f>S!$A$1</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount val="1"/><q:pt xmlns:c="urn:producer" idx="0"><q:v>1</q:v></q:pt></q:numCache></q:numRef></q:val></q:ser>"#,
            br#"<q:ser xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart"><q:idx val="0"/><q:order val="0"/><q:val><q:numRef><q:f>S!$A$1</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount xmlns:c="urn:producer" val="0"/></q:numCache></q:numRef></q:val></q:ser>"#,
            br#"<q:ser xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart"><q:idx val="0"/><q:order val="0"/><q:val><q:numRef><q:f xmlns:c="urn:producer">S!$A$1</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount val="0"/></q:numCache></q:numRef></q:val></q:ser>"#,
        ];
        for xml in cases {
            let result = std::panic::catch_unwind(|| Series::from_xml(xml));
            assert!(result.is_ok(), "series parser panicked");
            assert!(
                result.unwrap().is_err(),
                "malformed series parsed: {}",
                String::from_utf8_lossy(xml)
            );
        }
        assert!(
            NumericData::new(
                "S!$A$1".to_owned(),
                "General".to_owned(),
                vec![f64::INFINITY]
            )
            .is_err()
        );
    }

    #[test]
    fn series_preserves_unmodelled_children_byte_for_byte() {
        let xml = br#"<q:ser xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer" x:keep="series"><!--before--><q:idx val="0" x:keep="idx"/><x:between/><q:order val="0"/><q:tx x:keep="tx"><q:strRef x:keep="ref"><q:f x:keep="formula">S!$B$1</q:f><q:strCache x:keep="cache"><q:ptCount val="1" x:keep="count"/><!--point--><q:pt idx="0" x:keep="point"><q:v x:keep="value">Revenue</q:v></q:pt><x:cacheExt/></q:strCache></q:strRef></q:tx><q:spPr><d:noFill/><x:shapeExt/></q:spPr><q:dPt x:id="one"><q:idx val="0"/></q:dPt><q:dLbls x:id="labels"/><q:cat><q:strRef><q:f>S!$A$2:$A$3</q:f><q:strCache><q:ptCount val="2"/><q:pt idx="0"><q:v>North</q:v></q:pt><q:pt idx="1"><q:v>West</q:v></q:pt></q:strCache></q:strRef></q:cat><q:trendline x:id="trend"/><q:val><q:numRef><q:f>S!$B$2:$B$3</q:f><q:numCache><q:formatCode>0.0</q:formatCode><q:ptCount val="2"/><q:pt idx="0"><q:v>1.5</q:v></q:pt><q:pt idx="1"><q:v>2.5</q:v></q:pt></q:numCache></q:numRef></q:val><q:extLst><q:ext uri="keep"><x:data/></q:ext></q:extLst></q:ser>"#;
        let parsed = Series::from_xml(xml).unwrap();
        let written = parsed.to_xml().unwrap();
        for raw in [
            br#"<!--before-->"#.as_slice(),
            br#"<x:between/>"#.as_slice(),
            br#"<q:dPt x:id="one"><q:idx val="0"/></q:dPt>"#.as_slice(),
            br#"<q:dLbls x:id="labels"/>"#.as_slice(),
            br#"<q:trendline x:id="trend"/>"#.as_slice(),
            br#"<q:extLst><q:ext uri="keep"><x:data/></q:ext></q:extLst>"#.as_slice(),
            br#"<!--point-->"#.as_slice(),
            br#"<x:cacheExt/>"#.as_slice(),
        ] {
            assert!(
                written.windows(raw.len()).any(|window| window == raw),
                "preserved series bytes changed: {}",
                String::from_utf8_lossy(raw)
            );
        }
        let written = String::from_utf8(written.clone()).unwrap();
        assert!(written.contains(r#"x:keep="series""#));
        assert!(written.contains(r#"x:keep="formula""#));
        assert_eq!(parsed, Series::from_xml(written.as_bytes()).unwrap());
    }

    #[test]
    fn public_series_edits_do_not_duplicate_or_drop_preserved_payloads() {
        let opaque_xml = br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:order val="0"/><c:tx><c:v>opaque</c:v></c:tx><c:cat><c:multiLvlStrRef/></c:cat><c:val><c:numRef><c:f>S!$A$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val><c:bubbleSize><c:numLit/></c:bubbleSize></c:ser>"#;
        let parsed = Series::from_xml(opaque_xml).unwrap();

        let mut edited = parsed.clone();
        edited.name = Some(StringRef::new("S!$A$1".to_owned(), Vec::new()).unwrap());
        assert!(edited.to_xml().is_err());

        let mut edited = parsed.clone();
        edited.categories = Some(AxisData::Numeric(
            NumericData::new("S!$A$1".to_owned(), "General".to_owned(), Vec::new()).unwrap(),
        ));
        assert!(edited.to_xml().is_err());

        let mut edited = parsed;
        edited.bubble_size =
            Some(NumericData::new("S!$A$1".to_owned(), "General".to_owned(), Vec::new()).unwrap());
        assert!(edited.to_xml().is_err());

        let cache_xml = br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:x="urn:producer"><c:idx val="0"/><c:order val="0"/><c:val><c:numRef><c:f>S!$A$1:$A$2</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="2"/><c:pt idx="0"><c:v>1</c:v></c:pt><c:pt idx="1"><c:v>2</c:v></c:pt><c:extLst><c:ext uri="keep"><x:data/></c:ext></c:extLst></c:numCache></c:numRef></c:val></c:ser>"#;
        let parsed = Series::from_xml(cache_xml).unwrap();
        let tail = "<c:extLst><c:ext uri=\"keep\"><x:data/></c:ext></c:extLst>";

        let mut shortened = parsed.clone();
        shortened.values.values.truncate(1);
        let written = String::from_utf8(shortened.to_xml().unwrap()).unwrap();
        assert!(!written.contains("<c:pt idx=\"1\""));
        assert!(written.find("<c:pt idx=\"0\"").unwrap() < written.find(tail).unwrap());
        assert!(Series::from_xml(written.as_bytes()).is_ok());

        let mut grown = parsed;
        grown.values.values.push(3.0);
        let written = String::from_utf8(grown.to_xml().unwrap()).unwrap();
        assert!(written.find("<c:pt idx=\"2\"").unwrap() < written.find(tail).unwrap());
        assert!(Series::from_xml(written.as_bytes()).is_ok());

        let ordered_xml = br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><c:idx val="0"/><c:order val="0"/><c:marker><c:symbol val="circle"/></c:marker><c:val><c:numRef><c:f>S!$B$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val><c:extLst><c:ext uri="keep"/></c:extLst></c:ser>"#;
        let mut edited = Series::from_xml(ordered_xml).unwrap();
        edited.name = Some(StringRef::new("S!$B$1".to_owned(), Vec::new()).unwrap());
        edited.sp_pr = Some(CT_ShapeProperties::from_xml(br#"<a:spPr/>"#).unwrap());
        edited.categories = Some(AxisData::String(
            StringRef::new("S!$A$1".to_owned(), Vec::new()).unwrap(),
        ));
        edited.bubble_size =
            Some(NumericData::new("S!$C$1".to_owned(), "General".to_owned(), Vec::new()).unwrap());
        let written = String::from_utf8(edited.to_xml().unwrap()).unwrap();
        let positions: Vec<_> = [
            "<c:tx>",
            "<c:spPr",
            "<c:marker>",
            "<c:cat>",
            "<c:val>",
            "<c:bubbleSize>",
            "<c:extLst>",
        ]
        .iter()
        .map(|tag| written.find(tag).unwrap())
        .collect();
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(Series::from_xml(written.as_bytes()).is_ok());
    }

    #[test]
    fn every_corpus_series_round_trips_structurally() {
        let Some(corpus) = require_or_skip_corpus() else {
            return;
        };
        verify_fetched_corpus(&corpus);
        let mut series_count = 0usize;
        let mut chart_parts = 0usize;
        for path in manifest_paths() {
            let package = OpcPackage::open(corpus.join(path))
                .unwrap_or_else(|error| panic!("{path}: open failed: {error}"));
            for (part, xml) in &package.parts {
                if !is_chart_part(part) {
                    continue;
                }
                let chart = CT_ChartSpace::from_xml(xml)
                    .unwrap_or_else(|error| panic!("{path} {part}: parse failed: {error}"));
                let parsed_series =
                    chart.chart.plot_area.series().unwrap_or_else(|error| {
                        panic!("{path} {part}: series parse failed: {error}")
                    });
                for series in parsed_series {
                    let written = series.to_xml().unwrap_or_else(|error| {
                        panic!("{path} {part}: series write failed: {error}")
                    });
                    let reparsed = Series::from_xml(&written).unwrap_or_else(|error| {
                        panic!("{path} {part}: written series parse failed: {error}")
                    });
                    assert_eq!(series, reparsed, "{path} {part}: series model changed");
                    series_count += 1;
                }
                chart_parts += 1;
            }
        }
        assert!(
            series_count > 0,
            "the pinned corpus contained no supported series"
        );
        eprintln!(
            "ChartML series corpus gate checked {series_count} series across {chart_parts} chart parts"
        );
    }

    #[test]
    fn chart_space_reads_aliases_and_writes_fixed_prefixes_in_schema_order() {
        let xml = br#"<ch:chartSpace xmlns:ch="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:rel="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:x="urn:producer"><x:before/><ch:chart><ch:plotVisOnly val="0"/><ch:legend><x:legend/></ch:legend><ch:title><x:title/></ch:title><ch:plotArea><x:plot/></ch:plotArea><ch:dispBlanksAs val="span"/></ch:chart><ch:txPr><d:bodyPr/><d:p/></ch:txPr><ch:spPr><d:solidFill><d:srgbClr val="112233"/></d:solidFill></ch:spPr><x:after/></ch:chartSpace>"#;
        let parsed = CT_ChartSpace::from_xml(xml).unwrap();
        assert!(!parsed.chart.auto_title_deleted);
        assert!(!parsed.chart.plot_vis_only);
        assert_eq!(parsed.chart.disp_blanks_as, DispBlanksAs::Span);
        assert!(parsed.chart.title.is_some());
        assert!(parsed.chart.legend.is_some());
        assert!(parsed.sp_pr.is_some());
        assert!(parsed.tx_pr.is_some());

        let written = String::from_utf8(parsed.to_xml().unwrap()).unwrap();
        assert!(written.starts_with(
            r#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships""#
        ));
        for tag in [
            "<c:chart>",
            "<c:title>",
            "<c:plotArea>",
            "<c:legend>",
            "<c:plotVisOnly",
            "<c:dispBlanksAs",
            "<c:spPr>",
            "<c:txPr>",
        ] {
            assert!(written.contains(tag), "missing fixed-prefix tag {tag}");
        }
        let positions: Vec<_> = [
            "<c:title>",
            "<c:plotArea>",
            "<c:legend>",
            "<c:plotVisOnly",
            "<c:dispBlanksAs",
            "<c:spPr>",
            "<c:txPr>",
        ]
        .iter()
        .map(|tag| written.find(tag).unwrap())
        .collect();
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(written.contains("<a:solidFill><a:srgbClr"));
        assert!(written.contains("<a:bodyPr/><a:p/>"));
        assert_eq!(parsed, CT_ChartSpace::from_xml(written.as_bytes()).unwrap());
    }

    #[test]
    fn public_core_edits_keep_preserved_children_in_their_schema_slots() {
        let xml = br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><c:chart><c:pivotFmts><c:pivotFmt/></c:pivotFmts><c:plotArea/></c:chart><c:externalData r:id="rId1"/></c:chartSpace>"#;
        let mut parsed = CT_ChartSpace::from_xml(xml).unwrap();
        parsed.chart.auto_title_deleted = true;
        parsed.sp_pr = Some(CT_ShapeProperties::from_xml(br#"<a:spPr/>"#).unwrap());
        parsed.tx_pr =
            Some(CT_TextBody::from_xml(br#"<a:txBody><a:bodyPr/><a:p/></a:txBody>"#).unwrap());
        let written = String::from_utf8(parsed.to_xml().unwrap()).unwrap();
        let positions: Vec<_> = [
            "<c:autoTitleDeleted",
            "<c:pivotFmts>",
            "<c:plotArea",
            "<c:spPr",
            "<c:txPr",
            "<c:externalData",
        ]
        .iter()
        .map(|tag| written.find(tag).unwrap())
        .collect();
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(CT_ChartSpace::from_xml(written.as_bytes()).is_ok());
    }

    #[test]
    fn malformed_core_chart_values_return_errors_without_panicking() {
        let cases: &[&[u8]] = &[
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"/>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart/></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart><c:plotArea/><c:plotArea/></c:chart></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart><c:autoTitleDeleted val="yes"/><c:plotArea/></c:chart></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart><c:plotArea/><c:dispBlanksAs val="empty"/></c:chart></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="urn:wrong"><c:chart><c:plotArea/></c:chart></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:x="urn:foreign"><x:chart><x:plotArea/></x:chart></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart><c:plotArea/></c:chart><c:chart><c:plotArea/></c:chart></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart><c:title/><c:title/><c:plotArea/></c:chart></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart><c:plotArea/><c:plotVisOnly><c:child/></c:plotVisOnly></c:chart></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:foreign"><c:chart><c:plotArea/></c:chart><c:txPr><a:bodyPr/><a:p><x:r><x:t>foreign</x:t></x:r></a:p></c:txPr></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:foreign"><c:chart><c:plotArea/></c:chart><c:spPr><a:solidFill><x:srgbClr val="112233"/></a:solidFill></c:spPr></c:chartSpace>"#,
        ];
        for xml in cases {
            let result = std::panic::catch_unwind(|| CT_ChartSpace::from_xml(xml));
            assert!(result.is_ok(), "ChartML parser panicked");
            assert!(
                result.unwrap().is_err(),
                "malformed ChartML parsed: {}",
                String::from_utf8_lossy(xml)
            );
        }
    }

    #[test]
    fn core_chart_shells_preserve_unmodelled_children_byte_for_byte() {
        let preserved = [
            br#"<x:root x:id="1"><x:nested>one &amp; two</x:nested><!--root--></x:root>"#
                .as_slice(),
            br#"<x:title x:id="2"><x:nested><![CDATA[title]]></x:nested></x:title>"#.as_slice(),
            br#"<x:plot x:id="3"><x:nested>plot</x:nested></x:plot>"#.as_slice(),
            br#"<x:legend x:id="4"><x:nested>legend</x:nested></x:legend>"#.as_slice(),
            br#"<x:extension x:id="5"><x:nested>extension</x:nested></x:extension>"#.as_slice(),
            br#"<!--space-comment-->"#.as_slice(),
            br#"<?space processing?>"#.as_slice(),
            br#"<!--chart-comment-->"#.as_slice(),
            br#"<?title processing?>"#.as_slice(),
            br#"<!--flag-comment-->"#.as_slice(),
            br#"<?flag processing?>"#.as_slice(),
            br#"<x:producerShape/>"#.as_slice(),
            br#"<x:producerText/>"#.as_slice(),
        ];
        let xml = br#"<q:chartSpace xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer"><!--space-comment--><?space processing?><x:root x:id="1"><x:nested>one &amp; two</x:nested><!--root--></x:root><q:chart><!--chart-comment--><q:autoTitleDeleted val="0" x:keep="one &amp; two"><!--flag-comment--><?flag processing?></q:autoTitleDeleted><q:title data="keep"><?title processing?><x:title x:id="2"><x:nested><![CDATA[title]]></x:nested></x:title></q:title><q:plotArea data="keep"><x:plot x:id="3"><x:nested>plot</x:nested></x:plot></q:plotArea><q:legend data="keep"><x:legend x:id="4"><x:nested>legend</x:nested></x:legend></q:legend><x:extension x:id="5"><x:nested>extension</x:nested></x:extension></q:chart><q:spPr><x:producerShape/></q:spPr><q:txPr><a:bodyPr/><a:p><x:producerText/></a:p></q:txPr></q:chartSpace>"#;
        let parsed = CT_ChartSpace::from_xml(xml).unwrap();
        let written = parsed.to_xml().unwrap();
        for raw in preserved {
            assert!(
                written.windows(raw.len()).any(|window| window == raw),
                "preserved subtree changed: {}",
                String::from_utf8_lossy(raw)
            );
        }
        let written_text = String::from_utf8(written.clone()).unwrap();
        assert!(written_text.contains(r#"x:keep="one &amp; two""#));
        assert_eq!(parsed, CT_ChartSpace::from_xml(&written).unwrap());
    }

    #[test]
    fn every_corpus_chart_part_round_trips_structurally() {
        let Some(corpus) = require_or_skip_corpus() else {
            return;
        };
        verify_fetched_corpus(&corpus);
        let mut chart_parts = 0usize;
        let mut decks_with_charts = HashSet::new();
        for path in manifest_paths() {
            let package = OpcPackage::open(corpus.join(path))
                .unwrap_or_else(|error| panic!("{path}: open failed: {error}"));
            for (part, xml) in &package.parts {
                if !is_chart_part(part) {
                    continue;
                }
                let parsed = CT_ChartSpace::from_xml(xml)
                    .unwrap_or_else(|error| panic!("{path} {part}: parse failed: {error}"));
                let written = parsed
                    .to_xml()
                    .unwrap_or_else(|error| panic!("{path} {part}: write failed: {error}"));
                let original_preserved = preserved_chartml_regions(xml)
                    .unwrap_or_else(|error| panic!("{path} {part}: original scan failed: {error}"));
                let written_preserved = preserved_chartml_regions(&written)
                    .unwrap_or_else(|error| panic!("{path} {part}: written scan failed: {error}"));
                assert_eq!(
                    original_preserved.len(),
                    written_preserved.len(),
                    "{path} {part}: preserved ChartML region count changed"
                );
                for (index, (original, written)) in original_preserved
                    .iter()
                    .zip(&written_preserved)
                    .enumerate()
                {
                    assert_eq!(
                        original.parent_path, written.parent_path,
                        "{path} {part}: preserved region {index} parent changed"
                    );
                    assert_eq!(
                        original.boundary, written.boundary,
                        "{path} {part}: preserved region {index} boundary changed"
                    );
                    assert_eq!(
                        original.sibling_order, written.sibling_order,
                        "{path} {part}: preserved region {index} order changed"
                    );
                    assert_eq!(
                        original.xml, written.xml,
                        "{path} {part}: preserved region {index} bytes changed"
                    );
                }
                let reparsed = CT_ChartSpace::from_xml(&written)
                    .unwrap_or_else(|error| panic!("{path} {part}: written parse failed: {error}"));
                assert_eq!(parsed, reparsed, "{path} {part}: core model changed");
                chart_parts += 1;
                decks_with_charts.insert(path);
            }
        }
        assert!(
            chart_parts > 0,
            "the pinned corpus contained no chart parts"
        );
        eprintln!(
            "ChartML corpus gate checked {chart_parts} chart parts across {} decks",
            decks_with_charts.len()
        );
    }

    #[derive(Debug, Eq, PartialEq)]
    struct PreservedChartRegion {
        parent_path: Vec<Vec<u8>>,
        boundary: usize,
        sibling_order: usize,
        xml: Vec<u8>,
    }

    struct PreservationFrame {
        local_name: Vec<u8>,
        boundary: usize,
        next_sibling: usize,
    }

    fn preserved_chartml_regions(xml: &[u8]) -> Result<Vec<PreservedChartRegion>, String> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        let mut parents: Vec<PreservationFrame> = Vec::new();
        let mut preserved = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(element)) => {
                    let child = local_name(element.name().as_ref()).to_vec();
                    if parents.is_empty() && child == b"chartSpace" {
                        parents.push(PreservationFrame {
                            local_name: child,
                            boundary: 0,
                            next_sibling: 0,
                        });
                    } else if let Some(parent) = parents.last()
                        && is_opaque_chartml_child(&parent.local_name, &child)
                    {
                        let boundary =
                            preserved_child_boundary(&parent.local_name, &child, parent.boundary);
                        let xml = capture_element(&mut reader, &element)
                            .map_err(|error| error.to_string())?;
                        record_preserved_region(&mut parents, &mut preserved, boundary, xml);
                        advance_preservation_boundary(&mut parents, &child);
                    } else {
                        advance_preservation_boundary(&mut parents, &child);
                        if is_chartml_preservation_parent(&child) {
                            parents.push(PreservationFrame {
                                local_name: child,
                                boundary: 0,
                                next_sibling: 0,
                            });
                        } else {
                            capture_element(&mut reader, &element)
                                .map_err(|error| error.to_string())?;
                        }
                    }
                }
                Ok(Event::Empty(element)) => {
                    let name = element.name();
                    let child = local_name(name.as_ref()).to_vec();
                    if let Some(parent) = parents.last()
                        && is_opaque_chartml_child(&parent.local_name, &child)
                    {
                        let boundary =
                            preserved_child_boundary(&parent.local_name, &child, parent.boundary);
                        let xml =
                            capture_empty_element(&element).map_err(|error| error.to_string())?;
                        record_preserved_region(&mut parents, &mut preserved, boundary, xml);
                    }
                    advance_preservation_boundary(&mut parents, &child);
                }
                Ok(
                    event @ (Event::Comment(_)
                    | Event::PI(_)
                    | Event::CData(_)
                    | Event::GeneralRef(_)),
                ) if !parents.is_empty() => {
                    let boundary = parents.last().expect("preservation parent").boundary;
                    let xml = capture_event(event).map_err(|error| error.to_string())?;
                    record_preserved_region(&mut parents, &mut preserved, boundary, xml);
                }
                Ok(Event::End(_)) => {
                    parents.pop();
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(error) => return Err(error.to_string()),
            }
            buffer.clear();
        }
        Ok(preserved)
    }

    fn record_preserved_region(
        parents: &mut [PreservationFrame],
        preserved: &mut Vec<PreservedChartRegion>,
        boundary: usize,
        xml: Vec<u8>,
    ) {
        let sibling_order = parents.last().expect("preservation parent").next_sibling;
        parents
            .last_mut()
            .expect("preservation parent")
            .next_sibling += 1;
        preserved.push(PreservedChartRegion {
            parent_path: parents
                .iter()
                .map(|parent| parent.local_name.clone())
                .collect(),
            boundary,
            sibling_order,
            xml,
        });
    }

    fn advance_preservation_boundary(parents: &mut [PreservationFrame], child: &[u8]) {
        let Some(parent) = parents.last_mut() else {
            return;
        };
        parent.boundary = parent
            .boundary
            .max(preserved_boundary_after(&parent.local_name, child));
    }

    fn preserved_child_boundary(parent: &[u8], child: &[u8], current: usize) -> usize {
        match parent {
            b"chartSpace" => match child {
                b"date1904" => 0,
                b"lang" => 1,
                b"roundedCorners" => 2,
                b"style" => 3,
                b"clrMapOvr" => 4,
                b"pivotSource" => 5,
                b"protection" => 6,
                b"externalData" => 10,
                b"printSettings" => 11,
                b"userShapes" => 12,
                b"extLst" => 13,
                _ => current,
            },
            b"chart" => match child {
                b"pivotFmts" => 2,
                b"view3D" => 3,
                b"floor" => 4,
                b"sideWall" => 5,
                b"backWall" => 6,
                b"showDLblsOverMax" | b"extLst" => 11,
                _ => current,
            },
            _ => current,
        }
    }

    fn preserved_boundary_after(parent: &[u8], child: &[u8]) -> usize {
        match parent {
            b"chartSpace" => match child {
                b"date1904" => 1,
                b"lang" => 2,
                b"roundedCorners" => 3,
                b"style" => 4,
                b"clrMapOvr" => 5,
                b"pivotSource" => 6,
                b"protection" => 7,
                b"chart" => 8,
                b"spPr" => 9,
                b"txPr" => 10,
                b"externalData" => 11,
                b"printSettings" => 12,
                b"userShapes" => 13,
                b"extLst" => 14,
                _ => 0,
            },
            b"chart" => match child {
                b"title" => 1,
                b"autoTitleDeleted" => 2,
                b"pivotFmts" => 3,
                b"view3D" => 4,
                b"floor" => 5,
                b"sideWall" => 6,
                b"backWall" => 7,
                b"plotArea" => 8,
                b"legend" => 9,
                b"plotVisOnly" => 10,
                b"dispBlanksAs" => 11,
                b"showDLblsOverMax" | b"extLst" => 11,
                _ => 0,
            },
            _ => 0,
        }
    }

    fn is_opaque_chartml_child(parent: &[u8], child: &[u8]) -> bool {
        match parent {
            b"chartSpace" => !matches!(child, b"chart" | b"spPr" | b"txPr"),
            b"chart" => !matches!(
                child,
                b"title"
                    | b"autoTitleDeleted"
                    | b"plotArea"
                    | b"legend"
                    | b"plotVisOnly"
                    | b"dispBlanksAs"
            ),
            b"title" | b"plotArea" | b"legend" => true,
            _ => false,
        }
    }

    fn is_chartml_preservation_parent(parent: &[u8]) -> bool {
        matches!(
            parent,
            b"chartSpace"
                | b"chart"
                | b"title"
                | b"plotArea"
                | b"legend"
                | b"autoTitleDeleted"
                | b"plotVisOnly"
                | b"dispBlanksAs"
        )
    }

    #[test]
    fn rpptx_chart_is_an_unpublished_workspace_member() {
        let manifest = include_str!("../Cargo.toml");
        let workspace = include_str!("../../../Cargo.toml");
        assert!(manifest.contains("name = \"rpptx-chart\""));
        assert!(manifest.contains("version = \"0.0.0\""));
        assert!(manifest.contains("publish = false"));
        for dependency in [
            "oxml-core.workspace",
            "oxml-drawing.workspace",
            "quick-xml.workspace",
        ] {
            assert!(manifest.contains(dependency));
        }
        assert!(!manifest.contains("rpptx.workspace"));
        assert!(!manifest.contains("rdocx.workspace"));
        assert!(workspace.contains("\"crates/rpptx-chart\""));
        assert!(
            workspace
                .contains("rpptx-chart = { path = \"crates/rpptx-chart\", version = \"0.0.0\" }")
        );
    }

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn corpus_dir() -> PathBuf {
        std::env::var_os("RDOCX_PPTX_CORPUS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| workspace_root().join("corpus/pptx"))
    }

    fn require_or_skip_corpus() -> Option<PathBuf> {
        let corpus = corpus_dir();
        if corpus.is_dir() {
            return Some(corpus);
        }
        assert_ne!(
            std::env::var_os("RDOCX_PPTX_CORPUS_REQUIRED").as_deref(),
            Some(std::ffi::OsStr::new("1")),
            "the pinned corpus is required but {} does not exist",
            corpus.display()
        );
        eprintln!(
            "ChartML corpus gate skipped because {} is absent",
            corpus.display()
        );
        None
    }

    fn verify_fetched_corpus(corpus: &Path) {
        let status = Command::new("python3")
            .arg(workspace_root().join("scripts/fetch_pptx_corpus.py"))
            .arg("--check")
            .env("RDOCX_PPTX_CORPUS_DIR", corpus)
            .status()
            .expect("run corpus verifier");
        assert!(status.success(), "pinned corpus verification failed");
    }

    fn manifest_paths() -> Vec<&'static str> {
        let mut lines = MANIFEST.lines();
        assert_eq!(lines.next(), Some("path\tproducer\tsha256\turl"));
        let paths: Vec<_> = lines
            .map(|line| line.split('\t').next().expect("manifest path"))
            .collect();
        assert_eq!(paths.len(), EXPECTED_DECKS);
        paths
    }

    fn is_chart_part(part: &str) -> bool {
        let Some(stem) = part
            .strip_prefix("/ppt/charts/chart")
            .and_then(|name| name.strip_suffix(".xml"))
        else {
            return false;
        };
        !stem.is_empty() && stem.bytes().all(|byte| byte.is_ascii_digit())
    }
}
