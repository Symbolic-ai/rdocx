# S56 sprint review, pass 2

**Reviewed**: `sprint/s56` at
`71531edecfb3949512f357e3e039f3230fac448d` against merge base
`92659e7ba3742aab888a8d5603e42560ff3398fc`, 107 files and 29,182 changed
lines, crates: `rdocx`, `rdocx-cli`, `rdocx-html`, `rdocx-layout`,
`rdocx-opc`, `rdocx-oxml`, `rdocx-pdf`, `rdocx-py`, `rdocx-wasm`, and
`rpptx-py`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have
**Dispositions**: 0 fix-now, 0 tracked-follow-up, 1 human-action, 0 refuted

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Pass 1 remediation

S1 is fully resolved. Encrypted, RTF, ODT, and EPUB path saves now call one
crate-private helper. That helper retains create-new sibling staging, complete
buffer writes, file synchronization, portable replacement, and failed-stage
cleanup (`crates/rdocx/src/document.rs:5725`). A repository search finds no
second production staging implementation.

The existing failure-atomic regressions still exercise the consolidated path.
RTF, EPUB, and ODT each exhaust all 128 staging names and prove that the prior
destination bytes survive (`crates/rdocx/src/lib.rs:221`,
`crates/rdocx/src/epub.rs:4106`, `crates/rdocx/src/odt.rs:7308`). The encrypted
save regression proves that serialization failure preserves the destination and
that a subsequent atomic replacement reopens correctly
(`crates/rdocx/src/document.rs:6491`). All four focused tests passed at the
reviewed SHA, as did `cargo clippy -p rdocx --all-targets --all-features -- -D
warnings`.

## Human action

### H1, the stable release remains behind its separate final approval

`.claude/commands/release.md:87`
`.claude/plans/F-X055-design.md:134`

No release mutation belongs in this review. F-X055 remains reviewed in sprint
state and in progress in the delivery trackers. `/release v0.10.0` must present
the exact release evidence and obtain a new explicit approval immediately before
its first external mutation. Earlier sprint authorization does not satisfy this
boundary.

**Disposition**: human-action after current-HEAD full verification is recorded.

## Milestone gate

The M18 gate is: "each format round-trips at its declared fidelity level, and
every lossy conversion records a diagnostic naming what it dropped"
(`docs/hld/14-development-backlog.md:1457`).

The technical gate holds on the integrated result. The ODT writer reopens its
output through the F-179 reader and compares body order, formatting, lists,
tables, image bytes, and dimensions
(`crates/rdocx/tests/integration_test.rs:546`). Its loss-matrix regression keeps
supported siblings while checking exact diagnostics
(`crates/rdocx/src/odt.rs:6769`). The EPUB structure regression proves that the
spine and navigation follow source outline order
(`crates/rdocx/src/epub.rs:3987`), and the checksum-pinned EPUBCheck 5.3.0 test
validates the source-built publication (`crates/rdocx/src/epub.rs:5543`). The
SVG golden rasterises a representative shared-layout page at 150 dpi, requires
at least 0.99 SSIM against the PNG backend, and proves that a one-point
perturbation fails (`crates/rdocx/src/svg.rs:2208`).

The ordered-reader numbering interaction is also covered at the combined
exporter boundary. The ODT and EPUB regressions both prove that a
producer-defined numbering format does not acquire an invented marker. Those
tests and the ordered-reader source-order regression passed at the reviewed
SHA. The current-HEAD hash harness also passed with all 49 entries unchanged.
Publication and release-bound contribution comments remain pending behind H1,
so this pass does not claim that F-X055 or the sprint is ready to close.

## Not found

- `interaction`: ODT and EPUB retain the producer-defined numbering contract,
  SVG consumes immutable shared layout output, and ordered-reader additions do
  not introduce shared mutable conversion state.
- `duplication`: the four native path-save families share the single staging
  implementation in `document.rs`. S1's duplicate ODT and EPUB loops are gone.
- `layering`: no `oxml-*` manifest changed, and Cargo metadata reports no
  `oxml-*` dependency on an `rdocx-*` or `rpptx-*` crate.
- `harness`: all four completed AS_BUILT records declare 49 unchanged entries,
  no baseline file changed, and the current-HEAD harness check passed 49 of 49.
- `gate`: the named round-trip, diagnostic, external-validator, and golden tests
  exercise the M18 fidelity boundary. Release approval remains a separate human
  action.
- `docs`: the plan-listed HLD sections match the exporter, ordered-reader,
  dependency, test, atomic-save, and prepared-release behavior. Delivery
  ledgers consistently retain F-X055 in progress.
- `deps`: runtime `base64` has the private native SVG renderer as its named
  consumer. Exact `resvg` 0.48.1 is development-only validation infrastructure,
  and ODT and EPUB reuse the existing workspace `zip` dependency.
- `surface`: ODT, EPUB, SVG, and ordered-reader public additions match their
  approved designs. Python, WASM, CLI, Presentation, and public `oxml-pdf`
  surfaces gained no format entry point outside the declared scope.
