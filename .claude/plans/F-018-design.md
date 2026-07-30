# F-018, Create oxml-opc

**Status**: completed
**Sprint**: S04
**Size**: M
**Depends on**: none

## Problem

The format-neutral OPC implementation still lives in the published Word crate.
`crates/rdocx-opc/src/package.rs:24` defines the package model, while
`crates/rdocx-opc/src/package.rs:218` and
`crates/rdocx-opc/src/content_types.rs:173` expose DOCX-specific constructors.
The workspace has no `oxml-opc` member at `Cargo.toml:3`, so PresentationML
cannot reuse the package implementation without depending on a published
`rdocx-*` crate.

The extraction must preserve all existing rdocx code and package output. The
new development crate must remain at version 0.0.0 with publishing disabled,
and this story must not connect any published rdocx crate to it before the
rdocx 0.5.0 release boundary.

## Spec reference

- `docs/hld/03-architecture.md`, "Three families, one workspace", "The
  dependency rule", "Why these seams", and "Versioning".
- `docs/hld/04-opc-and-packaging.md`, "The package", "Generalising the
  constructors", and "What transfers unmodified".
- `docs/hld/11-migration-plan.md`, opening staged-migration contract and "Order
  of operations".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "The hash harness", and
  "New tests the extracted crates need", subsection `oxml-opc`.
- `docs/hld/15-build-and-toolchain.md`, "Publishing" and "Dependency policy".

## Approach

Add `crates/oxml-opc` as a workspace member and workspace dependency. Its
manifest uses `version = "0.0.0"` and `publish = false`, with the existing
workspace `zip`, `quick-xml`, and `thiserror` dependencies. It has no dependency
on `oxml-core`, `rdocx-*`, or `rpptx*`.

Stage copies of the four existing `rdocx-opc` source modules and their eleven
tests in the new crate. Leave `rdocx-opc` and every current consumer unchanged
until F-022 installs the compatibility shim after the rdocx 0.5.0 release
boundary. Apart from crate-neutral documentation and the constructors below,
the copied implementation remains byte-for-byte equivalent.

Replace the two DOCX-specific constructors only in `oxml-opc` with this public
surface:

```rust
impl OpcPackage {
    pub fn new() -> Self;
    pub fn with_main_part(part_name: &str, content_type: &str) -> Self;
}

impl Default for OpcPackage {
    fn default() -> Self;
}

impl ContentTypes {
    pub fn minimal() -> Self;
}
```

`ContentTypes::minimal()` contains only the universal `rels` and `xml`
defaults. `OpcPackage::new()` uses those content types and empty relationship
and part maps. `with_main_part()` starts from `new()`, adds the
`officeDocument` package relationship to the package-relative `part_name`, and
adds the content-type override under the corresponding leading-slash part key.
`Default` delegates to `new()` so the public zero-argument constructor clears
the workspace Clippy gate without introducing a second construction path.

Keep DOCX setup inside local test helpers. The moved tests exercise the same
behaviour without retaining a format-specific public constructor. F-019 owns
the additional relationship and content-type constants, so this story does not
add them.

## Rejected alternatives

- Delete or replace `rdocx-opc` in this story. That would connect published
  rdocx crates to an unpublished development crate before the release boundary.
- Keep public `new_docx` and add `new_pptx`. The leaf crate would accumulate
  format-specific presets, contrary to the constructor design in the OPC spec.
- Introduce a `PackageKind` enum or feature-gated constructors. Both make a
  format-neutral leaf aware of its consumers and add states the current story
  does not need.
- Add an `oxml-core` dependency. The architecture deliberately keeps
  `oxml-opc` independently consumable.
- Add PresentationML constants now. F-019 owns that separately reviewable
  surface.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | eleven moved `content_types`, `package`, and `relationship` tests | Existing parsing, writing, target resolution, relationship handling, deterministic saves, and round-trip behaviour pass from `oxml-opc` |
| unit | local DOCX fixture helper used by the two format-specific cases | The old DOCX test setup is reconstructed without a public DOCX constructor |
| unit | `minimal_content_types_contain_only_universal_defaults` | `minimal()` contains `rels` and `xml`, with no overrides |
| unit | `with_main_part_resolves_and_round_trips` | A generic main part creates the correct relationship and override, resolves through `main_document_part()`, and survives write and read |
| integration | `cargo check -p oxml-opc --all-targets` | The new crate compiles independently on every native target |

The backlog **test gate** is that the eleven moved tests pass, with the two
DOCX-specific cases rebuilt on a local fixture helper.

## HLD impact

None. The architecture, OPC constructor contract, staged migration order,
tests, and unpublished development version already describe this
implementation.

## Risk routing

- New crate, modules, and files. F-018 explicitly authorizes `oxml-opc`.
  Confirm no extra trait, generic parameter, feature flag, module, or file is
  introduced beyond the copied crate and required manifest.
- Crate dependency graph. Inspect `cargo tree -p oxml-opc` and assert that no
  `rdocx-*` or `rpptx*` package appears. Confirm `oxml-opc` does not depend on
  `oxml-core`.
- Parser and serializer. Keep the existing fixed namespace and child order on
  write, retain the moved content-type and relationship round-trip tests, and
  compare the copied parser and serializer bodies against `rdocx-opc`. The OPC
  tables model their complete child records, so there is no unmodelled subtree
  for `capture_element` to retain in this story.
- File move with no behaviour change. Compare the shared source bodies after
  excluding crate-neutral documentation and constructor blocks. Run the hash
  harness and require every existing digest to remain byte-identical.
- Version strings. Inspect the root manifest, `crates/oxml-opc/Cargo.toml`, and
  `Cargo.lock` diff. Confirm version 0.0.0, `publish = false`, and no change to
  the explicit seven-crate release allowlist. Do not tag or publish.

## Hash harness

Expected to remain unchanged. The published rdocx crates and their consumers
continue using the original implementation, and any digest delta blocks the
sprint.

## Implementation checklist

- [x] Add the unpublished version 0.0.0 crate to the workspace and dependency
      table.
- [x] Stage the existing OPC modules and all eleven tests in `oxml-opc`.
- [x] Generalise the content-type and package constructors without changing
      the remaining implementation.
- [x] Rebuild DOCX-specific test setup behind local helpers.
- [x] Keep `rdocx-opc` and all published rdocx consumers unchanged.
- [x] Run the focused tests, dependency inspection, manifest audit, full gate,
      and unchanged hash harness.

## Open questions

None. The story, specification, and release boundary determine a staged
unpublished crate. F-022 controls the later compatibility shim and consumer
switch.
