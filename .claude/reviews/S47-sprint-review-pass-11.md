# S47 sprint review, pass 11

**Reviewed**: `sprint/s47` at `28dbcef` against `d625bda4`, 54 files,
8,274 changed lines, crates: `rdocx-oxml`, `rdocx`, `rdocx-layout`, and
`rdocx-html`
**Review-bound extension**: Pass 11 continues under the explicit authorization
recorded in pass 4 on 2026-08-17 for as many passes as required to reach a
clean verdict.
**Verdict**: 2 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, rejecting one duplicate property revision deletes the unselected duplicate

`crates/rdocx/src/revision.rs:327`
`crates/rdocx/src/revision.rs:335`
`crates/rdocx/src/revision.rs:339`
`crates/rdocx/src/revision.rs:349`
`crates/rdocx/tests/regression_test.rs:429`

The duplicate repair retains the earlier valid occurrence as raw XML and
reports the latest valid occurrence as the typed revision. The new regression
proves sequential id-scoped acceptance, which removes only the selected change
element. Id-scoped rejection takes a different path. For `pPrChange`,
`rPrChange`, `tblPrChange`, and `sectPrChange`, selecting the latest change for
rejection replaces the complete property owner with that change's prior value.
The earlier unselected duplicate and every interleaved raw sibling disappear
with the owner. For duplicate `numPr/w:ins` markers, rejecting the selected
insertion removes the complete `w:numPr`, including the unselected marker.

This violates F-150's scoped-mutation contract and F-149's exact retention
contract. A fix must reject one selected duplicate without deleting any
unselected valid or raw occurrence, or fail atomically if that result cannot be
represented. Regressions must exercise rejection of the reported latest id for
all five owners, then save, reopen, report the earlier id, and resolve it in a
second scoped operation while retaining every interleaved raw subtree once.

### B2, owner-local bindings are not promoted for foreign retained occurrences

`crates/rdocx-oxml/src/properties.rs:277`
`crates/rdocx-oxml/src/properties.rs:360`
`crates/rdocx-oxml/src/properties.rs:414`
`crates/rdocx-oxml/src/properties.rs:434`
`crates/rdocx-oxml/src/table.rs:552`
`crates/rdocx-oxml/src/table.rs:579`
`crates/rdocx-oxml/src/document.rs:300`
`crates/rdocx-oxml/src/document.rs:324`

The pass-10 repair promotes owner-local declarations for namespace-correct
Word changes before storing them. The foreign same-local-name branches still
capture their child bytes directly. Section properties do the same for every
generic raw child. For example, an interleaved
`<x:pPrChange x:mark="raw"/>` whose only `xmlns:x` declaration is on
`w:pPr` is retained in `revision_xml`, but the writer reconstructs `w:pPr`
without that declaration and emits an unbound `x` prefix. The equivalent hole
exists for `numPr/w:ins`, `tblPrChange`, and `sectPrChange`. Run properties do
pass their generic raw branch through the binding promotion helper, so that
owner is not affected.

This leaves pass-10 B1 open for the raw occurrence in the required
valid, raw, valid sequence and can make an ordinary save namespace-invalid.
Every retained child path must promote the exact external bindings it uses,
not only the path that produces a typed revision. Regressions must declare a
foreign prefix only on each reconstructed owner, interleave that raw child
between two valid changes, and prove expanded-name preservation, source order,
single retention, save, reopen, reporting, and sequential scoped resolution.

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
end-of-milestone gate to S48. For the S47 contribution, all 54 `rdocx`
regression tests and all 10 focused `rdocx-oxml` revision tests pass at the
reviewed SHA. The focused tests establish aligned, absent, and mismatched
public run-property sidecars, owner-local aliases on every valid property
revision, adjacent and interleaved valid changes, save and reopen reporting,
and sequential id-scoped acceptance. The independent hash check reports all
49 entries unchanged. B1 and B2 show that scoped rejection and owner-local raw
bindings remain outside that evidence, so the tracked-change contribution is
still blocked.

## Not found

- `pass-10 B1 direct valid cases`: owner-local Word aliases on `pPrChange`,
  `numPr/w:ins`, `tblPrChange`, and `sectPrChange` are promoted onto both the
  retained and typed valid occurrences. Nested reconstructed `rPr`, `sectPr`,
  table, and paragraph property parse paths thread owner bindings into their
  typed children. Zero additional valid-alias findings beyond B2's raw path.
- `pass-10 B2 ordinary order`: adjacent and foreign-interleaved valid changes
  retain source order across all five owners, survive once, reopen with the
  latest defined occurrence reported, and resolve sequentially through the
  tested acceptance path. Zero additional ordering findings beyond B1's
  rejection path.
- `schema order and programmatic states`: retained run-property children are
  clamped ahead of a present schema-final `w:rPrChange` for aligned, empty, and
  length-mismatched public position sidecars. Zero schema-order findings.
- `reporting and selectors`: modeled revisions report once in tested document
  order. Malformed owners remain opaque. Author, date, shared-id, and nested
  selectors retain their tested counts and inside-out behavior. Zero
  additional selector findings beyond B1.
- `atomicity`: invalid ranges and malformed selected property shapes leave the
  live document and package unchanged in the focused regressions. Zero
  additional atomicity findings.
- `comments, hyperlinks, and content controls`: comment mutation remaps direct,
  control, and hyperlink raw boundaries. Hyperlink namespace collisions,
  targetless spans, nested revisions, and content-control display replacement
  retain their tested order and expanded names. Zero fresh interaction
  findings beyond B1 and B2.
- `duplication`: no second public revision representation or duplicate sprint
  helper was introduced.
- `layering`: no Cargo manifest changed, and no `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency.
- `deps`: no dependency was added.
- `harness`: the independent check reports all 49 entries unchanged, matching
  both S47 AS_BUILT entries.
- `docs and surface`: HLD 03, HLD 04, HLD 10, the two design plans, and
  AS_BUILT describe the intended ownership, atomicity, raw preservation,
  additive native facade, and low-level 0.8.0 compatibility boundary. Python,
  WASM, and CLI surfaces remain unchanged.
- `oracle`: the normalized-body regression remains pinned to Microsoft Word
  16.104 build 16.104.25121423 and compares normalized typed body structure.
- `delivery`: CURRENT_SPRINT, BACKLOG, SPRINT_TRACKER, and AS_BUILT agree on
  both completed stories, estimates, actuals, HLD files, and unchanged harness
  evidence.
