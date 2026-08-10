# F-125, all, pass 1

**Reviewed**: uncommitted working diff, 5 files and 1,133 changed lines
**Verdict**: 2 defects, 1 smell, 0 nitpicks

## Defects

### D1, sparse cache indexes are collapsed into dense positions

`crates/rpptx-chart/src/lib.rs:543`
`crates/rpptx-chart/src/lib.rs:404`

The geometry code derives the category count from `values.len()` and addresses
each value by its vector position. Parsed ChartML caches may legally have a
larger declared `c:ptCount` with omitted blank points, and the model preserves
those `c:pt/@idx` values. A cache with indexes 0 and 2 is therefore rendered in
slots 0 and 1 instead of slots 0 and 2. Scatter geometry also zips the dense x
and y vectors, so two equally sized sparse caches with different indexes pair
unrelated coordinates. Bars, lines, areas, radar polygons, and scatter points
can all be placed at the wrong category or paired with the wrong value.

### D2, family tests do not prove the planned coordinate calculations

`crates/rpptx-chart/src/lib.rs:8055`
`crates/rpptx-chart/src/lib.rs:8105`
`crates/rpptx-chart/src/lib.rs:8140`

The approved test plan requires hand-computed coordinates for bar direction,
grouping, gap, and overlap, ordered coordinates for line, scatter, and radar,
and exact baselines, angles, and radii for area, pie, and doughnut geometry.
The added tests mostly assert relative ordering, child counts, command kinds,
and closure. Ignoring the doughnut hole size or first-slice angle, using the
wrong area baseline, or mapping scatter and radar points to wrong coordinates
would still pass. The sparse-cache failure above is also uncovered. These
tests do not lock down the calculations that make up most of the story.

## Smells

### S1, pipeline wiring is assigned to an unrelated story

`.claude/plans/F-125-design.md:71`
`docs/hld/14-development-backlog.md:987`

The approved plan excludes chart relationship extraction by assigning it to
F-129, but F-129 is `oxml-py-support` in the canonical backlog. F-126 owns axes,
F-127 owns colours, and F-128 owns preserved fallbacks. None of those story
definitions owns resolving a chart relationship and placing supported geometry
into the native slide render input. Unless that ownership is corrected or a
story is filed, this concrete entry point can remain unreachable from normal
presentation rendering at the milestone gate.

## Nitpicks

None.

## Not found

- Panics: no unguarded production `unwrap`, `expect`, slice, index, or
  arithmetic panic path was found after accounting for established invariants.
- OOXML: no parser or serializer behavior changed, and no namespace,
  preservation, or schema child-order defect was found in this diff.
- Structure: no new trait, generic, wrapper, feature, crate, module, or source
  file was introduced. The concrete dependency points in the allowed
  format-specific to format-neutral direction.
- Correctness beyond D1: no additional wrong geometry calculation was found.
- Contract beyond S1: the implementation stays within the geometry-only scope
  and updates exactly the two approved HLD files.
- Tests beyond D2: the primary bar raster gate would fail if bar paths were
  removed, and the negative and determinism cases cover their stated basics.
- Nitpicks: none found.
