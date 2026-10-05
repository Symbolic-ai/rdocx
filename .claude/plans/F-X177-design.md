# F-X177, Accept unified fontdb source features

**Status**: completed
**Sprint**: S89
**Size**: S
**Depends on**: none

## Problem

`crates/oxml-layout/src/font.rs:2031` matches only binary and ordinary file
sources. Cargo feature unification can add `fontdb::Source::SharedFile` when
a downstream dependency such as `usvg` enables memmap, making the match
non-exhaustive. PR 269 reports that exact build failure.

## Spec reference

- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering" and "Feature flags", font construction and optional host discovery.
- `docs/hld/08-rendering-spec.md`, "The seam that makes this cheap", owned font bytes and face indices in layout.
- `docs/hld/14-development-backlog.md`, F-X177 compilation regression contract.

## Approach

Adopt the four-line fallback from PR 269 at
`7949573b92fd988a682a11b8e674af85e5df622e`, crediting `changjoon-park`.
Use `Database::with_face_data` to copy bytes and retain the selected face
index for source variants enabled through dependency feature unification.
Allow the unreachable fallback pattern when those features are absent.
The binary cache and the ordinary file cache keep their current paths.
No dependency version, default feature, public API or font asset changes.

Use the failing feature-enabled compile as the test gate. Run it before
implementation and record the exact diagnostic. Check all four default and
memmap combinations, including bundled-only construction. Refresh only the
changed crate's archive evidence if repository policy requires it.

## Rejected alternatives

- Enabling memmap by default changes every consumer's dependency features unnecessarily.
- Naming SharedFile directly cannot compile when fontdb does not expose that optional variant.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| compilation regression | `cargo check -p oxml-layout --features fontdb/memmap` | **Test gate.** Reproduces E0004 before the patch and passes after it. |
| compilation regression | default, no-default, and no-default with `fontdb/memmap` crate checks | Existing feature configurations remain supported. |
| unit | existing `oxml-layout` font tests with and without defaults, including unified memmap | Font selection, owned bytes and deterministic construction retain their tested behavior. |

## HLD impact

- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Layout, pagination, line breaking, text shaping**. Read `docs/hld/08-rendering-spec.md`. Use deterministic font mode for the hash gate and do not record a new baseline.
- Feature unification also requires `cargo test -p oxml-layout --no-default-features` and the explicit memmap compilation matrix, without introducing a new crate feature.

## Hash harness

Expected unchanged, all 49 entries. The fallback becomes reachable only for
fontdb sources introduced by unified dependency features.

## Implementation checklist

- [x] Reproduce the PR's compile failure against the claimed base.
- [x] Adopt the reviewed fallback and retain contributor provenance.
- [x] Verify default, no-default and unified memmap configurations.
- [x] Refresh scoped archive evidence and listed HLD sections.
- [x] Pass scoped verification and a zero-finding microscope.

## Open questions

None. The user explicitly requested inclusion of PR 269 in S89. Integration
uses the sprint branch and final `/close-sprint` main merge. Contributor
reconciliation follows that reviewed main push.
