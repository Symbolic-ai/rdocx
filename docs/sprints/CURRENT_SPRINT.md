# Current Sprint, S03

**Milestone**: M2 Core and package extraction.

**Goal**: Establish the unpublished `oxml-core` implementation and add the
shared unit and property models needed by both Word and PowerPoint. Preserve
existing rdocx behaviour and keep the rdocx 0.5.0 release line independent of
the development crate.

## Spec references

- `docs/hld/01-glossary.md`, for the canonical OOXML unit definitions and
  their exact storage scales.
- `docs/hld/02-scope-and-non-goals.md`, for sharing core, app, and custom
  properties between Word and PowerPoint through `oxml-core`.
- `docs/hld/03-architecture.md`, for the `oxml-core` ownership boundary,
  dependency direction, parser conventions, and `rdocx-oxml` facade role.
- `docs/hld/05-drawingml-model.md`, for the DrawingML consumers of `Angle`,
  `Centipoints`, and `Percent1000` and the legacy Word colour boundary.
- `docs/hld/11-migration-plan.md`, for the zero-call-site facade extraction,
  operation order, and behaviour-preservation constraints.
- `docs/hld/12-testing-strategy.md`, for unit round-trips, truncation tests,
  public XML text coverage, and cross-format app-properties fixtures.
- `docs/hld/14-development-backlog.md`, for the F-013 through F-017 contracts,
  dependencies, and test gates.
- `docs/hld/15-build-and-toolchain.md`, for publication ordering and the rule
  that `oxml-*` and `rpptx*` placeholders stay at 0.0.0 until PowerPoint
  development is complete.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-013 | Create oxml-core | M | in-progress | codex |
| F-014 | New unit types | M | pending | - |
| F-015 | rdocx-oxml becomes a facade | S | pending | - |
| F-016 | Length re-export | S | pending | - |
| F-017 | App and custom properties | M | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-013 establishes `oxml-core` and blocks every other story. After it lands,
F-014 and F-017 can proceed independently when their touched files do not
overlap. F-015 and F-016 are carried by explicit decision because their facade
switch would make published rdocx crates depend on the unpublished
`oxml-core` implementation.

## Definition of done for this sprint

- The generic unit, XML helper, raw XML, core-properties, error, and `Length`
  implementations live in `oxml-core`, with their existing tests moved intact.
- `Centipoints`, `Angle`, `Percent1000`, and `Length::mm` pass their specified
  conversion and round-trip assertions while existing float constructors keep
  truncating toward zero.
- Word and PowerPoint app-properties fixtures parse and round-trip without
  emitting fields belonging only to the other format, and custom properties
  round-trip with unmodelled XML preserved.
- Workspace tests pass and the hash harness remains unchanged.
- No `oxml-*` or `rpptx*` development crate is published beyond its existing
  0.0.0 placeholder.
- F-015 and F-016 remain pending with their carry reason recorded for the next
  planning decision.
