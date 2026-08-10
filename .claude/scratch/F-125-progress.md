# F-125 progress notes

## Current state

Implementation is complete. `render_geometry` lowers all seven supported plot
families into one backend-neutral group, with fixed plot margins, deterministic
series paints, exact bar grouping, stacked and percentage domains, cubic pie
and doughnut wedges, area closures, scatter mapping, radar polygons, and
markers. The six planned tests pass, including the deterministic tiny-skia
raster gate. Pass-1 remediation preserves declared sparse cache layouts for
category geometry, pairs scatter coordinates only at matching logical indexes,
and strengthens every family calculation with exact coordinate assertions.
Pass-2 remediation now applies `gap`, `zero`, and `span` blank policies to
line, area, and scatter paths, validates mutable scatter x caches, and keeps
finite extreme inputs from producing nonfinite geometry or unchecked aggregate
overflow.
The approved plan and HLD assign relationship resolution and
supported-versus-preserved routing to F-128.

## Changed areas

- `crates/rpptx-chart/src/lib.rs`
- `crates/rpptx-chart/Cargo.toml`
- `Cargo.lock`
- `docs/hld/03-architecture.md`
- `docs/hld/09-charts-spec.md`

## Last green check

`RDOCX_PPTX_CORPUS_DIR=/private/tmp/rdocx-f125-no-corpus
CARGO_TARGET_DIR=/private/tmp/rdocx-f125-pass3-tests cargo test -p
rpptx-chart --lib` passed 48 tests on 2026-08-10. The scoped clippy check passed
with warnings denied. Formatting, prose, diff, skill-sync, dependency-tree
checks, and all 28 hash-harness entries also passed.

The exact gate
`tests::bar_chart_rasterises_at_computed_positions` passed, failed when the
geometry entry point was temporarily reverted, and passed again after the
reversion was removed. Independent microscope pass 3 reports 0 defects and 0
smells.

## Blockers

None.

## Next action

Integrate the prepared worker branch with
`/integrate-feature F-125 work/f-125-codex --batch`.
