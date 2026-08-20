# F-163, Template syntax

**Status**: approved
**Sprint**: S50
**Size**: L
**Depends on**: none

## Problem

`Document::replace_text` accepts one literal placeholder and replacement and
delegates paragraph work to the cross-run mapper at
`crates/rdocx/src/document.rs:2538` and
`crates/rdocx-oxml/src/placeholder.rs:10`. It does not discover template tags,
resolve paths from structured data, reject malformed syntax, or provide one
atomic render operation. Word commonly splits a tag across several formatted
runs, so scanning each run independently would reproduce the exact failure the
existing mapper was written to prevent.

The document facade already owns the typed body and package state. Template
evaluation must use that ownership boundary, preserve the formatting of the
first matched run and the unmatched suffix, leave unmodelled XML in place, and
invalidate layout only after a successful render.

## Spec reference

- `docs/hld/03-architecture.md`, "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, "Package integrity".
- `docs/hld/06-presentationml-model.md`, "Preservation strategy", as the
  workspace rule for typed mutation beside preserved XML.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy".
- `docs/hld/14-development-backlog.md`, "Milestone 16, Document automation"
  and "F-163, Template syntax".

## Approach

Add one focused `rdocx::template` module and re-export no low-level OOXML
types. Add `serde_json` to the published `rdocx` crate and expose this additive
native method:

```rust
pub fn render_template(&mut self, data: &serde_json::Value) -> Result<usize>;
```

F-163 implements scalar tags. The proposed syntax is `{{ path.to.value }}`.
Paths traverse JSON objects by dotted component. String, number, and boolean
leaves render as text, while `null` renders as an empty string. Arrays and
objects are invalid scalar values. Missing paths, malformed tags, and invalid
scalar values return an error before any live document state changes.

The scanner concatenates paragraph text across ordinary runs and identifies
complete scalar tags with byte-safe offsets. Rendering stages a cloned typed
document, resolves every tag first, then applies replacements through the
existing cross-run placeholder mapper. A successful render replaces the live
typed document once, invalidates both layout caches once, and returns the
number of scalar tags replaced. The unchanged save path remains responsible
for package serialization.

Template control tags introduced by F-164 are recognized as reserved syntax
and are not treated as scalar paths. Scalar rendering covers every location
that `replace_text` covers today, including the main body, headers, footers,
text boxes, and chart labels. F-163 does not add binding methods. Python, WASM,
and CLI consumers continue to preserve a document rendered through the native
facade.

## Rejected alternatives

- Run-by-run scanning is rejected because Word can split one tag across any
  number of formatted runs.
- A new template trait is rejected because only one JSON data model exists
  today.
- A second document tree is rejected because the facade already owns the typed
  WordprocessingML tree.
- Expanding `document.rs` with the parser and evaluator is rejected because the
  file already combines the complete document facade and would become harder
  to reason about locally.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `a_tag_split_across_five_formatted_runs_preserves_surrounding_formatting` | The F-163 test gate: one scalar tag split across five differently formatted runs resolves once, the first matched run supplies replacement formatting, and unmatched prefix and suffix formatting remain unchanged. |
| unit | `dotted_scalar_paths_render_supported_json_leaves` | String, number, boolean, and null values render with the documented conversions. |
| unit | `invalid_template_input_leaves_the_document_unchanged` | Missing paths, malformed tags, and object or array scalar values return errors without changing typed XML or layout-visible text. |
| round-trip | `template_render_preserves_unmodelled_paragraph_xml` | A readable in-code paragraph containing producer XML is rendered, saved, reopened, and retains the unmodelled subtree byte for byte. |

The test gate is **unit**. A tag split across five runs with different
formatting resolves, and the surrounding formatting is preserved.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- **Any parser or serialiser**. Read
  `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add the round-trip preservation test
  above and verify schema child order and unmodelled subtree retention.
- **Public API of a published crate**. Read
  `docs/hld/10-bindings-spec.md` and the `CLAUDE.md` structural rules. State
  that the method is additive for the pre-1.0 native facade, run the full
  package dry-run, and assert that every generated `.crate` stays within the
  10 MiB limit.
- **A new module or file**. Read the `CLAUDE.md` structural rules. Obtain
  explicit approval for `crates/rdocx/src/template.rs` before implementation.
  The module keeps syntax parsing and evaluation in one behavior-bearing file.

## Hash harness

Expected to be unchanged. Template rendering is opt-in and no sample invokes
it.

## Implementation checklist

- [ ] Add the approved template module and the `serde_json` dependency.
- [ ] Implement byte-safe scalar tag discovery and dotted JSON lookup.
- [ ] Stage and validate all scalar replacements before mutating the document.
- [ ] Reuse the cross-run replacement mapper and invalidate layout once.
- [ ] Add the unit and round-trip tests, including the five-run formatting gate.
- [ ] Document the native-only additive API and unchanged binding surfaces.

## Open questions

None. The consolidated sprint design approval selected the proposed scalar
syntax and value rules, `serde_json::Value`, the additive facade method, the
focused template module, and scalar coverage matching `replace_text`.
