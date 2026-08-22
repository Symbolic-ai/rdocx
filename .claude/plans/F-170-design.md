# F-170, Agile encryption, write

**Status**: completed
**Sprint**: S52
**Size**: M
**Depends on**: F-169

## Problem

`OpcPackage::write_to` emits only a deterministic ZIP at
`crates/oxml-opc/src/package.rs:210`, and `Document::to_bytes` exposes only
that plain package at `crates/rdocx/src/document.rs:726`. Callers cannot save a
password-protected document that Word can open.

Writing must reuse the validated F-169 parameter and key-derivation model,
produce a complete CFB envelope transactionally, and avoid changing the live
package when parameter validation, randomness, serialization, encryption, or
authentication fails.

## Spec reference

- ECMA-376 Part 2, section 10, "Encryption" and Part 4, section 4.4,
  "Agile Encryption".
- `docs/hld/04-opc-and-packaging.md`, "The package" and "Package integrity".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy".
- `docs/hld/15-build-and-toolchain.md`, "Feature flags", "Packaging", and
  "Dependency policy".

## Approach

Extend the F-169 agile-encryption module with a fixed modern Word-compatible
write profile: AES-256-CBC, SHA-512, 100,000 spin iterations, fresh salts and
package key from the operating system CSPRNG, and the ECMA-376 data-integrity
HMAC. Serialize the ordinary OPC ZIP into a staging buffer, encrypt it in
4,096-byte segments, emit schema-ordered `EncryptionInfo`, and create the CFB
`EncryptionInfo`, `EncryptedPackage`, and `DataSpaces` streams only after every
input and output size has been checked.

Expose `OpcPackage::write_encrypted_to(output, password)` with a caller-owned
byte buffer, plus native `Document::save_encrypted` and
`Document::to_encrypted_bytes`. Reserving the complete encrypted envelope
before appending keeps the caller's existing buffer unchanged on allocation
failure. These methods reuse the F-169 `agile-encryption` feature. Ordinary
deterministic ZIP saving is unchanged. Encryption itself is intentionally
nondeterministic because fresh salts and keys are a security requirement.

## Rejected alternatives

- Accepting caller-selected algorithms would expose insecure and invalid
  combinations that no current consumer needs.
- Reusing deterministic salts would make fixture bytes convenient but would
  weaken real encrypted output.
- Mutating the package into an encrypted form would confuse the in-memory OPC
  model, which remains a decrypted package.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `agile_writer_emits_word_profile_parameters` | The descriptor uses AES-256-CBC, SHA-512, 100,000 spins, valid sizes, fixed schema order, and fresh randomness. |
| round-trip | `encrypted_document_decrypts_without_package_loss` | A document encrypted here decrypts through F-169 and retains every part, relationship, content type, and unmodelled XML byte. |
| round-trip | `two_encryptions_of_one_package_use_distinct_secrets` | Repeated encryption produces different salts, package keys, verifiers, and ciphertext while decrypting to the same ZIP. |
| regression | `failed_encryption_leaves_document_and_package_unchanged` | Empty passwords, oversized inputs, RNG failure injection, and output-reserve failure publish no partial result or live mutation. |
| differential | `word_opens_the_written_agile_document` | The reviewed output opens in the pinned local Microsoft Word build with the correct password and fails with the wrong one. |

The test gate is **round-trip**. A document encrypted here decrypts here, and
the parameters match a Word-encrypted reference byte for byte where the
specification fixes them.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Any parser or serialiser: re-read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add fixed-prefix, schema-order, and
  read-after-write preservation tests for the generated descriptor and CFB.
- Public API of published crates: state the additive feature-gated semver
  impact, run `cargo publish --dry-run -p oxml-opc` and
  `cargo publish --dry-run -p rdocx`, and check both archive sizes.
- WASM bindings and the existing default-off feature: run both locked WASM
  checks and prove encryption dependencies remain absent from the WASM graph.
- External oracle comparison: record the exact Microsoft Word version and the
  manual correct-password and wrong-password outcomes. The oracle is test
  evidence only and never a crate dependency.

## Hash harness

Expected to be unchanged. Existing plain package output remains byte-identical.

## Implementation checklist

- [x] Reuse F-169 parameter validation and key derivation.
- [x] Add the fixed secure write profile and injectable test randomness.
- [x] Encrypt staged deterministic ZIP bytes and emit a complete CFB envelope.
- [x] Add `OpcPackage` and native `Document` write methods.
- [x] Add round-trip, randomness, failure-atomicity, and Word-open tests.
- [x] Run focused encryption, package, WASM, supply-chain, packaging, and
      harness checks.

## Open questions

None. The fixed AES-256-CBC, SHA-512, 100,000-spin profile with fresh operating
system randomness and the pinned local Microsoft Word gate are approved.
