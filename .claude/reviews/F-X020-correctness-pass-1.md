# F-X020, correctness, pass 1

**Reviewed**: the uncommitted working tree. `Cargo.lock` is the only product
change, 16 dependencies advanced, no manifest touched. The rest of the diff is
the design plan, the ledgers and the F-X021 filing.
**Verdict**: 0 defects, 0 smells, 1 nice-to-have

## Defects

None.

The story's whole risk is that a dependency changes output. It does, and the
change was traced to a named dependency and characterised before completion,
which is what the plan required. Details under the harness section below.

## Smells

None.

## Nice-to-have

### N1, the design plan mis-classified `zlib-rs`
`.claude/plans/F-X020-design.md`

The plan listed `zlib-rs` among the fourteen updates with "no path to rendered
output". `zlib-rs` reaches `flate2`, then `png`, then `tiny-skia`, which
`oxml-pdf` uses to rasterise, so it does have a path. The conclusion happened to
survive, since PDF stream compression uses `miniz_oxide` rather than `flate2`
and `miniz_oxide` did not update, but the reasoning that produced the
classification was wrong.

Recorded rather than fixed: the plan is a record of what was believed at design
time, and the correction belongs here and in AS_BUILT rather than as a silent
edit to the plan's problem statement.

## Not found

Checked and produced nothing:

- **correctness**. No source changed. The 16 updates are semver-compatible
  patch and minor bumps taken with `cargo update` and no `--breaking`, so no API
  surface moved. The full workspace suite passes unchanged, 53 test binaries,
  zero failures.
- **deps**. No `Cargo.toml` changed, so no crate gained or lost a dependency and
  no new edge exists. Every one of the 16 keeps its existing consumer.
- **surface**. Nothing public changed.
- **layering**. Unchanged, for the same reason: no manifest moved.
- **security**. `cargo audit` reports zero vulnerability entries across the
  refreshed graph. `cargo deny check` passes all four sections. The single
  allowed warning is `ttf-parser` RUSTSEC-2026-0192, unmaintained rather than
  vulnerable, allowlisted in `deny.toml` with a documented route out. This story
  deliberately did not attempt to clear it.
- **toolchain**. `rust-toolchain.toml` still pins 1.97.1 and `rustc --version`
  confirms it. The WASM targets and the bundled-fonts-off path both build.

## Hash harness, and what it could not see

**28 of 28, unchanged.** That result is true and, on its own, misleading.

The refresh **changed all seven sample PDFs**. Every sample PNG is
byte-identical, which is why the harness is flat: it records `page1.png` and
three `word/*.xml` parts per sample and **no PDF at all**.

Traced as the plan required, by reverting the lockfile and applying suspects
alone. `font-types 0.12.2 -> 0.12.3` on its own moves all seven PDFs. It reaches
`read-fonts 0.41.0`, then `harfrust`, the text shaper.

Characterised before accepting it, using the repository's own pinned Poppler
oracle:

- Extracted text is identical in **7 of 7** samples under `pdftotext`.
- `pdfinfo` output is identical apart from the file size line: same page count,
  same page geometry, same producer.
- Sizes move by single-digit bytes. `contract.pdf` is 58169 at the baseline and
  58171 with `font-types` alone.
- Every sample PNG is byte-identical, so nothing visible at 150 DPI moved.

That is a serialisation-level difference in numbers written into the content
stream, with no semantic effect. Acceptable, and now recorded rather than
discovered later.

The durable problem is not this delta but that a gate reported green while a
first-class output changed across every sample. Filed as **F-X021, The hash
harness should cover PDF output (M)**, which also has to decide what a stable
PDF fingerprint is, since raw PDF bytes carry a creation date and object
ordering that need not be reproducible.
