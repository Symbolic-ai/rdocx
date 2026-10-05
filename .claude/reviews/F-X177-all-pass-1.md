# F-X177, all aspects, pass 1

**Reviewed**: Frozen working diff against `78813bd46516f41d233bb57e606911961e351822`, 6 files, 18 insertions and 8 deletions. AGENTS, CLAUDE, WORKFLOW, the canonical microscope command, approved plan and cited rendering and toolchain contracts were read.
**Verdict**: 0 defects, 0 smells, 0 nitpicks.

## Defects

Zero defects found.

## Smells

Zero smells found.

## Nitpicks

Zero nitpicks found.

## Not found

Correctness: zero findings. The four-line addition at
`crates/oxml-layout/src/font.rs:2042` uses `Database::with_face_data` for the
additional source variants. The local fontdb 0.23 implementation obtains the
selected face source and its original index, then invokes the closure with
borrowed bytes. `Arc::from(data)` creates owned `Arc<[u8]>` storage before the
closure returns. The existing binary cache and ordinary system-file arms at
`crates/oxml-layout/src/font.rs:2032` and
`crates/oxml-layout/src/font.rs:2041` are unchanged. Missing faces or failed file
reads retain the optional failure behavior.

Contract: zero findings. The change matches the four-line fallback from the
reviewed PR head named in the plan, including its comment and narrowly scoped
unreachable-pattern allowance. No dependency, default feature, public API or
font asset changes occur. The HLD changes are limited to the two files in the
approved impact list. Contributor provenance remains in the plan and backlog.

Panics: zero findings. No unwrap, expect, indexing, arithmetic or unsafe block
is introduced. Borrowed font data cannot escape through the returned owned
byte buffer.

OOXML: zero findings. This diff contains no parser, serializer, namespace or
schema-order changes.

Tests: zero findings. The recorded original-base command produced E0004 for
`Source::SharedFile` at the exact source match. Reverting the fallback restores
that non-exhaustive match, so the compile gate is meaningful. Worker logs show
checks and font tests passing in all four configurations: default, no-default,
memmap and no-default plus memmap. The logs report 122 unit tests with defaults,
120 without defaults and 3 doctests in each configuration. The deterministic
constructor at `crates/oxml-layout/src/font.rs:726` still creates a fresh
bundled-only database. Host discovery remains gated at
`crates/oxml-layout/src/font.rs:706`. The fallback does not enable discovery or
change the deterministic binary path. The recorded hash gate matches all 49
entries without baseline edits.

Structure: zero findings. The change adds one match arm in the existing
function, with no trait, generic, wrapper, module, crate or feature flag.

Package evidence: zero findings. The crate README and recorder agree on
4,634,240 observed compressed bytes, 9,269,857 normalized member bytes and 51
members. Independent read-only inspection of the regenerated archive found
4,634,245 compressed bytes, the same normalized member total and the same
member count. The five-byte gzip variance is within the existing measurement
tolerance, and the archive is below 10 MiB. Only the changed crate's inventory
row, constants and date were refreshed.

This review used independent code and contract inspection and read existing
worker gate logs. It did not rerun the full scoped gate. An earlier policy run
started before the README inventory refresh and failed its row assertion.
Current expected and observed rows match. Final frozen policy verification
remains a worker completion obligation and is not certified implicitly by this
review verdict.

No source, test or documentation files outside the review directory were
changed. This review pass ends with this report.
