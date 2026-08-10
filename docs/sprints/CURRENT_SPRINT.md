# Current Sprint, S32

**Milestone**: M12 Charts.

**Goal**: Finish chart rendering polish and close the M12 chart path. Resolve
series paint from direct ChartML styling or the effective theme accent cycle,
then route preserved unsupported charts through a cached image or a visible
diagnostic placeholder so authored and preserved charts both render predictably.

## Spec references

- `docs/hld/03-architecture.md`, for ownership between ChartML parsing,
  relationship resolution, slide resolution, and backend-neutral rendering.
- `docs/hld/05-drawingml-model.md`, for colour-map and theme lookup followed by
  the ordered transform stack that produces exact resolved RGBA values.
- `docs/hld/07-inheritance-and-resolution.md`, for effective theme and colour-map
  scope and the stable diagnostic contract for unsupported content.
- `docs/hld/08-rendering-spec.md`, for resolved relationship scopes, media bytes,
  backend-neutral page frames, visible fallbacks, and renderer diagnostics.
- `docs/hld/09-charts-spec.md`, for direct `c:spPr` precedence, the theme accent
  cycle, native chart routing, and cached-image or placeholder fallback policy.
- `docs/hld/12-testing-strategy.md`, for corpus preservation, deterministic
  rendering, and native PowerPoint acceptance evidence.
- `docs/hld/13-risks-and-open-questions.md`, for exact theme-colour verification,
  schema child ordering, and byte preservation of unsupported chart XML.
- `docs/hld/14-development-backlog.md`, for F-127 and F-128 dependencies,
  focused test gates, and the M12 end-of-milestone gate.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-127 | Chart colour resolution | M | in-progress | codex |
| F-128 | Preserved chart fallback | S | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order. Both stories depend on the
completed F-125 geometry contract and may begin independently. F-127 replaces
the placeholder series palette inside native geometry, while F-128 resolves the
chart relationship and selects native geometry or a preserved-content fallback
at the presentation rendering boundary.

## Definition of done for this sprint

- A direct series `c:spPr` colour wins when present. Otherwise an unstyled
  four-series chart resolves through the effective theme to accent1 through
  accent4 with the established colour transform pipeline.
- A preserved unsupported chart uses its cached image when available. Without
  one, it emits a labelled placeholder and a stable diagnostic rather than
  disappearing.
- A 3-D chart renders its cached image and records the unsupported-chart
  diagnostic required by F-128.
- The M12 gate holds end to end: a chart created by `rpptx` opens in PowerPoint,
  exposes editable source data, and enters the presentation rendering pipeline.
- The full workspace gate passes, all 28 deterministic hashes remain unchanged
  unless a design plan declares a reviewed delta, and development chart crates
  remain unpublished.
