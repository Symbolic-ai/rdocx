# F-164, Loops and conditionals

**Status**: completed
**Sprint**: S50
**Size**: L
**Depends on**: F-163

## Problem

F-163 supplies scalar tag discovery and structured lookup, but substitution
cannot add or remove document structure. The M16 contract requires nested
repetition and conditional inclusion at paragraph, table-row, and section
boundaries. Direct mutation while interpreting tags would leave a partly
generated document when an end marker, path, or nested scope is invalid.

The body model stores paragraphs, tables, content controls, raw XML, and final
section properties in schema order at `crates/rdocx-oxml/src/document.rs:708`.
Rows and paragraphs are cloneable typed values, which permits evaluation into a
staged output sequence without reparsing or losing their preservation
sidecars.

## Spec reference

- `docs/hld/03-architecture.md`, "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, "Package integrity".
- `docs/hld/06-presentationml-model.md`, "Preservation strategy".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy".
- `docs/hld/14-development-backlog.md`, "Milestone 16, Document automation"
  and "F-164, Loops and conditionals".

## Approach

Extend the approved template module and the existing
`Document::render_template` operation with control tags:

```text
{% for item in path.to.array %}
{% endfor %}
{% if path.to.value %}
{% endif %}
```

Markers occupy their own paragraph or table row and are removed from the
result. A stack parser pairs nested blocks before evaluation. Loop bodies are
evaluated once per array element with the loop variable pushed as a lexical
scope. Dotted lookup checks the innermost scope first, then the root. Conditions
use explicit JSON truthiness: false and null are false, zero and empty strings
or collections are false, and other values are true.

Paragraph blocks clone the body entries between marker paragraphs. Row blocks
clone the rows between marker rows inside one table. Section blocks are
top-level paragraph blocks whose staged content includes the section-ending
paragraph and its `sectPr`, so section ownership and order remain unchanged.
Markers may not cross containers or mix paragraph and row boundaries. A
preflight pass validates pairing, scope paths, container boundaries, and all
scalar leaves before replacing the live typed document.

Structural blocks are limited to the main body and its tables. Headers,
footers, text boxes, and chart labels receive scalar rendering only.

The evaluator recurses through nested blocks and applies F-163 scalar rendering
inside each produced clone. The same method and concrete data model remain in
place, so F-164 adds no new public type or dependency.

## Rejected alternatives

- Regex-only block matching is rejected because nested blocks require a stack
  and container-aware boundaries.
- Mutating the live vectors while scanning is rejected because a later syntax
  error must not leave partial output.
- Truthiness via Rust string conversion is rejected because JSON types need a
  stable, documented rule.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `a_nested_loop_and_conditional_generate_the_expected_document` | The F-164 test gate: a readable in-code data model drives a nested loop and conditional and produces the exact ordered paragraph and row text. |
| unit | `mismatched_or_cross_container_blocks_fail_without_mutation` | Missing, extra, crossed, and container-crossing markers are rejected before live state changes. |
| unit | `loop_scopes_shadow_root_values_and_restore_after_exit` | The innermost named item wins during a loop and outer or root lookup resumes afterward. |
| round-trip | `structural_generation_preserves_schema_order_and_raw_xml` | Generated paragraphs, rows, section properties, and preserved XML serialize in valid order and survive reopen. |

The test gate is **regression**. A template with a nested loop and a conditional
produces the expected document from a fixture data model.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- **Any parser or serialiser**. Read
  `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add the structural round-trip test
  above and verify `xsd:sequence` order and byte-for-byte preservation of
  unmodelled subtrees.
- **Public API of a published crate**. The F-163 method gains documented block
  behavior. Read `docs/hld/10-bindings-spec.md` and the `CLAUDE.md` structural
  rules, run the full package dry-run, and assert every `.crate` remains within
  the 10 MiB limit.

## Hash harness

Expected to be unchanged. Structural generation is opt-in and no sample invokes
it.

## Implementation checklist

- [x] Parse and pair nested `for` and `if` blocks with container boundaries.
- [x] Implement lexical loop scopes, dotted lookup, and explicit truthiness.
- [x] Evaluate paragraph, row, and section blocks into staged typed sequences.
- [x] Apply scalar rendering recursively inside produced clones.
- [x] Reject invalid templates atomically and invalidate layout once on success.
- [x] Add the nested regression fixture and structural preservation coverage.
- [x] Update the HLD with marker placement and scope semantics.

## Open questions

None. The consolidated sprint design approval selected the proposed control
syntax, dedicated marker paragraphs and rows, JSON truthiness, lexical lookup,
main-body structural scope, and complete section-ending paragraph semantics.
