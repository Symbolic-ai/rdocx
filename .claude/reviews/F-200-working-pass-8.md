# F-200, working, pass 8

**Reviewed**: working diff against
`cf7627aa280c65a245dbed8fbd2988e80dae9201`, 21 files with 3,059 tracked
insertions and 164 tracked deletions, plus the pass 1 through pass 7 review
records
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, a generated hyphen can claim an identical source-less logical item

`crates/rdocx-layout/src/paginator.rs:2805`

The source-less branch performs the generic exact-text lookup before it tries
the conditional-hyphen key at
`crates/rdocx-layout/src/paginator.rs:2814`. A generated conditional hyphen is
therefore indistinguishable from a real source-less logical item whose text is
also `-`. Word numbering markers enter the logical stream without a source or
field kind at `crates/rdocx-layout/src/engine.rs:4557`, and a retained or
computed field such as `REF` can have the same shape. On an RTL hybrid line
where the generated hyphen is visited first, it consumes that marker or field
slot through the exact branch. The real marker or field is then assigned the
hyphenated word's rank, which swaps their logical PDF and SVG extraction
positions while leaving their visual origins apparently correct. The private
tab-position discriminator does not distinguish either text element. The pass
7 regression uses `7` for its only other source-less text and therefore does
not exercise this collision.

### D2, a leader is assigned to the first tab rather than its owning tab

`crates/rdocx-layout/src/paginator.rs:2799`

Every positioned leader is matched to the first unused logical
`InlineItem::Tab`. A tab without a leader emits no positioned text and never
marks its slot used. For logical content containing a plain tab, intervening
text, and then a leader tab, the only leader is consequently ranked at the
plain tab's position and moves ahead of the intervening text during logical
PDF and SVG reconstruction. Multiple leader tabs can likewise exchange their
logical positions when bidi painting visits them in visual order. The pass 6
and pass 7 regressions each contain exactly one logical tab, so they prove that
a leader cannot claim a hyphen but do not prove leader-to-tab identity.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass 7 D1 is closed for one selected conditional hyphen beside source-bearing
  text, a distinct stored field, and zero or one shaped leader. The generated
  hyphen keeps `source: None`, its source-bearing prefix keeps its exact range,
  and positioned visual origins do not move. D1 above is the remaining
  identical source-less text collision.
- The pass 1 through pass 6 parser closures remain intact. Unsupported
  direction attributes, duplicate valid and malformed occurrences, canonical
  final carriers, typed clearing, schema placement, namespace handling, and
  repeated serialization retain their reviewed behavior.
- Natural digit levels, line-local L1 then L2 ordering, explicit run spans,
  absent-base inference, inline objects, conditional-hyphen demotion, stored
  field direction, and private drawing reflow remain on the shared bidi path.
- Source-bearing rich and legacy runs retain scalar source intervals and
  logical backend ordering. Malformed rich runs still use the shared validation
  contract before PDF, raster, and SVG access. No new panic, unchecked slice,
  arithmetic overflow, or suppressed error was found.
- Direction remains part of paragraph cache identity and private retained
  output. No separate warm-versus-fresh, cached-tail, body, table, header,
  footer, footnote, or endnote invalidation defect was found.
- DrawingML and PowerPoint retain paragraph-wide direction through forced
  breaks. The documented whole-group quarter-turn vertical approximations are
  unchanged.
- Public `ParagraphReflow` retains its established shape. The intentional
  pre-1.0 Word property and shared `TextSegment` additions are documented. No
  new module, dependency, reverse crate edge, forwarding wrapper, or structural
  smell was introduced.
- Exactly the five plan-listed HLD files changed. The recorded 49-of-49 hash,
  deterministic corpus, package, archive, portability, supply-chain, and
  accepted five-of-five output binding remain internally consistent. No
  threshold, baseline, or oracle identity was weakened. Those gates do not
  cover D1 or D2.
