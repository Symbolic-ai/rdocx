# Current Sprint, S42

**Milestone**: X Cross-cutting.

**Goal**: refresh the dependency lockfile and measure what it does to rendered
output, then carry S41's work to crates.io. Both publication trains move a minor
version, because S41 broke both public APIs rather than merely extending them.

## Spec references

- `docs/hld/12-testing-strategy.md`, "The hash harness", for the rule that an
  intentional delta lands as its own labelled commit with the expected change
  stated. That rule is what this sprint turns on.
- `docs/hld/15-build-and-toolchain.md`, for the pinned toolchain the refresh
  must keep working against, and for the release job contracts the two
  publication stories execute.
- `docs/hld/10-bindings-spec.md`, for the Python and WASM packages that inherit
  a version without gaining publication authority.
- `docs/hld/14-development-backlog.md`, for the F-X020 scope and its gate.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X020 | Refresh the dependency lockfile | S | done | - |
| F-X022 | Tag rpptx-v0.3.0 | S | pending | - |
| F-X023 | Tag v0.7.0 | S | pending | - |

## Sequencing note

Rows are listed in dependency order.

F-X020 ran first and alone, so that a refresh which moved a rendering baseline
could not compete with a release to explain the same delta. It did not move one,
though it did move every sample PDF, which is recorded in its AS_BUILT entry and
filed as F-X021.

F-X022 precedes F-X023, and the reason is a pin rather than a preference. The
stable crates depend on `oxml-layout`, so the incubating train has to be
resolvable at 0.3.0 on crates.io before the stable train that pins it can
publish. S39 released stable first because only one train moved that sprint.
Both move here, so the order reverses.

Each release story ends at a boundary this sprint cannot cross on its own.
`/release` is the only command permitted to create a `v*` or `rpptx-v*` tag or
start publication, and it requires separate immediate approval at the reviewed
SHA. Preparation lands in the sprint. Publication does not happen without that
approval.

## Definition of done for this sprint

- Every semver-compatible update outstanding at the sprint's start is taken, or
  the ones held back are named with a reason.
- `cargo audit` reports zero vulnerabilities and `cargo deny check` passes,
  with the `ttf-parser` unmaintained advisory still the single documented
  exception.
- The hash harness is either unchanged, or its delta names the dependency that
  caused it and was reviewed before the baseline was re-recorded.
- The pinned toolchain and MSRV still build the workspace.
- The fifteen incubating packages read 0.3.0 and the eleven workspace-version
  packages read 0.7.0, with every root pin, lock entry, README example, Python
  project version and WASM literal agreeing.
- The exact publication sets hold: fourteen incubating crates and seven stable
  crates, with `rpptx-wasm`, the Python packages and the WASM packages gaining
  no publication authority.
- Nothing is tagged or published without the separate approval `/release`
  requires. A sprint that ends prepared but unpublished is a complete sprint,
  not a carried one.
