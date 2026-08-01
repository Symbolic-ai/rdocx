use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_core::units::Emu;
use oxml_core::xml::{local_name, matches_local_name};
use oxml_core::xml_text::read_element_text;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer, XmlVersion};

use crate::namespace::{A_NS, reject_conflicting_a_prefix};
use crate::order::OrderedRawChildren;
use crate::text::CT_TextBody;

pub type Result<T> = std::result::Result<T, OxmlError>;
type RawAttributes = Vec<(String, String)>;

/// One DrawingML `a:tbl` with typed grid, rows, cells, and banding properties.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub struct CT_Table {
    pub properties: Option<CT_TableProperties>,
    pub grid: CT_TableGrid,
    pub rows: Vec<CT_TableRow>,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
    original_rows: Vec<CT_TableRow>,
}

/// The ordered column widths in one `a:tblGrid`.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub struct CT_TableGrid {
    pub columns: Vec<Emu>,
    column_raw: Vec<GridColumnRaw>,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

/// Direction, edge emphasis, banding, and style identity from `a:tblPr`.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Default)]
pub struct CT_TableProperties {
    pub right_to_left: bool,
    pub first_row: bool,
    pub first_column: bool,
    pub last_row: bool,
    pub last_column: bool,
    pub band_rows: bool,
    pub band_columns: bool,
    pub style_id: Option<String>,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

/// One table row and its required stored height.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub struct CT_TableRow {
    pub height: Emu,
    pub cells: Vec<CT_TableCell>,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
    original_cells: Vec<CT_TableCell>,
    origin_index: usize,
}

/// One table cell with typed text and the OOXML merge contract retained.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub struct CT_TableCell {
    pub text_body: Option<CT_TextBody>,
    pub row_span: u32,
    pub grid_span: u32,
    pub horizontal_merge: bool,
    pub vertical_merge: bool,
    cell_properties: Option<CellPropertiesRaw>,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
    origin_index: usize,
}

#[derive(Clone, Debug, Default)]
struct GridColumnRaw {
    width: Emu,
    attributes: RawAttributes,
    children: Vec<Vec<u8>>,
}

impl PartialEq for GridColumnRaw {
    fn eq(&self, other: &Self) -> bool {
        self.attributes == other.attributes && self.children == other.children
    }
}

impl Eq for GridColumnRaw {}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct CellPropertiesRaw {
    attributes: RawAttributes,
    children: Vec<Vec<u8>>,
}

impl PartialEq for CT_Table {
    fn eq(&self, other: &Self) -> bool {
        match (self.to_xml(), other.to_xml()) {
            (Ok(left), Ok(right)) => left == right,
            _ => {
                self.properties == other.properties
                    && self.grid == other.grid
                    && self.rows == other.rows
                    && self.raw_attributes == other.raw_attributes
                    && self.raw_children == other.raw_children
            }
        }
    }
}

impl Eq for CT_Table {}

impl PartialEq for CT_TableGrid {
    fn eq(&self, other: &Self) -> bool {
        match (self.canonical_xml(), other.canonical_xml()) {
            (Ok(left), Ok(right)) => left == right,
            _ => {
                self.columns == other.columns
                    && self.column_raw == other.column_raw
                    && self.raw_attributes == other.raw_attributes
                    && self.raw_children == other.raw_children
            }
        }
    }
}

impl Eq for CT_TableGrid {}

impl PartialEq for CT_TableProperties {
    fn eq(&self, other: &Self) -> bool {
        match (self.canonical_xml(), other.canonical_xml()) {
            (Ok(left), Ok(right)) => left == right,
            _ => {
                self.right_to_left == other.right_to_left
                    && self.first_row == other.first_row
                    && self.first_column == other.first_column
                    && self.last_row == other.last_row
                    && self.last_column == other.last_column
                    && self.band_rows == other.band_rows
                    && self.band_columns == other.band_columns
                    && self.style_id == other.style_id
                    && self.raw_attributes == other.raw_attributes
                    && self.raw_children == other.raw_children
            }
        }
    }
}

impl Eq for CT_TableProperties {}

impl PartialEq for CT_TableRow {
    fn eq(&self, other: &Self) -> bool {
        match (self.canonical_xml(), other.canonical_xml()) {
            (Ok(left), Ok(right)) => left == right,
            _ => {
                self.height == other.height
                    && self.cells == other.cells
                    && self.raw_attributes == other.raw_attributes
                    && self.raw_children == other.raw_children
            }
        }
    }
}

impl Eq for CT_TableRow {}

impl PartialEq for CT_TableCell {
    fn eq(&self, other: &Self) -> bool {
        match (self.canonical_xml(), other.canonical_xml()) {
            (Ok(left), Ok(right)) => left == right,
            _ => {
                self.text_body == other.text_body
                    && self.row_span == other.row_span
                    && self.grid_span == other.grid_span
                    && self.horizontal_merge == other.horizontal_merge
                    && self.vertical_merge == other.vertical_merge
                    && self.cell_properties == other.cell_properties
                    && self.raw_attributes == other.raw_attributes
                    && self.raw_children == other.raw_children
            }
        }
    }
}

impl Eq for CT_TableCell {}

impl CT_Table {
    /// Parses a complete `a:tbl` with any namespace prefix.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) if matches_local_name(start.name().as_ref(), b"tbl") => {
                    reject_conflicting_a_prefix(&start)?;
                    return Self::from_element(&mut reader, &start);
                }
                Event::Empty(start) if matches_local_name(start.name().as_ref(), b"tbl") => {
                    reject_conflicting_a_prefix(&start)?;
                    return Err(missing("a:tblGrid and a:tr"));
                }
                Event::Start(start) | Event::Empty(start) => return Err(unexpected(&start)),
                Event::Eof => return Err(missing("a:tbl")),
                _ => {}
            }
            buffer.clear();
        }
    }

    /// Parses an extracted table and retains namespace bindings inherited from
    /// its original ancestors so the standalone writer remains namespace
    /// complete.
    pub fn from_xml_with_inherited_namespaces(
        xml: &[u8],
        inherited_namespaces: &[(String, String)],
    ) -> Result<Self> {
        let local_a_namespace = root_a_namespace_declaration(xml)?;
        let mut table = Self::from_xml(xml)?;
        for (prefix, uri) in inherited_namespaces {
            if prefix == "a" {
                if uri != A_NS && local_a_namespace.as_deref() != Some(A_NS) {
                    return Err(OxmlError::InvalidValue(
                        "inherited xmlns:a conflicts with the fixed DrawingML writer namespace"
                            .to_owned(),
                    ));
                }
                continue;
            }
            if prefix == "xml" {
                if uri != "http://www.w3.org/XML/1998/namespace" {
                    return Err(OxmlError::InvalidValue(
                        "inherited xmlns:xml has the wrong namespace URI".to_owned(),
                    ));
                }
                continue;
            }
            let attribute_name = namespace_attribute_name(prefix)?;
            if table
                .raw_attributes
                .iter()
                .any(|(name, _)| name == &attribute_name)
            {
                continue;
            }
            table.raw_attributes.push((attribute_name, uri.to_owned()));
        }
        Ok(table)
    }

    fn from_element(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut properties = None;
        let mut grid = None;
        let mut rows = Vec::new();
        let mut raw_children = OrderedRawChildren::default();
        let mut boundary = 0usize;
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(child) => {
                    let name = local_name(child.name().as_ref()).to_vec();
                    if matches!(name.as_slice(), b"tblPr" | b"tblGrid" | b"tr") {
                        reject_conflicting_a_prefix(&child)?;
                    }
                    let raw = capture_element(reader, &child)?;
                    capture_table_child(
                        &name,
                        raw,
                        &mut properties,
                        &mut grid,
                        &mut rows,
                        &mut raw_children,
                        &mut boundary,
                    )?;
                }
                Event::Empty(child) => {
                    let name = local_name(child.name().as_ref()).to_vec();
                    if matches!(name.as_slice(), b"tblPr" | b"tblGrid" | b"tr") {
                        reject_conflicting_a_prefix(&child)?;
                    }
                    let raw = capture_empty_element(&child)?;
                    capture_table_child(
                        &name,
                        raw,
                        &mut properties,
                        &mut grid,
                        &mut rows,
                        &mut raw_children,
                        &mut boundary,
                    )?;
                }
                Event::End(end) if matches_local_name(end.name().as_ref(), b"tbl") => break,
                Event::Eof => return Err(missing("closing a:tbl")),
                _ => {}
            }
            buffer.clear();
        }
        let grid = grid.ok_or_else(|| missing("a:tblGrid"))?;
        if rows.is_empty() {
            return Err(missing("at least one a:tr"));
        }
        Ok(Self {
            properties,
            grid,
            original_rows: rows.clone(),
            rows,
            raw_attributes: raw_attributes(start, &[], true)?,
            raw_children,
        })
    }

    /// Serialises a self-contained table with fixed `a:` prefixes.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        if self.rows.is_empty() {
            return Err(missing("at least one a:tr"));
        }
        let mut writer = Writer::new(Vec::new());
        let mut start = BytesStart::new("a:tbl");
        start.push_attribute(("xmlns:a", A_NS));
        push_attributes(&mut start, &self.raw_attributes);
        writer.write_event(Event::Start(start))?;
        emit_raw(&mut writer, self.raw_children.at(0))?;
        if let Some(properties) = &self.properties {
            properties.write_xml(&mut writer)?;
        }
        emit_raw(&mut writer, self.raw_children.at(1))?;
        self.grid.write_xml(&mut writer)?;
        let current_row_origins = self
            .rows
            .iter()
            .map(|row| row.origin_index)
            .collect::<Vec<_>>();
        let original_row_origins = self
            .original_rows
            .iter()
            .map(|row| row.origin_index)
            .collect::<Vec<_>>();
        let row_matches = matched_original_indices(&current_row_origins, &original_row_origins);
        let original_to_current = invert_matches(&row_matches, self.original_rows.len());
        for boundary in 0..=self.rows.len() {
            emit_raw(
                &mut writer,
                self.raw_children
                    .at_reconciled(boundary, 2, &original_to_current, self.rows.len()),
            )?;
            if let Some(row) = self.rows.get(boundary) {
                row.write_xml(&mut writer)?;
            }
        }
        writer.write_event(Event::End(BytesEnd::new("a:tbl")))?;
        Ok(writer.into_inner())
    }
}

fn root_a_namespace_declaration(xml: &[u8]) -> Result<Option<String>> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) | Event::Empty(start) => {
                return decoded_attr(&start, b"xmlns:a");
            }
            Event::Eof => return Ok(None),
            _ => buffer.clear(),
        }
    }
}

fn namespace_attribute_name(prefix: &str) -> Result<String> {
    if prefix.is_empty() {
        return Ok("xmlns".to_owned());
    }
    if prefix == "xmlns" {
        return Err(OxmlError::InvalidValue(
            "xmlns cannot be used as an inherited namespace prefix".to_owned(),
        ));
    }
    let mut chars = prefix.chars();
    let Some(first) = chars.next() else {
        return Ok("xmlns".to_owned());
    };
    if !is_ncname_start(first) || !chars.all(is_ncname_continue) {
        return Err(OxmlError::InvalidValue(format!(
            "invalid inherited namespace prefix {prefix}"
        )));
    }
    Ok(format!("xmlns:{prefix}"))
}

fn is_ncname_start(value: char) -> bool {
    matches!(
        value,
        'A'..='Z'
            | '_'
            | 'a'..='z'
            | '\u{C0}'..='\u{D6}'
            | '\u{D8}'..='\u{F6}'
            | '\u{F8}'..='\u{2FF}'
            | '\u{370}'..='\u{37D}'
            | '\u{37F}'..='\u{1FFF}'
            | '\u{200C}'..='\u{200D}'
            | '\u{2070}'..='\u{218F}'
            | '\u{2C00}'..='\u{2FEF}'
            | '\u{3001}'..='\u{D7FF}'
            | '\u{F900}'..='\u{FDCF}'
            | '\u{FDF0}'..='\u{FFFD}'
            | '\u{10000}'..='\u{EFFFF}'
    )
}

fn is_ncname_continue(value: char) -> bool {
    is_ncname_start(value)
        || matches!(
            value,
            '-' | '.' | '0'..='9' | '\u{B7}' | '\u{300}'..='\u{36F}' | '\u{203F}'..='\u{2040}'
        )
}

#[allow(clippy::too_many_arguments)]
fn capture_table_child(
    name: &[u8],
    raw: Vec<u8>,
    properties: &mut Option<CT_TableProperties>,
    grid: &mut Option<CT_TableGrid>,
    rows: &mut Vec<CT_TableRow>,
    raw_children: &mut OrderedRawChildren,
    boundary: &mut usize,
) -> Result<()> {
    match name {
        b"tblPr" if *boundary == 0 && properties.is_none() => {
            *properties = Some(CT_TableProperties::from_xml(&raw)?);
            *boundary = 1;
        }
        b"tblGrid" if *boundary <= 1 && grid.is_none() => {
            *grid = Some(CT_TableGrid::from_xml(&raw)?);
            *boundary = 2;
        }
        b"tr" if grid.is_some() => {
            let mut row = CT_TableRow::from_xml(&raw)?;
            row.origin_index = rows.len();
            rows.push(row);
            *boundary = 2 + rows.len();
        }
        b"tblPr" | b"tblGrid" | b"tr" => {
            return Err(OxmlError::InvalidValue(
                "a:tbl children must be a:tblPr?, a:tblGrid, then a:tr+".to_owned(),
            ));
        }
        _ => raw_children.push(*boundary, raw),
    }
    Ok(())
}

fn matched_original_indices<T: PartialEq>(current: &[T], original: &[T]) -> Vec<Option<usize>> {
    let mut matches = vec![None; current.len()];
    let mut original_used = vec![false; original.len()];

    for index in 0..current.len().min(original.len()) {
        if current[index] == original[index] {
            matches[index] = Some(index);
            original_used[index] = true;
        }
    }
    for (current_index, value) in current.iter().enumerate() {
        if matches[current_index].is_some() {
            continue;
        }
        if let Some(original_index) = original
            .iter()
            .enumerate()
            .find(|(original_index, candidate)| {
                !original_used[*original_index] && *candidate == value
            })
            .map(|(original_index, _)| original_index)
        {
            matches[current_index] = Some(original_index);
            original_used[original_index] = true;
        }
    }
    for (current_index, matched) in matches.iter_mut().enumerate() {
        if matched.is_none() && current_index < original.len() && !original_used[current_index] {
            *matched = Some(current_index);
            original_used[current_index] = true;
        }
    }
    matches
}

fn invert_matches(
    current_to_original: &[Option<usize>],
    original_len: usize,
) -> Vec<Option<usize>> {
    let mut original_to_current = vec![None; original_len];
    for (current_index, original_index) in current_to_original.iter().enumerate() {
        if let Some(original_index) = original_index {
            original_to_current[*original_index] = Some(current_index);
        }
    }
    original_to_current
}

impl CT_TableProperties {
    fn canonical_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        self.write_xml(&mut writer)?;
        Ok(writer.into_inner())
    }

    fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) if matches_local_name(start.name().as_ref(), b"tblPr") => {
                    reject_conflicting_a_prefix(&start)?;
                    return Self::from_element(&mut reader, &start);
                }
                Event::Empty(start) if matches_local_name(start.name().as_ref(), b"tblPr") => {
                    reject_conflicting_a_prefix(&start)?;
                    return Self::from_start(&start, None, OrderedRawChildren::default());
                }
                Event::Start(start) | Event::Empty(start) => return Err(unexpected(&start)),
                Event::Eof => return Err(missing("a:tblPr")),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_element(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut style_id = None;
        let mut raw_children = OrderedRawChildren::default();
        let mut boundary = 0usize;
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(child)
                    if matches_local_name(child.name().as_ref(), b"tableStyleId") =>
                {
                    reject_conflicting_a_prefix(&child)?;
                    if style_id.is_some() {
                        return Err(OxmlError::InvalidValue(
                            "duplicate a:tableStyleId".to_owned(),
                        ));
                    }
                    style_id = Some(read_element_text(reader, child.name()));
                    boundary = 1;
                }
                Event::Empty(child)
                    if matches_local_name(child.name().as_ref(), b"tableStyleId") =>
                {
                    reject_conflicting_a_prefix(&child)?;
                    if style_id.replace(String::new()).is_some() {
                        return Err(OxmlError::InvalidValue(
                            "duplicate a:tableStyleId".to_owned(),
                        ));
                    }
                    boundary = 1;
                }
                Event::Start(child) => {
                    raw_children.push(boundary, capture_element(reader, &child)?)
                }
                Event::Empty(child) => raw_children.push(boundary, capture_empty_element(&child)?),
                Event::End(end) if matches_local_name(end.name().as_ref(), b"tblPr") => break,
                Event::Eof => return Err(missing("closing a:tblPr")),
                _ => {}
            }
            buffer.clear();
        }
        Self::from_start(start, style_id, raw_children)
    }

    fn from_start(
        start: &BytesStart<'_>,
        style_id: Option<String>,
        raw_children: OrderedRawChildren,
    ) -> Result<Self> {
        Ok(Self {
            right_to_left: bool_attr(start, b"rtl")?.unwrap_or(false),
            first_row: bool_attr(start, b"firstRow")?.unwrap_or(false),
            first_column: bool_attr(start, b"firstCol")?.unwrap_or(false),
            last_row: bool_attr(start, b"lastRow")?.unwrap_or(false),
            last_column: bool_attr(start, b"lastCol")?.unwrap_or(false),
            band_rows: bool_attr(start, b"bandRow")?.unwrap_or(false),
            band_columns: bool_attr(start, b"bandCol")?.unwrap_or(false),
            style_id,
            raw_attributes: raw_attributes(
                start,
                &[
                    b"rtl",
                    b"firstRow",
                    b"firstCol",
                    b"lastRow",
                    b"lastCol",
                    b"bandRow",
                    b"bandCol",
                ],
                false,
            )?,
            raw_children,
        })
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut start = BytesStart::new("a:tblPr");
        push_true(&mut start, "rtl", self.right_to_left);
        push_true(&mut start, "firstRow", self.first_row);
        push_true(&mut start, "firstCol", self.first_column);
        push_true(&mut start, "lastRow", self.last_row);
        push_true(&mut start, "lastCol", self.last_column);
        push_true(&mut start, "bandRow", self.band_rows);
        push_true(&mut start, "bandCol", self.band_columns);
        push_attributes(&mut start, &self.raw_attributes);
        if self.style_id.is_none() && self.raw_children.is_empty() {
            writer.write_event(Event::Empty(start))?;
            return Ok(());
        }
        writer.write_event(Event::Start(start))?;
        emit_raw(writer, self.raw_children.at(0))?;
        if let Some(style_id) = &self.style_id {
            writer.write_event(Event::Start(BytesStart::new("a:tableStyleId")))?;
            writer.write_event(Event::Text(BytesText::new(style_id)))?;
            writer.write_event(Event::End(BytesEnd::new("a:tableStyleId")))?;
        }
        emit_raw(writer, self.raw_children.at(1))?;
        writer.write_event(Event::End(BytesEnd::new("a:tblPr")))?;
        Ok(())
    }
}

impl CT_TableGrid {
    fn canonical_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        self.write_xml(&mut writer)?;
        Ok(writer.into_inner())
    }

    fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) if matches_local_name(start.name().as_ref(), b"tblGrid") => {
                    reject_conflicting_a_prefix(&start)?;
                    return Self::from_element(&mut reader, &start);
                }
                Event::Empty(start) if matches_local_name(start.name().as_ref(), b"tblGrid") => {
                    reject_conflicting_a_prefix(&start)?;
                    return Err(missing("at least one a:gridCol"));
                }
                Event::Start(start) | Event::Empty(start) => return Err(unexpected(&start)),
                Event::Eof => return Err(missing("a:tblGrid")),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_element(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut columns = Vec::new();
        let mut column_raw = Vec::new();
        let mut raw_children = OrderedRawChildren::default();
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(child) if matches_local_name(child.name().as_ref(), b"gridCol") => {
                    reject_conflicting_a_prefix(&child)?;
                    let raw = capture_element(reader, &child)?;
                    let (width, metadata) = parse_grid_column(&raw)?;
                    columns.push(width);
                    column_raw.push(metadata);
                }
                Event::Empty(child) if matches_local_name(child.name().as_ref(), b"gridCol") => {
                    reject_conflicting_a_prefix(&child)?;
                    let width = Emu(required_i64(&child, b"w")?);
                    columns.push(width);
                    column_raw.push(GridColumnRaw {
                        width,
                        attributes: raw_attributes(&child, &[b"w"], false)?,
                        children: Vec::new(),
                    });
                }
                Event::Start(child) => {
                    raw_children.push(columns.len(), capture_element(reader, &child)?)
                }
                Event::Empty(child) => {
                    raw_children.push(columns.len(), capture_empty_element(&child)?)
                }
                Event::End(end) if matches_local_name(end.name().as_ref(), b"tblGrid") => break,
                Event::Eof => return Err(missing("closing a:tblGrid")),
                _ => {}
            }
            buffer.clear();
        }
        if columns.is_empty() {
            return Err(missing("at least one a:gridCol"));
        }
        Ok(Self {
            columns,
            column_raw,
            raw_attributes: raw_attributes(start, &[], false)?,
            raw_children,
        })
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        if self.columns.is_empty() {
            return Err(missing("at least one a:gridCol"));
        }
        let column_matches = self.matched_column_indices()?;
        let original_to_current = invert_matches(&column_matches, self.column_raw.len());
        let mut start = BytesStart::new("a:tblGrid");
        push_attributes(&mut start, &self.raw_attributes);
        writer.write_event(Event::Start(start))?;
        for (boundary, (width, metadata_index)) in
            self.columns.iter().zip(&column_matches).enumerate()
        {
            emit_raw(
                writer,
                self.raw_children.at_reconciled(
                    boundary,
                    0,
                    &original_to_current,
                    self.columns.len(),
                ),
            )?;
            let metadata = metadata_index.map(|index| &self.column_raw[index]);
            let mut column = BytesStart::new("a:gridCol");
            let width = width.0.to_string();
            column.push_attribute(("w", width.as_str()));
            if let Some(metadata) = metadata {
                push_attributes(&mut column, &metadata.attributes);
            }
            match metadata {
                Some(metadata) if !metadata.children.is_empty() => {
                    writer.write_event(Event::Start(column))?;
                    for child in &metadata.children {
                        writer.get_mut().write_all(child)?;
                    }
                    writer.write_event(Event::End(BytesEnd::new("a:gridCol")))?;
                }
                _ => writer.write_event(Event::Empty(column))?,
            }
        }
        emit_raw(
            writer,
            self.raw_children.at_reconciled(
                self.columns.len(),
                0,
                &original_to_current,
                self.columns.len(),
            ),
        )?;
        writer.write_event(Event::End(BytesEnd::new("a:tblGrid")))?;
        Ok(())
    }

    fn matched_column_indices(&self) -> Result<Vec<Option<usize>>> {
        let original_widths = self
            .column_raw
            .iter()
            .map(|metadata| metadata.width)
            .collect::<Vec<_>>();
        if self.columns == original_widths {
            return Ok((0..self.columns.len()).map(Some).collect());
        }

        let identity_sensitive = self
            .column_raw
            .iter()
            .any(|metadata| !metadata.attributes.is_empty() || !metadata.children.is_empty())
            || !self.raw_children.is_empty();
        if !identity_sensitive {
            return Ok(matched_original_indices(&self.columns, &original_widths));
        }

        if has_duplicate_values(&self.columns) || has_duplicate_values(&original_widths) {
            return Err(ambiguous_grid_metadata());
        }

        let exact_matches = self
            .columns
            .iter()
            .map(|width| {
                original_widths
                    .iter()
                    .position(|original| original == width)
            })
            .collect::<Vec<_>>();
        let unmatched_current = exact_matches
            .iter()
            .filter(|matched| matched.is_none())
            .count();
        let unmatched_original = original_widths
            .iter()
            .enumerate()
            .filter(|(index, _)| !exact_matches.contains(&Some(*index)))
            .count();
        if unmatched_current > 0
            && unmatched_original > 0
            && (unmatched_current > 1 || unmatched_original > 1)
        {
            return Err(ambiguous_grid_metadata());
        }
        Ok(matched_original_indices(&self.columns, &original_widths))
    }
}

fn has_duplicate_values<T: PartialEq>(values: &[T]) -> bool {
    values
        .iter()
        .enumerate()
        .any(|(index, value)| values[..index].contains(value))
}

fn ambiguous_grid_metadata() -> OxmlError {
    OxmlError::InvalidValue(
        "edited table grid is ambiguous because preserved column metadata cannot be associated reliably"
            .to_owned(),
    )
}

fn parse_grid_column(xml: &[u8]) -> Result<(Emu, GridColumnRaw)> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) => {
                let width = Emu(required_i64(&start, b"w")?);
                let attributes = raw_attributes(&start, &[b"w"], false)?;
                let mut children = Vec::new();
                loop {
                    buffer.clear();
                    match reader.read_event_into(&mut buffer)? {
                        Event::Start(child) => children.push(capture_element(&mut reader, &child)?),
                        Event::Empty(child) => children.push(capture_empty_element(&child)?),
                        Event::End(end) if matches_local_name(end.name().as_ref(), b"gridCol") => {
                            break;
                        }
                        Event::Eof => return Err(missing("closing a:gridCol")),
                        _ => {}
                    }
                }
                return Ok((
                    width,
                    GridColumnRaw {
                        width,
                        attributes,
                        children,
                    },
                ));
            }
            Event::Eof => return Err(missing("a:gridCol")),
            _ => {}
        }
        buffer.clear();
    }
}

impl CT_TableRow {
    fn canonical_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        self.write_xml(&mut writer)?;
        Ok(writer.into_inner())
    }

    fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) if matches_local_name(start.name().as_ref(), b"tr") => {
                    reject_conflicting_a_prefix(&start)?;
                    return Self::from_element(&mut reader, &start);
                }
                Event::Empty(start) if matches_local_name(start.name().as_ref(), b"tr") => {
                    reject_conflicting_a_prefix(&start)?;
                    return Err(missing("at least one a:tc"));
                }
                Event::Start(start) | Event::Empty(start) => return Err(unexpected(&start)),
                Event::Eof => return Err(missing("a:tr")),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_element(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut cells = Vec::new();
        let mut raw_children = OrderedRawChildren::default();
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(child) if matches_local_name(child.name().as_ref(), b"tc") => {
                    reject_conflicting_a_prefix(&child)?;
                    let mut cell = CT_TableCell::from_xml(&capture_element(reader, &child)?)?;
                    cell.origin_index = cells.len();
                    cells.push(cell);
                }
                Event::Empty(child) if matches_local_name(child.name().as_ref(), b"tc") => {
                    reject_conflicting_a_prefix(&child)?;
                    let mut cell = CT_TableCell::from_xml(&capture_empty_element(&child)?)?;
                    cell.origin_index = cells.len();
                    cells.push(cell);
                }
                Event::Start(child) => {
                    raw_children.push(cells.len(), capture_element(reader, &child)?)
                }
                Event::Empty(child) => {
                    raw_children.push(cells.len(), capture_empty_element(&child)?)
                }
                Event::End(end) if matches_local_name(end.name().as_ref(), b"tr") => break,
                Event::Eof => return Err(missing("closing a:tr")),
                _ => {}
            }
            buffer.clear();
        }
        if cells.is_empty() {
            return Err(missing("at least one a:tc"));
        }
        Ok(Self {
            height: Emu(required_i64(start, b"h")?),
            original_cells: cells.clone(),
            cells,
            raw_attributes: raw_attributes(start, &[b"h"], false)?,
            raw_children,
            origin_index: 0,
        })
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        if self.cells.is_empty() {
            return Err(missing("at least one a:tc"));
        }
        let mut start = BytesStart::new("a:tr");
        let height = self.height.0.to_string();
        start.push_attribute(("h", height.as_str()));
        push_attributes(&mut start, &self.raw_attributes);
        writer.write_event(Event::Start(start))?;
        let current_cell_origins = self
            .cells
            .iter()
            .map(|cell| cell.origin_index)
            .collect::<Vec<_>>();
        let original_cell_origins = self
            .original_cells
            .iter()
            .map(|cell| cell.origin_index)
            .collect::<Vec<_>>();
        let cell_matches = matched_original_indices(&current_cell_origins, &original_cell_origins);
        let original_to_current = invert_matches(&cell_matches, self.original_cells.len());
        for boundary in 0..=self.cells.len() {
            emit_raw(
                writer,
                self.raw_children.at_reconciled(
                    boundary,
                    0,
                    &original_to_current,
                    self.cells.len(),
                ),
            )?;
            if let Some(cell) = self.cells.get(boundary) {
                cell.write_xml(writer)?;
            }
        }
        writer.write_event(Event::End(BytesEnd::new("a:tr")))?;
        Ok(())
    }
}

impl CT_TableCell {
    fn canonical_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        self.write_xml(&mut writer)?;
        Ok(writer.into_inner())
    }

    fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) if matches_local_name(start.name().as_ref(), b"tc") => {
                    reject_conflicting_a_prefix(&start)?;
                    return Self::from_element(&mut reader, &start);
                }
                Event::Empty(start) if matches_local_name(start.name().as_ref(), b"tc") => {
                    reject_conflicting_a_prefix(&start)?;
                    return Self::from_start(&start, None, None, OrderedRawChildren::default());
                }
                Event::Start(start) | Event::Empty(start) => return Err(unexpected(&start)),
                Event::Eof => return Err(missing("a:tc")),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_element(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut text_body = None;
        let mut cell_properties = None;
        let mut raw_children = OrderedRawChildren::default();
        let mut boundary = 0usize;
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(child) => {
                    let name = local_name(child.name().as_ref()).to_vec();
                    if matches!(name.as_slice(), b"txBody" | b"tcPr") {
                        reject_conflicting_a_prefix(&child)?;
                    }
                    let raw = capture_element(reader, &child)?;
                    capture_cell_child(
                        &name,
                        raw,
                        &mut text_body,
                        &mut cell_properties,
                        &mut raw_children,
                        &mut boundary,
                    )?;
                }
                Event::Empty(child) => {
                    let name = local_name(child.name().as_ref()).to_vec();
                    if matches!(name.as_slice(), b"txBody" | b"tcPr") {
                        reject_conflicting_a_prefix(&child)?;
                    }
                    let raw = capture_empty_element(&child)?;
                    capture_cell_child(
                        &name,
                        raw,
                        &mut text_body,
                        &mut cell_properties,
                        &mut raw_children,
                        &mut boundary,
                    )?;
                }
                Event::End(end) if matches_local_name(end.name().as_ref(), b"tc") => break,
                Event::Eof => return Err(missing("closing a:tc")),
                _ => {}
            }
            buffer.clear();
        }
        Self::from_start(start, text_body, cell_properties, raw_children)
    }

    fn from_start(
        start: &BytesStart<'_>,
        text_body: Option<CT_TextBody>,
        cell_properties: Option<CellPropertiesRaw>,
        raw_children: OrderedRawChildren,
    ) -> Result<Self> {
        Ok(Self {
            text_body,
            row_span: positive_u32_attr(start, b"rowSpan")?.unwrap_or(1),
            grid_span: positive_u32_attr(start, b"gridSpan")?.unwrap_or(1),
            horizontal_merge: bool_attr(start, b"hMerge")?.unwrap_or(false),
            vertical_merge: bool_attr(start, b"vMerge")?.unwrap_or(false),
            cell_properties,
            raw_attributes: raw_attributes(
                start,
                &[b"rowSpan", b"gridSpan", b"hMerge", b"vMerge"],
                false,
            )?,
            raw_children,
            origin_index: 0,
        })
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        if self.row_span == 0 || self.grid_span == 0 {
            return Err(OxmlError::InvalidValue(
                "a:tc spans must be positive".to_owned(),
            ));
        }
        let mut start = BytesStart::new("a:tc");
        let row_span = self.row_span.to_string();
        let grid_span = self.grid_span.to_string();
        if self.row_span != 1 {
            start.push_attribute(("rowSpan", row_span.as_str()));
        }
        if self.grid_span != 1 {
            start.push_attribute(("gridSpan", grid_span.as_str()));
        }
        push_true(&mut start, "hMerge", self.horizontal_merge);
        push_true(&mut start, "vMerge", self.vertical_merge);
        push_attributes(&mut start, &self.raw_attributes);
        if self.text_body.is_none()
            && self.cell_properties.is_none()
            && self.raw_children.is_empty()
        {
            writer.write_event(Event::Empty(start))?;
            return Ok(());
        }
        writer.write_event(Event::Start(start))?;
        emit_raw(writer, self.raw_children.at(0))?;
        if let Some(text_body) = &self.text_body {
            writer
                .get_mut()
                .write_all(&text_body.to_xml().map_err(|error| {
                    OxmlError::InvalidValue(format!("invalid table-cell text body: {error}"))
                })?)?;
        }
        emit_raw(writer, self.raw_children.at(1))?;
        if let Some(properties) = &self.cell_properties {
            let mut start = BytesStart::new("a:tcPr");
            push_attributes(&mut start, &properties.attributes);
            if properties.children.is_empty() {
                writer.write_event(Event::Empty(start))?;
            } else {
                writer.write_event(Event::Start(start))?;
                for child in &properties.children {
                    writer.get_mut().write_all(child)?;
                }
                writer.write_event(Event::End(BytesEnd::new("a:tcPr")))?;
            }
        }
        emit_raw(writer, self.raw_children.at(2))?;
        writer.write_event(Event::End(BytesEnd::new("a:tc")))?;
        Ok(())
    }
}

fn capture_cell_child(
    name: &[u8],
    raw: Vec<u8>,
    text_body: &mut Option<CT_TextBody>,
    cell_properties: &mut Option<CellPropertiesRaw>,
    raw_children: &mut OrderedRawChildren,
    boundary: &mut usize,
) -> Result<()> {
    match name {
        b"txBody" if *boundary == 0 && text_body.is_none() => {
            *text_body = Some(CT_TextBody::from_xml(&raw).map_err(|error| {
                OxmlError::InvalidValue(format!("invalid a:tc/a:txBody: {error}"))
            })?);
            *boundary = 1;
        }
        b"tcPr" if *boundary <= 1 && cell_properties.is_none() => {
            *cell_properties = Some(parse_cell_properties(&raw)?);
            *boundary = 2;
        }
        b"txBody" | b"tcPr" => {
            return Err(OxmlError::InvalidValue(
                "a:tc children must be a:txBody? then a:tcPr?".to_owned(),
            ));
        }
        _ => raw_children.push(*boundary, raw),
    }
    Ok(())
}

fn parse_cell_properties(xml: &[u8]) -> Result<CellPropertiesRaw> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) => {
                let attributes = raw_attributes(&start, &[], false)?;
                let mut children = Vec::new();
                loop {
                    buffer.clear();
                    match reader.read_event_into(&mut buffer)? {
                        Event::Start(child) => children.push(capture_element(&mut reader, &child)?),
                        Event::Empty(child) => children.push(capture_empty_element(&child)?),
                        Event::End(end) if matches_local_name(end.name().as_ref(), b"tcPr") => {
                            break;
                        }
                        Event::Eof => return Err(missing("closing a:tcPr")),
                        _ => {}
                    }
                }
                return Ok(CellPropertiesRaw {
                    attributes,
                    children,
                });
            }
            Event::Empty(start) => {
                return Ok(CellPropertiesRaw {
                    attributes: raw_attributes(&start, &[], false)?,
                    children: Vec::new(),
                });
            }
            Event::Eof => return Err(missing("a:tcPr")),
            _ => {}
        }
        buffer.clear();
    }
}

fn raw_attributes(
    start: &BytesStart<'_>,
    modelled: &[&[u8]],
    is_root: bool,
) -> Result<RawAttributes> {
    let mut raw = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute?;
        let key = attribute.key.as_ref();
        if modelled.contains(&key) || (is_root && key == b"xmlns:a") {
            continue;
        }
        raw.push((
            std::str::from_utf8(key)?.to_owned(),
            attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())?
                .into_owned(),
        ));
    }
    Ok(raw)
}

fn decoded_attr(start: &BytesStart<'_>, name: &[u8]) -> Result<Option<String>> {
    for attribute in start.attributes() {
        let attribute = attribute?;
        if attribute.key.as_ref() == name {
            return Ok(Some(
                attribute
                    .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())?
                    .into_owned(),
            ));
        }
    }
    Ok(None)
}

fn required_i64(start: &BytesStart<'_>, name: &[u8]) -> Result<i64> {
    let value = decoded_attr(start, name)?.ok_or_else(|| {
        missing(&format!(
            "{}@{}",
            String::from_utf8_lossy(local_name(start.name().as_ref())),
            String::from_utf8_lossy(name)
        ))
    })?;
    value.parse::<i64>().map_err(|error| {
        OxmlError::InvalidValue(format!(
            "invalid {}@{} {value}: {error}",
            String::from_utf8_lossy(local_name(start.name().as_ref())),
            String::from_utf8_lossy(name)
        ))
    })
}

fn positive_u32_attr(start: &BytesStart<'_>, name: &[u8]) -> Result<Option<u32>> {
    let Some(value) = decoded_attr(start, name)? else {
        return Ok(None);
    };
    let parsed = value.parse::<u32>().map_err(|error| {
        OxmlError::InvalidValue(format!(
            "invalid a:tc@{} {value}: {error}",
            String::from_utf8_lossy(name)
        ))
    })?;
    if parsed == 0 {
        return Err(OxmlError::InvalidValue(format!(
            "a:tc@{} must be positive",
            String::from_utf8_lossy(name)
        )));
    }
    Ok(Some(parsed))
}

fn bool_attr(start: &BytesStart<'_>, name: &[u8]) -> Result<Option<bool>> {
    let Some(value) = decoded_attr(start, name)? else {
        return Ok(None);
    };
    match value.as_str() {
        "true" | "1" => Ok(Some(true)),
        "false" | "0" => Ok(Some(false)),
        _ => Err(OxmlError::InvalidValue(format!(
            "invalid {}@{} boolean: {value}",
            String::from_utf8_lossy(local_name(start.name().as_ref())),
            String::from_utf8_lossy(name)
        ))),
    }
}

fn push_true(start: &mut BytesStart<'_>, name: &'static str, value: bool) {
    if value {
        start.push_attribute((name, "1"));
    }
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

fn missing(element: &str) -> OxmlError {
    OxmlError::MissingElement(element.to_owned())
}

fn unexpected(start: &BytesStart<'_>) -> OxmlError {
    OxmlError::UnexpectedElement(String::from_utf8_lossy(start.name().as_ref()).into_owned())
}

#[cfg(test)]
mod tests {
    use super::{A_NS, CT_Table, Emu, OxmlError};

    #[test]
    fn table_properties_preserve_style_and_banding_flags() {
        let xml = br#"<a:tbl xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:tblPr rtl="1" firstRow="true" firstCol="1" lastRow="true" lastCol="1" bandRow="true" bandCol="1"><a:tableStyleId>{5940675A-B579-460E-94D1-54222C63F5DA}</a:tableStyleId></a:tblPr><a:tblGrid><a:gridCol w="1"/></a:tblGrid><a:tr h="2"><a:tc/></a:tr></a:tbl>"#;
        let table = CT_Table::from_xml(xml).unwrap();
        let properties = table.properties.as_ref().unwrap();
        assert!(properties.right_to_left);
        assert!(properties.first_row && properties.first_column);
        assert!(properties.last_row && properties.last_column);
        assert!(properties.band_rows && properties.band_columns);
        assert_eq!(
            properties.style_id.as_deref(),
            Some("{5940675A-B579-460E-94D1-54222C63F5DA}")
        );
        assert_eq!(table, CT_Table::from_xml(&table.to_xml().unwrap()).unwrap());
    }

    #[test]
    fn table_reader_is_prefix_tolerant_and_writer_uses_schema_order() {
        let xml = br#"<q:tbl xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/main"><q:tblPr firstRow="1"/><q:tblGrid><q:gridCol w="100"/></q:tblGrid><q:tr h="200"><q:tc><q:txBody><q:bodyPr/><q:p/></q:txBody><q:tcPr/></q:tc></q:tr></q:tbl>"#;
        let table = CT_Table::from_xml(xml).unwrap();
        let written = table.to_xml().unwrap();
        let text = std::str::from_utf8(&written).unwrap();
        assert!(text.starts_with("<a:tbl xmlns:a="));
        assert_order(
            text,
            &[
                "<a:tblPr",
                "<a:tblGrid",
                "<a:tr",
                "<a:tc",
                "<a:txBody",
                "<a:tcPr",
            ],
        );
        let reparsed = CT_Table::from_xml(&written).unwrap();
        assert_eq!(table.grid, reparsed.grid);
        assert_eq!(table.rows[0], reparsed.rows[0]);
        assert_eq!(table, reparsed);
    }

    #[test]
    fn unmodelled_table_and_cell_content_is_preserved_in_place() {
        let xml = br#"<q:tbl xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer" x:table="kept"><x:before/><x:opaque xmlns:a="urn:producer"><a:data/></x:opaque><q:tblPr x:properties="kept"><x:fill/><q:tableStyleId>style</q:tableStyleId><x:effect/></q:tblPr><x:between/><q:tblGrid x:grid="kept"><x:grid-before/><q:gridCol w="100" x:column="kept"><x:column-child/></q:gridCol><x:grid-after/></q:tblGrid><x:before-row/><q:tr h="200" x:row="kept"><x:before-cell/><q:tc x:cell="kept"><x:before-text/><x:cell-opaque xmlns:a="urn:producer"><a:data/></x:cell-opaque><q:txBody><q:bodyPr/><q:p/></q:txBody><x:before-properties/><q:tcPr x:style="kept"><x:border/></q:tcPr><x:after-properties/></q:tc><x:after-cell/></q:tr><x:after/></q:tbl>"#;
        let table = CT_Table::from_xml(xml).unwrap();
        let written = table.to_xml().unwrap();
        let text = std::str::from_utf8(&written).unwrap();
        for raw in [
            r#"<x:before/>"#,
            r#"<x:opaque xmlns:a="urn:producer"><a:data/></x:opaque>"#,
            r#"<x:fill/>"#,
            r#"<x:effect/>"#,
            r#"<x:column-child/>"#,
            r#"<a:tcPr x:style="kept"><x:border/></a:tcPr>"#,
            r#"<x:cell-opaque xmlns:a="urn:producer"><a:data/></x:cell-opaque>"#,
            r#"<x:after-properties/>"#,
            r#"<x:after/>"#,
        ] {
            assert!(text.contains(raw), "missing {raw}");
        }
        assert_order(
            text,
            &[
                "<x:before-text",
                "<a:txBody",
                "<x:before-properties",
                "<a:tcPr",
                "<x:after-properties",
            ],
        );
        assert_eq!(table, CT_Table::from_xml(&written).unwrap());
    }

    #[test]
    fn collection_edits_keep_surviving_metadata_and_trailing_raw_children() {
        let xml = br#"<a:tbl xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer"><a:tblGrid><a:gridCol w="100" x:id="first"><x:first-child/></a:gridCol><a:gridCol w="200" x:id="second"><x:second-child/></a:gridCol><x:grid-tail/></a:tblGrid><a:tr h="300"><a:tc/><x:between-cells/><a:tc/><x:row-tail/></a:tr><x:between-rows/><a:tr h="400"><a:tc/></a:tr><x:table-tail/></a:tbl>"#;
        let mut table = CT_Table::from_xml(xml).unwrap();
        table.grid.columns.swap(0, 1);
        table.grid.columns.remove(1);
        table.rows[0].cells.remove(0);
        table.rows[0].cells[0].grid_span = 2;
        table.rows.remove(1);

        let written = table.to_xml().unwrap();
        let text = std::str::from_utf8(&written).unwrap();
        assert!(text.contains(r#"<a:gridCol w="200" x:id="second"><x:second-child/></a:gridCol>"#));
        assert!(!text.contains(r#"x:id="first""#));
        for raw in [
            "<x:grid-tail/>",
            "<x:between-cells/>",
            "<x:row-tail/>",
            "<x:between-rows/>",
            "<x:table-tail/>",
        ] {
            assert!(text.contains(raw), "missing {raw}");
        }
        assert!(text.contains(
            r#"<a:tr h="300"><x:between-cells/><a:tc gridSpan="2"/><x:row-tail/></a:tr>"#
        ));
        let reparsed = CT_Table::from_xml(&written).unwrap();
        assert_eq!(table.grid, reparsed.grid);
        assert_eq!(table.rows[0], reparsed.rows[0]);
        assert_eq!(table, reparsed);

        let mut width_edit = CT_Table::from_xml(xml).unwrap();
        width_edit.grid.columns[1] = Emu(250);
        let width_edit_xml = width_edit.to_xml().unwrap();
        assert!(
            std::str::from_utf8(&width_edit_xml)
                .unwrap()
                .contains(r#"<a:gridCol w="250" x:id="second"><x:second-child/></a:gridCol>"#)
        );
        assert_eq!(width_edit, CT_Table::from_xml(&width_edit_xml).unwrap());
    }

    #[test]
    fn ambiguous_grid_edits_return_a_typed_serialization_error() {
        let xml = br#"<a:tbl xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer"><a:tblGrid><a:gridCol w="100" x:id="first"/><a:gridCol w="200" x:id="second"/></a:tblGrid><a:tr h="300"><a:tc/></a:tr></a:tbl>"#;

        let mut collision = CT_Table::from_xml(xml).unwrap();
        collision.grid.columns[0] = Emu(200);
        assert_ambiguous_grid_error(collision.to_xml().unwrap_err());

        let mut combined_reorder_and_edit = CT_Table::from_xml(xml).unwrap();
        combined_reorder_and_edit.grid.columns.swap(0, 1);
        combined_reorder_and_edit.grid.columns[0] = Emu(100);
        assert_ambiguous_grid_error(combined_reorder_and_edit.to_xml().unwrap_err());

        let mut duplicate_insertion = CT_Table::from_xml(xml).unwrap();
        duplicate_insertion.grid.columns.insert(0, Emu(100));
        assert_ambiguous_grid_error(duplicate_insertion.to_xml().unwrap_err());

        let mut delete_and_edit = CT_Table::from_xml(xml).unwrap();
        delete_and_edit.grid.columns.remove(1);
        delete_and_edit.grid.columns[0] = Emu(150);
        assert_ambiguous_grid_error(delete_and_edit.to_xml().unwrap_err());

        let mut insert_and_edit = CT_Table::from_xml(xml).unwrap();
        insert_and_edit.grid.columns[0] = Emu(150);
        insert_and_edit.grid.columns.insert(1, Emu(300));
        assert_ambiguous_grid_error(insert_and_edit.to_xml().unwrap_err());
    }

    #[test]
    fn inherited_namespaces_make_extracted_table_output_self_contained() {
        let xml = br#"<q:tbl><q:tblGrid><q:gridCol w="100"/></q:tblGrid><q:tr h="200"><q:tc><x:extension/></q:tc></q:tr></q:tbl>"#;
        let inherited = vec![
            ("q".to_owned(), A_NS.to_owned()),
            ("x".to_owned(), "urn:producer".to_owned()),
        ];
        let table = CT_Table::from_xml_with_inherited_namespaces(xml, &inherited).unwrap();
        let written = table.to_xml().unwrap();
        let text = std::str::from_utf8(&written).unwrap();
        assert!(text.starts_with(&format!(r#"<a:tbl xmlns:a="{A_NS}""#)));
        assert!(text.contains(r#"xmlns:x="urn:producer""#));
        assert!(text.contains("<x:extension/>"));
        assert_eq!(table, CT_Table::from_xml(&written).unwrap());

        let locally_shadowed = br#"<q:tbl xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><q:tblGrid><q:gridCol w="100"/></q:tblGrid><q:tr h="200"><q:tc/></q:tr></q:tbl>"#;
        let producer_a = vec![("a".to_owned(), "urn:producer".to_owned())];
        assert!(
            CT_Table::from_xml_with_inherited_namespaces(locally_shadowed, &producer_a).is_ok()
        );
    }

    #[test]
    fn inherited_namespace_prefixes_accept_xml_ncname_unicode() {
        let xml = r#"<q:tbl><q:tblGrid><q:gridCol w="100"/></q:tblGrid><q:tr h="200"><q:tc><é:extension/></q:tc></q:tr></q:tbl>"#;
        let inherited = vec![
            ("q".to_owned(), A_NS.to_owned()),
            ("é".to_owned(), "urn:producer".to_owned()),
        ];
        let table =
            CT_Table::from_xml_with_inherited_namespaces(xml.as_bytes(), &inherited).unwrap();
        let written = table.to_xml().unwrap();
        let text = std::str::from_utf8(&written).unwrap();
        assert!(text.contains(r#"xmlns:é="urn:producer""#));
        assert!(text.contains("<é:extension/>"));
        assert_eq!(table, CT_Table::from_xml(&written).unwrap());
    }

    #[test]
    fn optional_child_edits_round_trip_as_equal_models() {
        let xml = br#"<a:tbl xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer"><a:tblPr><x:before-style/><a:tableStyleId>style</a:tableStyleId><x:after-style/></a:tblPr><x:between-properties-grid/><a:tblGrid><a:gridCol w="100"/></a:tblGrid><a:tr h="200"><a:tc><x:before-text/><a:txBody><a:bodyPr/><a:p/></a:txBody><x:between-text-properties/><a:tcPr/></a:tc></a:tr></a:tbl>"#;

        let mut no_properties = CT_Table::from_xml(xml).unwrap();
        no_properties.properties = None;
        let written = no_properties.to_xml().unwrap();
        assert_eq!(no_properties, CT_Table::from_xml(&written).unwrap());

        let mut no_style = CT_Table::from_xml(xml).unwrap();
        no_style.properties.as_mut().unwrap().style_id = None;
        let written = no_style.to_xml().unwrap();
        let reparsed = CT_Table::from_xml(&written).unwrap();
        assert_eq!(
            no_style.properties.as_ref().unwrap(),
            reparsed.properties.as_ref().unwrap()
        );
        assert_eq!(no_style, reparsed);

        let mut no_text = CT_Table::from_xml(xml).unwrap();
        no_text.rows[0].cells[0].text_body = None;
        let written = no_text.to_xml().unwrap();
        let reparsed = CT_Table::from_xml(&written).unwrap();
        assert_eq!(no_text.rows[0].cells[0], reparsed.rows[0].cells[0]);
        assert_eq!(no_text, reparsed);
    }

    fn assert_ambiguous_grid_error(error: OxmlError) {
        assert!(matches!(
            error,
            OxmlError::InvalidValue(message)
                if message == "edited table grid is ambiguous because preserved column metadata cannot be associated reliably"
        ));
    }

    fn assert_order(text: &str, tags: &[&str]) {
        let mut previous = 0usize;
        for tag in tags {
            let position = text[previous..].find(tag).unwrap() + previous;
            assert!(position >= previous, "{tag}");
            previous = position + tag.len();
        }
    }
}
