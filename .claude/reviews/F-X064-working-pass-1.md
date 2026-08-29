# F-X064, working, pass 1

**Reviewed**: complete uncommitted working diff against
`86ae615a61f35a9e7090b8585d7e3c872d8df25f`, 1 file with 170 insertions and
19 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: the private parser accepts the existing signed integer form or
  one decimal separator followed by a nonempty all-zero fraction, then uses a
  checked `i32` parse of the integer portion at
  `crates/rdocx-oxml/src/table.rs:178`. Table widths, cell widths, table
  indents, and default cell margins all reach that parser at
  `crates/rdocx-oxml/src/table.rs:200` and
  `crates/rdocx-oxml/src/table.rs:331`.
- Contract: there is no floating-point conversion, hashing substitute, public
  type change, dependency, module, or additional test binary. Missing Word
  width remains the existing zero default at
  `crates/rdocx-oxml/src/table.rs:323`, while malformed present values return
  `OxmlError::InvalidValue` rather than becoming zero at
  `crates/rdocx-oxml/src/table.rs:181` and
  `crates/rdocx-oxml/src/table.rs:192`.
- Namespace handling: the public width parser resolves declarations before
  delegating at `crates/rdocx-oxml/src/table.rs:317`, and the shared attribute
  predicate requires the bound Word prefix at
  `crates/rdocx-oxml/src/properties.rs:1668`. The regression covers an aliased
  Word namespace and foreign same-local attributes for both widths and margins
  at `crates/rdocx-oxml/src/table.rs:1881`.
- Unsupported union arms and errors: fractional, exponent, empty-fraction,
  overflow, percentage, universal-unit, malformed, and empty lexical forms are
  exercised through the affected width and margin paths at
  `crates/rdocx-oxml/src/table.rs:1918`. Production parsing adds no `unwrap`,
  indexing, unchecked arithmetic, or panic path.
- OOXML and serialization: canonical integer values and fixed `w` attributes
  continue through the existing serializer at
  `crates/rdocx-oxml/src/table.rs:340`. The round-trip regression checks every
  affected modeled site, table-property schema order, and byte-identical
  unmodelled XML at `crates/rdocx-oxml/src/table.rs:1959`.
- Public compatibility: `CT_TblWidth` retains its existing public fields and
  constructors at `crates/rdocx-oxml/src/table.rs:286`. The intended behavioral
  changes are limited to namespace checking, the approved decimal tolerance,
  and explicit errors for unsupported present values.
- Tests and riders: the red boundary, four focused regressions, 282-test crate
  suite, pinned five-document Word corpus, exact LibreOffice and Poppler gate,
  `git diff --check`, and unchanged 49 of 49 hash result are recorded at
  `.claude/scratch/F-X064-progress.md:21` and
  `.claude/scratch/F-X064-progress.md:29`.
- HLD scope and contribution evidence: the current backlog contract matches the
  implementation and negative gate at `docs/hld/14-development-backlog.md:3355`.
  The approved completion work remains limited to the three HLD files listed at
  `.claude/plans/F-X064-design.md:65`, and the plan records open PR 55 at exact
  source SHA `056d48fdf23f35e3538ef3d6ff78cf9e3863e3a5` at
  `.claude/plans/F-X064-design.md:40`.
- Structure: the implementation adds one private helper beside the two existing
  consumers at `crates/rdocx-oxml/src/table.rs:178`. It adds no forwarding
  wrapper, trait, generic parameter, dynamic dispatch, feature flag, or source
  indirection.
