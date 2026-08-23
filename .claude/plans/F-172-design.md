# F-172, Digital signature creation

**Status**: completed
**Sprint**: S53
**Size**: M
**Depends on**: F-171

## Problem

`OpcPackage` exposes only read-only signature verification at
`crates/oxml-opc/src/package.rs:300`. The existing signature module verifies a
strict RSA-SHA256 profile at `crates/oxml-opc/src/signature.rs:152`, but it
cannot construct the signature origin, signature relationships, manifest,
certificate payload, or authenticated package graph.

The facade forwards verification at `crates/rdocx/src/document.rs:543`. A
caller therefore has no supported way to sign the current typed document, and
ad hoc package mutation could leave a half-signed live package when key,
certificate, XML, relationship, or output staging fails.

## Spec reference

- ECMA-376 Part 2, section 13, "Digital Signatures".
- W3C XML Signature Syntax and Processing 1.1, `SignedInfo`, `Reference`,
  exclusive canonicalization, and RSA-SHA256 signing.
- `docs/hld/03-architecture.md`, "Why these seams".
- `docs/hld/04-opc-and-packaging.md`, "Relationship types", "Part naming",
  and "Package integrity".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and the signature gate.
- `docs/hld/15-build-and-toolchain.md`, "Feature flags", "Packaging", and
  "Dependency policy".

## Approach

Extend `crates/oxml-opc/src/signature.rs` so creation and verification share
the same canonicalization, relationship-transform, digest, and strict
algorithm constants. Add
`OpcPackage::sign(&mut self, private_key_pkcs8_der: &[u8], certificate_der: &[u8]) -> Result<SignatureReport>`
under the existing default-off `digital-signatures` feature.

Build a cloned candidate package. Allocate collision-free origin and signature
part names, create content-type overrides and internal relationships, and
declare every non-signature part plus every non-signature relationship in
canonical order. Serialize schema-ordered signature XML, sign canonical
`SignedInfo` with RSA-SHA256, then run the F-171 verifier over the candidate.
Commit the candidate only when cryptographic validity and complete coverage
both pass. Reject a private key that does not match the supplied certificate.

Add `Document::sign` as a native-only forwarding API. It clones and flushes
typed document state, signs the staged package, and commits the candidate only
after verification. Python, WASM, and CLI surfaces remain unchanged, and the
ordinary dependency graph continues to exclude signature dependencies.

## Rejected alternatives

- Signing selected parts would preserve the partial-coverage attack F-171 was
  built to expose.
- Mutating the live package while constructing the graph would make failure
  atomicity depend on every later operation succeeding.
- Supporting SHA-1 or caller-selected algorithms would enlarge the security
  surface and permit downgrade from the sprint's strict profile.
- A second signature module would duplicate the canonicalization rules used by
  verification.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `signature_creation_uses_schema_order_and_complete_canonical_references` | Signature children, relationship transforms, manifest references, and digests use the strict supported profile and deterministic order. |
| unit | `signature_creation_rejects_mismatched_or_unsupported_key_material` | Invalid PKCS#8, malformed X.509, non-RSA keys, and key-certificate mismatch fail before live mutation. |
| round-trip | `signed_package_verifies_with_complete_coverage` | A package signed here verifies through F-171 with every part and relationship covered. |
| regression | `every_signature_creation_failure_leaves_live_package_unchanged` | Key, certificate, canonicalization, relationship, serialization, signing, and allocation failures preserve package bytes and typed state. |
| round-trip | `word_for_mac_recognizes_and_protects_the_created_signature` | Microsoft Word for Mac 16.104 opens the generated DOCX, recognizes the embedded digital signature, and protects the document from editing. The same serialized and reopened bytes verify locally with RSA-SHA256 and complete declared coverage. |

The test gate is **round-trip**. A document signed here verifies locally after
serialization and reopen. Word for Mac recognizes the embedded signature and
protects the document from editing. This oracle does not report Windows
certificate trust.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Any parser or serialiser: re-read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add prefix-alias, schema-order,
  canonicalization, relationship-transform, and byte-preservation checks.
- Crate dependency graph: re-read `docs/hld/03-architecture.md`. Run
  `cargo tree -p oxml-opc --all-features`, the dependency-direction test, and
  checks proving signature dependencies remain absent from ordinary graphs.
- Public API of published crates: record the additive, feature-gated semver
  impact. Run dry-run packaging for `oxml-opc` and `rdocx`, and assert archive
  sizes.
- WASM and default-off feature isolation: run both WASM checks and verify the
  native signing graph is absent from Python, WASM, CLI, and default builds.
- External oracle comparison: pin Microsoft Word for Mac 16.104 build
  16.104.25121423 and record the generated document plus the observed signed
  document recognition and protection state. Do not claim Windows certificate
  trust.

## Hash harness

Expected to be unchanged. Signing is explicit, default-off, and no generated
sample is signed.

## Implementation checklist

- [x] Reuse the verified strict canonicalization and transform profile.
- [x] Stage the complete signature graph on a cloned package.
- [x] Generate collision-free origin and signature parts in schema order.
- [x] Sign every required part and relationship with RSA-SHA256.
- [x] Verify the staged package before committing it.
- [x] Add the native facade method without binding expansion.
- [x] Cover invalid input and every atomic failure boundary.
- [x] Run focused signature, package, dependency, WASM, packaging, oracle, and
      harness checks.

## Open questions

None. The public signing API accepts PKCS#8 private-key DER and X.509
certificate DER only. PEM parsing is outside this story.
