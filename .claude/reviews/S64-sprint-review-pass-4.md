# S64 sprint review, pass 4

**Reviewed**: dirty `sprint/s64` at
`984d14b3518bab22e5a219064cdb5035f222c386` plus the five-file, 2,922-line
pass-3 remediation against
`0582da0a38886f5ceeb65ab9afcd0797f6fa14b0`, 80 files, 17,337 changed
lines, crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`, `oxml-drawing`,
`oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`, `oxml-sml`,
`rdocx-wasm`, `rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`,
`rpptx-render`, `rpptx-wasm`, and `rpptx`
**Verdict**: 4 blocking, 1 should-fix, 0 nice-to-have

Pass 4 exceeds the command's default three-pass bound. It was run because the
implementing session explicitly requested an additional pass after the pass-3
defects and supplied a new remediation diff for review.

## Blocking

### B1, the corrected manifest does not record the claimed no-repair observation

`crates/rpptx/tests/integration.rs:9416`

The corrected manifest records only the PowerPoint version, two build strings,
and five artifacts. Unlike the legacy manifest at
`crates/rpptx/tests/integration.rs:9340`, it has no signed or no-repair field.
The corrected gate checks those build strings and artifact hashes at
`crates/rpptx/tests/integration.rs:9745`, but it never consumes evidence that
PowerPoint opened the corrected signed source without repair.

This contradicts the HLD statement that the embedded manifest records the
signed source's no-repair observation at
`docs/hld/12-testing-strategy.md:874` and the milestone requirement that the
manifest pin the no-repair signed source at
`docs/hld/14-development-backlog.md:1901`. Cryptographic signature validation
after reopening with Rust is useful, but it does not establish what PowerPoint
reported during the manual open. The fix must add and assert an explicit
recorded no-repair observation for the exact corrected signed source hash,
including enough provenance to distinguish it from the legacy observation.

### B2, the representative gate and a normal test still depend on one Mac's PowerPoint installation

`crates/rpptx/tests/integration.rs:3361`

The authentic SmartArt loader reads layout, quick-style, and colour resources
from a fixed `/Applications/Microsoft PowerPoint.app/...` root at
`crates/rpptx/tests/integration.rs:3416`. The corrected signature-surrogate
proof unconditionally reaches that loader through
`m21_representative_unsigned_deck_bytes(true)` at
`crates/rpptx/tests/integration.rs:9785`, even when the five hash-pinned oracle
artifacts are supplied through the portable environment directory. The
non-ignored representative round-trip test also reaches it at
`crates/rpptx/tests/integration.rs:9070` and fails before testing anything on a
Linux host, a Mac without PowerPoint, or a Mac with a different resource
installation.

The one configured oracle directory therefore is not sufficient to reproduce
the corrected gate, contrary to `docs/hld/12-testing-strategy.md:870`. It also
makes manually installed PowerPoint resources a requirement of the normal
all-features integration suite. The fix must keep external authentic resources
behind an explicit test-only input and availability boundary, while preserving
their SHA-256 checks. Normal tests must either use a self-contained source or
skip only the external-resource portion explicitly. The corrected oracle gate
must be runnable from its declared environment inputs without an undisclosed
machine path.

### B3, the movie predicate has no real geometry or paint sensitivity and takes text success as a literal

`crates/rpptx/tests/integration.rs:9853`

All three aligned movie frames are extracted and compared, but the loop checks
only Rust's exact visible tokens at
`crates/rpptx/tests/integration.rs:9874`. It obtains the PowerPoint token vector
from the static PDF rather than the sampled movie at
`crates/rpptx/tests/integration.rs:9878`, then calls the visual gate with literal
`true` at `crates/rpptx/tests/integration.rs:9890`. Unlike the static loop, the
movie loop never shifts a real frame, solidifies a real region, mutates real
source text, or recomputes the complete predicate after any isolated mutation.

The ordinary source-sensitivity test only proves that the encoded animation
bytes change at `crates/rpptx/tests/integration.rs:9283`. It does not prove that
each aligned PowerPoint comparison rejects the change. A wrong movie glyph or
paint result with unchanged coarse bands can therefore remain above the 0.45
regional floor without an exact movie-side text predicate or a calibrated
sensitivity test. This conflicts with the declared movie boundary and
cross-output mutation coverage at `docs/hld/12-testing-strategy.md:899`. The
fix must derive the Boolean passed to the movie gate from checked evidence and
show that isolated real text, geometry, and paint mutations fail the recomputed
predicate for the sampled frames.

### B4, the corrected notes gate never applies its ink boundary to Rust output

`crates/rpptx/tests/integration.rs:9946`

The corrected gate renders all three Rust notes pages, but it checks only their
count. The following loop computes the one-band monochrome boundary solely for
the PowerPoint pages at `crates/rpptx/tests/integration.rs:9957`. Its geometry
and paint sensitivity checks merely show that mutating a PowerPoint raster
changes that same raster. They do not compare the mutated result through a
Rust-versus-oracle notes predicate.

Rust notes can consequently acquire an extra ink band, lose the bounded
geometry, or move substantially while exact extracted tokens still pass at
`crates/rpptx/tests/integration.rs:9930`. That does not establish the declared
one bounded monochrome ink region per notes page at
`docs/hld/12-testing-strategy.md:906`. The fix must apply the bounded-ink
predicate to each Rust and PowerPoint notes page and prove a real isolated
mutation fails that acceptance predicate.

## Should-fix

### S1, the legacy non-acceptance regression still requires unrelated oracle bundles

`crates/rpptx/tests/integration.rs:10022`

The legacy test is correctly ignored and returns immediately after asserting
the blank PowerPoint page versus Rust fallback classification at
`crates/rpptx/tests/integration.rs:10119`. Before reaching that classification,
however, it requires four environment inputs and the signed source, unsigned
source, two static PDFs, movie, notes PDF, and handout PDF at
`crates/rpptx/tests/integration.rs:10045`. Only the minimal source and static
PDF are needed for the documented non-acceptance regression at
`docs/hld/12-testing-strategy.md:911`. Removing the unrelated setup would make
the regression smaller and independently reproducible without changing its
classification.

## Nice-to-have

None. Count: 0.

## Pass-3 closure

Pass-3 B1's page coverage is closed. The corrected signed source has three
pages and one SmartArt graph on page three at
`crates/rpptx/tests/integration.rs:9793`. The static loop renders and compares
all three pages at `crates/rpptx/tests/integration.rs:9807`, including real
shifted-raster and solid-paint rejection on every page at
`crates/rpptx/tests/integration.rs:9829`. Exact PowerPoint token vectors cover
all three pages at `crates/rpptx/tests/integration.rs:9796`, and exact Rust
vectors are checked at `crates/rpptx/tests/integration.rs:9821`. The source-built
round-trip also proves parsed SmartArt relationships and exact three-node text
at `crates/rpptx/tests/integration.rs:9088`.

Pass-3 B2's exact token helper is closed for static, notes, and handout text.
`m21_tokens_match` now compares the complete normalized vector at
`crates/rpptx/tests/integration.rs:8428`, and its extra, duplicate,
token-containing, and reordered sensitivities are asserted at
`crates/rpptx/tests/integration.rs:8112`. B3 and B4 above concern missing
movie-side and notes-raster acceptance coverage, not vector equality itself.

The corrected source and all four directly bound PowerPoint outputs have exact
SHA-256 values at `crates/rpptx/tests/integration.rs:9428`. The signature
surrogate proof allowlists only the two signature parts, their content-type
entries, the root origin relationship, and the origin-owned signature
relationship while requiring every non-signature part and relationship to be
identical at `crates/rpptx/tests/integration.rs:9626`. The signed source drives
Rust semantic and render output at `crates/rpptx/tests/integration.rs:9784`.

Pass-2 B3 remains closed. `DeterministicTimelineFrame` has exactly its original
three public fields at `crates/rpptx/src/lib.rs:142`, and its construction adds
no field at `crates/rpptx/src/lib.rs:3212`. The bindings HLD retains the same
contract at `docs/hld/10-bindings-spec.md:941`.

Pass-1 S1 remains closed. M21 reports 15 done and zero pending at
`docs/sprints/BACKLOG.md:39`, and the total row agrees at
`docs/sprints/BACKLOG.md:42`.

## Milestone gate

The M21 gate at `docs/hld/14-development-backlog.md:1896` does not yet hold.
The corrected signed deck now contains supported visible authentic SmartArt,
all three static pages are compared, exact token vectors are present, the
three movie frames are aligned, notes and handout cardinalities are correct,
the artifacts are hash-pinned, and the signature-only package delta is exact.
The missing corrected no-repair record, hidden machine-local SmartArt resource
dependency, incomplete movie sensitivity, and one-sided notes ink predicate
still block sprint closure and release.

## Not found

- No public API addition, production dependency change, feature-flag change,
  crate-layering violation, or new F-224/F-225 interaction defect was found in
  the five-file pass-3 remediation.
- No GUI, PowerPoint launch, printer action, tag, push, or publication side
  effect is performed by the corrected gate. Its external oracle artifacts are
  read only from the configured directory at
  `crates/rpptx/tests/integration.rs:9762`.
- No unexplained deterministic hash-harness delta was introduced. The HLD
  retains the unchanged 49-entry manifest at
  `docs/hld/12-testing-strategy.md:862`.
- The audio-poster mask remains confined to its declared page-one rectangle at
  `crates/rpptx/tests/integration.rs:8153`, and pages two and three are unmasked
  at `crates/rpptx/tests/integration.rs:9811`.
- The corrected handout checks one PowerPoint and one Rust page, exact token
  vectors, all three thumbnail bounds, the 0.05 normalized geometry limit, and
  a boundary mutation at `crates/rpptx/tests/integration.rs:9979`.
- No panic or cleanup path can produce a false positive in the corrected gate.
  Hashes are checked before parsing at `crates/rpptx/tests/integration.rs:9778`,
  subprocess status is asserted at `crates/rpptx/tests/integration.rs:9857`, and
  the temporary directory is removed after a passing run at
  `crates/rpptx/tests/integration.rs:9998`.
