//! Public comment handles and atomic document comment mutations.

use std::collections::{HashMap, HashSet};

use rdocx_oxml::comments::{CT_Comment, CT_Comments};
use rdocx_oxml::comments_extended::{CT_CommentEx, CT_CommentsEx};
use rdocx_oxml::document::BodyContent;
use rdocx_oxml::table::{CT_Tbl, CellContent};
use rdocx_oxml::text::{CT_P, CT_R, CommentRangeMarker, HyperlinkSpan, RunContent};

use crate::{Document, Error, Result};

pub(crate) const COMMENTS_EXTENDED_REL_TYPE: &str =
    "http://schemas.microsoft.com/office/2011/relationships/commentsExtended";
pub(crate) const COMMENTS_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml";
pub(crate) const COMMENTS_EXTENDED_CONTENT_TYPE: &str =
    "application/vnd.ms-word.commentsExtended+xml";
const DEFAULT_COMMENTS_PART: &str = "/word/comments.xml";
const DEFAULT_COMMENTS_EXTENDED_PART: &str = "/word/commentsExtended.xml";

/// A stable insertion point between runs in a body paragraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RunPosition {
    /// Index in the document body's paragraph and table sequence.
    pub body_index: usize,
    /// Run insertion index in the selected paragraph.
    pub run_index: usize,
}

/// A half-open document run range, inclusive at `start` and exclusive at `end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RunRange {
    pub start: RunPosition,
    pub end: RunPosition,
}

/// Read-only view of a comment and its thread metadata.
#[derive(Debug, Clone, Copy)]
pub struct CommentRef<'a> {
    inner: &'a CT_Comment,
    extension: Option<&'a CT_CommentEx>,
    parent_id: Option<i32>,
}

impl CommentRef<'_> {
    pub fn id(&self) -> i32 {
        self.inner.id
    }

    pub fn author(&self) -> Option<&str> {
        self.inner.author.as_deref()
    }

    pub fn initials(&self) -> Option<&str> {
        self.inner.initials.as_deref()
    }

    pub fn date(&self) -> Option<&str> {
        self.inner.date.as_deref()
    }

    pub fn text(&self) -> String {
        self.inner
            .paragraphs
            .iter()
            .map(CT_P::text)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn parent_id(&self) -> Option<i32> {
        self.parent_id
    }

    pub fn resolved(&self) -> bool {
        self.extension.and_then(|entry| entry.done).unwrap_or(false)
    }
}

impl Document {
    /// Return comments in their package part order.
    pub fn comments(&self) -> Vec<CommentRef<'_>> {
        let Some(comments) = self.comments.as_ref() else {
            return Vec::new();
        };
        let by_para_id = comments
            .comments
            .iter()
            .filter_map(|comment| first_para_id(comment).map(|para_id| (para_id, comment.id)))
            .collect::<HashMap<_, _>>();
        comments
            .comments
            .iter()
            .map(|comment| {
                let extension = first_para_id(comment).and_then(|para_id| {
                    self.comments_extended
                        .as_ref()?
                        .comments
                        .iter()
                        .find(|entry| entry.para_id == para_id)
                });
                let parent_id = extension
                    .and_then(|entry| entry.para_id_parent.as_deref())
                    .and_then(|para_id| by_para_id.get(para_id).copied());
                CommentRef {
                    inner: comment,
                    extension,
                    parent_id,
                }
            })
            .collect()
    }

    /// Add a comment over a half-open range of body paragraph runs.
    pub fn add_comment(
        &mut self,
        range: RunRange,
        author: &str,
        initials: Option<&str>,
        text: &str,
    ) -> Result<i32> {
        self.validate_run_range(range)?;
        let id = allocate_comment_id(self.comments.as_ref())?;
        let para_id = allocate_para_id(self.comments.as_ref(), self.comments_extended.as_ref())?;
        self.ensure_comment_models();

        let mut paragraph = CT_P::new();
        paragraph.add_run(text);
        self.comments
            .as_mut()
            .expect("comment model was initialized")
            .comments
            .push(CT_Comment {
                id,
                author: Some(author.to_owned()),
                date: None,
                initials: initials.map(str::to_owned),
                paragraphs: vec![paragraph],
                paragraph_ids: vec![Some(para_id.clone())],
                extra_attributes: Vec::new(),
                extra_xml: Vec::new(),
            });
        self.comments_extended
            .as_mut()
            .expect("comments-extended model was initialized")
            .comments
            .push(CT_CommentEx {
                para_id,
                para_id_parent: None,
                done: None,
                extra_attributes: Vec::new(),
            });

        let end = body_paragraph_mut(&mut self.document.body.content, range.end.body_index)
            .expect("range was validated");
        insert_comment_reference(end, range.end.run_index, id);
        let start = body_paragraph_mut(&mut self.document.body.content, range.start.body_index)
            .expect("range was validated");
        start.comment_ranges.push(CommentRangeMarker::Start {
            id,
            run_index: range.start.run_index,
            raw_before: raw_count_at(start, range.start.run_index),
        });
        let end = body_paragraph_mut(&mut self.document.body.content, range.end.body_index)
            .expect("range was validated");
        end.comment_ranges.push(CommentRangeMarker::End {
            id,
            run_index: range.end.run_index,
            raw_before: raw_count_at(end, range.end.run_index),
        });
        self.invalidate_layout();
        Ok(id)
    }

    /// Add a reply linked to the selected comment paragraph.
    pub fn reply_to(&mut self, parent_id: i32, author: &str, text: &str) -> Result<i32> {
        let parent_index = self
            .comments
            .as_ref()
            .and_then(|comments| {
                comments
                    .comments
                    .iter()
                    .position(|item| item.id == parent_id)
            })
            .ok_or_else(|| Error::Other(format!("comment id {parent_id} does not exist")))?;
        let id = allocate_comment_id(self.comments.as_ref())?;
        let existing_parent_para_id = first_para_id(
            &self
                .comments
                .as_ref()
                .expect("parent lookup proved the model exists")
                .comments[parent_index],
        )
        .map(str::to_owned);
        let parent_para_id = match existing_parent_para_id {
            Some(para_id) => para_id,
            None => allocate_para_id(self.comments.as_ref(), self.comments_extended.as_ref())?,
        };
        let para_id = allocate_para_id_with_reserved(
            self.comments.as_ref(),
            self.comments_extended.as_ref(),
            Some(&parent_para_id),
        )?;
        self.ensure_comment_models();

        let comments = self.comments.as_mut().expect("model was initialized");
        let parent = &mut comments.comments[parent_index];
        if parent.paragraphs.is_empty() {
            parent.paragraphs.push(CT_P::new());
        }
        if parent.paragraph_ids.len() < parent.paragraphs.len() {
            parent.paragraph_ids.resize(parent.paragraphs.len(), None);
        }
        parent.paragraph_ids[0] = Some(parent_para_id.clone());
        let mut paragraph = CT_P::new();
        paragraph.add_run(text);
        comments.comments.push(CT_Comment {
            id,
            author: Some(author.to_owned()),
            date: None,
            initials: None,
            paragraphs: vec![paragraph],
            paragraph_ids: vec![Some(para_id.clone())],
            extra_attributes: Vec::new(),
            extra_xml: Vec::new(),
        });

        let extended = self
            .comments_extended
            .as_mut()
            .expect("model was initialized");
        if extended
            .comments
            .iter()
            .all(|entry| entry.para_id != parent_para_id)
        {
            extended.comments.push(CT_CommentEx {
                para_id: parent_para_id.clone(),
                para_id_parent: None,
                done: None,
                extra_attributes: Vec::new(),
            });
        }
        extended.comments.push(CT_CommentEx {
            para_id,
            para_id_parent: Some(parent_para_id),
            done: None,
            extra_attributes: Vec::new(),
        });
        self.invalidate_layout();
        Ok(id)
    }

    /// Set or clear the resolved state for a comment.
    pub fn resolve_comment(&mut self, id: i32, resolved: bool) -> Result<bool> {
        let Some(comment_index) = self
            .comments
            .as_ref()
            .and_then(|comments| comments.comments.iter().position(|item| item.id == id))
        else {
            return Ok(false);
        };
        let existing_para_id = first_para_id(
            &self
                .comments
                .as_ref()
                .expect("comment lookup proved the model exists")
                .comments[comment_index],
        )
        .map(str::to_owned);
        let para_id = match existing_para_id {
            Some(para_id) => para_id,
            None => allocate_para_id(self.comments.as_ref(), self.comments_extended.as_ref())?,
        };
        self.ensure_comment_models();
        let comment = &mut self
            .comments
            .as_mut()
            .expect("model was initialized")
            .comments[comment_index];
        if comment.paragraphs.is_empty() {
            comment.paragraphs.push(CT_P::new());
        }
        if comment.paragraph_ids.len() < comment.paragraphs.len() {
            comment.paragraph_ids.resize(comment.paragraphs.len(), None);
        }
        comment.paragraph_ids[0] = Some(para_id.clone());
        let extended = self
            .comments_extended
            .as_mut()
            .expect("model was initialized");
        let root_para_id = thread_root_para_id(extended, &para_id);
        if let Some(entry) = extended
            .comments
            .iter_mut()
            .find(|entry| entry.para_id == root_para_id)
        {
            entry.done = Some(resolved);
        } else {
            extended.comments.push(CT_CommentEx {
                para_id: root_para_id,
                para_id_parent: None,
                done: Some(resolved),
                extra_attributes: Vec::new(),
            });
        }
        self.invalidate_layout();
        Ok(true)
    }

    /// Remove a comment and every reply descended from it.
    pub fn remove_comment(&mut self, id: i32) -> Result<bool> {
        let Some(comments) = self.comments.as_ref() else {
            return Ok(false);
        };
        if comments.comments.iter().all(|comment| comment.id != id) {
            return Ok(false);
        }

        let id_by_para = comments
            .comments
            .iter()
            .filter_map(|comment| first_para_id(comment).map(|para| (para.to_owned(), comment.id)))
            .collect::<HashMap<_, _>>();
        let mut removed_ids = HashSet::from([id]);
        let mut removed_para_ids = comments
            .comments
            .iter()
            .filter(|comment| comment.id == id)
            .filter_map(first_para_id)
            .map(str::to_owned)
            .collect::<HashSet<_>>();
        if let Some(extended) = self.comments_extended.as_ref() {
            loop {
                let mut changed = false;
                for entry in &extended.comments {
                    if entry
                        .para_id_parent
                        .as_ref()
                        .is_some_and(|parent| removed_para_ids.contains(parent))
                        && removed_para_ids.insert(entry.para_id.clone())
                    {
                        if let Some(comment_id) = id_by_para.get(&entry.para_id) {
                            removed_ids.insert(*comment_id);
                        }
                        changed = true;
                    }
                }
                if !changed {
                    break;
                }
            }
        }

        let comments = self.comments.as_mut().expect("model exists");
        let removed_comment_entries = comments
            .comments
            .iter()
            .map(|comment| removed_ids.contains(&comment.id))
            .collect::<Vec<_>>();
        comments.comments = comments
            .comments
            .drain(..)
            .zip(&removed_comment_entries)
            .filter_map(|(comment, remove)| (!remove).then_some(comment))
            .collect();
        remap_raw_positions(&mut comments.extra_xml, &removed_comment_entries);
        if let Some(extended) = self.comments_extended.as_mut() {
            let removed_extension_entries = extended
                .comments
                .iter()
                .map(|entry| removed_para_ids.contains(&entry.para_id))
                .collect::<Vec<_>>();
            extended.comments = extended
                .comments
                .drain(..)
                .zip(&removed_extension_entries)
                .filter_map(|(entry, remove)| (!remove).then_some(entry))
                .collect();
            remap_raw_positions(&mut extended.extra_xml, &removed_extension_entries);
        }
        for content in &mut self.document.body.content {
            remove_anchors_from_body_content(content, &removed_ids);
        }
        self.remove_owned_empty_comment_parts();
        self.invalidate_layout();
        Ok(true)
    }

    fn validate_run_range(&self, range: RunRange) -> Result<()> {
        if range.start > range.end {
            return Err(Error::Other(
                "comment range start must not follow its end".to_owned(),
            ));
        }
        for (label, position) in [("start", range.start), ("end", range.end)] {
            let paragraph = body_paragraph(&self.document.body.content, position.body_index)
                .ok_or_else(|| {
                    Error::Other(format!(
                        "comment range {label} body index {} is not a paragraph",
                        position.body_index
                    ))
                })?;
            if position.run_index > paragraph.runs.len() {
                return Err(Error::Other(format!(
                    "comment range {label} run index {} exceeds paragraph run count {}",
                    position.run_index,
                    paragraph.runs.len()
                )));
            }
        }
        Ok(())
    }

    fn ensure_comment_models(&mut self) {
        if self.comments.is_none() {
            self.comments = Some(CT_Comments::new());
        }
        if self.comments_part_name.is_none() {
            self.comments_part_name = Some(allocate_part_name(
                &self.package.parts,
                DEFAULT_COMMENTS_PART,
            ));
            self.comments_owned = true;
        }
        if self.comments_extended.is_none() {
            self.comments_extended = Some(CT_CommentsEx::new());
        }
        if self.comments_extended_part_name.is_none() {
            self.comments_extended_part_name = Some(allocate_part_name(
                &self.package.parts,
                DEFAULT_COMMENTS_EXTENDED_PART,
            ));
            self.comments_extended_owned = true;
        }
    }

    fn remove_owned_empty_comment_parts(&mut self) {
        if self
            .comments
            .as_ref()
            .is_some_and(|comments| comments.comments.is_empty())
            && self.comments_owned
        {
            if let Some(part) = self.comments_part_name.take() {
                remove_owned_part(self, &part, oxml_opc::relationship::rel_types::COMMENTS);
            }
            self.comments = None;
            self.comments_owned = false;
        }
        if self
            .comments_extended
            .as_ref()
            .is_some_and(|extended| extended.comments.is_empty())
            && self.comments_extended_owned
        {
            if let Some(part) = self.comments_extended_part_name.take() {
                remove_owned_part(self, &part, COMMENTS_EXTENDED_REL_TYPE);
            }
            self.comments_extended = None;
            self.comments_extended_owned = false;
        }
    }
}

fn first_para_id(comment: &CT_Comment) -> Option<&str> {
    comment.paragraph_ids.first()?.as_deref()
}

fn body_paragraph(content: &[BodyContent], index: usize) -> Option<&CT_P> {
    match content.get(index)? {
        BodyContent::Paragraph(paragraph) => Some(paragraph),
        BodyContent::Table(_) | BodyContent::RawXml(_) => None,
    }
}

fn body_paragraph_mut(content: &mut [BodyContent], index: usize) -> Option<&mut CT_P> {
    match content.get_mut(index)? {
        BodyContent::Paragraph(paragraph) => Some(paragraph),
        BodyContent::Table(_) | BodyContent::RawXml(_) => None,
    }
}

fn raw_count_at(paragraph: &CT_P, run_index: usize) -> usize {
    paragraph
        .extra_xml
        .iter()
        .filter(|(position, _)| *position == run_index)
        .count()
}

fn comment_reference_run(id: i32) -> CT_R {
    CT_R {
        properties: None,
        content: vec![RunContent::CommentReference { id, raw_before: 0 }],
        extra_xml: Vec::new(),
        alt_drawings: Vec::new(),
    }
}

fn insert_comment_reference(paragraph: &mut CT_P, run_index: usize, id: i32) {
    for marker in &mut paragraph.comment_ranges {
        match marker {
            CommentRangeMarker::Start { run_index: at, .. }
            | CommentRangeMarker::End { run_index: at, .. }
                if *at >= run_index =>
            {
                *at += 1
            }
            CommentRangeMarker::Start { .. } | CommentRangeMarker::End { .. } => {}
        }
    }
    let mut hyperlinks = Vec::with_capacity(paragraph.hyperlinks.len() + 1);
    for mut hyperlink in paragraph.hyperlinks.drain(..) {
        if hyperlink.run_start >= run_index {
            hyperlink.run_start += 1;
            hyperlink.run_end += 1;
            hyperlinks.push(hyperlink);
        } else if hyperlink.run_end > run_index {
            let suffix = HyperlinkSpan {
                rel_id: hyperlink.rel_id.clone(),
                anchor: hyperlink.anchor.clone(),
                run_start: run_index + 1,
                run_end: hyperlink.run_end + 1,
            };
            hyperlink.run_end = run_index;
            hyperlinks.push(hyperlink);
            hyperlinks.push(suffix);
        } else {
            hyperlinks.push(hyperlink);
        }
    }
    paragraph.hyperlinks = hyperlinks;
    for (position, _) in &mut paragraph.extra_xml {
        if *position >= run_index {
            *position += 1;
        }
    }
    paragraph.runs.insert(run_index, comment_reference_run(id));
}

fn thread_root_para_id(extended: &CT_CommentsEx, para_id: &str) -> String {
    let mut current = para_id.to_owned();
    let mut seen = HashSet::new();
    while seen.insert(current.clone()) {
        let Some(parent) = extended
            .comments
            .iter()
            .find(|entry| entry.para_id == current)
            .and_then(|entry| entry.para_id_parent.as_ref())
        else {
            break;
        };
        current.clone_from(parent);
    }
    current
}

fn allocate_comment_id(comments: Option<&CT_Comments>) -> Result<i32> {
    let occupied = comments
        .into_iter()
        .flat_map(|comments| comments.comments.iter().map(|comment| comment.id))
        .collect::<HashSet<_>>();
    if let Some(max) = occupied.iter().copied().max()
        && max < i32::MAX
    {
        return Ok(max + 1);
    }
    (0..=i32::MAX)
        .find(|candidate| !occupied.contains(candidate))
        .ok_or_else(|| Error::Other("no available comment id remains".to_owned()))
}

fn allocate_para_id(
    comments: Option<&CT_Comments>,
    extended: Option<&CT_CommentsEx>,
) -> Result<String> {
    allocate_para_id_with_reserved(comments, extended, None)
}

fn allocate_para_id_with_reserved(
    comments: Option<&CT_Comments>,
    extended: Option<&CT_CommentsEx>,
    reserved: Option<&str>,
) -> Result<String> {
    let mut occupied = comments
        .into_iter()
        .flat_map(|comments| comments.comments.iter())
        .flat_map(|comment| comment.paragraph_ids.iter())
        .filter_map(Option::as_deref)
        .filter_map(parse_para_id)
        .collect::<HashSet<_>>();
    occupied.extend(
        extended
            .into_iter()
            .flat_map(|extended| extended.comments.iter())
            .filter_map(|entry| parse_para_id(&entry.para_id)),
    );
    if let Some(reserved) = reserved.and_then(parse_para_id) {
        occupied.insert(reserved);
    }
    if let Some(max) = occupied.iter().copied().max()
        && max < u32::MAX
    {
        return Ok(format!("{:08X}", max + 1));
    }
    (1..=u32::MAX)
        .find(|candidate| !occupied.contains(candidate))
        .map(|candidate| format!("{candidate:08X}"))
        .ok_or_else(|| Error::Other("no available comment paragraph id remains".to_owned()))
}

fn parse_para_id(value: &str) -> Option<u32> {
    (value.len() == 8)
        .then(|| u32::from_str_radix(value, 16).ok())
        .flatten()
}

fn allocate_part_name(parts: &HashMap<String, Vec<u8>>, preferred: &str) -> String {
    if !parts.contains_key(preferred) {
        return preferred.to_owned();
    }
    let (stem, extension) = preferred.rsplit_once('.').unwrap_or((preferred, "xml"));
    (2..)
        .map(|index| format!("{stem}{index}.{extension}"))
        .find(|candidate| !parts.contains_key(candidate))
        .expect("the finite package cannot occupy every usize part suffix")
}

fn remove_owned_part(document: &mut Document, part: &str, relationship_type: &str) {
    document.package.parts.remove(part);
    document.package.content_types.overrides.remove(part);
    if let Some(relationships) = document.package.part_rels.get_mut(&document.doc_part_name) {
        relationships
            .items
            .retain(|relationship| relationship.rel_type != relationship_type);
    }
}

fn remove_anchors_from_body_content(content: &mut BodyContent, ids: &HashSet<i32>) {
    match content {
        BodyContent::Paragraph(paragraph) => remove_anchors_from_paragraph(paragraph, ids),
        BodyContent::Table(table) => remove_anchors_from_table(table, ids),
        BodyContent::RawXml(_) => {}
    }
}

fn remove_anchors_from_table(table: &mut CT_Tbl, ids: &HashSet<i32>) {
    for row in &mut table.rows {
        for cell in &mut row.cells {
            for content in &mut cell.content {
                match content {
                    CellContent::Paragraph(paragraph) => {
                        remove_anchors_from_paragraph(paragraph, ids)
                    }
                    CellContent::Table(table) => remove_anchors_from_table(table, ids),
                }
            }
        }
    }
}

fn remove_anchors_from_paragraph(paragraph: &mut CT_P, ids: &HashSet<i32>) {
    paragraph.comment_ranges.retain(|marker| match marker {
        CommentRangeMarker::Start { id, .. } | CommentRangeMarker::End { id, .. } => {
            !ids.contains(id)
        }
    });
    let mut removed = Vec::with_capacity(paragraph.runs.len());
    for run in &mut paragraph.runs {
        let removed_reference = run
            .content
            .iter()
            .any(|content| matches!(content, RunContent::CommentReference { id, .. } if ids.contains(id)));
        run.content.retain(|content| {
            !matches!(content, RunContent::CommentReference { id, .. } if ids.contains(id))
        });
        removed.push(
            removed_reference
                && run.content.is_empty()
                && run.extra_xml.is_empty()
                && run.alt_drawings.is_empty(),
        );
    }
    if removed.iter().all(|remove| !remove) {
        return;
    }
    let remap = |index: usize| {
        index.saturating_sub(
            removed
                .iter()
                .take(index.min(removed.len()))
                .filter(|remove| **remove)
                .count(),
        )
    };
    paragraph.runs = paragraph
        .runs
        .drain(..)
        .zip(&removed)
        .filter_map(|(run, remove)| (!remove).then_some(run))
        .collect();
    for marker in &mut paragraph.comment_ranges {
        match marker {
            CommentRangeMarker::Start { run_index, .. }
            | CommentRangeMarker::End { run_index, .. } => *run_index = remap(*run_index),
        }
    }
    for hyperlink in &mut paragraph.hyperlinks {
        hyperlink.run_start = remap(hyperlink.run_start);
        hyperlink.run_end = remap(hyperlink.run_end);
    }
    for (position, _) in &mut paragraph.extra_xml {
        *position = remap(*position);
    }
}

fn remap_raw_positions(extra_xml: &mut [(usize, Vec<u8>)], removed: &[bool]) {
    for (position, _) in extra_xml {
        *position = position.saturating_sub(
            removed
                .iter()
                .take((*position).min(removed.len()))
                .filter(|remove| **remove)
                .count(),
        );
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use super::*;

    const WORD_VERSION: &str = "16.104";
    const WORD_BUILD: &str = "16.104.25121423";
    const WORD_COMMENT_CANDIDATE_SHA256: &str =
        "a5ad0e8eb2d1a676daa07431deb2a0f11ee32e8bb92d099d14d5d16d43708adb";

    fn word_comment_candidate() -> Document {
        let mut document = Document::new();
        let mut paragraph = document.add_paragraph("");
        paragraph.add_run("Review ");
        paragraph.add_run("this sentence.");
        let root = document
            .add_comment(
                RunRange {
                    start: RunPosition {
                        body_index: 0,
                        run_index: 0,
                    },
                    end: RunPosition {
                        body_index: 0,
                        run_index: 2,
                    },
                },
                "Ada Lovelace",
                Some("AL"),
                "Please verify this sentence.",
            )
            .expect("add candidate comment");
        document
            .reply_to(root, "Ben", "Verified and ready.")
            .expect("add candidate reply");
        assert!(
            document
                .resolve_comment(root, true)
                .expect("resolve candidate thread")
        );
        document
    }

    #[test]
    fn comment_reference_is_inserted_at_the_half_open_end() {
        let mut document = Document::new();
        let mut paragraph = document.add_paragraph("");
        paragraph.add_run("left");
        paragraph.add_run("right");
        let id = document
            .add_comment(
                RunRange {
                    start: RunPosition {
                        body_index: 0,
                        run_index: 0,
                    },
                    end: RunPosition {
                        body_index: 0,
                        run_index: 1,
                    },
                },
                "Ada",
                None,
                "Review",
            )
            .unwrap();

        let BodyContent::Paragraph(paragraph) = &document.document.body.content[0] else {
            panic!("body item should remain a paragraph");
        };
        assert!(matches!(
            paragraph.runs[1].content.as_slice(),
            [RunContent::CommentReference { id: reference, .. }] if *reference == id
        ));
        assert!(paragraph.comment_ranges.iter().any(|marker| matches!(
            marker,
            CommentRangeMarker::End {
                id: marker_id,
                run_index: 1,
                ..
            } if *marker_id == id
        )));
    }

    #[test]
    fn comment_reference_splits_a_hyperlink_at_the_range_end() {
        let mut document = Document::new();
        let mut paragraph = document.add_paragraph("");
        paragraph.add_run("left");
        paragraph.add_run("right");
        let BodyContent::Paragraph(paragraph) = &mut document.document.body.content[0] else {
            panic!("body item should remain a paragraph");
        };
        paragraph.hyperlinks.push(HyperlinkSpan {
            rel_id: Some("rIdLink".to_owned()),
            anchor: None,
            run_start: 0,
            run_end: 2,
        });

        document
            .add_comment(
                RunRange {
                    start: RunPosition {
                        body_index: 0,
                        run_index: 0,
                    },
                    end: RunPosition {
                        body_index: 0,
                        run_index: 1,
                    },
                },
                "Ada",
                None,
                "Review",
            )
            .unwrap();

        let BodyContent::Paragraph(paragraph) = &document.document.body.content[0] else {
            panic!("body item should remain a paragraph");
        };
        assert_eq!(paragraph.hyperlinks.len(), 2);
        assert_eq!(paragraph.hyperlinks[0].run_start, 0);
        assert_eq!(paragraph.hyperlinks[0].run_end, 1);
        assert_eq!(paragraph.hyperlinks[1].run_start, 2);
        assert_eq!(paragraph.hyperlinks[1].run_end, 3);
    }

    #[test]
    fn resolving_a_reply_resolves_the_thread_root() {
        let mut document = Document::new();
        document.add_paragraph("thread");
        let root = document
            .add_comment(
                RunRange {
                    start: RunPosition {
                        body_index: 0,
                        run_index: 0,
                    },
                    end: RunPosition {
                        body_index: 0,
                        run_index: 1,
                    },
                },
                "Ada",
                None,
                "Review",
            )
            .unwrap();
        let reply = document.reply_to(root, "Ben", "Done").unwrap();

        assert!(document.resolve_comment(reply, true).unwrap());
        let comments = document.comments();
        assert!(comments[0].resolved());
        assert!(!comments[1].resolved());
    }

    #[test]
    fn removing_a_reference_run_keeps_an_unrelated_empty_run() {
        let mut paragraph = CT_P::new();
        paragraph.runs.push(CT_R {
            properties: None,
            content: Vec::new(),
            extra_xml: Vec::new(),
            alt_drawings: Vec::new(),
        });
        let mut reference = comment_reference_run(7);
        reference.properties = Some(Default::default());
        paragraph.runs.push(reference);

        remove_anchors_from_paragraph(&mut paragraph, &HashSet::from([7]));

        assert_eq!(paragraph.runs.len(), 1);
        assert!(paragraph.runs[0].content.is_empty());
    }

    #[test]
    fn word_comment_candidate_is_bound_to_recorded_sha() {
        let output = std::env::temp_dir().join(format!(
            "rdocx-f148-word-comment-{}.docx",
            std::process::id()
        ));
        word_comment_candidate()
            .save(&output)
            .expect("write SHA-bound candidate");
        assert_eq!(sha256(&output), WORD_COMMENT_CANDIDATE_SHA256);
        fs::remove_file(output).expect("remove temporary candidate");
    }

    #[test]
    #[ignore = "requires pinned Microsoft Word and human thread UI evidence"]
    fn word_opens_comment_reply_and_resolved_thread_without_repair() {
        let output = std::env::var_os("RDOCX_WORD_COMMENT_GATE_OUTPUT")
            .map(PathBuf::from)
            .expect("set RDOCX_WORD_COMMENT_GATE_OUTPUT to the SHA-bound .docx path");
        word_comment_candidate()
            .save(&output)
            .expect("write Word comment candidate");
        assert_eq!(sha256(&output), WORD_COMMENT_CANDIDATE_SHA256);
        let plist = "/Applications/Microsoft Word.app/Contents/Info.plist";
        assert_eq!(
            plist_value(plist, "CFBundleShortVersionString"),
            WORD_VERSION
        );
        assert_eq!(plist_value(plist, "CFBundleVersion"), WORD_BUILD);
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
            .expect("shasum output is utf8")
            .split_whitespace()
            .next()
            .expect("shasum digest")
            .to_owned()
    }

    fn plist_value(path: &str, key: &str) -> String {
        let output = Command::new("/usr/libexec/PlistBuddy")
            .args(["-c", &format!("Print :{key}"), path])
            .output()
            .expect("read application plist");
        assert!(output.status.success());
        String::from_utf8(output.stdout)
            .expect("plist value is utf8")
            .trim()
            .to_owned()
    }
}
