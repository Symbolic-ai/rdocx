//! Bounded OpenDocument Text import into the native Word document model.

use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read};
use std::path::Path;

use quick_xml::XmlVersion;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::ResolveResult;
use quick_xml::reader::NsReader;
use rdocx_oxml::document::BodyContent;
use rdocx_oxml::table::{
    CT_Row, CT_Tbl, CT_TblGrid, CT_TblGridCol, CT_TblPr, CT_TblWidth, CT_Tc, CT_TcPr, CellContent,
    VMerge,
};
use rdocx_oxml::text::CT_P;
use rdocx_oxml::units::Twips;
use zip::CompressionMethod;
use zip::read::ZipArchive;

use crate::paragraph::{Alignment, Paragraph};
use crate::run::Run;
use crate::{Document, Error, Length, ListLevel, PackageReadLimits, Result};

const OFFICE_NS: &str = "urn:oasis:names:tc:opendocument:xmlns:office:1.0";
const TEXT_NS: &str = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";
const STYLE_NS: &str = "urn:oasis:names:tc:opendocument:xmlns:style:1.0";
const FO_NS: &str = "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0";
const TABLE_NS: &str = "urn:oasis:names:tc:opendocument:xmlns:table:1.0";
const DRAW_NS: &str = "urn:oasis:names:tc:opendocument:xmlns:drawing:1.0";
const XLINK_NS: &str = "http://www.w3.org/1999/xlink";
const SVG_NS: &str = "urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0";
const MANIFEST_NS: &str = "urn:oasis:names:tc:opendocument:xmlns:manifest:1.0";
const ODT_MIMETYPE: &[u8] = b"application/vnd.oasis.opendocument.text";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OdtDiagnostic {
    pub path: String,
    pub message: String,
}

pub struct OdtReadResult {
    pub document: Document,
    pub diagnostics: Vec<OdtDiagnostic>,
}

#[derive(Clone, Copy)]
struct OdtLimits {
    archive: PackageReadLimits,
    xml_depth: usize,
    xml_nodes: usize,
    retained_text: usize,
    blocks: usize,
    runs: usize,
    rows: usize,
    columns: usize,
    cells: usize,
    diagnostics: usize,
}

impl OdtLimits {
    const DEFAULT: Self = Self {
        archive: PackageReadLimits {
            max_entries: 4_096,
            max_part_uncompressed_bytes: 64 * 1024 * 1024,
            max_total_uncompressed_bytes: 128 * 1024 * 1024,
        },
        xml_depth: 256,
        xml_nodes: 300_000,
        retained_text: 64 * 1024 * 1024,
        blocks: 100_000,
        runs: 100_000,
        rows: 10_000,
        columns: 256,
        cells: 50_000,
        diagnostics: 10_000,
    };
}

impl Document {
    /// Convert an ODT package into a fresh editable Word document.
    pub fn from_odt_bytes(bytes: &[u8]) -> Result<OdtReadResult> {
        from_odt_with_limits(bytes, OdtLimits::DEFAULT)
    }

    /// Convert an ODT package while applying caller-supplied archive bounds.
    pub fn from_odt_bytes_with_limits(
        bytes: &[u8],
        limits: PackageReadLimits,
    ) -> Result<OdtReadResult> {
        from_odt_with_limits(
            bytes,
            OdtLimits {
                archive: limits,
                ..OdtLimits::DEFAULT
            },
        )
    }

    /// Open and convert an ODT package from a path.
    pub fn open_odt<P: AsRef<Path>>(path: P) -> Result<OdtReadResult> {
        let file = std::fs::File::open(path)?;
        let declared = file.metadata()?.len();
        if declared > OdtLimits::DEFAULT.archive.max_total_uncompressed_bytes {
            return Err(odt_error(None, 0, "ODT input exceeds the size limit"));
        }
        let mut bytes = Vec::with_capacity(usize::try_from(declared).unwrap_or(0));
        file.take(
            OdtLimits::DEFAULT
                .archive
                .max_total_uncompressed_bytes
                .saturating_add(1),
        )
        .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > OdtLimits::DEFAULT.archive.max_total_uncompressed_bytes {
            return Err(odt_error(None, 0, "ODT input exceeds the size limit"));
        }
        Self::from_odt_bytes(&bytes)
    }
}

fn from_odt_with_limits(bytes: &[u8], limits: OdtLimits) -> Result<OdtReadResult> {
    let mut archive = OdtArchive::open(bytes, limits.archive)?;
    let mimetype = archive.read_required("mimetype")?;
    if mimetype.as_slice() != ODT_MIMETYPE {
        return Err(odt_error(
            Some("mimetype"),
            0,
            "ODT mimetype is missing or invalid",
        ));
    }
    let content_bytes = archive.read_required("content.xml")?;
    let styles_bytes = archive.read_optional("styles.xml")?;
    let manifest_bytes = archive.read_optional("META-INF/manifest.xml")?;

    let manifest = manifest_bytes
        .as_deref()
        .map(|xml| parse_xml("META-INF/manifest.xml", xml, limits))
        .transpose()?;
    let encrypted = encrypted_manifest_paths(manifest.as_ref())?;
    for required_part in ["/", "content.xml"] {
        if encrypted.contains(required_part) {
            return Err(odt_error(
                Some(required_part),
                0,
                "encrypted required ODT content is unsupported",
            ));
        }
    }
    if styles_bytes.is_some() && encrypted.contains("styles.xml") {
        return Err(odt_error(
            Some("styles.xml"),
            0,
            "encrypted ODT styles are unsupported",
        ));
    }

    let content = parse_xml("content.xml", &content_bytes, limits)?;
    let styles = styles_bytes
        .as_deref()
        .map(|xml| parse_xml("styles.xml", xml, limits))
        .transpose()?;

    Importer::new(archive, content, styles, encrypted, limits).project()
}

struct OdtArchive<'a> {
    archive: ZipArchive<Cursor<&'a [u8]>>,
    names: HashSet<String>,
    limit: u64,
}

impl<'a> OdtArchive<'a> {
    fn open(bytes: &'a [u8], limits: PackageReadLimits) -> Result<Self> {
        let raw_names = central_directory_names(bytes)?;
        if raw_names.len() > limits.max_entries {
            return Err(odt_error(None, 0, "ODT archive exceeds the entry limit"));
        }
        let mut names = HashSet::with_capacity(raw_names.len());
        for name in raw_names {
            validate_entry_name(&name)?;
            if !names.insert(name.clone()) {
                return Err(odt_error(Some(&name), 0, "duplicate ODT archive entry"));
            }
        }
        let mut archive = ZipArchive::new(Cursor::new(bytes))
            .map_err(|error| odt_error(None, 0, format!("invalid ODT ZIP: {error}")))?;
        if archive.len() > limits.max_entries {
            return Err(odt_error(None, 0, "ODT archive exceeds the entry limit"));
        }
        let mut total = 0_u64;
        for index in 0..archive.len() {
            let entry = archive.by_index(index).map_err(|error| {
                odt_error(None, 0, format!("cannot index ODT archive: {error}"))
            })?;
            let name = entry.name();
            validate_entry_name(name)?;
            if !entry.is_file() {
                return Err(odt_error(Some(name), 0, "ODT entry is not a regular file"));
            }
            if entry.encrypted() {
                return Err(odt_error(
                    Some(name),
                    0,
                    "encrypted ZIP entries are unsupported",
                ));
            }
            if !matches!(
                entry.compression(),
                CompressionMethod::Stored | CompressionMethod::Deflated
            ) {
                return Err(odt_error(
                    Some(name),
                    0,
                    "ODT entry uses unsupported compression",
                ));
            }
            if entry.size() > limits.max_part_uncompressed_bytes {
                return Err(odt_error(Some(name), 0, "ODT part exceeds the size limit"));
            }
            total = total
                .checked_add(entry.size())
                .filter(|total| *total <= limits.max_total_uncompressed_bytes)
                .ok_or_else(|| odt_error(None, 0, "ODT archive exceeds the expansion limit"))?;
        }
        if !names.contains("mimetype") || !names.contains("content.xml") {
            return Err(odt_error(None, 0, "ODT requires mimetype and content.xml"));
        }
        Ok(Self {
            archive,
            names,
            limit: limits.max_part_uncompressed_bytes,
        })
    }

    fn read_required(&mut self, name: &str) -> Result<Vec<u8>> {
        self.read_optional(name)?.ok_or_else(|| {
            odt_error(
                Some(name),
                0,
                format!("required ODT part {name} is missing"),
            )
        })
    }

    fn read_optional(&mut self, name: &str) -> Result<Option<Vec<u8>>> {
        if !self.names.contains(name) {
            return Ok(None);
        }
        let mut entry = self
            .archive
            .by_name(name)
            .map_err(|error| odt_error(Some(name), 0, format!("cannot read ODT part: {error}")))?;
        let mut bytes = Vec::with_capacity(usize::try_from(entry.size()).unwrap_or(0));
        entry
            .by_ref()
            .take(self.limit.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|error| {
                odt_error(Some(name), 0, format!("cannot expand ODT part: {error}"))
            })?;
        if bytes.len() as u64 > self.limit {
            return Err(odt_error(Some(name), 0, "ODT part exceeds the size limit"));
        }
        Ok(Some(bytes))
    }
}

fn central_directory_names(bytes: &[u8]) -> Result<Vec<String>> {
    const EOCD: &[u8; 4] = b"PK\x05\x06";
    const CENTRAL: &[u8; 4] = b"PK\x01\x02";
    let search_start = bytes.len().saturating_sub(65_557);
    let eocd = (search_start..bytes.len().saturating_sub(3))
        .rev()
        .find(|offset| &bytes[*offset..*offset + 4] == EOCD)
        .ok_or_else(|| odt_error(None, 0, "ODT ZIP end record is missing"))?;
    if eocd + 22 > bytes.len() {
        return Err(odt_error(None, eocd as u64, "truncated ODT ZIP end record"));
    }
    let disk = read_u16(bytes, eocd + 4)?;
    let central_disk = read_u16(bytes, eocd + 6)?;
    let disk_entries = read_u16(bytes, eocd + 8)?;
    let entries = read_u16(bytes, eocd + 10)?;
    let central_size = read_u32(bytes, eocd + 12)?;
    let central_offset = read_u32(bytes, eocd + 16)?;
    let comment = read_u16(bytes, eocd + 20)? as usize;
    if disk != 0 || central_disk != 0 || disk_entries != entries {
        return Err(odt_error(
            None,
            eocd as u64,
            "multi-disk ODT ZIP is unsupported",
        ));
    }
    if entries == u16::MAX || central_size == u32::MAX || central_offset == u32::MAX {
        return Err(odt_error(
            None,
            eocd as u64,
            "ZIP64 ODT archives are unsupported",
        ));
    }
    if eocd + 22 + comment != bytes.len() {
        return Err(odt_error(
            None,
            eocd as u64,
            "invalid ODT ZIP comment length",
        ));
    }
    let mut position = central_offset as usize;
    let central_end = position
        .checked_add(central_size as usize)
        .filter(|end| *end <= eocd)
        .ok_or_else(|| {
            odt_error(
                None,
                position as u64,
                "invalid ODT central directory bounds",
            )
        })?;
    let mut names = Vec::with_capacity(entries as usize);
    for _ in 0..entries {
        if position + 46 > central_end || &bytes[position..position + 4] != CENTRAL {
            return Err(odt_error(
                None,
                position as u64,
                "invalid ODT central directory entry",
            ));
        }
        let name_len = read_u16(bytes, position + 28)? as usize;
        let extra_len = read_u16(bytes, position + 30)? as usize;
        let comment_len = read_u16(bytes, position + 32)? as usize;
        let name_start = position + 46;
        let next = name_start
            .checked_add(name_len)
            .and_then(|value| value.checked_add(extra_len))
            .and_then(|value| value.checked_add(comment_len))
            .filter(|next| *next <= central_end)
            .ok_or_else(|| odt_error(None, position as u64, "truncated ODT central entry"))?;
        let name = std::str::from_utf8(&bytes[name_start..name_start + name_len])
            .map_err(|_| odt_error(None, position as u64, "ODT entry name is not UTF-8"))?;
        names.push(name.to_string());
        position = next;
    }
    if position != central_end {
        return Err(odt_error(
            None,
            position as u64,
            "unexpected ODT central directory bytes",
        ));
    }
    Ok(names)
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let value = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| odt_error(None, offset as u64, "truncated ODT ZIP field"))?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| odt_error(None, offset as u64, "truncated ODT ZIP field"))?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

fn validate_entry_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.starts_with('/')
        || name.starts_with('\\')
        || name.contains('\\')
        || name.contains('\0')
        || name
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err(odt_error(Some(name), 0, "unsafe ODT archive entry name"));
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct XmlName {
    namespace: Option<String>,
    local: String,
}

#[derive(Clone, Debug)]
struct XmlAttribute {
    name: XmlName,
    value: String,
}

#[derive(Clone, Debug)]
enum XmlChild {
    Element(XmlNode),
    Text(String),
}

#[derive(Clone, Debug)]
struct XmlNode {
    name: XmlName,
    attributes: Vec<XmlAttribute>,
    children: Vec<XmlChild>,
}

impl XmlNode {
    fn is(&self, namespace: &str, local: &str) -> bool {
        self.name.namespace.as_deref() == Some(namespace) && self.name.local == local
    }

    fn attr(&self, namespace: Option<&str>, local: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|attribute| {
                attribute.name.namespace.as_deref() == namespace && attribute.name.local == local
            })
            .map(|attribute| attribute.value.as_str())
    }

    fn elements(&self) -> impl Iterator<Item = &XmlNode> {
        self.children.iter().filter_map(|child| match child {
            XmlChild::Element(element) => Some(element),
            XmlChild::Text(_) => None,
        })
    }
}

fn parse_xml(part: &str, xml: &[u8], limits: OdtLimits) -> Result<XmlNode> {
    let mut reader = NsReader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut buffer = Vec::new();
    let mut stack: Vec<XmlNode> = Vec::new();
    let mut root = None;
    let mut nodes = 0_usize;
    let mut text_bytes = 0_usize;
    loop {
        let offset = reader.buffer_position();
        let (namespace, event) = reader
            .read_resolved_event_into(&mut buffer)
            .map_err(|error| odt_error(Some(part), offset, format!("malformed XML: {error}")))?;
        let namespace = namespace_value(namespace, part, offset)?;
        match event {
            Event::Start(element) => {
                nodes = nodes.checked_add(1).ok_or_else(|| {
                    odt_error(Some(part), offset, "ODT XML node count overflowed")
                })?;
                if nodes > limits.xml_nodes {
                    return Err(odt_error(
                        Some(part),
                        offset,
                        "ODT XML exceeds the node limit",
                    ));
                }
                if stack.len() >= limits.xml_depth {
                    return Err(odt_error(
                        Some(part),
                        offset,
                        "ODT XML exceeds the depth limit",
                    ));
                }
                stack.push(xml_node(&reader, namespace, &element, part, offset)?);
            }
            Event::Empty(element) => {
                nodes = nodes.checked_add(1).ok_or_else(|| {
                    odt_error(Some(part), offset, "ODT XML node count overflowed")
                })?;
                if nodes > limits.xml_nodes {
                    return Err(odt_error(
                        Some(part),
                        offset,
                        "ODT XML exceeds the node limit",
                    ));
                }
                if stack.len() >= limits.xml_depth {
                    return Err(odt_error(
                        Some(part),
                        offset,
                        "ODT XML exceeds the depth limit",
                    ));
                }
                attach_node(
                    &mut stack,
                    &mut root,
                    xml_node(&reader, namespace, &element, part, offset)?,
                    part,
                    offset,
                )?;
            }
            Event::End(_) => {
                let node = stack.pop().ok_or_else(|| {
                    odt_error(Some(part), offset, "ODT XML end tag has no open element")
                })?;
                attach_node(&mut stack, &mut root, node, part, offset)?;
            }
            Event::Text(text) => {
                let decoded = text.xml10_content().map_err(|error| {
                    odt_error(Some(part), offset, format!("invalid XML text: {error}"))
                })?;
                let value = quick_xml::escape::unescape(&decoded)
                    .map_err(|error| {
                        odt_error(
                            Some(part),
                            offset,
                            format!("unresolved XML entity: {error}"),
                        )
                    })?
                    .into_owned();
                append_text(&mut stack, value, &mut text_bytes, limits, part, offset)?;
            }
            Event::CData(text) => {
                let value = text
                    .xml10_content()
                    .map_err(|error| {
                        odt_error(Some(part), offset, format!("invalid XML CDATA: {error}"))
                    })?
                    .into_owned();
                append_text(&mut stack, value, &mut text_bytes, limits, part, offset)?;
            }
            Event::GeneralRef(reference) => {
                let value = if let Some(character) =
                    reference.resolve_char_ref().map_err(|error| {
                        odt_error(
                            Some(part),
                            offset,
                            format!("invalid XML reference: {error}"),
                        )
                    })? {
                    character.to_string()
                } else {
                    let reference_bytes: &[u8] = &reference;
                    match reference_bytes {
                        b"amp" => "&".to_string(),
                        b"lt" => "<".to_string(),
                        b"gt" => ">".to_string(),
                        b"apos" => "'".to_string(),
                        b"quot" => "\"".to_string(),
                        _ => {
                            return Err(odt_error(
                                Some(part),
                                offset,
                                "unresolved XML entity is unsupported",
                            ));
                        }
                    }
                };
                append_text(&mut stack, value, &mut text_bytes, limits, part, offset)?;
            }
            Event::DocType(_) => {
                return Err(odt_error(
                    Some(part),
                    offset,
                    "DTD declarations are forbidden",
                ));
            }
            Event::Eof => break,
            Event::Decl(_) | Event::Comment(_) | Event::PI(_) => {}
        }
        buffer.clear();
    }
    if !stack.is_empty() {
        return Err(odt_error(
            Some(part),
            xml.len() as u64,
            "unclosed XML element",
        ));
    }
    root.ok_or_else(|| odt_error(Some(part), 0, "ODT XML has no root element"))
}

fn namespace_value(
    namespace: ResolveResult<'_>,
    part: &str,
    offset: u64,
) -> Result<Option<String>> {
    match namespace {
        ResolveResult::Bound(namespace) => Ok(Some(
            std::str::from_utf8(namespace.as_ref())
                .map_err(|_| odt_error(Some(part), offset, "namespace URI is not UTF-8"))?
                .to_string(),
        )),
        ResolveResult::Unbound => Ok(None),
        ResolveResult::Unknown(_) => Err(odt_error(
            Some(part),
            offset,
            "ODT XML uses an unresolved namespace prefix",
        )),
    }
}

fn xml_node(
    reader: &NsReader<&[u8]>,
    namespace: Option<String>,
    element: &BytesStart<'_>,
    part: &str,
    offset: u64,
) -> Result<XmlNode> {
    let local = std::str::from_utf8(element.local_name().as_ref())
        .map_err(|_| odt_error(Some(part), offset, "element local name is not UTF-8"))?
        .to_string();
    let mut attributes = Vec::new();
    let mut seen = HashSet::new();
    for attribute in element.attributes() {
        let attribute = attribute.map_err(|error| {
            odt_error(
                Some(part),
                offset,
                format!("malformed XML attribute: {error}"),
            )
        })?;
        let (namespace, local) = reader.resolver().resolve_attribute(attribute.key);
        let namespace = namespace_value(namespace, part, offset)?;
        let local = std::str::from_utf8(local.as_ref())
            .map_err(|_| odt_error(Some(part), offset, "attribute local name is not UTF-8"))?
            .to_string();
        if !seen.insert((namespace.clone(), local.clone())) {
            return Err(odt_error(
                Some(part),
                offset,
                "duplicate expanded XML attribute",
            ));
        }
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
            .map_err(|error| {
                odt_error(
                    Some(part),
                    offset,
                    format!("invalid XML attribute value: {error}"),
                )
            })?
            .into_owned();
        attributes.push(XmlAttribute {
            name: XmlName { namespace, local },
            value,
        });
    }
    Ok(XmlNode {
        name: XmlName { namespace, local },
        attributes,
        children: Vec::new(),
    })
}

fn attach_node(
    stack: &mut [XmlNode],
    root: &mut Option<XmlNode>,
    node: XmlNode,
    part: &str,
    offset: u64,
) -> Result<()> {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(XmlChild::Element(node));
    } else if root.replace(node).is_some() {
        return Err(odt_error(
            Some(part),
            offset,
            "ODT XML has multiple root elements",
        ));
    }
    Ok(())
}

fn append_text(
    stack: &mut [XmlNode],
    value: String,
    retained: &mut usize,
    limits: OdtLimits,
    part: &str,
    offset: u64,
) -> Result<()> {
    if value.is_empty() {
        return Ok(());
    }
    if stack.is_empty() && value.chars().all(char::is_whitespace) {
        return Ok(());
    }
    *retained = retained
        .checked_add(value.len())
        .filter(|retained| *retained <= limits.retained_text)
        .ok_or_else(|| {
            odt_error(
                Some(part),
                offset,
                "ODT XML exceeds the retained text limit",
            )
        })?;
    let parent = stack
        .last_mut()
        .ok_or_else(|| odt_error(Some(part), offset, "text appears outside the XML root"))?;
    parent.children.push(XmlChild::Text(value));
    Ok(())
}

fn encrypted_manifest_paths(manifest: Option<&XmlNode>) -> Result<HashSet<String>> {
    let mut encrypted = HashSet::new();
    let Some(manifest) = manifest else {
        return Ok(encrypted);
    };
    if !manifest.is(MANIFEST_NS, "manifest") {
        return Err(odt_error(
            Some("META-INF/manifest.xml"),
            0,
            "invalid ODT manifest root",
        ));
    }
    for entry in manifest
        .elements()
        .filter(|entry| entry.is(MANIFEST_NS, "file-entry"))
    {
        let path = entry
            .attr(Some(MANIFEST_NS), "full-path")
            .or_else(|| entry.attr(None, "full-path"));
        if entry
            .elements()
            .any(|child| child.is(MANIFEST_NS, "encryption-data"))
        {
            let path = path.ok_or_else(|| {
                odt_error(
                    Some("META-INF/manifest.xml"),
                    0,
                    "encrypted manifest entry has no full path",
                )
            })?;
            encrypted.insert(path.to_string());
        }
    }
    Ok(encrypted)
}

#[derive(Clone, Debug, Default, PartialEq)]
struct EffectiveStyle {
    font: Option<String>,
    size: Option<f64>,
    bold: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
    strike: Option<bool>,
    color: Option<String>,
    background: Option<String>,
    vertical: Option<VerticalText>,
    alignment: Option<Alignment>,
    before: Option<Length>,
    after: Option<Length>,
    left: Option<Length>,
    right: Option<Length>,
    first: Option<Length>,
    line: Option<LineHeight>,
}

impl EffectiveStyle {
    fn overlay(&mut self, other: &Self) {
        macro_rules! replace {
            ($field:ident) => {
                if other.$field.is_some() {
                    self.$field = other.$field.clone();
                }
            };
        }
        replace!(font);
        replace!(size);
        replace!(bold);
        replace!(italic);
        replace!(underline);
        replace!(strike);
        replace!(color);
        replace!(background);
        replace!(vertical);
        replace!(alignment);
        replace!(before);
        replace!(after);
        replace!(left);
        replace!(right);
        replace!(first);
        replace!(line);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum VerticalText {
    Superscript,
    Subscript,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum LineHeight {
    Exact(f64),
    Multiple(f64),
}

#[derive(Clone, Debug)]
struct StyleDef {
    family: String,
    name: String,
    parent: Option<String>,
    values: EffectiveStyle,
}

struct Importer<'a> {
    archive: OdtArchive<'a>,
    content: XmlNode,
    styles_root: Option<XmlNode>,
    encrypted: HashSet<String>,
    styles: HashMap<(String, String), StyleDef>,
    defaults: HashMap<String, EffectiveStyle>,
    resolved: HashMap<(String, String), EffectiveStyle>,
    list_kinds: HashMap<String, Vec<bool>>,
    fonts: HashMap<String, String>,
    document: Document,
    diagnostics: Vec<OdtDiagnostic>,
    diagnostic_keys: HashSet<(String, String)>,
    limits: OdtLimits,
    blocks: usize,
    runs: usize,
    rows: usize,
    cells: usize,
    projected_text: usize,
}

impl<'a> Importer<'a> {
    fn new(
        archive: OdtArchive<'a>,
        content: XmlNode,
        styles_root: Option<XmlNode>,
        encrypted: HashSet<String>,
        limits: OdtLimits,
    ) -> Self {
        Self {
            archive,
            content,
            styles_root,
            encrypted,
            styles: HashMap::new(),
            defaults: HashMap::new(),
            resolved: HashMap::new(),
            list_kinds: HashMap::new(),
            fonts: HashMap::new(),
            document: Document::new(),
            diagnostics: Vec::new(),
            diagnostic_keys: HashSet::new(),
            limits,
            blocks: 0,
            runs: 0,
            rows: 0,
            cells: 0,
            projected_text: 0,
        }
    }

    fn project(mut self) -> Result<OdtReadResult> {
        if !self.content.is(OFFICE_NS, "document-content") {
            return Err(odt_error(
                Some("content.xml"),
                0,
                "invalid ODT content root",
            ));
        }
        if let Some(styles) = self.styles_root.clone() {
            if !styles.is(OFFICE_NS, "document-styles") {
                return Err(odt_error(Some("styles.xml"), 0, "invalid ODT styles root"));
            }
            self.collect_style_document(&styles, "styles.xml")?;
        }
        let content = self.content.clone();
        self.collect_style_document(&content, "content.xml")?;
        self.resolve_all_styles()?;

        let body = content
            .elements()
            .find(|node| node.is(OFFICE_NS, "body"))
            .and_then(|node| node.elements().find(|node| node.is(OFFICE_NS, "text")))
            .ok_or_else(|| odt_error(Some("content.xml"), 0, "ODT text body is missing"))?;
        self.project_container(body, "office:body/office:text", None)?;

        let bytes = self.document.to_bytes()?;
        let document = Document::from_bytes(&bytes)?;
        Ok(OdtReadResult {
            document,
            diagnostics: self.diagnostics,
        })
    }

    fn collect_style_document(&mut self, root: &XmlNode, part: &str) -> Result<()> {
        self.collect_fonts(root);
        self.walk_style_containers(root, part)
    }

    fn collect_fonts(&mut self, node: &XmlNode) {
        if node.is(STYLE_NS, "font-face")
            && let Some(name) = node.attr(Some(STYLE_NS), "name")
            && let Some(family) = node.attr(Some(SVG_NS), "font-family")
        {
            self.fonts.insert(
                name.to_string(),
                family.trim_matches(['\'', '"']).to_string(),
            );
        }
        for child in node.elements() {
            self.collect_fonts(child);
        }
    }

    fn walk_style_containers(&mut self, node: &XmlNode, part: &str) -> Result<()> {
        if node.is(STYLE_NS, "default-style") {
            let family = required_attr(node, STYLE_NS, "family", part)?;
            let values =
                self.parse_style_values(node, part, &format!("style:default-style[{family}]"))?;
            self.defaults
                .entry(family.to_string())
                .or_default()
                .overlay(&values);
        } else if node.is(STYLE_NS, "style") {
            let family = required_attr(node, STYLE_NS, "family", part)?;
            let name = required_attr(node, STYLE_NS, "name", part)?;
            let values = self.parse_style_values(node, part, &format!("style:style[{name}]"))?;
            let def = StyleDef {
                family: family.to_string(),
                name: name.to_string(),
                parent: node
                    .attr(Some(STYLE_NS), "parent-style-name")
                    .map(str::to_string),
                values,
            };
            let key = (def.family.clone(), def.name.clone());
            if self.styles.insert(key, def).is_some() {
                return Err(odt_error(Some(part), 0, "duplicate ODT style definition"));
            }
        } else if node.is(TEXT_NS, "list-style") {
            let name = required_attr(node, STYLE_NS, "name", part)?.to_string();
            let mut levels = vec![false; 9];
            for level in node.elements() {
                let index = level
                    .attr(Some(TEXT_NS), "level")
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(1)
                    .saturating_sub(1)
                    .min(8);
                if level.is(TEXT_NS, "list-level-style-bullet") {
                    levels[index] = true;
                } else if level.is(TEXT_NS, "list-level-style-number") {
                    levels[index] = false;
                }
            }
            self.list_kinds.insert(name, levels);
        }
        for child in node.elements() {
            self.walk_style_containers(child, part)?;
        }
        Ok(())
    }

    fn parse_style_values(
        &mut self,
        style: &XmlNode,
        part: &str,
        path: &str,
    ) -> Result<EffectiveStyle> {
        let mut values = EffectiveStyle::default();
        for properties in style.elements() {
            if properties.is(STYLE_NS, "text-properties") {
                for attribute in &properties.attributes {
                    let result = parse_text_property(attribute, &self.fonts);
                    match result {
                        Some(Ok(change)) => change.apply(&mut values),
                        Some(Err(message)) => self.diagnostic(path, message)?,
                        None if !is_ignorable_style_attribute(attribute) => self.diagnostic(
                            path,
                            format!("unsupported text property {}", attribute.name.local),
                        )?,
                        None => {}
                    }
                }
            } else if properties.is(STYLE_NS, "paragraph-properties") {
                for attribute in &properties.attributes {
                    let result = parse_paragraph_property(attribute);
                    match result {
                        Some(Ok(change)) => change.apply(&mut values),
                        Some(Err(message)) => self.diagnostic(path, message)?,
                        None if !is_ignorable_style_attribute(attribute) => self.diagnostic(
                            path,
                            format!("unsupported paragraph property {}", attribute.name.local),
                        )?,
                        None => {}
                    }
                }
            }
        }
        let _ = part;
        Ok(values)
    }

    fn resolve_all_styles(&mut self) -> Result<()> {
        let keys: Vec<(String, String)> = self.styles.keys().cloned().collect();
        for key in keys {
            self.resolve_style(&key.0, &key.1, &mut Vec::new())?;
        }
        Ok(())
    }

    fn resolve_style(
        &mut self,
        family: &str,
        name: &str,
        visiting: &mut Vec<(String, String)>,
    ) -> Result<EffectiveStyle> {
        let key = (family.to_string(), name.to_string());
        if let Some(value) = self.resolved.get(&key) {
            return Ok(value.clone());
        }
        if visiting.contains(&key) {
            return Err(odt_error(
                Some("styles.xml"),
                0,
                format!("ODT style inheritance cycle at {family}/{name}"),
            ));
        }
        let Some(definition) = self.styles.get(&key).cloned() else {
            self.diagnostic(
                &format!("style:{family}/{name}"),
                format!("missing ODT style {name}"),
            )?;
            return Ok(self.defaults.get(family).cloned().unwrap_or_default());
        };
        visiting.push(key.clone());
        let mut value = if let Some(parent) = definition.parent.as_deref() {
            self.resolve_style(family, parent, visiting)?
        } else {
            self.defaults.get(family).cloned().unwrap_or_default()
        };
        visiting.pop();
        value.overlay(&definition.values);
        self.resolved.insert(key, value.clone());
        Ok(value)
    }

    fn effective_style(&mut self, family: &str, name: Option<&str>) -> Result<EffectiveStyle> {
        match name {
            Some(name) => self.resolve_style(family, name, &mut Vec::new()),
            None => Ok(self.defaults.get(family).cloned().unwrap_or_default()),
        }
    }

    fn project_container(
        &mut self,
        container: &XmlNode,
        path: &str,
        list: Option<(u32, usize)>,
    ) -> Result<()> {
        let mut positions: HashMap<(Option<String>, String), usize> = HashMap::new();
        for child in container.elements() {
            let key = (child.name.namespace.clone(), child.name.local.clone());
            let index = positions.entry(key).or_default();
            *index += 1;
            let child_path = format!("{path}/{}[{}]", display_name(child), *index);
            if child.is(TEXT_NS, "p") || child.is(TEXT_NS, "h") {
                self.project_paragraph(child, &child_path, list)?;
            } else if child.is(TEXT_NS, "list") {
                self.project_list(child, &child_path, list.map_or(0, |(_, level)| level + 1))?;
            } else if child.is(TABLE_NS, "table") {
                self.project_table(child, &child_path)?;
            } else if child.is(TEXT_NS, "section") || child.is(OFFICE_NS, "text") {
                self.project_container(child, &child_path, list)?;
            } else if child.is(DRAW_NS, "frame") {
                let paragraph = XmlNode {
                    name: XmlName {
                        namespace: Some(TEXT_NS.to_string()),
                        local: "p".to_string(),
                    },
                    attributes: Vec::new(),
                    children: vec![XmlChild::Element(child.clone())],
                };
                self.project_paragraph(&paragraph, &child_path, list)?;
            } else {
                self.diagnostic(
                    &child_path,
                    format!("unsupported ODT subtree {}", display_name(child)),
                )?;
            }
        }
        Ok(())
    }

    fn project_list(&mut self, list: &XmlNode, path: &str, level: usize) -> Result<()> {
        if level >= 9 {
            return Err(odt_error(
                Some("content.xml"),
                0,
                "ODT list exceeds nine levels",
            ));
        }
        if list.attr(Some(TEXT_NS), "continue-numbering").is_some()
            || list.attr(Some(TEXT_NS), "continue-list").is_some()
        {
            self.diagnostic(path, "unsupported ODT list continuation semantics")?;
        }
        let style_name = list.attr(Some(TEXT_NS), "style-name");
        let levels = style_name
            .and_then(|name| self.list_kinds.get(name))
            .cloned()
            .unwrap_or_else(|| vec![true; 9]);
        if style_name.is_some_and(|name| !self.list_kinds.contains_key(name)) {
            self.diagnostic(path, "missing ODT list style, using bullets")?;
        }
        let definitions: Vec<ListLevel> = levels
            .iter()
            .map(|bullet| {
                if *bullet {
                    ListLevel::bullet()
                } else {
                    ListLevel::decimal()
                }
            })
            .collect();
        let num_id = self.document.add_list_definition(&definitions);
        for (item_index, item) in list
            .elements()
            .filter(|item| item.is(TEXT_NS, "list-item") || item.is(TEXT_NS, "list-header"))
            .enumerate()
        {
            let item_name = if item.is(TEXT_NS, "list-header") {
                "list-header"
            } else {
                "list-item"
            };
            let item_path = format!("{path}/text:{item_name}[{}]", item_index + 1);
            if item.attr(Some(TEXT_NS), "start-value").is_some() {
                self.diagnostic(&item_path, "unsupported ODT list start value")?;
            }
            let numbering = item.is(TEXT_NS, "list-item").then_some((num_id, level));
            self.project_container(item, &item_path, numbering)?;
        }
        Ok(())
    }

    fn project_paragraph(
        &mut self,
        node: &XmlNode,
        path: &str,
        list: Option<(u32, usize)>,
    ) -> Result<()> {
        self.bump_blocks()?;
        let style_name = node.attr(Some(TEXT_NS), "style-name");
        let paragraph_style = self.effective_style("paragraph", style_name)?;
        let mut pieces = Vec::new();
        self.collect_inline(node, path, &paragraph_style, &mut pieces)?;
        let mut paragraph = CT_P::new();
        {
            let mut target = Paragraph {
                inner: &mut paragraph,
            };
            apply_paragraph_style(&mut target, &paragraph_style);
            if node.is(TEXT_NS, "h") {
                let level = node
                    .attr(Some(TEXT_NS), "outline-level")
                    .and_then(|value| value.parse::<u32>().ok())
                    .unwrap_or(1)
                    .clamp(1, 9);
                target.set_style(&format!("Heading{level}"));
                target.set_outline_level(level - 1);
            }
            if let Some((num_id, level)) = list {
                target.set_numbering(num_id, level as u32);
            }
            for piece in pieces {
                match piece {
                    InlinePiece::Text(text, style) => {
                        self.bump_runs()?;
                        let mut run = target.add_run(&text);
                        apply_run_style(&mut run, &style);
                    }
                    InlinePiece::Break => {
                        self.bump_runs()?;
                        target.add_line_break();
                    }
                    InlinePiece::Tab => {
                        self.bump_runs()?;
                        target.add_tab();
                    }
                    InlinePiece::Image {
                        relationship,
                        width,
                        height,
                    } => {
                        self.bump_runs()?;
                        target.add_picture(&relationship, width, height);
                    }
                }
            }
        }
        self.document
            .document
            .body
            .content
            .push(BodyContent::Paragraph(paragraph));
        Ok(())
    }

    fn collect_inline(
        &mut self,
        node: &XmlNode,
        path: &str,
        inherited: &EffectiveStyle,
        output: &mut Vec<InlinePiece>,
    ) -> Result<()> {
        let mut whitespace = InlineWhitespace::default();
        self.collect_inline_inner(node, path, inherited, output, &mut whitespace)
    }

    fn collect_inline_inner(
        &mut self,
        node: &XmlNode,
        path: &str,
        inherited: &EffectiveStyle,
        output: &mut Vec<InlinePiece>,
        whitespace: &mut InlineWhitespace,
    ) -> Result<()> {
        let mut element_positions: HashMap<(Option<String>, String), usize> = HashMap::new();
        for child in &node.children {
            match child {
                XmlChild::Text(text) => {
                    self.bump_projected_text(text.len())?;
                    collapse_odt_text(text, inherited, whitespace, output);
                }
                XmlChild::Element(element) if element.is(TEXT_NS, "span") => {
                    let key = (element.name.namespace.clone(), element.name.local.clone());
                    let index = element_positions.entry(key).or_default();
                    *index += 1;
                    let child_path = format!("{path}/text:span[{}]", *index);
                    let mut style = inherited.clone();
                    let own =
                        self.effective_style("text", element.attr(Some(TEXT_NS), "style-name"))?;
                    style.overlay(&own);
                    self.collect_inline_inner(element, &child_path, &style, output, whitespace)?;
                }
                XmlChild::Element(element) if element.is(TEXT_NS, "s") => {
                    let count = repeated_count(element, TEXT_NS, "c", self.limits.retained_text)?;
                    self.bump_projected_text(count)?;
                    push_inline_text(output, " ".repeat(count), inherited.clone());
                    whitespace.pending = false;
                    whitespace.style = None;
                    whitespace.emitted = true;
                }
                XmlChild::Element(element) if element.is(TEXT_NS, "tab") => {
                    output.push(InlinePiece::Tab);
                    whitespace.pending = false;
                    whitespace.style = None;
                    whitespace.emitted = true;
                }
                XmlChild::Element(element) if element.is(TEXT_NS, "line-break") => {
                    output.push(InlinePiece::Break);
                    whitespace.pending = false;
                    whitespace.style = None;
                    whitespace.emitted = true;
                }
                XmlChild::Element(element) if element.is(DRAW_NS, "frame") => {
                    let key = (element.name.namespace.clone(), element.name.local.clone());
                    let index = element_positions.entry(key).or_default();
                    *index += 1;
                    let child_path = format!("{path}/draw:frame[{}]", *index);
                    if let Some(image) = self.project_image(element, &child_path)? {
                        output.push(image);
                        whitespace.pending = false;
                        whitespace.style = None;
                        whitespace.emitted = true;
                    }
                }
                XmlChild::Element(element) => {
                    let key = (element.name.namespace.clone(), element.name.local.clone());
                    let index = element_positions.entry(key).or_default();
                    *index += 1;
                    let child_path = format!("{path}/{}[{}]", display_name(element), *index);
                    self.diagnostic(
                        &child_path,
                        format!("unsupported ODT inline subtree {}", display_name(element)),
                    )?;
                }
            }
        }
        Ok(())
    }

    fn project_image(&mut self, frame: &XmlNode, path: &str) -> Result<Option<InlinePiece>> {
        let Some(image) = frame.elements().find(|node| node.is(DRAW_NS, "image")) else {
            self.diagnostic(path, "drawing frame has no supported image")?;
            return Ok(None);
        };
        let Some(href) = image.attr(Some(XLINK_NS), "href") else {
            self.diagnostic(path, "ODT image has no package target")?;
            return Ok(None);
        };
        let target = safe_image_target(href).ok_or_else(|| {
            odt_error(
                Some("content.xml"),
                0,
                format!("unsafe ODT image target {href}"),
            )
        })?;
        if self.encrypted.contains(&target) {
            return Err(odt_error(
                Some(&target),
                0,
                "referenced ODT image is encrypted",
            ));
        }
        let Some(bytes) = self.archive.read_optional(&target)? else {
            self.diagnostic(path, format!("missing ODT image target {target}"))?;
            return Ok(None);
        };
        let Some(info) = oxml_media::probe(&bytes) else {
            self.diagnostic(path, format!("unsupported ODT image content {target}"))?;
            return Ok(None);
        };
        let explicit = match (
            frame.attr(Some(SVG_NS), "width"),
            frame.attr(Some(SVG_NS), "height"),
        ) {
            (Some(width), Some(height)) => Some((
                parse_positive_length(width)
                    .map_err(|message| odt_error(Some("content.xml"), 0, message))?,
                parse_positive_length(height)
                    .map_err(|message| odt_error(Some("content.xml"), 0, message))?,
            )),
            (None, None) => None,
            _ => {
                self.diagnostic(
                    path,
                    "ODT image frame has only one dimension, using intrinsic size",
                )?;
                None
            }
        };
        let (width, height) = explicit
            .or_else(|| {
                info.native_size(72.0)
                    .map(|size| (Length::emu(size.width_emu), Length::emu(size.height_emu)))
            })
            .ok_or_else(|| odt_error(Some(&target), 0, "ODT image dimensions are unavailable"))?;
        let relationship = self.document.embed_image(&bytes, &target);
        Ok(Some(InlinePiece::Image {
            relationship,
            width,
            height,
        }))
    }

    fn project_table(&mut self, table: &XmlNode, path: &str) -> Result<()> {
        self.bump_blocks()?;
        let mut column_hint = 0_usize;
        for column in table
            .elements()
            .filter(|node| node.is(TABLE_NS, "table-column"))
        {
            column_hint = column_hint
                .checked_add(repeated_count(
                    column,
                    TABLE_NS,
                    "number-columns-repeated",
                    self.limits.columns,
                )?)
                .ok_or_else(|| odt_error(Some("content.xml"), 0, "ODT column count overflowed"))?;
        }
        let mut source_rows = Vec::new();
        collect_table_rows(table, &mut source_rows);
        let mut rows = Vec::new();
        for (row_index, row) in source_rows.into_iter().enumerate() {
            let repeated = repeated_count(row, TABLE_NS, "number-rows-repeated", self.limits.rows)?;
            for repeat in 0..repeated {
                self.rows = self
                    .rows
                    .checked_add(1)
                    .filter(|value| *value <= self.limits.rows)
                    .ok_or_else(|| {
                        odt_error(Some("content.xml"), 0, "ODT table exceeds the row limit")
                    })?;
                rows.push(self.parse_table_row(
                    row,
                    &format!("{path}/table:table-row[{}]", row_index + repeat + 1),
                )?);
            }
        }
        let columns = rows
            .iter()
            .map(|row| row.iter().map(|cell| cell.colspan).sum::<usize>())
            .max()
            .unwrap_or(column_hint)
            .max(column_hint);
        if columns == 0 || columns > self.limits.columns {
            return Err(odt_error(
                Some("content.xml"),
                0,
                "ODT table column count is invalid",
            ));
        }
        let mut tbl = CT_Tbl::new();
        let width = Twips(9000 / columns as i32);
        tbl.properties = Some(CT_TblPr {
            width: Some(CT_TblWidth::dxa(width.0 * columns as i32)),
            ..Default::default()
        });
        tbl.grid = Some(CT_TblGrid {
            columns: (0..columns).map(|_| CT_TblGridCol { width }).collect(),
        });
        let mut active: Vec<Option<(usize, usize)>> = vec![None; columns];
        for row in rows {
            let mut target = CT_Row::new();
            let mut column = 0_usize;
            let mut source = row.into_iter();
            while column < columns {
                if let Some((remaining, span)) = active[column] {
                    let mut cell = CT_Tc::new();
                    cell.properties = Some(CT_TcPr {
                        grid_span: (span > 1).then_some(span as u32),
                        v_merge: Some(VMerge::Continue),
                        ..Default::default()
                    });
                    target.cells.push(cell);
                    for slot in active.iter_mut().skip(column).take(span) {
                        *slot = if remaining > 1 {
                            Some((remaining - 1, span))
                        } else {
                            None
                        };
                    }
                    column += span;
                    continue;
                }
                let Some(model) = source.next() else {
                    target.cells.push(CT_Tc::new());
                    column += 1;
                    continue;
                };
                if column + model.colspan > columns {
                    return Err(odt_error(
                        Some("content.xml"),
                        0,
                        "ODT table span exceeds the grid",
                    ));
                }
                let mut cell = CT_Tc::new();
                cell.properties = Some(CT_TcPr {
                    grid_span: (model.colspan > 1).then_some(model.colspan as u32),
                    v_merge: (model.rowspan > 1).then_some(VMerge::Restart),
                    ..Default::default()
                });
                cell.content.clear();
                for paragraph in model.paragraphs {
                    cell.content.push(CellContent::Paragraph(paragraph));
                }
                if cell.paragraphs().is_empty() {
                    cell.content.push(CellContent::Paragraph(CT_P::new()));
                }
                if model.rowspan > 1 {
                    for slot in active.iter_mut().skip(column).take(model.colspan) {
                        if slot.is_some() {
                            return Err(odt_error(
                                Some("content.xml"),
                                0,
                                "overlapping ODT table spans",
                            ));
                        }
                        *slot = Some((model.rowspan - 1, model.colspan));
                    }
                }
                target.cells.push(cell);
                column += model.colspan;
            }
            if source.next().is_some() {
                return Err(odt_error(
                    Some("content.xml"),
                    0,
                    "ODT table row exceeds the grid",
                ));
            }
            tbl.rows.push(target);
        }
        self.document
            .document
            .body
            .content
            .push(BodyContent::Table(tbl));
        Ok(())
    }

    fn parse_table_row(&mut self, row: &XmlNode, path: &str) -> Result<Vec<TableCellModel>> {
        let mut output = Vec::new();
        for (index, cell) in row
            .elements()
            .filter(|node| {
                node.is(TABLE_NS, "table-cell") || node.is(TABLE_NS, "covered-table-cell")
            })
            .enumerate()
        {
            let repeat = repeated_count(
                cell,
                TABLE_NS,
                "number-columns-repeated",
                self.limits.columns,
            )?;
            for repeat_index in 0..repeat {
                self.cells = self
                    .cells
                    .checked_add(1)
                    .filter(|value| *value <= self.limits.cells)
                    .ok_or_else(|| {
                        odt_error(Some("content.xml"), 0, "ODT table exceeds the cell limit")
                    })?;
                if cell.is(TABLE_NS, "covered-table-cell") {
                    continue;
                }
                let colspan = repeated_count(
                    cell,
                    TABLE_NS,
                    "number-columns-spanned",
                    self.limits.columns,
                )?;
                let rowspan =
                    repeated_count(cell, TABLE_NS, "number-rows-spanned", self.limits.rows)?;
                let mut paragraphs = Vec::new();
                for (paragraph_index, paragraph) in cell
                    .elements()
                    .filter(|node| node.is(TEXT_NS, "p") || node.is(TEXT_NS, "h"))
                    .enumerate()
                {
                    let paragraph_path = format!(
                        "{path}/table:table-cell[{}]/text:p[{}]",
                        index + repeat_index + 1,
                        paragraph_index + 1
                    );
                    paragraphs.push(self.build_cell_paragraph(paragraph, &paragraph_path)?);
                }
                output.push(TableCellModel {
                    colspan,
                    rowspan,
                    paragraphs,
                });
            }
        }
        Ok(output)
    }

    fn build_cell_paragraph(&mut self, node: &XmlNode, path: &str) -> Result<CT_P> {
        self.bump_blocks()?;
        let paragraph_style =
            self.effective_style("paragraph", node.attr(Some(TEXT_NS), "style-name"))?;
        let mut pieces = Vec::new();
        self.collect_inline(node, path, &paragraph_style, &mut pieces)?;
        let mut paragraph = CT_P::new();
        let mut target = Paragraph {
            inner: &mut paragraph,
        };
        apply_paragraph_style(&mut target, &paragraph_style);
        for piece in pieces {
            match piece {
                InlinePiece::Text(text, style) => {
                    self.bump_runs()?;
                    let mut run = target.add_run(&text);
                    apply_run_style(&mut run, &style);
                }
                InlinePiece::Break => {
                    self.bump_runs()?;
                    target.add_line_break();
                }
                InlinePiece::Tab => {
                    self.bump_runs()?;
                    target.add_tab();
                }
                InlinePiece::Image {
                    relationship,
                    width,
                    height,
                } => {
                    self.bump_runs()?;
                    target.add_picture(&relationship, width, height);
                }
            }
        }
        Ok(paragraph)
    }

    fn diagnostic(&mut self, path: &str, message: impl Into<String>) -> Result<()> {
        let message = message.into();
        if !self
            .diagnostic_keys
            .insert((path.to_string(), message.clone()))
        {
            return Ok(());
        }
        if self.diagnostics.len() >= self.limits.diagnostics {
            return Err(odt_error(
                Some("content.xml"),
                0,
                "ODT exceeds the diagnostic limit",
            ));
        }
        self.diagnostics.push(OdtDiagnostic {
            path: path.to_string(),
            message,
        });
        Ok(())
    }

    fn bump_blocks(&mut self) -> Result<()> {
        self.blocks = self
            .blocks
            .checked_add(1)
            .filter(|value| *value <= self.limits.blocks)
            .ok_or_else(|| {
                odt_error(
                    Some("content.xml"),
                    0,
                    "ODT exceeds the projected block limit",
                )
            })?;
        Ok(())
    }

    fn bump_runs(&mut self) -> Result<()> {
        self.runs = self
            .runs
            .checked_add(1)
            .filter(|value| *value <= self.limits.runs)
            .ok_or_else(|| {
                odt_error(
                    Some("content.xml"),
                    0,
                    "ODT exceeds the projected run limit",
                )
            })?;
        Ok(())
    }

    fn bump_projected_text(&mut self, amount: usize) -> Result<()> {
        self.projected_text = self
            .projected_text
            .checked_add(amount)
            .filter(|value| *value <= self.limits.retained_text)
            .ok_or_else(|| {
                odt_error(
                    Some("content.xml"),
                    0,
                    "ODT exceeds the projected text limit",
                )
            })?;
        Ok(())
    }
}

#[derive(Clone)]
enum InlinePiece {
    Text(String, EffectiveStyle),
    Break,
    Tab,
    Image {
        relationship: String,
        width: Length,
        height: Length,
    },
}

#[derive(Default)]
struct InlineWhitespace {
    pending: bool,
    emitted: bool,
    style: Option<EffectiveStyle>,
}

fn collapse_odt_text(
    text: &str,
    style: &EffectiveStyle,
    whitespace: &mut InlineWhitespace,
    output: &mut Vec<InlinePiece>,
) {
    let mut visible = String::new();
    for character in text.chars() {
        if character.is_whitespace() {
            if !visible.is_empty() {
                push_inline_text(output, std::mem::take(&mut visible), style.clone());
            }
            whitespace.pending = true;
            whitespace.style.get_or_insert_with(|| style.clone());
        } else {
            if whitespace.pending && whitespace.emitted {
                push_inline_text(
                    output,
                    " ".to_string(),
                    whitespace.style.take().unwrap_or_else(|| style.clone()),
                );
            }
            visible.push(character);
            whitespace.pending = false;
            whitespace.style = None;
            whitespace.emitted = true;
        }
    }
    if !visible.is_empty() {
        push_inline_text(output, visible, style.clone());
    }
}

fn push_inline_text(output: &mut Vec<InlinePiece>, text: String, style: EffectiveStyle) {
    if let Some(InlinePiece::Text(previous, previous_style)) = output.last_mut()
        && *previous_style == style
    {
        previous.push_str(&text);
    } else {
        output.push(InlinePiece::Text(text, style));
    }
}

struct TableCellModel {
    colspan: usize,
    rowspan: usize,
    paragraphs: Vec<CT_P>,
}

#[derive(Clone)]
enum StyleChange {
    Font(String),
    Size(f64),
    Bold(bool),
    Italic(bool),
    Underline(bool),
    Strike(bool),
    Color(String),
    Background(String),
    Vertical(VerticalText),
    Alignment(Alignment),
    Before(Length),
    After(Length),
    Left(Length),
    Right(Length),
    First(Length),
    Line(LineHeight),
}

impl StyleChange {
    fn apply(self, style: &mut EffectiveStyle) {
        match self {
            Self::Font(value) => style.font = Some(value),
            Self::Size(value) => style.size = Some(value),
            Self::Bold(value) => style.bold = Some(value),
            Self::Italic(value) => style.italic = Some(value),
            Self::Underline(value) => style.underline = Some(value),
            Self::Strike(value) => style.strike = Some(value),
            Self::Color(value) => style.color = Some(value),
            Self::Background(value) => style.background = Some(value),
            Self::Vertical(value) => style.vertical = Some(value),
            Self::Alignment(value) => style.alignment = Some(value),
            Self::Before(value) => style.before = Some(value),
            Self::After(value) => style.after = Some(value),
            Self::Left(value) => style.left = Some(value),
            Self::Right(value) => style.right = Some(value),
            Self::First(value) => style.first = Some(value),
            Self::Line(value) => style.line = Some(value),
        }
    }
}

fn parse_text_property(
    attribute: &XmlAttribute,
    fonts: &HashMap<String, String>,
) -> Option<std::result::Result<StyleChange, String>> {
    let namespace = attribute.name.namespace.as_deref();
    let name = attribute.name.local.as_str();
    let value = attribute.value.trim();
    let lower = value.to_ascii_lowercase();
    let result = match (namespace, name) {
        (Some(STYLE_NS), "font-name") => Ok(StyleChange::Font(
            fonts
                .get(value)
                .cloned()
                .unwrap_or_else(|| value.to_string()),
        )),
        (Some(FO_NS), "font-family") => Ok(StyleChange::Font(
            value.trim_matches(['\'', '"']).to_string(),
        )),
        (Some(FO_NS), "font-size") => {
            parse_positive_length(value).map(|length| StyleChange::Size(length.to_pt()))
        }
        (Some(FO_NS), "font-weight") => match lower.as_str() {
            "bold" | "600" | "700" | "800" | "900" => Ok(StyleChange::Bold(true)),
            "normal" | "400" | "500" => Ok(StyleChange::Bold(false)),
            _ => Err(format!("unsupported font weight {value}")),
        },
        (Some(FO_NS), "font-style") => match lower.as_str() {
            "italic" | "oblique" => Ok(StyleChange::Italic(true)),
            "normal" => Ok(StyleChange::Italic(false)),
            _ => Err(format!("unsupported font style {value}")),
        },
        (Some(STYLE_NS), "text-underline-style") => Ok(StyleChange::Underline(lower != "none")),
        (Some(STYLE_NS), "text-line-through-style") => Ok(StyleChange::Strike(lower != "none")),
        (Some(FO_NS), "color") => parse_color(value).map(StyleChange::Color),
        (Some(FO_NS), "background-color") => parse_color(value).map(StyleChange::Background),
        (Some(STYLE_NS), "text-position") => {
            if lower.starts_with("super") {
                Ok(StyleChange::Vertical(VerticalText::Superscript))
            } else if lower.starts_with("sub") {
                Ok(StyleChange::Vertical(VerticalText::Subscript))
            } else {
                Err(format!("unsupported text position {value}"))
            }
        }
        _ => return None,
    };
    Some(result)
}

fn parse_paragraph_property(
    attribute: &XmlAttribute,
) -> Option<std::result::Result<StyleChange, String>> {
    let namespace = attribute.name.namespace.as_deref();
    let name = attribute.name.local.as_str();
    let value = attribute.value.trim();
    let lower = value.to_ascii_lowercase();
    let result = match (namespace, name) {
        (Some(FO_NS), "text-align") => match lower.as_str() {
            "start" | "left" => Ok(StyleChange::Alignment(Alignment::Left)),
            "center" => Ok(StyleChange::Alignment(Alignment::Center)),
            "end" | "right" => Ok(StyleChange::Alignment(Alignment::Right)),
            "justify" => Ok(StyleChange::Alignment(Alignment::Justify)),
            _ => Err(format!("unsupported paragraph alignment {value}")),
        },
        (Some(FO_NS), "margin-top") => parse_length(value, false).map(StyleChange::Before),
        (Some(FO_NS), "margin-bottom") => parse_length(value, false).map(StyleChange::After),
        (Some(FO_NS), "margin-left") => parse_length(value, false).map(StyleChange::Left),
        (Some(FO_NS), "margin-right") => parse_length(value, false).map(StyleChange::Right),
        (Some(FO_NS), "text-indent") => parse_length(value, true).map(StyleChange::First),
        (Some(FO_NS), "line-height") if lower.ends_with('%') => {
            match lower.trim_end_matches('%').parse::<f64>() {
                Ok(percentage)
                    if percentage.is_finite() && percentage > 0.0 && percentage <= 10_000.0 =>
                {
                    Ok(StyleChange::Line(LineHeight::Multiple(percentage / 100.0)))
                }
                Ok(_) => Err(format!(
                    "percentage line height is outside the supported range {value}"
                )),
                Err(_) => Err(format!("invalid percentage line height {value}")),
            }
        }
        (Some(FO_NS), "line-height") => parse_positive_length(value)
            .map(|length| StyleChange::Line(LineHeight::Exact(length.to_pt()))),
        _ => return None,
    };
    Some(result)
}

fn is_ignorable_style_attribute(attribute: &XmlAttribute) -> bool {
    matches!(
        (
            attribute.name.namespace.as_deref(),
            attribute.name.local.as_str()
        ),
        (Some(STYLE_NS), "font-name-asian" | "font-name-complex")
            | (Some(STYLE_NS), "font-size-asian" | "font-size-complex")
            | (Some(STYLE_NS), "font-weight-asian" | "font-weight-complex")
            | (Some(STYLE_NS), "font-style-asian" | "font-style-complex")
            | (Some(STYLE_NS), "writing-mode")
    )
}

fn apply_paragraph_style(paragraph: &mut Paragraph<'_>, style: &EffectiveStyle) {
    if let Some(value) = style.alignment {
        paragraph.set_alignment(value);
    }
    if let Some(value) = style.before {
        paragraph.set_space_before(value);
    }
    if let Some(value) = style.after {
        paragraph.set_space_after(value);
    }
    if let Some(value) = style.left {
        paragraph.set_indent_left(value);
    }
    if let Some(value) = style.right {
        paragraph.set_indent_right(value);
    }
    if let Some(value) = style.first {
        paragraph.set_signed_first_line_indent_value(Some(value));
    }
    if let Some(value) = style.line {
        match value {
            LineHeight::Exact(points) => paragraph.set_line_spacing(points),
            LineHeight::Multiple(multiple) => paragraph.set_line_spacing_multiple(multiple),
        }
    }
}

fn apply_run_style(run: &mut Run<'_>, style: &EffectiveStyle) {
    if let Some(value) = &style.font {
        run.set_font(value);
    }
    if let Some(value) = style.size {
        run.set_size(value);
    }
    if let Some(value) = style.bold {
        run.set_bold(value);
    }
    if let Some(value) = style.italic {
        run.set_italic(value);
    }
    if let Some(value) = style.underline {
        run.set_underline(value);
    }
    if let Some(value) = style.strike {
        run.set_strike(value);
    }
    if let Some(value) = &style.color {
        run.set_color(value);
    }
    if let Some(value) = &style.background {
        run.set_highlight(value);
    }
    match style.vertical {
        Some(VerticalText::Superscript) => run.set_superscript(),
        Some(VerticalText::Subscript) => run.set_subscript(),
        None => {}
    }
}

fn parse_length(value: &str, signed: bool) -> std::result::Result<Length, String> {
    let value = value.trim().to_ascii_lowercase();
    let units = [
        ("cm", 72.0 / 2.54),
        ("mm", 72.0 / 25.4),
        ("in", 72.0),
        ("pt", 1.0),
        ("pc", 12.0),
        ("px", 0.75),
    ];
    let (number, factor) = units
        .iter()
        .find_map(|(unit, factor)| value.strip_suffix(unit).map(|number| (number, *factor)))
        .ok_or_else(|| format!("unsupported ODT length {value}"))?;
    let points = number
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("invalid ODT length {value}"))?
        * factor;
    if !points.is_finite() || points.abs() > 1_000_000.0 || (!signed && points < 0.0) {
        return Err(format!("ODT length is outside the supported range {value}"));
    }
    Ok(Length::pt(points))
}

fn parse_positive_length(value: &str) -> std::result::Result<Length, String> {
    let length = parse_length(value, false)?;
    if length.to_emu() <= 0 {
        return Err(format!("ODT length must be positive {value}"));
    }
    Ok(length)
}

fn parse_color(value: &str) -> std::result::Result<String, String> {
    let value = value.trim();
    if value.len() == 7
        && value.starts_with('#')
        && value[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        Ok(value[1..].to_ascii_uppercase())
    } else {
        Err(format!("unsupported ODT color {value}"))
    }
}

fn repeated_count(node: &XmlNode, namespace: &str, local: &str, maximum: usize) -> Result<usize> {
    let Some(value) = node.attr(Some(namespace), local) else {
        return Ok(1);
    };
    let value = value.parse::<usize>().map_err(|_| {
        odt_error(
            Some("content.xml"),
            0,
            format!("invalid ODT repeat or span value {value}"),
        )
    })?;
    if value == 0 || value > maximum {
        return Err(odt_error(
            Some("content.xml"),
            0,
            "ODT repeat or span exceeds its bound",
        ));
    }
    Ok(value)
}

fn collect_table_rows<'a>(node: &'a XmlNode, rows: &mut Vec<&'a XmlNode>) {
    for child in node.elements() {
        if child.is(TABLE_NS, "table-row") {
            rows.push(child);
        } else if child.is(TABLE_NS, "table-rows") || child.is(TABLE_NS, "table-header-rows") {
            collect_table_rows(child, rows);
        }
    }
}

fn safe_image_target(href: &str) -> Option<String> {
    if href.is_empty()
        || href.starts_with('/')
        || href.starts_with('#')
        || href.contains(':')
        || href.contains('\\')
        || href.contains('\0')
        || href
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        None
    } else {
        Some(href.to_string())
    }
}

fn required_attr<'a>(
    node: &'a XmlNode,
    namespace: &str,
    local: &str,
    part: &str,
) -> Result<&'a str> {
    node.attr(Some(namespace), local).ok_or_else(|| {
        odt_error(
            Some(part),
            0,
            format!("{} requires {local}", display_name(node)),
        )
    })
}

fn display_name(node: &XmlNode) -> String {
    let prefix = match node.name.namespace.as_deref() {
        Some(OFFICE_NS) => "office",
        Some(TEXT_NS) => "text",
        Some(STYLE_NS) => "style",
        Some(TABLE_NS) => "table",
        Some(DRAW_NS) => "draw",
        Some(MANIFEST_NS) => "manifest",
        Some(_) => "foreign",
        None => "unbound",
    };
    format!("{prefix}:{}", node.name.local)
}

fn odt_error(part: Option<&str>, offset: u64, message: impl Into<String>) -> Error {
    Error::Odt {
        part: part.map(str::to_string),
        offset,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};

    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    use super::*;

    const PNG: &[u8] = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08, 0xd7, 0x63, 0xf8,
        0xcf, 0xc0, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0xe2, 0x21, 0xbc, 0x33, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];

    fn package_with(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut output = Cursor::new(Vec::new());
        let mut archive = ZipWriter::new(&mut output);
        for (name, bytes) in entries {
            let method = if *name == "mimetype" {
                CompressionMethod::Stored
            } else {
                CompressionMethod::Deflated
            };
            archive
                .start_file(
                    *name,
                    SimpleFileOptions::default().compression_method(method),
                )
                .unwrap();
            archive.write_all(bytes).unwrap();
        }
        archive.finish().unwrap();
        output.into_inner()
    }

    fn package(content: &str, styles: Option<&str>, extra: &[(&str, &[u8])]) -> Vec<u8> {
        let mut entries = vec![
            ("mimetype", ODT_MIMETYPE),
            ("content.xml", content.as_bytes()),
        ];
        if let Some(styles) = styles {
            entries.push(("styles.xml", styles.as_bytes()));
        }
        entries.extend_from_slice(extra);
        package_with(&entries)
    }

    fn content(body: &str, automatic: &str) -> String {
        format!(
            r#"<?xml version="1.0"?><office:document-content xmlns:office="{OFFICE_NS}" xmlns:text="{TEXT_NS}" xmlns:style="{STYLE_NS}" xmlns:fo="{FO_NS}" xmlns:table="{TABLE_NS}" xmlns:draw="{DRAW_NS}" xmlns:xlink="{XLINK_NS}" xmlns:svg="{SVG_NS}"><office:automatic-styles>{automatic}</office:automatic-styles><office:body><office:text>{body}</office:text></office:body></office:document-content>"#
        )
    }

    #[test]
    fn odt_archive_rejects_unsafe_duplicate_encrypted_and_oversized_entries() {
        let safe = package(&content("<text:p>x</text:p>", ""), None, &[]);
        assert!(Document::from_odt_bytes(&safe).is_ok());

        for name in [
            "../content.xml",
            "/content.xml",
            "bad\\name",
            "./content.xml",
            "bad\0name",
        ] {
            let unsafe_package = package_with(&[("mimetype", ODT_MIMETYPE), (name, b"x")]);
            assert!(
                Document::from_odt_bytes(&unsafe_package).is_err(),
                "accepted {name}"
            );
        }
        let mut duplicate = package_with(&[
            ("mimetype", ODT_MIMETYPE),
            ("content.xml", content("<text:p>x</text:p>", "").as_bytes()),
            ("dontent.xml", content("<text:p>y</text:p>", "").as_bytes()),
        ]);
        for offset in 0..duplicate.len().saturating_sub(b"dontent.xml".len()) {
            if &duplicate[offset..offset + b"dontent.xml".len()] == b"dontent.xml" {
                duplicate[offset..offset + b"content.xml".len()].copy_from_slice(b"content.xml");
            }
        }
        assert!(!duplicate.windows(11).any(|window| window == b"dontent.xml"));
        assert!(Document::from_odt_bytes(&duplicate).is_err());

        let mut encrypted = safe.clone();
        for offset in 0..encrypted.len().saturating_sub(10) {
            if &encrypted[offset..offset + 4] == b"PK\x03\x04" {
                encrypted[offset + 6] |= 1;
            } else if &encrypted[offset..offset + 4] == b"PK\x01\x02" {
                encrypted[offset + 8] |= 1;
            }
        }
        assert!(Document::from_odt_bytes(&encrypted).is_err());

        let mut unsupported_compression = safe.clone();
        for offset in 0..unsupported_compression.len().saturating_sub(12) {
            if &unsupported_compression[offset..offset + 4] == b"PK\x03\x04" {
                unsupported_compression[offset + 8..offset + 10]
                    .copy_from_slice(&12_u16.to_le_bytes());
            } else if &unsupported_compression[offset..offset + 4] == b"PK\x01\x02" {
                unsupported_compression[offset + 10..offset + 12]
                    .copy_from_slice(&12_u16.to_le_bytes());
            }
        }
        assert!(Document::from_odt_bytes(&unsupported_compression).is_err());

        let mut directory = Cursor::new(Vec::new());
        let mut directory_archive = ZipWriter::new(&mut directory);
        directory_archive
            .add_directory("folder/", SimpleFileOptions::default())
            .unwrap();
        directory_archive.finish().unwrap();
        assert!(Document::from_odt_bytes(directory.get_ref()).is_err());

        let manifest = format!(
            r#"<manifest:manifest xmlns:manifest="{MANIFEST_NS}"><manifest:file-entry manifest:full-path="Pictures/pixel.png"><manifest:encryption-data/></manifest:file-entry></manifest:manifest>"#
        );
        let image_body = content(
            r#"<text:p><draw:frame svg:width="1in" svg:height="1in"><draw:image xlink:href="Pictures/pixel.png"/></draw:frame></text:p>"#,
            "",
        );
        let manifest_encrypted = package(
            &image_body,
            None,
            &[
                ("META-INF/manifest.xml", manifest.as_bytes()),
                ("Pictures/pixel.png", PNG),
            ],
        );
        assert!(Document::from_odt_bytes(&manifest_encrypted).is_err());

        let encrypted_content_manifest = format!(
            r#"<manifest:manifest xmlns:manifest="{MANIFEST_NS}"><manifest:file-entry manifest:full-path="content.xml"><manifest:encryption-data/></manifest:file-entry></manifest:manifest>"#
        );
        let encrypted_content = package(
            &content("<text:p>x</text:p>", ""),
            None,
            &[(
                "META-INF/manifest.xml",
                encrypted_content_manifest.as_bytes(),
            )],
        );
        assert!(Document::from_odt_bytes(&encrypted_content).is_err());

        let entry_limits = PackageReadLimits {
            max_entries: 1,
            max_part_uncompressed_bytes: u64::MAX,
            max_total_uncompressed_bytes: u64::MAX,
        };
        assert!(Document::from_odt_bytes_with_limits(&safe, entry_limits).is_err());

        let part_limits = PackageReadLimits {
            max_entries: usize::MAX,
            max_part_uncompressed_bytes: 16,
            max_total_uncompressed_bytes: u64::MAX,
        };
        assert!(Document::from_odt_bytes_with_limits(&safe, part_limits).is_err());

        let total_limits = PackageReadLimits {
            max_entries: usize::MAX,
            max_part_uncompressed_bytes: u64::MAX,
            max_total_uncompressed_bytes: 16,
        };
        assert!(Document::from_odt_bytes_with_limits(&safe, total_limits).is_err());
    }

    #[test]
    fn odt_styles_resolve_defaults_parents_and_automatic_overrides() {
        let styles = format!(
            r##"<o:document-styles xmlns:o="{OFFICE_NS}" xmlns:s="{STYLE_NS}" xmlns:f="{FO_NS}" xmlns:v="{SVG_NS}"><o:font-face-decls><s:font-face s:name="Alias" v:font-family="'Liberation Serif'"/></o:font-face-decls><o:styles><s:default-style s:family="paragraph"><s:text-properties f:font-size="10pt"/></s:default-style><s:style s:name="Parent" s:family="paragraph"><s:text-properties s:font-name="Alias" f:font-weight="bold" f:font-style="italic" s:text-underline-style="solid" s:text-line-through-style="solid" f:color="#123456" f:background-color="#FEDCBA" s:text-position="super"/></s:style><s:style s:name="Child" s:family="paragraph" s:parent-style-name="Parent"><s:paragraph-properties f:text-align="center" f:margin-top="3pt" f:margin-bottom="4pt" f:margin-left="5pt" f:margin-right="6pt" f:text-indent="-2pt" f:line-height="150%"/></s:style></o:styles></o:document-styles>"##
        );
        let input = content(r#"<text:p text:style-name="Child">styled</text:p>"#, "");
        let parsed = Document::from_odt_bytes(&package(&input, Some(&styles), &[])).unwrap();
        let paragraph = parsed.document.paragraph(0).unwrap();
        assert_eq!(paragraph.alignment(), Some(Alignment::Center));
        let run = paragraph.run(0).unwrap();
        assert_eq!(run.bold_value(), Some(true));
        assert_eq!(run.italic_value(), Some(true));
        assert_eq!(run.underline_code_value(), Some(1));
        assert_eq!(run.strike_value(), Some(true));
        assert_eq!(run.size(), Some(10.0));
        assert_eq!(run.font_name(), Some("Liberation Serif"));
        assert_eq!(run.color(), Some("123456"));
        assert_eq!(run.highlight().as_deref(), Some("FEDCBA"));
        assert_eq!(run.vert_align(), Some("superscript"));
        assert_eq!(paragraph.space_before(), Some(Length::pt(3.0)));
        assert_eq!(paragraph.space_after(), Some(Length::pt(4.0)));
        assert_eq!(paragraph.indent_left(), Some(Length::pt(5.0)));
        assert_eq!(paragraph.indent_right(), Some(Length::pt(6.0)));
        assert_eq!(paragraph.first_line_indent(), Some(Length::pt(-2.0)));
        assert_eq!(paragraph.line_spacing_multiple(), Some(1.5));

        let cycle = format!(
            r#"<office:document-styles xmlns:office="{OFFICE_NS}" xmlns:style="{STYLE_NS}"><office:styles><style:style style:name="A" style:family="paragraph" style:parent-style-name="B"/><style:style style:name="B" style:family="paragraph" style:parent-style-name="A"/></office:styles></office:document-styles>"#
        );
        assert!(Document::from_odt_bytes(&package(&input, Some(&cycle), &[])).is_err());

        let missing_parent = format!(
            r#"<office:document-styles xmlns:office="{OFFICE_NS}" xmlns:style="{STYLE_NS}"><office:styles><style:style style:name="Child" style:family="paragraph" style:parent-style-name="Missing"/></office:styles></office:document-styles>"#
        );
        let parsed =
            Document::from_odt_bytes(&package(&input, Some(&missing_parent), &[])).unwrap();
        assert!(
            parsed
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("missing ODT style Missing"))
        );
    }

    #[test]
    fn odt_reader_rejects_malformed_or_unbounded_xml() {
        let malformed = package("<!DOCTYPE x><x/>", None, &[]);
        assert!(Document::from_odt_bytes(&malformed).is_err());
        let deep = format!(
            "{}x{}",
            "<text:span>".repeat(257),
            "</text:span>".repeat(257)
        );
        let input = content(&format!("<text:p>{deep}</text:p>"), "");
        assert!(Document::from_odt_bytes(&package(&input, None, &[])).is_err());
        let repeated = content(
            r#"<table:table><table:table-row table:number-rows-repeated="10001"><table:table-cell/></table:table-row></table:table>"#,
            "",
        );
        assert!(Document::from_odt_bytes(&package(&repeated, None, &[])).is_err());

        let unresolved = content("<text:p>&missing;</text:p>", "");
        assert!(Document::from_odt_bytes(&package(&unresolved, None, &[])).is_err());
        let unknown_namespace = content("<text:p><bad:span>x</bad:span></text:p>", "");
        assert!(Document::from_odt_bytes(&package(&unknown_namespace, None, &[])).is_err());

        let shallow_limits = OdtLimits {
            xml_depth: 1,
            ..OdtLimits::DEFAULT
        };
        assert!(parse_xml("content.xml", b"<root><empty/></root>", shallow_limits).is_err());

        let bad_styles = "<root/>";
        let input = content("<text:p>x</text:p>", "");
        assert!(Document::from_odt_bytes(&package(&input, Some(bad_styles), &[])).is_err());

        for (body, limits) in [
            (
                "<text:p>x</text:p>",
                OdtLimits {
                    blocks: 0,
                    ..OdtLimits::DEFAULT
                },
            ),
            (
                "<text:p>x</text:p>",
                OdtLimits {
                    runs: 0,
                    ..OdtLimits::DEFAULT
                },
            ),
            (
                r#"<text:p><text:s text:c="3"/><text:s text:c="3"/></text:p>"#,
                OdtLimits {
                    retained_text: 5,
                    ..OdtLimits::DEFAULT
                },
            ),
            (
                r#"<table:table><table:table-column table:number-columns-repeated="2"/><table:table-row><table:table-cell/></table:table-row></table:table>"#,
                OdtLimits {
                    columns: 1,
                    ..OdtLimits::DEFAULT
                },
            ),
            (
                r#"<table:table><table:table-row><table:table-cell table:number-columns-repeated="2"/></table:table-row></table:table>"#,
                OdtLimits {
                    cells: 1,
                    ..OdtLimits::DEFAULT
                },
            ),
            (
                "<text:p>before<office:annotation/>after</text:p>",
                OdtLimits {
                    diagnostics: 0,
                    ..OdtLimits::DEFAULT
                },
            ),
        ] {
            let input = content(body, "");
            let package = package(&input, None, &[]);
            assert!(
                from_odt_with_limits(&package, limits).is_err(),
                "accepted {body}"
            );
        }
    }

    #[test]
    fn unsupported_odt_content_is_diagnosed_without_dropping_supported_siblings() {
        let input = content(
            r#"<text:p>before<office:annotation>drop</office:annotation>after</text:p><text:list><text:list-item><text:p>item</text:p></text:list-item></text:list><table:table><table:table-column/><table:table-row><table:table-cell><text:p>cell</text:p></table:table-cell></table:table-row></table:table><text:p><draw:frame svg:width="1in" svg:height="1in"><draw:image xlink:href="Pictures/pixel.png"/></draw:frame></text:p>"#,
            "",
        );
        let parsed =
            Document::from_odt_bytes(&package(&input, None, &[("Pictures/pixel.png", PNG)]))
                .unwrap();
        assert_eq!(parsed.document.paragraph(0).unwrap().text(), "beforeafter");
        assert_eq!(parsed.document.paragraph(1).unwrap().text(), "item");
        assert_eq!(parsed.document.table_count(), 1);
        assert_eq!(parsed.document.images().len(), 1);
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(
            parsed.diagnostics[0].path,
            "office:body/office:text/text:p[1]/office:annotation[1]"
        );
    }

    #[test]
    fn odt_reader_projects_text_formatting_lists_tables_and_images() {
        let automatic = r##"<style:style style:name="P1" style:family="paragraph"><style:paragraph-properties fo:text-align="right"/></style:style><style:style style:name="T1" style:family="text"><style:text-properties fo:font-weight="bold" fo:font-style="italic" fo:color="#123456"/></style:style><text:list-style style:name="L1"><text:list-level-style-number text:level="1"/></text:list-style>"##;
        let body = r#"<text:p text:style-name="P1">A <text:span text:style-name="T1">formatted</text:span><text:tab/>line<text:line-break/>tail<draw:frame svg:width="1in" svg:height="0.5in"><draw:image xlink:href="Pictures/pixel.png"/></draw:frame></text:p><text:list text:style-name="L1"><text:list-item><text:p>one</text:p></text:list-item></text:list><table:table><table:table-column table:number-columns-repeated="2"/><table:table-row><table:table-cell table:number-columns-spanned="2"><text:p>wide</text:p><text:p>second</text:p></table:table-cell><table:covered-table-cell/></table:table-row><table:table-row><table:table-cell table:number-rows-spanned="2"><text:p>vertical</text:p></table:table-cell><table:table-cell><text:p>top</text:p></table:table-cell></table:table-row><table:table-row><table:covered-table-cell/><table:table-cell><text:p>bottom</text:p></table:table-cell></table:table-row></table:table>"#;
        let input = content(body, automatic);
        let parsed =
            Document::from_odt_bytes(&package(&input, None, &[("Pictures/pixel.png", PNG)]))
                .unwrap();
        let paragraph = parsed.document.paragraph(0).unwrap();
        assert_eq!(paragraph.alignment(), Some(Alignment::Right));
        assert_eq!(paragraph.run(1).unwrap().bold_value(), Some(true));
        assert_eq!(paragraph.run(1).unwrap().italic_value(), Some(true));
        assert_eq!(paragraph.run(1).unwrap().color(), Some("123456"));
        let numbering = parsed.document.paragraph(1).unwrap().numbering().unwrap();
        assert_eq!(
            parsed.document.numbering_is_bullet(numbering.0),
            Some(false)
        );
        let table = parsed.document.table(0).unwrap();
        let cell = table.cell(0, 0).unwrap();
        assert_eq!(cell.grid_span(), Some(2));
        assert_eq!(cell.paragraph_count(), 2);
        assert!(matches!(
            table.cell(1, 0).unwrap().v_merge(),
            Some(VMerge::Restart)
        ));
        assert!(matches!(
            table.cell(2, 0).unwrap().v_merge(),
            Some(VMerge::Continue)
        ));
        let images = parsed.document.images();
        assert_eq!(images.len(), 1);
        assert_eq!(
            (images[0].width_emu, images[0].height_emu),
            (914_400, 457_200)
        );

        let root = std::env::temp_dir().join(format!("rdocx-open-odt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir(&root).unwrap();
        let path = root.join("source.odt");
        std::fs::write(&path, package(&input, None, &[("Pictures/pixel.png", PNG)])).unwrap();
        let opened = Document::open_odt(&path).unwrap();
        assert_eq!(opened.document.text(), parsed.document.text());
        std::fs::remove_dir_all(root).unwrap();
    }
}
