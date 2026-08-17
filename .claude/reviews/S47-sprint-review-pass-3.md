# S47 sprint review, pass 3

**Reviewed**: `sprint/s47` at `9c37763` against `d625bda4`, 40 files,
4,781 changed lines, crates: `rdocx-oxml`, `rdocx`, `rdocx-layout`, and
`rdocx-html`
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, modeled hyperlinks still discard malformed and foreign raw children

`crates/rdocx-oxml/src/text.rs:826`
`crates/rdocx-oxml/src/text.rs:854`
`crates/rdocx-oxml/src/text.rs:1225`
`crates/rdocx-oxml/src/text.rs:1230`

The paragraph parser recognizes the outer hyperlink by local name alone. Its
child parser captures a content revision but retains it only when metadata
projection succeeds, and skips every foreign or otherwise unmodeled child.
When the same hyperlink contains at least one ordinary run, the caller keeps
only the parsed runs and valid revisions and reconstructs the hyperlink rather
than retaining its raw subtree. A malformed `w:ins` or foreign same-local-name
child beside a valid run is therefore lost on load and save. A foreign
`x:hyperlink` containing Word runs is also promoted into a modeled
`w:hyperlink`.

This contradicts F-149's round-trip gate and HLD 03's rule that invalid
revision metadata remains preserved but unreported. The fix must namespace
check the outer hyperlink and retain ordered raw children for a modeled
hyperlink, including malformed Word revisions and foreign elements, without
duplicating valid typed revisions. Regressions must cover load, save,
reporting, and scoped resolution with valid runs before and after the raw
children.

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
`hyperlink_and_nested_content_revisions_round_trip_and_report_in_order`,
`targetless_revision_only_hyperlinks_keep_sibling_order_when_resolved`, and
`hyperlink_nested_revisions_resolve_inside_out_when_scoped` pass. They show
that pass-1 B1 and pass-2 B1 and B2 are fixed for valid targetless hyperlinks,
both direct-sibling orders, nested document order, scoped inside-out
resolution, and exact counts. B1 shows that the round-trip claim still fails
for malformed and foreign raw children in a hyperlink that also has a run, so
the S47 tracked-change contribution remains blocked.

## Not found

- `interaction`: outside B1, no additional conflict between the F-149 ordered
  projection and F-150 resolver was found.
- `duplication`: no duplicate sprint helper or second public revision model
  was introduced.
- `layering`: no Cargo manifest changed, and no `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency.
- `harness`: no undeclared delta was found. The focused review run reports all
  49 entries unchanged, and both AS_BUILT entries record 49 of 49.
- `docs`: HLD 03, HLD 04, and HLD 10 contain the intended revision ownership,
  atomicity, and native-surface updates. No additional documentation gap was
  found beyond B1's implementation contradiction.
- `deps`: no dependency was added.
- `surface`: the native revision types and eight resolution methods are called
  for by F-149 and F-150. No unrequested binding surface was added.
- `targetless and ordering`: the pass-2 remediation retains revision-only
  targetless hyperlinks as ordered raw boundary content. Both hyperlink-before
  direct and direct-before-hyperlink cases round-trip and report in order.
- `nested resolution`: valid nested revisions are reported once in document
  order. Scoped actions resolve inside out and count each selected element
  once, including a selected descendant hidden by a removed ancestor.
- `namespace and serialization`: outside B1, selected content promotes required
  declarations, property rejection recovers owner declarations, and property
  changes serialize in their schema-defined final slots.
- `delivery`: CURRENT_SPRINT, BACKLOG, SPRINT_TRACKER, and AS_BUILT agree on the
  two completed stories and unchanged harness evidence.
