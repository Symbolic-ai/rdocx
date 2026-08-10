# S31 sprint review, pass 1

**Reviewed**: `sprint/s31` against merge base
`eab8f708709d6f2c734340574bb39ea30e078e34`, 32 files and 21,289 changed
lines, crates: `oxml-opc`, `rpptx-chart`, `rpptx-oxml`, `rpptx`
**Verdict**: 0 blocking, 1 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

### S1, authored charts never enter the new renderer in an integrated test

`crates/rpptx/tests/integration.rs:390`

The all-family authoring test stops after finding the expected plot element and
validating the package. The geometry gate separately renders a hand-written
ChartML fixture at `crates/rpptx-chart/src/lib.rs:10451`. No test extracts the
chart part produced by `Presentation::add_chart`, parses it, and passes it to
`render_chart` with deterministic fonts. A change in authored axis, cache, or
legend output can therefore become incompatible with the renderer while every
F-124, F-125, and F-126 gate remains green. Add one integrated deterministic
test that renders the authored bar candidate, or all seven authored families,
and asserts finite nonempty output and the expected raster evidence.

## Nice-to-have

None.

## Milestone gate

The M12 end gate is: "a chart created by rpptx opens in PowerPoint, its data is
editable, and it renders" at `docs/hld/14-development-backlog.md:915`.

The open and edit portions hold. The SHA-bound candidate and PowerPoint 16.104
build 16.104.25121423 observation record a clean open and the exact Edit Data
values at `docs/hld/09-charts-spec.md:435`. The feature review independently
binds that observation to the ignored candidate generator at
`.claude/reviews/F-124-all-pass-3.md:22`.

The S31 rendering components hold at their current boundary. Local review
reruns passed `bar_chart_rasterises_at_computed_positions`,
`zero_to_one_hundred_axis_uses_expected_ticks`, and
`labelled_chart_raster_is_deterministic`. The integrated run state records a
passing full verification and unchanged harness at feature-complete head
`e98bc074fd892e2d3a8a587038385aeeef8789a3` at
`.claude/scratch/S31-run.json:49`. The current source rerun also reports all 28
hash entries unchanged.

The end-of-milestone gate does not yet hold through the presentation rendering
pipeline. `rpptx-layout` still routes chart frames to unsupported content at
`crates/rpptx-layout/src/context.rs:512`, and the HLD assigns relationship
resolution and native render routing to pending F-128 at
`docs/hld/09-charts-spec.md:548`. This is expected for S31 because F-127 and
F-128 remain pending in S32. It is not a defect in the S31 contract.

## Not found

- Interaction: no conflicting geometry, scale, label, or package behavior was
  found beyond the missing integrated authoring-to-rendering test in S1.
- Duplication: no sprint-level duplicate helper or second chart model was
  introduced. F-124 consumes the existing `rpptx-chart` and `oxml-sml` types.
- Layering: Cargo metadata shows no new forbidden `oxml-*` dependency on an
  `rdocx-*` or `rpptx-*` crate. The new normal edges point from `rpptx` to
  `oxml-sml` and `rpptx-chart`, and from `rpptx-chart` to `oxml-layout`.
- Harness: every design declared no delta, the integrated record says
  unchanged, and the current rerun matched all 28 entries.
- Gate: the three S31 story gates and full integrated gate have evidence. The
  remaining M12 pipeline work is explicitly assigned to S32.
- Docs: HLD 03 and HLD 09 describe the new dependency edge, authoring package
  graph, renderer boundary, placeholder palette, and deferred F-127 and F-128
  work without contradiction.
- Deps: all new normal dependencies have named current consumers. New raster
  and PDF dependencies are test-only consumers of deterministic chart gates.
- Surface: `ChartKind`, `ChartData`, `Presentation::add_chart`,
  `ChartGeometry`, `render_geometry`, and `render_chart` are the public surfaces
  requested by F-124 through F-126. No extra trait, generic, feature, crate, or
  module was added.
