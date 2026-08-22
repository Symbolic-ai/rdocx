# F-168, all, pass 4

**Reviewed**: uncommitted `work/f-168-codex` diff, 4 files and 2,384 changed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Verified remediations

All three pass 3 findings are remediated. Native layout selects blank first and
enabled even variants without borrowing default watermark, header, or footer
content. The layout-only empty even reference records global activation without
creating a package relationship. The settings reader decodes XML attribute
values before evaluating `w:evenAndOddHeaders`. The section page-number reader
now requires the direct WordprocessingML child and namespaced `start` attribute,
rejects foreign and nested same-local elements, and decodes decimal and
hexadecimal numeric entities before parsing parity.

The complete diff also preserves the earlier repairs for package-visible first
and even watermark headers, same-type section inheritance, VML namespace state,
matching shapetype references, raw header byte preservation, API ownership,
schema child order, custom relationship target normalization, collision-safe
media identity, unresolved-image diagnostics, named and invalid colours,
margin-relative positioning, deterministic text shaping, watermark z-order,
atomic staging, cache invalidation, and the repository structural rules.

The focused suites passed with 15 `rdocx` watermark tests, 11
`rdocx-oxml` header and footer tests, the page-number namespace regression, and
the collision-safe media regression. `python3 scripts/prose_check.py` and
`git diff --check` also passed.
