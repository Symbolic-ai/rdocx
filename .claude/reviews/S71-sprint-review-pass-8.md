# S71 sprint review, pass 8

**Reviewed**: full integrated dependency prefix on `sprint/s71` at
`fd74551d62e985b56b1eff8a8f31f734242d50e2` against merge base
`fd069fd460fec11b27c2f6eb1004d4a9ee9bcade`, 123 files and 42,714 changed
lines, comprising 34,636 additions and 8,078 deletions, crates: `oxml-chart`,
`oxml-drawing`, `oxml-media`, `oxml-opc`, `rdocx-html`, `rdocx-layout`,
`rdocx-oxml`, `rdocx`, and `rpptx`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

**Bound extension**: scheduled dependency-prefix boundary. Passes 5 through 7
closed the earlier F-246 boundary. F-247 then enlarged the integrated prefix,
with F-X087 and F-X088 evidence also present at this HEAD. Global pass numbering
exceeds the configured bound only because earlier dependency prefixes reached
clean closure before this new boundary.

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M23 end gate requires all five private references to be generated from a
blank public facade, reopen without repair, match the required package
semantics and reviewed deterministic visual thresholds, produce identical DOCX
bytes on repeated generation, and report no unexplained preservation-only
fallback (`docs/hld/14-development-backlog.md:2214`). The gate does not yet hold
at this dependency-prefix boundary. F-248 remains pending
(`docs/sprints/CURRENT_SPRINT.md:45`), so style-linked numbering, counters, TOC,
and REF completion is outside this reviewed prefix. This pass does not assert
the five-reference end gate or the complete sprint definition of done.

The completed prefix has concrete evidence. F-247 records full numbering
round-trip, schema-order, modeled-property, atomicity, and byte-preservation
tests plus a full verification gate (`docs/sprints/AS_BUILT.md:12591`). This
pass directly reran the atomic invalid-mutation, style-reference removal, and
producer-extension preservation cases. It also exercised the authored line,
bar, pie, and doughnut chart cases, all ten `word_chart_` tests, the shared
`RgbColor` facade assertion, and the equivalent-construction-order identifier
regression. All 17 selected tests passed.

F-X087's source-built chart gate covers complete staged identifiers, theme
reuse and replacement, failure atomicity, editable workbooks, and all three
facade exports (`docs/hld/12-testing-strategy.md:693`). Its Word and Pages
candidate and semantic evidence are bound to the declared digest
(`docs/hld/14-development-backlog.md:4568`). F-X088 records all six focused
regressions, the complete workspace gate, 49 unchanged hashes, authenticated
reporter credit, and the closed Issue 69 evidence
(`docs/hld/14-development-backlog.md:4608`).

The repository sprint state records the full verification as passed at the
reviewed HEAD. In this pass, formatting, prose, generated-skill drift, and diff
checks passed. The deterministic hash harness regenerated the seven samples and
matched all 49 reviewed entries.

## Not found

- **Interaction**: F-247 validates definition, instance, style, direct
  paragraph, and related-story references in one candidate graph
  (`crates/rdocx/src/document.rs:8986`). Numbering merge remapping remains on
  the shared identifier owner (`crates/rdocx/src/document.rs:10823`). F-X087
  stages chart, workbook, drawing, theme, relationships, and identifiers before
  one commit (`crates/rdocx/src/document.rs:6408`). No cross-story partial state
  or conflicting owner was found.
- **Duplication**: numbering mutations share the existing bundle reservation,
  graph validation, and staged commit paths. Chart authoring and typed chart
  insertion share one package assembly path. No competing serializer,
  identifier allocator, or graph validator was introduced.
- **Layering and dependencies**: no manifest or lockfile changed. No `oxml-*`
  crate gained an `rdocx-*` or `rpptx-*` dependency, and the shared colour type
  remains owned by `oxml-drawing` and re-exported by `oxml-chart`
  (`crates/oxml-chart/src/lib.rs:29`).
- **Harness**: all 49 entries match. The only sprint baseline changes remain
  the two declared F-249 document XML entries, with the other 47 unchanged
  (`docs/sprints/AS_BUILT.md:12420`). F-247 and F-X087 each declare no further
  harness delta (`docs/sprints/AS_BUILT.md:12602`,
  `docs/hld/14-development-backlog.md:4580`).
- **Gate**: the selected cross-story tests passed, including deterministic
  chart allocation, numbering preservation, style-reference rejection, and
  equivalent construction-order package identity. The recorded full gate at
  this HEAD is clean.
- **Documentation**: current sprint status correctly leaves F-248 pending and
  the two scope exceptions in progress (`docs/sprints/CURRENT_SPRINT.md:45`).
  F-247 completion and both exception evidence records match the implemented
  prefix without claiming the M23 end gate.
- **Surface**: F-247's additive numbering API matches its accepted contract.
  F-X087 exposes the planned chart options and the same `RgbColor` through the
  three native facades (`crates/rdocx/src/lib.rs:81`,
  `crates/rpptx/src/lib.rs:22`). No unplanned binding surface or speculative
  abstraction was found.
