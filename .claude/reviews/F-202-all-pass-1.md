# F-202, all, pass 1

**Reviewed**: complete working-tree diff against `394b120`, 2 files with 144
insertions and 8 deletions, 152 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the invocation gate accepts an inactive counter

`crates/rdocx-layout/src/engine.rs:8830`

The assertion accepts zero page-layout invocations. If either paginator branch
stops updating the test-only counter, its reset value remains zero and the
story's named invocation-count gate still passes. Prove the instrumentation is
active by checking the initial 1,000-page layout count, then require the warm
count to be positive as well as at most two.

## Smells

None.

## Nitpicks

None.

## Not found

Contract, panics, OOXML, and structure produced no findings. The production
diff changes only the approved entry ceilings, leaves both byte ceilings
unchanged, and adds no public API, dependency, module, or file. Warm versus
fresh complete equality, page-frame identity, paragraph hit and build counts,
the exact 1,024 boundary, and safe 1,025 fallback are otherwise covered.
