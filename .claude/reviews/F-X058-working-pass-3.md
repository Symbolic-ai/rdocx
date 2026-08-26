# F-X058, working, pass 3

**Reviewed**: complete current working-tree diff against
`f74420c3b6f553ab2e3e139eb9e4f54074496adf`, 31 tracked files with 3,007
insertions and 173 deletions, plus seven untracked font, licence, notice, and
subset-provenance assets. The pass-1 and pass-2 review records were treated as
review evidence rather than implementation scope.
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, forced breaks restart bidi resolution and logical indexing at every styled run
`crates/rpptx-render/src/text.rs:520`

Any forced break prevents the paragraph-wide shaping branch, so the fallback
calls `shape_multilingual_text` separately for every text run. Each call starts
a new bidi paragraph and assigns logical indices from zero at
`crates/oxml-layout/src/font.rs:1348`. In an explicit RTL paragraph containing
two styled numeric runs followed by a forced break, both runs therefore have
level 2 and logical index 0. Line reordering paints them in reversed visual
order, while the extraction-order restoration at
`crates/rpptx-render/src/text.rs:920` cannot restore tied indices and leaves PDF
`ActualText` in visual rather than logical run order. Per-run bidi resolution
also loses weak and neutral context across style boundaries. The remediation
regression at `crates/rpptx-render/src/text.rs:3155` has only one text run on
each side of the break, so it proves the requested base level but not the
approved paragraph-wide logical sequence or logical extraction contract.

### D2, completed lines apply UAX 9 rule L2 without the required line-local rule L1
`crates/oxml-layout/src/line.rs:642`

The rich line breaker calls `BidiInfo::reorder_visual` directly on one stored
level per multilingual span. That operation applies rule L2 only. The function
has neither the paragraph's original bidi classes nor a line byte range, and
its base-direction argument is unused at
`crates/oxml-layout/src/line.rs:619`, so it cannot reset segment separators and
trailing whitespace to the paragraph level under rule L1 before reordering.
For example, when a mixed LTR paragraph wraps after whitespace between RTL
runs, the line-ending whitespace keeps its paragraph-resolved RTL level and is
painted with the RTL run instead of being reset to the LTR paragraph level.
The exact mixed-bidi regression covers only one unwrapped line without this
boundary, so the approved UAX 9 line-local ordering contract remains
incomplete.

### D3, malformed rich runs bypass validation in the SVG backend
`crates/rdocx/src/svg.rs:180`

PDF and raster consult `MultilingualGlyphRun::is_valid`, but SVG immediately
projects every public rich run to a legacy run. A two-glyph run with a
non-finite x advance is rejected by the shared validation at
`crates/oxml-layout/src/output.rs:255`, yet SVG passes that value to its x-list
emitter and serializes `NaN` into numeric SVG attributes. Other malformed
vector shapes can likewise reach the approximation path rather than a guarded
omission or diagnostic. This produces invalid SVG from a public value that the
other backends safely reject, and the existing SVG regression exercises only a
valid rich run.

## Smells

None.

## Nitpicks

None.

## Not found

The pass-2 D1 remediation restores the exact exhaustive `ResolvedParagraph`
shape. Its additive direction sidecar remains aligned with resolved shapes,
ordinary text bodies, and row-major table cells, and the established resolver
and renderer entrypoints retain their signatures. The exact single-run
numeric RTL forced-break case from pass-2 D2 now reaches level 2 on both lines.

Pass-1 font-coverage segmentation remains grapheme-safe. Valid rich runs retain
complete finite positioning and ordered cluster maps. PDF and raster guard
malformed run vectors without indexing panics. Valid SVG output remains
searchable and reports its explicit positioning approximation. Arabic joining,
Indic clusters, Thai source spans, CJK line edges, conditional hyphen source
rules, and the removal of the forwarding-only API produced no additional
findings.

OOXML namespace handling, foreign same-local-name rejection, unknown attribute
and child preservation, schema child ordering, dependency direction, public
enum compatibility, deterministic font ordering, Noto legal and subset
provenance, archive inventory and size, WASM routing, legacy Latin hash
isolation, and the prohibitions on new modules, traits, generics, feature flags,
and forwarding wrappers produced no additional findings.
