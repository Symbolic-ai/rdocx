# F-222 correctness pass 1

## Verdict

Not ready. Found 2 defects, 0 smells, and 0 nitpicks.

## Defects

### D1. ODP output is bounded only after the complete XML string is allocated

`OdpWriter::write` calls `content_xml` before applying the 64 MiB part limit at
`crates/rpptx/src/odp.rs:1009-1017`. The content writer then appends slide text,
table cell text, and notes directly to one unbounded `String` at
`crates/rpptx/src/odp.rs:1078-1129` and
`crates/rpptx/src/odp.rs:1132-1188`. A caller-controlled large text body can
therefore allocate and escape far beyond the declared output limit before the
serializer rejects it. This violates the plan's bounded writer contract.

### D2. Safe direct ODF content can be dropped without a diagnostic

`parse_page` emits diagnostics only for groups, connectors, and animation
containers at `crates/rpptx/src/odp.rs:791-799`. Other direct ODF presentation
children such as `draw:line`, `draw:polygon`, and `draw:page-thumbnail` fall
through silently. The diagnostic regression at
`crates/rpptx/tests/integration.rs:221-249` covers only an OOXML connector on
export and does not exercise an unsupported ODP import node or the diagnostic
ceiling. This violates the declared loss-aware import boundary.

## Smells

None.

## Nitpicks

None.

## Evidence

- Four ordinary F-222 integration tests pass. The pinned LibreOffice test is
  intentionally ignored in the ordinary suite.
- The pinned LibreOffice 26.2.5.2 two-direction structural and PDF record test
  passes when run explicitly outside the sandbox.
- `cargo clippy -p rpptx --all-targets --all-features -- -D warnings` passes.
- The hash harness remains 49 of 49 unchanged.
