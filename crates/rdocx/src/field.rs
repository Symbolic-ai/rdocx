//! Pure evaluation of Word fields against an explicit document context.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use oxml_core::custom_properties::CustomPropertyValue;
use oxml_opc::OpcPackage;
use oxml_opc::relationship::rel_types;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::{Namespace, ResolveResult};
use quick_xml::reader::NsReader;
use rdocx_oxml::content_control::{CT_Sdt, SdtContent};
use rdocx_oxml::document::{BodyContent, CT_Body, CT_SectPr};
use rdocx_oxml::footnotes::{CT_Footnotes, NoteType};
use rdocx_oxml::header_footer::CT_HdrFtr;
use rdocx_oxml::namespace::{W_NS, matches_local_name};
use rdocx_oxml::table::{CT_Row, CT_Tbl, CT_Tc, CellContent};
use rdocx_oxml::text::{CT_P, Field, FieldArgument, FieldInstruction, RunContent};

use crate::{Document, Error, Result, style};

/// A deterministic civil date and time supplied to field evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldDateTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

/// Explicit values that are not stored in the document package.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FieldEvaluationContext {
    pub now: Option<FieldDateTime>,
    pub file_name: Option<String>,
    pub file_path: Option<String>,
    pub merge_fields: BTreeMap<String, String>,
    pub included_text: BTreeMap<String, String>,
}

/// The result of evaluating one field in document order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldEvaluation {
    pub field_index: usize,
    pub instruction: String,
    pub cached_result: String,
    pub outcome: FieldOutcome,
}

/// A pure field-evaluation decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldOutcome {
    Resolved(String),
    DeferredPagination,
    KeepStored { diagnostic: String },
}

impl Document {
    /// Evaluate every typed field without mutating stored results.
    pub fn evaluate_fields(
        &self,
        context: &FieldEvaluationContext,
    ) -> Result<Vec<FieldEvaluation>> {
        let mut evaluator = Evaluator::new(self, context);

        let mut main = Vec::new();
        collect_body_paragraphs(&self.document.body, &mut main);
        evaluator.evaluate_story("main", &main);

        for (part_name, xml) in referenced_header_footer_parts(self, true) {
            if let Ok(part) = CT_HdrFtr::from_xml(&xml) {
                let paragraphs = part.paragraphs.iter().collect::<Vec<_>>();
                evaluator.evaluate_story(&format!("header:{part_name}"), &paragraphs);
            }
        }
        for (part_name, xml) in referenced_header_footer_parts(self, false) {
            if let Ok(part) = CT_HdrFtr::from_xml(&xml) {
                let paragraphs = part.paragraphs.iter().collect::<Vec<_>>();
                evaluator.evaluate_story(&format!("footer:{part_name}"), &paragraphs);
            }
        }

        let footnotes = normal_note_paragraphs(&self.footnotes);
        evaluator.evaluate_story("footnotes", &footnotes);

        for (_, xml) in relationship_parts(self, rel_types::ENDNOTES) {
            if let Ok(part) = CT_Footnotes::from_xml(&xml) {
                let endnotes = normal_note_paragraphs(&part);
                evaluator.evaluate_story("endnotes", &endnotes);
            }
        }

        Ok(evaluator.results)
    }

    /// Evaluate and materialize every typed field cache in document order.
    pub fn update_fields(&mut self, context: &FieldEvaluationContext) -> Result<usize> {
        let evaluations = self.evaluate_fields(context)?;
        let updates = evaluations
            .iter()
            .map(|evaluation| match &evaluation.outcome {
                FieldOutcome::Resolved(value) => CachedFieldUpdate {
                    cached_result: value.clone(),
                    dirty: false,
                },
                FieldOutcome::DeferredPagination | FieldOutcome::KeepStored { .. } => {
                    CachedFieldUpdate {
                        cached_result: evaluation.cached_result.clone(),
                        dirty: true,
                    }
                }
            })
            .collect::<Vec<_>>();
        if updates
            .iter()
            .any(|update| !update.cached_result.chars().all(valid_xml_character))
        {
            return Err(Error::Other(
                "field result contains a character forbidden by XML 1.0".to_owned(),
            ));
        }
        if updates.is_empty() {
            return Ok(0);
        }

        let mut document = self.document.clone();
        let mut footnotes = self.footnotes.clone();
        let mut staged_parts = Vec::new();
        let mut update_index = 0usize;

        apply_updates_to_body(&mut document.body, &updates, &mut update_index);

        for (part_name, xml) in referenced_header_footer_parts(self, true) {
            if let Ok(mut part) = CT_HdrFtr::from_xml(&xml) {
                let part_start = update_index;
                apply_updates_to_paragraphs(&mut part.paragraphs, &updates, &mut update_index);
                if update_index > part_start {
                    let paragraphs = part.paragraphs.iter().collect::<Vec<_>>();
                    let updated = patch_story_field_sources(
                        &xml,
                        &paragraphs,
                        PackageStoryKind::HeaderFooter,
                    )?;
                    CT_HdrFtr::from_xml(&updated)?;
                    staged_parts.push((part_name, updated));
                }
            }
        }
        for (part_name, xml) in referenced_header_footer_parts(self, false) {
            if let Ok(mut part) = CT_HdrFtr::from_xml(&xml) {
                let part_start = update_index;
                apply_updates_to_paragraphs(&mut part.paragraphs, &updates, &mut update_index);
                if update_index > part_start {
                    let paragraphs = part.paragraphs.iter().collect::<Vec<_>>();
                    let updated = patch_story_field_sources(
                        &xml,
                        &paragraphs,
                        PackageStoryKind::HeaderFooter,
                    )?;
                    CT_HdrFtr::from_xml(&updated)?;
                    staged_parts.push((part_name, updated));
                }
            }
        }

        apply_updates_to_notes(&mut footnotes, &updates, &mut update_index);

        for (part_name, xml) in relationship_parts(self, rel_types::ENDNOTES) {
            if let Ok(mut part) = CT_Footnotes::from_xml(&xml) {
                let part_start = update_index;
                apply_updates_to_notes(&mut part, &updates, &mut update_index);
                if update_index > part_start {
                    let paragraphs = normal_note_paragraphs(&part);
                    let updated =
                        patch_story_field_sources(&xml, &paragraphs, PackageStoryKind::Endnotes)?;
                    CT_Footnotes::from_xml(&updated)?;
                    staged_parts.push((part_name, updated));
                }
            }
        }

        if update_index != updates.len() {
            return Err(Error::Other(format!(
                "field update traversal consumed {update_index} of {} staged evaluations",
                updates.len()
            )));
        }

        let document_xml = document.to_xml()?;
        rdocx_oxml::document::CT_Document::from_xml(&document_xml)?;
        if !footnotes.footnotes.is_empty() {
            let footnotes_xml = footnotes.to_xml_footnotes()?;
            CT_Footnotes::from_xml(&footnotes_xml)?;
        }

        self.document = document;
        self.footnotes = footnotes;
        for (part_name, xml) in staged_parts {
            self.package.set_part(&part_name, xml);
        }
        self.invalidate_layout();
        Ok(updates.len())
    }

    /// Update typed field caches, then save the package to a file path.
    pub fn save_with_field_updates<P: AsRef<Path>>(
        &mut self,
        path: P,
        context: &FieldEvaluationContext,
    ) -> Result<()> {
        self.update_fields(context)?;
        self.save(path)
    }

    /// Update typed field caches, then save the package to bytes.
    pub fn to_bytes_with_field_updates(
        &mut self,
        context: &FieldEvaluationContext,
    ) -> Result<Vec<u8>> {
        self.update_fields(context)?;
        self.to_bytes()
    }
}

struct CachedFieldUpdate {
    cached_result: String,
    dirty: bool,
}

fn valid_xml_character(value: char) -> bool {
    matches!(value, '\u{0009}' | '\u{000A}' | '\u{000D}')
        || ('\u{0020}'..='\u{D7FF}').contains(&value)
        || ('\u{E000}'..='\u{FFFD}').contains(&value)
        || ('\u{10000}'..='\u{10FFFF}').contains(&value)
}

#[derive(Debug, Default)]
struct SequenceState {
    value: Option<i64>,
    heading_anchor: Option<usize>,
}

struct Evaluator<'a> {
    document: &'a Document,
    context: &'a FieldEvaluationContext,
    bookmarks: BTreeMap<String, String>,
    results: Vec<FieldEvaluation>,
    sequences: BTreeMap<(String, String), SequenceState>,
    nested_outcomes: Vec<BTreeMap<usize, FieldOutcome>>,
}

impl<'a> Evaluator<'a> {
    fn new(document: &'a Document, context: &'a FieldEvaluationContext) -> Self {
        let bookmarks = document
            .bookmarks()
            .into_iter()
            .filter(|bookmark| bookmark.issue().is_none())
            .filter_map(|bookmark| Some((bookmark.name()?.to_owned(), bookmark.text().to_owned())))
            .collect();
        Self {
            document,
            context,
            bookmarks,
            results: Vec::new(),
            sequences: BTreeMap::new(),
            nested_outcomes: Vec::new(),
        }
    }

    fn evaluate_story(&mut self, story: &str, paragraphs: &[&CT_P]) {
        for (paragraph_index, paragraph) in paragraphs.iter().enumerate() {
            for run in paragraph.runs() {
                for content in &run.content {
                    if let RunContent::Field(field) = content {
                        self.evaluate_field(field, story, paragraphs, paragraph_index);
                    }
                }
            }
        }
    }

    fn evaluate_field(
        &mut self,
        field: &Field,
        story: &str,
        paragraphs: &[&CT_P],
        paragraph_index: usize,
    ) -> FieldOutcome {
        let instruction = field.effective_instruction();
        let result_index = self.results.len();
        self.results.push(FieldEvaluation {
            field_index: result_index,
            instruction: instruction.raw.clone(),
            cached_result: field.cached_result.clone(),
            outcome: keep("field evaluation did not complete"),
        });

        self.nested_outcomes.push(BTreeMap::new());
        self.evaluate_nested_fields(field, &instruction, story, paragraphs, paragraph_index);
        let outcome = self.evaluate_instruction(&instruction, story, paragraphs, paragraph_index);
        self.nested_outcomes.pop();
        self.results[result_index].outcome = outcome.clone();
        outcome
    }

    fn evaluate_instruction(
        &mut self,
        instruction: &FieldInstruction,
        story: &str,
        paragraphs: &[&CT_P],
        paragraph_index: usize,
    ) -> FieldOutcome {
        if instruction.name.is_empty() {
            return keep("field instruction has no name");
        }
        if let Some(name) = unsupported_switch(instruction) {
            return keep(&format!(
                "field {} uses unsupported switch \\{name}",
                instruction.name
            ));
        }
        if let Err(diagnostic) = validate_instruction_shape(instruction) {
            return keep(&diagnostic);
        }

        let outcome = match instruction.name.as_str() {
            "PAGE" | "NUMPAGES" => FieldOutcome::DeferredPagination,
            "PAGEREF" => self.evaluate_pageref(instruction),
            "REF" => self.evaluate_ref(instruction),
            "IF" => self.evaluate_if(instruction, story, paragraphs, paragraph_index),
            "SEQ" => self.evaluate_seq(instruction, story, paragraphs, paragraph_index),
            "DOCPROPERTY" => self.evaluate_docproperty(instruction),
            "DOCVARIABLE" => self.evaluate_docvariable(instruction),
            "STYLEREF" => self.evaluate_styleref(instruction, paragraphs, paragraph_index),
            "INCLUDETEXT" => self.evaluate_includetext(instruction),
            "DATE" | "TIME" => self.evaluate_date_time(instruction),
            "FILENAME" => self.evaluate_filename(instruction),
            "AUTHOR" => self.evaluate_author(),
            "MERGEFIELD" => self.evaluate_mergefield(instruction),
            name => keep(&format!("field {name} is unsupported")),
        };

        match outcome {
            FieldOutcome::Resolved(value) => {
                match apply_formats(instruction, &value, self.context.now) {
                    Ok(value) => FieldOutcome::Resolved(value),
                    Err(diagnostic) => keep(&diagnostic),
                }
            }
            other => other,
        }
    }

    fn evaluate_nested_fields(
        &mut self,
        field: &Field,
        instruction: &FieldInstruction,
        story: &str,
        paragraphs: &[&CT_P],
        paragraph_index: usize,
    ) {
        for nested in field.effective_nested_fields_in_source_order(instruction) {
            let key = std::ptr::from_ref(nested) as usize;
            if !self
                .nested_outcomes
                .last()
                .is_some_and(|outcomes| outcomes.contains_key(&key))
            {
                let outcome = self.evaluate_field(nested, story, paragraphs, paragraph_index);
                self.nested_outcomes
                    .last_mut()
                    .expect("field evaluation frame exists")
                    .insert(key, outcome);
            }
        }
    }

    fn evaluate_ref(&self, instruction: &FieldInstruction) -> FieldOutcome {
        let Some(target) = text_argument(instruction, 0) else {
            return keep("REF requires a bookmark name");
        };
        match self.bookmarks.get(target) {
            Some(text) => FieldOutcome::Resolved(text.clone()),
            None => keep(&format!("REF target {target} was not found")),
        }
    }

    fn evaluate_pageref(&self, instruction: &FieldInstruction) -> FieldOutcome {
        let Some(target) = text_argument(instruction, 0) else {
            return keep("PAGEREF requires a bookmark name");
        };
        if self.bookmarks.contains_key(target) {
            FieldOutcome::DeferredPagination
        } else {
            keep(&format!("PAGEREF target {target} was not found"))
        }
    }

    fn evaluate_if(
        &mut self,
        instruction: &FieldInstruction,
        story: &str,
        paragraphs: &[&CT_P],
        paragraph_index: usize,
    ) -> FieldOutcome {
        if instruction.arguments.len() != 5 {
            return keep("IF requires two operands, an operator, and two results");
        }
        let arguments = instruction
            .arguments
            .iter()
            .map(|argument| self.resolve_argument(argument, story, paragraphs, paragraph_index))
            .collect::<Vec<_>>();
        let left = match &arguments[0] {
            Ok(value) => value,
            Err(diagnostic) => return keep(diagnostic),
        };
        let operator = match &arguments[1] {
            Ok(value) => value,
            Err(diagnostic) => return keep(diagnostic),
        };
        let right = match &arguments[2] {
            Ok(value) => value,
            Err(diagnostic) => return keep(diagnostic),
        };
        let Some(condition) = compare_if(left, operator, right) else {
            return keep(&format!("IF operator {operator} is unsupported"));
        };
        let selected = if condition { 3 } else { 4 };
        match &arguments[selected] {
            Ok(value) => FieldOutcome::Resolved(value.clone()),
            Err(diagnostic) => keep(diagnostic),
        }
    }

    fn resolve_argument(
        &mut self,
        argument: &FieldArgument,
        story: &str,
        paragraphs: &[&CT_P],
        paragraph_index: usize,
    ) -> std::result::Result<String, String> {
        match argument {
            FieldArgument::Text(value) => Ok(value.clone()),
            FieldArgument::Nested(field) => {
                let key = std::ptr::from_ref(field.as_ref()) as usize;
                let outcome = if let Some(outcome) = self
                    .nested_outcomes
                    .last()
                    .and_then(|outcomes| outcomes.get(&key))
                {
                    outcome.clone()
                } else {
                    let outcome = self.evaluate_field(field, story, paragraphs, paragraph_index);
                    self.nested_outcomes
                        .last_mut()
                        .expect("field evaluation frame exists")
                        .insert(key, outcome.clone());
                    outcome
                };
                match outcome {
                    FieldOutcome::Resolved(value) => Ok(value),
                    FieldOutcome::DeferredPagination => {
                        Err("nested field requires deferred pagination".to_owned())
                    }
                    FieldOutcome::KeepStored { diagnostic } => {
                        Err(format!("nested field was not resolved: {diagnostic}"))
                    }
                }
            }
        }
    }

    fn evaluate_seq(
        &mut self,
        instruction: &FieldInstruction,
        story: &str,
        paragraphs: &[&CT_P],
        paragraph_index: usize,
    ) -> FieldOutcome {
        let Some(identifier) = text_argument(instruction, 0) else {
            return keep("SEQ requires an identifier");
        };
        let heading_anchor = switch_text(instruction, "s").and_then(|level| {
            let level = level.parse::<u32>().ok()?.checked_sub(1)?;
            paragraphs[..=paragraph_index]
                .iter()
                .enumerate()
                .rev()
                .find(|(_, paragraph)| {
                    let style_id = paragraph
                        .properties
                        .as_ref()
                        .and_then(|properties| properties.style_id.as_deref());
                    let mut effective =
                        style::resolve_paragraph_properties(style_id, &self.document.styles);
                    if let Some(properties) = &paragraph.properties {
                        effective.merge_from(properties);
                    }
                    effective.outline_lvl == Some(level)
                })
                .map(|(index, _)| index)
        });
        if has_switch(instruction, "s") && heading_anchor.is_none() {
            return keep("SEQ heading restart has no matching heading");
        }

        let state = self
            .sequences
            .entry((story.to_owned(), identifier.to_ascii_lowercase()))
            .or_default();
        if heading_anchor.is_some() && state.heading_anchor != heading_anchor {
            state.value = None;
            state.heading_anchor = heading_anchor;
        }

        let value = if let Some(reset) = switch_text(instruction, "r") {
            let Ok(reset) = reset.parse::<i64>() else {
                return keep("SEQ reset value is not an integer");
            };
            state.value = Some(reset);
            reset
        } else if has_switch(instruction, "c") {
            let Some(value) = state.value else {
                return keep("SEQ repeat has no preceding value");
            };
            value
        } else {
            let Some(value) = state.value.unwrap_or(0).checked_add(1) else {
                return keep("SEQ value overflowed");
            };
            state.value = Some(value);
            value
        };
        if has_switch(instruction, "h") {
            FieldOutcome::Resolved(String::new())
        } else {
            FieldOutcome::Resolved(value.to_string())
        }
    }

    fn evaluate_docproperty(&self, instruction: &FieldInstruction) -> FieldOutcome {
        let Some(name) = text_argument(instruction, 0) else {
            return keep("DOCPROPERTY requires a property name");
        };
        if let Some(value) = self.core_property(name) {
            return FieldOutcome::Resolved(value.to_owned());
        }
        let matches = self
            .document
            .custom_properties
            .iter()
            .flat_map(|properties| &properties.properties)
            .filter(|property| {
                property
                    .name
                    .as_deref()
                    .is_some_and(|candidate| candidate.eq_ignore_ascii_case(name))
            })
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return keep(&format!(
                "DOCPROPERTY target {name} was not found or is ambiguous"
            ));
        }
        match custom_property_text(&matches[0].value) {
            Some(value) => FieldOutcome::Resolved(value),
            None => keep(&format!(
                "DOCPROPERTY target {name} has an unsupported value"
            )),
        }
    }

    fn core_property(&self, name: &str) -> Option<&str> {
        let properties = self.document.core_properties.as_ref()?;
        match normalized_name(name).as_str() {
            "title" => properties.title.as_deref(),
            "subject" => properties.subject.as_deref(),
            "author" | "creator" => properties.creator.as_deref(),
            "comments" | "description" => properties.description.as_deref(),
            "keywords" => properties.keywords.as_deref(),
            "lastsavedby" | "lastmodifiedby" => properties.last_modified_by.as_deref(),
            "createtime" | "created" => properties.created.as_deref(),
            "savedate" | "modified" => properties.modified.as_deref(),
            _ => None,
        }
    }

    fn evaluate_docvariable(&self, instruction: &FieldInstruction) -> FieldOutcome {
        let Some(name) = text_argument(instruction, 0) else {
            return keep("DOCVARIABLE requires a variable name");
        };
        let matches = self
            .document
            .settings
            .iter()
            .flat_map(|settings| settings.document_variables())
            .filter(|variable| variable.name.eq_ignore_ascii_case(name))
            .collect::<Vec<_>>();
        if matches.len() == 1 {
            FieldOutcome::Resolved(matches[0].value.clone())
        } else {
            keep(&format!(
                "DOCVARIABLE target {name} was not found or is ambiguous"
            ))
        }
    }

    fn evaluate_styleref(
        &self,
        instruction: &FieldInstruction,
        paragraphs: &[&CT_P],
        paragraph_index: usize,
    ) -> FieldOutcome {
        let Some(target) = text_argument(instruction, 0) else {
            return keep("STYLEREF requires a style name");
        };
        let style_ids = self
            .document
            .styles
            .styles
            .iter()
            .filter(|style| {
                style.style_id.eq_ignore_ascii_case(target)
                    || style
                        .name
                        .as_deref()
                        .is_some_and(|name| name.eq_ignore_ascii_case(target))
            })
            .map(|style| style.style_id.as_str())
            .collect::<Vec<_>>();
        let matches = |paragraph: &&CT_P| {
            paragraph
                .properties
                .as_ref()
                .and_then(|properties| properties.style_id.as_deref())
                .is_some_and(|id| style_ids.contains(&id))
        };
        let source = if has_switch(instruction, "l") {
            paragraphs
                .iter()
                .enumerate()
                .rev()
                .find(|(index, paragraph)| *index != paragraph_index && matches(paragraph))
        } else {
            paragraphs[..paragraph_index]
                .iter()
                .enumerate()
                .rev()
                .find(|(_, paragraph)| matches(paragraph))
                .or_else(|| {
                    paragraphs[paragraph_index + 1..]
                        .iter()
                        .enumerate()
                        .find(|(_, paragraph)| matches(paragraph))
                        .map(|(index, paragraph)| (paragraph_index + 1 + index, paragraph))
                })
        };
        let Some((source_index, source)) = source else {
            return keep(&format!("STYLEREF target {target} was not found"));
        };
        let mut effective = style::resolve_paragraph_properties(
            source
                .properties
                .as_ref()
                .and_then(|properties| properties.style_id.as_deref()),
            &self.document.styles,
        );
        if let Some(properties) = source.properties.as_ref() {
            effective.merge_from(properties);
        }
        if ["n", "r", "t", "w"]
            .iter()
            .any(|name| has_switch(instruction, name))
            && effective.num_id.is_some_and(|num_id| num_id != 0)
        {
            return keep("STYLEREF numbered source formatting is unsupported");
        }
        let mut value = source.text();
        if has_switch(instruction, "p") {
            value.push_str(if source_index < paragraph_index {
                " above"
            } else {
                " below"
            });
        }
        FieldOutcome::Resolved(value)
    }

    fn evaluate_includetext(&self, instruction: &FieldInstruction) -> FieldOutcome {
        if has_switch(instruction, "c") {
            return keep("INCLUDETEXT converter selection is unsupported");
        }
        let Some(source) = text_argument(instruction, 0) else {
            return keep("INCLUDETEXT requires a source name");
        };
        let key = text_argument(instruction, 1)
            .map(|bookmark| format!("{source}#{bookmark}"))
            .unwrap_or_else(|| source.to_owned());
        match self.context.included_text.get(&key) {
            Some(value) => FieldOutcome::Resolved(value.clone()),
            None => keep(&format!("INCLUDETEXT input {key} was not supplied")),
        }
    }

    fn evaluate_date_time(&self, instruction: &FieldInstruction) -> FieldOutcome {
        let Some(now) = self.context.now else {
            return keep(&format!(
                "{} requires an explicit date and time",
                instruction.name
            ));
        };
        if !valid_date_time(now) {
            return keep("field date and time is invalid");
        }
        let default_picture = if instruction.name == "DATE" {
            "M/d/yyyy"
        } else {
            "h:mm:ss AM/PM"
        };
        let picture = switch_text(instruction, "@").unwrap_or(default_picture);
        match format_date_time(now, picture) {
            Ok(value) => FieldOutcome::Resolved(value),
            Err(diagnostic) => keep(&diagnostic),
        }
    }

    fn evaluate_filename(&self, instruction: &FieldInstruction) -> FieldOutcome {
        if has_switch(instruction, "p") {
            return self
                .context
                .file_path
                .clone()
                .map(FieldOutcome::Resolved)
                .unwrap_or_else(|| keep("FILENAME path was not supplied"));
        }
        self.context
            .file_name
            .clone()
            .or_else(|| {
                self.context
                    .file_path
                    .as_deref()
                    .and_then(lexical_file_name)
                    .map(str::to_owned)
            })
            .map(FieldOutcome::Resolved)
            .unwrap_or_else(|| keep("FILENAME name was not supplied"))
    }

    fn evaluate_author(&self) -> FieldOutcome {
        self.document
            .core_properties
            .as_ref()
            .and_then(|properties| properties.creator.clone())
            .map(FieldOutcome::Resolved)
            .unwrap_or_else(|| keep("AUTHOR metadata was not found"))
    }

    fn evaluate_mergefield(&self, instruction: &FieldInstruction) -> FieldOutcome {
        let Some(name) = text_argument(instruction, 0) else {
            return keep("MERGEFIELD requires a field name");
        };
        let Some(value) = self.context.merge_fields.get(name) else {
            return keep(&format!("MERGEFIELD input {name} was not supplied"));
        };
        if value.is_empty() {
            return FieldOutcome::Resolved(String::new());
        }
        let mut result = String::new();
        if let Some(prefix) = switch_text(instruction, "b") {
            result.push_str(prefix);
        }
        result.push_str(value);
        if let Some(suffix) = switch_text(instruction, "f") {
            result.push_str(suffix);
        }
        FieldOutcome::Resolved(result)
    }
}

fn referenced_header_footer_parts(document: &Document, is_header: bool) -> Vec<(String, Vec<u8>)> {
    let Some(relationships) = document.package.get_part_rels(&document.doc_part_name) else {
        return Vec::new();
    };
    let relationship_type = if is_header {
        rel_types::HEADER
    } else {
        rel_types::FOOTER
    };
    let sections = document
        .document
        .body
        .content
        .iter()
        .filter_map(|content| match content {
            BodyContent::Paragraph(paragraph) => paragraph
                .properties
                .as_ref()
                .and_then(|properties| properties.sect_pr.as_ref()),
            BodyContent::Table(_) | BodyContent::ContentControl(_) | BodyContent::RawXml(_) => None,
        })
        .chain(document.document.body.sect_pr.iter());
    let mut seen = HashSet::new();
    let mut parts = Vec::new();
    for section in sections {
        let references = section_header_footer_references(section, is_header);
        for reference in references {
            let Some(relationship) = relationships.get_by_id(&reference.rel_id) else {
                continue;
            };
            if relationship.rel_type != relationship_type
                || relationship.target_mode.as_deref() == Some("External")
            {
                continue;
            }
            let part_name =
                OpcPackage::resolve_rel_target(&document.doc_part_name, &relationship.target);
            if seen.insert(part_name.clone())
                && let Some(xml) = document.package.get_part(&part_name)
            {
                parts.push((part_name, xml.to_vec()));
            }
        }
    }
    parts
}

fn section_header_footer_references(
    section: &CT_SectPr,
    is_header: bool,
) -> &[rdocx_oxml::header_footer::HdrFtrRef] {
    if is_header {
        &section.header_refs
    } else {
        &section.footer_refs
    }
}

fn relationship_parts(document: &Document, relationship_type: &str) -> Vec<(String, Vec<u8>)> {
    let Some(relationships) = document.package.get_part_rels(&document.doc_part_name) else {
        return Vec::new();
    };
    let mut seen = HashSet::new();
    relationships
        .items
        .iter()
        .filter(|relationship| relationship.rel_type == relationship_type)
        .filter(|relationship| relationship.target_mode.as_deref() != Some("External"))
        .filter_map(|relationship| {
            let part_name =
                OpcPackage::resolve_rel_target(&document.doc_part_name, &relationship.target);
            if !seen.insert(part_name.clone()) {
                return None;
            }
            Some((
                part_name.clone(),
                document.package.get_part(&part_name)?.to_vec(),
            ))
        })
        .collect()
}

fn normal_note_paragraphs(notes: &CT_Footnotes) -> Vec<&CT_P> {
    notes
        .footnotes
        .iter()
        .filter(|note| note.note_type == NoteType::Normal)
        .flat_map(|note| note.paragraphs.iter())
        .collect()
}

#[derive(Clone, Copy)]
enum PackageStoryKind {
    HeaderFooter,
    Endnotes,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StoryElementKind {
    HeaderFooterRoot,
    EndnotesRoot,
    NormalNote,
    Paragraph,
    Other,
}

struct StoryElement {
    kind: StoryElementKind,
    start: usize,
}

struct FieldSourceEdit {
    start: usize,
    end: usize,
    replacement: Vec<u8>,
}

fn patch_story_field_sources(
    xml: &[u8],
    paragraphs: &[&CT_P],
    story_kind: PackageStoryKind,
) -> Result<Vec<u8>> {
    let paragraph_spans = story_paragraph_spans(xml, story_kind)?;
    if paragraph_spans.len() != paragraphs.len() {
        return Err(Error::Other(format!(
            "package story paragraph scan found {} of {} typed paragraphs",
            paragraph_spans.len(),
            paragraphs.len()
        )));
    }
    let mut edits = Vec::new();
    for (paragraph, (paragraph_start, paragraph_end)) in paragraphs.iter().zip(paragraph_spans) {
        let mut search_start = 0usize;
        for run in paragraph.runs() {
            for content in &run.content {
                let RunContent::Field(field) = content else {
                    continue;
                };
                let Some((source, replacement)) = field.source_replacement()? else {
                    return Err(Error::Other(
                        "parsed package story field has no source XML".to_owned(),
                    ));
                };
                let Some(start) = find_typed_field_source(
                    xml,
                    paragraph_start,
                    paragraph_end,
                    source,
                    search_start,
                )?
                else {
                    return Err(Error::Other(
                        "package story field source was not found at its typed paragraph boundary"
                            .to_owned(),
                    ));
                };
                let end = start + source.len();
                edits.push(FieldSourceEdit {
                    start: paragraph_start + start,
                    end: paragraph_start + end,
                    replacement,
                });
                search_start = end;
            }
        }
    }

    let mut updated = xml.to_vec();
    for edit in edits.into_iter().rev() {
        updated.splice(edit.start..edit.end, edit.replacement);
    }
    Ok(updated)
}

fn story_paragraph_spans(xml: &[u8], story_kind: PackageStoryKind) -> Result<Vec<(usize, usize)>> {
    let mut reader = NsReader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut buffer = Vec::new();
    let mut stack = Vec::<StoryElement>::new();
    let mut paragraphs = Vec::new();
    loop {
        let before = reader.buffer_position() as usize;
        let (namespace, event) = reader
            .read_resolved_event_into(&mut buffer)
            .map_err(|error| Error::Other(format!("invalid package story XML: {error}")))?;
        let word = namespace_is_word(&namespace);
        match event {
            Event::Start(element) => {
                let parent = stack.last().map(|element| element.kind);
                let kind = match story_kind {
                    PackageStoryKind::HeaderFooter
                        if stack.is_empty()
                            && word
                            && (matches_local_name(element.name().as_ref(), b"hdr")
                                || matches_local_name(element.name().as_ref(), b"ftr")) =>
                    {
                        StoryElementKind::HeaderFooterRoot
                    }
                    PackageStoryKind::Endnotes
                        if stack.is_empty()
                            && word
                            && matches_local_name(element.name().as_ref(), b"endnotes") =>
                    {
                        StoryElementKind::EndnotesRoot
                    }
                    PackageStoryKind::Endnotes
                        if parent == Some(StoryElementKind::EndnotesRoot)
                            && word
                            && matches_local_name(element.name().as_ref(), b"endnote")
                            && note_is_normal(&element) =>
                    {
                        StoryElementKind::NormalNote
                    }
                    PackageStoryKind::HeaderFooter
                        if parent == Some(StoryElementKind::HeaderFooterRoot)
                            && word
                            && matches_local_name(element.name().as_ref(), b"p") =>
                    {
                        StoryElementKind::Paragraph
                    }
                    PackageStoryKind::Endnotes
                        if parent == Some(StoryElementKind::NormalNote)
                            && word
                            && matches_local_name(element.name().as_ref(), b"p") =>
                    {
                        StoryElementKind::Paragraph
                    }
                    _ => StoryElementKind::Other,
                };
                stack.push(StoryElement {
                    kind,
                    start: before,
                });
            }
            Event::End(_) => {
                let Some(element) = stack.pop() else {
                    return Err(Error::Other(
                        "package story XML has an unmatched end element".to_owned(),
                    ));
                };
                if element.kind == StoryElementKind::Paragraph {
                    paragraphs.push((element.start, reader.buffer_position() as usize));
                }
            }
            Event::Eof => {
                if !stack.is_empty() {
                    return Err(Error::Other(
                        "package story XML has an unclosed element".to_owned(),
                    ));
                }
                return Ok(paragraphs);
            }
            _ => {}
        }
        buffer.clear();
    }
}

fn note_is_normal(element: &BytesStart<'_>) -> bool {
    let mut id = 0i32;
    let mut note_type = None;
    for attribute in element.attributes().flatten() {
        if matches_local_name(attribute.key.as_ref(), b"id") {
            id = std::str::from_utf8(&attribute.value)
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
        } else if matches_local_name(attribute.key.as_ref(), b"type") {
            note_type = Some(String::from_utf8_lossy(&attribute.value).into_owned());
        }
    }
    !matches!(
        note_type.as_deref(),
        Some("separator" | "continuationSeparator" | "continuationNotice")
    ) && id > 0
}

fn find_typed_field_source(
    xml: &[u8],
    paragraph_start: usize,
    paragraph_end: usize,
    source: &[u8],
    search_start: usize,
) -> Result<Option<usize>> {
    let paragraph_xml = &xml[paragraph_start..paragraph_end];
    let mut candidate_start = search_start;
    while let Some(relative) = find_bytes(&paragraph_xml[candidate_start..], source) {
        let candidate = candidate_start + relative;
        if field_source_has_typed_ancestors(xml, paragraph_start + candidate, paragraph_start)? {
            return Ok(Some(candidate));
        }
        candidate_start = candidate + source.len().max(1);
    }
    Ok(None)
}

fn field_source_has_typed_ancestors(
    xml: &[u8],
    source_start: usize,
    paragraph_start: usize,
) -> Result<bool> {
    let mut reader = NsReader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut buffer = Vec::new();
    let mut ancestors = Vec::<(usize, bool, Vec<u8>)>::new();
    loop {
        let before = reader.buffer_position() as usize;
        let (namespace, event) = reader
            .read_resolved_event_into(&mut buffer)
            .map_err(|error| Error::Other(format!("invalid package paragraph XML: {error}")))?;
        let word = namespace_is_word(&namespace);
        match event {
            Event::Start(element) => {
                if before == source_start {
                    let name = element.name();
                    let local = local_name(name.as_ref());
                    return Ok(word
                        && matches!(local, b"fldSimple" | b"r")
                        && typed_field_ancestors(&ancestors, paragraph_start));
                }
                ancestors.push((before, word, local_name(element.name().as_ref()).to_vec()));
            }
            Event::Empty(element) => {
                if before == source_start {
                    let name = element.name();
                    let local = local_name(name.as_ref());
                    return Ok(word
                        && matches!(local, b"fldSimple" | b"r")
                        && typed_field_ancestors(&ancestors, paragraph_start));
                }
            }
            Event::End(_) => {
                ancestors.pop();
            }
            Event::Eof => return Ok(false),
            _ => {}
        }
        buffer.clear();
    }
}

fn typed_field_ancestors(ancestors: &[(usize, bool, Vec<u8>)], paragraph_start: usize) -> bool {
    let Some(paragraph_index) = ancestors
        .iter()
        .position(|(start, _, _)| *start == paragraph_start)
    else {
        return false;
    };
    let (_, paragraph_word, paragraph) = &ancestors[paragraph_index];
    *paragraph_word
        && paragraph.as_slice() == b"p"
        && ancestors
            .iter()
            .skip(paragraph_index + 1)
            .all(|(_, word, local)| {
                *word
                    && matches!(
                        local.as_slice(),
                        b"hyperlink"
                            | b"sdt"
                            | b"sdtContent"
                            | b"ins"
                            | b"del"
                            | b"moveFrom"
                            | b"moveTo"
                    )
            })
}

fn namespace_is_word(namespace: &ResolveResult<'_>) -> bool {
    matches!(namespace, ResolveResult::Bound(Namespace(uri)) if *uri == W_NS.as_bytes())
}

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        None
    } else {
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }
}

fn apply_updates_to_notes(
    notes: &mut CT_Footnotes,
    updates: &[CachedFieldUpdate],
    update_index: &mut usize,
) {
    for note in &mut notes.footnotes {
        if note.note_type == NoteType::Normal {
            apply_updates_to_paragraphs(&mut note.paragraphs, updates, update_index);
        }
    }
}

fn apply_updates_to_paragraphs(
    paragraphs: &mut [CT_P],
    updates: &[CachedFieldUpdate],
    update_index: &mut usize,
) {
    for paragraph in paragraphs {
        apply_updates_to_paragraph(paragraph, updates, update_index);
    }
}

fn apply_updates_to_body(
    body: &mut CT_Body,
    updates: &[CachedFieldUpdate],
    update_index: &mut usize,
) {
    for content in &mut body.content {
        match content {
            BodyContent::Paragraph(paragraph) => {
                apply_updates_to_paragraph(paragraph, updates, update_index);
            }
            BodyContent::Table(table) => apply_updates_to_table(table, updates, update_index),
            BodyContent::ContentControl(control) => {
                apply_updates_to_block_control(control, updates, update_index);
            }
            BodyContent::RawXml(_) => {}
        }
    }
}

fn apply_updates_to_table(
    table: &mut CT_Tbl,
    updates: &[CachedFieldUpdate],
    update_index: &mut usize,
) {
    for boundary in 0..=table.rows.len() {
        for (_, _, control) in table
            .content_controls
            .iter_mut()
            .filter(|(position, _, _)| *position == boundary)
        {
            apply_updates_to_block_control(control, updates, update_index);
        }
        if let Some(row) = table.rows.get_mut(boundary) {
            apply_updates_to_row(row, updates, update_index);
        }
    }
}

fn apply_updates_to_row(row: &mut CT_Row, updates: &[CachedFieldUpdate], update_index: &mut usize) {
    for boundary in 0..=row.cells.len() {
        for (_, _, control) in row
            .content_controls
            .iter_mut()
            .filter(|(position, _, _)| *position == boundary)
        {
            apply_updates_to_block_control(control, updates, update_index);
        }
        if let Some(cell) = row.cells.get_mut(boundary) {
            apply_updates_to_cell(cell, updates, update_index);
        }
    }
}

fn apply_updates_to_cell(
    cell: &mut CT_Tc,
    updates: &[CachedFieldUpdate],
    update_index: &mut usize,
) {
    for content in &mut cell.content {
        match content {
            CellContent::Paragraph(paragraph) => {
                apply_updates_to_paragraph(paragraph, updates, update_index);
            }
            CellContent::Table(table) => apply_updates_to_table(table, updates, update_index),
            CellContent::ContentControl(control) => {
                apply_updates_to_block_control(control, updates, update_index);
            }
        }
    }
}

fn apply_updates_to_block_control(
    control: &mut CT_Sdt,
    updates: &[CachedFieldUpdate],
    update_index: &mut usize,
) {
    for content in &mut control.content {
        match content {
            SdtContent::Paragraph(paragraph) => {
                apply_updates_to_paragraph(paragraph, updates, update_index);
            }
            SdtContent::Table(table) => apply_updates_to_table(table, updates, update_index),
            SdtContent::Row(row) => apply_updates_to_row(row, updates, update_index),
            SdtContent::Cell(cell) => apply_updates_to_cell(cell, updates, update_index),
            SdtContent::ContentControl(control) => {
                apply_updates_to_block_control(control, updates, update_index);
            }
            SdtContent::Run(_) | SdtContent::RawXml(_) => {}
        }
    }
}

fn apply_updates_to_run_control(
    control: &mut CT_Sdt,
    updates: &[CachedFieldUpdate],
    update_index: &mut usize,
) {
    for content in &mut control.content {
        match content {
            SdtContent::Run(run) => apply_updates_to_run(run, updates, update_index),
            SdtContent::Paragraph(paragraph) => {
                apply_updates_to_paragraph(paragraph, updates, update_index);
            }
            SdtContent::ContentControl(control) => {
                apply_updates_to_run_control(control, updates, update_index);
            }
            SdtContent::Table(_)
            | SdtContent::Row(_)
            | SdtContent::Cell(_)
            | SdtContent::RawXml(_) => {}
        }
    }
}

fn apply_updates_to_paragraph(
    paragraph: &mut CT_P,
    updates: &[CachedFieldUpdate],
    update_index: &mut usize,
) {
    for boundary in 0..=paragraph.runs.len() {
        for (_, _, _, control) in paragraph
            .content_controls
            .iter_mut()
            .filter(|(position, _, _, _)| *position == boundary)
        {
            apply_updates_to_run_control(control, updates, update_index);
        }
        if let Some(run) = paragraph.runs.get_mut(boundary) {
            apply_updates_to_run(run, updates, update_index);
        }
    }
}

fn apply_updates_to_run(
    run: &mut rdocx_oxml::text::CT_R,
    updates: &[CachedFieldUpdate],
    update_index: &mut usize,
) {
    for content in &mut run.content {
        if let RunContent::Field(field) = content {
            apply_updates_to_field(field, updates, update_index);
        }
    }
}

fn apply_updates_to_field(
    field: &mut Field,
    updates: &[CachedFieldUpdate],
    update_index: &mut usize,
) {
    let Some(update) = updates.get(*update_index) else {
        return;
    };
    field.cached_result.clone_from(&update.cached_result);
    field.dirty = Some(update.dirty);
    *update_index += 1;

    let nested_pointers = field
        .nested_fields_in_source_order()
        .into_iter()
        .map(|nested| std::ptr::from_ref(nested) as usize)
        .collect::<Vec<_>>();
    for pointer in nested_pointers {
        if let Some(nested) = nested_field_mut(field, pointer) {
            apply_updates_to_field(nested, updates, update_index);
        }
    }
}

fn nested_field_mut(field: &mut Field, pointer: usize) -> Option<&mut Field> {
    for argument in &mut field.instruction.arguments {
        if let FieldArgument::Nested(nested) = argument
            && std::ptr::from_ref(nested.as_ref()) as usize == pointer
        {
            return Some(nested.as_mut());
        }
    }
    for field_switch in &mut field.instruction.switches {
        if let Some(FieldArgument::Nested(nested)) = &mut field_switch.argument
            && std::ptr::from_ref(nested.as_ref()) as usize == pointer
        {
            return Some(nested.as_mut());
        }
    }
    None
}

fn collect_body_paragraphs<'a>(body: &'a CT_Body, output: &mut Vec<&'a CT_P>) {
    for content in &body.content {
        match content {
            BodyContent::Paragraph(paragraph) => output.push(paragraph),
            BodyContent::Table(table) => collect_table_paragraphs(table, output),
            BodyContent::ContentControl(control) => collect_control_paragraphs(control, output),
            BodyContent::RawXml(_) => {}
        }
    }
}

fn collect_table_paragraphs<'a>(table: &'a CT_Tbl, output: &mut Vec<&'a CT_P>) {
    for boundary in 0..=table.rows.len() {
        for (_, _, control) in table
            .content_controls
            .iter()
            .filter(|(position, _, _)| *position == boundary)
        {
            collect_control_paragraphs(control, output);
        }
        if let Some(row) = table.rows.get(boundary) {
            collect_row_paragraphs(row, output);
        }
    }
}

fn collect_row_paragraphs<'a>(row: &'a CT_Row, output: &mut Vec<&'a CT_P>) {
    for boundary in 0..=row.cells.len() {
        for (_, _, control) in row
            .content_controls
            .iter()
            .filter(|(position, _, _)| *position == boundary)
        {
            collect_control_paragraphs(control, output);
        }
        if let Some(cell) = row.cells.get(boundary) {
            collect_cell_paragraphs(cell, output);
        }
    }
}

fn collect_cell_paragraphs<'a>(cell: &'a CT_Tc, output: &mut Vec<&'a CT_P>) {
    for content in &cell.content {
        match content {
            CellContent::Paragraph(paragraph) => output.push(paragraph),
            CellContent::Table(table) => collect_table_paragraphs(table, output),
            CellContent::ContentControl(control) => collect_control_paragraphs(control, output),
        }
    }
}

fn collect_control_paragraphs<'a>(control: &'a CT_Sdt, output: &mut Vec<&'a CT_P>) {
    for content in &control.content {
        match content {
            SdtContent::Paragraph(paragraph) => output.push(paragraph),
            SdtContent::Table(table) => collect_table_paragraphs(table, output),
            SdtContent::Row(row) => collect_row_paragraphs(row, output),
            SdtContent::Cell(cell) => collect_cell_paragraphs(cell, output),
            SdtContent::ContentControl(control) => collect_control_paragraphs(control, output),
            SdtContent::Run(_) | SdtContent::RawXml(_) => {}
        }
    }
}

fn keep(diagnostic: &str) -> FieldOutcome {
    FieldOutcome::KeepStored {
        diagnostic: format!("{diagnostic}, stored display retained"),
    }
}

fn text_argument(instruction: &FieldInstruction, index: usize) -> Option<&str> {
    instruction.arguments.get(index).and_then(argument_text)
}

fn argument_text(argument: &FieldArgument) -> Option<&str> {
    match argument {
        FieldArgument::Text(value) => Some(value),
        FieldArgument::Nested(_) => None,
    }
}

fn has_switch(instruction: &FieldInstruction, name: &str) -> bool {
    instruction
        .switches
        .iter()
        .any(|switch| switch.name == name)
}

fn switch_text<'a>(instruction: &'a FieldInstruction, name: &str) -> Option<&'a str> {
    instruction.switches.iter().find_map(|switch| {
        (switch.name == name)
            .then_some(switch.argument.as_ref())
            .flatten()
            .and_then(argument_text)
    })
}

fn unsupported_switch(instruction: &FieldInstruction) -> Option<&str> {
    let allowed: &[&str] = match instruction.name.as_str() {
        "PAGE" | "NUMPAGES" => &["*", "#"],
        "REF" => &["h", "*", "#"],
        "PAGEREF" => &["h", "p", "*", "#"],
        "IF" => &["*", "#"],
        "SEQ" => &["n", "c", "h", "r", "s", "*", "#"],
        "DOCPROPERTY" | "DOCVARIABLE" => &["*", "#", "@"],
        "STYLEREF" => &["l", "n", "r", "t", "w", "p", "*", "#"],
        "INCLUDETEXT" => &["!", "c", "*", "#"],
        "DATE" | "TIME" => &["@", "*"],
        "FILENAME" => &["p", "*"],
        "AUTHOR" => &["*"],
        "MERGEFIELD" => &["b", "f", "m", "v", "*", "#", "@"],
        _ => return None,
    };
    instruction
        .switches
        .iter()
        .find(|switch| !allowed.contains(&switch.name.as_str()))
        .map(|switch| switch.name.as_str())
}

fn validate_instruction_shape(instruction: &FieldInstruction) -> std::result::Result<(), String> {
    if !instruction_quotes_are_balanced(&instruction.raw) {
        return Err(format!("field {} has unclosed quoting", instruction.name));
    }
    let argument_range = match instruction.name.as_str() {
        "PAGE" | "NUMPAGES" | "DATE" | "TIME" | "FILENAME" | "AUTHOR" => 0..=0,
        "REF" | "PAGEREF" | "SEQ" | "DOCPROPERTY" | "DOCVARIABLE" | "STYLEREF" | "MERGEFIELD" => {
            1..=1
        }
        "IF" => 5..=5,
        "INCLUDETEXT" => 1..=2,
        _ => return Ok(()),
    };
    if !argument_range.contains(&instruction.arguments.len()) {
        let expected = if argument_range.start() == argument_range.end() {
            argument_range.start().to_string()
        } else {
            format!(
                "{} through {}",
                argument_range.start(),
                argument_range.end()
            )
        };
        return Err(format!(
            "field {} requires {expected} positional operands",
            instruction.name
        ));
    }

    for switch in &instruction.switches {
        let requires_text = matches!(
            switch.name.as_str(),
            "*" | "#" | "@" | "r" | "s" | "b" | "f"
        ) || instruction.name == "INCLUDETEXT" && switch.name == "c";
        match (&switch.argument, requires_text) {
            (Some(FieldArgument::Text(_)), true) | (None, false) => {}
            (_, true) => {
                return Err(format!(
                    "field {} switch \\{} requires a text argument",
                    instruction.name, switch.name
                ));
            }
            (Some(_), false) => {
                return Err(format!(
                    "field {} switch \\{} does not take an argument",
                    instruction.name, switch.name
                ));
            }
        }
    }
    Ok(())
}

fn instruction_quotes_are_balanced(raw: &str) -> bool {
    let mut characters = raw.chars().peekable();
    let mut quoted = false;
    while let Some(character) = characters.next() {
        if quoted
            && character == '\\'
            && characters
                .peek()
                .is_some_and(|next| matches!(next, '"' | '\\'))
        {
            characters.next();
        } else if character == '"' {
            quoted = !quoted;
        }
    }
    !quoted
}

fn compare_if(left: &str, operator: &str, right: &str) -> Option<bool> {
    let numeric = left
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .zip(right.parse::<f64>().ok().filter(|value| value.is_finite()));
    let ordering = numeric
        .map(|(left, right)| left.total_cmp(&right))
        .unwrap_or_else(|| left.to_lowercase().cmp(&right.to_lowercase()));
    match operator {
        "=" => Some(if right.contains(['*', '?']) {
            wildcard_matches(right, left)
        } else {
            ordering.is_eq()
        }),
        "<>" => Some(if right.contains(['*', '?']) {
            !wildcard_matches(right, left)
        } else {
            !ordering.is_eq()
        }),
        "<" => Some(ordering.is_lt()),
        "<=" => Some(!ordering.is_gt()),
        ">" => Some(ordering.is_gt()),
        ">=" => Some(!ordering.is_lt()),
        _ => None,
    }
}

fn wildcard_matches(pattern: &str, value: &str) -> bool {
    let mut expression = String::from("(?is)^");
    for character in pattern.chars() {
        match character {
            '*' => expression.push_str(".*"),
            '?' => expression.push('.'),
            other => expression.push_str(&regex::escape(&other.to_string())),
        }
    }
    expression.push('$');
    regex::Regex::new(&expression).is_ok_and(|regex| regex.is_match(value))
}

fn custom_property_text(value: &CustomPropertyValue) -> Option<String> {
    match value {
        CustomPropertyValue::Lpstr(value)
        | CustomPropertyValue::Lpwstr(value)
        | CustomPropertyValue::FileTime(value) => Some(value.clone()),
        CustomPropertyValue::I4(value) => Some(value.to_string()),
        CustomPropertyValue::R8(value) if value.is_finite() => Some(value.to_string()),
        CustomPropertyValue::Bool(value) => Some(value.to_string()),
        CustomPropertyValue::Empty => Some(String::new()),
        CustomPropertyValue::R8(_) | CustomPropertyValue::Raw(_) => None,
    }
}

fn normalized_name(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn lexical_file_name(path: &str) -> Option<&str> {
    path.rsplit(['/', '\\'])
        .find(|component| !component.is_empty())
}

fn apply_formats(
    instruction: &FieldInstruction,
    value: &str,
    date_time: Option<FieldDateTime>,
) -> std::result::Result<String, String> {
    let mut output = value.to_owned();
    if let Some(picture) = switch_text(instruction, "#") {
        let number = value
            .parse::<f64>()
            .ok()
            .filter(|number| number.is_finite())
            .ok_or_else(|| "numeric field value is not a finite number".to_owned())?;
        output = format_numeric_picture(number, picture)?;
    }
    if let Some(picture) = switch_text(instruction, "@") {
        let date_time = if matches!(instruction.name.as_str(), "DATE" | "TIME") {
            date_time.ok_or_else(|| {
                "date-time formatting requires an explicit date and time".to_owned()
            })?
        } else {
            parse_field_date_time(value).ok_or_else(|| {
                "date-time field value is not a supported civil date and time".to_owned()
            })?
        };
        output = format_date_time(date_time, picture)?;
    }
    for format in instruction.switches.iter().filter_map(|switch| {
        (switch.name == "*")
            .then_some(switch.argument.as_ref())
            .flatten()
            .and_then(argument_text)
    }) {
        output = apply_general_format(&output, format)?;
    }
    Ok(output)
}

fn apply_general_format(value: &str, format: &str) -> std::result::Result<String, String> {
    if format == "ALPHABETIC" {
        return parse_positive_integer(value).map(|value| alphabetic(value, true));
    }
    if format == "ROMAN" {
        return parse_positive_integer(value).and_then(|value| roman(value, true));
    }
    match format.to_ascii_lowercase().as_str() {
        "upper" => Ok(value.to_uppercase()),
        "lower" => Ok(value.to_lowercase()),
        "firstcap" => Ok(capitalize_first(value)),
        "caps" => Ok(value
            .split_inclusive(char::is_whitespace)
            .map(capitalize_first)
            .collect()),
        "arabic" => parse_positive_integer(value).map(|value| value.to_string()),
        "alphabetic" => parse_positive_integer(value).map(|value| alphabetic(value, false)),
        "roman" => parse_positive_integer(value).and_then(|value| roman(value, false)),
        "ordinal" => parse_positive_integer(value).map(ordinal),
        "mergeformat" | "charformat" => Ok(value.to_owned()),
        other => Err(format!("general format {other} is unsupported")),
    }
}

fn parse_positive_integer(value: &str) -> std::result::Result<u32, String> {
    value
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| "general numeric format requires a positive integer".to_owned())
}

fn capitalize_first(value: &str) -> String {
    let mut characters = value.chars();
    characters
        .next()
        .map(|first| first.to_uppercase().chain(characters).collect())
        .unwrap_or_default()
}

fn alphabetic(mut value: u32, upper: bool) -> String {
    let mut output = Vec::new();
    while value > 0 {
        value -= 1;
        let base = if upper { b'A' } else { b'a' };
        output.push((base + (value % 26) as u8) as char);
        value /= 26;
    }
    output.iter().rev().collect()
}

fn roman(mut value: u32, upper: bool) -> std::result::Result<String, String> {
    if value > 3999 {
        return Err("Roman format supports values from 1 through 3999".to_owned());
    }
    let values = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut output = String::new();
    for (number, numeral) in values {
        while value >= number {
            output.push_str(numeral);
            value -= number;
        }
    }
    Ok(if upper { output } else { output.to_lowercase() })
}

fn ordinal(value: u32) -> String {
    let suffix = if (11..=13).contains(&(value % 100)) {
        "th"
    } else {
        match value % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    };
    format!("{value}{suffix}")
}

fn split_picture_sections(picture: &str) -> std::result::Result<Vec<String>, String> {
    let mut sections = vec![String::new()];
    let mut quote = None;
    for character in picture.chars() {
        match (character, quote) {
            ('"' | '\'', None) => {
                quote = Some(character);
                sections.last_mut().unwrap().push(character);
            }
            (character, Some(active)) if character == active => {
                quote = None;
                sections.last_mut().unwrap().push(character);
            }
            (';', None) => sections.push(String::new()),
            (other, _) => sections.last_mut().unwrap().push(other),
        }
    }
    if quote.is_some() || sections.len() > 3 {
        Err("numeric picture has invalid sections or quoting".to_owned())
    } else {
        Ok(sections)
    }
}

fn format_numeric_picture(value: f64, picture: &str) -> std::result::Result<String, String> {
    let sections = split_picture_sections(picture)?;
    let (section, magnitude, implicit_negative) = if value < 0.0 {
        if let Some(section) = sections.get(1) {
            (section.as_str(), -value, false)
        } else {
            (sections[0].as_str(), -value, true)
        }
    } else if value == 0.0 && sections.len() == 3 {
        (sections[2].as_str(), 0.0, false)
    } else {
        (sections[0].as_str(), value, false)
    };
    let placeholders = numeric_placeholder_indices(section);
    let Some(first) = placeholders.first().copied() else {
        return unquote_picture(section);
    };
    let last = placeholders.last().copied().unwrap() + 1;
    let prefix = unquote_picture(&section[..first])?;
    let suffix = unquote_picture(&section[last..])?;
    let (core, literals) = extract_numeric_literals(&section[first..last])?;
    let mut halves = core.split('.');
    let integer_picture = halves.next().unwrap_or_default();
    let decimal_picture = halves.next().unwrap_or_default();
    if halves.next().is_some()
        || integer_picture
            .chars()
            .any(|character| !matches!(character, '0' | '#' | ','))
        || decimal_picture
            .chars()
            .any(|character| !matches!(character, '0' | '#'))
    {
        return Err("numeric picture contains unsupported tokens".to_owned());
    }
    let maximum_decimals = decimal_picture.chars().count();
    let minimum_decimals = decimal_picture
        .chars()
        .filter(|character| *character == '0')
        .count();
    let integer_placeholders = integer_picture
        .chars()
        .filter(|character| matches!(character, '0' | '#'))
        .collect::<Vec<_>>();
    let decimal_placeholders = decimal_picture.chars().collect::<Vec<_>>();
    let formatted = format!("{magnitude:.maximum_decimals$}");
    let (integer, mut decimal) = formatted
        .split_once('.')
        .map(|(integer, decimal)| (integer.to_owned(), decimal.to_owned()))
        .unwrap_or((formatted, String::new()));
    while decimal.len() > minimum_decimals && decimal.ends_with('0') {
        decimal.pop();
    }
    let mut integer = integer;
    if !integer_placeholders.contains(&'0') && integer == "0" {
        integer.clear();
    }
    let missing_integer = integer_placeholders.len().saturating_sub(integer.len());
    let padding = integer_placeholders[..missing_integer]
        .iter()
        .map(|placeholder| if *placeholder == '0' { '0' } else { ' ' })
        .collect::<String>();
    if integer_picture.contains(',') {
        integer = group_digits(&integer);
    }
    let decimal_padding = decimal_placeholders[decimal.len()..]
        .iter()
        .map(|placeholder| if *placeholder == '0' { '0' } else { ' ' })
        .collect::<String>();
    let number = if decimal_placeholders.is_empty() {
        format!("{padding}{integer}")
    } else {
        format!("{padding}{integer}.{decimal}{decimal_padding}")
    };
    let number = insert_numeric_literals(
        &number,
        integer_picture
            .chars()
            .chain(decimal_picture.chars())
            .filter(|character| matches!(character, '0' | '#'))
            .count(),
        &literals,
    );
    Ok(format!(
        "{}{}{}{}",
        if implicit_negative { "-" } else { "" },
        prefix,
        number,
        suffix
    ))
}

fn numeric_placeholder_indices(value: &str) -> Vec<usize> {
    let mut quote = None;
    let mut indices = Vec::new();
    for (index, character) in value.char_indices() {
        match (character, quote) {
            ('"' | '\'', None) => quote = Some(character),
            (character, Some(active)) if character == active => quote = None,
            ('0' | '#', None) => indices.push(index),
            _ => {}
        }
    }
    indices
}

fn extract_numeric_literals(
    value: &str,
) -> std::result::Result<(String, Vec<(usize, String)>), String> {
    let mut pattern = String::new();
    let mut literals = Vec::new();
    let mut literal = String::new();
    let mut quote = None;
    let mut placeholder_count = 0usize;
    for character in value.chars() {
        match (character, quote) {
            ('"' | '\'', None) => quote = Some(character),
            (character, Some(active)) if character == active => {
                literals.push((placeholder_count, std::mem::take(&mut literal)));
                quote = None;
            }
            (character, Some(_)) => literal.push(character),
            (character @ ('0' | '#'), None) => {
                pattern.push(character);
                placeholder_count += 1;
            }
            (character, None) => pattern.push(character),
        }
    }
    if quote.is_some() {
        Err("numeric picture has unclosed quoting".to_owned())
    } else {
        Ok((pattern, literals))
    }
}

fn insert_numeric_literals(
    number: &str,
    placeholder_count: usize,
    literals: &[(usize, String)],
) -> String {
    if literals.is_empty() {
        return number.to_owned();
    }
    let slot_count = number
        .chars()
        .filter(|character| character.is_ascii_digit() || *character == ' ')
        .count();
    let extra_leading = slot_count.saturating_sub(placeholder_count);
    let mut output = String::new();
    let mut digits_written = 0usize;
    for (position, literal) in literals
        .iter()
        .filter(|(position, _)| extra_leading + position == 0)
    {
        let _ = position;
        output.push_str(literal);
    }
    for character in number.chars() {
        output.push(character);
        if character.is_ascii_digit() || character == ' ' {
            digits_written += 1;
            for (_, literal) in literals
                .iter()
                .filter(|(position, _)| extra_leading + position == digits_written)
            {
                output.push_str(literal);
            }
        }
    }
    output
}

fn unquote_picture(value: &str) -> std::result::Result<String, String> {
    let mut output = String::new();
    let mut quote = None;
    for character in value.chars() {
        match (character, quote) {
            ('"' | '\'', None) => quote = Some(character),
            (character, Some(active)) if character == active => quote = None,
            (character, _) => output.push(character),
        }
    }
    if quote.is_some() {
        Err("numeric picture has unclosed quoting".to_owned())
    } else {
        Ok(output)
    }
}

fn group_digits(value: &str) -> String {
    let mut output = String::new();
    for (index, character) in value.chars().enumerate() {
        if index > 0 && (value.len() - index).is_multiple_of(3) {
            output.push(',');
        }
        output.push(character);
    }
    output
}

fn valid_date_time(value: FieldDateTime) -> bool {
    value.month >= 1
        && value.month <= 12
        && value.day >= 1
        && value.day <= days_in_month(value.year, value.month)
        && value.hour < 24
        && value.minute < 60
        && value.second < 60
}

fn parse_field_date_time(value: &str) -> Option<FieldDateTime> {
    let value = value.trim().strip_suffix('Z').unwrap_or(value.trim());
    let (date, time) = value
        .split_once('T')
        .or_else(|| value.split_once(' '))
        .map_or((value, None), |(date, time)| (date, Some(time)));
    let mut date_parts = date.split('-');
    let year = date_parts.next()?.parse().ok()?;
    let month = date_parts.next()?.parse().ok()?;
    let day = date_parts.next()?.parse().ok()?;
    if date_parts.next().is_some() {
        return None;
    }
    let (hour, minute, second) = if let Some(time) = time {
        let mut time_parts = time.split(':');
        let hour = time_parts.next()?.parse().ok()?;
        let minute = time_parts.next()?.parse().ok()?;
        let second = time_parts.next()?.split('.').next()?.parse().ok()?;
        if time_parts.next().is_some() {
            return None;
        }
        (hour, minute, second)
    } else {
        (0, 0, 0)
    };
    let parsed = FieldDateTime {
        year,
        month,
        day,
        hour,
        minute,
        second,
    };
    valid_date_time(parsed).then_some(parsed)
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if year % 400 == 0 || year % 4 == 0 && year % 100 != 0 => 29,
        2 => 28,
        _ => 31,
    }
}

fn format_date_time(value: FieldDateTime, picture: &str) -> std::result::Result<String, String> {
    if !valid_date_time(value) {
        return Err("field date and time is invalid".to_owned());
    }
    let months = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let weekdays = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];
    let characters = picture.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0;
    while index < characters.len() {
        if characters[index] == '"' {
            index += 1;
            while index < characters.len() && characters[index] != '"' {
                output.push(characters[index]);
                index += 1;
            }
            if index == characters.len() {
                return Err("date-time picture has unclosed quoting".to_owned());
            }
            index += 1;
            continue;
        }
        if characters[index] == '\\' {
            index += 1;
            let Some(character) = characters.get(index) else {
                return Err("date-time picture ends with an escape".to_owned());
            };
            output.push(*character);
            index += 1;
            continue;
        }
        let rest = characters[index..].iter().collect::<String>();
        if rest.to_ascii_uppercase().starts_with("AM/PM") {
            output.push_str(if value.hour < 12 { "AM" } else { "PM" });
            index += 5;
            continue;
        }
        let token = characters[index];
        if !matches!(token, 'y' | 'M' | 'd' | 'H' | 'h' | 'm' | 's') {
            output.push(token);
            index += 1;
            continue;
        }
        let mut count = 1;
        while index + count < characters.len() && characters[index + count] == token {
            count += 1;
        }
        match token {
            'y' if count == 2 => output.push_str(&format!("{:02}", value.year.rem_euclid(100))),
            'y' => output.push_str(&format!("{:04}", value.year)),
            'M' if count == 1 => output.push_str(&value.month.to_string()),
            'M' if count == 2 => output.push_str(&format!("{:02}", value.month)),
            'M' if count == 3 => output.push_str(&months[value.month as usize - 1][..3]),
            'M' => output.push_str(months[value.month as usize - 1]),
            'd' if count == 1 => output.push_str(&value.day.to_string()),
            'd' if count == 2 => output.push_str(&format!("{:02}", value.day)),
            'd' if count == 3 => output.push_str(&weekdays[weekday(value)][..3]),
            'd' => output.push_str(weekdays[weekday(value)]),
            'H' if count == 1 => output.push_str(&value.hour.to_string()),
            'H' => output.push_str(&format!("{:02}", value.hour)),
            'h' if count == 1 => output.push_str(&(value.hour % 12).max(1).to_string()),
            'h' => output.push_str(&format!("{:02}", (value.hour % 12).max(1))),
            'm' if count == 1 => output.push_str(&value.minute.to_string()),
            'm' => output.push_str(&format!("{:02}", value.minute)),
            's' if count == 1 => output.push_str(&value.second.to_string()),
            's' => output.push_str(&format!("{:02}", value.second)),
            _ => unreachable!(),
        }
        index += count;
    }
    Ok(output)
}

fn weekday(value: FieldDateTime) -> usize {
    let mut year = i64::from(value.year);
    let mut month = i64::from(value.month);
    if month < 3 {
        month += 12;
        year -= 1;
    }
    let year_of_century = year.rem_euclid(100);
    let century = year.div_euclid(100);
    let h = (i64::from(value.day)
        + (13 * (month + 1)) / 5
        + year_of_century
        + year_of_century / 4
        + century / 4
        + 5 * century)
        .rem_euclid(7);
    ((h + 6) % 7) as usize
}

#[cfg(test)]
mod tests {
    use rdocx_oxml::document::BodyContent;
    use rdocx_oxml::properties::CT_PPr;
    use rdocx_oxml::text::{CT_P, CT_R, Field, FieldSwitch, RunContent};

    use super::*;

    fn document_with_fields(fields: &[(&str, &str)]) -> Document {
        let mut document = Document::new();
        let mut paragraph = CT_P::new();
        for (instruction, cached_result) in fields {
            paragraph.runs.push(CT_R {
                properties: None,
                content: vec![RunContent::Field(Field::new(instruction, cached_result))],
                extra_xml: Vec::new(),
                extra_xml_positions: Vec::new(),
                alt_drawings: Vec::new(),
            });
        }
        document
            .document
            .body
            .content
            .push(BodyContent::Paragraph(paragraph));
        document
    }

    fn document_with_parsed_paragraph(xml: &str) -> Document {
        let mut reader = quick_xml::Reader::from_str(xml);
        reader.config_mut().trim_text(false);
        let mut buffer = Vec::new();
        let paragraph = loop {
            match reader.read_event_into(&mut buffer).unwrap() {
                Event::Start(element) if matches_local_name(element.name().as_ref(), b"p") => {
                    break CT_P::from_xml(&mut reader).unwrap();
                }
                Event::Eof => panic!("missing paragraph"),
                _ => {}
            }
            buffer.clear();
        };
        let mut document = Document::new();
        document
            .document
            .body
            .content
            .push(BodyContent::Paragraph(paragraph));
        document
    }

    #[test]
    fn a_typed_field_is_reported_without_mutating_its_cache() {
        let document = document_with_fields(&[("AUTHOR", "stored")]);
        let results = document
            .evaluate_fields(&FieldEvaluationContext::default())
            .unwrap();
        assert_eq!(results.len(), 1, "the typed field must be evaluated");
    }

    #[test]
    fn raw_only_instruction_edits_use_the_serialized_instruction() {
        let mut document = document_with_fields(&[("DATE", "stored")]);
        let BodyContent::Paragraph(paragraph) = &mut document.document.body.content[0] else {
            unreachable!()
        };
        let RunContent::Field(field) = &mut paragraph.runs[0].content[0] else {
            unreachable!()
        };
        field.instruction.raw = "AUTHOR".to_owned();
        document.set_author("Ada Lovelace");

        let results = document
            .evaluate_fields(&FieldEvaluationContext::default())
            .unwrap();
        assert_eq!(results[0].instruction, "AUTHOR");
        assert_eq!(
            results[0].outcome,
            FieldOutcome::Resolved("Ada Lovelace".to_owned())
        );
    }

    #[test]
    fn same_run_nested_raw_instruction_edit_updates_saves_and_reopens() {
        let word_namespace = rdocx_oxml::namespace::W_NS;
        let xml = format!(
            concat!(
                r#"<w:p xmlns:w="{0}"><q:r xmlns:q="{0}" xmlns:x="urn:producer">"#,
                r#"<q:fldChar q:fldCharType="begin"/><q:instrText xml:space="preserve">IF </q:instrText>"#,
                r#"<q:fldChar q:fldCharType="begin" q:dirty="on"/><q:instrText>MERGEFIELD Old</q:instrText><x:nestedInstruction/><q:fldChar q:fldCharType="separate"/><q:t>stored nested</q:t><q:fldChar q:fldCharType="end"/>"#,
                r#"<q:instrText xml:space="preserve"> = &quot;new value&quot; &quot;yes&quot; &quot;no&quot;</q:instrText><q:fldChar q:fldCharType="separate"/><q:t>stored outer</q:t><q:fldChar q:fldCharType="end"/>"#,
                r#"</q:r></w:p>"#,
            ),
            word_namespace,
        );
        let mut document = document_with_parsed_paragraph(&xml);
        let BodyContent::Paragraph(paragraph) = &mut document.document.body.content[0] else {
            unreachable!()
        };
        let RunContent::Field(outer) = &mut paragraph.runs[0].content[0] else {
            unreachable!()
        };
        let Some(FieldArgument::Nested(nested)) = outer.instruction.arguments.first_mut() else {
            panic!("expected nested field")
        };
        nested.instruction.raw = "MERGEFIELD New".to_owned();

        let context = FieldEvaluationContext {
            merge_fields: BTreeMap::from([("New".to_owned(), "new value".to_owned())]),
            ..FieldEvaluationContext::default()
        };
        assert_eq!(document.update_fields(&context).unwrap(), 2);
        let saved = document.to_bytes().unwrap();
        let package = OpcPackage::from_reader(std::io::Cursor::new(&saved)).unwrap();
        let saved_xml =
            std::str::from_utf8(package.get_part("/word/document.xml").unwrap()).unwrap();
        assert!(saved_xml.contains("MERGEFIELD New"), "{saved_xml}");
        assert!(!saved_xml.contains("MERGEFIELD Old"), "{saved_xml}");
        assert_eq!(saved_xml.matches(">new value<").count(), 1, "{saved_xml}");
        assert!(saved_xml.contains(r#"w:dirty="0""#), "{saved_xml}");
        assert!(saved_xml.contains("<x:nestedInstruction/>"), "{saved_xml}");

        let reopened = Document::from_bytes(&saved).unwrap();
        let evaluations = reopened.evaluate_fields(&context).unwrap();
        assert_eq!(evaluations.len(), 2);
        assert_eq!(evaluations[0].cached_result, "yes");
        assert_eq!(evaluations[1].instruction, "MERGEFIELD New");
        assert_eq!(evaluations[1].cached_result, "new value");
        assert_eq!(
            evaluations[1].outcome,
            FieldOutcome::Resolved("new value".to_owned())
        );
    }

    #[test]
    fn raw_only_nested_edit_suppresses_its_stale_nested_operand() {
        let word_namespace = rdocx_oxml::namespace::W_NS;
        let xml = format!(
            concat!(
                r#"<w:p xmlns:w="{0}"><q:r xmlns:q="{0}" xmlns:x="urn:producer">"#,
                r#"<q:fldChar q:fldCharType="begin" q:dirty="on"/><q:instrText xml:space="preserve">IF </q:instrText>"#,
                r#"<q:fldChar q:fldCharType="begin" q:dirty="on"/><q:instrText xml:space="preserve">IF </q:instrText><x:middleBefore/>"#,
                r#"<q:fldChar q:fldCharType="begin" q:dirty="on"/><q:instrText>MERGEFIELD Stale</q:instrText><q:fldChar q:fldCharType="separate"/><q:t>stored grandchild</q:t><q:fldChar q:fldCharType="end"/>"#,
                r#"<x:middleAfter/><q:instrText xml:space="preserve"> = &quot;stale&quot; &quot;old yes&quot; &quot;old no&quot;</q:instrText><q:fldChar q:fldCharType="separate"/><q:t>stored middle</q:t><q:fldChar q:fldCharType="end"/>"#,
                r#"<q:instrText xml:space="preserve"> = &quot;new value&quot; &quot;yes&quot; &quot;no&quot;</q:instrText><q:fldChar q:fldCharType="separate"/><q:t>stored outer</q:t><q:fldChar q:fldCharType="end"/>"#,
                r#"</q:r></w:p>"#,
            ),
            word_namespace,
        );
        let mut document = document_with_parsed_paragraph(&xml);
        let BodyContent::Paragraph(paragraph) = &mut document.document.body.content[0] else {
            unreachable!()
        };
        let RunContent::Field(outer) = &mut paragraph.runs[0].content[0] else {
            unreachable!()
        };
        let Some(FieldArgument::Nested(middle)) = outer.instruction.arguments.first_mut() else {
            panic!("expected middle field")
        };
        assert!(matches!(
            middle.instruction.arguments.first(),
            Some(FieldArgument::Nested(_))
        ));
        middle.instruction.raw = "MERGEFIELD New".to_owned();

        let context = FieldEvaluationContext {
            merge_fields: BTreeMap::from([("New".to_owned(), "new value".to_owned())]),
            ..FieldEvaluationContext::default()
        };
        let before_save = document.evaluate_fields(&context).unwrap();
        assert_eq!(before_save.len(), 2);
        assert_eq!(before_save[0].field_index, 0);
        assert_eq!(before_save[1].field_index, 1);
        assert_eq!(before_save[1].instruction, "MERGEFIELD New");
        assert_eq!(
            before_save[1].outcome,
            FieldOutcome::Resolved("new value".to_owned())
        );

        assert_eq!(document.update_fields(&context).unwrap(), 2);
        let saved = document.to_bytes().unwrap();
        let package = OpcPackage::from_reader(std::io::Cursor::new(&saved)).unwrap();
        let saved_xml =
            std::str::from_utf8(package.get_part("/word/document.xml").unwrap()).unwrap();
        assert!(saved_xml.contains("MERGEFIELD New"), "{saved_xml}");
        assert!(!saved_xml.contains("MERGEFIELD Stale"), "{saved_xml}");
        assert!(!saved_xml.contains("stored grandchild"), "{saved_xml}");
        assert!(!saved_xml.contains("stored middle"), "{saved_xml}");
        assert!(saved_xml.contains("<x:middleBefore/>"), "{saved_xml}");
        assert!(saved_xml.contains("<x:middleAfter/>"), "{saved_xml}");
        assert!(
            saved_xml.find("<x:middleBefore/>").unwrap()
                < saved_xml.find("<x:middleAfter/>").unwrap(),
            "{saved_xml}"
        );
        assert_eq!(saved_xml.matches(r#"w:dirty="0""#).count(), 2);
        assert_eq!(saved_xml.matches(">new value<").count(), 1, "{saved_xml}");

        let reopened = Document::from_bytes(&saved).unwrap();
        let evaluations = reopened.evaluate_fields(&context).unwrap();
        assert_eq!(evaluations.len(), 2);
        assert_eq!(evaluations[0].field_index, 0);
        assert_eq!(evaluations[0].cached_result, "yes");
        assert_eq!(evaluations[1].field_index, 1);
        assert_eq!(evaluations[1].instruction, "MERGEFIELD New");
        assert_eq!(evaluations[1].cached_result, "new value");
        assert_eq!(
            evaluations[1].outcome,
            FieldOutcome::Resolved("new value".to_owned())
        );
    }

    #[test]
    fn multi_run_raw_only_nested_edit_preserves_every_run_scaffold() {
        let word_namespace = rdocx_oxml::namespace::W_NS;
        let xml = format!(
            concat!(
                r#"<w:p xmlns:w="{0}">"#,
                r#"<q:r xmlns:q="{0}" data-run="outer-start"><q:fldChar q:fldCharType="begin" q:dirty="on"/><q:instrText xml:space="preserve">IF </q:instrText></q:r>"#,
                r#"<q:r xmlns:q="{0}" xmlns:a="urn:start" data-run="start"><q:rPr><q:i/></q:rPr><q:fldChar q:fldCharType="begin" q:dirty="on"/><q:instrText xml:space="preserve">IF </q:instrText><a:prefix/><q:fldChar q:fldCharType="begin" q:dirty="on"/></q:r>"#,
                r#"<q:r xmlns:q="{0}" xmlns:m="urn:middle" data-run="middle"><q:rPr><q:u q:val="single"/></q:rPr><q:instrText>MERGEFIELD Stale</q:instrText><m:inside/></q:r>"#,
                r#"<q:r xmlns:q="{0}" xmlns:z="urn:end" data-run="end"><q:rPr><q:b/></q:rPr><q:fldChar q:fldCharType="separate"/><q:t>stored grandchild</q:t><q:fldChar q:fldCharType="end"/><z:suffix/><q:instrText xml:space="preserve"> = &quot;stale&quot; &quot;old yes&quot; &quot;old no&quot;</q:instrText><q:fldChar q:fldCharType="separate"/><q:t>stored middle</q:t><q:fldChar q:fldCharType="end"/></q:r>"#,
                r#"<q:r xmlns:q="{0}" data-run="outer-end"><q:instrText xml:space="preserve"> = &quot;new value&quot; &quot;yes&quot; &quot;no&quot;</q:instrText><q:fldChar q:fldCharType="separate"/><q:t>stored outer</q:t><q:fldChar q:fldCharType="end"/></q:r>"#,
                r#"</w:p>"#,
            ),
            word_namespace,
        );
        let mut document = document_with_parsed_paragraph(&xml);
        let BodyContent::Paragraph(paragraph) = &mut document.document.body.content[0] else {
            unreachable!()
        };
        let RunContent::Field(outer) = &mut paragraph.runs[0].content[0] else {
            unreachable!()
        };
        let Some(FieldArgument::Nested(middle)) = outer.instruction.arguments.first_mut() else {
            panic!("expected middle field")
        };
        assert!(matches!(
            middle.instruction.arguments.first(),
            Some(FieldArgument::Nested(_))
        ));
        middle.instruction.raw = "MERGEFIELD New".to_owned();

        let context = FieldEvaluationContext {
            merge_fields: BTreeMap::from([("New".to_owned(), "new value".to_owned())]),
            ..FieldEvaluationContext::default()
        };
        assert_eq!(document.update_fields(&context).unwrap(), 2);
        let saved = document.to_bytes().unwrap();
        let package = OpcPackage::from_reader(std::io::Cursor::new(&saved)).unwrap();
        let saved_xml =
            std::str::from_utf8(package.get_part("/word/document.xml").unwrap()).unwrap();
        assert!(saved_xml.contains("MERGEFIELD New"), "{saved_xml}");
        assert!(!saved_xml.contains("MERGEFIELD Stale"), "{saved_xml}");
        assert!(!saved_xml.contains("stored grandchild"), "{saved_xml}");
        for preserved in [
            r#"data-run="start""#,
            r#"data-run="middle""#,
            r#"data-run="end""#,
            r#"xmlns:a="urn:start""#,
            r#"xmlns:m="urn:middle""#,
            r#"xmlns:z="urn:end""#,
            "<q:i/>",
            r#"<q:u q:val="single"/>"#,
            "<q:b/>",
            "<a:prefix/>",
            "<m:inside/>",
            "<z:suffix/>",
        ] {
            assert!(
                saved_xml.contains(preserved),
                "missing {preserved}: {saved_xml}"
            );
        }
        assert!(
            saved_xml.find("<a:prefix/>").unwrap() < saved_xml.find("<m:inside/>").unwrap()
                && saved_xml.find("<m:inside/>").unwrap() < saved_xml.find("<z:suffix/>").unwrap(),
            "{saved_xml}"
        );

        let reopened = Document::from_bytes(&saved).unwrap();
        let evaluations = reopened.evaluate_fields(&context).unwrap();
        assert_eq!(evaluations.len(), 2);
        assert_eq!(evaluations[0].field_index, 0);
        assert_eq!(evaluations[0].cached_result, "yes");
        assert_eq!(evaluations[1].field_index, 1);
        assert_eq!(evaluations[1].instruction, "MERGEFIELD New");
        assert_eq!(evaluations[1].cached_result, "new value");
    }

    #[test]
    fn nested_if_and_comparison_operators_evaluate_recursively() {
        let mut document = document_with_fields(&[(r#"IF left = right yes no"#, "stored")]);
        let BodyContent::Paragraph(paragraph) = &mut document.document.body.content[0] else {
            unreachable!()
        };
        let RunContent::Field(outer) = &mut paragraph.runs[0].content[0] else {
            unreachable!()
        };
        outer.instruction.arguments[0] =
            FieldArgument::Nested(Box::new(Field::new("MERGEFIELD Score", "stored score")));
        outer.instruction.arguments[1] = FieldArgument::Text(">=".to_owned());
        outer.instruction.arguments[2] = FieldArgument::Text("2".to_owned());
        outer.instruction.arguments[4] =
            FieldArgument::Nested(Box::new(Field::new("UNKNOWN", "stored branch")));
        let mut context = FieldEvaluationContext::default();
        context
            .merge_fields
            .insert("Score".to_owned(), "10".to_owned());
        let results = document.evaluate_fields(&context).unwrap();
        assert_eq!(results[0].field_index, 0);
        assert_eq!(results[0].outcome, FieldOutcome::Resolved("yes".to_owned()));
        assert_eq!(results[1].field_index, 1);
        assert_eq!(results[1].outcome, FieldOutcome::Resolved("10".to_owned()));
        assert_eq!(results[2].field_index, 2);
        assert!(matches!(
            results[2].outcome,
            FieldOutcome::KeepStored { .. }
        ));

        for (operator, expected) in [
            ("=", "yes"),
            ("<>", "no"),
            ("<", "no"),
            ("<=", "yes"),
            (">", "no"),
            (">=", "yes"),
        ] {
            let document = document_with_fields(&[(
                &format!(r#"IF "2" {operator} "2" "yes" "no""#),
                "stored",
            )]);
            assert_eq!(
                document
                    .evaluate_fields(&FieldEvaluationContext::default())
                    .unwrap()[0]
                    .outcome,
                FieldOutcome::Resolved(expected.to_owned())
            );
        }
        let wildcard =
            document_with_fields(&[(r#"IF "Alphabet" = "A?pha*" "yes" "no""#, "stored")]);
        assert_eq!(
            wildcard
                .evaluate_fields(&FieldEvaluationContext::default())
                .unwrap()[0]
                .outcome,
            FieldOutcome::Resolved("yes".to_owned())
        );

        let mut unresolved = document_with_fields(&[(r#"IF left = right yes no"#, "outer")]);
        let BodyContent::Paragraph(paragraph) = &mut unresolved.document.body.content[0] else {
            unreachable!()
        };
        let RunContent::Field(outer) = &mut paragraph.runs[0].content[0] else {
            unreachable!()
        };
        outer.instruction.arguments[0] =
            FieldArgument::Nested(Box::new(Field::new("DATE", "stored date")));
        let outcomes = unresolved
            .evaluate_fields(&FieldEvaluationContext::default())
            .unwrap();
        assert!(matches!(
            outcomes[0].outcome,
            FieldOutcome::KeepStored { .. }
        ));
        assert!(matches!(
            outcomes[1].outcome,
            FieldOutcome::KeepStored { .. }
        ));
    }

    #[test]
    fn nested_if_reuses_the_eager_effective_instruction_outcome() {
        let mut document = document_with_fields(&[(r#"IF left = 1 yes no"#, "outer")]);
        let BodyContent::Paragraph(paragraph) = &mut document.document.body.content[0] else {
            unreachable!()
        };
        let RunContent::Field(outer) = &mut paragraph.runs[0].content[0] else {
            unreachable!()
        };
        outer.instruction.arguments[0] =
            FieldArgument::Nested(Box::new(Field::new("SEQ Figure", "stored sequence")));

        let results = document
            .evaluate_fields(&FieldEvaluationContext::default())
            .unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].outcome, FieldOutcome::Resolved("yes".to_owned()));
        assert_eq!(results[1].outcome, FieldOutcome::Resolved("1".to_owned()));
    }

    #[test]
    fn nested_outcome_frames_do_not_leak_between_outer_fields() {
        let mut document = document_with_fields(&[
            (r#"IF left = 1 first no"#, "first outer"),
            (r#"IF left = 2 second no"#, "second outer"),
        ]);
        let BodyContent::Paragraph(paragraph) = &mut document.document.body.content[0] else {
            unreachable!()
        };
        for run in &mut paragraph.runs {
            let RunContent::Field(outer) = &mut run.content[0] else {
                unreachable!()
            };
            outer.instruction.arguments[0] =
                FieldArgument::Nested(Box::new(Field::new("SEQ Figure", "stored sequence")));
        }

        let results = document
            .evaluate_fields(&FieldEvaluationContext::default())
            .unwrap();
        assert_eq!(results.len(), 4);
        assert_eq!(
            results
                .into_iter()
                .map(|result| result.outcome)
                .collect::<Vec<_>>(),
            [
                FieldOutcome::Resolved("first".to_owned()),
                FieldOutcome::Resolved("1".to_owned()),
                FieldOutcome::Resolved("second".to_owned()),
                FieldOutcome::Resolved("2".to_owned()),
            ]
        );
    }

    #[test]
    fn every_nested_field_is_reported_before_outer_fallback() {
        let mut document = document_with_fields(&[("UNKNOWN", "outer")]);
        let BodyContent::Paragraph(paragraph) = &mut document.document.body.content[0] else {
            unreachable!()
        };
        let RunContent::Field(outer) = &mut paragraph.runs[0].content[0] else {
            unreachable!()
        };
        outer
            .instruction
            .arguments
            .push(FieldArgument::Nested(Box::new(Field::new(
                "MERGEFIELD Name",
                "stored name",
            ))));
        outer.instruction.switches.push(FieldSwitch {
            name: "*".to_owned(),
            argument: Some(FieldArgument::Nested(Box::new(Field::new(
                "AUTHOR",
                "stored author",
            )))),
        });
        let context = FieldEvaluationContext {
            merge_fields: BTreeMap::from([("Name".to_owned(), "Ada".to_owned())]),
            ..FieldEvaluationContext::default()
        };
        let results = document.evaluate_fields(&context).unwrap();
        assert_eq!(results.len(), 3);
        assert!(matches!(
            results[0].outcome,
            FieldOutcome::KeepStored { .. }
        ));
        assert_eq!(results[1].outcome, FieldOutcome::Resolved("Ada".to_owned()));
        assert!(matches!(
            results[2].outcome,
            FieldOutcome::KeepStored { .. }
        ));
    }

    #[test]
    fn malformed_arity_and_extreme_inputs_keep_stored_without_panicking() {
        let document = document_with_fields(&[
            (r"DATE \@", "date"),
            (r"SEQ Figure \r", "reset"),
            ("PAGE extra", "page"),
            (r#"MERGEFIELD "Name"#, "merge"),
            (r##"MERGEFIELD Amount \# "$0.00"##, "picture"),
            (r"SEQ Figure \r 9223372036854775807", "max"),
            ("SEQ Figure", "overflow"),
        ]);
        let mut context = FieldEvaluationContext::default();
        context
            .merge_fields
            .insert("Name".to_owned(), "Ada".to_owned());
        context
            .merge_fields
            .insert("Amount".to_owned(), "12".to_owned());
        let results = document.evaluate_fields(&context).unwrap();
        assert!(
            results[..5]
                .iter()
                .all(|result| matches!(result.outcome, FieldOutcome::KeepStored { .. }))
        );
        assert_eq!(
            results[5].outcome,
            FieldOutcome::Resolved(i64::MAX.to_string())
        );
        assert_eq!(results[6].outcome, keep("SEQ value overflowed"));

        let date = document_with_fields(&[(r#"DATE \@ "dddd""#, "date")]);
        let context = FieldEvaluationContext {
            now: Some(FieldDateTime {
                year: i32::MIN,
                month: 1,
                day: 1,
                hour: 0,
                minute: 0,
                second: 0,
            }),
            ..FieldEvaluationContext::default()
        };
        assert!(matches!(
            date.evaluate_fields(&context).unwrap()[0].outcome,
            FieldOutcome::Resolved(_)
        ));
    }

    #[test]
    fn sequence_state_is_scoped_and_reset_by_supported_switches() {
        let document = document_with_fields(&[
            ("SEQ Figure", "0"),
            (r"SEQ Figure \c", "0"),
            (r"SEQ Figure \r 5", "0"),
            (r"SEQ Figure \h", "0"),
            (r"SEQ Figure \n \* ROMAN", "0"),
            ("SEQ Table", "0"),
        ]);
        let results = document
            .evaluate_fields(&FieldEvaluationContext::default())
            .unwrap();
        assert_eq!(
            results
                .into_iter()
                .map(|result| result.outcome)
                .collect::<Vec<_>>(),
            [
                FieldOutcome::Resolved("1".to_owned()),
                FieldOutcome::Resolved("1".to_owned()),
                FieldOutcome::Resolved("5".to_owned()),
                FieldOutcome::Resolved(String::new()),
                FieldOutcome::Resolved("VII".to_owned()),
                FieldOutcome::Resolved("1".to_owned()),
            ]
        );

        let story_document = document_with_fields(&[("SEQ Shared", "0")]);
        let BodyContent::Paragraph(paragraph) = &story_document.document.body.content[0] else {
            unreachable!()
        };
        let paragraphs = [paragraph];
        let story_context = FieldEvaluationContext::default();
        let mut evaluator = Evaluator::new(&story_document, &story_context);
        evaluator.evaluate_story("header:one", &paragraphs);
        evaluator.evaluate_story("footer:one", &paragraphs);
        assert_eq!(
            evaluator
                .results
                .into_iter()
                .map(|result| result.outcome)
                .collect::<Vec<_>>(),
            [
                FieldOutcome::Resolved("1".to_owned()),
                FieldOutcome::Resolved("1".to_owned()),
            ]
        );

        let mut heading_restart = Document::new();
        for (style_id, instruction) in [
            (Some("Heading1"), None),
            (None, Some(r"SEQ Figure \s 1")),
            (None, Some("SEQ Figure")),
            (Some("Heading1"), None),
            (None, Some(r"SEQ Figure \s 1")),
        ] {
            let mut paragraph = CT_P::new();
            paragraph.properties = style_id.map(|style_id| CT_PPr {
                style_id: Some(style_id.to_owned()),
                outline_lvl: Some(0),
                ..Default::default()
            });
            if let Some(instruction) = instruction {
                paragraph.runs.push(CT_R {
                    properties: None,
                    content: vec![RunContent::Field(Field::new(instruction, "stored"))],
                    extra_xml: Vec::new(),
                    extra_xml_positions: Vec::new(),
                    alt_drawings: Vec::new(),
                });
            }
            heading_restart
                .document
                .body
                .content
                .push(BodyContent::Paragraph(paragraph));
        }
        assert_eq!(
            heading_restart
                .evaluate_fields(&FieldEvaluationContext::default())
                .unwrap()
                .into_iter()
                .map(|result| result.outcome)
                .collect::<Vec<_>>(),
            [
                FieldOutcome::Resolved("1".to_owned()),
                FieldOutcome::Resolved("2".to_owned()),
                FieldOutcome::Resolved("1".to_owned()),
            ]
        );
    }

    #[test]
    fn missing_context_and_unsupported_fields_keep_their_cached_display() {
        let document = document_with_fields(&[("DATE", "stored"), ("UNKNOWN", "stored")]);
        let results = document
            .evaluate_fields(&FieldEvaluationContext::default())
            .unwrap();
        assert_eq!(results.len(), 2, "fallback outcomes must still be reported");
    }

    #[test]
    fn document_properties_variables_and_author_use_package_values() {
        let document = document_with_fields(&[("AUTHOR", "stored")]);
        let results = document
            .evaluate_fields(&FieldEvaluationContext::default())
            .unwrap();
        assert_eq!(results.len(), 1, "package-backed fields must be reported");
    }

    #[test]
    fn styleref_searches_the_approved_direction_and_scope() {
        let mut document = Document::new();
        document.add_style(style::StyleBuilder::paragraph("Heading1", "Heading 1"));
        let mut source = CT_P::new();
        source.properties = Some(CT_PPr {
            style_id: Some("Heading1".to_owned()),
            num_id: Some(7),
            num_ilvl: Some(0),
            ..Default::default()
        });
        source.runs.push(CT_R::new("numbered heading"));
        document
            .document
            .body
            .content
            .push(BodyContent::Paragraph(source));
        let mut field = CT_P::new();
        field.runs.push(CT_R {
            properties: None,
            content: vec![RunContent::Field(Field::new(
                r#"STYLEREF "Heading 1" \n"#,
                "stored",
            ))],
            extra_xml: Vec::new(),
            extra_xml_positions: Vec::new(),
            alt_drawings: Vec::new(),
        });
        document
            .document
            .body
            .content
            .push(BodyContent::Paragraph(field));
        let results = document
            .evaluate_fields(&FieldEvaluationContext::default())
            .unwrap();
        assert_eq!(results.len(), 1, "style fields must be reported");
        assert_eq!(
            results[0].outcome,
            keep("STYLEREF numbered source formatting is unsupported")
        );

        let BodyContent::Paragraph(source) = &mut document.document.body.content[0] else {
            unreachable!()
        };
        source.properties.as_mut().unwrap().num_id = Some(0);
        assert_eq!(
            document
                .evaluate_fields(&FieldEvaluationContext::default())
                .unwrap()[0]
                .outcome,
            FieldOutcome::Resolved("numbered heading".to_owned())
        );
    }

    #[test]
    fn date_time_filename_mergefield_and_includetext_use_only_explicit_context() {
        let document = document_with_fields(&[("FILENAME", "stored")]);
        let context = FieldEvaluationContext {
            file_name: Some("report.docx".to_owned()),
            ..FieldEvaluationContext::default()
        };
        let results = document.evaluate_fields(&context).unwrap();
        assert_eq!(
            results[0].outcome,
            FieldOutcome::Resolved("report.docx".to_owned())
        );
    }

    #[test]
    fn formatting_switches_match_the_pinned_word_matrix() {
        let document = document_with_fields(&[(r#"MERGEFIELD Name \* Upper"#, "stored")]);
        let mut context = FieldEvaluationContext::default();
        context
            .merge_fields
            .insert("Name".to_owned(), "field value".to_owned());
        let results = document.evaluate_fields(&context).unwrap();
        assert_eq!(
            results[0].outcome,
            FieldOutcome::Resolved("FIELD VALUE".to_owned())
        );
        assert_eq!(apply_general_format("FiELD", "Lower").unwrap(), "field");
        assert_eq!(
            apply_general_format("field VALUE", "FirstCap").unwrap(),
            "Field VALUE"
        );
        assert_eq!(
            apply_general_format("field value", "Caps").unwrap(),
            "Field Value"
        );
        assert_eq!(apply_general_format("27", "Arabic").unwrap(), "27");
        assert_eq!(apply_general_format("same", "MERGEFORMAT").unwrap(), "same");
        assert_eq!(apply_general_format("same", "Charformat").unwrap(), "same");
        assert_eq!(apply_general_format("27", "alphabetic").unwrap(), "aa");
        assert_eq!(apply_general_format("27", "ALPHABETIC").unwrap(), "AA");
        assert_eq!(apply_general_format("14", "roman").unwrap(), "xiv");
        assert_eq!(apply_general_format("14", "ROMAN").unwrap(), "XIV");
        assert_eq!(apply_general_format("22", "Ordinal").unwrap(), "22nd");
        assert_eq!(
            format_numeric_picture(1234.5, "#,##0.00").unwrap(),
            "1,234.50"
        );
        assert_eq!(
            format_numeric_picture(-12.0, "$0.00;($0.00);\"zero\"").unwrap(),
            "($12.00)"
        );
        assert_eq!(
            format_numeric_picture(0.0, "$0.00;($0.00);\"zero\"").unwrap(),
            "zero"
        );
        assert_eq!(format_numeric_picture(0.0, "#").unwrap(), " ");
        assert_eq!(format_numeric_picture(15.0, "$###").unwrap(), "$ 15");
        assert_eq!(
            format_numeric_picture(12.5, "$##0.00 'is sales tax'").unwrap(),
            "$ 12.50 is sales tax"
        );
        assert_eq!(format_numeric_picture(15.0, "#'x'##").unwrap(), " x15");
        assert_eq!(
            format_numeric_picture(123456.0, "000'-'000").unwrap(),
            "123-456"
        );
        let now = FieldDateTime {
            year: 2025,
            month: 12,
            day: 14,
            hour: 21,
            minute: 7,
            second: 5,
        };
        assert_eq!(
            format_date_time(now, "dddd, MMMM d, yyyy HH:mm:ss AM/PM").unwrap(),
            "Sunday, December 14, 2025 21:07:05 PM"
        );
        let property = Field::new(r#"DOCPROPERTY SavedAt \@ "MMMM d, yyyy""#, "stored");
        assert_eq!(
            apply_formats(&property.instruction, "2025-12-14T21:07:05Z", None).unwrap(),
            "December 14, 2025"
        );
        let merge = Field::new(r#"MERGEFIELD Date \@ "yyyy-MM-dd""#, "stored");
        assert_eq!(
            apply_formats(&merge.instruction, "2026-01-02", None).unwrap(),
            "2026-01-02"
        );
    }

    #[test]
    fn wildcard_if_matches_multiline_nested_ref_values() {
        let mut document =
            document_with_fields(&[(r#"IF left = "first*second" yes no"#, "stored")]);
        let BodyContent::Paragraph(paragraph) = &mut document.document.body.content[0] else {
            unreachable!()
        };
        let RunContent::Field(outer) = &mut paragraph.runs[0].content[0] else {
            unreachable!()
        };
        outer.instruction.arguments[0] =
            FieldArgument::Nested(Box::new(Field::new("REF Multi", "stored ref")));

        let BodyContent::Paragraph(paragraph) = &document.document.body.content[0] else {
            unreachable!()
        };
        let paragraphs = [paragraph];
        let context = FieldEvaluationContext::default();
        let mut evaluator = Evaluator::new(&document, &context);
        evaluator
            .bookmarks
            .insert("Multi".to_owned(), "first\nsecond".to_owned());
        evaluator.evaluate_story("main", &paragraphs);
        assert_eq!(
            evaluator.results[0].outcome,
            FieldOutcome::Resolved("yes".to_owned())
        );
        assert_eq!(
            evaluator.results[1].outcome,
            FieldOutcome::Resolved("first\nsecond".to_owned())
        );
    }
}
