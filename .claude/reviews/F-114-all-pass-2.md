# F-114, all, pass 2

**Reviewed**: revised working tree diff, 8 files, 1,396 changed lines, comprising 1,375 insertions and 21 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, Notes normalization strands preserved nonnumeric relationship references
`crates/rpptx/src/lib.rs:844`

The remediation removes every copied `SLIDE` relationship, creates a new
numeric relationship, and records each old id in the rewrite map at
`crates/rpptx/src/lib.rs:856`. The required remapper intentionally skips a
nonnumeric value at `crates/rpptx-oxml/src/relmap.rs:217`. A valid source notes
part can therefore contain preserved XML such as `r:id="slide-link"` referring
to its nonnumeric back relationship. Duplication leaves that attribute
unchanged but removes `slide-link` from the destination scope, so the operation
returns success with a dangling relationship and the next debug save can
panic at its validation boundary. The normalization helper at
`crates/rpptx/tests/integration.rs:3725` inspects only the relationship scope
and never places the nonnumeric source id in notes XML, so all three remediation
tests miss this preservation failure.

## Smells

None.

## Nitpicks

None.

## Prior findings

- Pass-1 D1 is resolved at `crates/rpptx/src/lib.rs:1206`. Media reachability
  now checks both package-root and part relationships, and the regression at
  `crates/rpptx/tests/integration.rs:200` retains a root-referenced media part.
- Pass-1 D2 is resolved for missing, multiple, and external notes back
  relationships at `crates/rpptx/src/lib.rs:844`. The residual nonnumeric raw
  reference case is D1 above.
- Pass-1 D3 is resolved at `crates/rpptx/tests/integration.rs:399` and
  `crates/rpptx-oxml/tests/integration.rs:40`. Both compatibility branches now
  carry numeric shape ids and connector endpoints with explicit rewritten-id
  assertions.

## Not found

- Correctness: no additional findings. Slide and producer order remain
  synchronized, internal relative targets are recomputed, external target mode
  is retained for unrelated relationships, and media reachability covers both
  relationship roots.
- Contract: no additional findings beyond D1. Index semantics, insertion
  position, custom-show behavior, graph staging, and the exact HLD impact match
  the approved plan.
- Panics: no additional findings. Public indices are checked before mutation,
  fallible remove and duplicate work remains staged on a clone, and new
  arithmetic and byte ranges are checked.
- OOXML: no additional findings. Slide-list boundaries remain reconciled by
  producer relationship id, custom-show entries are removed through
  namespace-aware byte splicing, and schema order and raw child boundaries are
  retained.
- Tests: the exact image-scope gate passes and is absent from the base. The
  complete `rpptx` and `rpptx-oxml` integration suites pass. No additional
  findings beyond the nonnumeric preservation case in D1.
- Structure: zero findings. No new file, module, trait, generic, feature,
  dependency, forwarding wrapper, or unjustified dynamic dispatch was added.
