# S45 sprint review, pass 1

**Reviewed**: `sprint/s45` at
`cd35981fbef821ea51d410ef887db7ce45fadc10` against
`2115db039d5745d624638b73b4c1adfbe8a7ecc4`, 66 files, 36,007 changed
lines, crates: `oxml-chart`, `oxml-layout`, `oxml-sml`, `rdocx`,
`rdocx-layout`, `rdocx-oxml`, `rpptx`, `rpptx-chart`, `rpptx-layout`, and
`rpptx-render`

**Verdict**: 2 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, the F-156 completion record names nonexistent HLD files

`docs/sprints/AS_BUILT.md:6891`

The entry says F-156 touched `docs/hld/01-product-scope.md`,
`docs/hld/02-system-context.md`, and `docs/hld/07-api-design.md`. None exists.
The integrated diff actually changes `docs/hld/01-glossary.md`,
`docs/hld/02-scope-and-non-goals.md`, and
`docs/hld/07-inheritance-and-resolution.md`. Correct the three paths before the
append-only entry becomes the closed sprint record.

### B2, the package README inventory retained two pre-extraction counts

`docs/hld/15-build-and-toolchain.md:326`

`docs/hld/15-build-and-toolchain.md:329`

The section now says the workspace has 27 packages but still calls the
crate-local set the other 25 packages. It also still names two deprecated shims
and only their `oxml-opc` and `oxml-pdf` destinations, despite this sprint
adding `rpptx-chart` as the third shim over `oxml-chart`. Update both adjacent
claims so the current HLD describes the 27-package tree and all three shims.

## Should-fix

None.

## Nice-to-have

None.

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

Interaction, duplication, layering, harness, dependency, gate, and public
surface review produced no findings. The production dependency graph points
from both format families into `oxml-chart`, `oxml-layout` remains free of a
chart dependency, and the only reverse family reference is the approved
dev-only `rdocx` golden dependency on `rpptx`. The integrated full gate observed
49 of 49 unchanged hash entries.
