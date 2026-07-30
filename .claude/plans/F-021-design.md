# F-021, Zip-slip hardening tests

**Status**: completed
**Sprint**: S04
**Size**: S
**Depends on**: F-018

## Problem

The package reader currently copies each ZIP entry name directly into its raw
part map at `crates/rdocx-opc/src/package.rs:57`, then only adds a leading slash
when it builds `OpcPackage::parts` at `crates/rdocx-opc/src/package.rs:100`.
An entry named `../../etc/passwd` therefore remains outside the canonical OPC
part-name shape, and an absolute entry is retained through a separate path.

Relationship targets already use `normalize_part_name` at
`crates/rdocx-opc/src/package.rs:202`, including a root-clamping assertion at
`crates/rdocx-opc/src/package.rs:335`. F-018 moves that implementation into
`oxml-opc`, but its planned moved tests do not exercise hostile archive entry
names at the ZIP boundary.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "The package" and "What transfers
  unmodified".
- `docs/hld/12-testing-strategy.md`, "New tests the extracted crates need",
  subsection `oxml-opc`.
- `docs/hld/14-development-backlog.md`, "F-021, Zip-slip hardening tests".

## Approach

After F-018 establishes `oxml-opc`, normalize every non-directory ZIP entry
name before inserting it into the raw package map. Reuse the package's existing
root-clamping path normalization, then store the archive key without a leading
slash so `[Content_Types].xml` and relationship paths retain their current
root-relative lookup shape. The public `parts` map continues to expose exactly
one leading slash.

Add code-built in-memory ZIP fixtures to the existing `package.rs` test module.
One fixture contains `../../etc/passwd` and asserts the part is available only
as `/etc/passwd`. The other contains an absolute entry and asserts it is
available through the same canonical root-relative package path. Do not extract
anything to the filesystem, change collision handling, or broaden validation
beyond the two backlog cases.

## Rejected alternatives

- Reject every entry containing `..`. The backlog gate explicitly requires an
  over-root traversal to be clamped, and valid relative OPC paths use parent
  segments elsewhere.
- Normalize only relationship targets. That leaves archive entry keys in a
  different namespace from the relationships that resolve to them.
- Use host filesystem canonicalization. OPC part names are package paths, and
  their interpretation must not depend on the operating system or a real
  filesystem.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `zip_entry_that_escapes_root_is_clamped_to_root` | A code-built ZIP entry named `../../etc/passwd` is present as `/etc/passwd` and no traversal-shaped key survives |
| regression | `absolute_zip_entry_is_normalized_to_package_root` | A code-built absolute entry is exposed through one canonical leading-slash part name |
| round-trip | existing `round_trip_package` | Ordinary opaque part bytes still survive package write and read unchanged |

The backlog test gate is that `../../etc/passwd` is clamped to the root and an
absolute entry is normalized.

## HLD impact

None. The OPC specification and testing strategy already require canonical
leading-slash part names and the two hardening cases.

## Risk routing

- Package parser entry point. Run all `oxml-opc` tests, including existing
  content-types, relationships, deterministic-save, and opaque-part round-trip
  coverage. Confirm that no XML parser or serializer changes and that the
  code-built hostile entries never reach the filesystem.

## Hash harness

Expected to remain unchanged. F-021 changes only the unpublished `oxml-opc`
archive reader before any rdocx consumer switches to it. Any delta blocks the
sprint.

## Implementation checklist

- [x] Wait for F-018 to establish `oxml-opc` and its moved package tests.
- [x] Normalize ZIP entry names before raw package classification.
- [x] Add the root-escaping code-built ZIP fixture and regression assertion.
- [x] Add the absolute-entry code-built ZIP fixture and regression assertion.
- [x] Run focused package tests, the integrated package round-trip, and the
      unchanged hash gate.

## Open questions

None. The draft assumes F-018 preserves the current root-clamping helper and
the sprint wave enforces the dependency before F-021 starts.
