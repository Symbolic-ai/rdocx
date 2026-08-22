# F-X044, Scale paragraph-cache lookup for editors

**Status**: approved
**Sprint**: S52
**Size**: M
**Depends on**: F-X040

## Problem

The reusable paragraph cache at `crates/rdocx-layout/src/engine.rs:1156`
clones `CT_P` into every lookup key, scans typed keys linearly, and removes and
reinserts every hit. Its 192-entry and 12 MiB limits at
`crates/rdocx-layout/src/engine.rs:524` thrash on the 700-paragraph editor
workload reported in PR 41.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "Performance".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The hash harness".

## Approach

Store a stable `u64` fingerprint beside every exact typed paragraph key and use
it only to prefilter candidates. Typed equality remains authoritative. Compute
the lookup fingerprint by borrowing the current paragraph, so hits do not clone
`CT_P`. Keep insertion-order eviction and do not remove and reinsert on a hit.
Raise the paragraph entry ceiling to 4,096 and its retained-byte ceiling to 56
MiB, leaving the existing table and restart allocations inside a 64 MiB
combined engine budget and room for F-X045. Preserve F-X040's rule that the
first traversal-sensitive block disables all later retained reads. Do not add
runtime timing instrumentation.

## Rejected alternatives

- A fingerprint as the cache identity can alias different paragraphs.
- LRU refresh retains an O(n) hit path in `VecDeque`.
- Unbounded growth makes editor performance a memory leak.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `paragraph_fingerprint_collision_requires_typed_equality` | Forced equal fingerprints never reuse different typed paragraphs. |
| regression | `editor_scale_paragraph_cache_avoids_warm_thrash` | A 700-paragraph document retains the safe workload and a one-paragraph edit rebuilds only its bounded region. |
| regression | `unsafe_prefix_still_disables_later_paragraph_hits` | Notes, fields, and numbering before a safe paragraph prevent stale downstream reuse. |
| regression | `paragraph_cache_failure_and_eviction_remain_bounded` | Late failure publishes nothing and FIFO entry and byte ceilings hold. |
| regression | `scaled_paragraph_cache_warm_equals_cold` | Pages, fonts, diagnostics, provenance, numbering, notes, fields, outlines, and bytes are equal. |

The test gate is **regression**. The deterministic hash harness remains
unchanged.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Layout and pagination: re-read `docs/hld/08-rendering-spec.md`, use
  deterministic font mode for every render gate, and require unchanged hashes.

## Hash harness

Expected to be unchanged. Lookup and eviction policy cannot alter output.

## Implementation checklist

- [ ] Add a borrowed fingerprint prefilter with typed equality.
- [ ] Remove hit-time paragraph cloning and queue refresh.
- [ ] Raise and prove the exact bounded editor-scale limits.
- [ ] Preserve traversal invalidation and transactional publication.
- [ ] Add collision, workload, failure, bounds, warm-cold, and hash tests.

## Open questions

None. The reported 700-paragraph workload justifies the 4,096-entry ceiling,
and the 64 MiB combined cap retains an explicit memory bound.
