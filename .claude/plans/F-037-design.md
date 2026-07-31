# F-037, Create oxml-pdf

**Status**: completed
**Sprint**: S08
**Size**: S
**Depends on**: F-029, F-024

## Problem

`crates/rdocx-pdf/src/lib.rs:1` exposes a backend tied to
`rdocx_layout::LayoutResult`, while `crates/oxml-layout/src/output.rs:297`
now owns the staged format-neutral result. PowerPoint cannot consume the
released backend without reversing the family dependency rule, and
`crates/rdocx-pdf/src/image.rs:31` still duplicates JPEG format and dimension
probing already owned by `oxml-media`.

The staged output enum also contains the planned path and group arms. The copy
must compile without pretending those later rendering stories are complete,
and released `rdocx-pdf` must remain a separate crate until F-046.

## Spec reference

- `docs/hld/03-architecture.md`, "The dependency rule", "Why these seams",
  and "Versioning".
- `docs/hld/08-rendering-spec.md`, "The seam that makes this cheap" and
  "The recursion hazard".
- `docs/hld/11-migration-plan.md`, "Order of operations".
- `docs/hld/12-testing-strategy.md`, "oxml-pdf".
- `docs/hld/14-development-backlog.md`, "F-037, Create oxml-pdf".

## Approach

Add the explicitly planned `crates/oxml-pdf` workspace member at version
`0.0.0` with `publish = false`. Copy the five backend source modules from
`rdocx-pdf`, switch their layout imports to `oxml-layout`, and add only the
direct dependencies used by the copied implementation. The crate exists today
as the second consumer of the staged `oxml-layout` contract, alongside the
staged tests that construct that contract directly.

Depend on `oxml-media` for format sniffing and JPEG dimensions. Delete the
copied JPEG marker walker and its duplicate header tests. Keep the local PNG
pixel decoder because `oxml-media` deliberately probes metadata only and does
not decode pixels for PDF embedding.

Keep `rdocx-pdf` source and manifest unchanged in this story. The staged
backend handles the existing text, line, rectangle, image, link, metadata, and
outline paths. New path and group rendering remains assigned to F-040 and
F-041, so exhaustiveness handling must make unsupported staged arms explicit
without adding placeholder rendering.

## Rejected alternatives

- Make `oxml-pdf` depend on `rdocx-layout`. That violates the format-neutral
  dependency boundary.
- Turn `rdocx-pdf` into a facade now. F-046 owns the published cutover.
- Move all image parsing into `oxml-media`. Pixel decoding is backend work,
  while that crate intentionally owns only format and header metadata.
- Render path and group arms during staging. F-040 and F-041 own those
  behaviours and their focused regressions.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit, gate | `render_empty_layout` | The staged backend writes a complete PDF for an empty page. |
| unit, gate | `render_with_lines_and_rects` | Existing primitive content renders through staged layout types. |
| unit, gate | `render_with_metadata` | Metadata survives the move. |
| unit, gate | `render_with_link_annotation` | Link annotations remain in the page annotation dictionary. |
| unit, gate | `render_with_outlines` | The outline tree survives the move. |
| unit, gate | `decode_jpeg_pass_through` | JPEG bytes and dimensions come through the shared probe path. |
| unit, gate | `indexed_png_expands_palette_to_rgb` | The retained pixel decoder expands indexed colour. |
| unit, gate | `indexed_png_honours_trns_alpha` | The retained decoder produces the soft-mask alpha channel. |
| regression | `staged_backend_has_no_released_family_dependency` | The normal dependency tree contains no `rdocx-*` or `rpptx-*` crate. |

The backlog test gate is that the eight moved tests pass.

## HLD impact

- `docs/hld/03-architecture.md`, update the `oxml-pdf` dependency statement to
  include the shared media metadata boundary used by the staged backend.
- `docs/hld/08-rendering-spec.md`, describe the staged copy as current reality
  while keeping path, group, and cutover work assigned to their later stories.

## Risk routing

- Crate dependency graph. Run `cargo tree -p oxml-pdf --edges normal` and
  confirm no `rdocx-*` or `rpptx-*` edge exists.
- New crate, modules, and files. The approved S08 boundary authorizes
  `crates/oxml-pdf/Cargo.toml` and the five copied source modules. The existing
  `rdocx-pdf` backend is the concrete implementation that justifies staging
  the same backend against the second, format-neutral layout contract today.
  Add no trait, generic parameter, feature flag, or forwarding wrapper.
- File copy with no intended behaviour change. Diff each staged source against
  `rdocx-pdf` and account for only crate imports, shared media probing,
  unsupported future arms, and the tests named above. The hash harness must be
  byte-identical.
- Layout and rendering. Use deterministic fonts for every baseline and require
  the consolidated 28-entry hash harness to remain unchanged.
- Staged package. Run `cargo package -p oxml-pdf --allow-dirty`. While the
  internal crates remain unpublished, preserve its expected extracted-build
  failure against the crates.io `0.0.0` placeholders, then run it with
  `--no-verify`. Enforce the existing 10 MiB archive ceiling and confirm
  `version = "0.0.0"` with `publish = false`. Do not publish the crate.

## Hash harness

Expected to remain unchanged. No released consumer uses `oxml-pdf` in F-037.

## Implementation checklist

- [x] Add the staged workspace member and dependency entries.
- [x] Copy and rewire the five backend source modules.
- [x] Replace duplicated JPEG header probing with `oxml-media`.
- [x] Preserve the local PNG pixel decoder and its focused coverage.
- [x] Make unsupported future element arms explicit without implementing them.
- [x] Pass the eight-test gate, dependency audit, package gate, and hash gate.

## Open questions

None. The new unpublished crate manifest and five copied source files are
explicitly approved for S08.
