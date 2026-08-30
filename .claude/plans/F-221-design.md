# F-221, Presentation encryption and signatures

**Status**: completed
**Sprint**: S59
**Size**: M
**Depends on**: F-169, F-170, F-171, F-172

## Problem

`rpptx` already owns an `OpcPackage`, but its manifest has no forwarding
feature for either shared package-security implementation. `Presentation`
exposes only plain ZIP constructors and serialization, so native callers
cannot open or write password-protected decks or inspect, verify, and create
package signatures.

Typed presentation state and retained package bytes are separate. Mutable
slide handles change typed state, while `staged_package` writes those changes
only at the serialization boundary. Verifying only the retained package after
a typed edit could therefore report the old signature as valid for content the
caller has already changed.

## Spec reference

- ECMA-376 Part 2, section 10, "Encryption", and section 13, "Digital
  Signatures".
- `docs/hld/03-architecture.md`, "Why these seams".
- `docs/hld/04-opc-and-packaging.md`, "Package integrity", Agile encryption,
  and digital signatures.
- `docs/hld/06-presentationml-model.md`, "Public facade" and "Preservation
  strategy".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability" and the
  existing package-security API precedent.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", Agile encryption gates,
  and signature fixtures.
- `docs/hld/14-development-backlog.md`, "F-221, Presentation encryption and
  signatures".
- `docs/hld/15-build-and-toolchain.md`, "Feature flags", "Packaging", and
  "Dependency policy".

## Approach

Add default-off `agile-encryption` and `digital-signatures` features to
`rpptx`, forwarding to the existing `oxml-opc` features. The named consumer is
the native `rpptx::Presentation` API. The default, Python, WASM, and CLI graphs
remain unchanged.

Re-export `PackageReadLimits` and the feature-gated shared signature report
types. Add native-only `open_encrypted`, `from_encrypted_bytes`,
`from_encrypted_bytes_with_limits`, `save_encrypted`, and
`to_encrypted_bytes`, matching the established Word facade contract. Encrypted
path publication uses a sibling temporary file and atomic rename. All
encryption and signature protocol logic remains in `oxml-opc`.

Add `verify_signatures(&self)` and native-only `sign(&mut self,
private_key_pkcs8_der, certificate_der)`. Both operate on the current staged
presentation package, not stale retained bytes. Signing stages typed content,
signs a cloned candidate through `OpcPackage::sign`, and commits the candidate
only after the shared verifier reports cryptographic validity and complete
coverage. After a relevant typed mutation, verification stages current bytes
and reports retained signatures as invalid rather than falsely preserving
validity.

No new file, module, trait, generic, crate, or production dependency is added.
The additive feature-gated API is pre-1.0 semver-compatible.

## Rejected alternatives

- Duplicating Agile encryption or XML Signature logic in `rpptx` would create
  a second security implementation.
- Verifying only `self.package` would permit stale validity after typed edits.
- Enabling either security feature by default would leak native cryptography
  into bindings and ordinary consumers.
- Removing signature infrastructure automatically would discard inspection
  evidence that can instead be reported accurately as invalid.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `encrypted_presentation_round_trips_with_password_and_preserves_package` | The right password reopens the deck, a wrong password fails, and unrelated parts, relationships, content types, and raw PresentationML subtrees survive. |
| integration | `failed_encrypted_save_leaves_destination_and_presentation_unchanged` | Password and publication failures are atomic. |
| integration | `signed_presentation_reopens_with_complete_fixture_coverage` | Source-encoded PKCS#8 and X.509 fixtures sign, serialize, reopen, and report cryptographic validity with complete coverage. |
| regression | `mutating_a_signed_presentation_never_reports_the_stale_signature_as_valid` | Nested slide, shape, text, core-property, and package-graph mutations verify staged current state and never retain a false valid result. |
| integration | `presentation_security_features_are_default_off_and_binding_manifests_do_not_enable_them` | Default, Python, WASM, and CLI graphs exclude both security features. |
| differential | `powerpoint_opens_the_written_agile_presentation` | Pinned PowerPoint 16.104 build 16.104.25121423 accepts the right password and rejects the wrong one. |

The test gate is **integration**. Pinned PowerPoint opens encrypted output,
signature verification matches trusted certificate fixtures, and mutation
never leaves a signature falsely reported as valid. The PowerPoint check is
manual evidence and no binary fixture enters the repository.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Any parser or serialiser: re-read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add staged schema-order,
  prefix-policy, and byte-preserved unmodelled subtree round-trip checks.
- Crate dependency graph: re-read `docs/hld/03-architecture.md`. Inspect both
  security feature selections and prove ordinary, Python, WASM, and CLI graphs
  exclude their cryptographic dependencies.
- Public API of a published crate: state the additive feature-gated semver
  impact. Run the `rpptx` publish dry run and assert the archive stays below
  10 MiB.
- WASM or PyO3 bindings: run the locked WASM target check and workspace tests
  with the required Python binding exclusions. Inspect manifests for feature
  leakage.
- New feature flags: the named consumer is native `rpptx::Presentation`. Keep
  both flags default-off, run the no-default checks, and run
  `cargo test -p oxml-layout --no-default-features`.
- External oracle comparison: pin and record PowerPoint 16.104 build
  16.104.25121423. Record correct-password and wrong-password observations as
  manual evidence, with no production dependency or binary fixture.

## Hash harness

Expected unchanged. Both features are default-off, and ordinary PPTX
serialization and rendering must remain byte-identical.

## Implementation checklist

- [x] Add the two default-off feature forwards and public type re-exports.
- [x] Add encrypted constructors and atomic encrypted output methods.
- [x] Add current-state verification and atomic signing.
- [x] Preserve signature parts for inspection while reporting them invalid after mutation.
- [x] Add source-built encryption, trusted-certificate, mutation, feature-isolation, and PowerPoint tests to the existing integration binary.
- [x] Run scoped all-feature and no-default `rpptx` checks plus every routed rider.

## Open questions

None. Signature parts remain inspectable after mutation, and verification runs
against the staged current package so changed content reports the retained
signature as invalid.
