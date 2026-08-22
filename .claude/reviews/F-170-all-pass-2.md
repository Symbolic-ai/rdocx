# F-170, all, pass 2

**Reviewed**: working-tree implementation diff, 10 files and 932 changed lines,
comprising 916 insertions and 16 deletions
**Verdict**: 0 defects, 1 smell, 0 nitpicks

## Defects

None.

## Smells

### S1, the test capture hook leaves a forwarding wrapper in production
`crates/oxml-opc/src/encryption.rs:434`

`encrypted_package_bytes_with_random` contains no generation logic and only
forwards to `encrypted_package_bytes_with_random_and_capture`. In a non-test
build the callee has no capture parameter, so production retains two functions
and a capture-named implementation where one function would suffice. This
violates the repository rule against wrappers that only forward and makes a
reader cross an extra boundary to find the code that runs. Keep the secret
capture test-only without retaining the forwarding layer in production.

## Nitpicks

None.

## Pass 1 remediation

- D1 is resolved. `crates/oxml-opc/src/encryption.rs:395` now accepts a caller
  byte buffer, stages the complete envelope, reserves the complete additional
  capacity, and appends only after a successful reserve. The injected partial
  append is truncated on failure, while the production reserve operation does
  not mutate existing elements.
- D2 is resolved. `crates/oxml-opc/src/encryption.rs:1144` decodes every
  DataSpaces field independently and asserts full stream consumption instead
  of deriving expected bytes from the production serializers.
- D3 is resolved. `crates/oxml-opc/src/encryption.rs:449` keeps secret capture
  types and parameters behind `cfg(test)`, and the randomness test directly
  compares the generated package key, verifier, and HMAC key.
- D4 is resolved. `crates/rdocx/src/document.rs:4216` uses platform-specific
  replace-existing behavior, including `MoveFileExW` with replacement and
  write-through flags on Windows.

## Not found

No correctness, contract, panic-safety, OOXML, test-gate, dependency,
feature-graph, public-API, or additional structural findings were found. The
AES-256-CBC and SHA-512 profile, 100,000 password iterations, password key
derivation, HMAC over the size-prefixed encrypted package, 4096-byte segment
encryption, CFB v3 envelope, schema-ordered descriptor, independent DataSpaces
fields, failure staging, and live-package preservation conform to the approved
plan and HLD. The progress record at `.claude/scratch/F-170-progress.md:8`
records Microsoft Word 16.104 correct-password success and wrong-password
rejection, and the attested ignored test passed.
