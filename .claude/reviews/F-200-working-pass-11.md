# F-200, working, pass 11

**Reviewed**: working diff against
`cf7627aa280c65a245dbed8fbd2988e80dae9201`, 22 tracked files with 4,509
insertions and 439 deletions, plus the pass 1 through pass 10 review records
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass 10 D1 is closed for production table paths. Table lowering returns the
  private direction-bearing semantics beside the unchanged public table block,
  cache hits rebuild current source nodes while retaining the resolved
  direction, rendering consumes the matching per-cell sidecar, and the cache
  accounts for every nested semantics vector
  (`crates/rdocx-layout/src/table.rs:151`,
  `crates/rdocx-layout/src/engine.rs:1998`,
  `crates/rdocx-layout/src/engine.rs:2504`,
  `crates/rdocx-layout/src/engine.rs:2610`,
  `crates/rdocx-layout/src/paginator.rs:3313`).
- Pass 10 D1 is also closed for production header and footer cache paths. Every
  cached variant retains an aligned private direction vector, its allocation
  is included in entry accounting, `SharedSection` carries the vectors beside
  the unchanged public content, and first, even, and default selection passes
  the matching vector to paragraph rendering
  (`crates/rdocx-layout/src/engine.rs:695`,
  `crates/rdocx-layout/src/engine.rs:3466`,
  `crates/rdocx-layout/src/engine.rs:6008`,
  `crates/rdocx-layout/src/paginator.rs:121`,
  `crates/rdocx-layout/src/paginator.rs:1115`,
  `crates/rdocx-layout/src/paginator.rs:3188`).
- Pass 10 D2 is closed for both note streams. Registration retains each
  original paragraph block, resolved base direction, and exact flattened line
  range. Footnote and endnote drawing then renders the selected original line
  slices through the same logical reconstruction used by other Word stories,
  while keeping their existing vertical positions and revision ranges
  (`crates/rdocx-layout/src/notes.rs:89`,
  `crates/rdocx-layout/src/notes.rs:174`,
  `crates/rdocx-layout/src/paginator.rs:1450`,
  `crates/rdocx-layout/src/paginator.rs:1492`,
  `crates/rdocx-layout/src/paginator.rs:1517`).
- Pass 10 D3 is closed with production-path evidence. The table regression
  enters `Engine::layout_body_table`, the header regression observes actual
  header cache hits, and both assert direction, current source paths, logical
  leader order, and right-to-left visual origins. The package-backed facade
  regression exercises real footnote and endnote parts through PDF and SVG and
  asserts paragraph-logical extraction
  (`crates/rdocx-layout/src/engine.rs:10189`,
  `crates/rdocx-layout/src/engine.rs:10235`,
  `crates/rdocx-layout/src/engine.rs:10275`,
  `crates/rdocx-layout/src/engine.rs:10328`,
  `crates/rdocx/tests/integration_test.rs:5605`,
  `crates/rdocx/tests/integration_test.rs:5645`).
- Cache source normalization remains exact. Paragraph rebinding covers legacy,
  rich, leader, and conditional-hyphen carriers, table cache semantics replace
  only the result-local source and structure fields, and source-less generated
  text remains source-less during logical ranking
  (`crates/rdocx-layout/src/engine.rs:3392`,
  `crates/rdocx-layout/src/engine.rs:3411`,
  `crates/rdocx-layout/src/engine.rs:2504`,
  `crates/rdocx-layout/src/paginator.rs:2751`,
  `crates/rdocx-layout/src/paginator.rs:2828`).
- The pass 1 through pass 9 parser and rendering closures remain intact.
  Direction occurrences retain valid and malformed ordering, natural levels
  survive explicit overrides and hyphenation, emitted-item provenance
  distinguishes fields, markers, leaders, and generated hyphens, and private
  drawing reflow carries the resolved base without changing public shapes
  (`crates/rdocx-oxml/src/properties.rs:2732`,
  `crates/rdocx-oxml/src/properties.rs:2799`,
  `crates/rdocx-oxml/src/properties.rs:2860`,
  `crates/rdocx-layout/src/paginator.rs:2305`,
  `crates/rdocx-layout/src/paginator.rs:2647`,
  `crates/rdocx-layout/src/paginator.rs:2992`).
- No new panic on public input, unchecked public indexing, arithmetic overflow,
  suppressed parser error, reverse dependency edge, module, dependency, trait,
  generic, feature flag, or public compatibility break was found. Public
  `Section`, `HeaderFooterContent`, `NoteLayout`, and `layout_table` retain their
  established shapes and entrypoints
  (`crates/rdocx-layout/src/paginator.rs:99`,
  `crates/rdocx-layout/src/paginator.rs:131`,
  `crates/rdocx-layout/src/notes.rs:41`,
  `crates/rdocx-layout/src/table.rs:130`).
- The five plan-listed HLD files describe the current shared logical and visual
  ordering contract, typed DrawingML direction, intentional pre-1.0 Word
  property additions, and deterministic regression surface without change
  history prose. The quarter-turn vertical approximation remains covered and
  unchanged (`docs/hld/03-architecture.md:127`,
  `docs/hld/05-drawingml-model.md:207`,
  `docs/hld/08-rendering-spec.md:459`,
  `docs/hld/10-bindings-spec.md:642`,
  `docs/hld/12-testing-strategy.md:983`,
  `crates/rpptx-render/src/text.rs:1420`).
- The recorded evidence binds the reviewed behavior to deterministic fonts,
  unchanged 49-of-49 legacy output, byte-identical accepted five-page output,
  affected suites, no-default and WASM portability, documentation, package
  archives, and supply-chain checks. No evidence claim substitutes for the
  production-path regressions above (`.claude/scratch/F-200-progress.md:471`).
