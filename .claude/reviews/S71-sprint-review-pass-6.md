# S71 sprint review, pass 6

**Reviewed**: full integrated dependency prefix on `sprint/s71` at
`a7ee38becd5338df7cf9ee37b0e86e25148d1c6f` against merge base
`fd069fd460fec11b27c2f6eb1004d4a9ee9bcade`, 95 files and 32,062 changed
lines, comprising 26,028 additions and 6,034 deletions, crates: `oxml-media`,
`oxml-opc`, `rdocx-layout`, `rdocx-oxml`, `rdocx`, `rpptx`
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

**Bound extension**: explicit user-directed dependency-prefix review. Pass 5
was the first review at the current F-246 boundary. Pass 6 is the second review
at that boundary and remains within its three-pass remediation sequence.

## Blocking

### B1, rich fragment style conflicts unwind instead of returning an atomic error
`crates/rdocx/src/field.rs:1654`
`crates/rdocx/src/document.rs:9914`
`crates/rdocx/tests/regression_test.rs:13474`

The pass 5 remediation correctly makes all three direct document-merge variants
validate the combined style graph on their staged candidates. The rich-fragment
importer is itself a fallible `Result` path, but it calls the public
`insert_document` wrapper. That wrapper now converts a combined style-graph
error into a panic with `expect`.

Two independently valid inputs reach this path. A destination may retain
`Normal` as its sole paragraph default while a fragment selects its own
`SourceDefault` as the sole paragraph default. The collision remapper renames
the fragment's differing `Normal` style but does not reconcile default status.
The combined graph then has two paragraph defaults, validation returns an
error, and `insert_document` unwinds out of `Document::mail_merge_rich` instead
of returning `Err`. This contradicts the established contract that invalid
fragments and allocation or validation failures fail atomically
(`docs/hld/10-bindings-spec.md:617`).

The adjusted repeated-fragment fixture remains valid and proves reciprocal
style-link remapping, numbering references, and the final style graph. The
existing atomic-error matrix does not include a style-graph conflict, so both
tests pass without reaching this unwind.

Use a fallible staged insertion helper from the rich importer, or perform an
equivalent combined-graph preflight there, and propagate the error. Add a
regression with a valid fragment that selects a different paragraph default.
It must prove `mail_merge_rich` returns `Err` without unwinding and leaves the
template bytes unchanged. Keep the repeated rich-fragment collision regression
green to prove valid identifier and reciprocal-link remapping still succeeds.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M23 end gate requires all five private references to be generated from a
blank public facade, reopen without repair, match the required package
semantics and reviewed deterministic visual thresholds, produce identical DOCX
bytes on repeated generation, and report no unexplained preservation-only
fallback (`docs/hld/14-development-backlog.md:2214`). The gate does not hold at
this dependency-prefix boundary. F-247 and F-248 remain pending
(`docs/sprints/CURRENT_SPRINT.md:37`), and B1 leaves a previously completed
fallible fragment-import contract broken by the F-246 remediation.

The pass 5 direct-merge finding is otherwise closed. The focused conflicting
default and reciprocal-link test passes for append, append-with-break, and
insertion, including byte-identical receiver state after each rejection. The
repeated rich-fragment collision test and existing invalid-rich-input matrix
also pass. The complete `rdocx` regression binary passed 354 tests with three
declared ignores. `cargo check -p rdocx --all-targets`, formatting, prose,
generated-skill drift, and diff checks passed. The deterministic hash harness
matched all 49 reviewed entries. None of that coverage supplies the missing
style-conflict fragment case described by B1.

## Not found

- **Pass 5 remediation**: direct append, append-with-break, and insertion now
  reject invalid combined style graphs before the staged candidate is
  published. Their receiver remains unchanged.
- **Rich-fragment success interaction**: the revised collision fixture retains
  two independently remapped occurrences, reciprocal paragraph and character
  links, numbering style references, and a valid final style graph.
- **Duplication**: the remediation reuses the F-246 graph validator and adds no
  second style-validation owner.
- **Layering and dependencies**: no manifest changed. No `oxml-*` crate gained
  an `rdocx-*` or `rpptx-*` dependency, and no dependency lacks a named
  consumer.
- **Harness**: all 49 entries match. The only baseline changes remain the two
  declared F-249 document XML entries recorded in the completion log.
- **Documentation and surface**: the remediation adds no public API and does
  not change the approved HLD behavior. The defect is an implementation
  interaction with the existing rich mail-merge contract, not missing planned
  surface.
