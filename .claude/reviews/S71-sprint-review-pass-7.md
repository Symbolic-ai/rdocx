# S71 sprint review, pass 7

**Reviewed**: full integrated dependency prefix on `sprint/s71` at
`8423f6f36aef2c54ce949eb3bc8e68e436660552` against merge base
`fd069fd460fec11b27c2f6eb1004d4a9ee9bcade`, 96 files and 32,181 changed
lines, comprising 26,147 additions and 6,034 deletions, crates: `oxml-media`,
`oxml-opc`, `rdocx-layout`, `rdocx-oxml`, `rdocx`, `rpptx`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

**Bound extension**: explicit user-directed dependency-prefix review. Pass 5
was the first review at the current F-246 boundary, pass 6 found the
rich-fragment error interaction, and pass 7 is the third and final review in
this boundary's remediation sequence.

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M23 end gate requires all five private references to be generated from a
blank public facade, reopen without repair, match the required package
semantics and reviewed deterministic visual thresholds, produce identical DOCX
bytes on repeated generation, and report no unexplained preservation-only
fallback (`docs/hld/14-development-backlog.md:2214`). The gate does not yet
hold at this dependency-prefix boundary because F-247 and F-248 remain pending
(`docs/sprints/CURRENT_SPRINT.md:37`). This pass does not assert otherwise.

The completed prefix and both merge remediations have direct evidence. The
conflicting-default and reciprocal-link regression passes for append,
append-with-break, and insertion while proving byte-identical receiver state
after rejection. The new fallible staged insertion path returns the combined
style-graph error to rich mail merge, and the invalid-rich-input matrix now
constructs an independently valid fragment with a different sole paragraph
default. It proves `mail_merge_rich` returns `Err` without unwinding and leaves
the template byte-identical. Public insertion, style deduplication, repeated
rich-fragment collision remapping, ordered rich output, and flat merge behavior
also pass.

The complete `rdocx` regression binary passed 354 tests with three declared
ignores. `cargo check -p rdocx --all-targets`, scoped all-feature clippy with
warnings denied, formatting, prose, generated-skill drift, and diff checks
passed. The deterministic hash harness matched all 49 reviewed entries.

## Not found

- **Pass 6 remediation**: `import_rich_fragment` now uses the shared fallible
  staged insertion path and propagates validation failure through `Result`.
  It exposes no partially built output, and the caller-owned candidate is
  discarded on error.
- **Direct and rich interaction**: direct public insertion retains its
  clone-validate-commit boundary. Rich import shares the mutation core without
  bypassing style validation. Valid reciprocal-link, numbering, package,
  relationship, and identity remaps still succeed.
- **Duplication**: one staged insertion owner now serves the public transaction
  wrapper and the existing rich importer. No duplicate style or numbering
  merge path was introduced.
- **Layering and dependencies**: no manifest changed. No `oxml-*` crate gained
  an `rdocx-*` or `rpptx-*` dependency, and no dependency lacks a named
  consumer.
- **Harness**: all 49 entries match. The only baseline changes remain the two
  declared F-249 document XML entries recorded in the completion log.
- **Documentation and surface**: the remediation adds only a crate-private
  helper and matches the existing rich mail-merge atomic-error contract. It
  adds no unplanned public surface and requires no HLD change.
