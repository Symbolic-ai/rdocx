# F-200, working, pass 4

**Reviewed**: working diff against
`cf7627aa280c65a245dbed8fbd2988e80dae9201`, 21 files with 2,039 tracked
insertions and 135 tracked deletions, plus the pass 1 through pass 3 review
records
**Verdict**: 2 defects, 1 smell, 0 nitpicks

## Defects

### D1, drawing reflow still bypasses bidi for field-only directional paragraphs

`crates/rdocx-layout/src/paginator.rs:1969`

The pass 3 direction carrier reaches the paginator, but reflow selects the
multilingual breaker only when an item is already `MultilingualText`. Stored
PAGE, NUMPAGES, and PAGEREF results remain ordinary `Text` elements at
`crates/rdocx-layout/src/engine.rs:4966`, even when their stored run direction
is explicit. An explicit-direction paragraph containing only those fields can
therefore use the multilingual breaker initially through the direction checks
at `crates/rdocx-layout/src/engine.rs:5140`, then fall back to
`break_into_lines` after a wrapping drawing changes its width. That fallback
ignores both the carried paragraph base and each field's run direction. The
pass 3 field regression stops at the initial paragraph block at
`crates/rdocx-layout/src/engine.rs:7693`, while the reflow regression includes
an Arabic rich item at `crates/rdocx-layout/src/engine.rs:13589`, so neither
test exercises this intersection.

### D2, source-less stored fields move out of logical extraction order

`crates/rdocx-layout/src/paginator.rs:2624`

Computed and retained field segments deliberately carry no source span at
`crates/rdocx-layout/src/engine.rs:4969`. When the paginator restores logical
text after visual bidi ordering, it attaches every source-less text element to
the most recently encountered source group, or to a leading bucket when no
group exists yet. It then sorts only the sourced groups at
`crates/rdocx-layout/src/paginator.rs:2652`. For an RTL visual sequence such as
a PAGE field, an English run, then a Hebrew run, the field stays in the leading
bucket and is emitted before both sourced runs instead of returning to its
original logical position between them. PDF consumes elements in that order at
`crates/oxml-pdf/src/writer.rs:1068`, and SVG does the same at
`crates/rdocx/src/svg.rs:174`. The pass 3 field regression asserts the stored
direction only, so it does not prove logical PDF or SVG order for a field mixed
with sourced rich and legacy text.

## Smells

### S1, reflow direction is encoded as a magic public inline item

`crates/rdocx-layout/src/block.rs:335`

`ParagraphReflow::items` is public content at
`crates/rdocx-layout/src/block.rs:330`, and callers can pass such blocks through
the public paginator at `crates/rdocx-layout/src/paginator.rs:322`. The pass 3
fix appends a synthetic empty marker with `FontId(u32::MAX)` and later treats
any last marker matching only that font, empty text, and empty glyph-id shape as
private direction state at `crates/rdocx-layout/src/block.rs:367`. This makes a
public content vector carry an undocumented positional sidecar. Inspection,
reordering, appending, or a caller-provided marker that matches the partial
predicate can expose, lose, or misinterpret paragraph direction. The state
needs a non-colliding representation that does not masquerade as reflowable
content, with any source impact handled under the plan's pre-1.0 compatibility
contract.

## Nitpicks

None.

## Not found

- Pass 3 D1 is closed for typed mutation. Modeled duplicate carriers replay
  only while the scalar direction remains present, and the regression clears
  both paragraph and run values and reparses them as absent.
- Pass 3 D2 is closed for the tested rich and conditionally hyphenated drawing
  path. The resolved paragraph base now survives both reflow passes. D1 is the
  remaining ordinary-field intersection.
- Pass 3 D3 is closed before pagination. Stored field text carries the resolved
  run direction directly. D1 and D2 are the remaining reflow and extraction
  interactions.
- Pass 1 and pass 2 closures remain intact for unsupported direction
  attributes, duplicate occurrence order, natural digit levels, conditional
  hyphenation, absent paragraph direction, inline objects, and source-bearing
  hybrid extraction.
- No new malformed-input panic, arithmetic overflow, unchecked rich-run access,
  namespace classification error, schema-order error, raw-replay regression,
  cache invalidation gap, backend validation gap, or quarter-turn vertical
  regression was found.
- Exactly the five plan-listed HLD files changed. Their current-state direction,
  extraction, compatibility, and test-gate descriptions agree with the intended
  contract, subject to D1 and D2.
- The recorded 49-of-49 hash result, five-of-five raw oracle result, current
  output hashes, deterministic corpus evidence, portability gates, package
  inventories, archive cap, and supply-chain result are internally consistent.
  No threshold, baseline, dependency direction, or evidence identity was
  weakened.
