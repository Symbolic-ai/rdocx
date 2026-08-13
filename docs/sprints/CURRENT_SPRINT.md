# Current Sprint, S34

**Milestone**: M13 Bindings and tooling.

**Goal**: Make the Python packages ready for typed use, parity validation,
cross-platform wheel production, and continuous integration. Complete the
rdocx compatibility evidence, reuse the validated path machinery for rpptx,
then make wheel and PR automation enforce the resulting package contract.

## Spec references

- `docs/hld/03-architecture.md`, for the shared Python-support boundary and the
  permitted `rdocx-py` and `rpptx-py` dependency directions.
- `docs/hld/10-bindings-spec.md`, for the hand-written stubs, `py.typed`, mixed
  package layout, parity suite, wheel matrix, tag namespace, and PR-time job.
- `docs/hld/12-testing-strategy.md`, for Python parity coverage and the binding
  exclusions required by workspace Rust gates.
- `docs/hld/13-risks-and-open-questions.md`, for the index-path aliasing risk
  that the reused presentation handles must preserve.
- `docs/hld/14-development-backlog.md`, for F-134 through F-138 dependencies
  and their named acceptance gates.
- `docs/hld/15-build-and-toolchain.md`, for abi3-py39, mixed-package version
  alignment, PyPI OIDC publication, and binding-safe CI behavior.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-136 | rpptx-py | L | in-progress | codex |
| F-134 | Type stubs and py.typed | M | in-progress | codex |
| F-135 | python-docx parity suite | M | in-progress | codex |
| F-137 | wheels.yml | M | in-progress | codex |
| F-138 | PR-time Python job | S | in-progress | codex |

## Sequencing note

Rows are listed in dependency order, not by F-ID. F-136 first reuses completed
F-129 and F-116. F-134 follows F-136 so both Python packages receive one typed
contract. F-135 then validates the settled rdocx surface without changing it.
F-137 follows F-134 and F-136 so both typed packages enter the wheel matrix.
F-138 follows F-137 and exercises the same build paths on pull requests.

## Definition of done for this sprint

- `mypy --strict` and `stubtest` pass against installed packages carrying
  `py.typed`.
- Every pinned python-docx 1.2.0 example inside the approved S33 surface runs
  with only the package namespace changed, and two-way round trips preserve
  normalized content.
- The seven pinned python-pptx 1.0.2 Getting Started examples run with only the
  package namespace changed through path-based `rpptx-py` handles.
- `wheels.yml` builds and installs the abi3-py39 target matrix through the
  `py-v*` OIDC publication path.
- The PR-time Python job builds the extension and runs pytest, and a binding
  test failure makes the job fail.
- Binding-focused gates pass with the required Rust workspace exclusions, and
  existing deterministic document and rendering outputs do not regress.
