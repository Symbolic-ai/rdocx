# F-166, Mail merge

**Status**: approved
**Sprint**: S51
**Size**: M
**Depends on**: F-161, F-164

## Problem

The facade can evaluate and materialize one `FieldEvaluationContext` at a time
(`crates/rdocx/src/field.rs:60`), and it can render one structured JSON value
into one staged document (`crates/rdocx/src/template.rs:138`). It has no
record-set operation that produces independent documents or one document with a
section for each record.

General field evaluation deliberately keeps the cached display for a missing
`MERGEFIELD` (`crates/rdocx/src/field.rs:762`). Mail merge instead requires an
absent record value to become an empty result. The merge needs that local policy
without changing ordinary field-update behavior.

## Spec reference

- `docs/hld/03-architecture.md`, "What stays put", field evaluation, field
  updates, and structured template ownership.
- `docs/hld/04-opc-and-packaging.md`, "Package integrity".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The hash harness".
- `docs/hld/14-development-backlog.md`, "Milestone 16, Document automation"
  and "F-166, Mail merge".

## Approach

Add two concrete native facade methods in the existing document, field, and
template modules:

```rust
pub fn mail_merge(
    &self,
    records: &[BTreeMap<String, String>],
) -> Result<Vec<Document>>;

pub fn mail_merge_sections(
    &self,
    records: &[BTreeMap<String, String>],
) -> Result<Document>;
```

Reuse the field evaluator and update traversal through a private merge policy
that maps only absent `MERGEFIELD` values to `Resolved("")`. Ordinary
`evaluate_fields` and `update_fields` keep their cached-display fallback.

Stage and validate one complete document and package clone per record before
returning any output. Separate mode returns those complete clones. Section mode
concatenates candidate body entries in record order. Every non-final record
moves its final section properties to a section-ending paragraph, and the final
record retains the body-final `sectPr`. It does not use the existing
default-letter append helper.

Combined section mode varies main-body paragraphs, tables, and content controls
per record. A template with record-varying `MERGEFIELD` content in headers,
footers, footnotes, or endnotes is rejected because cloning and remapping those
part-scoped stories would expand this M story materially. Every other package
part and relationship stays byte-preserved. Empty record sets are rejected, and
each non-final record ends with a next-page section boundary.

This story drives flat `MERGEFIELD` records only. It composes the staging
foundations from F-164 but does not implicitly evaluate `{{ }}` or `{% %}` tags.

## Rejected alternatives

- Change the general missing-field fallback. That would break F-161 and F-162.
- Use `append_with_break`. It invents default section properties and does not
  combine record-specific package story updates.
- Add a mode enum and output wrapper. Two concrete methods have simpler return
  types and fewer cases.
- Accept nested JSON records. That creates new merge semantics outside this
  story's flat `MERGEFIELD` contract.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `a_fixture_record_set_produces_separate_and_sectioned_documents` | Exact record-ordered outputs in both modes, with absent fields rendered empty |
| regression | `mail_merge_preserves_switches_and_general_field_policy` | Present values keep field switches, while ordinary updates still keep missing cached text |
| round-trip | `sectioned_mail_merge_preserves_section_properties_and_unmodelled_xml` | Save and reopen keep one next-page section per record, final `sectPr` order, tables, lists, and producer XML |
| regression | `a_failed_record_leaves_the_source_and_outputs_uncommitted` | Invalid record output exposes no partial result and leaves the source bytes unchanged |
| integration | `empty_and_single_record_merges_have_stable_boundaries` | Empty input errors and one record gains no spurious section break |

The **test gate**, from the backlog, is regression. A merge over a fixture
record set produces the expected documents, and an absent field renders empty
rather than failing.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- **Any parser or serialiser**. Read HLD 04 and HLD 06. Save and reopen the
  combined document, verify final `sectPr` schema order and fixed write
  prefixes, and prove unmodelled subtrees stay byte-identical.
- **Public API of a published crate**. Read HLD 10 and the structural rules.
  The two native methods are additive and do not expand Python, WASM, or CLI.
  Run the full package dry-run and assert every archive remains below 10 MiB.

## Hash harness

Expected unchanged across all 49 entries. Both operations are opt-in and no
sample invokes them. Any delta is unexplained and blocks integration.

## Implementation checklist

- [ ] Add the two native facade methods in existing files.
- [ ] Add the merge-local missing-as-empty evaluation policy.
- [ ] Stage and validate every per-record clone before returning output.
- [ ] Assemble body sections from source section properties in record order.
- [ ] Reject record-varying non-body story fields in combined mode.
- [ ] Add the gate, round-trip, atomicity, and boundary tests to existing test files.
- [ ] Run parser, packaging, and unchanged-harness riders.
- [ ] Update exactly HLD 03, HLD 04, HLD 10, and HLD 12.

## Open questions

None. Combined mode varies only main-body stories, uses next-page section
boundaries, rejects an empty record set, and drives flat `MERGEFIELD` values
without evaluating template tags.
