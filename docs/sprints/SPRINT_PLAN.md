# Sprint Plan

Sprint-by-sprint roadmap for the shared OOXML infrastructure, the Word and
PowerPoint families, and the conditional advanced spreadsheet programme.
Sprints are dependency and review boundaries, not fixed two-week containers.
Sprint clocks start at the first `/start-feature` of that sprint, not at a fixed
calendar date.

Each F-ID uses focused tests, applicable oracle cases, `/verify --scoped` and
a zero-finding `/microscope`. Full verification and sprint review cover the
final integrated result at closure. A formal dependency checkpoint completes
a prerequisite from scoped evidence and focused integration checks.

The active roadmap runs through S104, with earlier deferred cutover boundaries
retained in place. The sizing rationale and compression options are in
`docs/hld/14-development-backlog.md`.

M1 to M5 stage the extraction without changing released rdocx dependencies.
M7 to M12 build rpptx. Deferred M6 publication and rdocx cutover follow M12,
then M13 ships the bindings for both.

## Capacity calibration after S03

S01 through S03 completed 15 stories representing 24 estimated developer-days
in 15 recorded actual days. That is 5.00 stories per working week and 1.6
estimated days per actual day. The story-count rate is not used alone because
the remaining plan contains substantially more L and XL work than the first
three sprints.

The remaining 366 estimated developer-days therefore forecast to about 46
active working weeks at the observed weighted rate. Use 45 to 50 weeks as the
current planning range. Keep the existing dependency-defined sprint and
milestone boundaries, and size each implementation wave by dependencies,
exclusive resources, and estimated days rather than filling a fixed calendar
box. Recalculate after S06, when the evidence includes the first post-baseline
L story and the higher-risk layout extraction.

## Goals per sprint

### M1, Preparation and safety net

#### Sprint S01, The safety net

**Goal**: rendering is reproducible across machines, a byte-level baseline
exists for every sample, and the three shipped defects found during the audit
are fixed. Nothing has moved yet.

| F-ID | Title | Size |
|------|-------|------|
| F-001 | Deterministic font mode                      | M |
| F-002 | rust-toolchain.toml                          | S |
| F-003 | Output-stability hash harness                | L |
| F-004 | Caladea licence and the false OFL claim      | S |
| F-005 | Fix the image counter                        | S |
| F-006 | Fix the JPEG standalone-marker walk          | S |

F-001 gates F-003: a baseline recorded against system fonts would not reproduce
on another machine, which would make the harness worthless.

#### Sprint S02, Prerequisites and the pre-churn tag

**Goal**: everything the later milestones depend on is in place, and a
known-good published state is tagged immediately before the extraction begins.

| F-ID | Title | Size |
|------|-------|------|
| F-007 | Resolve core properties through the rel      | S |
| F-008 | Non-consuming setter twins                   | M |
| F-009 | Cache the layout result                      | M |
| F-010 | Reserve crate names                          | S |
| F-011 | Pin unit truncation behaviour                | S |
| F-012 | Tag v0.4.1                                   | S |

F-008 is required by M13 and improves the Rust API independently. F-011 must
land before anyone is tempted to change truncation to rounding.

### M2, Shared infrastructure extraction

#### Sprint S03, oxml-core

**Goal**: the generic types leave `rdocx-oxml` and 323 call sites do not change.

| F-ID | Title | Size |
|------|-------|------|
| F-013 | Create oxml-core                             | M |
| F-014 | New unit types                               | M |
| F-017 | App and custom properties                    | M |

#### Sprint S04, oxml-opc

**Goal**: the package layer is format-neutral and proven against a real pptx.

| F-ID | Title | Size |
|------|-------|------|
| F-018 | Create oxml-opc                              | M |
| F-019 | PresentationML relationship and content types| S |
| F-020 | oxml-opc reads a pptx                        | M |
| F-021 | Zip-slip hardening tests                     | S |

F-015 and F-016 carried from S03, and F-022 joined their deferred cutover. They
are rescheduled to S32.2 after PowerPoint development and shared-crate
publication readiness. Passing the rdocx 0.5.0 release boundary protects that
release but does not make an unpublished implementation available to later
package dry-runs. F-020 converts the plan's central package assumption into a
test.

### M3, Media

#### Sprint S05, oxml-media

**Goal**: stage and prove one crate that owns image sniffing, dimensions, and
naming without changing released rdocx dependencies.

| F-ID | Title | Size |
|------|-------|------|
| F-023 | oxml-media format sniffing                   | M |
| F-024 | Image probing and DPI                        | L |
| F-025 | MediaNamer                                   | S |
| F-026 | native_size with explicit DPI                | S |

F-027 and F-028 move to S32.2. F-027 retains a focused package regression for
the intentional change from trusted extensions to sniffed content types. The
existing hash harness remains unchanged because it does not collect package
content types or media relationship targets.

### M4, Layout primitives

#### Sprint S06, oxml-layout and the line.rs decoupling

**Goal**: the format-neutral layout types are staged in isolation, including
the one file that needs genuine API design.

| F-ID | Title | Size |
|------|-------|------|
| F-029 | Create oxml-layout                           | M |
| F-030 | Decouple line.rs                             | L |
| F-031 | Transform                                    | M |

F-030 is the highest drift risk in the extraction. Own PR, own review, gated
hard on the hash harness.

#### Sprint S07, The PositionedElement extension

**Goal**: the staged shared element type can express a rotated, clipped,
gradient-filled shape without changing released rdocx construction sites.

| F-ID | Title | Size |
|------|-------|------|
| F-032 | Path and PathCommand                         | M |
| F-033 | Paint and Stroke                             | M |
| F-034 | Path and Group arms                          | M |
| F-035 | The walk helper                              | S |
| F-036 | MediaId                                      | S |

Two new arms, not ten new fields. F-035 exists specifically to prevent the
recursion hazard that S09 then tests for.

### M5, PDF backend

#### Sprint S08, The coordinate system

**Goal**: the renderer moves to one global CTM with zero pixel change.

| F-ID | Title | Size |
|------|-------|------|
| F-037 | Create oxml-pdf                              | S |
| F-038 | Golden-PNG harness                           | M |
| F-039 | Global CTM flip                              | L |

F-039 is the single highest-risk change in the plan. It lands before
PresentationML rendering code exists, so a regression has only one possible
cause.

#### Sprint S09, Groups, paths and the recursion fix

**Goal**: nested content renders, and the three collection passes see inside
groups.

| F-ID | Title | Size |
|------|-------|------|
| F-040 | Group rendering                              | M |
| F-041 | Path rendering                               | M |
| F-042 | Rewrite the three collection passes on walk  | M |
| F-044 | ExtGState alpha                              | S |

F-042 is the R3 regression gate. Its three tests are the only thing standing
between this design and PDFs that silently lose fonts, images or links.

#### Sprint S10, Gradients and the rasteriser

**Goal**: both backends render everything the element types can express.

| F-ID | Title | Size |
|------|-------|------|
| F-043 | Gradient shading dictionaries                | L |
| F-045 | Rasteriser: groups, paths, gradients, dashes | L |

F-045 also fixes the dash pattern that all PNG output currently discards.

### M6, deferred shared publication and rdocx cutover

#### Sprint S11, Staged extraction gate

**Goal**: verify the isolated shared crates and continue PowerPoint development
without publishing them or changing released rdocx dependencies.

No publication or consumer-cutover story runs at S11. F-046 through F-051 are
rescheduled to S32.1 and S32.2 after PowerPoint development. S11 is the staged
extraction validation boundary before DrawingML construction begins.

### M7, DrawingML

#### Sprint S12, Colour

**Goal**: a theme colour with a transform stack resolves to the exact RGB
PowerPoint produces.

| F-ID | Title | Size |
|------|-------|------|
| F-052 | Create oxml-drawing and namespace constants  | S |
| F-053 | OrderedRawChildren                           | M |
| F-054 | Colour choices                               | M |
| F-055 | The colour transform stack                   | L |
| F-056 | Colour map resolution                        | M |

F-055's test gate is a table of 40 pairs sampled from real renders. Getting
`lumMod` wrong makes an entire deck the wrong shade.

#### Sprint S13, Geometry and fills

**Goal**: any shape's outline and fill can be described.

| F-ID | Title | Size |
|------|-------|------|
| F-057 | a:xfrm                                       | M |
| F-058 | Guide evaluator                              | L |
| F-059 | a:custGeom                                   | M |
| F-060 | Fills                                        | L |

F-058 is what makes the preset table a data problem in M10 rather than a code
problem.

#### Sprint S14, Lines, effects and text

**Goal**: the DrawingML text vocabulary is modelled.

| F-ID | Title | Size |
|------|-------|------|
| F-061 | Lines                                        | M |
| F-062 | Effects                                      | S |
| F-063 | Shape properties and style references        | M |
| F-064 | DrawingML text model                         | XL |
| F-064a | Text body properties and shell               | M |
| F-064b | Text paragraphs and runs                     | L |
| F-064c | Text bullets                                 | S |
| F-064d | Nine-level list styles                       | M |

F-064 is the umbrella gate. Its implementation is split into F-064a through
F-064d, and the parent closes only after every child closes.

#### Sprint S15, Theme

**Goal**: themes read and write, and rdocx adopts the shared type without
changing behaviour.

| F-ID | Title | Size |
|------|-------|------|
| F-065 | Theme read and write                         | L |
| F-066 | The rdocx Theme adapter                      | S |

F-066's test gate is the hash harness being unchanged. The Word tint and shade
path is deliberately left alone.

### M8, PresentationML

#### Sprint S16, Parts and the shape tree

**Goal**: the corpus round-trips with everything opaque, then with the core
parts modelled.

| F-ID | Title | Size |
|------|-------|------|
| F-067 | Create rpptx-oxml and the corpus harness     | M |
| F-068 | presentation.xml                             | M |
| F-069 | Slide, layout and master parts               | L |
| F-070 | The shape tree                               | L |

F-067's raw round-trip proves the OPC layer and the corpus harness before any
XML modelling exists.

#### Sprint S17, Placeholders, pictures and tables

| F-ID | Title | Size |
|------|-------|------|
| F-071 | Placeholders                                 | M |
| F-072 | Pictures                                     | M |
| F-073 | Graphic frames                               | M |
| F-074 | DrawingML tables                             | L |

#### Sprint S18, The long tail

| F-ID | Title | Size |
|------|-------|------|
| F-075 | Connectors                                   | S |
| F-076 | mc:AlternateContent                          | M |
| F-077 | Notes slides and notes master                | M |
| F-078 | relmap rewrite_rel_ids                       | M |

F-078 is what makes deep copy safe in M11. Without it a duplicated slide's
SmartArt points at the source slide's relationships.

#### Sprint S19, The read facade

**Goal**: open any deck and read it.

| F-ID | Title | Size |
|------|-------|------|
| F-079 | The rpptx read facade                        | L |
| F-080 | Modelled round-trip gate                     | M |

**This is the M8 gate**: all 50 decks round-trip and every one opens in
PowerPoint without a repair prompt.

### M9, Inheritance resolver

#### Sprint S20, The chains

**Goal**: every inherited property resolves.

| F-ID | Title | Size |
|------|-------|------|
| F-081 | ResolveCtx skeleton and placeholder chain    | M |
| F-082 | Effective transform and body properties      | M |
| F-083 | The seven-step list style merge              | L |
| F-084 | Format scheme reference resolution           | M |
| F-085 | Typeface resolution                          | S |

#### Sprint S21, Draw order and the contract

**Goal**: `ResolvedSlide` is frozen and correct.

| F-ID | Title | Size |
|------|-------|------|
| F-086 | Draw order and the flattener                 | L |
| F-087 | ResolvedSlide contract                       | M |
| F-088 | Visual differential tests                    | M |

F-086's test gate is that a rendered slide contains no "Click to edit Master
title style". Placeholders on layouts and masters are templates, never drawn.

### M10, Renderer

#### Sprint S22, Geometry

| F-ID | Title | Size |
|------|-------|------|
| F-089 | Resolve the preset geometry licensing question | S |
| F-090 | Preset table generator                       | L |
| F-091 | Preset evaluation and fallback               | M |
| F-092 | rpptx-render skeleton and RenderInput        | M |

F-089 is a decision, not code, and it blocks F-090. LibreOffice's table is
MPL-2.0 and cannot be used.

#### Sprint S23, Shapes

| F-ID | Title | Size |
|------|-------|------|
| F-093 | Shape geometry, fills and lines              | L |
| F-094 | Rotation, flips and groups                   | M |
| F-095 | Arrowheads                                   | S |
| F-096 | Pictures with crop and tile                  | M |
| F-097 | Backgrounds                                  | S |

Ships slides with shapes but no text.

#### Sprint S24, Text

| F-ID | Title | Size |
|------|-------|------|
| F-098 | Shape text layout                            | XL |
| F-098a | Text content box                             | M |
| F-098b | Paragraph inline resolution                  | L |
| F-098c | Line stacking                                | M |
| F-098d | Text anchoring                               | S |
| F-099 | Bullets                                      | M |
| F-100 | Autofit                                      | M |
| F-101 | Vertical text                                | S |

**The milestone that makes the project real.** F-098 is the umbrella gate for
F-098a through F-098d, which implement the content box, paragraph inline
resolution, line stacking, and anchoring. The parent closes only after every
child closes.

#### Sprint S25, Tables and the fidelity gate

| F-ID | Title | Size |
|------|-------|------|
| F-102 | Table rendering                              | L |
| F-103 | Hyperlinks, fields and diagnostics           | M |
| F-104 | SSIM fidelity harness                        | L |

**This is the M10 gate** and the natural point to cut an early
read-plus-render release if the schedule needs compressing.

### M11, Write API

#### Sprint S26, Slides

| F-ID | Title | Size |
|------|-------|------|
| F-105 | Bundled default.pptx                         | M |
| F-106 | ShapeIdAllocator and MediaStore              | M |
| F-107 | add_slide                                    | L |
| F-108 | validate()                                   | M |

F-108 will save more debugging time than any other story in the backlog.

#### Sprint S27, Shapes and text

| F-ID | Title | Size |
|------|-------|------|
| F-109 | Shape mutation facade                        | L |
| F-110 | add_textbox, add_shape, add_connector, group | M |
| F-111 | add_picture                                  | M |
| F-112 | Text frame mutation                          | L |

#### Sprint S28, Tables and acceptance

| F-ID | Title | Size |
|------|-------|------|
| F-113 | Table facade                                 | L |
| F-114 | remove_slide, move_slide, duplicate_slide    | M |
| F-115 | Slide and presentation properties            | S |
| F-116 | Cross-viewer acceptance                      | M |

**This is the M11 gate**: a generated deck opens clean in PowerPoint, Keynote,
Google Slides and LibreOffice.

### M12, Charts

#### Sprint S29, The data layer

| F-ID | Title | Size |
|------|-------|------|
| F-117 | oxml-sml workbook writer                     | L |
| F-118 | ChartML core types                           | L |
| F-119 | Series and data references                   | L |

F-119's caches are what actually render. A chart written without them is empty
in most viewers.

#### Sprint S30, Axes and plots

| F-ID | Title | Size |
|------|-------|------|
| F-120 | Axes                                         | L |
| F-121 | Bar and line plots                           | M |
| F-122 | Pie, doughnut, area, scatter and radar plots | L |
| F-123 | Data labels and number formats               | M |

#### Sprint S31, Authoring and rendering

| F-ID | Title | Size |
|------|-------|------|
| F-124 | add_chart                                    | L |
| F-125 | Chart rendering: geometry                    | L |
| F-126 | Chart rendering: axes, gridlines and labels  | L |

#### Sprint S32, Chart polish

| F-ID | Title | Size |
|------|-------|------|
| F-127 | Chart colour resolution                      | M |
| F-128 | Preserved chart fallback                     | S |

### Deferred shared publication and rdocx cutover

#### Sprint S32.1, Shared publication readiness

**Goal**: make the completed shared crates packageable, fully gated, and ready
for an explicitly approved publication without publishing from this sprint.

| F-ID | Title | Size |
|------|-------|------|
| F-047 | Packaging include and size gate              | M |
| F-048 | Automate split-family release preparation    | M |
| F-049 | Extend publish.yml to the extracted workspace| M |
| F-050 | CI matrix additions                          | S |

After S32.1, publication runs only through a separate reviewed release plan
with explicit approval. S32.2 cannot start until the registry contains the
approved shared-crate versions and a clean consumer resolves those versions.

#### Sprint S32.2, Released rdocx cutover

**Goal**: after the real shared crates are published through their approved
release plan, move released rdocx consumers onto them and document the cutover.

| F-ID | Title | Size |
|------|-------|------|
| F-X005 | Tag rpptx-v0.1.2                           | S |
| F-015 | rdocx-oxml becomes a facade                  | S |
| F-016 | Length re-export                             | S |
| F-022 | rdocx-opc deprecation shim                   | S |
| F-027 | rdocx adopts oxml-media                      | M |
| F-028 | add_picture_auto                             | S |
| F-046 | rdocx layout and PDF cutover                 | M |
| F-051 | CHANGELOG and migration notes                | S |

**This is the deferred M6 release gate.** The shared crates are real published
dependencies, released rdocx packages pass archive verification, and the hash
harness remains unchanged while a focused regression proves the F-027
content-type change.

### M13, Bindings and tooling

#### Sprint S33, rdocx-py

**Goal**: the handle design is validated against the settled API before it is
reused.

| F-ID | Title | Size |
|------|-------|------|
| F-129 | oxml-py-support                              | M |
| F-130 | rdocx-py core                                | L |
| F-131 | rdocx-py formatting and tables               | L |
| F-132 | Python enums, units and exceptions           | M |
| F-133 | rdocx-py rendering with allow_threads        | S |

#### Sprint S34, Wheels and rpptx-py

| F-ID | Title | Size |
|------|-------|------|
| F-134 | Type stubs and py.typed                      | M |
| F-135 | python-docx parity suite                     | M |
| F-136 | rpptx-py                                     | L |
| F-137 | wheels.yml                                   | M |
| F-138 | PR-time Python job                           | S |

#### Sprint S35, WASM

**Goal**: the wasm crates wrap the real facades and are watched by CI.

| F-ID | Title | Size |
|------|-------|------|
| F-139 | Rewrite rdocx-wasm                           | L |
| F-140 | wasm CI job                                  | S |
| F-141 | to_pdf in the browser                        | M |
| F-142 | rpptx-wasm                                   | M |

F-139 fixes a shipped defect that silently discards every package part except
two. F-140 is why it will not happen again.

#### Sprint S36, CLIs and local packaging

| F-ID | Title | Size |
|------|-------|------|
| F-143 | oxml-cli-support                             | S |
| F-144 | rpptx-cli                                    | L |
| F-145 | rpptx-cli thumbnail and outline              | M |
| F-146 | npm publication                              | S |
| F-X001 | rdocx-cli tests                             | M |
| F-X002 | README example correctness                  | S |
| F-X003 | Deduplicate the sample generators           | S |
| F-X004 | Fix the shared temp path in the test suite  | S |

**This is the v1 implementation gate.** The CLI and local package surfaces are
complete. Registry publication is explicitly deferred. The expanded
incubating Rust family requires a fresh version and reviewed release through
F-X006. npm registry publication requires a separate future story.

#### Sprint S37, Expanded Rust family release

**Goal**: prepare a fresh common incubating version and publish the complete
14-package Rust family through the reviewed release workflow after separate
final approval.

| F-ID | Title | Size |
|------|-------|------|
| F-X006 | Tag the expanded rpptx family               | S |

#### Sprint S38, Contributor integration and stable release

**Goal**: land PR 25 with its contributor credit intact, harden the new Word
composition APIs against package and table-geometry regressions, and make the
stable crate family documentation useful at the point of publication. Prepare
and publish the complete stable family at the breaking pre-1.0 0.5.0 boundary
only after the integrated result passes review and receives separate release
approval.

| F-ID | Title | Size |
|------|-------|------|
| F-X007 | Integrate PR 25 and stable crate documentation | L |
| F-X008 | Tag v0.5.0                                      | S |

F-X008 depends on F-X007. The GitHub PR is merged into the sprint branch so
the contributor remains credited in the pull request and merge record. Only
`/close-sprint` later merges the reviewed sprint to `main`.

#### Sprint S39, Workspace crate documentation and next-minor releases

**Goal**: every one of the 26 Cargo workspace packages declares a useful
README. Each README explains the crate's role, intended audience, relationship
to neighbouring packages, and includes a concrete example in the language or
command surface that users actually consume. Publish those READMEs for every
crates.io-eligible package through separate next-minor stable and incubating
release tags.

| F-ID | Title | Size |
|------|-------|------|
| F-X009 | README coverage for every workspace crate | L |
| F-X010 | Tag v0.6.0 | S |
| F-X011 | Tag rpptx-v0.2.0 | S |

The existing README runner becomes an exact 26-package contract. Published
archives must each contain one intended README, while unpublished Python and
WASM packages receive accurate local usage examples without gaining any new
publication authority. F-X010 publishes the seven stable crates first. F-X011
then publishes the fourteen incubating crates. Each tag has its own full
verification, clean review, and immediate release approval boundary.

#### Sprint S40, Restore pinned CI toolchains

**Goal**: restore a green hosted CI baseline after runner and package-manager
updates exposed unpinned or incorrectly validated external tools. Keep the
reviewed Poppler 26.01.0 rendering oracle and Binaryen 125 optimizer boundary
without changing product output or recorded rendering baselines.

| F-ID | Title | Size |
|------|-------|------|
| F-X012 | Restore pinned CI toolchains | M |

The story installs checksum-pinned Poppler 26.01.0 for every job that executes
its oracle-dependent tests, validates the official Binaryen 125 Linux version
string, and proves the complete pull-request workflow on a hosted runner. It
does not change a crate, release version, published package, or rendering
baseline.

#### Sprint S41, Footnote placement and floating drawing wrapping

**Goal**: land the parts of the external PR 2 contribution that current `main`
still lacks, rebuilt on the anchor architecture that superseded the
contributor's own. Fix the two footnote placement defects, then give anchored
drawings a real wrap model and make body text flow around them.

| F-ID | Title | Size |
|------|-------|------|
| F-X013 | Footnote and endnote placement | M |
| F-X013a | Footnote line advance | S |
| F-X013b | Footnote reservation and splitting | L |
| F-X013c | Endnotes at the document end | M |
| F-X014 | Kashida justification values | S |
| F-X015 | Anchored drawing wrap and alignment model | M |
| F-X016 | Floating drawing placement and text wrapping | L |

The note work lands first, because it is independent of the drawing work and
each child carries its own baseline delta. F-X013 was planned as a single M and
split into three children during its design, when splitting oversized notes and
correcting endnote placement were taken into scope. F-X014 is a one-line parser
widening carried in the same wave because it comes from the same contribution.
F-X015 adds the wrap and alignment surface without changing placement, which
keeps the harness flat and makes F-X016 the single story that owns the rendering
delta for wrapped drawings.

#### Sprint S42, Dependency refresh

**Goal**: take the outstanding semver-compatible dependency updates and measure
what they do to rendered output. Nothing here is a security fix, since the
advisory scan is already clean, so the value is in not letting the lockfile
drift far enough that a later update becomes a large unexplained delta.

| F-ID | Title | Size |
|------|-------|------|
| F-X020 | Refresh the dependency lockfile | S |
| F-X024 | Move the theme adapter into rdocx-oxml | M |
| F-X022 | Tag rpptx-v0.3.0 | S |
| F-X023 | Tag v0.7.0 | S |

Two of the sixteen pending updates are in the font and image decoding path, so
the hash harness decides whether the refresh is a no-op or a declared rendering
delta. It runs first and alone, because a refresh that moved a baseline should
not compete with a release to explain the same delta.

F-X024 then removes the reason the release order was impossible. Scoping the two
release stories exposed a cycle between the trains: `rdocx-layout` depends on
`oxml-layout`, and `oxml-drawing` depends on `rdocx-oxml` through the one
documented architecture exception. With both trains carrying breaking changes,
neither could publish first. Moving the theme adapter into `rdocx-oxml` inverts
that edge, so the dependency runs one way and incubating always publishes
first.

The two release stories then carry S41's work to crates.io in that order.

#### Sprint S43, Robustness and gate coverage

**Goal**: clear the follow-ups S41 and S42 filed. Three are defects that a real
document can reach, and two close gaps in the gates that let the other three
survive as long as they did.

| F-ID | Title | Size |
|------|-------|------|
| F-X018 | Unknown enumerated values must not fail a document open | M |
| F-X017 | Notes broken to their own section's width | S |
| F-X019 | Paragraph-relative later drawings should wrap | M |
| F-X021 | The hash harness should cover PDF output | M |
| F-X025 | /verify must run the release regressions | S |

F-X018 leads because it is the only one where a document fails to open rather
than rendering imperfectly. F-X014 fixed the three kashida values because a real
contribution reached them, and eight more parsers have the same shape.

F-X017 and F-X019 are the two limitations S41 recorded rather than hid, both
narrow and both reachable by a real document. F-X021 and F-X025 are the gates:
one gives the harness PDF coverage it has never had, the other makes `/verify`
run the release regressions that `publish.yml` treats as its publication gate.
The gate stories land last because neither blocks the three defect fixes, and
putting them first would delay work that users can actually hit.

#### Sprint S44, Gate coverage and specification repair

**Goal**: finish the job S43 started. S43 closed two gaps in the gates and
found, in passing, that the records describing those gates had drifted from
them. This sprint puts the two remaining gates where CI can see them and repairs
the documentation that tells every future session what is true.

| F-ID | Title | Size |
|------|-------|------|
| F-X026 | CI must run the release regressions too | S |
| F-X027 | Wire the golden-PNG gate into something | S |
| F-X029 | Path-filtered CI jobs | M |
| F-X028 | Repair the agent-facing documentation drift | M |

Every implementation milestone is closed, so this sprint carries no feature
work. Three of the four exist because S43 went looking at the instruments rather
than the product. F-X029 came out of a review of whether the workspace should be
split into separate repositories. That review also produced F-X030, archived
before the sprint started because the WASM packages are deliberately
unpublished and its stated problem does not exist.

No pair has a hard dependency, so the order is a preference. The one soft
coupling is F-X026 and F-X029, which both edit `ci.yml`. F-X026 first, because it is the narrower
half of a gap S43 half-closed: `/verify` runs the release preflights now and CI
still does not, so a contributor who skips the local gate can move a version
carrier and see a green pull request. F-X027 next, because the golden-PNG gate
is fully specified and wired into nothing, and deciding where it belongs needs a
judgement about the pinned Poppler build that F-X026 does not need.

F-X028 lands last and is the largest, because it is the only one that touches
`CLAUDE.md`, and a story that rewrites the file every other session reads first
should land against a tree the other two have already settled.

## Post-v1 roadmap, S45 onward

v1 shipped at S43 and every implementation milestone closed. S44 repairs the
gates and the records. From S45 the plan resumes at milestone granularity
against `14-development-backlog.md` M14 through M24.

The order is deliberate and each boundary is a stopping point. Stopping after
S45 leaves one chart engine serving both families. Stopping after S51 leaves a
document-automation product. S69 closes the first Word-depth programme. S73
closes from-scratch generation for the five private business documents, and S94
closes the broader modern DOCX authoring boundary. The advanced spreadsheet
programme starts only after S94 and only if its feasibility gate confirms a gap
worth filling in the Rust ecosystem.

| Sprints | Milestone | Stories | Days |
|---|---|---|---|
| S45 | M15, charts beyond PowerPoint | 4 | 12 |
| S46 to S48 | M14, Word collaboration layer | 9 | 28 |
| S49 to S51 | M16, document automation | 15 | 39 |
| S52 to S53 | M17, security and compliance | 7 | 23 |
| S54 to S56 | M18, format breadth | 8 | 26 |
| S57 to S58 | M20, fidelity at scale | 7 | 27 |
| S59 to S64 | M21, presentation depth | 15 | 60 |
| S65 to S69 | M22, Word depth | 12 | 44 |
| S70 to S73 | M23, from-scratch business documents | 24 | 112 |
| S74 to S75, S86, S89 to S94 | M24, modern DOCX authoring completeness | 47 | 219 |
| S95 to S104 | M19, advanced spreadsheets | 21 | 85 |

The table counts milestone stories only. S70 also carries four cross-cutting
Issue 67 and Issue 69 stories estimated at 12 developer-days. S76 through
S85 hold the separate contribution, issue repair and hosted CI programme.

### M15, Charts beyond PowerPoint

#### Sprint S45, One chart engine

**Goal**: make the chart engine serve Word as well as PowerPoint. It is the
cheapest milestone on the roadmap and the only one whose engine already exists
on the format-neutral side of the crate graph.

| F-ID | Title | Size |
|------|-------|------|
| F-156 | Extract oxml-chart | L |
| F-157 | Word chart part and embedded workbook | M |
| F-158 | Document::add_chart | M |
| F-159 | Chart rendering in the Word paginator | M |

F-156 is a file move and nothing else. The hash harness must be byte-identical
across it, and folding a behaviour change into it is forbidden. The other three
are strictly ordered, since each needs the part the one before it writes.

### M14, Word collaboration layer

#### Sprint S46, Comments and content controls

**Goal**: open the collaboration layer at both ends. Comments are the most
requested missing API in this space and content controls are the primitive every
document-assembly product is built on.

| F-ID | Title | Size |
|------|-------|------|
| F-147 | Comment model and part | M |
| F-148 | Comment API | M |
| F-152 | Content control model | L |
| F-153 | Content control binding | M |
| F-154 | Bookmarks and cross-references | M |

Two independent pairs, so they parallelise cleanly. F-154 joins this sprint
rather than the next because F-161 needs bookmarks to resolve `REF` and
`PAGEREF`, and that is two sprints away.

#### Sprint S47, Tracked changes

**Goal**: read, write and resolve revisions. The single most demanded enterprise
capability in this space and the one with no open-source answer in any language.

| F-ID | Title | Size |
|------|-------|------|
| F-149 | Revision model | L |
| F-150 | Accept and reject revisions | L |

Two stories, both L, and deliberately alone in a sprint. F-150 has to reproduce
what Word produces from the same input, which is the kind of correctness that
takes the time it takes.

#### Sprint S48, Revision display and protection

**Goal**: close M14. Show revisions, and read the author's intent about who may
change what.

| F-ID | Title | Size |
|------|-------|------|
| F-151 | Revision display in the renderer | M |
| F-155 | Document protection | M |

A short sprint on purpose. It carries the M14 end-of-milestone gate, which
covers four subsystems built across three sprints.

### M16, Document automation

#### Sprint S49, Fields

**Goal**: evaluate the field codes real documents are full of. Everything in
M16 rests on this.

| F-ID | Title | Size |
|------|-------|------|
| F-160 | Field instruction parser | L |
| F-161 | Field evaluation engine | L |
| F-162 | Field update policy | M |
| F-203 | Reader compatibility corrections | M |

F-160 through F-162 are strictly ordered. F-203 is an independent corrective
story for reader preservation. F-161 also depends on F-154 from S46, which is
why bookmarks landed early.

#### Sprint S50, Templating

**Goal**: turn substitution into generation.

| F-ID | Title | Size |
|------|-------|------|
| F-163 | Template syntax | L |
| F-164 | Loops and conditionals | L |
| F-165 | Repeating table rows and lists | M |

F-163 leads because the tag-split-across-runs problem is the one every naive
implementation gets wrong, and the two after it inherit whatever it decides.

#### Sprint S51, Automation milestone and community release

**Goal**: close M16, add the requested native reader and editor surfaces,
establish a custom reviewed release-notes ceremony, and publish the coherent
incubating and stable trains.

| F-ID | Title | Size |
|------|-------|------|
| F-166 | Mail merge | M |
| F-167 | Document comparison | L |
| F-168 | Watermarks | S |
| F-X032 | Expose complete Word layout results | S |
| F-X033 | Integrate PR 36 ordered body items | S |
| F-X034 | Reviewed release notes for every release | S |
| F-X035 | Tag rpptx-v0.4.0 | S |
| F-X036 | Tag v0.8.0 | S |
| F-X037 | Trace Word glyphs to source paragraphs | M |
| F-X038 | Cache relayout work across document edits | L |

F-167 is the flagship of every commercial library in this category. It is scoped
to body text, tables and list structure, with formatting-only differences
recorded as a diagnostic, which is what keeps it one story. The community API
surface combines complete layouts, source provenance, bounded relayout caches,
and direct ordered body items. The incubating release precedes stable 0.8.0
because `oxml-chart`, the low-level provenance types, and shared font-cache
work must exist on crates.io before the stable dependency graph can publish.
Both release tags retain separate final approval boundaries.

### M17, Security and compliance

#### Sprint S52, Encryption and renderer follow-ups

**Goal**: open the files that currently cannot be opened at all, then close the
community-reported rendering and interactive-layout gaps with exact public
regressions.

| F-ID | Title | Size |
|------|-------|------|
| F-169 | Agile encryption, read | L |
| F-170 | Agile encryption, write | M |
| F-171 | Digital signature verification | L |
| F-X039 | Share layout payloads and transfer reusable engines | M |
| F-X040 | Restart pagination and cache table blocks | L |
| F-X041 | Remove duplicated glyphs at break opportunities | M |
| F-X042 | Prove headers and footers in PDF output | S |
| F-X043 | Reuse bundled-fallback caller-font layouts | M |
| F-X044 | Scale paragraph-cache lookup for editors | M |
| F-X045 | Cache headers and footers transactionally | M |
| F-X046 | Reuse substituted pages exactly | S |
| F-X047 | Attribute empty Word paragraphs | S |

Reading comes first and matters most. A password-protected document is a hard
stop for a user today, where an unsigned one is only a missing assurance.
F-X039 establishes shared ownership and a checked editor handoff before F-X040
retains pagination tails. F-X041 fixes Issue 23 at the common layout layer, and
F-X042 closes Issue 15 with a public DOCX-to-PDF gate. F-X043 through F-X047
close PRs 40 and 41 after retaining their useful editor behavior behind exact
context identity, transactional publication, and bounded memory. The sprint
does not import unchecked engine setters, hash-authoritative reuse, or
unbounded caches from the draft branches.

#### Sprint S53, Security milestone and community release

**Goal**: close M17, harden the dense-form layout reported by the community,
and publish the complete incubating and stable Rust families with reviewed
contributor credit.

| F-ID | Title | Size |
|------|-------|------|
| F-172 | Digital signature creation | M |
| F-173 | Tagged PDF structure tree | L |
| F-175 | Redaction | M |
| F-X048 | Dense form table fidelity | L |
| F-174 | PDF/A conformance | M |
| F-X049 | Tag rpptx-v0.5.0 | S |
| F-X050 | Tag v0.9.0 | S |

F-173 is the one a LibreOffice-based pipeline cannot do well, and the layout
engine already knows the document semantics it needs, because
`audit_accessibility` reads them. F-X048 reimplements Issue 42 and PR 43 on the
current transactional, bounded cache model rather than importing the stacked
draft branch. The two release stories run last and retain separate final
approval boundaries. Incubating 0.5.0 publishes before stable 0.9.0 so every
stable dependency pin resolves from crates.io. Both reviewed changelog sections
credit and link every included external issue and pull request, and each record
receives a maintainer release note for its contributor.

### M18, Format breadth

#### Sprint S54, RTF and caller font aliases

**Goal**: open M18 with the inbound format that blocks the most corpora, and
honour the document-facing family names supplied with caller fonts.

| F-ID | Title | Size |
|------|-------|------|
| F-176 | RTF reader | L |
| F-177 | RTF writer | M |
| F-183 | Image export options | S |
| F-X051 | Honor caller-supplied font family aliases | M |

F-183 rides along because it is a day of work against an entry point every
format in this milestone shares. F-X051 closes Issue 44 and PR 45 on the
hardened reusable-engine path from F-X043. It remains independent of the RTF
reader and writer.

#### Sprint S55, HTML and ODT in

**Goal**: add the two remaining inbound formats, restore the v0.9 editor path
to the reviewed relayout budget, and close the associated migration and
contribution records.

| F-ID | Title | Size |
|------|-------|------|
| F-178 | HTML import | L |
| F-179 | ODT reader | L |
| F-X052 | Restore interactive relayout performance | L |
| F-X053 | Complete layout migration and contribution records | S |

F-178, F-179, and F-X052 are independent. HTML import is the most requested
inbound conversion in every comparable library's tracker, and ODT is
procurement-mandated across European public bodies and read by nothing in
Rust. F-X052 closes the separate performance residual reported in Issue 46
without weakening the exact, transactional, bounded cache contract. F-X053
runs after it, records the `MarkedContent` migration requirement, and closes
Issue 44, Issue 46, and PR 45 with the exact F-X051 and F-X052 evidence.
Issues 39 and 42 remain closed because their reporter confirmed that note
operations recovered and F-X048 fully replaced the dense-form patches.

#### Sprint S56, ODT, EPUB and SVG out

**Goal**: close M18, integrate the six pending reader contributions, and
publish the stable family at its next pre-1.0 beta boundary.

| F-ID | Title | Size |
|------|-------|------|
| F-180 | ODT writer | L |
| F-181 | EPUB export | M |
| F-182 | SVG page export | M |
| F-X054 | Integrate PRs 47 through 52 | L |
| F-X055 | Tag v0.10.0 | S |
| F-X056 | Tag rpptx-v0.6.0 | S |
| F-X057 | Tag v0.10.1 | S |

F-181 and F-182 both fall out of work that exists: EPUB from the outline API,
SVG from the same `PageFrame` the PDF and PNG backends already consume.
F-X054 audits the ordered reader APIs and parser fidelity changes proposed by
PRs 47 through 52 against current main, retaining six distinct contribution
records and authenticated credit to `@pedroassumpcao`. F-X055 runs only after
all four implementation stories complete. Its immutable v0.10.0 attempt
published only `rdocx-opc` and `rdocx-oxml` before package verification
stopped. F-X057 owns the complete stable publication at v0.10.1 with reviewed
notes, compatibility guidance, direct record links, and contributor credit.
It also owns all nine notifications and the six authorized pull-request
closures after the complete publication and release body verify.

The immutable v0.10.0 attempt published `rdocx-opc` and `rdocx-oxml`, then
proved that the stable source graph requires shared layout APIs newer than the
published incubating 0.5.0 family. F-X055 is retained as an archived failed
release attempt. F-X056 publishes the complete shared and Presentation family
at 0.6.0 while stable packages remain at 0.10.0. F-X057 then publishes the
complete stable family at 0.10.1 against those verified shared dependencies.
Each tag retains its own reviewed SHA, notes, full gate, and separate final
approval.

### M20, Fidelity at scale

#### Sprint S57, The Word corpus

**Goal**: measure the Word renderer against documents nobody here wrote. This is
the largest untested surface in the workspace.

| F-ID | Title | Size |
|------|-------|------|
| F-196 | Word corpus | M |
| F-197 | Word SSIM harness | L |
| F-201 | Large document performance | L |

PowerPoint has 50 fetched decks and an SSIM harness. Word has seven samples this
project generates itself, so it can only catch a regression against its own
output and can never catch a disagreement with Word.

#### Sprint S58, Text shaping and incremental layout

**Goal**: close M20 and finish every planned non-spreadsheet capability before
the advanced spreadsheet programme begins.

| F-ID | Title | Size |
|------|-------|------|
| F-198 | Hyphenation | L |
| F-199 | Complex script shaping | L |
| F-200 | Vertical and bidirectional text | M |
| F-202 | Incremental layout | L |
| F-X061 | Support staged dependency checkpoints in run-sprint | S |
| F-X062 | Reuse restart pagination with notes and headers | M |
| F-X063 | Avoid duplicate caller-font byte comparisons | S |
| F-X058 | Shared multilingual text substrate | L |
| F-X059 | Tag rpptx-v0.7.0 | S |
| F-X064 | Accept whole-valued decimal table measurements | S |
| F-X067 | Prime Word fidelity Cargo dependencies | S |
| F-X065 | Expose tracked table grid changes | S |
| F-X066 | Classify legacy VML horizontal rules | S |
| F-X060 | Tag v0.11.0 | S |
| F-X068 | Tag rpptx-v0.8.0 | S |
| F-X069 | Tag v0.11.1 | S |
| F-X070 | Yank incomplete v0.11.0 packages | S |
| F-X031 | Require the CI gate in branch protection | S |

F-198 changes line breaking and therefore every line after the first hyphenated
one, so it lands after the corpus exists to measure it. Expect a declared hash
harness delta, and expect it to be large.

F-X061 makes ordinary and release dependency checkpoints resumable inside one
sprint. F-X062 and F-X063 close the two independently reproduced editor
performance cliffs from Issues 53 and 54. F-X058 then establishes the complete
shared text contract needed by F-198,
F-199, and F-200. F-X059 publishes that contract as the incubating 0.7.0 family
before the stable Word consumers verify against crates.io. F-X064 lands the
first external reader fix, and F-X067 primes the locked Word fidelity graph
before F-X065 and F-X066 land the remaining reader fixes. Together these are
the hardened or directly adopted outcomes of PRs 55 through 58 before the
paused Word text stories resume. F-X060 records the immutable partial v0.11.0
attempt, which published two packages before registry verification exposed the
missing shared source boundary. F-X068 publishes that shared boundary at
0.8.0, and F-X069 publishes the complete stable recovery at 0.11.1. F-X070
then yanks the two incomplete 0.11.0 registry entries after separate approval
without moving the tag. Each release retains the separate final approval
required by `/release`.

F-X031 remains at the non-spreadsheet boundary after the recovery and cleanup.
F-X029 creates the stable
repository-side `ci-gate` in S44. This operational story makes it a required
GitHub check after the Word fidelity and release gates have settled and before
the larger spreadsheet programme starts.

### M21, Presentation depth

#### Sprint S59, Presentation collaboration and security

**Goal**: deepen the package and collaboration surfaces before adding dynamic
rendering behavior.

| F-ID | Title | Size |
|------|-------|------|
| F-217 | Presentation collaboration and navigation model | L |
| F-221 | Presentation encryption and signatures | M |

The sprint adds comments, replies, sections, footer metadata, protected package
handling, and signature policy. Executable payloads are preserved and
inspectable, never run. Binary `.ppt` remains out of scope.

#### Sprint S60, Animation and transitions

**Goal**: turn preserved timing XML into deterministic, bounded behavior.

| F-ID | Title | Size |
|------|-------|------|
| F-213 | Animation and transition timing model | L |
| F-214 | Timeline evaluation and transition rendering | L |

Static rendering remains unchanged. The new timeline path evaluates supported
entrance, exit, emphasis, motion, transition, and morph behavior at explicit
timestamps, with unsupported extensions preserved and diagnosed.

#### Sprint S61, Audio, video and animated export

**Goal**: complete the media timeline from package relationships to output.

| F-ID | Title | Size |
|------|-------|------|
| F-215 | Audio and video package model | L |
| F-216 | Media poster and playback rendering | M |
| F-227 | Animated GIF and video export | L |

Codec handling is bounded to reviewed backends. Unsupported media remains
extractable and visible through poster frames and diagnostics rather than being
dropped.

#### Sprint S62, SmartArt, embedded content, and reader contributions

**Goal**: model the two major opaque PresentationML surfaces without executing
untrusted content.

| F-ID | Title | Size |
|------|-------|------|
| F-218 | Embedded object and macro inventory | L |
| F-219 | SmartArt typed model | L |
| F-X071 | Integrate PRs 61 through 64 | L |

OLE, ActiveX, and VBA remain inventory, extraction, replacement, removal, and
preservation surfaces. SmartArt gains typed inspection and editing. F-X071
independently adopts the reviewed Word
reader contributions from PRs 61 through 64 and hardens namespace, schema-order,
and effective-style edge cases before integration.

#### Sprint S63, ODP, notes and handouts

**Goal**: broaden modern presentation interchange, complete the presenter and
audience output surfaces, and remove two reporter-confirmed Word editor cache
cliffs.

| F-ID | Title | Size |
|------|-------|------|
| F-220 | SmartArt layout and rendering | L |
| F-222 | ODP read and write | L |
| F-223 | Modern presentation package variants | M |
| F-226 | Notes and handout export | M |
| F-X072 | Keep paragraph caching across note references | M |
| F-X073 | Restart ordinary-prose pagination within the aggregate cache | L |

F-220 completes the carried SmartArt validator boundary before F-222 consumes
SmartArt through ODP interchange. ODP uses a declared LibreOffice differential
boundary. Notes pages and handouts reuse the existing master hierarchy and
shared PDF and image backends. Modern macro, template, and slide-show variants
build on the embedded-content inventory from S62.

F-X072 and F-X073 address issues 65 and 66 sequentially. The first keeps
paragraph-cache reads enabled after note-reference paragraphs. The second lets
ordinary prose publish bounded restart checkpoints and charges them against the
existing aggregate cache budget.

#### Sprint S64, HTML and PDF slide import

**Goal**: turn common modern content sources into editable or explicitly
preserved slide content.

| F-ID | Title | Size |
|------|-------|------|
| F-224 | HTML slide content import | L |
| F-225 | PDF page content import | L |
| F-X074 | Tag rpptx-v0.9.0 | S |
| F-X075 | Preserve restart pagination across page-spanning paragraphs | M |
| F-X076 | Tag v0.12.0 | S |

HTML maps a bounded DOM and CSS subset into shapes. PDF imports either a
preserved page graphic or the declared editable text, image, path, and link
subset. Neither path promises arbitrary browser or PDF-engine compatibility.
F-X074 publishes the completed M21 boundary as the exact 15-package
incubating 0.9.0 family after review and separate final approval.
F-X075 is a late cross-cutting correction for Issue 67. It preserves the
recorded pagination pass across page-spanning ordinary prose while retaining
only complete-block restart checkpoints and every existing unsafe-state
exclusion. Its stable package release follows the presentation release under a
separate reviewed release story. F-X076 publishes that exact stable family at
0.12.0 with the reviewed PR 61 through 64 and Issue 65 through 67 contribution
inventory.

### M22, Word depth

#### Sprint S65, OfficeMath

**Goal**: author, convert, lay out, and render modern Word equations.

| F-ID | Title | Size |
|------|-------|------|
| F-228 | OfficeMath model and authoring | L |
| F-229 | OfficeMath layout and PDF rendering | M |
| F-230 | MathML and LaTeX conversion | M |

The equation model preserves unsupported siblings and remains independent of
legacy Equation Editor objects.

#### Sprint S66, Fields and dynamic tables of contents

**Goal**: update the field-driven structures real reports depend on.

| F-ID | Title | Size |
|------|-------|------|
| F-231 | Extended field evaluation | L |
| F-232 | Dynamic table of contents rebuild | L |

TOC, TC, formula, mail-merge control, and barcode fields retain unsupported
instructions and cached results rather than substituting guessed values.

#### Sprint S67, Advanced automation and comparison

**Goal**: deepen the two flagship document-automation workflows.

| F-ID | Title | Size |
|------|-------|------|
| F-233 | Advanced mail merge | L |
| F-234 | Full-story document comparison | L |
| F-235 | Comparison granularity and ignore policy | M |

Mail merge gains nested regions, multiple data sources, images, fragments, and
format hooks. Comparison expands across related stories and adds explicit word,
character, and ignore policies.

#### Sprint S68, Embedded content, forms and building blocks

**Goal**: expose modern package content that is currently preserved but opaque.

| F-ID | Title | Size |
|------|-------|------|
| F-236 | Embedded object and macro inventory | L |
| F-237 | Forms, glossary, and building blocks | L |

Executable objects and macros remain non-executing inventory surfaces. Legacy
form fields are modeled only where they occur inside modern OOXML packages.

#### Sprint S69, Modern Word package and web variants

**Goal**: close Word depth without opening a legacy `.doc` programme.

| F-ID | Title | Size |
|------|-------|------|
| F-238 | Flat OPC and modern Word package variants | M |
| F-239 | MHTML import and export | M |
| F-X077 | Share strict XML lexical validation | M |
| F-X080 | Restore CI release readiness | S |
| F-X079 | Tag rpptx-v0.10.0 | S |
| F-X078 | Tag v0.13.0 | S |
| F-X081 | Tag rpptx-v0.11.0 | S |
| F-X082 | Tag v0.13.1 | S |

Flat OPC, DOCM, DOTX, DOTM, and bounded MHTML share the current document model.
Binary `.doc` and Word 2003 XML remain permanent non-goals. The shared strict
XML lexical validator removes the three copies exposed by S68 without changing
their fail-closed owner contracts. F-X080 restores the hosted package, Pandoc,
and Python binding gates before either release begins. F-X079 publishes the new
shared API as the incubating family at rpptx-v0.10.0 before F-238 consumes it
through the stable graph. The M22 end gate then runs only modern package
fixtures. F-X078 runs last and prepares and publishes the exact seven-package
stable family at v0.13.0 only after that gate is clean. The immutable v0.13.0
attempt published five low-level packages before its registry graph exposed
F-238's newer `oxml-opc` contract. F-X081 publishes the coherent shared 0.11.0
family, then F-X082 publishes the complete stable recovery at v0.13.1. Each
publication uses `/release` and its own separate final approval at the reviewed
SHA.

### M23, From-scratch business documents

#### Sprint S70, Audit, roadmap, and editor performance

**Goal**: replace the provisional Word-completeness outline with an
evidence-backed capability matrix and final sprint plan, establish the private
and sanitized conformance gates, improve the product README, close confirmed
Issue 67, and land the three independently measured Issue 69 corrections.

| F-ID | Title | Size |
|------|-------|------|
| F-240 | Modern DOCX completeness audit and private corpus matrix | L |
| F-241 | Public authoring conformance harness | L |
| F-242 | Root README product and capability overview | M |
| F-X083 | Close confirmed Issue 67 and intake Issue 69 | S |
| F-X084 | Narrow note-part paragraph cache invalidation | M |
| F-X085 | Memoize restart body identities once per layout | M |
| F-X086 | Provenance-safe restart after body-length changes | L |

F-X083 lands first so Issue 67 is not reimplemented and the live Issue 69
evidence is attributed correctly. F-X084 through F-X086 are independent after
that intake and may run in parallel, but only one story may own a rendering
baseline change. F-240 is the planning authority. F-241 and F-242 consume its
approved matrix, and F-240 updates the provisional sprint contents before
S70 closes. The M24 end gate is S94.

#### Sprint S71, Fresh package, styles, and numbering

**Goal**: make a blank document own the complete package, style, theme, font,
numbering, and deterministic identifier foundation required by the private
business-document corpus.

| F-ID | Title | Size |
|------|-------|------|
| F-243 | Word-compatible fresh package profiles | L |
| F-244 | Corpus settings and document properties | L |
| F-245 | Corpus themes, font tables, and embedded fonts | L |
| F-246 | Corpus style authoring | L |
| F-247 | Complete numbering level and instance model | L |
| F-248 | Style-linked numbering, counters, TOC, and REF | L |
| F-249 | Deterministic package identifier allocation | M |
| F-X087 | Portable authored Word charts from PR 71 | L |
| F-X088 | Verify and close Issue 69 after S70 fixes | S |
| F-X089 | Capability-led README family | L |

F-243 and F-249 establish package identity before dependent parts are authored.
F-245 precedes effective styles. F-247 precedes the two-sided style linkage and
numbering consumers in F-248.

F-X087 through F-X089 are user-approved scope exceptions that raise S71 from
the usual maximum of seven F-IDs to ten. F-X087 hardens Kevin Brown's PR 71 on
the sprint branch after F-245 and F-249. F-X088 verifies the completed S70
Issue 69 mechanisms and closes the issue with contributor credit and reviewed
evidence. F-X089 restores capability-led product and crate messaging while
retaining compiled examples, package evidence, and honest boundaries.

#### Sprint S72, Sections, stories, and content ownership

**Goal**: expose ordered sections and one relationship-safe content model for
the body and every related story required by from-scratch generation. The
user-approved issue wave also restores producer-package compatibility,
searchable PDF text, comparison with drawings, CLI automation, Python access,
and a reviewed Python release path. The final contributor wave also integrates
PRs 77 through 80 and restores the deterministic hosted render oracle.

| F-ID | Title | Size |
|------|-------|------|
| F-250 | Ordered mutable section facade | L |
| F-251 | Complete section and page geometry | L |
| F-252 | Rich per-section headers and footers | L |
| F-253 | Container-neutral story editing | L |
| F-254 | Generic insert, move, clone, and remove operations | L |
| F-255 | Part-scoped assets, links, and relationships | M |
| F-256 | Transactional cross-document fragment import | L |
| F-X090 | Accept part-local producer drawing identities | S |
| F-X091 | Serialize unused root default namespaces safely | M |
| F-X092 | Preserve logical reading order in generated PDFs | L |
| F-X093 | Preserve drawings through document comparison staging | M |
| F-X094a | Expose Word collaboration and redline commands in rdocx-cli | M |
| F-X094b | Structured CLI text and layout plus guarded replacement | L |
| F-X094c | Priority rdocx Python collaboration, comparison, layout, and TOC | L |
| F-X094d | rdocx Python sections, styles, rich stories, and hyperlinks | L |
| F-X094e | rpptx Python rendering, comments, and notes | L |
| F-X095 | Integrate PRs 77 through 80 and restore deterministic CI | L |
| F-X096 | Align Python distribution versions and release tags | M |
| F-X094f | Prepare the version-aligned Python release paths | M |

F-253 is the common owner model for every mutation. Section and relationship
work then converges in F-252, while F-256 lands last against the complete
dependency-remapping surface. F-X090 through F-X093 are one-story corrections
for Issues 72 through 75. F-X094a through F-X094f split Issue 76 into reviewable
CLI, binding, and release-preparation contracts. F-X092 is the only wave that
may move the PDF hash baseline and lands after every unchanged-baseline story.
F-X095 adopts the hardened outcomes of contributor PRs 77 through 80 after the
existing S72 reader, CLI, and binding work. It is the only added story allowed
to move the Word XML hashes, and its declared seven-entry delta must leave PDF
and PNG fingerprints unchanged. F-X096 makes the Python distribution versions
independent and reserves tag namespaces that cannot start a Rust publication.
S72 prepares `py-rdocx-v0.13.2` and `py-rpptx-v0.11.0`, then each publication
remains a separate `/release` action with fresh approval at the reviewed SHA.

#### Sprint S73, Tables, rich content, and private corpus gate

**Goal**: close M23 with complete corpus tables, measure-then-size layout, rich
HTML and run content, modeled drawings and text boxes, deterministic field
materialization, and five public-facade generators starting from a blank
document.

| F-ID | Title | Size |
|------|-------|------|
| F-257 | Complete M23 table authoring | L |
| F-258 | Complete M23 row and cell authoring | L |
| F-259 | Container measurement and equal-height layout | M |
| F-260 | Ordered run content authoring | L |
| F-261 | Rich HTML fragments in arbitrary containers | L |
| F-262 | Corpus drawings, text boxes, and watermarks | L |
| F-263 | Layout-backed fields and M23 corpus gate | L |
| F-X097 | Preserve namespace-scoped drawings and complex fields in comparison | M |
| F-X098 | Preserve content-control type payloads | M |
| F-X099 | Expose direct body ownership for story items | M |
| F-X100 | Preserve explicit false table toggles | S |
| F-X101 | Honor run-level page breaks during pagination | M |
| F-X102 | Resolve header and footer pictures in their story scope | M |
| F-X103 | Accept standard TOC switches and report rebuild diagnostics | M |
| F-X104 | Render DrawingML picture transparency | M |
| F-X105 | Separate slide-owned placeholders from master header flags | M |
| F-X106a | Expose indexed content mutation and counted replacement in Python | L |
| F-X106b | Expose paragraph and run formatting mutations in Python | M |
| F-X106c | Expose story mutation, hyperlinks, revisions, fields, and XML in Python | L |
| F-X107 | Clone and remove existing table rows | M |
| F-X108 | Replace an existing picture atomically | M |
| F-X109 | Split text runs at Unicode character offsets | M |
| F-X110 | Control field updates on document open | S |
| F-X111 | Attach portable CLI binaries to Rust releases | L |
| F-X113 | Preserve appended paragraphs in document comparison | M |
| F-X114 | Rebuild TOC entries with document styles and geometry | M |
| F-X115 | Preserve modern comment metadata and identity | M |
| F-X116 | Make Python story reads linear and complete | L |
| F-X117 | Render transparent and large raster pictures safely | M |
| F-X118 | Make notes rendering and replacement safe | L |
| F-X119 | Complete round-three Python authoring and inspection | L |
| F-X120 | Accept fractional DOCX line spacing values | S |
| F-X121 | Adopt PR 123 authored line-chart portability | S |
| F-X122 | Recover the immutable rpptx 0.12.0 release attempt | M |
| F-X112 | Publish the complete S73 package families | L |

The model stories land before layout and conversion consumers. F-X100 lands
before F-258 so the row and cell setters inherit lossless toggle semantics.
F-X101 and F-X109 follow ordered runs, F-X102 precedes corpus drawings, and
F-X103 precedes layout-backed fields. F-X106a through F-X106c serialize the
second Python binding round, then the row and image mutations consume that
surface. F-X113 and F-X117 are independent corrections. F-X114 follows the
field and private-corpus layout foundations. F-X115 follows exact comment
anchoring, F-X116 follows the complete story binding and split-run paths, and
F-X118 establishes notes mutation before F-X119 closes the round-three binding
surface. F-X120 and F-X121 are independent contributor corrections. F-X122
repairs the failed immutable incubating release attempt and prepares its patch
family. F-263
remains the private-corpus milestone gate. F-X111 prepares portable CLI assets
before F-X112 releases the exact two Rust and two Python families after F-X113
through F-X122 are complete, using separate approvals.
PR 101 contributes the F-X100 explicit
false table-toggle fix and the F-X103 TOC `\\z` support. Its current two-commit
shape is incorporated directly where conflict-free or through a reviewed
hardened equivalent, with contributor credit retained. The five client
documents and their renders remain private and uncommitted.

### M24, Modern DOCX authoring completeness

#### Sprint S74, Complete text, table, section, and settings semantics

**Goal**: expand the corpus-proven primitives into complete public paragraph,
run, table, international typography, page, and document-setting authoring.

| F-ID | Title | Size |
|------|-------|------|
| F-264 | Complete paragraph property authoring | L |
| F-265 | Complete run property and inline authoring | L |
| F-266 | International and vertical typography | L |
| F-267 | Complete table style and conditional formatting authoring | L |
| F-268 | Floating and advanced table layout | L |
| F-269 | Complete section page semantics | L |
| F-270 | Complete settings and web settings authoring | L |
| F-X123 | Accept producer TOC style variants | S |
| F-X124 | Make content cloning linear and explicit | M |
| F-X125 | Compare table grid changes | M |
| F-X126 | Preserve drawings through comparison acceptance | M |
| F-X127 | Collapse adjacent page break requests | S |
| F-X128 | Preserve Word paragraph and revision identities | M |
| F-X129 | Tolerate unmatched notes placeholders | S |
| F-X130 | Show package depth, footprint, and speed | L |

Each model change carries its own parser, writer, public API, and render gate.
F-266 consumes the completed paragraph and run properties, while F-268 consumes
the table and conditional-style surface. F-X123 through F-X129 resolve Issues
124 through 132 before those planned surfaces expand. Issues 126 and 132 share
F-X124 because they exercise the same public clone call. Issue 68 is an
answered roadmap question and requires no implementation story.
F-X130 follows F-264 through F-270 so all 27 package READMEs and both PyPI long
descriptions can present the completed S74 surface with reproducible feature,
footprint, and performance evidence.

#### Sprint S75, Contribution intake and table pagination correction

**Goal**: restore the hosted Python binding gate, integrate the open
contribution wave and its unassigned issue reports, then finish Issue 138's
table pagination correction before one reviewed mid-milestone merge.

| F-ID | Title | Size |
|------|-------|------|
| F-X134 | Keep Python story hyperlink snapshots linear | S |
| F-X135 | Integrate PRs 146 through 151 and resolve unassigned reports | L |
| F-X136 | Fix table row breaks and footer-only pages | L |

F-X134 restores the hosted Python binding gate before F-X135 integrates the
six remaining contributor branches and repairs open reports without a pull
request. F-X136 finishes the distinct Issue 138 line-height and row-splitting
report after F-X135's spacing and table-margin changes. F-X133 and F-271
through F-277 were carried to the then-planned S76 at the approved cutoff.
The unstarted S76 stories are now scheduled from S86 after repair intake.

#### Sprint S76, Contribution intake and core repair

**Goal**: review the open contribution wave at its own commits, reconcile
overlapping Word and package changes, and land the first integrated repair
prefix. A PR is evidence for a story, not a substitute for its acceptance gate.

| F-ID | Title | Size |
|------|-------|------|
| F-X137 | Package and CLI safety contribution wave | L |
| F-X138 | Word story and content contribution wave | L |
| F-X139 | Word identity and comparison contribution wave | L |

F-X137 reviews PRs 174, 178, 182, 185 and 197. F-X138 reviews PRs
177, 179, 180, 191, 195, 202, 210 and 211 in that dependency order.
F-X139 reviews PRs 183, 184, 190, 193, 198 and 205. Integrate one reviewed
contribution at a time on `sprint/s76`, separating behavior changes from
file moves and reconciling semantic overlap manually.
Every integrated prefix reruns the relevant focused checks, then the final
combined result takes `/verify --full` and `/sprint-review`.

#### Sprint S77, Binding and CLI contribution completion

**Goal**: integrate the remaining reviewed rendering, layout, Python,
presentation and CLI contributions on the completed S76 prefix, with one
combined verification and review boundary.

| F-ID | Title | Size |
|------|-------|------|
| F-X140 | Rendering and layout contribution wave | L |
| F-X141 | Word Python contribution wave | L |
| F-X142 | Presentation Python contribution wave | L |
| F-X143 | Revision listing and CLI story contribution wave | M |

F-X140 reviews PRs 175, 188, 196, 199, 200, 206 and 207. It owns the
first labelled and reviewed hash baseline update. F-X141 owns a second,
separate baseline update for native common styles and any reviewed TOC output
delta. No other S77 story may move the baseline.
PR 196's Presentation fidelity gate and PR 206's MSRV and Test failures must
be resolved on the replayed integrated result before sprint closure. F-X141
reviews PRs 176, 187, 194, 201, 203 and 212. PR 203 follows 201. F-X142
reviews PRs 173, 181, 189, 192, 208 and 209. PRs 208
and 209 follow 189. F-X141 also completes the named section-default, native
common-style and refreshable TOC gaps beyond those PRs. F-X142 completes the
full Issue 169 checklist, including the API and rendering items outside its
PR set. F-X143 reviews PRs 186 and 204, with 204 following
S76 PR 198. Replay only each stacked PR's incremental diff after its
parent lands, run focused checks on the replayed result, and require a rebase
and green head CI for any original PR selected for a direct merge. Close a
superseded PR only after its coverage is verified on `main`. Binding smoke,
package, rendering, documentation and release regression checks run on the
combined result, followed by `/verify --full` and `/sprint-review`.

#### Sprint S78, Word producer and comparison repair

**Goal**: land the new Word preservation and comparison contributions, then
finish the identity, producer and redline acceptance matrices.

| F-ID | Title | Size |
|------|-------|------|
| F-X151 | Word preservation and comparison PR intake | L |
| F-X144 | Identity and producer matrices across operations | L |
| F-X145 | Comparison options and redline completion | L |

F-X151 reviews PRs 214, 228, 229, 232, 233 and 239 against the S77 prefix.
F-X144 follows that integration and completes Issues 157, 159 and 160.
F-X145 then completes Issue 161, including edited-side comments and rebuilt
TOCs. Rebase every old head against its actual parent, review only its
incremental diff, and rerun the combined matrix and full sprint gate.

#### Sprint S79, Word layout, CLI diff and binding repair

**Goal**: finish the Word layout and CLI defects, then close the remaining
Word Python binding gaps before the production fixture gate.

| F-ID | Title | Size |
|------|-------|------|
| F-X146 | Word line height and inline picture spacing | L |
| F-X152 | Full-story CLI diff and count repair | M |
| F-X153 | Word Python supplemental contribution | M |

F-X146 reviews PRs 222, 225 and 237 and completes Issue 162 and Issue
226's rich-line symptom. F-X163 completes its plain-line symptom after PR
242. PR 222's current head reports a green CI run, and the integrated
fidelity cases must still pass after rebasing.
F-X152 reviews PR 236 and completes Issue 227. F-X153 reviews PR 220,
including its Issue 168 comment and run operations. The Word fixture and
focused layout oracles run on the integrated result before `/verify --full`
and `/sprint-review`.

#### Sprint S80, Word checklist and presentation preservation

**Goal**: complete the Word Python production checklist and land the new
presentation preservation and drawing contributions on a reviewed prefix.

| F-ID | Title | Size |
|------|-------|------|
| F-X147 | Complete rdocx Python production checklist | L |
| F-X154 | Presentation text and preservation repair | M |
| F-X155 | Presentation drawing API contribution | L |

F-X147 completes every Issue 168 item after F-X141 and F-X153 or records a
reviewed scope decision with a documented fallback where the reporter permits
one. F-X154 reviews PRs 218 and 223 and completes Issues 215 and 216.
F-X155 reviews PRs 219, 221, 224, 230 and 234. PR 219 follows PR 189,
and PR 234 follows PR 207. Their shape, line, effect and binding overlap
requires manual reconciliation and a fresh CI run.

#### Sprint S81, Presentation composition and deck checklists

**Goal**: finish the presentation authoring surface and prove the complete
Issue 169 and Issue 217 deck checklists against independent viewers.

| F-ID | Title | Size |
|------|-------|------|
| F-X156 | Presentation slide and table contribution | L |
| F-X148 | Complete rpptx Python production checklist | L |
| F-X157 | Complete deck-chain authoring checklist | L |

F-X156 reviews PRs 231, 235 and 238. PR 231 follows PR 181. F-X148 checks
all Issue 169 items, including shape hyperlinks, anchored comments and
built-in table styles, after F-X142 and F-X156. F-X157 completes Issue 217,
including outer shadows, connectors without theme effects, line ends, preset
geometry, cross-deck import and scoped replacement. Both checklist stories
require Python API, package validation, python-pptx reopening, LibreOffice
render comparison and explicit scope decisions for any unsupported operation.

#### Sprint S82, Production acceptance and issue closure evidence

**Goal**: turn the reporter's Word and deck fixtures into repeatable acceptance
checks, then reconcile every open issue criterion against the integrated result.

| F-ID | Title | Size |
|------|-------|------|
| F-X149 | Word fixture and workflow acceptance gate | L |
| F-X158 | Presentation fixture and workflow acceptance gate | L |
| F-X150 | Reconcile issue closure evidence | M |

F-X149 owns the 18 by 7 identity matrix, 11 by 8 producer matrix and complete
DOCX workflow. F-X158 owns the deck workflow and cross-viewer acceptance for
Issues 169, 170, 215, 216 and 217. F-X150 records criterion-level evidence
for the 22 issues present in its original snapshot, with Issue 158 last. A
missing criterion keeps its issue open. The new issues and contributions from
1 October are scheduled below, ahead of the deferred feature work.

#### Sprint S83, Word package validity and tolerant reads

**Goal**: make newly authored Word documents open in Word and accept the
reported style, drawing ID and measurement producer variants. Review one
separate baseline change for document validity before the tolerance work.

| F-ID | Title | Size |
|------|-------|------|
| F-X159 | Word document validity baseline | M |
| F-X160 | Tolerant style, drawing and measurement reads | L |

F-X159 reviews PR 240 and owns this sprint's only hash baseline update.
F-X160 reviews PRs 248, 249 and 250 against that prefix. It closes Issues
243, 246 and 247 only after both Issue 243 reporter cases and every affected
mutator or read path pass on the integrated result. Word opening, exact XML,
round-trip, CLI and Python checks accompany the deterministic hash gate.

#### Sprint S84, Integrated contribution and issue repair

**Goal**: finish the current contribution queue and every remaining open issue
criterion in one integrated sprint. Run scoped feature checks and dependency
checkpoints during implementation, then one full verification and sprint review
on the combined result.

| F-ID | Title | Size |
|------|-------|------|
| F-X169 | Reconcile live contributions and open issue contracts | L |
| F-X161 | Compact Word XML and namespace preservation | L |
| F-X162 | Per-paragraph section width and pagination | M |
| F-X163 | Plain-line trailing-space fit | M |
| F-X164 | Added shape theme style | M |
| F-X165 | Python and CLI tracked revision view | M |
| F-X166 | Picture and final-block comparison revisions | L |
| F-X167 | Accepted-view exporters and readers | L |
| F-X170 | High-level Word style formatting API | M |
| F-X168 | Current issue and contribution closure evidence | M |

F-X169 classifies all 31 open PRs against integrated main before implementation.
The 19 older heads need patch-equivalence and contributor disposition checks.
The 12 newer heads belong to F-X161 through F-X167. Complete F-X169 at a
scoped dependency-prefix checkpoint so F-X161, F-X164 and F-X165 may start.

F-X161 reviews PR 251 first and owns its labelled 20-entry OOXML hash delta.
F-X162 follows F-X161 with PR 241 and a separate section-layout baseline.
F-X163 follows F-X162 with PR 242, its separate hash delta and golden pixel
manifest. The three baseline changes are exclusive and reviewed in order.
F-X164 reviews PR 252 independently after intake, with python-pptx and viewer
checks. F-X165 reviews PR 256, then F-X166 reconciles PRs 257, 258, 260 and
261. F-X167 follows F-X165 and F-X166 with PRs 259, 262 and 263. F-X170
addresses Issue 264, opened after the 2 October intake, independently. F-X168
checks every open issue against the final integrated result. Formal
prerequisites complete through scoped checkpoints before their consumers
start. These checkpoints do not run the full workspace or sprint review.

PR 265 arrived on 3 October after F-X162 completed. It targets the same section
ownership defect as PR 241. F-X168 adopts its distinct mixed-orientation,
multipage table and content-control regression on the integrated F-X162 fix,
then records a superseded disposition for PR 265. Its head has no CI run and
reports incomplete local full-gate coverage, so the integrated sprint gate
supplies the missing evidence.

The ten F-IDs represent about 29 to 38 estimated developer-days, above the
normal sprint size by request. Keep their approved designs, individual
microscope reviews and scoped verification. Run the union of risk riders,
source-built round trips, reporter fixtures, the 18 by 7 identity and 11 by 8
producer matrices, Python and CLI checks, deterministic Word and PowerPoint
comparisons, the hash harness, `/verify --full` and `/sprint-review` once on
the final integrated S84 result. After `/close-sprint` merges that result to
main, close all 32 open PRs and nine open issues through the 3 October refresh
with evidence links. Feature work resumes at S86.

#### Live contribution and issue inventory, 2 October 2026

The 2 October intake reported 31 open PRs and eight open issues. Every PR in
that intake targets
`main`, has a green latest head CI run, has no submitted review, and is marked
conflicting with current `main`. PRs #179 and #259 retain failed historical
runs in the rollup, but their latest checks pass. Head CI does not prove that
the combined result passes. The source overlap column shows one current PR
with overlapping source files, not every possible conflict.

| PR | Base | Head branch and SHA | Latest CI | Review | Mergeability | Stack | Source overlap | F-ID | Issues |
|---|---|---|---|---|---|---|---|---|---|
| [#174](https://github.com/tensorbee/rdocx/pull/174) | `main` | `fix/cli-output-overwrite-and-closed-pipe` / `0f108ffb90` | green | none | conflict | - | #256 (`crates/rdocx-cli/src/main.rs`) | F-X137 | [156](https://github.com/tensorbee/rdocx/issues/156)*, [166](https://github.com/tensorbee/rdocx/issues/166)* |
| [#177](https://github.com/tensorbee/rdocx/pull/177) | `main` | `fix/body-readers-reach-content-controls` / `367c48ace8` | green | none | conflict | - | #263 (`crates/rdocx/src/document.rs`) | F-X138 | [160](https://github.com/tensorbee/rdocx/issues/160) |
| [#178](https://github.com/tensorbee/rdocx/pull/178) | `main` | `fix/atomic-library-save` / `4e582fcb09` | green | none | conflict | - | #193 (`crates/rdocx/src/document.rs`) | F-X137 | [164](https://github.com/tensorbee/rdocx/issues/164)* |
| [#179](https://github.com/tensorbee/rdocx/pull/179) | `main` | `fix/direct-body-index-coordinates` / `128e61bdee` | green | none | conflict | - | #191 (`crates/rdocx/src/document.rs`) | F-X138 | [163](https://github.com/tensorbee/rdocx/issues/163) |
| [#180](https://github.com/tensorbee/rdocx/pull/180) | `main` | `fix/text-box-rewrite-keeps-all-children` / `242e9984b3` | green | none | conflict | - | #211 (`crates/rdocx-oxml/src/placeholder.rs`) | F-X138 | [160](https://github.com/tensorbee/rdocx/issues/160) |
| [#182](https://github.com/tensorbee/rdocx/pull/182) | `main` | `fix/story-splice-namespace-facts` / `46c128e3c9` | green | none | conflict | - | #263 (`crates/rdocx/src/document.rs`) | F-X137 | [157](https://github.com/tensorbee/rdocx/issues/157) |
| [#183](https://github.com/tensorbee/rdocx/pull/183) | `main` | `fix/toc-rebuild-unbound-word-prefix` / `1bed176b32` | green | none | conflict | - | #193 (`crates/rdocx-oxml/src/text.rs`) | F-X139 | [159](https://github.com/tensorbee/rdocx/issues/159) |
| [#184](https://github.com/tensorbee/rdocx/pull/184) | `main` | `fix/compare-producer-noise` / `83b92bbf69` | green | none | conflict | - | #211 (`crates/rdocx-oxml/src/placeholder.rs`) | F-X139 | [159](https://github.com/tensorbee/rdocx/issues/159), [160](https://github.com/tensorbee/rdocx/issues/160) |
| [#185](https://github.com/tensorbee/rdocx/pull/185) | `main` | `fix/rewritten-part-roots-and-comments-part` / `88a4838704` | green | none | conflict | - | #193 (`crates/rdocx/src/document.rs`) | F-X137 | [160](https://github.com/tensorbee/rdocx/issues/160) |
| [#190](https://github.com/tensorbee/rdocx/pull/190) | `main` | `fix/compare-diagnostics-instead-of-refusals` / `4e0be1e8ea` | green | none | conflict | #184 | #211 (`crates/rdocx-oxml/src/placeholder.rs`) | F-X139 | [159](https://github.com/tensorbee/rdocx/issues/159), [160](https://github.com/tensorbee/rdocx/issues/160), [161](https://github.com/tensorbee/rdocx/issues/161) |
| [#191](https://github.com/tensorbee/rdocx/pull/191) | `main` | `fix/comment-anchoring-run-index` / `573d8c670a` | green | none | conflict | #179 | #211 (`crates/rdocx/src/document.rs`) | F-X138 | [163](https://github.com/tensorbee/rdocx/issues/163), [172](https://github.com/tensorbee/rdocx/issues/172) |
| [#193](https://github.com/tensorbee/rdocx/pull/193) | `main` | `fix/table-row-identity-attributes` / `7a8841fbcb` | green | none | conflict | #183 | #211 (`crates/rdocx-oxml/src/footnotes.rs`) | F-X139 | [159](https://github.com/tensorbee/rdocx/issues/159) |
| [#195](https://github.com/tensorbee/rdocx/pull/195) | `main` | `fix/replace-reaches-content-controls` / `bb76c4bd35` | green | none | conflict | #184 + #180 | #211 (`crates/rdocx/src/document.rs`) | F-X138 | [160](https://github.com/tensorbee/rdocx/issues/160) |
| [#197](https://github.com/tensorbee/rdocx/pull/197) | `main` | `fix/template-saved-as-document-content-type` / `8cdfd4a913` | green | none | conflict | - | #263 (`crates/rdocx/src/document.rs`) | F-X137 | - |
| [#198](https://github.com/tensorbee/rdocx/pull/198) | `main` | `feat/cli-compare-options-comment-dates` / `ff24553e60` | green | none | conflict | - | #263 (`crates/rdocx/src/document.rs`) | F-X139 | [161](https://github.com/tensorbee/rdocx/issues/161) |
| [#202](https://github.com/tensorbee/rdocx/pull/202) | `main` | `fix/replace-reaches-notes-and-tracked-insertions` / `5c3d81cf90` | green | none | conflict | #195 | #211 (`crates/rdocx-oxml/src/footnotes.rs`) | F-X138 | [160](https://github.com/tensorbee/rdocx/issues/160) |
| [#205](https://github.com/tensorbee/rdocx/pull/205) | `main` | `fix/comparison-marker-unit-boundaries` / `7704245c99` | green | none | conflict | - | #211 (`crates/rdocx-oxml/src/text.rs`) | F-X139 | [161](https://github.com/tensorbee/rdocx/issues/161) |
| [#210](https://github.com/tensorbee/rdocx/pull/210) | `main` | `fix/word-text-boxes-read-and-counted-once` / `7d8e03b7d1` | green | none | conflict | #195 | #211 (`crates/rdocx/src/document.rs`) | F-X138 | [160](https://github.com/tensorbee/rdocx/issues/160) |
| [#211](https://github.com/tensorbee/rdocx/pull/211) | `main` | `fix/text-inside-simple-fields-smart-tags-custom-xml` / `bce7ebcd26` | green | none | conflict | #202 | #202 (`crates/rdocx-oxml/src/footnotes.rs`) | F-X138 | [160](https://github.com/tensorbee/rdocx/issues/160) |
| [#241](https://github.com/tensorbee/rdocx/pull/241) | `main` | `fix/word-section-width-per-paragraph` / `09dd97d614` | green | none | conflict | - | #263 (`crates/rdocx-layout/src/engine.rs`) | F-X162 | - |
| [#242](https://github.com/tensorbee/rdocx/pull/242) | `main` | `fix/word-plain-line-fit-hangs-trailing-space` / `14358319b4` | green | none | conflict | #222, already integrated | #263 (`crates/rdocx/src/document.rs`) | F-X163 | [226](https://github.com/tensorbee/rdocx/issues/226) |
| [#251](https://github.com/tensorbee/rdocx/pull/251) | `main` | `fix/rewritten-parts-keep-word-layout` / `2ae5d6189f` | green | none | conflict | - | #193 (`crates/rdocx-oxml/src/footnotes.rs`) | F-X161 | [245](https://github.com/tensorbee/rdocx/issues/245) |
| [#252](https://github.com/tensorbee/rdocx/pull/252) | `main` | `fix/rpptx-added-shapes-get-the-theme-style` / `966b83ae7c` | green | none | conflict | - | #178 (`crates/rpptx/src/odp.rs`) | F-X164 | [244](https://github.com/tensorbee/rdocx/issues/244) |
| [#256](https://github.com/tensorbee/rdocx/pull/256) | `main` | `feat/revision-view-python-cli` / `ad088becbf` | green | none | conflict | - | #198 (`crates/rdocx-cli/src/main.rs`) | F-X165 | [253](https://github.com/tensorbee/rdocx/issues/253) |
| [#257](https://github.com/tensorbee/rdocx/pull/257) | `main` | `fix/compare-changed-picture` / `e6562e7821` | green | none | conflict | - | #211 (`crates/rdocx/src/document.rs`) | F-X166 | [254](https://github.com/tensorbee/rdocx/issues/254) |
| [#258](https://github.com/tensorbee/rdocx/pull/258) | `main` | `fix/compare-final-table-and-paragraph` / `42bd34f231` | green | none | conflict | - | #261 (`crates/rdocx/src/revision.rs`) | F-X166 | [255](https://github.com/tensorbee/rdocx/issues/255) |
| [#259](https://github.com/tensorbee/rdocx/pull/259) | `main` | `fix/html-markdown-export-tracked-changes` / `5bc41cb280` | green | none | conflict | - | #263 (`crates/rdocx-html/src/lib.rs`) | F-X167 | [253](https://github.com/tensorbee/rdocx/issues/253) (follow-up) |
| [#260](https://github.com/tensorbee/rdocx/pull/260) | `main` | `fix/reject-paragraph-property-change-keeps-mark` / `39a9c9d97b` | green | none | conflict | - | #261 (`crates/rdocx/src/revision.rs`) | F-X166 | [255](https://github.com/tensorbee/rdocx/issues/255) (dependency) |
| [#261](https://github.com/tensorbee/rdocx/pull/261) | `main` | `fix/compare-final-paragraph-keeps-properties` / `cdfcefbe95` | green | none | conflict | #258 + #260 | #260 (`crates/rdocx/src/revision.rs`) | F-X166 | [255](https://github.com/tensorbee/rdocx/issues/255) |
| [#262](https://github.com/tensorbee/rdocx/pull/262) | `main` | `fix/accepted-view-drops-deleted-paragraphs` / `32a12735ac` | green | none | conflict | #259 | #193 (`crates/rdocx/src/document.rs`) | F-X167 | [253](https://github.com/tensorbee/rdocx/issues/253) (follow-up) |
| [#263](https://github.com/tensorbee/rdocx/pull/263) | `main` | `fix/accepted-view-drops-deleted-rows` / `6252b10a8d` | green | none | conflict | - | #259 (`crates/rdocx-html/src/lib.rs`) | F-X167 | [253](https://github.com/tensorbee/rdocx/issues/253) (follow-up) |

The 19 older open heads, #174 through #211 in the table, predate the
completed S76 integration commits. The S76 AS_BUILT entries explicitly list
every one of these PRs and say that unique behavior commits were replayed while
measurement-only tails and duplicate ancestors were omitted. The integration
commits below are ancestors of the current sprint branch, and GitHub reports no
later revision to these PR heads. This is behavioral adoption, not identical
patch text. Their final disposition is superseded by the integrated work after
the combined S84 gates pass. Do not replay their stale branches.

| Older PRs | Current disposition | Integrated evidence | Final check |
|---|---|---|---|
| #174, #178, #182, #185, #197 | Behavior integrated, archive measurement tails omitted | `26b3f880`, F-X137 in `AS_BUILT.md` | CLI, package and save regressions on final main |
| #177, #179, #180, #191, #195, #202, #210, #211 | Behavior integrated, duplicate ancestors and measurement tails omitted | `c8975fb4`, F-X138 in `AS_BUILT.md` | Word story, replacement and Issue 160 matrix on final main |
| #183, #184, #190, #193, #198, #205 | Behavior integrated, measurement tails omitted | `992930ce`, F-X139 in `AS_BUILT.md` | Identity, comparison and CLI regressions on final main |

If a final gate exposes an omitted behavior, reconcile it under an approved
F-ID and revise this disposition before any PR closes. Contributor closure
comments must name the adopted integration commit and the final test evidence.

For pending work, review and replay #251 first, then #241, then #242, with
separate hash baseline commits and deterministic render evidence. #242 includes
#222, already integrated. Review #252 after its already integrated presentation
style prerequisites. Review #256 before the comparison and accepted-view set.
Then review #257, #258, #260, #261 in that order, with #261 depending on both
#258 and #260. Review #259, #262, #263 in that order, with #262 stacked on
#259. Rebase each head onto the reviewed prefix and reconcile shared files
against both approved plans. PR #195 also carries #180's code, so its older
stack must be compared incrementally. PR #257's comment supplies Word for Mac
picture evidence, while #260's comment explains #261's marker-order dependency.

The current conflicts require manual semantic reconciliation in
`rdocx-oxml` serializers, `rdocx-layout` line and section code,
`rdocx/src/comparison.rs`, accepted-view exporters, shared regression test
entrypoints and their HLD sections. Recheck the exclusive `hash_baseline.json`
updates from #251, #241 and #242 one at a time. #242 also changes the golden
pixel manifest. No current head CI job is failing. Missing GitHub reviews,
conflicting bases, and the untested combined result block direct PR merges.

| Open issue | PR coverage | Remaining acceptance and closure evidence | F-ID |
|---|---|---|---|
| [#158](https://github.com/tensorbee/rdocx/issues/158) | Earlier contribution waves, #251 and #242 | Keep both SHA-bound fixtures, the 18 by 7 identity and 11 by 8 producer matrices, and both complete workflows green on integrated main. Close only after #160 and #226 and all child criteria pass. | F-X169, F-X168 |
| [#160](https://github.com/tensorbee/rdocx/issues/160) | Earlier #177, #180, #184, #185, #190, #195, #202, #210, #211 and #251 | Recheck every body walker location, packed fields, no-op comments bytes, `mc:Ignorable` binding, and all 11 by 8 producer cells on serialized output after #251. | F-X161, F-X168 |
| [#226](https://github.com/tensorbee/rdocx/issues/226) | Integrated #222 and pending #242 | Prove plain and explicit `w:rtl=false` line breaks at 1.70 and 2.10 inches, with hanging space and no punctuation or space at line start, against pinned Word or LibreOffice output. | F-X163, F-X168 |
| [#244](https://github.com/tensorbee/rdocx/issues/244) | #252 | Added shape has python-pptx's `p:style`, visible fill and line in rpptx, LibreOffice and PowerPoint, reopens in python-pptx, and accepts theme effect index zero. | F-X164, F-X168 |
| [#245](https://github.com/tensorbee/rdocx/issues/245) | #251 | Edited package keeps untouched parts and regions, compact rewritten XML, one root `w` declaration, valid namespaces and the individually reviewed 20-entry OOXML hash delta. | F-X161, F-X168 |
| [#253](https://github.com/tensorbee/rdocx/issues/253) | #256, related #259, #262, #263 | Python PDF and page render plus both CLI paths select tracked or accepted view, default accepted, reject bad selectors, and give old plus new text versus new only. Check accepted exporters after the follow-ups. | F-X165, F-X167, F-X168 |
| [#254](https://github.com/tensorbee/rdocx/issues/254) | #257 | All three picture cases work at run and word granularities, retain both media parts, and accept or reject to the correct image, caption and size. Repeat the contributor's Word for Mac check on the integrated result. | F-X166, F-X168 |
| [#255](https://github.com/tensorbee/rdocx/issues/255) | #258, #260, #261 | All five final-table pairs work at run and word granularities, accept and reject to exact paragraphs, tables and properties, and open in Word with inserted or deleted table marks. | F-X166, F-X168 |

The 3 October refresh found [#264](https://github.com/tensorbee/rdocx/issues/264),
a new open feature request with no PR. A [3 October response](https://github.com/tensorbee/rdocx/issues/264#issuecomment-5967685523)
explains the active S84 implementation and keeps the issue open. `StyleBuilder` and
`Document` already create, update and manage styles, but simple style formatting
still requires raw `CT_PPr` and `CT_RPr` values. F-X170 owns a focused facade
for common paragraph and run settings, with a save and reopen example. F-X168
must verify the API before closing #264. The PR count and all recorded heads
remain unchanged at 31.

The later 3 October refresh found [PR #265](https://github.com/tensorbee/rdocx/pull/265),
a draft alternate to PR #241. It targets `main` from
`fix/mixed-section-measure` at `177dd7cbc4`, has no CI jobs or submitted
reviews, and GitHub reports it mergeable but blocked as a draft. It overlaps
`crates/rdocx-layout/src/engine.rs` with F-X162 and PR #241, with no stacked
dependency. Its production section selection is already covered by F-X162's
`main_story_item_sections` and the section-width tests. F-X168 will adopt the
new multipage, mixed-orientation table and content-control test on the
integrated implementation, run it under deterministic fonts, and cite the
combined gate before closing #265 as superseded. This makes 32 open PRs and
nine open issues at the latest refresh.
It addresses no open issue directly.

| New open issue | PR coverage | Remaining acceptance and closure evidence | F-ID |
|---|---|---|---|
| [#264](https://github.com/tensorbee/rdocx/issues/264) | None | Show convenient style creation and management through the existing `Document` surface, configure common paragraph and run formatting without constructing `CT_PPr` or `CT_RPr`, and prove that an applied style survives save and reopen. | F-X170, F-X168 |

An issue without a direct PR is #158, the umbrella acceptance contract. All
eight issues from the 2 October intake stay open until every criterion passes on integrated main. The
S84 consolidation exceeds one normal sprint. F-X169 records live intake,
F-X161 through F-X167 and F-X170 implement the remaining slices, and F-X168 closes the
evidence ledger last.
Each F-ID requires focused tests, applicable oracle and risk checks, scoped
verification and a zero-finding microscope. After all increments integrate,
one `/verify --full`, the hash harness and a clean `/sprint-review` precede
`/close-sprint`. The final issue closure check uses both full fixture workflows
and matrices, binding and CLI checks, deterministic render comparisons and the
combined hash baseline. Close every remaining PR and issue after the reviewed
result reaches main and its hosted CI gate passes. The S84 main push exposed a
missing pinned LibreOffice setup in the macOS Python job. F-X171 repairs that
hosted gate before the prepared closures are posted.

#### Sprint S85, Hosted CI repair and contribution closure

**Goal**: restore the required hosted CI gate on the S84 main result, then
reconcile all 32 reviewed PRs and nine issue contracts against the green
integrated result. Keep the published `s84` tag fixed and start the planned
feature work in S86.

| F-ID | Title | Size |
|------|-------|------|
| F-X171 | Pin LibreOffice for macOS Python acceptance | S |

The S84 local full gate and manual viewer evidence passed. The hosted
presentation binding job failed only because it had no `soffice` executable.
Install the exact reviewed LibreOffice image on that job, verify its digest and
build identity, and rerun the full Python suite and aggregate CI gate. Close
the S84 PR and issue queue only after `main` is green. This one-story repair
boundary avoids moving the already published `s84` tag.

#### Sprint S86, Rich related-story foundations and unified releases

**Goal**: establish rich header, footer, and note authoring on the existing
story model, remove repeated canonical-prefix rebinding from retained
elements, and prepare and publish one combined CLI and Python release per
family for Issue 266.

| F-ID | Title | Size |
|------|-------|------|
| F-X133 | Stop rebinding a canonical prefix on every retained element | S |
| F-271 | Uniform rich header and footer editing | L |
| F-272 | Rich footnote authoring | L |
| F-273 | Rich endnote authoring | L |
| F-X172 | Unified release artifacts and provenance | L |
| F-X173 | Prepare unified rpptx 0.13.0 family | M |
| F-X174 | Prepare unified rdocx 0.15.0 family | M |

F-X133 is independent and may proceed alongside F-271. F-272 establishes
the note-authoring model that F-273 extends. The other 11 stories from the
original S76 inventory move together to S87 in dependency order. Issue 266
adds a release pipeline story followed by the incubating and stable release
preparations. After `/close-sprint` merges the reviewed result to `main`, each
tag requires its own final approval at that merge SHA.

#### Sprint S87, Release archive evidence repair

**Goal**: repair the Linux package footprint checks exposed by the S86 main
CI run, then verify a clean release source for both Issue 266 families. The
previously planned related-story work moves to S88 because publication must
follow a reviewed green main merge.

| F-ID | Title | Size |
|------|-------|------|
| F-X175 | Refresh CLI archive evidence after release hardening | S |

The repair updates the two CLI archive measurements after the reviewed Windows
stack correction. Hosted CI, local package evidence, the full gate and a clean
sprint review must pass before `/close-sprint S87`. The two family publications
then follow through separate `/release` approvals at the S87 main merge SHA.

#### Sprint S88, Release inventory and version repair

**Goal**: repair the tag-only asset inventory failure before either registry
publishes Issue 266 packages. Preserve the failed `rpptx-v0.13.0` tag, prepare a
new PowerPoint version and the stable Word version from a reviewed main merge,
and run a build-only rehearsal and the complete gate before fresh approvals.

| F-ID | Title | Size |
|------|-------|------|
| F-X176 | Repair unified release inventory and respin PowerPoint | M |

The tag workflow's publication dry run does not retain `.crate` archives on a
fresh target directory. F-X176 checks the unpacked package inventory instead,
adds a clean-target regression, refreshes version carriers and notes, and
proves both selected release families before `/close-sprint S88`. The related
story wave moves to S89 so publication follows this focused repair.

#### Sprint S89, Related stories and safe fragment reuse

**Goal**: complete note separators and numbering policy, cross-story range
markers, deterministic fragment dependency remapping, and public glossary
creation. These four stories establish the shared story and package semantics
needed by later fields, templates, forms, and collaboration work.

| F-ID | Title | Size |
|------|-------|------|
| F-274 | Note separators, markers, and restart policy | L |
| F-275 | Cross-story bookmarks, ranges, and annotations | L |
| F-276 | Complete fragment conflict and dependency policy | L |
| F-277 | Glossary and building-block creation | L |
| F-X177 | Accept unified fontdb source features | S |

F-274 closes the note policy left by rich footnotes and endnotes. F-275 makes
paired markers valid across stories. F-276 then remaps a full fragment's
conflicting dependencies without dangling IDs, and F-277 uses that transaction
for building-block insertion. The sprint gate includes save-reopen checks for
relationships and untouched XML. This wave moved from S87 through S88 to leave
room for the release repairs.
The user added PR 269 during S89. F-X177 preserves downstream fontdb feature
unification and uses a separate wave between F-276 and F-277 because archive
evidence and HLD files are shared. The four authoring contracts retain their
dependency order.

#### Sprint S90, Fields, pagination, and navigation

**Goal**: provide a complete field construction surface and deterministic
field results across stories, then compose captions, cross-references, indexes,
bounded native citations and bibliography results.

| F-ID | Title | Size |
|------|-------|------|
| F-278 | General simple and complex field builder | L |
| F-279 | Pagination field materialization across stories | L |
| F-280 | Captions, sequences, and complete cross-references | M |
| F-281 | Indexes and tables of figures and authorities | L |
| F-282 | Citations and bibliography authoring | L |
| F-X179 | Correct multi-paragraph comment threads from PR 271 | S |
| F-X180 | Correct cell nil and none border precedence | S |
| F-X181 | Ignore page and column breaks inside table cells | S |
| F-X182 | Honor direct table alignment | M |
| F-X183 | Correct table margins and legacy positioning | M |
| F-X184 | Safe comment ownership during content removal | L |
| F-X185 | Expose comment anchor text and story location | M |
| F-X186 | Move comment anchors without losing threads | M |
| F-X187 | Scoped paragraph and cell text replacement | M |
| F-X188 | Preserve comment ownership when replacing or removing whole stories | L |
| F-X190 | Preserve cached text in multi-run complex field story snapshots | M |
| F-X191 | Ignore namespace declarations in numbering reader completeness | S |
| F-X189 | Prepare Word 0.16.0 and PowerPoint 0.14.0 families | L |

F-278 is the field substrate. F-279 uses the S89 note policy, and F-280 uses
S89 range markers. F-281 follows the field, pagination, and caption work.
F-282 follows F-278 and delivers its revised measured-subset contract.
F-283 carries the composed numbering and navigation result to S91. The S90
gate checks delivered field caches and page targets against pinned Word controls.
F-X178 is carried out of the S90 execution at the user's instruction to leave
Issue 264 alone. Its backlog row remains pending for separate work. F-282
covers the measured native source-authoring and formatter subset. F-X192 owns the
remaining full catalogue work in S91. F-283 is carried to S91.

F-X179 integrates PR 271 only after every Issue 270 criterion passes. It has
no dependency and preserves Issue 264 and F-X178 for separate work.

The user added Issues 272 and 273 with PRs 274 and 275 during S90.
F-X180 and F-X181 validate each full issue before accepting its contribution.
Their source and regression files overlap F-282 and F-283. Pause F-282 at a
saved checkpoint, run these independent fixes in separate waves, then resume
the bounded F-282 contract. F-283 is carried to S91. Source and Cargo ownership never overlap. Reconcile both
approved contracts at integration and check table geometry and bibliography
output. Issue 264 remains excluded.


Newly opened Issues 277 and 276 add F-X182 and F-X183. Run direct alignment
before legacy positioning, with F-X182 completed at a scoped dependency
checkpoint before F-X183 starts. Keep F-282 paused through these exclusive
source and hash-baseline waves, then resume its bounded S90 contract. F-283 is carried to S91. Each rendering
delta is separately declared and reviewed. Issue 264 remains excluded.
PR 279 supplies F-X182. PR 280 is stacked on it and supplies F-X183, including new Issue 278 for unset side margins. Validate both complete issue sets before contributor disposition.


Issue [281](https://github.com/tensorbee/rdocx/issues/281), raised by `hadim`,
adds roadmap feedback to this sprint intake. Record its unified CLI/Python
workflow, template filling, preservation-safe edits and rendering needs in the
existing F-184 decision at S95. The [maintainer response](https://github.com/tensorbee/rdocx/issues/281#issuecomment-6058792996)
answers scheduling and staged-delivery questions and requests concrete workflows
and preservation needs. Keep the discussion open for the F-184 decision. S90 delivers the documented
assessment, while spreadsheet implementation retains its affirmative decision
barrier.

Issues [282](https://github.com/tensorbee/rdocx/issues/282), [283](https://github.com/tensorbee/rdocx/issues/283), [284](https://github.com/tensorbee/rdocx/issues/284) and [285](https://github.com/tensorbee/rdocx/issues/285), reported by `hadim`, add F-X184 through F-X187. The intake was read against canonical 0a775842a8fd12f088bf4f2b3d0a049ddc3e976b. All four were opened on 2026-10-08 at 13:35 UTC and have no matching contribution PR at intake. Subsequent [PR 286](https://github.com/tensorbee/rdocx/pull/286), by `hadim`, targets Issue 285 at head `08361f6af99110cbae71bf9f0498a12feadc01db` and is included in F-X187 contribution assessment. Its patch must satisfy the approved scope and integrated verification before acceptance or closure. Their approved plans retain the complete issue criteria. Issue numbers 282 and 283 are distinct from existing feature IDs F-282 and F-283.

Pause F-282 only at an explicit saved external checkpoint. Run F-X184, F-X185, F-X186 and F-X187 in exclusive waves 11, 12, 13 and 14, respectively. F-X184 uses completed F-X179 and F-271. Complete each formal dependency before starting its consumer. F-X187 has no formal dependency but follows F-X186 because document, comment, binding and existing test entrypoints overlap. These waves exclusively own shared source, Cargo execution and HLD edits. Resume the revised measured-subset F-282 contract afterward. Remaining bibliography work is F-X192 in S91, and F-283 is carried to S91. Draft batch planning does not waive implementation barriers or the final integrated full gate. Issue 264 and F-X178 remain excluded. GitHub closure waits for verified sprint close and complete issue acceptance.

Subsequent [PR 287](https://github.com/tensorbee/rdocx/pull/287), by Hadrien
Mary (`hadim`), targets Issues 282, 283 and 284 at immutable head
`e4b216934eb3abc58ea1a42d72a902f90aa8e120`, based on
`20888b7a2c636e322ad23dc611f494beaac1c09b`. It is included in F-X184 through
F-X186 contribution assessment. Existing F-X184 acceptance retains atomic
partial-cut and fragment-detach refusal. Assess reusable anchor projection,
typed binding and movement changes against the full F-X185 and F-X186 plans,
without replacing the completed ownership protections or accepting the PR's
stated note and header removal gaps. The stacked F-X179 contribution is
already integrated. Read the exact contribution and preserve contributor
attribution, with PR reconciliation only after complete integrated acceptance
at verified sprint close. No additional F-ID or scope reduction is introduced.

Issues [288](https://github.com/tensorbee/rdocx/issues/288) and
[289](https://github.com/tensorbee/rdocx/issues/289), reported by `hadim` on
2026-10-08 at 16:18 UTC, are included in S90 acceptance assessment. Issue288
covers note removal, all header and footer replacement variants, shared story
references and building-block removal with complete comment thread and
companion preservation or atomic refusal. Compare every criterion with the
completed F-X184 implementation and add any missing implementation and tests
through the feature lifecycle before acceptance. Issue289 requires nested
block-control paragraph snapshots to carry accepted text and XML, including
comment anchor endpoints and other checked two-segment path consumers. Assess
it under F-X185's existing checked paragraph snapshot contract. Nested
paragraph enumeration is optional in that issue and does not change the
ordinary story inventory contract. Neither issue is closed from similarity to
an existing fix. Exact integrated evidence and full issue acceptance remain
required. The seven observed PR heads remain unchanged. Issue264 remains
excluded.

Issue292 is included in F-X188 with documented comment omission from
building-block transfers and physical glossary/main owner isolation.
The user corrected the exclusion to Issue281, which remains open for F-184.
Issue291 is included through F-X190, whose planned, verified multi-run field
snapshot correction runs in exclusive wave16 after F-X188 and before F-282 resumes. Issue264 remains
excluded. Assess updated stacked contributions selectively against complete
issue acceptance before merging or closing any record.

F-X188 owns exclusive wave15 after F-X187 and before F-282
resumes under its revised measured-subset contract. Its approved Issue288 contract completes the remaining replacement,
glossary and shared-reference ownership work. Issue289 is part of F-X185.

The user approved F-X189 for new Word0.16.0 and shared/PowerPoint0.14.0
release preparation. Existing unified versions are published at S88 and their
tags remain immutable. Run exclusive wave18 only after all included S90
implementation dependencies complete, including the revised F-282 measured
subset and F-X191. Prepare exact carriers, reviewed family notes and authenticated
contribution inventories over each complete previous-tag range, including
S89. Current local package/binding evidence precedes final integrated full
verification and review. The final reviewed and pushed sprint SHA must pass
the hosted build-only rehearsal before close. Publication follows close
through separate final exact-main-SHA approvals for each family, with shared
packages published before their new Word consumers. This is a user-approved
release preparation boundary, not completion of M24 or a1.0 decision.
Issue264 remains excluded and Issue281 remains open for F-184.

S90 contribution intake is frozen by the user at PR293. Do not search for
or add later issues or pull requests to this sprint. Finish the existing
measured bibliography, numbering reader and release preparation scope.
Remaining catalogue work and unfinished navigation move to S91.
Issues264 and281 remain open.

PR [293](https://github.com/tensorbee/rdocx/pull/293), contributed by Pedro
Assumpcao (`pedroassumpcao`) at head
`fd112a7ac3333709f62746b7065c5cc1b4be0eed`, adds F-X191. The user approved
its workflow records. Exclusive wave17 corrects the namespace-only numbering
reader flag while preserving declarations, foreign attributes, opaque children
and the separate authoring completeness contract. It has no formal dependency,
but shared source and Cargo ownership remain exclusive. Complete F-X191 before
F-X189 in wave18. Assess only the reader fix and focused tests, preserving
current archive policy rather than adopting contributor platform overrides.
Issue264 remains excluded and Issue281 stays open. PR disposition waits for
complete integrated acceptance at verified sprint close.

The user moved remaining bibliography work to F-X192 in S91 and explicitly
carried F-283 to S91 to publish completed S90 work. F-282 must satisfy its
revised measured-subset design, actual admission and atomic refusal contracts,
independent review and scoped gates. Its current 210-of-223 dense APA checkpoint
is not full catalogue parity. F-X189 depends on completed revised F-282 and
F-X191, not the carried S91 stories. Final integrated verification, sprint
review, package checks and exact-SHA publication approval remain required.

#### Sprint S91, Templates, controls, and forms

**Goal**: freeze the container-wide template grammar, create and bind content
controls, and expose legacy and modern forms plus safe mail-merge package
state through public APIs.

| F-ID | Title | Size |
|------|-------|------|
| F-X178 | Clearable direct run formatting setters | S |
| F-X192 | Complete the remaining Word bibliography catalogue | L |
| F-283 | Complete numbering-aware navigation fields | L |
| F-284 | Stable container-wide template grammar | L |
| F-285 | Content control creation and lifecycle | L |
| F-286 | Rich, repeating, and typed content controls | L |
| F-287 | Custom XML stores and data binding authoring | L |
| F-288 | Legacy form field creation | M |
| F-289 | Modern Word form authoring | L |
| F-290 | Mail-merge package and data-source authoring | M |

F-X178 is the explicit S90 carry for the user-excluded Issue264. Its next
eligible planning target is S91. It remains pending and requires a separate
scope decision before implementation, preserving the exclusion in S90.

F-X192 completes the bibliography catalogue deferred from S90 after the
reviewed F-282 native foundation. F-283 carries unfinished numbering-aware
navigation under its existing approved contract. Complete their actual
prerequisites before claim. These additions do not imply S90 delivered full
bibliography or navigation parity.

F-284 builds on S89 fragment transactions. F-285 establishes the control
lifecycle before F-286 and F-287 add typed values and bindings. F-288 uses the
S90 field builder. F-289 composes controls, bindings, ranges, and protection
into a complete form. F-290 may proceed alongside the form work after S90.
The sprint gate checks public-created templates and forms through save-reopen,
with explicit diagnostics for unavailable external mail-merge data.

#### Sprint S92, Collaboration authoring

**Goal**: create complete Word revisions, comments, permission ranges, and
comparison results, with caller-controlled identity and time instead of
ambient state.

| F-ID | Title | Size |
|------|-------|------|
| F-291 | Tracked insertion and deletion authoring | L |
| F-292 | Property revisions and move ranges | L |
| F-293 | Complete comments and modern comment metadata | L |
| F-294 | Permission ranges and protection integration | M |
| F-295 | Comparison output as complete revisions | L |
| F-296 | Collaboration identity and deterministic time policy | M |

F-291 establishes revision ownership. F-292 and F-293 then cover property,
move, and comment records across stories. F-294 uses the S91 form and
protection surface. F-295 composes complete comparison revisions, and F-296
closes identity and clock inputs. The sprint gate covers Word differential
results, accept and reject views, and byte-identical output in deterministic
mode.

#### Sprint S93, Drawings, diagrams, and embedded content

**Goal**: complete the visible and packaged Word object surface across every
valid insertion point without raw compatibility wrappers or executable payload
execution.

| F-ID | Title | Size |
|------|-------|------|
| F-297 | Complete Word drawing anchor and effect authoring | L |
| F-298 | Shapes, text boxes, groups, and connectors in Word | L |
| F-299 | AlternateContent, VML, and SVG compatibility authoring | L |
| F-300 | Charts at arbitrary Word insertion points | M |
| F-301 | SmartArt and diagram authoring in Word | L |
| F-302 | Embedded objects, icons, and alternative-format parts | M |
| F-303 | Drawing and embedded-content layout completion | L |

F-297 through F-302 build independent object families on the shared
relationship model. F-303 is their integrated deterministic layout and render
gate.

#### Sprint S94, Package extensibility and modern DOCX end gate

**Goal**: close package extension, accessibility, conformance, determinism,
resource, binding, documentation, and stability boundaries for modern DOCX
authoring.

| F-ID | Title | Size |
|------|-------|------|
| F-304 | Typed package extensibility facade | L |
| F-305 | Attached templates, web extensions, and task panes | L |
| F-306 | Executable compatibility attachment and signature rules | M |
| F-307 | Complete accessibility authoring and audit | L |
| F-308 | Fully modeled and losslessness diagnostics | L |
| F-309 | Strict, transitional, and repair-free conformance | L |
| F-310 | Determinism, resource limits, bindings, and stability gate | L |
| F-311 | Positioned paragraph frame placement | M |
| F-312 | Run visual effect render projection | L |

Package ownership precedes extension and executable-compatibility surfaces.
F-308 consumes every modeled family, F-309 validates its output, and F-310 is
the milestone-wide public API and operational gate. F-311 and F-312 close the
remaining positioned-paragraph and run-effect render checks. A 1.0 decision remains
separate from completing this sprint.

### M19, Advanced spreadsheets

#### Sprint S95, Spreadsheet decision, corpus and core model

**Goal**: decide whether a material Rust ecosystem gap still exists, then build
the ownership model only if that decision is affirmative.

| F-ID | Title | Size |
|------|-------|------|
| F-184 | Advanced spreadsheet go or no-go | S |
| F-204 | Spreadsheet corpus and compatibility matrix | M |
| F-185 | Workbook and worksheet model | L |

F-184 is a true go or no-go gate. It reassesses Calamine,
`rust_xlsxwriter`, `umya-spreadsheet`, `xls`, and any credible successor at S95,
then classifies each proposed feature as preserved, modeled and editable, or
executable. If the ecosystem provides the complete required lifecycle by then,
M19 is archived rather than implemented.

[Issue 281](https://github.com/tensorbee/rdocx/issues/281) supplies user demand
for a unified CLI/Python workflow, template filling, preservation-safe edits
and rendering. Include integration, installation and maintenance costs in
F-184, including reuse of a maintained engine behind a consistent facade.
S95 through S98 stage reading and loss-aware editing before calculation and
pivots. Rendering and distribution retain the S104 boundary. An earlier
distributable reader/editor requires a reviewed roadmap and release decision.

The reporter's [follow-up](https://github.com/tensorbee/rdocx/issues/281#issuecomment-6063901190)
defines a Google Sheets review workflow: read and change cell text, edit a
shared-formula column and preserve reviewer comments. Ordinary spreadsheet
reading is already covered in that workflow. The differentiators are one
CLI/Python shape and installation, Rust performance and safety, loss-aware
editing and later rendering without Excel. The requested first cut preserves
all unmodeled parts, including threaded comments and `commentsmeta*`, data
validation, conditional formatting and defined names. It reads values and
types, shared strings, formats and styles, merged ranges, formulas and cached
results. Edits include values, formulas and styles, rows and columns, sheets,
widths and heights, frozen panes and auto-filters, with atomic output. Shared
formula structure must survive. The decision must choose explicit stale-cache
invalidation and recalculate-on-load, or a separately bounded evaluator for
arithmetic, references, SUM, AVERAGE, MIN, MAX and IF. Chart authoring is not a
requested first-cut need. Rendering is a desired second step. These are inputs
to F-184, not S90 implementation or an approved earlier release boundary.

The [maintainer commitment](https://github.com/tensorbee/rdocx/issues/281#issuecomment-6064533994)
confirms preservation tests and the review workflow first, then reading and
styled edits with explicit stale-cache handling. A bounded evaluator and
rendering follow, with chart authoring deferred. Pivot refresh, Power Query
and scripting remain longer-term goals. F-184 must reconcile the scheduled
stories and any proposed reader/editor distribution boundary with that order
before approving implementation. S90 records this direction without claiming
that the conditional spreadsheet programme or an earlier release is approved.

#### Sprint S96, Styles, tables and structured references

**Goal**: model the indexed formatting and structured data semantics that
ordinary business workbooks rely on.

| F-ID | Title | Size |
|------|-------|------|
| F-186 | Shared strings, styles and number formats | L |
| F-189 | Formula parser | L |
| F-205 | Excel tables and structured references | L |

F-189 lands here because structured table references are part of the formula
grammar rather than a string convention. The sprint does not yet calculate
formulas.

#### Sprint S97, Advanced worksheet objects

**Goal**: cover the visible and interactive worksheet surface before the
streaming package boundary freezes it.

| F-ID | Title | Size |
|------|-------|------|
| F-206 | Advanced worksheet objects | L |

This includes comments, hyperlinks, rich text, drawings, grouping, panes,
sparklines, page breaks, and modern image cells. External content stays offline
unless an explicit bounded policy allows retrieval.

#### Sprint S98, Streaming read and write

**Goal**: prove that advanced workbooks remain bounded at the package boundary.

| F-ID | Title | Size |
|------|-------|------|
| F-187 | Reader | L |
| F-188 | Writer | L |

Both carry an asserted memory ceiling rather than a hoped-for one. A 100 MB
fixture is the gate, not a smoke test. Unsupported package parts and
relationships remain attached through unrelated typed edits.

#### Sprint S99, Calculation and sheet features

**Goal**: calculate ordinary and modern formulas, then expose the features that
depend on their results.

| F-ID | Title | Size |
|------|-------|------|
| F-190 | Calculation engine | L |
| F-191 | Charts in spreadsheets | M |
| F-192 | Conditional formatting and data validation | M |

F-190 is differential against the values Excel stored in the pinned corpus,
including dynamic arrays, spill ranges, and structured references. Unsupported
functions retain cached values with diagnostics. F-191 reuses `oxml-chart`
rather than creating a spreadsheet-only chart engine.

#### Sprint S100, Pivots and the Data Model boundary

**Goal**: move PivotTables from opaque preservation to typed local refresh and
define the boundary around proprietary analytical models.

| F-ID | Title | Size |
|------|-------|------|
| F-193 | Pivot cache and table model | L |
| F-207 | Pivot recalculation engine | L |
| F-208 | Slicers, pivot charts, and Data Model boundary | L |

Worksheet and table-backed pivots refresh locally and regenerate their cache
and visible cells. Slicers and pivot charts follow that refresh. OLAP, Power
Pivot, VertiPaq, and DAX state is preserved and inspectable but is not executed
under this milestone.

#### Sprint S101, Power Query language and package model

**Goal**: understand and evaluate M independently of external data access.

| F-ID | Title | Size |
|------|-------|------|
| F-209 | Power Query package and M language | L |

The package model preserves queries, connections, load destinations, and
refresh metadata. The language boundary covers the pure transformations needed
by the corpus before credentials, connectors, or network policy enter the
runtime.

#### Sprint S102, Power Query execution

**Goal**: refresh a bounded, useful connector set without weakening privacy or
offline determinism.

| F-ID | Title | Size |
|------|-------|------|
| F-210 | Power Query execution and connectors | L |

The first allowlist covers workbook tables, CSV, JSON, and HTTP. Credentials,
privacy levels, source combination, timeouts, byte limits, caching, and query
folding are explicit contracts. Proprietary and tenant-bound connectors remain
preserved with diagnostics.

#### Sprint S103, Office Scripts-compatible automation

**Goal**: automate the same workbook model through a versioned and sandboxed
TypeScript surface.

| F-ID | Title | Size |
|------|-------|------|
| F-211 | Office Scripts artifacts and ExcelScript surface | L |
| F-212 | Sandboxed Office Scripts runtime | L |

Scripts remain external artifacts associated with workbooks. The runtime does
not pretend to provide OneDrive, SharePoint, Power Automate, or Microsoft tenant
identity. It does provide bounded workbook, range, table, chart, pivot, and
query automation with atomic failure.

#### Sprint S104, Rendering, distribution and advanced end gate

**Goal**: close M19 as a headless advanced spreadsheet engine rather than a
file-format crate.

| F-ID | Title | Size |
|------|-------|------|
| F-194 | Sheet rendering | L |
| F-195 | rxlsx distribution | L |

The end gate runs one representative advanced workbook through read, edit,
formula and pivot recalculation, selected Power Query refresh, sandboxed script
automation, save, reopen, and deterministic PDF rendering. Every unavailable
execution surface remains preserved and diagnosed. Distribution follows the
shape M13 established only after that complete lifecycle passes.

## Future release boundaries

These are the planned points where a coherent crate family may publish. They
do not authorize a tag or publication by themselves. The sprint that reaches a
boundary adds an explicit release F-ID, selects the exact version from the
reviewed public API diff, and uses `/release` with its separate final approval
at the fully verified SHA.

| Boundary | Eligible publication | Why this point is coherent |
|---|---|---|
| End of S58, M20 | Stable `rdocx` family. Publish the incubating shared family first only if a shared crate version or stable dependency pin moved. | The Word corpus, shaping, pagination, and incremental layout gates have all settled, so the stable family can describe one measured fidelity boundary. |
| End of S64, M21 | Incubating `rpptx` family. Publish the stable family too only when the reviewed dependency diff requires new shared pins. | Collaboration, security, timing, media, SmartArt, ODP, handouts, HTML import, and PDF import form one complete presentation-depth boundary. |
| End of S69, M22 | Stable `rdocx` family at v0.13.0 through F-X078. Publish the incubating shared family first only if a shared crate version or stable dependency pin moved. | OfficeMath, fields, dynamic TOC, automation, comparison, embedded content, and modern package variants complete the planned Word-depth boundary. |
| End of S73, M23 | Stable `rdocx` family after the five-document public-API-only conformance gate passes. | This is the first boundary where the reference business documents can be authored from `Document::new()` without raw XML or a base template. |
| End of S94, M24 | Stable `rdocx` family after the modern Word authoring capability matrix is closed. Publish the incubating shared family first only if a shared crate version or stable dependency pin moved. | This boundary completes the planned modern DOCX authoring surface, lossless extensibility, accessibility, strict-package, determinism, binding, and stability gates. |
| End of S104, conditional M19 | The new `rxlsx` distribution family defined by F-195, plus only the existing families whose reviewed dependency pins moved. | F-195 is the first point where the conditional spreadsheet programme has a complete facade, CLI, WASM, Python, rendering, and advanced lifecycle gate. |

No intermediate sprint publishes merely because one subsystem compiles. A
security fix may still justify a separately planned patch release, but ordinary
feature work waits for the next boundary above. If F-184 archives M19, the S104
boundary disappears with the programme and no spreadsheet release namespace is
created.

## Cross-cutting

F-X001 through F-X004 are scheduled in S36 as the final cross-cutting v1
hardening wave. F-X007 and F-X008 handle the external Word contribution and
its stable-family release without rewriting the completed milestone history.
F-X013 through F-X016 carry the surviving half of the external PR 2 rendering
contribution, whose anchored-drawing placement was overtaken by the M7 anchor
work before it could land.
F-X031 carries the external branch-protection mutation deferred from F-X029 to
the Word fidelity boundary before the two depth milestones begin.

## Future ideas

These are discovery candidates, not scheduled work. They have no F-IDs, sizes,
dependency promises or delivery order. Each candidate needs a scope decision,
design plan and acceptance boundary before it can enter the backlog.

### Spreadsheet breadth

- Extend the advanced spreadsheet milestone with CSV, TSV, JSON and ODS output
  and broader structured-data import. Legacy binary XLS remains permanently
  excluded. XLSB requires its own future demand and architecture decision.
- Add worksheet copy and move operations, advanced paste options, named-range
  mutation and structured data import and export.
- Treat VBA and XLM as compatibility surfaces. Preserve, inventory, extract and
  optionally remove their projects and signatures without executing them.
- Consider DAX and VertiPaq execution only after the M19 Data Model preservation
  boundary has a licensed representative corpus and a separate security and
  performance decision.
- Add OLAP pivot refresh, proprietary Power Query connectors, custom connector
  SDK support, and broader query folding only against named consumers and
  reproducible service fixtures.

### Modern spreadsheet extensibility

- Preserve and inspect web-extension, task-pane and content-add-in parts,
  manifests, permissions and external resource locations. HTML, CSS and
  JavaScript assets should remain external web application content.
- Preserve namespaced custom-function formulas, cached results and add-in
  associations. The calculation engine should report unavailable custom
  functions instead of replacing their cached values.
- Preserve Python cell formulas, source, result previews and service metadata.
  Keep cloud execution outside the core library and expose the dependency as a
  diagnostic.
- Broaden the Office Scripts-compatible API only through versioned conformance
  fixtures. Microsoft-hosted script storage, tenant identity, Power Automate,
  and administrative policy remain integrations rather than local runtime
  claims.
- Add trusted custom-function hosts only after a separate capability and
  isolation design. Until then, retain namespaced formulas and cached results
  and report the unavailable function.

### Conversion and output

- Define a format-priority decision that weighs corpus prevalence, procurement
  needs, implementation risk and maintenance cost before adding another reader
  or writer.
- Consider XPS and PCL only when named users or a representative corpus justify
  them. Legacy binary Office formats remain excluded.
- Add a platform-neutral print-layout contract before considering operating
  system printer integration. PDF and image output should remain the portable
  default.
- Add per-page and per-shape rendering with explicit size, resolution, quality,
  compression, transparency and page-range controls across document families.

### Product surfaces

- Explore a self-hostable HTTP service for conversion, rendering, inspection
  and validation while keeping every operation available through local crates.
- Consider optional adapters for document summarisation, translation, grammar
  checking and presentation localisation. External model providers must remain
  outside the core document model.
- Add a browser viewer and lightweight editing surface only if the WASM APIs can
  remain the single implementation rather than creating a second document
  engine.

### Guardrails

Future breadth must preserve the properties that distinguish this workspace:

- Pure Rust implementations with no Office runtime or hidden conversion
  service.
- First-class native, CLI, Python and WASM entry points where the feature is
  technically available.
- Deterministic bundled-font rendering and declared output-baseline changes.
- Verbatim preservation of unmodelled XML and schema-correct child order.
- Shared OPC, DrawingML, chart, layout and rendering infrastructure rather than
  one implementation per document family.
- Offline operation by default, with network access isolated behind explicit
  adapters.
