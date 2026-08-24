# F-X052, all aspects, pass 1

**Reviewed**: working-tree diff, 8 files, 2,099 changed lines
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, cached headings lose result-local provenance

`crates/rdocx-layout/src/engine.rs:1102`

A cache-safe heading is first returned as a shared paragraph whose glyph sources
use the cache sentinel and whose real source node lives in the side overlay. The
heading branch clones that shared payload into `Owned`, but it neither applies
the overlay nor rebinds the cloned glyph sources. A heading after the first body
node therefore emits the sentinel source node rather than its actual body path.
This breaks exact warm and fresh provenance even on the cold provenance layout.

### D2, a font-trace overflow returns canonical cache provenance

`crates/rdocx-layout/src/engine.rs:1846`

Table layout canonicalizes every glyph source to the cache sentinel before it
checks whether the bounded font trace exists. When more than the trace limit of
resolution events occurs, `finish_paragraph_font_trace` returns `None` and the
function returns the canonicalized block as `Owned`. The side overlay is not
attached to an owned block, so a cache-safe table with an overflowed trace emits
the sentinel source node instead of its result-local nested source paths.

### D3, restart body accounting omits owned property payloads

`crates/rdocx-layout/src/engine.rs:2468`

`paragraph_key_retained_bytes` returns run and selected paragraph-vector bytes
without charging `CT_PPr` owned strings, tabs, borders, shading, or other
property allocations. `table_key_retained_bytes` at
`crates/rdocx-layout/src/engine.rs:2477` likewise omits table, row, and cell
property payloads and inherits the paragraph omission recursively. These
functions charge the cloned `Arc<CT_P>` and `Arc<CT_Tbl>` values in restart
records. A cache-safe paragraph with a very large style id, or a table with
large cache-safe property strings, can therefore retain more than the 8 MiB
partition while the candidate reports that it is within bounds.

## Smells

None.

## Nitpicks

None.

## Not found

- Contract drift beyond the defects above was not found.
- Independent panic or arithmetic defects were not found. The added expects
  and unreachable arms are guarded by private exact-topology invariants.
- OOXML order, namespace, whitespace, and unmodelled-subtree changes were not
  present in this diff.
- The named mixed-editor gate exercises the new cache, restart, page ownership,
  and instrumentation paths and cannot compile against the reverted engine.
- Structural violations were not found. The private paginator trait has the
  public and shared block implementations required by the approved design, and
  no new module, file, public type, or public generic was introduced.
