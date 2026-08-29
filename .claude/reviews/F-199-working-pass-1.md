# F-199, working, pass 1

**Reviewed**: working diff against
`4225fb60fa5c14301c25c759c185c667b179c698`, 12 feature files with 1,130
tracked insertions and 71 tracked deletions, plus 122 lines of oracle licence
and provenance text and one 15,344-byte oracle font fixture
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, rich conversion drops F-198 automatic hyphenation

`crates/rdocx-layout/src/engine.rs:6316`

`multilingual_candidate` deliberately accepts `HyphenatedText`, but
`shape_word_multilingual_items` rebuilds every accepted span only as
`InlineItem::MultilingualText` at line 6406. That variant carries Unicode break
opportunities but not the F-198 language or Liang hyphenation candidates. A
paragraph with automatic hyphenation enabled, an English word such as
`representation`, and Arabic, Devanagari, Thai, or CJK text anywhere in the
same paragraph therefore loses the conditional hyphen that the established
Word path emits. The current tests cover hyphenation and multilingual shaping
separately, so this interaction is not exercised.

### D2, one mixed Word text node receives only one language slot

`crates/rdocx-layout/src/engine.rs:6270`

The language is selected before shared script segmentation. The presence of
any bidirectional character makes the function return `w:lang/@w:bidi` for the
complete text node, and East Asian text similarly selects `@w:eastAsia` for
the complete node. For a single `w:t` containing Latin, Arabic, and Han text
with distinct `val`, `bidi`, and `eastAsia` values, all three shaped spans are
therefore sent to HarfRust with the Arabic language. This contradicts the
complete effective language projection claimed in
`docs/hld/03-architecture.md:128`. The regressions use separate single-script
runs and cannot expose this case.

### D3, rich field text loses its resolved language and character spacing

`crates/rdocx-layout/src/engine.rs:4909`

Eligible stored and computed field text is pushed as ordinary `Text` without a
corresponding entry in the new index-keyed `multilingual_styles` sidecar.
When another item activates rich layout, field text whose `field_kind` is
`None`, including a resolved `REF`, is reshaped as multilingual text. The
missing sidecar entry defaults its spacing to zero at
`crates/rdocx-layout/src/engine.rs:6405` and supplies no resolved language.
This discards spacing that was already applied to the legacy field segment and
can change its width and break position. No regression combines a styled field
with complex text.

### D4, drawing reflow undoes the reviewed Word rich baseline

`crates/rdocx-layout/src/paginator.rs:1967`

The initial rich layout replaces each exact-spaced line ascent with the Word
0.8em baseline at `crates/rdocx-layout/src/engine.rs:5071`. A wrapping drawing
then sends the retained rich items through `break_multilingual_into_lines` and
assigns those shared lines directly, without reapplying the Word baseline.
The paragraph keeps its exact line height but returns to the fallback font's
hhea ascent, moving its glyph origins and recreating the vertical metric error
that the oracle calibration fixed. The baseline regression has no wrapping
drawing, so it cannot detect this second layout path.

## Smells

None.

## Nitpicks

None.

## Not found

- Contract and public API: no new public type, field, entrypoint, dependency,
  module, or product font was introduced.
- Panics and errors: the new rich-run indexing is protected by validated
  cluster and glyph ranges, and shaping cardinality mismatches return layout
  errors.
- OOXML and preservation: the feature does not alter parser namespace handling,
  schema child order, raw XML retention, or the F-X066 run sidecar.
- Backends and source flow: ordinary rich runs retain logical Word source spans,
  and the existing PDF, raster, and SVG consumers receive validated positioned
  runs.
- Oracle, legal, and packaging: the static Thin fixture matches its recorded
  source and output hashes, its OFL file is byte-identical to the approved Noto
  licence, the three-file inventory is exact, and package archives exclude it.
- HLD and gates: exactly the five plan-listed HLD files changed. The recorded
  four-page raw oracle gate is green, and the 49-entry Latin hash remains
  unchanged.
- Structure: no additional smell beyond the correctness findings above was
  found in the existing-file implementation.
