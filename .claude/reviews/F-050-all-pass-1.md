# F-050, all, pass 1

**Reviewed**: working-tree diff against `5b2fa198`, 1 file and 36 changed
lines, with 34 additions and 2 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, Two workspace all-features jobs still include the binding crates

`.github/workflows/ci.yml:103`

`.github/workflows/ci.yml:124`

The clippy and documentation jobs still run the whole workspace with
`--all-features` without `--exclude rdocx-py --exclude rpptx-py`. The cited CI
matrix contract requires those exact exclusions on every all-features job, not
only the two workspace test jobs changed here. A pull request therefore runs
two all-features jobs outside the documented binding isolation rule. Add the
same exclusions before clippy's `-- -D warnings` delimiter and to the
documentation command.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness, apart from D1, the three new jobs use valid job identifiers,
  unique display names, valid step structure, and the commands specified by
  the approved design.
- Contract, the ordinary and MSRV workspace test jobs carry both exact binding
  exclusions. The three focused jobs match F-050 and do not add publication or
  unrelated release behavior.
- Panics, the YAML change adds no indexing, slicing, arithmetic, or unchecked
  runtime operation.
- OOXML, no parser, serializer, namespace, schema-order, whitespace, or
  unmodelled-subtree behavior is changed.
- Tests, the no-default job disables `oxml-layout`'s default `system-fonts`
  feature, the WASM job installs `wasm32-unknown-unknown` before checking the
  existing `rdocx-wasm` package, and the prose job runs both required Python
  gates. The base revision lacks all three job definitions, so reverting this
  diff removes the new coverage and supplies the negative-path comparison.
- Structure, the change adds no source file, module, crate, feature flag,
  trait, generic, wrapper, or indirection.
- GitHub Actions behavior, the workflow parses as YAML. Every new job checks
  out the repository, Rust jobs select a toolchain, the WASM target is installed
  by the toolchain action, and no new job has an unmet dependency.
- WASM scope, `rdocx-wasm` exists in the current workspace and `rpptx-wasm`
  does not. Omitting the future package matches the approved plan.
- MSRV behavior, the existing job keeps its explicit Rust 1.93 toolchain while
  gaining both binding exclusions. No stable-toolchain setting overrides it.
- Duplicate or missing focused jobs, there is exactly one no-default job, one
  WASM job, and one combined prose and generated-skill job.
- Prose and generated skills, both read-only gates pass on the reviewed tree.
- Hash behavior, the diff changes CI configuration only and contains no
  product or rendering behavior that could explain an output delta.
