# S47 sprint review, pass 2

**Reviewed**: `sprint/s47` at `55d9ad5` against `d625bda4`, 39 files,
4,578 changed lines, crates: `rdocx-oxml`, `rdocx`, `rdocx-layout`, and
`rdocx-html`
**Verdict**: 2 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, targetless hyperlinks still discard their revisions

`crates/rdocx-oxml/src/text.rs:935`
`.claude/plans/F-149-design.md:191`

The remediation collects content revisions from a `w:hyperlink`, but retains
the hyperlink and transfers those revisions into the paragraph only when
either `r:id` or `w:anchor` is present. Both attributes are optional in the
WordprocessingML hyperlink grammar. A valid hyperlink that carries only other
optional metadata, or no target metadata, therefore still loses its contained
`w:ins`, `w:del`, `w:moveFrom`, and `w:moveTo` elements during load. Those
revisions cannot round-trip, report, count, or resolve. This leaves the pass-1
finding open for one approved run-content placement. The fix must retain the
hyperlink and its captured revisions independently of those two attributes,
with a regression covering a targetless hyperlink.

### B2, revision-only hyperlinks are reordered against sibling revisions

`crates/rdocx-oxml/src/revision.rs:186`
`crates/rdocx-oxml/src/text.rs:1214`

A hyperlink whose only content is a revision wrapper has equal `run_start` and
`run_end`. Its revisions are stored in a hyperlink-only slot, while direct
paragraph revisions at the same run boundary are stored in `extra_xml`. The
collector emits every direct boundary revision before every hyperlink revision,
and serialization likewise writes the complete paragraph boundary before all
empty hyperlinks. For example, a revision-only hyperlink followed by a direct
revision at the same boundary is reported and saved in the opposite order.
This contradicts the F-149 document-order contract and means the new hyperlink
round-trip test proves only the case with ordinary runs on both sides. The fix
must put revision-only hyperlinks into the same ordered boundary model as their
sibling raw and typed content, then test both sibling orders through load, save,
reporting, and scoped resolution.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M14 gate is: "a document carrying tracked changes, comments, content
controls and bookmarks round-trips byte-identically in the parts this
milestone does not model, and every one of the four is readable and writable
through the public API."

S47 does not establish that gate because the sprint plan assigns the
end-of-milestone gate to S48. For the S47 contribution,
`hyperlink_and_nested_content_revisions_round_trip_and_report_in_order` and
`hyperlink_nested_revisions_resolve_inside_out_when_scoped` pass and show that
the remediation works for a targeted hyperlink with ordinary runs on both
sides. B1 and B2 show that the same behavior does not cover all reachable
hyperlink placements or preserve document order, so the S47 tracked-change
contribution remains blocked.

## Not found

- `interaction`: outside B1 and B2, no additional conflict between the F-149
  ownership model and the F-150 resolver was found.
- `duplication`: no duplicate sprint helper or second public revision model was
  introduced.
- `layering`: no Cargo manifest changed, and no `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency.
- `harness`: no undeclared delta was found. The integrated gate and both
  AS_BUILT entries record the hash harness unchanged at 49 of 49.
- `docs`: HLD 03, HLD 04, and HLD 10 contain the intended revision ownership,
  atomicity, and native-surface updates. No documentation gap was found beyond
  the implementation contradictions in B1 and B2.
- `deps`: no dependency was added.
- `surface`: the native revision types and eight resolution methods are called
  for by F-149 and F-150. No unrequested binding surface was added.
- `namespace`: the remediated wrapper parser and resolver are namespace-aware,
  and no additional namespace collision or promotion defect was found.
- `count and mutation`: the focused tests confirm one count per selected nested
  element, inside-out resolution, deleted-text conversion, and atomic commit for
  the covered placements. No additional defect was found beyond B1 and B2.
- `delivery`: CURRENT_SPRINT, BACKLOG, SPRINT_TRACKER, and AS_BUILT agree on the
  two completed stories and unchanged harness evidence.
