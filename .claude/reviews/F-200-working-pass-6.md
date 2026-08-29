# F-200, working, pass 6

**Reviewed**: working diff against
`cf7627aa280c65a245dbed8fbd2988e80dae9201`, 21 files with 2,677 tracked
insertions and 164 tracked deletions, plus the pass 1 through pass 5 review
records
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, a canonical final direction toggle still crosses an interleaved malformed occurrence

`crates/rdocx-oxml/src/properties.rs:240`

The pass 5 fix keeps the true occurrence on each candidate, but removes the
latest carrier whenever that final valid toggle has no unsupported attributes.
For a valid attributed `w:bidi`, then a nonempty malformed `w:bidi` subtree,
then a canonical `<w:bidi/>`, the first candidate becomes a raw duplicate at
occurrence zero, the malformed subtree remains at occurrence one, and the final
occurrence-two carrier is removed here. Without a remaining carrier,
`effective_ppr_raw_position` cannot collapse values around the final modeled
position at `crates/rdocx-oxml/src/properties.rs:2023`. Serialization therefore
writes the occurrence-zero raw duplicate, the generated final toggle, then the
occurrence-one malformed subtree. That changes valid, malformed, valid into
valid, valid, malformed. The shared removal and run-position logic produce the
same `w:rtl` failure. The new interleaving regression at
`crates/rdocx-oxml/src/properties.rs:2778` gives the final toggle an unsupported
attribute, so it retains the carrier and does not exercise this canonical-last
case.

### D2, a tab leader disables logical recovery for source-less markers and fields

`crates/rdocx-layout/src/paginator.rs:2793`

The pass 5 logical reconstruction requires every source-less positioned text
element to match a text-bearing logical item. A shaped tab leader is emitted as
source-less text at `crates/rdocx-layout/src/paginator.rs:2530`, but its logical
counterpart is `InlineItem::Tab`, which `inline_item_text` does not recognize at
`crates/rdocx-layout/src/paginator.rs:2828`. The failed lookup returns `None`
for the whole line and restores the old visual-order fallback. An RTL numbered
paragraph whose numbering suffix resolves through a dot-leader tab therefore
extracts the visually ordered body, leader, marker sequence instead of its
logical marker prefix. The same failure can displace a source-less stored field
on a line containing a leader. The new marker and field regressions contain no
tab leader, so the supported tab-stop interaction remains uncovered.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass 5 D1 is closed when the latest valid paragraph or run toggle retains an
  unsupported attribute. Its carrier establishes the intended boundary around
  interleaved malformed raw occurrences. D1 above is the remaining canonical
  final-toggle case.
- Pass 5 D2 is closed for numbering markers and stored fields on lines without
  generated leader text. Their logical input rank is restored while positioned
  origins remain visual. D2 above is the remaining tab-leader interaction.
- All earlier duplicate clearing, unsupported-attribute replay, natural numeric
  levels, line-local L1 then L2 ordering, conditional hyphenation, absent base
  inference, mixed rich and legacy extraction, inline object ordering, stored
  field direction, and private drawing-reflow direction paths remain intact.
- Direction remains in paragraph cache identity and private cached output. No
  warm-versus-fresh, cached-tail, body, header, footer, note, or restart
  invalidation defect was found.
- Malformed multilingual output still passes through shared validation before
  PDF, raster, and SVG indexing. No new panic, unchecked slice, arithmetic
  overflow, or suppressed error was found.
- DrawingML and PowerPoint direction remain on the shared bidi path. The
  documented quarter-turn vertical approximations remain unchanged.
- Public `ParagraphReflow` retains its prior shape. The intentional pre-1.0
  property and `TextSegment` additions are documented, and no new public API,
  module, dependency, reverse crate edge, or structural smell was introduced.
- Exactly the five plan-listed HLD files changed. The recorded 49-of-49 hash,
  deterministic corpus, portability, package, archive, and supply-chain
  evidence is internally consistent. Current Rust oracle images are byte-equal
  to the reviewed five-of-five output, and the fresh four-of-five Writer run
  still meets the unchanged 80 percent hard gate without accepting its changed
  host-font artifact. No threshold, baseline, or oracle identity was weakened.
