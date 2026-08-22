# F-X046, complete, pass 2

**Reviewed**: Working tree against claim base `b895215`, 5 implementation files and 592 changed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, font canonicalization still prevents exact substituted Arc reuse

`crates/rdocx-layout/src/engine.rs:1241`

The remediation retains the substituted page before font canonicalization, so
the cache now safely owns persistent font ids. When canonicalization is needed,
line 1249 must clone that retained page and rewrite the clone to result-local
ids. The next identical warm layout repeats the same clone and rewrite, which
means its returned page cannot be pointer-equal to the prior returned
substituted page. This avoids the pass-1 double-remap failure but does not meet
the approved contract that an exact hit reuses the prior substituted `Arc`.

The transition test does not expose this case. It clears `input.fonts` at
`crates/rdocx-layout/src/engine.rs:6821`, which makes
`load_additional_fonts` reset the manager and `next_id` to zero. Use an
unchanged font universe and switch a field run from one resolved bundled family
to another, then compare the post-transition result page `Arc` with the next
identical warm result. That produces a noncanonical persistent id and requires
the cache to retain or restore the prior result-local frame after skipping
field shaping.

### D2, the byte-bound regression bypasses the production capacity calculation

`crates/rdocx-layout/src/engine.rs:6879`

The test computes `vector_bytes` itself and passes that number into
`restart_cache_bytes`. It therefore remains green if the production expression
at line 1273 omits either page-pair vector, the substitution array, checkpoints,
outlines, body entries, or the font trace. It also does not exercise the nested
bookmark and font-identity capacities that D2 required. Move calculation of
all retained vector capacities into the helper used by production, or build a
real eligible record that exceeds the byte ceiling, then assert that retention
is dropped. The regression must fail when any production-owned retained
capacity stops being charged.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness outside D1: substitution keys compare page index, displayed page
  number, total pages, sorted bookmark targets, font trace, and revision view.
- Pass-1 D3: the mismatch matrix now covers displayed page number explicitly.
- Pagination boundary: field-bearing blocks still receive no pagination
  checkpoint and cannot enter restart pagination.
- OOXML: no parser, serializer, namespace, child-order, whitespace, or raw XML
  preservation change.
- Panics outside the addressed font-id path: no new untrusted indexing,
  slicing, unwrap, expect, or arithmetic failure.
- Structure: no new trait, generic, forwarding wrapper, feature flag, crate,
  module, or source file.
