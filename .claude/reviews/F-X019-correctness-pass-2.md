# F-X019, correctness, pass 2

**Reviewed**: the remediated F-X019 working diff on `work/f-x019-claude`, 3
files, 502 insertions and 24 deletions. Pass 1 raised 1 defect, 1 smell and 2
nitpicks. This pass re-reads the whole diff, not only the repairs.
**Verdict**: 0 defects, 0 smells, 2 nitpicks

## Defects

None.

D1 is fixed. `crates/rdocx-layout/src/paginator.rs:620-632` now leads with
"Wrapping drawings anchored to blocks after `block_idx`, positioned well enough
to flow this block's text around them", which is what the function returns, and
splits the rest into the two cases: a page or margin frame is positioned here,
a paragraph frame is positioned by the previous pass. The sentence that read as
a rule the code follows now states why the second pass exists. Nothing in the
comment claims a branch that no longer holds.

## Smells

None.

S1 is fixed, and fixed by removing the argument list rather than by suppressing
the lint. `paginator.rs:263-273` introduces `PassContext`, holding the six
values that are identical between the two passes, and `paginate_pass` takes
three arguments: the blocks, the context and the incoming resolved map. The
`#[allow(clippy::too_many_arguments)]` is gone, and so are the two eight-line
call sites, replaced by `paginate_pass(blocks, &context, &ResolvedWraps::new())`
and `paginate_pass(blocks, &context, &first.resolved)`.

This is a real reduction rather than a wrapper that only forwards: the struct is
built once, borrowed twice, and its second instantiation exists today in the
test at `paginator.rs:3313`. A reader comparing the two passes now sees exactly
what differs between them, which is the third argument.

## Nitpicks

Both carried from pass 1, deliberately.

- `crates/rdocx-layout/src/paginator.rs:637`, the look-ahead tests its running
  height before adding the current block, so the last block considered is the
  first one starting beyond the content height. Pre-existing, untouched by this
  diff, and generous rather than wrong.
- `crates/rdocx-layout/src/engine.rs:2822`, `make_lookahead_document` takes a
  bare vertical offset whose correct value differs per frame, and the two call
  sites pass `-120.0` and `150.0`. Fine at two call sites, and the helper's
  comment says why they differ.

## Not found

- **correctness**. Re-checked the key stability, the page scoping, the split and
  recursion paths, and the empty-map first pass. The remediation moved six
  values into a struct and rewrote a comment. It changed no control flow, which
  the identical test results confirm.
- **contract**, **panics**, **ooxml**, **structure**. Unchanged from pass 1 and
  re-checked. `PassContext` is the one new type, and it exists to remove an
  argument list that two call sites shared today.
- **tests**. 87 pass in `rdocx-layout`. The gate still fails against the unfixed
  code, which was verified by disabling the second pass: the earlier paragraph
  takes 18 lines against 18 and the assertion fires. Clippy is clean at
  `-D warnings` with no `allow` added anywhere in this diff.

## Exit condition

Zero defects, zero smells. Two nitpicks remain, recorded with their reasons.
