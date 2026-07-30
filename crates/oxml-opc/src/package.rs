//! OPC Package reader and writer for ZIP-based OOXML packages.

use std::collections::HashMap;
use std::io::{Read, Seek, Write};
use std::path::Path;

use zip::ZipWriter;
use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;

use crate::content_types::ContentTypes;
use crate::error::{OpcError, Result};
use crate::relationship::{Relationships, rel_types};

/// A single part within the OPC package.
#[derive(Debug, Clone)]
pub struct PackagePart {
    /// Part name (URI), e.g. "/word/document.xml"
    pub name: String,
    /// Raw bytes of the part content
    pub data: Vec<u8>,
}

/// An in-memory representation of an OPC package (ZIP archive).
#[derive(Debug, Clone)]
pub struct OpcPackage {
    /// Content types from `[Content_Types].xml`
    pub content_types: ContentTypes,
    /// Package-level relationships from `_rels/.rels`
    pub package_rels: Relationships,
    /// Part-level relationships keyed by the part they belong to.
    /// Key is the part name (e.g. "/word/document.xml"),
    /// value is the parsed relationships.
    pub part_rels: HashMap<String, Relationships>,
    /// All parts keyed by their URI (e.g. "/word/document.xml").
    pub parts: HashMap<String, Vec<u8>>,
}

impl OpcPackage {
    /// Open an OPC package from a file path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        Self::from_reader(file)
    }

    /// Open an OPC package from any reader that implements Read + Seek.
    pub fn from_reader<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut archive = ZipArchive::new(reader)?;
        let mut raw_parts: HashMap<String, Vec<u8>> = HashMap::new();

        // Read all entries from the ZIP
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            if entry.is_dir() {
                continue;
            }
            let name = normalize_part_name(entry.name());
            let name = name.strip_prefix('/').unwrap_or(&name).to_string();
            let mut data = Vec::new();
            entry.read_to_end(&mut data)?;
            raw_parts.insert(name, data);
        }

        // Parse [Content_Types].xml
        let ct_xml = raw_parts
            .get("[Content_Types].xml")
            .ok_or_else(|| OpcError::PartNotFound("[Content_Types].xml".into()))?;
        let content_types = ContentTypes::from_xml(ct_xml)?;

        // Parse package-level relationships: _rels/.rels
        let package_rels = if let Some(rels_xml) = raw_parts.get("_rels/.rels") {
            Relationships::from_xml(rels_xml)?
        } else {
            Relationships::new()
        };

        // Parse all part-level .rels files
        let mut part_rels = HashMap::new();
        let rels_entries: Vec<String> = raw_parts
            .keys()
            .filter(|k| k.ends_with(".rels") && *k != "_rels/.rels")
            .cloned()
            .collect();

        for rels_path in rels_entries {
            if let Some(xml_data) = raw_parts.get(&rels_path) {
                let rels = Relationships::from_xml(xml_data)?;
                // Convert rels path to the part name it belongs to.
                // e.g. "word/_rels/document.xml.rels" → "/word/document.xml"
                let part_name = rels_path_to_part_name(&rels_path);
                part_rels.insert(part_name, rels);
            }
        }

        // Build parts map with leading "/" normalized
        let mut parts = HashMap::new();
        for (name, data) in &raw_parts {
            if name == "[Content_Types].xml" || name == "_rels/.rels" || name.ends_with(".rels") {
                continue;
            }
            let normalized = if name.starts_with('/') {
                name.clone()
            } else {
                format!("/{name}")
            };
            parts.insert(normalized, data.clone());
        }

        Ok(OpcPackage {
            content_types,
            package_rels,
            part_rels,
            parts,
        })
    }

    /// Save the OPC package to a file path.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = std::fs::File::create(path)?;
        self.write_to(file)
    }

    /// Write the OPC package to any writer.
    pub fn write_to<W: Write + Seek>(&self, writer: W) -> Result<()> {
        let mut zip = ZipWriter::new(writer);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // Write [Content_Types].xml
        let ct_xml = self.content_types.to_xml()?;
        zip.start_file("[Content_Types].xml", options)?;
        zip.write_all(&ct_xml)?;

        // Write _rels/.rels
        let pkg_rels_xml = self.package_rels.to_xml()?;
        zip.start_file("_rels/.rels", options)?;
        zip.write_all(&pkg_rels_xml)?;

        // Write part-level .rels files. Both loops iterate in sorted order so
        // that saving the same package twice produces byte-identical output;
        // a HashMap's order would vary between runs.
        let mut rels_names: Vec<&String> = self.part_rels.keys().collect();
        rels_names.sort_unstable();
        for part_name in rels_names {
            let rels = &self.part_rels[part_name];
            let rels_path = part_name_to_rels_path(part_name);
            let rels_xml = rels.to_xml()?;
            zip.start_file(&rels_path, options)?;
            zip.write_all(&rels_xml)?;
        }

        // Write all parts
        let mut part_names: Vec<&String> = self.parts.keys().collect();
        part_names.sort_unstable();
        for name in part_names {
            // Strip leading "/" for ZIP entry name
            let zip_name = name.strip_prefix('/').unwrap_or(name);
            zip.start_file(zip_name, options)?;
            zip.write_all(&self.parts[name])?;
        }

        zip.finish()?;
        Ok(())
    }

    /// Get raw bytes of a part by its URI.
    pub fn get_part(&self, part_name: &str) -> Option<&[u8]> {
        self.parts.get(part_name).map(|v| v.as_slice())
    }

    /// Set (or replace) a part's raw bytes.
    pub fn set_part(&mut self, part_name: &str, data: Vec<u8>) {
        self.parts.insert(part_name.to_string(), data);
    }

    /// Get the relationships for a specific part.
    pub fn get_part_rels(&self, part_name: &str) -> Option<&Relationships> {
        self.part_rels.get(part_name)
    }

    /// Get or create the relationships for a specific part.
    pub fn get_or_create_part_rels(&mut self, part_name: &str) -> &mut Relationships {
        self.part_rels.entry(part_name.to_string()).or_default()
    }

    /// Resolve the target URI of a relationship relative to its source part.
    ///
    /// `.` and `..` segments are collapsed, so a target such as
    /// `../media/image1.png` on `/word/charts/chart1.xml` resolves to
    /// `/media/image1.png` and matches the key it is stored under. Without
    /// this, parts referenced through a parent directory could never be found.
    pub fn resolve_rel_target(source_part: &str, rel_target: &str) -> String {
        let joined = if rel_target.starts_with('/') {
            rel_target.to_string()
        } else {
            // Directory of the source part, including the trailing slash.
            let dir = match source_part.rfind('/') {
                Some(pos) => &source_part[..=pos],
                None => "/",
            };
            format!("{dir}{rel_target}")
        };
        normalize_part_name(&joined)
    }

    /// Find the main document part URI by looking at package relationships.
    pub fn main_document_part(&self) -> Option<String> {
        self.package_rels
            .get_by_type(rel_types::DOCUMENT)
            .map(|rel| {
                if rel.target.starts_with('/') {
                    rel.target.clone()
                } else {
                    format!("/{}", rel.target)
                }
            })
    }

    /// Create an empty package with the universal OPC content types.
    pub fn new() -> Self {
        OpcPackage {
            content_types: ContentTypes::minimal(),
            package_rels: Relationships::new(),
            part_rels: HashMap::new(),
            parts: HashMap::new(),
        }
    }

    /// Create a package whose office document relationship targets a main part.
    pub fn with_main_part(part_name: &str, content_type: &str) -> Self {
        let mut package = Self::new();
        let part_name = part_name.strip_prefix('/').unwrap_or(part_name);
        package.package_rels.add(rel_types::DOCUMENT, part_name);
        package
            .content_types
            .add_override(&format!("/{part_name}"), content_type);
        package
    }
}

impl Default for OpcPackage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
fn docx_package() -> OpcPackage {
    let mut package = OpcPackage::with_main_part(
        "word/document.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
    );
    package.content_types.add_override(
        "/word/styles.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
    );
    package
}

/// Collapse `.` and `..` segments in an absolute part name.
///
/// A `..` that would escape the package root is dropped, matching how OPC
/// consumers treat over-long parent traversals.
fn normalize_part_name(path: &str) -> String {
    if !path.contains("./") && !path.ends_with("/.") && !path.ends_with("/..") {
        return path.to_string();
    }

    let mut segments: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            other => segments.push(other),
        }
    }

    let mut out = String::with_capacity(path.len());
    for segment in segments {
        out.push('/');
        out.push_str(segment);
    }
    if out.is_empty() { "/".to_string() } else { out }
}

/// Convert a .rels file path to the part name it belongs to.
/// e.g. "word/_rels/document.xml.rels" → "/word/document.xml"
fn rels_path_to_part_name(rels_path: &str) -> String {
    // Strip the ".rels" suffix once (`trim_end_matches` would strip repeats,
    // turning "a.rels.rels" into "a") and drop only the "_rels" path segment
    // that directly precedes the file name.
    let without_suffix = rels_path.strip_suffix(".rels").unwrap_or(rels_path);
    let path = match without_suffix.rfind("_rels/") {
        Some(pos) => format!(
            "{}{}",
            &without_suffix[..pos],
            &without_suffix[pos + "_rels/".len()..]
        ),
        None => without_suffix.to_string(),
    };
    if path.starts_with('/') {
        path
    } else {
        format!("/{path}")
    }
}

/// Convert a part name to its .rels file path.
/// e.g. "/word/document.xml" → "word/_rels/document.xml.rels"
fn part_name_to_rels_path(part_name: &str) -> String {
    let name = part_name.strip_prefix('/').unwrap_or(part_name);
    if let Some(pos) = name.rfind('/') {
        let dir = &name[..pos];
        let file = &name[pos + 1..];
        format!("{dir}/_rels/{file}.rels")
    } else {
        format!("_rels/{name}.rels")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package_zip(entries: &[(&str, &[u8])]) -> std::io::Cursor<Vec<u8>> {
        let mut buffer = std::io::Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut buffer);
            let options = SimpleFileOptions::default();
            for (name, data) in entries {
                zip.start_file(name, options).unwrap();
                zip.write_all(data).unwrap();
            }
            zip.finish().unwrap();
        }
        buffer.set_position(0);
        buffer
    }

    const MINIMAL_CONTENT_TYPES: &[u8] =
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="xml" ContentType="application/xml"/>
</Types>"#;

    fn independently_built_pptx() -> std::io::Cursor<Vec<u8>> {
        const CONTENT_TYPES: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  <Override PartName="/ppt/slides/slide1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>
  <Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
  <Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
  <Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
</Types>"#;
        const PACKAGE_RELS: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>"#;
        const PRESENTATION_RELS: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide1.xml"/>
</Relationships>"#;
        const SLIDE_RELS: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
</Relationships>"#;
        const SLIDE_LAYOUT_RELS: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="../slideMasters/slideMaster1.xml"/>
</Relationships>"#;
        const SLIDE_MASTER_RELS: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml"/>
</Relationships>"#;
        const PRESENTATION: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst>
  <p:sldIdLst><p:sldId id="256" r:id="rId2"/></p:sldIdLst>
  <p:sldSz cx="12192000" cy="6858000"/>
  <p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>"#;
        const SLIDE: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld>
  <p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sld>"#;
        const SLIDE_LAYOUT: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<p:sldLayout xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld>
  <p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sldLayout>"#;
        const SLIDE_MASTER: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<p:sldMaster xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld>
  <p:clrMap accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" bg1="lt1" bg2="lt2" folHlink="folHlink" hlink="hlink" tx1="dk1" tx2="dk2"/>
  <p:sldLayoutIdLst><p:sldLayoutId id="1" r:id="rId1"/></p:sldLayoutIdLst>
  <p:txStyles><p:titleStyle/><p:bodyStyle/><p:otherStyle/></p:txStyles>
</p:sldMaster>"#;
        const THEME: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Minimal">
  <a:themeElements>
    <a:clrScheme name="Minimal">
      <a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1>
      <a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1>
      <a:dk2><a:srgbClr val="1F497D"/></a:dk2>
      <a:lt2><a:srgbClr val="EEECE1"/></a:lt2>
      <a:accent1><a:srgbClr val="4F81BD"/></a:accent1>
      <a:accent2><a:srgbClr val="C0504D"/></a:accent2>
      <a:accent3><a:srgbClr val="9BBB59"/></a:accent3>
      <a:accent4><a:srgbClr val="8064A2"/></a:accent4>
      <a:accent5><a:srgbClr val="4BACC6"/></a:accent5>
      <a:accent6><a:srgbClr val="F79646"/></a:accent6>
      <a:hlink><a:srgbClr val="0000FF"/></a:hlink>
      <a:folHlink><a:srgbClr val="800080"/></a:folHlink>
    </a:clrScheme>
    <a:fontScheme name="Minimal">
      <a:majorFont><a:latin typeface="Arial"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont>
      <a:minorFont><a:latin typeface="Arial"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont>
    </a:fontScheme>
    <a:fmtScheme name="Minimal">
      <a:fillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:fillStyleLst>
      <a:lnStyleLst><a:ln w="6350"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/></a:ln><a:ln w="12700"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/></a:ln><a:ln w="19050"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/></a:ln></a:lnStyleLst>
      <a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle></a:effectStyleLst>
      <a:bgFillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:bgFillStyleLst>
    </a:fmtScheme>
  </a:themeElements>
</a:theme>"#;

        package_zip(&[
            ("[Content_Types].xml", CONTENT_TYPES),
            ("_rels/.rels", PACKAGE_RELS),
            ("ppt/presentation.xml", PRESENTATION),
            ("ppt/_rels/presentation.xml.rels", PRESENTATION_RELS),
            ("ppt/slides/slide1.xml", SLIDE),
            ("ppt/slides/_rels/slide1.xml.rels", SLIDE_RELS),
            ("ppt/slideLayouts/slideLayout1.xml", SLIDE_LAYOUT),
            (
                "ppt/slideLayouts/_rels/slideLayout1.xml.rels",
                SLIDE_LAYOUT_RELS,
            ),
            ("ppt/slideMasters/slideMaster1.xml", SLIDE_MASTER),
            (
                "ppt/slideMasters/_rels/slideMaster1.xml.rels",
                SLIDE_MASTER_RELS,
            ),
            ("ppt/theme/theme1.xml", THEME),
        ])
    }

    fn pptx_package() -> OpcPackage {
        let mut package =
            OpcPackage::with_main_part("ppt/presentation.xml", crate::content_types::PRESENTATION);
        package
            .content_types
            .add_override("/ppt/slides/slide1.xml", crate::content_types::SLIDE);
        package.content_types.add_override(
            "/ppt/slideLayouts/slideLayout1.xml",
            crate::content_types::SLIDE_LAYOUT,
        );
        package.set_part("/ppt/presentation.xml", b"<p:presentation/>".to_vec());
        package.set_part("/ppt/slides/slide1.xml", b"<p:sld/>".to_vec());
        package.set_part(
            "/ppt/slideLayouts/slideLayout1.xml",
            b"<p:sldLayout/>".to_vec(),
        );
        package
            .get_or_create_part_rels("/ppt/presentation.xml")
            .add(rel_types::SLIDE, "slides/slide1.xml");
        package
            .get_or_create_part_rels("/ppt/slides/slide1.xml")
            .add(rel_types::SLIDE_LAYOUT, "../slideLayouts/slideLayout1.xml");
        package
    }

    #[test]
    fn with_main_part_resolves_and_round_trips() {
        let mut package = OpcPackage::with_main_part(
            "ppt/presentation.xml",
            "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml",
        );
        package.set_part("/ppt/presentation.xml", b"<p:presentation/>".to_vec());

        assert_eq!(
            package.main_document_part().as_deref(),
            Some("/ppt/presentation.xml")
        );
        assert_eq!(
            package
                .content_types
                .content_type_for("/ppt/presentation.xml"),
            Some(
                "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"
            )
        );

        let mut buffer = std::io::Cursor::new(Vec::new());
        package.write_to(&mut buffer).unwrap();
        buffer.set_position(0);
        let round_tripped = OpcPackage::from_reader(buffer).unwrap();

        assert_eq!(
            round_tripped.main_document_part().as_deref(),
            Some("/ppt/presentation.xml")
        );
        assert_eq!(
            round_tripped.get_part("/ppt/presentation.xml"),
            Some(b"<p:presentation/>".as_slice())
        );
    }

    #[test]
    fn pptx_package_resolves_main_slide_and_layout_parts() {
        let package = pptx_package();
        let mut buffer = std::io::Cursor::new(Vec::new());
        package.write_to(&mut buffer).unwrap();
        buffer.set_position(0);
        let reopened = OpcPackage::from_reader(buffer).unwrap();

        let presentation_part = reopened.main_document_part().unwrap();
        assert_eq!(presentation_part, "/ppt/presentation.xml");

        let slide_target = &reopened
            .get_part_rels(&presentation_part)
            .unwrap()
            .get_by_type(rel_types::SLIDE)
            .unwrap()
            .target;
        let slide_part = OpcPackage::resolve_rel_target(&presentation_part, slide_target);
        assert_eq!(slide_part, "/ppt/slides/slide1.xml");

        let layout_target = &reopened
            .get_part_rels(&slide_part)
            .unwrap()
            .get_by_type(rel_types::SLIDE_LAYOUT)
            .unwrap()
            .target;
        let layout_part = OpcPackage::resolve_rel_target(&slide_part, layout_target);
        assert_eq!(layout_part, "/ppt/slideLayouts/slideLayout1.xml");

        assert!(reopened.parts.contains_key(&presentation_part));
        assert!(reopened.parts.contains_key(&slide_part));
        assert!(reopened.parts.contains_key(&layout_part));
    }

    #[test]
    fn independently_built_pptx_opens_and_resolves_relationships() {
        let package = OpcPackage::from_reader(independently_built_pptx()).unwrap();

        let presentation_part = package.main_document_part().unwrap();
        assert_eq!(presentation_part, "/ppt/presentation.xml");

        let presentation_master_target = &package
            .get_part_rels(&presentation_part)
            .unwrap()
            .get_by_type(rel_types::SLIDE_MASTER)
            .unwrap()
            .target;
        let presentation_master_part =
            OpcPackage::resolve_rel_target(&presentation_part, presentation_master_target);
        assert_eq!(
            presentation_master_part,
            "/ppt/slideMasters/slideMaster1.xml"
        );

        let slide_target = &package
            .get_part_rels(&presentation_part)
            .unwrap()
            .get_by_type(rel_types::SLIDE)
            .unwrap()
            .target;
        let slide_part = OpcPackage::resolve_rel_target(&presentation_part, slide_target);
        assert_eq!(slide_part, "/ppt/slides/slide1.xml");

        let layout_target = &package
            .get_part_rels(&slide_part)
            .unwrap()
            .get_by_type(rel_types::SLIDE_LAYOUT)
            .unwrap()
            .target;
        let layout_part = OpcPackage::resolve_rel_target(&slide_part, layout_target);
        assert_eq!(layout_part, "/ppt/slideLayouts/slideLayout1.xml");

        let master_target = &package
            .get_part_rels(&layout_part)
            .unwrap()
            .get_by_type(rel_types::SLIDE_MASTER)
            .unwrap()
            .target;
        let master_part = OpcPackage::resolve_rel_target(&layout_part, master_target);
        assert_eq!(master_part, "/ppt/slideMasters/slideMaster1.xml");
        assert_eq!(master_part, presentation_master_part);

        let master_layout_target = &package
            .get_part_rels(&master_part)
            .unwrap()
            .get_by_type(rel_types::SLIDE_LAYOUT)
            .unwrap()
            .target;
        let master_layout_part = OpcPackage::resolve_rel_target(&master_part, master_layout_target);
        assert_eq!(master_layout_part, layout_part);

        let theme_target = &package
            .get_part_rels(&master_part)
            .unwrap()
            .get_by_type(rel_types::THEME)
            .unwrap()
            .target;
        let theme_part = OpcPackage::resolve_rel_target(&master_part, theme_target);
        assert_eq!(theme_part, "/ppt/theme/theme1.xml");

        assert!(package.parts.contains_key(&presentation_part));
        assert!(package.parts.contains_key(&slide_part));
        assert!(package.parts.contains_key(&layout_part));
        assert!(package.parts.contains_key(&master_part));
        assert!(package.parts.contains_key(&theme_part));
    }

    #[test]
    fn presentation_layout_target_resolves_one_directory_up() {
        assert_eq!(
            OpcPackage::resolve_rel_target(
                "/ppt/slides/slide1.xml",
                "../slideLayouts/slideLayout1.xml"
            ),
            "/ppt/slideLayouts/slideLayout1.xml"
        );
    }

    #[test]
    fn rels_path_conversion() {
        assert_eq!(
            rels_path_to_part_name("word/_rels/document.xml.rels"),
            "/word/document.xml"
        );
        assert_eq!(
            part_name_to_rels_path("/word/document.xml"),
            "word/_rels/document.xml.rels"
        );
    }

    #[test]
    fn resolve_relative_target() {
        assert_eq!(
            OpcPackage::resolve_rel_target("/word/document.xml", "styles.xml"),
            "/word/styles.xml"
        );
        assert_eq!(
            OpcPackage::resolve_rel_target("/word/document.xml", "/word/styles.xml"),
            "/word/styles.xml"
        );
    }

    #[test]
    fn resolve_target_collapses_parent_segments() {
        // Charts and headers routinely reference media through a parent dir.
        assert_eq!(
            OpcPackage::resolve_rel_target("/word/charts/chart1.xml", "../media/image1.png"),
            "/word/media/image1.png"
        );
        assert_eq!(
            OpcPackage::resolve_rel_target("/word/document.xml", "./styles.xml"),
            "/word/styles.xml"
        );
        assert_eq!(
            OpcPackage::resolve_rel_target("/word/document.xml", "../../../etc/passwd"),
            "/etc/passwd"
        );
    }

    #[test]
    fn zip_entry_that_escapes_root_is_clamped_to_root() {
        let archive = package_zip(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("../../etc/passwd", b"not a real password file"),
        ]);

        let package = OpcPackage::from_reader(archive).unwrap();

        assert_eq!(
            package.get_part("/etc/passwd"),
            Some(b"not a real password file".as_slice())
        );
        assert!(package.parts.keys().all(|name| !name.contains("..")));
    }

    #[test]
    fn absolute_zip_entry_is_normalized_to_package_root() {
        let archive = package_zip(&[
            ("/[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("/absolute/path.xml", b"<absolute/>"),
        ]);

        let package = OpcPackage::from_reader(archive).unwrap();

        assert_eq!(
            package.get_part("/absolute/path.xml"),
            Some(b"<absolute/>".as_slice())
        );
        assert_eq!(
            package
                .parts
                .keys()
                .filter(|name| name.as_str() == "/absolute/path.xml")
                .count(),
            1
        );
    }

    #[test]
    fn rels_path_suffix_is_stripped_once() {
        assert_eq!(
            rels_path_to_part_name("word/_rels/document.xml.rels"),
            "/word/document.xml"
        );
        // A part whose own name ends in ".rels" must keep that segment.
        assert_eq!(
            rels_path_to_part_name("word/_rels/odd.rels.rels"),
            "/word/odd.rels"
        );
    }

    #[test]
    fn saved_packages_are_byte_identical() {
        let mut pkg = docx_package();
        for i in 0..40 {
            pkg.set_part(&format!("/word/media/image{i}.png"), vec![i as u8]);
        }
        pkg.get_or_create_part_rels("/word/document.xml")
            .add(rel_types::STYLES, "styles.xml");

        let write = || {
            let mut buf = std::io::Cursor::new(Vec::new());
            pkg.write_to(&mut buf).unwrap();
            buf.into_inner()
        };
        assert_eq!(write(), write());
    }

    #[test]
    fn new_docx_package() {
        let pkg = docx_package();
        assert!(pkg.main_document_part().is_some());
        assert_eq!(pkg.main_document_part().unwrap(), "/word/document.xml");
    }

    #[test]
    fn round_trip_package() {
        let mut pkg = docx_package();
        pkg.set_part("/word/document.xml", b"<document/>".to_vec());

        // Write to memory
        let mut buf = std::io::Cursor::new(Vec::new());
        pkg.write_to(&mut buf).unwrap();

        // Read back
        buf.set_position(0);
        let pkg2 = OpcPackage::from_reader(buf).unwrap();
        assert_eq!(
            pkg2.get_part("/word/document.xml"),
            Some(b"<document/>".as_slice())
        );
        assert!(pkg2.main_document_part().is_some());
    }
}
