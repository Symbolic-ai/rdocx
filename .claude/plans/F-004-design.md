# F-004, Caladea licence and the false OFL claim

**Status**: approved
**Sprint**: S01
**Size**: S
**Depends on**: none

## Problem

The four Caladea TTFs in `crates/rdocx-layout/fonts/` have no accompanying
licence or notice, while `crates/rdocx-layout/src/bundled_fonts.rs:12` says all
bundled fonts use the SIL Open Font License. Caladea is Apache-2.0, so the
published crate currently has an inaccurate claim and incomplete attribution.

## Spec reference

- `docs/hld/15-build-and-toolchain.md`, "Packaging".
- `docs/hld/13-risks-and-open-questions.md`, "Font licensing, currently shipping".

## Approach

Add `fonts/LICENSE-Caladea` with the Apache License 2.0 text and
`fonts/NOTICE-Caladea` with Caladea attribution. Correct the module comment to
name the licence for each family. Add a unit test beside `bundled_font_data`
that maps every distinct bundled family to an existing licence file, keeping
the check in the crate whose embedded assets it protects.

## Rejected alternatives

- One generic licence statement was rejected because the bundled families use
  different licences.
- A repository-root notice was rejected because assets outside the crate can
  be omitted from the published package.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `every_bundled_font_family_has_a_licence_file` | Each distinct family returned by `bundled_font_data` maps to a checked-in licence file. |
| integration | `cargo package -p rdocx-layout --list` | The Caladea licence and notice are included in the published crate. |

The **test gate** is `every_bundled_font_family_has_a_licence_file`.

## HLD impact

- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Bundled fonts. Read `docs/hld/15-build-and-toolchain.md`. Verify the real
  licence per family and inspect `cargo package -p rdocx-layout --list` so the
  new attribution ships with the TTFs.
- New files. The structural rules in `CLAUDE.md` require explicit approval for
  `fonts/LICENSE-Caladea` and `fonts/NOTICE-Caladea`.

## Hash harness

Expected to be unchanged because only licence assets, tests, and comments move.

## Implementation checklist

- [ ] Add the Apache-2.0 licence and Caladea notice under the crate's fonts directory.
- [ ] Correct the bundled-font module documentation.
- [ ] Add the family-to-licence regression test.
- [ ] Verify the files are present in the packaged crate.

## Open questions

None. `fonts/LICENSE-Caladea` and `fonts/NOTICE-Caladea` are approved.
