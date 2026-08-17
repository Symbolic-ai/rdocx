# S48 sprint review, pass 1

**Reviewed**: `sprint/s48` at `701f230` against `7fc81d95`, 40 files,
3,086 changed lines, crates: `rdocx-oxml`, `rdocx-layout`, `rdocx`, and
`rdocx-html`
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, the combined M14 collaboration gate has no executable evidence

`docs/sprints/CURRENT_SPRINT.md:58`
`crates/rdocx/tests/integration_test.rs:56`
`crates/rdocx/tests/integration_test.rs:1299`
`crates/rdocx/tests/regression_test.rs:95`
`crates/rdocx/tests/regression_test.rs:2013`

The sprint definition requires one document carrying tracked changes,
comments, content controls, and bookmarks. That document must prove that all
four subsystems are readable and writable through the public API while every
unmodelled part stays byte-identical. The available tests divide that evidence
among separate fixtures. The revision traversal fixture includes content
controls but no comments or bookmarks. The comment round-trip fixture contains
no revisions, controls, or bookmarks. The revision-view golden and bookmark
API regressions are also independent documents. No test composes all four or
exercises their public mutations before save and reopen.

This leaves the sprint goal and the M14 end gate asserted rather than tested.
The fix must add one mixed-document regression in an existing integration
binary. It must read and write every subsystem through the public facade, save
and reopen the document, and compare every unmodelled part or subtree with its
source bytes.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M14 gate is: "a document carrying tracked changes, comments, content
controls and bookmarks round-trips byte-identically in the parts this
milestone does not model, and every one of the four is readable and writable
through the public API."

The gate does not hold yet. The individual comment, revision, content-control,
and bookmark tests establish each feature in isolation, and the F-151 golden
test establishes both revision render views. B1 is the missing evidence that
the four collaboration models coexist and remain writable in one package.

## Not found

- `interaction`: F-151's render selector and F-155's read-only settings
  projection coexist on `Document` without shared mutation or cache state.
- `duplication`: no duplicate sprint helper or second settings or revision
  representation was introduced.
- `layering`: no Cargo manifest changed, and no `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency.
- `harness`: neither hash baseline changed. Integrated evidence and both
  AS_BUILT entries report the hash harness unchanged at 49 of 49.
- `docs`: the F-151 and F-155 HLD impact lists match the updated HLD files. No
  contradiction was found beyond the unproved milestone gate in B1.
- `deps`: no dependency was added.
- `surface`: the revision-view options and document-protection metadata are
  required by the approved stories. Python, WASM, and CLI surfaces remain
  unchanged.
- `structure`: the new settings module was explicitly approved. No new trait,
  generic parameter, forwarding wrapper, feature flag, or crate was added.
