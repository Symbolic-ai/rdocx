# F-172, all, pass 4

**Reviewed**: complete F-172 tracked diff against claim base `0766667efe82d16507da4e6e38b4a3bf7a6fe7e6`, 13 files and 975 changed lines including two prior tracked review records. The implementation and contract scope is 11 files and 840 changed lines. Pass 3 was also reviewed as the remediation contract.
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Pass 3 remediation

- D1 is resolved. The complete unsigned relationship graph is validated before
  either signature part name is allocated or inserted
  (`crates/oxml-opc/src/signature.rs:185`). Focused regressions prove that
  dangling relationships to both future origin and future signature names fail
  without live mutation (`crates/oxml-opc/src/signature.rs:1778`).
- D2 is resolved. Every relationship set in the unsigned graph rejects either
  digital-signature relationship type regardless of source
  (`crates/oxml-opc/src/signature.rs:336`). Package-root and ordinary-part
  misplaced relationship regressions cover both types
  (`crates/oxml-opc/src/signature.rs:1790`).
- D3 is resolved. Staged verification now rejects the candidate when any report
  is cryptographically invalid or has incomplete coverage
  (`crates/oxml-opc/src/signature.rs:237`). The orphan-signature regression
  proves failure remains atomic (`crates/oxml-opc/src/signature.rs:1805`).

## Not found

- Correctness and contract adherence: no additional findings.
- Package graph validation, allocation, collision handling, staged commit, and
  live-state atomicity: no additional findings.
- OOXML schema order, namespace handling, canonicalization, relationship
  transforms, deterministic ordering, and complete coverage: no findings.
- Strict DER input handling, RSA-SHA256 behavior, certificate and key matching,
  and signature-time serialization: no findings.
- Panics, unchecked indexing, slicing, and arithmetic on untrusted inputs: no
  findings.
- Test strength and revised external-oracle honesty: no findings. The gate
  reopens the exact emitted bytes for local cryptographic and coverage checks,
  limits the Word for Mac evidence to recognition and editing protection, and
  does not claim Windows certificate trust
  (`crates/oxml-opc/src/signature.rs:1840`).
- Dependency direction, default-off feature isolation, native facade scope,
  Python, WASM, and CLI isolation, public API packaging, HLD impact scope, and
  repository structure rules: no findings.
