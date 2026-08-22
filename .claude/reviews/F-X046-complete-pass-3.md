# F-X046, complete, pass 3

**Reviewed**: Working tree against claim base `b895215`, 5 files and 690 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, field-free pages lose direct sharing after font canonicalization

`crates/rdocx-layout/src/engine.rs:1161`

The noncanonical font-id remediation marks a page for restoration only when it
has substitution state. A field-free page pushes `None` and continues without
setting `reuse_result_pages`. In a restart record that contains any field,
pagination restart is intentionally disabled, so the checkpoint path at line
1205 cannot mark that adjacent field-free page either. When persistent font ids
need canonicalization at line 1243, `Arc::make_mut` clones the recovered
pristine page, and the restoration loop leaves that clone in the result. The
page therefore fails the plan and HLD requirement that field-free output share
its retained pristine frame directly.

The existing noncanonical transition fixture has many field-free pages but
asserts pointer identity only for the field page at
`crates/rdocx-layout/src/engine.rs:6829`. Assert an unchanged field-free page is
also pointer-equal between `transitioned` and `warm`, then mark every exact
recovered field-free page for post-canonicalization restoration without
creating a pagination checkpoint.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-2 D1 for substituted pages: an exact field-page hit now skips shaping,
  canonicalizes only the pristine working page, and restores the prior
  result-local substituted `Arc`. The noncanonical bundled-family transition
  proves pointer identity.
- Pass-2 D2: production now constructs the complete candidate first and the
  accounting helper reads all outer vector capacities and nested body,
  outline, bookmark-target, font-identity, pristine-page, and substituted-page
  payloads directly from that candidate. Focused tests cover each capacity.
- Pass-1 D3: displayed page number has an explicit mismatch case.
- Pagination boundary: field-bearing blocks still produce zero restart
  checkpoints and cannot enter restart pagination.
- Correctness outside the defect above: all substitution inputs are compared
  exactly, mismatches reshape, and warm and cold results remain equal.
- Panics: no new untrusted indexing, slicing, unwrap, expect, or arithmetic
  failure.
- OOXML: no parser, serializer, namespace, child-order, whitespace, or raw XML
  preservation change.
- Structure: no new trait, generic, forwarding wrapper, feature flag, crate,
  module, or source file.
