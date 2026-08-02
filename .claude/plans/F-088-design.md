# F-088, Visual differential tests

**Status**: approved
**Sprint**: S21
**Size**: M
**Depends on**: F-087

## Problem

The exact 40-case PowerPoint colour oracle already exists at
`crates/oxml-drawing/src/color.rs:1583`, but no integrated test proves that the
frozen `ResolvedSlide` boundary applies colour mapping, theme lookup,
inheritance, suppression, and draw order correctly on complete decks. The
existing corpus gate in `crates/rpptx/tests/integration.rs:37` validates model
round trips and the read facade, not resolved visual intent.

The M9 gate therefore lacks a durable differential record and a one-time manual
review of decks whose correct appearance exposes master logos, prompt-text
leaks, backgrounds, placeholders, and exact theme colours.

## Spec reference

- `docs/hld/07-inheritance-and-resolution.md`, "Draw order", "Colour", "The
  output contract", and "Testing this".
- `docs/hld/12-testing-strategy.md`, "Test categories", "The deck corpus",
  and "PresentationML resolver and renderer tests".
- `docs/hld/14-development-backlog.md`, "F-088, Visual differential tests".
- `docs/hld/15-build-and-toolchain.md`, "Publishing".

## Approach

Extend the existing `rpptx` integration test binary rather than creating a new
test target. Add `rpptx-layout` only as a development dependency and use the
existing OPC relationship helpers to assemble the presentation, slide,
layout, master, theme, colour-map, and default-text inputs for each selected
corpus slide.

Add a stable normalized dump of the resolved visual boundary. It records the
effective background, ordered shape kinds and bounds, concrete RGBA values,
text, unsupported markers, and diagnostics without serialisation-specific
bytes. Compare inheritance-sensitive fields against pinned python-pptx 1.0.2
where its object model exposes them. Keep the oracle version assertion in the
executable gate and classify any known divergence explicitly.

Use existing pinned corpus decks that make the policies visually obvious,
including `WithMaster.pptx`, `backgrounds.pptx`,
`placeholder-layout-color.pptx`, and
`bug58144-headers-footers-2007.pptx`. Do not add binary fixtures. Run the
existing 40-case exact PowerPoint 16.104 build 16.104.25121423 colour table as
part of the focused gate. Perform the required one-time manual PowerPoint
review of the selected original decks and the normalized resolved evidence,
recording exact paths, application build, result, and any unsupported content.

No crate is published, no release tag is created, and no new source or test
file is added.

## Rejected alternatives

- Add committed screenshot or deck binaries. The fetched corpus is the sole
  binary fixture exception.
- Compare raw XML or serialized bytes with python-pptx. Prefixes, attribute
  order, and whitespace are not the resolver contract.
- Treat PowerPoint as an automated object-model oracle. Native Office is the
  one-time manual visual acceptance check, while the pinned Python comparison
  supplies repeatable structure.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `resolved_visual_dump_matches_python_pptx_1_0_2` | Pinned oracle and Rust agree on exposed visual inheritance fields |
| integration | `visual_decks_resolve_in_expected_draw_order` | Background, logo, placeholders, and slide content follow the frozen contract |
| regression | `visual_decks_never_emit_template_prompt_text` | Master and layout prompt strings do not enter resolved output |
| regression | `visual_decks_emit_each_master_logo_once` | Master artwork is neither dropped nor duplicated |
| unit | `powerpoint_colour_transform_oracle_matches_all_forty_pairs` | All 40 exact RGB or RGBA pairs retain the pinned PowerPoint result |
| manual | `selected_visual_decks_reviewed_once_in_powerpoint` | Exact paths and PowerPoint build are recorded with a clean verdict |

The backlog test gate is named explicitly: the exact 40-case colour table
passes and the selected differential decks receive their one-time manual
review.

## HLD impact

- `docs/hld/07-inheritance-and-resolution.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- External oracle comparison. Pin python-pptx to 1.0.2 in the executable
  command, assert the resolved version, compare normalized trees rather than
  bytes, and keep the oracle out of production dependencies.
- Layout and rendering comparison. Use deterministic font mode for any
  repository-generated render evidence. Record no baseline against system
  fonts.
- Theme colour, tint, shade, and colour mapping. Run the exact 40-case
  PowerPoint oracle and the inheritance-sensitive deck checks without touching
  the deliberately different Word colour path.
- Crate dependency graph. Keep `rpptx-layout` as a test-only dependency of the
  facade and confirm no `oxml-*` crate gains an `rpptx-*` dependency.

## Hash harness

Expected to be unchanged. Differential evidence and resolver-only tests do not
alter Word rendering.

## Implementation checklist

- [ ] Assemble complete resolver inputs through package relationships.
- [ ] Add normalized resolved-visual output to the existing test binary.
- [ ] Compare supported fields against pinned python-pptx 1.0.2.
- [ ] Gate prompt suppression, logo count, draw order, and concrete colours.
- [ ] Run and record the existing exact 40-case colour table.
- [ ] Complete and record the one-time manual PowerPoint review.

## Open questions

None. Existing corpus decks and the already pinned oracle versions provide the
required evidence without new binary files.
