# F-181, integration, pass 2

**Reviewed**: the staged F-181 squash integration against `ebee604`, 24 files,
7,435 insertions and 17 deletions, including the pass-1 HLD remediation, the
reconciled F-180, F-181, and F-182 native contracts, and the staged EPUB public
exports
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the promised transitive binding-graph consequence is still unstated

`docs/hld/15-build-and-toolchain.md:547`

The corrected policy now properly limits Ring, SHA-256, and X.509 parsing to
the default-off `digital-signatures` feature and identifies `base64` separately
as an ordinary runtime dependency. It then promises that the binding-graph
consequences are described below. The later paragraph only says that Python,
WASM, and CLI crates add no direct edge at
`docs/hld/15-build-and-toolchain.md:569`. It does not state that all three
ordinary binding graphs nevertheless include `base64` transitively through
their `rdocx` dependencies at `crates/rdocx-py/Cargo.toml:32`,
`crates/rdocx-wasm/Cargo.toml:27`, and `crates/rdocx-cli/Cargo.toml:28`.
`rdocx` declares the unconditional direct edge at
`crates/rdocx/Cargo.toml:26`. The distinction between manifest ownership and
resolved graph membership therefore remains incomplete.

## Smells

None.

## Nitpicks

None.

## Verification

- `cargo test --locked -p rdocx --lib epub::tests`: 33 passed, 1 ignored.
- `cargo check --locked -p rdocx --all-targets`: passed.
- `cargo tree -p rdocx-py -i base64 --edges normal`: confirmed the transitive
  `base64 -> rdocx -> rdocx-py` graph.
- `cargo tree -p rdocx-wasm -i base64 --edges normal`: confirmed the transitive
  `base64 -> rdocx -> rdocx-wasm` graph.
- `cargo tree -p rdocx-cli -i base64 --edges normal`: confirmed the transitive
  `base64 -> rdocx -> rdocx-cli` graph.
- `python3 scripts/prose_check.py --staged` and `git diff --cached --check`:
  passed.

## Not found

No fresh correctness, packaging, public-export, native-surface, binding-API,
ODT, SVG, or EPUB contract defect was found in the staged code integration.
`crates/rdocx/src/lib.rs:28` keeps the EPUB implementation private, and
`crates/rdocx/src/lib.rs:50` exports only the approved native result values.
The reconciled ODT, EPUB, and SVG descriptions in
`docs/hld/10-bindings-spec.md:243`, `docs/hld/10-bindings-spec.md:252`, and
`docs/hld/10-bindings-spec.md:553` remain mutually consistent and do not add
Python, WASM, CLI, Presentation, or public `oxml-pdf` APIs. No smells or
nitpicks were found.
