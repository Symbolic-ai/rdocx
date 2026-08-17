# S47 sprint review, pass 4

**Reviewed**: `sprint/s47` at `ca6bf07` against `d625bda4`, 44 files,
4,819 changed lines, crates: `rdocx-oxml`, `rdocx`, `rdocx-layout`, and
`rdocx-html`
**Review-bound extension**: On 2026-08-17, the user explicitly authorized pass
4 and as many further passes as required to reach a clean verdict.
**Verdict**: 3 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, a valid revision inside a malformed wrapper is hidden from reporting but still resolved

`crates/rdocx-oxml/src/text.rs:1261`
`crates/rdocx-oxml/src/text.rs:1267`
`crates/rdocx/src/revision.rs:701`
`crates/rdocx/src/revision.rs:792`

The low-level hyperlink parser treats a malformed content revision as one raw
child and does not project any descendants. `Document::revisions` therefore
reports no valid revision nested inside that malformed owner. The facade XML
tree makes the malformed Word wrapper modeled before metadata validation,
then continues modeling revision children solely from the owner's local name.
An id-scoped action or `accept_all` can consequently resolve a valid nested
revision that the public inspection API did not report, changing the raw
malformed subtree that F-149 requires to remain opaque and unchanged.

The fix must make a revision wrapper whose required metadata is invalid an
opaque boundary for both reporting and resolution. Add a regression with a
valid selected revision nested inside a malformed hyperlink child. Inspection
must omit it, every scoped and all-revision action must return zero, and the
malformed subtree must remain byte-identical.

### B2, comment removal can discard or misorder retained hyperlink revisions and raw children

`crates/rdocx-oxml/src/text.rs:738`
`crates/rdocx-oxml/src/text.rs:749`
`crates/rdocx-oxml/src/text.rs:753`
`crates/rdocx-oxml/src/text.rs:1430`
`crates/rdocx/src/comments.rs:967`

The pass-3 remediation stores raw hyperlink children at relative run
boundaries, but the existing comment-removal mutation remaps only paragraph
revision boundaries and hyperlink start and end indexes. It does not remap
the new relative raw-child boundaries. If removal deletes the hyperlink's
only ordinary run, the collapsed span is discarded even when it still owns a
valid revision or preserved raw child. The hyperlink serializer can emit
those children only through the retained span, so they disappear. Removing an
earlier run from a multi-run hyperlink also leaves later raw children at stale
relative boundaries, which can change their order or leave them past the new
end.

This is an S46 and S47 interaction that violates the collaboration-layer
preservation contract. The fix must remap hyperlink-local raw boundaries and
typed revision slots with the same collapsed-boundary ordering used for
paragraph content. A hyperlink that loses its last run must remain serializable
when it still owns revisions or raw children. Add regressions for sole-run and
multi-run hyperlinks through `remove_comment`, load, save, reporting, and
subsequent scoped revision resolution.

### B3, relationship attributes on modeled hyperlinks are not namespace checked

`crates/rdocx-oxml/src/properties.rs:1097`
`crates/rdocx-oxml/src/text.rs:1308`
`crates/rdocx-oxml/src/text.rs:1340`
`crates/rdocx-oxml/src/text.rs:1352`
`crates/rdocx/src/paragraph.rs:793`

`parse_hyperlink_attributes` receives the result of `word_prefixes_at`, which
contains only prefixes bound to the Word namespace. Its generic binding lookup
therefore cannot resolve a relationship namespace. The fallback accepts the
literal prefix `r` without checking whether the hyperlink locally rebound it,
and it fails to project an aliased prefix that is actually bound to the
relationship namespace. A foreign `r:id` can consequently appear as a public
hyperlink relationship id, while a namespace-correct aliased id is omitted
from that typed surface. HTML, Markdown, layout, and the native hyperlink API
consume this projection.

The fix must pass a real in-scope prefix-to-namespace map to hyperlink
attribute parsing and recognize `r:id` only by expanded name. Preserve all
unmodeled owner declarations and attributes without duplication. Add namespace
collision regressions for a locally rebound `r`, an aliased relationship
prefix, and foreign same-local-name attributes through load, save, and public
hyperlink inspection.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M14 gate is: "a document carrying tracked changes, comments, content
controls and bookmarks round-trips byte-identically in the parts this
milestone does not model, and every one of the four is readable and writable
through the public API."

S47 does not establish that gate because the sprint plan assigns the complete
end-of-milestone gate to S48. For the S47 contribution,
`modeled_hyperlinks_preserve_unreported_raw_children_and_foreign_owners`,
`resolving_a_modeled_hyperlink_keeps_unreported_raw_children`,
`hyperlink_nested_revisions_resolve_inside_out_when_scoped`, and
`targetless_revision_only_hyperlinks_keep_sibling_order_when_resolved` pass.
They establish the direct pass-3 cases for a foreign outer owner, ordered raw
siblings, valid reporting, and scoped resolution. B1 through B3 show that the
same preservation and namespace contracts fail under malformed nesting,
comment mutation, and relationship-prefix collisions, so the S47 contribution
remains blocked.

## Not found

- `prior review findings`: pass-1 B1 and pass-2 B1 and B2 remain fixed for
  valid nested revisions, targetless hyperlinks, both direct-sibling orders,
  inside-out scoped resolution, and exact counts. The direct pass-3 examples
  now preserve malformed and foreign children beside ordinary runs, and a
  foreign outer hyperlink is retained as raw XML.
- `duplication`: no duplicate sprint helper or second public revision model
  was introduced.
- `layering`: no Cargo manifest changed, and no `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency.
- `harness`: no undeclared delta was found. The focused review evidence and
  both AS_BUILT entries record the hash harness unchanged at 49 of 49.
- `docs`: HLD 03, HLD 04, and HLD 10 contain the intended revision ownership,
  atomicity, and native-surface updates. No separate documentation gap was
  found beyond the implementation contradictions in B1 through B3.
- `deps`: no dependency was added.
- `surface`: the native revision types and eight resolution methods are called
  for by F-149 and F-150. No unrequested binding surface was added.
- `schema order and duplication`: the direct hyperlink remediation emits each
  valid revision and retained raw sibling once at its recorded boundary.
- `delivery`: CURRENT_SPRINT, BACKLOG, SPRINT_TRACKER, and AS_BUILT agree on the
  two completed stories and unchanged harness evidence.
