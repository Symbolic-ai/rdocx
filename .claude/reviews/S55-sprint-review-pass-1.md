# S55 sprint review, pass 1

**Reviewed**: `sprint/s55` against
`939d1aedbd0d28824f99316669db3995c76b9b1d`, 41 files, 8,623 insertions and
457 deletions, crates: `oxml-layout`, `rdocx-layout`, and `rdocx`
**Verdict**: 0 blocking, 1 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

### S1, the HTML as-built entry names an API that does not exist

`docs/sprints/AS_BUILT.md:8868`

The completion record says F-178 added `Document::from_html_bytes`, but the
approved and implemented native surface contains only `Document::from_html`
and `Document::open_html`. The record therefore sends future callers toward a
nonexistent method and contradicts the current bindings specification. Remove
the method name from the as-built entry. Do not add another public API merely
to make the record true.

## Nice-to-have

None.

## Milestone gate

The M18 gate is: "each format round-trips at its declared fidelity level, and
every lossy conversion records a diagnostic naming what it dropped."

M18 remains open through S56, so its complete gate is not yet due. The S55
portion holds. `html_import_projects_a_reopenable_word_document` proves the
HTML projection saves and reopens, while
`unsupported_html_css_is_diagnosed_without_dropping_supported_siblings` proves
loss diagnostics retain supported siblings. The exact pinned LibreOffice gate
`odt_reader_matches_pinned_libreoffice_structure` proves the declared ODT
structural boundary. Source-built ODT regressions cover unsupported siblings,
archive and XML bounds, save, and reopen. The integrated 49-entry hash harness
is unchanged.

The sprint-specific performance gate also holds.
`mixed_editor_relayout_reuses_every_safe_unchanged_block_and_page` proves exact
warm and cold structure, provenance, semantic content, and bounded reuse.
Fresh four-pair integrated A/B runs produced 58 pages each. Native cold and
warm ratios topped out at 1.04 and 0.89. Bundled-fallback cold, typing, checked
undo, and table mutation topped out at 1.10, 1.14, 1.14, and 1.14. Every value
is within the required 1.25 budget.

## Not found

- `interaction`: HTML and ODT project through separate private importers into
  the same typed Word tree without sharing mutable conversion state. The
  relayout changes preserve their saved output through the unchanged facade
  boundary.
- `duplication`: no competing archive owner, document model, parser facade, or
  layout cache path was added.
- `layering`: no `oxml-*` crate gained an `rdocx-*` or `rpptx-*` dependency.
  The private paginator input trait has the two current implementations required
  by the structural rule.
- `harness`: every as-built entry declares the unchanged 49-of-49 result, which
  matches the integrated harness run and unchanged baseline file.
- `gate`: the S55 definition of done has named regression, differential,
  benchmark, package, and authenticated GitHub evidence.
- `docs`: every HLD file named by the four approved plans was updated to current
  intent. No contradictory current-state section remains.
- `deps`: `scraper` 0.27 has the private HTML importer as its named consumer.
  The direct `zip` edge reuses the existing workspace dependency for the
  private ODT importer.
- `surface`: the additive native HTML and ODT constructors, result types,
  diagnostics, and error variants are called for by F-178 and F-179. Python,
  WASM, and CLI surfaces remain unchanged.
