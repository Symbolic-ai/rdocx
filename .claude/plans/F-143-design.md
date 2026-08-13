# F-143, oxml-cli-support

**Status**: completed
**Sprint**: S36
**Size**: S
**Depends on**: none

## Problem

The planned Word and presentation command-line tools need identical range
parsing, default output-path selection, and versioned JSON envelopes. No
format-neutral owner exists today. Leaving these rules in each binary would
create two subtly different user contracts.

The existing `rdocx-cli` already implements its own output default and emits
unversioned inspect JSON. F-143 must establish the shared contract and migrate
those two existing consumers without changing its flags or its established
zero-based `render --page` option.

## Spec reference

- `docs/hld/03-architecture.md`, "Three families, one workspace".
- `docs/hld/10-bindings-spec.md`, "CLI".
- `docs/hld/14-development-backlog.md`, "F-143, oxml-cli-support".
- `docs/hld/15-build-and-toolchain.md`, package and publication order.

## Approach

Create publishable, currently unpublished `oxml-cli-support` 0.1.2 with three
concrete helpers:

- `parse_range(&str) -> Result<Vec<usize>>` accepts positive one-based
  comma-separated values and inclusive ranges, then sorts and deduplicates.
  It rejects zero, empty components, descending ranges, and malformed input.
- `default_output_path(&Path, &str) -> PathBuf` replaces or adds the requested
  extension without changing the parent or multi-dot stem.
- A JSON-envelope helper accepts an object, adds the reserved top-level
  `"schema": 1`, preserves sibling fields, and rejects non-objects or a caller
  supplied `schema` key.

Migrate only the existing rdocx inspect JSON and convert default output path to
these helpers. Keep the range helper unused there until a real Word command
needs one. Add no trait, generic abstraction, builder, feature flag, or serde
model.

## Rejected alternatives

- Copy helpers into both CLIs. That creates two public contracts.
- Add a trait or generic command framework. There are no two concrete command
  engines that need abstraction.
- Change existing rdocx page indexing to one-based. F-143 does not own that
  compatibility change.
- Use zero-based range syntax. User-facing slide selection follows the
  established one-based convention.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit, gate | `range_2_4_through_6_is_the_expected_set` | `2,4-6` is exactly `[2, 4, 5, 6]` |
| unit | range rejection matrix | Zero, empty, descending, and malformed components fail while valid duplicates are sorted and deduplicated |
| unit | JSON envelope matrix | Schema is exactly 1, payload fields survive, and reserved-key collisions and non-objects fail |
| unit | output path matrix | Relative, extensionless, and multi-dot input paths receive the requested extension |
| integration | rdocx shared-contract regression | Existing inspect JSON has top-level schema 1 and default convert output uses the helper without changing flags |

Sensitivity changes the inclusive range end and schema literal independently,
proves the named gates fail, restores both sources byte-identically, and reruns
green.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- New crate and files. Obtain explicit approval for
  `crates/oxml-cli-support/Cargo.toml` and
  `crates/oxml-cli-support/src/lib.rs`.
- Crate dependency graph. Read HLD03 and prove the shared crate has no rdocx or
  rpptx dependency while `rdocx-cli` points inward to it.
- Public API and publication metadata. State the initial additive surface, run
  the exact patched publication dry run and archive-size check, and inspect
  manifests, lockfile, release-family membership, and tag template.

## Hash harness

Expected unchanged. The helpers do not participate in document generation or
rendering.

## Implementation checklist

- [x] Create the approved shared crate and wire workspace metadata.
- [x] Implement range, output-path, and JSON-envelope helpers.
- [x] Migrate only the existing rdocx inspect JSON and convert output default.
- [x] Add focused compatibility and mutation-sensitive regressions.
- [x] Run dependency, publication, workflow, and hash riders.

## Open questions

None. The two exact new crate paths and future published incubating 0.1.2
metadata are approved. This prepares a release path but does not publish
anything.
