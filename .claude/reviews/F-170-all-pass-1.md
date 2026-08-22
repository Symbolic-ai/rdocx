# F-170, all, pass 1

**Reviewed**: working-tree diff, 10 files and 722 changed lines, comprising 706
insertions and 16 deletions
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, the generic writer path can publish a partial encrypted envelope
`crates/oxml-opc/src/encryption.rs:407`

Staging the CFB before this call does not make `Write::write_all` transactional.
A writer can return `Ok(n)` for a nonzero prefix and then return an error, which
leaves that prefix published when this method returns `Err`. The failure test at
`crates/oxml-opc/src/encryption.rs:1188` uses a writer that rejects its first
write, so it cannot detect the partial-write case required by the approved
failure-atomicity contract.

### D2, the DataSpaces byte checks use the production serializers as their oracle
`crates/oxml-opc/src/encryption.rs:1091`

Each asserted value is produced by the same private helper that supplied the
emitted stream. A defect in a length field, version, transform field, string
encoding, or reserved value therefore changes both sides and leaves the test
green. The fixed Word-compatible CFB contract needs independent expected bytes
or independently decoded field assertions for the four DataSpaces streams.

### D3, the randomness test does not prove that the plaintext secrets are fresh
`crates/oxml-opc/src/encryption.rs:1149`

The test compares encrypted package-key and verifier fields, plus package
ciphertext. Those values still differ when the underlying package key or
verifier is constant because the independently random salts change the derived
keys and initialization vectors. It also never compares the decrypted HMAC
keys. The approved contract requires fresh package keys, verifiers, and HMAC
keys on every save, so the test must recover or otherwise inspect those
plaintext secrets.

### D4, encrypted save cannot replace an existing destination on Windows
`crates/rdocx/src/document.rs:4204`

`std::fs::rename` replaces an existing destination on Unix but fails when the
destination exists on Windows. The public native API is cross-platform, and the
test at `crates/rdocx/src/document.rs:4885` explicitly expects a successful
atomic replacement. This path needs a platform-correct replacement operation
that retains the original destination on failure.

## Smells

None.

## Nitpicks

None.

## Not found

No additional cryptographic, key-derivation, HMAC, segmented-encryption,
password-failure, live-package mutation, public-API, feature-graph, dependency,
OOXML descriptor-order, panic-safety, or structural-simplicity findings were
found. The emitted CFB v3 DataSpaces streams match the source-encoded Microsoft
Word 16.104 reference bytes. This review does not claim execution of the ignored
manual Word password gate.
