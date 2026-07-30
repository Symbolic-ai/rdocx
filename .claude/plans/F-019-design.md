# F-019, PresentationML relationship and content types

**Status**: approved
**Sprint**: S04
**Size**: S
**Depends on**: F-018

## Problem

The OPC relationship table is still Word-oriented. The implementation staged
by F-018 comes from `crates/rdocx-opc/src/relationship.rs:9`, where the public
constants stop at Word and shared office relationships. It has no thumbnail,
extended-property, custom-property, slide, layout, master, notes, or
presentation settings relationship types.

The content-type implementation at
`crates/rdocx-opc/src/content_types.rs:23` exposes the table model but no
shared MIME constants. Without a format-neutral constant surface, later
PresentationML code would repeat long schema strings at each call site and a
copy error would still compile.

## Spec reference

- `docs/hld/03-architecture.md`, "The dependency rule", "Why these seams",
  and "Versioning".
- `docs/hld/04-opc-and-packaging.md`, "Relationship types" and
  "Generalising the constructors".
- `docs/hld/06-presentationml-model.md`, "Parts" and `presentation.xml`.
- `docs/hld/12-testing-strategy.md`, "New tests the extracted crates need",
  subsection `oxml-opc`.
- `docs/hld/15-build-and-toolchain.md`, "Publishing".

## Approach

Extend `oxml_opc::relationship::rel_types` with the constants specified by the
OPC design. Keep `CORE_PROPERTIES` unchanged, add package-level `THUMBNAIL`,
shared `EXTENDED_PROPERTIES` and `CUSTOM_PROPERTIES`, and the PresentationML
relationships `SLIDE`, `SLIDE_LAYOUT`, `SLIDE_MASTER`, `NOTES_SLIDE`,
`NOTES_MASTER`, `PRES_PROPS`, `VIEW_PROPS`, `TABLE_STYLES`, and
`HANDOUT_MASTER`. Use the package relationship namespace only for package-level
types. Use the officeDocument relationship namespace for the shared and
PresentationML types.

Make F-018's existing `content_types` module public while retaining the root
re-exports of `ContentType` and `ContentTypes`. Add public string constants to
that module for the two universal defaults, shared core, extended, and custom
properties, and the PresentationML parts listed in the OPC and PresentationML
specifications. Include distinct presentation and slideshow main-part values
because the content-type override is the only `.pptx` versus `.ppsx`
distinction. Reuse the constants from `ContentTypes::minimal()` so the module
does not create a second spelling of the universal MIME strings.

Keep the change inside F-018's existing `lib.rs`, `relationship.rs`, and
`content_types.rs`. Do not add a dependency, feature flag, trait, generic
parameter, source file, rdocx consumer edit, or publication configuration.

Add one table-driven test that enumerates every relationship and content-type
constant. The relationship table asserts uniqueness, no whitespace, and the
correct package or officeDocument URI prefix. The content-type table asserts
uniqueness, no whitespace, and a valid MIME type shape. Listing every constant
in the test is deliberate because adding a constant without classifying it
must require a test-table update.

## Rejected alternatives

- Define constants in `rpptx`. OPC relationship and content-type strings are
  packaging concepts and belong in the format-neutral package crate.
- Add a typed relationship enum. Current callers and both package formats use
  string values, and no second representation exists to justify another type.
- Rename the existing `DOCUMENT` constant. That would create needless churn
  and is not required to add the PresentationML surface.
- Connect `rdocx-*` consumers to the new constants. The published family must
  remain independent of unpublished development crates until the rdocx 0.5.0
  release boundary passes.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `relationship_and_content_type_constants_are_unique_and_well_formed` | Every public constant appears once, relationship URIs use the correct namespace, MIME values have the expected shape, and no value contains whitespace |
| integration | `cargo check -p oxml-opc --all-targets` | The additive constant surface compiles without a dependency or consumer change |

The backlog **test gate** is the table test asserting every constant is unique
and well-formed.

## HLD impact

None. The architecture, OPC constant groups, PresentationML parts, and
development publication boundary already describe this surface.

## Risk routing

- Public API of a reserved crate. Treat the constants and module visibility as
  additive while the crate remains version 0.0.0 and unpublished. Inspect the
  manifest, run `cargo package -p oxml-opc`, and assert that the local archive
  remains below 10 MiB. Do not tag or publish.
- New module surface. F-019 explicitly authorizes the public `content_types`
  constants module. Reuse F-018's existing `content_types.rs` and add no new
  file or forwarding wrapper.

## Hash harness

Expected to remain unchanged. The constants have no published rdocx call site,
and any digest delta blocks the sprint.

## Implementation checklist

- [ ] Add the package-level, shared-property, and PresentationML relationship
      constants to `oxml_opc::relationship::rel_types`.
- [ ] Expose the existing `content_types` module and add the shared property and
      PresentationML MIME constants.
- [ ] Reuse the universal constants in `ContentTypes::minimal()`.
- [ ] Add the complete table-driven uniqueness and well-formedness gate.
- [ ] Keep every rdocx consumer and publication setting unchanged.
- [ ] Run focused tests, the package archive check, and the unchanged hash
      harness.

## Open questions

None. Keep the module limited to the universal defaults, shared properties,
and PresentationML parts named by F-019. Moving Word-specific MIME strings is
outside this story and would cross the deferred publication cutover.
