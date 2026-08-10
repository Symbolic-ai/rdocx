# F-119, all, pass 7

**Reviewed**: the remediated F-119 working diff from `87b5d92`, 3 tracked
files, 2,169 insertions and 60 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, cache raw emission adds the generic parameter the contract forbids

`crates/rpptx-chart/src/lib.rs:1436`

`emit_cache_point_raw` introduces `W: Write`, but both of its callers receive
`Writer<Vec<u8>>` at lines 1355 and 1390. There is only one instantiation
today, so the generic does not meet the repository's two-instantiation rule.
It also directly contradicts the completed design contract at
`.claude/plans/F-119-design.md:34`, which says F-119 adds no generic parameter.
Make this helper accept `Writer<Vec<u8>>`, matching the cache and point writers
that call it.

## Smells

None.

## Nitpicks

None.

## Not found

The pass 6 opaque-wrapper remediation retains each unsupported `c:tx`,
`c:cat`, and `c:bubbleSize` occurrence and rejects a public typed replacement
before writing a duplicate. Cache raw boundaries are emitted exactly once for
unchanged, shortened, grown, and zero-point vectors. Sparse indexes and their
larger logical count remain stable when cardinality is unchanged, while an
edited cardinality becomes dense and sequential. Preserved cache tails remain
after the final written point in both shrink and grow cases.

No other correctness, contract, panic, OOXML namespace, schema-order,
preservation, cache-consistency, test-gate, public-surface, or structural
findings were found.
