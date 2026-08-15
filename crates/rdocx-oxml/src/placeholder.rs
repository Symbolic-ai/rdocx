//! Placeholder replacement across paragraphs, tables, and headers/footers.
//!
//! Handles the cross-run splitting problem: a placeholder like `{{name}}`
//! may be split across multiple `<w:r>` elements in the OOXML source.

use crate::header_footer::CT_HdrFtr;
use crate::table::CT_Tbl;
use crate::text::{CT_P, CT_R, InlineChild, ParagraphChild, RunContent};

/// Replace all occurrences of `placeholder` with `replacement` in a paragraph.
///
/// Handles placeholders split across multiple runs within the same ordered
/// container and preserves the formatting of the first matched run. A match
/// never crosses a hyperlink, field, or unsupported-node boundary.
/// Returns the number of replacements made.
pub fn replace_in_paragraph(para: &mut CT_P, placeholder: &str, replacement: &str) -> usize {
    if placeholder.is_empty() {
        return 0;
    }

    replace_literal_in_paragraph_children(&mut para.content, placeholder, replacement)
}

fn replace_literal_in_paragraph_children(
    children: &mut Vec<ParagraphChild>,
    placeholder: &str,
    replacement: &str,
) -> usize {
    let mut total = 0;
    let mut index = 0;
    while index < children.len() {
        if matches!(children[index], ParagraphChild::Run(_)) {
            let end = children[index..]
                .iter()
                .position(|child| !matches!(child, ParagraphChild::Run(_)))
                .map_or(children.len(), |offset| index + offset);
            let mut runs = children
                .drain(index..end)
                .map(|child| match child {
                    ParagraphChild::Run(run) => run,
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>();
            total += replace_literal_in_runs(&mut runs, placeholder, replacement);
            let inserted = runs.len();
            children.splice(index..index, runs.into_iter().map(ParagraphChild::Run));
            index += inserted;
            continue;
        }

        match &mut children[index] {
            ParagraphChild::Hyperlink(hyperlink) => {
                if hyperlink.text().contains(placeholder) {
                    total += replace_literal_in_inline_children(
                        hyperlink.children_mut(),
                        placeholder,
                        replacement,
                    );
                }
            }
            ParagraphChild::SimpleField(field) => {
                if field.text().contains(placeholder) {
                    total += replace_literal_in_inline_children(
                        field.children_mut(),
                        placeholder,
                        replacement,
                    );
                }
            }
            ParagraphChild::Insertion(_)
            | ParagraphChild::Unsupported(_)
            | ParagraphChild::Run(_) => {}
        }
        index += 1;
    }
    total
}

fn replace_literal_in_inline_children(
    children: &mut Vec<InlineChild>,
    placeholder: &str,
    replacement: &str,
) -> usize {
    let mut total = 0;
    let mut index = 0;
    while index < children.len() {
        if matches!(children[index], InlineChild::Run(_)) {
            let end = children[index..]
                .iter()
                .position(|child| !matches!(child, InlineChild::Run(_)))
                .map_or(children.len(), |offset| index + offset);
            let mut runs = children
                .drain(index..end)
                .map(|child| match child {
                    InlineChild::Run(run) => run,
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>();
            total += replace_literal_in_runs(&mut runs, placeholder, replacement);
            let inserted = runs.len();
            children.splice(index..index, runs.into_iter().map(InlineChild::Run));
            index += inserted;
            continue;
        }
        if let InlineChild::SimpleField(field) = &mut children[index]
            && field.text().contains(placeholder)
        {
            total +=
                replace_literal_in_inline_children(field.children_mut(), placeholder, replacement);
        }
        index += 1;
    }
    total
}

fn replace_literal_in_runs(runs: &mut Vec<CT_R>, placeholder: &str, replacement: &str) -> usize {
    replace_matches_in_runs(runs, |text| {
        text.match_indices(placeholder)
            .map(|(start, _)| TextReplacement {
                start,
                end: start + placeholder.len(),
                replacement: replacement.to_string(),
            })
            .collect()
    })
}

#[derive(Debug, Clone, Copy)]
struct TextNodeLocation {
    run_index: usize,
    content_index: usize,
    segment_start: usize,
    segment_end: usize,
}

#[derive(Debug)]
struct TextSegment {
    text: String,
    nodes: Vec<TextNodeLocation>,
}

#[derive(Debug)]
struct TextReplacement {
    start: usize,
    end: usize,
    replacement: String,
}

fn replace_matches_in_runs(
    runs: &mut Vec<CT_R>,
    matches: impl Fn(&str) -> Vec<TextReplacement>,
) -> usize {
    let segments = build_text_segments(runs);
    let mut count = 0;

    for segment in &segments {
        let replacements = matches(&segment.text);
        count += replacements.len();
        for replacement in replacements.iter().rev() {
            apply_text_replacement(runs, &segment.nodes, replacement);
        }
    }

    for run in runs.iter_mut() {
        run.content
            .retain(|content| !matches!(content, RunContent::Text(text) if text.text.is_empty()));
    }
    runs.retain(|run| !run.content.is_empty());
    count
}

fn build_text_segments(runs: &[CT_R]) -> Vec<TextSegment> {
    let mut segments = Vec::new();
    let mut text = String::new();
    let mut nodes = Vec::new();

    for (run_index, run) in runs.iter().enumerate() {
        for (content_index, content) in run.content.iter().enumerate() {
            match content {
                RunContent::Text(value) => {
                    let segment_start = text.len();
                    text.push_str(&value.text);
                    nodes.push(TextNodeLocation {
                        run_index,
                        content_index,
                        segment_start,
                        segment_end: text.len(),
                    });
                }
                _ => finish_text_segment(&mut segments, &mut text, &mut nodes),
            }
        }
    }
    finish_text_segment(&mut segments, &mut text, &mut nodes);
    segments
}

fn finish_text_segment(
    segments: &mut Vec<TextSegment>,
    text: &mut String,
    nodes: &mut Vec<TextNodeLocation>,
) {
    if !text.is_empty() {
        segments.push(TextSegment {
            text: std::mem::take(text),
            nodes: std::mem::take(nodes),
        });
    } else {
        nodes.clear();
    }
}

fn apply_text_replacement(
    runs: &mut [CT_R],
    nodes: &[TextNodeLocation],
    replacement: &TextReplacement,
) {
    debug_assert!(replacement.start < replacement.end);
    let first_index = nodes
        .iter()
        .position(|node| replacement.start < node.segment_end)
        .expect("replacement start must belong to a text node");
    let last_index = nodes
        .iter()
        .rposition(|node| replacement.end > node.segment_start)
        .expect("replacement end must belong to a text node");
    let first = nodes[first_index];
    let last = nodes[last_index];
    let local_start = replacement.start - first.segment_start;
    let local_end = replacement.end - last.segment_start;

    if first_index == last_index {
        let text = text_at_mut(runs, first);
        text.text
            .replace_range(local_start..local_end, replacement.replacement.as_str());
        update_space_preservation(text);
        return;
    }

    let first_text = text_at_mut(runs, first);
    first_text.text.truncate(local_start);
    first_text.text.push_str(&replacement.replacement);
    update_space_preservation(first_text);

    for node in &nodes[(first_index + 1)..last_index] {
        text_at_mut(runs, *node).text.clear();
    }

    let last_text = text_at_mut(runs, last);
    last_text.text.drain(..local_end);
    update_space_preservation(last_text);
}

fn text_at_mut(runs: &mut [CT_R], location: TextNodeLocation) -> &mut crate::text::CT_Text {
    let RunContent::Text(text) = &mut runs[location.run_index].content[location.content_index]
    else {
        unreachable!("text segment location must refer to text")
    };
    text
}

fn update_space_preservation(text: &mut crate::text::CT_Text) {
    text.preserve_space = text.text.starts_with(' ') || text.text.ends_with(' ');
}

/// Replace all occurrences of `placeholder` in all paragraphs of a slice.
pub fn replace_in_paragraphs(paras: &mut [CT_P], placeholder: &str, replacement: &str) -> usize {
    paras
        .iter_mut()
        .map(|p| replace_in_paragraph(p, placeholder, replacement))
        .sum()
}

/// Replace all occurrences of `placeholder` in a table (recursively handles nested tables).
pub fn replace_in_table(table: &mut CT_Tbl, placeholder: &str, replacement: &str) -> usize {
    use crate::table::CellContent;

    let mut count = 0;
    for row in &mut table.rows {
        for cell in &mut row.cells {
            for content in &mut cell.content {
                match content {
                    CellContent::Paragraph(p) => {
                        count += replace_in_paragraph(p, placeholder, replacement);
                    }
                    CellContent::Table(nested) => {
                        count += replace_in_table(nested, placeholder, replacement);
                    }
                    CellContent::Unsupported(_) => {}
                }
            }
        }
    }
    count
}

/// Replace all occurrences of `placeholder` in a header or footer.
pub fn replace_in_header_footer(hf: &mut CT_HdrFtr, placeholder: &str, replacement: &str) -> usize {
    replace_in_paragraphs(&mut hf.paragraphs, placeholder, replacement)
}

/// Replace placeholders in text boxes and shapes within a raw XML part.
///
/// Walks the XML, finds `w:txbxContent` elements at any depth, parses their
/// child `w:p` elements using `CT_P::from_xml`, performs replacement, and
/// re-serializes back. Returns the modified XML and replacement count.
pub fn replace_in_xml_part(
    xml: &[u8],
    placeholder: &str,
    replacement: &str,
) -> crate::error::Result<(Vec<u8>, usize)> {
    replace_many_in_xml_part(xml, &[(placeholder, replacement)])
}

/// Apply several placeholder replacements to a raw XML part in one pass.
///
/// Equivalent to calling [`replace_in_xml_part`] once per pair, but parses and
/// re-serialises the part a single time — which matters when filling a template
/// with many fields.
pub fn replace_many_in_xml_part(
    xml: &[u8],
    replacements: &[(&str, &str)],
) -> crate::error::Result<(Vec<u8>, usize)> {
    rewrite_text_boxes(xml, &mut |paragraphs| {
        replacements
            .iter()
            .map(|(placeholder, replacement)| {
                replace_in_paragraphs(paragraphs, placeholder, replacement)
            })
            .sum()
    })
}

/// Apply a regex replacement to text boxes and shapes within a raw XML part.
///
/// The regex counterpart to [`replace_in_xml_part`]; `replacement` supports
/// `$1`-style capture group references.
pub fn replace_regex_in_xml_part(
    xml: &[u8],
    re: &regex::Regex,
    replacement: &str,
) -> crate::error::Result<(Vec<u8>, usize)> {
    rewrite_text_boxes(xml, &mut |paragraphs| {
        replace_regex_in_paragraphs(paragraphs, re, replacement)
    })
}

/// Walk `xml`, handing the paragraphs of each `w:txbxContent` element to
/// `edit`, and re-serialise. Returns the rewritten XML and the summed count.
fn rewrite_text_boxes(
    xml: &[u8],
    edit: &mut dyn FnMut(&mut Vec<CT_P>) -> usize,
) -> crate::error::Result<(Vec<u8>, usize)> {
    use crate::namespace::matches_local_name;
    use quick_xml::events::Event;
    use quick_xml::{Reader, Writer};

    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Vec::new());
    let mut buf = Vec::new();
    let mut total_count = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e)) if matches_local_name(e.name().as_ref(), b"txbxContent") => {
                // We found a txbxContent element. Collect its contents as raw XML,
                // parse paragraphs, do replacement, and re-serialize.
                writer.write_event(Event::Start(e.clone()))?;

                // Read all events inside txbxContent
                let mut depth = 1u32;
                let mut inner_buf = Vec::new();

                // Collect paragraphs from inside txbxContent
                let mut paragraphs: Vec<CT_P> = Vec::new();

                loop {
                    match reader.read_event_into(&mut inner_buf) {
                        Ok(Event::Start(ref ie)) => {
                            if matches_local_name(ie.name().as_ref(), b"p") && depth == 1 {
                                // Parse this paragraph: collect its XML, then parse via CT_P
                                let mut para_writer = Writer::new(Vec::new());
                                // Write the opening <w:p> tag
                                para_writer.write_event(Event::Start(ie.clone()))?;
                                let mut pdepth = 1u32;
                                let mut pbuf = Vec::new();
                                loop {
                                    match reader.read_event_into(&mut pbuf) {
                                        Ok(Event::Start(ref pe)) => {
                                            pdepth += 1;
                                            para_writer.write_event(Event::Start(pe.clone()))?;
                                        }
                                        Ok(Event::End(ref pe)) => {
                                            pdepth -= 1;
                                            para_writer.write_event(Event::End(pe.clone()))?;
                                            if pdepth == 0 {
                                                break;
                                            }
                                        }
                                        Ok(ref ev) => {
                                            para_writer.write_event(ev.clone())?;
                                        }
                                        Err(e) => return Err(e.into()),
                                    }
                                    pbuf.clear();
                                }
                                let para_xml = para_writer.into_inner();

                                // Parse the paragraph
                                let mut para_reader = Reader::from_reader(para_xml.as_slice());
                                para_reader.config_mut().trim_text(true);
                                let mut prbuf = Vec::new();
                                // Advance past the <w:p> start tag
                                loop {
                                    match para_reader.read_event_into(&mut prbuf) {
                                        Ok(Event::Start(ref pe))
                                            if matches_local_name(pe.name().as_ref(), b"p") =>
                                        {
                                            break;
                                        }
                                        Ok(Event::Eof) => break,
                                        _ => {}
                                    }
                                    prbuf.clear();
                                }
                                let para = CT_P::from_xml(&mut para_reader)?;
                                paragraphs.push(para);
                            } else {
                                depth += 1;
                                // Non-paragraph element inside txbxContent; skip it
                                reader.read_to_end_into(ie.name(), &mut Vec::new())?;
                                depth -= 1;
                            }
                        }
                        Ok(Event::End(ref ie)) => {
                            if matches_local_name(ie.name().as_ref(), b"txbxContent") && depth == 1
                            {
                                break;
                            }
                            depth -= 1;
                        }
                        Ok(_) => {
                            // Whitespace/text at top level of txbxContent, skip
                        }
                        Err(e) => return Err(e.into()),
                    }
                    inner_buf.clear();
                }

                // Hand the collected paragraphs to the caller's edit function
                total_count += edit(&mut paragraphs);

                // Re-serialize paragraphs into the writer
                for p in &paragraphs {
                    p.to_xml(&mut writer)?;
                }

                // Write closing txbxContent tag
                writer.write_event(Event::End(quick_xml::events::BytesEnd::new(
                    "w:txbxContent",
                )))?;
            }
            Ok(ev) => {
                writer.write_event(ev)?;
            }
            Err(e) => return Err(e.into()),
        }
        buf.clear();
    }

    Ok((writer.into_inner(), total_count))
}

/// Replace placeholders in chart XML parts.
///
/// Chart text uses DrawingML runs: `a:r` → `a:t` (not `w:r`/`w:t`).
/// Also replaces in string cache values (`c:v`).
/// Returns modified XML and replacement count.
pub fn replace_in_chart_xml(
    xml: &[u8],
    placeholder: &str,
    replacement: &str,
) -> crate::error::Result<(Vec<u8>, usize)> {
    replace_many_in_chart_xml(xml, &[(placeholder, replacement)])
}

/// Apply several placeholder replacements to chart XML in one pass.
pub fn replace_many_in_chart_xml(
    xml: &[u8],
    replacements: &[(&str, &str)],
) -> crate::error::Result<(Vec<u8>, usize)> {
    use crate::namespace::matches_local_name;
    use quick_xml::events::{BytesEnd, BytesText, Event};
    use quick_xml::{Reader, Writer};

    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Vec::new());
    let mut buf = Vec::new();
    let mut total_count = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            // <a:t> holds DrawingML run text, <c:v> a chart value cache entry.
            Ok(Event::Start(ref e))
                if matches_local_name(e.name().as_ref(), b"t")
                    || matches_local_name(e.name().as_ref(), b"v") =>
            {
                let name = e.name();
                writer.write_event(Event::Start(e.borrow()))?;

                // Consume the element as a whole. Reading it event by event
                // would split the value at every entity reference and could
                // miss a placeholder straddling one.
                let mut text = crate::xml_text::read_element_text(&mut reader, name)?;
                for (placeholder, replacement) in replacements {
                    let occurrences = text.matches(placeholder).count();
                    if occurrences > 0 {
                        total_count += occurrences;
                        text = text.replace(placeholder, replacement);
                    }
                }
                writer.write_event(Event::Text(BytesText::new(&text)))?;

                writer.write_event(Event::End(BytesEnd::new(
                    std::str::from_utf8(name.as_ref()).unwrap_or_default(),
                )))?;
            }
            Ok(ev) => {
                writer.write_event(ev)?;
            }
            Err(e) => return Err(e.into()),
        }
        buf.clear();
    }

    Ok((writer.into_inner(), total_count))
}

/// Replace all regex matches in a paragraph with the replacement string.
///
/// The `replacement` string supports capture group references: `$1`, `$2`, etc.
/// Uses the same cross-run char map algorithm as literal replacement.
/// Returns the number of replacements made.
pub fn replace_regex_in_paragraph(para: &mut CT_P, re: &regex::Regex, replacement: &str) -> usize {
    replace_regex_in_paragraph_children(&mut para.content, re, replacement)
}

fn replace_regex_in_paragraph_children(
    children: &mut Vec<ParagraphChild>,
    re: &regex::Regex,
    replacement: &str,
) -> usize {
    let mut total = 0;
    let mut index = 0;
    while index < children.len() {
        if matches!(children[index], ParagraphChild::Run(_)) {
            let end = children[index..]
                .iter()
                .position(|child| !matches!(child, ParagraphChild::Run(_)))
                .map_or(children.len(), |offset| index + offset);
            let mut runs = children
                .drain(index..end)
                .map(|child| match child {
                    ParagraphChild::Run(run) => run,
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>();
            total += replace_regex_in_runs(&mut runs, re, replacement);
            let inserted = runs.len();
            children.splice(index..index, runs.into_iter().map(ParagraphChild::Run));
            index += inserted;
            continue;
        }
        match &mut children[index] {
            ParagraphChild::Hyperlink(hyperlink) => {
                if re.is_match(&hyperlink.text()) {
                    total +=
                        replace_regex_in_inline_children(hyperlink.children_mut(), re, replacement);
                }
            }
            ParagraphChild::SimpleField(field) => {
                if re.is_match(&field.text()) {
                    total +=
                        replace_regex_in_inline_children(field.children_mut(), re, replacement);
                }
            }
            ParagraphChild::Insertion(_)
            | ParagraphChild::Unsupported(_)
            | ParagraphChild::Run(_) => {}
        }
        index += 1;
    }
    total
}

fn replace_regex_in_inline_children(
    children: &mut Vec<InlineChild>,
    re: &regex::Regex,
    replacement: &str,
) -> usize {
    let mut total = 0;
    let mut index = 0;
    while index < children.len() {
        if matches!(children[index], InlineChild::Run(_)) {
            let end = children[index..]
                .iter()
                .position(|child| !matches!(child, InlineChild::Run(_)))
                .map_or(children.len(), |offset| index + offset);
            let mut runs = children
                .drain(index..end)
                .map(|child| match child {
                    InlineChild::Run(run) => run,
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>();
            total += replace_regex_in_runs(&mut runs, re, replacement);
            let inserted = runs.len();
            children.splice(index..index, runs.into_iter().map(InlineChild::Run));
            index += inserted;
            continue;
        }
        if let InlineChild::SimpleField(field) = &mut children[index]
            && re.is_match(&field.text())
        {
            total += replace_regex_in_inline_children(field.children_mut(), re, replacement);
        }
        index += 1;
    }
    total
}

fn replace_regex_in_runs(runs: &mut Vec<CT_R>, re: &regex::Regex, replacement: &str) -> usize {
    replace_matches_in_runs(runs, |text| {
        re.captures_iter(text)
            .filter_map(|captures| {
                let matched = captures.get(0)?;
                if matched.is_empty() {
                    return None;
                }
                let mut expanded = String::new();
                captures.expand(replacement, &mut expanded);
                Some(TextReplacement {
                    start: matched.start(),
                    end: matched.end(),
                    replacement: expanded,
                })
            })
            .collect()
    })
}

/// Replace regex matches in all paragraphs.
pub fn replace_regex_in_paragraphs(
    paras: &mut [CT_P],
    re: &regex::Regex,
    replacement: &str,
) -> usize {
    paras
        .iter_mut()
        .map(|p| replace_regex_in_paragraph(p, re, replacement))
        .sum()
}

/// Replace regex matches in a table (recursively handles nested tables).
pub fn replace_regex_in_table(table: &mut CT_Tbl, re: &regex::Regex, replacement: &str) -> usize {
    use crate::table::CellContent;

    let mut count = 0;
    for row in &mut table.rows {
        for cell in &mut row.cells {
            for content in &mut cell.content {
                match content {
                    CellContent::Paragraph(p) => {
                        count += replace_regex_in_paragraph(p, re, replacement);
                    }
                    CellContent::Table(nested) => {
                        count += replace_regex_in_table(nested, re, replacement);
                    }
                    CellContent::Unsupported(_) => {}
                }
            }
        }
    }
    count
}

/// Replace regex matches in a header or footer.
pub fn replace_regex_in_header_footer(
    hf: &mut CT_HdrFtr,
    re: &regex::Regex,
    replacement: &str,
) -> usize {
    replace_regex_in_paragraphs(&mut hf.paragraphs, re, replacement)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::properties::CT_RPr;
    use crate::text::CT_Text;

    fn make_para(texts: &[&str]) -> CT_P {
        let mut p = CT_P::new();
        for text in texts {
            p.add_run(text);
        }
        p
    }

    #[test]
    fn replace_single_run() {
        let mut p = make_para(&["Hello {{name}}, welcome!"]);
        let count = replace_in_paragraph(&mut p, "{{name}}", "Alice");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "Hello Alice, welcome!");
    }

    #[test]
    fn replace_cross_two_runs() {
        let mut p = make_para(&["Hello {{", "name}}, welcome!"]);
        let count = replace_in_paragraph(&mut p, "{{name}}", "Bob");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "Hello Bob, welcome!");
    }

    #[test]
    fn replace_cross_three_runs() {
        let mut p = make_para(&["{{", "na", "me}}"]);
        let count = replace_in_paragraph(&mut p, "{{name}}", "Charlie");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "Charlie");
    }

    #[test]
    fn replacement_spans_adjacent_text_nodes_in_one_run() {
        let mut paragraph = CT_P::new();
        let run = paragraph.add_run("");
        run.content = vec![
            RunContent::Text(CT_Text::new("before {{")),
            RunContent::Text(CT_Text::new("name}} after")),
        ];

        assert_eq!(replace_in_paragraph(&mut paragraph, "{{name}}", "Alice"), 1);
        assert_eq!(paragraph.text(), "before Alice after");
        let run = paragraph.run(0).unwrap();
        assert_eq!(run.content.len(), 2);
    }

    #[test]
    fn replacement_does_not_cross_a_run_control() {
        let mut paragraph = CT_P::new();
        let run = paragraph.add_run("");
        run.content = vec![
            RunContent::Text(CT_Text::new("{{")),
            RunContent::Tab,
            RunContent::Text(CT_Text::new("name}}")),
        ];

        assert_eq!(replace_in_paragraph(&mut paragraph, "{{name}}", "Alice"), 0);
        assert_eq!(paragraph.text(), "{{\tname}}");
    }

    #[test]
    fn regex_replacement_spans_text_nodes_but_not_controls() {
        let mut paragraph = CT_P::new();
        let first = paragraph.add_run("");
        first.content = vec![
            RunContent::Text(CT_Text::new("ID-")),
            RunContent::Text(CT_Text::new("123")),
        ];
        let second = paragraph.add_run("");
        second.content = vec![
            RunContent::Text(CT_Text::new(" ID-")),
            RunContent::Tab,
            RunContent::Text(CT_Text::new("456")),
        ];
        let regex = regex::Regex::new(r"ID-(\d+)").unwrap();

        assert_eq!(
            replace_regex_in_paragraph(&mut paragraph, &regex, "[$1]"),
            1
        );
        assert_eq!(paragraph.text(), "[123] ID-\t456");
    }

    #[test]
    fn replacement_does_not_cross_hyperlink_containment_boundary() {
        let mut paragraph = CT_P::new();
        paragraph.add_run("{{");
        let mut hyperlink = crate::text::CT_Hyperlink::new(Some("rId1".to_string()), None);
        hyperlink.add_run("name}}");
        paragraph.add_hyperlink(hyperlink);

        assert_eq!(replace_in_paragraph(&mut paragraph, "{{name}}", "Alice"), 0);
        assert_eq!(paragraph.text(), "{{name}}");
        assert!(matches!(
            paragraph.content.as_slice(),
            [ParagraphChild::Run(_), ParagraphChild::Hyperlink(_)]
        ));
    }

    #[test]
    fn replacement_across_runs_inside_hyperlink_preserves_hyperlink_node() {
        let mut paragraph = CT_P::new();
        let mut hyperlink = crate::text::CT_Hyperlink::new(Some("rId1".to_string()), None);
        hyperlink.add_run("{{na");
        hyperlink.add_run("me}}");
        paragraph.add_hyperlink(hyperlink);

        assert_eq!(replace_in_paragraph(&mut paragraph, "{{name}}", "Alice"), 1);
        let ParagraphChild::Hyperlink(hyperlink) = &paragraph.content[0] else {
            panic!("expected hyperlink")
        };
        assert_eq!(hyperlink.rel_id(), Some("rId1"));
        assert_eq!(hyperlink.text(), "Alice");
    }

    #[test]
    fn replace_preserves_formatting() {
        let mut p = CT_P::new();
        // Run 0: bold "Hello "
        let mut r0 = CT_R::new("Hello ");
        r0.properties = Some(CT_RPr {
            bold: Some(true),
            ..Default::default()
        });
        p.content.push(ParagraphChild::Run(r0));

        // Run 1: italic "{{name}}"
        let mut r1 = CT_R::new("{{name}}");
        r1.properties = Some(CT_RPr {
            italic: Some(true),
            ..Default::default()
        });
        p.content.push(ParagraphChild::Run(r1));

        // Run 2: "!"
        p.add_run("!");

        let count = replace_in_paragraph(&mut p, "{{name}}", "Alice");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "Hello Alice!");

        // Run 0 should still be bold
        assert_eq!(
            p.run(0).unwrap().properties.as_ref().unwrap().bold,
            Some(true)
        );
        // Run 1 (now "Alice") should still be italic
        assert_eq!(
            p.run(1).unwrap().properties.as_ref().unwrap().italic,
            Some(true)
        );
    }

    #[test]
    fn replace_multiple_occurrences() {
        let mut p = make_para(&["{{x}} and {{x}}"]);
        let count = replace_in_paragraph(&mut p, "{{x}}", "Y");
        assert_eq!(count, 2);
        assert_eq!(p.text(), "Y and Y");
    }

    #[test]
    fn replace_no_match() {
        let mut p = make_para(&["Hello World"]);
        let count = replace_in_paragraph(&mut p, "{{missing}}", "X");
        assert_eq!(count, 0);
        assert_eq!(p.text(), "Hello World");
    }

    #[test]
    fn replace_in_table_recursive() {
        use crate::table::{CT_Row, CT_Tc, CellContent};

        let mut table = CT_Tbl::new();
        let mut row = CT_Row::new();
        let mut cell = CT_Tc::new();

        // Add a paragraph with placeholder
        let mut p = CT_P::new();
        p.add_run("Value: {{val}}");
        cell.content = vec![CellContent::Paragraph(p)];

        // Add a nested table with placeholder
        let mut nested = CT_Tbl::new();
        let mut nrow = CT_Row::new();
        let mut ncell = CT_Tc::new();
        let mut np = CT_P::new();
        np.add_run("Nested: {{val}}");
        ncell.content = vec![CellContent::Paragraph(np)];
        nrow.cells.push(ncell);
        nested.rows.push(nrow);
        cell.content.push(CellContent::Table(nested));

        row.cells.push(cell);
        table.rows.push(row);

        let count = replace_in_table(&mut table, "{{val}}", "42");
        assert_eq!(count, 2);

        // Verify
        let para = match &table.rows[0].cells[0].content[0] {
            CellContent::Paragraph(p) => p,
            _ => panic!("expected paragraph"),
        };
        assert_eq!(para.text(), "Value: 42");
    }

    #[test]
    fn replace_in_header_footer_test() {
        let mut hf = CT_HdrFtr::new();
        let mut p = CT_P::new();
        p.add_run("Company: {{company}}");
        hf.paragraphs.push(p);

        let count = replace_in_header_footer(&mut hf, "{{company}}", "Acme Corp");
        assert_eq!(count, 1);
        assert_eq!(hf.text(), "Company: Acme Corp");
    }

    #[test]
    fn replace_empty_placeholder_noop() {
        let mut p = make_para(&["Hello"]);
        let count = replace_in_paragraph(&mut p, "", "X");
        assert_eq!(count, 0);
    }

    #[test]
    fn replace_in_textbox_xml() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"
            xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body>
<w:p><w:r><mc:AlternateContent><mc:Choice>
<w:drawing><wp:anchor xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing">
<a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
<a:graphicData><wps:wsp><wps:txbx>
<w:txbxContent>
<w:p><w:r><w:t>Hello {{name}}</w:t></w:r></w:p>
</w:txbxContent>
</wps:txbx></wps:wsp></a:graphicData></a:graphic>
</wp:anchor></w:drawing>
</mc:Choice></mc:AlternateContent></w:r></w:p>
</w:body>
</w:document>"#;

        let (result, count) = replace_in_xml_part(xml, "{{name}}", "Alice").unwrap();
        assert_eq!(count, 1);
        let result_str = String::from_utf8(result).unwrap();
        assert!(result_str.contains("Hello Alice"));
        assert!(!result_str.contains("{{name}}"));
    }

    #[test]
    fn replace_in_vml_textbox() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:v="urn:schemas-microsoft-com:vml">
<w:body>
<w:p><w:r><w:pict><v:shape>
<v:textbox>
<w:txbxContent>
<w:p><w:r><w:t>Company: {{company}}</w:t></w:r></w:p>
</w:txbxContent>
</v:textbox>
</v:shape></w:pict></w:r></w:p>
</w:body>
</w:document>"#;

        let (result, count) = replace_in_xml_part(xml, "{{company}}", "Acme").unwrap();
        assert_eq!(count, 1);
        let result_str = String::from_utf8(result).unwrap();
        assert!(result_str.contains("Company: Acme"));
    }

    #[test]
    fn replace_in_xml_part_no_textbox() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p><w:r><w:t>Hello {{name}}</w:t></w:r></w:p></w:body>
</w:document>"#;

        let (_, count) = replace_in_xml_part(xml, "{{name}}", "Alice").unwrap();
        // No text boxes, so no replacements from the raw XML pass
        assert_eq!(count, 0);
    }

    #[test]
    fn replace_in_chart_title() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"
              xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
<c:chart>
<c:title><c:tx><c:rich>
<a:p><a:r><a:t>{{chart_title}}</a:t></a:r></a:p>
</c:rich></c:tx></c:title>
</c:chart>
</c:chartSpace>"#;

        let (result, count) = replace_in_chart_xml(xml, "{{chart_title}}", "Sales Report").unwrap();
        assert_eq!(count, 1);
        let result_str = String::from_utf8(result).unwrap();
        assert!(result_str.contains("Sales Report"));
        assert!(!result_str.contains("{{chart_title}}"));
    }

    #[test]
    fn replace_in_chart_axis_and_cache() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"
              xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
<c:chart>
<c:plotArea>
<c:catAx><c:title><c:tx><c:rich>
<a:p><a:r><a:t>{{axis}}</a:t></a:r></a:p>
</c:rich></c:tx></c:title></c:catAx>
<c:barChart><c:ser><c:cat><c:strRef><c:strCache>
<c:pt idx="0"><c:v>{{label}}</c:v></c:pt>
</c:strCache></c:strRef></c:cat></c:ser></c:barChart>
</c:plotArea>
</c:chart>
</c:chartSpace>"#;

        let (result, count) = replace_in_chart_xml(xml, "{{axis}}", "Quarter").unwrap();
        assert_eq!(count, 1);
        let result_str = String::from_utf8(result).unwrap();
        assert!(result_str.contains("Quarter"));

        let (result2, count2) =
            replace_in_chart_xml(result_str.as_bytes(), "{{label}}", "Q1").unwrap();
        assert_eq!(count2, 1);
        let result2_str = String::from_utf8(result2).unwrap();
        assert!(result2_str.contains("Q1"));
    }

    #[test]
    fn replace_utf8_multibyte_placeholder() {
        // Test that multi-byte UTF-8 placeholders work correctly.
        // This verifies the byte-offset to char-index conversion in build_char_map.
        let mut p = make_para(&["こんにちは{{名前}}さん"]);
        let count = replace_in_paragraph(&mut p, "{{名前}}", "太郎");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "こんにちは太郎さん");
    }

    #[test]
    fn replace_utf8_cross_run() {
        // Multi-byte placeholder split across runs
        let mut p = make_para(&["こんにちは{{", "名前}}さん"]);
        let count = replace_in_paragraph(&mut p, "{{名前}}", "花子");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "こんにちは花子さん");
    }

    #[test]
    fn replace_utf8_multiple_occurrences() {
        let mut p = make_para(&["{{名前}}と{{名前}}"]);
        let count = replace_in_paragraph(&mut p, "{{名前}}", "太郎");
        assert_eq!(count, 2);
        assert_eq!(p.text(), "太郎と太郎");
    }

    #[test]
    fn replace_emoji_placeholder() {
        // Emojis are 4-byte UTF-8, good stress test
        let mut p = make_para(&["Hello {{🎉name🎉}}, welcome!"]);
        let count = replace_in_paragraph(&mut p, "{{🎉name🎉}}", "World");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "Hello World, welcome!");
    }

    // ---- Regex replacement tests ----

    #[test]
    fn regex_replace_simple() {
        let re = regex::Regex::new(r"\{\{name\}\}").unwrap();
        let mut p = make_para(&["Hello {{name}}, welcome!"]);
        let count = replace_regex_in_paragraph(&mut p, &re, "Alice");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "Hello Alice, welcome!");
    }

    #[test]
    fn regex_replace_with_capture_groups() {
        let re = regex::Regex::new(r"(\d+)-(\d+)-(\d+)").unwrap();
        let mut p = make_para(&["Date: 2024-01-15"]);
        let count = replace_regex_in_paragraph(&mut p, &re, "$3/$2/$1");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "Date: 15/01/2024");
    }

    #[test]
    fn regex_replace_multiple_matches() {
        let re = regex::Regex::new(r"\b[A-Z]\w+").unwrap();
        let mut p = make_para(&["Hello World Today"]);
        let count = replace_regex_in_paragraph(&mut p, &re, "X");
        assert_eq!(count, 3);
        assert_eq!(p.text(), "X X X");
    }

    #[test]
    fn regex_replace_cross_run() {
        let re = regex::Regex::new(r"\{\{name\}\}").unwrap();
        let mut p = make_para(&["Hello {{", "name}}, welcome!"]);
        let count = replace_regex_in_paragraph(&mut p, &re, "Bob");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "Hello Bob, welcome!");
    }

    #[test]
    fn regex_replace_no_match() {
        let re = regex::Regex::new(r"xyz\d+").unwrap();
        let mut p = make_para(&["Hello World"]);
        let count = replace_regex_in_paragraph(&mut p, &re, "X");
        assert_eq!(count, 0);
        assert_eq!(p.text(), "Hello World");
    }

    #[test]
    fn regex_replace_in_table() {
        use crate::table::{CT_Row, CT_Tc, CellContent};

        let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
        let mut table = CT_Tbl::new();
        let mut row = CT_Row::new();
        let mut cell = CT_Tc::new();
        let mut p = CT_P::new();
        p.add_run("Value: {{item}}");
        cell.content = vec![CellContent::Paragraph(p)];
        row.cells.push(cell);
        table.rows.push(row);

        let count = replace_regex_in_table(&mut table, &re, "[$1]");
        assert_eq!(count, 1);

        let para = match &table.rows[0].cells[0].content[0] {
            CellContent::Paragraph(p) => p,
            _ => panic!("expected paragraph"),
        };
        assert_eq!(para.text(), "Value: [item]");
    }
}
