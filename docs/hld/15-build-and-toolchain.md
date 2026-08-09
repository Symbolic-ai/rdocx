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
`Document::render_page_to_png_deterministic()` and
`Document::to_pdf_deterministic()`. Both reuse the separate cached
bundled-font-only layout. The PDF facade passes that layout directly to
`rdocx_pdf::render_to_pdf`, which gives the golden-PNG gate deterministic PDF
input without changing the normal PDF API. Existing constructors and rendering
methods still load system fonts for library users.

The hash harness, golden-PNG gate and SSIM harness use the deterministic path.
The normal rendering API does not change its font-discovery behaviour.

`rpptx-render::layout_presentation_deterministic` applies the same rule to a
whole presentation. It shares page lowering with
`layout_presentation`, bypasses system-font discovery, and adds only explicit
font files from `RenderInput`. The `rpptx` corpus example is an unpublished
development target and does not change any crate publication setting.

The `oxml-layout` `--no-default-features` path disables host system font
discovery while retaining bundled fonts for deterministic construction. This is
also the same font-isolation path the WASM build needs.

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

`publish.yml` explicitly publishes the seven released rdocx packages in
dependency order. It does not use workspace-wide publication. Archive
verification is not skipped. Authentication, network, compilation and
duplicate-version failures all fail the job instead of being relabelled as
success.

The `oxml-*` and `rpptx*` placeholders remain at 0.0.0 until PowerPoint
development is complete. A normal `v*` release must not publish a later version
of any package in those families. Their eventual publication requires its own
reviewed release plan and explicit approval.

Implemented development crates keep the reserved `version = "0.0.0"` and set
`publish = false` in the workspace. The release workflow remains an explicit
allowlist of the seven released rdocx packages, so adding implementation code
does not turn a reserved name into a publication candidate.

`oxml-sml` is an implemented development crate under that rule. It is a
workspace member and workspace dependency at version 0.0.0 with
`publish = false`. Its normal graph contains only `oxml-opc`, `quick-xml`, and
`thiserror`, and its package contains only the generated Cargo metadata,
lockfile, manifest, README, and single source file. It is not present in the
release allowlist.

Two tag namespaces:

| Tag | Workflow | Publishes |
|---|---|---|
| `v*` | `publish.yml` | crates.io, the lockstep family |
| `rpptx-v*` | none until development is complete | no publication |
| `py-v*` | `wheels.yml` | PyPI via OIDC trusted publishing |

Wheels are separate so a Rust patch release does not rebuild twelve wheels, and
a binding-only fix does not force a crates.io release.

## Release process

The unsafe `scripts/release.sh` is deleted. Version changes are targeted F-ID
edits to `[workspace.package]`, the internal pins in
`[workspace.dependencies]`, and `Cargo.lock`. They are reviewed before a tag is
possible and never rewrite README prose by pattern.

`/release vX.Y.Z` is the only command allowed to create or push a `v*` release
tag or start crates.io publication. It requires a clean sprint branch, a full
verification and clean sprint review recorded at the exact HEAD, passing
package dry-runs, an absent local and remote tag, and a separate final approval
immediately before the push. `/close-sprint` remains the only command allowed
to merge `main` or create an `sNN` tag.

The tag starts `publish.yml`. Its Linux runner reproduces the deterministic hash
baseline before crates.io publication begins. Publication succeeds only after
all seven current crates and the GitHub release are externally verified.
`rdocx-wasm` inherits the workspace version but stays `publish = false` because
its distribution path is npm.

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

**A dedicated Presentation fidelity job** fetches the pinned 50-deck corpus,
installs LibreOffice and Poppler, and runs `scripts/pptx_ssim_harness.py
--check` on macOS. The harness rejects any LibreOffice version other than
26.2.5.2 build `cd7284b4cbbfeb507e630c1aac019f4157393acb` and any pdftoppm
version other than 26.01.0 before rendering begins. This turns a package-manager
upgrade into an explicit pin review rather than an unexplained score delta. The
job records the 0.95 SSIM on 80 percent trend reference. It fails on incomplete
corpus coverage, renderer or oracle failure, missing output, dimension mismatch,
or a dropped bounded shape, but not solely on a missed SSIM trend. The job
retains the gate JSON, render manifest, and per-slide score TSV as its evidence
artifact.

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
