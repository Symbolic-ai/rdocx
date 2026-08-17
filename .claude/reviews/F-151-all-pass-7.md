# F-151, all, pass 7

**Reviewed**: complete remediated working-tree diff against `HEAD`, 13 files, 1,347 changed lines, with 1,297 additions and 50 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, revision-only hyperlinks lose their link annotation
`crates/rdocx-oxml/src/text.rs:1094`
`crates/rdocx-layout/src/engine.rs:722`
`crates/rdocx-layout/src/paginator.rs:1953`

When a hyperlink contains only a revision wrapper, the paragraph parser takes
the empty-direct-run branch and records the revision as a paragraph-level raw
revision rather than associating it with a `HyperlinkSpan`. The projected run
therefore has no hyperlink index, receives no resolved URL, and produces no
link annotation in either view. Accepting an insertion materializes its run
inside the hyperlink, after which reparsing creates a span and the resolved
document does produce the annotation. The accepted render is therefore not
semantically equivalent to the accepted-and-resolved document.

### D2, the new PDF save test uses a shared fixed path
`crates/rdocx/tests/integration_test.rs:2126`
`crates/rdocx/tests/integration_test.rs:2134`

Every test process writes, reads, and deletes the same tracked-output path.
Two concurrent `cargo test` runs can both save successfully, then one can
delete the file before the other reaches its unwrapped read. This reintroduces
the cross-process race that F-X004 removed from file-writing tests. The path
must carry process identity or use isolated temporary storage.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-6 D1, D2, and D3 are resolved. Correctness and contract review produced
only D1 above. The test aspect produced only D2. Production panic safety,
OOXML preservation and schema ordering, and structural-rule compliance
produced no findings. The focused `rdocx-layout` unit suite plus the `rdocx`
regression and integration suites pass.
