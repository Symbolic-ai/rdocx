# F-X032, all aspects, pass 2

**Reviewed**: uncommitted working diff, 2 files and 417 changed lines, with 359 insertions and 58 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Pass 1 D1 is closed. The caller-font fixture creates a parseable family and
byte sequence absent from bundled fonts, then resolves every positioned
sourced run's font id to those exact caller-owned bytes.

Pass 1 D2 is closed. The test now populates the accepted cache before both
uncached tracked layouts and proves the original accepted `Arc` survives with
no additional accepted-layout invocation.

Pass 1 D3 is closed. Public integration coverage now renders distinct accepted
and tracked revision projections through the caller-font accessors and checks
both the recorded view and visible text.

No findings remain in correctness, contract, panics, OOXML ordering and
preservation, tests, public API shape, cache semantics, `Arc` identity,
accepted and tracked isolation, caller-font ownership, provenance resolution,
PDF and raster reuse, threading, or structure.
