# S64 sprint review, pass 3

**Reviewed**: dirty `sprint/s64` at
`984d14b3518bab22e5a219064cdb5035f222c386` plus the five-file, 1,627
inserted-line pass-2 remediation against
`0582da0a38886f5ceeb65ab9afcd0797f6fa14b0`, 80 files, 16,066 changed
lines, crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`, `oxml-drawing`,
`oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`, `oxml-sml`,
`rdocx-wasm`, `rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`,
`rpptx-render`, `rpptx-wasm`, and `rpptx`
**Verdict**: 2 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, the three-page static gate compares only the first frame

`crates/rpptx/tests/integration.rs:9427`

The test proves that the signed static PDF has three pages, but Rust rendering
and both signed and unsigned PowerPoint normalization select only page index
zero at `crates/rpptx/tests/integration.rs:9481`. The helper hard-codes
`-f 1`, `-l 1`, and `-singlefile` at
`crates/rpptx/tests/integration.rs:8064`. No later comparison visits pages two
or three.

This omits two thirds of the representative deck from the declared static
fidelity gate. In particular, `transfer_smartart_slide_from` appends its source
slide at `crates/rpptx/src/lib.rs:2105`, so the SmartArt slide added by the
representative builder at `crates/rpptx/tests/integration.rs:8801` is the third
slide and never receives a static text, geometry, or raster comparison. The
assertion that signed and signature-free static outputs are identical at
`crates/rpptx/tests/integration.rs:9493` likewise proves only their first pages,
while the HLD claims their normalized static output is identical at
`docs/hld/12-testing-strategy.md:881`.

The fix must rasterize and compare every static page from Rust, the signed
PowerPoint PDF, and the signature-free PowerPoint PDF. Each page needs its
declared text, band geometry, and regional-raster predicate. The combined gate
must therefore exercise the actual SmartArt frame rather than infer it from a
page count or handout thumbnail geometry.

### B2, the declared exact-text predicate accepts subsequences and unordered output

`crates/rpptx/tests/integration.rs:8358`

`m21_text_occurs_in_order` repeatedly uses substring `find`. It accepts extra
or duplicated text between expected terms and accepts a required term inside a
different word. The static and animation gates therefore do not enforce the
exact text order required at `docs/hld/14-development-backlog.md:1904`.
The source mutation now exercises a real changed presentation, but it covers
only one reordered title and does not expose these false positives at
`crates/rpptx/tests/integration.rs:9524`.

The gap also reaches the other output classes. PowerPoint animation order is
inferred from the static PDF with the same subsequence helper and a band count
at `crates/rpptx/tests/integration.rs:9610`. Rust notes and handout output only
check that each expected word occurs somewhere, without order or multiplicity,
at `crates/rpptx/tests/integration.rs:9745`. Unexpected, duplicated, or
misordered text can pass while the low regional SSIM threshold remains above
0.45.

The fix must compare normalized exact token or run sequences, including
multiplicity and rejection of unexpected visible text, on both Rust and
PowerPoint evidence at every applicable output boundary. Sensitivity must add
or duplicate real source text and prove the recomputed predicate fails for
static, animation, notes, and handout output.

## Should-fix

None. Count: 0.

## Nice-to-have

None. Count: 0.

## Pass-2 closure

Pass-2 B1's signature-surrogate and portability requirements are otherwise
closed. The embedded manifest pins both source hashes, both no-repair
observations, PowerPoint 16.104 build identities, direct signed outputs, and
derived signature-free outputs at `crates/rpptx/tests/integration.rs:9134`.
Artifacts come from explicit environment paths at
`crates/rpptx/tests/integration.rs:9334`. The comparison allowlists exactly the
two signature parts, content-type additions, root origin relationship, and
origin-owned signature relationship while requiring every other part and
relationship to agree at `crates/rpptx/tests/integration.rs:9198`. The signed
canonical source drives all Rust semantic and render checks at
`crates/rpptx/tests/integration.rs:9383`. Signed and signature-free no-repair
evidence and output-source bindings are recorded, and no developer-local path
remains.

Pass-2 B2's real-input geometry and raster sensitivity requirements are closed.
Static output is shifted and recomputed at
`crates/rpptx/tests/integration.rs:9518`, a real source text mutation is rendered
at `crates/rpptx/tests/integration.rs:9524`, and a real regional paint mutation
is recomputed at `crates/rpptx/tests/integration.rs:9552`. The same three checks
run for every animation sample at `crates/rpptx/tests/integration.rs:9623`.
B2 above is the remaining exactness failure in the predicate.

Pass-2 B3 is closed exactly. `DeterministicTimelineFrame` retains its original
three public fields at `crates/rpptx/src/lib.rs:142`, its constructor has no
added field at `crates/rpptx/src/lib.rs:3212`, and the bindings HLD retains the
original contract at `docs/hld/10-bindings-spec.md:941`.

Pass-1 S1 remains closed. M21 reports 15 done and zero pending at
`docs/sprints/BACKLOG.md:39`, and the overall counts agree at
`docs/sprints/BACKLOG.md:42`.

## Milestone gate

The M21 gate at `docs/hld/14-development-backlog.md:1896` does not yet hold.
The signed canonical deck, signature-free surrogate proof, PowerPoint pins,
no-repair observations, output hashes, audio mask, and real mutation machinery
are present. The gate still omits two static frames, including the SmartArt
slide, and its text checks are not exact. Release and sprint closure remain
blocked.

## Not found

- No signature-surrogate allowlist gap, output-source binding gap, GUI or
  printer side effect, hard-coded developer path, dependency change, feature
  flag, or production-layering change was found in the pass-2 remediation.
- No cleanup or panic-path defect was found in the test-only subprocess and
  temporary-artifact flow. The comparison directory is removed after a passing
  run unless retention is explicitly requested at
  `crates/rpptx/tests/integration.rs:9767`.
- No unexplained hash expectation change was introduced. The 49-entry harness
  remains declared unchanged at `docs/hld/12-testing-strategy.md:862`.
- The audio mask remains bounded to the declared poster rectangle at
  `crates/rpptx/tests/integration.rs:8112`, with the Rust red-poster and
  PowerPoint suppressed-poster observations asserted at
  `crates/rpptx/tests/integration.rs:9558`.
- Notes and handout page sizes, representative bounds, thumbnail bounds, and
  normalized 0.05 geometry checks are present at
  `crates/rpptx/tests/integration.rs:9690`. B2 concerns text exactness, not those
  geometry predicates.
