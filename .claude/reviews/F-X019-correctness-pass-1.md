# F-X019, correctness, pass 1

**Reviewed**: the F-X019 working diff on `work/f-x019-claude`, 3 files, 507
insertions and 24 deletions. `crates/rdocx-layout/src/paginator.rs`,
`engine.rs` and `docs/hld/03-architecture.md`.
**Verdict**: 1 defect, 1 smell, 2 nitpicks

## Defects

### D1, the function's summary line now contradicts the function
`crates/rdocx-layout/src/paginator.rs:620`

```rust
/// Wrapping drawings anchored to blocks after `block_idx` whose position
/// does not depend on where their own paragraph lands.
```

That was true, and after this diff it is exactly backwards for half the
function's output. `lookahead_wraps` now also returns drawings whose position
depends entirely on where their own paragraph lands, which is the story. The
body of the comment was updated and the summary line was not, so the two
disagree three lines apart.

The sentence in the middle has the same problem: "Looking ahead is only sound
when the drawing's vertical frame is the page or a margin" reads as a rule the
code follows, and the code stopped following it in this diff. It needs to
become the reason the second pass exists rather than a statement of what the
function does.

A stale summary on a function whose subtlety is the reason it has a fourteen
line comment is worse than no comment, because a reader who trusts it will
conclude the resolved branch is dead code.

## Smells

### S1, an argument-count lint silenced rather than answered
`crates/rdocx-layout/src/paginator.rs:290`

```rust
#[allow(clippy::too_many_arguments)]
fn paginate_pass(
```

`paginate_with_media` already carried seven arguments and this story adds an
eighth, which trips the lint. `AGENTS.md` is explicit that a function tripping a
complexity lint needs more structure and not less, and an `allow` is less. The
eight are also passed twice, at `paginator.rs:236` and `:262`, with only the
last differing between the two calls, so the repetition is visible at the call
site as well as the definition.

Recorded rather than changed, because the alternative is a parameter struct that
exists only to be destructured immediately, which `AGENTS.md` names as a wrapper
that only forwards. The two rules point in opposite directions here and the
honest answer is to record which one was chosen and why, rather than to pretend
the lint was satisfied.

## Nitpicks

- `crates/rdocx-layout/src/paginator.rs:634`, `height` is accumulated but the
  loop condition tests it before adding the current block, so the last block
  tested is the first one that starts beyond the content height rather than the
  last that fits. Pre-existing behaviour, unchanged by this diff, and it makes
  the look-ahead marginally generous rather than wrong.
- `crates/rdocx-layout/src/engine.rs:2822`, `make_lookahead_document` takes the
  vertical offset as a bare `f64` whose correct value differs per frame, and the
  two call sites pass `-120.0` and `150.0` with the reason in a comment on the
  helper rather than at the call. Fine at two call sites.

## Not found

- **correctness**. The key `(block index, anchor index)` is stable across passes
  because both passes walk the same slice. `.iter().enumerate().skip(n)` yields
  absolute indices, which is what the resolved map is keyed on, and was checked
  rather than assumed. A paragraph moved to a later page by the recursion at
  `paginator.rs:1286` records its final placement only, since the recursion
  returns immediately after. A split paragraph's continuation carries
  `anchored: Vec::new()`, so it cannot double-record.
- **contract**. Two passes, gated on a predicate, with the first pass identical
  to a single-pass run because `resolved_in` is empty. The plan's rejected
  alternatives stay rejected: no fixed-point iteration, no unconditional second
  pass, no predicted paragraph top.
- **panics**. No new `unwrap`, indexing or slicing. `resolved_in.get` returns
  `Option`. `std::mem::take` on the outgoing map leaves it valid.
- **ooxml**, no parser or serialiser touched.
- **structure**. `ResolvedWraps` is a type alias, `PassResult` is a return type
  with three fields that all have callers today, and `is_paragraph_relative_wrap`
  has two callers today, the look-ahead and the predicate. No new trait, generic,
  crate, module or file.
- **tests**. The gate,
  `a_paragraph_relative_wrapping_drawing_pushes_earlier_text_aside`, was
  confirmed to fail against the unfixed code: disabling the second pass makes
  the earlier paragraph take 18 lines against 18, and the assertion fires. The
  page-relative control, the predicate test, the empty-map test, the page-scoping
  test and the recording test all pass, 87 in the crate.
