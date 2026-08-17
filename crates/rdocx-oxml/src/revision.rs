//! Tracked revision metadata and read-only content projections.

use quick_xml::events::Event;
use quick_xml::{Reader, XmlVersion};

use crate::content_control::{CT_Sdt, SdtContent};
use crate::document::{BodyContent, CT_Document, CT_SectPr};
use crate::numbering::{parse_scoped_ppr, parse_scoped_rpr, word_prefixes_at};
use crate::properties::{CT_PPr, CT_RPr, is_word_element};
use crate::raw_xml::capture_empty_element;
use crate::table::{CT_Row, CT_Tbl, CT_TblPr, CT_Tc, CellContent};
use crate::text::{CT_P, CT_R, hyperlink_revision_index};

/// The modeled tracked-change element kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevisionKind {
    Insertion,
    Deletion,
    MoveFrom,
    MoveTo,
    RunPropertyChange,
    ParagraphPropertyChange,
    TablePropertyChange,
    SectionPropertyChange,
}

/// The typed content projected from a preserved revision subtree.
#[derive(Debug, Clone, PartialEq)]
pub enum RevisionContent {
    Runs(Vec<CT_R>),
    Marker,
    PriorRunProperties(Box<CT_RPr>),
    PriorParagraphProperties(Box<CT_PPr>),
    PriorTableProperties(Box<CT_TblPr>),
    PriorSectionProperties(Box<CT_SectPr>),
}

/// A tracked revision whose captured subtree remains its serialization source.
#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub struct CT_Revision {
    kind: RevisionKind,
    id: i32,
    author: String,
    timestamp: Option<String>,
    raw_xml: Vec<u8>,
    content: RevisionContent,
    nested_revisions: Vec<(usize, CT_Revision)>,
}

impl CT_Revision {
    pub(crate) fn from_raw(raw_xml: Vec<u8>, word_prefixes: &[String]) -> Option<Self> {
        Self::try_from_raw(raw_xml, word_prefixes).ok()
    }

    fn try_from_raw(raw_xml: Vec<u8>, word_prefixes: &[String]) -> crate::Result<Self> {
        let mut reader = Reader::from_reader(raw_xml.as_slice());
        reader.config_mut().trim_text(false);
        let mut buffer = Vec::new();
        let (kind, prefixes) = loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) => {
                    let prefixes = word_prefixes_at(&start, word_prefixes)?;
                    let kind =
                        revision_kind(start.name().as_ref(), &prefixes).ok_or_else(|| {
                            crate::OxmlError::InvalidValue(
                                "unsupported revision element".to_owned(),
                            )
                        })?;
                    let id = required_word_attribute(&start, b"id", &prefixes)?.parse()?;
                    let author = required_word_attribute(&start, b"author", &prefixes)?;
                    let timestamp = optional_word_attribute(&start, b"date", &prefixes)?;
                    break ((kind, id, author, timestamp), prefixes);
                }
                Event::Empty(start) => {
                    let prefixes = word_prefixes_at(&start, word_prefixes)?;
                    let kind =
                        revision_kind(start.name().as_ref(), &prefixes).ok_or_else(|| {
                            crate::OxmlError::InvalidValue(
                                "unsupported revision element".to_owned(),
                            )
                        })?;
                    let id = required_word_attribute(&start, b"id", &prefixes)?.parse()?;
                    let author = required_word_attribute(&start, b"author", &prefixes)?;
                    let timestamp = optional_word_attribute(&start, b"date", &prefixes)?;
                    return Ok(Self {
                        kind,
                        id,
                        author,
                        timestamp,
                        raw_xml,
                        content: RevisionContent::Marker,
                        nested_revisions: Vec::new(),
                    });
                }
                Event::Eof => {
                    return Err(crate::OxmlError::MissingElement(
                        "revision element".to_owned(),
                    ));
                }
                _ => {}
            }
            buffer.clear();
        };
        let (kind, id, author, timestamp) = kind;
        let (content, nested_revisions) = parse_content(&mut reader, kind, &prefixes)?;
        Ok(Self {
            kind,
            id,
            author,
            timestamp,
            raw_xml,
            content,
            nested_revisions,
        })
    }

    pub fn kind(&self) -> RevisionKind {
        self.kind
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn author(&self) -> &str {
        &self.author
    }

    pub fn timestamp(&self) -> Option<&str> {
        self.timestamp.as_deref()
    }

    pub fn content(&self) -> &RevisionContent {
        &self.content
    }

    pub(crate) fn write_xml<W: std::io::Write>(
        &self,
        writer: &mut quick_xml::Writer<W>,
    ) -> crate::Result<()> {
        writer.get_mut().write_all(&self.raw_xml)?;
        Ok(())
    }
}

impl CT_Document {
    /// Return every valid modeled main-document revision in document order.
    pub fn revisions(&self) -> Vec<&CT_Revision> {
        let mut revisions = Vec::new();
        for content in &self.body.content {
            collect_body_content(content, &mut revisions);
        }
        if let Some(section) = &self.body.sect_pr {
            collect_section_properties(section, &mut revisions);
        }
        revisions
    }
}

fn collect_body_content<'a>(content: &'a BodyContent, revisions: &mut Vec<&'a CT_Revision>) {
    match content {
        BodyContent::Paragraph(paragraph) => collect_paragraph(paragraph, revisions),
        BodyContent::Table(table) => collect_table(table, revisions),
        BodyContent::ContentControl(control) => collect_control(control, revisions),
        BodyContent::RawXml(_) => {}
    }
}

fn collect_paragraph<'a>(paragraph: &'a CT_P, revisions: &mut Vec<&'a CT_Revision>) {
    if let Some(properties) = &paragraph.properties {
        if let Some(revision) = &properties.numbering_revision {
            revisions.push(revision);
        }
        if let Some(run_properties) = &properties.rpr {
            collect_run_properties(run_properties, revisions);
        }
        if let Some(section) = &properties.sect_pr {
            collect_section_properties(section, revisions);
        }
        if let Some(change) = &properties.change {
            revisions.push(change);
        }
    }

    for boundary in 0..=paragraph.runs.len() {
        let raw_count = paragraph
            .extra_xml
            .iter()
            .filter(|(at, _)| *at == boundary)
            .count();
        for raw_index in 0..=raw_count {
            for (_, _, _, control) in
                paragraph
                    .content_controls
                    .iter()
                    .filter(|(at, raw_before, _, _)| {
                        *at == boundary && (*raw_before).min(raw_count) == raw_index
                    })
            {
                collect_control(control, revisions);
            }
            for (_, _, revision) in paragraph.revisions.iter().filter(|(at, raw_before, _)| {
                *at == boundary
                    && hyperlink_revision_index(*raw_before).is_none()
                    && (*raw_before).min(raw_count) == raw_index
            }) {
                collect_revision(revision, revisions);
            }
        }
        for (_, _, revision) in paragraph
            .revisions
            .iter()
            .filter(|(at, slot, _)| *at == boundary && hyperlink_revision_index(*slot).is_some())
        {
            collect_revision(revision, revisions);
        }
        if let Some(run) = paragraph.runs.get(boundary) {
            collect_run(run, revisions);
        }
    }
}

fn collect_run<'a>(run: &'a CT_R, revisions: &mut Vec<&'a CT_Revision>) {
    if let Some(properties) = &run.properties {
        collect_run_properties(properties, revisions);
    }
}

fn collect_run_properties<'a>(properties: &'a CT_RPr, revisions: &mut Vec<&'a CT_Revision>) {
    revisions.extend(properties.revision_markers.iter());
    if let Some(change) = &properties.change {
        revisions.push(change);
    }
}

fn collect_section_properties<'a>(properties: &'a CT_SectPr, revisions: &mut Vec<&'a CT_Revision>) {
    if let Some(change) = &properties.change {
        revisions.push(change);
    }
}

fn collect_table<'a>(table: &'a CT_Tbl, revisions: &mut Vec<&'a CT_Revision>) {
    if let Some(change) = table
        .properties
        .as_ref()
        .and_then(|properties| properties.change.as_ref())
    {
        revisions.push(change);
    }
    for boundary in 0..=table.rows.len() {
        collect_controls_at_table_boundary(table, boundary, revisions);
        if let Some(row) = table.rows.get(boundary) {
            collect_row(row, revisions);
        }
    }
}

fn collect_controls_at_table_boundary<'a>(
    table: &'a CT_Tbl,
    boundary: usize,
    revisions: &mut Vec<&'a CT_Revision>,
) {
    let raw_count = table
        .extra_xml
        .iter()
        .filter(|(at, _)| *at == boundary)
        .count();
    for raw_index in 0..=raw_count {
        for (_, _, control) in table.content_controls.iter().filter(|(at, raw_before, _)| {
            *at == boundary && (*raw_before).min(raw_count) == raw_index
        }) {
            collect_control(control, revisions);
        }
    }
}

fn collect_row<'a>(row: &'a CT_Row, revisions: &mut Vec<&'a CT_Revision>) {
    if let Some(properties) = &row.properties {
        revisions.extend(properties.revision_markers.iter());
    }
    for boundary in 0..=row.cells.len() {
        let raw_count = row
            .extra_xml
            .iter()
            .filter(|(at, _)| *at == boundary)
            .count();
        for raw_index in 0..=raw_count {
            for (_, _, control) in row.content_controls.iter().filter(|(at, raw_before, _)| {
                *at == boundary && (*raw_before).min(raw_count) == raw_index
            }) {
                collect_control(control, revisions);
            }
        }
        if let Some(cell) = row.cells.get(boundary) {
            collect_cell(cell, revisions);
        }
    }
}

fn collect_cell<'a>(cell: &'a CT_Tc, revisions: &mut Vec<&'a CT_Revision>) {
    for content in &cell.content {
        match content {
            CellContent::Paragraph(paragraph) => collect_paragraph(paragraph, revisions),
            CellContent::Table(table) => collect_table(table, revisions),
            CellContent::ContentControl(control) => collect_control(control, revisions),
        }
    }
}

fn collect_control<'a>(control: &'a CT_Sdt, revisions: &mut Vec<&'a CT_Revision>) {
    for (content_index, content) in control.content.iter().enumerate() {
        for (_, revision) in control
            .revisions
            .iter()
            .filter(|(at, _)| *at == content_index)
        {
            collect_revision(revision, revisions);
        }
        match content {
            SdtContent::Paragraph(paragraph) => collect_paragraph(paragraph, revisions),
            SdtContent::Table(table) => collect_table(table, revisions),
            SdtContent::Row(row) => collect_row(row, revisions),
            SdtContent::Cell(cell) => collect_cell(cell, revisions),
            SdtContent::Run(run) => collect_run(run, revisions),
            SdtContent::ContentControl(control) => collect_control(control, revisions),
            SdtContent::RawXml(_) => {}
        }
    }
}

fn collect_revision<'a>(revision: &'a CT_Revision, revisions: &mut Vec<&'a CT_Revision>) {
    revisions.push(revision);
    if let RevisionContent::Runs(runs) = revision.content() {
        for boundary in 0..=runs.len() {
            for (_, nested) in revision
                .nested_revisions
                .iter()
                .filter(|(at, _)| *at == boundary)
            {
                collect_revision(nested, revisions);
            }
            if let Some(run) = runs.get(boundary) {
                collect_run(run, revisions);
            }
        }
    } else {
        for (_, nested) in &revision.nested_revisions {
            collect_revision(nested, revisions);
        }
    }
}

fn revision_kind(name: &[u8], prefixes: &[String]) -> Option<RevisionKind> {
    [
        (b"ins".as_slice(), RevisionKind::Insertion),
        (b"del".as_slice(), RevisionKind::Deletion),
        (b"moveFrom".as_slice(), RevisionKind::MoveFrom),
        (b"moveTo".as_slice(), RevisionKind::MoveTo),
        (b"rPrChange".as_slice(), RevisionKind::RunPropertyChange),
        (
            b"pPrChange".as_slice(),
            RevisionKind::ParagraphPropertyChange,
        ),
        (b"tblPrChange".as_slice(), RevisionKind::TablePropertyChange),
        (
            b"sectPrChange".as_slice(),
            RevisionKind::SectionPropertyChange,
        ),
    ]
    .into_iter()
    .find_map(|(local, kind)| is_word_element(name, local, prefixes).then_some(kind))
}

fn parse_content(
    reader: &mut Reader<&[u8]>,
    kind: RevisionKind,
    word_prefixes: &[String],
) -> crate::Result<(RevisionContent, Vec<(usize, CT_Revision)>)> {
    let mut runs = Vec::new();
    let mut nested_revisions = Vec::new();
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) => {
                let prefixes = word_prefixes_at(&start, word_prefixes)?;
                let name = start.name();
                if is_word_element(name.as_ref(), b"r", &prefixes) {
                    runs.push(CT_R::from_xml_with_prefixes(reader, &prefixes)?);
                } else if revision_kind(name.as_ref(), &prefixes).is_some() {
                    let raw = crate::raw_xml::capture_element(reader, &start)?;
                    if let Some(revision) = CT_Revision::from_raw(raw, &prefixes) {
                        nested_revisions.push((runs.len(), revision));
                    }
                } else if is_word_element(name.as_ref(), b"hyperlink", &prefixes) {
                    parse_run_content_container(
                        reader,
                        b"hyperlink",
                        &prefixes,
                        &mut runs,
                        &mut nested_revisions,
                    )?;
                } else if is_word_element(name.as_ref(), b"rPr", &prefixes)
                    && kind == RevisionKind::RunPropertyChange
                {
                    let raw = crate::raw_xml::capture_element(reader, &start)?;
                    return Ok((
                        RevisionContent::PriorRunProperties(Box::new(parse_scoped_rpr(
                            &raw, &prefixes,
                        )?)),
                        nested_revisions,
                    ));
                } else if is_word_element(name.as_ref(), b"pPr", &prefixes)
                    && kind == RevisionKind::ParagraphPropertyChange
                {
                    let raw = crate::raw_xml::capture_element(reader, &start)?;
                    return Ok((
                        RevisionContent::PriorParagraphProperties(Box::new(parse_scoped_ppr(
                            &raw, &prefixes,
                        )?)),
                        nested_revisions,
                    ));
                } else if is_word_element(name.as_ref(), b"tblPr", &prefixes)
                    && kind == RevisionKind::TablePropertyChange
                {
                    return Ok((
                        RevisionContent::PriorTableProperties(Box::new(
                            CT_TblPr::from_xml_with_prefixes(reader, &prefixes)?,
                        )),
                        nested_revisions,
                    ));
                } else if is_word_element(name.as_ref(), b"sectPr", &prefixes)
                    && kind == RevisionKind::SectionPropertyChange
                {
                    return Ok((
                        RevisionContent::PriorSectionProperties(Box::new(
                            CT_SectPr::from_xml_with_prefixes(reader, &prefixes)?,
                        )),
                        nested_revisions,
                    ));
                } else {
                    reader.read_to_end_into(name, &mut Vec::new())?;
                }
            }
            Event::Empty(start) => {
                let prefixes = word_prefixes_at(&start, word_prefixes)?;
                if revision_kind(start.name().as_ref(), &prefixes).is_some() {
                    let raw = capture_empty_element(&start)?;
                    if let Some(revision) = CT_Revision::from_raw(raw, &prefixes) {
                        nested_revisions.push((runs.len(), revision));
                    }
                }
            }
            Event::End(_) | Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if runs.is_empty() {
        Ok((RevisionContent::Marker, nested_revisions))
    } else {
        Ok((RevisionContent::Runs(runs), nested_revisions))
    }
}

fn parse_run_content_container(
    reader: &mut Reader<&[u8]>,
    closing_local: &[u8],
    word_prefixes: &[String],
    runs: &mut Vec<CT_R>,
    nested_revisions: &mut Vec<(usize, CT_Revision)>,
) -> crate::Result<()> {
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) => {
                let prefixes = word_prefixes_at(&start, word_prefixes)?;
                let name = start.name();
                if is_word_element(name.as_ref(), b"r", &prefixes) {
                    runs.push(CT_R::from_xml_with_prefixes(reader, &prefixes)?);
                } else if revision_kind(name.as_ref(), &prefixes).is_some() {
                    let raw = crate::raw_xml::capture_element(reader, &start)?;
                    if let Some(revision) = CT_Revision::from_raw(raw, &prefixes) {
                        nested_revisions.push((runs.len(), revision));
                    }
                } else if is_word_element(name.as_ref(), b"hyperlink", &prefixes) {
                    parse_run_content_container(
                        reader,
                        b"hyperlink",
                        &prefixes,
                        runs,
                        nested_revisions,
                    )?;
                } else {
                    reader.read_to_end_into(name, &mut Vec::new())?;
                }
            }
            Event::Empty(start) => {
                let prefixes = word_prefixes_at(&start, word_prefixes)?;
                if revision_kind(start.name().as_ref(), &prefixes).is_some() {
                    let raw = capture_empty_element(&start)?;
                    if let Some(revision) = CT_Revision::from_raw(raw, &prefixes) {
                        nested_revisions.push((runs.len(), revision));
                    }
                }
            }
            Event::End(end)
                if is_word_element(end.name().as_ref(), closing_local, word_prefixes) =>
            {
                break;
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(())
}

fn required_word_attribute(
    start: &quick_xml::events::BytesStart<'_>,
    local: &[u8],
    prefixes: &[String],
) -> crate::Result<String> {
    optional_word_attribute(start, local, prefixes)?.ok_or_else(|| {
        crate::OxmlError::MissingElement(format!("w:{} attribute", String::from_utf8_lossy(local)))
    })
}

fn optional_word_attribute(
    start: &quick_xml::events::BytesStart<'_>,
    local: &[u8],
    prefixes: &[String],
) -> crate::Result<Option<String>> {
    for attribute in start.attributes() {
        let attribute = attribute?;
        let key = attribute.key.as_ref();
        let Some(separator) = key.iter().position(|byte| *byte == b':') else {
            continue;
        };
        if key.get(separator + 1..) == Some(local)
            && prefixes
                .iter()
                .any(|prefix| prefix.as_bytes() == &key[..separator])
        {
            return Ok(Some(
                attribute
                    .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())?
                    .into_owned(),
            ));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::namespace::W_NS;
    use crate::text::RunContent;

    #[test]
    fn revision_attributes_are_prefix_tolerant_and_namespace_checked() {
        let raw = format!(
            r#"<word:ins xmlns:word="{W_NS}" word:id="7" word:author="Ada"><word:r><word:t>added</word:t></word:r></word:ins>"#
        )
        .into_bytes();
        let revision = CT_Revision::from_raw(raw, &["word".to_owned()])
            .expect("aliased WordprocessingML revision should be projected");
        assert_eq!(revision.id(), 7);
        assert_eq!(revision.author(), "Ada");
        assert_eq!(revision.timestamp(), None);
        assert_eq!(revision.kind(), RevisionKind::Insertion);

        let escaped = format!(
            r#"<word:ins xmlns:word="{W_NS}" word:id="8" word:author="A &amp; B" word:date="2026-08-17T09:00:00+01:00 &amp; later"/>"#
        )
        .into_bytes();
        let escaped = CT_Revision::from_raw(escaped, &["word".to_owned()])
            .expect("escaped revision metadata should parse");
        assert_eq!(escaped.author(), "A & B");
        assert_eq!(
            escaped.timestamp(),
            Some("2026-08-17T09:00:00+01:00 & later")
        );

        let foreign = br#"<x:ins xmlns:x="urn:not-word" x:id="9" x:author="Eve"/>"#.to_vec();
        assert!(CT_Revision::from_raw(foreign, &["word".to_owned()]).is_none());

        let missing_author = format!(r#"<word:del xmlns:word="{W_NS}" word:id="9"/>"#).into_bytes();
        assert!(CT_Revision::from_raw(missing_author, &["word".to_owned()]).is_none());

        let document_xml = format!(
            r#"<word:document xmlns:word="{W_NS}" xmlns:x="urn:not-word"><word:body><word:p><word:pPr><word:pPrChange word:id="10" word:author="Gia"><word:pPr><word:jc word:val="right"/></word:pPr></word:pPrChange><x:pPrChange x:id="11" x:author="No"/><word:pPrChange word:id="12"/></word:pPr></word:p></word:body></word:document>"#
        );
        let document = CT_Document::from_xml(document_xml.as_bytes()).expect("document parses");
        assert_eq!(document.revisions().len(), 1);
        assert_eq!(document.revisions()[0].id(), 10);
        let output = String::from_utf8(document.to_xml().expect("document writes")).unwrap();
        assert!(output.contains(r#"<word:pPrChange word:id="10" word:author="Gia">"#));
        assert!(output.contains(r#"<x:pPrChange x:id="11" x:author="No"/>"#));
        assert!(output.contains(r#"<word:pPrChange word:id="12"/>"#));

        let collision_xml = format!(
            r#"<word:document xmlns:word="{W_NS}" xmlns:x="urn:not-word"><word:body>
<word:tbl><word:tblPr><word:tblPrChange word:id="20" word:author="Tab"><word:tblPr><x:jc x:val="center"/><word:jc x:val="right"/><word:tblBorders><x:top x:val="single" word:sz="invalid"/><word:bottom x:val="single"/></word:tblBorders><word:tblCellMar><x:top x:w="120"/><word:bottom x:w="240"/></word:tblCellMar></word:tblPr></word:tblPrChange></word:tblPr><word:tblGrid/><word:tr><word:tc><word:p/></word:tc></word:tr></word:tbl>
<word:sectPr><word:sectPrChange word:id="21" word:author="Sec"><word:sectPr><x:titlePg/><word:pgSz x:w="12240" x:h="15840"/><word:pgMar x:producer="not-a-number" word:top="720"/></word:sectPr></word:sectPrChange></word:sectPr>
</word:body></word:document>"#
        );
        let collision = CT_Document::from_xml(collision_xml.as_bytes()).expect("document parses");
        let collision_revisions = collision.revisions();
        assert!(matches!(
            collision_revisions[0].content(),
            RevisionContent::PriorTableProperties(properties)
                if properties.jc.is_none()
                    && properties
                        .borders
                        .as_ref()
                        .is_some_and(|borders| borders.top.is_none()
                            && borders.bottom.as_ref().is_some_and(|edge| {
                                edge.val == crate::shared::ST_Border::None
                            }))
                    && properties
                        .cell_margin
                        .as_ref()
                        .is_some_and(|margin| margin.top.is_none() && margin.bottom.is_none())
        ));
        assert!(matches!(
            collision_revisions[1].content(),
            RevisionContent::PriorSectionProperties(properties)
                if properties.title_pg.is_none()
                    && properties.page_width.is_none()
                    && properties.page_height.is_none()
                    && properties.margin_top == Some(crate::units::Twips(720))
        ));

        let paragraph_collision_xml = format!(
            r#"<word:document xmlns:word="{W_NS}" xmlns:x="urn:not-word"><word:body><word:p><word:pPr><word:pPrChange word:id="22" word:author="Para"><word:pPr><word:pBdr><x:top x:val="single" word:sz="invalid"/><word:bottom x:val="single"/></word:pBdr></word:pPr></word:pPrChange></word:pPr></word:p></word:body></word:document>"#
        );
        let paragraph_collision =
            CT_Document::from_xml(paragraph_collision_xml.as_bytes()).expect("document parses");
        assert!(matches!(
            paragraph_collision.revisions()[0].content(),
            RevisionContent::PriorParagraphProperties(properties)
                if properties.borders.as_ref().is_some_and(|borders| {
                    borders.top.is_none()
                        && borders.bottom.as_ref().is_some_and(|edge| {
                            edge.val == crate::shared::ST_Border::None
                        })
                })
        ));

        let control_xml = format!(
            r#"<word:document xmlns:word="{W_NS}"><word:body><word:p><word:sdt><word:sdtContent><word:ins word:id="30" word:author="Sdt"><word:r><word:t>inside</word:t></word:r></word:ins></word:sdtContent></word:sdt></word:p></word:body></word:document>"#
        );
        let control = CT_Document::from_xml(control_xml.as_bytes()).expect("document parses");
        assert_eq!(
            control
                .revisions()
                .iter()
                .map(|revision| revision.id())
                .collect::<Vec<_>>(),
            vec![30]
        );
    }

    #[test]
    fn property_changes_write_in_their_schema_final_slots() {
        let xml = format!(
            r#"<w:document xmlns:w="{W_NS}"><w:body>
<w:p><w:pPr><w:jc w:val="center"/><w:pPrChange w:id="1" w:author="Ada"><w:pPr><w:jc w:val="left"/></w:pPr></w:pPrChange></w:pPr>
<w:r><w:rPr><w:b/><w:rPrChange w:id="2" w:author="Ben"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:t>x</w:t></w:r></w:p>
<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/><w:tblPrChange w:id="3" w:author="Cy"><w:tblPr><w:jc w:val="right"/></w:tblPr></w:tblPrChange></w:tblPr><w:tblGrid/><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl>
<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:sectPrChange w:id="4" w:author="Dee"><w:sectPr><w:titlePg/></w:sectPr></w:sectPrChange></w:sectPr>
</w:body></w:document>"#
        );
        let document = CT_Document::from_xml(xml.as_bytes()).expect("document parses");
        let output =
            String::from_utf8(document.to_xml().expect("document writes")).expect("UTF-8 output");

        for change in ["pPrChange", "rPrChange", "tblPrChange", "sectPrChange"] {
            assert_eq!(
                output.matches(&format!("<w:{change} ")).count(),
                1,
                "{output}"
            );
        }
        assert!(
            output.find("<w:jc w:val=\"center\"").unwrap() < output.find("<w:pPrChange ").unwrap()
        );
        assert!(output.find("<w:b").unwrap() < output.find("<w:rPrChange ").unwrap());
        assert!(output.find("<w:tblW").unwrap() < output.find("<w:tblPrChange ").unwrap());
        assert!(output.find("<w:pgSz").unwrap() < output.find("<w:sectPrChange ").unwrap());
    }

    #[test]
    fn revision_elements_round_trip_unchanged_and_report_metadata() {
        let revisions = [
            r#"<w:ins w:id="1" w:author="Ada" w:date="2026-08-01T10:00:00Z"><w:r><w:t>added</w:t></w:r></w:ins>"#,
            r#"<w:del w:id="2" w:author="Ben"><w:r><w:delText xml:space="preserve"> gone </w:delText></w:r></w:del>"#,
            r#"<w:moveFrom w:id="3" w:author="Cy"><w:r><w:t>from</w:t></w:r></w:moveFrom>"#,
            r#"<w:moveTo w:id="4" w:author="Dee"><w:r><w:t>to</w:t></w:r></w:moveTo>"#,
        ];
        let xml = format!(
            r#"<w:document xmlns:w="{W_NS}"><w:body><w:p>{}<w:r><w:rPr><w:ins w:id="5" w:author="Eve"/><w:rPrChange w:id="6" w:author="Fox"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:t>end</w:t></w:r><x:ins xmlns:x="urn:not-word" x:id="99" x:author="No"/></w:p><w:tbl><w:tblPr><w:tblPrChange w:id="7" w:author="Gia"><w:tblPr><w:jc w:val="center"/></w:tblPr></w:tblPrChange></w:tblPr><w:tblGrid/><w:tr><w:trPr><w:del w:id="8" w:author="Hal"/></w:trPr><w:tc><w:p><w:pPr><w:pPrChange w:id="9" w:author="Ivy"><w:pPr><w:jc w:val="right"/></w:pPr></w:pPrChange></w:pPr></w:p></w:tc></w:tr></w:tbl><w:sectPr><w:sectPrChange w:id="10" w:author="Jay"><w:sectPr><w:titlePg/></w:sectPr></w:sectPrChange></w:sectPr></w:body></w:document>"#,
            revisions.concat()
        );
        let document = CT_Document::from_xml(xml.as_bytes()).expect("document parses");
        let reported = document.revisions();
        assert_eq!(
            reported
                .iter()
                .map(|revision| revision.id())
                .collect::<Vec<_>>(),
            (1..=10).collect::<Vec<_>>()
        );
        assert_eq!(reported[0].author(), "Ada");
        assert_eq!(reported[0].timestamp(), Some("2026-08-01T10:00:00Z"));
        assert_eq!(reported[1].timestamp(), None);
        assert_eq!(reported[1].kind(), RevisionKind::Deletion);
        let RevisionContent::Runs(runs) = reported[1].content() else {
            panic!("deletion should project runs");
        };
        assert!(matches!(
            runs[0].content.as_slice(),
            [RunContent::DeletedText(text)] if text.text == " gone " && text.preserve_space
        ));
        assert!(matches!(
            reported[5].content(),
            RevisionContent::PriorRunProperties(properties) if properties.italic == Some(true)
        ));
        assert!(matches!(
            reported[6].content(),
            RevisionContent::PriorTableProperties(properties) if properties.jc.is_some()
        ));
        assert!(matches!(
            reported[8].content(),
            RevisionContent::PriorParagraphProperties(properties) if properties.jc.is_some()
        ));
        assert!(matches!(
            reported[9].content(),
            RevisionContent::PriorSectionProperties(properties) if properties.title_pg == Some(true)
        ));

        let output =
            String::from_utf8(document.to_xml().expect("document writes")).expect("UTF-8 output");
        for raw in revisions {
            assert!(
                output.contains(raw),
                "missing exact subtree {raw} in {output}"
            );
        }
        for raw in [
            r#"<w:rPrChange w:id="6" w:author="Fox"><w:rPr><w:i/></w:rPr></w:rPrChange>"#,
            r#"<w:tblPrChange w:id="7" w:author="Gia"><w:tblPr><w:jc w:val="center"/></w:tblPr></w:tblPrChange>"#,
            r#"<w:pPrChange w:id="9" w:author="Ivy"><w:pPr><w:jc w:val="right"/></w:pPr></w:pPrChange>"#,
            r#"<w:sectPrChange w:id="10" w:author="Jay"><w:sectPr><w:titlePg/></w:sectPr></w:sectPrChange>"#,
        ] {
            assert!(
                output.contains(raw),
                "missing exact subtree {raw} in {output}"
            );
        }
        assert!(output.contains(r#"<x:ins xmlns:x="urn:not-word" x:id="99" x:author="No"/>"#));
        let reopened = CT_Document::from_xml(output.as_bytes()).expect("output reparses");
        assert_eq!(reopened.revisions().len(), 10);
    }

    #[test]
    fn hyperlink_and_nested_content_revisions_round_trip_and_report_in_order() {
        let nested = r#"<w:ins w:id="11" w:author="Ada"><w:del w:id="12" w:author="Ben"><w:r><w:delText>nested</w:delText></w:r></w:del></w:ins>"#;
        let hyperlink_before = r#"<w:hyperlink><w:ins w:id="21" w:author="Cy"><w:r><w:t>before</w:t></w:r></w:ins></w:hyperlink>"#;
        let direct_after =
            r#"<w:del w:id="22" w:author="Dee"><w:r><w:delText>after</w:delText></w:r></w:del>"#;
        let direct_before =
            r#"<w:ins w:id="23" w:author="Eve"><w:r><w:t>before</w:t></w:r></w:ins>"#;
        let hyperlink_after = r#"<w:hyperlink><w:del w:id="24" w:author="Fox"><w:r><w:delText>after</w:delText></w:r></w:del></w:hyperlink>"#;
        let xml = format!(
            r#"<w:document xmlns:w="{W_NS}" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body><w:p><w:hyperlink r:id="rId5"><w:r><w:t xml:space="preserve">before </w:t></w:r>{nested}<w:r><w:t xml:space="preserve"> after</w:t></w:r></w:hyperlink></w:p><w:p>{hyperlink_before}{direct_after}{direct_before}{hyperlink_after}</w:p></w:body></w:document>"#
        );
        let document = CT_Document::from_xml(xml.as_bytes()).expect("document parses");

        assert_eq!(
            document
                .revisions()
                .iter()
                .map(|revision| revision.id())
                .collect::<Vec<_>>(),
            [11, 12, 21, 22, 23, 24]
        );
        let output =
            String::from_utf8(document.to_xml().expect("document writes")).expect("UTF-8 output");
        assert!(output.contains(nested), "missing exact subtree in {output}");
        for raw in [
            hyperlink_before,
            direct_after,
            direct_before,
            hyperlink_after,
        ] {
            assert!(
                output.contains(raw),
                "missing exact subtree {raw} in {output}"
            );
        }
        let positions = [21, 22, 23, 24].map(|id| {
            output
                .find(&format!(r#"w:id="{id}""#))
                .expect("revision id remains present")
        });
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));

        let reopened = CT_Document::from_xml(output.as_bytes()).expect("output reparses");
        assert_eq!(
            reopened
                .revisions()
                .iter()
                .map(|revision| revision.id())
                .collect::<Vec<_>>(),
            [11, 12, 21, 22, 23, 24]
        );
    }
}
