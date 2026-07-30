# S04 sprint review, pass 2

**Reviewed**: `sprint/s04` against
`f464f756f5d425683d7a1c83173c84418e4c1011`, 27 files, 2,516 changed lines,
crates: `oxml-opc`
**Verdict**: 1 blocking, 1 should-fix, 0 nice-to-have

## Blocking

### B1, the independent archive is still not a valid PresentationML package

`crates/oxml-opc/src/package.rs:379`

The new archive is independent of `OpcPackage::write_to`, but its slide-layout
part has no relationship to a slide master and the archive contains no slide
master or theme. The PresentationML contract requires at least one slide master
and requires every layout to have exactly one `slideMaster` relationship at
`docs/hld/06-presentationml-model.md:12` and
`docs/hld/06-presentationml-model.md:18`. The test at
`crates/oxml-opc/src/package.rs:491` proves that the OPC reader accepts this
seven-entry archive, but it does not establish that the archive is a real,
valid `.pptx`. Add the required master, theme, relationships, content-type
overrides, and presentation master reference, or use an independently produced
valid deck while preserving the code-built fixture policy.

## Should-fix

### S1, the revised migration order conflicts with the remaining sprint roadmap

`docs/hld/11-migration-plan.md:66`

The pass-1 migration-order finding is resolved for the S04 staging and deferred
cutover itself. The revised HLD now places PowerPoint implementation before all
released-rdocx cutovers. The remaining roadmap still schedules `rdocx` adoption
of `oxml-media` in S05 at `docs/sprints/SPRINT_PLAN.md:108`, and schedules the
shared-infrastructure publication release in S11 before M7 PowerPoint work
begins at `docs/sprints/SPRINT_PLAN.md:196`. A future sprint therefore cannot
follow both tracked plans or the no-development-crate-publication rule. Replan
the affected S05 through S11 cutover and release stories so staging can proceed
without directing an early consumer switch or publication.

## Nice-to-have

None.

## Milestone gate

The M2 gate is: hash harness unchanged, and `OpcPackage` opens a real `.pptx`
in a test.

The hash half holds. An independent `python3 scripts/hash_harness.py --check`
run matched all 28 entries, and `scripts/hash_baseline.json` has no sprint
diff. The package test suite also passes all 19 tests, including
`independently_built_pptx_opens_and_resolves_relationships`. The real `.pptx`
half does not hold because the archive omits the slide-master and theme graph
required by `docs/hld/06-presentationml-model.md:12`. The milestone gate is
therefore not met.

## Not found

No additional interaction issue was found across F-018 through F-021. The
duplicate OPC implementation is the approved staged copy bounded by the
deferred F-022 cutover. `cargo tree -p oxml-opc --edges normal` shows only
`quick-xml`, `thiserror`, and `zip` as direct dependencies, with no `rdocx-*` or
`rpptx*` layering violation. Each dependency has a direct named consumer. Every
sprint commit declares the hash harness unchanged, and the baseline check
agrees. The generic constructors, public constants module, package constants,
and copied package API are all required by the approved stories, so no extra
public surface was found.
