//! Public comment handles and atomic document comment mutations.

use std::collections::{BTreeMap, HashMap, HashSet};

use oxml_opc::OpcPackage;
use rdocx_oxml::comments::{CT_Comment, CT_Comments};
use rdocx_oxml::comments_extended::{CT_CommentEx, CT_CommentsEx};
use rdocx_oxml::content_control::{CT_Sdt, SdtContent};
use rdocx_oxml::document::BodyContent;
use rdocx_oxml::table::{CT_Row, CT_Tbl, CT_Tc, CellContent};
use rdocx_oxml::text::{CT_P, RangeAnchor};
#[cfg(test)]
use rdocx_oxml::text::{CT_R, CommentRangeMarker, HyperlinkSpan, RunContent};

use crate::document::visit_body_paragraphs_mut;
use crate::{ContentLocation, Document, Error, Result};

/// Refuses comment fields holding a character XML 1.0 cannot carry, as
/// python-docx refuses such text.
fn reject_non_xml_comment(author: &str, initials: Option<&str>, text: &str) -> Result<()> {
    oxml_core::xml::reject_non_xml_characters("comment author", author)?;
    oxml_core::xml::reject_non_xml_characters("comment initials", initials.unwrap_or_default())?;
    oxml_core::xml::reject_non_xml_characters("comment text", text)?;
    Ok(())
}

pub(crate) const COMMENTS_EXTENDED_REL_TYPE: &str =
    "http://schemas.microsoft.com/office/2011/relationships/commentsExtended";
pub(crate) const COMMENTS_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml";
pub(crate) const COMMENTS_EXTENDED_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtended+xml";
const DEFAULT_COMMENTS_PART: &str = "/word/comments.xml";
const DEFAULT_COMMENTS_EXTENDED_PART: &str = "/word/commentsExtended.xml";

/// A stable insertion point between runs in a body paragraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RunPosition {
    /// Direct body child index, where a table or a block content control
    /// counts as one child. `Document::find_content_index` returns it.
    pub body_index: usize,
    /// Run boundary in the selected paragraph, counted over the runs that
    /// `Paragraph::runs` lists, including the runs inside inline content
    /// controls and tracked insertions.
    pub run_index: usize,
}

/// A half-open document run range, inclusive at `start` and exclusive at `end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RunRange {
    pub start: RunPosition,
    pub end: RunPosition,
}

/// A stable insertion point between runs in a checked story paragraph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StoryRunPosition {
    pub location: ContentLocation,
    pub run_index: usize,
}

/// A half-open run range within one checked story owner.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StoryRunRange {
    pub start: StoryRunPosition,
    pub end: StoryRunPosition,
}

/// Immutable summary of one correlated bookmark or one reported marker issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookmarkRef {
    id: Option<i32>,
    name: Option<String>,
    range: Option<RunRange>,
    direct_range: Option<RunRange>,
    text: String,
    issue: Option<String>,
}

impl BookmarkRef {
    pub fn id(&self) -> Option<i32> {
        self.id
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Return the accepted-view half-open range reported by `Document::bookmarks`.
    ///
    /// Its body index is the paragraph ordinal counted recursively through
    /// tables and block content controls, which field numbering reads.
    /// [`Self::direct_range`] reports the direct body child index instead.
    pub fn range(&self) -> Option<RunRange> {
        self.range
    }

    /// Return the same range with the direct body child index that
    /// `RunPosition`, `Document::add_bookmark` and
    /// `Document::find_content_index` use.
    ///
    /// It is `None` when either marker sits in a table cell or a block content
    /// control, which have no direct body index. Run indexes are the same
    /// accepted-view boundaries as [`Self::range`], which are the
    /// `RunPosition` run indexes that `Document::add_bookmark` and
    /// `Document::add_comment` take.
    pub fn direct_range(&self) -> Option<RunRange> {
        self.direct_range
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn issue(&self) -> Option<&str> {
        self.issue.as_deref()
    }
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
    pub(crate) fn ensure_fragment_comment_models_staged(&mut self) -> Result<String> {
        self.ensure_comment_models()?;
        self.ensure_comment_relationships()?;
        self.comments_part_name
            .clone()
            .ok_or_else(|| Error::Other("comments part name is missing".to_owned()))
    }

    pub(crate) fn fragment_comment_dependency_xml(
        source: Option<&CT_Comments>,
        source_extended: Option<&CT_CommentsEx>,
        ids: &[String],
    ) -> Result<Option<Vec<u8>>> {
        if ids.is_empty() {
            return Ok(None);
        }
        let source = source.ok_or_else(|| {
            Error::Other("document fragment references a missing comments part".to_owned())
        })?;
        let included_ids = selected_fragment_comment_ids(source, source_extended, ids)?;
        let mut selected = source.clone();
        selected
            .comments
            .retain(|comment| included_ids.contains(&comment.id));
        selected.extra_xml.clear();
        Ok(Some(selected.to_xml()?))
    }

    pub(crate) fn import_fragment_comments_staged(
        &mut self,
        source: Option<&CT_Comments>,
        source_extended: Option<&CT_CommentsEx>,
        ids: &[String],
    ) -> Result<BTreeMap<String, String>> {
        if ids.is_empty() {
            return Ok(BTreeMap::new());
        }
        let source = source.ok_or_else(|| {
            Error::Other("document fragment references a missing comments part".to_owned())
        })?;
        self.ensure_comment_models()?;
        self.ensure_comment_relationships()?;
        let included_ids = selected_fragment_comment_ids(source, source_extended, ids)?;

        let mut remap = BTreeMap::<String, String>::new();
        let mut paragraph_remap = HashMap::<String, String>::new();
        let mut reserved_para_ids =
            occupied_para_ids(self.comments.as_ref(), self.comments_extended.as_ref());
        for comment in source
            .comments
            .iter()
            .filter(|comment| included_ids.contains(&comment.id))
        {
            let new_id = self.identifiers.reserve_comment_id()?;
            remap.insert(comment.id.to_string(), new_id.to_string());
            for old_para_id in comment.paragraph_ids.iter().flatten() {
                let new_para_id = allocate_para_id_from_occupied(&mut reserved_para_ids)?;
                paragraph_remap.insert(old_para_id.clone(), new_para_id);
            }
        }

        let mut imported_comments = Vec::new();
        for source_comment in source
            .comments
            .iter()
            .filter(|comment| included_ids.contains(&comment.id))
        {
            let new_id = remap
                .get(&source_comment.id.to_string())
                .and_then(|id| id.parse::<i32>().ok())
                .ok_or_else(|| {
                    Error::Other("document fragment comment remap is incomplete".to_owned())
                })?;
            let mut imported = source_comment.clone();
            imported.id = new_id;
            for para_id in imported.paragraph_ids.iter_mut().flatten() {
                *para_id = paragraph_remap.get(para_id).cloned().ok_or_else(|| {
                    Error::Other("document fragment paragraph remap is incomplete".to_owned())
                })?;
            }
            imported_comments.push(imported);
        }
        self.comments
            .as_mut()
            .expect("fragment comment models were initialized")
            .comments
            .extend(imported_comments);

        if let Some(source_extended) = source_extended {
            let mut imported_extended = Vec::new();
            for extension in &source_extended.comments {
                let Some(para_id) = paragraph_remap.get(&extension.para_id).cloned() else {
                    continue;
                };
                let para_id_parent = extension
                    .para_id_parent
                    .as_ref()
                    .map(|parent| {
                        paragraph_remap.get(parent).cloned().ok_or_else(|| {
                            Error::Other(
                                "document fragment comment parent is outside the selected closure"
                                    .to_owned(),
                            )
                        })
                    })
                    .transpose()?;
                let mut imported = extension.clone();
                imported.para_id = para_id;
                imported.para_id_parent = para_id_parent;
                imported_extended.push(imported);
            }
            self.comments_extended
                .as_mut()
                .expect("fragment comment extension model was initialized")
                .comments
                .extend(imported_extended);
        }
        self.comments_dirty = true;
        Ok(remap)
    }

    /// Return bookmarks and malformed marker reports in main-story paragraph order.
    ///
    /// Reported body indexes count typed paragraphs recursively through tables and
    /// block content controls. Reported run indexes use accepted-view run boundaries.
    /// `BookmarkRef::direct_range` also reports the direct body child index when
    /// both markers sit in direct body paragraphs.
    pub fn bookmarks(&self) -> Vec<BookmarkRef> {
        #[derive(Clone)]
        struct Marker {
            position: RunPosition,
            direct_position: Option<RunPosition>,
            start: bool,
            id: Option<i32>,
            name: Option<String>,
        }

        let mut markers = Vec::new();
        let mut paragraphs = Vec::new();
        for (direct_index, item) in self.document.body.content.iter().enumerate() {
            let mut item_paragraphs = Vec::new();
            collect_main_story_paragraphs(std::slice::from_ref(item), &mut item_paragraphs);
            let direct_index = matches!(item, BodyContent::Paragraph(_)).then_some(direct_index);
            paragraphs.extend(
                item_paragraphs
                    .into_iter()
                    .map(|paragraph| (paragraph, direct_index)),
            );
        }
        for (body_index, (paragraph, direct_index)) in paragraphs.into_iter().enumerate() {
            for marker in &paragraph.bookmark_markers {
                let run_index = marker.projected_run_index();
                markers.push(Marker {
                    position: RunPosition {
                        body_index,
                        run_index,
                    },
                    direct_position: direct_index.map(|body_index| RunPosition {
                        body_index,
                        run_index,
                    }),
                    start: marker.is_start(),
                    id: marker.id(),
                    name: marker.name().map(str::to_owned),
                });
            }
        }

        let mut by_id: HashMap<i32, Vec<usize>> = HashMap::new();
        let mut results = Vec::new();
        for (index, marker) in markers.iter().enumerate() {
            if let Some(id) = marker.id {
                by_id.entry(id).or_default().push(index);
            } else {
                results.push((
                    index,
                    BookmarkRef {
                        id: None,
                        name: marker.name.clone(),
                        range: None,
                        direct_range: None,
                        text: String::new(),
                        issue: Some("bookmark marker has a malformed or missing id".to_owned()),
                    },
                ));
            }
        }

        for (id, indices) in by_id {
            let starts = indices
                .iter()
                .copied()
                .filter(|index| markers[*index].start)
                .collect::<Vec<_>>();
            let ends = indices
                .iter()
                .copied()
                .filter(|index| !markers[*index].start)
                .collect::<Vec<_>>();
            let first = indices.iter().copied().min().unwrap_or(0);
            let name = starts
                .first()
                .and_then(|index| markers[*index].name.clone());
            let (range, issue) = if starts.len() != 1 || ends.len() != 1 {
                (
                    None,
                    Some(format!(
                        "bookmark id {id} has {} start markers and {} end markers",
                        starts.len(),
                        ends.len()
                    )),
                )
            } else if name.is_none() {
                (None, Some(format!("bookmark id {id} has a missing name")))
            } else {
                let candidate = RunRange {
                    start: markers[starts[0]].position,
                    end: markers[ends[0]].position,
                };
                if candidate.start > candidate.end
                    || (candidate.start == candidate.end && starts[0] > ends[0])
                {
                    (
                        None,
                        Some(format!("bookmark id {id} ends before it starts")),
                    )
                } else {
                    (Some(candidate), None)
                }
            };
            let direct_range = range.and_then(|_| {
                Some(RunRange {
                    start: markers[starts[0]].direct_position?,
                    end: markers[ends[0]].direct_position?,
                })
            });
            let text = range
                .map(|_| {
                    bookmark_range_text(
                        &self.document.body.content,
                        markers[starts[0]].position.body_index,
                        markers[starts[0]].position.run_index,
                        markers[ends[0]].position.body_index,
                        markers[ends[0]].position.run_index,
                    )
                })
                .unwrap_or_default();
            results.push((
                first,
                BookmarkRef {
                    id: Some(id),
                    name,
                    range,
                    direct_range,
                    text,
                    issue,
                },
            ));
        }

        let mut name_counts = HashMap::new();
        for (_, bookmark) in &results {
            if bookmark.range.is_some()
                && let Some(name) = bookmark.name.as_deref()
            {
                *name_counts.entry(name.to_owned()).or_insert(0usize) += 1;
            }
        }
        for (_, bookmark) in &mut results {
            if bookmark
                .name
                .as_ref()
                .is_some_and(|name| name_counts.get(name).copied().unwrap_or(0) > 1)
            {
                bookmark.issue = Some(format!(
                    "bookmark name {} is duplicated",
                    bookmark.name.as_deref().unwrap_or("")
                ));
                bookmark.range = None;
                bookmark.direct_range = None;
                bookmark.text.clear();
            }
        }
        results.sort_by_key(|(index, _)| *index);
        results.into_iter().map(|(_, bookmark)| bookmark).collect()
    }

    /// Insert a bookmark over a half-open range of body paragraph runs.
    ///
    /// Run indexes count the runs that `Paragraph::runs` lists. The markers
    /// go inside an inline content control when the range starts or ends
    /// between two of its runs, and around the control when the range covers
    /// it. A range that cannot be anchored exactly, such as one that crosses
    /// the edge of a control or ends between two runs of a tracked insertion,
    /// is an error and leaves the document unchanged.
    pub fn add_bookmark(&mut self, name: &str, range: RunRange) -> Result<i32> {
        self.insert_bookmark(name, range)
    }

    fn insert_bookmark(&mut self, name: &str, range: RunRange) -> Result<i32> {
        validate_bookmark_name(name)?;
        validate_bookmark_range(&self.document.body.content, range)?;
        if self
            .bookmarks()
            .iter()
            .any(|bookmark| bookmark.name() == Some(name))
        {
            return Err(Error::Other(format!("bookmark name {name} already exists")));
        }
        let mut identifiers = self.identifiers.clone();
        let id = identifiers.reserve_bookmark_id()?;
        anchor_body_range(
            &mut self.document.body.content,
            range,
            RangeAnchor::Bookmark { id, name },
            "bookmark",
        )?;
        self.identifiers = identifiers;
        self.invalidate_layout();
        Ok(id)
    }

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
    ///
    /// Run indexes count the runs that `Paragraph::runs` lists, and the
    /// range markers are placed as [`Self::add_bookmark`] places its markers.
    /// The reference run follows the end marker. A range that cannot be
    /// anchored exactly is an error and leaves the document unchanged.
    pub fn add_comment(
        &mut self,
        range: RunRange,
        author: &str,
        initials: Option<&str>,
        text: &str,
    ) -> Result<i32> {
        self.add_comment_with_date(range, author, initials, text, None)
    }

    /// Add a dated comment over a half-open range of body paragraph runs.
    ///
    /// `date`, when present, must be an RFC 3339 timestamp. No date is the
    /// deterministic default used by [`Document::add_comment`].
    pub fn add_comment_with_date(
        &mut self,
        range: RunRange,
        author: &str,
        initials: Option<&str>,
        text: &str,
        date: Option<&str>,
    ) -> Result<i32> {
        reject_non_xml_comment(author, initials, text)?;
        let mut candidate = self.clone_for_staging();
        let id = candidate.add_comment_staged(range, author, initials, text, date)?;
        candidate.flush_dirty_related_story_models()?;
        self.commit_staged_mutation(candidate);
        Ok(id)
    }

    /// Add a dated comment over a checked body, table-cell, header, footer, or note run range.
    ///
    /// A body location can also name a paragraph inside a block content
    /// control with the two-segment path that
    /// [`Document::paragraph_story_location`] returns. Run indexes count the
    /// runs that `Paragraph::runs` lists, as in [`Document::add_comment`].
    pub fn add_story_comment_with_date(
        &mut self,
        range: StoryRunRange,
        author: &str,
        initials: Option<&str>,
        text: &str,
        date: Option<&str>,
    ) -> Result<i32> {
        let related_part = matches!(
            range.start.location.story().kind(),
            crate::StoryKind::Header
                | crate::StoryKind::Footer
                | crate::StoryKind::Footnote
                | crate::StoryKind::Endnote
        );
        let mut candidate = self.clone_for_staging();
        let id = candidate.add_story_comment_staged(range, author, initials, text, date)?;
        if related_part {
            let reopened = candidate.prepare_and_reopen_staged()?;
            self.commit_staged_mutation(reopened);
        } else {
            candidate.flush_dirty_related_story_models()?;
            self.commit_staged_mutation(candidate);
        }
        Ok(id)
    }

    /// Add a comment over a checked body, table-cell, header, footer, or note run range, as
    /// [`Self::add_story_comment_with_date`] does without a date.
    pub fn add_story_comment(
        &mut self,
        range: StoryRunRange,
        author: &str,
        initials: Option<&str>,
        text: &str,
    ) -> Result<i32> {
        self.add_story_comment_with_date(range, author, initials, text, None)
    }

    fn add_story_comment_staged(
        &mut self,
        range: StoryRunRange,
        author: &str,
        initials: Option<&str>,
        text: &str,
        date: Option<&str>,
    ) -> Result<i32> {
        validate_comment_date(date)?;
        if range.start.location.story() != range.end.location.story()
            || range.start.location.index_path() > range.end.location.index_path()
        {
            return Err(Error::Other(
                "comment story range start must not follow its end".to_owned(),
            ));
        }
        if matches!(
            range.start.location.story().kind(),
            crate::StoryKind::Header
                | crate::StoryKind::Footer
                | crate::StoryKind::Footnote
                | crate::StoryKind::Endnote
        ) {
            let mut identifiers = self.identifiers.clone();
            let id = identifiers.reserve_comment_id()?;
            self.anchor_related_story_comment(
                &range.start.location,
                range.start.run_index,
                &range.end.location,
                range.end.run_index,
                id,
            )?;
            self.ensure_comment_models()?;
            self.ensure_comment_relationships()?;
            self.push_comment_definition(id, author, initials, text, date)?;
            self.identifiers = identifiers;
            self.comments_dirty = true;
            self.invalidate_layout();
            return Ok(id);
        }
        let mut start = self.story_paragraph_mut(&range.start.location)?.clone();
        let mut end = self.story_paragraph_mut(&range.end.location)?.clone();
        for (label, position, paragraph) in
            [("start", &range.start, &start), ("end", &range.end, &end)]
        {
            let run_count = paragraph.accepted_run_paths().len();
            if position.run_index > run_count {
                return Err(Error::Other(format!(
                    "comment range {label} run index {} exceeds paragraph run count {run_count}",
                    position.run_index,
                )));
            }
        }
        if range.start.location == range.end.location && range.start.run_index > range.end.run_index
        {
            return Err(Error::Other(
                "comment story range start must not follow its end".to_owned(),
            ));
        }

        let mut identifiers = self.identifiers.clone();
        let id = identifiers.reserve_comment_id()?;
        if range.start.location == range.end.location {
            anchor_paragraph_range(
                &mut start,
                Some(range.start.run_index),
                Some(range.end.run_index),
                RangeAnchor::Comment(id),
                "comment",
            )?;
            *self.story_paragraph_mut(&range.start.location)? = start;
        } else {
            anchor_paragraph_range(
                &mut start,
                Some(range.start.run_index),
                None,
                RangeAnchor::Comment(id),
                "comment",
            )?;
            anchor_paragraph_range(
                &mut end,
                None,
                Some(range.end.run_index),
                RangeAnchor::Comment(id),
                "comment",
            )?;
            *self.story_paragraph_mut(&range.start.location)? = start;
            *self.story_paragraph_mut(&range.end.location)? = end;
        }

        self.ensure_comment_models()?;
        self.ensure_comment_relationships()?;
        self.push_comment_definition(id, author, initials, text, date)?;
        self.identifiers = identifiers;
        self.comments_dirty = true;
        self.invalidate_layout();
        Ok(id)
    }

    fn add_comment_staged(
        &mut self,
        range: RunRange,
        author: &str,
        initials: Option<&str>,
        text: &str,
        date: Option<&str>,
    ) -> Result<i32> {
        validate_comment_date(date)?;
        self.validate_run_range(range)?;
        self.ensure_comment_models()?;
        self.ensure_comment_relationships()?;
        let mut identifiers = self.identifiers.clone();
        let id = identifiers.reserve_comment_id()?;
        self.push_comment_definition(id, author, initials, text, date)?;
        anchor_body_range(
            &mut self.document.body.content,
            range,
            RangeAnchor::Comment(id),
            "comment",
        )?;
        self.identifiers = identifiers;
        self.comments_dirty = true;
        self.invalidate_layout();
        Ok(id)
    }

    /// Add a comment on the `occurrence`-th match of `anchor`, counted from
    /// zero, in the main story.
    ///
    /// Matches are case-sensitive and non-overlapping, in document order
    /// through body paragraphs, tables and block content controls, and one
    /// match never spans two paragraphs. They are found in the literal run
    /// text that [`Document::split_run`] offsets count, so tabs and breaks
    /// have no width. The runs at both ends of the match are split and the
    /// comment is anchored on the runs between the splits as
    /// [`Self::add_comment`] anchors a run range. `date`, when present, must
    /// be an RFC 3339 timestamp. A missing occurrence or a match that cannot
    /// be anchored exactly is an error and leaves the document unchanged. A
    /// match is not exact when its range would also show text that the
    /// literal text leaves out, such as the result of a field between two of
    /// its runs.
    pub fn add_comment_on_text(
        &mut self,
        anchor: &str,
        occurrence: usize,
        author: &str,
        initials: Option<&str>,
        text: &str,
        date: Option<&str>,
    ) -> Result<i32> {
        reject_non_xml_comment(author, initials, text)?;
        let mut candidate = self.clone_for_staging();
        let id = candidate
            .add_comment_on_text_staged(anchor, occurrence, author, initials, text, date)?;
        candidate.flush_dirty_related_story_models()?;
        self.commit_staged_mutation(candidate);
        Ok(id)
    }

    fn add_comment_on_text_staged(
        &mut self,
        anchor: &str,
        occurrence: usize,
        author: &str,
        initials: Option<&str>,
        text: &str,
        date: Option<&str>,
    ) -> Result<i32> {
        validate_comment_date(date)?;
        if anchor.is_empty() {
            return Err(Error::Other(
                "comment anchor text must not be empty".to_owned(),
            ));
        }
        self.ensure_comment_models()?;
        self.ensure_comment_relationships()?;
        let mut identifiers = self.identifiers.clone();
        let id = identifiers.reserve_comment_id()?;
        let mut remaining = occurrence;
        let mut anchored = None;
        visit_body_paragraphs_mut(&mut self.document.body.content, &mut |paragraph| {
            if anchored.is_some() {
                return;
            }
            let literal = paragraph.accepted_literal_text();
            let matches = literal
                .match_indices(anchor)
                .map(|(byte, _)| byte)
                .collect::<Vec<_>>();
            let Some(byte) = matches.get(remaining) else {
                remaining -= matches.len();
                return;
            };
            let start = literal[..*byte].chars().count();
            let end = start + anchor.chars().count();
            anchored = Some(
                paragraph
                    .split_accepted_literal_span(start, end)
                    .map_err(|error| {
                        Error::Other(format!("comment anchor text cannot be split: {error}"))
                    })
                    .and_then(|(start, end)| {
                        anchor_paragraph_range(
                            paragraph,
                            Some(start),
                            Some(end),
                            RangeAnchor::Comment(id),
                            "comment",
                        )
                    })
                    .and_then(|()| {
                        // Preserved children such as `w:fldSimple` are not in
                        // the literal text, but the range shows their text.
                        let shown = paragraph.comment_range_text(id).unwrap_or_default();
                        if shown == anchor {
                            Ok(())
                        } else {
                            Err(Error::Other(format!(
                                "comment anchor text {anchor:?} occurrence {occurrence} cannot be anchored exactly: its range would show {shown:?}"
                            )))
                        }
                    }),
            );
        });
        anchored.ok_or_else(|| {
            // Every paragraph was searched, so `remaining` counts past them all.
            let found = occurrence - remaining;
            let times = if found == 1 { "time" } else { "times" };
            Error::Other(format!(
                "comment anchor text {anchor:?} has no occurrence {occurrence}: it occurs {found} {times} in the main story"
            ))
        })??;
        self.push_comment_definition(id, author, initials, text, date)?;
        self.identifiers = identifiers;
        self.comments_dirty = true;
        self.invalidate_layout();
        Ok(id)
    }

    /// Append comment `id`, holding one text paragraph, and its thread entry
    /// to the comment models, which must already exist.
    fn push_comment_definition(
        &mut self,
        id: i32,
        author: &str,
        initials: Option<&str>,
        text: &str,
        date: Option<&str>,
    ) -> Result<()> {
        let para_id = allocate_para_id(self.comments.as_ref(), self.comments_extended.as_ref())?;
        let mut paragraph = CT_P::new();
        paragraph.add_run(text);
        self.comments
            .as_mut()
            .expect("comment model was initialized")
            .comments
            .push(CT_Comment {
                id,
                author: Some(author.to_owned()),
                date: date.map(str::to_owned),
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
        Ok(())
    }

    /// Add a reply linked to the selected comment paragraph.
    pub fn reply_to(&mut self, parent_id: i32, author: &str, text: &str) -> Result<i32> {
        self.reply_to_with_date(parent_id, author, text, None)
    }

    /// Add a dated reply linked to the selected comment paragraph.
    ///
    /// `date`, when present, must be an RFC 3339 timestamp.
    pub fn reply_to_with_date(
        &mut self,
        parent_id: i32,
        author: &str,
        text: &str,
        date: Option<&str>,
    ) -> Result<i32> {
        let mut candidate = self.clone_for_staging();
        let id = candidate.reply_to_staged(parent_id, author, text, date)?;
        candidate.flush_dirty_related_story_models()?;
        self.commit_staged_mutation(candidate);
        Ok(id)
    }

    fn reply_to_staged(
        &mut self,
        parent_id: i32,
        author: &str,
        text: &str,
        date: Option<&str>,
    ) -> Result<i32> {
        validate_comment_date(date)?;
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
        self.ensure_comment_models()?;
        self.ensure_comment_relationships()?;
        let id = self.identifiers.reserve_comment_id()?;

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
            date: date.map(str::to_owned),
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
        self.comments_dirty = true;
        self.invalidate_layout();
        Ok(id)
    }

    /// Set or clear the resolved state for a comment.
    pub fn resolve_comment(&mut self, id: i32, resolved: bool) -> Result<bool> {
        let mut candidate = self.clone_for_staging();
        let updated = candidate.resolve_comment_staged(id, resolved)?;
        if updated {
            candidate.flush_dirty_related_story_models()?;
            self.commit_staged_mutation(candidate);
        }
        Ok(updated)
    }

    fn resolve_comment_staged(&mut self, id: i32, resolved: bool) -> Result<bool> {
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
        self.ensure_comment_models()?;
        self.ensure_comment_relationships()?;
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
        self.comments_dirty = true;
        self.invalidate_layout();
        Ok(true)
    }

    /// Remove a comment and every reply descended from it.
    pub fn remove_comment(&mut self, id: i32) -> Result<bool> {
        let mut candidate = self.clone_for_staging();
        let removed = candidate.remove_comment_staged(id)?;
        if removed {
            candidate.flush_dirty_related_story_models()?;
            self.commit_staged_mutation(candidate);
        }
        Ok(removed)
    }

    fn remove_comment_staged(&mut self, id: i32) -> Result<bool> {
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
        self.identifiers
            .retire_authored_comment_ids(removed_ids.iter().copied());
        self.remove_owned_empty_comment_parts();
        self.comments_dirty = self.comments.is_some();
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
            let run_count = paragraph.accepted_run_paths().len();
            if position.run_index > run_count {
                return Err(Error::Other(format!(
                    "comment range {label} run index {} exceeds paragraph run count {run_count}",
                    position.run_index,
                )));
            }
        }
        Ok(())
    }

    fn ensure_comment_models(&mut self) -> Result<()> {
        self.identifiers.observe_package_graph(&self.package)?;
        if self.comments.is_none() {
            self.comments = Some(CT_Comments::new());
        }
        if self.comments_part_name.is_none() {
            self.comments_part_name = Some(
                self.identifiers
                    .reserve_preferred_part_name(DEFAULT_COMMENTS_PART)?,
            );
            self.comments_owned = true;
        }
        if self.comments_extended.is_none() {
            self.comments_extended = Some(CT_CommentsEx::new());
        }
        if self.comments_extended_part_name.is_none() {
            self.comments_extended_part_name = Some(
                self.identifiers
                    .reserve_preferred_part_name(DEFAULT_COMMENTS_EXTENDED_PART)?,
            );
            self.comments_extended_owned = true;
        }
        Ok(())
    }

    fn ensure_comment_relationships(&mut self) -> Result<()> {
        let comments_part = self
            .comments_part_name
            .clone()
            .ok_or_else(|| Error::Other("comments part name is missing".to_owned()))?;
        let comments_extended_part = self
            .comments_extended_part_name
            .clone()
            .ok_or_else(|| Error::Other("comments-extended part name is missing".to_owned()))?;
        self.ensure_part_relationship_checked(
            &comments_part,
            oxml_opc::relationship::rel_types::COMMENTS,
            COMMENTS_CONTENT_TYPE,
        )
        .map_err(|error| {
            Error::Other(format!("comments relationship allocation failed: {error}"))
        })?;
        self.ensure_part_relationship_checked(
            &comments_extended_part,
            COMMENTS_EXTENDED_REL_TYPE,
            COMMENTS_EXTENDED_CONTENT_TYPE,
        )
        .map_err(|error| {
            Error::Other(format!(
                "comments-extended relationship allocation failed: {error}"
            ))
        })?;
        Ok(())
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

fn validate_comment_date(date: Option<&str>) -> Result<()> {
    if let Some(value) = date
        && crate::revision::parse_rfc3339(value).is_none()
    {
        return Err(Error::Other(format!(
            "invalid RFC 3339 comment timestamp: {value}"
        )));
    }
    Ok(())
}

fn selected_fragment_comment_ids(
    source: &CT_Comments,
    source_extended: Option<&CT_CommentsEx>,
    ids: &[String],
) -> Result<HashSet<i32>> {
    let mut included_ids = ids
        .iter()
        .map(|id| {
            id.parse::<i32>()
                .map_err(|_| Error::Other(format!("document fragment comment id {id} is invalid")))
        })
        .collect::<Result<HashSet<_>>>()?;
    for id in &included_ids {
        if !source.comments.iter().any(|comment| comment.id == *id) {
            return Err(Error::Other(format!(
                "document fragment comment id {id} has no definition"
            )));
        }
    }
    if let Some(extended) = source_extended {
        loop {
            let included_para_ids = source
                .comments
                .iter()
                .filter(|comment| included_ids.contains(&comment.id))
                .filter_map(first_para_id)
                .collect::<HashSet<_>>();
            let before = included_ids.len();
            for extension in &extended.comments {
                if extension
                    .para_id_parent
                    .as_deref()
                    .is_some_and(|parent| included_para_ids.contains(parent))
                    && let Some(comment) = source
                        .comments
                        .iter()
                        .find(|comment| first_para_id(comment) == Some(&extension.para_id))
                {
                    included_ids.insert(comment.id);
                }
            }
            if included_ids.len() == before {
                break;
            }
        }
    }
    Ok(included_ids)
}

fn first_para_id(comment: &CT_Comment) -> Option<&str> {
    comment.paragraph_ids.first()?.as_deref()
}

fn validate_bookmark_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(Error::Other("bookmark name must not be empty".to_owned()));
    }
    if name.starts_with('_') {
        return Err(Error::Other(format!(
            "bookmark name {name} is reserved for producer use"
        )));
    }
    if name.len() > 40
        || !name
            .chars()
            .next()
            .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
        || name
            .chars()
            .any(|character| !(character == '_' || character.is_ascii_alphanumeric()))
    {
        return Err(Error::Other(format!(
            "bookmark name {name} is not a valid Word bookmark name"
        )));
    }
    Ok(())
}

fn validate_bookmark_range(content: &[BodyContent], range: RunRange) -> Result<()> {
    if range.start > range.end {
        return Err(Error::Other(
            "bookmark range start must not follow its end".to_owned(),
        ));
    }
    for (label, position) in [("start", range.start), ("end", range.end)] {
        let paragraph = body_paragraph(content, position.body_index).ok_or_else(|| {
            Error::Other(format!(
                "bookmark range {label} body index {} is not a paragraph",
                position.body_index
            ))
        })?;
        let run_count = paragraph.accepted_run_paths().len();
        if position.run_index > run_count {
            return Err(Error::Other(format!(
                "bookmark range {label} run index {} exceeds paragraph run count {run_count}",
                position.run_index,
            )));
        }
    }
    Ok(())
}

fn bookmark_range_text(
    content: &[BodyContent],
    start_body_index: usize,
    start_run_index: usize,
    end_body_index: usize,
    end_run_index: usize,
) -> String {
    let mut paragraphs = Vec::<String>::new();
    for body_index in start_body_index..=end_body_index {
        let Some(paragraph) = story_paragraph(content, body_index) else {
            continue;
        };
        let runs = paragraph.accepted_bookmark_runs();
        let start = if body_index == start_body_index {
            start_run_index
        } else {
            0
        };
        let end = if body_index == end_body_index {
            end_run_index
        } else {
            runs.len()
        };
        let start = start.min(runs.len());
        let end = end.min(runs.len());
        paragraphs.push(runs[start..end].iter().map(|run| run.text()).collect());
    }
    paragraphs.join("\n")
}

fn story_paragraph(content: &[BodyContent], index: usize) -> Option<&CT_P> {
    let mut remaining = index;
    for item in content {
        if let Some(paragraph) = paragraph_in_body_content(item, &mut remaining) {
            return Some(paragraph);
        }
    }
    None
}

fn body_paragraph_mut(content: &mut [BodyContent], index: usize) -> Option<&mut CT_P> {
    match content.get_mut(index)? {
        BodyContent::Paragraph(paragraph) => Some(paragraph),
        BodyContent::Table(_) | BodyContent::ContentControl(_) | BodyContent::RawXml(_) => None,
    }
}

fn body_paragraph(content: &[BodyContent], index: usize) -> Option<&CT_P> {
    match content.get(index)? {
        BodyContent::Paragraph(paragraph) => Some(paragraph),
        BodyContent::Table(_) | BodyContent::ContentControl(_) | BodyContent::RawXml(_) => None,
    }
}

/// Write range markers over a validated body range. The paragraphs change
/// only when every marker can be placed exactly.
fn anchor_body_range(
    content: &mut [BodyContent],
    range: RunRange,
    anchor: RangeAnchor<'_>,
    label: &str,
) -> Result<()> {
    let (start, end) = (range.start, range.end);
    let paragraph = |content: &[BodyContent], index| {
        body_paragraph(content, index)
            .cloned()
            .expect("range was validated")
    };
    let mut first = paragraph(content, start.body_index);
    if start.body_index == end.body_index {
        anchor_paragraph_range(
            &mut first,
            Some(start.run_index),
            Some(end.run_index),
            anchor,
            label,
        )?;
    } else {
        let mut last = paragraph(content, end.body_index);
        anchor_paragraph_range(&mut first, Some(start.run_index), None, anchor, label)?;
        anchor_paragraph_range(&mut last, None, Some(end.run_index), anchor, label)?;
        *body_paragraph_mut(content, end.body_index).expect("range was validated") = last;
    }
    *body_paragraph_mut(content, start.body_index).expect("range was validated") = first;
    Ok(())
}

/// Write range markers at accepted-view run boundaries, the run index space
/// that `Paragraph::runs` lists. A missing side continues in another
/// paragraph.
fn anchor_paragraph_range(
    paragraph: &mut CT_P,
    start: Option<usize>,
    end: Option<usize>,
    anchor: RangeAnchor<'_>,
    label: &str,
) -> Result<()> {
    paragraph
        .anchor_accepted_range(start, end, anchor)
        .map_err(|error| Error::Other(format!("{label} range cannot be anchored: {error}")))
}

fn collect_main_story_paragraphs<'a>(content: &'a [BodyContent], output: &mut Vec<&'a CT_P>) {
    for item in content {
        match item {
            BodyContent::Paragraph(paragraph) => output.push(paragraph),
            BodyContent::Table(table) => collect_table_paragraphs(table, output),
            BodyContent::ContentControl(control) => {
                collect_control_paragraphs(control, BlockControlOwner::Body, output)
            }
            BodyContent::RawXml(_) => {}
        }
    }
}

fn collect_table_paragraphs<'a>(table: &'a CT_Tbl, output: &mut Vec<&'a CT_P>) {
    for boundary in 0..=table.rows.len() {
        for (_, _, control) in table
            .content_controls
            .iter()
            .filter(|(at, _, _)| *at == boundary)
        {
            collect_control_paragraphs(control, BlockControlOwner::Table, output);
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
            .filter(|(at, _, _)| *at == boundary)
        {
            collect_control_paragraphs(control, BlockControlOwner::Row, output);
        }
        if let Some(cell) = row.cells.get(boundary) {
            collect_cell_paragraphs(cell, output);
        }
    }
}

fn collect_cell_paragraphs<'a>(cell: &'a CT_Tc, output: &mut Vec<&'a CT_P>) {
    for item in &cell.content {
        match item {
            CellContent::Paragraph(paragraph) => output.push(paragraph),
            CellContent::Table(table) => collect_table_paragraphs(table, output),
            CellContent::ContentControl(control) => {
                collect_control_paragraphs(control, BlockControlOwner::Cell, output)
            }
        }
    }
}

#[derive(Clone, Copy)]
enum BlockControlOwner {
    Body,
    Table,
    Row,
    Cell,
}

fn collect_control_paragraphs<'a>(
    control: &'a CT_Sdt,
    owner: BlockControlOwner,
    output: &mut Vec<&'a CT_P>,
) {
    for item in &control.content {
        match (owner, item) {
            (
                BlockControlOwner::Body | BlockControlOwner::Cell,
                SdtContent::Paragraph(paragraph),
            ) => output.push(paragraph),
            (BlockControlOwner::Body | BlockControlOwner::Cell, SdtContent::Table(table)) => {
                collect_table_paragraphs(table, output)
            }
            (BlockControlOwner::Table, SdtContent::Row(row)) => collect_row_paragraphs(row, output),
            (BlockControlOwner::Row, SdtContent::Cell(cell)) => {
                collect_cell_paragraphs(cell, output)
            }
            (_, SdtContent::ContentControl(control)) => {
                collect_control_paragraphs(control, owner, output)
            }
            _ => {}
        }
    }
}

fn paragraph_in_body_content<'a>(
    content: &'a BodyContent,
    remaining: &mut usize,
) -> Option<&'a CT_P> {
    match content {
        BodyContent::Paragraph(paragraph) => take_paragraph(paragraph, remaining),
        BodyContent::Table(table) => paragraph_in_table(table, remaining),
        BodyContent::ContentControl(control) => {
            paragraph_in_control(control, BlockControlOwner::Body, remaining)
        }
        BodyContent::RawXml(_) => None,
    }
}

fn paragraph_in_table<'a>(table: &'a CT_Tbl, remaining: &mut usize) -> Option<&'a CT_P> {
    for boundary in 0..=table.rows.len() {
        for (_, _, control) in table
            .content_controls
            .iter()
            .filter(|(at, _, _)| *at == boundary)
        {
            if let Some(paragraph) =
                paragraph_in_control(control, BlockControlOwner::Table, remaining)
            {
                return Some(paragraph);
            }
        }
        if let Some(row) = table.rows.get(boundary)
            && let Some(paragraph) = paragraph_in_row(row, remaining)
        {
            return Some(paragraph);
        }
    }
    None
}

fn paragraph_in_row<'a>(row: &'a CT_Row, remaining: &mut usize) -> Option<&'a CT_P> {
    for boundary in 0..=row.cells.len() {
        for (_, _, control) in row
            .content_controls
            .iter()
            .filter(|(at, _, _)| *at == boundary)
        {
            if let Some(paragraph) =
                paragraph_in_control(control, BlockControlOwner::Row, remaining)
            {
                return Some(paragraph);
            }
        }
        if let Some(cell) = row.cells.get(boundary)
            && let Some(paragraph) = paragraph_in_cell(cell, remaining)
        {
            return Some(paragraph);
        }
    }
    None
}

fn paragraph_in_cell<'a>(cell: &'a CT_Tc, remaining: &mut usize) -> Option<&'a CT_P> {
    for content in &cell.content {
        let paragraph = match content {
            CellContent::Paragraph(paragraph) => take_paragraph(paragraph, remaining),
            CellContent::Table(table) => paragraph_in_table(table, remaining),
            CellContent::ContentControl(control) => {
                paragraph_in_control(control, BlockControlOwner::Cell, remaining)
            }
        };
        if paragraph.is_some() {
            return paragraph;
        }
    }
    None
}

fn paragraph_in_control<'a>(
    control: &'a CT_Sdt,
    owner: BlockControlOwner,
    remaining: &mut usize,
) -> Option<&'a CT_P> {
    for content in &control.content {
        let paragraph = match (owner, content) {
            (
                BlockControlOwner::Body | BlockControlOwner::Cell,
                SdtContent::Paragraph(paragraph),
            ) => take_paragraph(paragraph, remaining),
            (BlockControlOwner::Body | BlockControlOwner::Cell, SdtContent::Table(table)) => {
                paragraph_in_table(table, remaining)
            }
            (BlockControlOwner::Table, SdtContent::Row(row)) => paragraph_in_row(row, remaining),
            (BlockControlOwner::Row, SdtContent::Cell(cell)) => paragraph_in_cell(cell, remaining),
            (_, SdtContent::ContentControl(control)) => {
                paragraph_in_control(control, owner, remaining)
            }
            _ => None,
        };
        if paragraph.is_some() {
            return paragraph;
        }
    }
    None
}

fn take_paragraph<'a>(paragraph: &'a CT_P, remaining: &mut usize) -> Option<&'a CT_P> {
    if *remaining == 0 {
        Some(paragraph)
    } else {
        *remaining -= 1;
        None
    }
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
    let mut occupied = occupied_para_ids(comments, extended);
    if let Some(reserved) = reserved.and_then(parse_para_id) {
        occupied.insert(reserved);
    }
    allocate_para_id_from_occupied(&mut occupied)
}

fn occupied_para_ids(
    comments: Option<&CT_Comments>,
    extended: Option<&CT_CommentsEx>,
) -> HashSet<u32> {
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
    occupied
}

fn allocate_para_id_from_occupied(occupied: &mut HashSet<u32>) -> Result<String> {
    if let Some(max) = occupied.iter().copied().max()
        && max < u32::MAX
    {
        let allocated = max + 1;
        occupied.insert(allocated);
        return Ok(format!("{allocated:08X}"));
    }
    let allocated = (1..=u32::MAX)
        .find(|candidate| !occupied.contains(candidate))
        .ok_or_else(|| Error::Other("no available comment paragraph id remains".to_owned()))?;
    occupied.insert(allocated);
    Ok(format!("{allocated:08X}"))
}

fn parse_para_id(value: &str) -> Option<u32> {
    (value.len() == 8)
        .then(|| u32::from_str_radix(value, 16).ok())
        .flatten()
}

fn remove_owned_part(document: &mut Document, part: &str, relationship_type: &str) {
    document.package.remove_part(part);
    document.package.remove_part_rels(part);
    document.package.content_types.remove_override(part);
    document.identifiers.retire_authored_part(part);
    let owner = document.doc_part_name.clone();
    let mut removed_relationship_ids = Vec::new();
    if let Some(relationships) = document.package.get_part_rels_mut(&owner) {
        relationships.items.retain(|relationship| {
            let targets_part = relationship.rel_type == relationship_type
                && crate::document::relationship_is_internal(relationship)
                && OpcPackage::resolve_rel_target(&owner, &relationship.target) == part;
            let remove = targets_part
                && !document
                    .identifiers
                    .relationship_is_preserved(&owner, &relationship.id);
            if remove {
                removed_relationship_ids.push(relationship.id.clone());
            }
            !remove
        });
        if relationships.items.is_empty() {
            document.package.remove_part_rels(&owner);
        }
    }
    document
        .identifiers
        .retire_authored_story_relationships(&owner, removed_relationship_ids);
}

fn remove_anchors_from_body_content(content: &mut BodyContent, ids: &HashSet<i32>) {
    match content {
        BodyContent::Paragraph(paragraph) => remove_anchors_from_paragraph(paragraph, ids),
        BodyContent::Table(table) => remove_anchors_from_table(table, ids),
        BodyContent::ContentControl(control) => remove_anchors_from_control(control, ids),
        BodyContent::RawXml(_) => {}
    }
}

fn remove_anchors_from_table(table: &mut CT_Tbl, ids: &HashSet<i32>) {
    for (_, _, control) in &mut table.content_controls {
        remove_anchors_from_control(control, ids);
    }
    for row in &mut table.rows {
        remove_anchors_from_row(row, ids);
    }
}

fn remove_anchors_from_row(row: &mut CT_Row, ids: &HashSet<i32>) {
    for (_, _, control) in &mut row.content_controls {
        remove_anchors_from_control(control, ids);
    }
    for cell in &mut row.cells {
        remove_anchors_from_cell(cell, ids);
    }
}

fn remove_anchors_from_cell(cell: &mut CT_Tc, ids: &HashSet<i32>) {
    for content in &mut cell.content {
        match content {
            CellContent::Paragraph(paragraph) => remove_anchors_from_paragraph(paragraph, ids),
            CellContent::Table(table) => remove_anchors_from_table(table, ids),
            CellContent::ContentControl(control) => remove_anchors_from_control(control, ids),
        }
    }
}

fn remove_anchors_from_control(control: &mut CT_Sdt, ids: &HashSet<i32>) {
    // Markers written inside `w:sdtContent` are preserved children there.
    control.remove_comment_anchors(&ids.iter().copied().collect::<Vec<_>>());
    for content in &mut control.content {
        match content {
            SdtContent::Paragraph(paragraph) => remove_anchors_from_paragraph(paragraph, ids),
            SdtContent::Table(table) => remove_anchors_from_table(table, ids),
            SdtContent::Row(row) => remove_anchors_from_row(row, ids),
            SdtContent::Cell(cell) => remove_anchors_from_cell(cell, ids),
            SdtContent::ContentControl(control) => remove_anchors_from_control(control, ids),
            SdtContent::Run(_) | SdtContent::RawXml(_) => {}
        }
    }
}

fn remove_anchors_from_paragraph(paragraph: &mut CT_P, ids: &HashSet<i32>) {
    for (_, _, _, control) in &mut paragraph.content_controls {
        remove_anchors_from_control(control, ids);
    }
    let ids = ids.iter().copied().collect::<Vec<_>>();
    paragraph.remove_comment_anchors(&ids);
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
        "b7e1f39a5af80d9928ed671fa45557d485c2c70c8761439a09df66c027274995";

    fn word_comment_candidate() -> Document {
        let mut document =
            Document::new_with_profile(crate::document::WordCreationProfile::Minimal(
                crate::document::WordPackageClass::Document,
            ));
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
    fn adding_a_comment_reserves_both_relationships_before_publication() {
        let mut source = Document::new_with_profile(crate::document::WordCreationProfile::Minimal(
            crate::document::WordPackageClass::Document,
        ));
        source.add_paragraph("review");
        let source_bytes = source.to_bytes().unwrap();
        let mut package =
            oxml_opc::OpcPackage::from_reader(std::io::Cursor::new(source_bytes)).unwrap();
        package
            .get_or_create_part_rels("/word/document.xml")
            .add_with_id(
                &format!("rId{}", u32::MAX),
                "urn:exhaustion",
                "unchanged.bin",
            );
        let mut bytes = std::io::Cursor::new(Vec::new());
        package.write_to(&mut bytes).unwrap();
        let mut document = Document::from_bytes(bytes.get_ref()).unwrap();
        let package_bytes = |document: &Document| {
            let mut bytes = std::io::Cursor::new(Vec::new());
            document.package.write_to(&mut bytes).unwrap();
            bytes.into_inner()
        };
        let before = package_bytes(&document);

        let error = document
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
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("comments relationship allocation failed")
        );
        assert_eq!(package_bytes(&document), before);
        assert!(document.comments.is_none());
        assert!(document.comments_extended.is_none());
        assert!(document.comments_part_name.is_none());
        assert!(document.comments_extended_part_name.is_none());
    }

    #[test]
    fn reply_and_resolve_roll_back_comments_extended_relationship_exhaustion() {
        fn exhausted_document() -> Document {
            let mut source =
                Document::new_with_profile(crate::document::WordCreationProfile::Minimal(
                    crate::document::WordPackageClass::Document,
                ));
            source.add_paragraph("review");
            source
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
            let mut package =
                OpcPackage::from_reader(std::io::Cursor::new(source.to_bytes().unwrap())).unwrap();
            package.parts.remove(DEFAULT_COMMENTS_EXTENDED_PART);
            package
                .content_types
                .overrides
                .remove(DEFAULT_COMMENTS_EXTENDED_PART);
            let relationships = package.get_or_create_part_rels("/word/document.xml");
            relationships
                .items
                .retain(|relationship| relationship.rel_type != COMMENTS_EXTENDED_REL_TYPE);
            relationships.add_with_id(
                &format!("rId{}", u32::MAX),
                "urn:exhaustion",
                "unchanged.bin",
            );
            let mut bytes = std::io::Cursor::new(Vec::new());
            package.write_to(&mut bytes).unwrap();
            Document::from_bytes(bytes.get_ref()).unwrap()
        }

        let package_bytes = |document: &Document| {
            let mut bytes = std::io::Cursor::new(Vec::new());
            document.package.write_to(&mut bytes).unwrap();
            bytes.into_inner()
        };

        let mut reply = exhausted_document();
        let root = reply.comments()[0].id();
        let before = package_bytes(&reply);
        let error = reply.reply_to(root, "Ben", "Done").unwrap_err();
        assert!(
            error
                .to_string()
                .contains("comments-extended relationship allocation failed")
        );
        assert_eq!(package_bytes(&reply), before);
        assert!(reply.comments_extended.is_none());
        assert!(reply.comments_extended_part_name.is_none());
        assert_eq!(reply.comments().len(), 1);

        let mut resolved = exhausted_document();
        let root = resolved.comments()[0].id();
        let before = package_bytes(&resolved);
        let error = resolved.resolve_comment(root, true).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("comments-extended relationship allocation failed")
        );
        assert_eq!(package_bytes(&resolved), before);
        assert!(resolved.comments_extended.is_none());
        assert!(resolved.comments_extended_part_name.is_none());
        assert!(!resolved.comments()[0].resolved());
    }

    #[test]
    fn successful_legacy_comment_upgrades_keep_reserved_relationships_and_parts() {
        fn legacy_document() -> Document {
            let mut source =
                Document::new_with_profile(crate::document::WordCreationProfile::Minimal(
                    crate::document::WordPackageClass::Document,
                ));
            source.add_paragraph("review");
            source
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
            let mut package =
                OpcPackage::from_reader(std::io::Cursor::new(source.to_bytes().unwrap())).unwrap();
            package.parts.remove(DEFAULT_COMMENTS_EXTENDED_PART);
            package
                .content_types
                .overrides
                .remove(DEFAULT_COMMENTS_EXTENDED_PART);
            package
                .get_or_create_part_rels("/word/document.xml")
                .items
                .retain(|relationship| relationship.rel_type != COMMENTS_EXTENDED_REL_TYPE);
            let mut bytes = std::io::Cursor::new(Vec::new());
            package.write_to(&mut bytes).unwrap();
            Document::from_bytes(bytes.get_ref()).unwrap()
        }

        for reply in [false, true] {
            let mut document = legacy_document();
            let root = document.comments()[0].id();
            if reply {
                document.reply_to(root, "Ben", "Done").unwrap();
            } else {
                assert!(document.resolve_comment(root, true).unwrap());
            }
            let extended_part = document.comments_extended_part_name.clone().unwrap();
            let extended_relationship = document
                .package
                .get_part_rels("/word/document.xml")
                .and_then(|relationships| relationships.get_by_type(COMMENTS_EXTENDED_REL_TYPE))
                .unwrap()
                .id
                .clone();

            let hyperlink = document.add_hyperlink_relationship("https://example.com");
            let imported_part = document
                .identifiers
                .reserve_fragment_part_name(&extended_part)
                .unwrap();

            assert_ne!(hyperlink, extended_relationship);
            assert_ne!(imported_part, extended_part);
        }
    }

    #[test]
    fn removing_the_last_owned_comment_retires_its_complete_identifier_bundle() {
        fn final_document(with_history: bool) -> Document {
            let mut document =
                Document::new_with_profile(crate::document::WordCreationProfile::Minimal(
                    crate::document::WordPackageClass::Document,
                ));
            document.add_paragraph("review");
            let range = RunRange {
                start: RunPosition {
                    body_index: 0,
                    run_index: 0,
                },
                end: RunPosition {
                    body_index: 0,
                    run_index: 1,
                },
            };
            if with_history {
                let old = document.add_comment(range, "Ada", None, "Old").unwrap();
                assert!(document.remove_comment(old).unwrap());
                assert!(document.comments_part_name.is_none());
                assert!(document.comments_extended_part_name.is_none());
                assert!(document.package.get_part(DEFAULT_COMMENTS_PART).is_none());
                assert!(
                    document
                        .package
                        .get_part(DEFAULT_COMMENTS_EXTENDED_PART)
                        .is_none()
                );
            }
            let id = document.add_comment(range, "Ada", None, "Final").unwrap();
            assert_eq!(id, 0);
            document
        }

        let mut history = final_document(true);
        let mut direct = final_document(false);
        let history_bytes = history.to_bytes().unwrap();
        let direct_bytes = direct.to_bytes().unwrap();
        assert_eq!(history_bytes, direct_bytes);
        let history_package = OpcPackage::from_reader(std::io::Cursor::new(history_bytes)).unwrap();
        let direct_package = OpcPackage::from_reader(std::io::Cursor::new(direct_bytes)).unwrap();
        assert_eq!(
            history_package.get_part(DEFAULT_COMMENTS_PART),
            direct_package.get_part(DEFAULT_COMMENTS_PART)
        );
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
            tooltip: None,
            doc_location: None,
            run_start: 0,
            run_end: 2,
            extra_attributes: Vec::new(),
            extra_xml: Vec::new(),
            preserved_raw_before: None,
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
            extra_xml_positions: Vec::new(),
            alt_drawings: Vec::new(),
        });
        paragraph.runs.push(CT_R {
            properties: Some(Default::default()),
            content: vec![RunContent::CommentReference {
                id: 7,
                raw_before: 0,
            }],
            extra_xml: Vec::new(),
            extra_xml_positions: Vec::new(),
            alt_drawings: Vec::new(),
        });

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
