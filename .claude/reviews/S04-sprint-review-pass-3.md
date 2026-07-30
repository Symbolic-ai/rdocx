# S04 sprint review, pass 3

**Reviewed**: `sprint/s04` against
`f464f756f5d425683d7a1c83173c84418e4c1011`, 31 files, 2,888 changed lines,
crates: `oxml-opc`
**Verdict**: 1 blocking, 1 should-fix, 0 nice-to-have

## Blocking

### B1, the independent archive still violates the slide-layout ID schema

`crates/oxml-opc/src/package.rs:402`

The fixture now supplies and resolves the complete presentation to master and
slide, slide to layout, layout to master, and master to layout and theme graph.
Its `p`, `a`, and `r` prefixes are also bound where used. However,
`p:sldLayoutId` is written with `id="1"`. PresentationML defines that attribute
as `ST_SlideLayoutId`, whose minimum value is 2147483648. The archive therefore
remains schema-invalid even though `OpcPackage` can parse its ZIP and
relationship parts. Use a conforming layout ID and add an assertion that locks
the schema range, so the M2 real-pptx gate cannot regress to a merely connected
archive.

## Should-fix

### S1, the migration plan still describes obsolete release versions and tooling

`docs/hld/11-migration-plan.md:156`

The same document records the seven released rdocx crates at 0.4.1 on line 127,
but later says the cutover takes rdocx to 0.3.0. It also says on line 168 that
the current `publish.yml` uses `cargo publish --workspace`, while the actual
workflow remains the seven-package allowlist and the roadmap defers expanding
it to F-049. Update these statements to the current version-independent cutover
plan and explicit allowlist model. Leaving them in place makes the permanent
migration guidance disagree with the release boundary introduced by this
sprint.

## Nice-to-have

None.

## Milestone gate

The M2 gate is: hash harness unchanged, and `OpcPackage` opens a real `.pptx`
in a test.

The hash half holds. An independent `python3 scripts/hash_harness.py --check`
run matched all 28 entries, and `scripts/hash_baseline.json` has no sprint
diff. The focused independent-archive test passes and proves every required
relationship target resolves to a present part. The real `.pptx` half does not
hold because the slide master contains the out-of-range layout ID described in
B1. The milestone gate is therefore not met.

## Not found

No additional interaction or duplication issue was found across F-018 through
F-021. `cargo tree -p oxml-opc --edges normal` shows only `quick-xml`,
`thiserror`, and `zip` as direct dependencies, with no `rdocx-*` or `rpptx*`
layering violation. No released rdocx crate or current publication workflow is
changed by the sprint delta. The roadmap now stages F-029 through F-045 in
isolated crates, leaves `rdocx-layout` and `rdocx-pdf` unchanged, and defers all
shared publication and released-rdocx consumer cutovers to S32.1 and S32.2
after PowerPoint development. The new crate remains at 0.0.0 with
`publish = false`. Every dependency has a named consumer, no hash baseline
change exists, and no public surface beyond the approved story contracts was
found.
