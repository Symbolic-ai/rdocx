# F-X046, complete, pass 1

**Reviewed**: Working tree against claim base `b895215`, 5 files and 426 changed lines
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, reused substituted pages can be remapped as persistent font ids

`crates/rdocx-layout/src/engine.rs:1189`

The reuse hit installs `cache.pages`, but those pages were retained after
result-local font canonicalization at line 1282. The same layout then passes
the reused page through `canonicalize_layout_fonts` at line 1211, whose rewrite
at line 2118 interprets every page font id as a persistent manager id. After a
font-context transition leaves a nonzero persistent id, the first layout in the
new context caches a page whose font id has been canonicalized back to zero.
The next identical warm layout reuses that page and tries to resolve or remap
zero as a persistent id. It can return `FontNotFound`, panic at the indexed
rewrite, or bind the wrong face when zero still names another retained font.
Exact page reuse must preserve whether the cached page holds persistent or
result-local ids, or occur after canonicalization without applying the remap a
second time.

### D2, the restart byte total omits retained vector allocations

`crates/rdocx-layout/src/engine.rs:2198`

The byte calculation sums the page payloads but does not charge the backing
arrays for either `Vec<Arc<PageFrame>>`. The new substitution array is charged
with `len()` at line 2213 rather than retained capacity. `shrink_to_fit` is not
required to make capacity equal length. A record whose reported size is at or
just below 2 MiB can therefore retain more than the promised ceiling. This
violates the plan's bounded-accounting contract and the HLD's retained-capacity
rule. Charge all three outer vector capacities, with saturating arithmetic, and
add a byte-bound case rather than only the existing 33-page entry-bound case.

### D3, displayed page number mismatch is not covered by the regression gate

`crates/rdocx-layout/src/engine.rs:6743`

The mismatch matrix mutates page index, total pages, bookmark pages, font
identity, and revision view, but never mutates `page_number`. The HLD requires
focused coverage for displayed page number at
`docs/hld/12-testing-strategy.md:195`, and that value is an independent member
of the reuse key at `crates/rdocx-layout/src/engine.rs:1166`. Add the missing
mutation so removing or mis-comparing that key member fails the named gate.

## Smells

None.

## Nitpicks

None.

## Not found

- Contract scope beyond the defects above: no unrelated behavior or API change.
- Panics beyond the font-id failure path described in D1: no additional unsafe
  indexing, slicing, unwrap, expect, or arithmetic on untrusted input.
- OOXML: no parser, serializer, namespace, schema-order, whitespace, or raw XML
  preservation change.
- Structure: no new trait, generic, forwarding wrapper, feature flag, crate,
  module, or source file.
- Exact identity: pristine page equality is promoted to `Arc` identity before
  the substitution key is checked.
- Pagination boundary: field-bearing blocks still yield zero restart
  checkpoints and cannot enter restart pagination.
