# F-X001, all aspects, pass 1

**Reviewed**: untracked working-tree implementation, 1 file and 338 inserted lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the text fixture cannot detect loss of body content order

`crates/rdocx-cli/tests/integration.rs:124`

The fixture creates both body paragraphs before appending the table, so its
expected output is identical whether the command traverses body content in
document order or groups every paragraph before every table. The current
command takes the latter path at `crates/rdocx-cli/src/commands.rs:97`, despite
its public comment promising document order and the facade contract preserving
it. A document containing a paragraph, then a table, then another paragraph is
therefore printed as both paragraphs followed by the table, while this test
still passes. Construct an interleaved fixture and require the exact ordered
text output. The resulting regression must fail until the command preserves
the body sequence.

### D2, repeated system-font rendering does not prove deterministic rendering

`crates/rdocx-cli/tests/integration.rs:305`

The test renders the same nearly blank fixture twice on one host and compares
the resulting bytes. That proves only repeatability within one installed font
environment. It neither selects the deterministic bundled-font facade nor
uses a fixture that distinguishes font selection. The command path under test
calls the ordinary system-font-aware renderer at
`crates/rdocx-cli/src/commands.rs:313`, so the output may differ across CI and
developer hosts while this test remains green on each host independently.
Exercise visible text and bind the command assertion to bundled-font-only
rendering, as required by the plan's rendering risk rider.

## Smells

None.

## Nitpicks

None.

## Not found

- Contract and structure. There is exactly one approved integration entrypoint
  and no new dependency, helper module, trait, generic, feature flag, or binary
  fixture. The suite invokes each of the exact seven commands through
  `CARGO_BIN_EXE_rdocx`.
- Command coverage. Inspect binds the schema-1 envelope, convert binds the
  shared default output and all four supported formats, diff checks its
  non-verdict exit status, replace reopens the result, and validate checks both
  success and structural-error exit status. Apart from D1 and D2, the asserted
  observable outputs match the command contracts.
- Sensitivity. The recorded misspelled-subcommand mutation and forced validate
  success are relevant and fail the focused tests. They do not expose D1 or D2.
- Resource and temporary-path isolation. Process ID plus an atomic per-process
  counter separates concurrently running test workspaces. Every command output
  is directed beneath its owning workspace, cleanup is scoped to that exact
  directory, and a stale collision fails creation instead of consuming old
  contents.
- Fixture construction. Valid DOCX inputs and the dangling internal
  relationship are constructed in code. The corruption targets the main
  document relationship set and survives package save, so the validate fixture
  exercises a real missing-part error.
- Panics and OOXML. The diff adds test-only assertions and fixture operations.
  It changes no production parser, serializer, schema ordering, namespace, or
  unmodelled XML preservation path.
