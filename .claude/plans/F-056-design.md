# F-056, Colour map resolution

**Status**: approved
**Sprint**: S12
**Size**: M
**Depends on**: F-055

## Problem

DrawingML scheme colours name semantic slots such as `bg1` and `tx1`, but the
actual theme slot is selected by the master colour map before theme lookup. The
required three-stage order and the dark-master inversion are specified at
`docs/hld/05-drawingml-model.md:51`, while no `oxml-drawing` implementation
exists yet. PresentationML parsing of `p:clrMap` and `p:clrMapOvr` belongs to
the later slide-part story at `docs/hld/14-development-backlog.md:545`, so this
story must establish resolution without pulling PresentationML types into the
DrawingML crate.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Colour, the part everyone gets wrong".
- `docs/hld/07-inheritance-and-resolution.md`, "5. Colour" and "The resolver".
- `docs/hld/14-development-backlog.md`, "F-056, Colour map resolution".

## Approach

Extend the existing `color.rs` from F-054 and F-055 with a format-neutral
`ColorMap` value that maps the twelve semantic scheme slots to concrete theme
slots. Provide the standard Office mapping as `Default`, an explicit
constructor for parsed master values, and an override operation representing
`p:clrMapOvr` without parsing `p:` XML in this crate.

Add a resolution entry point that accepts a `ColorChoice`, `ColorMap`, and
theme colour lookup. For `schemeClr`, map the semantic name first, look up the
mapped theme colour second, then apply the transform stack in document order.
Direct RGB, system, and preset choices bypass the colour map. Keep the API
concrete, with no trait or generic parameter.

## Rejected alternatives

- Parse `p:clrMap` directly in `oxml-drawing`. F-069 owns PresentationML slide,
  layout, and master parsing, and this would reverse the intended layer split.
- Treat `bg1` and `tx1` as fixed aliases. Colour maps are per master, and dark
  masters deliberately invert them.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `standard_colour_map_uses_office_theme_slots` | The default semantic slots select their standard concrete theme slots. |
| unit | `dark_master_colour_map_inverts_background_and_text` | A map with `bg1 -> dk1` and `tx1 -> lt1` resolves the two semantic choices to the dark-master RGB values. |
| unit | `colour_map_override_wins_before_theme_lookup` | An override replaces only named mappings and leaves the other master mappings intact. |
| regression | `direct_colours_bypass_the_master_colour_map` | RGB, system, and preset choices do not change when the map changes. |

The **test gate** is: a dark master inverting `bg1` and `tx1` resolves
correctly.

## HLD impact

None. The implementation follows the existing three-stage resolution contract.

## Risk routing

- **Theme colour, tint, shade, colour mapping**: read
  `docs/hld/05-drawingml-model.md`. The extra check is the dark-master exact-RGB
  regression, with the legacy `rdocx_oxml::theme::apply_tint_shade` path left
  untouched.

## Hash harness

Expected to be unchanged. This story adds unpublished DrawingML resolution and
does not alter the released Word rendering path.

## Implementation checklist

- [ ] Add the concrete `ColorMap` representation and standard mapping.
- [ ] Add master-map override composition without PresentationML parsing.
- [ ] Resolve scheme colours through map, theme, then transforms.
- [ ] Keep direct colour choices independent of the map.
- [ ] Add the dark-master, override, and direct-colour tests.

## Open questions

None. F-069 explicitly owns the later `p:` XML parser, so this story exposes a
format-neutral mapping value and resolution API only.
