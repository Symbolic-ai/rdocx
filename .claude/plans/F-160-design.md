# F-160, Field instruction parser

**Status**: approved
**Sprint**: S49
**Size**: L
**Depends on**: none

## Problem

`crates/rdocx-oxml/src/text.rs` parses `w:fldSimple` into the narrow
`FieldType` projection, but complex `w:fldChar` fields have only a separate
hyperlink-oriented parser. The latter rejects malformed, dirty, and nested
fields from `CT_P::from_xml`, hardcodes the `w:` prefix, and does not return a
general field name, arguments, switches, and cached result.

The merged reader changes must instead preserve unsupported complex sequences
as raw XML while reporting safe, recursively structured fields. Prefix aliases
and the default WordprocessingML namespace must be recognized without treating
foreign same-local-name XML as WordprocessingML.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-160, Field instruction parser".
- `docs/hld/03-architecture.md`, "What stays put" and "Crate-level
  conventions".
- `docs/hld/04-opc-and-packaging.md`, "Facade conventions" and the Word
  package-preservation rules.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and in-code fixtures.

## Approach

Replace the complex-field hyperlink scanner with one recursive parser in
`rdocx-oxml::text`. The parser records field-marker ownership while each run is
read with its in-scope WordprocessingML bindings. This keeps the original raw
run children unchanged and allows complex-field recognition to distinguish
WordprocessingML aliases from foreign collisions after paragraph parsing.

Expose a concrete parsed field projection with the normalized field name,
arguments, switches, cached-result run range, dirty state, and child fields.
`CT_P` reports complete structured fields, including dirty fields, while
unsupported, malformed, or unclosed sequences remain opaque and never make the
paragraph or document fail to open. The original raw XML remains the
serialization source in every case. `Document::links()` excludes dirty fields
until F-162 defines the update policy.

Use the same instruction tokenizer for `w:fldSimple` and complex fields. It
keeps quoted arguments intact, identifies backslash-prefixed switches, and
preserves the original instruction for compatibility with the existing
`FieldType` writer and layout paths. The existing `Document::links()` method
derives HYPERLINK links only from a valid parsed field and its cached result.

## Rejected alternatives

- Keep the hyperlink-only parser and merely make its failures non-fatal. It
  cannot meet F-160's field-name, argument, switch, or nested-field contract.
- Reparse preserved raw XML after paragraph parsing without namespace context.
  Inherited aliases and default namespaces then become ambiguous, while foreign
  collisions can be misclassified.
- Materialize fields as synthetic runs. That would alter producer XML and lose
  the exact run boundaries required for round-trip preservation.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit, gate | `complex_fields_parse_nested_operands_and_split_instructions` | Nested field operands and instructions split across runs produce a recursive projection with the expected names, arguments, and switches |
| unit | `simple_and_complex_fields_share_the_instruction_tokenizer` | Quoted arguments and switches have the same parsed representation for both OOXML field forms |
| unit | `complex_fields_accept_aliases_and_default_word_namespaces` | Aliased and default WordprocessingML field markers are parsed |
| regression | `foreign_field_marker_collisions_remain_opaque` | Same-local-name foreign elements do not produce a field projection or alter the raw XML |
| round-trip | `unsupported_complex_fields_remain_raw_without_failing_document_open` | Dirty, malformed, and unclosed sequences open, preserve their bytes, and report no projection |
| regression | `links_uses_a_valid_complex_hyperlink_cached_result` | A valid HYPERLINK reports its target and cached visible text, while an invalid field reports no link |

The **test gate**, from the backlog, is unit. Every field form in the corpus
parses, including nested fields and instructions split across runs.

Fixtures are assembled in existing test modules. No new test binary or binary
fixture is added.

## HLD impact

- `docs/hld/03-architecture.md`

Record the recursive field projection, in-scope namespace identity, and opaque
fallback for unsupported field sequences.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Add alias,
  default-namespace, foreign-collision, fixed-prefix, nested, and byte-preserving
  round-trip coverage.
- Public API of a published crate. Read HLD 10 and the structural rules. The
  parsed-field accessor is additive and required by F-160. Run the package
  dry-run and archive-size assertion during full verification.

## Hash harness

Expected unchanged. Parsed documents retain the original field XML as their
serialization source, and the existing samples do not introduce a field
projection mutation.

## Implementation checklist

- [ ] Define the additive recursive field projection and instruction token
  types in `rdocx-oxml::text`.
- [ ] Record namespace-aware complex-field markers while parsing runs without
  changing preserved raw XML.
- [ ] Parse complete complex fields recursively, retain dirty state, and
  degrade malformed, unsupported, or unclosed sequences to opaque XML.
- [ ] Route simple fields through the shared instruction tokenizer without
  changing their existing writer and layout behavior.
- [ ] Derive public HYPERLINK links from valid complex-field projections and
  cached result ranges.
- [ ] Add the unit, regression, and byte-preservation coverage in the test
  plan.
- [ ] Run the parser, facade, layout, package, and hash-harness checks.
- [ ] Update the listed HLD architecture section at completion.

## Open questions

None. The parser reports complete structured fields and dirty state, retains
original XML for every other sequence, and leaves evaluation and update policy
to F-161 and F-162.
