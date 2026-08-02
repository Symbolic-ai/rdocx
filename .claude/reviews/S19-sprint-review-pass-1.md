# S19 sprint review, pass 1

**Reviewed**: `sprint/s19` against `a7dd1ac204e839437d8e491ec09cc26fcf88a892`, 23 files, 2301 changed lines, crates: `oxml-drawing`, `rpptx`, `rpptx-oxml`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M8 gate requires all 50 corpus decks to round-trip and every generated deck
to open in PowerPoint without a repair prompt at
`docs/hld/14-development-backlog.md:566`.

The gate holds. `all_corpus_modelled_parts_reparse_structurally` dispatches all
seven modelled roots and requires nonzero coverage at
`crates/rpptx/tests/integration.rs:38`.
`all_modelled_corpus_packages_match_expected_parts` compares canonical bytes
for rewritten roots, original bytes for unmodelled parts, and package structure
at `crates/rpptx/tests/integration.rs:60`. The required 50-deck integrated run
passed all 11 `rpptx` tests, including the pinned python-pptx 1.0.2
differential. The native acceptance record at
`.claude/plans/F-080-design.md:186` identifies PowerPoint 16.104 build
16.104.25121423 and records 50 exact-path opens and clean closes with no repair
prompt, timeout, path mismatch, or presentation-count mismatch. The integrated
hash harness matched all 28 entries.

## Not found

- Interaction: the facade writes presentation, slide, and notes-slide roots at
  `crates/rpptx/src/lib.rs:140`, while the package gate supplies the other four
  modelled root families at `crates/rpptx/tests/integration.rs:83`. The expected
  package comparison covers their combined result.
- Duplication: no competing facade, dispatcher, relationship resolver, or
  second integration binary was added.
- Layering: `rpptx` depends toward `oxml-opc` and `rpptx-oxml`. No `oxml-*`
  crate gained an `rpptx-*` dependency.
- Harness: both feature plans declare no Word rendering delta, both AS_BUILT
  entries record unchanged, and the integrated harness matched 28 of 28.
- Gate: the required corpus, differential, package, and native PowerPoint
  evidence all completed without a skipped deck.
- Docs: the approved HLD impact files describe the facade, optional
  format-scheme name, self-contained blip namespace, and exact modelled versus
  unmodelled byte boundary.
- Deps: the new production dependencies belong to the `rpptx` facade. The
  `oxml-drawing` dependency is test-only for direct theme dispatch.
- Surface: the public types and methods at `crates/rpptx/src/lib.rs:57` match
  the approved F-079 read contract. No mutation API, trait, generic abstraction,
  forwarding wrapper, feature flag, or publication surface was added.
