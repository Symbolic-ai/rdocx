//! Deterministic native document comparison and tracked-revision generation.

use std::collections::HashSet;
use std::ops::Range;

use quick_xml::events::Event;
use quick_xml::name::{Namespace, ResolveResult};
use quick_xml::reader::NsReader;
use quick_xml::{Reader, Writer};
use rdocx_oxml::content_control::{CT_Sdt, SdtContent};
use rdocx_oxml::document::{BodyContent, CT_Document};
use rdocx_oxml::namespace::W_NS;
use rdocx_oxml::properties::CT_PPr;
use rdocx_oxml::table::{CT_Row, CT_Tbl, CT_TblPr, CT_Tc, CT_TrPr, CellContent};
use rdocx_oxml::text::{CT_P, CT_R, RunContent};

use crate::revision::validate_revision_timestamp;
use crate::{Document, Error, Result};

type ControlPropertySignature<'a> = Option<(
    Option<&'a str>,
    Option<&'a str>,
    Option<i32>,
    Option<rdocx_oxml::content_control::SdtType>,
    Option<&'a rdocx_oxml::content_control::CT_DataBinding>,
)>;

/// A comparison difference that cannot be represented as a content revision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparisonDiagnostic {
    pub location: String,
    pub message: String,
}

struct Metadata<'a> {
    author: &'a str,
    timestamp: &'a str,
    ids: IdAllocator,
}

struct IdAllocator {
    used: HashSet<i32>,
    next: i32,
}

impl IdAllocator {
    fn new(used: HashSet<i32>) -> Self {
        Self { used, next: 0 }
    }

    fn allocate(&mut self) -> Result<i32> {
        while self.used.contains(&self.next) {
            self.next = self
                .next
                .checked_add(1)
                .ok_or_else(|| Error::Other("comparison revision ids are exhausted".to_owned()))?;
        }
        let id = self.next;
        self.used.insert(id);
        self.next = self.next.saturating_add(1);
        Ok(id)
    }

    fn revision(
        &mut self,
        kind: &str,
        author: &str,
        timestamp: &str,
        inner: &str,
    ) -> Result<String> {
        let id = self.allocate()?;
        let author = quick_xml::escape::escape(author);
        let timestamp = quick_xml::escape::escape(timestamp);
        Ok(format!(
            r#"<w:{kind} w:id="{id}" w:author="{author}" w:date="{timestamp}">{inner}</w:{kind}>"#
        ))
    }

    fn marker(&mut self, kind: &str, author: &str, timestamp: &str) -> Result<String> {
        let id = self.allocate()?;
        let author = quick_xml::escape::escape(author);
        let timestamp = quick_xml::escape::escape(timestamp);
        Ok(format!(
            r#"<w:{kind} w:id="{id}" w:author="{author}" w:date="{timestamp}"/>"#
        ))
    }
}

impl Document {
    /// Compare the modeled main body with `edited` and record content changes.
    pub fn compare(
        &mut self,
        edited: &Document,
        author: &str,
        timestamp: &str,
    ) -> Result<Vec<ComparisonDiagnostic>> {
        validate_revision_timestamp(timestamp)?;
        if !self.revisions().is_empty() || !edited.revisions().is_empty() {
            return Err(Error::Other(
                "document comparison requires inputs without existing modeled revisions".to_owned(),
            ));
        }

        let original_xml = self.document.to_xml()?;
        let edited_xml = edited.document.to_xml()?;
        let mut used_ids = word_ids(&original_xml)?;
        used_ids.extend(word_ids(&edited_xml)?);
        let mut metadata = Metadata {
            author,
            timestamp,
            ids: IdAllocator::new(used_ids),
        };
        let mut diagnostics = Vec::new();
        let tracked_body = compare_body(
            &self.document,
            &edited.document,
            &mut metadata,
            &mut diagnostics,
        )?;
        let tracked_xml = replace_body_inner(&original_xml, &tracked_body)?;
        let tracked = CT_Document::from_xml(tracked_xml.as_bytes())?;
        tracked.to_xml()?;

        let mut candidate = self.clone_for_staging();
        candidate.document = tracked;
        let mut accepted = candidate.clone_for_staging();
        accepted.accept_all()?;
        let accepted_body = normalized_body(&accepted.document);
        let edited_body = normalized_body(&edited.document);
        if accepted_body != edited_body {
            return Err(Error::Other(format!(
                "comparison acceptance does not reproduce the edited body: {accepted_body:?} != {edited_body:?}"
            )));
        }
        let mut rejected = candidate.clone_for_staging();
        rejected.reject_all()?;
        if normalized_body(&rejected.document) != normalized_body(&self.document) {
            return Err(Error::Other(
                "comparison rejection does not reproduce the original body".to_owned(),
            ));
        }

        self.document = candidate.document;
        self.invalidate_layout();
        Ok(diagnostics)
    }
}

fn compare_body(
    original: &CT_Document,
    edited: &CT_Document,
    metadata: &mut Metadata<'_>,
    diagnostics: &mut Vec<ComparisonDiagnostic>,
) -> Result<String> {
    let original_signatures = original
        .body
        .content
        .iter()
        .map(body_signature)
        .collect::<Vec<_>>();
    let edited_signatures = edited
        .body
        .content
        .iter()
        .map(body_signature)
        .collect::<Vec<_>>();
    let aligned = expand_body_alignment(
        align(&original_signatures, &edited_signatures),
        &original.body.content,
        &edited.body.content,
    );
    let mut output: Vec<(bool, String)> = Vec::new();
    for (position, (original_index, edited_index)) in aligned.iter().copied().enumerate() {
        let next_is_paragraph = aligned.get(position + 1).is_some_and(|(left, right)| {
            right
                .and_then(|index| edited.body.content.get(index))
                .or_else(|| left.and_then(|index| original.body.content.get(index)))
                .is_some_and(|content| matches!(content, BodyContent::Paragraph(_)))
        });
        match (original_index, edited_index) {
            (Some(left), Some(right)) => output.push((
                matches!(edited.body.content[right], BodyContent::Paragraph(_)),
                compare_body_content(
                    &original.body.content[left],
                    &edited.body.content[right],
                    &body_location(&edited.body.content[right], right),
                    metadata,
                    diagnostics,
                )?,
            )),
            (Some(left), None) => {
                let content = &original.body.content[left];
                if matches!(content, BodyContent::Paragraph(_)) && !next_is_paragraph {
                    mark_previous_paragraph(&mut output, "del", metadata)?;
                    output.push((true, deleted_paragraph_content(content, metadata)?));
                } else {
                    output.push((
                        matches!(content, BodyContent::Paragraph(_)),
                        deleted_body_content(content, metadata)?,
                    ));
                }
            }
            (None, Some(right)) => {
                let content = &edited.body.content[right];
                if matches!(content, BodyContent::Paragraph(_)) && !next_is_paragraph {
                    mark_previous_paragraph(&mut output, "ins", metadata)?;
                    output.push((true, inserted_paragraph_content(content, metadata)?));
                } else {
                    output.push((
                        matches!(content, BodyContent::Paragraph(_)),
                        inserted_body_content(content, metadata)?,
                    ));
                }
            }
            (None, None) => unreachable!(),
        }
    }
    let mut output = output.into_iter().map(|(_, xml)| xml).collect::<String>();
    output.push_str(&section_xml(original)?);
    Ok(output)
}

fn mark_previous_paragraph(
    output: &mut [(bool, String)],
    kind: &str,
    metadata: &mut Metadata<'_>,
) -> Result<()> {
    let Some((true, paragraph)) = output.last_mut() else {
        return Err(Error::Other(
            "comparison needs an adjacent paragraph for a final paragraph change".to_owned(),
        ));
    };
    let marker = metadata
        .ids
        .marker(kind, metadata.author, metadata.timestamp)?;
    let paragraph_properties = direct_word_element_spans(paragraph, "pPr")?;
    if let Some(properties_span) = paragraph_properties.first() {
        let properties = &paragraph[properties_span.clone()];
        let run_properties = direct_word_element_spans(properties, "rPr")?;
        let updated = if let Some(run_span) = run_properties.first() {
            let run_properties = &properties[run_span.clone()];
            let updated_run = append_word_child(run_properties, "rPr", &marker)?;
            let mut updated = properties.to_owned();
            updated.replace_range(run_span.clone(), &updated_run);
            updated
        } else {
            append_word_child(properties, "pPr", &format!("<w:rPr>{marker}</w:rPr>"))?
        };
        paragraph.replace_range(properties_span.clone(), &updated);
    } else {
        let open = paragraph
            .find('>')
            .ok_or_else(|| Error::Other("paragraph XML has no start".to_owned()))?
            + 1;
        *paragraph = format!(
            "{}<w:pPr><w:rPr>{marker}</w:rPr></w:pPr>{}",
            &paragraph[..open],
            &paragraph[open..]
        );
    }
    Ok(())
}

fn deleted_paragraph_content(content: &BodyContent, metadata: &mut Metadata<'_>) -> Result<String> {
    let BodyContent::Paragraph(paragraph) = content else {
        unreachable!("caller checked paragraph content")
    };
    let mut output = String::from("<w:p>");
    if let Some(properties) = &paragraph.properties {
        output.push_str(&property_xml(properties)?);
    }
    for run in &paragraph.runs {
        let run = deleted_run_xml(run)?;
        output.push_str(&metadata.ids.revision(
            "del",
            metadata.author,
            metadata.timestamp,
            &run,
        )?);
    }
    output.push_str("</w:p>");
    Ok(output)
}

fn inserted_paragraph_content(
    content: &BodyContent,
    metadata: &mut Metadata<'_>,
) -> Result<String> {
    let BodyContent::Paragraph(paragraph) = content else {
        unreachable!("caller checked paragraph content")
    };
    let mut output = String::from("<w:p>");
    if let Some(properties) = &paragraph.properties {
        output.push_str(&property_xml(properties)?);
    }
    for run in &paragraph.runs {
        let run = run_xml(run)?;
        output.push_str(&metadata.ids.revision(
            "ins",
            metadata.author,
            metadata.timestamp,
            &run,
        )?);
    }
    output.push_str("</w:p>");
    Ok(output)
}

fn compare_body_content(
    original: &BodyContent,
    edited: &BodyContent,
    location: &str,
    metadata: &mut Metadata<'_>,
    diagnostics: &mut Vec<ComparisonDiagnostic>,
) -> Result<String> {
    match (original, edited) {
        (BodyContent::Paragraph(left), BodyContent::Paragraph(right)) => {
            compare_paragraph(left, right, location, metadata, diagnostics)
        }
        (BodyContent::Table(left), BodyContent::Table(right)) => {
            compare_table(left, right, location, metadata, diagnostics)
        }
        (BodyContent::ContentControl(left), BodyContent::ContentControl(right)) => {
            compare_control(left, right, location, metadata, diagnostics)
        }
        _ if body_signature(original) == body_signature(edited) => body_content_xml(original),
        _ => Err(Error::Other(format!(
            "comparison cannot replace unlike body structures at {location}"
        ))),
    }
}

fn deleted_body_content(content: &BodyContent, metadata: &mut Metadata<'_>) -> Result<String> {
    match content {
        BodyContent::Paragraph(paragraph) => deleted_paragraph(paragraph, metadata),
        BodyContent::Table(table) => marked_table(table, "del", metadata),
        BodyContent::ContentControl(_) | BodyContent::RawXml(_) => Err(Error::Other(
            "comparison cannot delete an unmatched content-control or opaque body node".to_owned(),
        )),
    }
}

fn inserted_body_content(content: &BodyContent, metadata: &mut Metadata<'_>) -> Result<String> {
    match content {
        BodyContent::Paragraph(paragraph) => inserted_paragraph(paragraph, metadata),
        BodyContent::Table(table) => marked_table(table, "ins", metadata),
        BodyContent::ContentControl(_) | BodyContent::RawXml(_) => Err(Error::Other(
            "comparison cannot insert an unmatched content-control or opaque body node".to_owned(),
        )),
    }
}

fn compare_paragraph(
    original: &CT_P,
    edited: &CT_P,
    location: &str,
    metadata: &mut Metadata<'_>,
    diagnostics: &mut Vec<ComparisonDiagnostic>,
) -> Result<String> {
    if !original.hyperlinks.is_empty()
        || !edited.hyperlinks.is_empty()
        || !original.comment_ranges.is_empty()
        || !edited.comment_ranges.is_empty()
        || !original.bookmark_markers.is_empty()
        || !edited.bookmark_markers.is_empty()
        || !original.content_controls.is_empty()
        || !edited.content_controls.is_empty()
        || !original.extra_xml.is_empty()
        || !edited.extra_xml.is_empty()
    {
        return compare_complex_paragraph(original, edited, location, metadata, diagnostics);
    }

    let mut output = String::from("<w:p>");
    output.push_str(&paragraph_properties_xml(
        original,
        edited,
        location,
        metadata,
        diagnostics,
    )?);
    let original_signatures = original.runs.iter().map(run_signature).collect::<Vec<_>>();
    let edited_signatures = edited.runs.iter().map(run_signature).collect::<Vec<_>>();
    let aligned = align(&original_signatures, &edited_signatures);
    validate_field_alignment(
        &aligned,
        &original.runs,
        &edited.runs,
        &original_signatures,
        &edited_signatures,
        location,
    )?;
    for (left, right) in aligned {
        match (left, right) {
            (Some(i), Some(j)) => {
                if original_signatures[i] == edited_signatures[j] {
                    if original.runs[i].properties != edited.runs[j].properties {
                        formatting_diagnostic(diagnostics, format!("{location}/run[{j}]"));
                    }
                    output.push_str(&paragraph_owned_run_xml(&original.runs[i])?);
                } else {
                    let deleted = deleted_run_xml(&original.runs[i])?;
                    output.push_str(&metadata.ids.revision(
                        "del",
                        metadata.author,
                        metadata.timestamp,
                        &deleted,
                    )?);
                    let inserted = paragraph_owned_run_xml(&edited.runs[j])?;
                    output.push_str(&metadata.ids.revision(
                        "ins",
                        metadata.author,
                        metadata.timestamp,
                        &inserted,
                    )?);
                }
            }
            (Some(i), None) => {
                let run = deleted_run_xml(&original.runs[i])?;
                output.push_str(&metadata.ids.revision(
                    "del",
                    metadata.author,
                    metadata.timestamp,
                    &run,
                )?);
            }
            (None, Some(j)) => {
                let run = paragraph_owned_run_xml(&edited.runs[j])?;
                output.push_str(&metadata.ids.revision(
                    "ins",
                    metadata.author,
                    metadata.timestamp,
                    &run,
                )?);
            }
            (None, None) => unreachable!(),
        }
    }
    output.push_str("</w:p>");
    Ok(output)
}

fn compare_complex_paragraph(
    original: &CT_P,
    edited: &CT_P,
    location: &str,
    metadata: &mut Metadata<'_>,
    diagnostics: &mut Vec<ComparisonDiagnostic>,
) -> Result<String> {
    if paragraph_signature(original) == paragraph_signature(edited) {
        if paragraph_formatting(original) != paragraph_formatting(edited) {
            formatting_diagnostic(diagnostics, location.to_owned());
        }
        for (index, (left, right)) in original.runs.iter().zip(&edited.runs).enumerate() {
            if left.properties != right.properties {
                formatting_diagnostic(diagnostics, format!("{location}/run[{index}]"));
            }
        }
        for (index, ((_, _, _, left), (_, _, _, right))) in original
            .content_controls
            .iter()
            .zip(&edited.content_controls)
            .enumerate()
        {
            compare_control(
                left,
                right,
                &format!("{location}/content-control[{index}]"),
                metadata,
                diagnostics,
            )?;
        }
        return paragraph_xml(original);
    }
    if original.hyperlinks != edited.hyperlinks
        || original.comment_ranges != edited.comment_ranges
        || original.bookmark_markers != edited.bookmark_markers
        || original.extra_xml != edited.extra_xml
        || paragraph_control_boundaries(original) != paragraph_control_boundaries(edited)
        || original.properties != edited.properties
    {
        return Err(Error::Other(format!(
            "comparison cannot revise paragraph boundary structures at {location}"
        )));
    }

    let source = paragraph_xml(original)?;
    let original_signatures = original.runs.iter().map(run_signature).collect::<Vec<_>>();
    let edited_signatures = edited.runs.iter().map(run_signature).collect::<Vec<_>>();
    let aligned = align(&original_signatures, &edited_signatures);
    validate_field_alignment(
        &aligned,
        &original.runs,
        &edited.runs,
        &original_signatures,
        &edited_signatures,
        location,
    )?;
    let mut output = String::new();
    let mut cursor = 0usize;
    for (left, right) in aligned {
        let old_run = left
            .map(|index| paragraph_owned_run_xml(&original.runs[index]))
            .transpose()?;
        if let Some(old_run) = &old_run {
            let relative = source[cursor..].find(old_run).ok_or_else(|| {
                Error::Other(format!(
                    "comparison could not locate a serialized run at {location}"
                ))
            })?;
            let start = cursor + relative;
            output.push_str(&source[cursor..start]);
            cursor = start + old_run.len();
        }
        match (left, right) {
            (Some(i), Some(j)) if original_signatures[i] == edited_signatures[j] => {
                if original.runs[i].properties != edited.runs[j].properties {
                    formatting_diagnostic(diagnostics, format!("{location}/run[{j}]"));
                }
                output.push_str(old_run.as_deref().expect("paired run has source XML"));
            }
            (Some(i), Some(j)) => {
                let deleted = deleted_run_xml(&original.runs[i])?;
                output.push_str(&metadata.ids.revision(
                    "del",
                    metadata.author,
                    metadata.timestamp,
                    &deleted,
                )?);
                let inserted = paragraph_owned_run_xml(&edited.runs[j])?;
                output.push_str(&metadata.ids.revision(
                    "ins",
                    metadata.author,
                    metadata.timestamp,
                    &inserted,
                )?);
            }
            (Some(i), None) => {
                let deleted = deleted_run_xml(&original.runs[i])?;
                output.push_str(&metadata.ids.revision(
                    "del",
                    metadata.author,
                    metadata.timestamp,
                    &deleted,
                )?);
            }
            (None, Some(j)) => {
                let inserted = paragraph_owned_run_xml(&edited.runs[j])?;
                output.push_str(&metadata.ids.revision(
                    "ins",
                    metadata.author,
                    metadata.timestamp,
                    &inserted,
                )?);
            }
            (None, None) => unreachable!(),
        }
    }
    output.push_str(&source[cursor..]);
    let spans = direct_word_element_spans(&output, "sdt")?;
    if spans.len() != original.content_controls.len() {
        return Err(Error::Other(format!(
            "comparison could not correlate paragraph content controls at {location}"
        )));
    }
    let mut replacements = Vec::with_capacity(spans.len());
    for (index, ((_, _, _, left), (_, _, _, right))) in original
        .content_controls
        .iter()
        .zip(&edited.content_controls)
        .enumerate()
    {
        replacements.push(compare_control_from_xml(
            left,
            right,
            &format!("{location}/content-control[{index}]"),
            metadata,
            diagnostics,
            &output[spans[index].clone()],
        )?);
    }
    replace_direct_word_elements(&output, "sdt", &replacements)
}

fn paragraph_control_boundaries(paragraph: &CT_P) -> Vec<(usize, usize, usize)> {
    paragraph
        .content_controls
        .iter()
        .map(|(at, raw_before, markers_before, _)| (*at, *raw_before, *markers_before))
        .collect()
}

fn paragraph_properties_xml(
    original: &CT_P,
    edited: &CT_P,
    location: &str,
    metadata: &mut Metadata<'_>,
    diagnostics: &mut Vec<ComparisonDiagnostic>,
) -> Result<String> {
    let original_numbering = paragraph_numbering(original);
    let edited_numbering = paragraph_numbering(edited);
    let mut current = original.properties.clone();
    if original_numbering != edited_numbering {
        let edited_properties = edited.properties.as_ref();
        let current_properties = current.get_or_insert_with(CT_PPr::default);
        current_properties.num_id = edited_properties.and_then(|properties| properties.num_id);
        current_properties.num_ilvl = edited_properties.and_then(|properties| properties.num_ilvl);
        let previous = original
            .properties
            .as_ref()
            .map(property_xml)
            .transpose()?
            .unwrap_or_else(|| "<w:pPr/>".to_owned());
        let change =
            metadata
                .ids
                .revision("pPrChange", metadata.author, metadata.timestamp, &previous)?;
        let mut current_xml = property_xml(current_properties)?;
        if current_xml.is_empty() {
            current_xml.push_str("<w:pPr></w:pPr>");
        }
        return inject_before_close(&current_xml, "</w:pPr>", &change);
    }
    if paragraph_formatting(original) != paragraph_formatting(edited) {
        formatting_diagnostic(diagnostics, location.to_owned());
    }
    current
        .as_ref()
        .map(property_xml)
        .transpose()
        .map(|xml| xml.unwrap_or_default())
}

fn deleted_paragraph(paragraph: &CT_P, metadata: &mut Metadata<'_>) -> Result<String> {
    let mut output = String::from("<w:p>");
    output.push_str(&paragraph_mark_properties(
        paragraph.properties.as_ref(),
        "del",
        metadata,
    )?);
    for run in &paragraph.runs {
        let run = deleted_run_xml(run)?;
        output.push_str(&metadata.ids.revision(
            "del",
            metadata.author,
            metadata.timestamp,
            &run,
        )?);
    }
    output.push_str("</w:p>");
    Ok(output)
}

fn inserted_paragraph(paragraph: &CT_P, metadata: &mut Metadata<'_>) -> Result<String> {
    let mut output = String::from("<w:p>");
    output.push_str(&paragraph_mark_properties(
        paragraph.properties.as_ref(),
        "ins",
        metadata,
    )?);
    for run in &paragraph.runs {
        let run = run_xml(run)?;
        output.push_str(&metadata.ids.revision(
            "ins",
            metadata.author,
            metadata.timestamp,
            &run,
        )?);
    }
    output.push_str("</w:p>");
    Ok(output)
}

fn paragraph_mark_properties(
    properties: Option<&CT_PPr>,
    kind: &str,
    metadata: &mut Metadata<'_>,
) -> Result<String> {
    let marker = metadata
        .ids
        .marker(kind, metadata.author, metadata.timestamp)?;
    let mut properties = properties.cloned().unwrap_or_default();
    properties.rpr = Some(properties.rpr.take().unwrap_or_default());
    let mut xml = property_xml(&properties)?;
    let run_properties = direct_word_element_spans(&xml, "rPr")?;
    if let Some(run_span) = run_properties.first() {
        let updated = append_word_child(&xml[run_span.clone()], "rPr", &marker)?;
        xml.replace_range(run_span.clone(), &updated);
        Ok(xml)
    } else {
        append_word_child(&xml, "pPr", &format!("<w:rPr>{marker}</w:rPr>"))
    }
}

fn compare_table(
    original: &CT_Tbl,
    edited: &CT_Tbl,
    location: &str,
    metadata: &mut Metadata<'_>,
    diagnostics: &mut Vec<ComparisonDiagnostic>,
) -> Result<String> {
    if table_formatting(original) != table_formatting(edited) {
        formatting_diagnostic(diagnostics, location.to_owned());
    }
    if original.grid != edited.grid {
        return Err(Error::Other(format!(
            "comparison cannot revise a table grid change at {location}"
        )));
    }
    if original.extra_xml != edited.extra_xml
        || table_control_boundaries(original) != table_control_boundaries(edited)
    {
        return Err(Error::Other(format!(
            "comparison cannot revise table boundary structures at {location}"
        )));
    }
    let original_signatures = original.rows.iter().map(row_signature).collect::<Vec<_>>();
    let edited_signatures = edited.rows.iter().map(row_signature).collect::<Vec<_>>();
    let source = table_xml(original)?;
    let row_spans = direct_word_element_spans(&source, "tr")?;
    if row_spans.len() != original.rows.len() {
        return Err(Error::Other(format!(
            "comparison could not correlate direct table rows at {location}"
        )));
    }
    let aligned = align(&original_signatures, &edited_signatures);
    let mut output = String::new();
    let mut cursor = 0usize;
    for (alignment_index, &(left, right)) in aligned.iter().enumerate() {
        if let Some(index) = left {
            let span = &row_spans[index];
            output.push_str(&source[cursor..span.start]);
            cursor = span.end;
        } else {
            let next_row_start = aligned[alignment_index + 1..]
                .iter()
                .find_map(|(next_left, _)| next_left.map(|index| row_spans[index].start))
                .unwrap_or_else(|| source.rfind("</w:tbl>").unwrap_or(source.len()));
            output.push_str(&source[cursor..next_row_start]);
            cursor = next_row_start;
        }
        match (left, right) {
            (Some(i), Some(j)) => {
                output.push_str(&compare_row(
                    &original.rows[i],
                    &edited.rows[j],
                    &format!("{location}/row[{j}]"),
                    metadata,
                    diagnostics,
                )?);
            }
            (Some(i), None) => output.push_str(&marked_row(&original.rows[i], "del", metadata)?),
            (None, Some(j)) => output.push_str(&marked_row(&edited.rows[j], "ins", metadata)?),
            (None, None) => unreachable!(),
        }
    }
    output.push_str(&source[cursor..]);
    let spans = direct_word_element_spans(&output, "sdt")?;
    if spans.len() != original.content_controls.len() {
        return Err(Error::Other(format!(
            "comparison could not correlate table content controls at {location}"
        )));
    }
    let mut replacements = Vec::with_capacity(spans.len());
    for (index, ((_, _, left), (_, _, right))) in original
        .content_controls
        .iter()
        .zip(&edited.content_controls)
        .enumerate()
    {
        replacements.push(compare_control_from_xml(
            left,
            right,
            &format!("{location}/content-control[{index}]"),
            metadata,
            diagnostics,
            &output[spans[index].clone()],
        )?);
    }
    replace_direct_word_elements(&output, "sdt", &replacements)
}

fn table_control_boundaries(table: &CT_Tbl) -> Vec<(usize, usize)> {
    table
        .content_controls
        .iter()
        .map(|(at, raw_before, _)| (*at, *raw_before))
        .collect()
}

fn marked_table(table: &CT_Tbl, kind: &str, metadata: &mut Metadata<'_>) -> Result<String> {
    let source = table_xml(table)?;
    let replacements = table
        .rows
        .iter()
        .map(|row| marked_row(row, kind, metadata))
        .collect::<Result<Vec<_>>>()?;
    let output = replace_direct_word_elements(&source, "tr", &replacements)?;
    let control_spans = direct_word_element_spans(&output, "sdt")?;
    if control_spans.len() != table.content_controls.len() {
        return Err(Error::Other(
            "comparison could not correlate table content controls while marking rows".to_owned(),
        ));
    }
    let replacements = table
        .content_controls
        .iter()
        .zip(&control_spans)
        .map(|((_, _, control), span)| {
            mark_control_owned_rows(control, kind, metadata, &output[span.clone()])
        })
        .collect::<Result<Vec<_>>>()?;
    replace_direct_word_elements(&output, "sdt", &replacements)
}

fn mark_control_owned_rows(
    control: &CT_Sdt,
    kind: &str,
    metadata: &mut Metadata<'_>,
    original_xml: &str,
) -> Result<String> {
    let mut content = String::new();
    for child in &control.content {
        content.push_str(&match child {
            SdtContent::Paragraph(paragraph) => paragraph_xml(paragraph)?,
            SdtContent::Table(table) => marked_table(table, kind, metadata)?,
            SdtContent::Row(row) => marked_row(row, kind, metadata)?,
            SdtContent::Cell(cell) => cell_xml(cell)?,
            SdtContent::Run(run) => paragraph_owned_run_xml(run)?,
            SdtContent::ContentControl(nested) => {
                let nested_xml = control_xml(nested)?;
                mark_control_owned_rows(nested, kind, metadata, &nested_xml)?
            }
            SdtContent::RawXml(raw) => String::from_utf8(raw.clone()).map_err(utf8_error)?,
        });
    }
    replace_element_inner(original_xml, "w:sdtContent", &content)
}

fn marked_row(row: &CT_Row, kind: &str, metadata: &mut Metadata<'_>) -> Result<String> {
    let marker = metadata
        .ids
        .marker(kind, metadata.author, metadata.timestamp)?;
    let mut xml = row_xml(row)?;
    let properties = direct_word_element_spans(&xml, "trPr")?;
    if let Some(properties_span) = properties.first() {
        let updated = append_word_child(&xml[properties_span.clone()], "trPr", &marker)?;
        xml.replace_range(properties_span.clone(), &updated);
        Ok(xml)
    } else {
        let open = xml
            .find('>')
            .ok_or_else(|| Error::Other("row XML has no start".to_owned()))?
            + 1;
        Ok(format!(
            "{}<w:trPr>{marker}</w:trPr>{}",
            &xml[..open],
            &xml[open..]
        ))
    }
}

fn compare_row(
    original: &CT_Row,
    edited: &CT_Row,
    location: &str,
    metadata: &mut Metadata<'_>,
    diagnostics: &mut Vec<ComparisonDiagnostic>,
) -> Result<String> {
    if original.cells.len() != edited.cells.len()
        || original.extra_xml != edited.extra_xml
        || row_control_boundaries(original) != row_control_boundaries(edited)
    {
        return Err(Error::Other(format!(
            "comparison cannot revise row boundary structures at {location}"
        )));
    }
    if row_formatting(original) != row_formatting(edited) {
        formatting_diagnostic(diagnostics, location.to_owned());
    }
    let mut output = row_xml(original)?;
    let cell_spans = direct_word_element_spans(&output, "tc")?;
    if cell_spans.len() != original.cells.len() {
        return Err(Error::Other(format!(
            "comparison could not correlate row cells at {location}"
        )));
    }
    let mut cell_replacements = Vec::with_capacity(cell_spans.len());
    for (index, (left, right)) in original.cells.iter().zip(&edited.cells).enumerate() {
        cell_replacements.push(compare_cell_from_xml(
            left,
            right,
            &format!("{location}/cell[{index}]"),
            metadata,
            diagnostics,
            &output[cell_spans[index].clone()],
        )?);
    }
    output = replace_direct_word_elements(&output, "tc", &cell_replacements)?;

    let control_spans = direct_word_element_spans(&output, "sdt")?;
    if control_spans.len() != original.content_controls.len() {
        return Err(Error::Other(format!(
            "comparison could not correlate row content controls at {location}"
        )));
    }
    let mut control_replacements = Vec::with_capacity(control_spans.len());
    for (index, ((_, _, left), (_, _, right))) in original
        .content_controls
        .iter()
        .zip(&edited.content_controls)
        .enumerate()
    {
        control_replacements.push(compare_control_from_xml(
            left,
            right,
            &format!("{location}/content-control[{index}]"),
            metadata,
            diagnostics,
            &output[control_spans[index].clone()],
        )?);
    }
    replace_direct_word_elements(&output, "sdt", &control_replacements)
}

fn row_control_boundaries(row: &CT_Row) -> Vec<(usize, usize)> {
    row.content_controls
        .iter()
        .map(|(at, raw_before, _)| (*at, *raw_before))
        .collect()
}

fn compare_cell_from_xml(
    original: &CT_Tc,
    edited: &CT_Tc,
    location: &str,
    metadata: &mut Metadata<'_>,
    diagnostics: &mut Vec<ComparisonDiagnostic>,
    original_xml: &str,
) -> Result<String> {
    if original.content.len() != edited.content.len() || original.extra_xml != edited.extra_xml {
        return Err(Error::Other(format!(
            "comparison cannot revise cell boundary structures at {location}"
        )));
    }
    if original.properties != edited.properties {
        formatting_diagnostic(diagnostics, location.to_owned());
    }
    let children = cell_child_spans(original, original_xml, location)?;
    let mut replacements = Vec::with_capacity(children.len());
    for (index, ((left, right), (_, span))) in original
        .content
        .iter()
        .zip(&edited.content)
        .zip(&children)
        .enumerate()
    {
        let child_location = format!("{location}/content[{index}]");
        let source = &original_xml[span.clone()];
        replacements.push(match (left, right) {
            (CellContent::Paragraph(left), CellContent::Paragraph(right)) => {
                compare_paragraph(left, right, &child_location, metadata, diagnostics)?
            }
            (CellContent::Table(left), CellContent::Table(right)) => {
                compare_table(left, right, &child_location, metadata, diagnostics)?
            }
            (CellContent::ContentControl(left), CellContent::ContentControl(right)) => {
                compare_control_from_xml(
                    left,
                    right,
                    &child_location,
                    metadata,
                    diagnostics,
                    source,
                )?
            }
            _ => {
                return Err(Error::Other(format!(
                    "comparison cannot revise incompatible cell children at {child_location}"
                )));
            }
        });
    }
    replace_ranges(original_xml, &children, &replacements)
}

fn compare_control(
    original: &CT_Sdt,
    edited: &CT_Sdt,
    location: &str,
    metadata: &mut Metadata<'_>,
    diagnostics: &mut Vec<ComparisonDiagnostic>,
) -> Result<String> {
    let original_xml = control_xml(original)?;
    compare_control_from_xml(
        original,
        edited,
        location,
        metadata,
        diagnostics,
        &original_xml,
    )
}

fn compare_control_from_xml(
    original: &CT_Sdt,
    edited: &CT_Sdt,
    location: &str,
    metadata: &mut Metadata<'_>,
    diagnostics: &mut Vec<ComparisonDiagnostic>,
    original_xml: &str,
) -> Result<String> {
    if control_property_signature(original) != control_property_signature(edited) {
        return Err(Error::Other(format!(
            "comparison cannot revise content-control properties at {location}"
        )));
    }
    let original_content = modeled_control_content(original);
    let edited_content = modeled_control_content(edited);
    let whitespace_slots = control_whitespace_slots(original)?;
    let original_signatures = original_content
        .iter()
        .map(|content| control_content_signature(content))
        .collect::<Vec<_>>();
    let edited_signatures = edited_content
        .iter()
        .map(|content| control_content_signature(content))
        .collect::<Vec<_>>();
    let aligned = expand_control_alignment(
        align(&original_signatures, &edited_signatures),
        &original_content,
        &edited_content,
    );
    let mut content: Vec<(bool, String)> = Vec::new();
    let mut whitespace_emitted = vec![false; whitespace_slots.len()];
    for (position, (left, right)) in aligned.iter().copied().enumerate() {
        let whitespace_boundary = left.or_else(|| {
            let right = right?;
            let (Some(next_left), None) = aligned.get(position + 1).copied()? else {
                return None;
            };
            (matches!(original_content[next_left], SdtContent::Paragraph(_))
                && matches!(edited_content[right], SdtContent::Table(_)))
            .then_some(next_left)
        });
        if let Some(index) = whitespace_boundary
            && !whitespace_emitted[index]
            && !whitespace_slots[index].is_empty()
        {
            content.push((false, whitespace_slots[index].clone()));
            whitespace_emitted[index] = true;
        }
        let next_is_paragraph = aligned.get(position + 1).is_some_and(|(left, right)| {
            right
                .and_then(|index| edited_content.get(index).copied())
                .or_else(|| left.and_then(|index| original_content.get(index).copied()))
                .is_some_and(|content| matches!(content, SdtContent::Paragraph(_)))
        });
        let child_location = format!("{location}/content[{position}]");
        match (left, right) {
            (Some(i), Some(j)) => content.push((
                matches!(edited_content[j], SdtContent::Paragraph(_)),
                compare_control_content(
                    original_content[i],
                    edited_content[j],
                    &child_location,
                    metadata,
                    diagnostics,
                )?,
            )),
            (Some(i), None) => {
                if matches!(original_content[i], SdtContent::Paragraph(_)) && !next_is_paragraph {
                    mark_previous_paragraph(&mut content, "del", metadata)?;
                    content.push((
                        true,
                        marked_control_content(original_content[i], "del", false, metadata)?,
                    ));
                } else {
                    content.push((
                        matches!(original_content[i], SdtContent::Paragraph(_)),
                        marked_control_content(original_content[i], "del", true, metadata)?,
                    ));
                }
            }
            (None, Some(j)) => {
                if matches!(edited_content[j], SdtContent::Paragraph(_)) && !next_is_paragraph {
                    mark_previous_paragraph(&mut content, "ins", metadata)?;
                    content.push((
                        true,
                        marked_control_content(edited_content[j], "ins", false, metadata)?,
                    ));
                } else {
                    content.push((
                        matches!(edited_content[j], SdtContent::Paragraph(_)),
                        marked_control_content(edited_content[j], "ins", true, metadata)?,
                    ));
                }
            }
            (None, None) => unreachable!(),
        }
    }
    if let Some(trailing) = whitespace_slots.last()
        && !trailing.is_empty()
    {
        content.push((false, trailing.clone()));
    }
    let content = content.into_iter().map(|(_, xml)| xml).collect::<String>();
    replace_element_inner(original_xml, "w:sdtContent", &content)
}

fn compare_control_content(
    original: &SdtContent,
    edited: &SdtContent,
    location: &str,
    metadata: &mut Metadata<'_>,
    diagnostics: &mut Vec<ComparisonDiagnostic>,
) -> Result<String> {
    match (original, edited) {
        (SdtContent::Paragraph(left), SdtContent::Paragraph(right)) => {
            compare_paragraph(left, right, location, metadata, diagnostics)
        }
        (SdtContent::Table(left), SdtContent::Table(right)) => {
            compare_table(left, right, location, metadata, diagnostics)
        }
        (SdtContent::ContentControl(left), SdtContent::ContentControl(right)) => {
            compare_control(left, right, location, metadata, diagnostics)
        }
        (SdtContent::Row(left), SdtContent::Row(right)) => {
            compare_row(left, right, location, metadata, diagnostics)
        }
        (SdtContent::Cell(left), SdtContent::Cell(right)) => compare_cell_from_xml(
            left,
            right,
            location,
            metadata,
            diagnostics,
            &cell_xml(left)?,
        ),
        (SdtContent::Run(left), SdtContent::Run(right)) => {
            if run_signature(left) == run_signature(right) {
                if left.properties != right.properties {
                    formatting_diagnostic(diagnostics, location.to_owned());
                }
                run_xml(left)
            } else {
                let deleted = metadata.ids.revision(
                    "del",
                    metadata.author,
                    metadata.timestamp,
                    &deleted_run_xml(left)?,
                )?;
                let inserted = metadata.ids.revision(
                    "ins",
                    metadata.author,
                    metadata.timestamp,
                    &run_xml(right)?,
                )?;
                Ok(format!("{deleted}{inserted}"))
            }
        }
        (SdtContent::RawXml(left), SdtContent::RawXml(right)) if left == right => {
            String::from_utf8(left.clone()).map_err(utf8_error)
        }
        _ => Err(Error::Other(format!(
            "comparison cannot revise incompatible content-control children at {location}"
        ))),
    }
}

fn marked_control_content(
    content: &SdtContent,
    kind: &str,
    paragraph_marker: bool,
    metadata: &mut Metadata<'_>,
) -> Result<String> {
    match content {
        SdtContent::Paragraph(paragraph) if paragraph_marker && kind == "del" => {
            deleted_paragraph(paragraph, metadata)
        }
        SdtContent::Paragraph(paragraph) if paragraph_marker => {
            inserted_paragraph(paragraph, metadata)
        }
        SdtContent::Paragraph(paragraph) if kind == "del" => {
            deleted_paragraph_content(&BodyContent::Paragraph(paragraph.clone()), metadata)
        }
        SdtContent::Paragraph(paragraph) => {
            inserted_paragraph_content(&BodyContent::Paragraph(paragraph.clone()), metadata)
        }
        SdtContent::Table(table) => marked_table(table, kind, metadata),
        SdtContent::Row(row) => marked_row(row, kind, metadata),
        SdtContent::Run(run) if kind == "del" => metadata.ids.revision(
            kind,
            metadata.author,
            metadata.timestamp,
            &deleted_run_xml(run)?,
        ),
        SdtContent::Run(run) => {
            metadata
                .ids
                .revision(kind, metadata.author, metadata.timestamp, &run_xml(run)?)
        }
        SdtContent::RawXml(raw) => String::from_utf8(raw.clone()).map_err(utf8_error),
        SdtContent::Cell(_) | SdtContent::ContentControl(_) => Err(Error::Other(
            "comparison cannot add or remove an unmatched cell or nested content control"
                .to_owned(),
        )),
    }
}

fn formatting_diagnostic(diagnostics: &mut Vec<ComparisonDiagnostic>, location: String) {
    diagnostics.push(ComparisonDiagnostic {
        location,
        message: "formatting differs and the original formatting was retained".to_owned(),
    });
}

fn body_location(content: &BodyContent, index: usize) -> String {
    match content {
        BodyContent::Paragraph(_) => format!("body/paragraph[{index}]"),
        BodyContent::Table(_) => format!("body/table[{index}]"),
        BodyContent::ContentControl(_) => format!("body/content-control[{index}]"),
        BodyContent::RawXml(_) => format!("body/raw[{index}]"),
    }
}

fn align(original: &[String], edited: &[String]) -> Vec<(Option<usize>, Option<usize>)> {
    let mut lcs = vec![vec![0usize; edited.len() + 1]; original.len() + 1];
    for i in (0..original.len()).rev() {
        for j in (0..edited.len()).rev() {
            lcs[i][j] = if original[i] == edited[j] {
                1 + lcs[i + 1][j + 1]
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }
    let mut matches = Vec::new();
    let (mut i, mut j) = (0usize, 0usize);
    while i < original.len() && j < edited.len() {
        if original[i] == edited[j] {
            matches.push((i, j));
            i += 1;
            j += 1;
        } else if lcs[i + 1][j] >= lcs[i][j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    let mut result = Vec::new();
    let (mut left, mut right) = (0usize, 0usize);
    for (matched_left, matched_right) in matches
        .into_iter()
        .chain(std::iter::once((original.len(), edited.len())))
    {
        let paired = (matched_left - left).min(matched_right - right);
        for offset in 0..paired {
            result.push((Some(left + offset), Some(right + offset)));
        }
        for index in left + paired..matched_left {
            result.push((Some(index), None));
        }
        for index in right + paired..matched_right {
            result.push((None, Some(index)));
        }
        if matched_left < original.len() {
            result.push((Some(matched_left), Some(matched_right)));
        }
        left = matched_left.saturating_add(1);
        right = matched_right.saturating_add(1);
    }
    result
}

fn expand_body_alignment(
    aligned: Vec<(Option<usize>, Option<usize>)>,
    original: &[BodyContent],
    edited: &[BodyContent],
) -> Vec<(Option<usize>, Option<usize>)> {
    let mut expanded = Vec::with_capacity(aligned.len());
    for (left, right) in aligned {
        match (left, right) {
            (Some(i), Some(j))
                if matches!(original[i], BodyContent::Paragraph(_))
                    && matches!(edited[j], BodyContent::Table(_)) =>
            {
                expanded.push((None, Some(j)));
                expanded.push((Some(i), None));
            }
            (Some(i), Some(j))
                if matches!(original[i], BodyContent::Table(_))
                    && matches!(edited[j], BodyContent::Paragraph(_)) =>
            {
                expanded.push((Some(i), None));
                expanded.push((None, Some(j)));
            }
            pair => expanded.push(pair),
        }
    }
    expanded
}

fn expand_control_alignment(
    aligned: Vec<(Option<usize>, Option<usize>)>,
    original: &[&SdtContent],
    edited: &[&SdtContent],
) -> Vec<(Option<usize>, Option<usize>)> {
    let mut expanded = Vec::with_capacity(aligned.len());
    for (left, right) in aligned {
        match (left, right) {
            (Some(i), Some(j))
                if matches!(original[i], SdtContent::Paragraph(_))
                    && matches!(edited[j], SdtContent::Table(_)) =>
            {
                expanded.push((None, Some(j)));
                expanded.push((Some(i), None));
            }
            (Some(i), Some(j))
                if matches!(original[i], SdtContent::Table(_))
                    && matches!(edited[j], SdtContent::Paragraph(_)) =>
            {
                expanded.push((Some(i), None));
                expanded.push((None, Some(j)));
            }
            pair => expanded.push(pair),
        }
    }
    expanded
}

fn body_signature(content: &BodyContent) -> String {
    match content {
        BodyContent::Paragraph(paragraph) => format!("p:{}", paragraph_signature(paragraph)),
        BodyContent::Table(table) => format!("t:{}", table_signature(table)),
        BodyContent::ContentControl(control) => format!("s:{}", control_signature(control)),
        BodyContent::RawXml(raw) => format!("x:{raw:?}"),
    }
}

fn paragraph_signature(paragraph: &CT_P) -> String {
    let numbering = paragraph_numbering(paragraph);
    format!(
        "{numbering:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}",
        paragraph.runs.iter().map(run_signature).collect::<Vec<_>>(),
        paragraph.hyperlinks,
        paragraph.comment_ranges,
        paragraph.bookmark_markers,
        paragraph.extra_xml,
        paragraph
            .content_controls
            .iter()
            .map(|(at, raw_before, markers_before, control)| (
                at,
                raw_before,
                markers_before,
                control_signature(control),
            ))
            .collect::<Vec<_>>(),
    )
}

fn paragraph_numbering(paragraph: &CT_P) -> Option<(Option<u32>, Option<u32>)> {
    paragraph.properties.as_ref().and_then(|properties| {
        (properties.num_id.is_some() || properties.num_ilvl.is_some())
            .then_some((properties.num_id, properties.num_ilvl))
    })
}

fn run_signature(run: &CT_R) -> String {
    format!(
        "{:?}:{:?}:{:?}",
        run.content
            .iter()
            .map(run_content_signature)
            .collect::<Vec<_>>(),
        run.extra_xml,
        run.extra_xml_positions
    )
}

fn run_content_signature(content: &RunContent) -> String {
    match content {
        RunContent::Field(field) => format!(
            "field:{:?}:{:?}:{:?}",
            field.instruction, field.cached_result, field.dirty
        ),
        content => format!("{content:?}"),
    }
}

fn table_signature(table: &CT_Tbl) -> String {
    format!(
        "{:?}:{:?}:{:?}:{:?}",
        table.grid,
        table.rows.iter().map(row_signature).collect::<Vec<_>>(),
        table.extra_xml,
        table
            .content_controls
            .iter()
            .map(|(at, raw_before, control)| (at, raw_before, control_signature(control)))
            .collect::<Vec<_>>()
    )
}

fn row_signature(row: &CT_Row) -> String {
    format!(
        "{:?}:{:?}:{:?}",
        row.cells.iter().map(cell_signature).collect::<Vec<_>>(),
        row.extra_xml,
        row.content_controls
            .iter()
            .map(|(at, raw_before, control)| (at, raw_before, control_signature(control)))
            .collect::<Vec<_>>()
    )
}

fn cell_signature(cell: &rdocx_oxml::table::CT_Tc) -> String {
    format!(
        "{:?}:{:?}",
        cell.content
            .iter()
            .map(|content| match content {
                CellContent::Paragraph(paragraph) => paragraph_signature(paragraph),
                CellContent::Table(table) => table_signature(table),
                CellContent::ContentControl(control) => control_signature(control),
            })
            .collect::<Vec<_>>(),
        cell.extra_xml
    )
}

fn control_signature(control: &CT_Sdt) -> String {
    let properties = control_property_signature(control);
    format!(
        "{:?}:{:?}",
        properties,
        control
            .content
            .iter()
            .filter_map(|content| match content {
                SdtContent::Paragraph(paragraph) => Some(paragraph_signature(paragraph)),
                SdtContent::Table(table) => Some(table_signature(table)),
                SdtContent::Row(row) => Some(row_signature(row)),
                SdtContent::Cell(cell) => Some(cell_signature(cell)),
                SdtContent::Run(run) => Some(run_signature(run)),
                SdtContent::ContentControl(control) => Some(control_signature(control)),
                SdtContent::RawXml(raw) if raw.iter().all(u8::is_ascii_whitespace) => None,
                SdtContent::RawXml(raw) => Some(format!("{raw:?}")),
            })
            .collect::<Vec<_>>()
    )
}

fn control_content_signature(content: &SdtContent) -> String {
    match content {
        SdtContent::Paragraph(paragraph) => format!("p:{}", paragraph_signature(paragraph)),
        SdtContent::Table(table) => format!("t:{}", table_signature(table)),
        SdtContent::Row(row) => format!("r:{}", row_signature(row)),
        SdtContent::Cell(cell) => format!("c:{}", cell_signature(cell)),
        SdtContent::Run(run) => format!("u:{}", run_signature(run)),
        SdtContent::ContentControl(control) => format!("s:{}", control_signature(control)),
        SdtContent::RawXml(raw) => format!("x:{raw:?}"),
    }
}

fn control_property_signature(control: &CT_Sdt) -> ControlPropertySignature<'_> {
    control.properties.as_ref().map(|properties| {
        (
            properties.alias.as_deref(),
            properties.tag.as_deref(),
            properties.id,
            properties.control_type,
            properties.data_binding.as_ref(),
        )
    })
}

fn modeled_control_content(control: &CT_Sdt) -> Vec<&SdtContent> {
    control
        .content
        .iter()
        .filter(|content| {
            !matches!(content, SdtContent::RawXml(raw) if raw.iter().all(u8::is_ascii_whitespace))
        })
        .collect()
}

fn control_whitespace_slots(control: &CT_Sdt) -> Result<Vec<String>> {
    let modeled_count = modeled_control_content(control).len();
    let mut slots = vec![String::new(); modeled_count + 1];
    let mut modeled_before = 0usize;
    for content in &control.content {
        match content {
            SdtContent::RawXml(raw) if raw.iter().all(u8::is_ascii_whitespace) => {
                slots[modeled_before].push_str(std::str::from_utf8(raw).map_err(utf8_error)?);
            }
            _ => modeled_before += 1,
        }
    }
    Ok(slots)
}

fn normalized_body(document: &CT_Document) -> Vec<String> {
    document.body.content.iter().map(body_signature).collect()
}

fn paragraph_formatting(paragraph: &CT_P) -> Option<CT_PPr> {
    paragraph.properties.clone().map(|mut properties| {
        properties.num_id = None;
        properties.num_ilvl = None;
        properties.numbering_revision = None;
        properties.numbering_revision_xml.clear();
        properties.change = None;
        properties.revision_xml.clear();
        properties
    })
}

fn table_formatting(table: &CT_Tbl) -> Option<CT_TblPr> {
    table.properties.clone().map(|mut properties| {
        properties.change = None;
        properties.revision_xml.clear();
        properties
    })
}

fn row_formatting(row: &CT_Row) -> Option<CT_TrPr> {
    row.properties.clone().map(|mut properties| {
        properties.revision_markers.clear();
        properties.revision_xml.clear();
        properties
    })
}

fn body_content_xml(content: &BodyContent) -> Result<String> {
    match content {
        BodyContent::Paragraph(paragraph) => paragraph_xml(paragraph),
        BodyContent::Table(table) => table_xml(table),
        BodyContent::ContentControl(control) => control_xml(control),
        BodyContent::RawXml(raw) => String::from_utf8(raw.clone()).map_err(utf8_error),
    }
}

fn paragraph_xml(paragraph: &CT_P) -> Result<String> {
    let mut bytes = Vec::new();
    paragraph.to_xml(&mut Writer::new(&mut bytes))?;
    String::from_utf8(bytes).map_err(utf8_error)
}

fn property_xml(properties: &CT_PPr) -> Result<String> {
    let mut bytes = Vec::new();
    properties.to_xml(&mut Writer::new(&mut bytes))?;
    String::from_utf8(bytes).map_err(utf8_error)
}

fn run_xml(run: &CT_R) -> Result<String> {
    let mut bytes = Vec::new();
    run.to_xml(&mut Writer::new(&mut bytes))?;
    String::from_utf8(bytes).map_err(utf8_error)
}

fn paragraph_owned_run_xml(run: &CT_R) -> Result<String> {
    if !run_is_field(run) {
        return run_xml(run);
    }
    let mut paragraph = CT_P::new();
    paragraph.runs.push(run.clone());
    let xml = paragraph_xml(&paragraph)?;
    let start = xml
        .find('>')
        .ok_or_else(|| Error::Other("serialized paragraph has no start".to_owned()))?
        + 1;
    let end = xml
        .rfind("</w:p>")
        .ok_or_else(|| Error::Other("serialized paragraph has no end".to_owned()))?;
    Ok(xml[start..end].to_owned())
}

fn run_is_field(run: &CT_R) -> bool {
    matches!(run.content.as_slice(), [RunContent::Field(_)])
}

fn validate_field_alignment(
    aligned: &[(Option<usize>, Option<usize>)],
    original: &[CT_R],
    edited: &[CT_R],
    original_signatures: &[String],
    edited_signatures: &[String],
    location: &str,
) -> Result<()> {
    for (left, right) in aligned {
        let left_field = left.is_some_and(|index| run_is_field(&original[index]));
        let right_field = right.is_some_and(|index| run_is_field(&edited[index]));
        let unchanged_pair = match (*left, *right) {
            (Some(i), Some(j)) => original_signatures[i] == edited_signatures[j],
            _ => false,
        };
        if (left_field || right_field) && !unchanged_pair {
            return Err(Error::Other(format!(
                "comparison cannot revise a modeled field at {location}"
            )));
        }
    }
    Ok(())
}

fn deleted_run_xml(run: &CT_R) -> Result<String> {
    let mut deleted = run.clone();
    for content in &mut deleted.content {
        if let RunContent::Text(text) = content {
            *content = RunContent::DeletedText(text.clone());
        }
    }
    run_xml(&deleted)
}

fn table_xml(table: &CT_Tbl) -> Result<String> {
    let mut bytes = Vec::new();
    table.to_xml(&mut Writer::new(&mut bytes))?;
    String::from_utf8(bytes).map_err(utf8_error)
}

fn row_xml(row: &CT_Row) -> Result<String> {
    let mut bytes = Vec::new();
    row.to_xml(&mut Writer::new(&mut bytes))?;
    String::from_utf8(bytes).map_err(utf8_error)
}

fn cell_xml(cell: &CT_Tc) -> Result<String> {
    let mut bytes = Vec::new();
    cell.to_xml(&mut Writer::new(&mut bytes))?;
    String::from_utf8(bytes).map_err(utf8_error)
}

fn control_xml(control: &CT_Sdt) -> Result<String> {
    let document = CT_Document {
        body: rdocx_oxml::document::CT_Body {
            content: vec![BodyContent::ContentControl(control.clone())],
            sect_pr: None,
        },
        ..CT_Document::new()
    };
    let xml = document.to_xml()?;
    extract_body_inner(&xml).map(str::to_owned)
}

fn section_xml(document: &CT_Document) -> Result<String> {
    let section = CT_Document {
        body: rdocx_oxml::document::CT_Body {
            content: Vec::new(),
            sect_pr: document.body.sect_pr.clone(),
        },
        ..CT_Document::new()
    };
    let xml = section.to_xml()?;
    extract_body_inner(&xml).map(str::to_owned)
}

fn replace_body_inner(document: &[u8], body: &str) -> Result<String> {
    let xml = std::str::from_utf8(document).map_err(utf8_error)?;
    let start = xml
        .find("<w:body")
        .and_then(|at| xml[at..].find('>').map(|offset| at + offset + 1))
        .ok_or_else(|| Error::Other("serialized document has no w:body start".to_owned()))?;
    let end = xml
        .rfind("</w:body>")
        .ok_or_else(|| Error::Other("serialized document has no w:body end".to_owned()))?;
    Ok(format!("{}{}{}", &xml[..start], body, &xml[end..]))
}

fn extract_body_inner(document: &[u8]) -> Result<&str> {
    let xml = std::str::from_utf8(document).map_err(utf8_error)?;
    let start = xml
        .find("<w:body")
        .and_then(|at| xml[at..].find('>').map(|offset| at + offset + 1))
        .ok_or_else(|| Error::Other("serialized document has no w:body start".to_owned()))?;
    let end = xml
        .rfind("</w:body>")
        .ok_or_else(|| Error::Other("serialized document has no w:body end".to_owned()))?;
    Ok(&xml[start..end])
}

fn replace_element_inner(xml: &str, name: &str, inner: &str) -> Result<String> {
    let opening = format!("<{name}");
    let closing = format!("</{name}>");
    let start = xml
        .find(&opening)
        .and_then(|at| xml[at..].find('>').map(|offset| at + offset + 1))
        .ok_or_else(|| Error::Other(format!("serialized XML has no {name} start")))?;
    let end = xml
        .rfind(&closing)
        .ok_or_else(|| Error::Other(format!("serialized XML has no {name} end")))?;
    Ok(format!("{}{}{}", &xml[..start], inner, &xml[end..]))
}

fn inject_before_close(xml: &str, closing: &str, addition: &str) -> Result<String> {
    let at = xml
        .rfind(closing)
        .ok_or_else(|| Error::Other(format!("serialized XML has no {closing}")))?;
    Ok(format!("{}{}{}", &xml[..at], addition, &xml[at..]))
}

fn append_word_child(xml: &str, local: &str, addition: &str) -> Result<String> {
    let closing = format!("</w:{local}>");
    if xml.ends_with(&closing) {
        return inject_before_close(xml, &closing, addition);
    }
    let Some(slash) = xml.rfind("/>") else {
        return Err(Error::Other(format!(
            "serialized w:{local} has no closing boundary"
        )));
    };
    Ok(format!(
        "{}>{addition}</w:{local}>{}",
        &xml[..slash],
        &xml[slash + 2..]
    ))
}

fn direct_word_element_spans(xml: &str, local: &str) -> Result<Vec<Range<usize>>> {
    let mut reader = Reader::from_reader(xml.as_bytes());
    reader.config_mut().trim_text(false);
    let expected = format!("w:{local}");
    let mut spans = Vec::new();
    let mut open = Vec::new();
    let mut depth = 0usize;
    let mut buffer = Vec::new();
    loop {
        let before = reader.buffer_position() as usize;
        let event = reader
            .read_event_into(&mut buffer)
            .map_err(|error| Error::Other(format!("comparison XML scan failed: {error}")))?;
        let after = reader.buffer_position() as usize;
        match event {
            Event::Start(element) => {
                let target = depth == 1 && element.name().as_ref() == expected.as_bytes();
                open.push(target.then_some(before));
                depth += 1;
            }
            Event::Empty(element) => {
                if depth == 1 && element.name().as_ref() == expected.as_bytes() {
                    spans.push(before..after);
                }
            }
            Event::End(_) => {
                depth = depth.saturating_sub(1);
                if let Some(Some(start)) = open.pop() {
                    spans.push(start..after);
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(spans)
}

fn replace_direct_word_elements(xml: &str, local: &str, replacements: &[String]) -> Result<String> {
    let spans = direct_word_element_spans(xml, local)?;
    if spans.len() != replacements.len() {
        return Err(Error::Other(format!(
            "comparison expected {} direct w:{local} elements, found {}",
            replacements.len(),
            spans.len()
        )));
    }
    let mut output = xml.to_owned();
    for (span, replacement) in spans.into_iter().zip(replacements).rev() {
        output.replace_range(span, replacement);
    }
    Ok(output)
}

fn cell_child_spans(
    cell: &CT_Tc,
    xml: &str,
    location: &str,
) -> Result<Vec<(String, Range<usize>)>> {
    let candidates = direct_modeled_child_spans(xml)?;
    let mut spans = Vec::with_capacity(cell.content.len());
    let mut candidate_index = 0usize;
    for (index, child) in cell.content.iter().enumerate() {
        for (_, raw) in cell.extra_xml.iter().filter(|(at, _)| *at == index) {
            let raw = std::str::from_utf8(raw).map_err(utf8_error)?;
            if candidates
                .get(candidate_index)
                .is_some_and(|(_, span)| &xml[span.clone()] == raw)
            {
                candidate_index += 1;
            }
        }
        let expected_name = match child {
            CellContent::Paragraph(_) => "w:p",
            CellContent::Table(_) => "w:tbl",
            CellContent::ContentControl(_) => "w:sdt",
        };
        let (name, span) = candidates.get(candidate_index).ok_or_else(|| {
            Error::Other(format!(
                "comparison could not correlate serialized cell content at {location}"
            ))
        })?;
        if name != expected_name {
            return Err(Error::Other(format!(
                "comparison found {name} instead of {expected_name} at {location}"
            )));
        }
        spans.push((name.clone(), span.clone()));
        candidate_index += 1;
    }
    Ok(spans)
}

fn direct_modeled_child_spans(xml: &str) -> Result<Vec<(String, Range<usize>)>> {
    let mut reader = Reader::from_reader(xml.as_bytes());
    reader.config_mut().trim_text(false);
    let mut spans = Vec::new();
    let mut open: Vec<Option<(String, usize)>> = Vec::new();
    let mut depth = 0usize;
    let mut buffer = Vec::new();
    loop {
        let before = reader.buffer_position() as usize;
        let event = reader
            .read_event_into(&mut buffer)
            .map_err(|error| Error::Other(format!("comparison XML scan failed: {error}")))?;
        let after = reader.buffer_position() as usize;
        match event {
            Event::Start(element) => {
                let name = std::str::from_utf8(element.name().as_ref())
                    .map_err(utf8_error)?
                    .to_owned();
                let modeled = depth == 1 && matches!(name.as_str(), "w:p" | "w:tbl" | "w:sdt");
                open.push(modeled.then_some((name, before)));
                depth += 1;
            }
            Event::Empty(element) => {
                let name = std::str::from_utf8(element.name().as_ref())
                    .map_err(utf8_error)?
                    .to_owned();
                if depth == 1 && matches!(name.as_str(), "w:p" | "w:tbl" | "w:sdt") {
                    spans.push((name, before..after));
                }
            }
            Event::End(_) => {
                depth = depth.saturating_sub(1);
                if let Some(Some((name, start))) = open.pop() {
                    spans.push((name, start..after));
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(spans)
}

fn replace_ranges(
    xml: &str,
    spans: &[(String, Range<usize>)],
    replacements: &[String],
) -> Result<String> {
    if spans.len() != replacements.len() {
        return Err(Error::Other(
            "comparison replacement count does not match modeled children".to_owned(),
        ));
    }
    let mut output = xml.to_owned();
    for ((_, span), replacement) in spans.iter().zip(replacements).rev() {
        output.replace_range(span.clone(), replacement);
    }
    Ok(output)
}

fn word_ids(xml: &[u8]) -> Result<HashSet<i32>> {
    let mut reader = NsReader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut ids = HashSet::new();
    let mut buffer = Vec::new();
    loop {
        let (_, event) = reader
            .read_resolved_event_into(&mut buffer)
            .map_err(|error| Error::Other(format!("comparison XML scan failed: {error}")))?;
        match event {
            Event::Start(element) | Event::Empty(element) => {
                for attribute in element.attributes() {
                    let attribute = attribute.map_err(|error| Error::Other(error.to_string()))?;
                    let (namespace, local) = reader.resolver().resolve_attribute(attribute.key);
                    if local.as_ref() == b"id"
                        && matches!(namespace, ResolveResult::Bound(Namespace(uri)) if uri == W_NS.as_bytes())
                        && let Ok(value) = std::str::from_utf8(&attribute.value)
                        && let Ok(id) = value.parse::<i32>()
                    {
                        ids.insert(id);
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(ids)
}

fn utf8_error(error: impl std::fmt::Display) -> Error {
    Error::Other(format!("comparison XML is not UTF-8: {error}"))
}
