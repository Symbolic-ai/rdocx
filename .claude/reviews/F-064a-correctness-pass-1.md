# F-064a, correctness, pass 1

**Reviewed**: working diff, 4 files, 1021 additions and 5 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: no invalid inset range, autofit percentage range, duplicate
  schema choice, missing required shell child, or raw-boundary error found.
- Contract: no layout calculation, resize behavior, paragraph model, or work
  outside the approved text-body shell and body-property scope found.
- Panics: no panic path over malformed roots, attributes, empty choices,
  duplicate children, truncated XML, or opaque paragraph content found.
- OOXML: no body-property sequence error, text-body sequence error, prefix
  intolerance, noncanonical modelled prefix, or dropped raw child found.
- Tests: no missing approved gate, autofit form, strict range edge,
  malformed-input case, or vacuous preservation assertion found.
- Structure: no unjustified trait, generic parameter, wrapper, feature flag,
  dependency edge, module, or file found.
