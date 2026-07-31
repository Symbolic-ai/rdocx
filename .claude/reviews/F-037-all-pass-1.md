# F-037, all aspects, pass 1

**Reviewed**: working diff, 9 files, 1842 insertions and 9 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness found no divergence from the copied backend for the supported
text, line, rectangle, image, link, metadata, outline, and raster paths.

Contract found only the approved changes: format-neutral layout imports,
shared media probing, explicit unsupported path and group arms, constructor
use in moved tests, and the unpublished staged-package clarification.

Panics found no new production panic surface. Existing indexing and outline
assumptions are unchanged from `rdocx-pdf`, and image input remains guarded by
bounds and decoder checks.

OOXML found no parser, serialiser, namespace, child-order, or raw-subtree
change.

Tests found all eight gate cases capable of failing against the observed
pre-implementation stubs. The retained indexed PNG regressions and the normal
dependency-tree audit add focused coverage without changing the gate.

Structure found no new trait, generic parameter, feature flag, forwarding
wrapper, or released-family dependency. The new crate, manifest, and five
modules are the files explicitly approved by the F-037 design boundary.
