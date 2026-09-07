# F-X085, Memoize restart body identities once per layout

**Status**: completed
**Sprint**: S70
**Size**: M
**Depends on**: F-X075, F-X083

## Problem

Each retained body entry owns a fingerprint and exact serialized identity, but
`RestartBodyEntry::matches` serializes the current block again after a
fingerprint match (`crates/rdocx-layout/src/engine.rs:1118` and
`crates/rdocx-layout/src/engine.rs:1145`). The unchanged-body, first-change,
and common-suffix scans overlap and call that comparison repeatedly
(`crates/rdocx-layout/src/engine.rs:1847`). Restart record publication can then
serialize the changed-region entry again through `for_content`
(`crates/rdocx-layout/src/engine.rs:2242`).

The HLD requires exact typed identity after the fingerprint prefilter and a
bounded aggregate cache, but it does not justify repeating serialization work
inside one layout (`docs/hld/08-rendering-spec.md:727` and
`docs/hld/08-rendering-spec.md:767`).

## Spec reference

- `docs/hld/08-rendering-spec.md`, "Performance", exact restart identity,
  prefix and suffix scans, and aggregate retained-work limits.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", editor-scale restart and
  cache-bound regressions.
- `docs/hld/14-development-backlog.md`, "F-X085, Memoize restart body
  identities once per layout".
- GitHub Issue 69 and offered commit `eff0ea0c28b5eaf08180b09b58e0c0f486b7433b`.

## Approach

Introduce one private concrete tri-state identity memo local to a layout. One
slot per candidate body block distinguishes not computed, computed and
unserializable, and computed exact bytes. Split fingerprint comparison from
exact identity comparison so a fingerprint miss never serializes. Share the
memo across unchanged-body, first-change, common-suffix, and restart-record
publication. Move an already computed identity into the new retained entry so
publication cannot serialize it twice.

Measure the memo's peak populated slots and retained byte capacities under
`cfg(test)`. Drop it before returning from the layout and leave the persistent
restart-cache accounting unchanged. Review the offered patch as input, but
tighten its scope because it memoizes only the three scans and still permits
publication and argument evaluation to repeat work.

Keep the construct in `engine.rs`. It has one concrete use and needs no trait,
generic parameter, module, file, public API, dependency, or feature flag.

## Rejected alternatives

- Replace exact bytes with a digest. A collision would violate the authoritative
  equality contract.
- Compute every identity eagerly. Early fingerprint mismatch should remain
  cheaper.
- Retain the memo between layouts. That would duplicate restart-cache identity
  and invalidation state.
- Add a lifetime-generic helper. There is only one instantiation today.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `restart_body_identities_are_computed_at_most_once_per_layout` | A 715-block mixed body computes at most 715 candidate identities across scans and publication, with exact warm and fresh equality. |
| regression | `memoized_restart_identity_keeps_exact_bytes_authoritative_after_fingerprint_match` | A same-fingerprint, different-identity candidate still misses through the memo path. |
| regression | `restart_identity_memo_transient_memory_is_bounded` | Peak slots and retained capacity are measured and bounded, then the memo is dropped after the layout. |
| regression | existing 1,000-page and aggregate overflow gates | Persistent restart state keeps the 5,216-entry and 64 MiB ceiling. |

The **test gate is regression**. A 715-block edit computes at most 715 candidate
identities, warm and fresh layouts remain equal, the cache stays bounded, and
deterministic corpus hashes do not move.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Layout, pagination, line breaking, and text shaping**. Use deterministic
  bundled fonts for every baseline, preserve exact identity equality, and treat
  any hash delta as blocking.

## Hash harness

Expected unchanged across all 49 entries. The story removes repeated comparison
work without changing pagination or rendered output.

## Implementation checklist

- [x] Add identity-computation and transient-memory instrumentation and tests.
- [x] Add the private tri-state per-layout memo.
- [x] Separate fingerprint filtering from exact identity comparison.
- [x] Share the memo across all scans and restart-record publication.
- [x] Reuse computed identity bytes when publishing retained entries.
- [x] Preserve unserializable failure and aggregate cache behavior.
- [x] Run scoped checks, full verification, hash checks, and routed riders.
- [x] Update exactly the listed HLD files.

## Open questions

None. The backlog's at-most-once requirement applies to the complete layout,
including restart-record publication.
