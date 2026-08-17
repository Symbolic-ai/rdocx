# F-161, Field evaluation engine

**Status**: approved
**Sprint**: S49
**Size**: L
**Depends on**: F-160, F-154

## Problem

The current layout path evaluates PAGE, NUMPAGES, REF, and PAGEREF, then skips
all `FieldType::Other` values (`crates/rdocx-layout/src/engine.rs:884`). The
document facade has bookmark and core-property state, but no complete field
evaluation context. It does not load custom properties or document variables,
retain a source filename for byte-opened documents, or accept merge data,
included content, or a deterministic clock (`crates/rdocx/src/document.rs:52`).

F-161 must consume F-160's grammar and evaluate thirteen named field families
plus formatting switches. It must keep existing pagination substitution intact
and retain cached display text whenever an instruction is unsupported,
malformed, or missing required input.

## Spec reference

- `docs/hld/14-development-backlog.md`, "Milestone 16, Document automation"
  and "F-161, Field evaluation engine".
- `docs/hld/03-architecture.md`, "What stays put" and field ownership.
- `docs/hld/08-rendering-spec.md`, "Word bookmark field pagination".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The hash harness".

## Approach

Add a pure, non-mutating evaluator in a focused native facade module:

```rust
pub struct FieldEvaluationContext {
    pub now: Option<FieldDateTime>,
    pub file_name: Option<String>,
    pub file_path: Option<String>,
    pub merge_fields: BTreeMap<String, String>,
    pub included_text: BTreeMap<String, String>,
}

pub struct FieldEvaluation {
    pub field_index: usize,
    pub instruction: String,
    pub cached_result: String,
    pub outcome: FieldOutcome,
}

pub enum FieldOutcome {
    Resolved(String),
    DeferredPagination,
    KeepStored { diagnostic: String },
}

impl Document {
    pub fn evaluate_fields(
        &self,
        context: &FieldEvaluationContext,
    ) -> Result<Vec<FieldEvaluation>>;
}
```

The document-order `field_index` is valid for one unchanged document snapshot.
It is not persisted into OOXML. F-161 stays read-only. F-162 alone decides when
an outcome replaces a cached result.

Resolve package-backed values from bookmarks, styles, core and custom document
properties, and settings document variables. Resolve DATE, TIME, FILENAME,
MERGEFIELD, and INCLUDETEXT only from explicit context. Do not read the ambient
clock or filesystem. Missing context, a missing target, a malformed instruction,
or an unsupported field returns `KeepStored` with a stable diagnostic.

Implement recursive IF evaluation, document-order SEQ state, approved STYLEREF
search, REF and PAGEREF lookup, property and variable lookup, metadata fields,
merge lookup, safe included-text lookup, and the approved general, numeric, and
date formatting matrix. PAGE, NUMPAGES, and PAGEREF values that require layout
remain on the established post-pagination path and return
`DeferredPagination` from the pure facade evaluator.

The approved common matrix is:

- IF operators `=`, `<>`, `<`, `<=`, `>`, and `>=`, including nested field
  operands and `?` or `*` wildcards for equality comparisons.
- SEQ default and `\\n`, repeat `\\c`, hide `\\h`, reset `\\r n`, and heading
  restart `\\s level`. Main text, headers, footers, footnotes, and endnotes keep
  independent sequence state.
- STYLEREF default nearest search and `\\l` last-on-page direction. Paragraph
  numbering switches `\\n`, `\\r`, `\\t`, and `\\w` stay on cached fallback
  until numbered source text is available through the typed model. The `\\p`
  relative-position switch is supported for typed source placement.
- FILENAME basename by default and the explicit context path for `\\p`.
- INCLUDETEXT supplied whole text or supplied bookmark text. `\\!` is accepted
  because nested included fields are never evaluated implicitly. Converter
  selection through `\\c` stays on cached fallback.
- MERGEFIELD exact-name lookup with `\\b` prefix and `\\f` suffix. `\\m` and
  `\\v` are accepted without altering the supplied string.
- General formats `\\* Upper`, `Lower`, `FirstCap`, `Caps`, `Arabic`,
  `alphabetic`, `ALPHABETIC`, `roman`, `ROMAN`, and `Ordinal`.
  `MERGEFORMAT` and `Charformat` preserve source result formatting without
  changing the evaluated string.
- Numeric `\\#` pictures covering required and optional digits, decimal and
  grouping separators, quoted literals, and positive, negative, and zero
  sections. Date-time `\\@` pictures cover the Word tokens used by the pinned
  corpus for year, month, day, weekday, hour, minute, second, and AM or PM.

Traverse every typed `CT_P` location owned by the facade: body paragraphs,
tables, content controls, headers, footers, footnotes, and endnotes. Text boxes
that remain raw XML are preserved and reported as outside the typed evaluation
boundary.

The new `crates/rdocx/src/field.rs` module is justified by thirteen concrete
handlers, recursive evaluation, sequence state, formatting, and the F-162 and
F-166 consumers. No trait or generic is introduced because there is only one
evaluator and one context representation.

## Rejected alternatives

- Read the current clock or filesystem implicitly. That makes tests
  nondeterministic and turns INCLUDETEXT into an unexpected external read.
- Mutate cached results during evaluation. That collapses F-161 into F-162 and
  violates the explicit leave-alone policy.
- Put every handler in `document.rs`. That file already owns package and facade
  coordination, and adding the complete evaluator would make it harder to
  answer what either responsibility does.
- Add one evaluator trait per field. There is no second implementation today.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `every_supported_field_matches_the_pinned_word_result` | One readable in-code document covers every supported field and compares document-order results with exact literals from the pinned Word build |
| unit | `nested_if_and_comparison_operators_evaluate_recursively` | Nested operands plus the approved string and numeric comparison operators |
| unit | `sequence_state_is_scoped_and_reset_by_supported_switches` | Independent identifiers, repeat and reset behaviour, and number formatting |
| regression | `missing_context_and_unsupported_fields_keep_their_cached_display` | No blank result and stable diagnostics |
| regression | `document_properties_variables_and_author_use_package_values` | Core, custom, settings variable, and author lookup |
| regression | `styleref_searches_the_approved_direction_and_scope` | The approved story traversal and nearest-style behaviour |
| regression | `date_time_filename_mergefield_and_includetext_use_only_explicit_context` | Fixed date-time, supplied names, merge record, and include map with no ambient I/O |
| regression | existing `ref_and_pageref_resolve_to_the_bookmark_text_and_final_page` | Existing bookmark and pagination behaviour remains intact |
| unit | `formatting_switches_match_the_pinned_word_matrix` | The approved general, numeric, date-time, and field-specific switch subset |

The **test gate**, from the backlog, is regression. Each supported field matches
the readable expected set captured from Microsoft Word 16.104 build
16.104.25121423. The in-code input identity and normalized document-order
results are recorded with the oracle identity. The oracle is test
infrastructure only and no binary fixture is committed.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

Record evaluator ownership and inputs, deterministic external values,
pagination deferral, the additive native-only API, unchanged binding surfaces,
and the pinned Word field matrix.

## Risk routing

- Layout, pagination, line breaking, and text shaping. Read HLD 08, preserve
  the single-pagination algorithm, and use bundled deterministic fonts for all
  render evidence.
- Any parser or serialiser. Settings gains a document-variable projection and
  fields consume F-160 storage. Check aliases, fixed prefixes after mutation,
  schema order, and byte-preservation of unmodelled settings and field XML.
- Public API of published crates. Read HLD 10 and the structural rules. The
  native evaluator API is additive and does not expand Python, WASM, or CLI.
  Run the full package dry-run and archive size assertion.
- A new module or file. Obtain explicit approval for
  `crates/rdocx/src/field.rs`. No new trait, generic, crate, or feature exists.
- An external oracle comparison. Follow differential-testing, pin the exact
  Word version and input identity, keep expected values readable, and triage
  every disagreement rather than accepting a raw diff.

## Hash harness

Expected unchanged across all current entries. Evaluation is read-only and
requires explicit context, so ordinary sample generation must not rewrite
field caches or change rendered output.

## Implementation checklist

- [ ] Consume the approved F-160 field AST and source placement.
- [ ] Add the approved native evaluation context, outcome, and document-order result API.
- [ ] Load read-only custom properties and settings document variables.
- [ ] Implement the approved field and formatting matrix.
- [ ] Keep missing, malformed, and unsupported values on cached fallback with diagnostics.
- [ ] Keep PAGE, NUMPAGES, and PAGEREF on the established pagination path.
- [ ] Add explicit date-time, filename, merge, and included-text inputs with no ambient I/O.
- [ ] Add the pinned Word regression matrix and focused negative tests.
- [ ] Run parser, deterministic layout, package, oracle, and unchanged hash riders.
- [ ] Update exactly HLD 03, HLD 08, HLD 10, and HLD 12 at completion.

## Open questions

None. The additive read-only API, the new `crates/rdocx/src/field.rs` module,
explicit context with no ambient I/O, common switch matrix, and typed paragraph
story scope above are approved.
