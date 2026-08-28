# F-199, working, pass 2

**Reviewed**: working diff against
`4225fb60fa5c14301c25c759c185c667b179c698`, 12 feature files with 1,383
tracked insertions and 71 tracked deletions, plus 122 lines of oracle licence
and provenance text and one 15,344-byte oracle font fixture
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, owned rich paragraphs retain the cache placeholder source node

`crates/rdocx-layout/src/engine.rs:3294`

`rebind_paragraph_source` rewrites only legacy text, markers, and tab leaders.
It does not rebuild `LineItem::MultilingualText` or the retained
`InlineItem::MultilingualText` values with the requested source node. This is
observable when a cached block is converted to an owned heading at
`crates/rdocx-layout/src/engine.rs:1255` and for every cacheable rich header or
footer, whose blocks are first built with `CACHE_SOURCE_NODE` at
`crates/rdocx-layout/src/engine.rs:6061` and then passed through this helper at
`crates/rdocx-layout/src/engine.rs:3336`. The resulting rich run can expose
source node 1 for the wrong story, or expose a source even when layout was
requested without provenance. The current source regressions exercise body
paragraph semantics, which override the cached node during pagination, and do
not cover these owned paths.

### D2, restored hyphenatable spans leave paragraph-wide bidi ordering

`crates/rdocx-layout/src/engine.rs:6490`

The D1 remediation converts every non-complex span of a hyphenatable run back
to legacy `HyphenatedText`. The shared rich line breaker constructs its bidi
paragraph only from `MultilingualText` at
`crates/oxml-layout/src/line.rs:622`, so those restored Latin spans no longer
participate in L1 and L2 ordering with the complex spans beside them. For an
automatic-hyphenation paragraph whose logical text begins with Arabic and ends
with an eligible English word, the Arabic run stays in its logical slot rather
than exchanging visual position with the English run under the RTL paragraph
level. The new regression at `crates/rdocx-layout/src/engine.rs:8073` places
English before Arabic, which proves the conditional hyphen survives but cannot
detect this reversed strong-script interaction.

### D3, concurrent macOS oracle runs can unregister each other's fonts

`scripts/docx_ssim_harness.py:268`

The macOS harness registers the four font URLs at CoreText session scope and
unregisters them after each Writer conversion, but it has no cross-process
serialization around that shared registration lifetime. Two harness
invocations can therefore overlap, after which the first invocation reaches
the unregister loop at `scripts/docx_ssim_harness.py:291` while the second
Writer process still depends on the same URLs. This already produced different
Arabic evidence scores in the recorded remediation work and required the
overlapping evidence to be discarded. A hard oracle gate must prevent that
interleaving rather than rely on callers to run one process at a time.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-1 D1's narrow conditional-hyphen symptom is fixed, although D2 above
  identifies a separate bidi interaction introduced by that remediation.
- Pass-1 D2 is fixed. Direct, bidirectional, and East Asian language values are
  selected at scalar-safe byte boundaries, and the slice helper adjusts the
  existing Unicode-scalar source intervals with checked arithmetic.
- Pass-1 D3 is fixed for both retained field text and a resolved `REF`. The
  private sidecar carries the resolved language and applies character spacing
  once after rich reshaping while preserving field metadata exclusions.
- Pass-1 D4 is fixed. The reviewed 0.8em ascent is retained on rich and legacy
  inline items before line construction, so wrapping-drawing reflow consumes
  the same baseline as the initial line pass.
- Correctness and contract beyond the cited defects: the Word projection uses
  the existing F-X058 rich values, preserves clusters and positioning, and
  keeps the F-X066 run model and OOXML raw/schema behavior unchanged.
- Panics and errors: byte slices are formed only from `char_indices`
  boundaries, source arithmetic is checked, and rich run construction retains
  the shared validation contract.
- Field substitution and notes: generated substitution fields remain on the
  legacy path, while rich note references preserve their note identity through
  pagination and endnote discovery.
- Latin identity and gates: the current evidence records 49 of 49 hashes
  unchanged, 194 layout tests passing, and the four-page hard raw oracle gate
  passing at the approved scores in a final isolated run.
- Public and shared compatibility: no new public type, field, dependency,
  module, or product font was added. The existing pre-1.0 rich variants remain
  the only complex-text representation.
- Oracle asset, provenance, and licence: the source and output SHA-256 values
  match the recorded bytes, the copied OFL is byte-identical to the product
  licence, and the three-file oracle inventory is exact.
- HLD scope and structure: exactly the five plan-listed HLD files changed, and
  no additional structural smell was found.
