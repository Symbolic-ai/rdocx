# F-180, all aspects, recovery2 pass 1

**Reviewed**: Entire uncommitted F-180 implementation diff, 9 files, 6,766 additions and 2,021 deletions, plus all original and recovery reviews and the complete progress record
**Verdict**: 3 defects, 2 smells, 0 nitpicks

## Defects

### D1, applied Word style identities disappear without a diagnostic
`crates/rdocx/src/odt.rs:2070`

`paragraph_properties_have_unsupported` does not classify `style_id`, and
`run_properties_have_unsupported` at `crates/rdocx/src/odt.rs:2090` has the
same omission. The writer intentionally materializes effective formatting
instead of retaining Word style identifiers. A paragraph using a custom
paragraph style and a run using a custom character style therefore reopen with
the projected formatting but without either style identity, while
`OdtWriteResult::diagnostics` is empty for those losses. This contradicts the
approved diagnostic contract and the milestone rule that every lossy
conversion names what it dropped.

### D2, malformed retained paragraph metadata can still be dropped silently
`crates/rdocx/src/odt.rs:626`

The line-rule scan reports only values other than `auto` and `exact`, while the
generic predicate does not inspect `line_rule` or an unmatched `num_ilvl`. A
caller-built `CT_PPr` with `line_rule: Some("exact")` and no `line_spacing`
emits no line-height attribute and receives no diagnostic. A `num_ilvl` with no
`num_id` is likewise ignored by list detection and the loss predicate. Both
retained properties disappear on reopen without the stable path-aware
diagnostic required for unsupported or malformed source properties.

### D3, the dependency HLD lists ZIP features that are deliberately disabled
`docs/hld/15-build-and-toolchain.md:574`

The updated dependency-policy paragraph says the workspace enables Deflate64
and `time` in addition to Deflate. The actual workspace dependency enables
only `deflate-flate2-zlib-rs` at `Cargo.toml:89`, and the adjacent manifest
comment explicitly says the other codecs and timestamp support are disabled.
This approved HLD impact file therefore does not describe the current
dependency graph. The incorrect `time` claim is especially misleading because
enabling it changes `SimpleFileOptions::default()` from a fixed timestamp to
the current clock and would invalidate deterministic ODT bytes.

## Smells

### S1, the exact loss matrix omits the remaining silent-property cases
`crates/rdocx/src/odt.rs:6301`

The regression named for unsupported document content asserts a large exact
diagnostic vector, but it has no applied paragraph style, applied run style,
standalone line rule, or numbering level without an id. D1 and D2 can therefore
regress or remain absent while the test described as the loss matrix stays
green.

### S2, the determinism check samples one wall-clock instant
`crates/rdocx/src/odt.rs:4940`

The test serializes twice back to back and compares bytes, but does not assert
the timestamp or other fixed metadata of each ZIP entry. The current disabled
`zip/time` feature makes the test pass for the right reason today. If feature
unification enables clock timestamps later, both writes can still land in the
same two-second DOS timestamp bucket and leave this determinism gate green.

## Nitpicks

None.

## Recovery findings verified

- Second-recovery D1 is fixed. The source-built regression writes and reopens
  every accepted automatic line height from 1 through 24,000 twips, including
  all formerly underflowing values, and compares the exact retained integer.
- Second-recovery D2 is fixed. A drawing with neither inline nor anchor payload
  receives the exact run-content diagnostic, and text on both sides survives
  reopening.
- Second-recovery D3 is fixed. Direct and inherited distributed alignment both
  receive the exact `pPr/jc` simplification diagnostic and reopen as ordinary
  justification.
- All original and earlier recovery fixes remain present for formatting,
  numbering, tables, images, bounds, diagnostics, and atomic saves.

## Not found

- **Formatting and content**: apart from D1 and D2, no additional defect was
  found in effective paragraph or run projection, whitespace, fields, heading
  levels, list kind and nesting, multiple cell paragraphs, or sibling
  preservation.
- **Tables and media**: no additional defect was found in horizontal or
  vertical spans, covered-cell placement, continuation validation, image
  relationship checks, supported media bytes, or exact image dimensions.
- **Packaging and ownership**: with the actual workspace features, ZIP entry
  order, compression, permissions, timestamp defaults, manifest membership,
  media order, and repeated bytes are deterministic. Serialization does not
  mutate the source document, and failed staging preserves the destination.
- **Bounds and panics**: no new reachable panic or unchecked arithmetic was
  found in table geometry, style allocation, media resolution, generated XML,
  ZIP construction, or atomic replacement. Writer `expect` sites are backed by
  immutable facts established during the scan.
- **API and structure**: the only additive native surface is
  `OdtWriteResult`, `Document::to_odt_bytes`, and `Document::save_odt`. No new
  crate, module, source file, dependency, feature, trait, generic parameter,
  wrapper-only abstraction, Python, WASM, or CLI surface was added.
- **HLD scope**: the six modified HLD files exactly match the approved impact
  list and describe current state apart from D3. No unlisted HLD contradiction
  was found.
- **Verification evidence**: `cargo test -p rdocx odt_writer_ --lib` passes all
  29 focused writer tests, including the exhaustive automatic-line-height,
  empty-drawing, and distributed-alignment regressions. `git diff --check`
  passes before this review artifact is added.
