# S09 sprint review, pass 2

**Reviewed**: `sprint/s09` at `2a00000b` against merge base `06b28f14`,
21 files, 1,910 changed lines, crates: `oxml-pdf`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Pass 1 resolution

Pass 1 S1 is resolved. The recursion-hazard table now identifies the stable
font collector and the image and annotation passes inside `write_pdf` at
`docs/hld/08-rendering-spec.md:126`, without obsolete source-line coordinates.

## Milestone gate

The M5 gate is: "golden-PNG diffs of the whole sample corpus show zero pixel
changes" (`docs/hld/14-development-backlog.md:322`). It holds against the
reviewed F-039 baseline. Consolidated verification at `db639053` observed exact
equality for all seven page-one RGBA buffers under `pdftoppm version 26.01.0`.
The deliberate one-pixel `quote` mutation failed and named the changed sample.
Only the pass 1 review and HLD reference correction followed that verification.

The S09 implementation gate is directly exercised. Paths cover every approved
paint operator at `crates/oxml-pdf/src/writer.rs:1106`. Alpha reuse and PDF
constants are checked at `crates/oxml-pdf/src/writer.rs:1226`. Three-deep group
state is checked at `crates/oxml-pdf/src/writer.rs:1352`. Nested font, image,
and transformed-link collection is checked from
`crates/oxml-pdf/src/writer.rs:1444`. The 28-entry hash baseline has no sprint
diff, and the full workspace, package dry-run, archive-size, dependency, and
supply-chain gates passed without publishing a crate.

## Not found

- `interaction`: visible paths and group clips share geometry emission, all
  alpha consumers share one registry, and collection identity matches recursive
  content order. No jointly incorrect feature interaction was found.
- `duplication`: the sprint added one path geometry emitter, one alpha registry,
  and one recursive content seam.
- `layering`: `oxml-pdf` has no `rdocx-*` or `rpptx-*` dependency at
  `crates/oxml-pdf/Cargo.toml:15`.
- `harness`: neither baseline changed. The exact hash and pixel comparisons both
  passed on the integrated implementation.
- `gate`: focused operator and structure tests, exact rendering checks, the
  mutation proof, and the full workspace gate cover the sprint definition of
  done.
- `docs`: the rendering, testing, risk, and backlog HLD sections describe the
  integrated state, including the corrected stable pass references.
- `deps`: no manifest or lockfile changed.
- `surface`: all S09 renderer helpers and state remain private.
- `publication`: every staged `oxml-*` crate remains at version 0.0.0 with
  publication disabled. Cargo performed dry-runs only.
- `ledgers`: all four delivery records agree on S09 completion and unchanged
  harness output.
