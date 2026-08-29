# F-X058, working, pass 2

**Reviewed**: complete remediated working-tree diff against
`f74420c3b6f553ab2e3e139eb9e4f54074496adf`, 29 tracked files with 2,605
insertions and 56 deletions, plus seven untracked font, licence, notice, and
subset-provenance assets. The pass-1 review record was treated as review
evidence rather than implementation scope.
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the direction carrier breaks an established public struct shape
`crates/rpptx-layout/src/lib.rs:314`

`ResolvedParagraph` is an exhaustive public struct whose fields are public, and
the remediation adds `base_direction` at
`crates/rpptx-layout/src/lib.rs:321`. Existing downstream code that constructs
the complete 0.6-era literal now fails to compile until it names the new field.
That contradicts the approved requirement to keep every established layout
struct shape unchanged and expose direction through an additive API. The
claimed legacy source-compatibility checklist item is therefore not true for
the PowerPoint layout surface.

### D2, an explicit RTL paragraph loses its base direction when it contains a forced break
`crates/rpptx-render/src/text.rs:471`

The paragraph-wide rich path is entered only when every inline item is text.
`ResolvedTextRun::Break` becomes `InlineItem::LineBreak` at
`crates/rpptx-render/src/text.rs:163`, which sends the paragraph through the
fallback loop. That loop shapes a text item richly only when its characters
independently require multilingual layout at
`crates/rpptx-render/src/text.rs:496`, even when `base_direction` is explicitly
RTL. An `rtl="1"` paragraph containing `123`, a forced break, and `456`
therefore leaves both numeric lines as legacy LTR text. The downstream rich
line breaker cannot recover the direction because its base-direction argument
is unused at `crates/oxml-layout/src/line.rs:619` and it reorders only
multilingual items at `crates/oxml-layout/src/line.rs:628`. The regression at
`crates/rpptx-render/src/text.rs:3134` covers only one uninterrupted numeric
run, so it does not detect this path.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-1 D2 is remediated in production by refining script and bidi spans at
font-coverage boundaries while retaining ICU grapheme boundaries. Pass-1 D3 is
remediated by preserving logical SVG text and emitting an explicit positioning
approximation diagnostic for every rich run. Pass-1 D4 is remediated by total
bidi-level, finite-position, and ordered in-bounds cluster validation, with the
line breaker returning an error rather than panicking. Pass-1 D5 is remediated
by shared run validation and guarded PDF, raster, and font-collection access.
Pass-1 D6 now has exact Arabic glyph and cluster claims, exact Thai source-safe
spans, exact mixed-bidi visual order, and completed CJK line-edge assertions.
Pass-1 S1 is gone, and no forwarding-only shaping API remains.

OOXML namespace handling, foreign same-local-name rejection, unknown attribute
and child preservation, schema child ordering, SVG searchability, PDF logical
`ActualText`, raster positioned access, conditional-hyphen fitting and source
rules, font authenticity and provenance, deterministic fallback order, archive
limits, dependency direction, error and panic paths beyond D2, public rich-run
validation beyond D1, exact named acceptance tests beyond D2, legacy 49-entry
hash isolation, and the prohibitions on new modules, traits, generics, feature
flags, and forwarding wrappers produced no additional findings.
