# F-221, all, pass 1

**Reviewed**: uncommitted working tree implementation diff, 3 files, 336 changed lines, with 336 additions and 0 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, verification rewrites untouched signed PresentationML before checking it
`crates/rpptx/src/lib.rs:676`

`verify_signatures` always calls `staged_package`. That helper unconditionally
serializes `presentation.xml` at `crates/rpptx/src/lib.rs:770` and every slide
and notes slide at `crates/rpptx/src/lib.rs:825`, even when the caller has made
no typed mutation. Those serializers write fixed prefixes and canonical
modeled shells, so a valid signature created by PowerPoint or another producer
over lexically different but valid PresentationML is checked against rewritten
bytes and is falsely reported invalid immediately after open. The current
regression at `crates/rpptx/tests/integration.rs:147` cannot catch this because
it signs only after the facade has already canonicalized its own fixture.

Preserve retained bytes for unchanged modeled parts and stage only parts whose
typed state has changed. Add a regression that signs a source-built package
before the `Presentation` facade opens it, using producer-shaped XML that is
not byte-identical to the fixed-prefix serializer, then require an untouched
open and verify to remain cryptographically valid.

### D2, the encrypted-save atomicity test never exercises password failure
`crates/rpptx/tests/integration.rs:130`

The approved test contract requires both password and publication failures to
leave the destination and presentation unchanged. This test creates a
directory at the destination and passes a valid password, so it exercises only
the publication failure after encryption succeeds. The empty-password failure
path in `write_encrypted_to` is not exercised through `save_encrypted`, and a
regression that opens or truncates the destination before password validation
would pass this gate. Start with sentinel destination bytes, call
`save_encrypted` with an invalid empty password, and assert both the sentinel
bytes and presentation bytes remain unchanged. Keep the directory case as the
separate publication-failure assertion.

### D3, the mandatory PowerPoint encryption oracle has no F-221 gate or evidence
`.claude/plans/F-221-design.md:85`

The approved differential test requires pinned PowerPoint 16.104 build
16.104.25121423 to accept the generated encrypted deck with the right password
and reject it with the wrong password. The added F-221 test block ends with the
feature-isolation test at `crates/rpptx/tests/integration.rs:218` and contains
neither `powerpoint_opens_the_written_agile_presentation` nor an evidence
record for those two observations. The existing PowerPoint constants and
visual-deck evidence cover older unencrypted fixtures, not this generated
encrypted artifact. Record the generated artifact identity, exact pinned
build, correct-password success, and wrong-password rejection in the existing
integration binary, following the repository's established manual-evidence
pattern.

## Smells

None.

## Nitpicks

None.

## Not found

Panic safety produced zero findings. OOXML schema child order, namespace
handling, and unmodeled subtree preservation produced no additional finding
beyond D1's rewriting of unchanged signed part bytes. Cross-platform atomic
path publication produced zero findings. Public API shape and feature leakage
produced zero findings. Structure produced zero findings. Smells produced zero
findings. Nitpicks produced zero findings.

The new features are default-off and have the named native `Presentation`
consumer. Separate `cargo tree` inspections of ordinary `rpptx`, `rpptx-py`,
`rpptx-wasm`, and `rpptx-cli` graphs found neither security feature in those
graphs. No new crate, module, file, trait, generic, builder, forwarding wrapper,
or production dependency was introduced. The native sibling-file publication
uses exclusive creation, syncs before replacement, cleans failed staging
files, and uses the established Unix rename and Windows `MoveFileExW` paths.

The complete `rpptx` integration binary with both security features enabled
passes 99 tests with 7 ignored. Passing tests do not clear D1 through D3 because
the producer-signed lexical variation, password-failure save path, and F-221
PowerPoint oracle are absent from the gate.
