# F-142, all, pass 4

**Reviewed**: the complete 16-path working diff, 1,188 insertions and 515 deletions, against the approved plan, progress notes, passes 1 through 3, and HLD 03, 08, 10, 12, 14, and 15
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- The post-pass-3 regression correction is accurate and sensitive to the
  intended native profile. `crates/rpptx-render/src/lib.rs:3510` requires the
  facade default to retain `default-template`, `render`, and `system-fonts`.
  `crates/rpptx-render/src/lib.rs:3514` requires weak system-font forwarding,
  so selecting `system-fonts` alone does not activate the optional renderer.
  `crates/rpptx-render/src/lib.rs:3517` requires the facade renderer dependency
  to remain optional with its defaults disabled. Removing render from native
  defaults, making the renderer unconditional, or restoring its default
  features makes the exact assertion fail. The focused regression passed.
- The Python binding still selects the complete native presentation profile
  explicitly at `crates/rpptx-py/Cargo.toml:31`, and the corrected assertion at
  `crates/rpptx-render/src/lib.rs:3520` fixes that consumer contract. The
  resolved feature tree contained exactly `default-template`, `render`, and
  `system-fonts` from `rpptx-py`. It did not rely on the facade package's
  default feature set because the workspace dependency is default-off at
  `Cargo.toml:60`.
- The independent wrapper manifest contract remains coherent. It requires the
  default-off workspace facade dependency, native facade defaults, the exact
  render feature edge, an empty wrapper default, and a bundled-template-only
  facade dependency at `crates/rpptx-wasm/src/lib.rs:461`. The focused contract
  test passed. The inspected default wasm32 tree omitted `rpptx-render`,
  `oxml-pdf`, `tiny-skia`, host font discovery, and `getrandom`. The render tree
  added the renderer, PDF backend, and rasterizer without host font discovery.
- The facade render extraction remains package-owned and deterministic.
  `crates/rpptx/src/lib.rs:506` stages current facade mutations before assembly,
  and `crates/rpptx/src/lib.rs:3625` retains relationship validation, scoped
  media, chart and hyperlink resolution, deterministic font construction,
  bounded-image filtering, source-to-resolved count rejection, and page-count
  rejection. The corpus example delegates to this one path at
  `crates/rpptx/examples/render_deck.rs:154`. The focused no-default facade
  check and facade-to-example parity regression passed.
- The binding surface remains the approved bounded wrapper over one concrete
  `rpptx::Presentation`. Default methods delegate construction, opening,
  serialization, slide counting, and mutation to the facade. Only `toPdf` is
  render-gated at `crates/rpptx-wasm/src/lib.rs:48`. No second package model,
  parser, serializer, mutable alias, or exported panic path was found.
- Passes 1 and 2 remain resolved. The exact normal-default size gate builds the
  current crate, pins wasm-pack 0.15.0 and wasm-opt 125, validates the complete
  reviewed argument vectors, binds deterministic gzip bytes to the freshly
  optimized WebAssembly, and enforces the decimal limit at
  `crates/rpptx-wasm/src/lib.rs:263`. The progress record reports 519,060 bytes
  and byte-identical render-default sensitivity restoration.
- Contract and HLD scope match the approved plan. Exactly the five listed HLD
  files changed, and they describe current facade ownership, WASM profiles,
  tests, packaging, and local CI status. No backlog-shape change, aspiration,
  changelog prose, unapproved file, npm metadata, publication action, baseline
  change, or tracked generated artifact was found.
- Panics, OOXML, and structure produced no findings. New panic sites are test
  only or length-proven. The wrapper adds no parser or serializer and therefore
  changes no schema order or unmodelled XML preservation path. The two approved
  wrapper files are the only new source paths, and no trait, generic parameter,
  forwarding-only layer, or unnamed feature consumer was added. Diff hygiene
  passed after the focused checks.
