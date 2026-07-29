# 15, Build and toolchain

## Toolchain pinning

The repository pins its development toolchain in `rust-toolchain.toml`:

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.97.1"
components = ["rustfmt", "clippy"]
targets = ["wasm32-unknown-unknown"]
```

MSRV stays declared separately as `rust-version = "1.93"` in
`[workspace.package]`, and the MSRV CI job continues to pin it explicitly. The
two answer different questions: the toolchain file says what this repository is
developed with, `rust-version` says what it is guaranteed to compile with.

## Deterministic rendering

`crates/rdocx-layout/src/font.rs:93` calls `db.load_system_fonts()`. System
fonts differ between machines and between a developer's machine and a CI runner.

**This makes any recorded render baseline unreproducible**, which would poison
both the hash harness and the SSIM gate: a baseline recorded locally would
mismatch CI in a way indistinguishable from a real regression.

`rdocx-layout` provides an explicit deterministic mode:

```rust
impl FontManager {
    /// Bundled fonts only. System font loading is bypassed entirely.
    /// Rendering is then reproducible across machines and platforms.
    pub fn new_deterministic() -> Result<Self>;
}
```

The constructor starts with a fresh font database, loads only the checked-in
bundled font bytes, and returns a layout error when `bundled-fonts` is disabled.
It never calls system-font discovery. Document-embedded fonts remain explicit
layout inputs, so they are deterministic too.

`Engine::new_deterministic()` and `layout_document_deterministic()` carry that
database through layout. The public facade exposes
`Document::render_page_to_png_deterministic()`, which lays out and rasterises a
page through the same path. Existing constructors and rendering methods still
load system fonts for library users.

The hash harness, golden-PNG gate and SSIM harness use the deterministic path.
The normal rendering API does not change its font-discovery behaviour.

The `--no-default-features` path verifies that deterministic construction fails
early without bundled fonts. This is also the same font-isolation path the WASM
build needs.

Builds otherwise stay native. There is no development container. The Linux-only
work, manylinux wheels, `wasm32` checks and the LibreOffice render oracle, runs
on CI runners as it always would have.

## Feature flags

| Crate | Feature | Default | Notes |
|---|---|---|---|
| `rdocx-layout` | `bundled-fonts` | on | The 20 bundled TTFs and the deterministic rendering path |
| `oxml-layout` | `system-fonts` | on | **New.** Off for wasm, where `fontconfig` will not build |
| `rpptx` | `default-template` | on | The bundled `default.pptx` |
| `rpptx` | `render` | on | Pulls in `rpptx-render` and `oxml-pdf` |
| `rdocx-py`, `rpptx-py` | `extension-module` | off | Must stay off for `cargo test` |

`fontdb`'s `fontconfig` feature is enabled workspace-wide today. It does nothing
useful on musl or Windows and must be gated per-target for wheel builds.

## Packaging

`crates/rdocx-layout/fonts/` is **6.8 MB of TTFs outside `src/`**, published
today only because `cargo publish --no-verify` skips the build-from-archive
check, and there is no `include` or `exclude` in the manifest.

```toml
[package]
include = ["src/**/*", "fonts/*.ttf", "fonts/LICENSE-*", "fonts/NOTICE-*", "README.md"]
```

Drop `--no-verify`, and assert the resulting `.crate` size against the 10 MiB
crates.io limit in CI. Roughly 3.5 to 4 MB compressed is expected, but measure
rather than assume.

The same treatment applies to `crates/rpptx/assets/default.pptx`. **An asset
must live under its own crate's directory**: a workspace-root `assets/` compiles
locally but is not collected into the published tarball.

Every bundled font family has its licence under the crate's `fonts/` directory.
Caladea ships with the full Apache License 2.0 text in `LICENSE-Caladea` and its
copyright, trademark and designer attribution in `NOTICE-Caladea`. The
`bundled_fonts.rs` module documentation identifies Caladea as Apache-2.0 and
the Carlito and Liberation families as SIL Open Font License 1.1.

## Publishing

The dependency order grows to roughly twenty crates:

```
oxml-core -> oxml-opc -> oxml-media -> oxml-drawing -> oxml-layout -> oxml-pdf
  -> oxml-sml -> oxml-cli-support
  -> rdocx-oxml -> rdocx-layout -> rdocx-html -> rdocx -> rdocx-cli
  -> rpptx-oxml -> rpptx-layout -> rpptx-render -> rpptx-chart -> rpptx -> rpptx-cli
```

The fourteen future crates.io names in this graph are reserved at version
0.0.0 under the owner `mantissaman`: `oxml-core`, `oxml-opc`, `oxml-media`,
`oxml-drawing`, `oxml-layout`, `oxml-pdf`, `oxml-sml`, `oxml-cli-support`,
`rpptx-oxml`, `rpptx-layout`, `rpptx-render`, `rpptx-chart`, `rpptx`, and
`rpptx-cli`. Each placeholder is dependency-free and exposes no usable API.

`oxml-py-support`, `rpptx-py`, and `rpptx-wasm` are not reserved on crates.io.
The binding crates are not published there, and the WASM packages use the npm
publication path.

`publish.yml` currently sleeps 60 seconds between each of seven crates, which is
six minutes of unconditional waiting that is **still racy**. At twenty crates it
is twenty minutes. Replace with `cargo publish --workspace`, which handles
ordering and index propagation and is available at the pinned toolchain.

Also narrow `|| echo "already published"`. It currently swallows authentication
failures, network errors and genuine compile errors identically to a real
duplicate. Match on the actual "already exists" message and re-raise everything
else.

Two tag namespaces:

| Tag | Workflow | Publishes |
|---|---|---|
| `v*` | `publish.yml` | crates.io, the lockstep family |
| `rpptx-v*` | `publish.yml` | crates.io, the incubating family |
| `py-v*` | `wheels.yml` | PyPI via OIDC trusted publishing |

Wheels are separate so a Rust patch release does not rebuild twelve wheels, and
a binding-only fix does not force a crates.io release.

## Release process

`scripts/release.sh` is deleted. It is BSD-`sed` only, its version replacement
is an unanchored global substitution, and its README rewrite globally replaces
the bare string `"0.2"` across the whole file, silently corrupting anything else
that happens to be quoted that way.

`cargo-release` replaces it:

```toml
# release.toml
consolidate-commits = true
pre-release-commit-message = "Release v{{version}}"
tag-name = "v{{version}}"
tag = true
push = true
publish = false          # publishing is publish.yml's job, on the tag
```

```toml
# crates/rpptx*/Cargo.toml, during incubation
[package.metadata.release]
shared-version = false
tag-name = "rpptx-v{{version}}"
```

```bash
cargo release 0.3.0 --workspace --exclude rpptx --exclude rpptx-oxml \
                    --exclude rpptx-layout --exclude rpptx-render --exclude rpptx-chart --execute
cargo release 0.1.4 -p rpptx-oxml -p rpptx-layout -p rpptx-render -p rpptx --execute
```

The Python package version tracks the Rust train through a
`pre-release-replacements` entry so the wheel version and the crate version
cannot diverge.

## CI job matrix

Listed in `12-testing-strategy.md`. Two additions specific to this document:

**`--exclude rdocx-py --exclude rpptx-py` on every `--all-features` job.**
`pyo3/extension-module` tells the linker the Python symbols come from the host
interpreter, which is false for a test binary. On Linux this is an
unresolved-symbol link failure that is easy to misdiagnose as something else.

**A `wasm32-unknown-unknown` check job**, without which the binding crates drift
again exactly as `rdocx-wasm` already has.

## Dependency policy

`deny.toml` is well-documented and stays as it is. The codec bans (`zstd`,
`bzip2`, `lzma-rust2`, `ppmd-rust`, `aes`) apply to the whole workspace, since
`zip` is in every graph through `oxml-opc`.

The single advisory exception, an unmaintained transitive font dependency,
carries its exit route in a comment. Keep that discipline: an ignore without a
stated exit route is a permanent ignore.

New dependencies are added only with a named consumer. The workspace already
minimises feature sets deliberately, and the comments in the root manifest
explaining why `zip` and `fontdb` are trimmed should be preserved rather than
regenerated.
