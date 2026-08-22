# F-170, all, pass 3

**Reviewed**: working-tree implementation diff, 10 files and 925 changed lines,
comprising 909 insertions and 16 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Pass 2 remediation

S1 is resolved. `crates/oxml-opc/src/encryption.rs:446` is now the single
randomized envelope generator and contains the validation, secret generation,
encryption, authentication, and CFB assembly logic directly. Its optional
secret capture parameter and capture type exist only under `cfg(test)`. The
production build has no capture parameter, capture-named implementation, or
forwarding-only wrapper.

## Prior remediation recheck

- D1 remains resolved at `crates/oxml-opc/src/encryption.rs:395`. The complete
  envelope is staged before the caller buffer is reserved, and bytes are
  appended only after the reserve succeeds without changing the buffer length.
- D2 remains resolved at `crates/oxml-opc/src/encryption.rs:1134`. Tests decode
  and assert every DataSpaces field independently, including complete stream
  consumption.
- D3 remains resolved at `crates/oxml-opc/src/encryption.rs:1209`. The test-only
  capture compares the plaintext package key, verifier, and HMAC key from two
  independently seeded saves.
- D4 remains resolved at `crates/rdocx/src/document.rs:4216`. Unix uses atomic
  rename, while Windows uses replace-existing and write-through flags through
  `MoveFileExW`.

## Not found

No correctness, contract, panic-safety, OOXML, test-gate, dependency,
feature-graph, public-API, or structural findings were found. The fixed
AES-256-CBC and SHA-512 profile, 100,000 password iterations, key derivation,
HMAC over the complete size-prefixed encrypted stream, 4096-byte segmentation,
CFB v3 and DataSpaces envelope, schema-ordered descriptor, failure atomicity,
live-package preservation, HLD updates, and recorded Microsoft Word 16.104
attestation conform to the approved plan.
