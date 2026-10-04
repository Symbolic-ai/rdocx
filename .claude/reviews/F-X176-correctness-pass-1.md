# F-X176, correctness, pass 1

**Reviewed**: working diff against `9f5e2a06`, 56 files, 253 added and 201 deleted lines.
**Verdict**: 1 defect, 0 smells, 0 nitpicks.

## Defects

### D1, stable release note names the wrong reviewed source

`CHANGELOG.md:12`

The `v0.15.0` highlights say the distribution moves from the reviewed S86
source. F-X176 changes the shared dependency pins and tag workflow, and the
planned stable release will be tagged from the S88 main merge. The published
release body would give readers the wrong provenance for that release. Name
the final reviewed S88 source in the highlights.

## Smells

None.

## Nitpicks

None.

## Not found

No additional defects in the selected package directory comparison, its
missing and extra package tests, version and lockfile carriers, README
measurements, tag workflow ordering, or unchanged hash harness. The diff adds
no OOXML parser or serializer path and no new trait, generic, crate or module.
