# F-X046, Reuse substituted pages exactly

**Status**: completed
**Sprint**: S52
**Size**: S
**Depends on**: F-X040

## Problem

Repeated warm layout reshapes PAGE, NUMPAGES, and PAGEREF fields before
F-X040 can restore equality with retained final pages. PR 41 avoids that work
but retains pristine and substituted pairs without an exact context contract or
memory bound.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "Word bookmark field pagination" and
  "Performance".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The hash harness".

## Approach

Extend the existing bounded restart record with exact pristine and substituted
page pairs. A page may bypass field reshaping only when its pristine `Arc`
identity, page index, total-page count, bookmark targets, font identity,
displayed page number, revision view, and complete substitution input match. Pages without fields
remain shared directly. Count retained pairs inside the existing 32-entry and
2 MiB restart budget, and drop the optimization on any mismatch or eviction.
Field-bearing blocks remain excluded from pagination restart. Their pair record
therefore carries zero checkpoints and cannot broaden F-X040's safe boundary.

## Rejected alternatives

- Pixel comparison misses field text, provenance, and invisible state.
- Total page count alone does not validate PAGEREF or page content.
- A second unbounded page cache duplicates the restart record.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `unchanged_page_fields_reuse_substituted_frames` | Stable PAGE, NUMPAGES, and PAGEREF pages reuse their prior substituted `Arc`. |
| regression | `changed_substitution_context_reshapes_pages` | Page index, displayed page number, page count, bookmark target, pristine content, font, or revision changes miss. |
| regression | `substituted_page_reuse_is_bounded_and_complete_equal` | Field-free pages stay shared, eviction respects restart limits, and warm and cold outputs match completely. |

The test gate is **regression**. Deterministic PDF and raster output and the
hash harness remain unchanged.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Layout and pagination: re-read `docs/hld/08-rendering-spec.md`, use
  deterministic font mode, and require byte-identical hashes.

## Hash harness

Expected to be unchanged. The optimization skips work only for exact retained
substitution state.

## Implementation checklist

- [x] Record exact substitution inputs beside retained page pairs.
- [x] Reuse before shaping only under complete identity.
- [x] Charge retained pairs to the restart budget.
- [x] Add field hit, mismatch, eviction, warm-cold, backend, and hash tests.

## Open questions

None. Reusing the existing restart record avoids another cache and gives the
optimization an established transaction and bound.
