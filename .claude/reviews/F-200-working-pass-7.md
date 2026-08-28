# F-200, working, pass 7

**Reviewed**: working diff against
`cf7627aa280c65a245dbed8fbd2988e80dae9201`, 21 files with 2,853 tracked
insertions and 164 tracked deletions, plus the pass 1 through pass 6 review
records
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, a selected conditional hyphen disables logical bidi extraction

`crates/rdocx-layout/src/paginator.rs:2799`

A conditional hyphen is emitted as a separate source-less text segment at
`crates/oxml-layout/src/line.rs:1116`. Its text is `-`, so it cannot match the
original source-bearing `InlineItem::HyphenatedText` word through the exact
text branch at `crates/rdocx-layout/src/paginator.rs:2793`. The only fallback
here assigns every otherwise unmatched source-less text element to an unused
`InlineItem::Tab`. Without a tab, the lookup fails and the `?` at
`crates/rdocx-layout/src/paginator.rs:2806` abandons logical reconstruction for
the whole line. With a shaped tab leader, the hyphen can consume the tab's
logical slot before the actual leader, or the leader can consume it first and
leave the hyphen unmatched. Either path restores the visual-order fallback for
an RTL line that actually selects F-198 conditional hyphenation, which can put
hyphenated legacy text, a source-less marker, or a stored field on the wrong
side of rich text during PDF and SVG extraction. The pass 6 regression at
`crates/rdocx-layout/src/paginator.rs:3698` includes a leader but no selected
conditional hyphen, while the existing mixed bidi and hyphenation tests do not
assert logical backend extraction after a hyphen is inserted.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass 6 D1 is closed. Removing a canonical final paragraph or run direction
  carrier keeps earlier raw duplicates and malformed occurrences on their
  original side of the modeled value. Unsupported attributes, typed clearing,
  and repeated serialization remain stable.
- Pass 6 D2 is closed for source-less markers and fields on a line with a
  shaped tab leader but no generated conditional hyphen. Exact source-less
  matches and the one tab fallback restore logical order without changing
  visual origins. D1 above is the remaining F-198 interaction.
- Earlier natural numeric levels, line-local L1 then L2 ordering, absent base
  inference, inline object ordering, duplicate direction handling, stored-field
  direction, and private drawing-reflow direction remain intact.
- Logical source spans stay scalar-based across script, font, cluster, and bidi
  segmentation. Rich-run validation still protects PDF, raster, and SVG from
  malformed public glyph arrays. No new panic, unchecked slice, arithmetic
  overflow, or suppressed error was found.
- Direction remains part of paragraph cache identity and private retained
  output. No separate warm-versus-fresh, cached-tail, body, header, footer,
  footnote, or endnote invalidation defect was found.
- DrawingML and PowerPoint remain on the shared paragraph direction path.
  Forced breaks and the documented quarter-turn vertical approximations remain
  unchanged.
- Public `ParagraphReflow` retains its prior shape. The intentional pre-1.0
  Word property and shared `TextSegment` additions are documented. No new
  public API, module, dependency, reverse crate edge, or structural smell was
  introduced.
- Exactly the five plan-listed HLD files changed. The recorded 49-of-49 hash,
  deterministic corpus, package, archive, portability, supply-chain, and
  accepted five-of-five output binding remain internally consistent. No
  threshold, baseline, or oracle identity was weakened. Those gates do not
  cover D1's conditional-hyphen extraction interaction.
