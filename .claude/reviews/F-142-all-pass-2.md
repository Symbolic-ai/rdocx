# F-142, all, pass 2

**Reviewed**: the complete 15-file working diff, 1,175 insertions and 510 deletions, against the approved plan, progress notes, pass 1, and HLD 03, 08, 10, 12, 14, and 15
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the named size gate measures no defaults instead of the normal default profile

`crates/rpptx-wasm/src/lib.rs:127`
`crates/rpptx-wasm/src/lib.rs:190`
`crates/rpptx-wasm/src/lib.rs:200`
`crates/rpptx-wasm/src/lib.rs:282`
`.claude/plans/F-142-design.md:41`
`.claude/plans/F-142-design.md:68`
`docs/hld/12-testing-strategy.md:394`

The remediated gate now invokes the current crate build and binds the gzip to
the freshly optimized WebAssembly bytes, which closes pass 1 D1's artifact
substitution hole. It nevertheless forces `--no-default-features` in the
invoked wasm-pack command and makes that argument part of the accepted exact
pipeline. The contract requires measurement of the normal default profile,
and its sensitivity explicitly permits enabling render in defaults as the
mutation. If `default = []` becomes `default = ["render"]`, this named gate
still builds the same no-default artifact and remains below the limit. The
separate manifest test would detect that edit, but it does not make the named
size gate measure the profile it claims to gate. Build with the crate's actual
defaults so any present or future default feature contributes to the measured
artifact, while retaining the independent manifest assertion that render is
off by default.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-1 D1 provenance remediation: `crates/rpptx-wasm/src/lib.rs:181` builds
  the current crate into an isolated scratch directory, invokes wasm-opt 125
  and deterministic gzip, and retains the exact executed arguments.
  `crates/rpptx-wasm/src/lib.rs:285` requires a WebAssembly module and the
  exact gzip header, while `crates/rpptx-wasm/src/lib.rs:300` requires the
  decompressed gzip bytes to equal the freshly optimized bytes. The exact
  ignored gate passed at 519,060 bytes with wasm-pack 0.15.0 and wasm-opt 125.
  The unrelated-gzip and wrong-pipeline regression also passed.
- Pass-1 D2 HLD remediation: `docs/hld/15-build-and-toolchain.md:142` now
  describes `rpptx-wasm` as an implemented unpublished workspace crate and
  leaves only npm publication to F-146. The feature table and local-gate prose
  at `docs/hld/15-build-and-toolchain.md:80` and
  `docs/hld/15-build-and-toolchain.md:259` match the implementation.
- Correctness and facade ownership: `crates/rpptx-wasm/src/lib.rs:7` owns one
  concrete `rpptx::Presentation`. The default methods delegate package opening,
  saving, counting, and mutation to that facade. The render-only method at
  `crates/rpptx-wasm/src/lib.rs:48` delegates to the facade's deterministic PDF
  boundary.
- Render extraction: `crates/rpptx/src/lib.rs:504` stages the current facade
  package before package-to-render-input assembly. The assembly at
  `crates/rpptx/src/lib.rs:3625` retains relationship validation, deterministic
  fonts, scoped resources, source-to-resolved count checks, and page-count
  checks. The corpus example delegates to that boundary, and the complete
  all-feature `rpptx` suite passed 104 native and integration tests with seven
  explicit external gates ignored.
- Profile dependency boundaries: `crates/rpptx-wasm/Cargo.toml:23` keeps the
  wrapper's render feature off by default and forwards it only to
  `rpptx/render`. The inspected default wasm32 tree omits `rpptx-render`,
  `oxml-pdf`, tiny-skia, fontconfig, and getrandom. The render tree adds the
  renderer, PDF backend, and rasterizer without host font discovery. Both
  wasm32 checks and the no-default `rpptx` library check passed.
- JavaScript and package boundaries: `crates/rpptx-wasm/src/lib.rs:377`
  reflectively exercises generated method names and both `Uint8Array`
  directions. `crates/rpptx-wasm/src/lib.rs:75` checks part inventory, content
  types, relationship scopes, and facade reopenability. Pass 1 records green
  Node suites for both profiles, and no later binding code changed those gates.
- Panics and hostile input: no new panic reachable through an exported binding
  method was found. The new `unwrap` and `expect` calls are confined to tests
  or to slices whose lengths were established immediately beforehand at
  `crates/rpptx/src/lib.rs:3996`.
- OOXML preservation: no parser or serializer was added in the binding. The
  wrapper delegates to the facade, and the moved renderer reads the staged
  complete package without introducing another package authority.
- Structure and scope: the two approved files under `crates/rpptx-wasm` are the
  only new files. No new trait, generic parameter, forwarding-only layer, npm
  metadata, publication action, or unapproved HLD edit was introduced.
- Hygiene: focused default and render native tests, strict binding Clippy,
  formatting, prose, generated-skill sync, and diff checks passed. No tracked
  generated artifact appeared.
