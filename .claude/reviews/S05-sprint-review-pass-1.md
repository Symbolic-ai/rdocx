# S05 sprint review, pass 1

**Reviewed**: `sprint/s05` at `a7d8bae` against
`80e0f283bde9e57ed479598bd861566eee880eeb`, 25 files, 2,362 changed lines
with 2,300 additions and 62 deletions, crates: `oxml-media`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M3 gate is: "the staged crate passes its tests and the hash harness remains
unchanged. The sniffed content-type delta waits for F-027."

The gate holds. `cargo test -p oxml-media --locked` passes all 22 unit and
regression tests, including magic-byte precedence, every-prefix parser safety,
PNG and JPEG DPI handling, maximum-suffix naming, and native EMU sizing.
`python3 scripts/hash_harness.py --check` reports all 28 entries unchanged.
The sprint diff changes no released `rdocx-*` crate or dependency list, and
F-027 remains pending at `docs/sprints/BACKLOG.md:83`.

The publication boundary also holds. `crates/oxml-media/Cargo.toml:3` keeps the
crate at 0.0.0, `crates/oxml-media/Cargo.toml:10` disables publication, and
`cargo tree -p oxml-media --edges normal --locked` reports no dependencies.
The sprint changes neither `.github/workflows/publish.yml` nor any released
rdocx manifest. The observed full gate used dry-run packaging only, with every
upload aborted.

## Not found

- **Interaction**: F-023 sniffing is the sole dispatch path for F-024 probing,
  F-026 consumes the integrated `ImageInfo` contract directly, and F-025
  naming state is independent. No jointly incorrect behavior was found.
- **Duplication**: no duplicate sprint helper or competing media abstraction
  was added. The byte readers and parser helpers each have one implementation.
- **Layering**: `oxml-media` remains a dependency-free format-neutral leaf and
  gains no `rdocx-*` or `rpptx*` edge.
- **Harness**: no output delta was found. All four AS_BUILT entries declare the
  same unchanged 28-entry result as the integrated gate.
- **Gate**: no unsupported gate assertion was found. The focused tests and
  hash check provide direct evidence for both halves of the M3 gate.
- **Docs**: no stale or contradictory sprint-owned HLD section was found. The
  F-026 plan's three-file HLD impact is reflected in architecture, media, and
  backlog contracts.
- **Deps**: no crate dependency was added to `oxml-media`, and no released
  dependency graph changed.
- **Surface**: no unplanned public API was found. `ImageFormat`, `resolve`,
  `ImageInfo`, `probe`, `NativeSize`, and `MediaNamer` match F-023 through
  F-026.
- **Parser safety**: no unchecked slice access, unchecked offset arithmetic,
  or malformed-header panic path was found. PNG, JPEG, GIF, BMP, and all three
  WebP layouts use bounded reads, and every-prefix regressions pass.
- **Exact contracts**: no mismatch was found in sniff-first resolution,
  per-axis DPI fallback, truncating EMU conversion, positive suffix scanning,
  or integer-boundary wrap.
- **Publication**: no publishable development crate, release allowlist change,
  tag, upload, or released-consumer cutover was found.
