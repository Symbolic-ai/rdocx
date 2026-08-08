# F-105, Bundled default.pptx

**Status**: completed
**Sprint**: S26
**Size**: M
**Depends on**: F-065

## Problem

The `rpptx` facade can only open existing packages. `Presentation` has no
constructor at `crates/rpptx/src/lib.rs:80`, and the crate manifest has no
feature or packaged presentation asset at `crates/rpptx/Cargo.toml:1`. Building
the complete Office theme and eleven standard layouts in Rust would duplicate a
large producer-owned template and create a write-only construction path.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Presentation and slides".
- `docs/hld/03-architecture.md`, "Three families, one workspace".
- `docs/hld/06-presentationml-model.md`, "The bundled template".
- `docs/hld/14-development-backlog.md`, "F-105, Bundled default.pptx".
- `docs/hld/15-build-and-toolchain.md`, "Feature flags", "Packaging", and
  "Publishing".

## Approach

Add `crates/rpptx/assets/default.pptx` as the crate-local zero-slide 16:9
template described by the specification. Its package graph contains one
master, eleven standard layouts, a full theme, presentation properties, view
properties, table styles, and a notes master. Record its source and licence in
the existing crate documentation and verify its exact package contents in the
existing integration test binary.

Add the manifest feature `default-template`, enabled by default. Behind that
feature, add this facade constructor:

```rust
impl Presentation {
    pub fn new() -> Result<Self>;
}
```

`new()` passes `include_bytes!("../assets/default.pptx")` to the same
`from_bytes` path used for caller-supplied packages. This keeps one parser and
one owned package representation. No alternative template generator, source
module, or dependency is introduced. The crate remains version `0.0.0` and
`publish = false`.

## Rejected alternatives

- Generate the master, layouts, theme, and auxiliary parts in Rust. This is
  roughly 2,500 lines of producer XML with no second runtime need.
- Store the asset at workspace root. Cargo would omit it from the crate
  archive.
- Make the template unconditional. The specification requires consumers to be
  able to compile the facade without bundled template bytes.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `new_presentation_uses_the_bundled_zero_slide_template` | `Presentation::new()` returns a 16:9 deck with zero slides and reopens after deterministic serialization |
| integration | `bundled_template_has_the_documented_part_graph` | The asset has one master, eleven layouts, full theme and auxiliary parts, a notes master, and no slide parts |
| regression | `rpptx_without_default_template_feature_still_compiles` | `cargo check -p rpptx --no-default-features --all-targets` succeeds without exposing `Presentation::new()` |
| packaging | `rpptx_package_contains_the_default_template` | `cargo package --list -p rpptx` includes `assets/default.pptx` and the crate remains unpublished |
| acceptance | `new_presentation_opens_in_powerpoint_without_repair` | A file emitted from `Presentation::new()` opens in native PowerPoint without a repair prompt |

The backlog test gate is named explicitly: `Presentation::new()` produces a
deck PowerPoint opens without repair.

## HLD impact

None. The existing HLD already specifies the constructor, feature, asset
location, package graph, and publication state.

## Risk routing

- Bundled asset under `crates/rpptx/assets/`: read
  `docs/hld/15-build-and-toolchain.md`. Check the asset is inside the crate,
  present in `cargo package --list -p rpptx`, and has recorded source and
  licence evidence.
- New feature flag and change to `default`: read the structural rules in
  `CLAUDE.md`. The named current consumer is `Presentation::new()`. Run both
  default and `--no-default-features` checks.
- New file: read the structural rules in `CLAUDE.md`. The explicitly named
  story artifact is `crates/rpptx/assets/default.pptx`. Add no source module.
- Parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Reopen the serialized template and
  verify package relationships, schema-valid roots, and preserved opaque
  parts.
- Public API of an unpublished crate: read `docs/hld/10-bindings-spec.md` and
  the structural rules in `CLAUDE.md`. State that there is no released semver
  impact and inspect the crate package contents and size.

## Hash harness

Expected to be unchanged. The asset and unpublished PowerPoint facade do not
participate in the Word rendering hashes.

## Implementation checklist

- [x] Add the crate-local zero-slide template with recorded provenance.
- [x] Add the default-enabled `default-template` feature.
- [x] Add `Presentation::new()` through the existing parser path.
- [x] Verify the exact template package graph and deterministic reopen.
- [x] Prove both feature modes compile and the package archive contains the
  asset.
- [x] Produce native PowerPoint no-repair evidence for the backlog gate.

## Completion evidence

The asset was generated with python-pptx 1.0.2 from its MIT-licensed default
template. The slide size is 12,192,000 by 6,858,000 EMU with
`type="screen16x9"`, and the notes-master infrastructure was materialized before
the temporary slide was removed. The checked-in asset SHA-256 is
`8ecd98d4e52c8ece061cb36c8baa9f0424b362454d820f6df1e425c662b9a057`.

On 2026-08-08, `Presentation::new()` emitted
`/private/tmp/F-105-new.pptx` with SHA-256
`cd7982cd0ffeb2c5155bb0b2de0536ee599b1eccd3016a5aeadb52f7a40bfa05`.
Microsoft PowerPoint 16.104 opened it as the sole presentation at the expected
path and returned to zero open presentations after a no-save close. The
AppleScript run completed without a timeout or repair flow.

## Open questions

None. The story and cited HLD explicitly authorize the named asset path,
feature, template topology, and constructor.
