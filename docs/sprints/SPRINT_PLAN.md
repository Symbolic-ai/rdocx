# Sprint Plan

Sprint-by-sprint roadmap for the oxml extraction and the rpptx build. Sprints
are dependency and review boundaries, not fixed two-week containers. Sprint
clocks start at the first `/start-feature` of that sprint, not at a fixed
calendar date.

37 numbered sprints plus two deferred cutover sprints across 13 milestones,
roughly 390 developer-days. The sizing rationale and compression options are in
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

## Cross-cutting

F-X001 through F-X004 are scheduled in S36 as the final cross-cutting v1
hardening wave.
