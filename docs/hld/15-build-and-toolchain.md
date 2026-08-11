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

Normal `oxml-layout` construction may load system fonts. System fonts differ
between machines and between a developer's machine and a CI runner.

**This makes any recorded render baseline unreproducible**, which would poison
both the hash harness and the SSIM gate: a baseline recorded locally would
mismatch CI in a way indistinguishable from a real regression.

`oxml-layout` provides an explicit deterministic mode:

```rust
impl FontManager {
    /// Bundled fonts only. System font loading is bypassed entirely.
    /// Rendering is then reproducible across machines and platforms.
    pub fn new_deterministic() -> Result<Self>;
}
```

The constructor starts with a fresh font database and loads only the checked-in
bundled font bytes. It never calls system-font discovery. Document-embedded
fonts remain explicit layout inputs, so they are deterministic too.

`Engine::new_deterministic()` and `layout_document_deterministic()` carry that
database through layout. The public facade exposes
`Document::render_page_to_png_deterministic()` and
`Document::to_pdf_deterministic()`. Both reuse the separate cached
bundled-font-only layout. The PDF facade passes that layout directly to
`oxml_pdf::render_to_pdf`, which gives the golden-PNG gate deterministic PDF
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
| `oxml-layout` | `system-fonts` | on | Off for wasm, where `fontconfig` will not build |
| `rpptx` | `default-template` | on | The bundled `default.pptx` |
| `rpptx` | `render` | on | Pulls in `rpptx-render` and `oxml-pdf` |
| `rdocx-py`, `rpptx-py` | `extension-module` | off | Must stay off for `cargo test` |

`fontdb`'s `fontconfig` feature is enabled workspace-wide today. It does nothing
useful on musl or Windows and must be gated per-target for wheel builds.

## Packaging

`oxml-layout` packages its source and bundled font assets through an explicit
manifest inventory:

```toml
[package]
include = [
    "src/**/*",
    "fonts/*.ttf",
    "fonts/LICENSE-*",
    "fonts/NOTICE-*",
]
```

The dedicated package CI job compares `cargo package -p oxml-layout --list`
against all 20 TTFs, the three family licence files, and the Caladea notice. It
then runs verified packaging without `--no-verify` and rejects a missing
archive or one larger than the crates.io 10 MiB limit. `oxml-layout` is a
published 0.1.2 package, while the release workflow remains the authority for
every later publication.

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

The fourteen crates.io names in this graph are reserved under the owner
`mantissaman`: `oxml-core`, `oxml-opc`, `oxml-media`,
`oxml-drawing`, `oxml-layout`, `oxml-pdf`, `oxml-sml`, `oxml-cli-support`,
`rpptx-oxml`, `rpptx-layout`, `rpptx-render`, `rpptx-chart`, `rpptx`, and
`rpptx-cli`. The unimplemented `oxml-cli-support` and `rpptx-cli` entries
remain dependency-free 0.0.0 placeholders. The 12 implemented packages use
the reviewed release path described below.

`oxml-py-support`, `rpptx-py`, and `rpptx-wasm` are not reserved on crates.io.
The binding crates are not published there, and the WASM packages use the npm
publication path.

All 12 implemented shared and PowerPoint packages are published at the common
incubating version 0.1.2. They are
`oxml-core`, `oxml-opc`, `oxml-media`, `oxml-layout`, `oxml-drawing`,
`oxml-pdf`, `oxml-sml`, `rpptx-oxml`, `rpptx-chart`, `rpptx-layout`,
`rpptx-render`, and `rpptx`. The reviewed `rpptx-v0.1.2` release activated that
exact allowlist after its separate final approval. Manifest eligibility alone
does not authorize any later publication.

`publish.yml` accepts stable `v*` and incubating `rpptx-v*` tags. Before either
real allowlist it reproduces the hash harness, runs the self-contained
incubating metadata regression to require the exact versions, pins, lockfile
entries, and non-empty package descriptions without external development
tools, and runs
`cargo publish --workspace --dry-run` with an exact local source patch for each
member of the 19-package publishable union. Cargo rewrites packaged path
dependencies to the registry, so the patches keep verification on the reviewed
workspace graph before those versions exist there. They do not enter generated
archives and the dry run uploads nothing. The stable path then publishes only
the seven released rdocx packages in dependency order. The incubating path
publishes only the 12 candidates above in dependency order. Every real command
keeps archive verification enabled. Registry waits separate dependency layers,
and authentication, network, compilation and duplicate-version failures fail
the job.

The generated archives remain subject to the crates.io 10 MiB ceiling.
`oxml-layout` contains all 20 bundled fonts and their required legal files, and
`rpptx` contains `assets/default.pptx`. No binding or WASM package is in either
crates.io allowlist.

Two tag namespaces:

| Tag | Workflow | Publishes |
|---|---|---|
| `v*` | `publish.yml` | crates.io, the exact seven-package stable family |
| `rpptx-v*` | `publish.yml` | crates.io, the exact 12-package incubating family |
| `py-v*` | `wheels.yml` | PyPI via OIDC trusted publishing |

Wheels are separate so a Rust patch release does not rebuild twelve wheels, and
a binding-only fix does not force a crates.io release.

## Release process

The unsafe `scripts/release.sh` is deleted. Version changes are targeted F-ID
edits to `[workspace.package]`, the internal pins in
`[workspace.dependencies]`, and `Cargo.lock`. They are reviewed before a tag is
possible and never rewrite README prose by pattern.

`cargo-release` preparation is configured in Cargo metadata. The ten packages
that inherit `[workspace.package].version`, including the unpublished
`rdocx-wasm`, `rdocx-py`, and `oxml-py-support` packages, use cargo-release's
effective `workspace` shared-version group and the `v{{version}}` tag template.
The exact published stable family remains the seven packages listed above.
The 12 implemented `oxml-*` and `rpptx*` packages are prepared at explicit
version 0.1.2, use the named `incubating` group, and carry the
`rpptx-v{{version}}` template.
Workspace settings consolidate the preparation commit, upgrade internal
dependency requirements, and retain archive verification. Publishing, tag
creation, and pushing are disabled, and no README replacement is configured.
Preparation therefore changes only the selected manifests and `Cargo.lock`.
External release actions remain owned by `/release`.

`/release {vX.Y.Z | rpptx-vX.Y.Z}` is the only command allowed to create or push
either crates.io release tag or start crates.io publication. It selects exactly
one namespace. The stable path validates the workspace version, its internal
pins, and the exact seven-package stable set. The incubating path validates the
common explicit version, workspace pins, and the exact 12-package incubating
set.

Both paths require a clean sprint branch, full verification and a clean sprint
review recorded at the exact HEAD, a workspace dry run containing exactly the
19-package union and its exact local patch set, archives below 10 MiB with
required assets, an absent local and remote requested tag, and a separate final
approval immediately before the first mutation. `/release` pushes only the
requested tag. `/close-sprint` remains the only command allowed to merge
`main` or create an `sNN` tag.

The requested tag starts `publish.yml`. Its Linux runner reproduces the
deterministic hash baseline, release metadata check, and full workspace dry run
before crates.io publication begins. Success requires every package in the
selected family to report the requested version and expected owner, plus a
matching GitHub release targeting the reviewed SHA. `rdocx-wasm` inherits the
stable workspace version but stays `publish = false` because its distribution
path is npm.

The Python package version tracks the Rust train through a
`pre-release-replacements` entry so the wheel version and the crate version
cannot diverge.

## CI job matrix

Listed in `12-testing-strategy.md`. The matrix carries these
repository-specific gates:

**A dedicated `oxml-layout` package job.** It checks the exact bundled font and
legal-file inventory, builds and verifies the generated archive, and enforces
the crates.io 10 MiB limit.

**`--exclude rdocx-py --exclude rpptx-py` on every `--all-features` job.**
`pyo3/extension-module` tells the linker the Python symbols come from the host
interpreter, which is false for a test binary. On Linux this is an
unresolved-symbol link failure that is easy to misdiagnose as something else.

**A dedicated no-default-features job.** It runs `cargo test -p oxml-layout
--no-default-features`, which exercises the font-isolation path used by WASM.

**A `wasm32-unknown-unknown` check job.** It installs the target and checks the
existing `rdocx-wasm` crate. The future `rpptx-wasm` package remains deferred
to F-138.

**A prose and generated-skill job.** It runs `scripts/prose_check.py` and
`scripts/sync_agent_skills.py --check` as separate steps, so voice-rule or
adapter drift fails before integration.

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
