# F-X070, working, pass 1

**Reviewed**: complete diff from claim base `fa798ca359df9819cb1863b39ce27364ba58b872` through the working tree, 10 files, 132 insertions and 22 deletions
**Verdict**: 2 defects, 1 smell, 0 nitpicks

## Defects

### D1, The allowlist regression accepts forbidden mutation commands

`scripts/test_sprint_workflow.py:5322`

The helper extracts only lines that start with `cargo yank` and then checks
that the prose still contains the prohibition sentence. Adding a
`gh release create v0.11.0`, `git push origin --delete refs/tags/v0.11.0`, or
`gh issue close 53` command leaves both assertions satisfied. All three added
commands were exercised independently and the helper accepted them. The test
therefore does not enforce the advertised ban on tag, release, notification,
or closure mutations, even though the design calls this an exact cleanup
allowlist.

### D2, The touched migration section still states obsolete families as current

`docs/hld/11-migration-plan.md:142`

The section still says that all seven rdocx crates are published at 0.6.0 and
that the complete shared and PowerPoint family remains at 0.1.3. The same
section now adds an exception tied to the complete stable 0.11.1 recovery,
while the other updated HLD files identify stable 0.11.1 and shared 0.8.0 as
the current published families. HLD files describe current reality, so this
newly touched section is internally inconsistent and contradicts the other
five F-X070 HLD updates.

## Smells

### S1, The record-state prohibition is broader than the external scope

`docs/hld/11-migration-plan.md:164`

`docs/hld/14-development-backlog.md:3360`

Both additions say that no other record state changes, without qualifying the
statement as external contribution-record state. The design checklist still
requires the normal delivery-record completion at
`.claude/plans/F-X070-design.md:117`. The unqualified HLD wording can therefore
be read as forbidding the sprint ledger transition that completion requires.

## Nitpicks

None.

## Not found

No additional correctness, contract, approval-boundary, immutable-history,
preflight, structure, panic, or OOXML findings were found. The two authorized
`cargo yank` commands use valid syntax and currently name only
`rdocx-opc@0.11.0` and `rdocx-oxml@0.11.0`. Read-only registry checks confirmed
all seven 0.11.1 packages live and unyanked under sole owner `mantissaman`, the
two named 0.11.0 packages live and unyanked, and the other five 0.11.0 packages
absent. The remote annotated tag still dereferences to
`25350d000ed7ed96bf4f6e371f01f8fbc8e2cec4`, and no v0.11.0 GitHub release
exists. No external mutation was performed. The focused cleanup regression,
the 90-test workflow module, prose check, and diff check pass.
