# F-244, all aspects, pass 3

**Reviewed**: working diff against `322a706d158613c5b67e175d7abe7c2c062b442c`, 12 files, 1,728 insertions and 37 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: staged settings and property mutations keep their package graph
  and typed state atomic, including removal and tolerant loading.
- Contract: the public surface remains within the bounded property and settings
  families in the approved design.
- Panics: checked validation covers caller-controlled identifiers, names, unit
  values, relationship allocation, and XML serialization paths.
- OOXML: fixed write prefixes, schema-positioned replacements, and unmodelled
  subtree preservation remain intact for parsed aliases and duplicate groups.
- Tests: the four named gates prove save and reopen, selective pruning,
  preservation, and deterministic output. The existing foreign-property
  regression proves malformed lookalikes remain preserved but unevaluated.
- Structure: the diff introduces no crate, module, trait, generic parameter,
  forwarding wrapper, or feature flag.
- Workflow: the capability-owner assertion removes completed F-244 by the same
  narrow rule already used for completed F-249 while the sprint ledgers retain
  the active F-244 owner until integration.
