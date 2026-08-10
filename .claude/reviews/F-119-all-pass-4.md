# F-119, all, pass 4

**Reviewed**: the remediated F-119 working diff from `87b5d92`, 3
implementation files, 1,993 insertions and 44 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, projected series drop inherited namespace declarations needed by preserved payloads

`crates/rpptx-chart/src/lib.rs:395`

The plot-area projection now carries ancestor bindings far enough to recognize
an aliased series, but `Series` stores only declarations physically present on
the `c:ser` start tag. `Series::to_xml` at line 448 therefore cannot re-emit an
alias declared on `c:chartSpace`, `c:plotArea`, or the plot root. A valid chart
that declares `q` as ChartML on an ancestor and keeps an unsupported
`q:marker`, namespaced attribute, or producer extension inside `q:ser` produces
a standalone series with the preserved `q:` bytes but no `xmlns:q` binding.
The same gap lets an inherited conflicting `c` or `a` binding change the
meaning of preserved bytes when the writer installs its fixed bindings. Retain
the effective declarations required by preserved content, and reject inherited
conflicts on prefixes the standalone writer fixes.

### D2, unsupported optional wrappers do not reserve their schema occurrence

`crates/rpptx-chart/src/lib.rs:557`

The `c:tx`, `c:cat`, and `c:bubbleSize` branches record an occurrence only when
their wrapper contains a supported reference choice. An unsupported first
wrapper is pushed to raw children without marking the field as seen, so a
second wrapper with a supported choice is accepted and both are emitted. For
example, a `c:tx` containing an opaque `c:v` followed by a `c:tx` containing a
`c:strRef` round-trips as two `c:tx` children instead of returning the duplicate
modelled-child error required by the plan and HLD 09. Track wrapper occurrence
separately from whether its payload can be typed.

### D3, the completed design contract contradicts the implemented sparse-cache rule

`.claude/plans/F-119-design.md:70`

The completed plan says a declared point count that differs from the parsed
point count is rejected, while the parser at
`crates/rpptx-chart/src/lib.rs:993` accepts a larger logical count and HLD 09
documents that sparse producer caches retain it. Sparse caches are valid corpus
behavior, so the implementation and current-state HLD agree, but the
machine-consumed design contract still describes the opposite acceptance
rule. Revise the plan to distinguish an invalid count or out-of-range index
from a valid sparse cache before treating its checklist as complete.

## Smells

None.

## Nitpicks

None.

## Not found

The two pass 3 trigger cases are remediated. Mixed ChartML aliases resolve by
namespace URI throughout the typed reference and cache readers, and an
inherited foreign plot alias is no longer projected as ChartML. No additional
panic, numeric validation, fixed-prefix ordering, test-gate, crate-surface, or
structural findings were found.
