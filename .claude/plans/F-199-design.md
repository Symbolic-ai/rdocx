# F-199, Complex script shaping

**Status**: completed
**Sprint**: S58
**Size**: L
**Depends on**: F-196, F-X059, F-X066

## Problem

Font fallback chooses one font for a complete run and explicitly documents the
mixed-script limitation at `crates/oxml-layout/src/font.rs:612`. HarfRust
guesses properties for the whole string, then layout discards shaping clusters
and glyph offsets at `crates/oxml-layout/src/font.rs:991`. The line breaker
uses Unicode opportunities at `crates/oxml-layout/src/line.rs:480`, but has no
script segmentation, Thai dictionary boundary, or protection for Arabic and
Indic shaping clusters.

The public shaped and layout values carry glyph ids and advances without
cluster mappings or offsets at `crates/oxml-layout/src/font.rs:49`,
`crates/oxml-layout/src/line.rs:131`, and
`crates/oxml-layout/src/output.rs:198`. PDF also assumes glyph position maps
one-to-one to Unicode at `crates/oxml-pdf/src/font.rs:32`, which is false for
ligatures and reordered scripts.

The deterministic bundled fonts are Latin-only. A regression at
`crates/oxml-layout/src/font.rs:2093` documents missing CJK coverage. The five
document Word corpus contains Arabic and Latin in `rtl.docx`, but no Indic,
Thai, or CJK fixture. The story's multi-script golden gate therefore requires
an approved font and corpus expansion before implementation can satisfy it.

## Spec reference

- `docs/hld/03-architecture.md`, shared shaping and exact-slice line breaking.
- `docs/hld/08-rendering-spec.md`, exact shaping, source spans, and deterministic output.
- `docs/hld/10-bindings-spec.md`, public layout types and versioning.
- `docs/hld/12-testing-strategy.md`, Word corpus, oracle pins, SSIM evidence, and hard gates.
- `docs/hld/14-development-backlog.md`, "F-199, Complex script shaping".
- `docs/hld/15-build-and-toolchain.md`, bundled fonts, licences, packaging, and artifact limits.

## Approach

Consume F-X058's published additive rich shaping path. It already owns script
and font coverage segmentation, explicit HarfRust properties, cluster-safe
breaking, positioned glyphs, logical source ranges, PDF ToUnicode mapping, and
the migrated incubating PowerPoint consumers. Do not add fields to legacy
shared structs or create another shaping representation.

Migrate Word layout and its PDF and raster path onto the same published rich
values. Project complete `w:lang` information from the F-198 parser, preserve
logical source ownership, and prove that Word and PowerPoint now consume the
same shaped span result. Retain exact-slice reshaping and every F-X058 Arabic,
Indic, Thai, and CJK structural regression while adding the final Word facade
and corpus evidence.

The approved script scope remains Arabic, Devanagari, Thai, and Simplified
Chinese. F-X058 owns the approved Noto families, reproducible CJK subset,
authentic legal files, fallback order, and `icu_segmenter` dependency. F-199
verifies that inventory and adds the multi-script documents as source-built
fixtures in existing test and harness entrypoints. Send those exact packages
through pinned LibreOffice and Poppler. The existing 0.95 SSIM on at least 80
percent of pages is a hard gate for the new fixtures.

## Rejected alternatives

- Keep one font and guessed direction for a whole run. Mixed-script input then
  loses glyphs or shapes a strong script with the wrong context.
- Preserve only advances. Complex marks and reordered glyphs need offsets and
  cluster mappings.
- Rewrite source text into visual order. That breaks extraction, selection,
  diagnostics, and round-trip fidelity.
- Split graphemes at font boundaries. A visible cluster must remain one shaping
  unit.
- Use system fonts for oracle baselines. Their coverage and metrics are not
  deterministic.
- Create new modules for segmentation or line rules. The existing font and
  line files already own those operations.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `arabic_joining_survives_script_and_line_break_boundaries` | Joining context and source ranges remain exact across fitting |
| unit | `indic_clusters_are_never_split_or_mapped_as_independent_scalars` | Reordered clusters retain one logical source interval and shaped offsets |
| unit | `thai_words_offer_approved_breaks_without_losing_source_text` | The approved Thai mechanism offers exact word boundaries |
| unit | `cjk_prohibited_punctuation_never_starts_or_ends_a_line` | Opening, closing, and nonstarter rules are enforced |
| integration | `mixed_script_fallback_uses_each_covering_font_without_boxes` | Script and coverage segmentation selects deterministic bundled fonts |
| integration | `complex_shaping_preserves_clusters_offsets_and_logical_source_spans` | Word, PowerPoint, PDF, and raster paths preserve exact shaped data |
| regression | `latin_shaping_and_hash_outputs_remain_byte_identical` | Existing Latin paths and all hash outputs remain unchanged |
| golden | `multi_script_corpus_pages_meet_the_reviewed_oracle_contract` | Every approved script fixture meets the chosen pinned-oracle gate |

The backlog test gate is **golden**: multi-script corpus pages match the oracle
within the recorded threshold.

## HLD impact

- `docs/hld/03-architecture.md`, script and font segmentation ownership and cluster flow.
- `docs/hld/08-rendering-spec.md`, complex shaping, break rules, offsets, source spans, and searchable output.
- `docs/hld/10-bindings-spec.md`, intentional public layout-struct source impact.
- `docs/hld/12-testing-strategy.md`, multi-script corpus inputs and exact acceptance rule.
- `docs/hld/15-build-and-toolchain.md`, bundled font families, provenance, packaging, and size impact.

## Risk routing

- Layout, line breaking, and shaping. Use deterministic bundled-font mode and
  reject incidental baseline changes.
- Bundled fonts. Read `docs/hld/15-build-and-toolchain.md`, include the real
  licence and notice for every family, verify deterministic lookup, inspect
  package contents, and enforce crate, wheel, and WASM size limits.
- Public API of a published crate. Read `docs/hld/10-bindings-spec.md`, state
  the pre-1.0 source break, run package dry-runs, and enforce archive limits.
- Crate dependency graph. Read `docs/hld/03-architecture.md` and keep any
  segmentation or Thai dependency inside `oxml-layout` without a reverse edge.
- External oracle comparison. Follow `.claude/skills/differential-testing.md`,
  pin tool and corpus identities, render at the recorded DPI, and keep complete
  page evidence.

## Hash harness

Expected unchanged at 49 of 49. The new path must be an identity for current
Latin fixtures, and broader fonts must not change Latin fallback. Any delta is
unexpected and must not be folded into F-198's isolated output change.

## Implementation checklist

- [x] Migrate Word onto F-X058's published rich shaping path.
- [x] Project complete effective Word language and exact logical source spans.
- [x] Prove Word and PowerPoint consume the same segmented result.
- [x] Retain Arabic, Indic, Thai, CJK, offset, and searchable-text regressions.
- [x] Verify the approved deterministic fonts, licences, notices, and subset provenance.
- [x] Add the approved pinned multi-script corpus inputs and oracle evidence.
- [x] Prove the existing Latin and hash outputs remain byte-identical.
- [x] Run all risk riders and update exactly the listed HLD files.

## Open questions

None. The approved scripts, fonts, generated fixtures, ICU4X segmentation, new
font and licence files, and hard multi-script SSIM gate are specified above.
