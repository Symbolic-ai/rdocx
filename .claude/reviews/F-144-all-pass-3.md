# F-144, all, pass 3

**Reviewed**: uncommitted `work/f-144-codex` implementation, 15 files and 1,100
changed lines, including all pass 1 and pass 2 remediation plus the four
approved new CLI crate paths
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, diff allocates an unbounded quadratic slide matrix

`crates/rpptx-cli/src/commands.rs:204`

Both input deck lengths come from untrusted files, but the longest-common-
subsequence implementation allocates `(slides_a + 1) * (slides_b + 1)`
`usize` cells without a bound. Two 10,000-slide decks request roughly 800 MB
for this matrix before accounting for row allocations and parsed packages.
Larger inputs can abort the process in the allocator instead of returning a
bounded command error. Use a memory-bounded LCS reconstruction or reject slide
counts whose checked matrix size exceeds an explicit budget, and add a command
regression at the rejected boundary.

## Smells

None.

## Nitpicks

None.

## Prior finding re-evaluation

- **Pass 1 D1 and pass 2 D1, clean CI corpus setup**: resolved. Both the primary
  test job and the MSRV job fetch and verify the pinned corpus before their
  workspace tests. The structural regression checks both jobs and their step
  order.
- **Pass 1 D2, unbounded raster allocation**: resolved. Convert validates every
  page and render validates every selected page against the pixel budget before
  rasterization. Both boundary regressions passed.
- **Pass 1 D3, missing plain inspect metadata**: resolved. Plain inspect reports
  all eight modeled core-property fields, and its populated regression passed.
- **Pass 2 D2, zero-slide PNG conversion**: resolved. The command now fails
  before layout or output creation, and the filesystem postcondition is tested.

## Not found

- **Public replacement semantics**: reverse snapshot mutation remains correct
  for same-run and cross-run matches, including UTF-8 byte boundaries. Breaks,
  fields, and alternate content remain boundaries. Groups and table cells are
  traversed.
- **Package preservation and OOXML**: replacement retains the first run's
  formatting, the unmatched suffix's formatting, raw run content, and opaque
  package parts. No CLI path reads or writes raw PresentationML directly.
- **Other CLI correctness**: exactly the seven approved F-144 commands remain.
  Shared JSON, output, and one-based range contracts are used. No fresh inspect,
  text, convert, replace, validate, or render selection defect was found.
- **Panics**: apart from the allocator failure in D1, indexing and slicing sites
  are protected by local loop or match invariants. Raster dimensions are
  checked before allocation.
- **Tests**: all eight CLI integration tests passed with the verified external
  50-deck corpus, including both prior-remediation regressions. The workflow
  suite reached 33 passing tests and one environment-only permission error when
  it attempted to create a temporary directory inside this reviewer-read-only
  worktree.
- **Structure and dependencies**: no extra crate, module, test binary, trait,
  wrapper, generic, feature, or forbidden reverse dependency was added.
- **Publication and harness**: release changes remain manifest eligibility,
  allowlist, workflow, and dry-run preparation only. No tag or publication
  action is introduced, and no rendering baseline or hash-harness behavior
  changes.
- **Hygiene**: `git diff --check` passed before this review record was written.
