# F-114, all, pass 1

**Reviewed**: working tree diff, 8 files, 1,204 changed lines, comprising 1,183 insertions and 21 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, Media pruning ignores package-root relationships
`crates/rpptx/src/lib.rs:1192`

The reachability check searches only `package.part_rels`. It never searches
`package.package_rels`. If a removed slide uses a media part that is also the
target of an internal package-root relationship, such as a thumbnail
relationship, removal deletes the still-reachable part and leaves that root
relationship dangling. The contract requires deletion only when no remaining
internal package relationship reaches the candidate. The regression at
`crates/rpptx/tests/integration.rs:162` creates only two slide-level users, so
it does not exercise this remaining-reference case or its own claim about
preserving pre-existing reachability.

### D2, Notes duplication does not guarantee one fresh slide back relationship
`crates/rpptx/src/lib.rs:835`

The notes copy passes the source relationship scope directly to the generic
scope duplicator. That helper only changes the target when it encounters an
existing internal `SLIDE` relationship at `crates/rpptx/src/lib.rs:1140`.
A notes part with no back relationship therefore produces a copied notes part
with none, multiple back relationships remain multiple, and an external back
relationship remains external through the earlier branch. These inputs are
accepted by the current notes loader, but duplication succeeds with a graph
that violates the required single relationship back to the duplicate. The
test at `crates/rpptx/tests/integration.rs:418` starts from a fixture that
already has exactly one valid back relationship and only selects the first
match, so it cannot detect the missing or duplicate cases.

### D3, Fresh shape-id coverage omits compatibility content
`crates/rpptx/tests/integration.rs:332`

The named regression checks ordinary and connector ids, but its
`AlternateContent` fixture at `crates/rpptx/tests/integration.rs:3737` contains
no numeric `p:cNvPr/@id` in either branch. The lower-level test at
`crates/rpptx-oxml/tests/integration.rs:40` also has no compatibility content.
An implementation that skips every id and connector endpoint inside
`mc:AlternateContent`, including non-selected choices, would still pass both
tests. The plan and HLD explicitly require fresh ids across compatibility
content, so the regression does not prove that part of its contract.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: no additional findings. Slide and producer order remain
  synchronized, relative internal targets are recomputed, external target mode
  is retained, and nonnumeric relationship ids remain scoped correctly.
- Contract: no additional findings beyond D2 and D3. The three public methods,
  index semantics, insertion position, custom-show exclusion, and listed HLD
  updates match the approved plan.
- Panics: zero findings. Remove and duplicate stage fallible work on a clone,
  move validates both indices before mutation, and new arithmetic and byte
  ranges are checked.
- OOXML: zero findings. Slide-list raw boundaries are reconciled by producer
  relationship id, custom-show removal is namespace-aware and byte-spliced,
  and the writers retain schema order and fixed prefixes.
- Tests: the exact gate passes and is absent from the base, so reverting the
  implementation makes it fail to compile. No additional findings beyond the
  reachability gap in D1 and compatibility coverage gap in D3.
- Structure: zero findings. No new file, module, trait, generic, feature,
  dependency, forwarding wrapper, or unjustified dynamic dispatch was added.
