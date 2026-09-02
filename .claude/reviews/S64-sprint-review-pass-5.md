# S64 sprint review, pass 5

**Reviewed**: dirty `sprint/s64` at
`984d14b3518bab22e5a219064cdb5035f222c386` plus the five-file, 2,506-line
pass-4 remediation against
`0582da0a38886f5ceeb65ab9afcd0797f6fa14b0`, 80 files, 16,929 changed
lines, crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`, `oxml-drawing`,
`oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`, `oxml-sml`,
`rdocx-wasm`, `rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`,
`rpptx-render`, `rpptx-wasm`, and `rpptx`
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

Pass 5 exceeds the command's default three-pass bound. It was run because the
implementing session explicitly requested another review of the pass-4
remediation.

## Blocking

### B1, the authentic signed package is no longer tied to the representative semantic proof

`crates/rpptx/tests/integration.rs:9792`

The corrected release gate opens the captured authentic signed source, checks
generic validation and signatures, then asserts only three rendered pages and
one SmartArt item on page three. It does not check the package class, comments,
sections, exact audio bytes, timing ownership, SmartArt relationship IDs,
parsed data, layout, style, colour parts, or the three SmartArt data-model
points. It also does not save and reopen that captured source before making the
semantic assertions.

Those detailed checks exist only in the portable test beginning at
`crates/rpptx/tests/integration.rs:9135`, but that test deliberately builds the
different minimal SmartArt fallback package at
`crates/rpptx/tests/integration.rs:9136`. Its signature-only OPC delta at
`crates/rpptx/tests/integration.rs:9138` therefore does not bind the portable
semantic assertions to the hash-pinned authentic source. The authentic
generator is reference-only and merely prints candidate hashes at
`crates/rpptx/tests/integration.rs:9124`. The release gate neither recreates
that package nor compares the captured package against an unsigned authentic
surrogate.

As a result, the tests prove a portable minimal package has the full semantic
combination and a separate captured authentic package has the expected visual
outputs. They do not prove the milestone's one-deck requirement at
`docs/hld/14-development-backlog.md:1896`. A captured deck with missing
comments, sections, exact audio, package variant, or parsed SmartArt
relationships could still pass if its visible outputs and one SmartArt count
remained acceptable. This also contradicts the statement that every Rust
semantic check starts from the canonical signed bytes at
`docs/hld/12-testing-strategy.md:881` and the complete graph claim at
`docs/hld/12-testing-strategy.md:902`.

The fix must assert the full representative semantic contract directly on the
hash-pinned captured signed source, including save and reopen preservation. It
must prove that page three's three visible tokens come from the expected parsed
SmartArt graph and relationships rather than from unrelated visible shapes.
Alternatively, an exact non-signature package comparison may bind those same
assertions to the captured source without reintroducing an installed-resource
dependency into the release gate.

## Should-fix

None. Count: 0.

## Nice-to-have

None. Count: 0.

## Pass-4 closure

Pass-4 B1 is closed. The corrected manifest records `signed=true`,
`signed_open_no_repair=true`, the exact observed active name, the observed
source hash, PowerPoint 16.104 build identities, and every source and output
hash at `crates/rpptx/tests/integration.rs:9403`. The release gate asserts each
provenance link before opening the artifacts at
`crates/rpptx/tests/integration.rs:9743`.

Pass-4 B2 is closed. The fixed `/Applications` SmartArt path remains reachable
only through the explicitly ignored reference writer at
`crates/rpptx/tests/integration.rs:9084`. The non-ignored portable test uses the
self-contained minimal source at `crates/rpptx/tests/integration.rs:9135`, and
the corrected release gate reads only its five files from
`RPPTX_M21_CORRECTED_POWERPOINT_ORACLE_DIR` at
`crates/rpptx/tests/integration.rs:9770`.

Pass-4 B3 is closed. Each of the three movie samples independently records its
time, aligned Rust time, exact observed visible-token vector, and observed ink
band count at `crates/rpptx/tests/integration.rs:9456`. The gate derives its
text Boolean from Rust output and that observation at
`crates/rpptx/tests/integration.rs:9895`. Every frame rejects a real shifted
PowerPoint raster, solid-filled PowerPoint raster, and source-text-mutated Rust
frame through the recomputed predicate at
`crates/rpptx/tests/integration.rs:9906`.

Pass-4 B4 is closed. The notes detector measures exact band cardinality,
normalized component width and height, and monochrome occupancy symmetrically
at `crates/rpptx/tests/integration.rs:8548`. The gate checks all three Rust and
PowerPoint pages with exact per-page tokens and declared page sizes at
`crates/rpptx/tests/integration.rs:9937`. Real geometry and paint mutations on
both sides fail the combined predicate at
`crates/rpptx/tests/integration.rs:10009`.

Pass-4 S1 is closed. The legacy blank-versus-fallback regression now accepts
only one environment-supplied static PDF and its exact hash at
`crates/rpptx/tests/integration.rs:10078`. It performs only the documented
three-page classification before cleanup.

## Milestone gate

The M21 gate at `docs/hld/14-development-backlog.md:1896` does not yet hold.
The provenance, portability, static page coverage, movie observations and
sensitivity, symmetric notes boundary, handout boundary, and isolated legacy
classification are now present. The remaining blocker is that the authentic
signed package has not itself been proven to carry and round-trip the complete
representative semantic combination.

## Not found

- No fresh F-224/F-225 interaction, production dependency, feature flag, crate
  layering, or release-family defect was found in the pass-4 remediation.
- No public API change was introduced. `DeterministicTimelineFrame` retains its
  original three fields at `crates/rpptx/src/lib.rs:142`, and its constructor is
  unchanged at `crates/rpptx/src/lib.rs:3212`.
- All three static pages are rendered and compared at
  `crates/rpptx/tests/integration.rs:9813`. Each page has exact token checks and
  rejects real seven-pixel geometry and solid-paint mutations at
  `crates/rpptx/tests/integration.rs:9827`.
- The handout gate retains exact Rust and PowerPoint tokens, one-page
  cardinality, all three thumbnail bounds, the 0.05 normalized geometry limit,
  and boundary sensitivity at `crates/rpptx/tests/integration.rs:9954` and
  `crates/rpptx/tests/integration.rs:10054`.
- No GUI, printer, fixed developer artifact path, tag, push, or publication
  side effect is present in the normal or corrected M21 gate. External captured
  artifacts remain ignored and environment-directed at
  `crates/rpptx/tests/integration.rs:9740`.
- No unexplained deterministic hash-harness delta was introduced. The HLD
  retains the unchanged 49-entry manifest at
  `docs/hld/12-testing-strategy.md:862`.
- M21 remains 15 done and zero pending at `docs/sprints/BACKLOG.md:39`, and the
  total row agrees at `docs/sprints/BACKLOG.md:42`.
- No cleanup or panic path can produce a false positive in the corrected gate.
  Artifact hashes are checked before parsing at
  `crates/rpptx/tests/integration.rs:9786`, movie subprocess status is asserted
  at `crates/rpptx/tests/integration.rs:9878`, and temporary output is removed
  only after all predicates pass at `crates/rpptx/tests/integration.rs:10073`.
