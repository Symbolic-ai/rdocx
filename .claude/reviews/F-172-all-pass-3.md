# F-172, all, pass 3

**Reviewed**: complete F-172 diff against claim base `0766667efe82d16507da4e6e38b4a3bf7a6fe7e6`, 13 files and 898 changed lines including two prior review records. The implementation and contract scope is 11 files and 763 changed lines.
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, signature allocation can repair a pre-existing dangling relationship
`crates/oxml-opc/src/signature.rs:195`

The creator allocates and inserts the origin and signature parts before it
validates the package relationship graph. A package with an ordinary dangling
relationship to `_xmlsignatures/origin.sigs` or
`_xmlsignatures/sig1.xml` therefore gains the missing target during signing.
The later manifest validation sees a resolved target and accepts the package.
This returns success instead of failing closed for the pre-existing dangling
graph required by `docs/hld/04-opc-and-packaging.md:380`.

### D2, malformed signature-typed relationships are excluded from validation
`crates/oxml-opc/src/signature.rs:365`

Relationship filtering excludes both signature relationship types solely by
type, regardless of their source or whether they are one of the two edges just
created by the signer. The coverage check repeats the same global exclusion at
`crates/oxml-opc/src/signature.rs:1009`. A digital-signature relationship from
the package root, or a signature-origin relationship from an ordinary part,
can therefore carry an external or missing target without reaching the
relationship transform or producing an uncovered issue. Signing can succeed
with a malformed package graph instead of rejecting it.

### D3, staged verification ignores invalid reports beside the new signature
`crates/oxml-opc/src/signature.rs:245`

The creator selects only the report for the newly allocated signature and
commits when that one report passes. The collision test permits an occupied
`/_xmlsignatures/sig7.xml` at
`crates/oxml-opc/src/signature.rs:1661`, but discovery reports that unlinked XML
part as an invalid orphan at `crates/oxml-opc/src/signature.rs:509`. Signing
therefore returns success even though verifying the committed package also
returns an invalid signature report. This does not satisfy the contract that
the staged candidate passes shared verification before commit.

## Smells

None.

## Nitpicks

None.

## Not found

- The revised external oracle is scoped truthfully. Word for Mac evidence is
  limited to signature recognition and editing protection, and no Windows
  certificate-trust verdict is claimed.
- The oracle gate writes one byte sequence, reopens those exact bytes, and
  separately asserts RSA-SHA256 validity and complete declared coverage.
- No additional findings in PKCS#8 and X.509 DER parsing, RSA key and
  certificate matching, signature XML schema order, exclusive
  canonicalization, deterministic reference order, or signature-time format.
- No additional findings in cloned package and document atomicity, native
  facade gating, Python, WASM, and CLI isolation, dependency direction,
  published API scope, HLD impact scope, or repository structure rules.
- No additional panics, unchecked indexing, slicing, or arithmetic issues were
  found on untrusted inputs.
