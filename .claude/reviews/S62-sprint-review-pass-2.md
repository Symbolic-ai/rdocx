# S62 sprint review, pass 2

**Reviewed**: `sprint/s62` at
`401b4d4be6a14390a47a9d78d6c29bd5414d5d67` against `main` at
`a320b976bdbff1e83234fe3d5d1d988b4e183428`, 73 files and 14,839 changed
lines, crates: oxml-opc, rdocx-html, rdocx-layout, rdocx-oxml, rdocx,
rpptx-oxml, rpptx
**Boundary**: completed F-218, F-219, and F-X071, with F-220 explicitly
carried to S63 and its worker implementation excluded
**Verdict**: 0 blocking, 1 should-fix, 0 nice-to-have
**Clean status**: clean before this review output

## Blocking

None.

## Should-fix

### S1, remove the remaining S62 claims that SmartArt rendering shipped

`docs/sprints/CURRENT_SPRINT.md:8`
`docs/sprints/SPRINT_PLAN.md:1160`

The sprint goal still says S62 gives the bounded SmartArt corpus deterministic
rendering, and the S62 plan paragraph still says SmartArt gains rendering in
this sprint. Those statements contradict the explicit carry record at
`docs/sprints/CURRENT_SPRINT.md:61` and the moved F-220 row at
`docs/sprints/SPRINT_PLAN.md:1172`. The integrated production delta contains
the F-219 typed model but none of the F-220 layout or rendering implementation.
Before close, make both summary sentences describe typed inspection and editing
only, leaving supported SmartArt rendering with S63. This is a delivery-record
correction, not a source or dependency defect.

## Nice-to-have

None.

## Milestone gate

The M21 gate is: "one representative modern deck round-trips its comments,
sections, SmartArt, media, animation timeline, signatures, and package variant
without repair. Its static frames, animated export, notes, and handouts match
the pinned PowerPoint oracle at their declared fidelity boundaries."

The M21 gate does not yet hold, and S62 does not claim that it does. The gate
remains open at `docs/hld/14-development-backlog.md:1896`. F-220 supplies the
required SmartArt geometry and SSIM differential at
`docs/hld/14-development-backlog.md:1951`, and it is correctly pending in S63
at `docs/sprints/BACKLOG.md:419`. The carry record identifies the two remaining
fail-closed validator gaps and requires a clean microscope pass before F-222 at
`docs/sprints/CURRENT_SPRINT.md:61`. Carrying F-220 is therefore compatible
with closing S62, while M21 continues through S64.

The completed S62 story gates hold. F-218's inventory, extraction, mutation,
signature, producing-scope, and atomicity coverage is anchored at
`crates/rpptx/tests/integration.rs:9` and
`crates/rpptx/tests/integration.rs:394`. F-219's schema-order and round-trip
coverage is anchored at `crates/rpptx-oxml/tests/integration.rs:138` and its
supported and unsupported package round trips at
`crates/rpptx/tests/integration.rs:1483` and
`crates/rpptx/tests/integration.rs:1523`. F-X071's effective default-style
numbering, owner-namespace, nested-revision, and complex-field regressions are
anchored at `crates/rdocx/src/document.rs:11271`,
`crates/rdocx/tests/regression_test.rs:2990`,
`crates/rdocx/tests/regression_test.rs:3929`, and
`crates/rdocx-oxml/src/text.rs:6197`.

At the exact integrated HEAD, the rpptx-oxml suite passed 15 unit and 151
integration tests, including all 50 pinned decks. The rpptx suite passed 26
unit and 175 non-ignored integration tests. Its two python-pptx differentials
also passed when rerun with a sandbox-writable uv cache. The rdocx-oxml suite
passed 334 unit tests and one doctest. The rdocx library passed 326 tests with
three ignored, and its regression binary passed 180 tests with one ignored.
The rdocx-html and rdocx-layout suites passed. The LibreOffice integration
could not launch inside this audit sandbox, while the exact delivery record
reports that boundary and the full workspace gate passed at
`docs/sprints/AS_BUILT.md:10671`. The dependency-direction regression,
`git diff --check`, prose check, generated-skill drift check, and the 49-entry
hash harness passed directly.

## Not found

- **Interaction** produced zero findings. F-218 consolidates and validates the
  current package for embedded-content mutation and persisted signature state,
  while F-219 stages and reopens diagram edits and transfers. F-X071 changes
  only the Word reader family. No jointly incorrect ownership, mutation, or
  preservation path was found.
- **Duplication** produced zero findings. Executable-content graph logic remains
  in the approved private rpptx module, diagram XML remains in one approved
  rpptx-oxml module, and F-X071 extends the existing Word models and facades.
- **Layering** produced zero findings. No oxml crate gained an rdocx or rpptx
  dependency, and the one-way rpptx-render dependency regression passed.
- **Harness** produced zero findings. No baseline changed, direct execution
  matched 49 of 49 entries, and all three completion records report the same
  unchanged result at `docs/sprints/AS_BUILT.md:10582`,
  `docs/sprints/AS_BUILT.md:10630`, and
  `docs/sprints/AS_BUILT.md:10677`.
- **Gate** produced zero findings beyond the explicitly open M21 gate. The
  completed feature gates have named regression evidence, and the carried
  differential is not represented as completed.
- **Docs** produced no finding beyond S1. The approved HLD impact sets are
  present, the backlog assigns F-220 to S63, and the completed-feature ledgers
  consistently list F-218, F-219, and F-X071 only.
- **Dependencies** produced zero findings. F-218 gives rpptx named consumers for
  the existing workspace quick-xml and SHA-256 implementations at
  `crates/rpptx/Cargo.toml:51`. F-219 and F-X071 add no dependency, and no
  dependency direction is reversed.
- **Public surface and compatibility** produced zero findings. F-218 and F-219
  add the approved native pre-1.0 rpptx surfaces. F-X071 adds facade reader
  facts and intentionally changes exhaustive low-level rdocx-oxml struct
  literals for raw preservation, as recorded at
  `docs/hld/10-bindings-spec.md:702`. Python, WASM, and CLI surfaces are
  unchanged. S62 publishes no package and needs no standalone version bump.
  The intentional low-level source breaks remain assigned to the next stable
  family release boundary.
- **Structure** produced zero findings. The two new source modules had explicit
  plan approval, and no unapproved crate, feature, trait, generic, dynamic
  dispatch, wrapper, integration binary, or binary fixture was added.
