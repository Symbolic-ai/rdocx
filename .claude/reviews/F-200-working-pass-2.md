# F-200, working, pass 2

**Reviewed**: working diff against
`cf7627aa280c65a245dbed8fbd2988e80dae9201`, 19 files with 1,471 tracked
insertions and 126 tracked deletions, plus the pass 1 review record
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, duplicate typed direction toggles still lose or merge unsupported attributes

Paragraph direction attribute carriers record their actual modeled occurrence at
`crates/rdocx-oxml/src/properties.rs:458`, but the serializer removes every
carrier from raw replay at `crates/rdocx-oxml/src/properties.rs:1862` and only
attaches a carrier whose occurrence matches a generated element at
`crates/rdocx-oxml/src/properties.rs:1905`. Since the scalar `bidi` field emits
one generated occurrence, an unsupported attribute retained from a second
valid `w:bidi` occurrence is silently dropped. The run parser has the inverse
problem. Every valid `w:rtl` carrier is hard-coded to occurrence zero at
`crates/rdocx-oxml/src/properties.rs:1234`, so attributes from multiple valid
occurrences are merged onto the one generated `w:rtl` at
`crates/rdocx-oxml/src/properties.rs:2088`, potentially producing duplicate
qualified attributes. The single-occurrence attribute test at
`crates/rdocx-oxml/src/properties.rs:2614` and the valid-plus-nonempty malformed
duplicate test at `crates/rdocx-oxml/src/properties.rs:2634` do not exercise this
combined boundary. Pass 1 D1 and D2 are therefore not fully closed for malformed
duplicate typed toggles.

### D2, conditional hyphenation discards resolved levels from explicit-direction spans

The rich shaper now resolves natural byte levels inside an explicit override,
but `shape_word_multilingual_items` converts every non-complex hyphenatable span
back to the span's base `TextSegment` at
`crates/rdocx-layout/src/engine.rs:6616`. That projection has the requested
direction but no resolved bidi level. Line ordering consequently assigns the
one paragraph-relative directional level to every non-whitespace legacy span at
`crates/oxml-layout/src/line.rs:691` instead of using the level already computed
by the rich shaper. For example, the separately breakable spans of an explicit
RTL English run such as `ABC 123` lose their higher even levels and can be
reversed as separate odd-level items. The override regression checks only one
numeric rich span at `crates/rdocx-layout/src/engine.rs:7593`, while the
hyphenation interaction checks only the relative origins of one Arabic rich run
and one English legacy run at `crates/rdocx-layout/src/engine.rs:7467`. Pass 1
D3 is therefore not closed across the required F-198 interaction.

### D3, hybrid rich and hyphenated lines remain in visual extraction order

The paginator restores logical payload order only among positions that already
hold `MultilingualText` at `crates/rdocx-layout/src/paginator.rs:2607`. It does
not include legacy `Text` elements produced by the conditional-hyphenation
demotion at `crates/rdocx-layout/src/engine.rs:6616`. A visual RTL line that
contains Arabic rich text and hyphenatable English therefore keeps the English
`Text` element before the Arabic `MultilingualText` element in the output
sequence. PDF consumes that sequence directly at
`crates/oxml-pdf/src/writer.rs:1068`, with ordinary text emitted separately from
rich `ActualText` at `crates/oxml-pdf/src/writer.rs:1111` and
`crates/oxml-pdf/src/writer.rs:1155`. SVG likewise walks the positioned sequence
directly at `crates/rdocx/src/svg.rs:167`. The existing hybrid regression proves
only the visual origins at `crates/rdocx-layout/src/engine.rs:7474`. It does not
prove paragraph-logical PDF extraction or SVG DOM order. Pass 1 D4 is therefore
not closed for the F-198 hybrid path, contrary to the logical extraction
contract recorded at `docs/hld/08-rendering-spec.md:487`.

## Smells

None.

## Nitpicks

None.

## Not found

- Auto base inference is now shared by logical alignment, indents, rich shaping,
  and line ordering at `crates/rdocx-layout/src/engine.rs:5098`. Pass 1 D5 is
  closed.
- Tabs, images, groups, and figures enter the bidi paragraph as neutral
  characters at `crates/oxml-layout/src/line.rs:642`, and L2 reorders their
  original items without changing text source spans at
  `crates/oxml-layout/src/line.rs:703`. Pass 1 D6 is closed.
- The shared validated multilingual-run contract still protects PDF, raster,
  and SVG from malformed public glyph arrays. No new panic or unchecked source
  conversion was found.
- Direction fields remain part of retained paragraph identity, while paragraphs
  with raw property sidecars remain cache-ineligible at
  `crates/rdocx-layout/src/engine.rs:2277`. No restart, cached-tail,
  header/footer, footnote, or endnote invalidation defect was found.
- PowerPoint still uses paragraph-wide direction through forced breaks, and the
  documented quarter-turn transforms remain covered at
  `crates/rpptx-render/src/text.rs:1503` and
  `crates/rpptx-render/src/text.rs:1527`.
- Exactly the five plan-listed HLD files changed. The recorded 49-of-49 hash,
  package, portability, and five-page oracle evidence remains internally
  consistent. The remediated Rust oracle images are byte-identical to the prior
  accepted 5-of-5 run, and no threshold or baseline was changed.
- No new module, dependency, public abstraction, stable reverse edge, or
  structural smell was found.
