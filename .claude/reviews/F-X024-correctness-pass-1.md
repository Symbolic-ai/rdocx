# F-X024, correctness, pass 1

**Reviewed**: the uncommitted working tree. Two manifests swapped, one `impl`
and its helper moved between crates, two tests moved with it, one fixture
constant made public, one invariant test added, and two documents that described
the removed exception.
**Verdict**: 0 defects, 0 smells, 1 nice-to-have

## Defects

None.

## Smells

None.

## Nice-to-have

### N1, the invariant test cannot exercise its own crate
`crates/oxml-drawing/src/lib.rs`

`no_shared_crate_depends_on_a_format_crate` checks all nine `oxml-*` manifests,
and it genuinely discriminates: adding `rdocx-oxml.workspace = true` to
`oxml-layout` makes it fail with the offending line named.

It cannot be exercised against `oxml-drawing` itself, which is the crate that
carried the old exception. Reintroducing that exact edge now produces
`rdocx-oxml -> oxml-drawing -> rdocx-oxml`, a genuine cargo dependency cycle
that fails to resolve before any test runs. That is a stronger guarantee than
the test provides, not a weaker one, so nothing needs doing. Recorded because a
reader who tries to prove the test works by editing `oxml-drawing` will get a
confusing cargo error rather than a clean assertion failure.

## Not found

Checked and produced nothing:

- **correctness**. The moved `impl` is the same code with paths rewritten for
  its new crate. `shared_theme_adapter_matches_the_legacy_theme_projection`
  still compares the projection against `Theme::from_xml` on the same source and
  still passes, which is the direct evidence that the conversion did not change.
- **orphan rule**. `impl From<&CT_OfficeStyleSheet> for Theme` is legal in
  `rdocx-oxml` because `Theme` is local there. Verified by compiling rather than
  by reasoning.
- **dependency direction**. `cargo tree -i rdocx-oxml` now lists only `rdocx-*`
  consumers. No `oxml-*` package depends on any `rdocx-*` or `rpptx-*` package,
  which is the invariant the story exists to restore and is now a test.
- **cycles**. `rdocx-oxml -> oxml-drawing -> oxml-core` is acyclic, and the
  workspace resolves.
- **panics**, **ooxml**. No parsing, serialisation or fallible path changed.
- **structure**. No new module, trait, generic or feature flag. One constant
  became public, with a named consumer that exists today: the moved regression
  needs the fixture and a dev-dependency back on `rdocx-oxml` would rebuild the
  edge the story just removed.
- **surface**. `oxml-drawing` loses a trait impl and gains a public constant.
  `rdocx-oxml` gains the impl. All three are breaking or additive changes to
  crates already taking a breaking minor bump this sprint, so the version
  consequence is already accounted for.
- **contract**. Matches the plan, including the accepted cost that `rdocx-oxml`
  now pulls `oxml-drawing`, so a Word-only consumer compiles DrawingML.
- **docs**. `docs/hld/03-architecture.md` no longer claims an exception, its
  dependency diagram is redrawn, and the paragraph now explains why the adapter
  sits where it does. `CLAUDE.md`'s layout table matches.

## Hash harness

**Unchanged, 28 of 28.** Expected: the conversion is the same code in a
different crate and nothing calls it in a rendering path. A delta would have
meant the move changed behaviour.
