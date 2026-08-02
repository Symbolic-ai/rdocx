# F-081, full, pass 1

**Reviewed**: working-tree diff, 9 files, 300 insertions and 6 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, Placeholder chain exceeds the approved visibility

`crates/rpptx-layout/src/context.rs:37`

The approved design specifies a crate-visible placeholder-chain helper, and the
resolver HLD keeps it private. The implementation exposes it as a public method
on the re-exported `ResolveCtx`, creating API surface no current external
consumer needs. Restrict the method to the crate while retaining the staged
F-082 consumer path.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: `crates/rpptx-layout/src/context.rs:41` returns no chain for a
  non-placeholder or a missing layout counterpart, then
  `crates/rpptx-layout/src/context.rs:54` derives the master key from the
  matched layout placeholder.
- Panics: production resolution at `crates/rpptx-layout/src/context.rs:37` is
  total and contains no `unwrap`, `expect`, indexing, or unchecked arithmetic.
- OOXML: `crates/rpptx-layout/src/context.rs:69` reads only the existing typed
  model, preserves document order, and follows groups and only the selected MC
  fallback without changing serialisation state.
- Tests: the gate at `crates/rpptx-layout/src/context.rs:149` distinguishes the
  layout and master matches and fails if either hop is removed. The second-hop
  regression at `crates/rpptx-layout/src/context.rs:170` fails if the slide key
  is reused for the master lookup.
- Structure: the concrete context at `crates/rpptx-layout/src/context.rs:9`
  introduces no trait, generic abstraction, forwarding wrapper, or feature
  flag. The six new files are the exact approved source layout.
