# F-109 progress notes

## Current state

Implementation, tests, review, HLD, and verification are complete. The two
defects from microscope pass 1 are remediated, and microscope pass 2 reports 0
defects and 0 smells.
An inserted group transform now moves preserved group-property children after
the transform boundary. The nested-group regression now proves two distinct
shape ids and sibling order survive mutation.
The listed HLD now describes the mutable borrow handles, supported setters,
read-only `AlternateContent` projection, and raw XML preservation limits.

## Changed areas

- `crates/rpptx/tests/integration.rs`
- `crates/oxml-drawing/src/geometry.rs`
- `crates/oxml-drawing/src/order.rs`
- `crates/rpptx-oxml/src/connector.rs`
- `crates/rpptx-oxml/src/graphic_frame.rs`
- `crates/rpptx-oxml/src/namespace.rs`
- `crates/rpptx-oxml/src/picture.rs`
- `crates/rpptx-oxml/src/shape_tree.rs`
- `crates/rpptx/Cargo.toml`
- `crates/rpptx/src/lib.rs`
- `Cargo.lock`

## Last green check

The required non-fast `/verify` passed on 2026-08-08 over the post-remediation
diff. Formatting, clippy, all changed-crate suites, the corpus-backed workspace
suite, hash harness, prose, skill drift, no-default-features, wasm, and docs all
passed. The hash harness matched all 28 entries. The focused
`shape_mutation_setters_survive_save_and_reload` gate passed. Searches against
the claim `HEAD` found neither `slide_mut` and its setters nor the named gate,
which proves the consumer test would fail to compile before this feature.

## Blockers

None.

## Next action

Commit the reviewed implementation and prepare the validated integration
handoff.
