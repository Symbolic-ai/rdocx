# F-X043, complete, pass 1

**Reviewed**: working diff, 2 files, 388 changed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness, contract, panics, OOXML, tests, and structure produced no
findings. The new path uses the deterministic engine, retains the strict
caller-only path, keeps engine state private, checks the complete retained-work
context including exact font bytes before transfer, and preserves both engines
on rejection. The focused tests exercise caller precedence, bundled-only
fallback, warm reuse, warm-cold equality, staged publication, poison recovery,
and compatible and incompatible transfer.
