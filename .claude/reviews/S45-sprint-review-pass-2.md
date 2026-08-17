# S45 sprint review, pass 2

**Reviewed**: `sprint/s45` at
`509ba18634bb8c70bcdca2a430552e82447c1663` against
`2115db039d5745d624638b73b4c1adfbe8a7ecc4`, 67 files, 36,079 changed
lines, crates: `oxml-chart`, `oxml-layout`, `oxml-sml`, `rdocx`,
`rdocx-layout`, `rdocx-oxml`, `rpptx`, `rpptx-chart`, `rpptx-layout`, and
`rpptx-render`

**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Remediation verified

- B1: the F-156 as-built entry now names the three HLD files that exist and
  changed in the integrated diff.
- B2: the HLD README inventory now states 26 crate-local files beside the root
  guide and names all three deprecated compatibility shims and destinations.

## Milestone gate

The gate is: "a Word document gains a native chart that opens editable in
Word, and renders identically to the same chart in a deck."

It holds. Microsoft Word 16.104 opened the SHA-bound Word candidate without
repair, and the user confirmed that changing its embedded data through Edit
Data worked. `word_and_powerpoint_chart_pixels_are_identical` authored both
families from one `ChartData` source and produced 750 by 450 pixel crops at 150
DPI with bundled fonts and `pdftoppm 26.01.0`, with zero differing RGBA pixels.
The Word artifact SHA-256 is
`e50845637449e2af4b8e2dbf16f5f6f53e5f598a00401fcc34c13f5d5716a1c4`, and
the PowerPoint artifact SHA-256 is
`7525e9a088c5fbf58fa1ed98cdfa0ec2fabf998662112ced7a6b6521f2c4edfc`.

## Not found

Interaction, duplication, layering, harness, gate, documentation, dependency,
and public surface review produced no findings. Both facades use the single
shared authoring implementation. Production dependencies point inward to
`oxml-chart`, while the cross-family golden uses the named dev-only `rpptx`
consumer. `oxml-layout` remains free of chart and format-family dependencies,
and the deprecated `rpptx-chart` package has no active production consumer.
The integrated hash harness remains unchanged at 49 of 49.
