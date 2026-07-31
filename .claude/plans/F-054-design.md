# F-054, Colour choices

**Status**: completed
**Sprint**: S12
**Size**: M
**Depends on**: none

## Problem

DrawingML represents colour through four choice elements, but no shared model
currently exists. The old Word theme parser recognises only a subset and keeps
Word-specific behaviour at `crates/rdocx-oxml/src/theme.rs:1`, while the new
crate must model `a:srgbClr`, `a:schemeClr`, `a:sysClr`, and `a:prstClr`
without moving or changing that legacy path.

## Spec reference

- `docs/hld/03-architecture.md`, "Crate-level conventions".
- `docs/hld/05-drawingml-model.md`, "Colour, the part everyone gets wrong" and
  "Do not touch the Word path".
- `docs/hld/14-development-backlog.md`, "F-054, Colour choices".

## Approach

Add `color.rs` with a concrete `ColorChoice` enum and a validated `RgbColor`
value. Represent sRGB as three bytes, scheme and preset identifiers as owned
strings, and system colour as its system identifier plus optional `lastClr`
fallback. Provide nested-element parse and write functions using quick-xml,
prefix-tolerant local-name matching, and fixed `a:` output.

F-054 does not interpret transform children yet. Since every child is raw at
this stage, preserve those subtrees byte for byte in one document-ordered
`Vec<Vec<u8>>`. F-055 adopts F-053's `OrderedRawChildren` after both stories
are integrated and replaces known transform placeholders without moving
unknown siblings. Reject malformed six-digit RGB values with a concrete error
defined in `color.rs` rather than silently substituting black.

## Rejected alternatives

- Reuse `rdocx_oxml::theme::Theme`. It is read-only, incomplete, and its maths
  intentionally remain Word-specific.
- Store every colour as an unchecked string. That allows malformed RGB input
  into later exact resolution code.
- Model the transform stack in this story. F-055 owns transform semantics and
  its external-oracle gate.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `srgb_colour_parses_and_round_trips` | Six-digit RGB values preserve their exact colour through fixed-prefix output. |
| round-trip | `scheme_colour_parses_and_round_trips` | Semantic theme slot names survive parse and write. |
| round-trip | `system_colour_uses_and_preserves_last_colour` | Both the system identifier and optional `lastClr` fallback survive. |
| round-trip | `preset_colour_parses_and_round_trips` | Preset identifiers survive parse and write. |
| regression | `unknown_colour_children_are_preserved_in_place` | An unmodelled child remains byte-identical for F-055 to model later. |
| unit | `malformed_srgb_values_are_rejected` | Non-hex and non-six-digit inputs return an error. |

The **test gate** is: each form parses and round-trips.

## HLD impact

None. The four choices and preservation behaviour are already specified.

## Risk routing

- **Theme colour, tint, shade, colour mapping**: read
  `docs/hld/05-drawingml-model.md`. The extra check is an empty diff under the
  legacy Word theme implementation plus an unchanged hash harness.
- **Any parser or serialiser**: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. The required reading supplies the OPC
  and PresentationML context. The extra checks cover prefix-tolerant input,
  fixed-prefix output, schema order, and byte-for-byte raw preservation.
- **Crate dependency graph and a new module or file**: read
  `docs/hld/03-architecture.md` and the structural rules in `CLAUDE.md`. The
  extra checks are the dependency scan and confirmation that F-054 explicitly
  authorises `color.rs`.

## Hash harness

Expected to be unchanged. The released Word theme parser and renderer remain
untouched.

## Implementation checklist

- [x] Add the concrete colour choice and RGB value types.
- [x] Parse all four colour elements with local-name matching.
- [x] Write all four forms with the fixed `a:` prefix.
- [x] Preserve unknown child XML in its original slot.
- [x] Keep raw children in a simple ordered vector until F-055 integrates
  F-053's helper.
- [x] Reject malformed RGB values and add all round-trip tests.

## Open questions

None. Transform children remain raw until F-055, keeping the two story
contracts separate.
