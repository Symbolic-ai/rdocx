# F-171, correctness, pass 1

**Reviewed**: Working tree, 12 files, 1,797 added lines and 5 removed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, non-signature relationships on signature infrastructure escape coverage

`crates/oxml-opc/src/signature.rs:590`

The coverage walk skips every relationship owned by a signature origin or
signature part. An added external relationship on the signature origin has no
target part to trigger the part coverage check, so the report can remain
cryptographically valid and coverage complete even though a non-signature
relationship is uncovered. Only the two signature relationship types should
be exempt. The existing filter at line 636 already expresses that boundary.

### D2, schema singleton elements accept duplicates

`crates/oxml-opc/src/signature.rs:984`

`direct_child` returns the first matching child without rejecting another
match. A signature containing two `SignedInfo`, `SignatureValue`,
`CanonicalizationMethod`, `SignatureMethod`, `DigestMethod`, or `DigestValue`
elements can therefore verify against the first value despite violating the
XML Signature schema. This ambiguity is unsafe for a fail-closed verifier and
lets malformed signature XML report success.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in the contract, panics, OOXML, tests, or structure
aspects.
