# F-149, Revision model

**Status**: completed
**Sprint**: S47
**Size**: L
**Depends on**: none

## Problem

Paragraph revision wrappers are preserved today only as indexed raw XML in
`crates/rdocx-oxml/src/text.rs:896`, so callers cannot inspect their identity,
author, timestamp, kind, or content. Run children such as `w:delText` are also
opaque. Property changes are less safe. `w:pPrChange`, `w:rPrChange`, and
`w:tblPrChange` enter parser branches that discard unsupported child elements
in `crates/rdocx-oxml/src/properties.rs:129`,
`crates/rdocx-oxml/src/properties.rs:659`, and
`crates/rdocx-oxml/src/table.rs:453`. `w:sectPrChange` survives as raw XML but
has no typed metadata or prior-property projection.

F-150 cannot accept or reject a scoped change until the model retains the
change placement, stable identity, metadata, wrapper content, and recorded
prior properties. The model must add that information without normalizing
producer XML during an ordinary load and save.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-149, Revision model".
- `docs/hld/03-architecture.md`, "What stays put" and "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, "Package integrity".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy".

## Approach

Add one low-level revision model whose parsed value is a read-only projection
and whose captured raw subtree remains the sole serialization source until a
later accept or reject operation replaces it. This follows the existing
alternate-content preservation pattern and prevents typed inspection from
changing prefix choice, whitespace, attributes, or unsupported descendants.

```rust
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

pub struct CT_Revision {
    kind: RevisionKind,
    id: i32,
    author: String,
    timestamp: Option<String>,
    raw_xml: Vec<u8>,
    content: RevisionContent,
}

enum RevisionContent {
    Runs(Vec<CT_R>),
    Marker,
    PriorRunProperties(Box<CT_RPr>),
    PriorParagraphProperties(Box<CT_PPr>),
    PriorTableProperties(Box<CT_TblPr>),
    PriorSectionProperties(Box<CT_SectPr>),
}
```

`RunContent::DeletedText(CT_Text)` represents `w:delText` inside a deletion
projection. It is not a separate revision because it carries no independent
identity or metadata.

Store paragraph content revisions at ordered run boundaries beside the
existing raw, comment, bookmark, and content-control boundary data. Add typed
revision fields at each schema-defined final change slot in run, paragraph,
table, and section properties. Model story-listed contextual `w:ins` and
`w:del` markers wherever the existing main-document property grammar admits
them. Preserve their placement so F-150 can distinguish a content wrapper from
a paragraph-mark or row marker. Leave unlisted revision vocabularies such as
`w:numberingChange`, `w:trPrChange`, `w:tcPrChange`, `w:cellIns`, and
`w:cellDel` outside this story.

Invalid revision elements that lack a parseable required `w:id` or `w:author`
remain raw and unreported rather than making a previously readable document
fail to open. A valid element may omit `w:date`, which is exposed as `None`.

Recursively collect valid modeled revisions in main-body document order,
including revisions nested in tables and content controls. Add the native
facade API without expanding Python, WASM, or CLI surfaces:

```rust
#[derive(Debug, Clone, Copy)]
pub struct RevisionRef<'a> { /* private */ }

impl RevisionRef<'_> {
    pub fn id(&self) -> i32;
    pub fn author(&self) -> &str;
    pub fn timestamp(&self) -> Option<&str>;
    pub fn kind(&self) -> RevisionKind;
}

impl Document {
    pub fn revisions(&self) -> Vec<RevisionRef<'_>>;
}
```

Add these focused source modules if approved:

```text
crates/rdocx-oxml/src/revision.rs
crates/rdocx/src/revision.rs
```

No new crate, dependency, trait, generic parameter, or feature flag is added.

## Rejected alternatives

- Parse metadata on every facade accessor while leaving every revision opaque.
  That repeats XML parsing and gives F-150 no stable prior-property model.
- Serialize the typed projection immediately. That would normalize untouched
  producer XML and violate the exact preservation boundary.
- Put the low-level model in `text.rs` and the facade in `document.rs`. Both
  files already coordinate several unrelated features, and revision behavior
  would become harder to locate.
- Model every tracked-change element in WordprocessingML. The story names a
  bounded set, and unlisted elements continue to survive as raw XML.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `revision_attributes_are_prefix_tolerant_and_namespace_checked` | Aliased Word prefixes parse, foreign same-local-name elements remain raw, and absent dates are accepted |
| unit | `property_changes_write_in_their_schema_final_slots` | Run, paragraph, table, and section changes are emitted once after ordinary properties |
| unit | `numbering_preservation_does_not_duplicate_typed_changes` | Property preservation and typed revision emission do not write a change subtree twice |
| round-trip | `revision_elements_round_trip_unchanged_and_report_metadata` | Insertions, deletions, moves, deleted text, contextual markers, and all four property changes retain exact raw subtrees and report identity, author, timestamp, kind, and typed content |
| integration | `nested_revisions_are_reported_once_in_document_order` | Main-body revisions inside paragraphs, tables, cells, and content controls are collected without skips or duplicates |

The **test gate**, from the backlog, is round-trip. Every revision element
survives a load and save unchanged, and each valid modeled revision is reported
with its author, timestamp, identity, and kind.

Tests stay in crate-local modules and the existing
`crates/rdocx/tests/integration_test.rs` entrypoint. No new test binary is
added.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`

Record revision ownership, the raw-source and read-only-projection contract,
ordered recursive traversal, the native metadata API, and the unchanged
binding surfaces.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Add schema-order,
  prefix-tolerant, namespace-collision, malformed-metadata, recursive
  round-trip, and exact raw-subtree preservation checks.
- Public API of published crates. Read HLD 10 and the structural rules. The
  low-level types and native facade accessor are additive. Run
  `cargo publish --workspace --dry-run` and assert every produced `.crate`
  remains below 10 MiB.
- New modules or files. The two focused revision modules require explicit
  approval before implementation.

## Hash harness

Expected unchanged across all entries. Existing samples contain no modeled
tracked revisions. Any delta is unexplained and blocks the sprint.

## Implementation checklist

- [x] Add the approved low-level and facade revision modules and exports.
- [x] Parse namespace-correct revision metadata while retaining raw subtrees.
- [x] Project wrapper runs, deleted text, contextual markers, and prior properties.
- [x] Emit property changes once in their schema-final positions.
- [x] Reconcile numbering preservation with the new typed change slots.
- [x] Collect main-body revisions recursively in document order.
- [x] Add the native `RevisionRef` API without changing binding surfaces.
- [x] Add namespace, order, no-duplication, exact round-trip, and traversal tests.
- [x] Run focused checks plus the declared parser and packaging riders.
- [x] Update exactly HLD 03 and HLD 10 at completion.

## Open questions

None. The two focused revision modules are approved. The story covers every
reachable main-document placement of the listed `w:ins` and `w:del` elements,
including contextual property markers. Malformed elements remain raw and
unreported, missing dates are allowed, shared ids remain separate reported
entries, and revision discovery does not extend to headers, footers, notes,
comments, or styles in this story.
