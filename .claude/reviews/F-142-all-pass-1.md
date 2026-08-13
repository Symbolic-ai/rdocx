# F-142, all, pass 1

**Reviewed**: the complete 15-file working diff, 974 insertions and 507 deletions, against the approved plan, progress notes, and HLD 03, 08, 10, 12, 14, and 15
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the size gate accepts an artifact unrelated to the current default build

`crates/rpptx-wasm/src/lib.rs:128`
`crates/rpptx-wasm/src/lib.rs:142`
`crates/rpptx-wasm/src/lib.rs:153`
`.claude/plans/F-142-design.md:41`
`docs/hld/12-testing-strategy.md:394`

The named gate checks the versions of caller-supplied `wasm-pack` and
`wasm-opt` executables, then independently reads an arbitrary caller-supplied
gzip path. It never invokes either tool, runs `gzip -n -9`, checks that the gzip
contains a WebAssembly module, or binds the measured bytes to a default-profile
build of the current `rpptx-wasm` source. It also checks only gzip magic and
mtime, not the exact level-nine header or absence of optional filename data.
This makes the central story gate vacuous. Independently pointing
`RPPTX_WASM_GZIP_PATH` at an unrelated 20-byte deterministic gzip of an empty
file made `default_profile_is_under_one_megabyte_and_round_trips_a_deck` pass.
The round-trip assertions later in the test do not use that artifact. The gate
must construct or cryptographically attest the exact current default artifact
after the approved wasm-bindgen, wasm-opt 125, and `gzip -n -9` pipeline, then
measure those same bytes. Sensitivity must reject artifact substitution as well
as padding.

### D2, HLD15 still describes the implemented wrapper as deferred

`crates/rpptx-wasm/Cargo.toml:2`
`docs/hld/15-build-and-toolchain.md:142`

The working tree adds `rpptx-wasm` as an implemented workspace crate, but the
plan-listed build and packaging specification still says that `rpptx-wasm`
remains deferred to F-142. HLD files describe current intent, so this sentence
is stale in the very change that fulfills F-142. It should state the current
unpublished binding status and leave only the npm publication path deferred to
F-146.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness and facade ownership: the wrapper owns one concrete
  `rpptx::Presentation`. The extracted renderer assembly remains on the facade
  side, and the corpus example delegates to it. The complete `rpptx` test suite
  passed with 104 tests passing and seven explicitly ignored external gates.
- Default and render profiles: native tests passed in both profiles. The
  default wasm32 tree omits `rpptx-render`, `oxml-pdf`, `tiny-skia`, host font
  discovery, and `getrandom`. The render tree adds the renderer, PDF backend,
  and rasterizer without host font discovery. Both wasm32 checks passed.
- JavaScript boundaries: exact wasm-pack 0.15.0 Node tests passed one default
  test and two render-profile tests. The tests reflectively exercise the
  generated names and `Uint8Array` conversions, and the render profile returns
  a complete PDF envelope.
- Round trip and OOXML: the native and Node gates retain the constructed deck's
  part inventory, content types, relationship scopes, and reopenable slide
  count. No new parser or serializer, schema-order regression, or loss of
  unmodelled XML was found.
- Panics and errors: no new panic reachable through untrusted exported input
  was found. Binding failures map facade errors to string-valued `JsValue`s.
  New `unwrap` and `expect` calls are confined to tests or length-proven slices.
- Structure and surface: the two approved new crate files are the only new
  files. No speculative trait, generic, module, wrapper layer, npm metadata, or
  publication action was added. The published facade surface is additive and
  gated behind the named `render` feature.
- Tooling and artifacts: the root lock contains one coherent unpublished
  `rpptx-wasm` package entry. The no-default facade library check, formatting,
  prose, generated-skill drift, metadata, and diff checks passed.
