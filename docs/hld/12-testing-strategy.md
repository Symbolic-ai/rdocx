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

Four gates run against it:

1. **DrawingML structural round-trip**: every `a:txBody` and `a:spPr` parses,
   serialises and reparses to a structurally equal value. The pinned corpus has
   6,898 text bodies and 8,643 shape-property elements. This is the carried M7
   exit gate at the first point where the external corpus exists.
2. **Raw round-trip**: open and canonically save with every document part
   treated as opaque. Every decompressed part stays byte-identical, while
   content types and relationships stay structurally equal. ZIP metadata and
   compression are not model state. This proves the OPC layer and the corpus
   harness before any PresentationML modelling exists.
3. **Modelled round-trip** (M8 exit): parse, serialise, reparse, compare
   structurally, and compare the saved package part by part.
4. **Opens without repair** (M8 and M11): every saved deck opened manually in
   PowerPoint once per milestone. Not automatable, and not skippable.

## The render fidelity gate

Roughly 50 decks rendered to PNG at 150 dpi and compared with
`libreoffice --convert-to png` using a perceptual diff.

**Target: at least 0.95 SSIM on at least 80 percent of slides, and 100 percent
of slides rendering without a panic or a dropped shape.**

The second half of that is the hard gate. The first is a trend line.

LibreOffice is the oracle only because PowerPoint is not scriptable on CI
runners, and LibreOffice has its own rendering bugs. **SSIM regressions are
therefore review-required, not automatic failures.** Spot-check against real
PowerPoint output once per milestone.

Stand this harness up in M10 alongside the first text rendering, not afterwards.

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

- Write a document with `rdocx`, open it with `python-docx`, assert text, styles
  and tables survive. Then the reverse.
- The same for `rpptx` and `python-pptx`.

Both libraries are free CI dev dependencies.

Plus `mypy --strict` over a typing smoke file and `stubtest` against the stubs,
so hand-written stubs cannot drift.

## What CI runs

| Job | Command |
|---|---|
| test | `cargo test --workspace --all-features --exclude rdocx-py --exclude rpptx-py` |
| clippy | `--workspace --all-targets --all-features -- -D warnings` |
| fmt | `cargo fmt --all -- --check` |
| doc | `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS=-D warnings` |
| msrv | the pinned MSRV toolchain |
| supply-chain | `cargo-deny check` |
| no-default-features | `cargo test -p oxml-layout --no-default-features` |
| wasm | `cargo check --target wasm32-unknown-unknown` plus `wasm-pack test --node` |
| python | `maturin develop && pytest`, plus `mypy` and `stubtest` |
| packaging | `cargo publish --dry-run` plus a `.crate` size assertion |
| prose | the voice rules over tracked Markdown |

The `--exclude` on the binding crates is required, not cosmetic:
`pyo3/extension-module` tells the linker that Python symbols come from the host
interpreter, which is false for a test binary, and on Linux this is an
unresolved-symbol link failure that is easy to misdiagnose.

## Gaps being closed

Stated plainly, because they are why two shipped defects went unnoticed:

- **`rdocx-cli` has zero tests** despite being a published binary.
- **`rdocx-wasm` has zero tests and no CI job**, which is exactly why its
  part-dropping save path survived a release.
- **PDF and PNG output is only checked for non-emptiness**, so layout
  regressions are invisible. The hash harness closes this.
