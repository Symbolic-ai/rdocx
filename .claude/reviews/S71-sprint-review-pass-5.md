# S71 sprint review, pass 5

**Reviewed**: full integrated dependency prefix on `sprint/s71` at
`3162b809d5310fdb7e18b5a4ab6d005c95485074` against merge base
`fd069fd460fec11b27c2f6eb1004d4a9ee9bcade`, 94 files and 32,354 changed
lines, comprising 26,089 additions and 6,265 deletions, crates: `oxml-media`,
`oxml-opc`, `rdocx-layout`, `rdocx-oxml`, `rdocx`, `rpptx`
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

**Bound extension**: scheduled dependency-prefix boundary. Passes 3 and 4
reviewed the prior F-244 and F-245 boundary. F-246 has now enlarged the
integrated prefix with the validated style graph, so pass 5 is the first review
at this new boundary and begins its bounded remediation sequence.

## Blocking

### B1, document merges can publish an invalid style graph
`crates/rdocx/src/document.rs:9856`
`crates/rdocx/src/document.rs:9994`
`crates/rdocx/src/style.rs:429`

The three public document-merge variants copy every incoming style whose id is
absent and commit the candidate without running the new complete-graph
validation. Two individually valid documents can therefore become invalid
when combined. For example, a destination can retain `Normal` as its default
paragraph style while the source selects a source-only paragraph style as its
default. `merge_styles` skips the source's nondefault `Normal`, copies its
source-only default, and publishes two paragraph defaults, which the validator
at `style.rs:429` rejects only if the caller checks afterward. A style-id
collision involving one side of an incoming reciprocal paragraph and
character link can likewise publish a one-way link.

This violates the sprint contract that style graphs have atomic validation and
the F-246 completion record's claim that mutations validate a complete
candidate before publication. Reconcile or remap incoming defaults and graph
edges, then validate the full candidate before every append, append-with-break,
and insertion commit. Regression coverage must combine independently valid
documents with both a conflicting default and a reciprocal-link collision,
and prove either a valid deterministic merge or failure with the receiver
unchanged.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M23 end gate requires all five private references to be generated from a
blank public facade, reopen without repair, match the required package
semantics and reviewed deterministic visual thresholds, produce identical
DOCX bytes on repeated generation, and report no unexplained preservation-only
fallback (`docs/hld/14-development-backlog.md:2214`). The gate does not hold at
this dependency-prefix boundary. F-247 and F-248 remain pending
(`docs/sprints/CURRENT_SPRINT.md:37`), and B1 leaves the completed-prefix style
graph invariant open.

The newly integrated F-246 evidence does establish its direct authoring path.
The source-built style differential, invalid-mutation atomicity, save-and-reopen,
and live-reference removal gates are recorded as passing
(`docs/sprints/AS_BUILT.md:12548`). The integrated full gate is recorded at
`11f2a2fe653b56c3cc2a3231ff3195c58ede1e72`, including pinned LibreOffice and
Poppler riders and all package archives below 10 MiB
(`docs/sprints/AS_BUILT.md:12552`). Those tests do not combine two valid public
style graphs through the document merge APIs, so they do not close B1.

## Not found

- **Duplication**: the pass 3 duplicate theme-language setter remains removed.
  F-246 extends the existing style, document, resolver, and table owners rather
  than adding a second style model.
- **Layering and dependencies**: no manifest changed. No `oxml-*` crate gained
  an `rdocx-*` or `rpptx-*` dependency, and no new dependency lacks a named
  consumer.
- **Harness**: the baseline contains only the two declared F-249
  `word/document.xml` changes, with the reviewed identifier-allocation reason
  recorded in `scripts/hash_baseline.json:53`. F-244, F-245, and F-246 each
  record all 49 entries unchanged.
- **Documentation**: every HLD file listed by F-246's design plan changed, and
  the sprint, backlog, tracker, capability matrix, README, and completion log
  agree on the five completed and two pending stories.
- **Public surface**: every new F-246 Rust style method and the `StyleType`
  re-export is called for by the approved plan. Python, WASM, and CLI add no
  unplanned mutation surface.
