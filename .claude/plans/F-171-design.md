# F-171, Digital signature verification

**Status**: approved
**Sprint**: S52
**Size**: L
**Depends on**: none

## Problem

`OpcPackage` retains package and part relationships in memory at
`crates/oxml-opc/src/package.rs:52`, but `oxml-opc` defines no digital-signature
origin relationship, signature parser, transform implementation, or coverage
report. A caller therefore cannot distinguish a valid complete package
signature from an altered package or a signature that intentionally covers
only a harmless subset.

Verification must authenticate `SignedInfo`, every declared manifest
reference, and the OPC relationship transform. Missing, duplicate, external,
or uncovered relationships must be named and must never be summarized as a
weaker success.

## Spec reference

- ECMA-376 Part 2, section 13, "Digital Signatures".
- W3C XML Signature Syntax and Processing 1.1, `SignedInfo`, `Reference`,
  canonicalization, and RSA-SHA256 verification.
- `docs/hld/03-architecture.md`, "Why these seams".
- `docs/hld/04-opc-and-packaging.md`, "Relationship types", "Part naming",
  and "Package integrity".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy".
- `docs/hld/15-build-and-toolchain.md`, "Packaging" and "Dependency policy".

## Approach

Add a feature-gated `oxml-opc` signature module that discovers the signature
origin only through package relationships, resolves each internal signature
part, parses XML by expanded name, applies the OPC relationship transform and
exclusive canonicalization, verifies each reference digest, and verifies
RSA-SHA256 `SignedInfo` against the embedded X.509 public key. Unsupported or
weak algorithms fail closed with a named error instead of downgrading.

Expose `OpcPackage::verify_signatures() -> Result<Vec<SignatureReport>>` under
a default-off `digital-signatures` feature. Each report includes signature
part, signer certificate identity, cryptographic status, declared covered
parts and relationships, and any missing, changed, duplicate, external, or
uncovered item. Add native `Document::verify_signatures` as a direct borrow of
the package result. Verification is read-only and does not claim certificate
chain trust, which belongs to caller policy.

Use minimal RSA, SHA-256, base64, and X.509 parsing dependencies. Keep XML
canonicalization and the OPC relationship transform in the new signature
module because they are protocol logic, not general XML utilities.

## Rejected alternatives

- Reporting only the `SignatureValue` result would bless partial-coverage
  attacks.
- Treating `_xmlsignatures` names as conventional paths bypasses the OPC
  relationship graph and accepts orphaned or redirected parts.
- Validating the certificate trust chain would require an ambient trust store
  and policy that this story does not define.
- Supporting SHA-1 would turn legacy compatibility into a security downgrade.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `signature_parser_is_prefix_tolerant_and_algorithm_strict` | Alias prefixes parse, canonicalization is stable, and unsupported digest, signature, transform, or key algorithms fail closed. |
| unit | `relationship_transform_selects_exact_ids_in_canonical_order` | Only declared internal relationships are transformed, duplicates and external targets fail, and output order is deterministic. |
| regression | `valid_signature_reports_complete_declared_coverage` | An independently assembled signed DOCX verifies `SignedInfo`, every reference, every declared part, and every required relationship. |
| regression | `modified_signed_part_is_named` | A one-byte mutation fails with the changed part URI and expected versus actual digest context. |
| regression | `partial_or_malformed_coverage_never_reports_success` | Missing, duplicate, external, absent-target, orphan-signature, and uncovered relationship cases are individually named. |
| round-trip | `verification_does_not_change_package_bytes` | Verification is read-only and deterministic saving preserves all signature and unrelated parts. |

The test gate is **regression**. A validly signed document verifies, and a
document modified after signing fails with the changed part named.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Any parser or serialiser: re-read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add prefix-alias, schema-order,
  canonicalization, relationship-transform, and preserved-signature tests.
- Crate dependency graph and new dependencies: re-read
  `docs/hld/03-architecture.md`. Run `cargo tree -p oxml-opc --all-features`
  and the dependency-direction test.
- Public API of published crates: state additive feature-gated semver impact,
  run `cargo publish --dry-run -p oxml-opc` and
  `cargo publish --dry-run -p rdocx`, and check archive sizes.
- WASM bindings and a new default-off feature: run both locked WASM checks,
  verify signature dependencies are absent from the WASM graph, and test the
  default-off native graph.
- New module and file: explicit approval is required for
  `crates/oxml-opc/src/signature.rs`. Keep parsing, canonicalization,
  relationship transforms, verification, and reports together there.
- External oracle comparison: use an independently constructed in-source
  signature fixture and the locally installed Microsoft Word as an optional
  openability check. Record exact producer metadata and never add an oracle as
  a production dependency.

## Hash harness

Expected to be unchanged. Verification is read-only and default-off.

## Implementation checklist

- [ ] Add the approved default-off feature and minimal dependency graph.
- [ ] Discover signature origins and parts through normalized relationships.
- [ ] Parse and canonicalize the supported XML Signature profile.
- [ ] Verify reference digests, relationship transforms, and RSA-SHA256.
- [ ] Compute complete coverage and return typed failure reports.
- [ ] Add native `Document` forwarding without binding expansion.
- [ ] Add independent in-source valid, tampered, partial, and malformed cases.
- [ ] Run focused signature, package, no-default, WASM, supply-chain, package,
      and harness checks.

## Open questions

None. The new module, default-off dependency graph, fail-closed RSA-SHA256
profile, and caller-owned certificate trust policy are approved.
