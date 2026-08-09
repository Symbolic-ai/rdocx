use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_core::units::Emu;
use oxml_drawing::namespace::A_NS;
use oxml_drawing::order::OrderedRawChildren;
use oxml_drawing::text::CT_TextListStyle;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer, XmlVersion};

use crate::namespace::{
    FIXED_MODEL_PREFIXES, NamespaceBindings, P_NS, R_NS, all_attributes, root_attributes,
};

const MIN_SLIDE_ID: u32 = 256;
const MAX_SLIDE_ID: u32 = 2_147_483_647;

pub type Result<T> = std::result::Result<T, OxmlError>;
type RawAttributes = Vec<(String, String)>;
type ParsedSlideList = (Vec<CT_SlideId>, RawAttributes, OrderedRawChildren);
type ParsedMasterList = (Vec<CT_SlideMasterId>, RawAttributes, OrderedRawChildren);
type ParsedIdentifier = (Option<u32>, String, RawAttributes, OrderedRawChildren);

/// The typed size of a presentation slide or notes page in EMUs.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_SlideSize {
    pub cx: Emu,
    pub cy: Emu,
    pub kind: Option<String>,
    raw_attributes: Vec<(String, String)>,
    raw_children: OrderedRawChildren,
}

impl CT_SlideSize {
    pub fn new(cx: Emu, cy: Emu) -> Result<Self> {
        validate_dimension("size", "cx", cx)?;
        validate_dimension("size", "cy", cy)?;
        Ok(Self {
            cx,
            cy,
            kind: None,
            raw_attributes: Vec::new(),
            raw_children: OrderedRawChildren::default(),
        })
    }
}

/// One ordered slide identifier and its unresolved OPC relationship id.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_SlideId {
    pub id: u32,
    pub relationship_id: String,
    raw_attributes: Vec<(String, String)>,
    raw_children: OrderedRawChildren,
}

impl CT_SlideId {
    pub fn new(id: u32, relationship_id: impl Into<String>) -> Result<Self> {
        validate_slide_id(id)?;
        Ok(Self {
            id,
            relationship_id: relationship_id.into(),
            raw_attributes: Vec::new(),
            raw_children: OrderedRawChildren::default(),
        })
    }
}

/// One slide-master identifier and its unresolved OPC relationship id.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_SlideMasterId {
    /// The producer-assigned master id, when present in the source part.
    pub id: Option<u32>,
    pub relationship_id: String,
    raw_attributes: Vec<(String, String)>,
    raw_children: OrderedRawChildren,
}

impl CT_SlideMasterId {
    pub fn new(id: u32, relationship_id: impl Into<String>) -> Self {
        Self {
            id: Some(id),
            relationship_id: relationship_id.into(),
            raw_attributes: Vec::new(),
            raw_children: OrderedRawChildren::default(),
        }
    }
}

/// The typed shell for `/ppt/presentation.xml`.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_Presentation {
    pub slide_master_ids: Vec<CT_SlideMasterId>,
    pub slide_ids: Vec<CT_SlideId>,
    pub slide_size: Option<CT_SlideSize>,
    pub notes_size: CT_SlideSize,
    pub default_text_style: Option<CT_TextListStyle>,
    raw_attributes: Vec<(String, String)>,
    slide_master_list_present: bool,
    slide_master_list_attributes: Vec<(String, String)>,
    slide_master_list_raw_children: OrderedRawChildren,
    slide_id_list_present: bool,
    slide_id_list_attributes: Vec<(String, String)>,
    slide_id_list_raw_children: OrderedRawChildren,
    original_slide_relationship_ids: Vec<String>,
    raw_children: OrderedRawChildren,
}

#[derive(Default)]
struct PresentationParseState {
    raw_attributes: RawAttributes,
    slide_master_ids: Option<Vec<CT_SlideMasterId>>,
    slide_master_list_attributes: RawAttributes,
    slide_master_list_raw_children: OrderedRawChildren,
    slide_ids: Option<Vec<CT_SlideId>>,
    slide_id_list_attributes: RawAttributes,
    slide_id_list_raw_children: OrderedRawChildren,
    slide_size: Option<CT_SlideSize>,
    notes_size: Option<CT_SlideSize>,
    default_text_style: Option<CT_TextListStyle>,
    raw_children: OrderedRawChildren,
    boundary: usize,
}

impl PresentationParseState {
    fn capture_child(
        &mut self,
        name: &[u8],
        is_presentationml: bool,
        namespaces: &NamespaceBindings,
        raw: Vec<u8>,
    ) -> Result<()> {
        if !is_presentationml {
            self.raw_children.push(self.boundary, raw);
            return Ok(());
        }

        match name {
            b"sldMasterIdLst" => {
                namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                if self.slide_master_ids.is_some() {
                    return Err(duplicate("sldMasterIdLst"));
                }
                let (items, attributes, children) = parse_master_list(&raw, namespaces)?;
                self.slide_master_ids = Some(items);
                self.slide_master_list_attributes = attributes;
                self.slide_master_list_raw_children = children;
                self.boundary = self.boundary.max(1);
            }
            b"sldIdLst" => {
                namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                if self.slide_ids.is_some() {
                    return Err(duplicate("sldIdLst"));
                }
                let (items, attributes, children) = parse_slide_list(&raw, namespaces)?;
                self.slide_ids = Some(items);
                self.slide_id_list_attributes = attributes;
                self.slide_id_list_raw_children = children;
                self.boundary = self.boundary.max(4);
            }
            b"sldSz" => {
                namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                if self.slide_size.is_some() {
                    return Err(duplicate("sldSz"));
                }
                self.slide_size = Some(parse_size(&raw, "sldSz")?);
                self.boundary = self.boundary.max(5);
            }
            b"notesSz" => {
                namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                if self.notes_size.is_some() {
                    return Err(duplicate("notesSz"));
                }
                self.notes_size = Some(parse_size(&raw, "notesSz")?);
                self.boundary = self.boundary.max(6);
            }
            b"defaultTextStyle" => {
                namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                if self.default_text_style.is_some() {
                    return Err(duplicate("defaultTextStyle"));
                }
                self.default_text_style = Some(
                    CT_TextListStyle::from_xml(&raw)
                        .map_err(|error| invalid_value(error.to_string()))?,
                );
                self.boundary = self.boundary.max(13);
            }
            _ => {
                let slot = root_schema_boundary(name).unwrap_or(self.boundary);
                self.raw_children.push(slot, raw);
                self.boundary = self.boundary.max(slot);
            }
        }
        Ok(())
    }

    fn finish(self) -> Result<CT_Presentation> {
        let notes_size = self
            .notes_size
            .ok_or_else(|| OxmlError::MissingElement("p:notesSz".to_owned()))?;
        let original_slide_relationship_ids = self
            .slide_ids
            .as_ref()
            .map(|slides| {
                slides
                    .iter()
                    .map(|slide| slide.relationship_id.clone())
                    .collect()
            })
            .unwrap_or_default();
        let presentation = CT_Presentation {
            slide_master_list_present: self.slide_master_ids.is_some(),
            slide_master_ids: self.slide_master_ids.unwrap_or_default(),
            slide_id_list_present: self.slide_ids.is_some(),
            slide_ids: self.slide_ids.unwrap_or_default(),
            slide_size: self.slide_size,
            notes_size,
            default_text_style: self.default_text_style,
            raw_attributes: self.raw_attributes,
            slide_master_list_attributes: self.slide_master_list_attributes,
            slide_master_list_raw_children: self.slide_master_list_raw_children,
            slide_id_list_attributes: self.slide_id_list_attributes,
            slide_id_list_raw_children: self.slide_id_list_raw_children,
            original_slide_relationship_ids,
            raw_children: self.raw_children,
        };
        presentation.validate()?;
        Ok(presentation)
    }
}

impl CT_Presentation {
    pub fn new(notes_size: CT_SlideSize) -> Self {
        Self {
            slide_master_ids: Vec::new(),
            slide_ids: Vec::new(),
            slide_size: None,
            notes_size,
            default_text_style: None,
            raw_attributes: Vec::new(),
            slide_master_list_present: false,
            slide_master_list_attributes: Vec::new(),
            slide_master_list_raw_children: OrderedRawChildren::default(),
            slide_id_list_present: false,
            slide_id_list_attributes: Vec::new(),
            slide_id_list_raw_children: OrderedRawChildren::default(),
            original_slide_relationship_ids: Vec::new(),
            raw_children: OrderedRawChildren::default(),
        }
    }

    /// Parses a complete PresentationML presentation root with any prefix.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(element) => {
                    let namespaces = NamespaceBindings::default().with_start(&element)?;
                    if local_name(element.name().as_ref()) == b"presentation"
                        && namespaces.element_uri(element.name().as_ref()) == Some(P_NS)
                    {
                        namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                        return Self::from_element(&mut reader, &element, &namespaces);
                    }
                    return Err(OxmlError::UnexpectedElement(element_name(&element)));
                }
                Event::Empty(element) => {
                    let namespaces = NamespaceBindings::default().with_start(&element)?;
                    if local_name(element.name().as_ref()) == b"presentation"
                        && namespaces.element_uri(element.name().as_ref()) == Some(P_NS)
                    {
                        namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                        return Err(OxmlError::MissingElement("p:notesSz".to_owned()));
                    }
                    return Err(OxmlError::UnexpectedElement(element_name(&element)));
                }
                Event::Eof => {
                    return Err(OxmlError::MissingElement(
                        "PresentationML presentation".to_owned(),
                    ));
                }
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_element(
        reader: &mut Reader<&[u8]>,
        start: &BytesStart<'_>,
        root_namespaces: &NamespaceBindings,
    ) -> Result<Self> {
        let mut state = PresentationParseState {
            raw_attributes: root_attributes(start, FIXED_MODEL_PREFIXES)?,
            ..PresentationParseState::default()
        };
        let mut buffer = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(element) => {
                    let name = local_name(element.name().as_ref()).to_vec();
                    let namespaces = root_namespaces.with_start(&element)?;
                    let is_presentationml =
                        namespaces.element_uri(element.name().as_ref()) == Some(P_NS);
                    let raw = capture_element(reader, &element)?;
                    state.capture_child(&name, is_presentationml, &namespaces, raw)?;
                }
                Event::Empty(element) => {
                    let name = local_name(element.name().as_ref()).to_vec();
                    let namespaces = root_namespaces.with_start(&element)?;
                    let is_presentationml =
                        namespaces.element_uri(element.name().as_ref()) == Some(P_NS);
                    let raw = capture_empty_element(&element)?;
                    state.capture_child(&name, is_presentationml, &namespaces, raw)?;
                }
                Event::End(element) if local_name(element.name().as_ref()) == b"presentation" => {
                    return state.finish();
                }
                Event::Eof => {
                    return Err(OxmlError::MissingElement(
                        "closing p:presentation".to_owned(),
                    ));
                }
                _ => {}
            }
            buffer.clear();
        }
    }

    fn validate(&self) -> Result<()> {
        if let Some(size) = &self.slide_size {
            validate_size("sldSz", size)?;
        }
        validate_size("notesSz", &self.notes_size)
    }

    /// Writes fixed `p`, `a`, and `r` prefixes in presentation schema order.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut writer = Writer::new(Vec::new());
        let mut root = BytesStart::new("p:presentation");
        root.push_attribute(("xmlns:p", P_NS));
        root.push_attribute(("xmlns:a", A_NS));
        root.push_attribute(("xmlns:r", R_NS));
        push_attributes(&mut root, &self.raw_attributes);
        writer.write_event(Event::Start(root))?;

        emit_raw(&mut writer, self.raw_children.at(0))?;
        if self.slide_master_list_present || !self.slide_master_ids.is_empty() {
            write_master_list(self, &mut writer)?;
        }
        emit_raw(&mut writer, self.raw_children.at(1))?;
        emit_raw(&mut writer, self.raw_children.at(2))?;
        emit_raw(&mut writer, self.raw_children.at(3))?;
        if self.slide_id_list_present || !self.slide_ids.is_empty() {
            write_slide_list(self, &mut writer)?;
        }
        emit_raw(&mut writer, self.raw_children.at(4))?;
        if let Some(size) = &self.slide_size {
            write_size(size, "p:sldSz", &mut writer)?;
        }
        emit_raw(&mut writer, self.raw_children.at(5))?;
        write_size(&self.notes_size, "p:notesSz", &mut writer)?;
        emit_raw(&mut writer, self.raw_children.at(6))?;
        for boundary in 7..=12 {
            emit_raw(&mut writer, self.raw_children.at(boundary))?;
        }
        if let Some(style) = &self.default_text_style {
            style
                .write_xml_as(&mut writer, "p:defaultTextStyle")
                .map_err(|error| invalid_value(error.to_string()))?;
        }
        emit_raw(&mut writer, self.raw_children.at(13))?;
        emit_raw(&mut writer, self.raw_children.at(14))?;
        emit_raw(&mut writer, self.raw_children.at(15))?;
        writer.write_event(Event::End(BytesEnd::new("p:presentation")))?;
        Ok(writer.into_inner())
    }

    /// Removes references to one presentation relationship from custom shows.
    pub fn remove_custom_show_slide(&mut self, relationship_id: &str) -> Result<()> {
        let xml = self.to_xml()?;
        let rewritten = remove_custom_show_slide_references(&xml, relationship_id)?;
        *self = Self::from_xml(&rewritten)?;
        Ok(())
    }
}

/// Returns relationship ids referenced by slides inside preserved custom shows.
pub fn custom_show_relationship_ids(xml: &[u8]) -> Result<Vec<String>> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut scopes = vec![NamespaceBindings::default()];
    let mut depth = 0usize;
    let mut custom_show_depth = None;
    let mut ids = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) => {
                let parent = scopes.last().ok_or_else(|| {
                    invalid_value("missing custom-show namespace scope".to_owned())
                })?;
                let scope = parent.with_start(&element)?;
                let is_custom_show = scope.element_uri(element.name().as_ref()) == Some(P_NS)
                    && local_name(element.name().as_ref()) == b"custShowLst";
                if custom_show_depth.is_some() || is_custom_show {
                    collect_custom_show_slide_id(&element, &scope, &mut ids)?;
                }
                scopes.push(scope);
                depth += 1;
                if is_custom_show {
                    custom_show_depth = Some(depth);
                }
            }
            Event::Empty(element) => {
                let parent = scopes.last().ok_or_else(|| {
                    invalid_value("missing custom-show namespace scope".to_owned())
                })?;
                let scope = parent.with_start(&element)?;
                if custom_show_depth.is_some() {
                    collect_custom_show_slide_id(&element, &scope, &mut ids)?;
                }
            }
            Event::End(_) => {
                if custom_show_depth == Some(depth) {
                    custom_show_depth = None;
                }
                if depth == 0 || scopes.len() == 1 {
                    return Err(invalid_value(
                        "custom-show XML has an unmatched closing tag".to_owned(),
                    ));
                }
                depth -= 1;
                scopes.pop();
            }
            Event::Eof => {
                if depth != 0 || scopes.len() != 1 {
                    return Err(invalid_value(
                        "custom-show XML ended before its root closed".to_owned(),
                    ));
                }
                return Ok(ids);
            }
            _ => {}
        }
        buffer.clear();
    }
}

fn collect_custom_show_slide_id(
    element: &BytesStart<'_>,
    scope: &NamespaceBindings,
    ids: &mut Vec<String>,
) -> Result<()> {
    if scope.element_uri(element.name().as_ref()) != Some(P_NS)
        || local_name(element.name().as_ref()) != b"sld"
    {
        return Ok(());
    }
    for attribute in element.attributes() {
        let attribute = attribute?;
        if scope.attribute_uri(attribute.key.as_ref()) == Some(R_NS)
            && local_name(attribute.key.as_ref()) == b"id"
        {
            ids.push(
                attribute
                    .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())?
                    .into_owned(),
            );
        }
    }
    Ok(())
}

fn remove_custom_show_slide_references(xml: &[u8], relationship_id: &str) -> Result<Vec<u8>> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut scopes = vec![NamespaceBindings::default()];
    let mut depth = 0usize;
    let mut custom_show_depth = None;
    let mut removals = Vec::new();
    loop {
        let event_start = reader.buffer_position() as usize;
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) => {
                let parent = scopes.last().ok_or_else(|| {
                    invalid_value("missing custom-show namespace scope".to_owned())
                })?;
                let scope = parent.with_start(&element)?;
                let is_custom_show = scope.element_uri(element.name().as_ref()) == Some(P_NS)
                    && local_name(element.name().as_ref()) == b"custShowLst";
                if custom_show_depth.is_some()
                    && custom_show_slide_relationship_id(&element, &scope)?.as_deref()
                        == Some(relationship_id)
                {
                    reader.read_to_end(element.name())?;
                    removals.push(event_start..reader.buffer_position() as usize);
                } else {
                    scopes.push(scope);
                    depth += 1;
                    if is_custom_show {
                        custom_show_depth = Some(depth);
                    }
                }
            }
            Event::Empty(element) => {
                let parent = scopes.last().ok_or_else(|| {
                    invalid_value("missing custom-show namespace scope".to_owned())
                })?;
                let scope = parent.with_start(&element)?;
                if custom_show_depth.is_some()
                    && custom_show_slide_relationship_id(&element, &scope)?.as_deref()
                        == Some(relationship_id)
                {
                    removals.push(event_start..reader.buffer_position() as usize);
                }
            }
            Event::End(_) => {
                if custom_show_depth == Some(depth) {
                    custom_show_depth = None;
                }
                if depth == 0 || scopes.len() == 1 {
                    return Err(invalid_value(
                        "custom-show XML has an unmatched closing tag".to_owned(),
                    ));
                }
                depth -= 1;
                scopes.pop();
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if depth != 0 || scopes.len() != 1 {
        return Err(invalid_value(
            "custom-show XML ended before its root closed".to_owned(),
        ));
    }
    let mut rewritten = xml.to_vec();
    for range in removals.into_iter().rev() {
        rewritten.drain(range);
    }
    Ok(rewritten)
}

fn custom_show_slide_relationship_id(
    element: &BytesStart<'_>,
    scope: &NamespaceBindings,
) -> Result<Option<String>> {
    if scope.element_uri(element.name().as_ref()) != Some(P_NS)
        || local_name(element.name().as_ref()) != b"sld"
    {
        return Ok(None);
    }
    for attribute in element.attributes() {
        let attribute = attribute?;
        if scope.attribute_uri(attribute.key.as_ref()) == Some(R_NS)
            && local_name(attribute.key.as_ref()) == b"id"
        {
            return Ok(Some(
                attribute
                    .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())?
                    .into_owned(),
            ));
        }
    }
    Ok(None)
}

fn parse_slide_list(xml: &[u8], inherited: &NamespaceBindings) -> Result<ParsedSlideList> {
    let mut reader = Reader::from_reader(xml);
    let mut list_namespaces = inherited.clone();
    let mut items = Vec::new();
    let mut attributes = Vec::new();
    let mut raw_children = OrderedRawChildren::default();
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if local_name(element.name().as_ref()) == b"sldIdLst" => {
                attributes = all_attributes(&element)?;
                list_namespaces = inherited.with_start(&element)?;
            }
            Event::Empty(element) if local_name(element.name().as_ref()) == b"sldIdLst" => {
                attributes = all_attributes(&element)?;
                return Ok((items, attributes, raw_children));
            }
            Event::Start(element) => {
                let namespaces = list_namespaces.with_start(&element)?;
                let raw = capture_element(&mut reader, &element)?;
                if local_name(element.name().as_ref()) == b"sldId"
                    && namespaces.element_uri(element.name().as_ref()) == Some(P_NS)
                {
                    namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                    items.push(parse_slide_id(&raw, &namespaces)?);
                } else {
                    raw_children.push(items.len(), raw);
                }
            }
            Event::Empty(element) => {
                let namespaces = list_namespaces.with_start(&element)?;
                let raw = capture_empty_element(&element)?;
                if local_name(element.name().as_ref()) == b"sldId"
                    && namespaces.element_uri(element.name().as_ref()) == Some(P_NS)
                {
                    namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                    items.push(parse_slide_id(&raw, &namespaces)?);
                } else {
                    raw_children.push(items.len(), raw);
                }
            }
            Event::End(element) if local_name(element.name().as_ref()) == b"sldIdLst" => {
                return Ok((items, attributes, raw_children));
            }
            Event::Eof => return Err(OxmlError::MissingElement("closing p:sldIdLst".to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

fn parse_master_list(xml: &[u8], inherited: &NamespaceBindings) -> Result<ParsedMasterList> {
    let mut reader = Reader::from_reader(xml);
    let mut list_namespaces = inherited.clone();
    let mut items = Vec::new();
    let mut attributes = Vec::new();
    let mut raw_children = OrderedRawChildren::default();
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if local_name(element.name().as_ref()) == b"sldMasterIdLst" => {
                attributes = all_attributes(&element)?;
                list_namespaces = inherited.with_start(&element)?;
            }
            Event::Empty(element) if local_name(element.name().as_ref()) == b"sldMasterIdLst" => {
                attributes = all_attributes(&element)?;
                return Ok((items, attributes, raw_children));
            }
            Event::Start(element) => {
                let namespaces = list_namespaces.with_start(&element)?;
                let raw = capture_element(&mut reader, &element)?;
                if local_name(element.name().as_ref()) == b"sldMasterId"
                    && namespaces.element_uri(element.name().as_ref()) == Some(P_NS)
                {
                    namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                    items.push(parse_master_id(&raw, &namespaces)?);
                } else {
                    raw_children.push(items.len(), raw);
                }
            }
            Event::Empty(element) => {
                let namespaces = list_namespaces.with_start(&element)?;
                let raw = capture_empty_element(&element)?;
                if local_name(element.name().as_ref()) == b"sldMasterId"
                    && namespaces.element_uri(element.name().as_ref()) == Some(P_NS)
                {
                    namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                    items.push(parse_master_id(&raw, &namespaces)?);
                } else {
                    raw_children.push(items.len(), raw);
                }
            }
            Event::End(element) if local_name(element.name().as_ref()) == b"sldMasterIdLst" => {
                return Ok((items, attributes, raw_children));
            }
            Event::Eof => {
                return Err(OxmlError::MissingElement(
                    "closing p:sldMasterIdLst".to_owned(),
                ));
            }
            _ => {}
        }
        buffer.clear();
    }
}

fn parse_slide_id(xml: &[u8], namespaces: &NamespaceBindings) -> Result<CT_SlideId> {
    let (id, relationship_id, raw_attributes, raw_children) =
        parse_identifier(xml, "sldId", namespaces)?;
    let id = id.ok_or_else(|| missing_attribute("sldId", "id"))?;
    Ok(CT_SlideId {
        id,
        relationship_id,
        raw_attributes,
        raw_children,
    })
}

fn parse_master_id(xml: &[u8], namespaces: &NamespaceBindings) -> Result<CT_SlideMasterId> {
    let (id, relationship_id, raw_attributes, raw_children) =
        parse_identifier(xml, "sldMasterId", namespaces)?;
    Ok(CT_SlideMasterId {
        id,
        relationship_id,
        raw_attributes,
        raw_children,
    })
}

fn parse_identifier(
    xml: &[u8],
    expected: &str,
    inherited: &NamespaceBindings,
) -> Result<ParsedIdentifier> {
    let mut reader = Reader::from_reader(xml);
    let mut id = None;
    let mut relationship_id = None;
    let mut raw_attributes = Vec::new();
    let mut raw_children = OrderedRawChildren::default();
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if local_name(element.name().as_ref()) == expected.as_bytes() => {
                let namespaces = inherited.with_start(&element)?;
                parse_identifier_attributes(
                    &element,
                    expected,
                    &namespaces,
                    &mut id,
                    &mut relationship_id,
                    &mut raw_attributes,
                )?;
            }
            Event::Empty(element) if local_name(element.name().as_ref()) == expected.as_bytes() => {
                let namespaces = inherited.with_start(&element)?;
                parse_identifier_attributes(
                    &element,
                    expected,
                    &namespaces,
                    &mut id,
                    &mut relationship_id,
                    &mut raw_attributes,
                )?;
                break;
            }
            Event::Start(element) => raw_children.push(0, capture_element(&mut reader, &element)?),
            Event::Empty(element) => raw_children.push(0, capture_empty_element(&element)?),
            Event::End(element) if local_name(element.name().as_ref()) == expected.as_bytes() => {
                break;
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok((
        id,
        relationship_id.ok_or_else(|| missing_attribute(expected, "r:id"))?,
        raw_attributes,
        raw_children,
    ))
}

fn parse_identifier_attributes(
    element: &BytesStart<'_>,
    expected: &str,
    namespaces: &NamespaceBindings,
    id: &mut Option<u32>,
    relationship_id: &mut Option<String>,
    raw_attributes: &mut Vec<(String, String)>,
) -> Result<()> {
    for (name, value) in all_attributes(element)? {
        if name == "id" {
            if id.is_some() {
                return Err(duplicate("id attribute"));
            }
            *id = Some(parse_u32(expected, "id", &value)?);
        } else if local_name(name.as_bytes()) == b"id"
            && namespaces.attribute_uri(name.as_bytes()) == Some(R_NS)
        {
            if relationship_id.replace(value).is_some() {
                return Err(duplicate("relationship id attribute"));
            }
        } else {
            raw_attributes.push((name, value));
        }
    }
    Ok(())
}

fn parse_size(xml: &[u8], expected: &str) -> Result<CT_SlideSize> {
    let mut reader = Reader::from_reader(xml);
    let mut cx = None;
    let mut cy = None;
    let mut kind = None;
    let mut raw_attributes = Vec::new();
    let mut raw_children = OrderedRawChildren::default();
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if local_name(element.name().as_ref()) == expected.as_bytes() => {
                parse_size_attributes(
                    &element,
                    expected,
                    &mut cx,
                    &mut cy,
                    &mut kind,
                    &mut raw_attributes,
                )?;
            }
            Event::Empty(element) if local_name(element.name().as_ref()) == expected.as_bytes() => {
                parse_size_attributes(
                    &element,
                    expected,
                    &mut cx,
                    &mut cy,
                    &mut kind,
                    &mut raw_attributes,
                )?;
                break;
            }
            Event::Start(element) => raw_children.push(0, capture_element(&mut reader, &element)?),
            Event::Empty(element) => raw_children.push(0, capture_empty_element(&element)?),
            Event::End(element) if local_name(element.name().as_ref()) == expected.as_bytes() => {
                break;
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    let size = CT_SlideSize {
        cx: cx.ok_or_else(|| missing_attribute(expected, "cx"))?,
        cy: cy.ok_or_else(|| missing_attribute(expected, "cy"))?,
        kind,
        raw_attributes,
        raw_children,
    };
    validate_size(expected, &size)?;
    Ok(size)
}

fn parse_size_attributes(
    element: &BytesStart<'_>,
    expected: &str,
    cx: &mut Option<Emu>,
    cy: &mut Option<Emu>,
    kind: &mut Option<String>,
    raw_attributes: &mut Vec<(String, String)>,
) -> Result<()> {
    for (name, value) in all_attributes(element)? {
        match name.as_str() {
            "cx" if cx.is_none() => *cx = Some(parse_emu(expected, "cx", &value)?),
            "cy" if cy.is_none() => *cy = Some(parse_emu(expected, "cy", &value)?),
            "type" if kind.is_none() => *kind = Some(value),
            "cx" | "cy" | "type" => return Err(duplicate(&format!("{expected} @{name}"))),
            _ => raw_attributes.push((name, value)),
        }
    }
    Ok(())
}

fn write_master_list<W: Write>(
    presentation: &CT_Presentation,
    writer: &mut Writer<W>,
) -> Result<()> {
    let mut list = BytesStart::new("p:sldMasterIdLst");
    push_attributes(&mut list, &presentation.slide_master_list_attributes);
    if presentation.slide_master_ids.is_empty()
        && presentation.slide_master_list_raw_children.is_empty()
    {
        writer.write_event(Event::Empty(list))?;
        return Ok(());
    }
    writer.write_event(Event::Start(list))?;
    emit_raw(writer, presentation.slide_master_list_raw_children.at(0))?;
    for (index, item) in presentation.slide_master_ids.iter().enumerate() {
        write_identifier(
            "p:sldMasterId",
            item.id,
            &item.relationship_id,
            &item.raw_attributes,
            &item.raw_children,
            writer,
        )?;
        emit_raw(
            writer,
            presentation.slide_master_list_raw_children.at(index + 1),
        )?;
    }
    writer.write_event(Event::End(BytesEnd::new("p:sldMasterIdLst")))?;
    Ok(())
}

fn write_slide_list<W: Write>(
    presentation: &CT_Presentation,
    writer: &mut Writer<W>,
) -> Result<()> {
    let mut list = BytesStart::new("p:sldIdLst");
    push_attributes(&mut list, &presentation.slide_id_list_attributes);
    if presentation.slide_ids.is_empty() && presentation.slide_id_list_raw_children.is_empty() {
        writer.write_event(Event::Empty(list))?;
        return Ok(());
    }
    writer.write_event(Event::Start(list))?;
    let original_to_current = presentation
        .original_slide_relationship_ids
        .iter()
        .map(|relationship_id| {
            presentation
                .slide_ids
                .iter()
                .position(|slide| slide.relationship_id == *relationship_id)
        })
        .collect::<Vec<_>>();
    for (index, item) in presentation.slide_ids.iter().enumerate() {
        emit_raw(
            writer,
            presentation.slide_id_list_raw_children.at_reconciled(
                index,
                0,
                &original_to_current,
                presentation.slide_ids.len(),
            ),
        )?;
        write_identifier(
            "p:sldId",
            Some(item.id),
            &item.relationship_id,
            &item.raw_attributes,
            &item.raw_children,
            writer,
        )?;
    }
    emit_raw(
        writer,
        presentation.slide_id_list_raw_children.at_reconciled(
            presentation.slide_ids.len(),
            0,
            &original_to_current,
            presentation.slide_ids.len(),
        ),
    )?;
    writer.write_event(Event::End(BytesEnd::new("p:sldIdLst")))?;
    Ok(())
}

fn write_identifier<W: Write>(
    tag: &str,
    id: Option<u32>,
    relationship_id: &str,
    raw_attributes: &[(String, String)],
    raw_children: &OrderedRawChildren,
    writer: &mut Writer<W>,
) -> Result<()> {
    let mut start = BytesStart::new(tag);
    let id_value = id.map(|value| value.to_string());
    if let Some(id_value) = &id_value {
        start.push_attribute(("id", id_value.as_str()));
    }
    start.push_attribute(("r:id", relationship_id));
    push_attributes(&mut start, raw_attributes);
    if raw_children.is_empty() {
        writer.write_event(Event::Empty(start))?;
    } else {
        writer.write_event(Event::Start(start))?;
        emit_raw(writer, raw_children.at(0))?;
        writer.write_event(Event::End(BytesEnd::new(tag)))?;
    }
    Ok(())
}

fn write_size<W: Write>(size: &CT_SlideSize, tag: &str, writer: &mut Writer<W>) -> Result<()> {
    validate_size(tag, size)?;
    let cx = size.cx.0.to_string();
    let cy = size.cy.0.to_string();
    let mut start = BytesStart::new(tag);
    start.push_attribute(("cx", cx.as_str()));
    start.push_attribute(("cy", cy.as_str()));
    if let Some(kind) = &size.kind {
        start.push_attribute(("type", kind.as_str()));
    }
    push_attributes(&mut start, &size.raw_attributes);
    if size.raw_children.is_empty() {
        writer.write_event(Event::Empty(start))?;
    } else {
        writer.write_event(Event::Start(start))?;
        emit_raw(writer, size.raw_children.at(0))?;
        writer.write_event(Event::End(BytesEnd::new(tag)))?;
    }
    Ok(())
}

fn validate_slide_id(id: u32) -> Result<()> {
    if (MIN_SLIDE_ID..=MAX_SLIDE_ID).contains(&id) {
        Ok(())
    } else {
        Err(invalid_value(format!(
            "PowerPoint slide id {id} is outside {MIN_SLIDE_ID}..={MAX_SLIDE_ID}"
        )))
    }
}

fn validate_size(element: &str, size: &CT_SlideSize) -> Result<()> {
    validate_dimension(element, "cx", size.cx)?;
    validate_dimension(element, "cy", size.cy)
}

fn validate_dimension(element: &str, attribute: &str, value: Emu) -> Result<()> {
    if value.0 > 0 {
        Ok(())
    } else {
        Err(invalid_value(format!(
            "{element} @{attribute} must be a positive EMU value"
        )))
    }
}

fn parse_emu(element: &str, attribute: &str, value: &str) -> Result<Emu> {
    let parsed = value
        .parse::<i64>()
        .map_err(|_| invalid_value(format!("{element} has malformed @{attribute}: {value}")))?;
    let emu = Emu(parsed);
    validate_dimension(element, attribute, emu)?;
    Ok(emu)
}

fn parse_u32(element: &str, attribute: &str, value: &str) -> Result<u32> {
    value
        .parse::<u32>()
        .map_err(|_| invalid_value(format!("{element} has malformed @{attribute}: {value}")))
}

fn push_attributes(start: &mut BytesStart<'_>, attributes: &[(String, String)]) {
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

fn root_schema_boundary(name: &[u8]) -> Option<usize> {
    match name {
        b"notesMasterIdLst" => Some(2),
        b"handoutMasterIdLst" => Some(3),
        b"smartTags" => Some(7),
        b"embeddedFontLst" => Some(8),
        b"custShowLst" => Some(9),
        b"photoAlbum" => Some(10),
        b"custDataLst" => Some(11),
        b"kiosk" => Some(12),
        b"modifyVerifier" => Some(14),
        b"extLst" => Some(15),
        _ => None,
    }
}

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

fn element_name(element: &BytesStart<'_>) -> String {
    String::from_utf8_lossy(element.name().as_ref()).into_owned()
}

fn missing_attribute(element: &str, attribute: &str) -> OxmlError {
    OxmlError::MissingElement(format!("{element} requires @{attribute}"))
}

fn duplicate(element: &str) -> OxmlError {
    invalid_value(format!("duplicate PresentationML {element}"))
}

fn invalid_value(message: String) -> OxmlError {
    OxmlError::InvalidValue(message)
}
