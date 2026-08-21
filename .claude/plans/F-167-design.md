# F-167, Document comparison

**Status**: completed
**Sprint**: S51
**Size**: L
**Depends on**: F-149

## Problem

F-149 and F-150 provide revision inspection and atomic resolution, but the
facade can only enumerate, accept, or reject revisions
(`crates/rdocx/src/revision.rs:91`). It cannot generate tracked revisions by
comparing an original with an edited document.

The required ownership boundaries already exist. Body content distinguishes
paragraphs, tables, content controls, and preserved raw XML
(`crates/rdocx-oxml/src/document.rs:19`). Paragraphs retain revisions at
ordered run boundaries (`crates/rdocx-oxml/src/text.rs:1518`), and table rows
retain contextual revision markers (`crates/rdocx-oxml/src/table.rs:737`). The
comparison must compose those representations without turning formatting-only
changes into content revisions.

## Spec reference

- `docs/hld/03-architecture.md`, "What stays put" and "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, "Package integrity".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The hash harness".
- `docs/hld/14-development-backlog.md`, "Milestone 16, Document automation"
  and "F-167, Document comparison".

## Approach

Add an additive native mutation and diagnostic value:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparisonDiagnostic {
    pub location: String,
    pub message: String,
}

impl Document {
    pub fn compare(
        &mut self,
        edited: &Document,
        author: &str,
        timestamp: &str,
    ) -> Result<Vec<ComparisonDiagnostic>>;
}
```

Create `crates/rdocx/src/comparison.rs` so generation does not further overload
the revision-resolution module. Rename the private template clone helper to a
general staging clone and reuse it. Reject inputs with existing modeled
revisions so accepted and tracked baselines are unambiguous.

Use a deterministic hierarchical longest-common-subsequence implementation over
concrete semantic signature vectors, with no dependency. Align body paragraphs
and tables, paragraph runs, and table rows. Coalesce adjacent deletion and
insertion operations into replacements. Generate canonical fixed-prefix
`w:del`, `w:ins`, paragraph-mark, row, and numbering property revisions at
schema-valid boundaries with collision-free ids. Preserve content-control shells
and recurse through their modeled content. Whole-table, nested-table, and
content-control content changes are in scope.

Compare non-structural formatting separately. Report it as a stable diagnostic
and retain the original formatting without adding a revision. The gate's
"exactly" means normalized modeled main-body text, table, and list structure,
excluding diagnostic-only formatting and unrelated package bytes.

Before committing, accept all generated revisions on one staged copy and
compare its normalized scoped structure with `edited`. Reject them on another
staged copy and compare with the original. Any alignment, metadata, parse,
serialization, or postcondition failure leaves the original unchanged.

## Rejected alternatives

- Add a diff dependency. A concrete hierarchical LCS is sufficient.
- Compare package bytes. The story is scoped to modeled main-body structure.
- Diff characters or words. That loses exact edited run boundaries and creates
  formatting ambiguity.
- Emit property revisions for formatting-only changes. The backlog requires
  diagnostics instead.
- Put generation in `revision.rs`. Resolution and comparison are separate
  responsibilities, and that file is already large.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `repeated_content_produces_a_deterministic_comparison` | Duplicate paragraphs, runs, and rows choose stable matches and ids |
| unit | `comparison_metadata_is_escaped_and_ids_do_not_collide` | Author XML escaping, RFC 3339 timestamp validation, and collision-free ids |
| regression, gate | `accepting_a_comparison_reproduces_the_edited_body_exactly` | Body text, paragraph, table, cell, nested table, content-control, and list edits normalize to the edited body |
| regression | `rejecting_a_comparison_reproduces_the_original_body_exactly` | Generated revisions reverse without residual markers or tracked empty containers |
| regression | `formatting_only_changes_report_diagnostics_without_revisions` | Stable locations and messages are returned while original formatting remains |
| integration | `a_failed_comparison_leaves_the_original_package_unchanged` | Invalid metadata or unsupported structure leaves document, package, and caches unchanged |
| round-trip | `comparison_preserves_unmodelled_xml_byte_for_byte` | Unrelated raw body, paragraph, table, cell, and content-control XML survives compare and reopen |

The **test gate**, from the backlog, is regression. Comparing a document with
its edited copy produces revisions that, when accepted, reproduce the edited
copy exactly within the declared structural scope.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- **Any parser or serialiser**. Read HLD 04 and HLD 06. Check schema order,
  fixed-prefix generated XML, prefix-tolerant reparse, escaped metadata, and a
  byte-identical round trip for unmodelled subtrees.
- **Public API of a published crate**. Read HLD 10 and the structural rules.
  The method and diagnostic type are additive and native-only. Run the full
  package dry-run and archive size assertion.
- **A new module or file**. Explicit approval is required for
  `crates/rdocx/src/comparison.rs`. No new trait, crate, dependency, generic,
  feature flag, or binding surface is introduced.

## Hash harness

Expected unchanged across all 49 entries. Samples do not invoke comparison.
Any delta is unexplained and blocks integration.

## Implementation checklist

- [x] Confirm the API, metadata, exactness, existing-revision, and formatting policy.
- [x] Obtain approval for `crates/rdocx/src/comparison.rs`.
- [x] Generalize the private staging clone helper.
- [x] Implement deterministic hierarchical alignment.
- [x] Generate run, paragraph, row, and numbering revisions in schema order.
- [x] Emit formatting-only diagnostics without property revisions.
- [x] Preserve raw XML and content-control ownership boundaries.
- [x] Validate accepted and rejected postconditions before committing.
- [x] Add the named gate and focused unit, integration, and round-trip coverage.
- [x] Run the parser, packaging, full verification, and unchanged-harness riders.
- [x] Update exactly HLD 03, HLD 04, HLD 10, and HLD 12.

## Open questions

None. The focused comparison module, mutating metadata-bearing API, normalized
modeled main-body exactness, existing-revision rejection, nested structure
scope, and diagnostic-only formatting policy are approved.
