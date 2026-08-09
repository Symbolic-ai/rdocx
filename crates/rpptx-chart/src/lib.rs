#![allow(non_camel_case_types)]

use std::fmt;
use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_core::xml::{local_name, matches_local_name};
use oxml_drawing::order::OrderedRawChildren;
use oxml_drawing::shape_props::{CT_ShapeProperties, ShapePropertiesError};
use oxml_drawing::text::{CT_TextBody, TextError};
use quick_xml::events::{BytesEnd, BytesStart, Event};
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
    fn from_xml(xml: &[u8]) -> Result<Self> {
        let (raw_attributes, raw_children) = parse_raw_shell(xml, b"plotArea", "c:plotArea")?;
        Ok(Self {
            raw_attributes,
            raw_children,
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
                    state.parse_child(&name, raw)?;
                }
                Event::Empty(element) => {
                    let name = if element_is_in_namespace(&element, C_NS, &namespaces)? {
                        local_name(element.name().as_ref()).to_vec()
                    } else {
                        Vec::new()
                    };
                    let raw = capture_empty_element(&element)?;
                    state.parse_child(&name, raw)?;
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

    fn parse_child(&mut self, name: &[u8], raw: Vec<u8>) -> Result<()> {
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
                    CT_PlotArea::from_xml(&raw)?,
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
        bindings.retain(|(current, _)| current != &prefix);
        bindings.push((prefix, namespace));
    }
    Ok(bindings)
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
        CT_ChartSpace, CT_ShapeProperties, CT_TextBody, DispBlanksAs, capture_event, local_name,
    };

    const MANIFEST: &str = include_str!("../../../scripts/pptx-corpus-manifest.tsv");
    const EXPECTED_DECKS: usize = 50;

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
