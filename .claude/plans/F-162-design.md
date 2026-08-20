# F-162, Field update policy

**Status**: completed
**Sprint**: S49
**Size**: M
**Depends on**: F-161

## Problem

The current `Document::save` and `Document::to_bytes` paths serialize in-memory
state directly through `flush_to_package` (`crates/rdocx/src/document.rs:620`).
There is no API that materializes F-161 evaluation outcomes into cached field
results, and the field model does not retain Word's `w:dirty` state. Ordinary
saves therefore cannot express update on demand, update on save, and leave
alone as distinct, testable policies.

The existing settings model is a read-only source-byte projection
(`crates/rdocx-oxml/src/settings.rs:130`). Its `w:updateFields` setting asks
Word to recalculate on open. That is different from this story's requirement
to control cached results and field-local dirty flags inside rdocx.

## Spec reference

- `docs/hld/14-development-backlog.md`, "Milestone 16, Document automation"
  and "F-162, Field update policy".
- `docs/hld/03-architecture.md`, "What stays put" and facade ownership.
- `docs/hld/08-rendering-spec.md`, "Word bookmark field pagination" if
  pagination-dependent caches are materialized.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The hash harness".

## Approach

Expose the three policies through explicit native operations instead of a
stored policy enum whose context would be unavailable at save time:

```rust
impl Document {
    pub fn update_fields(
        &mut self,
        context: &FieldEvaluationContext,
    ) -> Result<usize>;

    pub fn save_with_field_updates<P: AsRef<Path>>(
        &mut self,
        path: P,
        context: &FieldEvaluationContext,
    ) -> Result<()>;

    pub fn to_bytes_with_field_updates(
        &mut self,
        context: &FieldEvaluationContext,
    ) -> Result<Vec<u8>>;
}
```

`update_fields` is update on demand. The two new save methods are update on
save. Existing `save` and `to_bytes` remain leave alone and preserve cached
results plus dirty spelling exactly.

Stage all F-161 evaluations before mutation. If evaluation fails, leave the
document, caches, flags, and layout state unchanged. A resolved result replaces
only the typed cached result and clears `w:dirty`. Unsupported, malformed,
missing-context, and pagination-deferred PAGE, NUMPAGES, and PAGEREF outcomes
retain the cached result. An explicit update request marks those fields dirty
so Word may retry. Invalidate the layout caches once after a successful mutation
batch.

Apply the policy across the same approved typed paragraph boundary as F-161:
body paragraphs, tables, content controls, headers, footers, footnotes, and
endnotes. Raw text boxes remain byte-preserved and unevaluated.

Reuse F-161's `field.rs` module and the F-160 recursive field ownership tree.
Do not add an F-162 module. Do not synthesize or mutate `w:updateFields`, since
delegating work to Word on open does not materialize the cache required by the
gate and would broaden the settings preservation contract.

## Rejected alternatives

- Store a `FieldUpdatePolicy` and borrowed evaluation context in `Document`.
  Save-time external data has no safe document lifetime and two enum states
  would be observationally identical until a method call.
- Recompute fields in existing `save` by default. That violates leave alone
  and silently replaces intentionally stale cached results.
- Blank unsupported results. The sprint gate explicitly requires their cached
  display to survive.
- Set only `w:updateFields`. That asks another application to recalculate and
  does not produce the expected rdocx cache.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `field_update_policies_produce_the_expected_result_cache_and_dirty_flag` | On-demand, update-on-save, and leave-alone produce the approved cache and field-local dirty state |
| regression | `unsupported_fields_keep_their_cached_result_when_updates_run` | Unsupported and missing-context fields never become blank and are marked dirty after an explicit update request |
| regression | `ordinary_save_leaves_cached_field_results_and_dirty_flags_alone` | Existing save APIs preserve cache and dirty bytes |
| regression | `field_update_failure_leaves_document_bytes_unchanged` | Staging makes mutation atomic on error |
| round-trip | `simple_and_complex_dirty_flags_preserve_aliases_and_unmodelled_content` | On-off spellings parse, mutation writes fixed prefixes in schema order, and unmodelled neighbours survive byte for byte |
| regression | existing PAGE, NUMPAGES, REF, and PAGEREF tests | Update policy does not regress render-time field substitution |

The **test gate**, from the backlog, is regression. Each policy produces the
expected result cache, and an unsupported field retains its cached result.
Tests join the existing `rdocx` regression binary and `rdocx-oxml` unit module.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`

Record facade update ownership, atomic explicit update boundaries, cache and
dirty preservation, additive native-only save APIs, and unchanged binding
surfaces.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Check dirty on-off aliases,
  fixed-prefix mutation output, schema order, and byte-preservation of
  unmodelled simple and complex field content.
- Public API of published `rdocx`. Read HLD 10 and the structural rules. The
  three native methods are additive and do not expand Python, WASM, or CLI.
  Run the full package dry-run and archive-size assertion.
- Layout, pagination, line breaking, and text shaping only if pagination values
  are materialized. Preserve the one-pass algorithm and use deterministic fonts
  for every render check.
- No new F-162 module or file. It reuses the approved F-161 module and introduces
  no trait, generic, crate, or feature flag.

## Hash harness

Expected unchanged. Existing saves remain leave alone. Any sample delta means
the default path recomputed a field or changed serialization unexpectedly and
blocks integration.

## Implementation checklist

- [x] Retain and mutate field-local `w:dirty` on the F-160 model.
- [x] Traverse the approved F-161 story scope in document order.
- [x] Stage evaluation outcomes before changing any cache.
- [x] Apply resolved results, preserve unresolved results, and update dirty state.
- [x] Add explicit update-on-demand and update-on-save APIs.
- [x] Keep existing save and byte APIs leave alone.
- [x] Invalidate layout once after a successful mutation batch.
- [x] Add policy, unsupported fallback, atomicity, dirty, and round-trip regressions.
- [x] Run focused checks, package riders, and the unchanged hash harness.
- [x] Update exactly the approved HLD impact files at completion.

## Open questions

None. Field-local `w:dirty` is approved without settings-level `w:updateFields`.
Pagination-dependent caches are preserved and marked dirty on explicit update,
and the traversal scope exactly matches F-161.
