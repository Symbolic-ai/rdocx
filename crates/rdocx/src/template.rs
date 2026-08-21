//! Structured template parsing and JSON evaluation.

use std::collections::BTreeMap;

use quick_xml::Reader;
use quick_xml::events::Event;
use rdocx_oxml::content_control::{CT_Sdt, SdtContent};
use rdocx_oxml::document::{BodyContent, CT_Document};
use rdocx_oxml::header_footer::CT_HdrFtr;
use rdocx_oxml::namespace::matches_local_name;
use rdocx_oxml::placeholder;
use rdocx_oxml::table::{CT_Row, CT_Tbl, CT_Tc, CellContent};
use rdocx_oxml::text::{CT_P, RunContent};
use serde_json::Value;

use crate::document::Document;
use crate::error::{Error, Result};

#[derive(Debug)]
struct Replacement {
    literal: String,
    value: String,
    occurrences: usize,
}

#[derive(Debug)]
struct PendingReplacement {
    sentinel: String,
    value: String,
    occurrences: usize,
}

#[derive(Debug)]
struct SentinelPool {
    next: usize,
    forbidden: Vec<String>,
    pending: Vec<PendingReplacement>,
}

impl SentinelPool {
    fn new(sources: &[String]) -> Self {
        Self {
            next: 0,
            forbidden: sources.to_vec(),
            pending: Vec::new(),
        }
    }

    fn stage(&mut self, value: &str, occurrences: usize) -> String {
        loop {
            let sentinel = format!("\u{e000}rdocx-template-{}\u{e001}", self.next);
            self.next += 1;
            let collides = self
                .forbidden
                .iter()
                .any(|source| source.contains(&sentinel))
                || self
                    .pending
                    .iter()
                    .any(|replacement| replacement.value.contains(&sentinel))
                || value.contains(&sentinel);
            if !collides {
                self.forbidden.push(sentinel.clone());
                self.pending.push(PendingReplacement {
                    sentinel: sentinel.clone(),
                    value: value.to_owned(),
                    occurrences,
                });
                return sentinel;
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Scope {
    name: String,
    value: Value,
    deferred: bool,
}

enum ResolvedValue<'a> {
    Value(&'a Value),
    Deferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EndKind {
    For,
    If,
}

impl EndKind {
    fn name(self) -> &'static str {
        match self {
            Self::For => "endfor",
            Self::If => "endif",
        }
    }
}

#[derive(Debug)]
enum Control {
    For { name: String, path: Vec<String> },
    If { path: Vec<String> },
    End(EndKind),
}

#[derive(Debug)]
enum Block<T> {
    Item {
        source_index: usize,
        value: T,
    },
    For {
        name: String,
        path: Vec<String>,
        body: Vec<Block<T>>,
    },
    If {
        path: Vec<String>,
        body: Vec<Block<T>>,
    },
}

#[derive(Debug)]
struct Evaluated<T> {
    source_index: usize,
    value: T,
}

#[derive(Debug, Clone)]
struct TableRowItem {
    row: CT_Row,
    controls: Vec<(usize, CT_Sdt)>,
}

pub(crate) fn render(document: &mut Document, data: &Value) -> Result<usize> {
    let original_sources = document.clone_for_template().template_sources()?;
    let mut sentinels = SentinelPool::new(&original_sources);
    let mut candidate = document.clone_for_template();
    validate_repeated_numbering(&candidate)?;
    let (body_count, structural_change) = render_body(&mut candidate, data, &mut sentinels)?;

    let remaining_sources = candidate.template_sources()?;
    let external_replacements = resolve_replacements(&remaining_sources, data, &[])?;
    let external_count = external_replacements
        .iter()
        .map(|replacement| replacement.occurrences)
        .sum::<usize>();
    let mut first_pass = Vec::with_capacity(external_replacements.len());
    for replacement in &external_replacements {
        let sentinel = sentinels.stage(&replacement.value, replacement.occurrences);
        first_pass.push((replacement.literal.as_str(), sentinel));
    }
    let pairs = first_pass
        .iter()
        .map(|(literal, sentinel)| (*literal, sentinel.as_str()))
        .collect::<Vec<_>>();
    let replaced = candidate.apply_template_pairs(&pairs);
    if replaced != external_count {
        return Err(Error::Other(format!(
            "template changed while staging: discovered {external_count} tags but replaced {replaced}"
        )));
    }

    let expected = body_count + external_count;
    let restoration = sentinels
        .pending
        .iter()
        .map(|replacement| (replacement.sentinel.as_str(), replacement.value.as_str()))
        .collect::<Vec<_>>();
    let restored = candidate.apply_template_pairs(&restoration);
    let expected_restored = sentinels
        .pending
        .iter()
        .map(|replacement| replacement.occurrences)
        .sum::<usize>();
    if restored != expected_restored || expected != expected_restored {
        return Err(Error::Other(format!(
            "template staging lost replacements: expected {expected_restored} sentinels but found {restored}"
        )));
    }

    if !structural_change && expected == 0 {
        return Ok(0);
    }
    candidate.document.to_xml()?;
    document.commit_template(candidate);
    Ok(expected)
}

fn validate_repeated_numbering(document: &Document) -> Result<()> {
    let blocks = parse_blocks(&document.document.body.content, body_marker)?;
    validate_body_numbering_blocks(&blocks, document, false)
}

fn validate_body_numbering_blocks(
    blocks: &[Block<BodyContent>],
    document: &Document,
    repeated: bool,
) -> Result<()> {
    for block in blocks {
        match block {
            Block::Item { value, .. } => {
                validate_body_content_numbering(value, document, repeated)?;
            }
            Block::For { body, .. } => {
                validate_body_numbering_blocks(body, document, true)?;
            }
            Block::If { body, .. } => {
                validate_body_numbering_blocks(body, document, repeated)?;
            }
        }
    }
    Ok(())
}

fn validate_body_content_numbering(
    content: &BodyContent,
    document: &Document,
    repeated: bool,
) -> Result<()> {
    match content {
        BodyContent::Paragraph(paragraph) => {
            validate_paragraph_numbering(paragraph, document, repeated)
        }
        BodyContent::Table(table) => validate_table_numbering(table, document, repeated),
        BodyContent::ContentControl(control) => {
            validate_control_numbering(control, document, repeated)
        }
        BodyContent::RawXml(_) => Ok(()),
    }
}

fn validate_table_numbering(table: &CT_Tbl, document: &Document, repeated: bool) -> Result<()> {
    let blocks = parse_blocks(&table.rows, row_marker)?;
    validate_row_numbering_blocks(&blocks, &table.content_controls, document, repeated)?;
    for (_, _, control) in table
        .content_controls
        .iter()
        .filter(|(at, _, _)| *at == table.rows.len())
    {
        validate_control_numbering(control, document, repeated)?;
    }
    Ok(())
}

fn validate_row_numbering_blocks(
    blocks: &[Block<CT_Row>],
    controls: &[(usize, usize, CT_Sdt)],
    document: &Document,
    repeated: bool,
) -> Result<()> {
    for block in blocks {
        match block {
            Block::Item {
                source_index,
                value,
            } => {
                validate_row_numbering(value, document, repeated)?;
                for (_, _, control) in controls.iter().filter(|(at, _, _)| at == source_index) {
                    validate_control_numbering(control, document, repeated)?;
                }
            }
            Block::For { body, .. } => {
                validate_row_numbering_blocks(body, controls, document, true)?;
            }
            Block::If { body, .. } => {
                validate_row_numbering_blocks(body, controls, document, repeated)?;
            }
        }
    }
    Ok(())
}

fn validate_row_numbering(row: &CT_Row, document: &Document, repeated: bool) -> Result<()> {
    for cell in &row.cells {
        validate_cell_numbering(cell, document, repeated)?;
    }
    for (_, _, control) in &row.content_controls {
        validate_control_numbering(control, document, repeated)?;
    }
    Ok(())
}

fn validate_cell_numbering(cell: &CT_Tc, document: &Document, repeated: bool) -> Result<()> {
    for content in &cell.content {
        match content {
            CellContent::Paragraph(paragraph) => {
                validate_paragraph_numbering(paragraph, document, repeated)?;
            }
            CellContent::Table(table) => {
                validate_table_numbering(table, document, repeated)?;
            }
            CellContent::ContentControl(control) => {
                validate_control_numbering(control, document, repeated)?;
            }
        }
    }
    Ok(())
}

fn validate_control_numbering(control: &CT_Sdt, document: &Document, repeated: bool) -> Result<()> {
    for content in &control.content {
        match content {
            SdtContent::Paragraph(paragraph) => {
                validate_paragraph_numbering(paragraph, document, repeated)?;
            }
            SdtContent::Table(table) => {
                validate_table_numbering(table, document, repeated)?;
            }
            SdtContent::Row(row) => validate_row_numbering(row, document, repeated)?,
            SdtContent::Cell(cell) => validate_cell_numbering(cell, document, repeated)?,
            SdtContent::ContentControl(nested) => {
                validate_control_numbering(nested, document, repeated)?;
            }
            SdtContent::Run(_) | SdtContent::RawXml(_) => {}
        }
    }
    Ok(())
}

fn validate_paragraph_numbering(
    paragraph: &CT_P,
    document: &Document,
    repeated: bool,
) -> Result<()> {
    if !repeated {
        return Ok(());
    }
    let Some(properties) = &paragraph.properties else {
        return Ok(());
    };
    let Some(num_id) = properties.num_id else {
        if properties.num_ilvl.is_some() {
            return Err(template_error(
                "a repeated paragraph numbering level has no numId",
            ));
        }
        return Ok(());
    };
    let level = properties.num_ilvl.unwrap_or(0);
    if !document.template_numbering_reference_exists(num_id, level) {
        return Err(template_error(&format!(
            "a repeated paragraph references missing numbering numId {num_id} level {level}"
        )));
    }
    Ok(())
}

fn render_body(
    document: &mut Document,
    data: &Value,
    sentinels: &mut SentinelPool,
) -> Result<(usize, bool)> {
    let blocks = parse_blocks(&document.document.body.content, body_marker)?;
    let mut structural_change = blocks_have_controls(&blocks);
    let mut scopes = Vec::new();
    let mut render_item = |content: &mut BodyContent,
                           root: &Value,
                           scopes: &[Scope],
                           sentinels: &mut SentinelPool| {
        render_body_item(content, root, scopes, sentinels, &mut structural_change)
    };
    let (evaluated, count) =
        evaluate_blocks(&blocks, data, &mut scopes, sentinels, &mut render_item)?;
    document.document.body.content = evaluated
        .into_iter()
        .map(|evaluated| evaluated.value)
        .collect();
    Ok((count, structural_change))
}

fn render_body_item(
    content: &mut BodyContent,
    root: &Value,
    scopes: &[Scope],
    sentinels: &mut SentinelPool,
    structural_change: &mut bool,
) -> Result<usize> {
    match content {
        BodyContent::Paragraph(paragraph) => {
            stage_paragraph_scalars(paragraph, root, scopes, sentinels)
        }
        BodyContent::Table(table) => {
            render_table(table, root, scopes, sentinels, structural_change)
        }
        BodyContent::ContentControl(control) => {
            stage_control_scalars(control, root, scopes, sentinels)
        }
        BodyContent::RawXml(_) => Ok(0),
    }
}

fn render_table(
    table: &mut CT_Tbl,
    root: &Value,
    scopes: &[Scope],
    sentinels: &mut SentinelPool,
    structural_change: &mut bool,
) -> Result<usize> {
    let old_len = table.rows.len();
    let old_controls = std::mem::take(&mut table.content_controls);
    let mut controls_by_row = vec![Vec::new(); old_len];
    let mut trailing_controls = Vec::new();
    for (at, before, control) in old_controls {
        if at < old_len {
            controls_by_row[at].push((before, control));
        } else if at == old_len {
            trailing_controls.push((before, control));
        }
    }
    let items = std::mem::take(&mut table.rows)
        .into_iter()
        .zip(controls_by_row)
        .map(|(row, controls)| TableRowItem { row, controls })
        .collect::<Vec<_>>();
    for (index, item) in items.iter().enumerate() {
        if row_marker(&item.row)?.is_some()
            && (table.extra_xml.iter().any(|(at, _)| *at == index) || !item.controls.is_empty())
        {
            return Err(template_error(
                "a marker row cannot carry table-level preserved siblings",
            ));
        }
    }

    let blocks = parse_blocks(&items, |item| row_marker(&item.row))?;
    *structural_change |= blocks_have_controls(&blocks);
    let old_extra = std::mem::take(&mut table.extra_xml);
    let mut local_scopes = scopes.to_vec();
    let mut render_item = |item: &mut TableRowItem,
                           root: &Value,
                           scopes: &[Scope],
                           sentinels: &mut SentinelPool| {
        let mut count =
            render_nested_tables_in_row(&mut item.row, root, scopes, sentinels, structural_change)?;
        count += stage_row_scalars(&mut item.row, root, scopes, sentinels)?;
        for (_, control) in &mut item.controls {
            count += render_nested_tables_in_control(
                control,
                root,
                scopes,
                sentinels,
                structural_change,
            )?;
            count += stage_control_scalars(control, root, scopes, sentinels)?;
        }
        Ok(count)
    };
    let (evaluated, mut count) = evaluate_blocks(
        &blocks,
        root,
        &mut local_scopes,
        sentinels,
        &mut render_item,
    )?;

    let mut extra_xml = Vec::new();
    let mut content_controls = Vec::new();
    let new_len = evaluated.len();
    let mut rows = Vec::with_capacity(new_len);
    for (new_index, evaluated) in evaluated.into_iter().enumerate() {
        extra_xml.extend(
            old_extra
                .iter()
                .filter(|(at, _)| *at == evaluated.source_index)
                .map(|(_, raw)| (new_index, raw.clone())),
        );
        for (before, control) in evaluated.value.controls {
            content_controls.push((new_index, before, control));
        }
        rows.push(evaluated.value.row);
    }
    extra_xml.extend(
        old_extra
            .iter()
            .filter(|(at, _)| *at == old_len)
            .map(|(_, raw)| (new_len, raw.clone())),
    );
    for (before, mut control) in trailing_controls {
        count += render_nested_tables_in_control(
            &mut control,
            root,
            scopes,
            sentinels,
            structural_change,
        )?;
        count += stage_control_scalars(&mut control, root, scopes, sentinels)?;
        content_controls.push((new_len, before, control));
    }
    table.rows = rows;
    table.extra_xml = extra_xml;
    table.content_controls = content_controls;
    Ok(count)
}

fn render_nested_tables_in_row(
    row: &mut CT_Row,
    root: &Value,
    scopes: &[Scope],
    sentinels: &mut SentinelPool,
    structural_change: &mut bool,
) -> Result<usize> {
    let mut count = 0;
    for cell in &mut row.cells {
        count += render_nested_tables_in_cell(cell, root, scopes, sentinels, structural_change)?;
    }
    Ok(count)
}

fn render_nested_tables_in_cell(
    cell: &mut CT_Tc,
    root: &Value,
    scopes: &[Scope],
    sentinels: &mut SentinelPool,
    structural_change: &mut bool,
) -> Result<usize> {
    let mut count = 0;
    for content in &mut cell.content {
        match content {
            CellContent::Table(table) => {
                count += render_table(table, root, scopes, sentinels, structural_change)?;
            }
            CellContent::ContentControl(control) => {
                count += render_nested_tables_in_control(
                    control,
                    root,
                    scopes,
                    sentinels,
                    structural_change,
                )?;
            }
            CellContent::Paragraph(_) => {}
        }
    }
    Ok(count)
}

fn render_nested_tables_in_control(
    control: &mut CT_Sdt,
    root: &Value,
    scopes: &[Scope],
    sentinels: &mut SentinelPool,
    structural_change: &mut bool,
) -> Result<usize> {
    let mut count = 0;
    for content in &mut control.content {
        match content {
            SdtContent::Table(table) => {
                count += render_table(table, root, scopes, sentinels, structural_change)?;
            }
            SdtContent::Row(row) => {
                count +=
                    render_nested_tables_in_row(row, root, scopes, sentinels, structural_change)?;
            }
            SdtContent::Cell(cell) => {
                count +=
                    render_nested_tables_in_cell(cell, root, scopes, sentinels, structural_change)?;
            }
            SdtContent::ContentControl(nested) => {
                count += render_nested_tables_in_control(
                    nested,
                    root,
                    scopes,
                    sentinels,
                    structural_change,
                )?;
            }
            SdtContent::Paragraph(_) | SdtContent::Run(_) | SdtContent::RawXml(_) => {}
        }
    }
    Ok(count)
}

fn body_marker(content: &BodyContent) -> Result<Option<Control>> {
    match content {
        BodyContent::Paragraph(paragraph) => marker_from_sources(&[paragraph_text(paragraph)]),
        BodyContent::Table(_) | BodyContent::RawXml(_) => Ok(None),
        BodyContent::ContentControl(control) => {
            reject_unsupported_control_sources(&control_sources(control))?;
            Ok(None)
        }
    }
}

fn row_marker(row: &CT_Row) -> Result<Option<Control>> {
    let mut direct_sources = Vec::new();
    collect_row_marker_sources(row, &mut direct_sources);
    if !direct_sources
        .iter()
        .any(|source| source.contains("{%") || source.contains("%}"))
    {
        return Ok(None);
    }

    let mut all_sources = Vec::new();
    collect_row(row, &mut all_sources);
    marker_from_sources(&all_sources)
}

fn marker_from_sources(sources: &[String]) -> Result<Option<Control>> {
    let with_controls = sources
        .iter()
        .filter(|source| source.contains("{%") || source.contains("%}"))
        .collect::<Vec<_>>();
    if with_controls.is_empty() {
        return Ok(None);
    }
    let nonempty = sources
        .iter()
        .filter(|source| !source.trim().is_empty())
        .collect::<Vec<_>>();
    if nonempty.len() != 1 || with_controls.len() != 1 {
        return Err(template_error(
            "a control marker must occupy its own paragraph or table row",
        ));
    }
    parse_control(nonempty[0]).map(Some)
}

fn reject_unsupported_control_sources(sources: &[String]) -> Result<()> {
    if sources
        .iter()
        .any(|source| source.contains("{%") || source.contains("%}"))
    {
        return Err(template_error(
            "structural controls are supported only in main-body paragraphs and table rows",
        ));
    }
    Ok(())
}

fn parse_control(source: &str) -> Result<Control> {
    let trimmed = source.trim();
    if !trimmed.starts_with("{%") || !trimmed.ends_with("%}") {
        return Err(template_error(
            "a control marker must occupy its own paragraph or table row",
        ));
    }
    let inner = trimmed[2..trimmed.len() - 2].trim();
    if inner.is_empty()
        || inner.contains("{{")
        || inner.contains("{%")
        || inner.contains("}}")
        || inner.contains("%}")
    {
        return Err(template_error("malformed control tag"));
    }
    let words = inner.split_whitespace().collect::<Vec<_>>();
    match words.as_slice() {
        ["for", name, "in", path] => {
            validate_scope_name(name)?;
            Ok(Control::For {
                name: (*name).to_owned(),
                path: parse_path(path)?.into_iter().map(str::to_owned).collect(),
            })
        }
        ["if", path] => Ok(Control::If {
            path: parse_path(path)?.into_iter().map(str::to_owned).collect(),
        }),
        ["endfor"] => Ok(Control::End(EndKind::For)),
        ["endif"] => Ok(Control::End(EndKind::If)),
        _ => Err(template_error("unsupported or malformed control tag")),
    }
}

fn validate_scope_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.contains('.')
        || name.chars().any(|character| {
            character.is_whitespace() || matches!(character, '{' | '}' | '%' | '\0')
        })
    {
        return Err(template_error("invalid loop variable"));
    }
    Ok(())
}

fn parse_blocks<T: Clone>(
    items: &[T],
    marker: impl Fn(&T) -> Result<Option<Control>>,
) -> Result<Vec<Block<T>>> {
    let (blocks, next) = parse_block_level(items, 0, None, &marker)?;
    if next != items.len() {
        return Err(template_error("unexpected control marker"));
    }
    Ok(blocks)
}

fn parse_block_level<T: Clone, F: Fn(&T) -> Result<Option<Control>>>(
    items: &[T],
    mut index: usize,
    expected: Option<EndKind>,
    marker: &F,
) -> Result<(Vec<Block<T>>, usize)> {
    let mut blocks = Vec::new();
    while index < items.len() {
        match marker(&items[index])? {
            None => {
                blocks.push(Block::Item {
                    source_index: index,
                    value: items[index].clone(),
                });
                index += 1;
            }
            Some(Control::For { name, path }) => {
                let (body, next) = parse_block_level(items, index + 1, Some(EndKind::For), marker)?;
                blocks.push(Block::For { name, path, body });
                index = next;
            }
            Some(Control::If { path }) => {
                let (body, next) = parse_block_level(items, index + 1, Some(EndKind::If), marker)?;
                blocks.push(Block::If { path, body });
                index = next;
            }
            Some(Control::End(actual)) => {
                let Some(expected) = expected else {
                    return Err(template_error(&format!(
                        "unexpected {} marker",
                        actual.name()
                    )));
                };
                if actual != expected {
                    return Err(template_error(&format!(
                        "expected {} before {}",
                        expected.name(),
                        actual.name()
                    )));
                }
                return Ok((blocks, index + 1));
            }
        }
    }
    if let Some(expected) = expected {
        return Err(template_error(&format!(
            "missing {} marker",
            expected.name()
        )));
    }
    Ok((blocks, index))
}

fn blocks_have_controls<T>(blocks: &[Block<T>]) -> bool {
    blocks
        .iter()
        .any(|block| !matches!(block, Block::Item { .. }))
}

fn evaluate_blocks<T: Clone, F>(
    blocks: &[Block<T>],
    root: &Value,
    scopes: &mut Vec<Scope>,
    sentinels: &mut SentinelPool,
    render_item: &mut F,
) -> Result<(Vec<Evaluated<T>>, usize)>
where
    F: FnMut(&mut T, &Value, &[Scope], &mut SentinelPool) -> Result<usize>,
{
    let mut output = Vec::new();
    let mut count = 0;
    for block in blocks {
        match block {
            Block::Item {
                source_index,
                value,
            } => {
                let mut value = value.clone();
                count += render_item(&mut value, root, scopes, sentinels)?;
                output.push(Evaluated {
                    source_index: *source_index,
                    value,
                });
            }
            Block::For { name, path, body } => {
                let values = match resolve_value(root, scopes, path)? {
                    ResolvedValue::Value(Value::Array(values)) => values.clone(),
                    ResolvedValue::Deferred => Vec::new(),
                    _ => {
                        return Err(template_error(&format!(
                            "loop path `{}` does not resolve to an array",
                            path.join(".")
                        )));
                    }
                };
                if values.is_empty() {
                    scopes.push(Scope {
                        name: name.clone(),
                        value: Value::Null,
                        deferred: true,
                    });
                    let mut validation_sentinels = SentinelPool::new(&[]);
                    evaluate_blocks(body, root, scopes, &mut validation_sentinels, render_item)?;
                    scopes.pop();
                }
                for value in values {
                    scopes.push(Scope {
                        name: name.clone(),
                        value,
                        deferred: false,
                    });
                    let evaluated = evaluate_blocks(body, root, scopes, sentinels, render_item)?;
                    scopes.pop();
                    output.extend(evaluated.0);
                    count += evaluated.1;
                }
            }
            Block::If { path, body } => match resolve_value(root, scopes, path)? {
                ResolvedValue::Value(value) if is_truthy(value) => {
                    let evaluated = evaluate_blocks(body, root, scopes, sentinels, render_item)?;
                    output.extend(evaluated.0);
                    count += evaluated.1;
                }
                ResolvedValue::Value(_) | ResolvedValue::Deferred => {
                    let mut validation_sentinels = SentinelPool::new(&[]);
                    evaluate_blocks(body, root, scopes, &mut validation_sentinels, render_item)?;
                }
            },
        }
    }
    Ok((output, count))
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(value) => value.as_f64().is_some_and(|number| number != 0.0),
        Value::String(value) => !value.is_empty(),
        Value::Array(value) => !value.is_empty(),
        Value::Object(value) => !value.is_empty(),
    }
}

fn resolve_replacements(
    sources: &[String],
    data: &Value,
    scopes: &[Scope],
) -> Result<Vec<Replacement>> {
    let mut resolved = BTreeMap::<String, (String, usize)>::new();
    for source in sources {
        scan_source(source, data, scopes, &mut resolved)?;
    }
    Ok(resolved
        .into_iter()
        .map(|(literal, (value, occurrences))| Replacement {
            literal,
            value,
            occurrences,
        })
        .collect())
}

fn scan_source(
    source: &str,
    data: &Value,
    scopes: &[Scope],
    resolved: &mut BTreeMap<String, (String, usize)>,
) -> Result<()> {
    let mut cursor = 0;
    while cursor < source.len() {
        let marker =
            next_marker(&source[cursor..]).map(|(offset, marker)| (cursor + offset, marker));
        let Some((start, marker)) = marker else {
            break;
        };
        match marker {
            "{{" => {
                let content_start = start + 2;
                let Some(relative_end) = source[content_start..].find("}}") else {
                    return Err(template_error("unclosed scalar tag"));
                };
                let end = content_start + relative_end;
                let inner = &source[content_start..end];
                if inner.contains("{{") || inner.contains("{%") || inner.contains("%}") {
                    return Err(template_error("nested or mismatched scalar tag"));
                }
                let path = parse_path(inner)?
                    .into_iter()
                    .map(str::to_owned)
                    .collect::<Vec<_>>();
                let value = match resolve_value(data, scopes, &path)? {
                    ResolvedValue::Value(value) => scalar_text(value, &path)?,
                    ResolvedValue::Deferred => {
                        cursor = end + 2;
                        continue;
                    }
                };
                let literal = source[start..end + 2].to_owned();
                let entry = resolved
                    .entry(literal)
                    .or_insert_with(|| (value.clone(), 0));
                if entry.0 != value {
                    return Err(template_error(
                        "one scalar literal resolved to different values in one scope",
                    ));
                }
                entry.1 += 1;
                cursor = end + 2;
            }
            "{%" => {
                let content_start = start + 2;
                let Some(relative_end) = source[content_start..].find("%}") else {
                    return Err(template_error("unclosed control tag"));
                };
                let end = content_start + relative_end;
                let inner = source[content_start..end].trim();
                if inner.is_empty()
                    || inner.contains("{{")
                    || inner.contains("{%")
                    || inner.contains("}}")
                {
                    return Err(template_error("malformed control tag"));
                }
                cursor = end + 2;
            }
            "}}" | "%}" => return Err(template_error("closing template delimiter without opener")),
            _ => unreachable!("next_marker returns known markers"),
        }
    }
    Ok(())
}

fn stage_paragraph_scalars(
    paragraph: &mut CT_P,
    root: &Value,
    scopes: &[Scope],
    sentinels: &mut SentinelPool,
) -> Result<usize> {
    let replacements = resolve_replacements(&[paragraph_text(paragraph)], root, scopes)?;
    let mut count = 0;
    for replacement in replacements {
        let sentinel = sentinels.stage(&replacement.value, replacement.occurrences);
        let replaced =
            placeholder::replace_in_paragraph(paragraph, &replacement.literal, &sentinel);
        if replaced != replacement.occurrences {
            return Err(Error::Other(format!(
                "template changed while staging: discovered {} tags but replaced {replaced}",
                replacement.occurrences
            )));
        }
        count += replaced;
    }
    Ok(count)
}

fn stage_row_scalars(
    row: &mut CT_Row,
    root: &Value,
    scopes: &[Scope],
    sentinels: &mut SentinelPool,
) -> Result<usize> {
    let mut sources = Vec::new();
    collect_row(row, &mut sources);
    let replacements = resolve_replacements(&sources, root, scopes)?;
    if replacements.is_empty() {
        return Ok(0);
    }
    let mut table = CT_Tbl::new();
    table.rows.push(std::mem::take(row));
    let mut count = 0;
    for replacement in replacements {
        let sentinel = sentinels.stage(&replacement.value, replacement.occurrences);
        let replaced = placeholder::replace_in_table(&mut table, &replacement.literal, &sentinel);
        if replaced != replacement.occurrences {
            return Err(Error::Other(format!(
                "template changed while staging: discovered {} tags but replaced {replaced}",
                replacement.occurrences
            )));
        }
        count += replaced;
    }
    let Some(rendered) = table.rows.pop() else {
        return Err(Error::Other("template staging lost a table row".to_owned()));
    };
    *row = rendered;
    Ok(count)
}

fn stage_control_scalars(
    control: &mut CT_Sdt,
    root: &Value,
    scopes: &[Scope],
    sentinels: &mut SentinelPool,
) -> Result<usize> {
    let replacements = resolve_replacements(&control_sources(control), root, scopes)?;
    if replacements.is_empty() {
        return Ok(0);
    }
    let mut table = CT_Tbl::new();
    table.content_controls.push((0, 0, control.clone()));
    let mut count = 0;
    for replacement in replacements {
        let sentinel = sentinels.stage(&replacement.value, replacement.occurrences);
        let replaced = placeholder::replace_in_table(&mut table, &replacement.literal, &sentinel);
        if replaced != replacement.occurrences {
            return Err(Error::Other(format!(
                "template changed while staging: discovered {} tags but replaced {replaced}",
                replacement.occurrences
            )));
        }
        count += replaced;
    }
    let Some((_, _, rendered)) = table.content_controls.pop() else {
        return Err(Error::Other(
            "template staging lost a content control".to_owned(),
        ));
    };
    *control = rendered;
    Ok(count)
}

fn resolve_value<'a>(
    root: &'a Value,
    scopes: &'a [Scope],
    path: &[String],
) -> Result<ResolvedValue<'a>> {
    let display_path = path.join(".");
    for scope in scopes.iter().rev() {
        if path
            .first()
            .is_some_and(|component| component == &scope.name)
        {
            if scope.deferred {
                return Ok(ResolvedValue::Deferred);
            }
            return traverse_path(&scope.value, &path[1..], &display_path)
                .map(ResolvedValue::Value);
        }
    }
    let mut saw_deferred = false;
    for scope in scopes.iter().rev() {
        if scope.deferred {
            saw_deferred = true;
            continue;
        }
        if let Some(value) = try_traverse_path(&scope.value, path) {
            return Ok(ResolvedValue::Value(value));
        }
    }
    if let Some(value) = try_traverse_path(root, path) {
        return Ok(ResolvedValue::Value(value));
    }
    if saw_deferred {
        for scope in scopes.iter().rev().filter(|scope| !scope.deferred) {
            if path.first().is_some_and(|component| {
                scope
                    .value
                    .as_object()
                    .is_some_and(|object| object.contains_key(component))
            }) {
                return traverse_path(&scope.value, path, &display_path).map(ResolvedValue::Value);
            }
        }
        if path.first().is_some_and(|component| {
            root.as_object()
                .is_some_and(|object| object.contains_key(component))
        }) {
            return traverse_path(root, path, &display_path).map(ResolvedValue::Value);
        }
        return Ok(ResolvedValue::Deferred);
    }
    traverse_path(root, path, &display_path).map(ResolvedValue::Value)
}

fn try_traverse_path<'a>(mut value: &'a Value, path: &[String]) -> Option<&'a Value> {
    for component in path {
        value = value.as_object()?.get(component)?;
    }
    Some(value)
}

fn traverse_path<'a>(
    mut value: &'a Value,
    path: &[String],
    display_path: &str,
) -> Result<&'a Value> {
    for component in path {
        let Value::Object(object) = value else {
            return Err(template_error(&format!(
                "path `{display_path}` crosses a non-object value"
            )));
        };
        let Some(next) = object.get(component) else {
            return Err(template_error(&format!("missing path `{display_path}`")));
        };
        value = next;
    }
    Ok(value)
}

fn scalar_text(value: &Value, path: &[String]) -> Result<String> {
    match value {
        Value::String(value) => Ok(value.clone()),
        Value::Number(value) => Ok(value.to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Null => Ok(String::new()),
        Value::Array(_) | Value::Object(_) => Err(template_error(&format!(
            "path `{}` does not resolve to a scalar value",
            path.join(".")
        ))),
    }
}

fn next_marker(source: &str) -> Option<(usize, &'static str)> {
    ["{{", "{%", "}}", "%}"]
        .into_iter()
        .filter_map(|marker| source.find(marker).map(|offset| (offset, marker)))
        .min_by_key(|(offset, _)| *offset)
}

fn parse_path(inner: &str) -> Result<Vec<&str>> {
    let path = inner.trim();
    if path.is_empty() {
        return Err(template_error("empty scalar path"));
    }
    let components = path.split('.').collect::<Vec<_>>();
    if components.iter().any(|component| {
        component.is_empty()
            || component.chars().any(|character| {
                character.is_whitespace() || matches!(character, '{' | '}' | '%' | '\0')
            })
    }) {
        return Err(template_error("invalid dotted scalar path"));
    }
    Ok(components)
}

fn template_error(message: &str) -> Error {
    Error::Other(format!("invalid template: {message}"))
}

pub(crate) fn body_sources(document: &CT_Document) -> Vec<String> {
    let mut sources = Vec::new();
    for content in &document.body.content {
        match content {
            BodyContent::Paragraph(paragraph) => sources.push(paragraph_text(paragraph)),
            BodyContent::Table(table) => collect_table(table, &mut sources),
            BodyContent::ContentControl(control) => collect_control(control, &mut sources),
            BodyContent::RawXml(_) => {}
        }
    }
    sources
}

pub(crate) fn header_footer_sources(header_footer: &CT_HdrFtr) -> Vec<String> {
    header_footer
        .paragraphs
        .iter()
        .map(paragraph_text)
        .collect()
}

fn paragraph_text(paragraph: &CT_P) -> String {
    paragraph
        .runs
        .iter()
        .flat_map(|run| &run.content)
        .filter_map(|content| match content {
            RunContent::Text(text) => Some(text.text.as_str()),
            _ => None,
        })
        .collect()
}

fn collect_table(table: &CT_Tbl, sources: &mut Vec<String>) {
    for (_, _, control) in &table.content_controls {
        collect_control(control, sources);
    }
    for row in &table.rows {
        collect_row(row, sources);
    }
}

fn control_sources(control: &CT_Sdt) -> Vec<String> {
    let mut sources = Vec::new();
    collect_control(control, &mut sources);
    sources
}

fn collect_control(control: &CT_Sdt, sources: &mut Vec<String>) {
    for content in &control.content {
        match content {
            SdtContent::Paragraph(paragraph) => sources.push(paragraph_text(paragraph)),
            SdtContent::Table(table) => collect_table(table, sources),
            SdtContent::Row(row) => collect_row(row, sources),
            SdtContent::Cell(cell) => collect_cell(cell, sources),
            SdtContent::ContentControl(nested) => collect_control(nested, sources),
            SdtContent::Run(_) | SdtContent::RawXml(_) => {}
        }
    }
}

fn collect_row(row: &CT_Row, sources: &mut Vec<String>) {
    for (_, _, control) in &row.content_controls {
        collect_control(control, sources);
    }
    for cell in &row.cells {
        collect_cell(cell, sources);
    }
}

fn collect_row_marker_sources(row: &CT_Row, sources: &mut Vec<String>) {
    for (_, _, control) in &row.content_controls {
        collect_control_marker_sources(control, sources);
    }
    for cell in &row.cells {
        collect_cell_marker_sources(cell, sources);
    }
}

fn collect_cell_marker_sources(cell: &CT_Tc, sources: &mut Vec<String>) {
    for content in &cell.content {
        match content {
            CellContent::Paragraph(paragraph) => sources.push(paragraph_text(paragraph)),
            CellContent::Table(_) => {}
            CellContent::ContentControl(control) => {
                collect_control_marker_sources(control, sources);
            }
        }
    }
}

fn collect_control_marker_sources(control: &CT_Sdt, sources: &mut Vec<String>) {
    for content in &control.content {
        match content {
            SdtContent::Paragraph(paragraph) => sources.push(paragraph_text(paragraph)),
            SdtContent::Table(_) => {}
            SdtContent::Row(row) => collect_row_marker_sources(row, sources),
            SdtContent::Cell(cell) => collect_cell_marker_sources(cell, sources),
            SdtContent::ContentControl(nested) => {
                collect_control_marker_sources(nested, sources);
            }
            SdtContent::Run(run) => sources.push(
                run.content
                    .iter()
                    .filter_map(|content| match content {
                        RunContent::Text(text) => Some(text.text.as_str()),
                        _ => None,
                    })
                    .collect(),
            ),
            SdtContent::RawXml(_) => {}
        }
    }
}

fn collect_cell(cell: &CT_Tc, sources: &mut Vec<String>) {
    for content in &cell.content {
        match content {
            CellContent::Paragraph(paragraph) => sources.push(paragraph_text(paragraph)),
            CellContent::Table(table) => collect_table(table, sources),
            CellContent::ContentControl(control) => collect_control(control, sources),
        }
    }
}

pub(crate) fn text_box_sources(xml: &[u8]) -> Result<Vec<String>> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut sources = Vec::new();
    let mut buffer = Vec::new();
    let mut in_text_box = false;

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(element))
                if matches_local_name(element.name().as_ref(), b"txbxContent") =>
            {
                in_text_box = true;
            }
            Ok(Event::Start(element))
                if in_text_box && matches_local_name(element.name().as_ref(), b"p") =>
            {
                let paragraph = CT_P::from_xml(&mut reader)?;
                sources.push(paragraph_text(&paragraph));
            }
            Ok(Event::Start(element)) if in_text_box => {
                reader
                    .read_to_end_into(element.name(), &mut Vec::new())
                    .map_err(template_xml_error)?;
            }
            Ok(Event::End(element))
                if in_text_box && matches_local_name(element.name().as_ref(), b"txbxContent") =>
            {
                in_text_box = false;
            }
            Ok(_) => {}
            Err(error) => return Err(template_xml_error(error)),
        }
        buffer.clear();
    }

    Ok(sources)
}

pub(crate) fn chart_sources(xml: &[u8]) -> Result<Vec<String>> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut sources = Vec::new();
    let mut buffer = Vec::new();

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(element))
                if matches_local_name(element.name().as_ref(), b"t")
                    || matches_local_name(element.name().as_ref(), b"v") =>
            {
                sources.push(oxml_core::xml_text::read_element_text(
                    &mut reader,
                    element.name(),
                ));
            }
            Ok(_) => {}
            Err(error) => return Err(template_xml_error(error)),
        }
        buffer.clear();
    }

    Ok(sources)
}

fn template_xml_error(error: quick_xml::Error) -> Error {
    Error::Other(format!("invalid template XML: {error}"))
}
