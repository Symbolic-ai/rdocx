//! Minimal SpreadsheetML workbook writer for editable OOXML chart data.
//!
//! This crate writes one worksheet with text and numeric columns. It is not a
//! general spreadsheet library and deliberately excludes workbook reading,
//! formulas, multiple worksheets, charts, and arbitrary cell styling.

use std::collections::HashMap;
use std::io::Cursor;

use oxml_opc::relationship::rel_types;
use oxml_opc::{OpcPackage, content_types};
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use thiserror::Error;

const SML_NS: &str = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
const REL_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const MAX_COLUMNS: usize = 16_384;
const MAX_ROWS: usize = 1_048_576;
const FIRST_CUSTOM_NUMBER_FORMAT_ID: u32 = 164;

/// A typed worksheet column.
#[derive(Debug, Clone, PartialEq)]
pub enum Column {
    /// A shared-string header followed by shared-string values.
    Text {
        /// The row-one header.
        header: String,
        /// Values beginning in row two.
        values: Vec<String>,
    },
    /// A shared-string header followed by numeric values.
    Number {
        /// The row-one header.
        header: String,
        /// Values beginning in row two.
        values: Vec<f64>,
        /// An optional Excel number format for every value cell.
        number_format: Option<String>,
    },
}

impl Column {
    fn header(&self) -> &str {
        match self {
            Self::Text { header, .. } | Self::Number { header, .. } => header,
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Text { values, .. } => values.len(),
            Self::Number { values, .. } => values.len(),
        }
    }
}

/// A validated, single-sheet chart workbook.
#[derive(Debug, Clone, PartialEq)]
pub struct Workbook {
    sheet_name: String,
    columns: Vec<Column>,
}

/// A workbook construction or serialization error.
#[derive(Debug, Error)]
pub enum Error {
    /// Worksheet names must follow Excel's one-sheet naming rules.
    #[error("invalid worksheet name: {0}")]
    InvalidSheetName(&'static str),
    /// At least one column is required.
    #[error("a workbook must contain at least one column")]
    NoColumns,
    /// SpreadsheetML addresses stop at column XFD.
    #[error("workbook has {0} columns, exceeding the SpreadsheetML limit of 16384")]
    TooManyColumns(usize),
    /// SpreadsheetML numeric cells cannot carry NaN or infinity.
    #[error("column {column}, row {row} contains a nonfinite number")]
    NonFiniteNumber {
        /// Zero-based column index.
        column: usize,
        /// One-based worksheet row, including the header row.
        row: usize,
    },
    /// The row-one header leaves room for at most 1,048,575 data values.
    #[error("column {column} has {values} values, exceeding the SpreadsheetML limit of 1048575")]
    TooManyRows {
        /// Zero-based column index.
        column: usize,
        /// Number of data values in the column.
        values: usize,
    },
    /// Shared-string indexes and counts are SpreadsheetML unsigned integers.
    #[error("workbook has {0} shared-string references, exceeding the u32 limit")]
    TooManySharedStrings(u64),
    /// XML serialization failed.
    #[error("SpreadsheetML XML error: {0}")]
    Xml(#[from] quick_xml::Error),
    /// Writing XML to its byte buffer failed.
    #[error("SpreadsheetML I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// OPC package construction failed.
    #[error("SpreadsheetML package error: {0}")]
    Opc(#[from] oxml_opc::OpcError),
}

/// Result type for workbook operations.
pub type Result<T> = std::result::Result<T, Error>;

impl Workbook {
    /// Validate and construct a single-sheet workbook.
    pub fn new(sheet_name: impl Into<String>, columns: Vec<Column>) -> Result<Self> {
        let sheet_name = sheet_name.into();
        validate_sheet_name(&sheet_name)?;
        if columns.is_empty() {
            return Err(Error::NoColumns);
        }
        if columns.len() > MAX_COLUMNS {
            return Err(Error::TooManyColumns(columns.len()));
        }
        for (column, value) in columns.iter().enumerate() {
            if value.len() >= MAX_ROWS {
                return Err(Error::TooManyRows {
                    column,
                    values: value.len(),
                });
            }
            if let Column::Number { values, .. } = value {
                for (value_index, value) in values.iter().enumerate() {
                    if !value.is_finite() {
                        return Err(Error::NonFiniteNumber {
                            column,
                            row: value_index + 2,
                        });
                    }
                }
            }
        }
        let text_value_counts = columns
            .iter()
            .filter_map(|column| match column {
                Column::Text { values, .. } => Some(values.len()),
                Column::Number { .. } => None,
            })
            .collect::<Vec<_>>();
        validate_shared_string_count(columns.len(), &text_value_counts)?;
        Ok(Self {
            sheet_name,
            columns,
        })
    }

    /// Serialize this workbook as deterministic `.xlsx` bytes.
    pub fn to_xlsx_bytes(&self) -> Result<Vec<u8>> {
        let shared_strings = SharedStrings::from_columns(&self.columns)?;
        let styles = Styles::from_columns(&self.columns);
        let mut package = OpcPackage::with_main_part("xl/workbook.xml", content_types::WORKBOOK);

        package
            .content_types
            .add_override("/xl/worksheets/sheet1.xml", content_types::WORKSHEET);
        package
            .content_types
            .add_override("/xl/sharedStrings.xml", content_types::SHARED_STRINGS);
        if !styles.formats.is_empty() {
            package
                .content_types
                .add_override("/xl/styles.xml", content_types::STYLES);
        }

        let workbook_rels = package.get_or_create_part_rels("/xl/workbook.xml");
        let worksheet_rel_id = workbook_rels.add(rel_types::WORKSHEET, "worksheets/sheet1.xml");
        workbook_rels.add(rel_types::SHARED_STRINGS, "sharedStrings.xml");
        if !styles.formats.is_empty() {
            workbook_rels.add(rel_types::STYLES, "styles.xml");
        }

        package.set_part("/xl/workbook.xml", self.workbook_xml(&worksheet_rel_id)?);
        package.set_part(
            "/xl/worksheets/sheet1.xml",
            self.worksheet_xml(&shared_strings, &styles)?,
        );
        package.set_part("/xl/sharedStrings.xml", shared_strings.to_xml()?);
        if !styles.formats.is_empty() {
            package.set_part("/xl/styles.xml", styles.to_xml()?);
        }

        let mut output = Cursor::new(Vec::new());
        package.write_to(&mut output)?;
        Ok(output.into_inner())
    }

    /// Return the formula-addressable data range for a column.
    pub fn formula_range(&self, column: usize) -> Option<String> {
        let data_len = self.columns.get(column)?.len();
        if data_len == 0 {
            return None;
        }
        let name = formula_sheet_name(&self.sheet_name);
        let address = column_name(column);
        Some(format!("{name}!${address}$2:${address}${}", data_len + 1))
    }

    fn workbook_xml(&self, worksheet_rel_id: &str) -> Result<Vec<u8>> {
        let mut writer = xml_writer()?;
        let mut workbook = BytesStart::new("workbook");
        workbook.push_attribute(("xmlns", SML_NS));
        workbook.push_attribute(("xmlns:r", REL_NS));
        writer.write_event(Event::Start(workbook))?;
        writer.write_event(Event::Start(BytesStart::new("sheets")))?;
        let mut sheet = BytesStart::new("sheet");
        let sheet_name = encode_spreadsheet_text(&self.sheet_name);
        sheet.push_attribute(("name", sheet_name.as_str()));
        sheet.push_attribute(("sheetId", "1"));
        sheet.push_attribute(("r:id", worksheet_rel_id));
        writer.write_event(Event::Empty(sheet))?;
        writer.write_event(Event::End(BytesEnd::new("sheets")))?;
        writer.write_event(Event::End(BytesEnd::new("workbook")))?;
        Ok(writer.into_inner())
    }

    fn worksheet_xml(&self, strings: &SharedStrings, styles: &Styles) -> Result<Vec<u8>> {
        let mut writer = xml_writer()?;
        let mut worksheet = BytesStart::new("worksheet");
        worksheet.push_attribute(("xmlns", SML_NS));
        writer.write_event(Event::Start(worksheet))?;
        writer.write_event(Event::Start(BytesStart::new("sheetData")))?;

        write_row_start(&mut writer, 1)?;
        for (column, value) in self.columns.iter().enumerate() {
            write_shared_string_cell(
                &mut writer,
                &cell_address(column, 1),
                strings.index(value.header()),
            )?;
        }
        writer.write_event(Event::End(BytesEnd::new("row")))?;

        let row_count = self.columns.iter().map(Column::len).max().unwrap_or(0);
        for value_index in 0..row_count {
            let row = value_index + 2;
            write_row_start(&mut writer, row)?;
            for (column_index, column) in self.columns.iter().enumerate() {
                let address = cell_address(column_index, row);
                match column {
                    Column::Text { values, .. } => {
                        if let Some(value) = values.get(value_index) {
                            write_shared_string_cell(&mut writer, &address, strings.index(value))?;
                        }
                    }
                    Column::Number {
                        values,
                        number_format,
                        ..
                    } => {
                        if let Some(value) = values.get(value_index) {
                            write_number_cell(
                                &mut writer,
                                &address,
                                *value,
                                number_format
                                    .as_deref()
                                    .and_then(|format| styles.style_index(format)),
                            )?;
                        }
                    }
                }
            }
            writer.write_event(Event::End(BytesEnd::new("row")))?;
        }

        writer.write_event(Event::End(BytesEnd::new("sheetData")))?;
        writer.write_event(Event::End(BytesEnd::new("worksheet")))?;
        Ok(writer.into_inner())
    }
}

fn validate_sheet_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(Error::InvalidSheetName("name is empty"));
    }
    if name.encode_utf16().count() > 31 {
        return Err(Error::InvalidSheetName("name exceeds 31 UTF-16 code units"));
    }
    if name.starts_with('\'') || name.ends_with('\'') {
        return Err(Error::InvalidSheetName(
            "name begins or ends with an apostrophe",
        ));
    }
    if name.chars().any(|character| {
        matches!(character, ':' | '\\' | '/' | '?' | '*' | '[' | ']') || character.is_control()
    }) {
        return Err(Error::InvalidSheetName(
            "name contains a forbidden character",
        ));
    }
    Ok(())
}

fn formula_sheet_name(name: &str) -> String {
    if name.chars().enumerate().all(|(index, character)| {
        character.is_ascii_alphanumeric() || character == '_' || (character == '.' && index > 0)
    }) && name
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
    {
        name.to_string()
    } else {
        format!("'{}'", name.replace('\'', "''"))
    }
}

fn encode_spreadsheet_text(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for (index, character) in value.char_indices() {
        if character == '_' && starts_spreadsheet_escape(value.as_bytes(), index) {
            encoded.push_str("_x005F_");
        } else if matches!(character as u32, 0x0000..=0x001F | 0xFFFE | 0xFFFF) {
            encoded.push_str(&format!("_x{:04X}_", character as u32));
        } else {
            encoded.push(character);
        }
    }
    encoded
}

fn starts_spreadsheet_escape(bytes: &[u8], index: usize) -> bool {
    bytes.get(index..index + 7).is_some_and(|candidate| {
        candidate[0] == b'_'
            && candidate[1].eq_ignore_ascii_case(&b'x')
            && candidate[2..6].iter().all(u8::is_ascii_hexdigit)
            && candidate[6] == b'_'
    })
}

fn validate_shared_string_count(header_count: usize, text_value_counts: &[usize]) -> Result<u32> {
    let mut count = header_count as u64;
    for value_count in text_value_counts {
        count = count.saturating_add(*value_count as u64);
        if count > u32::MAX as u64 {
            return Err(Error::TooManySharedStrings(count));
        }
    }
    u32::try_from(count).map_err(|_| Error::TooManySharedStrings(count))
}

fn column_name(mut index: usize) -> String {
    let mut reversed = Vec::new();
    loop {
        reversed.push((b'A' + (index % 26) as u8) as char);
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }
    reversed.into_iter().rev().collect()
}

fn cell_address(column: usize, row: usize) -> String {
    format!("{}{row}", column_name(column))
}

fn xml_writer() -> Result<Writer<Vec<u8>>> {
    let mut writer = Writer::new(Vec::new());
    writer.write_event(Event::Decl(BytesDecl::new(
        "1.0",
        Some("UTF-8"),
        Some("yes"),
    )))?;
    Ok(writer)
}

fn write_row_start(writer: &mut Writer<Vec<u8>>, row: usize) -> Result<()> {
    let row = row.to_string();
    let mut start = BytesStart::new("row");
    start.push_attribute(("r", row.as_str()));
    writer.write_event(Event::Start(start))?;
    Ok(())
}

fn write_shared_string_cell(writer: &mut Writer<Vec<u8>>, address: &str, index: u32) -> Result<()> {
    let mut cell = BytesStart::new("c");
    cell.push_attribute(("r", address));
    cell.push_attribute(("t", "s"));
    writer.write_event(Event::Start(cell))?;
    write_text_element(writer, "v", &index.to_string())?;
    writer.write_event(Event::End(BytesEnd::new("c")))?;
    Ok(())
}

fn write_number_cell(
    writer: &mut Writer<Vec<u8>>,
    address: &str,
    value: f64,
    style_index: Option<u32>,
) -> Result<()> {
    let style_index = style_index.map(|index| index.to_string());
    let mut cell = BytesStart::new("c");
    cell.push_attribute(("r", address));
    if let Some(index) = style_index.as_deref() {
        cell.push_attribute(("s", index));
    }
    writer.write_event(Event::Start(cell))?;
    write_text_element(writer, "v", &value.to_string())?;
    writer.write_event(Event::End(BytesEnd::new("c")))?;
    Ok(())
}

fn write_text_element(writer: &mut Writer<Vec<u8>>, name: &str, value: &str) -> Result<()> {
    writer.write_event(Event::Start(BytesStart::new(name)))?;
    writer.write_event(Event::Text(BytesText::new(value)))?;
    writer.write_event(Event::End(BytesEnd::new(name)))?;
    Ok(())
}

struct SharedStrings {
    values: Vec<String>,
    indexes: HashMap<String, u32>,
    count: u32,
}

impl SharedStrings {
    fn from_columns(columns: &[Column]) -> Result<Self> {
        let text_value_counts = columns
            .iter()
            .filter_map(|column| match column {
                Column::Text { values, .. } => Some(values.len()),
                Column::Number { .. } => None,
            })
            .collect::<Vec<_>>();
        let count = validate_shared_string_count(columns.len(), &text_value_counts)?;
        let mut shared = Self {
            values: Vec::new(),
            indexes: HashMap::new(),
            count,
        };
        for column in columns {
            shared.intern(column.header())?;
        }
        let row_count = columns.iter().map(Column::len).max().unwrap_or(0);
        for row in 0..row_count {
            for column in columns {
                if let Column::Text { values, .. } = column
                    && let Some(value) = values.get(row)
                {
                    shared.intern(value)?;
                }
            }
        }
        Ok(shared)
    }

    fn intern(&mut self, value: &str) -> Result<u32> {
        if let Some(index) = self.indexes.get(value) {
            return Ok(*index);
        }
        let index = u32::try_from(self.values.len()).map_err(|_| {
            Error::TooManySharedStrings((self.values.len() as u64).saturating_add(1))
        })?;
        self.values.push(value.to_string());
        self.indexes.insert(value.to_string(), index);
        Ok(index)
    }

    fn index(&self, value: &str) -> u32 {
        self.indexes[value]
    }

    fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = xml_writer()?;
        let count = self.count.to_string();
        let unique_count = self.values.len().to_string();
        let mut table = BytesStart::new("sst");
        table.push_attribute(("xmlns", SML_NS));
        table.push_attribute(("count", count.as_str()));
        table.push_attribute(("uniqueCount", unique_count.as_str()));
        writer.write_event(Event::Start(table))?;
        for value in &self.values {
            writer.write_event(Event::Start(BytesStart::new("si")))?;
            let mut text = BytesStart::new("t");
            if value.starts_with(char::is_whitespace) || value.ends_with(char::is_whitespace) {
                text.push_attribute(("xml:space", "preserve"));
            }
            writer.write_event(Event::Start(text))?;
            let value = encode_spreadsheet_text(value);
            writer.write_event(Event::Text(BytesText::new(&value)))?;
            writer.write_event(Event::End(BytesEnd::new("t")))?;
            writer.write_event(Event::End(BytesEnd::new("si")))?;
        }
        writer.write_event(Event::End(BytesEnd::new("sst")))?;
        Ok(writer.into_inner())
    }
}

struct Styles {
    formats: Vec<String>,
    indexes: HashMap<String, u32>,
}

impl Styles {
    fn from_columns(columns: &[Column]) -> Self {
        let mut formats = Vec::new();
        let mut indexes = HashMap::new();
        for column in columns {
            if let Column::Number {
                number_format: Some(format),
                ..
            } = column
                && !indexes.contains_key(format)
            {
                let index = u32::try_from(formats.len() + 1).expect("column limit fits u32");
                indexes.insert(format.clone(), index);
                formats.push(format.clone());
            }
        }
        Self { formats, indexes }
    }

    fn style_index(&self, format: &str) -> Option<u32> {
        self.indexes.get(format).copied()
    }

    fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = xml_writer()?;
        let mut root = BytesStart::new("styleSheet");
        root.push_attribute(("xmlns", SML_NS));
        writer.write_event(Event::Start(root))?;

        let format_count = self.formats.len().to_string();
        let mut num_formats = BytesStart::new("numFmts");
        num_formats.push_attribute(("count", format_count.as_str()));
        writer.write_event(Event::Start(num_formats))?;
        for (offset, format) in self.formats.iter().enumerate() {
            let id = (FIRST_CUSTOM_NUMBER_FORMAT_ID + offset as u32).to_string();
            let mut entry = BytesStart::new("numFmt");
            entry.push_attribute(("numFmtId", id.as_str()));
            let format = encode_spreadsheet_text(format);
            entry.push_attribute(("formatCode", format.as_str()));
            writer.write_event(Event::Empty(entry))?;
        }
        writer.write_event(Event::End(BytesEnd::new("numFmts")))?;

        write_fonts(&mut writer)?;
        write_fills(&mut writer)?;
        write_borders(&mut writer)?;
        write_base_cell_style(&mut writer)?;

        let cell_format_count = (self.formats.len() + 1).to_string();
        let mut cell_formats = BytesStart::new("cellXfs");
        cell_formats.push_attribute(("count", cell_format_count.as_str()));
        writer.write_event(Event::Start(cell_formats))?;
        write_xf(&mut writer, "0", false)?;
        for offset in 0..self.formats.len() {
            let id = (FIRST_CUSTOM_NUMBER_FORMAT_ID + offset as u32).to_string();
            write_xf(&mut writer, &id, true)?;
        }
        writer.write_event(Event::End(BytesEnd::new("cellXfs")))?;

        writer.write_event(Event::Start(counted("cellStyles", "1")))?;
        let mut normal = BytesStart::new("cellStyle");
        normal.push_attribute(("name", "Normal"));
        normal.push_attribute(("xfId", "0"));
        normal.push_attribute(("builtinId", "0"));
        writer.write_event(Event::Empty(normal))?;
        writer.write_event(Event::End(BytesEnd::new("cellStyles")))?;
        writer.write_event(Event::Empty(counted("dxfs", "0")))?;
        let mut tables = counted("tableStyles", "0");
        tables.push_attribute(("defaultTableStyle", "TableStyleMedium2"));
        tables.push_attribute(("defaultPivotStyle", "PivotStyleLight16"));
        writer.write_event(Event::Empty(tables))?;
        writer.write_event(Event::End(BytesEnd::new("styleSheet")))?;
        Ok(writer.into_inner())
    }
}

fn counted<'a>(name: &'a str, count: &'a str) -> BytesStart<'a> {
    let mut start = BytesStart::new(name);
    start.push_attribute(("count", count));
    start
}

fn write_fonts(writer: &mut Writer<Vec<u8>>) -> Result<()> {
    writer.write_event(Event::Start(counted("fonts", "1")))?;
    writer.write_event(Event::Start(BytesStart::new("font")))?;
    for (name, key, value) in [
        ("sz", "val", "11"),
        ("color", "theme", "1"),
        ("name", "val", "Calibri"),
        ("family", "val", "2"),
        ("scheme", "val", "minor"),
    ] {
        let mut child = BytesStart::new(name);
        child.push_attribute((key, value));
        writer.write_event(Event::Empty(child))?;
    }
    writer.write_event(Event::End(BytesEnd::new("font")))?;
    writer.write_event(Event::End(BytesEnd::new("fonts")))?;
    Ok(())
}

fn write_fills(writer: &mut Writer<Vec<u8>>) -> Result<()> {
    writer.write_event(Event::Start(counted("fills", "2")))?;
    for pattern in ["none", "gray125"] {
        writer.write_event(Event::Start(BytesStart::new("fill")))?;
        let mut fill = BytesStart::new("patternFill");
        fill.push_attribute(("patternType", pattern));
        writer.write_event(Event::Empty(fill))?;
        writer.write_event(Event::End(BytesEnd::new("fill")))?;
    }
    writer.write_event(Event::End(BytesEnd::new("fills")))?;
    Ok(())
}

fn write_borders(writer: &mut Writer<Vec<u8>>) -> Result<()> {
    writer.write_event(Event::Start(counted("borders", "1")))?;
    writer.write_event(Event::Start(BytesStart::new("border")))?;
    for name in ["left", "right", "top", "bottom", "diagonal"] {
        writer.write_event(Event::Empty(BytesStart::new(name)))?;
    }
    writer.write_event(Event::End(BytesEnd::new("border")))?;
    writer.write_event(Event::End(BytesEnd::new("borders")))?;
    Ok(())
}

fn write_base_cell_style(writer: &mut Writer<Vec<u8>>) -> Result<()> {
    writer.write_event(Event::Start(counted("cellStyleXfs", "1")))?;
    write_xf(writer, "0", false)?;
    writer.write_event(Event::End(BytesEnd::new("cellStyleXfs")))?;
    Ok(())
}

fn write_xf(writer: &mut Writer<Vec<u8>>, number_format: &str, apply: bool) -> Result<()> {
    let mut format = BytesStart::new("xf");
    format.push_attribute(("numFmtId", number_format));
    format.push_attribute(("fontId", "0"));
    format.push_attribute(("fillId", "0"));
    format.push_attribute(("borderId", "0"));
    format.push_attribute(("xfId", "0"));
    if apply {
        format.push_attribute(("applyNumberFormat", "1"));
    }
    writer.write_event(Event::Empty(format))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Cursor;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use oxml_opc::OpcPackage;

    use super::*;

    const EXCEL_VERSION: &str = "16.104";
    const EXCEL_BUILD: &str = "16.104.25121423";
    const LIBREOFFICE_VERSION: &str =
        "LibreOffice 26.2.5.2 cd7284b4cbbfeb507e630c1aac019f4157393acb";
    const VIEWER_ARTIFACT_SHA256: &str =
        "8f8d12aa4ebe94f86c8164fd251cdb23845f985090be0fb6c77242aaa0fba329";

    fn sample_workbook() -> Workbook {
        Workbook::new(
            "Sales '24",
            vec![
                Column::Text {
                    header: "Category".into(),
                    values: vec!["North & West".into(), "North & West".into()],
                },
                Column::Number {
                    header: "Revenue".into(),
                    values: vec![12.5, 20.0],
                    number_format: Some("$#,##0.00".into()),
                },
            ],
        )
        .expect("sample workbook should be valid")
    }

    #[test]
    fn formula_ranges_quote_sheet_names_and_track_column_lengths() {
        let workbook = Workbook::new(
            "Sales '24",
            vec![
                Column::Text {
                    header: "Category".into(),
                    values: vec!["North".into(), "South".into(), "West".into()],
                },
                Column::Number {
                    header: "Revenue".into(),
                    values: vec![12.5],
                    number_format: None,
                },
                Column::Text {
                    header: "Empty".into(),
                    values: Vec::new(),
                },
            ],
        )
        .unwrap();
        assert_eq!(
            workbook.formula_range(0).as_deref(),
            Some("'Sales ''24'!$A$2:$A$4")
        );
        assert_eq!(
            workbook.formula_range(1).as_deref(),
            Some("'Sales ''24'!$B$2:$B$2")
        );
        assert_eq!(workbook.formula_range(2), None);
        assert_eq!(workbook.formula_range(3), None);
        assert_eq!(
            Workbook::new("Sheet1", vec![text_column()])
                .unwrap()
                .formula_range(0)
                .as_deref(),
            Some("Sheet1!$A$2:$A$2")
        );
        assert_eq!(column_name(16_383), "XFD");
    }

    #[test]
    fn invalid_workbook_inputs_fail_before_package_construction() {
        assert!(Workbook::new("", vec![text_column()]).is_err());
        assert!(Workbook::new("bad/name", vec![text_column()]).is_err());
        assert!(Workbook::new("'bad", vec![text_column()]).is_err());
        assert!(Workbook::new("12345678901234567890123456789012", vec![text_column()]).is_err());
        assert!(Workbook::new("Data", Vec::new()).is_err());
        assert!(Workbook::new("Data", vec![text_column(); MAX_COLUMNS + 1]).is_err());
        assert!(
            Workbook::new(
                "Data",
                vec![Column::Number {
                    header: "Value".into(),
                    values: vec![f64::NAN],
                    number_format: None,
                }]
            )
            .is_err()
        );

        let maximum = Workbook::new(
            "Data",
            vec![Column::Number {
                header: "Value".into(),
                values: vec![0.0; MAX_ROWS - 1],
                number_format: None,
            }],
        )
        .expect("maximum valid data row should be accepted");
        assert_eq!(
            maximum.formula_range(0).as_deref(),
            Some("Data!$A$2:$A$1048576")
        );
        assert!(matches!(
            Workbook::new(
                "Data",
                vec![Column::Number {
                    header: "Value".into(),
                    values: vec![0.0; MAX_ROWS],
                    number_format: None,
                }]
            ),
            Err(Error::TooManyRows {
                column: 0,
                values: MAX_ROWS
            })
        ));
    }

    #[test]
    fn spreadsheet_strings_escape_xml_and_reserved_sequences_exactly() {
        let workbook = Workbook::new(
            "Data_x000D_",
            vec![
                Column::Text {
                    header: "<&\r\u{1}\u{fffe}_x000D_".into(),
                    values: vec![" value\t\n\u{b}\u{ffff}_x000A_ ".into()],
                },
                Column::Number {
                    header: "Number".into(),
                    values: vec![1.0],
                    number_format: Some("0\t\n<&\r\u{2}\u{fffe}_x000D_".into()),
                },
            ],
        )
        .unwrap();
        let package = OpcPackage::from_reader(Cursor::new(workbook.to_xlsx_bytes().unwrap()))
            .expect("escaped workbook should reopen");
        let workbook_xml =
            std::str::from_utf8(package.get_part("/xl/workbook.xml").unwrap()).unwrap();
        assert!(workbook_xml.contains("name=\"Data_x005F_x000D_\""));
        let strings =
            std::str::from_utf8(package.get_part("/xl/sharedStrings.xml").unwrap()).unwrap();
        assert!(strings.contains("&lt;&amp;_x000D__x0001__xFFFE__x005F_x000D_"));
        assert!(strings.contains(
            "<t xml:space=\"preserve\"> value_x0009__x000A__x000B__xFFFF__x005F_x000A_ </t>"
        ));
        assert!(!strings.as_bytes().contains(&0x01));
        assert!(!strings.as_bytes().contains(&0x0b));
        let styles = std::str::from_utf8(package.get_part("/xl/styles.xml").unwrap()).unwrap();
        assert!(
            styles.contains(
                "formatCode=\"0_x0009__x000A_&lt;&amp;_x000D__x0002__xFFFE__x005F_x000D_\""
            )
        );
        assert!(!styles.as_bytes().contains(&0x02));
    }

    #[test]
    fn shared_string_count_rejects_the_first_value_past_u32() {
        assert_eq!(
            validate_shared_string_count(1, &[(u32::MAX - 1) as usize]).unwrap(),
            u32::MAX
        );
        assert!(matches!(
            validate_shared_string_count(1, &[u32::MAX as usize]),
            Err(Error::TooManySharedStrings(count)) if count == u32::MAX as u64 + 1
        ));
    }

    #[test]
    fn workbook_package_has_the_minimal_editable_part_graph() {
        let bytes = sample_workbook()
            .to_xlsx_bytes()
            .expect("workbook should serialize");
        let package = OpcPackage::from_reader(Cursor::new(bytes)).expect("package should reopen");
        assert_eq!(
            package.main_document_part().as_deref(),
            Some("/xl/workbook.xml")
        );
        for part in [
            "/xl/workbook.xml",
            "/xl/worksheets/sheet1.xml",
            "/xl/sharedStrings.xml",
            "/xl/styles.xml",
        ] {
            assert!(package.get_part(part).is_some(), "missing {part}");
            assert!(package.content_types.content_type_for(part).is_some());
        }
        let rels = package
            .get_part_rels("/xl/workbook.xml")
            .expect("workbook relationships");
        let worksheet_rel = rels
            .get_by_type(rel_types::WORKSHEET)
            .expect("worksheet relationship");
        let workbook = std::str::from_utf8(package.get_part("/xl/workbook.xml").unwrap()).unwrap();
        assert!(workbook.contains(&format!("r:id=\"{}\"", worksheet_rel.id)));
        for relationship in &rels.items {
            let target = OpcPackage::resolve_rel_target("/xl/workbook.xml", &relationship.target);
            assert!(
                package.get_part(&target).is_some(),
                "missing target {target}"
            );
        }

        let sheet =
            std::str::from_utf8(package.get_part("/xl/worksheets/sheet1.xml").unwrap()).unwrap();
        assert!(sheet.contains("r=\"B2\" s=\"1\"><v>12.5</v>"));
        let shared_strings = std::str::from_utf8(
            package
                .get_part("/xl/sharedStrings.xml")
                .expect("shared strings part"),
        )
        .unwrap();
        assert!(shared_strings.contains("North &amp; West"));

        let no_styles = Workbook::new("Data", vec![text_column()])
            .unwrap()
            .to_xlsx_bytes()
            .unwrap();
        let no_styles = OpcPackage::from_reader(Cursor::new(no_styles)).unwrap();
        assert!(no_styles.get_part("/xl/styles.xml").is_none());
        assert!(
            !no_styles
                .content_types
                .overrides
                .contains_key("/xl/styles.xml")
        );
    }

    #[test]
    fn equal_strings_share_one_stable_shared_string_index() {
        let workbook = sample_workbook();
        let first = workbook.to_xlsx_bytes().expect("first serialization");
        let second = workbook.to_xlsx_bytes().expect("second serialization");
        assert_eq!(first, second);
        let package = OpcPackage::from_reader(Cursor::new(first)).expect("package should reopen");
        let shared = std::str::from_utf8(
            package
                .get_part("/xl/sharedStrings.xml")
                .expect("shared strings part"),
        )
        .expect("shared strings should be UTF-8");
        assert!(shared.contains("count=\"4\""));
        assert!(shared.contains("uniqueCount=\"3\""));
        assert_eq!(shared.matches("North &amp; West").count(), 1);
    }

    #[test]
    #[ignore = "requires Excel and LibreOffice Calc"]
    fn generated_workbook_opens_cleanly_in_excel_and_libreoffice_calc() {
        let output = std::env::var_os("OXML_SML_VIEWER_GATE_OUTPUT")
            .map(std::path::PathBuf::from)
            .expect("set OXML_SML_VIEWER_GATE_OUTPUT to the SHA-bound .xlsx path");
        let bytes = sample_workbook()
            .to_xlsx_bytes()
            .expect("workbook should serialize");
        fs::write(&output, bytes).expect("write viewer gate workbook");
        assert_eq!(sha256(&output), VIEWER_ARTIFACT_SHA256);
        assert_excel_acceptance(&output);
        assert_libreoffice_acceptance(&output);
    }

    #[test]
    fn viewer_gate_candidate_is_bound_to_recorded_sha() {
        let output =
            std::env::temp_dir().join(format!("oxml-sml-f117-sha-{}.xlsx", std::process::id()));
        fs::write(&output, sample_workbook().to_xlsx_bytes().unwrap()).unwrap();
        assert_eq!(sha256(&output), VIEWER_ARTIFACT_SHA256);
        fs::remove_file(output).unwrap();
    }

    fn text_column() -> Column {
        Column::Text {
            header: "Name".into(),
            values: vec!["A".into()],
        }
    }

    fn sha256(path: &Path) -> String {
        let output = Command::new("shasum")
            .args(["-a", "256"])
            .arg(path)
            .output()
            .unwrap_or_else(|error| panic!("{}: run shasum: {error}", path.display()));
        assert!(
            output.status.success(),
            "shasum failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .to_string()
    }

    fn assert_excel_acceptance(path: &Path) {
        let application = "/Applications/Microsoft Excel.app/Contents/Info.plist";
        assert_eq!(
            plist_value(application, "CFBundleShortVersionString"),
            EXCEL_VERSION
        );
        assert_eq!(plist_value(application, "CFBundleVersion"), EXCEL_BUILD);

        let path = path.to_string_lossy();
        let script = format!(
            "with timeout of 120 seconds\ntell application \"Microsoft Excel\"\nactivate\nset gateWorkbook to open workbook workbook file name \"{path}\"\ntry\nif (count of worksheets of gateWorkbook) is not 1 then error \"worksheet count mismatch\"\nset gateSheet to worksheet 1 of gateWorkbook\nif (name of gateSheet) is not \"Sales '24\" then error \"worksheet name mismatch\"\nif ((value of range \"A1\" of gateSheet) as text) is not \"Category\" then error \"A1 mismatch\"\nif ((value of range \"B1\" of gateSheet) as text) is not \"Revenue\" then error \"B1 mismatch\"\nif ((value of range \"A2\" of gateSheet) as text) is not \"North & West\" then error \"A2 mismatch\"\nif ((value of range \"A3\" of gateSheet) as text) is not \"North & West\" then error \"A3 mismatch\"\nif ((value of range \"B2\" of gateSheet) as real) is not 12.5 then error \"B2 mismatch\"\nif ((value of range \"B3\" of gateSheet) as real) is not 20 then error \"B3 mismatch\"\nclose gateWorkbook saving no\non error errorMessage number errorNumber\ntry\nclose gateWorkbook saving no\nend try\nerror errorMessage number errorNumber\nend try\nend tell\nend timeout\n"
        );
        let result = Command::new("osascript")
            .args(["-e", &script])
            .output()
            .expect("launch Excel acceptance script");
        assert!(
            result.status.success(),
            "Excel F-117 acceptance failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }

    fn assert_libreoffice_acceptance(path: &Path) {
        let executable = std::env::var_os("OXML_SML_LIBREOFFICE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("soffice"));
        let version = Command::new(&executable)
            .arg("--version")
            .output()
            .expect("read LibreOffice version");
        assert!(version.status.success());
        assert_eq!(
            String::from_utf8(version.stdout).unwrap().trim(),
            LIBREOFFICE_VERSION
        );

        let output_dir =
            std::env::temp_dir().join(format!("oxml-sml-f117-libreoffice-{}", std::process::id()));
        let profile_dir = std::env::temp_dir().join(format!(
            "oxml-sml-f117-libreoffice-profile-{}",
            std::process::id()
        ));
        if output_dir.exists() {
            fs::remove_dir_all(&output_dir).expect("remove stale LibreOffice output");
        }
        if profile_dir.exists() {
            fs::remove_dir_all(&profile_dir).expect("remove stale LibreOffice profile");
        }
        fs::create_dir_all(&output_dir).expect("create LibreOffice output");
        let profile = format!("-env:UserInstallation=file://{}", profile_dir.display());
        let result = Command::new(&executable)
            .args(["--headless", &profile, "--convert-to", "xlsx", "--outdir"])
            .arg(&output_dir)
            .arg(path)
            .output()
            .expect("run LibreOffice Calc acceptance");
        assert!(
            result.status.success(),
            "LibreOffice Calc F-117 import or export failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let reopened_path = output_dir.join(path.file_name().unwrap());
        assert!(reopened_path.is_file());
        let reopened = OpcPackage::open(&reopened_path).expect("reopen LibreOffice workbook");
        let sheet = std::str::from_utf8(
            reopened
                .get_part("/xl/worksheets/sheet1.xml")
                .expect("LibreOffice worksheet"),
        )
        .unwrap();
        assert!(sheet.contains("dimension ref=\"A1:B3\""));
        assert!(sheet.contains("r=\"B2\" s=\"1\" t=\"n\"><v>12.5</v>"));
        assert!(sheet.contains("r=\"B3\" s=\"1\" t=\"n\"><v>20</v>"));
        let strings = std::str::from_utf8(
            reopened
                .get_part("/xl/sharedStrings.xml")
                .expect("LibreOffice shared strings"),
        )
        .unwrap();
        for expected in ["Category", "Revenue", "North &amp; West"] {
            assert!(strings.contains(expected), "missing {expected}");
        }
        fs::remove_dir_all(&output_dir).expect("remove LibreOffice output");
        if profile_dir.exists() {
            fs::remove_dir_all(&profile_dir).expect("remove LibreOffice profile");
        }
    }

    fn plist_value(application: &str, key: &str) -> String {
        let output = Command::new("defaults")
            .args(["read", application, key])
            .output()
            .expect("read Excel Info.plist");
        assert!(output.status.success());
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    }
}
