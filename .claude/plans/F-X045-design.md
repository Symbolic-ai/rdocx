# F-X045, Cache headers and footers transactionally

**Status**: completed
**Sprint**: S52
**Size**: M
**Depends on**: F-X040, F-X042

## Problem

Body paragraphs and safe tables reuse retained layout work, but every section
header and footer is rebuilt on each edit. PR 41 demonstrates the performance
opportunity, but its hash-only key omits media and full geometry, publishes
before whole-layout success, and has no retained-byte ceiling.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "Headers and footers" and "Performance".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The hash harness".

## Approach

Add a private typed header and footer cache beside the paragraph and table
caches. Exact identity includes variant and story kind, the complete section
properties and geometry, resolved referenced part bytes, media and watermark
inputs, styles, numbering, notes, theme, revision view, fonts, source mode, and
the remaining reusable-engine context. Cache only variants whose traversal
state is fully represented. Rebind current source ids and replay diagnostics
and exact font traces on a hit. Stage pending entries and publish only after the
whole layout succeeds. Bound header and footer retention to 64 entries and 4
MiB while keeping the complete engine at or below 64 MiB.

## Rejected alternatives

- A 64-bit hash cannot be authoritative for rendering reuse.
- Part text plus content width misses page height, media, and watermark state.
- Immediate publication survives late layout failures.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `safe_header_footer_variants_reuse_exactly` | Default, first, even, inherited, header, and footer variants hit only under exact identity and replay diagnostics, fonts, and provenance. |
| regression | `header_footer_media_geometry_and_context_changes_miss` | Text, image bytes, watermark, same-width different-height geometry, styles, fonts, revision, and section changes rebuild. |
| regression | `header_footer_cache_publishes_transactionally_and_stays_bounded` | Late failure publishes nothing and entry plus true retained-byte ceilings hold. |
| regression | `cached_header_footer_warm_equals_cold` | Complete Word layout and deterministic PDF bytes equal a fresh layout. |

The test gate is **regression**. The exact F-X042 first, even, default,
inherited, and multi-section PDF proof remains green.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Layout and pagination: re-read `docs/hld/08-rendering-spec.md`, use
  deterministic fonts for all backend evidence, and require unchanged hashes.

## Hash harness

Expected to be unchanged. Header and footer reuse must equal a fresh build.

## Implementation checklist

- [x] Define the complete typed safe header and footer identity.
- [x] Add diagnostic, font-trace, and source replay.
- [x] Stage publication under the whole-layout transaction.
- [x] Enforce the 64-entry and 4 MiB retained bounds.
- [x] Add variant, invalidation, failure, bounds, warm-cold, PDF, and hash tests.

## Open questions

None. Exact identity and conservative bypass resolve the unsafe cases in the
PR prototype.
