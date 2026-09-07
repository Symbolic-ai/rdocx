# F-242, Root README product and capability overview

**Status**: completed
**Sprint**: S70
**Size**: M
**Depends on**: F-240

## Problem

The root README mixes current product guidance with volatile and unattributed
comparison claims about download counts, costs, binary sizes, memory, and cold
start (`README.md:235`). It presents authoring and preservation as undifferentiated
yes or no capabilities, so a reader cannot tell whether a feature is publicly
creatable, only readable, or merely retained during unrelated edits.

The existing README gate compiles seven root Rust fences and validates pinned
version and CLI literals, but it does not compare claims with a canonical
capability matrix or validate local anchors (`scripts/readme_doctests.py:51`
and `scripts/readme_doctests.py:176`). F-240 will provide the stable vocabulary
this rewrite needs.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, the approved authoring, preservation,
  unsupported, and permanent non-goal classifications.
- `docs/hld/10-bindings-spec.md`, native, Python, WASM, and CLI surface limits.
- `docs/hld/12-testing-strategy.md`, README doctest and claim validation gates.
- `docs/hld/14-development-backlog.md`, "F-242, Root README product and
  capability overview".
- `docs/hld/15-build-and-toolchain.md`, README CI and package metadata checks.

## Approach

Rewrite the root README around a stable product promise, a short classification
legend, a matrix-backed major-category table, explicit authoring versus read,
mutate, preserve-only, and non-goal distinctions, installation, three concise
compiling examples, crate and binding surfaces, evidence-based alternatives,
and links to the canonical roadmap and status records.

Remove volatile size, speed, popularity, and price claims unless an official
source and reproducible date-bound measurement are part of the repository
evidence. Compare python-docx, docx-rs, docx4j, and Aspose.Words only on official
documented functionality, license, runtime, and host dependency. Use unknown
where evidence does not support a claim. Link each rdocx capability statement
to a stable F-240 matrix id.

Extend `scripts/readme_doctests.py` to compile exactly three root examples,
derive version requirements from Cargo metadata, validate local links and
anchors, accept only approved official evidence links for comparisons, and
assert README classifications against the F-240 matrix. Add mutation-sensitive
coverage to `scripts/test_sprint_workflow.py`. Keep live external URL checks as
a focused implementation and review command, not a flaky default CI gate.

The completion review found that the cross-cutting F-X002 entry still described
the previous six-example gate. The approved contract correction adds the
backlog HLD to the impact list so that entry states the same three-example gate
as this plan.

## Rejected alternatives

- Preserve the current large comparison tables and update their numbers. They
  would become stale again and several claims lack reproducible evidence.
- Duplicate the full backlog in README. The canonical roadmap already owns
  detailed status.
- Describe preservation-only behavior as authoring. That is the ambiguity this
  story removes.
- Resolve external URLs in every CI run. Network availability is not a stable
  product gate.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_root_readme_capability_claims_match_the_approved_matrix` | Every product claim maps to the F-240 classification and evidence. |
| regression | `test_root_readme_versions_match_workspace_manifests` | Installation requirements derive from the current package metadata. |
| integration | root README doctest runner | All three examples compile against the local workspace and packaged README bytes match. |
| regression | local link and anchor mutation matrix | Broken local paths, anchors, or canonical-roadmap links fail. |
| regression | approved comparison evidence matrix | Unsupported comparison claims and non-official evidence links fail. |
| external | focused official URL check | Every retained official comparison source resolves during implementation review. |

The **test gate is regression**. README examples compile, links resolve,
version requirements match manifests, and every capability claim maps to the
approved matrix.

## HLD impact

- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

none. The story edits the existing README, documentation checks, and test file.
It changes no parser, serializer, public API, binding, dependency, feature,
asset, release carrier, or rendered output.

## Hash harness

Expected unchanged across all 49 entries. Documentation and validation logic do
not alter generated documents.

## Implementation checklist

- [x] Map root product claims to approved F-240 capability ids.
- [x] Replace volatile comparison claims with official functional evidence.
- [x] Rewrite installation, capability legend, examples, surfaces, and roadmap.
- [x] Reduce root Rust examples to three concise compiling cases.
- [x] Derive versions and validate local links and anchors in the README gate.
- [x] Add matrix alignment and approved-evidence mutation tests.
- [x] Run focused official URL checks, doctests, prose, full verification, and
  the unchanged hash harness.
- [x] Update exactly the listed HLD files and leave shared sprint records to the
  integrator.

## Open questions

None. Functional comparisons against the four named alternatives use official
sources, while volatile performance and popularity claims are removed.
