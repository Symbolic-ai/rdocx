# S04 sprint review, pass 1

**Reviewed**: `sprint/s04` against `f464f756f5d425683d7a1c83173c84418e4c1011`, 25 files, 2,339 lines, crates: `oxml-opc`
**Verdict**: 1 blocking, 1 should-fix, 0 nice-to-have

## Blocking

### B1, the real-pptx half of the M2 gate is not exercised

`docs/hld/14-development-backlog.md:136`

The milestone gate requires `OpcPackage` to open a real `.pptx`, but the only
PowerPoint package evidence constructs an `OpcPackage` with the implementation
under test at `crates/oxml-opc/src/package.rs:348`, writes it with that same
implementation, and reopens those bytes at
`crates/oxml-opc/src/package.rs:410`. This is useful round-trip coverage, but it
cannot detect failures that appear only in an independently produced archive.
The placeholder payloads also use an unbound `p` prefix, so the fixture is not a
valid PresentationML document. Exercise a valid independently produced `.pptx`,
using the repository's corpus exception if needed, or revise the permanent M2
gate through an explicit HLD decision if a code-built self-round-trip was the
intended contract.

## Should-fix

### S1, the migration order still requires the sequence S04 deliberately reversed

`docs/hld/11-migration-plan.md:60`

The permanent migration plan requires the `rdocx-oxml` facade and `Length`
cutover before creating `oxml-opc`. The integrated sprint instead completes
F-018 while carrying F-015 and F-016, as recorded at
`docs/sprints/CURRENT_SPRINT.md:33` and
`docs/sprints/CURRENT_SPRINT.md:38`. That deferral is intentional and protects
published packages from unpublished shared crates, but the HLD still directs a
future session to follow the obsolete order. Update the migration plan to state
the staged-copy order and the deferred publication cutover as current intent.

## Nice-to-have

None.

## Milestone gate

The M2 gate is: hash harness unchanged, and `OpcPackage` opens a real `.pptx` in
a test.

The hash half holds. `scripts/hash_baseline.json` has no sprint diff, and the
four completed AS_BUILT entries record all 28 integrated entries unchanged,
including `docs/sprints/AS_BUILT.md:548` and
`docs/sprints/AS_BUILT.md:639`. The real `.pptx` half does not hold for the
reason in B1. The milestone gate is therefore not met by the integrated S04
evidence.

## Not found

No additional interaction issue was found across F-018 through F-021. The
duplicate OPC implementation is the approved staged copy and is bounded by the
carried F-022 cutover. No `oxml-*` dependency on an `rdocx-*` or `rpptx*` crate
was introduced. The new `quick-xml`, `thiserror`, and `zip` edges each have a
direct `oxml-opc` consumer. No undeclared hash delta or baseline edit was found.
The generic constructors, constant modules, and package APIs are called for by
the approved stories, so no extra public surface was found.
