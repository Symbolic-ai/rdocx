# S58 sprint review, pass 15

**Reviewed**: `sprint/s58` at
`f6c74eb4a51a1b2b37ee51b017f33d85f333abf5` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 171 files, 16,225 changed
lines. Crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-layout`, `rdocx-oxml`, `rdocx-wasm`, `rpptx`,
`rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-render`,
and `rpptx-wasm`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This fifteenth pass is the explicitly authorized dependency-prefix checkpoint
after F-199 completion. Pass 14 was clean. The incremental feature delta adds
the reviewed F-199 Word consumer and oracle evidence, then records its delivery
state. The full sprint delta remains in scope for the interaction audit.

## Blocking

None. 0 blocking findings.

## Should-fix

None. 0 should-fix findings.

## Nice-to-have

None. 0 nice-to-have findings.

## Milestone gate

The M20 end gate is:

> The Word corpus renders at the declared SSIM threshold, and text shaping is
> correct for the scripts the corpus contains.

The gate is defined at `docs/hld/14-development-backlog.md:1817`. It remains
explicitly unclaimed at this scheduled dependency-prefix checkpoint. F-199 is
done, but F-200, F-X060, and F-X031 remain pending at
`docs/sprints/CURRENT_SPRINT.md:43` through
`docs/sprints/CURRENT_SPRINT.md:47`. The remaining directional-text,
stable-publication, and protected-branch acceptance conditions remain open at
`docs/sprints/CURRENT_SPRINT.md:73` through
`docs/sprints/CURRENT_SPRINT.md:98`. The M20 ledger correspondingly retains
F-200 as its one pending story at `docs/sprints/BACKLOG.md:393` through
`docs/sprints/BACKLOG.md:405`.

The completed F-199 prefix does have evidence for its own shaping portion. The
exact-head full verification record reports 49 of 49 unchanged and four of four
multi-script pages at raw SSIM 0.95 or better at
`.claude/scratch/S58-run.json:450` through
`.claude/scratch/S58-run.json:453`. The HLD records the individual Arabic,
Devanagari, Thai, and Simplified Chinese scores and the unchanged hard rule at
`docs/hld/12-testing-strategy.md:754` through
`docs/hld/12-testing-strategy.md:762`. This evidence supports the completed
dependency prefix. It does not claim the final M20 or S58 end gate.

## Not found

- **F-198 hyphenation and F-X058 rich shaping interaction, 0 findings**: shared
  multilingual layout still owns conditional hyphenation, clusters, source
  ranges, and line-local UAX 9 ordering at
  `docs/hld/03-architecture.md:127` through
  `docs/hld/03-architecture.md:141`. The integrated regression keeps an
  RTL-first Arabic span and hyphenatable English in the same rich breaker at
  `crates/rdocx-layout/src/engine.rs:7355`, and the mixed-script regression
  proves the generated conditional hyphen remains present at
  `crates/rdocx-layout/src/engine.rs:8150`. Word's exact language projection,
  0.8em rich baseline, cluster-safe fitting, and backend contract agree with
  `docs/hld/08-rendering-spec.md:459` through
  `docs/hld/08-rendering-spec.md:484`.
- **F-X062 and F-X063 retained-layout interaction, 0 findings**: automatic
  hyphenation remains part of retained-context equality at
  `crates/rdocx-layout/src/engine.rs:598` through
  `crates/rdocx-layout/src/engine.rs:625`. Restart identity serializes the
  authoritative paragraph or table XML at
  `crates/rdocx-layout/src/engine.rs:804`, and the regression mutates all three
  modeled Word language slots plus retained foreign attributes at
  `crates/rdocx-layout/src/engine.rs:9291`. Rich paragraphs cannot enter the
  legacy paragraph cache because multilingual inline and line items fail closed
  with uncacheable accounting at `crates/rdocx-layout/src/engine.rs:3660`
  through `crates/rdocx-layout/src/engine.rs:3674`. The F-X063 warm path skips
  only the already-authoritative second font-byte comparison at
  `crates/rdocx-layout/src/engine.rs:1075` through
  `crates/rdocx-layout/src/engine.rs:1095`. The four 700-paragraph related-story
  contracts remain present at `crates/rdocx-layout/src/engine.rs:9702` through
  `crates/rdocx-layout/src/engine.rs:9865`.
- **Oracle locking, provenance, licence, and packaging, 0 findings**: the
  harness binds the exact source and output SHA-256 values, HarfBuzz identity,
  generation command, three-file inventory, and byte-identical Noto licence at
  `scripts/docx_ssim_harness.py:123` through
  `scripts/docx_ssim_harness.py:173`. The fixed POSIX lock has bounded timeout
  and unconditional descriptor release at `scripts/docx_ssim_harness.py:236`
  through `scripts/docx_ssim_harness.py:281`, with overlap, timeout, and error
  cleanup covered at `scripts/docx_ssim_harness.py:871` through
  `scripts/docx_ssim_harness.py:914`. The provenance file names the published
  source bytes and reproducible command at
  `scripts/oracle-fonts/PROVENANCE.md:1` through
  `scripts/oracle-fonts/PROVENANCE.md:29`. The HLD keeps this oracle-only asset
  outside crate archives and the 24-font product inventory at
  `docs/hld/15-build-and-toolchain.md:200` through
  `docs/hld/15-build-and-toolchain.md:209`.
- **HLD scope and public surface, 0 findings**: the F-199 delivery record lists
  exactly HLD 03, 08, 10, 12, and 15 at
  `docs/sprints/AS_BUILT.md:9914` through
  `docs/sprints/AS_BUILT.md:9917`, matching the plan at
  `.claude/plans/F-199-design.md:94` through
  `.claude/plans/F-199-design.md:100`. The current binding contract activates
  the existing additive rich values without adding a type, field, entrypoint,
  binding method, or dependency at `docs/hld/10-bindings-spec.md:642` through
  `docs/hld/10-bindings-spec.md:658`. The incremental feature commit changes no
  Cargo manifest or lockfile, so it adds no crate edge or unnamed dependency.
- **Harness and delivery records, 0 findings**: F-199's AS_BUILT record names
  the same four raw scores, complete corpus evidence, risk riders, and 49 of 49
  unchanged result at `docs/sprints/AS_BUILT.md:9919` through
  `docs/sprints/AS_BUILT.md:9929`. F-199 is done in the sprint and backlog at
  `docs/sprints/CURRENT_SPRINT.md:43` and
  `docs/sprints/BACKLOG.md:401`. Its single tracker entry agrees on sprint,
  size, estimate, actual, date, and scope at
  `docs/sprints/SPRINT_TRACKER.md:343`. The backlog summary remains consistent
  with six of seven M20 stories complete and one pending at
  `docs/sprints/BACKLOG.md:38`.
- **Interaction, duplication, layering, deps, surface, and structure, 0
  findings**: F-199 consumes the existing shared multilingual representation
  rather than introducing a second shaper or line breaker. No new reverse edge,
  feature flag, public abstraction, or test binary appears in its integrated
  delta. The latest feature review reports zero defects, zero smells, and zero
  nitpicks at `.claude/reviews/F-199-working-pass-3.md:1` through
  `.claude/reviews/F-199-working-pass-3.md:19` and records the interaction and
  risk-rider evidence at `.claude/reviews/F-199-working-pass-3.md:21` through
  `.claude/reviews/F-199-working-pass-3.md:75`.
