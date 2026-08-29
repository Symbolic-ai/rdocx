# F-X070, Yank incomplete v0.11.0 packages

**Status**: approved
**Sprint**: S58
**Size**: S
**Depends on**: F-X069

## Problem

The immutable v0.11.0 attempt left two live crates.io entries,
`rdocx-opc@0.11.0` and `rdocx-oxml@0.11.0`, without the other five packages in
their lockstep stable family. Once v0.11.1 is complete, ordinary dependency
selection should not offer those incomplete entries as if they represented a
usable release.

Yanking changes external registry state and is not part of ordinary release
publication. It must occur only after the complete recovery verifies, with its
own immediate approval and exact readback evidence. The immutable v0.11.0 tag
and failed workflow evidence remain part of the historical record.

## Spec reference

- `docs/hld/03-architecture.md`, "Versioning" and immutable family boundaries.
- `docs/hld/10-bindings-spec.md`, "Packaging" and publication authority.
- `docs/hld/12-testing-strategy.md`, release and registry evidence gates.
- `docs/hld/14-development-backlog.md`, "F-X070, Yank incomplete v0.11.0 packages".
- `docs/hld/15-build-and-toolchain.md`, "Publishing" and "Release process".

## Approach

First verify that all seven stable 0.11.1 packages are live, unyanked, and
owned by the authenticated publisher. Reconfirm that exactly
`rdocx-opc@0.11.0` and `rdocx-oxml@0.11.0` exist from the partial attempt, the
other five 0.11.0 entries are absent, the annotated `v0.11.0` tag still targets
reviewed SHA `25350d000ed7ed96bf4f6e371f01f8fbc8e2cec4`, and no v0.11.0 GitHub
release exists.

Stop and request a separate final approval immediately before the first yank.
After approval, run the exact two registry mutations for version 0.11.0 and no
others. Read crates.io back independently and record the yanked flags, owners,
complete live 0.11.1 family, absent five 0.11.0 entries, immutable tag target,
and absent v0.11.0 GitHub release. Do not delete or move tags, create a release,
post comments, close records, or alter any other version.

## Rejected alternatives

- Leave the partial entries live. The user approved removing them from ordinary
  dependency selection after recovery.
- Delete the registry entries. crates.io supports yanking, not deletion.
- Move or delete the v0.11.0 tag. That would rewrite verified history.
- Yank before v0.11.1 verifies. That could remove the only published pieces
  before a complete replacement exists.
- Combine the cleanup with F-X069 publication. Separate authority and evidence
  make the destructive registry mutation auditable.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_partial_v0_11_0_cleanup_contract` | The cleanup allowlist is exactly two packages at 0.11.0 and forbids tag, release, notification, closure, or other-version mutation. |
| preflight | crates.io, tag, and GitHub release readback | Seven 0.11.1 packages are live, exactly two 0.11.0 packages exist, five are absent, the tag target is immutable, and no v0.11.0 release exists. |
| integration | post-yank crates.io readback | Both incomplete 0.11.0 entries are yanked and owned correctly while all seven 0.11.1 entries remain live and unyanked. |
| verification | `/verify --full` | The tracked cleanup record and unchanged source still pass the complete repository gate. |

The **test gate is integration**. Completion requires a separately approved
real yank of exactly the two incomplete entries plus independent readback of
the complete stable recovery and preserved immutable release history.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Release scripting and external registry mutation**. Re-read the release
  authority and publishing HLD, resolve exact crate/version targets through
  read-only checks, require `/verify --full`, and obtain a separate immediate
  approval before the first yank.
- **Crate dependency graph**. Prove the complete 0.11.1 graph remains live and
  the five never-published 0.11.0 packages remain absent.
- **Public API of published crates**. Preserve every archive, tag, and source
  record. Yanking changes selection only and does not rewrite package bytes.

## Hash harness

Expected unchanged across all 49 entries. This story changes registry and
delivery evidence only. Any output delta blocks completion.

## Implementation checklist

- [ ] Verify F-X069 completed and all seven stable 0.11.1 packages are live.
- [ ] Add the exact two-package cleanup contract regression.
- [ ] Reconfirm the pre-yank registry, tag, release, and owner evidence.
- [ ] Update exactly the five listed HLD files for the pending cleanup state.
- [ ] Run `/verify --full` and the unchanged hash gate.
- [ ] Stop for separate final approval immediately before the first yank.
- [ ] Yank exactly `rdocx-opc@0.11.0` and `rdocx-oxml@0.11.0`.
- [ ] Verify yanked flags, complete live 0.11.1 family, immutable tag, and absent v0.11.0 release.
- [ ] Complete the delivery records without any unrelated external mutation.

## Open questions

None. The user approved the two-package post-recovery cleanup. The real yank
retains its separate final go or no-go.
