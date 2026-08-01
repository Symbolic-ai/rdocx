# F-068, correctness, pass 2

**Reviewed**: remediated working tree against the claimed base, 6 files, 1,126
insertions and 19 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, nested namespace conflicts can still rebind canonical model elements

`crates/rpptx-oxml/src/presentation.rs:335`

Conflict rejection runs only on the presentation root. A typed alternate-prefix
child can locally bind `p`, `a`, or `r` to a producer namespace. That declaration
is retained as a raw attribute, then placed on a canonical `p:` wrapper or an
element containing canonical `r:id` or `a:` descendants. The local declaration
therefore changes the namespace of newly written model XML. Apply the same
conflict check at every modelled root, list, and identifier boundary before
accepting it as typed content.

## Smells

None.

## Nitpicks

None.

## Not found

No other correctness, contract, panic-path, OOXML ordering or preservation,
test-strength, or structural findings. All pass-1 findings are otherwise
resolved.
