# F-X059, working, pass 1

**Reviewed**: complete working diff against
`2b439f60c29d38c192c5ca18fe89cf268c981286`, 48 tracked files with 253
insertions and 176 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness and release contract produced no findings. The canonical carrier
test names the exact 15-package publishable family in dependency order and
requires every manifest, workspace pin, lock record, publication flag, README
literal, Rust assertion, CI literal, and publish preflight to agree at 0.7.0 at
`scripts/test_sprint_workflow.py:4834`. The real workflow publishes the same
ordered allowlist at `.github/workflows/publish.yml:72`. Stable workspace pins
remain 0.10.1 at `Cargo.toml:71`, and the stable publication allowlist remains
separate at `.github/workflows/publish.yml:55`.

The immutable registry regression produced no findings. It builds an isolated
consumer with exact `rdocx-layout@0.10.1`, a fresh Cargo home, and no local
patch at `scripts/test_sprint_workflow.py:4616`. It requires published
`oxml-layout@0.6.0` and explicitly rejects 0.7.0 at
`scripts/test_sprint_workflow.py:4662`. The focused registry check passes.

Release notes and contribution inventory produced no findings. The prepared
section contains Highlights, Added, Fixed, Compatibility, and Contributors in
the required order at `CHANGELOG.md:7`. It limits its claims to F-X058's shared
multilingual substrate, reserves stable Word integration and final acceptance
for later stories at `CHANGELOG.md:50`, and records that the selected history
contains no authenticated external issue or pull-request record at
`CHANGELOG.md:56`. Deterministic release-note check and render both pass, and
the notes regression rejects issue, pull-request, and stable Word claims at
`scripts/test_sprint_workflow.py:4964`.

WASM, package, legal, font, and asset review produced no findings.
`rpptx-wasm` is prepared at 0.7.0 but remains unpublished and outside workspace
dependency pins at `scripts/test_sprint_workflow.py:4877`. Its mutation tests
reject family, tag-template, and inherited-version drift at
`scripts/test_sprint_workflow.py:5079`. The package evidence covers the exact
patched 22-package dry run, the 10 MiB ceiling, the complete 24-font legal and
provenance inventory selected by `crates/oxml-layout/Cargo.toml:13`, and the
default presentation asset carried by `rpptx`. Both WASM checks and the
binding metadata assertions are recorded green with no added publication
authority.

Tests, HLD, and structure produced no findings. The 49-entry deterministic
hash harness is unchanged, all 85 sprint-workflow tests pass, and the focused
carrier, stable, registry, notes, metadata, package, README, and Rust assertion
gates cover the modified carriers. Exactly the five plan-listed HLD files are
updated. They distinguish current 0.7.0 preparation from the last published
0.6.0 family and preserve separate final release approval at
`docs/hld/15-build-and-toolchain.md:252`. The diff adds no runtime behavior,
public type, dependency, source module, forwarding wrapper, panic path, OOXML
parser or serializer change, schema-order change, or unmodelled XML handling
change.
