# F-020, oxml-opc reads a pptx

**Status**: approved
**Sprint**: S04
**Size**: M
**Depends on**: F-019

## Problem

The package navigation code is intended to be format-neutral, but its current
tests only construct DOCX packages. `crates/rdocx-opc/src/package.rs:205`
locates the office document relationship generically, and
`crates/rdocx-opc/src/package.rs:185` resolves relative targets, but the test
module at `crates/rdocx-opc/src/package.rs:296` never proves those operations
against a PresentationML package graph.

A slide layout sits beside the `slides` directory rather than below it. Without
a code-built pptx-shaped fixture, the shared OPC extraction could preserve all
Word tests while still failing to resolve
`../slideLayouts/slideLayout1.xml` from a slide.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "The package", "What transfers
  unmodified", and "Part naming".
- `docs/hld/06-presentationml-model.md`, "Parts" and `presentation.xml`.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "New tests the
  extracted crates need", subsection `oxml-opc`.
- `docs/hld/14-development-backlog.md`, "F-020, oxml-opc reads a pptx".

## Approach

Add a local test helper inside F-018's existing `package.rs` test module. It
constructs an in-memory package with
`OpcPackage::with_main_part("ppt/presentation.xml", content_types::PRESENTATION)`,
then adds `/ppt/presentation.xml`, `/ppt/slides/slide1.xml`, and
`/ppt/slideLayouts/slideLayout1.xml` parts with their F-019 content types.

Add a presentation relationship from `/ppt/presentation.xml` to
`slides/slide1.xml`, and add a slide relationship from
`/ppt/slides/slide1.xml` to `../slideLayouts/slideLayout1.xml`. Write the
package to an in-memory cursor and reopen it through
`OpcPackage::from_reader` so the fixture exercises ZIP loading, content types,
package relationships, and part relationship path conversion rather than only
the builder's in-memory maps.

The round-trip test asserts all of these results:

- `main_document_part()` returns `/ppt/presentation.xml`.
- Resolving the presentation's slide target returns
  `/ppt/slides/slide1.xml`.
- Resolving the slide's layout target returns
  `/ppt/slideLayouts/slideLayout1.xml`.
- The reopened package contains the presentation, slide, and layout parts under
  normalized leading-slash keys.

Keep the fixture in the existing inline test module. Do not add a binary
fixture, integration-test binary, source file, dependency, or production API.
If the gate exposes a production parser or serializer defect, stop and revise
this design before changing that code because the parser risk rider would then
apply.

## Rejected alternatives

- Check only `resolve_rel_target` with two strings. That proves path algebra
  but not that a pptx relationship graph survives write and read.
- Add a binary `.pptx` fixture. Repository policy requires code-constructed
  fixtures, and a binary would enlarge the published crate without improving
  this small graph test.
- Add a new integration test file. It creates another binary to link, while the
  existing package test module can exercise the same public calls.
- Parse PresentationML XML in this story. OPC owns ZIP parts, content types,
  and relationships. Presentation XML belongs to `rpptx-oxml`.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `pptx_package_resolves_main_slide_and_layout_parts` | A code-built package writes and reopens, `main_document_part()` resolves `/ppt/presentation.xml`, and slide and layout relationships resolve to normalized part keys |
| unit | `presentation_layout_target_resolves_one_directory_up` | `resolve_rel_target("/ppt/slides/slide1.xml", "../slideLayouts/slideLayout1.xml")` returns `/ppt/slideLayouts/slideLayout1.xml` directly |

The backlog **test gate** is that `main_document_part()` resolves
`/ppt/presentation.xml` and that the slide-layout target resolves correctly.

## HLD impact

None. The OPC and testing specifications already state that the existing
navigation reads this package graph and require this exact fixture.

## Risk routing

None. The planned diff adds tests inside an existing module and changes no
parser, serializer, public API, dependency edge, feature flag, or file. If a
production change becomes necessary, revise the plan and reroute the actual
diff before editing it.

## Hash harness

Expected to remain unchanged. This story adds an in-memory test fixture and no
rdocx production call site.

## Implementation checklist

- [ ] Build the pptx-shaped package through F-018 and F-019 public APIs.
- [ ] Add presentation-to-slide and slide-to-layout relationships.
- [ ] Write and reopen the fixture entirely in memory.
- [ ] Assert main-part, slide-part, and parent-directory layout resolution.
- [ ] Assert all three parts use normalized package keys after reopening.
- [ ] Run focused `oxml-opc` tests and the unchanged hash harness.

## Open questions

None. F-018 supplies the generic constructor, F-019 supplies the package
constants, and the OPC specification fixes the package graph and expected
targets.
