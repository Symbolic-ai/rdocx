# F-282, ooxml, pass 2

**Reviewed**: Separate independent review of the frozen bounded remediation in `work/f-282-codex` at HEAD31524cc698be5eeaf0850aef0d7d913b1ec5a17f. Production remediation changes bibliography.rs by159 added and32 removed lines relative to the pass1 snapshot. Three regression functions add83 lines to the existing entrypoint. The unchanged qualified scanner, source graph and structured cache paths retain their pass1 hashes. The approved bounded worker design SHA256 isde75b471deae4f587e4f015d2bafed68409f797c7f717debf3508302a18af342. F-X192 owns the remaining catalogue and F-283 is carried to S91.
**Verdict**: 0 defects, 0 smells, 0 nitpicks.

## Defects

None found in this aspect after remediation.

## Smells

None found in this aspect.

## Nitpicks

None recorded.

## Previous finding dispositions

### D1, unknown Word namespace attribute loss, resolved in source

`crates/rdocx/src/bibliography.rs:1059`
`crates/rdocx/src/bibliography.rs:1175`
`crates/rdocx/src/bibliography.rs:1210`
`crates/rdocx/tests/regression_test.rs:67833`

Reset validation now checks qualified attribute local names against the actual particle-specific inventory. An unknown local name in the Word namespace is refused just as an unowned foreign attribute is. Shared scalar val ownership applies to the recognized scalar and toggle particles, with separate inventories for structured attribute sets such as indentation, paragraph spacing, fonts, language and shading. Unknown particle children remain raw instead of becoming owned merely through their namespace.

Validation descends into the original run-property container before determining whether a generated counterpart exists. Thus both replaced and removed recognized run children receive the same guard. Removal of an unmatched entire run-property container additionally refuses its own attributes, opaque content, unowned children and history particles. When a counterpart exists, its original wrapper and unowned children remain the raw serialization authority.

The new regression exercises unknown attributes on replaced indentation, removed keepNext, replaced recursive color and removed recursive bold. Each case requires refusal, complete package identity, preserved raw particle and unchanged ContentLocation identity. Existing supported native controls and foreign attribute controls remain present. These controls are statically sound, but their compiled execution is pending.

### D2, spaced closing QName rename, resolved in source

`crates/rdocx/src/bibliography.rs:1629`
`crates/rdocx/tests/regression_test.rs:67856`

The closing name range now starts at the parsed content-end boundary plus the literal closing delimiter. It no longer derives the name from the end of the element, so whitespace before the final bracket is outside the edited span. Scalar property and contributor role renames share the repaired path.

The new regression changes both identities under b and producer namespace aliases, using space, tab, newline and CRLF-plus-space closing spellings. It asserts the projected source, exact preserved QName suffix spelling, reopen and no-op byte identity. No scalar text or namespace binding is rewritten by the closing-name repair. Compiled before and after execution is pending.

## Mixed transaction proof adjunct

`crates/rdocx/tests/regression_test.rs:67881`
`crates/rdocx/src/bibliography.rs:16187`

The new mixed-owner control first demonstrates that the isolated1033 owner is eligible. It then combines that owner with a recognized unfinished1054 owner and requires error, unchanged complete package bytes, source inventory, both caches and both ContentLocations. A separate unrecognized syntax case requires successful eligible refresh plus retained cache diagnostics. This distinguishes incomplete-standard atomic refusal from successful retention reports without changing production policy. The test is proof for the existing transaction boundary, so it need not fail against original production. Its runtime result is pending and belongs to the final gate.

## Not found

- Namespace and preservation: no remaining finding in expanded-name identity, source graph validation, local attribute ownership, raw foreign particles, source no-op edits, instruction character maps or legal closing-tag whitespace.
- Schema order: no new finding in paragraph and run property ordering, recursive history preservation or the generated structured cache sequence.
- Ownership and atomicity: no new finding in physical owner offsets, retained outside prefix and suffix, lock and protected simple-owner handling, checked overlapping edits, candidate staging or rollback boundaries.
- Regression intent: no new finding in the three added controls. Existing APA1340 assertions remain present with no new skip or ignore. This aspect does not certify their execution.
- Overrestriction: no new unjustified admission change was found in the remediation. Ambiguous or unowned metadata is refused rather than silently deleted. Existing structured particle child refusal remains unchanged.
- Proprietary implementation: no proprietary XSL import or execution was introduced.

## Authentication and limits

Freeze binder `/private/tmp/S90-F282-remediation-before-20261009/freeze-binder.json` has SHA2562c0c5816dc112c5ece17ce1b2568c3c71b2467f63e756398771ab6cc3f5c03b5. All eleven records independently matched before inspection and again before writing this report.

| Source | SHA256 |
|---|---|
| bibliography.rs | 1d7baae3c96cc5e46235296e518033f2cbac1f86cb786b6cddb1542df24c37dc |
| regression_test.rs | 02475930ba20b5e169370835d2279b145436ad678b4aaac28aee9f9f13584873 |
| field.rs | 63db78f9e205c56f2982c46a5999092c0041e00204256455669d7b6da07001d7 |
| document.rs | 4057ef506a747776554fc57832eabc7daadc516f8ab8618b5af2961bb08910be |
| lib.rs | 2071060310142958b0fb1276ee967bf69840d039c85afd19dc0209e28a4d32a0 |
| layout engine.rs | 90f328bb37575d257ae0b34fbeb1ef3ebdf6539f4c1dafe71ad4db84e3db9e6e |
| oxml text.rs | cc6bb3ec2b64f2fec3aa06b55820386daa8ca42fd5de51020b53bd97bcd8f354 |

This is an independent static OOXML aspect review. Compiled pre-fix failures, post-fix passes, formatting, F-X191 prefix reconciliation, scoped verification and final ALL acceptance remain pending. No Cargo, Git mutation, network, native UI or source write occurred. The only written file is this review. The pass ends here and returns control to orchestration.
