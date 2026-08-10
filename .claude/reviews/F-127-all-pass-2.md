# F-127, all, pass 2

**Reviewed**: remediated working diff from claim base `b1a4abd`, 4 files,
661 changed lines, with 555 additions and 106 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, Theme-slot transforms are dropped from direct scheme colours
`crates/rpptx-chart/src/lib.rs:1955`

`concrete_theme_colour` copies the raw sRGB value, or raw system fallback,
without applying the transforms stored on that theme colour choice. A series
with direct `a:schemeClr val="accent1"` therefore renders the untransformed base
when the referenced `a:clrScheme/a:accent1` colour carries a tint, shade,
luminance, or channel transform. The final colour must include the theme-slot
transform stack before applying the transforms on the series colour choice.
The current transform-order test puts every transform on the series
`a:schemeClr`, so this valid split across theme and series remains uncovered.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-1 D1 is fixed. Filled area and radar alpha now composes the resolved
  alpha with the existing 55 percent policy, including transparent `a:noFill`.
- Pass-1 D2 is fixed. Unused unresolvable theme slots no longer block a direct
  sRGB series colour, and the new regression exercises that precedence.
- Panics: no new reachable panic, unchecked slice, or untrusted arithmetic
  issue was found.
- OOXML: no schema-order, namespace, whitespace, or raw-preservation issue was
  found.
- Structure: no unjustified trait, generic, wrapper, module, file, or dynamic
  dispatch was introduced.
