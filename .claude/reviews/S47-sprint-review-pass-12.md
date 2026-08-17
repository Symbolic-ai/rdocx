# S47 sprint review, pass 12

**Reviewed**: `sprint/s47` at `d0c4462` against `d625bda4`, 55 files,
8,686 changed lines, crates: `rdocx-oxml`, `rdocx`, `rdocx-layout`, and
`rdocx-html`
**Review-bound extension**: Pass 12 continues under the explicit authorization
recorded in pass 4 on 2026-08-17 for as many passes as required to reach a
clean verdict.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

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
end-of-milestone gate to S48. For the S47 contribution, the complete
`rdocx-oxml` suite passes with 210 unit tests and one documentation test. The
complete `rdocx` suite passes with 90 unit tests, 87 integration tests, 54
regression tests, and two documentation tests. Two tests requiring manual Word
evidence remain ignored outside the S47 revision gate. The independent hash
check reports all 49 entries unchanged, and the prose gate reports zero
violations.

The pinned `accepting_every_revision_matches_word_normalized_body_xml`
regression passes against Microsoft Word 16.104 build 16.104.25121423. This
establishes the F-150 normalized-body gate at the reviewed SHA. The complete
M14 mixed-document gate remains correctly deferred to S48.

## Not found

- `pass-11 B1`: `crates/rdocx/tests/regression_test.rs:405` covers adjacent
  valid duplicates and interleaved raw siblings across `pPr`, `rPr`, `tblPr`,
  `sectPr`, and `numPr`. Id-scoped rejection of each reported latest identity
  retains the earlier identity and every raw sibling once. Save and reopen
  report the earlier identities, a second scoped rejection resolves them, and
  `reject_all` resolves all ten modeled occurrences while retaining each raw
  sibling once. The resolver retains unselected revision children at
  `crates/rdocx/src/revision.rs:586` and uses the earliest selected prior value
  for the all-revision case at `crates/rdocx/src/revision.rs:327`.
- `pass-11 B2`: `crates/rdocx-oxml/src/revision.rs:879` covers owner-local Word
  aliases and foreign same-local aliases for paragraph, numbering, table, and
  section changes. It proves exact single retention, valid and foreign binding
  promotion, valid, raw, valid source order, save, reopen, reporting, and a
  second serialization. The pre-existing run-property case covers the same
  owner-local preservation path at `crates/rdocx-oxml/src/revision.rs:770`.
- `interaction`: duplicate preservation, property rejection, namespace
  recovery, comments, hyperlinks, content controls, contextual markers, and
  nested inside-out resolution pass together in the complete affected crate
  suites. Zero interaction findings.
- `schema order`: property changes remain in their schema-final slots, earlier
  duplicate changes remain ahead of the typed final occurrence, and foreign
  interleaved occurrences retain their relative order. Zero ordering findings.
- `atomicity`: malformed selected property shapes and invalid date ranges
  retain document bytes and live state in the focused regressions. Zero
  atomicity findings.
- `reporting and selectors`: recursive reporting, exact author matching,
  inclusive instant ranges, shared ids, malformed opaque revisions, and
  inside-out nested resolution retain their tested counts and order. Zero
  selector findings.
- `duplication`: no second public revision representation or duplicate sprint
  helper was introduced.
- `layering`: no Cargo manifest changed, and no `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency.
- `deps`: no dependency was added.
- `harness`: all 49 entries are unchanged, matching both S47 AS_BUILT entries.
- `docs and surface`: HLD 03, HLD 04, HLD 10, the two design plans, and
  AS_BUILT describe the implemented ownership, atomic staging, raw
  preservation, additive native facade, and low-level 0.8.0 compatibility
  boundary. Python, WASM, and CLI surfaces remain unchanged.
- `oracle`: the normalized-body fixture names the exact Word build and compares
  normalized typed body structure rather than serialized bytes.
- `delivery`: CURRENT_SPRINT, BACKLOG, SPRINT_TRACKER, and AS_BUILT agree on
  both completed stories, estimates, actuals, HLD files, and unchanged harness
  evidence.
