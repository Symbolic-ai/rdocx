# F-067, Create rpptx-oxml and the corpus harness

**Status**: approved
**Sprint**: S16
**Size**: M
**Depends on**: none

## Problem

The workspace has no PresentationML crate or fetched deck corpus. The workspace
member list stops at the Word crates in `Cargo.toml:3`, while the M8 contract
requires an unpublished `rpptx-oxml` crate and a reproducible 50-deck harness.
The existing OPC reader expands every ZIP entry into part bytes at
`crates/oxml-opc/src/package.rs:47` and its writer rebuilds a canonical archive
at `crates/oxml-opc/src/package.rs:124`, so exact source-archive equality is not
currently preserved by open and save.

This story also owns the carried M7 entry gate. The repository has inline
DrawingML coverage, but no test enumerates every `a:txBody` and `a:spPr` in the
external corpus before M8 modelling begins.

## Spec reference

- `docs/hld/03-architecture.md`, "Three families, one workspace" and "The
  dependency rule".
- `docs/hld/04-opc-and-packaging.md`, "The package" and "What transfers
  unmodified".
- `docs/hld/12-testing-strategy.md`, "The deck corpus".
- `docs/hld/13-risks-and-open-questions.md`, "Q3, the deck corpus".
- `docs/hld/14-development-backlog.md`, "F-067, Create rpptx-oxml and the corpus
  harness".
- `docs/hld/15-build-and-toolchain.md`, "Publishing".

## Approach

Add `crates/rpptx-oxml` as a workspace member and workspace dependency at
version `0.0.0` with `publish = false`. The crate depends from the
PresentationML family toward `oxml-opc`, `oxml-core`, and `oxml-drawing`, with
no reverse family edge. Its initial library surface contains namespace
constants and the corpus-facing package entry needed by the later S16 models.

Add a standard-library Python fetcher and a tracked manifest containing one
stable URL, producer classification, expected relative path, and SHA-256 per
deck. The approved corpus is 49 valid Apache POI slideshow test decks pinned at
commit `11ede1db13c554b4341266faeb84e327fc316379`, plus the public Google Slides
export referenced by the MIT-licensed `gdown` project. The fetcher writes only
to ignored `corpus/pptx`, verifies every digest, rejects missing or extra
manifest entries, and supports a check-only mode that does not mutate the
corpus.

Add one crate-local corpus test entrypoint. It requires exactly 50 fetched
decks, opens each through `OpcPackage`, confirms the presentation main part,
and compares every decompressed package part and relationship after canonical
save. This is the approved meaning of byte-identical for F-067 because OPC
archive metadata and compression are not model state. The test also walks XML
parts to run the carried `CT_TextBody` and `CT_ShapeProperties` structural
round-trips for every `a:txBody` and `a:spPr`. It reports the deck and part path
on the first failure. A small code-built package test remains available when
the external corpus is absent, but it does not count as corpus evidence.

## Rejected alternatives

- Commit binary deck fixtures. The testing strategy makes the fetched external
  corpus the only binary-fixture exception and keeps it outside published
  crates.
- Treat generated decks as the 50-deck corpus. They cannot represent the
  producer diversity that is the purpose of the gate.
- Add a Presentation package wrapper that only forwards `OpcPackage`. The
  structural rules prohibit forwarding-only wrappers.
- Publish the implemented crate. PowerPoint development crates remain at
  version 0.0.0 with publication disabled until development is complete.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `rpptx_oxml_is_an_unpublished_workspace_member` | The manifest remains at 0.0.0 with publication disabled |
| integration | `corpus_manifest_is_complete_and_verified` | Exactly 50 pinned decks exist with matching SHA-256 values and producer metadata |
| round-trip | `all_corpus_decks_round_trip_opaquely` | Every deck satisfies the approved byte-identity definition with no XML modelling |
| round-trip | `carried_m7_drawingml_gate_passes_for_the_corpus` | Every corpus `a:txBody` and `a:spPr` serialises and reparses structurally |

The test gate is: the carried M7 DrawingML gate passes, and all 50 decks
round-trip byte-identically with no XML modelling.

## HLD impact

- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`

## Risk routing

- Any parser or serialiser. Recheck fixed write prefixes, schema ordering, and
  byte preservation for unmodelled subtrees across the corpus.
- Crate dependency graph and a new cross-family `use`. Run
  `cargo tree -p rpptx-oxml` and confirm that no `oxml-*` crate gains an
  `rpptx-*` dependency.
- A new crate, module, or file. Obtain explicit approval for the crate,
  manifest, fetcher, corpus manifest, and test entrypoint before implementation.

## Hash harness

Expected to be unchanged. The new crate and external corpus do not participate
in Word sample generation or rendering.

## Implementation checklist

- [ ] Add the unpublished `rpptx-oxml` workspace member and dependency edge.
- [ ] Add PresentationML namespace constants and the minimal package entry.
- [ ] Add and validate the pinned 50-deck corpus manifest and fetcher.
- [ ] Add the opaque corpus round-trip test with precise failure context.
- [ ] Execute the carried M7 `a:txBody` and `a:spPr` structural gate.
- [ ] Run the crate, dependency-tree, corpus, prose, and hash checks.

## Open questions

None. The user approved the pinned public corpus, decompressed part-by-part
equality after canonical save, and creation of the new crate, source module,
fetch script, tracked corpus manifest, and single test entrypoint.
