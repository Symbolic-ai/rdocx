# F-247, default, pass 2

**Reviewed**: uncommitted working tree diff, 20 files, 4,302 changed lines
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, Live-reference validation is limited to direct main-body numbering
`crates/rdocx/src/document.rs:8966`
`crates/rdocx/src/document.rs:9385`
`crates/rdocx/src/document.rs:9102`

The pass-1 fix collects only direct `num_id` and `num_ilvl` values from typed
main-body paragraphs. A paragraph whose effective numbering comes from its
paragraph style has no direct `num_id`, even though the public effective-property
resolver applies the style's numbering. Imported headers, footers, notes, and
other related stories are outside `self.document.body` as well. Moving the live
instance to a definition that lacks the inherited or related-story level
therefore still passes `validate_numbering_graph` and commits a dangling level
reference. The existing removal check already scans every package XML part, but
the whole-graph validator does not use that scope. Pass-1 D2 is only partially
fixed.

### D2, Modelled multi-level-type metadata still drops producer payload
`crates/rdocx-oxml/src/numbering.rs:3279`
`crates/rdocx-oxml/src/numbering.rs:3396`

`w:multiLevelType` remains the one modelled abstract-definition scalar that
reads only `w:val`. Its nonempty parser skips the subtree, its empty parser does
not capture attributes, and its writer always emits a new value-only element.
An imported `w:multiLevelType` carrying an extension attribute or child loses
that payload on an unchanged open and save. This violates the definition-level
producer-extension preservation contract. The pass-1 scalar-leaf fix covers
`CT_Lvl`, but not this `CT_AbstractNum` leaf.

### D3, RTF export converts newly typed formats and none to decimal
`crates/rdocx/src/rtf.rs:1237`
`crates/rdocx/src/rtf.rs:1478`
`crates/rdocx/src/rtf.rs:915`

The RTF writer first filters an OOXML format through `public_number_format`,
which still recognizes only the seven legacy variants. Every newly typed
standard format, including `ListNumberFormat::None`, becomes `None` and then
uses the decimal fallback when the list table is written. For example, a
public-authored `none` level emits `levelnfc0` instead of `levelnfc255`, so an
RTF consumer invents a decimal marker. The exhaustive numeric branches added
to `list_format_value` cannot be reached for these document formats. This
contradicts the stated no-invented-marker behavior and loses supported format
identity during export.

### D4, Public ListLevel equality includes hidden preservation provenance
`crates/rdocx/src/document.rs:13387`
`crates/rdocx/src/document.rs:13404`

`ListLevel` still derives `PartialEq`, but its new private `source_level`
snapshot participates in that equality. Two values with identical public
numbering properties compare unequal when one came from inspection and the
other was constructed by the caller, or when their hidden raw OOXML differs.
That also contaminates the derived equality of `NumberingDefinitionLevel` and
`NumberingDefinition`. Preservation provenance is not a public level property
and must not silently change the established semantic equality behavior. The
existing `CT_TabStop` provenance model explicitly excludes its source marker
from equality at `crates/rdocx-oxml/src/borders.rs:250`.

## Smells

None.

## Nitpicks

None.

## Pass-1 findings rechecked

- D1 is fixed for `CT_Lvl` scalar leaves. Parsing now captures raw payload at
  `crates/rdocx-oxml/src/numbering.rs:2680`, and serialization replays or
  overlays it at `crates/rdocx-oxml/src/numbering.rs:2987`.
- D2 is partially fixed for direct main-body references at
  `crates/rdocx/src/document.rs:8966`. D1 records the remaining scopes.
- D3 is fixed by facade graph preflight at
  `crates/rdocx/src/document.rs:14035`, with the atomic regression at
  `crates/rdocx/src/document.rs:21709`.
- D4 is fixed by explicit `NumberingDefinitionLevel` identities at
  `crates/rdocx/src/document.rs:13576` and identifier-based update matching at
  `crates/rdocx/src/document.rs:8803`.

## Earlier findings rechecked

- Producer-defined format tokens remain typed at
  `crates/rdocx/src/document.rs:13744`.
- Inspected levels retain source state for omission-preserving updates at
  `crates/rdocx/src/document.rs:13652` and
  `crates/rdocx/src/document.rs:13923`.
- Invalid typed level-attribute aliases fail closed during graph validation at
  `crates/rdocx/src/document.rs:14035`.
- Extended abstract-number references retain and replay raw XML at
  `crates/rdocx-oxml/src/numbering.rs:3714` and
  `crates/rdocx-oxml/src/numbering.rs:3790`.
- Override unchanged detection includes the original level identifier at
  `crates/rdocx-oxml/src/numbering.rs:3622`.
- Unmodelled diagnostics inspect every retained level scalar at
  `crates/rdocx/src/document.rs:13761` and retained override scalar at
  `crates/rdocx/src/document.rs:13802`.

## Checks run

- `git diff --check`, passed.
- `cargo fmt --all --check`, passed.
- `cargo test -p rdocx numbering --lib`, 25 passed.
- `cargo test -p rdocx-oxml numbering --lib`, 62 passed.

## Not found

No independent panic, structure, smell, or nitpick findings were found. The
correctness, contract, OOXML, and test-gate findings are D1 through D4.
