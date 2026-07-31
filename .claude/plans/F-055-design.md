# F-055, The colour transform stack

**Status**: approved
**Sprint**: S12
**Size**: L
**Depends on**: F-054, F-014

## Problem

Office themes apply colour transforms in document order, and use `lumMod` and
`lumOff` on most accent-derived colours. The required transform set and maths
are specified at `docs/hld/05-drawingml-model.md:62`, but no shared
implementation exists. Reusing or correcting Word's naive sRGB tint and shade
function would silently change every released Word render, which the spec
forbids at `docs/hld/05-drawingml-model.md:81`.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Colour, the part everyone gets wrong" and
  "Do not touch the Word path".
- `docs/hld/07-inheritance-and-resolution.md`, "5. Colour" and "Testing this".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The hash harness".
- `docs/hld/14-development-backlog.md`, "F-055, The colour transform stack".

## Approach

Extend `color.rs` with a concrete `ColorTransform` enum and a `ResolvedColor`
carrying RGB bytes plus alpha. Parse transform children in document order,
preserve unknown siblings through `OrderedRawChildren`, and write known
transforms with fixed `a:` names and their schema-specific integer values.

Implement all 28 schema transforms: alpha, blue, green, red, hue, saturation,
and luminance absolute, modulation, and offset forms, plus tint, shade,
complement, inverse, gray, gamma, and inverse gamma. Use `Angle` for `hue` and
`hueOff`, and schema-specific percentage types for the other valued elements.
Implement separate spec-correct sRGB-to-linear and linear-to-sRGB helpers,
linear-gamma tint and shade, and RGB-to-HSL operations. Clamp only at the
boundary each ECMA operation defines, then round final channel bytes
consistently. Apply the stored vector strictly left to right.

Build a temporary 40-shape oracle deck through a dev-only `oxml-opc`
dependency, render it with installed Microsoft PowerPoint 16.104 build
16.104.25121423, and export each named shape directly as PNG. Sample a uniform
5 by 5 centre block and require all 25 pixels to have identical raw RGBA.
Commit the resulting input and exact expected RGBA values as a readable Rust
table, not as a binary fixture. Use 28 single-transform rows, one for each
schema element, plus 12 ordered stacks covering non-commutativity, channel and
alpha clamping, HSL operations, and gamma operations. Pin the PowerPoint build
in a constant consumed by the test harness. The ignored generator writes only
temporary deck and PNG files and prints candidate rows for review. It never
rewrites source. PowerPoint remains one-time test infrastructure and never
becomes a production dependency.

## Rejected alternatives

- Call `rdocx_oxml::theme::apply_tint_shade`. Its deliberately naive Word
  behaviour is a held compatibility contract.
- Treat tint, shade, and luminance as direct sRGB interpolation. ECMA-376 uses
  linear gamma for tint and shade and HSL luminance for `lumMod` and `lumOff`.
- Generate expected values from the same Rust formula. That would test the
  implementation against itself instead of the real PowerPoint oracle.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `powerpoint_colour_transform_oracle_matches_all_forty_pairs` | Forty readable input and transform stacks resolve to the exact RGBA sampled from direct PowerPoint shape PNG exports on version 16.104 build 16.104.25121423. |
| unit | `colour_transforms_apply_in_document_order` | Two non-commuting transforms produce the left-to-right result and not the reversed result. |
| unit | `linear_gamma_round_trip_preserves_channel_endpoints` | The conversion helpers preserve 0 and 1 and remain within the required tolerance for intermediate values. |
| unit | `alpha_transforms_clamp_to_the_valid_range` | Alpha, alphaMod, and alphaOff compose and clamp correctly. |
| round-trip | `known_and_unknown_transform_children_keep_document_order` | Modelled transforms and raw siblings re-emit in their original slots. |
| verification | `python3 scripts/hash_harness.py --check` | The released Word implementation is untouched and the 28-entry hash harness stays exact without introducing a forbidden dependency from `oxml-drawing` to `rdocx-oxml`. |

The **test gate** is: a table of 40 (theme colour, transform) pairs sampled
from real PowerPoint renders resolves to exact RGBA.

## HLD impact

- `docs/hld/05-drawingml-model.md`: replace the incomplete 16-variant enum with
  the complete 28-element schema set, use `Angle` for `hue` and `hueOff`, and
  use the correct `gamma` and `invGamma` names.

## Risk routing

- **Theme colour, tint, shade, colour mapping**: read
  `docs/hld/05-drawingml-model.md`. The extra checks are the 40 exact RGB pairs,
  ordered-transform regression, an empty Word-theme diff, and the unchanged
  hash harness.
- **Any parser or serialiser**: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. The extra checks cover schema order,
  prefix-tolerant input, fixed-prefix output, and byte-for-byte preservation of
  unknown transform children.
- **External oracle comparison**: read
  `.claude/skills/differential-testing.md`. The extra checks pin PowerPoint
  16.104 build 16.104.25121423 in the harness, compare exact sampled RGBA
  rather than package bytes, require a uniform centre block, and classify every
  disagreement before changing an expectation.
- **Crate dependency graph**: add `oxml-opc` as a dev dependency for the
  ignored temporary-deck generator only. Inspect normal and development edges,
  prove the production graph remains limited to `oxml-core` and quick-xml, and
  prove there is no `rdocx-*` or `rpptx-*` edge.

## Hash harness

Expected to be unchanged. Spec-correct functions are new `oxml-drawing` APIs,
and the deliberately naive Word function remains byte-for-byte unchanged.

## Implementation checklist

- [ ] Add every required transform variant and XML mapping.
- [ ] Implement ordered resolution with linear-gamma and HSL helpers.
- [ ] Generate and directly export 28 single-transform and 12 ordered-stack
  shapes through the dev-only `oxml-opc` oracle helper.
- [ ] Commit the readable 40-case table with its consumed oracle-build pin.
- [ ] Add order, alpha, conversion, and mixed raw-child tests.
- [ ] Prove the Word path and all 28 hashes are unchanged.

## Open questions

None. The approved scope is the complete 28-element DrawingML transform set,
with the stale HLD enum corrected. Microsoft PowerPoint 16.104 build
16.104.25121423 provides the pinned real-render oracle for the 40-row table.
