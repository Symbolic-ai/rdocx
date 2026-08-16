# F-X022, Tag rpptx-v0.3.0

**Status**: approved
**Sprint**: S42
**Size**: S
**Depends on**: F-X024

## Problem

S41 broke the incubating family's public API rather than merely extending it.
`oxml-layout` renamed `TextSegment::footnote_id` and `GlyphRun::footnote_id` to
`note` and changed the type from `Option<i32>` to `Option<NoteRef>`, and added
`line_prefix_widths` and `line_suffix_widths` to `LineBreakParams`. Under semver
a 0.x minor bump is the correct response, so the train moves 0.2.0 to 0.3.0.

F-X024 is what makes this releasable at all. Before it, `oxml-drawing` depended
on `rdocx-oxml`, so the two publication trains were mutually dependent and
neither could go first. That edge is gone, and the incubating train now depends
on nothing in the stable train, so it publishes first and unconditionally.

## Spec reference

- `docs/hld/15-build-and-toolchain.md`, for the publish workflow's incubating
  predicate and its exact package set.
- `docs/hld/10-bindings-spec.md`, for the WASM package that inherits a version
  without gaining publication authority.
- `docs/hld/14-development-backlog.md`, "F-X022, Tag rpptx-v0.3.0".

## Approach

Move every carrier of the incubating version together. The inventory, taken
rather than assumed:

- **15 crate manifests** with a literal `version = "0.2.0"`: the eight `oxml-*`
  publishable crates, `rpptx`, `rpptx-cli`, `rpptx-chart`, `rpptx-layout`,
  `rpptx-oxml`, `rpptx-render` and `rpptx-wasm`.
- **14 root `[workspace.dependencies]` pins** at `version = "0.2.0"`.
  `rpptx-wasm` has no pin because nothing depends on it.
- **11 crate READMEs** quoting the version in a dependency example.
- **7 Rust sources** asserting the version or a pin string, in `oxml-drawing`,
  `rpptx-chart`, `rpptx-render`, `rdocx-wasm`, `rpptx-wasm`, and the `rpptx` and
  `rpptx-oxml` integration tests.
- **`Cargo.lock`**, regenerated rather than hand-edited.

The stable train stays at 0.6.0 in this story. Its pins on the incubating crates
move to 0.3.0, because the stable code depends on the new `oxml-layout` API and
must resolve against 0.3.0 once it is published.

`rpptx-wasm` moves to 0.3.0 and stays `publish = false`. The published set is
exactly fourteen.

## Rejected alternatives

- **A patch bump to 0.2.1.** The API broke. A patch bump would let a caller
  resolve 0.2.1 expecting 0.2.0's `footnote_id` and fail to compile.
- **Hold the stable pins at 0.2.0.** They would resolve against a published
  `oxml-layout` whose `TextSegment` has no `note` field, so `rdocx-layout` would
  not build.
- **Publish `rpptx-wasm`.** Outside the fourteen-package set by standing
  decision, and this story does not revisit it.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | the existing manifest assertions | The seven Rust sources that pin the version string agree with the manifests, which is what catches a partial bump |
| regression | the full workspace suite | Nothing resolves against a stale version |
| golden | `python3 scripts/hash_harness.py --check` | A version bump moves no rendered output |

**Test gate**, from the backlog: the incubating release regression, the patched
workspace dry run with archives under 10 MiB, README compilation, `cargo deny`,
and 28 unchanged hashes.

## HLD impact

None. Versions are recorded in manifests and the tracker, not in the spec set.

## Risk routing

Matched row: **Release scripting, version strings**.

- Inspect every manifest, lockfile and README version diff. The inventory above
  is that inspection, and completion states the counts actually changed.
- Require a clean full gate and a separate final approval before tagging. The
  approval is `/release`'s, at the reviewed SHA, and this story does not
  pre-authorise it.

The layout row does not match: no rendering code is touched and the harness must
stay flat.

## Hash harness

**Expected unchanged.** A version string reaches no rendered byte. A delta would
mean something other than a version moved.

## Implementation checklist

- [x] Record the pre-change harness state
- [x] 15 manifests to 0.3.0
- [x] 14 root pins to 0.3.0
- [x] 11 README dependency examples
- [x] 7 Rust sources asserting the version or a pin string
- [x] Regenerate `Cargo.lock`
- [x] Full suite, harness, README doctests, packaging dry run, `cargo deny`
- [x] `/microscope F-X022 --working`
- [x] `/verify`
- [ ] Stop. `/release rpptx-v0.3.0` is a separate command needing separate
      approval at the reviewed SHA

## Release boundary

**Test gate**: deferred to `/release rpptx-v0.3.0`.

Preparation is complete and reviewed. The gate this story is measured by needs
real publication, so it stays open until `/release rpptx-v0.3.0` verifies every
registry version and the GitHub release. Per the release-preparation exception
in `.claude/commands/complete-feature.md`, this plan stays `approved`, the F-ID
stays `reviewed` in the run state and `in-progress` in both delivery trackers,
and no AS_BUILT entry is written yet.

## Open questions

None.
