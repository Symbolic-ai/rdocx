# F-090, Preset table generator

**Status**: completed
**Sprint**: S22
**Size**: L
**Depends on**: F-089, F-058

## Problem

The guide evaluator exists in `crates/oxml-drawing/src/geometry.rs:1`, but no
preset definition table exists and `tools/gen-presets/` is absent. As a result,
`crates/rpptx-layout/src/context.rs:463` treats every preset as a bounds
fallback. The renderer specification at
`docs/hld/08-rendering-spec.md:243` requires an offline generator and a
checked-in output so builds do not fetch or parse XML.

F-089 permits the official ECMA-376 fifth-edition electronic addendum under its
BSD terms. The source, notice, generator, and generated Rust must now make that
decision reproducible and auditable.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "Preset geometry".
- `docs/hld/12-testing-strategy.md`, "The pptx corpus".
- `docs/hld/13-risks-and-open-questions.md`, the resolved preset provenance
  decision.
- `docs/hld/14-development-backlog.md`, "F-090, Preset table generator".

## Approach

Add the explicitly planned `tools/gen-presets/` directory with the pinned
`presetShapeDefinitions.xml`, its required Ecma BSD notice, and one Python
generator. The source is copied verbatim from the official fifth-edition Part 1
electronic addendum and checked against the F-089 SHA-256 before generation.

The generator parses all 187 top-level definitions using the Python standard
library. The source contains 186 unique preset names because two byte-identical
`upDownArrow` definitions are present. The generator rejects a repeated name
whose source bytes conflict, deduplicates the identical pair, canonicalises
each unique definition into a complete `a:custGeom` byte string, and emits a
deterministic 186-key match table in
`crates/oxml-drawing/src/preset_shape_data.rs`. The existing
`crates/oxml-drawing/src/lib.rs` declares the generated module privately.
Generated output carries the upstream provenance, source hash, and BSD notice.
No `build.rs`, network access, new Cargo dependency, trait, generic parameter,
or feature flag is introduced.

The command supports `--check` and an optional corpus path. Check mode compares
fresh output byte for byte with the checked-in file and scans every `.pptx`
member for `a:prstGeom@prst`, failing if any corpus name is absent from the
table. It also asserts exactly 187 direct definitions and 186 unique source
names.

## Design deviations

The approved plan described 187 unique source names. Verification of the pinned
official XML found 187 direct definitions and 186 unique names because
`upDownArrow` occurs twice with byte-identical definition XML. The generator
therefore rejects conflicting duplicates and deduplicates this identical pair.
This follows the official source without inventing a non-standard lookup key.

## Rejected alternatives

- Generate at build time. That makes builds depend on a large XML input and
  hides generated changes from review.
- Hand-maintain a Rust match table. It cannot prove provenance or
  byte-identical regeneration.
- Emit only presets currently found in the corpus. The source already provides
  the complete standard table and future decks should not require code edits.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `generator_reproduces_checked_in_table` | Running the generator in check mode produces a byte-identical Rust file |
| corpus | `generated_table_covers_every_corpus_preset` | Every preset name in all 50 pinned decks exists in the generated table |
| unit | `source_has_187_direct_definitions` | The pinned official XML contains 187 direct definitions, 186 unique names, and no conflicting duplicate |
| unit | `generated_lookup_has_known_and_unknown_cases` | A standard name returns bytes and an unknown name returns none |

The backlog test gate is that the generated table covers every preset name in
the corpus and regenerates byte-identically. Run
`python3 tools/gen-presets/generate.py --check --corpus corpus/pptx`.

## HLD impact

None. F-089 records the source decision and the existing rendering mechanism
already requires this exact checked-in generator.

## Risk routing

- A new module and files. `CLAUDE.md` requires explicit approval. F-090 itself
  explicitly names `tools/gen-presets/` and a checked-in generated file, and
  the invoked sprint authorises that planned structure. The concrete files are
  limited to the source XML, licence notice, generator, generated Rust module,
  and the existing `lib.rs` module declaration.

## Hash harness

Expected to be unchanged. The table is not connected to either rendering path
until F-091.

## Implementation checklist

- [x] Vendor the exact official XML and required BSD notice.
- [x] Implement deterministic parsing and Rust emission.
- [x] Generate 186 unique preset entries from all 187 direct definitions.
- [x] Add byte-identical `--check` mode.
- [x] Scan all 50 corpus decks and reject missing preset names.
- [x] Expose only a private generated lookup for F-091.

## Open questions

None. F-089 fixes the source and licence, while the HLD fixes the offline,
checked-in mechanism and complete-table direction.
