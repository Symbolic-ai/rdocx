# F-116, Cross-viewer acceptance

**Status**: approved
**Sprint**: S28
**Size**: M
**Depends on**: F-107 through F-115

## Problem

The M11 write surface has focused structural and native PowerPoint tests, but
it lacks one generated deck that exercises the complete API together and is
accepted by PowerPoint, Keynote, Google Slides, and LibreOffice Impress. The
sprint gate requires exactly ten slides, clean package validation, and clean
open behavior in all four viewers.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, the M11 write API.
- `docs/hld/04-opc-and-packaging.md`, package integrity.
- `docs/hld/06-presentationml-model.md`, facade and validation.
- `docs/hld/12-testing-strategy.md`, acceptance evidence and pinned native
  applications.
- `docs/hld/14-development-backlog.md`, "F-116, Cross-viewer acceptance".

## Approach

Add one ignored acceptance gate and shared deck-construction helper to the
existing `crates/rpptx/tests/integration.rs`. The helper creates exactly ten
slides from `Presentation::new()` and exercises every F-107 through F-115 write
capability across the deck. It covers slide creation, mutable shapes, every
shape constructor, pictures, text frames and formatting, tables and merges,
slide move, duplicate, and remove, slide and presentation properties, core
properties, ordinary save, and slideshow save. The final deck remains exactly
ten slides after collection edits.

Before any viewer check, require `validate()` to return no issues, save the
deck, reopen it through `Presentation`, verify the final order and representative
relationships, and record its SHA-256. The test artifact stays temporary after
the accepted SHA and evidence are committed.

Automate the locally scriptable viewers:

- Microsoft PowerPoint must match version 16.104, Info.plist build
  16.104.25121423, and AppleScript build 1214. Open the deck, assert ten slides,
  observe no repair flow, then close without saving.
- Keynote must match the installed version and bundle build recorded by the
  acceptance run. Open the deck, assert ten slides, observe no import warning,
  then close without saving.
- LibreOffice Impress must match version 26.2.5.2 and build
  `cd7284b4cbbfeb507e630c1aac019f4157393acb`. Run a headless import and PDF
  export, assert success, and assert the output has ten pages.
- Google Slides is a browser service without a stable exposed application
  version. Import the same file through the signed-in browser, confirm ten
  slides and no conversion error, export it once, and record the acceptance
  date and browser build with the result.

Store the reviewed evidence beside the ignored test as a tracked constant,
following the existing native acceptance records. It names the application,
version or service date, build where available, input SHA-256, slide count,
and clean-open verdict. The test verifies the evidence schema and that every
row binds to the same artifact SHA.

Do not add a new test binary, fixture, source module, or durable generated
deck. No production API is introduced.

## Rejected alternatives

- Test ten independent files. The milestone gate is intended to expose
  interactions in one complete package graph.
- Treat LibreOffice alone as sufficient. The story explicitly requires four
  viewers.
- Claim a Google Slides application version. The service does not publish a
  stable version suitable for this record.
- Check in the generated deck. Deterministic construction and its accepted SHA
  are smaller and avoid another binary fixture.
- Add a second integration-test file. That creates another link target and the
  repository requires additions to the existing binary.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `ten_slide_write_api_deck_validates_and_reopens` | Exactly ten slides exercise F-107 through F-115, validation is clean, and relationships survive reopen |
| package | `ten_slide_write_api_deck_saves_as_presentation_and_show` | `.pptx` and `.ppsx` use correct main content types and valid graphs |
| acceptance, gate | `generated_ten_slide_write_api_deck_opens_clean_in_all_four_viewers` | The same SHA opens without repair or conversion error in PowerPoint, Keynote, Google Slides, and LibreOffice |
| evidence | `cross_viewer_acceptance_evidence_is_complete_and_bound_to_one_artifact` | Four rows name exact version or service date, build where available, ten slides, one SHA, and a clean verdict |

The story gate is named explicitly: a generated 10-slide deck exercising every
feature opens clean in all four viewers.

## HLD impact

- `docs/hld/12-testing-strategy.md`

Document the generated M11 artifact, its SHA-256, the complete feature matrix,
the exact four-viewer procedure, pinned application versions and builds,
Google Slides acceptance date and browser build, and the clean results.

## Risk routing

- External oracle comparison: use the differential-testing rules. Pin local
  application versions, compare viewer-visible structure and clean-open
  behavior rather than XML bytes, and keep each viewer's evidence distinct.
- Parser or serialiser: the generated deck must validate and reopen before
  external use, with valid content types, relationship targets, and schema
  order.
- Bundled template use: confirm the deck starts from the checked-in default
  asset and does not alter or replace it.

The story adds no production dependency, public API, source module, test
binary, feature, or baseline. The Google Slides browser step is a required
human-action acceptance operation, not a skipped automatic gate.

## Hash harness

Expected unchanged. This story adds acceptance coverage only. All 28
deterministic hashes must match.

## Implementation checklist

- [ ] Build one exact ten-slide deck using every F-107 through F-115 feature.
- [ ] Validate, save, reopen, and bind evidence to the artifact SHA-256.
- [ ] Add the ignored four-viewer gate to the existing integration binary.
- [ ] Run PowerPoint, Keynote, and LibreOffice checks at pinned versions.
- [ ] Import and inspect the same artifact in Google Slides through the browser.
- [ ] Record the four-viewer evidence beside the test.
- [ ] Update exactly HLD 12.
- [ ] Run focused checks, risk riders, `/verify --full`, and the hash harness.

## Open questions

None. Google Slides is identified by acceptance date and browser build because
the service exposes no stable application version. The same generated SHA is
used for all four viewers.
