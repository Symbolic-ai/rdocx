# F-040, correctness, pass 1

**Reviewed**: uncommitted worker diff, 5 files, 212 changed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: save, matrix, clip, opacity, recursion, and restore occur in
  contract order with balanced graphics state at three levels.
- Contract: effects and raster group support remain staged, while nested image
  registration stays reserved for F-042 rather than using a wrong flat key.
- Panics: no input-reachable panic or unchecked indexing was introduced.
- OOXML: no parsing, namespace, whitespace, schema child order, or unmodelled
  XML behavior changed.
- Tests: the backlog balance gate and focused matrix, clip, opacity, and effect
  assertions would fail when the group arm is reverted.
- Structure: recursion remains private to the existing writer without a new
  trait, wrapper, module, feature, or dependency.
