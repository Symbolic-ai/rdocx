//! Native inventory and bounded replacement of Word building blocks.

use std::collections::HashSet;

use oxml_opc::OpcPackage;
use oxml_opc::relationship::rel_types;
use rdocx_oxml::document::CT_Body;
use rdocx_oxml::glossary::{CT_DocPart, CT_GlossaryDocument};

use crate::{ContentLocation, Document, DocumentFragment, Error, FragmentConflictPolicy, Result};

/// The supported classification of a glossary entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingBlockKind {
    AutoText,
    BuildingBlock,
}

/// The editable supported projection of one existing glossary entry.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildingBlock {
    pub name: String,
    pub kind: BuildingBlockKind,
    pub category: Option<String>,
    pub description: Option<String>,
    pub guid: Option<String>,
    pub gallery: Option<String>,
    pub behaviors: Vec<String>,
    pub body: CT_Body,
}

/// One building block and its stable package-scoped identity.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildingBlockInfo {
    pub glossary_part: String,
    pub ordinal: usize,
    pub block: BuildingBlock,
}

pub(crate) fn load_glossary(
    package: &OpcPackage,
    document_part: &str,
) -> Result<Option<(String, CT_GlossaryDocument)>> {
    let Some(relationships) = package.get_part_rels(document_part) else {
        return Ok(None);
    };
    let glossary = relationships
        .items
        .iter()
        .filter(|relationship| {
            relationship.rel_type == rel_types::GLOSSARY_DOCUMENT
                && crate::document::relationship_is_internal(relationship)
        })
        .collect::<Vec<_>>();
    if glossary.is_empty() {
        return Ok(None);
    }
    if glossary.len() != 1 {
        return Err(Error::Other(
            "the main document must own at most one glossary relationship".to_owned(),
        ));
    }
    let mut ids = HashSet::new();
    if relationships
        .items
        .iter()
        .any(|relationship| !ids.insert(relationship.id.as_str()))
    {
        return Err(Error::Other(
            "the main document relationship ids must be unique".to_owned(),
        ));
    }
    let relationship = glossary[0];
    validate_internal_target(document_part, &relationship.target)?;
    let part_name = OpcPackage::resolve_rel_target(document_part, &relationship.target);
    let xml = package
        .get_part(&part_name)
        .ok_or_else(|| Error::Other("the glossary relationship target is missing".to_owned()))?;
    if package.content_types.override_for(&part_name)
        != Some(oxml_opc::content_types::WORD_GLOSSARY)
    {
        return Err(Error::Other(
            "the glossary relationship target has the wrong content type".to_owned(),
        ));
    }
    Ok(Some((part_name, CT_GlossaryDocument::from_xml(xml)?)))
}

pub(crate) fn validate_internal_target(source_part: &str, target: &str) -> Result<()> {
    if !relationship_target_is_normalized_pack_uri(target) {
        return Err(Error::Other(
            "the glossary target is not a safe part URI".to_owned(),
        ));
    }
    let mut depth = if target.starts_with('/') {
        0
    } else {
        source_part
            .trim_start_matches('/')
            .rsplit_once('/')
            .map(|(directory, _)| directory.split('/').filter(|part| !part.is_empty()).count())
            .unwrap_or(0)
    };
    for component in target.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                depth = depth.checked_sub(1).ok_or_else(|| {
                    Error::Other("the glossary target escapes the package root".to_owned())
                })?;
            }
            _ => depth += 1,
        }
    }
    Ok(())
}

fn relationship_target_is_normalized_pack_uri(target: &str) -> bool {
    if target.is_empty()
        || target.ends_with('/')
        || target.contains("//")
        || target.contains(['\\', '?', '#'])
        || !target.is_ascii()
    {
        return false;
    }
    if !target.starts_with('/')
        && target
            .split('/')
            .next()
            .is_some_and(|segment| segment.contains(':'))
    {
        return false;
    }
    let bytes = target.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'%' {
            let (Some(&high), Some(&low)) = (bytes.get(index + 1), bytes.get(index + 2)) else {
                return false;
            };
            if !high.is_ascii_hexdigit()
                || !low.is_ascii_hexdigit()
                || high.is_ascii_lowercase()
                || low.is_ascii_lowercase()
            {
                return false;
            }
            let decoded = (pack_uri_hex_value(high) << 4) | pack_uri_hex_value(low);
            if decoded.is_ascii_alphanumeric()
                || matches!(decoded, b'-' | b'.' | b'_' | b'~' | b'/' | b'\\')
                || decoded.is_ascii_control()
            {
                return false;
            }
            index += 3;
            continue;
        }
        if !(byte.is_ascii_alphanumeric()
            || matches!(
                byte,
                b'-' | b'.'
                    | b'_'
                    | b'~'
                    | b'!'
                    | b'$'
                    | b'&'
                    | b'\''
                    | b'('
                    | b')'
                    | b'*'
                    | b'+'
                    | b','
                    | b';'
                    | b'='
                    | b':'
                    | b'@'
                    | b'/'
            ))
        {
            return false;
        }
        index += 1;
    }
    target
        .split('/')
        .filter(|segment| !segment.is_empty())
        .all(|segment| matches!(segment, "..") || (segment != "." && !segment.ends_with('.')))
}

fn pack_uri_hex_value(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'A'..=b'F' => byte - b'A' + 10,
        _ => 0,
    }
}

fn facade_block(part: &CT_DocPart) -> BuildingBlock {
    BuildingBlock {
        name: part.properties.name.clone().unwrap_or_default(),
        kind: if part.properties.types.iter().any(|value| value == "autoExp") {
            BuildingBlockKind::AutoText
        } else {
            BuildingBlockKind::BuildingBlock
        },
        category: part.properties.category.clone(),
        description: part.properties.description.clone(),
        guid: part.properties.guid.clone(),
        gallery: part.properties.gallery.clone(),
        behaviors: part.properties.behaviors.clone(),
        body: part.body.clone(),
    }
}

fn apply_block(part: &mut CT_DocPart, block: BuildingBlock) -> Result<()> {
    if block.name.is_empty() {
        return Err(Error::Other(
            "a building block name must not be empty".to_owned(),
        ));
    }
    if block.category.is_some() != block.gallery.is_some() {
        return Err(Error::Other(
            "a building block category requires both name and gallery".to_owned(),
        ));
    }
    if block
        .gallery
        .as_deref()
        .is_some_and(|gallery| !valid_building_block_gallery(gallery))
    {
        return Err(Error::Other(
            "a building block gallery must be a schema enumeration".to_owned(),
        ));
    }
    if block
        .behaviors
        .iter()
        .any(|behavior| !matches!(behavior.as_str(), "content" | "p" | "pg"))
    {
        return Err(Error::Other(
            "a building block behavior must be content, p, or pg".to_owned(),
        ));
    }
    if block.guid.as_deref().is_some_and(|guid| !valid_guid(guid)) {
        return Err(Error::Other(
            "a building block GUID must use braced uppercase lexical form".to_owned(),
        ));
    }
    part.properties.name = Some(block.name);
    part.properties.category = block.category;
    part.properties.description = block.description;
    part.properties.guid = block.guid;
    part.properties.gallery = block.gallery;
    part.properties.behaviors = block.behaviors;
    match block.kind {
        BuildingBlockKind::AutoText => {
            if !part.properties.types.iter().any(|value| value == "autoExp") {
                part.properties.types.push("autoExp".to_owned());
            }
        }
        BuildingBlockKind::BuildingBlock => {
            part.properties.types.retain(|value| value != "autoExp");
        }
    }
    part.body = block.body;
    Ok(())
}

fn valid_building_block_gallery(value: &str) -> bool {
    matches!(
        value,
        "placeholder"
            | "any"
            | "default"
            | "docParts"
            | "coverPg"
            | "eq"
            | "ftrs"
            | "hdrs"
            | "pgNum"
            | "tbls"
            | "watermarks"
            | "autoTxt"
            | "txtBox"
            | "pgNumT"
            | "pgNumB"
            | "pgNumMargins"
            | "tblOfContents"
            | "bib"
            | "custQuickParts"
            | "custCoverPg"
            | "custEq"
            | "custFtrs"
            | "custHdrs"
            | "custPgNum"
            | "custTbls"
            | "custWatermarks"
            | "custAutoTxt"
            | "custTxtBox"
            | "custPgNumT"
            | "custPgNumB"
            | "custPgNumMargins"
            | "custTblOfContents"
            | "custBib"
            | "custom1"
            | "custom2"
            | "custom3"
            | "custom4"
            | "custom5"
    )
}

fn valid_guid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 38
        && bytes.first() == Some(&b'{')
        && bytes.last() == Some(&b'}')
        && [9, 14, 19, 24]
            .into_iter()
            .all(|index| bytes.get(index) == Some(&b'-'))
        && bytes.iter().enumerate().all(|(index, byte)| {
            matches!(index, 0 | 9 | 14 | 19 | 24 | 37) || matches!(byte, b'0'..=b'9' | b'A'..=b'F')
        })
}

impl Document {
    /// Inventory relationship-owned glossary entries in source order.
    pub fn building_blocks(&self) -> Result<Vec<BuildingBlockInfo>> {
        let Some(glossary) = self.glossary.as_ref() else {
            return Ok(Vec::new());
        };
        let part_name = self
            .glossary_part_name
            .as_ref()
            .expect("loaded glossary has a part name");
        Ok(glossary
            .doc_parts
            .iter()
            .enumerate()
            .map(|(ordinal, part)| BuildingBlockInfo {
                glossary_part: part_name.clone(),
                ordinal,
                block: facade_block(part),
            })
            .collect())
    }

    /// Replace one existing glossary entry through a staged save and reopen.
    pub fn replace_building_block(
        &mut self,
        glossary_part: &str,
        ordinal: usize,
        mut block: BuildingBlock,
    ) -> Result<BuildingBlockInfo> {
        if self.glossary_part_name.as_deref() != Some(glossary_part) {
            return Err(Error::Other("stale glossary part identity".to_owned()));
        }
        let mut candidate = self.clone_for_staging();
        let part = candidate
            .glossary
            .as_mut()
            .and_then(|glossary| glossary.doc_parts.get_mut(ordinal))
            .ok_or_else(|| Error::Other("stale building block ordinal".to_owned()))?;
        if facade_block(part).body != block.body {
            let mut writer = quick_xml::Writer::new(Vec::new());
            block.body.to_xml(&mut writer)?;
            let xml = writer.into_inner();
            let scoped = String::from_utf8(xml)
                .map_err(|_| Error::Other("invalid glossary body encoding".into()))?
                .replacen(
                    "<w:body>",
                    &format!("<w:body xmlns:w=\"{}\">", rdocx_oxml::namespace::W_NS),
                    1,
                );
            let omitted = Document::omit_comment_markers(scoped.as_bytes())?;
            let body = glossary_body_as_document(&omitted)?;
            block.body = rdocx_oxml::document::CT_Document::from_xml(&body)?.body;
        }
        apply_block(part, block.clone())?;
        candidate.glossary_dirty = true;
        candidate.preserve_glossary_drawing_ids_staged()?;
        candidate.reconcile_comment_removal(self)?;
        let reopened = candidate.prepare_and_reopen_staged()?;
        let result = reopened
            .building_blocks()?
            .into_iter()
            .find(|entry| entry.glossary_part == glossary_part && entry.ordinal == ordinal)
            .ok_or_else(|| {
                Error::Other("building block identity did not survive reopen".to_owned())
            })?;
        if result.block != block {
            return Err(Error::Other(
                "building block replacement did not survive reopen".to_owned(),
            ));
        }
        self.commit_staged_mutation(reopened);
        Ok(result)
    }
}

impl Document {
    pub(crate) fn checked_building_block(&self, entry: &BuildingBlockInfo) -> Result<()> {
        let current = self.building_blocks()?.into_iter().find(|current| {
            current.glossary_part == entry.glossary_part && current.ordinal == entry.ordinal
        });
        if current.as_ref() != Some(entry) {
            return Err(Error::Other("stale building block snapshot".to_owned()));
        }
        Ok(())
    }

    fn ensure_glossary_staged(&mut self) -> Result<String> {
        let glossary_relationships =
            self.package
                .get_part_rels(&self.doc_part_name)
                .map_or(Vec::new(), |rels| {
                    rels.items
                        .iter()
                        .filter(|rel| rel.rel_type == rel_types::GLOSSARY_DOCUMENT)
                        .collect()
                });
        if glossary_relationships
            .iter()
            .any(|rel| !crate::document::relationship_is_internal(rel))
            || glossary_relationships.len() > 1
        {
            return Err(Error::Other(
                "unsafe or duplicate glossary relationship".to_owned(),
            ));
        }
        let loaded = load_glossary(&self.package, &self.doc_part_name)?;
        if let Some((part, _)) = loaded {
            return Ok(part);
        }
        let part = self.reserve_document_part_bundle(
            None,
            "/word/glossary/document.xml",
            rel_types::GLOSSARY_DOCUMENT,
            oxml_opc::content_types::WORD_GLOSSARY,
        )?;
        self.glossary = Some(CT_GlossaryDocument::new()?);
        self.glossary_part_name = Some(part.clone());
        self.glossary_dirty = true;
        Ok(part)
    }

    fn add_building_block_staged(
        &mut self,
        mut part: CT_DocPart,
        block: BuildingBlock,
    ) -> Result<usize> {
        let glossary = self
            .glossary
            .as_mut()
            .ok_or_else(|| Error::Other("glossary model is missing".to_owned()))?;
        if glossary
            .doc_parts
            .iter()
            .any(|part| part.properties.name.as_deref() == Some(&block.name))
        {
            return Err(Error::Other("duplicate building block name".to_owned()));
        }
        apply_block(&mut part, block)?;
        let ordinal = glossary.doc_parts.len();
        glossary.doc_parts.push(part);
        self.glossary_dirty = true;
        self.preserve_glossary_drawing_ids_staged()?;
        Ok(ordinal)
    }

    /// Create a dependency-free owned building block.
    pub fn create_building_block(&mut self, block: BuildingBlock) -> Result<BuildingBlockInfo> {
        let mut writer = quick_xml::Writer::new(Vec::new());
        block.body.to_xml(&mut writer)?;
        let body_xml = writer.into_inner();
        let content = body_xml
            .strip_prefix(b"<w:body>")
            .and_then(|xml| xml.strip_suffix(b"</w:body>"))
            .ok_or_else(|| Error::Other("invalid building block body wrapper".to_owned()))?;
        let mut part = CT_DocPart::from_body_xml(content)?;
        apply_block(&mut part, block.clone())?;
        validate_relationship_free_body(content)?;
        if block.body.sect_pr.is_some() {
            return Err(Error::Other(
                "building blocks cannot own section properties".to_owned(),
            ));
        }
        let mut candidate = self.clone_for_staging();
        candidate.prepare_staged_package()?;
        candidate.ensure_glossary_staged()?;
        let ordinal = candidate.add_building_block_staged(part, block)?;
        let reopened = candidate.prepare_and_reopen_staged()?;
        let result = reopened.building_blocks()?.remove(ordinal);
        self.commit_staged_mutation(reopened);
        Ok(result)
    }

    /// Create an entry with content and dependencies from an owned fragment.
    /// The metadata body's content must be empty. The fragment supplies its body.
    pub fn create_building_block_from_fragment(
        &mut self,
        mut block: BuildingBlock,
        fragment: &DocumentFragment,
        policy: FragmentConflictPolicy,
    ) -> Result<BuildingBlockInfo> {
        if !block.body.content.is_empty()
            || block.body.sect_pr.is_some()
            || fragment.contains_section_properties()
        {
            return Err(Error::Other(
                "fragment building block requires empty metadata body and no section properties"
                    .to_owned(),
            ));
        }
        let mut candidate = self.clone_for_staging();
        candidate.prepare_staged_package()?;
        let part_name = candidate.ensure_glossary_staged()?;
        let content = fragment.import_content_staged(&mut candidate, &part_name, policy)?;
        let part = CT_DocPart::from_body_xml(&content)?;
        block.body = part.body.clone();
        let ordinal = candidate.add_building_block_staged(part, block)?;
        let reopened = candidate.prepare_and_reopen_staged()?;
        let result = reopened.building_blocks()?.remove(ordinal);
        self.commit_staged_mutation(reopened);
        Ok(result)
    }

    /// Update an entry only while its inventory snapshot still matches.
    pub fn update_building_block(
        &mut self,
        entry: &BuildingBlockInfo,
        block: BuildingBlock,
    ) -> Result<BuildingBlockInfo> {
        self.checked_building_block(entry)?;
        if self
            .building_blocks()?
            .iter()
            .any(|other| other.ordinal != entry.ordinal && other.block.name == block.name)
        {
            return Err(Error::Other("duplicate building block name".to_owned()));
        }
        if block.body != entry.block.body {
            return Err(Error::Other(
                "update must retain the existing dependency-backed body".to_owned(),
            ));
        }
        self.replace_building_block(&entry.glossary_part, entry.ordinal, block)
    }

    /// Replace entry content and metadata through the dependency import transaction.
    pub fn update_building_block_from_fragment(
        &mut self,
        entry: &BuildingBlockInfo,
        mut block: BuildingBlock,
        fragment: &DocumentFragment,
        policy: FragmentConflictPolicy,
    ) -> Result<BuildingBlockInfo> {
        self.checked_building_block(entry)?;
        if !block.body.content.is_empty()
            || block.body.sect_pr.is_some()
            || fragment.contains_section_properties()
        {
            return Err(Error::Other(
                "fragment building block requires empty metadata body and no section properties"
                    .to_owned(),
            ));
        }
        let mut candidate = self.clone_for_staging();
        candidate.prepare_staged_package()?;
        let content =
            fragment.import_content_staged(&mut candidate, &entry.glossary_part, policy)?;
        let glossary = candidate
            .glossary
            .as_mut()
            .ok_or_else(|| Error::Other("glossary model is missing".to_owned()))?;
        if glossary
            .doc_parts
            .iter()
            .enumerate()
            .any(|(ordinal, part)| {
                ordinal != entry.ordinal && part.properties.name.as_deref() == Some(&block.name)
            })
        {
            return Err(Error::Other("duplicate building block name".to_owned()));
        }
        let part = glossary
            .doc_parts
            .get_mut(entry.ordinal)
            .ok_or_else(|| Error::Other("stale building block ordinal".to_owned()))?;
        part.replace_body_content_xml(&content)?;
        block.body = part.body.clone();
        apply_block(part, block)?;
        candidate.glossary_dirty = true;
        candidate.preserve_glossary_drawing_ids_staged()?;
        candidate.reconcile_comment_removal(self)?;
        let reopened = candidate.prepare_and_reopen_staged()?;
        let result = reopened.building_blocks()?.remove(entry.ordinal);
        self.commit_staged_mutation(reopened);
        Ok(result)
    }

    /// Remove one snapshot-checked entry, retaining the valid glossary bundle.
    pub fn remove_building_block(&mut self, entry: &BuildingBlockInfo) -> Result<BuildingBlock> {
        self.checked_building_block(entry)?;
        let mut candidate = self.clone_for_staging();
        candidate
            .glossary
            .as_mut()
            .ok_or_else(|| Error::Other("glossary model is missing".to_owned()))?
            .doc_parts
            .remove(entry.ordinal);
        candidate.glossary_dirty = true;
        candidate.preserve_glossary_drawing_ids_staged()?;
        candidate.reconcile_comment_removal(self)?;
        let reopened = candidate.prepare_and_reopen_staged()?;
        self.commit_staged_mutation(reopened);
        Ok(entry.block.clone())
    }

    /// Capture retained entry XML with the glossary's relationship scope.
    pub fn building_block_fragment(&self, entry: &BuildingBlockInfo) -> Result<DocumentFragment> {
        self.checked_building_block(entry)?;
        let mut candidate = self.clone_for_staging();
        candidate.prepare_staged_package()?;
        let xml = candidate
            .package
            .get_part(&entry.glossary_part)
            .ok_or_else(|| Error::Other("glossary part is missing".to_owned()))?;
        let glossary = CT_GlossaryDocument::from_xml(xml)?;
        let range = glossary.body_range(entry.ordinal)?;
        let scope = crate::document::story_namespace_scope_at(xml, range.start)?;
        let body = crate::document::close_content_fragment_namespaces(&xml[range], &scope)?;
        // Replace just the outer wrapper. Every body attribute and namespace
        // remains on the source document body for package-authoritative capture.
        candidate.glossary_comment_owner()?;
        let wrapped = glossary_body_as_document(&body).map_err(Error::from)?;
        let wrapped = Document::omit_comment_markers(&wrapped)?;
        candidate.omit_glossary_fragment_note_comments(&entry.glossary_part, &wrapped)?;
        let content = crate::document::package_authoritative_body_fragment(
            &wrapped,
            false,
            &std::collections::BTreeMap::new(),
        )?;
        DocumentFragment::from_part_content(&candidate, &entry.glossary_part, content)
    }

    /// Insert an entry through the shared dependency import transaction.
    pub fn insert_building_block(
        &mut self,
        destination: &ContentLocation,
        entry: &BuildingBlockInfo,
        policy: FragmentConflictPolicy,
    ) -> Result<()> {
        let fragment = self.building_block_fragment(entry)?;
        self.import_fragment(destination, &fragment, policy)
    }
}

fn validate_relationship_free_body(content: &[u8]) -> Result<()> {
    crate::document::validate_fragment_block_content(content)?;
    let wrapped = format!(
        "<w:body xmlns:w=\"{}\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">",
        rdocx_oxml::namespace::W_NS
    );
    let mut xml = wrapped.into_bytes();
    xml.extend_from_slice(content);
    xml.extend_from_slice(b"</w:body>");
    if !crate::field::relationship_ids_in_xml(&xml)?.is_empty() {
        return Err(Error::Other(
            "building block relationships require an owned document fragment".to_owned(),
        ));
    }
    let mut reader = quick_xml::reader::NsReader::from_reader(xml.as_slice());
    let mut buffer = Vec::new();
    loop {
        let (namespace, event) = reader
            .read_resolved_event_into(&mut buffer)
            .map_err(rdocx_oxml::error::OxmlError::from)?;
        if let quick_xml::events::Event::Start(element) | quick_xml::events::Event::Empty(element) =
            event
        {
            if matches!(namespace, quick_xml::name::ResolveResult::Bound(namespace) if namespace.as_ref() == rdocx_oxml::namespace::W_NS.as_bytes())
                && matches!(
                    element.local_name().as_ref(),
                    b"footnoteReference"
                        | b"endnoteReference"
                        | b"commentReference"
                        | b"commentRangeStart"
                        | b"commentRangeEnd"
                        | b"sectPr"
                        | b"pStyle"
                        | b"rStyle"
                        | b"tblStyle"
                        | b"numId"
                        | b"dataBinding"
                )
            {
                return Err(Error::Other(
                    "building block companion references require an owned document fragment"
                        .to_owned(),
                ));
            }
        } else if matches!(event, quick_xml::events::Event::Eof) {
            break;
        }
        buffer.clear();
    }
    Ok(())
}

fn glossary_body_as_document(body: &[u8]) -> rdocx_oxml::error::Result<Vec<u8>> {
    let mut reader = quick_xml::Reader::from_reader(body);
    let mut buffer = Vec::new();
    let scope = crate::document::story_namespace_scope_at(body, 0)
        .map_err(|error| rdocx_oxml::error::OxmlError::InvalidValue(error.to_string()))?;
    let prefix = (0u64..)
        .map(|index| format!("f277{index}"))
        .find(|prefix| !scope.contains_key(prefix))
        .ok_or_else(|| {
            rdocx_oxml::error::OxmlError::InvalidValue("wrapper prefix exhausted".to_owned())
        })?;
    let document_name = format!("{prefix}:document");
    let body_name = format!("{prefix}:body");
    let declaration = format!("xmlns:{prefix}");
    let mut writer = quick_xml::Writer::new(Vec::new());
    writer.write_event(quick_xml::events::Event::Start(
        quick_xml::events::BytesStart::new(&document_name)
            .with_attributes([(declaration.as_str(), rdocx_oxml::namespace::W_NS)]),
    ))?;
    loop {
        match reader.read_event_into(&mut buffer)? {
            quick_xml::events::Event::Start(element) => {
                let mut start = quick_xml::events::BytesStart::new(&body_name);
                for attribute in element.attributes() {
                    let attribute = attribute?;
                    start.push_attribute(attribute);
                }
                writer.write_event(quick_xml::events::Event::Start(start))?;
                let begin = reader.buffer_position() as usize;
                let end = body.iter().rposition(|byte| *byte == b'<').ok_or_else(|| {
                    rdocx_oxml::error::OxmlError::InvalidValue(
                        "building block body end is missing".to_owned(),
                    )
                })?;
                writer.get_mut().extend_from_slice(&body[begin..end]);
                writer.write_event(quick_xml::events::Event::End(
                    quick_xml::events::BytesEnd::new(&body_name),
                ))?;
                break;
            }
            quick_xml::events::Event::Empty(_) => {
                return Err(rdocx_oxml::error::OxmlError::InvalidValue(
                    "cannot insert an empty building block".to_owned(),
                ));
            }
            quick_xml::events::Event::Eof => {
                return Err(rdocx_oxml::error::OxmlError::InvalidValue(
                    "building block body is missing".to_owned(),
                ));
            }
            _ => {}
        }
    }
    writer.write_event(quick_xml::events::Event::End(
        quick_xml::events::BytesEnd::new(&document_name),
    ))?;
    Ok(writer.into_inner())
}

// Changed properties preserve their existing wrappers and unsupported siblings.
pub(crate) fn bind_placeholder_xml(xml: &[u8], entry: &BuildingBlockInfo) -> Result<Vec<u8>> {
    let properties = word_child(xml, "sdtPr")?.unwrap_or_else(|| {
        format!(
            r#"<w:sdtPr xmlns:w="{}"></w:sdtPr>"#,
            rdocx_oxml::namespace::W_NS
        )
        .into_bytes()
    });
    let placeholder = word_child(&properties, "placeholder")?.unwrap_or_else(|| {
        format!(
            r#"<w:placeholder xmlns:w="{}"></w:placeholder>"#,
            rdocx_oxml::namespace::W_NS
        )
        .into_bytes()
    });
    let placeholder = set_word_children(
        &placeholder,
        &[(
            "docPart",
            updated_word_value(&placeholder, "docPart", &entry.block.name)?,
        )],
    )?;
    let mut updates = vec![("placeholder", placeholder)];
    for kind in ["docPartObj", "docPartList"] {
        if let Some(selection) = word_child(&properties, kind)? {
            let mut selectors = vec![(
                "docPartGallery",
                updated_word_value(&selection, "docPartGallery", "placeholder")?,
            )];
            if let Some(category) = &entry.block.category {
                selectors.push((
                    "docPartCategory",
                    updated_word_value(&selection, "docPartCategory", category)?,
                ));
            }
            updates.push((kind, set_word_children(&selection, &selectors)?));
        }
    }
    let properties = set_word_children(&properties, &updates)?;
    set_word_children(xml, &[("sdtPr", properties)])
}

fn word_value(local: &str, value: &str) -> rdocx_oxml::error::Result<Vec<u8>> {
    let mut writer = quick_xml::Writer::new(Vec::new());
    let mut element = quick_xml::events::BytesStart::new(format!("w:{local}"));
    element.push_attribute(("xmlns:w", rdocx_oxml::namespace::W_NS));
    element.push_attribute(("w:val", value));
    writer.write_event(quick_xml::events::Event::Empty(element))?;
    Ok(writer.into_inner())
}

fn updated_word_value(parent: &[u8], local: &str, value: &str) -> Result<Vec<u8>> {
    let Some(xml) = word_child(parent, local)? else {
        return Ok(word_value(local, value)?);
    };
    let mut reader = quick_xml::reader::NsReader::from_reader(xml.as_slice());
    let mut buffer = Vec::new();
    let event = reader
        .read_event_into(&mut buffer)
        .map_err(rdocx_oxml::error::OxmlError::from)?;
    let (element, empty) = match event {
        quick_xml::events::Event::Start(element) => (element, false),
        quick_xml::events::Event::Empty(element) => (element, true),
        _ => return Err(Error::Other("invalid document-part value root".to_owned())),
    };
    let name = String::from_utf8_lossy(element.name().as_ref()).into_owned();
    let mut replacement = quick_xml::events::BytesStart::new(name);
    let mut seen = false;
    for attribute in element.attributes() {
        let attribute = attribute.map_err(rdocx_oxml::error::OxmlError::from)?;
        let (namespace, name) = reader.resolver().resolve_attribute(attribute.key);
        if matches!(namespace, quick_xml::name::ResolveResult::Bound(namespace) if namespace.as_ref() == rdocx_oxml::namespace::W_NS.as_bytes())
            && name.as_ref() == b"val"
        {
            if seen {
                return Err(Error::Other("duplicate document-part value".to_owned()));
            }
            let key = std::str::from_utf8(attribute.key.as_ref())
                .map_err(|error| Error::Other(error.to_string()))?;
            replacement.push_attribute((key, value));
            seen = true;
        } else {
            replacement.push_attribute(attribute);
        }
    }
    if !seen {
        return Err(Error::Other(
            "document-part selection value is missing".to_owned(),
        ));
    }
    let opening_end = reader.buffer_position() as usize;
    let mut writer = quick_xml::Writer::new(Vec::new());
    writer
        .write_event(if empty {
            quick_xml::events::Event::Empty(replacement)
        } else {
            quick_xml::events::Event::Start(replacement)
        })
        .map_err(rdocx_oxml::error::OxmlError::from)?;
    writer.get_mut().extend_from_slice(&xml[opening_end..]);
    Ok(writer.into_inner())
}

struct ControlChildren {
    children: Vec<(String, std::ops::Range<usize>)>,
    opening: usize,
    closing: usize,
    empty: bool,
}

fn word_children(xml: &[u8]) -> rdocx_oxml::error::Result<ControlChildren> {
    let mut reader = quick_xml::reader::NsReader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut depth = 0usize;
    let mut pending = None;
    let mut children = Vec::new();
    let mut opening = 0;
    loop {
        let before = reader.buffer_position() as usize;
        let (namespace, event) = reader.read_resolved_event_into(&mut buffer)?;
        let word = matches!(namespace, quick_xml::name::ResolveResult::Bound(namespace) if namespace.as_ref() == rdocx_oxml::namespace::W_NS.as_bytes());
        match event {
            quick_xml::events::Event::Start(element) => {
                if depth == 0 {
                    opening = reader.buffer_position() as usize;
                }
                if depth == 1 && word {
                    pending = Some((
                        String::from_utf8_lossy(element.local_name().as_ref()).into_owned(),
                        before,
                    ));
                }
                depth += 1;
            }
            quick_xml::events::Event::Empty(element) => {
                if depth == 0 {
                    return Ok(ControlChildren {
                        children,
                        opening: reader.buffer_position() as usize,
                        closing: before,
                        empty: true,
                    });
                }
                if depth == 1 && word {
                    children.push((
                        String::from_utf8_lossy(element.local_name().as_ref()).into_owned(),
                        before..reader.buffer_position() as usize,
                    ));
                }
            }
            quick_xml::events::Event::End(_) => {
                if depth == 2
                    && let Some((local, start)) = pending.take()
                {
                    children.push((local, start..reader.buffer_position() as usize));
                }
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Ok(ControlChildren {
                        children,
                        opening,
                        closing: before,
                        empty: false,
                    });
                }
            }
            quick_xml::events::Event::Eof => {
                return Err(rdocx_oxml::error::OxmlError::MissingElement(
                    "property root".to_owned(),
                ));
            }
            _ => {}
        }
        buffer.clear();
    }
}

fn word_child(xml: &[u8], local: &str) -> Result<Option<Vec<u8>>> {
    let ControlChildren { children, .. } = word_children(xml)?;
    let matches = children
        .iter()
        .filter(|(name, _)| name == local)
        .collect::<Vec<_>>();
    if matches.len() > 1 {
        return Err(Error::Other(format!(
            "duplicate w:{local} in content control"
        )));
    }
    matches
        .first()
        .map(|(_, range)| {
            let scope = crate::document::story_namespace_scope_at(xml, range.start)?;
            crate::document::close_content_fragment_namespaces(&xml[range.clone()], &scope)
        })
        .transpose()
}

fn set_word_children(xml: &[u8], updates: &[(&str, Vec<u8>)]) -> Result<Vec<u8>> {
    let ControlChildren {
        children,
        opening,
        closing,
        empty,
    } = word_children(xml)?;
    if empty {
        let mut reader = quick_xml::Reader::from_reader(xml);
        let mut buffer = Vec::new();
        let quick_xml::events::Event::Empty(element) = reader
            .read_event_into(&mut buffer)
            .map_err(rdocx_oxml::error::OxmlError::from)?
        else {
            return Err(Error::Other("invalid empty property wrapper".to_owned()));
        };
        let name = String::from_utf8_lossy(element.name().as_ref()).into_owned();
        let mut output = xml[..opening].to_vec();
        let slash = output
            .iter()
            .rposition(|byte| *byte == b'/')
            .ok_or_else(|| Error::Other("empty property slash missing".to_owned()))?;
        output.remove(slash);
        for (_, replacement) in updates {
            output.extend_from_slice(replacement);
        }
        output.extend_from_slice(format!("</{name}>").as_bytes());
        return Ok(output);
    }
    let mut edits = Vec::new();
    for (local, replacement) in updates {
        let matches = children
            .iter()
            .filter(|(name, _)| name == local)
            .collect::<Vec<_>>();
        if matches.len() > 1 {
            return Err(Error::Other(format!(
                "duplicate w:{local} in content control"
            )));
        }
        let range = matches.first().map_or_else(
            || {
                // sdtPr must precede sdtEndPr and sdtContent.
                if *local == "sdtPr" {
                    opening..opening
                } else {
                    let order = |name: &str| match name {
                        "docPartGallery" => 0,
                        "docPartCategory" => 1,
                        "docPartUnique" => 2,
                        _ => u8::MAX,
                    };
                    let position = children
                        .iter()
                        .find(|(name, _)| order(name) > order(local) && order(name) != u8::MAX)
                        .map_or(closing, |(_, range)| range.start);
                    position..position
                }
            },
            |(_, range)| range.clone(),
        );
        edits.push((range, replacement));
    }
    edits.sort_by_key(|(range, _)| range.start);
    let mut output = xml.to_vec();
    for (range, replacement) in edits.into_iter().rev() {
        output.splice(range, replacement.iter().copied());
    }
    Ok(output)
}
