# F-221, all, pass 3

**Reviewed**: uncommitted working tree implementation diff, 3 files, 471 changed lines, with 446 additions and 25 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness produced zero findings. The pass 1 selective-staging defect remains
remediated. `staged_package` compares the retained presentation, slide, and
notes roots before serializing them at `crates/rpptx/src/lib.rs:770`,
`crates/rpptx/src/lib.rs:834`, and `crates/rpptx/src/lib.rs:853`. The
producer-prefixed regression at `crates/rpptx/tests/integration.rs:203` passes
and proves an untouched producer-signed presentation retains valid signature
bytes. The current-state regression at
`crates/rpptx/tests/integration.rs:227` passes for slide, shape, text,
core-property, and package-graph mutations, with each retained signature
reported as cryptographically invalid.

Contract produced zero findings. The facade forwards both default-off features
at `crates/rpptx/Cargo.toml:22` and `crates/rpptx/Cargo.toml:24`, exposes the
planned encrypted constructors and writers at `crates/rpptx/src/lib.rs:519`,
`crates/rpptx/src/lib.rs:531`, and `crates/rpptx/src/lib.rs:899`, and exposes
current-state verification and atomic signing at `crates/rpptx/src/lib.rs:670`
and `crates/rpptx/src/lib.rs:679`. These are additive feature-gated pre-1.0
native APIs. The ordinary, Python, WASM, and CLI feature graphs contain none of
the encryption or signature features or their cryptographic dependencies.

Panic safety produced zero findings. The added production paths use fallible
package, serialization, encryption, signature, allocation, and file operations.
They add no unchecked indexing, slicing, arithmetic, `unwrap`, or `expect` on
untrusted input.

OOXML schema order, namespace handling, and unmodelled subtree preservation
produced zero findings. Encryption and signature protocol logic remains in
`oxml-opc`. The facade stages the retained package, rewrites only changed typed
roots, preserves signature infrastructure for inspection, and leaves unrelated
parts, relationships, content types, and raw PresentationML bytes intact.

Tests and gate sensitivity produced zero findings. Reverting the production
diff removes both requested Cargo features and facade methods, so the added
feature-enabled integration gate does not compile. The complete integration
binary passes 100 tests with 8 ignored. A no-default library check with both
security features passes. The encrypted round trip compares the complete
decrypted staged package, the signing gate verifies cryptographic validity and
complete coverage after reopen, and the feature-isolation test covers every
binding manifest.

The pass 1 encrypted-save atomicity defect remains remediated. The empty-password
case at `crates/rpptx/tests/integration.rs:160` retains sentinel destination
bytes and presentation state. The separate directory case at
`crates/rpptx/tests/integration.rs:170` exercises publication failure. The
implementation stages encryption before exclusive sibling-file creation,
syncs the complete file, uses atomic replacement on Unix and Windows, and
removes a failed staging file at `crates/rpptx/src/lib.rs:421`.

Security produced zero findings. Verification distinguishes cryptographic
validity and complete coverage from caller-owned certificate trust. Signing
uses a staged clone and commits only a shared-verifier result with both
properties true at `crates/rpptx/src/lib.rs:691`. Invalid key material leaves
the live presentation unchanged, and encrypted read and write delegate to the
bounded authenticated shared implementation.

The pass 2 PowerPoint-oracle defect is remediated. The test pins PowerPoint
16.104 build 16.104.25121423 at
`crates/rpptx/tests/integration.rs:47`, binds the candidate SHA-256 at
`crates/rpptx/tests/integration.rs:57`, and requires both password observations
at `crates/rpptx/tests/integration.rs:135`. The progress record contains the
user-confirmed correct-password open and wrong-password rejection for that
exact candidate at `.claude/scratch/F-221-progress.md:44` and
`.claude/scratch/F-221-progress.md:45`. The ignored gate passed with the exact
`correct-password-opened;wrong-password-rejected` observation value and
independently recomputed candidate SHA-256
`a0d33171c63ec084231daeef3b35718f5a2d709a5c92c9c0e2017ccaf9fa52d6`.

Structure produced zero findings. No new crate, module, file, trait, generic,
builder, forwarding wrapper, or production dependency was introduced. The two
new feature flags have the existing native `Presentation` facade as their
named consumer. Smells produced zero findings. Nitpicks produced zero
findings.
