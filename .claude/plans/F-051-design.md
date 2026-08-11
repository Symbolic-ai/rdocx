# F-051, CHANGELOG and migration notes

**Status**: approved
**Sprint**: S32.2
**Size**: S
**Depends on**: F-015, F-016, F-022, F-027, F-028, F-046, F-X005

## Problem

The repository has no root CHANGELOG, and `README.md:248` still presents the
old crate ownership without explaining the published shared family,
deprecation shims, moved APIs, or intentional breaking surfaces. Downstream
users cannot determine which imports remain compatible and which constructors,
types, or exhaustive matches require migration.

## Spec reference

- `docs/hld/03-architecture.md`, final crate ownership.
- `docs/hld/04-opc-and-packaging.md`, shared OPC and media ownership.
- `docs/hld/08-rendering-spec.md`, shared layout and PDF ownership.
- `docs/hld/11-migration-plan.md`, deprecated crates and breaking cutover.
- `docs/hld/14-development-backlog.md`, "F-051, CHANGELOG and migration notes".
- `docs/hld/15-build-and-toolchain.md`, version families and release process.

## Approach

Create the story-authorized root `CHANGELOG.md` with an Unreleased section and
a compact migration table. Name every moved or deprecated crate and its
replacement, including `rdocx-opc -> oxml-opc` and
`rdocx-pdf -> oxml-pdf`. Distinguish permanent facades such as `rdocx-oxml`
and `rdocx-layout` from deprecated shims.

Document the shared `Length`, OPC, media, layout, and PDF import paths, the new
`add_picture_auto` method, and the breaking surfaces observed in the integrated
code. These include removed Word-specific OPC constructors, shared error inner
types, shared line and output types, and non-exhaustive layout structures.
State the actual published shared version and keep any future stable rdocx
version labelled Unreleased rather than inventing a release number.

Update the root README crate table and add one migration link to the CHANGELOG.
Use only final integrated names and behavior from the preceding stories.

## Rejected alternatives

- Put migration notes only in HLD files. Those are design specifications, not
  downstream release documentation.
- Describe `rdocx-oxml` or `rdocx-layout` as renamed crates. Both remain real
  format-specific owners.
- Promise a stable release number or date. This sprint does not invoke the
  stable release workflow.
- Create a second migration document. One CHANGELOG plus the README link keeps
  the user-facing source singular.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| documentation, gate | exact migration-table assertion | Every renamed or deprecated crate is named with its replacement |
| documentation | public-path audit against integrated code | Every documented import, removed constructor, error type, and new method matches the code |
| regression | `python3 scripts/prose_check.py` | Repository voice rules pass |
| packaging | README and CHANGELOG link inspection | Published package documentation has a valid repository-relative migration path |

The backlog gate is that every renamed crate appears in the CHANGELOG with its
replacement.

## HLD impact

None. This story consumes the final architecture and migration specification
without changing system intent.

## Risk routing

- Public API documentation. Compare every migration statement to the exact
  integrated Rust surface and package descriptions.
- Version strings. Name the verified shared 0.1.1 release and leave the stable
  rdocx train Unreleased. Do not tag or publish.
- New file. F-051 explicitly authorizes the root CHANGELOG. Add no second
  migration artifact.

## Hash harness

Expected unchanged. Documentation does not alter package or rendering output.

## Implementation checklist

- [ ] Create the root Unreleased CHANGELOG and migration table.
- [ ] Name every deprecated shim and shared replacement.
- [ ] Document retained facades and all intentional breaking surfaces.
- [ ] Document the additive media sizing API and published shared version.
- [ ] Refresh the README crate table and link the migration notes.
- [ ] Run exact path, version, prose, package-doc, and hash checks.

## Open questions

None. The integrated cutover defines the documentation surface.
