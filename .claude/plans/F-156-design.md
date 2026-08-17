# F-156, Extract oxml-chart

**Status**: completed
**Sprint**: S45
**Size**: L
**Depends on**: none

## Problem

The format-neutral ChartML model and renderer still ship from the
PresentationML family. The workspace registers `crates/rpptx-chart` and its
dependency at `Cargo.toml:15` and `Cargo.toml:65`, while the crate manifest at
`crates/rpptx-chart/Cargo.toml:2` names and describes it as PowerPoint-specific.
The implementation itself depends only on shared `oxml-*` crates at
`crates/rpptx-chart/Cargo.toml:19`, so Word cannot consume the chart engine
without taking a misleading format-family dependency.

The current direct consumers in `crates/rpptx/src/lib.rs:39` and
`crates/rpptx-layout/src/context.rs:31` also bind the old name into the active
implementation. The extraction must change ownership and paths without
changing a single parser, serializer, geometry, or rendered byte.

## Spec reference

- `docs/hld/03-architecture.md`, "Three families, one workspace", "The
  dependency rule", and "Why these seams".
- `docs/hld/04-opc-and-packaging.md`, deterministic saves and XML preservation.
- `docs/hld/09-charts-spec.md`, "The ChartML model" and "Rendering".
- `docs/hld/11-migration-plan.md`, "Release tooling".
- `docs/hld/12-testing-strategy.md`, "The hash harness", "The deck corpus",
  and "The render fidelity gate".
- `docs/hld/14-development-backlog.md`, "F-156, Extract oxml-chart".
- `docs/hld/15-build-and-toolchain.md`, publishing order, package checks, and
  pinned viewer gates.

## Approach

Move the complete implementation, README, and tests from `crates/rpptx-chart`
to a new published `crates/oxml-chart` package at the existing incubating
version. Register `oxml-chart` as the shared workspace dependency and update
every active consumer, release-order list, package assertion, README doctest,
and repository path assertion to use the new crate directly.

Update the exact incubating publish allowlist and dependency-order paragraph
to publish `oxml-chart` before its deprecated `rpptx-chart` shim.

Reduce `rpptx-chart` to the established deprecated compatibility shape:

```rust
#![doc = include_str!("../README.md")]

pub use oxml_chart::*;
```

Its manifest keeps the existing `rpptx-chart` package identity and version,
changes the description to `deprecated: moved to oxml-chart`, and depends only
on `oxml-chart`. Add one exact type-identity regression through the shim. Keep
all implementation tests under `oxml-chart` and rename only path-derived test
labels and temporary prefixes where the old package name would otherwise be
false.

Treat the move as mechanical. Do not alter public items, XML bytes, validation,
geometry, defaults, or tests except for crate paths and ownership assertions.

## Rejected alternatives

- Let Word depend on `rpptx-chart`. The code is shared, and leaving the old
  family name as the owner preserves the dependency lie this story removes.
- Delete `rpptx-chart`. It is a published crate, so removal would break current
  consumers instead of giving them a migration path.
- Refactor the 15,000-line crate during the move. The story forbids combining
  behavior change with a file move, and the hash result would no longer isolate
  ownership from behavior.
- Split the implementation into new modules. The extraction needs one new
  owner, not a structural rewrite.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `python3 scripts/hash_harness.py --check` | All 49 entries remain byte-identical across the move |
| regression | `cargo test -p oxml-chart` | Every existing chart model, geometry, raster, corpus, and viewer test runs through the new crate path |
| integration | `legacy_shim_retains_shared_chart_type` | A public `rpptx_chart` type is exactly accepted as the corresponding `oxml_chart` type |
| dependency | chart dependency assertions | Active consumers depend on `oxml-chart`, the shim depends only on it, and no shared crate gains a format-family edge |
| packaging | package dry-runs and archive inspection | Both chart packages contain the right README and source, resolve registry dependencies, and remain below 10 MiB |
| regression | repository path assertions | Release order, README doctests, manifests, scripts, and documentation contain no stale active-owner path |

The test gate is regression. The hash harness is byte-identical across the
move, and every existing chart test passes against the new path.

## HLD impact

- `docs/hld/01-glossary.md`
- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/07-inheritance-and-resolution.md`
- `docs/hld/09-charts-spec.md`
- `docs/hld/11-migration-plan.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

Replace active ownership and dependency references with `oxml-chart`. Keep the
deprecated `rpptx-chart` shim explicit where migration or release ordering
needs to describe it.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Run the complete existing
  prefix, child-order, round-trip, and raw-subtree preservation coverage from
  the new crate without changing implementation bodies.
- Crate dependency graph and new cross-family uses. Read HLD 03. Run the
  architecture dependency check and inspect `cargo tree -p oxml-chart --edges
  normal` plus `cargo tree -p rpptx-chart --edges normal`.
- Public API of a published crate. Read HLD 10 and the structural rules. State
  the additive `oxml-chart` surface and the source-compatible deprecated shim,
  then run both package dry-runs and size assertions.
- A new crate, module, or file. The explicit F-156 story and this sprint
  invocation authorize the new `oxml-chart` manifest, README, and crate root.
  No new trait, generic parameter, or module is introduced.
- Release scripting and version strings. Read the release command and HLD 15.
  Inspect every manifest, lockfile, README, release-order assertion, and
  package list affected by the rename. Do not tag or publish.
- A file move with no behaviour change. Require the 49-entry hash harness and
  existing exact chart raster tests to remain unchanged. Any delta blocks the
  sprint and is not recorded as a new baseline.

## Hash harness

Expected unchanged across all 49 entries. Any package, PDF, or PNG delta is a
defect in this mechanical extraction and blocks integration.

## Implementation checklist

- [x] Move the complete chart implementation and tests to `oxml-chart`.
- [x] Register the new shared dependency and point active consumers at it.
- [x] Reduce `rpptx-chart` to a deprecated exact re-export with a type-identity test.
- [x] Update release, package, README, doctest, and repository path assertions.
- [x] Run focused chart, dependency, package, and unchanged-output checks.
- [x] Update exactly the listed HLD files.

## Open questions

None. The story explicitly authorizes the new shared crate and fixes the
compatibility-shim pattern. Behavior, public item names, version, and output
are held constant.
