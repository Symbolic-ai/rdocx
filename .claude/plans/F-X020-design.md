# F-X020, Refresh the dependency lockfile

**Status**: completed
**Sprint**: S42
**Size**: S
**Depends on**: none

## Problem

Sixteen semver-compatible dependency updates are outstanding. None is a
security fix: `cargo audit` reports zero vulnerabilities across 152
dependencies, and `cargo deny check advisories` passes.

The reason to take them anyway is that a lockfile left to drift turns a routine
refresh into a large delta nobody can attribute. Taking sixteen small updates
now, and measuring what they do, is cheaper than taking sixty later and
discovering that rendering moved for a reason buried among them.

Two of the sixteen are in the rendering path, and the dependency tree says
exactly how:

- **`font-types 0.12.2 -> 0.12.3`** reaches `read-fonts 0.41.0`, then
  **`harfrust 0.12.0`**, then `oxml-layout`. Harfrust is the text shaper, so a
  change here can move glyph positions on every rendered page. This is the
  highest-risk update in the set.
- **`zune-core 0.5.1 -> 0.5.3`** reaches `zune-jpeg`, which
  `crates/oxml-pdf/src/raster.rs` uses to decode JPEG. The rasteriser produces
  the PNGs the hash harness records, so a decode change moves those bytes for
  any sample carrying a JPEG.

A third, `font-types 0.11.3`, also sits in the tree via `skrifa` and
`subsetter` for PDF font embedding, but nothing updates it: only the 0.12 line
has a newer release.

The remaining fourteen are build-time, CLI or error-formatting crates with no
path to rendered output: `aho-corasick`, `bytemuck_derive`, `cc`, `clap`,
`clap_builder`, `find-msvc-tools`, `futures-core`, `futures-task`,
`futures-util`, `minicov`, `regex-automata`, `thiserror`, `thiserror-impl`,
`zlib-rs`.

## Spec reference

- `docs/hld/12-testing-strategy.md`, "The hash harness", for the rule that an
  intentional delta lands as its own labelled commit with the expected change
  stated. That rule is what this story turns on.
- `docs/hld/15-build-and-toolchain.md`, for the pinned 1.97.1 toolchain and the
  MSRV the refresh must keep building against.
- `docs/hld/14-development-backlog.md`, "F-X020, Refresh the dependency
  lockfile".

## Approach

`cargo update`, with no `--breaking`. Semver-compatible updates only, which is
what keeps this an S rather than a story that has to read sixteen changelogs for
API breaks.

Then measure, in this order, because the order is what makes a delta
attributable:

1. The full workspace suite. A behavioural change in a dependency shows up here
   first and with a better error message than a hash mismatch gives.
2. The hash harness. This is the real gate.
3. The pinned toolchain and the WASM targets, since `cc` and `find-msvc-tools`
   are build-time crates and a build-script regression would surface here.

**If the harness is unchanged**, the story is a clean refresh and the lockfile
change is the whole diff.

**If the harness moves**, the delta is traced to a specific dependency before
anything is re-recorded. The two candidates are named above, and the trace is
mechanical: revert the lockfile, apply the suspect update alone with
`cargo update -p <crate>`, and re-run the harness. A delta that traces to none
of the rendering-path dependencies is unexplained and blocks the story rather
than prompting a baseline re-record.

`ttf-parser` and its unmaintained advisory are untouched. Clearing
RUSTSEC-2026-0192 needs the `fontdb` to `fontique` swap described in
`deny.toml`, not a lockfile refresh, and this story deliberately does not
attempt it.

## Rejected alternatives

- **`cargo update --breaking`, or bumping major versions.** Sixteen semver
  compatible updates and a set of API breaks are different stories with
  different risks. Mixing them means a harness delta could come from either.
- **Update only the fourteen safe crates and hold the two rendering ones.**
  Tempting, and wrong: it leaves exactly the two updates that need measuring
  unmeasured, and defers them to a sprint where they will be harder to isolate.
- **Re-record the baseline first and compare afterwards.** That inverts the
  gate. The baseline changes only through a delta that was explained first.
- **Take the `fontique` swap now** to clear the unmaintained advisory. A real
  piece of work with its own design, not a lockfile refresh.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | the existing workspace suite | No behavioural change in any crate. This is the gate that catches a dependency changing semantics rather than bytes |
| golden | `python3 scripts/hash_harness.py --check` | Rendered output is unchanged, or its delta is traced to a named rendering-path dependency |

**Test gate**, from the backlog: the full workspace suite and the hash harness.

This story adds no new test. That is deliberate and worth stating rather than
leaving as an omission: there is no new behaviour to pin, and the existing 28
baselines plus 53 test binaries are precisely the instrument this change needs.
Writing a test that asserts a version number would pin the lockfile, not the
behaviour, and would have to be edited by every future refresh.

## HLD impact

None, unless the harness moves and a baseline is re-recorded, in which case
`docs/hld/12-testing-strategy.md` keeps its existing description of the harness
and only the recorded values change.

## Risk routing

Two rows match.

- **Layout, pagination, line breaking, text shaping.** `font-types` reaches the
  shaper through `harfrust`. Deterministic font mode for any baseline recorded,
  and a re-record is deliberate and separately committed. This row is the reason
  the harness is the gate rather than a formality.
- **A new dependency, or a change to one.** Every one of the sixteen keeps its
  existing named consumer, since this is a version refresh and not a new edge.
  No `Cargo.toml` changes: the diff is `Cargo.lock` alone.

## Hash harness

**28 of 28, unchanged, and that is not the whole answer.**

The harness did not move. All seven sample PDFs did. The harness records
`page1.png` and three `word/*.xml` parts per sample and no PDF, so a change
confined to PDF output is invisible to it.

Traced as this section required: `font-types 0.12.2 -> 0.12.3` alone moves all
seven, reaching the shaper through `read-fonts` and `harfrust`. Characterised
with the pinned Poppler oracle before being accepted: extracted text identical
in 7 of 7 samples, `pdfinfo` identical apart from file size, sizes moving by
single-digit bytes, every PNG byte-identical. A serialisation-level difference
with no semantic effect.

No baseline was re-recorded, because no recorded baseline moved. The gap the
episode exposed is filed as F-X021.

## Implementation checklist

- [x] Record the pre-change harness state and the exact update list
- [x] `cargo update`, no `--breaking`
- [x] Full workspace suite
- [x] Hash harness, and trace any delta to a named dependency before proceeding
- [x] Pinned toolchain, WASM targets, no-default-features path
- [x] `cargo audit` and `cargo deny check` still clean
- [x] `/microscope F-X020 --working`
- [x] `/verify`

## Open questions

None.
