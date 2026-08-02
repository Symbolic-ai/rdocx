# S21 sprint review, pass 1

**Reviewed**: `sprint/s21` against `5c739b2a8789abcda96ada1fe59ff56a04f3a73a`, 29 files, 4,409 changed lines, crates: `rpptx-oxml`, `rpptx-layout`, `rpptx`
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, the corpus gate accepts unresolved slides

`crates/rpptx-layout/src/context.rs:2015`

The sprint definition requires every corpus slide to resolve without unresolved
theme references, but the named gate asserts only that at least one slide
resolved. `resolve_pinned_corpus` counts every `ResolveError` as a contextual
error at `crates/rpptx-layout/src/context.rs:2360`, and the companion test at
`crates/rpptx-layout/src/context.rs:2263` explicitly accepts those errors. Theme
reference scanning therefore covers only successful slides. The gate can pass
while one or more corpus slides never produce a `ResolvedSlide`. The gate must
assert that every enumerated slide resolved successfully, then retain the
zero-theme-reference assertion over that complete set.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M9 end gate is: "the contract is frozen and published to the render track."
The public owned contract is present in `crates/rpptx-layout/src/lib.rs:17`, and
the renderer seam is documented. The draw-order, exact-colour, differential,
prompt-suppression, logo-multiplicity, and one-time native PowerPoint evidence
are executable in `crates/rpptx/tests/integration.rs:425` through
`crates/rpptx/tests/integration.rs:603`. The full verification gate and all 28
unchanged hashes passed. The milestone is not ready because B1 leaves the
complete-corpus part of the sprint contract unproved.

## Not found

No additional interaction, duplication, layering, harness, gate, docs, deps, or
surface findings were found. The new production dependency is the named
consumer of backend-neutral contract types, both corpus-only dependencies are
dev dependencies, no `oxml-*` crate gained an upward dependency, HLD changes
match the integrated behavior, and no hash baseline changed.
