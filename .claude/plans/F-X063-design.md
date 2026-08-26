# F-X063, Avoid duplicate caller-font byte comparisons

**Status**: completed
**Sprint**: S58
**Size**: S
**Depends on**: F-X052

## Problem

GitHub Issue 54 isolates a warm WASM relayout regression to one redundant
operation. `FontManager::load_additional_fonts` performs the authoritative
ordered family-and-byte comparison. `ReusableEngineContext::matches_input`
then compares the same retained `FontFile` byte vectors again on every layout.
With five caller fonts totalling about 22 MiB, that second pass dominates a
one-page editor relayout.

The existing caller-slice copy and the font manager's comparison predate this
regression and remain necessary for the current API. The fix is to avoid only
the repeated retained-context pass while preserving exact invalidation and
checked transfer.

## Spec reference

- `docs/hld/08-rendering-spec.md`, reusable engine context and caller-font identity.
- `docs/hld/12-testing-strategy.md`, deterministic work accounting and WASM regressions.
- `docs/hld/14-development-backlog.md`, "F-X063, Avoid duplicate caller-font byte comparisons".
- `docs/hld/15-build-and-toolchain.md`, reusable layout gates and WASM checks.
- [GitHub Issue 54](https://github.com/tensorbee/rdocx/issues/54), five-font reproduction and unsound shallow-comparison caveat.

## Approach

Keep exact font bytes in `ReusableEngineContext`. Split its private
compatibility check so normal warm layout skips the context's font equality
only after `FontManager::load_additional_fonts` reports that the exact ordered
font set is unchanged. Every other retained input remains exactly compared.

Checked engine transfer does not perform the preceding font-manager load, so
it continues to compare retained font family names and bytes exactly. A same
family and same length with changed bytes must reject reuse in both normal
layout and transfer.

Add test-only work accounting around the retained-context font equality, not
around the authoritative font-manager comparison. A generated five-font input
totalling about 22 MiB with 40 aliases must report zero repeated context bytes
on warm layout. Use structural work counts rather than a wall-clock threshold.

Do not add fingerprints that still scan every byte, shallow family-and-length
identity, `Arc` public API changes, a feature flag, dependency, module, trait,
or new test binary. The public `FontFile`, `LayoutInput`, and facade signatures
remain unchanged.

## Rejected alternatives

- Compare only family and byte length. Different fonts of equal length would reuse stale work.
- Hash 22 MiB on every layout. That replaces one full pass with another.
- Change `FontFile.data` to `Arc`. The reported regression does not require a public API break.
- Remove the font-manager comparison too. It is the authoritative changed-font invalidation.
- Use browser timing as the only gate. The reporter records large run-to-run variance.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `warm_layout_does_not_repeat_retained_context_font_byte_equality` | Five generated fonts totalling about 22 MiB perform zero second-pass context byte work |
| unit | `same_length_changed_font_bytes_still_invalidate_reusable_work` | Exact font-manager equality rejects an equal-length mutation and fresh output matches |
| unit | checked transfer font regression | Transfer retains exact ordered family-and-byte compatibility |
| regression | `five_large_caller_fonts_and_forty_aliases_keep_warm_and_fresh_layouts_equal` | The public bundled-fallback path retains page identity and complete fresh equality |
| integration | both WASM target checks | The reporter's execution target continues to compile with no binding change |

The **test gate is regression**. Five generated caller fonts totalling about
22 MiB and 40 aliases perform zero repeated retained-context font-byte work on
warm layout, same-length changed bytes still invalidate reuse, checked transfer
stays exact, and warm output equals fresh output.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Layout and pagination**. Use bundled deterministic fonts, exact warm and
  fresh equality, retained page identity, and test-only work accounting.
- **WASM**. Run both WASM target checks because the measured regression is
  WASM-specific, while retaining unchanged binding metadata.
- **External reporter evidence**. Credit the exact alternating-build
  attribution and retain the caveat that family-and-length comparison was only
  a deliberately unsound measurement patch.

## Hash harness

Expected unchanged across all 49 entries. This removes redundant comparison
work only. Any output delta blocks integration.

## Implementation checklist

- [x] Add real failing work-count, changed-byte, transfer, and facade regressions.
- [x] Separate warm context comparison from authoritative font equality.
- [x] Preserve exact checked-transfer font comparison.
- [x] Prove zero repeated context byte work and complete fresh equality.
- [x] Run focused suites, WASM, hash, full verification, and microscope.
- [x] Update exactly the four listed HLD files.

## Open questions

None. The fix preserves the public API and the authoritative exact byte check
while removing only the duplicated retained-context pass reported in Issue 54.
