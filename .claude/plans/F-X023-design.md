# F-X023, Tag v0.7.0

**Status**: completed
**Sprint**: S42
**Size**: S
**Depends on**: F-X022

## Problem

S41 broke the stable family's public API rather than merely extending it, so a
0.x minor bump is the correct response and the train moves 0.6.0 to 0.7.0.

- `rdocx-oxml` added `note_type` to `CT_Footnote`, six fields to `CT_Anchor`
  and four variants to `WrapType`. Each breaks an exhaustive match or a struct
  literal.
- `rdocx-layout` added `reflow` and `content_offset_top` to `ParagraphBlock`,
  and seven fields to `AnchoredDrawing`.

The `rdocx` facade's own API is unchanged. `Document::footnotes()` still returns
`Vec<(i32, String)>` and `RunRef::footnote_id()` is untouched, so a consumer of
the facade alone sees no break. It moves with its train regardless, because the
eleven workspace-version packages share one version.

## Spec reference

- `docs/hld/15-build-and-toolchain.md`, for the publish workflow's stable
  predicate and its exact seven-package set.
- `docs/hld/10-bindings-spec.md`, for the Python and WASM packages that inherit
  a version without gaining publication authority.
- `docs/hld/14-development-backlog.md`, "F-X023, Tag v0.7.0".

## Approach

The stable train inherits, so the version itself moves in one line. The
inventory, taken rather than assumed:

- **`[workspace.package].version`**, one line, which carries all eleven
  workspace-version packages.
- **9 root `[workspace.dependencies]` pins** at `version = "0.6.0"`.
- **2 Python project versions**, `rdocx-py` and `rpptx-py`.
- **The `rdocx-wasm` contract literals** asserting the `rdocx` and
  `rdocx-layout` pins.
- **The `ci.yml` `verify_package` literal** for `@tensorbee/rdocx-wasm`.
- **`test_stable_release_family_is_prepared_at_0_6_0`**, renamed to `..._0_7_0`
  with its expectations moved, and `publish.yml` updated to invoke the new name.
- **`Cargo.lock`**, regenerated rather than hand-edited.

The incubating train stays at 0.3.0, where F-X022 left it.

**Learned from F-X022.** That story moved every carrier under `crates/` and
stopped, missing the Python release regression that `publish.yml` runs as its
gate and the `ci.yml` WASM literal. Neither `cargo test` nor `/verify` runs the
Python suite, so the gap passed every local gate and would have failed in CI at
publication. This story runs `python3 -m unittest scripts.test_sprint_workflow`
as part of its own gate rather than trusting `/verify` to cover it.

## Rejected alternatives

- **A patch bump to 0.6.1.** The API broke. A caller resolving 0.6.1 and
  matching exhaustively on `WrapType` would fail to compile.
- **Bump only the seven published crates.** The eleven share one workspace
  version. Splitting them would mean eleven literals and the same problem the
  incubating train has.
- **Publish the Python or WASM packages.** Outside the seven-package set by
  standing decision, and this story does not revisit it.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_stable_release_family_is_prepared_at_0_7_0` | The eleven-package train, nine pins, lock entries, Python versions, WASM literals and exact seven-package publication set all read 0.7.0 |
| regression | the full workspace suite | Nothing resolves against a stale version |
| golden | `python3 scripts/hash_harness.py --check` | A version bump moves no rendered output |

**Test gate**, from the backlog: the stable release regression, the patched
workspace dry run with archives under 10 MiB, README compilation, `cargo deny`,
and 28 unchanged hashes.

## HLD impact

None. Versions live in manifests and the tracker, not the spec set.

## Risk routing

Matched row: **Release scripting, version strings**.

- Inspect every manifest, lockfile and README version diff. The inventory above
  is that inspection.
- Require a clean full gate and a separate final approval before tagging. The
  approval belongs to `/release` at the reviewed SHA, and this story does not
  pre-authorise it.

## Hash harness

**Expected unchanged.** A version string reaches no rendered byte.

## Release boundary

**Test gate**: deferred to `/release v0.7.0`.

Preparation is complete and reviewed. The gate this story is measured by needs
real publication, so it stays open until `/release v0.7.0` verifies every
registry version and the GitHub release. Per the release-preparation exception
in `.claude/commands/complete-feature.md`, this plan stays `approved`, the F-ID
stays `reviewed` in the run state and `in-progress` in both delivery trackers,
and no AS_BUILT entry is written yet.

## Implementation checklist

- [x] Record the pre-change harness state
- [x] `[workspace.package].version` to 0.7.0
- [x] 9 root pins to 0.7.0
- [x] 2 Python project versions
- [x] `rdocx-wasm` contract literals and the `ci.yml` WASM literal
- [x] Rename the stable release regression and update `publish.yml`
- [x] Regenerate `Cargo.lock`
- [x] Full suite, harness, README doctests, `cargo deny`, and the Python
      release regressions
- [x] `/microscope F-X023 --working`
- [x] `/verify`
- [x] `/release v0.7.0`, published and verified at `ab52cd2`

## Open questions

None.
