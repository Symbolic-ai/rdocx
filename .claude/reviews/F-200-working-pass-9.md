# F-200, working, pass 9

**Reviewed**: working diff against
`cf7627aa280c65a245dbed8fbd2988e80dae9201`, 21 files with 3,514 tracked
insertions and 165 tracked deletions, plus the pass 1 through pass 8 review
records
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, cache source rebinding defeats the exact provenance match

`crates/rdocx-layout/src/paginator.rs:2836`

The new source match requires the logical reflow item and the rendered element
to carry the same source node. Cacheable body paragraphs are laid out with
`CACHE_SOURCE_NODE` at `crates/rdocx-layout/src/engine.rs:1925`, then retained
as a shared block while `ParagraphSemantics` carries the real node. Rendering
rebinds the positioned element to that real node at
`crates/rdocx-layout/src/paginator.rs:2433`, but it does not rebind the shared
`ParagraphReflow::items`. Every source-bearing lookup therefore misses in a
provenance layout as soon as a source-less leader, transformed run, field, or
generated hyphen activates `logical_reflow_elements`.

The related header and footer cache path also leaves
`InlineItem::HyphenatedText` untouched in the reflow rebinder at
`crates/rdocx-layout/src/engine.rs:3351`. Its conditional-hyphen lookup then
rejects the rebound line source at `crates/rdocx-layout/src/paginator.rs:2922`.
A cache-safe bidi paragraph or header with automatic hyphenation and a tab
leader consequently falls back to the visual-source heuristic instead of the
reviewed exact ranking. PDF and SVG can expose the leader or generated hyphen
in visual rather than logical order. Warm and fresh equality does not catch
this because both paths render the same placeholder-backed block.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass 8 D1 is closed for an uncached line. A generated conditional hyphen no
  longer claims an identical source-less numbering marker or untyped stored
  field, and all three keep their reviewed styles and logical ranks.
- Pass 8 D2 is closed for an uncached line. No-leader tabs participate in the
  private tab map, multiple leaders retain their owning logical slots after
  visual reversal, and their positioned origins do not move. D1 above is the
  remaining cache-boundary failure of that mapping.
- The pass 1 through pass 7 parser and layout closures remain intact.
  Unsupported toggle attributes, valid and malformed occurrence order,
  canonical final carriers, typed clearing, schema placement, namespace
  handling, and repeated serialization retain their reviewed behavior.
- Paragraph and run direction, natural digit and whitespace levels, line-local
  L1 then L2 ordering, conditional-hyphen demotion, stored field direction,
  inline objects, and private drawing reflow remain on the shared bidi path.
- Source-bearing rich and legacy values retain scalar intervals. Malformed
  multilingual runs still use the common validation contract before PDF,
  raster, and SVG access. No new panic, unchecked slice, arithmetic overflow,
  or suppressed error was found.
- Paragraph direction remains in retained cache identity. Related stories,
  note references, headers, footers, footnotes, endnotes, restart completion,
  and cached-tail behavior show no separate invalidation or duplication defect.
- DrawingML and PowerPoint retain paragraph-wide direction through forced
  breaks. The documented whole-group quarter-turn approximations are
  unchanged.
- Public `ParagraphReflow` retains its established shape. The intentional
  pre-1.0 Word property and shared `TextSegment` source impacts are documented.
  No new public API, module, dependency, reverse crate edge, forwarding
  wrapper, or structural smell was introduced.
- Exactly the five plan-listed HLD files changed and describe current behavior.
  The recorded 49-of-49 hash, deterministic corpus, package, archive,
  portability, supply-chain, and accepted five-of-five output binding remain
  internally consistent. No baseline, threshold, font, or oracle identity was
  weakened. Those gates do not exercise D1's source-node mismatch.
