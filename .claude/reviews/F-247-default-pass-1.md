# F-247, default, pass 1

**Reviewed**: uncommitted working tree diff, 17 files, 3,473 changed lines
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, Modelled level leaves discard producer extensions
`crates/rdocx-oxml/src/numbering.rs:2654`
`crates/rdocx-oxml/src/numbering.rs:2902`

The start, number-format, suffix, level-text, and justification branches read
only `w:val`. A nonempty occurrence is then skipped with `read_to_end_into`, and
an empty occurrence is never captured. The serializer always emits a new
value-only element. An imported standard leaf such as `w:start` with a producer
attribute or nested producer child therefore loses that payload on an unchanged
open and save. This contradicts the design's byte-identical extension-retention
contract and the HLD preservation claim.

### D2, Whole-graph validation does not validate live paragraph levels
`crates/rdocx/src/document.rs:8886`
`crates/rdocx/src/document.rs:8921`
`crates/rdocx/src/document.rs:9007`

`validate_numbering_graph` validates only definitions and instances. It never
walks live paragraph numbering references. For example, a paragraph can use
instance N at level 2, then `update_numbering_instance` can move N to a
definition containing only level 0. The staged candidate passes validation and
is committed, leaving the paragraph pointed at a level its definition does not
own. This violates the approved contract that live paragraph references are
validated before publication.

### D3, Fresh facade updates can commit state that numbering serialization rejects
`crates/rdocx/src/document.rs:13852`
`crates/rdocx/src/document.rs:8775`
`crates/rdocx-oxml/src/numbering.rs:2871`

When a caller supplies a fresh `ListLevel` instead of the exact inspected
projection, the fallback merge copies every retained level attribute onto the
new typed level without checking conflicts. An imported malformed `w:tplc` or
`w:tentative` is retained as an extra attribute. Adding the corresponding valid
typed property through the fresh value then passes graph validation and the
document commits the candidate. A later save fails in the serializer because
the typed and retained attributes conflict. The mutation API must reject this
before publication and preserve the original document atomically.

### D4, Definition CRUD cannot round-trip valid sparse level identifiers
`crates/rdocx/src/document.rs:13474`
`crates/rdocx/src/document.rs:8751`
`crates/rdocx/src/document.rs:13888`

The owned definition projection exposes levels only by vector position and
does not expose each `w:ilvl`. Update validation nevertheless treats that vector
position as the level identifier. A valid imported definition containing only
level 2, for example with level text `%3.` or a restart after level 0, is
projected as a one-element vector and revalidated as level 0. Passing an
unchanged inspected definition back to `update_numbering_definition` therefore
fails, and callers cannot discover the identifier needed to reconstruct the
level. The new inspect and update facade is not complete for imported sparse
definitions.

## Smells

None.

## Nitpicks

None.

## Prior findings rechecked

- Producer-defined format tokens now remain typed through the public projection
  at `crates/rdocx/src/document.rs:13646`.
- Inspected levels retain a source snapshot and an unchanged update preserves
  omitted properties at `crates/rdocx/src/document.rs:13554` and
  `crates/rdocx/src/document.rs:13804`.
- Typed level-attribute conflicts now fail closed in the serializer at
  `crates/rdocx-oxml/src/numbering.rs:2871`. D3 identifies the remaining facade
  path around that check.
- Extended abstract-number references retain raw XML at
  `crates/rdocx-oxml/src/numbering.rs:3567` and replay it at
  `crates/rdocx-oxml/src/numbering.rs:3655`.
- Override raw-replay equality now includes the original `w:ilvl` at
  `crates/rdocx-oxml/src/numbering.rs:3488`.
- Unmodelled diagnostics now inspect retained typed-leaf payloads at
  `crates/rdocx/src/document.rs:13662` and
  `crates/rdocx/src/document.rs:13683`.

## Checks run

- `git diff --check`, passed.
- `cargo fmt --all --check`, passed.
- `cargo check -p rdocx --all-targets`, passed with an isolated target directory.
- `cargo test -p rdocx numbering --lib`, 23 passed.

## Not found

No independent panics, structure, or nitpick findings were found. Correctness,
contract, OOXML, and test-gate issues are covered by D1 through D4.
