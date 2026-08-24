# F-178, correctness, pass 3

**Reviewed**: the twice-remediated F-178 working diff against `7bfa238`, 9
implementation and test files with 2,395 additions and 8 deletions, plus the
approved plan update
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: path reads are capped at limit plus one, parser and projection
  counters fail closed, and block, inline, list, table, span, CSS, and
  diagnostic order remain deterministic.
- Contract: the native additive API covers document and fragment input, stable
  diagnostics, supported CSS, source-ordered content, bounded resources, and
  save and reopen validation without adding network authority or bindings.
- Panics: production expectations cover static selectors and already-checked
  element nodes. Untrusted sizes, indexes, spans, counters, and conversions are
  bounded or checked.
- OOXML: preformatted and HTML line breaks use counted `w:br` runs. Paragraph,
  table, cell, grid-span, vertical-merge, and numbering output uses the existing
  typed schema-order writers.
- Tests: the six named gates fail if the API is reverted and cover the declared
  limits, limit-plus-one reads, comment preflight, saved line breaks, parser
  repair, unsupported constructs, CSS, lists, spans, and reopenability.
- Structure: the approved private module and direct `scraper` dependency have
  one concrete owner. No unjustified trait, generic instantiation, dynamic
  dispatch, forwarding-only wrapper, feature flag, crate, or second document
  model was found.
- Pass 2 D1 is resolved by counting both explicit and preformatted break runs.
- Pass 2 D2 is resolved by scanning each comment through its closing marker,
  with separate tests for repeated comments and markup-looking comment text.
