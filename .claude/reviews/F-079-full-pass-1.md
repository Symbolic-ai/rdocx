# F-079, full, pass 1

**Reviewed**: The uncommitted F-079 working diff, 6 files and 982 added lines
**Verdict**: 1 defect, 1 smell, 0 nitpicks

## Defects

### D1, package-root relationship targets are not URI-normalized
`crates/rpptx/src/lib.rs:97`

The main presentation part is formed by prepending `/` to a relative
`officeDocument` target. A valid package relationship target such as
`./custom/presentation-main.xml` therefore becomes
`/./custom/presentation-main.xml`, which does not match the normalized
`/custom/presentation-main.xml` part key and returns `MissingPart`. Targets with
root-clamped `..` segments fail for the same reason. This contradicts the
approved relationship-resolved facade contract. Resolve the target through the
OPC path helper and add this valid relative-target case to the graph tests.

## Smells

### S1, option_record is a forwarding-only helper
`crates/rpptx/examples/dump_deck.rs:39`

`option_record` only forwards its argument to `owned_option_record`, and the
latter also accepts `Option<&str>`. This adds a second lookup site without a
distinct contract and violates the repository rule against wrappers that only
forward. Call the concrete formatter directly or keep one helper.

## Nitpicks

None.

## Not found

- Panics: no panic path was found in facade or example code for untrusted deck
  input. Indexed public access returns `Option` as approved.
- OOXML: no schema-order, prefix, whitespace, or opaque-subtree loss was found
  in this diff. The facade delegates root parsing and fixed-prefix writing to
  the existing typed roots, and the selected alternate-content view remains
  read-only.
- Versioning and dependency direction: no finding. The crate is `0.0.0`, has
  `publish = false`, and the normal dependency graph adds no reverse
  `oxml-*` to `rpptx-*` edge.
- Test execution: with a sandbox-safe uv cache, all seven focused integration
  tests, including the required 50-deck python-pptx 1.0.2 differential gate,
  passed. Scoped clippy and workspace formatting checks also passed. The
  missing normalized main-target case is covered by D1.
