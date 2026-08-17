# S47 sprint review, pass 10

**Reviewed**: `sprint/s47` at `0ebe3e3` against `d625bda4`, 52 files,
7,931 changed lines, crates: `rdocx-oxml`, `rdocx`, `rdocx-layout`, and
`rdocx-html`
**Review-bound extension**: Pass 10 continues under the explicit authorization
recorded in pass 4 on 2026-08-17 for as many passes as required to reach a
clean verdict.
**Verdict**: 2 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, owner-local namespace bindings still disappear outside run properties

`crates/rdocx-oxml/src/properties.rs:231`
`crates/rdocx-oxml/src/properties.rs:356`
`crates/rdocx-oxml/src/properties.rs:545`
`crates/rdocx-oxml/src/table.rs:529`
`crates/rdocx-oxml/src/table.rs:626`
`crates/rdocx-oxml/src/document.rs:277`
`crates/rdocx-oxml/src/document.rs:522`

The pass-9 repair promotes declarations that exist only on `w:rPr` onto the
retained child roots that use them. The equivalent paragraph, numbering,
table, and section paths still capture only child bytes and later reconstruct
fixed `w:pPr`, `w:numPr`, `w:tblPr`, and `w:sectPr` owners. A valid aliased
change such as `wa:pPrChange` can therefore be projected while `wa` is in the
parser's inherited scope, then written as an unbound prefix after the only
`xmlns:wa` declaration on its owner is discarded. The same defect applies to
`numPr/w:ins`, `tblPrChange`, and `sectPrChange`, including duplicate changes
that move from the typed field into a raw vector.

This violates the F-149 round-trip gate and can make an ordinary save produce
namespace-invalid XML. The owner-binding promotion must cover every property
owner that reconstructs its start element, with save and reopen regressions
for single and duplicate aliased changes whose declarations exist only on the
property owner.

### B2, an intervening raw change is moved ahead of an earlier valid duplicate

`crates/rdocx-oxml/src/properties.rs:234`
`crates/rdocx-oxml/src/properties.rs:237`
`crates/rdocx-oxml/src/properties.rs:359`
`crates/rdocx-oxml/src/table.rs:532`
`crates/rdocx-oxml/src/document.rs:280`
`crates/rdocx-oxml/src/revision.rs:795`

The duplicate repair keeps the latest valid change in the typed field and
appends the displaced earlier change to the same raw vector that already holds
malformed or foreign siblings. For the input sequence `valid-1`, `malformed`,
`valid-2`, parsing first appends `malformed`, then appends the displaced
`valid-1`. Serialization therefore produces `malformed`, `valid-1`,
`valid-2`. Run properties use the same append order for equal change-slot
positions. Numbering, paragraph, table, and section properties all have the
same state transition.

The new duplicate tests cover adjacent valid pairs, so they prove exact single
retention and scoped resolution only when no raw sibling intervenes. The fix
must retain source order across valid and unprojected occurrences for all five
owners, then prove save, reopen, reporting, and sequential id-scoped resolution
with a distinct malformed or foreign change between two valid identities.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M14 gate is: "a document carrying tracked changes, comments, content
controls and bookmarks round-trips byte-identically in the parts this
milestone does not model, and every one of the four is readable and writable
through the public API."

S47 does not establish the complete gate because the sprint plan assigns the
end-of-milestone gate to S48. For the S47 contribution, all 9 focused
`rdocx-oxml` revision tests pass at the reviewed SHA, including the new
programmatic sidecar, owner-local run-property namespace, and adjacent
duplicate cases. The independent hash check reports all 49 entries unchanged.
B1 and B2 show that the same namespace and ordered-preservation contracts do
not yet hold across the other property owners and interleaved duplicate input,
so the tracked-change contribution remains blocked.

## Not found

- `pass-9 B1`: the positioned writer clamps every aligned raw position at or
  beyond the change slot ahead of a present `w:rPrChange`. The fallback writer
  also keeps raw children ahead of the change when the sidecar is empty or its
  length is mismatched. The direct regression exercises all three states.
- `pass-9 B2`: retained foreign, aliased non-empty Word, and revision children
  now receive the exact required declarations from a reconstructed `w:rPr`,
  and the focused save and reopen regression passes. B1 is the corresponding
  uncovered behavior for the other reconstructed property owners.
- `pass-9 B3`: adjacent valid duplicate changes for run, paragraph, table,
  section, and numbering properties survive once, expose one typed identity,
  and resolve sequentially by id. B2 is limited to a raw occurrence
  interleaved between valid duplicates.
- `interaction`: outside B1 and B2, no additional conflict was found between
  revision projection, scoped resolution, comments, hyperlinks, content
  controls, and run-content mutation.
- `duplication`: no second public revision representation or duplicate sprint
  helper was introduced.
- `layering`: no Cargo manifest changed, and no `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency.
- `harness`: the independent check reports all 49 entries unchanged, matching
  both S47 AS_BUILT entries.
- `docs`: HLD 03, HLD 04, HLD 10, and the two design plans describe the intended
  ownership, atomicity, preservation, and native API boundaries. No separate
  documentation gap was found.
- `deps`: no dependency was added.
- `surface`: the low-level 0.8.0 compatibility boundary is recorded, the
  `rdocx::Document` facade remains additive, and Python, WASM, and CLI surfaces
  remain unchanged.
- `oracle`: the normalized-body regression remains pinned to Microsoft Word
  16.104 build 16.104.25121423 and compares normalized typed body structure.
- `delivery`: CURRENT_SPRINT, BACKLOG, SPRINT_TRACKER, and AS_BUILT agree on
  both completed stories, estimates, actuals, HLD files, and unchanged harness
  evidence.
