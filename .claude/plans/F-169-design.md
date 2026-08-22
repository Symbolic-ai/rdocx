# F-169, Agile encryption, read

**Status**: completed
**Sprint**: S52
**Size**: L
**Depends on**: none

## Problem

`OpcPackage::from_reader_with_limits` assumes the input is a ZIP archive and
constructs `ZipArchive` directly at `crates/oxml-opc/src/package.rs:78`.
`Document::from_bytes_with_limits` passes every input to that path at
`crates/rdocx/src/document.rs:503`, so an OOXML package wrapped in the Compound
File Binary Format with `EncryptionInfo` and `EncryptedPackage` streams cannot
be opened at all.

The read boundary must authenticate the password and encrypted package before
publishing an `OpcPackage`. Wrong passwords, malformed parameters, truncated
streams, and integrity failures must leave callers with no partially parsed or
mutated package.

## Spec reference

- ECMA-376 Part 2, section 10, "Encryption" and Part 4, section 4.4,
  "Agile Encryption".
- `docs/hld/03-architecture.md`, "Why these seams".
- `docs/hld/04-opc-and-packaging.md`, "The package" and "Package integrity".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy".
- `docs/hld/15-build-and-toolchain.md`, "Feature flags", "Packaging", and
  "Dependency policy".

## Approach

Add a feature-gated `oxml-opc` agile-encryption module that owns the CFB outer
container, namespace-aware `EncryptionInfo` parsing, password normalization,
spin-count key derivation, verifier checks, block-key derivation, AES-CBC
decryption, and encrypted-package HMAC validation. The parser accepts the
agile AES key sizes and SHA-family hashes declared by ECMA-376, rejects unknown
algorithms and inconsistent sizes, bounds allocations before reading streams,
and returns decrypted ZIP bytes only after authentication succeeds.

Expose `OpcPackage::from_encrypted_reader_with_limits(reader, password,
limits)` plus the unbounded convenience form. Add native-only
`Document::open_encrypted`, `Document::from_encrypted_bytes`, and
`Document::from_encrypted_bytes_with_limits` forwarding methods. The ordinary
ZIP constructors remain unchanged. Put the dependencies behind an
`agile-encryption` feature that is off by default and is not enabled by Python,
WASM, or CLI crates.

Use `cfb` for the compound container, RustCrypto `aes` and `cbc` for AES-CBC,
and the minimal SHA, HMAC, and base64 support needed by the agile XML. Remove
the `aes` crate ban from `deny.toml`, whose current rationale at
`deny.toml:32` is avoiding unused ZIP codecs, and replace it with a manifest
comment explaining the named encryption consumer and default-off edge.

## Rejected alternatives

- Parsing CFB and implementing AES locally duplicates security-critical code
  and creates a second source of bugs.
- Auto-detecting encrypted input in `Document::from_bytes` cannot supply a
  password and would make an existing API's failure behavior ambiguous.
- Enabling encryption by default would pull native cryptography and random
  support into consumers that do not request it, including the WASM graph.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `agile_parameters_reject_unknown_or_inconsistent_algorithms` | Namespace aliases parse, supported AES and SHA combinations validate, and malformed sizes, salts, spins, and algorithms fail before allocation or decryption. |
| unit | `wrong_password_never_releases_a_package_key` | The encrypted verifier check fails closed and does not produce plaintext. |
| regression | `word_agile_document_opens_only_with_its_password` | A Microsoft Word reference encoded in source opens with the correct password and names a stable wrong-password error otherwise. |
| regression | `tampered_agile_package_fails_before_zip_parsing` | Modified ciphertext and HMAC bytes fail authentication without constructing an `OpcPackage`. |
| round-trip | `decrypted_package_preserves_every_unrelated_part` | The authenticated ZIP parses with limits and a deterministic save retains unrelated parts, relationships, content types, and preserved XML. |

The test gate is **regression**. A password-protected document produced by
Word opens with the right password and fails cleanly with the wrong one.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Any parser or serialiser: re-read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add prefix-alias, schema-order, and
  preservation tests for the encryption XML boundary.
- Crate dependency graph and new cross-crate use: re-read
  `docs/hld/03-architecture.md`. Run `cargo tree -p oxml-opc --all-features`
  and the repository dependency-direction test.
- Public API of published crates: re-read `docs/hld/10-bindings-spec.md` and
  the structural rules. Record the additive, feature-gated semver impact, run
  `cargo publish --dry-run -p oxml-opc` and `cargo publish --dry-run -p rdocx`,
  and check both archive sizes.
- WASM bindings and a new default-off feature: run both locked WASM checks,
  verify the feature does not enter either WASM graph, and test `oxml-opc` and
  `rdocx` with default features disabled.
- New module and file: explicit approval is required for
  `crates/oxml-opc/src/encryption.rs`. Keep the complete encryption path in
  that one module instead of adding forwarding wrappers.
- External oracle comparison: use the locally installed Microsoft Word as the
  openability oracle, record its exact version, store no binary fixture file,
  and encode the reviewed reference bytes in source.

## Hash harness

Expected to be unchanged. The default graph and every ordinary DOCX path stay
unchanged.

## Implementation checklist

- [x] Add the approved default-off dependency and feature graph.
- [x] Parse and validate the CFB streams and agile encryption descriptor.
- [x] Derive and verify the password key without exposing unauthenticated data.
- [x] Decrypt and authenticate the package into a bounded temporary buffer.
- [x] Add the `OpcPackage` and native `Document` constructors.
- [x] Add in-source Word reference, malformed-input, and preservation tests.
- [x] Run focused crypto, package, no-default, WASM, supply-chain, package, and
      harness checks.

## Open questions

None. The new module, dependency policy change, complete agile AES and SHA read
profile, and pinned local Microsoft Word oracle are approved.
