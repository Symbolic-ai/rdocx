# F-X058, Shared multilingual text substrate

**Status**: completed
**Sprint**: S58
**Size**: L
**Depends on**: F-196, F-197, F-X061

## Problem

F-198, F-199, and F-200 all need new behavior in the published shared layout
family. Implementing a stable Word consumer before that shared API is published
makes the registry package gate fail, as the F-198 worktree proved when
`rdocx-layout` used `InlineItem::HyphenatedText` against published
`oxml-layout@0.6.0`.

Publishing incomplete APIs one story at a time would require several shared
releases. Adding fields to existing exhaustive public structs would also make
the current stable workspace fail before the release, because its 0.10.1
source still constructs the 0.6.0 shapes. S58 needs one additive shared
substrate that keeps the legacy path source-compatible until stable consumers
opt into the published 0.7.0 path.

## Spec reference

- `docs/hld/03-architecture.md`, shared shaping, line breaking, and package-family direction.
- `docs/hld/05-drawingml-model.md`, text property hierarchy and raw reconciliation.
- `docs/hld/08-rendering-spec.md`, exact shaping, source spans, complex breaking, direction, and deterministic output.
- `docs/hld/10-bindings-spec.md`, public layout types, pre-1.0 versioning, packaging, and WASM.
- `docs/hld/12-testing-strategy.md`, deterministic structural, golden, and package gates.
- `docs/hld/15-build-and-toolchain.md`, bundled fonts, licences, dependencies, and archive limits.
- `docs/hld/14-development-backlog.md`, "F-X058, Shared multilingual text substrate".

## Approach

Build the complete format-neutral API and behavior needed by the three stable
stories inside existing incubating files. Keep every 0.6-era public struct and
function shape unchanged. Add sibling rich values and non-exhaustive variants
for positioned glyphs, logical cluster ranges, source ranges, script,
language, paragraph and span direction, and line-local visual order. Existing
`ShapedText`, `TextSegment`, `GlyphRun`, and legacy layout entrypoints remain
valid and preserve byte-identical behavior.

The additive public shape is:

```rust
pub enum TextDirection {
    Auto,
    LeftToRight,
    RightToLeft,
}

pub struct GlyphCluster {
    pub glyph_start: u32,
    pub glyph_end: u32,
    pub char_start: u32,
    pub char_end: u32,
}

pub struct MultilingualTextSegment { /* validated private fields */ }
pub struct MultilingualGlyphRun { /* complete positioned output */ }
```

`MultilingualTextSegment` uses a validating constructor and read-only
accessors for its base formatting, language, direction, bidi level, x and y
advances, x and y offsets, and clusters. `MultilingualGlyphRun` carries the
corresponding positioned output. Add `InlineItem::MultilingualText`,
`LineItem::MultilingualText`, and the existing output enum's multilingual text
variant. Add paragraph-scoped multilingual shaping and line-breaking functions
that accept the existing concrete `FontManager` and an explicit base
direction. Do not add fields to `LineBreakParams`, a trait, a generic, a
forwarding wrapper, or a new module.

Add the conditional-hyphen `InlineItem` path and its full shared implementation
from the paused F-198 worktree. The line breaker computes approved Liang
opportunities, chooses the farthest fitting break, emits the generated hyphen
with no source, preserves contiguous source spans, and removes discretionary
behavior in no-wrap mode. Use exact `hypher` 0.1.7 with only the approved
English, French, German, and Spanish data.

Add script and font coverage segmentation for Arabic, Devanagari, Thai, and
Simplified Chinese. Preserve graphemes and shaping clusters, pass explicit
script, language, and direction to HarfRust, and retain x and y advances,
offsets, cluster mappings, and logical source ranges. Use exact
`icu_segmenter` 2.3.0 with compiled data for complex-script boundaries. Add a
direct exact `unicode-bidi` 0.3.18 dependency for UAX 9 levels and line-local
visual ordering. These dependencies remain in the format-neutral layout layer
and introduce no reverse edge.

Expose direction through additive APIs rather than adding fields to existing
struct literals. Type DrawingML paragraph direction in its existing schema
position and migrate the incubating PowerPoint layout, PDF, raster, and SVG
paths to the rich representation. Preserve logical text for extraction and PDF
ToUnicode while painting in visual order. Keep the documented whole-group
quarter-turn vertical approximation unchanged.

Bundle the approved Noto Sans Arabic, Noto Sans Devanagari, and Noto Sans Thai
families plus a reproducible Noto Sans CJK SC subset limited to the approved
fixture repertoire and punctuation. Include authentic licences, notices,
provenance, deterministic fallback order, subset instructions, package
inventory, and archive-size evidence. Do not use system fonts for structural
or baseline evidence.

F-X058 does not type Word settings or properties, add Word authoring APIs,
enable rich Word layout, create the final source-built Word corpus, or accept
the final LibreOffice SSIM result. F-198 consumes the published conditional
hyphen path. F-199 consumes the published rich shaping path and owns the final
multi-script oracle. F-200 consumes the published direction path for Word and
owns the final bidi oracle. This boundary lets F-X059 publish one complete
shared API before any stable consumer requires it.

The paused F-198 branch is preserved. After F-X059 publishes 0.7.0, rebase or
reconstruct its worker diff against the new sprint base, drop the now-landed
`oxml-layout` conditional-hyphen implementation and dependency edits, retain
its Word parser, facade, authoring, showcase, HLD, and baseline work, then run
its complete gates against registry 0.7.0.

## Rejected alternatives

- Remove the F-198 enum variant while relying on unpublished shared behavior. That makes the compile gate green but the registry package render incorrectly.
- Add fields to existing public structs. Current stable 0.10.1 source constructs those shapes before 0.7.0 exists.
- Release once per product story. The three stories share one coherent text contract and would create avoidable registry churn.
- Duplicate line fitting in `rdocx-layout`. Drawing reflow and PowerPoint need the same format-neutral decisions.
- Rewrite logical text into visual order. That corrupts extraction, selection, source attribution, and round trips.
- Add a new module or feature flag. Existing font, line, output, drawing, and backend files are the direct ownership points.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `automatic_hyphenation_selects_the_farthest_fitting_break_and_has_no_source` | Conditional hyphens fit exactly, carry no source, and preserve original spans |
| unit | `unwrapped_hyphenated_text_never_emits_a_conditional_hyphen` | No-wrap layout strips discretionary behavior |
| unit | `arabic_joining_survives_script_and_line_break_boundaries` | Explicit shaping preserves joining and logical clusters |
| unit | `indic_clusters_are_never_split_or_mapped_as_independent_scalars` | Reordered clusters retain offsets and one logical interval |
| unit | `thai_words_offer_approved_breaks_without_losing_source_text` | ICU boundaries produce exact source-safe opportunities |
| unit | `cjk_prohibited_punctuation_never_starts_or_ends_a_line` | Opening, closing, and nonstarter rules hold |
| unit | `mixed_direction_line_uses_uax9_visual_order_without_changing_logical_text` | Line-local visual order and logical source order coexist |
| round-trip | DrawingML paragraph direction regression | Typed direction preserves unknown attributes, children, and schema order |
| integration | rich PowerPoint text path regression | Script fallback, clusters, offsets, direction, PDF search, raster, and SVG consume one shared result |
| regression | legacy stable source compatibility fixture | Unmodified stable 0.10.1 consumers compile against the additive local API |
| regression | `latin_shaping_and_hash_outputs_remain_byte_identical` | Legacy shaping and all 49 hash entries remain unchanged |
| packaging | deterministic font and legal-file inventory | Approved fonts, subset provenance, licences, notices, and archive limits verify |

The **test gate is regression**. Shared deterministic tests prove conditional
hyphens, exact logical source spans, cluster-safe Arabic and Indic shaping,
Thai and CJK breaking, bidi visual order, searchable logical text, and
unchanged legacy Latin hashes.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/05-drawingml-model.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/11-migration-plan.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Layout and pagination**. Use bundled deterministic fonts, exact logical and
  visual structural comparisons, full hash verification, and no legacy
  baseline recording.
- **Parser and serializer**. Exercise namespace aliases, foreign
  same-local-name rejection, schema order, duplicate handling, and unknown XML
  preservation for DrawingML direction.
- **Public API of published crates**. Record additive pre-1.0 types and
  variants, compile the legacy consumer fixture, run package dry runs, and
  enforce archive limits.
- **Crate dependency graph**. Keep `hypher`, `icu_segmenter`, and
  `unicode-bidi` in `oxml-layout`, run dependency-direction checks, and prove no
  reverse edge.
- **Bundled fonts and assets**. Verify checksums, authentic legal files,
  reproducible CJK subsetting, archive contents, and the 10 MiB limit.
- **External oracle comparison**. Retain structural evidence here. Final
  source-built Word corpus and pinned LibreOffice acceptance remain in F-199
  and F-200.
- **WASM bindings**. Run both WASM targets and no-default-feature checks for
  the shared layout crate.

## Hash harness

Expected unchanged across all 49 entries. Legacy consumers do not opt into the
new rich path, and F-198 retains exclusive ownership of the declared
`feature_showcase` baseline movement. Any output delta blocks integration.

## Implementation checklist

- [x] Add shared failing stubs without creating a new test binary or module.
- [x] Extract the reviewed conditional-hyphen implementation from the F-198 worktree.
- [x] Add additive rich shaping, cluster, offset, source, and direction types.
- [x] Preserve every legacy public struct and entrypoint shape.
- [x] Implement script and coverage segmentation plus complex breaking.
- [x] Implement UAX 9 line-local ordering with logical extraction.
- [x] Type DrawingML direction and migrate incubating consumers.
- [x] Add approved deterministic fonts, legal files, and reproducible CJK subset evidence.
- [x] Prove legacy stable source compatibility and unchanged Latin hashes.
- [x] Run scoped tests, no-default, WASM, package, archive, dependency, and full verification riders.
- [x] Update exactly the seven listed HLD files.
- [x] Reach a clean microscope before handoff and integration.

## Open questions

None. The user approved one additive shared substrate, the named dependency and
font choices, and the 0.7.0 release boundary before stable consumers resume.
