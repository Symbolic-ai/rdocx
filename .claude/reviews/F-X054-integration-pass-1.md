# F-X054, integration, pass 1

**Reviewed**: staged squash integration against `dc9d53f`, 35 files and 4,512
changed lines with 4,411 additions and 101 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, EPUB invents decimal markers for producer-defined numbering
`crates/rdocx/src/epub.rs:2545`

`detect_list` sends every format other than `Bullet` and `None`, including
`ST_NumberFormat::Other("chicago")`, through `ListKind::Ordered`. A paragraph
using that producer-defined format is therefore wrapped in an HTML `ol`, which
supplies a decimal marker and advances the EPUB counter. The earlier diagnostic
scan also treats the definition as resolved, so it does not report the existing
flattening diagnostic. This violates the integrated F-X054 contract that an
unknown numbering format must not invent a marker. Known EPUB numbering
behavior remains green, but there is no integrated regression for the new
data-bearing variant.

### D2, ODT invents decimal markers for producer-defined numbering
`crates/rdocx/src/odt.rs:1249`

The ODT list-style branch asks only whether a level is a bullet. Every false
result emits `text:list-level-style-number` with `style:num-format="1"`.
`list_level_is_bullet` returns false for `ST_NumberFormat::Other(_)`, so an
unknown producer token becomes a visible decimal marker even though its
semantics are unknown. The diagnostic reports that substitution but does not
prevent it. This contradicts the staged HLD and F-X054 contract that export
consumers decline to invent a marker for producer-defined formats. The current
ODT diagnostic test covers a modeled `UpperRoman` value, not `Other(String)`.

## Smells

None.

## Nitpicks

None.

## Not found

The `lib.rs` conflict resolution retains `SvgDiagnostic`, `SvgRenderResult`,
and `CellItemRef` together with the complete F-X054 public exports. The EPUB
projection boundary clones the non-`Copy` numbering value without changing
known numbering behavior. The complete EPUB suite passed 33 tests with the
pinned external EPUBCheck test ignored by its environment guard. The ODT suite
passed 36 tests, the SVG suite passed 19 tests, the layout numbering suite
passed 14 tests, and the complete native regression binary passed all 169
tests.

No additional integration findings were found in automatic merges for document
namespace ownership and replay, ordered body and descendant facts, visible text
failure propagation, low-level numbering round trip, HTML and Markdown marker
suppression, RTF diagnostics, Python exception mapping, legacy flattened
accessors, public enum exhaustiveness, OOXML child order, panic safety, or the
repository structural rules. Focused Rust and Python binding checks and clippy
passed with warnings denied. The staged HLD edits are limited to the four files
listed by F-X054 and remain compatible with the already integrated F-180,
F-181, and F-182 contracts. Staged prose checking and `git diff --check`
passed.
