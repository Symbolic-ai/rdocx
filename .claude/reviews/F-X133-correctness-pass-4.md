# F-X133, correctness, pass 4

**Reviewed**: working diff, 9 plan, source, spec, and measurement files, 273 added lines and 44 deleted lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness, contract, panics, OOXML namespace scope, schema order, raw child preservation, tests, and structure have no findings. The root serializers declare the prefixes claimed by the scope, including the conditional canonical `wp` declaration. The guard restores the prior binding mask on nested return and unwind. The writer compares both prefix and URI, leaving different URI shadows intact. Standalone and comment paragraphs keep their local bindings. The named gate failed before the new `r`, `mc`, and `wp` skip and passes with it.
