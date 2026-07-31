# F-029, Create oxml-layout

**Status**: completed
**Sprint**: S06
**Size**: M
**Depends on**: none

## Problem

The format-neutral output and font implementation still lives inside the
released `rdocx-layout` crate. `crates/rdocx-layout/src/output.rs` has no docx
dependency, while `font.rs`, `bundled_fonts.rs`, and `error.rs` are also needed
by the PowerPoint renderer. Copying them into a staged crate must not change the
released implementation, its public `Document::load_fonts_from_dir` behavior,
or any rendered output.

## Spec reference

- `docs/hld/03-architecture.md`, "Why these seams" and "Versioning".
- `docs/hld/08-rendering-spec.md`, "The seam that makes this cheap" and "Four
  latent defects to fix".
- `docs/hld/11-migration-plan.md`, "Order of operations" and "Preserve
  behaviour, do not improve it".
- `docs/hld/13-risks-and-open-questions.md`, "R8, oxml-layout packaging".
- `docs/hld/14-development-backlog.md`, "F-029, Create oxml-layout".
- `docs/hld/15-build-and-toolchain.md`, "Feature flags", "Packaging", and
  "Publishing".

## Approach

Add the explicitly planned `crates/oxml-layout` workspace crate at version
0.0.0 with `publish = false`. Copy the current format-neutral `output.rs`,
`font.rs`, `bundled_fonts.rs`, `error.rs`, and bundled font directory into the
new crate, plus the required crate root and manifest. Move the staged
`FontFile { family: String, data: Vec<u8> }` definition into `font.rs` and make
the staged font manager consume it there. Do not alter or delete any
`rdocx-layout` source.

Expose the specified default `system-fonts` feature. The staged crate depends
on `fontdb` directly with its base `std` support, and the feature forwards the
filesystem and fontconfig capabilities. This keeps the feature local even
though the released crate still uses the workspace dependency configuration.
Bundled fonts remain inside the staged crate and deterministic construction
never discovers system fonts. The existing default path is the named feature
consumer, and the required no-default-features test is the current disabled
path consumer.

Keep the public surface to the copied output types, `FontFile`, font manager
types, bundled font access, and layout error. Do not introduce a layout input,
engine, paginator, wrapper, trait, or speculative extension.

## Rejected alternatives

- Move the released files now. The migration plan requires isolated staging
  and defers the consumer cutover until the shared crates can be published.
- Inherit the workspace `fontdb` feature set unchanged. That would make the new
  `system-fonts` feature unable to disable filesystem discovery in its scoped
  no-default build.
- Add an `input.rs` containing only `FontFile`. The type belongs beside the
  staged font manager, and a forwarding module would add another place to look.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | copied `font.rs` test suite | The existing resolution, shaping, metrics, cache, and deterministic-font tests retain their names and behavior in `oxml-layout`. |
| regression | `font_manager_with_no_fonts_returns_an_error` | An empty explicit font database reports `FontNotFound` rather than panicking. |
| unit | `every_bundled_font_family_has_a_licence_file` | Every staged family has its copied licence and Caladea notice. |
| regression | `rdocx_load_fonts_from_dir_is_unchanged` | The sprint diff contains no released `rdocx-layout` source or manifest change. |
| integration | `no_default_features_omits_system_font_discovery` | `cargo test -p oxml-layout --no-default-features` compiles and exercises the disabled system-font path. |

The backlog test gate is that the copied tests pass in `oxml-layout`, and the
existing `Document::load_fonts_from_dir` remains unchanged.

## HLD impact

None. The architecture, migration, rendering, testing, risk, and packaging
documents already specify this staged crate and its feature boundary.

## Risk routing

- Layout and text shaping. Use deterministic font mode for the hash gate and
  require all 28 entries to remain unchanged.
- Crate dependency graph. Run `cargo tree -p oxml-layout --edges normal` and
  reject every `rdocx-*` or `rpptx*` dependency.
- Bundled fonts. Inspect the packaged archive for all 20 TTF files, the three
  licence files, and `NOTICE-Caladea`, then assert the archive is below 10 MiB.
- New feature flag. Name `FontManager::new` as the enabled consumer and the
  no-default-features test as the disabled consumer. Run both feature modes.
- New crate, modules, and files. F-029 explicitly authorizes only the crate,
  manifest, crate root, four copied source modules, and font assets described
  above. Add no trait or generic parameter.
- File copies with no behavior change. Diff the copied source against
  `rdocx-layout`, account only for crate-path, feature, and `FontFile` changes,
  and require the hash harness to remain byte-identical.
- Version strings and release boundary. Inspect the root manifest, new
  manifest, lockfile, and publish workflow. Keep the crate at 0.0.0 with
  publication disabled and outside the seven-crate release allowlist.

## Hash harness

Expected to remain unchanged. The staged crate has no released consumer, and
any output delta blocks the sprint.

## Implementation checklist

- [x] Add the unpublished `oxml-layout` workspace member and local feature.
- [x] Copy only the approved format-neutral source and bundled font assets.
- [x] Move staged `FontFile` into the font module without touching rdocx.
- [x] Add the empty-font regression and preserve copied font and output tests.
- [x] Run both feature modes, dependency, archive-content, size, release, and
      unchanged-hash riders.

## Open questions

None.
