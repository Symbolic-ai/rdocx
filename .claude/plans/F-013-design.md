# F-013, Create oxml-core

**Status**: approved
**Sprint**: S03
**Size**: M
**Depends on**: none

## Problem

Format-neutral units, XML helpers, raw XML capture, core properties, and the
public `Length` type still live under the Word crates at
`crates/rdocx-oxml/src/lib.rs:13` and `crates/rdocx/src/lib.rs:26`. The root
workspace at `Cargo.toml:3` has no `oxml-core` member, so PresentationML cannot
reuse those implementations without depending on `rdocx-*`.

The extraction must keep every existing rdocx path working and every output
hash unchanged. It must also keep `xml_text` entity handling and the unit
truncation tests intact.

## Spec reference

- `docs/hld/03-architecture.md`, "Three families, one workspace", "The
  dependency rule", and "Crate-level conventions".
- `docs/hld/11-migration-plan.md`, "The facade trick", "Order of operations",
  and "Preserve behaviour, do not improve it".
- `docs/hld/12-testing-strategy.md`, "New tests the extracted crates need",
  subsection `oxml-core`.
- `docs/hld/15-build-and-toolchain.md`, "Publishing".

## Approach

Add `crates/oxml-core` as a workspace member at version 0.0.0 with
`publish = false`, and register it as a workspace dependency without adding it
to the release workflow. The crate is justified by the existing
`rdocx-oxml` and `rdocx` consumers that will adopt it after the rdocx 0.5.0
release boundary.

Stage copies of `error.rs`, `units.rs`, `raw_xml.rs`, `xml_text.rs`, and
`core_properties.rs` in the new crate, move `Length` into the shared units
surface, and add `xml.rs` for `matches_local_name`, `local_name`, `get_attr`,
`R_NS`, and `MC_NS`. Keep the original Word modules temporarily so the
workspace remains green at this independently revertible step. F-015 and
F-016 remove those originals when they switch the existing facades.

Make `xml_text` public and extend its tests for CDATA, mixed content, nested
elements, unknown entities, and split `GeneralRef` events. Do not change the
observable behaviour of copied code.

## Rejected alternatives

- Delete the Word modules immediately. That leaves the workspace broken until
  F-015 and prevents F-013 from being independently revertible.
- Change every existing import to `oxml_core`. The facade plan exists to avoid
  hundreds of unrelated call-site edits.
- Give `oxml-core` a release path now. The user requires all `oxml-*` and
  `rpptx*` development crates to remain unpublished until PowerPoint work is
  complete.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | moved `units`, `raw_xml`, `core_properties`, and `xml_text` tests | Existing behaviour passes unchanged from `oxml-core` |
| unit | `xml_text_handles_cdata_mixed_nested_and_general_refs` | The newly public helper covers every required event form without data loss |
| integration | `cargo check --workspace --all-targets` | The staged extraction leaves every existing Word consumer compiling |

The backlog test gate is that the moved tests pass unchanged in `oxml-core`.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/11-migration-plan.md`
- `docs/hld/15-build-and-toolchain.md`

The update will reconcile the development-version rule and describe the staged
copy followed by F-015 and F-016 deletion, which keeps each story green.

## Risk routing

- New crate, modules, and files. The sprint invocation supplies explicit
  authorization. The two existing consumers are `rdocx-oxml` and `rdocx`.
- Crate dependency graph. Assert `oxml-core` has no `rdocx-*` or `rpptx*`
  dependency and inspect `cargo tree -p oxml-core`.
- Public API of a published family. State the additive semver impact, run
  `cargo package -p oxml-core`, assert the archive is below 10 MiB, and retain
  `publish = false`.
- File move with no behaviour change. Require the deterministic hash harness
  to remain byte-identical and do not update its baseline.
- Unit conversion. Preserve the pinned casts that truncate toward zero and run
  the copied positive and negative truncation tests.
- Parser and serializer. Run prefix-tolerance, child-order, and raw-subtree
  round-trip tests for the copied XML modules.

## Hash harness

Expected to remain unchanged. Any delta blocks the sprint.

## Implementation checklist

- [ ] Apply the approved version and publication boundary to `oxml-core` and
      add the workspace dependency.
- [ ] Stage the shared source and existing tests without changing behaviour.
- [ ] Add the shared XML helper module and remove its internal duplication.
- [ ] Make `xml_text` public and add the specified event coverage.
- [ ] Prove the new crate has no dependency edge into either format family.
- [ ] Run the focused crate, packaging, workspace-check, and hash gates.

## Open questions

Resolved. Keep `oxml-core` at 0.0.0 with publishing disabled. Carry F-015 and
F-016 so no published rdocx package depends on the unpublished implementation.
Temporary source duplication remains until those carried facade stories land.
