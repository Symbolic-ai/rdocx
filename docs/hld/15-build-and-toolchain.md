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
bundled font bytes. It never calls system-font discovery or clones the normal
process snapshot. Document-embedded fonts remain explicit layout inputs, so
they are deterministic too. Caller-font construction starts from an empty
database and likewise cannot observe bundled or system fonts.

`Engine::new_deterministic()` and `layout_document_deterministic()` carry that
database through layout. The public facade exposes
`Document::render_page_to_png_deterministic()`,
`Document::to_pdf_deterministic()`, and deterministic SVG page export. All
three reuse the separate cached bundled-font-only layout. The PDF facade passes
that layout directly to
`oxml_pdf::render_to_pdf`, which gives the golden-PNG gate deterministic PDF
input without changing the normal PDF API. Existing constructors and rendering
methods still load system fonts for library users.

The hash harness, golden-PNG gate and SSIM harness use the deterministic path.
The normal rendering API still discovers system fonts, but now captures the
bundled plus system face table once per process. Installing, removing, or
replacing system fonts requires a process restart. File-backed font bytes use a
separate bounded process cache keyed by canonical file identity, so faces at
different TTC indices share one byte buffer. Both process caches are compiled
only with `system-fonts`, and poisoned file-cache locks recover by rebuilding
the requested entry.

Tagged Word PDFs use the same deterministic layout and writer path. Structure
node references, page-local MCIDs, parent-tree keys, conditional PDF/UA
metadata, and XMP bytes are derived only from ordered layout input. A
`.notdef` glyph suppresses the PDF/UA claim. These outputs add no clock, random
identifier, host lookup, or system-font dependency. Untagged Presentation
layouts keep the existing writer path.

PDF/A-2b and PDF/A-3b use the same deterministic layout boundary. The writer
adds no clock, random source, network lookup, or host colour service. Its fixed
XMP and file identifiers depend only on the selected profile and ordered
layout metadata. Ordinary PDF output remains on the existing byte-compatible
path.

Reusable managers bound shaping to 2,048 entries and 16 MiB, file bytes to 256
entries and 128 MiB, and coverage, resolution, and paragraph traces by explicit
entry ceilings. The reusable Word engine bounds both pending and published
paragraph entries at 256 entries and 16 MiB with retained-capacity accounting.
These caches add no feature flag or dependency and remain available in the
default-off graphs without enabling host discovery.

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
| `oxml-opc` | `agile-encryption` | off | Adds native read and fixed-profile write support for password-protected OOXML packages |
| `rdocx` | `agile-encryption` | off | Forwards encrypted native document opens and saves to `oxml-opc` |
| `oxml-opc` | `digital-signatures` | off | Adds OPC signature creation, verification, and coverage reports |
| `rdocx` | `digital-signatures` | off | Forwards native signature creation and verification to `oxml-opc` |
| `oxml-layout` | `system-fonts` | on | Off for wasm, where `fontconfig` will not build |
| `rdocx-layout` | `system-fonts` | on | Forwards host discovery to `oxml-layout` |
| `rdocx` | `system-fonts` | on | Forwards through the complete native layout graph |
| `rpptx-render` | `system-fonts` | on | Preserves host discovery for normal presentation rendering |
| `rpptx` | `system-fonts` | on | Preserves native presentation font resolution |
| `rpptx` | `default-template` | on | The bundled `default.pptx` |
| `rpptx` | `render` | on | Pulls in `rpptx-render` and `oxml-pdf` |
| `rpptx-wasm` | `render` | off | Adds `toPdf` through the deterministic facade renderer |
| `rdocx-py`, `rpptx-py` | `extension-module` | off | Must stay off for `cargo test` |

The workspace dependency entries for `oxml-layout`, `oxml-pdf`,
`rdocx-layout`, and `rdocx` are default-off so a member can select the exact
graph. Direct native `rdocx` and `rdocx-layout` builds retain default-on system
fonts. The CLI and Python binding opt in explicitly, while `rdocx-wasm` does
not. Its generated `toPdf` method calls the normal `Document::to_pdf` facade,
which therefore uses document-embedded and bundled fonts without host font
discovery in that graph. Native `rpptx`, `rpptx-render`, and the presentation
Python binding retain system fonts through the same explicit forwarding
pattern. Bundled font bytes remain available in both modes. The Python, WASM,
and CLI manifests do not opt in to `agile-encryption`.
They also do not opt in to `digital-signatures`.

Raster encoding dependencies are intentionally narrow. `jpeg-encoder` and
`tiff` are direct `oxml-pdf` dependencies with default features disabled,
because the format-neutral backend is their named consumer. The selected
versions pass the repository license and advisory policy, compile for the WASM
target graph, and do not add a format-family dependency edge.

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

The external PowerPoint and Word corpora remain outside every published crate
under the ignored `corpus/` directory. Their tracked manifests pin immutable
source URLs and SHA-256 values. The Word manifest also pins one of each required
category plus the reviewed `Apache-2.0` or `MIT` licence identity and immutable
licence URL.

A separate crate-local packaging rule applies to
`crates/rpptx/assets/default.pptx`. **An asset must live under its own crate's
directory**: a workspace-root `assets/` compiles locally but is not collected
into the published tarball.

Public API changes in published crates run the verified workspace packaging
dry run against local patches for every internal crate. Every generated
archive must remain below the crates.io 10 MiB limit. Default-off features are
present in package metadata even when archive verification builds the default
graph.

The format-neutral semantic types are part of the published `oxml-layout`
source inventory. The tagged writer implementation, including
`src/structure.rs`, is part of the published `oxml-pdf` source inventory. No
runtime oracle, installer, validation profile, or generated PDF is packaged in
either crate.

`oxml-pdf` also packages `assets/sRGB2014.icc` and
`assets/LICENSE-sRGB2014`. The 3,024-byte profile comes from the International
Color Consortium registry and has SHA-256
`384b832de3412066743b52a75ee906b6fb9fb8d9e09e936fc2c43223815c6e0a`.
The adjacent legal file records the ICC distribution terms, source, retrieval
date, byte size, and digest. The verified package inventory must contain both
files and the shared raster encoder source graph while every generated archive
stays below the crates.io 10 MiB ceiling.

The publishable `rpptx-cli` binary contains nine commands. Its `thumbnail`
command uses the deterministic presentation renderer, and its `outline`
command depends only on facade traversal. The package dry run and archive-size
gate therefore cover the complete command surface without adding runtime
assets to the CLI crate.

Every bundled font family has its licence under the crate's `fonts/` directory.
Caladea ships with the full Apache License 2.0 text in `LICENSE-Caladea` and its
copyright, trademark and designer attribution in `NOTICE-Caladea`. The
`bundled_fonts.rs` module documentation identifies Caladea as Apache-2.0 and
the Carlito and Liberation families as SIL Open Font License 1.1.

## Publishing

The dependency order grows to roughly twenty crates:

```
oxml-core -> oxml-opc -> oxml-media -> oxml-drawing -> oxml-layout -> oxml-pdf
  -> oxml-sml -> oxml-cli-support -> oxml-chart
  -> rdocx-oxml -> rdocx-layout -> rdocx-html -> rdocx -> rdocx-cli
  -> rpptx-oxml -> rpptx-chart -> rpptx-layout -> rpptx-render -> rpptx -> rpptx-cli
```

The fifteen crates.io candidates in this graph are owned by `mantissaman`:
`oxml-core`, `oxml-opc`, `oxml-media`,
`oxml-drawing`, `oxml-layout`, `oxml-pdf`, `oxml-sml`, `oxml-cli-support`,
`oxml-chart`, `rpptx-oxml`, `rpptx-layout`, `rpptx-render`, `rpptx-chart`,
`rpptx`, and `rpptx-cli`. All 15 implemented packages use the reviewed release
path described below. The earlier 12-package family is published at 0.1.2.
`oxml-cli-support` and `rpptx-cli` are publishable but remain unpublished at
that version. The original 14-package family is published at the immutable
0.1.3 and 0.2.0 boundaries.

`oxml-py-support`, `rpptx-py`, and `rpptx-wasm` are not reserved on crates.io.
The binding crates are not published there. `rpptx-wasm` is an implemented
workspace crate with no crates.io publication path. Its reviewed npm surface is
the local `@tensorbee/rpptx-wasm` bundler tarball. Registry publication remains
unconfigured and unauthorized.

The exact incubating crates.io allowlist now contains 15 implemented shared
and PowerPoint packages. They are
`oxml-core`, `oxml-opc`, `oxml-media`, `oxml-layout`, `oxml-drawing`,
`oxml-pdf`, `oxml-sml`, `oxml-cli-support`, `oxml-chart`, `rpptx-oxml`, `rpptx-chart`,
`rpptx-layout`, `rpptx-render`, `rpptx`, and `rpptx-cli`. All 15 are published
at 0.6.0 from the annotated `rpptx-v0.6.0` tag at reviewed SHA
`55fb2f54caf91d7dedc8936b4c7b116354590628`. The earlier 0.5.0 and 0.4.0
registry releases remain available, and no existing version or tag was moved. Manifest
eligibility and allowlist membership do not authorize a later publication
without a separately approved `/release` invocation at the exact reviewed
SHA. The unpublished `rpptx-wasm` preparation member is also at 0.6.0 but has
no crates.io publication path.

`publish.yml` accepts stable `v*` and incubating `rpptx-v*` tags. Before either
real allowlist it reproduces the hash harness and runs self-contained stable
and incubating metadata regressions without external development tools. The
stable regression requires published workspace version 0.10.1, nine internal
pins, eleven inherited lockfile packages, two Python project versions, unpublished
`rdocx-wasm`, stable README requirements, and the exact seven-package crates.io
set. The incubating regression requires the exact 0.6.0 versions, pins,
lockfile entries, publication flags, and non-empty package descriptions.

**The same regressions run in the canonical local gate.** `/verify` step 6 runs
`python3 -m unittest scripts.test_sprint_workflow`, the module holding both
preflights and the pinned-toolchain assertions, so a version carrier that moves
without its assertion moving with it fails before the sprint closes rather than
on the tag. Without that step the preflights run for the first time at
publication, which is what S42 demonstrated when F-X022 passed the entire local
gate and still left the incubating preflight and the `ci.yml` WASM literal
asserting the previous version.

The pull-request CI workflow runs the same complete module in its dedicated
`release-regressions` job. The job has no condition or failure-tolerant path,
and checkout plus a locked cargo-release 1.1.3 installation precede the exact
whole-module command. This keeps both release family preflights and future
release-contract regressions in the ordinary CI gate with the external command
their stable-family checks require.

Every stable or incubating tag also requires one reviewed `CHANGELOG.md`
section whose second-level heading is the exact tag. The required ordered
subsections are Highlights, Added, Fixed, Compatibility, and Contributors.
The deterministic workflow CLI checks that contract and renders only the
reviewed body. Both modes are read-only. `publish.yml` runs the check before
either crates.io allowlist, stores one render in runner-temporary storage, and
byte-compares a fresh render with that artifact immediately before passing it
to `gh release create --notes-file`. Generated GitHub notes are not a release
source.

The workflow then runs
`cargo publish --workspace --dry-run` with an exact local source patch for each
member of the 22-package publishable union. Cargo rewrites packaged path
dependencies to the registry, so the patches keep verification on the reviewed
workspace graph before those versions exist there. They do not enter generated
archives and the dry run uploads nothing. The stable path then publishes only
the seven released rdocx packages in dependency order. The incubating path
publishes only the 15 candidates above in dependency order. Every real command
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
| `rpptx-v*` | `publish.yml` | crates.io, the exact 15-package incubating family |
| `py-v*` | `wheels.yml` | PyPI via OIDC trusted publishing |

Wheels are separate so a Rust patch release does not rebuild twelve wheels, and
a binding-only fix does not force a crates.io release. `wheels.yml` builds and
uploads each of the twelve cp39-abi3 wheels and both source distributions
without publication authority. Its separate publish job depends on the whole
artifact graph, checks the exact artifact counts, binds to the `pypi`
environment, and receives `id-token: write` only for a `py-v*` tag event.
Manual dispatch exercises the build graph without publishing. All actions and
the maturin version are pinned, and no long-lived PyPI secret is present.

## Release process

The unsafe `scripts/release.sh` is deleted. Version changes are targeted F-ID
edits to `[workspace.package]`, the internal pins in
`[workspace.dependencies]`, and `Cargo.lock`. They are reviewed before a tag is
possible and never rewrite README prose by pattern.

`cargo-release` preparation is configured in Cargo metadata. The eleven packages
that inherit `[workspace.package].version`, including the unpublished
`rdocx-wasm`, `rdocx-py`, `rpptx-py`, and `oxml-py-support` packages, use
cargo-release's effective `workspace` shared-version group and the
`v{{version}}` tag template. That shared-version group and its two Python
project versions and rdocx WASM contract literals are at 0.10.1. The exact
seven-package stable family is published from the annotated `v0.10.1` tag at
reviewed SHA `ae0dcb162a7805e59e5890464b226765645ad547`.
The immutable v0.10.0 attempt published only `rdocx-opc` and `rdocx-oxml`
before package verification failed. The remaining five packages and GitHub
release were not published at that version. The last complete stable family is
0.10.1. Earlier immutable
registry releases remain available. No binding, WASM, Python, npm, or
incubating package gained publication authority from the stable release.
The 16 implemented `oxml-*` and `rpptx*` package manifests use explicit version
0.6.0, the named `incubating` group, and the `rpptx-v{{version}}` template. The
exact 15-package crates.io family listed above is published from the annotated
`rpptx-v0.6.0` tag at reviewed SHA
`55fb2f54caf91d7dedc8936b4c7b116354590628`. The preparation group also
contains unpublished `rpptx-wasm`, while the crates.io allowlist remains
exactly 15 packages. Earlier immutable registry releases remain available.
Workspace settings consolidate the preparation commit, upgrade internal
dependency requirements, and retain archive verification. Publishing, tag
creation, and pushing are disabled, and no README replacement is configured.
Preparation therefore changes only the selected manifests and `Cargo.lock`.
External release actions remain owned by `/release`.

`/release {vX.Y.Z | rpptx-vX.Y.Z}` is the only command allowed to create or push
either crates.io release tag or start crates.io publication. It selects exactly
one namespace. The stable path validates the workspace version, its internal
pins, and the exact seven-package stable set. The incubating path validates the
common explicit version, workspace pins, and the exact 15-package incubating
set.

`/release-notes TAG` is the deliberate preparation ceremony for the same two
namespaces. It derives human-written highlights, additions, fixes,
compatibility guidance, and contributor credit from reviewed repository
evidence, then updates the exact changelog section for review with the code.
`/release` renders and inspects that section at the reviewed SHA before its
separate final approval. After publication it requires the GitHub release body
to equal the same rendered bytes.

The ceremony also builds one selected-family inventory of every included
GitHub issue and pull request. The reviewed notes link each record directly and
credit its authenticated external reporter or contributor with the specific
outcome that landed, including hardened equivalents of unmerged reference
implementations. Before approval, `/release` reports the complete inventory and
the planned record-specific comments. After successful registry publication
and exact release-body verification, it posts those comments with the final tag
and GitHub release link, then retains their URLs in the release evidence. A
missing link, credit, inventory entry, or notification blocks completion of the
release F-ID.

Both paths require a clean sprint branch, full verification and a clean sprint
review recorded at the exact HEAD, a workspace dry run containing exactly the
22-package union and its exact local patch set, archives below 10 MiB with
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

**Path-filtered expensive jobs with one aggregate gate.** A `changes` job uses
`dorny/paths-filter` v4.0.3 at immutable reviewed commit
`ceb8a2b8f2d89434be7ff52d3de7ec3738c5cc9d`. Its inline filters cover the full
inputs of `test`, `msrv`, `wasm`, `python-bindings`,
`presentation-fidelity`, `hash-harness`, `supply-chain`, and `prose`. Every
filter also covers the CI workflow itself. The detector alone receives
`pull-requests: read` in addition to repository content read.

The `ci-gate` job always runs and depends on change detection plus every
filtered job. It fails unless selected jobs succeeded and unselected jobs were
skipped. This includes failure, cancellation, unexpected-skip, and
change-detector failure paths. A documentation-only change runs prose while
the filtered product jobs skip, yet the same stable aggregate gate reports.
The scheduled path skips change detection and requires supply-chain success.
Unfiltered jobs retain their ordinary triggers.

**A dedicated `oxml-layout` package job.** It checks the exact bundled font and
legal-file inventory, builds and verifies the generated archive, and enforces
the crates.io 10 MiB limit.

**`--exclude rdocx-py --exclude rpptx-py` on every `--all-features` job.**
`pyo3/extension-module` tells the linker the Python symbols come from the host
interpreter, which is false for a test binary. On Linux this is an
unresolved-symbol link failure that is easy to misdiagnose as something else.

**A dedicated no-default-features job.** It runs `cargo test -p oxml-layout
--no-default-features`, which exercises the font-isolation path used by WASM.

**The golden-PNG gate in the test job.** After the full workspace suite, the
same Ubuntu 24.04 job runs `python3 scripts/golden_png_harness.py --check` with
the Poppler 26.01.0 installation already on `PATH`. The step is unconditional
and propagates a decoded-pixel mismatch as a CI failure.

**Workspace package READMEs in the docs job.** Every one of the 27 workspace
packages explicitly declares one distinct README. The root file is the
high-level `rdocx` guide. The other 26 packages use focused crate-local files.
The documents describe purpose, direct use, neighbouring package boundaries,
publication status, and an example suited to the actual consumer surface. The
three deprecated shims direct new consumers to `oxml-opc`, `oxml-pdf`, and
`oxml-chart`.

After the workspace documentation build, `scripts/readme_doctests.py` checks
the exact 27-package inventory, validates Rust, shell, Python, and JavaScript
snippets, and compiles 27 Rust examples across the 21 Rust-library READMEs. It
discovers each primary and companion rlib from one Cargo build graph and passes
them to rustdoc with the repository edition, dependency search path, matching
external crate bindings, and warnings denied.
The same runner is part of canonical non-fast verification. It creates each of
the 22 publishable archives, requires exactly one packaged README, and
byte-compares it with the declared source. Version, tag, publication, and
release-family metadata remain unchanged.

**A WASM target and Node job.** It installs the `wasm32-unknown-unknown` target,
uses exact Node 24.11.1 and wasm-pack 0.15.0, and checks both facade-backed WASM
crates with the locked workspace graph. It then runs both packages' inline Node
regressions. It installs the official Binaryen version 125 Linux archive after
checking exact SHA-256
`7c3bc16599c8274a04d34a504fe4be2047884f900e0e2da2f6fb9cd667183be4`,
places its `wasm-opt` on `PATH`, and verifies the exact official identity
`wasm-opt version 125 (version_125)`.

Both WASM manifests use release optimization arguments `-Oz`,
`--enable-bulk-memory`, and `--enable-nontrapping-float-to-int`. The job builds
the exact scoped release bundler packages with locked dependencies, packs each
one locally, installs it into a separate fresh consumer with an isolated cache
and scripts disabled, and checks exact identity, WASM, JavaScript glue, public
TypeScript declarations, and imports. The package gate grants no registry
authentication, token, OIDC, publication, release, or tag authority. The
document suite also requires generated `toPdf` to return a complete PDF with an
embedded bundled Carlito font. Checkout, setup-node, the Rust toolchain, and the
Rust cache use reviewed full commit SHAs. The presentation render-profile and
optimized-size gates remain local.

**A dedicated Python artifact workflow.** Its product matrix is the Cartesian
product of `rdocx` and `rpptx` with manylinux_2_28 x86_64 and aarch64,
musllinux_1_2 x86_64, macOS x86_64 and arm64, and Windows x86_64. A second
two-package job builds the source distributions. Native cells install wheels
into fresh environments and run the compatible pytest, exact mypy, and
stubtest gates. Each musllinux cell performs a fresh Alpine install and runs
the applicable package parity suite.
The Poppler-versioned binding render gate stays in its pinned environment
rather than running on generic wheel hosts.

**A pull-request Python bindings job.** It runs on macOS 26 with one matrix row
for each Python package. Every row creates an isolated Python 3.12.9 environment,
installs exact maturin, pytest, and package-oracle versions, builds the current
extension with `maturin develop --locked`, and runs the package's complete
pytest directory. The Poppler installation supplies the reviewed 26.01.0 tools
asserted by the rdocx rendering suite. Build and test failures propagate
directly, with no failure-tolerant condition, inherited pytest override, or
fallback. The top-level pull-request trigger is operative and the job condition
uses only the change detector's `python_bindings` output. Workflow root
permissions are repository content read, and only the detector adds pull-request
metadata read. No job grants an OIDC token. Checkout v6.0.2, setup-python
v6.2.0, rust-cache v2.9.1, and the selected stable rust-toolchain revision use
reviewed full commit SHAs and exact input maps.

**One checksum-pinned Poppler installer.**
`scripts/install_pinned_poppler.py` downloads the official 26.01.0 source
archive, verifies SHA-256
`1cb944a4b88847f5fb6551683bc799db59f04990f5d8be07aba2acbf38601089`,
and builds only `pdftoppm`, `pdfinfo`, and `pdftotext` in an isolated directory.
It caps the download at 8 MiB and streams at most 2,048 safe archive members
with at most 64 MiB of expanded content. A populated prefix fails closed, so a
successful invocation cannot substitute unrelated binaries that print the
right version. All three finished tools must report exact 26.01.0 identities.
Test, MSRV, both Python binding rows, and Presentation fidelity use this single
unconditional step before any oracle-dependent command. Package managers may
install build prerequisites but do not install Poppler itself.

**Pinned corpus-test runtime.** Test and MSRV install uv 0.10.2 with official
`astral-sh/setup-uv` commit
`20cfd1bf945f4377ade1205e4dbc17946fc9a30d`. Each job disables the action cache,
uses only its runner-temporary uv cache, fetches and verifies both pinned
corpora, and runs the broad workspace suite with `RUST_MIN_STACK=8388608`. The
pin makes the python-pptx oracle executable available on a clean Ubuntu host.
The stack budget is scoped to these two corpus-heavy jobs and does not alter
product runtime behavior.

The same two clean Ubuntu 24.04 jobs install LibreOffice 26.2.5.2 from the official
Linux x86-64 Debian archive before the workspace suite. The archive SHA-256 is
`2f03bfb2ac9f33ea7c77331b4b7a23300fb0ed7443566046bf8b5bc51c1bed1e`.
The installer streams under fixed download, member, and expanded-byte bounds,
rejects unsafe entries and any populated `/opt/libreoffice26.2` prefix, then
requires exact identity
`LibreOffice 26.2.5.2 cd7284b4cbbfeb507e630c1aac019f4157393acb`.
It installs the explicit Ubuntu NSS, NSPR, D-Bus, Cairo, GLib, X11, CUPS,
font, and Kerberos runtime-library set needed by that official build. This
makes the unconditional `oxml-chart` viewer tests and the `rdocx` ODT
structural differential self-contained without changing the separate macOS
Presentation fidelity setup. The ODT gate uses an isolated LibreOffice profile
and rejects any runtime identity other than the exact pinned build.

**A prose and generated-skill job.** It runs `scripts/prose_check.py` and
`scripts/sync_agent_skills.py --check` as separate steps, so voice-rule or
adapter drift fails before integration.

**A dedicated release regression job.** It runs
`python3 -m unittest scripts.test_sprint_workflow` after checkout, without a job
condition, successful fallback or `continue-on-error`. The complete module is
the pull-request gate for release-family version carriers and their workflow
contracts.

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

`deny.toml` keeps the unused ZIP codec bans (`zstd`, `bzip2`, `lzma-rust2`, and
`ppmd-rust`) across the workspace, since `zip` is in every graph through
`oxml-opc`. AES, CBC, CFB, HMAC, SHA, and operating-system randomness
dependencies have the named
default-off `oxml-opc/agile-encryption` consumer and do not enter ordinary,
Python, WASM, or CLI graphs.
Ring, SHA-256, and X.509 parsing have the named default-off
`oxml-opc/digital-signatures` consumer. They do not enter ordinary, Python,
WASM, or CLI graphs. That feature also uses `base64`, whose ordinary runtime
consumer and binding-graph consequences are described below. The creator uses
operating-system randomness for strict RSA-SHA256 signing and requires a
matching embedded public key. The verifier authenticates with that embedded
key. Certificate-chain trust requires caller policy and no ambient trust-store
dependency is part of the workspace graph.

PDF/A conformance adds no crate dependency. The sRGB2014 bytes are a
crate-local compile-time asset, and veraPDF remains external test
infrastructure rather than production logic.

The single advisory exception, an unmaintained transitive font dependency,
carries its exit route in a comment. Keep that discipline: an ignore without a
stated exit route is a permanent ignore.

New dependencies are added only with a named consumer. The workspace already
minimises feature sets deliberately, and the comments in the root manifest
explaining why `zip` and `fontdb` are trimmed should be preserved rather than
regenerated.

The private native Word SVG renderer is the direct runtime consumer of
`base64`, which embeds exact layout font bytes and page image bytes into
self-contained data URLs. No `oxml-*`, Presentation, Python, WASM, or CLI crate
adds a direct edge. The Python, WASM, and CLI graphs inherit `base64`
transitively through their ordinary `rdocx` dependency. Exact resvg 0.48.1 is
an `rdocx` development dependency for the 150 dpi SSIM oracle only. It receives
explicit layout fonts, never system fonts, and does not enter the runtime graph
or generated `rdocx` archive.

`scraper` 0.27 has one direct named consumer, the private inbound HTML importer
inside `rdocx`. Default features are disabled and only `errors` is enabled so
HTML5 parser repair diagnostics remain available without an unrelated feature
surface. No `oxml-*`, `rdocx-html`, Presentation, Python, WASM, or CLI crate
declares the dependency directly. The complete facade graph passes Rust 1.93,
both wasm32 checks, the workspace dependency policy, and the `rdocx` package
dry run. The packaged archive remains below the 10 MiB ceiling.

The existing workspace `zip` 8.1 dependency has direct named consumers in the
private ODT reader and writer inside `rdocx`. It retains the workspace's
disabled default features and enables only `deflate-flate2-zlib-rs`. Deflate64
and clock-backed timestamp support remain disabled. No new external package or
`oxml-*` dependency edge is added. The reader validates bounded archive
metadata before parsing. The writer fixes entry order, compression,
permissions, the 1980-01-01 timestamp, and other metadata while bounding XML,
media, entries, and total retained output. Neither direction reuses
`OpcPackage` for a non-OPC format.

The private EPUB writer inside `rdocx` is another named consumer of that same
workspace `zip` dependency. It adds no runtime package. EPUBCheck 5.3.0 remains
external development and CI validation infrastructure. The reviewed W3C
release ZIP SHA-256 is
`6c07e68584b2e2ce2f89fe06e1246dfead3eb36b46b340e7d93524f29dcff6c5`,
and the extracted validator JAR SHA-256 is
`f7f96617c929371821609b88c8484d6dc9f24fe916499863c46094c5fb778a65`.
The tracked test job downloads that exact release, verifies both identities,
exports the verified JAR path, and runs the source-built EPUB oracle as a
required CI step before the full workspace suite. The validator is not included
in any crate archive.
