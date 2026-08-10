# S31 sprint review, pass 2

**Reviewed**: `sprint/s31` against merge base
`eab8f708709d6f2c734340574bb39ea30e078e34`, 33 files and 21,436 changed
lines, crates: `oxml-opc`, `rpptx-chart`, `rpptx-oxml`, `rpptx`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M12 end gate is: "a chart created by rpptx opens in PowerPoint, its data is
editable, and it renders" at `docs/hld/14-development-backlog.md:915`.

The open and edit portions hold. The SHA-bound candidate and PowerPoint 16.104
build 16.104.25121423 observation record a clean open and the exact Edit Data
values at `docs/hld/09-charts-spec.md:435`.

The S31 rendering boundary now has direct integrated evidence. The remediating
test creates the bar chart through `Presentation::add_chart`, extracts and
parses that authored chart part, and passes it to `render_chart` with two fresh
deterministic font managers at `crates/rpptx/tests/integration.rs:436`. It
asserts repeatable output at `crates/rpptx/tests/integration.rs:457`, finite and
nonempty paths and text at `crates/rpptx/tests/integration.rs:465`, and minimum
path and label counts at `crates/rpptx/tests/integration.rs:495`. The pass 2
rerun of `authored_chart_enters_renderer_deterministically` passed. The current
hash-harness rerun also matched all 28 entries.

The end-of-milestone gate does not yet hold through the presentation rendering
pipeline. `rpptx-layout` still routes chart frames to unsupported content at
`crates/rpptx-layout/src/context.rs:512`, and HLD 09 assigns relationship
resolution and native render routing to pending F-128 at
`docs/hld/09-charts-spec.md:549`. This is the documented S32 boundary and does
not contradict the S31 definition of done at
`docs/sprints/CURRENT_SPRINT.md:47`.

## Not found

- Interaction: S1 from pass 1 is closed. The authored package output now
  enters the parser and labelled renderer in one deterministic test, including
  finite nonempty paths and labels. No conflicting package, geometry, scale,
  label, or legend behavior was found across F-124 through F-126.
- Duplication: no second chart model or duplicate sprint helper was introduced.
  F-124 authors the existing `rpptx-chart` model and F-125 and F-126 share one
  geometry entry point and plot rectangle.
- Layering: the added normal edges are `rpptx` to `oxml-sml` and
  `rpptx-chart`, plus `rpptx-chart` to `oxml-layout`. No `oxml-*` crate gained
  an edge to `rdocx-*` or `rpptx-*`.
- Harness: all three plans declare no delta, every AS_BUILT entry records no
  delta, and the pass 2 rerun matched all 28 deterministic entries.
- Gate: the three S31 story gates have direct test or native viewer evidence.
  The remaining M12 pipeline boundary is explicitly assigned to F-128.
- Docs: HLD 03 and HLD 09 match the authored package graph, dependency edge,
  renderer behavior, deterministic font boundary, and deferred F-127 and F-128
  work.
- Deps: every new normal dependency has a named current consumer. The new
  `oxml-pdf` and `tiny-skia` edges are test-only consumers of chart raster and
  viewer gates.
- Surface: `ChartKind`, `ChartData`, `Presentation::add_chart`,
  `ChartGeometry`, `render_geometry`, and `render_chart` are the public surfaces
  approved by F-124 through F-126. No unrelated trait, generic, feature, crate,
  or module was added.
