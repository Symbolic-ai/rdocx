# F-181, integration, pass 3

**Reviewed**: the staged F-181 squash integration against `ebee604`, 25 files,
7,499 insertions and 20 deletions, including the pass-2 dependency-policy
remediation and the combined F-180, F-181, and F-182 native contracts
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Verification

- `docs/hld/15-build-and-toolchain.md:567` identifies the private native Word
  SVG renderer in `rdocx` as the direct runtime `base64` consumer.
  `crates/rdocx/Cargo.toml:26` declares that ordinary dependency.
- `docs/hld/15-build-and-toolchain.md:569` excludes direct `base64` edges from
  `oxml-*`, Presentation, Python, WASM, and CLI crates. The next sentence at
  `docs/hld/15-build-and-toolchain.md:570` now states explicitly that the
  Python, WASM, and CLI graphs inherit `base64` transitively through `rdocx`.
  Their ordinary manifest edges are at `crates/rdocx-py/Cargo.toml:32`,
  `crates/rdocx-wasm/Cargo.toml:27`, and
  `crates/rdocx-cli/Cargo.toml:28`.
- `cargo tree --locked -p {rdocx-py,rdocx-wasm,rdocx-cli} -i base64
  --edges normal` confirmed each exact `base64 -> rdocx -> binding` path.
  Normal-edge trees for the checked `oxml-*` crates and the `rpptx`,
  `rpptx-py`, `rpptx-wasm`, and `rpptx-cli` graphs contained no `base64`.
  This matches the family boundary at `docs/hld/03-architecture.md:42`.
- `cargo test --locked -p rdocx --lib epub::tests`: 33 passed and 1 ignored.
- `cargo test --locked -p rdocx --lib odt::tests`: 36 passed.
- `cargo test --locked -p rdocx --lib svg::tests`: 19 passed.
- `cargo check --locked -p rdocx --all-targets`: passed.
- `python3 -m unittest scripts.test_sprint_workflow`: 70 passed.
- `cargo fmt --all --check`, `python3 scripts/prose_check.py --staged`, and
  `git diff --cached --check`: passed.

## Not found

No correctness, contract, panic, OOXML or EPUB packaging, test-gate,
public-export, dependency-graph, HLD-discipline, or structural defect was found.
The private module and approved result exports remain at
`crates/rdocx/src/lib.rs:28` and `crates/rdocx/src/lib.rs:50`. The native EPUB
contract at `docs/hld/10-bindings-spec.md:252` remains additive and explicitly
leaves Python, WASM, and CLI surfaces unchanged at
`docs/hld/10-bindings-spec.md:288`. The EPUB ZIP boundary at
`docs/hld/04-opc-and-packaging.md:40` remains consistent with the ODT boundary
and does not enter `oxml-opc`. The staged F-180, F-181, and F-182 HLD and facade
changes remain mutually consistent. No smells or nitpicks were found.
