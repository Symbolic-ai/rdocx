# S47 sprint review, pass 1

**Reviewed**: `sprint/s47` at `92453d3` against `d625bda4`, 38 files,
4,184 changed lines, crates: `rdocx-oxml`, `rdocx`, `rdocx-layout`, and
`rdocx-html`
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, reachable run-content revisions are dropped or left unmodelled

`crates/rdocx-oxml/src/text.rs:842`
`crates/rdocx-oxml/src/revision.rs:389`
`crates/rdocx/src/revision.rs:764`
`crates/rdocx/src/revision.rs:792`

The paragraph hyperlink parser accepts only direct `w:r` children and skips
every other element subtree. A valid `w:ins`, `w:del`, `w:moveFrom`, or
`w:moveTo` inside `w:hyperlink` is therefore discarded during load. It cannot
round-trip or appear in `Document::revisions`. The low-level revision
projection has the same direct-run boundary, and the resolver models only
`w:r` below hyperlinks and content-revision wrappers. A content revision
nested inside another selected content revision is therefore invisible to the
required inside-out selection, validation, and count.

This breaks the F-149 round-trip gate and the F-150 nested-resolution contract
at legal run-content placements. The fix must preserve and project the four
content wrapper kinds throughout the approved run-content grammar, include
them once in document order, teach the resolver the same placement paths, and
add exact round-trip plus scoped inside-out resolution regressions for a
hyperlink wrapper and a content wrapper nested in another wrapper.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M14 gate is: "a document carrying tracked changes, comments, content
controls and bookmarks round-trips byte-identically in the parts this
milestone does not model, and every one of the four is readable and writable
through the public API."

S47 does not establish that gate. The sprint plan assigns the end-of-milestone
gate to S48. Within S47, `revision_elements_round_trip_unchanged_and_report_metadata`
and `accepting_every_revision_matches_word_normalized_body_xml` provide evidence
for the direct placements they exercise, including the pinned Microsoft Word
16.104 build 16.104.25121423 normalized-body oracle. B1 shows that the claimed
tracked-change coverage does not yet extend to all approved run-content
placements, so the S47 contribution to the milestone gate remains blocked.

## Not found

- `duplication`: no duplicate sprint helper or second revision representation
  was introduced.
- `layering`: no Cargo manifest changed, and no `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency.
- `harness`: no undeclared delta was found. Integrated full verification
  records the harness unchanged, and both AS_BUILT entries record 49 of 49.
- `docs`: the design-plan HLD impact lists match updates to HLD 03, HLD 04,
  and HLD 10. No additional documentation finding was found beyond B1's
  implementation contradiction.
- `deps`: no dependency was added.
- `surface`: `RevisionKind`, `RevisionRef`, `Document::revisions`, and the
  eight accept and reject methods are called for by F-149 and F-150. No
  Python, WASM, or CLI revision surface was added.
- `delivery`: CURRENT_SPRINT, BACKLOG, SPRINT_TRACKER, and AS_BUILT agree that
  F-149 and F-150 are complete, with matching estimates, actuals, HLD files,
  and unchanged-harness evidence.

