# F-X025, correctness, pass 1

**Reviewed**: the F-X025 working diff on `work/f-x025-claude`, 6 files, 105
insertions and 14 deletions. `.claude/commands/verify.md`, its generated
adapter, `scripts/test_sprint_workflow.py`, two HLD sections and the plan.
**Verdict**: 0 defects, 1 smell, 2 nitpicks

## Defects

None.

The wiring is real and self-defending.
`test_verify_runs_the_release_regressions` at
`scripts/test_sprint_workflow.py:4035` asserts the step is present **and**
asserts that a copy with the step removed fails the same helper, so the test
cannot pass vacuously if someone deletes the line it is guarding. The mutation
half was checked by hand as well as asserted.

`test_every_test_publish_yml_names_resolves_to_a_real_test` at
`scripts/test_sprint_workflow.py:4050` reads the real `publish.yml`, extracts
both dotted paths, and resolves each to a callable on a real class. It asserts
`named` is non-empty first, so a `publish.yml` that stopped naming tests fails
rather than passing over an empty set. That guard is the difference between a
test and a decoration.

## Smells

### S1, the resolver reads the module through `globals()`
`scripts/test_sprint_workflow.py:4062`

```python
cls = globals().get(class_name)
```

Correct today, and it silently ties the test to the assumption that every class
`publish.yml` can name lives in this module's top-level namespace. A preflight
moved into a nested class or a sibling module would resolve to `None` and be
reported as "is not a test in this module", which is true and unhelpful: the
message would send a reader looking for a rename that did not happen.

The module prefix the test strips (`scripts.test_sprint_workflow.`) already
encodes the assumption, so the scope is at least stated. Recorded rather than
changed, because the alternative is `importlib` machinery for a case that does
not exist and the story is meant to be small.

## Nitpicks

- `docs/hld/15-build-and-toolchain.md:173`, the new paragraph says `/verify`
  step 6 rather than naming the step by its content. A renumbering of the
  command's steps would date it. The command document is the contract and does
  name it by content, so the drift would be cosmetic.
- `.claude/commands/verify.md:41`, the step now carries three paragraphs of
  rationale where the neighbouring checks carry one. The S42 story is the
  reason the step exists and is worth keeping, and it does make step 6 the
  longest in the document.

## Not found

- **contract**. The diff does what the plan describes: the step, the
  regenerated adapter, two tests, and the two stale figures. It touches no
  version carrier, so the inspection the risk routing demands returns an empty
  diff, as predicted.
- **correctness**, elsewhere. `publish.yml` is unchanged, so the publication
  gate itself is untouched. `python3 -m unittest scripts.test_sprint_workflow`
  runs 48 tests in about 2 seconds from a clean tree.
- **panics**, **ooxml**. No Rust, no parser, no serialiser.
- **structure**. No new trait, generic, module or file. One extracted helper,
  `assert_verify_runs_the_release_regressions`, which exists because the
  mutation test needs to call the same assertion twice. That is the second
  caller, today.
- **tests**. Both gates were checked against the state they guard. Beyond the
  asserted mutation, the end-to-end halves were demonstrated: a stale version
  in `crates/rpptx/Cargo.toml` fails both preflights, and the S42 defect
  reproduced exactly, `ci.yml`'s `@tensorbee/rpptx-wasm` literal set back to
  0.2.0, fails three tests including both WASM job assertions. A clean tree
  passes 48 of 48.
