# F-200, working, pass 5

**Reviewed**: working diff against
`cf7627aa280c65a245dbed8fbd2988e80dae9201`, 21 files with 2,544 tracked
insertions and 164 tracked deletions, plus the pass 1 through pass 4 review
records
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, an interleaved malformed direction duplicate moves after the later valid toggle

`crates/rdocx-oxml/src/properties.rs:227`

Every valid modeled toggle carrier is stored at occurrence zero, even after the
parser has counted intervening occurrences. Consider a valid `w:bidi`, then a
nonempty malformed `w:bidi` subtree, then another valid `w:bidi`. The first
carrier becomes a raw duplicate at occurrence zero, the malformed subtree keeps
occurrence one, and the final carrier is again recorded at occurrence zero.
Serialization sorts those raw positions at
`crates/rdocx-oxml/src/properties.rs:1933`, then emits the one generated scalar
toggle between occurrence-zero and occurrence-one raw values at
`crates/rdocx-oxml/src/properties.rs:1963`. The result is first valid, last
valid, malformed instead of the original first valid, malformed, last valid.
The same shared candidate logic and run writer produce the equivalent `w:rtl`
failure. The regressions cover valid followed by malformed and adjacent valid
duplicates separately, but not this interleaving, so duplicate schema order and
raw replay are still incomplete.

### D2, source-less numbering markers remain in visual extraction order

`crates/rdocx-layout/src/paginator.rs:2737`

The new logical reconstruction runs only when a line contains a source-less
field. A Word numbering marker is also emitted as text with no source and no
field kind at `crates/rdocx-layout/src/engine.rs:4557`, so an ordinary numbered
RTL line skips the reconstruction. The fallback then attaches every source-less
element to the preceding group in visual order at
`crates/rdocx-layout/src/paginator.rs:2663`. For the existing logical marker,
numeric, Arabic paragraph whose painted order is Arabic, numeric, marker, this
places the marker after the numeric source group during PDF and SVG extraction
instead of restoring the logical marker prefix. The leading-edge regression at
`crates/rdocx-layout/src/engine.rs:7659` checks only painted line items, and the
new source-less field regression contains no numbering marker. Logical
extraction is therefore still incomplete for the approved bullet and label
interaction.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass 4 D1 is closed for body paragraphs. Exact resolved direction now travels
  through private shared-block and paragraph-view state, and field-only drawing
  reflow selects the bidi breaker without adding content to public
  `ParagraphReflow::items`.
- Pass 4 D2 is closed for the isolated sourced-text and stored-field sequence.
  Field kind and logical input position restore PAGE text while each positioned
  run keeps its visual origin. D2 above is the remaining source-less non-field
  interaction.
- Pass 4 S1 is closed. The synthetic `FontId(u32::MAX)` marker and its
  positional decoding are absent, while the public `ParagraphReflow` shape
  remains unchanged.
- Prior natural numeric levels, line-local L1 then L2 ordering, conditional
  hyphenation demotion, mixed rich and legacy extraction, absent-direction
  inference, inline object ordering, stored-field direction, drawing reflow,
  and source-span behavior remain intact.
- Direction remains part of paragraph cache identity and private cache output.
  No warm-versus-fresh, cached-tail, header, footer, note, or restart
  invalidation defect was found.
- Malformed multilingual output still uses the shared validation contract
  before PDF, raster, and SVG indexing. No new panic, unchecked slice, integer
  overflow, or error suppression was found.
- DrawingML direction, PowerPoint paragraph sidecars, forced breaks, and the
  documented quarter-turn vertical approximations remain unchanged and
  compatible with the shared direction path.
- The intentional pre-1.0 `TextSegment` and Word property field additions are
  documented. No new public `ParagraphReflow` field, entrypoint, module,
  dependency, reverse crate edge, or structural smell was introduced.
- Exactly the five plan-listed HLD files changed. The recorded 49-of-49 hash,
  five-of-five raw oracle, current output hashes, deterministic corpus,
  portability, package, archive, and supply-chain evidence remains internally
  consistent. No threshold, baseline, or oracle identity was weakened.
