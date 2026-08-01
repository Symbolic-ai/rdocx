use std::fmt;
use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_core::xml::{get_attr, local_name, matches_local_name};
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::fill::{Fill, FillError};
use crate::order::OrderedRawChildren;

const MAX_LINE_WIDTH_EMU: u32 = 20_116_800;

/// Errors produced while parsing or writing DrawingML line properties.
#[derive(Debug)]
pub enum LineError {
    Xml(OxmlError),
    Fill(FillError),
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
    LineWidthOutOfRange(u32),
}

impl fmt::Display for LineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml(error) => error.fmt(formatter),
            Self::Fill(error) => error.fmt(formatter),
            Self::UnexpectedElement(element) => {
                write!(formatter, "unexpected DrawingML line element: {element}")
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
            Self::LineWidthOutOfRange(value) => write!(
                formatter,
                "DrawingML line width is outside 0 to {MAX_LINE_WIDTH_EMU}: {value}"
            ),
        }
    }
}

impl std::error::Error for LineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml(error) => Some(error),
            Self::Fill(error) => Some(error),
            _ => None,
        }
    }
}

impl From<OxmlError> for LineError {
    fn from(error: OxmlError) -> Self {
        Self::Xml(error)
    }
}

impl From<FillError> for LineError {
    fn from(error: FillError) -> Self {
        Self::Fill(error)
    }
}

pub type Result<T> = std::result::Result<T, LineError>;

/// DrawingML line cap behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineCap {
    Round,
    Square,
    Flat,
}

impl LineCap {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "rnd" => Some(Self::Round),
            "sq" => Some(Self::Square),
            "flat" => Some(Self::Flat),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Round => "rnd",
            Self::Square => "sq",
            Self::Flat => "flat",
        }
    }
}

/// The eleven values in `ST_PresetLineDashVal`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ST_PresetLineDashVal {
    Solid,
    Dot,
    SystemDot,
    Dash,
    SystemDash,
    LargeDash,
    DashDot,
    SystemDashDot,
    LargeDashDot,
    LargeDashDotDot,
    SystemDashDotDot,
}

impl ST_PresetLineDashVal {
    pub const ALL: [Self; 11] = [
        Self::Solid,
        Self::Dot,
        Self::SystemDot,
        Self::Dash,
        Self::SystemDash,
        Self::LargeDash,
        Self::DashDot,
        Self::SystemDashDot,
        Self::LargeDashDot,
        Self::LargeDashDotDot,
        Self::SystemDashDotDot,
    ];

    fn parse(value: &str) -> Option<Self> {
        match value {
            "solid" => Some(Self::Solid),
            "dot" => Some(Self::Dot),
            "sysDot" => Some(Self::SystemDot),
            "dash" => Some(Self::Dash),
            "sysDash" => Some(Self::SystemDash),
            "lgDash" => Some(Self::LargeDash),
            "dashDot" => Some(Self::DashDot),
            "sysDashDot" => Some(Self::SystemDashDot),
            "lgDashDot" => Some(Self::LargeDashDot),
            "lgDashDotDot" => Some(Self::LargeDashDotDot),
            "sysDashDotDot" => Some(Self::SystemDashDotDot),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Dot => "dot",
            Self::SystemDot => "sysDot",
            Self::Dash => "dash",
            Self::SystemDash => "sysDash",
            Self::LargeDash => "lgDash",
            Self::DashDot => "dashDot",
            Self::SystemDashDot => "sysDashDot",
            Self::LargeDashDot => "lgDashDot",
            Self::LargeDashDotDot => "lgDashDotDot",
            Self::SystemDashDotDot => "sysDashDotDot",
        }
    }

    /// Returns alternating painted and unpainted lengths relative to line width.
    pub const fn dash_array(self) -> &'static [u16] {
        match self {
            Self::Solid => &[],
            Self::Dot | Self::SystemDot => &[1, 1],
            Self::Dash => &[4, 3],
            Self::SystemDash => &[3, 1],
            Self::LargeDash => &[8, 3],
            Self::DashDot => &[4, 3, 1, 3],
            Self::SystemDashDot => &[3, 1, 1, 1],
            Self::LargeDashDot => &[8, 3, 1, 3],
            Self::LargeDashDotDot => &[8, 3, 1, 3, 1, 3],
            Self::SystemDashDotDot => &[3, 1, 1, 1, 1, 1],
        }
    }
}

/// One custom painted and unpainted dash pair, in thousandths of a percent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DashStop {
    pub dash: i32,
    pub space: i32,
    raw_children: OrderedRawChildren,
}

impl DashStop {
    pub fn new(dash: i32, space: i32) -> Result<Self> {
        validate_positive_percentage("ds", "d", dash)?;
        validate_positive_percentage("ds", "sp", space)?;
        Ok(Self {
            dash,
            space,
            raw_children: OrderedRawChildren::default(),
        })
    }
}

/// A preset line dash and any extension children inside it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresetDash {
    pub value: ST_PresetLineDashVal,
    raw_children: OrderedRawChildren,
}

impl PresetDash {
    pub fn new(value: ST_PresetLineDashVal) -> Self {
        Self {
            value,
            raw_children: OrderedRawChildren::default(),
        }
    }
}

/// Custom line dash stops in document order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CustomDash {
    pub stops: Vec<DashStop>,
    raw_children: OrderedRawChildren,
}

/// Either a DrawingML preset or custom line dash.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LineDash {
    Preset(PresetDash),
    Custom(CustomDash),
}

/// DrawingML line join behavior.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LineJoin {
    Round {
        raw_children: OrderedRawChildren,
    },
    Bevel {
        raw_children: OrderedRawChildren,
    },
    Miter {
        limit: Option<i32>,
        raw_children: OrderedRawChildren,
    },
}

impl LineJoin {
    pub fn round() -> Self {
        Self::Round {
            raw_children: OrderedRawChildren::default(),
        }
    }

    pub fn bevel() -> Self {
        Self::Bevel {
            raw_children: OrderedRawChildren::default(),
        }
    }

    pub fn miter(limit: Option<i32>) -> Result<Self> {
        if let Some(limit) = limit {
            validate_positive_percentage("miter", "lim", limit)?;
        }
        Ok(Self::Miter {
            limit,
            raw_children: OrderedRawChildren::default(),
        })
    }
}

/// DrawingML line endpoint shape.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineEndType {
    None,
    Triangle,
    Stealth,
    Diamond,
    Oval,
    Arrow,
}

impl LineEndType {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "none" => Some(Self::None),
            "triangle" => Some(Self::Triangle),
            "stealth" => Some(Self::Stealth),
            "diamond" => Some(Self::Diamond),
            "oval" => Some(Self::Oval),
            "arrow" => Some(Self::Arrow),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Triangle => "triangle",
            Self::Stealth => "stealth",
            Self::Diamond => "diamond",
            Self::Oval => "oval",
            Self::Arrow => "arrow",
        }
    }
}

/// DrawingML line endpoint width or length.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineEndSize {
    Small,
    Medium,
    Large,
}

impl LineEndSize {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "sm" => Some(Self::Small),
            "med" => Some(Self::Medium),
            "lg" => Some(Self::Large),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Small => "sm",
            Self::Medium => "med",
            Self::Large => "lg",
        }
    }
}

/// One head or tail endpoint and its optional dimensions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LineEnd {
    pub kind: Option<LineEndType>,
    pub width: Option<LineEndSize>,
    pub length: Option<LineEndSize>,
    raw_children: OrderedRawChildren,
}

/// DrawingML `a:ln` properties.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CT_LineProperties {
    pub width: Option<u32>,
    pub cap: Option<LineCap>,
    pub fill: Option<Fill>,
    pub dash: Option<LineDash>,
    pub join: Option<LineJoin>,
    pub head_end: Option<LineEnd>,
    pub tail_end: Option<LineEnd>,
    raw_children: OrderedRawChildren,
}

impl CT_LineProperties {
    /// Parses one complete `a:ln` element with any namespace prefix.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) if matches_local_name(element.name().as_ref(), b"ln") => {
                    return Self::from_element(&mut reader, &element);
                }
                Event::Empty(element) if matches_local_name(element.name().as_ref(), b"ln") => {
                    return Self::from_start(&element);
                }
                Event::Start(element) | Event::Empty(element) => {
                    return Err(unexpected(&element));
                }
                Event::Eof => {
                    return Err(LineError::Xml(OxmlError::MissingElement(
                        "DrawingML line properties".to_owned(),
                    )));
                }
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_start(start: &BytesStart<'_>) -> Result<Self> {
        let width = optional_parse(start, b"w")?;
        if let Some(width) = width {
            validate_line_width(width)?;
        }
        Ok(Self {
            width,
            cap: optional_enum(start, b"cap", LineCap::parse)?,
            ..Self::default()
        })
    }

    fn from_element(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut line = Self::from_start(start)?;
        let mut boundary = 0;
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) if is_fill(element.name().as_ref()) => {
                    line.fill = Some(Fill::from_element(reader, &element)?);
                    boundary = 1;
                }
                Event::Empty(element) if is_fill(element.name().as_ref()) => {
                    line.fill = Some(Fill::from_empty_element(&element)?);
                    boundary = 1;
                }
                Event::Start(element)
                    if matches_local_name(element.name().as_ref(), b"prstDash") =>
                {
                    line.dash = Some(LineDash::Preset(parse_preset_dash(reader, &element)?));
                    boundary = 2;
                }
                Event::Empty(element)
                    if matches_local_name(element.name().as_ref(), b"prstDash") =>
                {
                    line.dash = Some(LineDash::Preset(parse_empty_preset_dash(&element)?));
                    boundary = 2;
                }
                Event::Start(element)
                    if matches_local_name(element.name().as_ref(), b"custDash") =>
                {
                    line.dash = Some(LineDash::Custom(parse_custom_dash(reader)?));
                    boundary = 2;
                }
                Event::Empty(element)
                    if matches_local_name(element.name().as_ref(), b"custDash") =>
                {
                    line.dash = Some(LineDash::Custom(CustomDash::default()));
                    boundary = 2;
                }
                Event::Start(element) if is_join(element.name().as_ref()) => {
                    line.join = Some(parse_join(reader, &element)?);
                    boundary = 3;
                }
                Event::Empty(element) if is_join(element.name().as_ref()) => {
                    line.join = Some(parse_empty_join(&element)?);
                    boundary = 3;
                }
                Event::Start(element)
                    if matches_local_name(element.name().as_ref(), b"headEnd") =>
                {
                    line.head_end = Some(parse_line_end(reader, &element)?);
                    boundary = 4;
                }
                Event::Empty(element)
                    if matches_local_name(element.name().as_ref(), b"headEnd") =>
                {
                    line.head_end = Some(parse_empty_line_end(&element)?);
                    boundary = 4;
                }
                Event::Start(element)
                    if matches_local_name(element.name().as_ref(), b"tailEnd") =>
                {
                    line.tail_end = Some(parse_line_end(reader, &element)?);
                    boundary = 5;
                }
                Event::Empty(element)
                    if matches_local_name(element.name().as_ref(), b"tailEnd") =>
                {
                    line.tail_end = Some(parse_empty_line_end(&element)?);
                    boundary = 5;
                }
                Event::Start(element) => line
                    .raw_children
                    .push(boundary, capture_element(reader, &element)?),
                Event::Empty(element) => line
                    .raw_children
                    .push(boundary, capture_empty_element(&element)?),
                Event::End(element) if matches_local_name(element.name().as_ref(), b"ln") => break,
                Event::Eof => return Err(missing_end("ln")),
                _ => {}
            }
            buffer.clear();
        }
        Ok(line)
    }

    /// Writes this line with canonical DrawingML prefixes and schema order.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        self.write_xml(&mut writer)?;
        Ok(writer.into_inner())
    }

    /// Writes this line into an existing XML writer.
    pub fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        if let Some(width) = self.width {
            validate_line_width(width)?;
        }
        if matches!(self.fill.as_ref(), Some(Fill::Blip(_))) {
            return Err(LineError::UnexpectedElement("a:blipFill".to_owned()));
        }
        let mut start = BytesStart::new("a:ln");
        let width = self.width.map(|value| value.to_string());
        if let Some(width) = width.as_deref() {
            start.push_attribute(("w", width));
        }
        if let Some(cap) = self.cap {
            start.push_attribute(("cap", cap.as_str()));
        }
        if self.fill.is_none()
            && self.dash.is_none()
            && self.join.is_none()
            && self.head_end.is_none()
            && self.tail_end.is_none()
            && self.raw_children.is_empty()
        {
            return write_empty(writer, start);
        }

        write_start(writer, start)?;
        emit_raw(writer, self.raw_children.at(0))?;
        if let Some(fill) = &self.fill {
            fill.write_xml(writer)?;
        }
        emit_raw(writer, self.raw_children.at(1))?;
        if let Some(dash) = &self.dash {
            write_dash(writer, dash)?;
        }
        emit_raw(writer, self.raw_children.at(2))?;
        if let Some(join) = &self.join {
            write_join(writer, join)?;
        }
        emit_raw(writer, self.raw_children.at(3))?;
        if let Some(end) = &self.head_end {
            write_line_end(writer, "a:headEnd", end)?;
        }
        emit_raw(writer, self.raw_children.at(4))?;
        if let Some(end) = &self.tail_end {
            write_line_end(writer, "a:tailEnd", end)?;
        }
        emit_raw(writer, self.raw_children.at(5))?;
        write_end(writer, "a:ln")
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }
}

fn parse_empty_preset_dash(start: &BytesStart<'_>) -> Result<PresetDash> {
    let value = get_attr(start, b"val").unwrap_or_else(|| "solid".to_owned());
    let value = ST_PresetLineDashVal::parse(&value).ok_or_else(|| invalid(start, b"val", value))?;
    Ok(PresetDash::new(value))
}

fn parse_preset_dash(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<PresetDash> {
    let mut dash = parse_empty_preset_dash(start)?;
    dash.raw_children = capture_all_children(reader, b"prstDash")?;
    Ok(dash)
}

fn parse_custom_dash(reader: &mut Reader<&[u8]>) -> Result<CustomDash> {
    let mut dash = CustomDash::default();
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) if matches_local_name(element.name().as_ref(), b"ds") => {
                let mut stop = parse_empty_dash_stop(&element)?;
                stop.raw_children = capture_all_children(reader, b"ds")?;
                dash.stops.push(stop);
            }
            Event::Empty(element) if matches_local_name(element.name().as_ref(), b"ds") => {
                dash.stops.push(parse_empty_dash_stop(&element)?);
            }
            Event::Start(element) => dash
                .raw_children
                .push(dash.stops.len(), capture_element(reader, &element)?),
            Event::Empty(element) => dash
                .raw_children
                .push(dash.stops.len(), capture_empty_element(&element)?),
            Event::End(element) if matches_local_name(element.name().as_ref(), b"custDash") => {
                break;
            }
            Event::Eof => return Err(missing_end("custDash")),
            _ => {}
        }
        buffer.clear();
    }
    Ok(dash)
}

fn parse_empty_dash_stop(start: &BytesStart<'_>) -> Result<DashStop> {
    DashStop::new(required_parse(start, b"d")?, required_parse(start, b"sp")?)
}

fn write_dash<W: Write>(writer: &mut Writer<W>, dash: &LineDash) -> Result<()> {
    match dash {
        LineDash::Preset(dash) => {
            let mut start = BytesStart::new("a:prstDash");
            start.push_attribute(("val", dash.value.as_str()));
            if dash.raw_children.is_empty() {
                return write_empty(writer, start);
            }
            write_start(writer, start)?;
            emit_raw(writer, dash.raw_children.at(0))?;
            write_end(writer, "a:prstDash")
        }
        LineDash::Custom(dash) => {
            if dash.stops.is_empty() && dash.raw_children.is_empty() {
                return write_empty(writer, BytesStart::new("a:custDash"));
            }
            write_start(writer, BytesStart::new("a:custDash"))?;
            for boundary in 0..=dash.stops.len() {
                emit_raw(writer, dash.raw_children.at(boundary))?;
                if let Some(stop) = dash.stops.get(boundary) {
                    write_dash_stop(writer, stop)?;
                }
            }
            write_end(writer, "a:custDash")
        }
    }
}

fn write_dash_stop<W: Write>(writer: &mut Writer<W>, stop: &DashStop) -> Result<()> {
    validate_positive_percentage("ds", "d", stop.dash)?;
    validate_positive_percentage("ds", "sp", stop.space)?;
    let dash = stop.dash.to_string();
    let space = stop.space.to_string();
    let mut start = BytesStart::new("a:ds");
    start.push_attribute(("d", dash.as_str()));
    start.push_attribute(("sp", space.as_str()));
    if stop.raw_children.is_empty() {
        return write_empty(writer, start);
    }
    write_start(writer, start)?;
    emit_raw(writer, stop.raw_children.at(0))?;
    write_end(writer, "a:ds")
}

fn parse_empty_join(start: &BytesStart<'_>) -> Result<LineJoin> {
    match local_name(start.name().as_ref()) {
        b"round" => Ok(LineJoin::round()),
        b"bevel" => Ok(LineJoin::bevel()),
        b"miter" => LineJoin::miter(optional_parse(start, b"lim")?),
        _ => Err(unexpected(start)),
    }
}

fn parse_join(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<LineJoin> {
    let mut join = parse_empty_join(start)?;
    let raw = capture_all_children(reader, local_name(start.name().as_ref()))?;
    match &mut join {
        LineJoin::Round { raw_children }
        | LineJoin::Bevel { raw_children }
        | LineJoin::Miter { raw_children, .. } => *raw_children = raw,
    }
    Ok(join)
}

fn write_join<W: Write>(writer: &mut Writer<W>, join: &LineJoin) -> Result<()> {
    let (name, limit, raw_children) = match join {
        LineJoin::Round { raw_children } => ("a:round", None, raw_children),
        LineJoin::Bevel { raw_children } => ("a:bevel", None, raw_children),
        LineJoin::Miter {
            limit,
            raw_children,
        } => ("a:miter", *limit, raw_children),
    };
    if let Some(limit) = limit {
        validate_positive_percentage("miter", "lim", limit)?;
    }
    let limit_text = limit.map(|value| value.to_string());
    let mut start = BytesStart::new(name);
    if let Some(limit) = limit_text.as_deref() {
        start.push_attribute(("lim", limit));
    }
    if raw_children.is_empty() {
        return write_empty(writer, start);
    }
    write_start(writer, start)?;
    emit_raw(writer, raw_children.at(0))?;
    write_end(writer, name)
}

fn parse_empty_line_end(start: &BytesStart<'_>) -> Result<LineEnd> {
    Ok(LineEnd {
        kind: optional_enum(start, b"type", LineEndType::parse)?,
        width: optional_enum(start, b"w", LineEndSize::parse)?,
        length: optional_enum(start, b"len", LineEndSize::parse)?,
        raw_children: OrderedRawChildren::default(),
    })
}

fn parse_line_end(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<LineEnd> {
    let mut end = parse_empty_line_end(start)?;
    end.raw_children = capture_all_children(reader, local_name(start.name().as_ref()))?;
    Ok(end)
}

fn write_line_end<W: Write>(writer: &mut Writer<W>, name: &str, end: &LineEnd) -> Result<()> {
    let mut start = BytesStart::new(name);
    if let Some(kind) = end.kind {
        start.push_attribute(("type", kind.as_str()));
    }
    if let Some(width) = end.width {
        start.push_attribute(("w", width.as_str()));
    }
    if let Some(length) = end.length {
        start.push_attribute(("len", length.as_str()));
    }
    if end.raw_children.is_empty() {
        return write_empty(writer, start);
    }
    write_start(writer, start)?;
    emit_raw(writer, end.raw_children.at(0))?;
    write_end(writer, name)
}

fn capture_all_children(reader: &mut Reader<&[u8]>, end_name: &[u8]) -> Result<OrderedRawChildren> {
    let mut raw_children = OrderedRawChildren::default();
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) => raw_children.push(0, capture_element(reader, &element)?),
            Event::Empty(element) => raw_children.push(0, capture_empty_element(&element)?),
            Event::End(element) if matches_local_name(element.name().as_ref(), end_name) => break,
            Event::Eof => return Err(missing_end(&String::from_utf8_lossy(end_name))),
            _ => {}
        }
        buffer.clear();
    }
    Ok(raw_children)
}

fn is_fill(name: &[u8]) -> bool {
    matches!(
        local_name(name),
        b"noFill" | b"solidFill" | b"gradFill" | b"pattFill"
    )
}

fn is_join(name: &[u8]) -> bool {
    matches!(local_name(name), b"round" | b"bevel" | b"miter")
}

fn validate_line_width(value: u32) -> Result<()> {
    if value <= MAX_LINE_WIDTH_EMU {
        Ok(())
    } else {
        Err(LineError::LineWidthOutOfRange(value))
    }
}

fn validate_positive_percentage(element: &str, attribute: &str, value: i32) -> Result<()> {
    if value > 0 {
        Ok(())
    } else {
        Err(LineError::InvalidAttribute {
            element: element.to_owned(),
            attribute: attribute.to_owned(),
            value: value.to_string(),
        })
    }
}

fn required_parse<T: std::str::FromStr>(start: &BytesStart<'_>, name: &[u8]) -> Result<T> {
    let value = get_attr(start, name).ok_or_else(|| LineError::MissingAttribute {
        element: String::from_utf8_lossy(local_name(start.name().as_ref())).into_owned(),
        attribute: String::from_utf8_lossy(name).into_owned(),
    })?;
    value.parse().map_err(|_| invalid(start, name, value))
}

fn optional_parse<T: std::str::FromStr>(start: &BytesStart<'_>, name: &[u8]) -> Result<Option<T>> {
    get_attr(start, name)
        .map(|value| value.parse().map_err(|_| invalid(start, name, value)))
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

fn invalid(start: &BytesStart<'_>, attribute: &[u8], value: String) -> LineError {
    LineError::InvalidAttribute {
        element: String::from_utf8_lossy(local_name(start.name().as_ref())).into_owned(),
        attribute: String::from_utf8_lossy(attribute).into_owned(),
        value,
    }
}

fn unexpected(start: &BytesStart<'_>) -> LineError {
    LineError::UnexpectedElement(String::from_utf8_lossy(start.name().as_ref()).into_owned())
}

fn missing_end(name: &str) -> LineError {
    LineError::Xml(OxmlError::MissingElement(format!("closing a:{name}")))
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
    use super::{
        CT_LineProperties, LineCap, LineDash, LineEndSize, LineEndType, LineError, LineJoin,
        ST_PresetLineDashVal,
    };

    #[test]
    fn every_preset_line_dash_value_maps_to_a_dash_array() {
        let expected: &[(&str, &[u16])] = &[
            ("solid", &[]),
            ("dot", &[1, 1]),
            ("sysDot", &[1, 1]),
            ("dash", &[4, 3]),
            ("sysDash", &[3, 1]),
            ("lgDash", &[8, 3]),
            ("dashDot", &[4, 3, 1, 3]),
            ("sysDashDot", &[3, 1, 1, 1]),
            ("lgDashDot", &[8, 3, 1, 3]),
            ("lgDashDotDot", &[8, 3, 1, 3, 1, 3]),
            ("sysDashDotDot", &[3, 1, 1, 1, 1, 1]),
        ];

        assert_eq!(ST_PresetLineDashVal::ALL.len(), expected.len());
        for (value, (token, dash_array)) in ST_PresetLineDashVal::ALL.iter().zip(expected) {
            assert_eq!(value.as_str(), *token);
            assert_eq!(value.dash_array(), *dash_array);
        }
    }

    #[test]
    fn line_properties_round_trip_width_fill_dash_cap_join_and_ends() {
        let xml = br#"<z:ln w="12700" cap="rnd"><z:solidFill><z:schemeClr val="accent1"/></z:solidFill><z:custDash><z:ds d="200000" sp="100000"/><z:ds d="50000" sp="25000"/></z:custDash><z:miter lim="800000"/><z:headEnd type="triangle" w="lg" len="sm"/><z:tailEnd type="oval" w="med" len="lg"/></z:ln>"#;
        let parsed = CT_LineProperties::from_xml(xml).unwrap();

        assert_eq!(parsed.width, Some(12_700));
        assert_eq!(parsed.cap, Some(LineCap::Round));
        assert!(matches!(&parsed.dash, Some(LineDash::Custom(dash)) if dash.stops.len() == 2));
        assert!(matches!(
            parsed.join,
            Some(LineJoin::Miter {
                limit: Some(800_000),
                ..
            })
        ));
        assert!(
            matches!(parsed.head_end, Some(ref end) if end.kind == Some(LineEndType::Triangle) && end.width == Some(LineEndSize::Large) && end.length == Some(LineEndSize::Small))
        );
        assert!(
            matches!(parsed.tail_end, Some(ref end) if end.kind == Some(LineEndType::Oval) && end.width == Some(LineEndSize::Medium) && end.length == Some(LineEndSize::Large))
        );

        let written = parsed.to_xml().unwrap();
        assert_eq!(CT_LineProperties::from_xml(&written).unwrap(), parsed);
    }

    #[test]
    fn line_properties_write_schema_order_and_preserve_unknown_children() {
        let xml = br#"<z:ln w="25400"><x:before x:id="1"/><z:solidFill><z:srgbClr val="102030"/></z:solidFill><x:afterFill><x:item>one &amp; two</x:item><!--note--></x:afterFill><z:prstDash val="lgDashDot"><x:dashExt x:v="kept"/></z:prstDash><x:afterDash/><z:round><x:joinExt/></z:round><z:headEnd type="arrow"><x:headExt/></z:headEnd><x:betweenEnds/><z:tailEnd type="diamond"/><x:after/></z:ln>"#;
        let written = CT_LineProperties::from_xml(xml).unwrap().to_xml().unwrap();

        assert_eq!(written, br#"<a:ln w="25400"><x:before x:id="1"/><a:solidFill><a:srgbClr val="102030"/></a:solidFill><x:afterFill><x:item>one &amp; two</x:item><!--note--></x:afterFill><a:prstDash val="lgDashDot"><x:dashExt x:v="kept"/></a:prstDash><x:afterDash/><a:round><x:joinExt/></a:round><a:headEnd type="arrow"><x:headExt/></a:headEnd><x:betweenEnds/><a:tailEnd type="diamond"/><x:after/></a:ln>"#);
    }

    #[test]
    fn malformed_line_values_return_errors_without_panicking() {
        let cases: &[&[u8]] = &[
            br#"<a:ln w="wide"/>"#,
            br#"<a:ln w="20116801"/>"#,
            br#"<a:ln cap="curved"/>"#,
            br#"<a:ln><a:prstDash val="longer"/></a:ln>"#,
            br#"<a:ln><a:custDash><a:ds sp="100000"/></a:custDash></a:ln>"#,
            br#"<a:ln><a:custDash><a:ds d="0" sp="100000"/></a:custDash></a:ln>"#,
            br#"<a:ln><a:miter lim="0"/></a:ln>"#,
            br#"<a:ln><a:headEnd type="spear"/></a:ln>"#,
            br#"<a:ln><a:tailEnd w="huge"/></a:ln>"#,
        ];
        for xml in cases {
            let result = std::panic::catch_unwind(|| CT_LineProperties::from_xml(xml));
            assert!(
                result.is_ok(),
                "line parser panicked for {}",
                String::from_utf8_lossy(xml)
            );
            assert!(
                result.unwrap().is_err(),
                "malformed line parsed successfully"
            );
        }
        assert!(matches!(
            CT_LineProperties::from_xml(cases[1]),
            Err(LineError::LineWidthOutOfRange(20_116_801))
        ));
    }
}
