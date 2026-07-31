# F-038, all aspects, pass 1

**Reviewed**: the five-file working diff, 599 inserted lines and 8 deleted
lines, against the approved F-038 design and its cited rendering, testing,
risk, backlog, and deterministic-build sections
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness, contract, panic safety, tests, and structure produced no findings.
OOXML ordering, namespace handling, and unmodelled subtree preservation are not
applicable because this diff neither parses nor serialises OOXML.

The public facade is the approved additive method and directly reuses the
existing deterministic layout cache. The test-only harness records Poppler
26.01.0, rasterises at 150 DPI, decodes PNG pixels to RGBA, and compares exact
dimensions and pixel digests. The observed negative gate changes one decoded
pixel and reports `proposal` precisely. No binary fixture or crate dependency
was added.
