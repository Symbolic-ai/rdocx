# F-X070, working, pass 2

**Reviewed**: complete diff from claim base `fa798ca359df9819cb1863b39ce27364ba58b872` through the working tree, 10 files, 227 insertions and 36 deletions, plus pass 1 remediation
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, The exact allowlist still has command-placement bypasses

`scripts/test_sprint_workflow.py:5318`

`scripts/test_sprint_workflow.py:5333`

The remediation rejects the three original added commands when they appear as
bare lines inside `## Approach`, but the helper scans only that section and its
command classifier recognizes only a fixed executable prefix. An unauthorized
`gh release create v0.11.0` placed after `## Approach` is ignored. Inside the
section, `env gh release create v0.11.0`,
`env git push origin --delete refs/tags/v0.11.0`, and
`env cargo publish --registry crates-io -p rdocx` are also ignored. These four
mutations were exercised independently and the helper accepted all of them.
The regression therefore still does not enforce the plan-wide exact mutation
allowlist promised by the test plan.

## Smells

None.

## Nitpicks

None.

## Not found

Pass 1 D2 is resolved. The migration section now states stable 0.11.1 and
shared 0.8.0 as current, and retains 0.6.0 and 0.1.3 as historical boundaries.
Pass 1 S1 is resolved. The HLD and plan now distinguish prohibited external
contribution-record mutation from required local sprint ledgers, progress,
review, and handoff records.

No additional correctness, contract, approval-boundary, immutable-history,
preflight, structure, panic, OOXML, or test findings were found. The current
plan contains exactly the two authorized valid `cargo yank` commands, retains
the separate immediate approval boundary, and leaves all external mutation
checklist items incomplete. The focused cleanup regression, the 90-test
workflow module, prose check, and diff check pass. No external mutation was
performed.
