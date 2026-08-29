# F-X058, working, pass 4

**Reviewed**: complete current working-tree diff against
`f74420c3b6f553ab2e3e139eb9e4f54074496adf`, 31 tracked files with 3,178
insertions and 173 deletions, plus seven untracked font, licence, notice, and
subset-provenance assets. The pass-1 through pass-3 review records were treated
as review evidence rather than implementation scope.
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, line-local L1 resets are discarded inside one shaped span
`crates/oxml-layout/src/line.rs:669`

`reordered_levels` correctly computes one adjusted level per byte, but the
line breaker retains only the level at each multilingual span start before it
calls L2. A single styled LTR-base run such as `אבג   אבג` is shaped with the
first breakable span containing both `אבג` and its following spaces. UAX 9 L1
resets those trailing-space bytes to the paragraph level, while this code
samples the span's first Hebrew byte and keeps one RTL level for the whole
span. The painter therefore cannot place the reset whitespace independently
of the RTL glyphs on the completed line. The new regression constructs Hebrew
and whitespace as separate styled inputs at
`crates/oxml-layout/src/line.rs:1639`, which guarantees a span boundary and
does not exercise the ordinary one-run case. Line fitting must preserve or
create the adjusted-level boundary before applying L2.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-3 D1 is otherwise remediated. PowerPoint shapes all styled text through
one paragraph-wide bidi resolution, reconstructs forced-break controls at the
original run boundaries, and preserves monotonically increasing logical
indices without adding or changing a public type or entrypoint. The exact
four-run forced-break regression proves logical extraction order, visual x
order on both lines, and line y order.

Pass-3 D3 is remediated. SVG now uses the same
`MultilingualGlyphRun::is_valid` preflight as PDF and raster, omits malformed
positioning with a diagnostic, and cannot serialize the tested non-finite
advance. Valid rich SVG remains searchable. PDF font collection and emission
and raster drawing continue to reject malformed vectors before indexing them.

Logical source ranges and cluster maps remain contiguous through paragraph
shaping and line fitting. Explicit DrawingML direction still reaches numeric
and Latin forced-break content through the additive sidecar. The exhaustive
`ResolvedParagraph` shape and established resolver and renderer entrypoints
remain source compatible.

Conditional hyphenation, grapheme-safe script and font coverage segmentation,
Arabic and Indic shaping, Thai and CJK boundaries, OOXML preservation and
schema order, dependency direction, deterministic font and legal provenance,
archive inventory, WASM isolation, and legacy Latin hash isolation produced no
additional findings. The structural prohibitions on new modules, traits,
generics, feature flags, and forwarding wrappers also produced no findings.
