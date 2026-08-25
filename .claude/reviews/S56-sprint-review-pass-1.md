# S56 sprint review, pass 1

**Reviewed**: `sprint/s56` at
`ce9602913e19e3b6a30fc9804162fc091d6a4b06` against merge base
`92659e7ba3742aab888a8d5603e42560ff3398fc`, 106 files and 29,031 changed
lines, crates: `rdocx`, `rdocx-cli`, `rdocx-html`, `rdocx-layout`,
`rdocx-opc`, `rdocx-oxml`, `rdocx-pdf`, `rdocx-py`, `rdocx-wasm`, and
`rpptx-py`
**Verdict**: 0 blocking, 1 should-fix, 0 nice-to-have
**Dispositions**: 1 fix-now, 0 tracked-follow-up, 1 human-action, 0 refuted

## Blocking

None.

## Should-fix

### S1, the two new archive writers duplicate the atomic-save implementation

`crates/rdocx/src/odt.rs:146`
`crates/rdocx/src/epub.rs:3789`

F-180 and F-181 each added the same 32-line staging loop under a different
private name. Both allocate the same sibling temporary path, retry the same 128
collisions, write and sync the complete byte buffer, call the same portable
replacement primitive, and remove the same failed staging file. This duplicates
the existing RTF and encrypted-save mechanism as well, so fixes to atomic save
behavior now have four production sites to reconcile. Replace the copies with
one crate-private concrete helper used by the applicable save paths. The helper
must retain create-new staging, file synchronization, portable replacement,
failure cleanup, and tested preservation of an existing destination.

**Disposition**: fix-now.

## Nice-to-have

None.

## Human action

### H1, the stable release remains behind its separate final approval

`.claude/commands/release.md:87`
`.claude/plans/F-X055-design.md:134`

No release mutation belongs in this review. F-X055 is correctly reviewed in
the sprint state and remains in progress in the delivery trackers. After S1 is
fixed, full verification and a clean sprint review cover the final prepared
SHA, `/release v0.10.0` must present the exact release evidence and obtain a
new explicit approval immediately before its first external mutation. Earlier
sprint authorization does not satisfy this boundary.

**Disposition**: human-action after a clean pass is recorded.

## Milestone gate

The M18 gate is: "each format round-trips at its declared fidelity level, and
every lossy conversion records a diagnostic naming what it dropped"
(`docs/hld/14-development-backlog.md:1457`).

The technical gate holds on the integrated result. The ODT writer gate reopens
through the F-179 reader and compares body order, effective formatting, lists,
table spans, media bytes, and dimensions in
`odt_writer_round_trip_preserves_supported_document_content`
(`crates/rdocx/tests/integration_test.rs:546`). Its complete loss matrix keeps
supported siblings in
`unsupported_document_content_is_diagnosed_without_dropping_supported_odt_siblings`
(`crates/rdocx/src/odt.rs:6797`). The EPUB structure regression proves spine
and navigation order against the document outline
(`crates/rdocx/src/epub.rs:4015`), and the checksum-pinned EPUBCheck 5.3.0 gate
validates the source-built publication (`crates/rdocx/src/epub.rs:5571`). The
SVG golden rasterises the shared representative page at 150 dpi and compares it
with the PNG backend (`crates/rdocx/src/svg.rs:2208`), while the public
integration gate covers searchable text, geometry, fonts, images, safe links,
and clipping (`crates/rdocx/tests/integration_test.rs:51`).

The four completed S56 records all declare an unchanged 49-of-49 harness, and
the integrated full verification records the same result
(`.claude/scratch/S56-run.json:75`). The earlier M18 RTF, HTML, ODT-reader, and
image-export gates remain recorded in the S54 and S55 completion evidence. S56
therefore supplies the remaining format evidence without an unexplained output
delta. The v0.10.0 publication and release-bound contribution comments remain
deliberately pending behind H1, so this pass does not claim that the release
story or sprint is ready to close.

## Not found

- `interaction`: ODT and EPUB consume the reconciled producer-defined numbering
  behavior without inventing decimal markers. SVG consumes immutable shared
  layout output. Ordered-reader additions preserve established flattened
  accessors, and the combined native surfaces do not share mutable conversion
  state.
- `layering`: no `oxml-*` manifest changed, and Cargo metadata reports no new
  `oxml-*` dependency on an `rdocx-*` or `rpptx-*` crate.
- `harness`: every completed as-built record declares 49 unchanged entries, the
  integrated verification agrees, and no baseline file changed.
- `gate`: the named round-trip, regression, external-validator, and golden tests
  exercise the M18 fidelity boundary rather than asserting it. The pending
  release approval is identified separately as H1.
- `docs`: every plan-listed HLD section describes current exporter, ordered
  reader, dependency, test, and prepared-release behavior. Delivery ledgers
  consistently mark F-180, F-181, F-182, and F-X054 done while retaining
  F-X055 in progress.
- `deps`: runtime `base64` has the private native SVG renderer as its named
  consumer. Exact `resvg` 0.48.1 is development-only validation infrastructure,
  and EPUB and ODT reuse the existing workspace `zip` dependency.
- `surface`: ODT, EPUB, SVG, and ordered-reader public additions match their
  approved stories. Python, WASM, CLI, Presentation, and public `oxml-pdf`
  surfaces gained no format entry point outside the declared scope.

