# F-160, Field instruction parser

**Status**: completed
**Sprint**: S49
**Size**: L
**Depends on**: none

## Problem

The Word text model recognises only `PAGE`, `NUMPAGES`, `REF`, and `PAGEREF`,
with every other instruction held as an opaque string
(`crates/rdocx-oxml/src/text.rs:31`). `w:fldSimple` parsing flattens child runs
to one display string (`crates/rdocx-oxml/src/text.rs:1131`), while the current
instruction parser is a whitespace split that cannot retain quoted arguments,
switch arguments, or nested fields (`crates/rdocx-oxml/src/text.rs:2223`).
Complex `w:fldChar` and `w:instrText` sequences are not modelled at all.

The serializer also canonicalises every modelled field as `w:fldSimple` and
reconstructs PAGE and NUMPAGES instructions without their original switches
(`crates/rdocx-oxml/src/text.rs:1372`). F-160 needs one grammar for simple and
complex fields without losing the source form, stored display, run boundaries,
or producer XML that later stories do not interpret.

## Spec reference

- `docs/hld/14-development-backlog.md`, "Milestone 16, Document automation"
  and "F-160, Field instruction parser".
- `docs/hld/03-architecture.md`, "What stays put" and its Word text model
  ownership rules.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability" and the
  intentional pre-1.0 low-level break recorded for the 0.8 family.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The hash harness".

## Approach

Replace the narrow `FieldType` projection with a recursive field model:

```rust
pub struct Field {
    pub instruction: FieldInstruction,
    pub cached_result: String,
    pub dirty: Option<bool>,
    source: FieldSource,
}

pub struct FieldInstruction {
    pub raw: String,
    pub name: String,
    pub arguments: Vec<FieldArgument>,
    pub switches: Vec<FieldSwitch>,
}

pub enum FieldArgument {
    Text(String),
    Nested(Box<Field>),
}

pub struct FieldSwitch {
    pub name: String,
    pub argument: Option<FieldArgument>,
}
```

`Box<Field>` is required only to make the recursive concrete AST finite. It is
not dynamic dispatch. `FieldSource` remains private and retains whether a field
was simple or complex plus the original complex run partition and raw slots.
Unchanged fields therefore write their original form and boundaries. F-162
will own dirty-flag mutation, but the flag is placed in the model now to avoid
a second breaking public shape change.

Implement a hand-written lexer for quoted and escaped arguments, field-specific
and general switches, and nested field operands. A paragraph-level stack pairs
complex begin, separate, and end markers, concatenates instruction text across
runs, and parses it through the same lexer as `w:fldSimple`. Malformed,
misplaced, or unbalanced sequences remain preserved raw content and do not
become typed fields. Existing PAGE, NUMPAGES, REF, and PAGEREF layout behaviour
continues through a private compatibility classifier over the new instruction.

Keep the grammar, AST, and XML integration in `text.rs`. The approved new module
belongs only to the facade evaluator in F-161, so F-160 adds no file.

## Rejected alternatives

- Keep using `split_whitespace`. It cannot represent quoted, split-run, or
  nested instructions required by the gate.
- Add one `FieldType` variant per evaluated field. That mixes F-160 grammar
  ownership with F-161 evaluation semantics.
- Canonicalise complex fields as `w:fldSimple`. That loses run formatting,
  source form, and unchanged producer structure.
- Keep both `FieldType` and a second recursive AST. Two public models of the
  same field would diverge and make F-162 mutation ambiguous.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit, gate | `field_instruction_corpus_parses_every_simple_complex_split_and_nested_form` | A readable in-code matrix covers every S49 field name, quoted and escaped arguments, comparison operands, `\\*`, `\\#`, `\\@`, field-specific switches, split `instrText`, and nested fields |
| unit | `malformed_complex_fields_remain_untyped_and_preserved` | Misplaced and unbalanced markers never invent a field or hide the literal stored result |
| round-trip | `unchanged_complex_fields_keep_source_runs_and_unmodelled_neighbours` | Prefix aliases read, fixed prefixes write after mutation, and unchanged complex run partitions plus unmodelled XML survive byte for byte |
| regression | existing PAGE, NUMPAGES, REF, and PAGEREF tests | Existing field rendering and complete instruction retention do not regress |

The **test gate**, from the backlog, is unit. Every field form in the readable
in-code corpus parses, including nested fields and instructions split across
runs. No binary fixture or new integration test binary is added.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`

Record the shared simple and complex recursive grammar, source-form and stored
result preservation, the low-level 0.8 source break, and the unchanged native
facade and binding surfaces.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Check prefix-tolerant read,
  fixed-prefix mutation output, schema child order, malformed fallback, and a
  round trip proving every unmodelled subtree stays byte-identical.
- Public API of a published crate. Read HLD 10 and the structural rules. The
  `FieldType` replacement is the already-declared pre-1.0 low-level break for
  the 0.8 family. Run the full package dry-run and assert every `.crate` remains
  below 10 MiB.
- Layout, pagination, line breaking, and text shaping if compatibility changes
  reach `rdocx-layout`. Run render evidence with bundled deterministic fonts
  and do not record a system-font baseline.
- A new module or file only if the consolidated module approval is granted.
  No trait, generic parameter, crate, or feature flag is introduced.

## Hash harness

Expected unchanged across all current entries. Parser expansion must retain
the current serialized and rendered output for existing fields. Any delta is
unexpected and blocks integration.

## Implementation checklist

- [x] Define the recursive field, instruction, argument, switch, source, and dirty models.
- [x] Replace the narrow `FieldType` projection and adapt exhaustive matches.
- [x] Implement quoted, escaped, switch-aware instruction lexing.
- [x] Parse `w:fldSimple` through the shared grammar.
- [x] Parse nested complex fields and split `w:instrText` with a paragraph stack.
- [x] Preserve malformed sequences, source form, run partitions, and raw neighbours.
- [x] Preserve existing PAGE, NUMPAGES, REF, and PAGEREF layout behaviour.
- [x] Add the in-code corpus gate plus malformed, prefix, order, and round-trip tests.
- [x] Run focused checks, package riders, and the unchanged hash harness.
- [x] Update exactly HLD 03 and HLD 10 at completion.

## Open questions

None. The approved gate is the readable in-code matrix described above, the
unified low-level field AST replaces `FieldType`, and F-160 remains in
`text.rs`.
