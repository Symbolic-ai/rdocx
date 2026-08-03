use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_drawing::color::{ColorChoice, ColorMap, ColorMapSlot, ThemeColorSlot};
use oxml_drawing::fill::Fill;
use oxml_drawing::namespace::A_NS;
use oxml_drawing::order::OrderedRawChildren;
use oxml_drawing::text::CT_TextListStyle;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::namespace::{
    FIXED_MODEL_PREFIXES, NamespaceBindings, P_NS, R_NS, all_attributes, is_fixed_xmlns,
    root_attributes,
};
use crate::shape_tree::CT_ShapeTree;

pub type Result<T> = std::result::Result<T, OxmlError>;
type RawAttributes = Vec<(String, String)>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ColorMapOverrideKind {
    Master,
    Override(ColorMap),
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_ColorMapOverride {
    pub kind: ColorMapOverrideKind,
    raw_attributes: RawAttributes,
    mapping_attributes: RawAttributes,
    mapping_children: OrderedRawChildren,
    raw_children: OrderedRawChildren,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub struct CT_CommonSlideData {
    pub name: Option<String>,
    pub background: Option<CT_Background>,
    pub shape_tree: CT_ShapeTree,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

/// The renderer-visible choice carried by one preserved `p:bg` subtree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BackgroundRendering {
    Properties(Option<Box<Fill>>),
    Reference {
        index: u32,
        color: Option<ColorChoice>,
    },
    Unsupported(&'static str),
}

/// A slide background whose original bytes remain the sole serialisation source.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_Background {
    raw_xml: Vec<u8>,
    rendering: BackgroundRendering,
}

impl CT_Background {
    /// Returns the original subtree used as the sole serialisation source.
    pub fn raw_xml(&self) -> &[u8] {
        &self.raw_xml
    }

    /// Returns the typed projection used only by inheritance resolution.
    pub fn rendering(&self) -> &BackgroundRendering {
        &self.rendering
    }

    fn from_fragment(xml: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) => {
                    let namespaces = inherited.with_start(&start)?;
                    if local_name(start.name().as_ref()) != b"bg"
                        || namespaces.element_uri(start.name().as_ref()) != Some(P_NS)
                    {
                        return Err(unexpected(&start));
                    }
                    let rendering = parse_background_rendering(&mut reader, &namespaces)?;
                    return Ok(Self {
                        raw_xml: xml.to_vec(),
                        rendering,
                    });
                }
                Event::Empty(start) => {
                    return Err(OxmlError::MissingElement(format!(
                        "{} requires p:bgPr or p:bgRef",
                        String::from_utf8_lossy(start.name().as_ref())
                    )));
                }
                Event::Eof => return Err(OxmlError::MissingElement("p:bg".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_MasterTextStyles {
    pub title_style: CT_TextListStyle,
    pub body_style: CT_TextListStyle,
    pub other_style: CT_TextListStyle,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

/// Presence-sensitive header and footer visibility inherited by a slide.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_HeaderFooter {
    pub slide_number: Option<bool>,
    pub header: Option<bool>,
    pub footer: Option<bool>,
    pub date_time: Option<bool>,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

impl CT_HeaderFooter {
    pub fn slide_number_enabled(&self) -> bool {
        self.slide_number.unwrap_or(true)
    }

    pub fn footer_enabled(&self) -> bool {
        self.footer.unwrap_or(true)
    }

    pub fn date_time_enabled(&self) -> bool {
        self.date_time.unwrap_or(true)
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub struct CT_Slide {
    pub common_slide_data: CT_CommonSlideData,
    pub color_map_override: Option<CT_ColorMapOverride>,
    pub show_master_shapes: Option<bool>,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub struct CT_SlideLayout {
    pub common_slide_data: CT_CommonSlideData,
    pub color_map_override: Option<CT_ColorMapOverride>,
    pub show_master_shapes: Option<bool>,
    pub header_footer: Option<CT_HeaderFooter>,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub struct CT_SlideMaster {
    pub common_slide_data: CT_CommonSlideData,
    pub color_map: ColorMap,
    pub text_styles: Option<CT_MasterTextStyles>,
    pub header_footer: Option<CT_HeaderFooter>,
    raw_attributes: RawAttributes,
    color_map_attributes: RawAttributes,
    color_map_children: OrderedRawChildren,
    raw_children: OrderedRawChildren,
}

#[derive(Clone, Copy)]
enum RootKind {
    Slide,
    Layout,
    Master,
}

impl RootKind {
    const fn local_name(self) -> &'static [u8] {
        match self {
            Self::Slide => b"sld",
            Self::Layout => b"sldLayout",
            Self::Master => b"sldMaster",
        }
    }

    const fn tag(self) -> &'static str {
        match self {
            Self::Slide => "p:sld",
            Self::Layout => "p:sldLayout",
            Self::Master => "p:sldMaster",
        }
    }

    fn raw_boundary(self, name: &[u8], current: usize) -> (usize, usize) {
        let before = match self {
            Self::Slide => match name {
                b"transition" => Some(2),
                b"timing" => Some(3),
                b"extLst" => Some(4),
                _ => None,
            },
            Self::Layout => match name {
                b"hf" => Some(2),
                b"timing" => Some(3),
                b"transition" => Some(4),
                b"extLst" => Some(5),
                _ => None,
            },
            Self::Master => match name {
                b"sldLayoutIdLst" => Some(2),
                b"transition" => Some(3),
                b"timing" => Some(4),
                b"hf" => Some(5),
                b"extLst" => Some(7),
                _ => None,
            },
        };
        before.map_or((current, current), |before| {
            (current.max(before), current.max(before + 1))
        })
    }
}

#[derive(Default)]
struct ParsedRoot {
    common_slide_data: Option<CT_CommonSlideData>,
    color_map_override: Option<CT_ColorMapOverride>,
    color_map: Option<ParsedColorMap>,
    text_styles: Option<CT_MasterTextStyles>,
    header_footer: Option<CT_HeaderFooter>,
    show_master_shapes: Option<bool>,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
    boundary: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ParsedColorMap {
    pub(crate) value: ColorMap,
    pub(crate) raw_attributes: RawAttributes,
    pub(crate) raw_children: OrderedRawChildren,
}

impl CT_Slide {
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let parsed = parse_root(xml, RootKind::Slide)?;
        Ok(Self {
            common_slide_data: required(parsed.common_slide_data, "p:cSld")?,
            color_map_override: parsed.color_map_override,
            show_master_shapes: parsed.show_master_shapes,
            raw_attributes: parsed.raw_attributes,
            raw_children: parsed.raw_children,
        })
    }

    pub fn to_xml(&self) -> Result<Vec<u8>> {
        write_slide_like(
            RootKind::Slide,
            &self.common_slide_data,
            self.color_map_override.as_ref(),
            self.show_master_shapes,
            None,
            &self.raw_attributes,
            &self.raw_children,
        )
    }
}

impl CT_SlideLayout {
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let parsed = parse_root(xml, RootKind::Layout)?;
        Ok(Self {
            common_slide_data: required(parsed.common_slide_data, "p:cSld")?,
            color_map_override: parsed.color_map_override,
            show_master_shapes: parsed.show_master_shapes,
            header_footer: parsed.header_footer,
            raw_attributes: parsed.raw_attributes,
            raw_children: parsed.raw_children,
        })
    }

    pub fn to_xml(&self) -> Result<Vec<u8>> {
        write_slide_like(
            RootKind::Layout,
            &self.common_slide_data,
            self.color_map_override.as_ref(),
            self.show_master_shapes,
            self.header_footer.as_ref(),
            &self.raw_attributes,
            &self.raw_children,
        )
    }
}

impl CT_SlideMaster {
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let parsed = parse_root(xml, RootKind::Master)?;
        let color_map = required(parsed.color_map, "p:clrMap")?;
        Ok(Self {
            common_slide_data: required(parsed.common_slide_data, "p:cSld")?,
            color_map: color_map.value,
            text_styles: parsed.text_styles,
            header_footer: parsed.header_footer,
            raw_attributes: parsed.raw_attributes,
            color_map_attributes: color_map.raw_attributes,
            color_map_children: color_map.raw_children,
            raw_children: parsed.raw_children,
        })
    }

    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        write_root_start(&mut writer, RootKind::Master, None, &self.raw_attributes)?;
        emit_raw(&mut writer, self.raw_children.at(0))?;
        self.common_slide_data.write_xml(&mut writer)?;
        emit_raw(&mut writer, self.raw_children.at(1))?;
        write_color_map(
            &mut writer,
            "p:clrMap",
            &self.color_map,
            &self.color_map_attributes,
            &self.color_map_children,
        )?;
        for boundary in 2..=5 {
            emit_raw(&mut writer, self.raw_children.at(boundary))?;
        }
        if let Some(header_footer) = &self.header_footer {
            header_footer.write_xml(&mut writer)?;
        }
        emit_raw(&mut writer, self.raw_children.at(6))?;
        if let Some(styles) = &self.text_styles {
            styles.write_xml(&mut writer)?;
        }
        emit_raw(&mut writer, self.raw_children.at(7))?;
        emit_raw(&mut writer, self.raw_children.at(8))?;
        writer.write_event(Event::End(BytesEnd::new(RootKind::Master.tag())))?;
        Ok(writer.into_inner())
    }
}

fn parse_root(xml: &[u8], kind: RootKind) -> Result<ParsedRoot> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) => {
                let namespaces = NamespaceBindings::default().with_start(&start)?;
                if local_name(start.name().as_ref()) != kind.local_name()
                    || namespaces.element_uri(start.name().as_ref()) != Some(P_NS)
                {
                    return Err(unexpected(&start));
                }
                namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                return parse_root_children(&mut reader, &start, &namespaces, kind);
            }
            Event::Empty(start) => {
                let namespaces = NamespaceBindings::default().with_start(&start)?;
                if local_name(start.name().as_ref()) != kind.local_name()
                    || namespaces.element_uri(start.name().as_ref()) != Some(P_NS)
                {
                    return Err(unexpected(&start));
                }
                return Err(OxmlError::MissingElement(format!(
                    "{} requires p:cSld",
                    kind.tag()
                )));
            }
            Event::Eof => return Err(OxmlError::MissingElement(kind.tag().to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

fn parse_root_children(
    reader: &mut Reader<&[u8]>,
    start: &BytesStart<'_>,
    root_namespaces: &NamespaceBindings,
    kind: RootKind,
) -> Result<ParsedRoot> {
    let mut parsed = ParsedRoot {
        raw_attributes: root_attributes(start, FIXED_MODEL_PREFIXES)?,
        ..ParsedRoot::default()
    };
    if !matches!(kind, RootKind::Master) {
        parsed.show_master_shapes = parse_optional_bool_attribute(start, "showMasterSp")?;
        parsed
            .raw_attributes
            .retain(|(name, _)| name != "showMasterSp");
    }
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(child) => {
                let namespaces = root_namespaces.with_start(&child)?;
                let name = local_name(child.name().as_ref()).to_vec();
                let is_p = namespaces.element_uri(child.name().as_ref()) == Some(P_NS);
                let raw = capture_element(reader, &child)?;
                parsed.capture_child(&name, is_p, &namespaces, raw, kind)?;
            }
            Event::Empty(child) => {
                let namespaces = root_namespaces.with_start(&child)?;
                let name = local_name(child.name().as_ref()).to_vec();
                let is_p = namespaces.element_uri(child.name().as_ref()) == Some(P_NS);
                let raw = capture_empty_element(&child)?;
                parsed.capture_child(&name, is_p, &namespaces, raw, kind)?;
            }
            Event::End(end) if local_name(end.name().as_ref()) == kind.local_name() => {
                return Ok(parsed);
            }
            Event::Eof => return Err(OxmlError::MissingElement(format!("closing {}", kind.tag()))),
            _ => {}
        }
        buffer.clear();
    }
}

impl ParsedRoot {
    fn capture_child(
        &mut self,
        name: &[u8],
        is_p: bool,
        namespaces: &NamespaceBindings,
        raw: Vec<u8>,
        kind: RootKind,
    ) -> Result<()> {
        if is_p && name == b"cSld" {
            namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
            if self.common_slide_data.is_some() {
                return Err(duplicate("cSld"));
            }
            self.common_slide_data = Some(CT_CommonSlideData::from_fragment(&raw, namespaces)?);
            self.boundary = self.boundary.max(1);
            return Ok(());
        }
        if is_p && name == b"clrMapOvr" && !matches!(kind, RootKind::Master) {
            namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
            if self.color_map_override.is_some() {
                return Err(duplicate("clrMapOvr"));
            }
            self.color_map_override = Some(CT_ColorMapOverride::from_fragment(&raw, namespaces)?);
            self.boundary = self.boundary.max(2);
            return Ok(());
        }
        if is_p && name == b"clrMap" && matches!(kind, RootKind::Master) {
            namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
            if self.color_map.is_some() {
                return Err(duplicate("clrMap"));
            }
            self.color_map = Some(parse_color_map(&raw, namespaces)?);
            self.boundary = self.boundary.max(2);
            return Ok(());
        }
        if is_p && name == b"txStyles" && matches!(kind, RootKind::Master) {
            namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
            if self.text_styles.is_some() {
                return Err(duplicate("txStyles"));
            }
            self.text_styles = Some(CT_MasterTextStyles::from_fragment(&raw, namespaces)?);
            self.boundary = self.boundary.max(7);
            return Ok(());
        }
        if is_p && name == b"hf" && matches!(kind, RootKind::Layout | RootKind::Master) {
            namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
            if self.header_footer.is_some() {
                return Err(duplicate("hf"));
            }
            self.header_footer = Some(CT_HeaderFooter::from_fragment(&raw, namespaces)?);
            self.boundary = self.boundary.max(match kind {
                RootKind::Layout => 3,
                RootKind::Master => 6,
                RootKind::Slide => unreachable!(),
            });
            return Ok(());
        }
        let (at, after) = if is_p {
            kind.raw_boundary(name, self.boundary)
        } else {
            (self.boundary, self.boundary)
        };
        self.raw_children.push(at, raw);
        self.boundary = after;
        Ok(())
    }
}

fn write_slide_like(
    kind: RootKind,
    common: &CT_CommonSlideData,
    color_override: Option<&CT_ColorMapOverride>,
    show_master_shapes: Option<bool>,
    header_footer: Option<&CT_HeaderFooter>,
    attributes: &RawAttributes,
    raw: &OrderedRawChildren,
) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Vec::new());
    write_root_start(&mut writer, kind, show_master_shapes, attributes)?;
    emit_raw(&mut writer, raw.at(0))?;
    common.write_xml(&mut writer)?;
    emit_raw(&mut writer, raw.at(1))?;
    if let Some(color_override) = color_override {
        color_override.write_xml(&mut writer)?;
    }
    match kind {
        RootKind::Slide => {
            for boundary in 2..=5 {
                emit_raw(&mut writer, raw.at(boundary))?;
            }
        }
        RootKind::Layout => {
            emit_raw(&mut writer, raw.at(2))?;
            if let Some(header_footer) = header_footer {
                header_footer.write_xml(&mut writer)?;
            }
            for boundary in 3..=6 {
                emit_raw(&mut writer, raw.at(boundary))?;
            }
        }
        RootKind::Master => unreachable!(),
    }
    writer.write_event(Event::End(BytesEnd::new(kind.tag())))?;
    Ok(writer.into_inner())
}

fn write_root_start<W: Write>(
    writer: &mut Writer<W>,
    kind: RootKind,
    show_master_shapes: Option<bool>,
    attributes: &RawAttributes,
) -> Result<()> {
    let mut root = BytesStart::new(kind.tag());
    root.push_attribute(("xmlns:p", P_NS));
    root.push_attribute(("xmlns:a", A_NS));
    root.push_attribute(("xmlns:r", R_NS));
    if let Some(show_master_shapes) = show_master_shapes {
        root.push_attribute(("showMasterSp", if show_master_shapes { "1" } else { "0" }));
    }
    push_attributes(&mut root, attributes);
    writer.write_event(Event::Start(root))?;
    Ok(())
}

impl CT_HeaderFooter {
    fn from_fragment(xml: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) => {
                    let namespaces = inherited.with_start(&start)?;
                    if local_name(start.name().as_ref()) != b"hf"
                        || namespaces.element_uri(start.name().as_ref()) != Some(P_NS)
                    {
                        return Err(unexpected(&start));
                    }
                    let mut value = Self::from_start(&start)?;
                    value.raw_children = capture_shell_children(&mut reader, b"hf")?;
                    return Ok(value);
                }
                Event::Empty(start) => return Self::from_start(&start),
                Event::Eof => return Err(OxmlError::MissingElement("p:hf".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_start(start: &BytesStart<'_>) -> Result<Self> {
        let mut raw_attributes = root_attributes(start, FIXED_MODEL_PREFIXES)?;
        raw_attributes
            .retain(|(name, _)| !matches!(name.as_str(), "sldNum" | "hdr" | "ftr" | "dt"));
        Ok(Self {
            slide_number: parse_optional_bool_attribute(start, "sldNum")?,
            header: parse_optional_bool_attribute(start, "hdr")?,
            footer: parse_optional_bool_attribute(start, "ftr")?,
            date_time: parse_optional_bool_attribute(start, "dt")?,
            raw_attributes,
            raw_children: OrderedRawChildren::default(),
        })
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut start = BytesStart::new("p:hf");
        push_optional_bool_attribute(&mut start, "sldNum", self.slide_number);
        push_optional_bool_attribute(&mut start, "hdr", self.header);
        push_optional_bool_attribute(&mut start, "ftr", self.footer);
        push_optional_bool_attribute(&mut start, "dt", self.date_time);
        push_attributes(&mut start, &self.raw_attributes);
        if self.raw_children.is_empty() {
            writer.write_event(Event::Empty(start))?;
        } else {
            writer.write_event(Event::Start(start))?;
            emit_raw(writer, self.raw_children.at(0))?;
            writer.write_event(Event::End(BytesEnd::new("p:hf")))?;
        }
        Ok(())
    }
}

fn parse_optional_bool_attribute(start: &BytesStart<'_>, name: &str) -> Result<Option<bool>> {
    let Some((_, value)) = all_attributes(start)?
        .into_iter()
        .find(|(attribute, _)| attribute == name)
    else {
        return Ok(None);
    };
    match value.as_str() {
        "true" | "1" => Ok(Some(true)),
        "false" | "0" => Ok(Some(false)),
        _ => Err(OxmlError::InvalidValue(format!(
            "invalid boolean @{name} value {value}"
        ))),
    }
}

fn push_optional_bool_attribute(
    start: &mut BytesStart<'_>,
    name: &'static str,
    value: Option<bool>,
) {
    if let Some(value) = value {
        start.push_attribute((name, if value { "1" } else { "0" }));
    }
}

impl CT_CommonSlideData {
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) => {
                    let namespaces = NamespaceBindings::default().with_start(&start)?;
                    if local_name(start.name().as_ref()) == b"cSld"
                        && namespaces.element_uri(start.name().as_ref()) == Some(P_NS)
                    {
                        return Self::from_reader(&mut reader, &start, &namespaces);
                    }
                    return Err(unexpected(&start));
                }
                Event::Empty(start) => {
                    let namespaces = NamespaceBindings::default().with_start(&start)?;
                    if local_name(start.name().as_ref()) != b"cSld"
                        || namespaces.element_uri(start.name().as_ref()) != Some(P_NS)
                    {
                        return Err(unexpected(&start));
                    }
                    return Err(OxmlError::MissingElement(
                        "p:cSld requires p:spTree".to_owned(),
                    ));
                }
                Event::Eof => return Err(OxmlError::MissingElement("p:cSld".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }

    pub(crate) fn from_fragment(xml: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) => {
                    let namespaces = inherited.with_start(&start)?;
                    return Self::from_reader(&mut reader, &start, &namespaces);
                }
                Event::Eof => return Err(OxmlError::MissingElement("p:cSld".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_reader(
        reader: &mut Reader<&[u8]>,
        start: &BytesStart<'_>,
        namespaces: &NamespaceBindings,
    ) -> Result<Self> {
        namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
        let attributes = all_attributes(start)?;
        let name = attributes
            .iter()
            .find(|(key, _)| key == "name")
            .map(|(_, value)| value.clone());
        let raw_attributes = attributes
            .into_iter()
            .filter(|(key, _)| key != "name" && !is_fixed_xmlns(key, FIXED_MODEL_PREFIXES))
            .collect();
        let mut background = None;
        let mut shape_tree = None;
        let mut raw_children = OrderedRawChildren::default();
        let mut boundary = 0usize;
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(child) => {
                    let child_ns = namespaces.with_start(&child)?;
                    let local = local_name(child.name().as_ref()).to_vec();
                    let is_p = child_ns.element_uri(child.name().as_ref()) == Some(P_NS);
                    let raw = capture_element(reader, &child)?;
                    capture_common_child(
                        &local,
                        is_p,
                        raw,
                        &mut background,
                        &mut shape_tree,
                        &child_ns,
                        &mut raw_children,
                        &mut boundary,
                    )?;
                }
                Event::Empty(child) => {
                    let child_ns = namespaces.with_start(&child)?;
                    let local = local_name(child.name().as_ref()).to_vec();
                    let is_p = child_ns.element_uri(child.name().as_ref()) == Some(P_NS);
                    let raw = capture_empty_element(&child)?;
                    capture_common_child(
                        &local,
                        is_p,
                        raw,
                        &mut background,
                        &mut shape_tree,
                        &child_ns,
                        &mut raw_children,
                        &mut boundary,
                    )?;
                }
                Event::End(end) if local_name(end.name().as_ref()) == b"cSld" => break,
                Event::Eof => return Err(OxmlError::MissingElement("closing p:cSld".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
        Ok(Self {
            name,
            background,
            shape_tree: required(shape_tree, "p:spTree")?,
            raw_attributes,
            raw_children,
        })
    }

    pub(crate) fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut start = BytesStart::new("p:cSld");
        if let Some(name) = &self.name {
            start.push_attribute(("name", name.as_str()));
        }
        push_attributes(&mut start, &self.raw_attributes);
        writer.write_event(Event::Start(start))?;
        emit_raw(writer, self.raw_children.at(0))?;
        if let Some(background) = &self.background {
            writer.get_mut().write_all(background.raw_xml())?;
        }
        emit_raw(writer, self.raw_children.at(1))?;
        self.shape_tree.write_xml(writer)?;
        for boundary in 2..=5 {
            emit_raw(writer, self.raw_children.at(boundary))?;
        }
        writer.write_event(Event::End(BytesEnd::new("p:cSld")))?;
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn capture_common_child(
    name: &[u8],
    is_p: bool,
    raw: Vec<u8>,
    background: &mut Option<CT_Background>,
    shape_tree: &mut Option<CT_ShapeTree>,
    namespaces: &NamespaceBindings,
    children: &mut OrderedRawChildren,
    boundary: &mut usize,
) -> Result<()> {
    if is_p && name == b"bg" {
        let parsed = CT_Background::from_fragment(&raw, namespaces)?;
        if background.replace(parsed).is_some() {
            return Err(duplicate("bg"));
        }
        *boundary = (*boundary).max(1);
    } else if is_p && name == b"spTree" {
        let parsed = CT_ShapeTree::from_fragment(&raw, &namespaces.entries())?;
        if shape_tree.replace(parsed).is_some() {
            return Err(duplicate("spTree"));
        }
        *boundary = (*boundary).max(2);
    } else {
        let before = if is_p {
            match name {
                b"custDataLst" => Some(2),
                b"controls" => Some(3),
                b"extLst" => Some(4),
                _ => None,
            }
        } else {
            None
        };
        if let Some(before) = before {
            let at = (*boundary).max(before);
            children.push(at, raw);
            *boundary = at.max(before + 1);
        } else {
            children.push(*boundary, raw);
        }
    }
    Ok(())
}

fn parse_background_rendering(
    reader: &mut Reader<&[u8]>,
    namespaces: &NamespaceBindings,
) -> Result<BackgroundRendering> {
    let mut rendering = None;
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(child) => {
                let child_ns = namespaces.with_start(&child)?;
                let is_p = child_ns.element_uri(child.name().as_ref()) == Some(P_NS);
                let raw = capture_element(reader, &child)?;
                let parsed = match (is_p, local_name(child.name().as_ref())) {
                    (true, b"bgPr") => Some(parse_background_properties(&raw, &child_ns)?),
                    (true, b"bgRef") => Some(parse_background_reference(&raw, &child_ns)?),
                    _ => None,
                };
                if let Some(parsed) = parsed
                    && rendering.replace(parsed).is_some()
                {
                    return Err(duplicate("background choice"));
                }
            }
            Event::Empty(child) => {
                let child_ns = namespaces.with_start(&child)?;
                let is_p = child_ns.element_uri(child.name().as_ref()) == Some(P_NS);
                let raw = capture_empty_element(&child)?;
                let parsed = match (is_p, local_name(child.name().as_ref())) {
                    (true, b"bgPr") => Some(BackgroundRendering::Properties(None)),
                    (true, b"bgRef") => Some(parse_background_reference(&raw, &child_ns)?),
                    _ => None,
                };
                if let Some(parsed) = parsed
                    && rendering.replace(parsed).is_some()
                {
                    return Err(duplicate("background choice"));
                }
            }
            Event::End(end) if local_name(end.name().as_ref()) == b"bg" => break,
            Event::Eof => return Err(OxmlError::MissingElement("closing p:bg".to_owned())),
            _ => {}
        }
        buffer.clear();
    }
    required(rendering, "p:bgPr or p:bgRef")
}

fn parse_background_properties(
    xml: &[u8],
    inherited: &NamespaceBindings,
) -> Result<BackgroundRendering> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) => {
                let namespaces = inherited.with_start(&start)?;
                if local_name(start.name().as_ref()) != b"bgPr"
                    || namespaces.element_uri(start.name().as_ref()) != Some(P_NS)
                {
                    return Err(unexpected(&start));
                }
                return parse_background_fill_children(&mut reader, &namespaces);
            }
            Event::Empty(_) => return Ok(BackgroundRendering::Properties(None)),
            Event::Eof => return Err(OxmlError::MissingElement("p:bgPr".to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

fn parse_background_fill_children(
    reader: &mut Reader<&[u8]>,
    namespaces: &NamespaceBindings,
) -> Result<BackgroundRendering> {
    let mut fill = None;
    let mut unsupported = false;
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(child) => {
                let child_ns = namespaces.with_start(&child)?;
                let is_drawing = child_ns.element_uri(child.name().as_ref()) == Some(A_NS);
                let qualified_name = child.name();
                let name = local_name(qualified_name.as_ref());
                let is_fill = is_drawing && is_fill_name(name);
                let raw = capture_element(reader, &child)?;
                if is_fill {
                    let parsed = Fill::from_xml(&raw).map_err(map_drawing_error)?;
                    if fill.replace(parsed).is_some() {
                        return Err(duplicate("background fill"));
                    }
                } else if is_drawing && name == b"grpFill" {
                    unsupported = true;
                }
            }
            Event::Empty(child) => {
                let child_ns = namespaces.with_start(&child)?;
                let is_drawing = child_ns.element_uri(child.name().as_ref()) == Some(A_NS);
                let qualified_name = child.name();
                let name = local_name(qualified_name.as_ref());
                if is_drawing && is_fill_name(name) {
                    let raw = capture_empty_element(&child)?;
                    let parsed = Fill::from_xml(&raw).map_err(map_drawing_error)?;
                    if fill.replace(parsed).is_some() {
                        return Err(duplicate("background fill"));
                    }
                } else if is_drawing && name == b"grpFill" {
                    unsupported = true;
                }
            }
            Event::End(end) if local_name(end.name().as_ref()) == b"bgPr" => break,
            Event::Eof => return Err(OxmlError::MissingElement("closing p:bgPr".to_owned())),
            _ => {}
        }
        buffer.clear();
    }
    if unsupported {
        Ok(BackgroundRendering::Unsupported("group fill"))
    } else {
        Ok(BackgroundRendering::Properties(fill.map(Box::new)))
    }
}

fn parse_background_reference(
    xml: &[u8],
    inherited: &NamespaceBindings,
) -> Result<BackgroundRendering> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) => {
                let namespaces = inherited.with_start(&start)?;
                if local_name(start.name().as_ref()) != b"bgRef"
                    || namespaces.element_uri(start.name().as_ref()) != Some(P_NS)
                {
                    return Err(unexpected(&start));
                }
                let index = required_attribute(&start, "idx")?.parse::<u32>()?;
                let (color, unsupported) =
                    parse_background_reference_color(&mut reader, &namespaces)?;
                return if unsupported {
                    Ok(BackgroundRendering::Unsupported("reference colour"))
                } else {
                    Ok(BackgroundRendering::Reference { index, color })
                };
            }
            Event::Empty(start) => {
                let index = required_attribute(&start, "idx")?.parse::<u32>()?;
                return Ok(BackgroundRendering::Reference { index, color: None });
            }
            Event::Eof => return Err(OxmlError::MissingElement("p:bgRef".to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

fn parse_background_reference_color(
    reader: &mut Reader<&[u8]>,
    namespaces: &NamespaceBindings,
) -> Result<(Option<ColorChoice>, bool)> {
    let mut color = None;
    let mut unsupported = false;
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(child) => {
                let child_ns = namespaces.with_start(&child)?;
                let is_drawing = child_ns.element_uri(child.name().as_ref()) == Some(A_NS);
                let qualified_name = child.name();
                let name = local_name(qualified_name.as_ref());
                if is_drawing && is_color_name(name) {
                    let parsed =
                        ColorChoice::from_xml(reader, &child).map_err(map_drawing_error)?;
                    if color.replace(parsed).is_some() {
                        return Err(duplicate("background reference colour"));
                    }
                } else {
                    capture_element(reader, &child)?;
                    unsupported |= is_drawing && is_unmodelled_color_name(name);
                }
            }
            Event::Empty(child) => {
                let child_ns = namespaces.with_start(&child)?;
                let is_drawing = child_ns.element_uri(child.name().as_ref()) == Some(A_NS);
                let qualified_name = child.name();
                let name = local_name(qualified_name.as_ref());
                if is_drawing && is_color_name(name) {
                    let parsed = ColorChoice::from_empty_xml(&child).map_err(map_drawing_error)?;
                    if color.replace(parsed).is_some() {
                        return Err(duplicate("background reference colour"));
                    }
                } else {
                    unsupported |= is_drawing && is_unmodelled_color_name(name);
                }
            }
            Event::End(end) if local_name(end.name().as_ref()) == b"bgRef" => break,
            Event::Eof => return Err(OxmlError::MissingElement("closing p:bgRef".to_owned())),
            _ => {}
        }
        buffer.clear();
    }
    Ok((color, unsupported))
}

fn is_fill_name(name: &[u8]) -> bool {
    matches!(
        name,
        b"noFill" | b"solidFill" | b"gradFill" | b"pattFill" | b"blipFill"
    )
}

fn is_color_name(name: &[u8]) -> bool {
    matches!(name, b"srgbClr" | b"schemeClr" | b"sysClr" | b"prstClr")
}

fn is_unmodelled_color_name(name: &[u8]) -> bool {
    matches!(name, b"scrgbClr" | b"hslClr")
}

fn required_attribute(start: &BytesStart<'_>, name: &str) -> Result<String> {
    all_attributes(start)?
        .into_iter()
        .find_map(|(key, value)| (key == name).then_some(value))
        .ok_or_else(|| {
            OxmlError::MissingElement(format!(
                "{} requires @{name}",
                String::from_utf8_lossy(start.name().as_ref())
            ))
        })
}

impl CT_ColorMapOverride {
    pub(crate) fn from_fragment(xml: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        let start = loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) => break start.into_owned(),
                Event::Eof => return Err(OxmlError::MissingElement("p:clrMapOvr".to_owned())),
                _ => {}
            }
            buffer.clear();
        };
        let namespaces = inherited.with_start(&start)?;
        namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
        let raw_attributes = root_attributes(&start, FIXED_MODEL_PREFIXES)?;
        let mut kind = None;
        let mut mapping_attributes = Vec::new();
        let mut mapping_children = OrderedRawChildren::default();
        let mut raw_children = OrderedRawChildren::default();
        let mut boundary = 0usize;
        loop {
            buffer.clear();
            match reader.read_event_into(&mut buffer)? {
                Event::Start(child) => {
                    let child_ns = namespaces.with_start(&child)?;
                    let local = local_name(child.name().as_ref()).to_vec();
                    let is_a = child_ns.element_uri(child.name().as_ref()) == Some(A_NS);
                    let raw = capture_element(&mut reader, &child)?;
                    capture_override_choice(
                        &local,
                        is_a,
                        raw,
                        &mut kind,
                        &mut mapping_attributes,
                        &mut mapping_children,
                        &mut raw_children,
                        &mut boundary,
                        &child_ns,
                    )?;
                }
                Event::Empty(child) => {
                    let child_ns = namespaces.with_start(&child)?;
                    let local = local_name(child.name().as_ref()).to_vec();
                    let is_a = child_ns.element_uri(child.name().as_ref()) == Some(A_NS);
                    let raw = capture_empty_element(&child)?;
                    capture_override_choice(
                        &local,
                        is_a,
                        raw,
                        &mut kind,
                        &mut mapping_attributes,
                        &mut mapping_children,
                        &mut raw_children,
                        &mut boundary,
                        &child_ns,
                    )?;
                }
                Event::End(end) if local_name(end.name().as_ref()) == b"clrMapOvr" => break,
                Event::Eof => {
                    return Err(OxmlError::MissingElement("closing p:clrMapOvr".to_owned()));
                }
                _ => {}
            }
        }
        Ok(Self {
            kind: required(kind, "a:masterClrMapping or a:overrideClrMapping")?,
            raw_attributes,
            mapping_attributes,
            mapping_children,
            raw_children,
        })
    }

    pub(crate) fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut start = BytesStart::new("p:clrMapOvr");
        push_attributes(&mut start, &self.raw_attributes);
        writer.write_event(Event::Start(start))?;
        emit_raw(writer, self.raw_children.at(0))?;
        match &self.kind {
            ColorMapOverrideKind::Master => {
                write_mapping_shell(
                    writer,
                    "a:masterClrMapping",
                    &self.mapping_attributes,
                    &self.mapping_children,
                )?;
            }
            ColorMapOverrideKind::Override(map) => {
                write_color_map(
                    writer,
                    "a:overrideClrMapping",
                    map,
                    &self.mapping_attributes,
                    &self.mapping_children,
                )?;
            }
        }
        emit_raw(writer, self.raw_children.at(1))?;
        writer.write_event(Event::End(BytesEnd::new("p:clrMapOvr")))?;
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn capture_override_choice(
    name: &[u8],
    is_a: bool,
    raw: Vec<u8>,
    kind: &mut Option<ColorMapOverrideKind>,
    attributes: &mut RawAttributes,
    mapping_children: &mut OrderedRawChildren,
    raw_children: &mut OrderedRawChildren,
    boundary: &mut usize,
    namespaces: &NamespaceBindings,
) -> Result<()> {
    if is_a && matches!(name, b"masterClrMapping" | b"overrideClrMapping") {
        namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
        if kind.is_some() {
            return Err(duplicate("colour-map override choice"));
        }
        if name == b"overrideClrMapping" {
            let parsed = parse_color_map(&raw, namespaces)?;
            *attributes = parsed.raw_attributes;
            *mapping_children = parsed.raw_children;
            *kind = Some(ColorMapOverrideKind::Override(parsed.value));
        } else {
            let (attrs, children) = parse_raw_shell(&raw)?;
            *attributes = attrs;
            *mapping_children = children;
            *kind = Some(ColorMapOverrideKind::Master);
        }
        *boundary = 1;
    } else {
        raw_children.push(*boundary, raw);
    }
    Ok(())
}

impl CT_MasterTextStyles {
    fn from_fragment(xml: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        let start = loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) => break start.into_owned(),
                Event::Eof => return Err(OxmlError::MissingElement("p:txStyles".to_owned())),
                _ => {}
            }
            buffer.clear();
        };
        let namespaces = inherited.with_start(&start)?;
        namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
        let raw_attributes = root_attributes(&start, FIXED_MODEL_PREFIXES)?;
        let mut styles: [Option<CT_TextListStyle>; 3] = [None, None, None];
        let mut raw_children = OrderedRawChildren::default();
        let mut boundary = 0usize;
        loop {
            buffer.clear();
            match reader.read_event_into(&mut buffer)? {
                Event::Start(child) => {
                    let child_ns = namespaces.with_start(&child)?;
                    let name = local_name(child.name().as_ref()).to_vec();
                    let is_p = child_ns.element_uri(child.name().as_ref()) == Some(P_NS);
                    let raw = capture_element(&mut reader, &child)?;
                    capture_text_style(
                        &name,
                        is_p,
                        raw,
                        &mut styles,
                        &mut raw_children,
                        &mut boundary,
                        &child_ns,
                    )?;
                }
                Event::Empty(child) => {
                    let child_ns = namespaces.with_start(&child)?;
                    let name = local_name(child.name().as_ref()).to_vec();
                    let is_p = child_ns.element_uri(child.name().as_ref()) == Some(P_NS);
                    let raw = capture_empty_element(&child)?;
                    capture_text_style(
                        &name,
                        is_p,
                        raw,
                        &mut styles,
                        &mut raw_children,
                        &mut boundary,
                        &child_ns,
                    )?;
                }
                Event::End(end) if local_name(end.name().as_ref()) == b"txStyles" => break,
                Event::Eof => {
                    return Err(OxmlError::MissingElement("closing p:txStyles".to_owned()));
                }
                _ => {}
            }
        }
        let [title, body, other] = styles;
        Ok(Self {
            title_style: required(title, "p:titleStyle")?,
            body_style: required(body, "p:bodyStyle")?,
            other_style: required(other, "p:otherStyle")?,
            raw_attributes,
            raw_children,
        })
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut start = BytesStart::new("p:txStyles");
        push_attributes(&mut start, &self.raw_attributes);
        writer.write_event(Event::Start(start))?;
        emit_raw(writer, self.raw_children.at(0))?;
        self.title_style
            .write_xml_as(writer, "p:titleStyle")
            .map_err(map_drawing_error)?;
        emit_raw(writer, self.raw_children.at(1))?;
        self.body_style
            .write_xml_as(writer, "p:bodyStyle")
            .map_err(map_drawing_error)?;
        emit_raw(writer, self.raw_children.at(2))?;
        self.other_style
            .write_xml_as(writer, "p:otherStyle")
            .map_err(map_drawing_error)?;
        emit_raw(writer, self.raw_children.at(3))?;
        writer.write_event(Event::End(BytesEnd::new("p:txStyles")))?;
        Ok(())
    }
}

fn capture_text_style(
    name: &[u8],
    is_p: bool,
    raw: Vec<u8>,
    styles: &mut [Option<CT_TextListStyle>; 3],
    children: &mut OrderedRawChildren,
    boundary: &mut usize,
    namespaces: &NamespaceBindings,
) -> Result<()> {
    let index = if is_p {
        match name {
            b"titleStyle" => Some(0),
            b"bodyStyle" => Some(1),
            b"otherStyle" => Some(2),
            _ => None,
        }
    } else {
        None
    };
    if let Some(index) = index {
        namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
        if styles[index].is_some() {
            return Err(duplicate(std::str::from_utf8(name).unwrap_or("text style")));
        }
        styles[index] = Some(CT_TextListStyle::from_xml(&raw).map_err(map_drawing_error)?);
        *boundary = (*boundary).max(index + 1);
    } else {
        children.push(*boundary, raw);
    }
    Ok(())
}

pub(crate) fn parse_color_map(
    xml: &[u8],
    namespaces: &NamespaceBindings,
) -> Result<ParsedColorMap> {
    namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) => {
                let value = color_map_from_start(&start)?;
                let raw_attributes = color_map_raw_attributes(&start)?;
                let raw_children =
                    capture_shell_children(&mut reader, local_name(start.name().as_ref()))?;
                return Ok(ParsedColorMap {
                    value,
                    raw_attributes,
                    raw_children,
                });
            }
            Event::Empty(start) => {
                return Ok(ParsedColorMap {
                    value: color_map_from_start(&start)?,
                    raw_attributes: color_map_raw_attributes(&start)?,
                    raw_children: OrderedRawChildren::default(),
                });
            }
            Event::Eof => return Err(OxmlError::MissingElement("colour map".to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

fn color_map_from_start(start: &BytesStart<'_>) -> Result<ColorMap> {
    let attrs = all_attributes(start)?;
    let value = |name: &str| -> Result<ThemeColorSlot> {
        let raw = attrs
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
            .ok_or_else(|| OxmlError::MissingElement(format!("colour map requires @{name}")))?;
        parse_theme_slot(raw)
            .ok_or_else(|| OxmlError::InvalidValue(format!("invalid colour-map slot {raw}")))
    };
    Ok(ColorMap::new(
        value("bg1")?,
        value("tx1")?,
        value("bg2")?,
        value("tx2")?,
        value("accent1")?,
        value("accent2")?,
        value("accent3")?,
        value("accent4")?,
        value("accent5")?,
        value("accent6")?,
        value("hlink")?,
        value("folHlink")?,
    ))
}

fn parse_theme_slot(value: &str) -> Option<ThemeColorSlot> {
    Some(match value {
        "dk1" => ThemeColorSlot::Dark1,
        "lt1" => ThemeColorSlot::Light1,
        "dk2" => ThemeColorSlot::Dark2,
        "lt2" => ThemeColorSlot::Light2,
        "accent1" => ThemeColorSlot::Accent1,
        "accent2" => ThemeColorSlot::Accent2,
        "accent3" => ThemeColorSlot::Accent3,
        "accent4" => ThemeColorSlot::Accent4,
        "accent5" => ThemeColorSlot::Accent5,
        "accent6" => ThemeColorSlot::Accent6,
        "hlink" => ThemeColorSlot::Hyperlink,
        "folHlink" => ThemeColorSlot::FollowedHyperlink,
        _ => return None,
    })
}

fn color_map_raw_attributes(start: &BytesStart<'_>) -> Result<RawAttributes> {
    const KNOWN: [&str; 12] = [
        "bg1", "tx1", "bg2", "tx2", "accent1", "accent2", "accent3", "accent4", "accent5",
        "accent6", "hlink", "folHlink",
    ];
    Ok(all_attributes(start)?
        .into_iter()
        .filter(|(name, _)| {
            !KNOWN.contains(&name.as_str()) && !is_fixed_xmlns(name, FIXED_MODEL_PREFIXES)
        })
        .collect())
}

pub(crate) fn write_color_map<W: Write>(
    writer: &mut Writer<W>,
    tag: &str,
    map: &ColorMap,
    attributes: &RawAttributes,
    children: &OrderedRawChildren,
) -> Result<()> {
    let mut start = BytesStart::new(tag);
    for (name, slot) in color_map_slots() {
        start.push_attribute((name, map.theme_slot(slot).as_str()));
    }
    push_attributes(&mut start, attributes);
    if children.is_empty() {
        writer.write_event(Event::Empty(start))?;
    } else {
        writer.write_event(Event::Start(start))?;
        emit_raw(writer, children.at(0))?;
        writer.write_event(Event::End(BytesEnd::new(tag)))?;
    }
    Ok(())
}

fn color_map_slots() -> [(&'static str, ColorMapSlot); 12] {
    [
        ("bg1", ColorMapSlot::Background1),
        ("tx1", ColorMapSlot::Text1),
        ("bg2", ColorMapSlot::Background2),
        ("tx2", ColorMapSlot::Text2),
        ("accent1", ColorMapSlot::Accent1),
        ("accent2", ColorMapSlot::Accent2),
        ("accent3", ColorMapSlot::Accent3),
        ("accent4", ColorMapSlot::Accent4),
        ("accent5", ColorMapSlot::Accent5),
        ("accent6", ColorMapSlot::Accent6),
        ("hlink", ColorMapSlot::Hyperlink),
        ("folHlink", ColorMapSlot::FollowedHyperlink),
    ]
}

fn parse_raw_shell(xml: &[u8]) -> Result<(RawAttributes, OrderedRawChildren)> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) => {
                let attrs = root_attributes(&start, FIXED_MODEL_PREFIXES)?;
                let children =
                    capture_shell_children(&mut reader, local_name(start.name().as_ref()))?;
                return Ok((attrs, children));
            }
            Event::Empty(start) => {
                return Ok((
                    root_attributes(&start, FIXED_MODEL_PREFIXES)?,
                    OrderedRawChildren::default(),
                ));
            }
            Event::Eof => return Err(OxmlError::MissingElement("mapping element".to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

fn capture_shell_children(
    reader: &mut Reader<&[u8]>,
    root_name: &[u8],
) -> Result<OrderedRawChildren> {
    let mut children = OrderedRawChildren::default();
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(child) => children.push(0, capture_element(reader, &child)?),
            Event::Empty(child) => children.push(0, capture_empty_element(&child)?),
            Event::End(end) if local_name(end.name().as_ref()) == root_name => return Ok(children),
            Event::Eof => {
                return Err(OxmlError::MissingElement(
                    "closing mapping element".to_owned(),
                ));
            }
            _ => {}
        }
        buffer.clear();
    }
}

fn write_mapping_shell<W: Write>(
    writer: &mut Writer<W>,
    tag: &str,
    attributes: &RawAttributes,
    children: &OrderedRawChildren,
) -> Result<()> {
    let mut start = BytesStart::new(tag);
    push_attributes(&mut start, attributes);
    if children.is_empty() {
        writer.write_event(Event::Empty(start))?;
    } else {
        writer.write_event(Event::Start(start))?;
        emit_raw(writer, children.at(0))?;
        writer.write_event(Event::End(BytesEnd::new(tag)))?;
    }
    Ok(())
}

fn push_attributes(start: &mut BytesStart<'_>, attributes: &RawAttributes) {
    for (name, value) in attributes {
        start.push_attribute((name.as_str(), value.as_str()));
    }
}

fn emit_raw<'a, W: Write>(
    writer: &mut Writer<W>,
    children: impl Iterator<Item = &'a [u8]>,
) -> Result<()> {
    for child in children {
        writer.get_mut().write_all(child)?;
    }
    Ok(())
}

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

fn required<T>(value: Option<T>, name: &str) -> Result<T> {
    value.ok_or_else(|| OxmlError::MissingElement(name.to_owned()))
}

fn duplicate(name: &str) -> OxmlError {
    OxmlError::InvalidValue(format!("duplicate PresentationML {name}"))
}

fn unexpected(element: &BytesStart<'_>) -> OxmlError {
    OxmlError::UnexpectedElement(String::from_utf8_lossy(element.name().as_ref()).into_owned())
}

fn map_drawing_error(error: impl ToString) -> OxmlError {
    OxmlError::InvalidValue(error.to_string())
}
