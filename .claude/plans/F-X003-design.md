# F-X003, Deduplicate the sample generators

**Status**: approved
**Sprint**: S36
**Size**: S
**Depends on**: none

## Problem

`crates/rdocx/examples/generate_samples.rs` is a second 783-line feature
showcase generator that overlaps the canonical 2,461-line
`generate_all_samples.rs`. The hash and golden-PNG harnesses both invoke only
`generate_all_samples`, which already emits every one of the seven named
samples they consume. Keeping the unused second implementation makes sample
coverage and fixes easy to apply in only one of two places.

## Spec reference

- `docs/hld/11-migration-plan.md`, "The safety net comes first".
- `docs/hld/12-testing-strategy.md`, "The hash harness" and "The golden-PNG gate".
- `docs/hld/14-development-backlog.md`, "F-X003, Deduplicate the sample generators".

## Approach

Delete `crates/rdocx/examples/generate_samples.rs`. Retain
`generate_all_samples.rs` as the sole sample generator and do not move its
builders into a new module. Prove that this one executable still creates all
seven DOCX and page-one PNG inputs required by the hash harness, and all seven
PDF inputs required by the golden-PNG harness.

## Rejected alternatives

- Extract shared builders into a new module. Only one generator is consumed,
  so a shared module would increase indirection instead of reducing cases.
- Keep both examples and synchronize them. That preserves the duplication the
  story exists to remove.
- Change the sample contents while deleting the duplicate. A behavioral change
  cannot be reviewed separately from this structural cleanup.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | `python3 scripts/hash_harness.py --check` | The sole generator produces every DOCX and PNG required by the 28-entry harness with unchanged bytes |
| golden | `python3 scripts/golden_png_harness.py --check` | The sole generator produces every PDF required by the page-one pixel gate |
| regression | repository path search | `generate_all_samples` is the only sample generator and both harnesses invoke it |

Sensitivity temporarily removes one named sample from the canonical generator,
proves the exact hash gate fails on the missing output, restores the source
byte-identically, and reruns green.

## HLD impact

None. HLD11 and HLD12 already name `generate_all_samples` as the canonical
generator and require no mechanism change.

## Risk routing

- Behavior-neutral file deletion. Read the workflow hash-harness rule and prove
  all 28 entries remain byte-identical. Do not combine a sample behavior change
  with the deletion.

## Hash harness

Expected unchanged. This story removes only an unused duplicate executable.

## Implementation checklist

- [ ] Delete the obsolete `generate_samples.rs` example.
- [ ] Confirm no tracked code or documentation still invokes it.
- [ ] Run the hash and golden-PNG gates from the sole generator.
- [ ] Prove the missing-sample sensitivity and byte-identical restoration.
- [ ] Run format, prose, skill-drift, and diff checks.

## Open questions

None.
