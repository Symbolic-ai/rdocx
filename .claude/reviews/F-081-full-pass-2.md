# F-081, full, pass 2

**Reviewed**: working-tree diff, 10 files, 345 insertions and 6 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: `crates/rpptx-layout/src/context.rs:43` returns no chain for a
  non-placeholder or a missing layout counterpart. The second hop at
  `crates/rpptx-layout/src/context.rs:56` derives its key from the matched
  layout placeholder, and recursive lookup at
  `crates/rpptx-layout/src/context.rs:71` retains shape-tree order.
- Contract: `crates/rpptx-layout/src/context.rs:39` now keeps the helper within
  the crate. The narrow dead-code allowance records the approved staged F-082
  consumer without exposing speculative public API.
- Panics: production resolution at `crates/rpptx-layout/src/context.rs:39` is
  total and contains no `unwrap`, `expect`, indexing, or unchecked arithmetic.
- OOXML: `crates/rpptx-layout/src/context.rs:71` reads only the existing typed
  model and follows groups plus the selected MC fallback without modifying XML
  or serialisation state.
- Tests: the gate at `crates/rpptx-layout/src/context.rs:151` distinguishes the
  layout and master matches. The second-hop regression at
  `crates/rpptx-layout/src/context.rs:172` fails if the slide key is reused for
  the master lookup, and the recursive test at
  `crates/rpptx-layout/src/context.rs:187` covers a group and selected fallback.
- Structure: the concrete context at `crates/rpptx-layout/src/context.rs:9`
  introduces no trait, generic abstraction, forwarding wrapper, or feature
  flag. The six new files are the exact approved source layout.
