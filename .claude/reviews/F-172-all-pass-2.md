# F-172, all, pass 2

**Reviewed**: uncommitted F-172 working tree against `HEAD`, 11 files and 746 changed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Pass 1 remediation

- D1 is resolved. The signed package object now contains `Manifest` followed by
  `SignatureProperties`, with a targeted OPC `SignatureTime` and a matching root
  signature id (`crates/oxml-opc/src/signature.rs:217`). The regression test
  asserts the exact object child order, property target, format, and UTC value
  (`crates/oxml-opc/src/signature.rs:1612`).
- D2 is resolved. Creation rejects any package that already declares a digital
  signature origin before allocating or staging signature infrastructure
  (`crates/oxml-opc/src/signature.rs:185`). The atomic failure regression signs
  once and then proves a further creation attempt leaves the package unchanged
  (`crates/oxml-opc/src/signature.rs:1678`).
- D3 is resolved. Every part relationship source is checked before its
  relationship part name is derived, and it must be a normalized existing
  package part (`crates/oxml-opc/src/signature.rs:294`). The regression covers
  both an orphan source and the formerly panicking empty source while asserting
  no live mutation (`crates/oxml-opc/src/signature.rs:1745`).
- D4 is resolved. The local round-trip gate serializes the signed package,
  reopens those bytes, and verifies cryptographic validity and complete coverage
  on the reopened package (`crates/oxml-opc/src/signature.rs:1705`). The ignored
  Word gate writes the generated DOCX but cannot pass without an explicit
  `valid` token that its message reserves for a human Word verdict
  (`crates/oxml-opc/src/signature.rs:1765`). The human verdict remains pending
  and is not claimed by the implementation record
  (`.claude/scratch/F-172-progress.md:40`).

## Not found

- Correctness and contract adherence: no additional findings.
- Atomicity and collision handling: no additional findings.
- OOXML schema order, namespace handling, canonicalization, and relationship
  coverage: no additional findings.
- Strict DER input handling, RSA-SHA256 behavior, and certificate-key matching:
  no additional findings.
- Panics, unchecked indexing, slicing, and arithmetic on untrusted input: no
  findings.
- Dependency direction, platform support, public API shape, and packaging: no
  findings.
- Test strength, documentation agreement, and external-oracle gating: no
  additional findings. The HLD explicitly distinguishes the pending human Word
  verdict from document openability (`docs/hld/12-testing-strategy.md:78`).
- Structural smells under `AGENTS.md`: none found.
