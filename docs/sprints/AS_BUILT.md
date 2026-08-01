# As Built

Append-only completion log. One entry per F-ID, written by `/complete-feature`
at the moment of completion, describing what was actually built rather than what
was planned.

Entries are never edited after the fact. When a later story changes something
recorded here, the later story gets its own entry. The design intent lives in
`docs/hld/`, the plan lives in `.claude/plans/F-XXX-design.md`, and this file is
the record of what happened.

Newest entries at the bottom.

## Entry template

```markdown
### F-XXX, Short title

**Sprint.** SNN
**Completed.** YYYY-MM-DD
**Size.** S | M | L, estimated N days, actual N days

**What was built.** One paragraph. What exists now that did not before, in terms
a reader who has not seen the diff can follow.

**Non-obvious choices.** Anything a future reader would otherwise have to
reverse-engineer from the code, and the reason for it. Rejected alternatives
belong here, not in a comment.

**Deviations from the design plan.** What changed between
`.claude/plans/F-XXX-design.md` and the implementation, and why. "None" is a
valid and common answer.

**Spec sections touched.** The `docs/hld/` sections this story implements or
contradicts. If it contradicts one, say which and confirm the spec was updated.

**Tests.** The test gate from `docs/hld/14-development-backlog.md`, plus any
others added. Name them.

**Hash harness.** Unchanged, or the expected delta and its justification.
Mandatory for every story in M1 through M6.

**Notes for future sessions.** Anything that will not be obvious in three
months. Traps found, assumptions made, follow-up worth filing.
```

## Entries

### F-001, Deterministic font mode

**Sprint.** S01
**Completed.** 2026-07-29
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `FontManager`, the layout engine, and `Document` now expose
an explicit rendering path that loads checked-in bundled fonts and never
discovers host fonts. The existing `bundled-fonts` feature is default-on for
the current `rdocx` consumer.

**Non-obvious choices.** Determinism is explicit rather than ambient. Normal
library rendering keeps its system-font path, while deterministic rendering
returns a clear error when bundled fonts are disabled.

**Deviations from the design plan.** The plan was revised to correct the
manifest's missing default declaration after implementation discovery showed
that the code and HLD already described bundled fonts as default-on. Microscope
pass 1 also strengthened the golden gate to inspect the actual resolved font
bytes rather than compare two calls under one environment.

**Spec sections touched.** `docs/hld/15-build-and-toolchain.md`, "Deterministic
rendering" and "Feature flags".

**Tests.** `deterministic_font_manager_uses_only_bundled_fonts`,
`deterministic_font_manager_requires_bundled_fonts`, and
`deterministic_render_is_independent_of_system_fonts`.

**Hash harness.** Unchanged. F-003 recorded the first baseline after this path
was integrated.

**Notes for future sessions.** The end-to-end test verifies every font buffer
used by the inspected layout belongs to the checked-in bundled set.

### F-002, rust-toolchain.toml

**Sprint.** S01
**Completed.** 2026-07-29
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The repository now selects Rust 1.97.1 with `rustfmt`,
`clippy`, and `wasm32-unknown-unknown` through `rust-toolchain.toml`.

**Non-obvious choices.** The workspace and CI MSRV declarations remain 1.93.
The development toolchain and the compatibility floor answer different
questions.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/15-build-and-toolchain.md`, "Toolchain
pinning".

**Tests.** `rustup show active-toolchain`, installed component and target
inspection, and confirmation of every 1.93 MSRV declaration.

**Hash harness.** Unchanged.

**Notes for future sessions.** Rustup may synchronize channel metadata before
reporting the repository override, even when the toolchain is installed.

### F-003, Output-stability hash harness

**Sprint.** S01
**Completed.** 2026-07-29
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `scripts/hash_harness.py` regenerates seven samples and
compares SHA-256 values for three OOXML parts and deterministic page-one PNG
output at 150 dpi. Check mode is read-only, while update mode requires a
non-empty reason.

**Non-obvious choices.** Missing optional parts are recorded as JSON `null`
rather than omitted. The baseline has 28 sorted entries, so additions,
removals, and byte changes are reported separately.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/12-testing-strategy.md`, "The hash
harness".

**Tests.** Python comparison and reason-refusal unit tests,
`python3 scripts/hash_harness.py --check`, and a temporary writer whitespace
injection that left the structural round-trip test green while changing all
seven `document.xml` digests.

**Hash harness.** Expected initial delta. Added 28 entries with reason
`F-003 initial deterministic baseline`. Manifest SHA-256 is
`9a3c64d61df793b9d8f7203df9cb966fb67201518b4f7fc0f2e68d276aaaca8f`.

**Notes for future sessions.** `invoice` has no `word/numbering.xml`, which is
the single explicit null entry in the initial baseline.

### F-004, Caladea licence and the false OFL claim

**Sprint.** S01
**Completed.** 2026-07-29
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The `rdocx-layout` package now carries the Apache-2.0
licence and Caladea notice beside the four TTFs. Bundled-font documentation
names the correct licence per family, and a test enforces licence coverage.

**Non-obvious choices.** Attribution files live under the crate's `fonts/`
directory so they are included in the published archive with the assets they
cover.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/13-risks-and-open-questions.md`, "Known
defects being carried", and `docs/hld/15-build-and-toolchain.md`, "Packaging".

**Tests.** `every_bundled_font_family_has_a_licence_file`, the full
`rdocx-layout` suite, upstream TTF provenance checks, and the package file list.

**Hash harness.** Unchanged.

**Notes for future sessions.** The checked-in Caladea files match the
`crosextrafonts-20130214` source archive, and the notice fields match embedded
TTF metadata.

### F-005, Fix the image counter

**Sprint.** S01
**Completed.** 2026-07-29
**Size.** S, estimated 1 day, actual 1 day

**What was built.** Imported media names now seed allocation from the greatest
positive numeric suffix rather than the part count. Allocation avoids existing
suffixes and remains collision-free at the finite `usize` boundary.

**Non-obvious choices.** When the greatest suffix cannot be incremented, the
counter wraps to one and skips occupied suffixes until it finds a free name.
Ordinary packages still allocate exactly maximum plus one.

**Deviations from the design plan.** Microscope passes 1 and 2 exposed overflow
and overwrite cases at `usize::MAX`. The plan was clarified to add checked
wrapping and occupied-suffix skipping without adding a media-namer abstraction.

**Spec sections touched.** `docs/hld/04-opc-and-packaging.md`, "Part naming",
and `docs/hld/13-risks-and-open-questions.md`, "Known defects being carried".

**Tests.** `next_image_name_uses_the_highest_existing_index_not_the_part_count`,
`malformed_media_names_do_not_change_the_highest_image_index`,
`occupied_max_image_suffix_wraps_to_a_free_low_number`, and
`max_minus_one_allocates_max_then_wraps_safely`.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Suffix parsing is extension-independent and
reads only consecutive ASCII digits immediately after `image`.

### F-006, Fix the JPEG standalone-marker walk

**Sprint.** S01
**Completed.** 2026-07-29
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The JPEG dimension walk now handles SOI, TEM, and restart
markers without reading nonexistent lengths, validates every length-bearing
segment, tolerates marker fill bytes, and terminates at EOI.

**Non-obvious choices.** The parser remains a small header walk because PDF
output passes JPEG bytes through unchanged and needs only dimensions.

**Deviations from the design plan.** Microscope pass 1 found that EOI was being
skipped like a restart marker. Pass 2 verified immediate termination and the
new trailing-data regression.

**Spec sections touched.** `docs/hld/04-opc-and-packaging.md`, "Media", and
`docs/hld/13-risks-and-open-questions.md`, "Known defects being carried".

**Tests.** `jpeg_restart_marker_before_sof_preserves_dimensions`,
`every_truncated_jpeg_header_returns_without_panicking`, and
`jpeg_bytes_after_eoi_cannot_supply_dimensions`.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** SOF still has to appear before EOI. Trailing
bytes after a completed JPEG cannot supply dimensions.

### F-007, Resolve core properties through the relationship

**Sprint.** S02
**Completed.** 2026-07-30
**Size.** S, estimated 1 day, actual 1 day

**What was built.** Document metadata now resolves through the package-level
core-properties relationship, preserves a custom part target across load and
save, and creates the conventional target only when the relationship is
missing. `rdocx-opc` exposes the standard relationship type publicly.

**Non-obvious choices.** The facade retains a private copy of the stable
relationship URI so the `rdocx 0.3.0` package can still verify against the
published `rdocx-opc 0.3.0` dependency before both move to 0.4.1.

**Deviations from the design plan.** The full packaging gate exposed the
published-dependency compatibility issue after workspace tests passed. An
independent microscope pass approved the private URI because both integration
gates cross-check it against the public constant.

**Spec sections touched.** `docs/hld/04-opc-and-packaging.md`, "Relationship
types" and "Part naming".

**Tests.** `core_properties_at_relationship_target_round_trip_in_place`,
`metadata_round_trip`, focused rdocx and OPC suites, and the clean package
dry-run.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** A non-standard target is authoritative. Saving
must not create an orphaned `/docProps/core.xml` part.

### F-008, Non-consuming setter twins

**Sprint.** S02
**Completed.** 2026-07-30
**Size.** M, estimated 2 days, actual 1 day

**What was built.** All 61 consuming builders across `Paragraph`, `Run`,
`Table`, `Row`, and `Cell` now delegate to non-consuming `set_*` twins.

**Non-obvious choices.** Action builders receive literal `set_*` names as the
story required. Existing builder names and chaining behavior remain unchanged.

**Deviations from the design plan.** The backlog's paragraph-level bold gate
was corrected to obtain a `Run`, where bold formatting belongs. Integration
with F-007 required retaining two independent additions to the shared test
file, followed by a clean microscope pass.

**Spec sections touched.** `docs/hld/03-architecture.md`, "Facade conventions",
`docs/hld/10-bindings-spec.md`, "Two supporting decisions", and
`docs/hld/14-development-backlog.md`, "F-008, Non-consuming setter twins (M)".

**Tests.** `non_consuming_setters_mutate_borrowed_wrappers` and
`non_consuming_setters_match_consuming_builders`, plus all 68 integrated rdocx
integration tests.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep mutation bodies in the in-place setters so
builder and binding behavior remain single-sourced.

### F-009, Cache the layout result

**Sprint.** S02
**Completed.** 2026-07-30
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `Document` caches normal and deterministic layout results
in separate thread-safe slots, exposes cloned page layout access, and clears
both caches across direct mutations and mutable-accessor paths.

**Non-obvious choices.** `Mutex<Option<Arc<LayoutResult>>>` preserves the
`Document: Send + Sync` binding contract. Caller-supplied font layouts remain
uncached because their inputs are not part of a stable document cache key.

**Deviations from the design plan.** None. The approved plan had already
replaced the backlog's thread-local `RefCell<Option<Rc<_>>>` proposal.

**Spec sections touched.** `docs/hld/08-rendering-spec.md`, "Performance",
`docs/hld/10-bindings-spec.md`, "Two supporting decisions",
`docs/hld/13-risks-and-open-questions.md`, "Known defects being carried", and
`docs/hld/14-development-backlog.md`, "F-009, Cache the layout result (M)".

**Tests.** `rendering_all_pages_performs_one_layout`,
`document_mutation_invalidates_cached_layout`,
`mutable_accessor_invalidates_cached_layout`,
`font_modes_use_isolated_layout_caches`, and
`document_remains_send_and_sync`.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Any new mutable accessor must invalidate both
layout modes before returning the borrow.

### F-010, Reserve crate names

**Sprint.** S02
**Completed.** 2026-07-30
**Size.** S, estimated 1 day, actual 1 day

**What was built.** Fourteen approved `oxml-*` and `rpptx*` names were
published as dependency-free `0.0.0` placeholders and verified as owned by
`mantissaman`.

**Non-obvious choices.** Python and wasm binding names were excluded because
their documented distribution channels are PyPI and npm. Publications ran
sequentially through crates.io's rolling new-crate rate limit.

**Deviations from the design plan.** None. The registry required repeated
cooldown windows, and the workflow stopped after every HTTP 429 before
resuming at the exact rejected name.

**Spec sections touched.** `docs/hld/13-risks-and-open-questions.md`,
"Q2, PyPI name availability", and `docs/hld/15-build-and-toolchain.md`,
"Publishing".

**Tests.** Exact `cargo info <name>@0.0.0` and owner checks for all fourteen
names, package inspection, publish dry-runs, and archive-size checks.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** The placeholders reserve names only. They expose
no implementation API and do not change any existing `rdocx 0.3.0` crate.

### F-011, Pin unit truncation behaviour

**Sprint.** S02
**Completed.** 2026-07-30
**Size.** S, estimated 1 day, actual 1 day

**What was built.** Fractional positive and negative tests now pin truncation
toward zero for every float constructor on `Length`, `Twips`, and `Emu`.

**Non-obvious choices.** The vectors cross the half-unit boundary so temporary
rounding mutations fail while the existing production casts remain unchanged.

**Deviations from the design plan.** Microscope pass 1 corrected one invalid
HLD heading citation. Pass 2 found no defects or smells.

**Spec sections touched.** `docs/hld/11-migration-plan.md`, "Preserve
behaviour, do not improve it", and `docs/hld/12-testing-strategy.md`, "New
tests the extracted crates need".

**Tests.** `length_float_constructors_truncate_toward_zero`,
`twips_float_constructors_truncate_toward_zero`, and
`emu_float_constructors_truncate_toward_zero`, including temporary rounding
mutations that made every gate fail.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** A change from casts to rounding is a behavior
change even when whole-unit conversion tests continue to pass.

### F-012, Tag v0.4.1

**Sprint.** S02
**Completed.** 2026-07-30
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The workspace was published as seven lockstep rdocx crates
at 0.4.1 from the reviewed S02 SHA. A dedicated `/release` command now owns
`v*` tags and publication, while the tag workflow verifies the deterministic
hash baseline and publishes only the approved rdocx allowlist.

**Non-obvious choices.** The published `v0.4.0` mainline was merged into S02
before release, preserving its contract changes and retargeting the planned
0.3.1 release to 0.4.1. The fourteen `oxml-*` and `rpptx*` placeholders remain
at 0.0.0 until PowerPoint development is complete.

**Deviations from the design plan.** The original plan targeted 0.3.1 before
the separate 0.4.0 release appeared. The reconciled plan and release evidence
target 0.4.1. The publication workflow retains deliberate registry-index waits
because real publication is explicitly allowlisted instead of workspace-wide.

**Spec sections touched.** `docs/hld/11-migration-plan.md`, release boundary,
`docs/hld/13-risks-and-open-questions.md`, release risks,
`docs/hld/14-development-backlog.md`, M1 gate and F-012, and
`docs/hld/15-build-and-toolchain.md`, "Publishing" and "Release process".

**Tests.** `/verify --full` passed at
`6e02a4b6417c9bb0c245237bdf8168dd06310c39`. The package dry-run produced
exactly seven archives below 10 MiB, including all 20 TTFs and required licence
files in `rdocx-layout`. GitHub Actions run 30522998328 passed, every exact
`cargo info <crate>@0.4.1` lookup succeeded, all owners were `mantissaman`, and
the GitHub release tag peeled to the reviewed SHA.

**Hash harness.** Unchanged. All 28 entries matched locally and on the Linux
publication runner.

**Notes for future sessions.** The release workflow must remain restricted to
the seven rdocx crates until PowerPoint development is complete. After S02 is
merged, forward-merge `main` into `feature/release-0.5.0` before that release
branch continues.

### F-013, Create oxml-core

**Sprint.** S03
**Completed.** 2026-07-30
**Size.** M, estimated 2 days, actual 1 day

**What was built.** A new private `oxml-core` workspace crate now owns staged
copies of the format-neutral error, units, raw XML, XML text, core properties,
and `Length` implementations. It also provides shared namespace-aware XML
helpers and public XML text handling with focused event coverage.

**Non-obvious choices.** The crate remains at 0.0.0 with `publish = false`.
The existing Word implementations stay in place until F-015 and F-016 can
switch the facades without putting an unpublished dependency into a published
rdocx package.

**Deviations from the design plan.** None. The approved plan already specified
the staged copy and delayed facade switch.

**Spec sections touched.** `docs/hld/03-architecture.md`, "Three families, one
workspace", `docs/hld/11-migration-plan.md`, "The facade trick" and "Order of
operations", and `docs/hld/15-build-and-toolchain.md`, "Publishing".

**Tests.** The moved unit, raw XML, core properties, and XML text tests,
`xml_text_handles_cdata_mixed_nested_and_general_refs`, workspace compilation,
package verification, and the dependency-tree direction check.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Do not publish this crate or connect published
rdocx packages to it until the PowerPoint development publication boundary is
explicitly lifted.

### F-014, New unit types

**Sprint.** S03
**Completed.** 2026-07-30
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml_core::units` now exposes `Centipoints`, `Angle`, and
`Percent1000` with the exact OOXML storage scales. `Length::mm` adds direct
millimetre construction through 36,000 EMUs per millimetre.

**Non-obvious choices.** Float conversions retain Rust cast semantics and
truncate positive and negative fractional values toward zero. No generic unit
abstraction or `Length::to_mm` accessor was added.

**Deviations from the design plan.** None.

**Spec sections touched.** None. The existing glossary and DrawingML model
already specify the implemented types and scales.

**Tests.** `centipoints_round_trip_points`, `angle_round_trip_degrees`,
`percent1000_round_trip_percent`,
`new_unit_float_constructors_truncate_toward_zero`, and
`length_millimetres_round_trip`.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Preserve the pinned truncation rule when these
types gain format consumers.

### F-017, App and custom properties

**Sprint.** S03
**Completed.** 2026-07-30
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml-core` now provides a shared application-properties
union for Word and PowerPoint plus typed custom properties for text, signed
integers, floating point values, Booleans, file times, and empty values.
Parsers preserve child order and retain unsupported property subtrees as raw
XML.

**Non-obvious choices.** Parsed application children replay their encountered
order, while constructed values use canonical schema order. Unsupported custom
value types remain raw XML instead of being coerced to strings.

**Deviations from the design plan.** Microscope passes 1 and 2 found malformed
root acceptance and inconsistent self-closing typed values. Both were fixed
with regression tests before the clean pass 3.

**Spec sections touched.** None. The existing scope, architecture, and testing
documents already describe the shared model.

**Tests.** `word_app_properties_round_trip_without_presentation_fields`,
`powerpoint_app_properties_round_trip_without_word_fields`,
`unknown_app_property_subtree_is_preserved_verbatim`,
`custom_property_value_types_round_trip`,
`unknown_custom_property_value_is_preserved_verbatim`, and malformed-root and
self-closing-value regressions.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep unknown XML preservation and schema child
order intact when adding further extended-property variants.

### F-018, Create oxml-opc

**Sprint.** S04
**Completed.** 2026-07-30
**Size.** M, estimated 2 days, actual 1 day

**What was built.** A new private `oxml-opc` workspace crate now owns a staged
copy of the format-neutral OPC package, content-type, relationship, and error
implementation. Generic `OpcPackage::new`, `OpcPackage::with_main_part`, and
`ContentTypes::minimal` replace the DOCX-specific public constructors in the
new crate.

**Non-obvious choices.** The crate remains at 0.0.0 with `publish = false` and
depends only on `quick-xml`, `thiserror`, and `zip`. The existing `rdocx-opc`
implementation and every released rdocx consumer remain untouched until the
real shared crates have an approved publication path.

**Deviations from the design plan.** None.

**Spec sections touched.** None. The existing architecture, OPC, migration,
testing, and build specifications already describe the staged extraction.

**Tests.** Eleven moved OPC tests rebuilt around private DOCX helpers,
`minimal_content_types_contain_only_universal_defaults`,
`with_main_part_resolves_and_round_trips`, independent dependency inspection,
local package verification, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep `rdocx-opc` and released consumers on the
published implementation until the deferred F-022 cutover is packageable.

### F-019, PresentationML relationship and content types

**Sprint.** S04
**Completed.** 2026-07-30
**Size.** S, estimated 1 day, actual 1 day

**What was built.** `oxml-opc` now exposes package, shared-property, and
PresentationML relationship constants plus universal, shared-property, and
PresentationML content-type constants. The generic minimal constructor reuses
the universal values.

**Non-obvious choices.** Word-specific MIME constants remain outside this
story. The table-driven gate lists every public value and asserts uniqueness,
the correct relationship namespace, the `application` MIME top-level type,
and the absence of whitespace or extra slashes.

**Deviations from the design plan.** Microscope pass 1 found that the initial
MIME-shape assertion accepted arbitrary top-level types and extra slashes. The
gate was tightened before clean pass 2.

**Spec sections touched.** None. The existing OPC and PresentationML
specifications already enumerate the required constants.

**Tests.** `relationship_and_content_type_constants_are_unique_and_well_formed`,
all-target compilation, a 12,155-byte integrated local package, dependency
inspection, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Add future package constants to the exhaustive
table so a new value cannot bypass namespace and MIME-shape classification.

### F-020, oxml-opc reads a pptx

**Sprint.** S04
**Completed.** 2026-07-30
**Size.** M, estimated 2 days, actual 1 day

**What was built.** A code-built PowerPoint package fixture writes and reopens
a presentation, slide, and slide-layout graph entirely in memory. It proves
main-part discovery, relationship round-tripping, normalized package keys, and
parent-directory layout resolution.

**Non-obvious choices.** The fixture lives in the existing `package.rs` test
module. It adds no binary fixture, integration-test target, dependency,
production API, or production-code change.

**Deviations from the design plan.** Sprint review pass 1 required a second
fixture built directly as a valid PresentationML ZIP, independent of
`OpcPackage::write_to`, so the M2 real-package gate does not rely on a
self-round-trip.

**Spec sections touched.** None. The existing OPC and testing specifications
already require this exact package graph.

**Tests.** `pptx_package_resolves_main_slide_and_layout_parts`,
`presentation_layout_target_resolves_one_directory_up`,
`independently_built_pptx_opens_and_resolves_relationships`, all integrated
`oxml-opc` tests, and the integrated full gate. The original two named tests
were observed failing for their intended reasons before completion.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** OPC navigation is now proven against both Word
and PowerPoint package shapes before any PresentationML parser is introduced.
The close workflow preserves the S32.2 target for publication-bound cutovers
instead of forcing them through every intervening sprint.

### F-021, Zip-slip hardening tests

**Sprint.** S04
**Completed.** 2026-07-30
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The `oxml-opc` reader now normalizes ZIP entry names before
classifying package metadata and parts. Root-escaping traversal clamps to the
package root, and absolute entries become canonical leading-slash part names.

**Non-obvious choices.** Normalization uses OPC package-path algebra rather
than host filesystem canonicalization, and no archive entry is extracted to
the filesystem. The released `rdocx-opc` reader remains unchanged.

**Deviations from the design plan.** None.

**Spec sections touched.** None. The existing OPC and testing specifications
already require canonical package names and both hostile cases.

**Tests.** `zip_entry_that_escapes_root_is_clamped_to_root`,
`absolute_zip_entry_is_normalized_to_package_root`, the opaque package
round-trip, deterministic save, all integrated OPC parser tests, and the full
gate. Both hostile cases were observed failing before the normalization fix.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Preserve package-path normalization before raw
metadata classification when the reader gains further ZIP validation.

### F-023, oxml-media format sniffing

**Sprint.** S05
**Completed.** 2026-07-30
**Size.** M, estimated 2 days, actual 1 day

**What was built.** The new dependency-free `oxml-media` crate classifies PNG,
JPEG, GIF, BMP, TIFF, WebP, SVG, EMF, and WMF from bytes. It exposes canonical
extension and content-type mappings plus sniff-first resolution with extension
fallback and a PNG default.

**Non-obvious choices.** The staged crate remains at version 0.0.0 with
publication disabled. Released rdocx consumers still use their existing image
paths until the deferred cutover after PowerPoint development.

**Deviations from the design plan.** Microscope review tightened SVG prolog
handling and standard WMF recognition before the final clean pass.

**Spec sections touched.** None. The existing media and migration contracts
already specify the staged crate and sniffing precedence.

**Tests.** Magic-byte coverage for every format, canonical mappings,
misleading-extension precedence, unknown-image fallback, every-prefix safety,
the dependency and package riders, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep format detection byte-led when the released
consumers move to `oxml-media`.

### F-024, Image probing and DPI

**Sprint.** S05
**Completed.** 2026-07-30
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `oxml-media` now probes pixel dimensions, optional DPI,
bit depth, channel count, and alpha metadata from bounded PNG, JPEG, GIF, BMP,
and WebP headers.

**Non-obvious choices.** All fixtures are constructed in code. Unsupported,
inconsistent, and truncated headers return `None`, and every indexed read is
bounds checked without adding a decoder dependency.

**Deviations from the design plan.** Microscope review found and drove fixes
for BMP mask placement and alpha classification plus WebP frame, profile,
canvas, RIFF-padding, and parity validation before clean pass 4.

**Spec sections touched.** None. The implementation follows the existing
binary-header parsing contract.

**Tests.** PNG `pHYs`, JFIF density units, EXIF before progressive SOF, GIF,
BMP, all WebP layouts, every-prefix truncation, 22 integrated media tests, and
the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Preserve bounded parsing and the truncation
loops whenever another header layout is accepted.

### F-025, MediaNamer

**Sprint.** S05
**Completed.** 2026-07-30
**Size.** S, estimated 1 day, actual 1 day

**What was built.** `MediaNamer` scans exact directory and stem matches for
positive numeric suffixes, allocates after the maximum, and wraps safely after
`usize::MAX` without emitting zero or reusing an occupied name.

**Non-obvious choices.** The caller's extension is preserved verbatim. The
allocator retains occupied suffixes so integer wrap can search safely from one.

**Deviations from the design plan.** Microscope pass 1 found that root package
parts were not recognized. The fix normalizes the empty root split back to `/`
and adds root, trailing-slash, and non-PNG regression coverage.

**Spec sections touched.** None. The existing media contract already requires
maximum-suffix allocation.

**Tests.** All four F-005 sentence-named regressions, root and directory
normalization, caller extension handling, 22 integrated media tests, and the
integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** The released rdocx allocator remains active
until the deferred consumer cutover.

### F-026, native_size with explicit DPI

**Sprint.** S05
**Completed.** 2026-07-30
**Size.** S, estimated 1 day, actual 1 day

**What was built.** `ImageInfo::native_size` returns dependency-free
`NativeSize` values with explicit width and height EMU fields. Each axis uses
finite positive declared DPI when available and otherwise uses the caller's
finite positive default.

**Non-obvious choices.** Pixel conversion uses 914400 EMU per inch and
truncates toward zero. Invalid effective DPI or dimensions outside the `i64`
range return `None`.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/03-architecture.md`,
`docs/hld/04-opc-and-packaging.md`, and
`docs/hld/14-development-backlog.md` now state the dependency-free return type.

**Tests.** Declared-DPI precedence, independent per-axis fallback, fractional
truncation, invalid input, dependency and package riders, 22 integrated media
tests, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** The later consumer cutover must supply its own
default DPI and convert no units outside this API.

### F-029, Create oxml-layout

**Sprint.** S06
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** The unpublished `oxml-layout` crate now stages
format-neutral layout output, font management, bundled deterministic fonts,
and layout errors without changing a released consumer.

**Non-obvious choices.** Bundled fonts are always available for deterministic
rendering. The default `system-fonts` feature controls only host font discovery,
and the no-default path keeps the same bundled archive.

**Deviations from the design plan.** Sprint review pass 1 found that two HLD
passages still described an older bundled-fonts-off no-default path. The
implementation followed the intended boundary, and the stale current-intent
wording was corrected during sprint remediation.

**Spec sections touched.** `docs/hld/12-testing-strategy.md` and
`docs/hld/15-build-and-toolchain.md` now state that the no-default path disables
system discovery while retaining bundled deterministic fonts.

**Tests.** Default and no-default font-manager tests, empty-font error handling,
bundled-font licence coverage, dependency isolation, archive contents and size,
released-crate isolation, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep deterministic fonts inside `oxml-layout`
when consumers migrate. System discovery remains an optional capability, not a
determinism requirement.

### F-030, Decouple line.rs

**Sprint.** S06
**Completed.** 2026-07-31
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `oxml-layout` now has a greedy line breaker with owned
layout types for alignment, tabs, leaders, underline, and spacing. Explicit
wrapping control can retain width overflow while forced line, page, and column
breaks continue to split content.

**Non-obvious choices.** Tab positions and exact or minimum spacing are stored
in points. Multiple spacing stores a factor, and the staged boundary contains
neither twips nor stringly line rules.

**Deviations from the design plan.** Microscope pass 1 strengthened the copied
tests to use deterministic bundled fonts and made the leader regression prove
real glyph shaping. Production behavior remained on the approved design.

**Spec sections touched.** None. The existing migration and rendering contracts
already define the owned line-breaking boundary and deferred rdocx converter.

**Tests.** All 11 copied compatibility tests, four owned spacing and wrapping
regressions, deterministic leader shaping, both feature modes, dependency and
package riders, released-line-breaker isolation, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** The later rdocx cutover must perform schema-unit
conversion before constructing these types. Do not move Word-specific enums
back across this boundary.

### F-031, Transform

**Sprint.** S06
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml-layout` now exports a concrete six-coefficient affine
transform with rotation about a point, self-first composition, point
application, exact identity checks, and four-corner rectangle bounds.

**Non-obvious choices.** `self.then(next)` applies `self` first and `next`
second, matching the point equations and PDF `cm` concatenation order. Identity
comparison is exact so small intentional transforms are never discarded.

**Deviations from the design plan.** Microscope pass 1 strengthened the
composition gate with fully nonzero matrices and replaced an exact quarter-turn
bounds case with a negative 30-degree rotation. Production algebra was already
correct.

**Spec sections touched.** None. The existing rendering and testing contracts
already specify the matrix representation and composition order.

**Tests.** Identity neutrality, fractional and positive rotation, fully nonzero
hand-computed PDF composition, negative-rotation four-corner bounds, exact
identity, package and dependency riders, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Convert DrawingML rotation units to degrees
before this boundary, and preserve self-first composition when group transforms
begin consuming the type.

### F-032, Path and PathCommand

**Sprint.** S07
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml-layout` now exports backend-neutral path commands,
fill rules, conservative bounds, and constructors for rectangles, rounded
rectangles, and four-cubic ellipses.

**Non-obvious choices.** Bounds include cubic control points without solving
curve extrema. Rounded rectangles use one circular radius clamped from zero to
half the shorter side.

**Deviations from the design plan.** None.

**Spec sections touched.** None. The existing rendering contract already
defines the path representation and conservative bounds.

**Tests.** Ellipse bounds within the control hull, cubic control-point bounds,
empty paths, fill-rule independence, closure, and radius clamping.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Backend path conversion must preserve command
order and choose the fill operator from `FillRule`.

### F-033, Paint and Stroke

**Sprint.** S07
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml-layout` now exports solid, linear, radial, and tiled
paint plus gradient stops, line caps, line joins, and arbitrary dash arrays for
strokes.

**Non-obvious choices.** Only a one-stop gradient is normalized at
construction, becoming solid paint. Empty and multi-stop gradients remain
unchanged for the later backend normalization stage.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/14-development-backlog.md` now records
F-036 as an explicit dependency because tiled paint stores `MediaId`.

**Tests.** Single-stop linear and radial degradation, multi-stop preservation,
stroke defaults, tile media identity, package, dependency, and feature-mode
checks.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Sorting, clamping, and duplicate-stop precedence
remain backend construction work and must not be moved silently into this
model.

### F-034, Path and Group arms

**Sprint.** S07
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** The staged positioned-element model now carries painted
paths and nested groups with transforms, clips, opacity, effects, and children.
Page backgrounds, layout diagnostics, and constructors for the two
non-exhaustive result structs are also available.

**Non-obvious choices.** `Diagnostic` initially carries one message.
`PositionedElement` and `Effect` are non-exhaustive enums, while `PageFrame`
and `LayoutResult` are non-exhaustive structs with neutral constructor defaults.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/08-rendering-spec.md` and
`docs/hld/14-development-backlog.md` now state the exact non-exhaustive targets,
minimal diagnostic shape, constructor contract, and unpublished staging
boundary.

**Tests.** Path and group payload preservation, transform direction, neutral
page and result defaults, external constructor doctests, and the integrated
full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Group transforms map child-local coordinates
into their parent. Preserve that direction in every recursive backend.

### F-035, The walk helper

**Sprint.** S07
**Completed.** 2026-07-31
**Size.** S, estimated 1 day, actual 1 day

**What was built.** `oxml-layout::walk` visits every non-group element once in
depth-first document order while carrying its accumulated child-to-page
transform.

**Non-obvious choices.** Group containers are not yielded. With self-first
composition, each group transform is composed before the accumulated parent
transform.

**Deviations from the design plan.** None.

**Spec sections touched.** None. The existing recursion-hazard contract already
defines the helper and its consumers.

**Tests.** Three-deep traversal with three ordered leaves, hand-computed nested
transform order, group exclusion, and root identity.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Font, image, and link collection passes must use
this helper when the PDF backend migrates.

### F-036, MediaId

**Sprint.** S07
**Completed.** 2026-07-31
**Size.** S, estimated 1 day, actual 1 day

**What was built.** Staged output and line image values now use a stable
content-addressed `MediaId` derived from raw bytes instead of relationship-local
embed identifiers.

**Non-obvious choices.** The compact handle uses fixed 64-bit FNV-1a and is
documented as a renderer key rather than a collision-free content guarantee.
This story adds no media store.

**Deviations from the design plan.** None.

**Spec sections touched.** None. The existing rendering contract already
defines the content-addressed handle and replacement boundary.

**Tests.** Equal bytes collapse to one set key, different fixtures differ,
staged output images use the handle, and line conversion preserves it.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Relationship resolution must happen before
constructing this renderer key. Part-local relationship names must not cross
the shared layout boundary.

### F-037, Create oxml-pdf

**Sprint.** S08
**Completed.** 2026-07-31
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The workspace now contains an unpublished `oxml-pdf`
backend at version 0.0.0. It consumes `oxml-layout` and `oxml-media`, keeps the
eight moved backend gates, and removes the copied image-header parsers.

**Non-obvious choices.** The staged crate has no dependency on an `rdocx-*` or
`rpptx-*` crate. It remains excluded from publication while the PowerPoint
development line is incomplete.

**Deviations from the design plan.** A normal staged package archive was built,
but extracted verification resolves the unpublished 0.0.0 dependencies from
crates.io placeholder packages. The reviewed package rider therefore used
`cargo package --no-verify`. The 19.6 KiB archive stayed below the 10 MiB gate,
and no package was published.

**Spec sections touched.** `docs/hld/03-architecture.md` and
`docs/hld/08-rendering-spec.md` now record the staged backend boundary and its
dependency direction.

**Tests.** Fifteen staged backend tests, including the eight moved gates,
dependency-tree inspection, archive inspection and size, and the integrated
full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep `oxml-pdf` unpublished and independent of
released format crates until the shared publication and cutover sprint.

### F-038, Golden-PNG harness

**Sprint.** S08
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `Document::to_pdf_deterministic` exposes the bundled-font
PDF path, and the golden harness rasterises page one of all seven samples at
150 DPI before comparing decoded RGBA dimensions and SHA-256 digests exactly.

**Non-obvious choices.** The manifest records `pdftoppm version 26.01.0` and
contains digests rather than binary fixtures. Comparison has no tolerance.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/12-testing-strategy.md` and
`docs/hld/15-build-and-toolchain.md` now define the deterministic facade,
rasterizer identity, manifest contents, and exact pixel gate.

**Tests.** Four harness unit tests, seven exact sample comparisons, a synthetic
one-pixel `proposal` failure that names only that sample, facade coverage,
package checks, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Update the golden manifest only for a declared,
reviewed rendering change with a non-empty reason. Continue comparing decoded
pixels exactly.

### F-039, Global CTM flip

**Sprint.** S08
**Completed.** 2026-07-31
**Size.** L, estimated 4 days, actual 1 day

**What was built.** Both PDF writers now emit one page-level
`q 1 0 0 -1 0 H cm`. Text and images cancel that outer reflection locally,
while lines and rectangles use top-left coordinates and link annotations stay
outside the content transform.

**Non-obvious choices.** The correct image operator is
`[w 0 0 -h x y+h]`. Omitting the added height moves images outside the page
under the outer CTM.

**Deviations from the design plan.** The original operator used `y`, which was
corrected to `y+h` before integration. Poppler 26.01.0 then produced exactly
four one-pixel vertical antialias swaps at x 112, two in `invoice` and two in
`quote`. The user approved that exact delta, the manifest changed once with an
F-039 reason, and all seven buffers now compare exactly.

**Spec sections touched.** `docs/hld/08-rendering-spec.md`,
`docs/hld/12-testing-strategy.md`, `docs/hld/13-risks-and-open-questions.md`,
and `docs/hld/14-development-backlog.md` now record the corrected operator,
mirrored released-backend exception, and exact rendering evidence.

**Tests.** Page CTM, top-left geometry, upright text and images, unchanged
annotation coordinates, exact pre-update four-pixel evidence, seven exact
post-update buffers, one-pixel injection rejection, and the integrated full
gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Preserve the same page and image matrices in
both PDF writers until the F-046 cutover removes the released duplicate. Do not
introduce a pixel tolerance.

### F-040, Group rendering

**Sprint.** S09
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** The staged PDF writer recursively emits nested groups with
balanced graphics-state saves and restores, local matrices, optional clipping,
shared opacity states, and children in document order.

**Non-obvious choices.** Group clips reuse F-041's private geometry emitter and
group opacity reuses F-044's document-wide alpha registry. Effects remain
staged, and the raster group path remains owned by F-045.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/08-rendering-spec.md`,
`docs/hld/12-testing-strategy.md`, and `docs/hld/14-development-backlog.md` now
describe recursive group emission and its exact ordering.

**Tests.** `three_deep_groups_balance_graphics_state`, transform ordering,
non-zero and even-odd clipping, shared group opacity, staged effects, the exact
seven-sample golden gate, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Content emission must preserve group boundaries.
Flattening with `walk` is correct for collection passes but would lose clip and
opacity scope here.

### F-041, Path rendering

**Sprint.** S09
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Staged PDF paths now emit move, line, cubic, and close
geometry plus solid fill and stroke state. Paint selection covers `f`, `f*`,
`S`, `B`, and `B*` with balanced graphics state.

**Non-obvious choices.** The private geometry emitter also serves group clips.
Gradient and tile components remain staged, while a supported solid component
still renders when the other paint component is not yet supported.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/08-rendering-spec.md`,
`docs/hld/12-testing-strategy.md`, and `docs/hld/14-development-backlog.md` now
state the supported geometry, stroke state, and paint operators.

**Tests.** Fill-only, stroke-only, combined, even-odd, command-order, cap, join,
miter, dash, staged-component, exact seven-sample golden, and integrated full
gates.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** F-043 owns gradient resources. Preserve the
single geometry emitter so visible paths and group clips cannot diverge.

### F-042, Rewrite the three collection passes on walk

**Sprint.** S09
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Font usage, image registration, and every link annotation
pass now traverse nested leaves through `walk`. Image resources, annotations,
and recursive emission share depth-first leaf ordinals, and grouped link
rectangles apply the accumulated transform before PDF page conversion.

**Non-obvious choices.** A private per-page emission state carries the resource
maps and leaf ordinal through recursion. It keeps identity aligned without
public model fields, pointer keys, or parallel traversal implementations.

**Deviations from the design plan.** None. Microscope pass 1 strengthened the
grouped-link test to prove the page `/Annots` reference as well as the
transformed annotation dictionary.

**Spec sections touched.** `docs/hld/08-rendering-spec.md`,
`docs/hld/12-testing-strategy.md`, `docs/hld/13-risks-and-open-questions.md`,
and `docs/hld/14-development-backlog.md` now close the R3 mitigation with the
implemented traversal and identity contract.

**Tests.** Nested font subsetting, nested XObject registration and use, nested
transformed link annotations, depth-first leaf identity, top-level stability,
the exact seven-sample golden gate, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Any new resource collection pass must use the
same depth-first leaf contract as recursive content emission.

### F-044, ExtGState alpha

**Sprint.** S09
**Completed.** 2026-07-31
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The staged PDF writer allocates one document-wide
`/ExtGState` for each normalized non-opaque alpha value, writes matching `CA`
and `ca`, and exposes only the states each page uses. Text, lines, rectangles,
solid paths, and group opacity share the registry.

**Non-obvious choices.** Keys use the serialized `f32` value, finite inputs are
clamped, negative zero is normalized, and non-finite values remain opaque. A
path with different fill and stroke alpha repeats its geometry so each paint
operation can select the correct state.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/08-rendering-spec.md`,
`docs/hld/12-testing-strategy.md`, `docs/hld/13-risks-and-open-questions.md`,
and `docs/hld/14-development-backlog.md` now describe shared PDF alpha and its
deterministic raster gate.

**Tests.** Equal-state reuse, distinct states, matching `CA` and `ca`, opaque
content, shared path alpha, differing fill and stroke alpha, exact midpoint
raster compositing, the exact seven-sample golden gate, and the integrated full
gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Reuse this private registry for any new PDF
primitive with alpha. Do not allocate graphics states per element.

### F-043, Gradient shading dictionaries

**Sprint.** S10
**Completed.** 2026-07-31
**Size.** L, estimated 4 days, actual 1 day

**What was built.** The staged PDF writer now emits deterministic type 2
shading patterns, axial and radial shadings, and type 3 stitching functions
over interval type 2 functions. Page-local pattern resources and fill or stroke
operators address each gradient occurrence by stable depth-first identity.

**Non-obvious choices.** Pattern matrices compose the accumulated group
transform with the global page flip so paint and path geometry share top-left
coordinates. Stops are clamped and sorted, the last repeated offset wins, and
stop alpha uses the documented opaque DeviceRGB fallback over white.

**Deviations from the design plan.** Microscope pass 1 corrected the pattern
matrix to include the global page flip. Pass 2 strengthened the page-local
resource test so it fails if pattern names leak between pages.

**Spec sections touched.** `docs/hld/08-rendering-spec.md`,
`docs/hld/12-testing-strategy.md`, and `docs/hld/14-development-backlog.md` now
describe the implemented PDF gradient resource graph, normalization, matrices,
operators, and exact sampled gate.

**Tests.** Linear and radial resource structure, stop normalization, mixed
solid paint, gradient stroke operators, page-local resources, and exact rotated
gradient samples under Poppler 26.01.0. The integrated 57-test `oxml-pdf`
suite, seven-buffer golden gate, and full workspace gate also pass.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep PDF and raster gradient stop normalization
aligned. The integrated gate completed normally after a worker-only duplicate
hash rerun had stalled in the macOS loader without changing files or evidence.

### F-045, Rasteriser groups, paths, gradients, dashes, and background

**Sprint.** S10
**Completed.** 2026-07-31
**Size.** L, estimated 4 days, actual 1 day

**What was built.** The staged tiny-skia backend now recursively renders group
transforms and intersecting clips, composites group opacity once per subtree,
draws backend-neutral paths with solid or gradient paint, honours line and path
dashes, and paints supported page backgrounds.

**Non-obvious choices.** Scoped opacity uses a scratch pixmap so overlapping
children are not attenuated individually. Non-extended gradient domains receive
an explicit mask, while tile paint and group effects remain outside the S10
contract.

**Deviations from the design plan.** Microscope pass 1 added explicit raster
stop normalization with clamping, stable sorting, and last-repeated-offset
semantics to match F-043. Pass 2 was clean.

**Spec sections touched.** `docs/hld/08-rendering-spec.md`,
`docs/hld/12-testing-strategy.md`, and `docs/hld/14-development-backlog.md` now
describe recursive raster state, paint translation, dashes, backgrounds, and
the deterministic sampled gates.

**Tests.** Twelve raster tests cover the rotated-rectangle and dashed-line
gates, nested transform order, clip intersection, group opacity, fill rules,
linear and radial gradients, gradient domains and normalization, path dashes,
and page backgrounds. The integrated 57-test `oxml-pdf` suite, exact golden
gate, deliberate one-pixel rejection, and full workspace gate also pass.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Preserve recursive draw order and scoped state.
Flattening groups would lose clip and opacity semantics even though `walk` is
correct for collection passes.

### F-052, Create oxml-drawing and namespace constants

**Sprint.** S12
**Completed.** 2026-07-31
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The workspace now contains the unpublished `oxml-drawing`
crate with the DrawingML, relationships, and package namespace constants used
by later model stories.

**Non-obvious choices.** The crate remains at version 0.0.0 with publication
disabled. It started without dependencies so the format-neutral boundary was
established before parsers were added.

**Deviations from the design plan.** None.

**Spec sections touched.** No HLD file changed. The implementation follows the
existing crate boundary in `docs/hld/03-architecture.md` and development
publication policy in `docs/hld/15-build-and-toolchain.md`.

**Tests.** Namespace URI assertions, workspace membership and publication
state checks, package inspection, dependency inspection, and the integrated
full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep every `oxml-*` production edge
format-neutral. The final S12 graph contains no `rdocx-*` or `rpptx-*` edge.

### F-053, OrderedRawChildren

**Sprint.** S12
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `OrderedRawChildren` stores unmodelled XML subtrees at
caller-defined schema boundaries so modelled children can be written in schema
order without moving or dropping unknown siblings.

**Non-obvious choices.** The helper is concrete and schema-boundary based. It
does not own a generic parser policy or hide which child sequence the caller
implements.

**Deviations from the design plan.** None.

**Spec sections touched.** No HLD file changed. The helper implements the
existing child-order and verbatim-preservation contracts in
`docs/hld/05-drawingml-model.md`.

**Tests.** Raw children before, between, and after modelled children, multiple
children at one boundary, byte-for-byte nested subtree preservation, and the
integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Owning parsers still decide their schema
boundaries. Do not append unknown children at the end of a parent.

### F-054, Colour choices

**Sprint.** S12
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `ColorChoice` now models sRGB, scheme, system, and preset
DrawingML colours with validated RGB values, prefix-tolerant parsing,
fixed-prefix writing, and ordered raw-child preservation.

**Non-obvious choices.** System colours preserve `lastClr` as their portable
fallback. Unknown child subtrees remain byte-for-byte raw rather than becoming
partially modelled data.

**Deviations from the design plan.** Microscope pass 1 corrected the shared
`OxmlError` conversion so parser errors retain their source contract. Pass 2
was clean.

**Spec sections touched.** No HLD file changed. The four choices and
preservation behaviour were already specified in
`docs/hld/05-drawingml-model.md`.

**Tests.** All four colour forms parse and round-trip, malformed RGB and system
fallback values fail, unknown nested children retain their exact bytes, and
the integrated full gate passes.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Parse only children the shared renderer
consumes. Preserve every other subtree at its original boundary.

### F-055, The colour transform stack

**Sprint.** S12
**Completed.** 2026-07-31
**Size.** L, estimated 4 days, actual 1 day

**What was built.** All 28 DrawingML colour transforms parse, serialize, and
apply in document order. A readable 40-case table records exact RGBA sampled
from PowerPoint 16.104 build 16.104.25121423.

**Non-obvious choices.** The oracle starts from a PowerPoint-authored native
shape shell, validates the transformed deck without repair, and captures each
shape through the native clipboard PNG payload. This replaced the PowerPoint
`save as picture` command, which returned success without creating a file on
the pinned build.
Linear-light, HSL, alpha, and PNG quantization rules are kept explicit.

**Deviations from the design plan.** The approved plan was revised before
completion to record the clipboard transport. Microscope pass 1 found that
explicit empty start and end transform pairs were preserved raw instead of
modelled. The parser now models those pairs while preserving unexpected
nonempty transform content verbatim. Pass 2 was clean.

**Spec sections touched.** `docs/hld/05-drawingml-model.md`, "Colour, the part
everyone gets wrong", now lists all 28 transforms and the exact resolution and
partial-alpha boundary rules.

**Tests.** The 40 exact PowerPoint cases, all 28 XML mappings, transform order,
linear-gamma conversion, partial alpha, explicit empty pairs, unexpected
nested content, raw-child order, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Expected oracle values are evidence, not values
generated from the Rust formulas. Re-run the ignored generator only with the
pinned PowerPoint build and an explicit native shell.

### F-056, Colour map resolution

**Sprint.** S12
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** A concrete 12-slot `ColorMap` provides the standard Office
mapping, selective layout or slide overrides, and resolution in map, theme,
then transform order. Direct RGB, system, and preset choices bypass the map.

**Non-obvious choices.** Semantic and theme slots are validated enums. The
resolver takes a concrete lookup slice instead of a trait or generic, and a
missing system name uses its `lastClr` fallback.

**Deviations from the design plan.** Microscope pass 1 strengthened all 12
default mappings, all 11 untouched override slots, exact direct-colour results,
and the system fallback path. Pass 2 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
three-stage resolution contract in `docs/hld/05-drawingml-model.md` and leaves
`p:clrMap` parsing to F-069.

**Tests.** Standard mapping, selective override composition, the exact dark
master inversion gate with transforms, direct-colour bypass and system
fallback, plus the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** PresentationML parsers should construct this
format-neutral value. They must not move `p:` parsing into `oxml-drawing`.

### F-057, a:xfrm

**Sprint.** S13
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml-drawing` now models DrawingML transforms with shape
and child coordinate rectangles, rotation, flips, prefix-tolerant parsing,
fixed-prefix writing, ordered raw-child preservation, and finite affine
composition.

**Non-obvious choices.** Child coordinates map into the parent rectangle before
rotation and flips are composed. Zero child extents return a typed error rather
than producing non-finite matrix coefficients.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
transform contract in `docs/hld/05-drawingml-model.md`.

**Tests.** `nested_group_transform_composes_to_the_hand_computed_matrix`,
prefix and schema-order writing, raw-child preservation, zero-extent rejection,
and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep DrawingML coordinate conversion in this
format model. Renderer transforms remain backend-neutral values.

### F-058, Guide evaluator

**Sprint.** S13
**Completed.** 2026-07-31
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `oxml-drawing` now parses and evaluates all 17 DrawingML
guide formula tokens from an owned seeded environment, applies adjust-value
overrides in declaration order, evaluates local path commands, and lowers
clockwise arcs to finite cubic Beziers in segments no larger than 90 degrees.

**Non-obvious choices.** Office interoperability defines `mod` as a Euclidean
norm and applies `sqrt` to the absolute input. Multi-turn sweeps are valid and
bounded by a segment-count guard rather than rejected at one full turn.

**Deviations from the design plan.** The approved plan was corrected before
implementation to use the standard 17 formula tokens. Microscope pass 1 found
that valid multi-turn arcs were rejected. A 450-degree regression fixed the
defect, and pass 2 was clean.

**Spec sections touched.** `docs/hld/05-drawingml-model.md`, "Geometry", now
records the standard formula set, owned environment, Office deviations, and
arc-lowering semantics.

**Tests.** `hand_written_custom_geometry_guides_produce_expected_path_coordinates`,
all formula operations, Office deviations, invalid math, finite arc endpoints,
multi-turn sweeps, unit-angle conversion, truncation riders, and the integrated
full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep formula operands owned because custom
geometry supplies document data. Reject every non-finite intermediate before a
renderer sees it.

### F-059, a:custGeom

**Sprint.** S13
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml-drawing` now parses, writes, and evaluates
`a:custGeom` adjust lists, guide lists, text rectangles, path lists, and path
commands. Reads tolerate arbitrary prefixes, writes use the fixed `a:` prefix,
and unknown subtrees remain byte-for-byte at their schema boundaries.

**Non-obvious choices.** The approved schema-valid custom geometry fixture is
inline because the repository has no fetched deck corpus. The real corpus gate
remains at the M7 boundary.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
geometry and preservation contracts in `docs/hld/05-drawingml-model.md`.

**Tests.** `corpus_custom_geometry_round_trips_and_evaluates_to_a_closed_path`,
prefix and child-order writing, raw-child preservation, malformed-input
handling, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Replace or supplement the inline fixture when
the separately fetched M7 deck corpus is available. Do not commit a binary deck
only to duplicate the same XML boundary test.

### F-060, Fills

**Sprint.** S13
**Completed.** 2026-07-31
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `oxml-drawing` now models no fill, solid fill, linear and
path gradients, pattern fill, and stretched or tiled picture fill with source
rectangles. Gradient stops retain document order, reads tolerate arbitrary
prefixes, and writers emit fixed-prefix schema order.

**Non-obvious choices.** Picture fills retain relationship identifiers as
owned strings without introducing an OPC dependency. Modelled leaf elements
also retain ordered raw children so nested extensions survive round trips.

**Deviations from the design plan.** Microscope pass 1 found that a pattern
colour wrapper containing only an extension was dropped. Pass 2 found the same
class of loss in modelled leaf elements. Both were fixed with focused
regressions, and pass 3 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
fill module and preservation contracts in `docs/hld/05-drawingml-model.md`.

**Tests.** `every_fill_form_round_trips_and_gradient_stops_keep_document_order`,
prefix and schema-order writing, nested raw preservation, malformed-value
rejection, released Word theme isolation, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Resolve picture relationships and media bytes in
the PresentationML package layer. Do not add an OPC dependency to this crate.

### F-061, Lines

**Sprint.** S14
**Completed.** 2026-08-01
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml-drawing` now models `a:ln` width, fill, preset and
custom dashes, cap and join choices, and head and tail ends. Reads accept any
prefix, writes use fixed `a:` prefixes, and unsupported children retain their
schema boundaries.

**Non-obvious choices.** Every preset dash token maps to a concrete dash array
without a fallback. The wire model retains schema values while the mapping
provides the later renderer boundary.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
line and preservation contracts in `docs/hld/05-drawingml-model.md`.

**Tests.** `every_preset_line_dash_value_maps_to_a_dash_array`, complete line
round trips, schema-order writing, raw-child preservation, malformed-value
handling, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep dash interpretation in the renderer seam.
The XML layer should continue to preserve the declared line vocabulary.

### F-062, Effects

**Sprint.** S14
**Completed.** 2026-08-01
**Size.** S, estimated 1 day, actual 1 day

**What was built.** `oxml-drawing` now models effect lists and outer shadows,
including geometry and colour, while retaining unsupported effects such as
glow as raw XML at their exact positions.

**Non-obvious choices.** The effect list models only the values needed by the
current renderer contract. Unsupported effect subtrees remain authoritative
wire data rather than partial models.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
effect and preservation contracts in `docs/hld/05-drawingml-model.md`.

**Tests.** `a_shape_with_glow_round_trips_with_glow_intact_as_raw_xml`, outer
shadow round trips, schema-order output, malformed-value handling, and the
integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Add another typed effect only when a current
consumer needs it. Raw preservation already protects unsupported effects.

### F-063, Shape properties and style references

**Sprint.** S14
**Completed.** 2026-08-01
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml-drawing` now composes transforms, geometry, fills,
lines, and effects through `a:spPr`, and models line, fill, effect, and font
style references with optional colour choices.

**Non-obvious choices.** Style index zero is valid. Fill indices greater than
1000 select the background-fill list with an offset of 1000, so index 1001
resolves to background fill style 1.

**Deviations from the design plan.** Microscope pass 1 found that zero indices
and colourless style references were rejected. Both schema-valid cases were
added, and pass 2 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
shape composition and style-matrix rules in `docs/hld/05-drawingml-model.md`.

**Tests.** `fill_ref_1001_resolves_to_background_fill_style_1`, all four style
reference forms, zero and colourless references, shape schema order,
malformed-input handling, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** The format-neutral reference model reuses
`ColorChoice`. It does not alter the released Word theme path.

### F-064a, Text body properties and shell

**Sprint.** S14
**Completed.** 2026-08-01
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml-drawing` now owns a typed text-body shell and body
properties for insets, anchoring, wrapping, vertical direction, autofit, and
schema-ordered raw children.

**Non-obvious choices.** The first child kept later list styles and paragraphs
opaque until their dependent stories arrived. This preserved a usable shell
without anticipating their models.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
text-body and autofit contracts in `docs/hld/05-drawingml-model.md`.

**Tests.** `every_body_property_autofit_form_round_trips_in_schema_order`,
prefix handling, raw-child boundaries, malformed attributes, and the
integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Autofit remains a wire-model choice. Layout
policy belongs to the later renderer milestone.

### F-064b, Text paragraphs and runs

**Sprint.** S14
**Completed.** 2026-08-01
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `oxml-drawing` now models paragraphs, paragraph and run
properties, regular runs, fields, line breaks, text spans, fonts, hyperlinks,
spacing values, and ordered unsupported XML.

**Non-obvious choices.** Significant leading or trailing text is emitted with
`xml:space="preserve"`. Qualified relationship and whitespace attributes are
matched exactly so hostile prefixes cannot masquerade as `r:id` or
`xml:space`.

**Deviations from the design plan.** Microscope pass 1 found that local-name
fallback could interpret hostile qualified attributes as modelled attributes.
Exact matching and regressions fixed the issue, and pass 2 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
text and preservation contracts in `docs/hld/05-drawingml-model.md`.

**Tests.** `leading_and_trailing_text_whitespace_survives_via_xml_space_preserve`,
paragraph content order, property units, hyperlink attributes, hostile
prefixes, malformed input, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Relationship resolution remains outside this
crate. The parser stores relationship identifiers without adding an OPC edge.

### F-064c, Text bullets

**Sprint.** S14
**Completed.** 2026-08-01
**Size.** S, estimated 1 day, actual 1 day

**What was built.** Paragraph properties now carry character, automatic, and
explicit no-bullet choices with optional font, point or percentage size, and
colour components in DrawingML schema order.

**Non-obvious choices.** All 41 automatic-numbering tokens are explicit. The
wire model retains literal bullet characters, while Wingdings conversion stays
with the later renderer work.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
bullet and preservation contracts in `docs/hld/05-drawingml-model.md`.

**Tests.** `every_modelled_bullet_form_round_trips_in_schema_order`, every
numbering token, optional component order, raw preservation, malformed values,
and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep font-symbol conversion out of structural
round trips so original bullet codepoints remain recoverable.

### F-064d, Nine-level list styles

**Sprint.** S14
**Completed.** 2026-08-01
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `a:lstStyle` now has nine explicit optional paragraph
property slots. The text-body model composes typed body properties, list
styles, and paragraphs into one structural round trip.

**Non-obvious choices.** Fixed slots make invalid list-level numbers
unrepresentable. Unsupported list-style siblings remain raw at their captured
boundaries, and modelled levels write in ascending schema order.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
nine-level text style chain in `docs/hld/07-inheritance-and-resolution.md`.

**Tests.** `schema_valid_text_body_using_all_nine_list_levels_round_trips_structurally`,
ascending level order, raw sibling preservation, invalid levels, and the
integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** The inline schema-valid fixture covers this
sprint. The fetched deck corpus remains required at the M7 boundary.

### F-064, DrawingML text model

**Sprint.** S14
**Completed.** 2026-08-01
**Size.** XL, estimated 0 days after split, actual 1 day

**What was built.** The umbrella closed after F-064a through F-064d delivered
the complete staged DrawingML text hierarchy for body properties, list styles,
paragraphs, runs, fields, breaks, whitespace, and bullets.

**Non-obvious choices.** The XL story carries no duplicate implementation.
Its four child stories own the natural schema boundaries and their individual
evidence, while this entry records the integrated contract.

**Deviations from the design plan.** The approved split used inline
schema-valid XML for S14. The fetched external deck corpus remains the M7
boundary gate, as planned.

**Spec sections touched.** `docs/hld/14-development-backlog.md` already records
the four-child split and the parent closure rule. No further prose change was
required.

**Tests.** The complete nine-level text-body structural round trip,
`leading_and_trailing_text_whitespace_survives_via_xml_space_preserve`, every
child test gate, and the integrated full workspace gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Run the separately fetched PowerPoint deck
corpus before closing M7. Do not publish the PowerPoint development crates
before PowerPoint development is complete.
