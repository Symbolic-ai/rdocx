# 12, Testing strategy

## Test taxonomy

Six categories. Every story's design plan picks the applicable ones and names
exactly one as its test gate.

| Category | Purpose | Where |
|---|---|---|
| `unit` | Pure logic, no I/O | `crates/<crate>/src/*.rs` under `#[cfg(test)]` |
| `integration` | Multi-crate behaviour through the public API | `crates/<crate>/tests/` |
| `regression` | Locks down one named past failure | `crates/rdocx/tests/regression_test.rs` and the rpptx equivalent |
| `round-trip` | Parse, serialise, reparse, compare | corpus-driven |
| `golden` | Byte or pixel comparison against a recorded baseline | the hash harness |
| `differential` | Compared against an external oracle | LibreOffice for renders, python-docx and python-pptx for the bindings |

The existing repository convention is preserved: **no binary fixture files.**
Fixtures are constructed in code, including hand-assembled PNG and JPEG headers
with precomputed CRCs. It keeps the `.crate` payload small and the diffs
readable. The corpus in the next section is the one deliberate exception, and it
lives outside the published crates.

Regression tests are named as sentences describing the failure they prevent, so
a reintroduction is obvious from the test name alone rather than from a diff.
The existing file is the model: `zero_column_tables_do_not_panic`,
`saving_is_reproducible`.

## The hash harness

The single highest-value mechanism in the plan is
`scripts/hash_harness.py --check`. It deletes the expected generated outputs,
runs `generate_all_samples`, and records the flushed `word/document.xml`,
`word/styles.xml`, and `word/numbering.xml` state plus the page-one PNG for each
of the seven samples. PNGs are rendered at 150 dpi through the deterministic
font path.

The sorted `scripts/hash_baseline.json` manifest has 28 entries. Each entry is
either a SHA-256 digest or JSON `null` when an optional XML part is absent.
Check mode reads the manifest without modifying it and reports added, removed,
and changed entries. Baseline writes require `--update --reason <text>`, and an
empty reason is rejected. Generated PNGs remain ignored under `samples/`.

It exists because the extraction changes unit conversion and text-shaping input
types, and both alter output **without failing to compile**. Structural
round-trip tests cannot see that class of defect.

Rules:

- Every PR in M1 through M6 gates on it.
- Baseline updates require a non-empty review reason.
- An intentional behavioural change lands as its own labelled commit with its
  expected delta stated in the message and reviewed.
- An unexplained delta blocks the merge.

## The golden-PNG gate

`python3 scripts/golden_png_harness.py --check` generates deterministic PDFs
for the seven `samples/` documents, rasterises page one at 150 DPI with
`pdftoppm`, and compares decoded RGBA pixels. The rasteriser is test
infrastructure only. Its exact version is printed on every run and recorded in
`scripts/golden_pixel_manifest.json`. The current manifest records
`pdftoppm version 26.01.0`.

Each readable manifest entry contains the page width, height, and SHA-256 digest
of the decoded RGBA buffer. There are no committed PNG fixtures. Check mode
requires identical dimensions and a zero-pixel-difference digest, then reports
the first differing sample precisely. Reviewed updates use `--update --reason
<text>`, and an empty reason is rejected.

The gate deliberately compares pixels rather than PDF bytes. The operator
stream legitimately changes when the per-element Y flip becomes one global
CTM. The reviewed Poppler 26.01.0 baseline includes exactly four
stroke-antialias changes. In `invoice`, pixels `(112, 397)` and `(112, 398)`
swap `fcf5f5ff` and `ffffffff`. In `quote`, pixels `(112, 303)` and
`(112, 304)` swap `f4fafaff` and `ffffffff`. The other five samples remain
exact. This is a baseline, not a tolerance, so check mode still requires exact
equality for all seven buffers. The regression proof runs `--check
--inject-one-pixel <sample>`, copies that generated PNG to a temporary
directory, changes exactly one decoded pixel, and requires check mode to fail
with the sample name.

## The deck corpus

Fifty real `.pptx` files are stored outside the published crates and fetched by
`scripts/fetch_pptx_corpus.py` into the ignored `corpus/pptx` directory. The
tracked manifest pins each URL, producer, relative path, and SHA-256. It
contains 49 Apache POI slideshow test decks at commit
`11ede1db13c554b4341266faeb84e327fc316379` and one public Google Slides export.
`--check` verifies the complete directory without changing it. The set spans
producers because non-Microsoft writers are where parser assumptions break:

- PowerPoint 2016 and Microsoft 365
- Google Slides export
- Keynote export
- LibreOffice Impress
- A multi-master corporate template
- Decks containing SmartArt, charts, embedded video, and ink

The read-facade differential runs `dump_deck` over all fifty decks and compares
its normalized records with python-pptx 1.0.2. The executable test command pins
that exact oracle version with `uv run --with python-pptx==1.0.2` and rejects a
different resolved version. Records cover slide id and name, recursive shape
path and structural kind, ordinary shape text, row-major table text, aggregate
slide text, and optional speaker-note text. Empty python-pptx names and shape
text capability without a stored `p:txBody` are normalized to the facade's
explicit `Option` contract.

Four gates run against it:

1. **DrawingML structural round-trip**: every `a:txBody` and `a:spPr` parses,
   serialises and reparses to a structurally equal value. The pinned corpus has
   6,898 text bodies and 8,643 shape-property elements. This is the carried M7
   exit gate at the first point where the external corpus exists. Every
   `ppt/tableStyles.xml` part also parses, serialises, and reparses through the
   typed table style model while retaining unsupported XML at its boundary.
2. **Raw round-trip**: open and canonically save with every document part
   treated as opaque. Every decompressed part stays byte-identical, while
   content types and relationships stay structurally equal. ZIP metadata and
   compression are not model state. This proves the OPC layer and the corpus
   harness before any PresentationML modelling exists.
3. **Modelled round-trip** (M8 exit): parse and serialise the presentation,
   slide, layout, master, notes slide, notes master, and theme roots. Reparse
   each canonical result and compare it structurally. Build the expected
   package from those exact modelled bytes, retain the original bytes for all
   unmodelled parts, save through deterministic OPC output, reopen, and compare
   content types, relationships, part names, part counts, and every part byte
   against that expectation. The gate requires nonzero corpus coverage for all
   seven root types.
4. **Opens without repair** (M8 and M11): every saved deck opened manually in
   PowerPoint once per milestone. Not automatable, and not skippable.

The M9 resolver gate selects `WithMaster.pptx`, `backgrounds.pptx`,
`placeholder-layout-color.pptx`, and
`bug58144-headers-footers-2007.pptx` for native visual acceptance. Its
repeatable normalized differential also includes `60810.pptx`, whose master
picture appears exactly once on every enabled slide and zero times on its two
`showMasterSp="0"` layouts. The executable oracle pins python-pptx 1.0.2 and
compares ordered source shape kind, bounds and text. Concrete resolver evidence
retains RGBA fills and unsupported diagnostics, including exact cyan `#00FFFF`
for the inherited placeholder run. The one-time PowerPoint record in the
integration test names build 16.104.25121423, the exact original paths and the
clean no-repair verdict.

These automated visual tests use the same external-corpus policy as the other
corpus gates. A missing configured corpus skips them when
`RDOCX_PPTX_CORPUS_REQUIRED` is unset and fails them when it is set. The
one-time native acceptance record does not require the external files to remain
present after review.

## The M11 cross-viewer acceptance gate

The M11 gate uses one deterministic ten-slide deck built from the checked-in
default template by `build_f116_ten_slide_deck` in the existing `rpptx`
integration binary. No generated deck is checked in. The reviewed temporary
candidate is `/private/tmp/rdocx-f116-m11-write-api.pptx`, with SHA-256
`d36da6e8849eabd4487d2572baea19c3716ee7d0fe03aaa4714a28ce3c41de4f`.
Its ordinary `.pptx` and slideshow `.ppsx` forms reopen through the facade,
use the correct main content types, and return no validation issues.

The one deck covers the complete M11 write surface:

| Story | Candidate coverage |
|---|---|
| F-107 | ten slides synthesised from the bundled template |
| F-108 | clean structural validation before and after reopen |
| F-109 | position, size, rotation, name, fill, line, and adjustment mutation |
| F-110 | textbox, preset shape, three connector forms, and group construction |
| F-111 | package-deduplicated pictures with slide-scoped relationships |
| F-112 | paragraphs, bullet, run properties, and direct Latin font |
| F-113 | cells, fill, margins, banding, width, merge, and split |
| F-114 | image-bearing duplication, removal, and final-index move |
| F-115 | slide size, core properties, hidden state, background set and clear, and slideshow save |

Every viewer receives that exact SHA. Microsoft PowerPoint checks its pinned
version, Info.plist build, and AppleScript build before opening, counting ten
slides, and closing without saving. Keynote records a user-confirmed open and
ten-slide inspection against its installed version and bundle build.
LibreOffice runs a headless import and `impress_pdf_Export` with hidden slides
enabled, then `pdfinfo` must report ten pages. Google Slides imports through a
signed-in browser, reports ten slides without a conversion error, and exports
once. Its row records the acceptance date and browser build rather than an
application version. The ignored gate reruns the automatable PowerPoint and
LibreOffice checks and validates all four SHA-bound evidence rows. It does not
replace the Keynote or Google human-action evidence with unsupported UI
automation.

The evidence bound to the reviewed SHA is:

| Viewer | Version or date | Build | Result |
|---|---|---|---|
| Microsoft PowerPoint | 16.104 | Info.plist 16.104.25121423, AppleScript 1214 | clean, opened ten slides and closed without saving |
| Apple Keynote | 14.4 | 7043.0.93 | clean, user-confirmed human-action open of ten slides without a conversion error, then closed |
| Google Slides | accepted 2026-08-09 | Google Chrome 151.0.7922.76, build 7922.76 | clean, saved to Drive, showed slides 1 through 10 without a conversion error, and started one Microsoft PowerPoint download |
| LibreOffice Impress | 26.2.5.2 | cd7284b4cbbfeb507e630c1aac019f4157393acb | clean, headless import and ten-page PDF export |

All four rows record clean observations against the same artifact SHA. The
Keynote row is user-confirmed human-action evidence. The Google Slides row is
bound to the acceptance date and browser build without recording the private
import URL.

## The render fidelity gate

The 50-deck pinned corpus is rendered through bundled fonts at 150 dpi.
LibreOffice 26.2.5.2 with build
`cd7284b4cbbfeb507e630c1aac019f4157393acb` exports PDF through the
`impress_pdf_Export` filter with `ExportHiddenSlides=true`, then pdftoppm
26.01.0 rasterises every page at the same 150 dpi. The hidden-slide option is
part of the asserted command because a default PDF export omits five corpus
slides.

The harness decodes both PNGs through the existing strict decoder and computes
global luminance SSIM after compositing RGBA over white. It uses population
variance and covariance with the standard 8-bit constants `K1=0.01`,
`K2=0.03`, and `L=255`. Dimensions must match exactly. Per-slide scores and
paths are written to TSV, while the summary reports coverage, minimum, median,
and maximum.

**Trend reference: at least 0.95 SSIM on at least 80 percent of slides. Hard
automatic gate: every slide renders without panic, missing output, dimension
mismatch, or a dropped bounded shape. Hard manual gate: the pinned native
PowerPoint representative review is recorded and accepted.**

The trend line is not a PowerPoint-conformance threshold. A calibration over
all 34 slides of the ecodesign representative uses Microsoft PowerPoint 16.104
and the same pdftoppm 26.01.0 raster path. Native PowerPoint against the pinned
LibreOffice oracle produces zero slides at or above 0.95 SSIM, with median
0.650406194 and maximum 0.940934972. Slide 25 reproduces the recorded native PNG
hash, which confirms that the calibration uses the accepted native pipeline.
An implementation can therefore agree more closely with PowerPoint and still
move away from the LibreOffice trend line.

LibreOffice is the oracle only because PowerPoint is not scriptable on CI
runners, and LibreOffice has its own rendering bugs. **SSIM regressions are
therefore review-required, not automatic failures.** Spot-check against real
PowerPoint output once per milestone. The CI comparison records whether the
trend reference was met but does not fail solely because it was missed. Exact
oracle versions, full corpus coverage, valid dimensions, successful rendering,
and zero dropped bounded shapes remain enforced.

CI retains `gate-evidence.json`, `render-manifest.tsv`, and
`ssim-results.tsv` as the Presentation fidelity evidence artifact. The image
trees stay job-local because the TSV identifies every deck, slide, score, and
paired path without uploading hundreds of redundant raster files.

Stand this harness up in M10 alongside the first text rendering, not afterwards.

The M10 native spot-check uses Microsoft PowerPoint 16.104, Info.plist build
16.104.25121423 and AppleScript build 1214. PowerPoint PDF exports are
rasterised by pdftoppm 26.01.0 at 150 dpi. The low representative is
`sample_pptx_grouping_issues.pptx` slide 1 at LibreOffice SSIM -0.177170506.
PowerPoint confirms the white background and complete grouped geometry, while
the Rust render has a wrong red background and missing or misplaced groups.
The median representative is
`at.ecodesign.www_downloads_Vertiefungsvortrag_elektronik.pptx` slide 25 at
SSIM 0.172346895. PowerPoint confirms a full chart and product image which are
absent from the Rust render. The high representative is `crop-to-0.pptx` slide
2 at SSIM 1.0, an intentionally blank white slide matching at a glance.
LibreOffice follows PowerPoint for the substantive low and median content.

The temporary native PDF SHA-256 values are
`bd1511f546c970cddb9602f6b5421a3490e3ff22e5da74ca183e2e57b73a8f24`,
`99503f6dce0773c64da5b52e917d0d3f1f21aaddb0214532dda4c5131fdaa320`,
and `d5ce8e607f805914768d314ed6bb0f7f8fb762f9f62d680666e119f5c1afdf65`
for low, median, and high. Their 150 dpi PNG SHA-256 values are
`6ee02b21b8ee7ec1dd741ffd3a4b0bc2fe7a0d917c5b3c1d6c1b2aa69d7a088b`,
`85610d4b6778432355ab498f2a5da3bce6831cf502703d08caae70988307a49c`,
and `100875bd72e1c1ebe08263aac08bfb28dfd974a7f0f270ea98e0bbf9b9c7cbd2`.

Table rendering has an additional deterministic gate. A banded table with a
two-dimensional merge must produce the expected sampled fills, visible text,
merged bounds, and exactly one physical stroke per border segment. Separate
regressions prove that continuation cells emit no duplicate fill, border, or
text and that cell margins feed the shared fixed-box text path. Raster evidence
uses deterministic font mode.

## New tests the extracted crates need

These crates have never seen a non-docx package, so the existing tests do not
cover the cases that matter now.

**`oxml-opc`**
- `with_main_part("ppt/presentation.xml", ...)` then `main_document_part()`
  resolves, and the package round-trips.
- A pptx-shaped package: package rels to `presentation.xml`, slide rels to
  `slide1.xml`. Assert
  `resolve_rel_target("/ppt/slides/slide1.xml", "../slideLayouts/slideLayout1.xml")`
  resolves correctly. The `..` traversal is currently exercised only by docx
  headers.
- Every `rel_types` constant is unique and well-formed. Cheap, and the only
  thing that catches a copy-paste typo among the new constants.
- Zip-slip: a part named `../../etc/passwd` and an absolute-path entry are
  normalised or rejected. The code handles it, nothing tests it, and the crate
  is about to become a public shared component.

**`oxml-core`**
- New unit round-trips: `Centipoints::from_pt(18.0).0 == 1800`,
  `Angle::from_degrees(90.0).0 == 5_400_000`,
  `Percent1000::from_percent(75.0).to_fraction() == 0.75`.
- Existing `Length`, `Twips` and `Emu` constructors have positive and negative
  truncation-pinning tests that move with the units into `oxml-core`.
- `xml_text` becomes public API, so add CDATA, mixed content, nested elements,
  unknown entities, and the `GeneralRef` split case.
- `AppProperties` parses a Word `app.xml` **and** a PowerPoint one, leaving the
  other format's fields `None`, and omits them on write.

**`oxml-media`**
- Sniffing every format from magic bytes, and **sniff beats extension**: a
  `.png` that is really a JPEG resolves to JPEG.
- DPI from PNG `pHYs` with unit 1 and unit 0, and from JPEG JFIF density units
  1 and 2, including a file with EXIF before the SOF.
- **A truncation loop per format**: `for n in 0..data.len()`, assert no panic.
  Cheap, and it catches every slice-index bug in one shot.
- The counter fix, named as a sentence:
  `next_image_name_uses_the_highest_existing_index_not_the_part_count`.

**`oxml-layout`**
- `Transform` composition order matches the PDF `cm` operator.
- `walk()` flattens nested groups and accumulates the transform correctly.
- `FontManager` with no fonts returns an error rather than panicking, and
  `--no-default-features` is in the CI matrix so the system-font-discovery-off
  path is exercised while bundled deterministic fonts remain available.

**`oxml-pdf`**
- Three-deep groups balance `q` and `Q`, emit each `cm` before child content,
  and apply the declared clip rule and shared opacity state before recursion.
- `Path` with solid fill only, solid stroke only, and both, produces `f`, `S`
  and `B`. The combined case also proves `q`/`Q` counts balance, which catches
  the classic unbalanced graphics-state bug.
- Repeated equal alpha values produce one ExtGState with matching `CA` and
  `ca`, while distinct values remain distinct and opaque content emits none.
- A 50 percent black fill over white produces the exact midpoint pixel in the
  deterministic raster path.
- Linear and radial path gradients produce type 2 patterns, type 2 or type 3
  shadings, and type 3 stitching functions over interval type 2 functions.
  Structural tests also pin stop normalization, fill and stroke pattern
  operators, mixed solid paint, and page-local pattern resources.
- A 90 degree group rotation turns a linear gradient's sampled colour change
  vertical when rasterised at 72 dpi with the recorded Poppler 26.01.0.
- **`Group` containing `Text` finds the font.** The regression test for the
  recursion hazard.
- `Group` containing `Image` registers the XObject.
- `Group` containing `LinkAnnotation` emits it with a transformed rectangle.
- A preceding leaf proves nested XObject registration and recursive emission
  use the same depth-first ordinal.
- Raster: a rotated rectangle at 72 dpi has a filled interior pixel and an empty
  corner, and phase-zero line and path dashes have exact painted runs and gaps.
  Nested group samples pin transform order, clip intersection, and subtree
  opacity. Fill-rule, linear and radial gradient, gradient-domain, and page
  background samples pin the remaining paint translations. These are
  deterministic unit tests with no golden files.

## Binding tests

The parity suites are worth more than any number of Rust-side assertions,
because the whole value proposition is compatibility:

- The rdocx gate asserts exact `python-docx==1.2.0`, then executes the explicit
  seventeen-example S33 documentation manifest from stable v1.2.0 tagged
  sources. Sixteen bodies change only the import namespace. The exact
  Quickstart held-row body uses one declared public row re-fetch before its
  second cell assignment to respect strict global revision invalidation. Each
  manifest entry pins its source URL, heading, exact source statements,
  transformation and normalized structural assertion. The two-way
  differential authors the same paragraphs, runs, direct formatting, tables
  and cells with each writer, reads both files through both libraries, and
  directly compares normalized public records including distinct relative and
  absolute line spacing, units, enums, and saved table style.
- The same for `rpptx` and `python-pptx`.

The rpptx binding gate executes the seven python-pptx 1.0.2 Getting Started
workflows with the import namespace changed from `pptx` to `rpptx` and the
minimal public re-fetches required after structural writes. Its differential
rider asserts the exact oracle version, compares each writer through both
readers, and directly compares the normalized rpptx-authored and
python-pptx-authored records. It never compares package bytes and the oracle is
not a runtime dependency.

Both libraries are test-only CI dependencies. Neither oracle is a runtime or
published-crate dependency, and neither differential compares package bytes or
commits binary fixtures.

Each package has a strict typing smoke program that consumes its installed
public surface. Fresh cp39-abi3 wheels must contain the native-extension stub
and `py.typed` marker, pass exact `mypy==2.3.0 --strict`, and pass `stubtest`
against both installed packages. Strict mypy also checks every inline-typed
pure-Python source in each installed wheel. Representative enum-input,
return-type, inline-source, constructor, and member mutations must make those
gates fail, so hand-written stubs cannot drift.

The document WASM wrapper has a package-preservation Node gate and a PDF gate
in its single defaults-off profile. The PDF gate calls generated `toPdf`
through reflection and requires `%PDF-` through `%%EOF`, a Type 0 font, a
`FontFile2` stream, and the bundled Carlito base font. This proves the public
JavaScript name, complete output, and embedded fallback font at the generated
boundary.

The presentation WASM wrapper has one Node round-trip gate in its default
profile and a second Node gate with `render` enabled. The first crosses the
generated JavaScript `Uint8Array` boundary and proves that facade-owned slide
mutation preserves the complete package. The second produces a complete PDF.
The final normal-default artifact is built with exact wasm-pack 0.15.0,
optimized with reviewed wasm-opt 125, compressed with `gzip -n -9`, and
rejected at 1,000,000 decimal bytes. The wrapper manifest keeps render out of
defaults while its facade dependency selects the bundled template explicitly.
A padded artifact or render-enabled default must make the exact named size gate
fail.

The `rpptx` CLI integration gate corrupts a relationship and requires
`validate` to exit nonzero. It then requires all 50 manifest decks to validate
with a zero exit and never skips a missing corpus. The primary workspace-test
job and the MSRV job fetch and verify the pinned corpus before running Cargo
tests. Command regressions also prove bounded DPI, bounded diff work,
zero-slide PNG failure without output, and one-slide-at-a-time PNG conversion.
The thumbnail and outline gate requires an exactly 320-pixel-wide proportional
slide-one PNG and recursive paragraph output with stable level indentation.
Regressions cover nonstandard aspect ratios, shared output defaulting, grouped
text order, embedded paragraph-break normalization, and field-only title
identity so the title appears exactly once.

The `rdocx` CLI has one integration binary that invokes the compiled executable
through `CARGO_BIN_EXE_rdocx`. Its seven tests cover `inspect`, `text`,
`convert`, `diff`, `replace`, `validate`, and `render` with in-code DOCX and
corrupt-package fixtures. The assertions bind schema 1, default paths, exact
stdout, exit-status verdicts, output validity, replacement persistence,
document-order text, and bundled-font deterministic render bytes. Process ID
and an atomic counter isolate temporary workspaces across concurrent runs.

## What CI runs

| Job | Command |
|---|---|
| test | Fetch the pinned corpus, then run `cargo test --workspace --all-features --exclude rdocx-py --exclude rpptx-py` |
| no-default-features | `cargo test -p oxml-layout --no-default-features` |
| wasm | Locked `wasm32-unknown-unknown` checks, `wasm-pack test --node`, and local bundler pack and fresh-install gates for `rdocx-wasm` and `rpptx-wasm` |
| prose | `python3 scripts/prose_check.py` and `python3 scripts/sync_agent_skills.py --check` |
| hash-harness | `python3 scripts/hash_harness.py --check` |
| presentation-fidelity | Fetch the pinned corpus, then run `python3 scripts/pptx_ssim_harness.py --check` on the pinned macOS render stack |
| clippy | `cargo clippy --workspace --all-targets --all-features --exclude rdocx-py --exclude rpptx-py -- -D warnings` |
| fmt | `cargo fmt --all -- --check` |
| doc | `cargo doc --workspace --no-deps --all-features --exclude rdocx-py --exclude rpptx-py` with `RUSTDOCFLAGS=-D warnings` |
| package-oxml-layout | Verify the exact font and legal-file inventory, then build and size-check the verified archive |
| msrv | Fetch the pinned corpus, then run `cargo test --workspace --all-features --exclude rdocx-py --exclude rpptx-py` under Rust 1.93 |
| python-bindings | On pull requests, build each Python package with `maturin develop --locked` in its own Python 3.12.9 environment, then run its complete pytest directory |
| supply-chain | `cargo-deny check` |
| python-wheels | On manual dispatch or a `py-v*` tag, build six cp39-abi3 wheels for each Python package and one source distribution per package, then install and test every compatible artifact in a fresh environment |

The `--exclude` pair on every all-feature command is required, not cosmetic:
`pyo3/extension-module` tells the linker that Python symbols come from the host
interpreter, which is false for a test binary, and on Linux this is an
unresolved-symbol link failure that is easy to misdiagnose.

The wheel workflow runs the installed `rdocx` suites except the
Poppler-versioned rendering gate, which belongs to its pinned render job. It
runs the installed `rpptx` documented-example and differential suite. Native
cells also check the inline Python sources with exact `mypy==2.3.0 --strict`
and run `stubtest` across every public and native-extension module. The
musllinux cells install into clean Python 3.9 Alpine environments and run the
same package parity suites. Repository unit tests
parse the exact two-package, six-target product and use negative mutations to
prove that package, target, clean-install, parity, artifact dependency, and
tag-only OIDC requirements are sensitive before the hosted matrix runs.

The pull-request binding job has one matrix row for `rdocx` and one for
`rpptx`. It uses Python 3.12.9 with exact `maturin==1.13.3` and
`pytest==9.1.1`, installs `python-docx==1.2.0` or `python-pptx==1.0.2` for the
applicable row, and installs the Poppler toolchain required by the full rdocx
rendering suite. Each row creates a fresh environment, builds the extension,
then runs every test in that package's binding test directory. The build and
pytest commands are separate ordinary steps with no successful fallback or
`continue-on-error`, so either failure makes the pull-request check fail.
The operative top-level `pull_request` trigger schedules the job without a job
condition. Neither the job nor its pytest step has an environment or condition
that can suppress execution. Root permissions are exactly `contents: read`,
with no `id-token: write` grant anywhere in the workflow. Checkout v6.0.2,
setup-python v6.2.0, rust-cache v2.9.1, and the selected stable rust-toolchain
revision are bound to full reviewed commit SHAs. Their operative input maps are
exact and cannot be satisfied by comments.

The pull-request WASM job uses exact Node 24.11.1 and wasm-pack 0.15.0. It
installs the official Binaryen version 125 Linux archive only after verifying
its pinned SHA-256, places that optimizer on `PATH`, and requires the exact
version string. It target-checks both WASM packages with `--locked`, then runs
both inline suites through `wasm-pack test --node`.

Both manifests bind release optimization to `-Oz`,
`--enable-bulk-memory`, and `--enable-nontrapping-float-to-int`. The last flag
is required by nontrapping conversion operations emitted by the Rust 1.93
standard library. CI builds the exact `@tensorbee/rdocx-wasm` and
`@tensorbee/rpptx-wasm` release bundler packages with locked dependencies. Each
package is packed locally, installed into a separate fresh consumer through an
isolated npm cache with scripts disabled, and checked for its exact name,
version, WASM, JavaScript glue, public declaration, and import. The steps are
unconditional and propagate ordinary non-zero command status. Structured
regressions reject optimizer, checksum, package, target, scope, locking,
installation, authentication, publication, and tag mutations.

The job retains root `contents: read` permission and has no npm publication,
registry authentication, token, OIDC, release, or tag authority. Checkout
v6.0.2, setup-node v6.5.0, rust-cache v2.9.1, and the selected stable
rust-toolchain revision are bound to full reviewed commit SHAs.

## Gaps being closed

Stated plainly, because they are why two shipped defects went unnoticed:

- **Command-level output contracts need explicit coverage.** The published
  `rdocx-cli` surface has one compiled-binary integration test for each of its
  seven commands.
- **PDF and PNG output is only checked for non-emptiness**, so layout
  regressions are invisible. The hash harness closes this.
