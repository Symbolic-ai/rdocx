# F-048, all, pass 1

**Reviewed**: uncommitted working-tree diff, 22 files, 160 added lines and 0 removed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the stable group assertion does not match cargo-release's effective configuration

`crates/rdocx/Cargo.toml:4`
`crates/rdocx/Cargo.toml:15`
`scripts/test_sprint_workflow.py:136`

All eight stable packages inherit `[workspace.package].version`. Cargo-release
1.1.3 consequently resolves their effective `shared-version` to its reserved
`workspace` group, even though each manifest contains the literal `stable`
value. The new test parses raw TOML and therefore passes without detecting that
the approved plan's named stable group is not the group cargo-release uses.
The incubating packages do resolve to the requested `incubating` group. Make
the stable contract agree with Cargo workspace inheritance and test the
effective cargo-release configuration rather than an overridden manifest
value.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness beyond D1: no other wrong version-selection, dependency-pin, or
  lockfile behavior found in either preparation trial.
- Contract beyond D1: no extra external release action or README replacement
  was added.
- Panics: no runtime indexing, slicing, `unwrap`, `expect`, or arithmetic was
  introduced.
- OOXML: not applicable because the diff changes only Cargo metadata and its
  workflow tests.
- Tests beyond D1: the focused tests fail when the manifest metadata is
  reverted, and the temporary stable and incubating preparation commits prove
  the expected manifest and lockfile changes.
- Structure: no trait, generic, wrapper, feature flag, crate, module, or source
  file was introduced.
- Release safety: publication, tag creation, and pushing remain disabled.
  Stable and incubating tag templates remain `v{{version}}` and
  `rpptx-v{{version}}`, and neither preparation trial touched README prose.
- Dependency graph: no dependency edge changed. The only `oxml-*` edge into
  `rdocx-*` remains the documented Theme adapter exception.
