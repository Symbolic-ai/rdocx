# Current Sprint, S01

**Milestone**: M1 Preparation and safety net.

**Goal**: Make every future change measurable before anything moves. Rendering
becomes reproducible across machines, a byte-level baseline exists for every
sample, and the three shipped defects found during the architecture audit are
fixed and released.

## Spec references

- `docs/hld/11-migration-plan.md`, for why the safety net comes first: the
  extraction changes unit conversion and text-shaping input types, both of which
  alter output without failing to compile.
- `docs/hld/12-testing-strategy.md`, for the hash harness definition and its
  rules, and for the gaps this milestone closes.
- `docs/hld/15-build-and-toolchain.md`, for toolchain pinning and the
  deterministic font mode that F-003 depends on.
- `docs/hld/13-risks-and-open-questions.md`, for the three known defects carried
  into F-004, F-005 and F-006, and for R1 which this whole sprint exists to
  mitigate.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-001 | Deterministic font mode | M | in-progress | codex |
| F-002 | rust-toolchain.toml | S | in-progress | codex |
| F-003 | Output-stability hash harness | L | pending | - |
| F-004 | Caladea licence and the false OFL claim | S | in-progress | codex |
| F-005 | Fix the image counter | S | pending | - |
| F-006 | Fix the JPEG standalone-marker walk | S | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

**F-001 blocks F-003.** `crates/rdocx-layout/src/font.rs:93` calls
`load_system_fonts()`, so a PNG baseline recorded on one machine would not
reproduce on another. A harness whose baselines are not reproducible is worse
than no harness, because it produces failures indistinguishable from real
regressions.

F-002, F-004, F-005 and F-006 are independent of everything and of each other.
They can be picked up in any order or in parallel.

F-004 is worth landing regardless of this project. `crates/rdocx-layout/fonts/`
ships four Caladea TTFs with no licence file, while `bundled_fonts.rs:12` claims
all bundled fonts are SIL OFL. Caladea is Apache-2.0, so the stated licence is
wrong and the attribution requirement is unmet in a crate that is published
today behind a default-on feature.

## Definition of done for this sprint

- `cargo test --workspace` is green.
- The hash harness records a baseline, and that baseline reproduces byte for
  byte on a second machine.
- The harness fails when a deliberate whitespace change is injected into a
  writer, proving it can see what the existing 320 tests cannot.
- Every bundled font family has a licence file, and the doc comment is accurate.
