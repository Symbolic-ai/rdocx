# F-X048, correctness, pass 3

**Reviewed**: working-tree diff against exact claim base
`fa3dacad97a58de7faf317eedc294f25bf95dfd9`, 15 files and 2,504 changed
lines, with 2,261 additions and 243 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- **Pass 2 D1**: `CT_TblStylePr::region` participates in the preserved-subtree
  equality at `crates/rdocx-oxml/src/styles.rs:823`. A changed region therefore
  cannot reuse stale raw XML. The regression at
  `crates/rdocx-oxml/src/styles.rs:1087` combines a new region with a typed
  conditional shading mutation, requires the old values to disappear,
  preserves the unmodelled producer child, and reparses both typed changes.
  The focused regression passes.
- **Pass 1 D1 through D7**: exact-row overflow remains clipped, vertical merges
  retain their terminal edge and exact grid span, merge-only minimum geometry
  grows the final eligible row, conditional shading and `cnfStyle` participate
  in the cascade, direct paragraph-mark metrics control the empty line,
  character-relative cell anchors retain paragraph indent, and typed
  conditional-style mutations serialize once.
- **Pass 1 S1**: the changed regression target retains the Clippy remediation.
- **Full diff correctness**: no additional wrong logic, off-by-one error,
  unhandled input, or operator-precedence defect was found in the recursive
  table lowering, cache and provenance recursion, table-style projection,
  pagination, facade addition, focused regressions, HLD updates, or declared
  two-entry hash-baseline change.
- **Panics and OOXML**: no new document-controlled panic path, namespace error,
  schema-order error, or loss of an unmodelled conditional-style child was
  found.
- **Structure and scope**: no unjustified trait, generic, wrapper, feature flag,
  crate, module, or source file was introduced. The four changed HLD files are
  exactly the design plan's HLD impact list.
- **External observation**: Microsoft Word 16.104 build 16.104.25121423 is not
  installed on this host. The planned external observation remains honestly
  unperformed, and no headless output is represented as Word evidence.
- **Uncited findings**: none.
