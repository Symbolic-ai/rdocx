# F-200, working, pass 1

**Reviewed**: working diff against
`cf7627aa280c65a245dbed8fbd2988e80dae9201`, 19 files with 943 tracked
insertions and 121 tracked deletions
**Verdict**: 6 defects, 0 smells, 0 nitpicks

## Defects

### D1, typed Word direction toggles discard unsupported attributes

Both empty `w:bidi` and `w:rtl` are parsed directly into booleans at
`crates/rdocx-oxml/src/properties.rs:428` and
`crates/rdocx-oxml/src/properties.rs:1185`. Their complete element bytes are
not retained. The nonempty branches also capture raw bytes but discard them
when the element has no child at `crates/rdocx-oxml/src/properties.rs:347` and
`crates/rdocx-oxml/src/properties.rs:1257`. A modeled toggle such as
`<w:rtl xmlns:x="urn:foreign" x:flag="1"/>` therefore serializes as a canonical
toggle without `x:flag`. This violates the approved requirement to type these
properties without losing unknown XML. The round-trip test covers unknown
siblings, not unsupported attributes on the modeled elements.

### D2, a malformed duplicate direction element is replayed before the valid occurrence

The paragraph parser assigns every retained nonempty `w:bidi` occurrence the
hard-coded position `(PPR_BIDI_SLOT, 0)` at
`crates/rdocx-oxml/src/properties.rs:354`. The run parser does the same for
`w:rtl` at `crates/rdocx-oxml/src/properties.rs:1270`. This ignores the
occurrence already advanced for the modeled element. The serializer sorts raw
children by that recorded occurrence and writes them before the generated
occurrence at `crates/rdocx-oxml/src/properties.rs:1751` and
`crates/rdocx-oxml/src/properties.rs:1922`. Consequently, a valid direction
toggle followed by a malformed same-name subtree is emitted in the opposite
relative order. That breaks exact schema-positioned raw replay and repeated
round trips for malformed input.

### D3, a run override flattens the resolved bidi levels inside the run

An explicit segment direction replaces every byte-level UAX 9 result with one
forced level at `crates/oxml-layout/src/font.rs:1405`. Line-local processing
again replaces the adjusted level for every non-whitespace segment at
`crates/oxml-layout/src/line.rs:681`. Digits inside an RTL run therefore lose
their higher even level, and trailing or internal whitespace grouped with text
cannot receive its independent L1 reset before L2. The new integration test at
`crates/rdocx-layout/src/engine.rs:7527` uses an RTL `"123"` run but asserts
only its direction and source span, so it does not detect reversed numeric
glyph order or the whitespace regression.

### D4, Word emits rich positioned runs in visual order and loses paragraph-logical extraction order

The Word paginator appends `MultilingualGlyphRun` elements while walking the
visually reordered line and does not restore logical run order at
`crates/rdocx-layout/src/paginator.rs:2492`. The PDF backend then creates one
independent `ActualText` span per positioned run at
`crates/oxml-pdf/src/writer.rs:1155`, so extraction across multiple visual runs
follows visual element order rather than logical source order. PowerPoint
already preserves coordinates while sorting rich elements by `logical_index`
at `crates/rpptx-render/src/text.rs:924`, but the Word path has no equivalent.
The PDF regression at `crates/oxml-pdf/src/writer.rs:1759` contains only one
run and cannot prove paragraph-level logical extraction across mixed-direction
runs. SVG DOM order and diagnostics consume the same Word positioned sequence.

### D5, an absent Word bidi property gets two conflicting base directions

The Word engine maps an absent `w:bidi` to `TextDirection::Auto` at
`crates/rdocx-layout/src/engine.rs:4455`. It then treats `Auto` as left to right
when resolving start and end indents at `crates/rdocx-layout/src/engine.rs:4460`
and start and end alignment at `crates/rdocx-layout/src/convert.rs:25`.
However, line layout passes `None` to `BidiInfo` for the same `Auto` value at
`crates/oxml-layout/src/line.rs:653`, which infers the paragraph level from the
first strong character. An Arabic-first paragraph without `w:bidi` can
therefore reorder with an RTL base while its logical alignment and indents are
resolved as LTR. The explicit `w:bidi=true` test does not cover this absent
property boundary.

### D6, inline nontext objects do not participate in line-local bidi ordering

The bidi paragraph builder includes only text, markers, and multilingual text
at `crates/oxml-layout/src/line.rs:630`, and silently skips images, groups,
figures, and tabs at `crates/oxml-layout/src/line.rs:636`. L2 then reorders only
the collected text positions at `crates/oxml-layout/src/line.rs:689`. An inline
drawing beside RTL text remains at its logical array position instead of
participating as an object replacement neutral in the line's visual order.
This produces incorrect placement for mixed RTL text and inline Word drawings,
and no new bidi regression combines text with an inline object.

## Smells

None.

## Nitpicks

None.

## Not found

- Source ownership and cache invalidation: checked source-range construction
  remains scalar-based, and the F-X062 restart identity includes serialized
  paragraph content. The new paragraph and run direction fields therefore
  invalidate retained work without weakening exact warm-versus-fresh checks.
- F-198 and F-199 interaction outside D3: automatic hyphenation remains on the
  rich path, language and script segmentation retain source boundaries, and
  the five-script structural and oracle evidence exercises mixed shaping.
- Backend safety: malformed multilingual runs still use the shared validation
  contract before PDF, raster, or SVG indexing. No new untrusted-input panic or
  unchecked integer conversion was found.
- DrawingML and vertical behavior: the existing typed DrawingML direction path
  remains additive, PowerPoint preserves logical rich-run order, and the
  documented quarter-turn vertical approximation is unchanged and covered by
  its existing regressions.
- Public compatibility and dependency direction: the pre-1.0 exhaustive field
  additions are documented in the plan-listed bindings HLD. No new public
  abstraction, module, dependency, stable reverse edge, or forwarding-only
  wrapper was introduced.
- Tests, HLD, and evidence: exactly the five plan-listed HLD files changed. The
  recorded focused suites, 49-of-49 hash gate, four portability riders, package
  checks, and five-of-five raw oracle threshold are green. No HLD scope or
  structural smell was found beyond the correctness defects above.
