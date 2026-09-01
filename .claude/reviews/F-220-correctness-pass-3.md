# F-220, correctness, pass 3

**Reviewed**: claim-base `3d87d827351508b38ffb22103f26351c3f18c0ca` through exact source head `0e027955fa9c825a703a213b8706047ac9611489`, 6 files and 3,226 changed lines (3,171 additions, 55 deletions), including the complete pass-2 D1 to D6 remediation delta, the approved revised plan, cited HLD sections, progress record, prior reviews, and full tracked and untracked state
**Verdict**: 7 defects, 0 smells, 0 nitpicks. Not ready because the defects remain and the external PowerPoint completion evidence is absent.

## Defects

### D1, family inference does not implement a declared identity and algorithm mapping
`crates/rpptx-oxml/src/diagram.rs:1857`

The parser concatenates the unique id, title, categories, and every algorithm,
then chooses the first family whose loose substring occurs anywhere. This is
neither an exact identity mapping nor an identity and algorithm pair. It also
checks `cycle` before `relationship`. The source-built relationship oracle case
uses identity `urn:f220:relationship` with algorithm `cycle` at
`crates/rpptx/tests/integration.rs:594`, so it is classified as Cycle and never
exercises the Relationship branch. An unrelated identity containing `list` or
`process` with an allowed linear algorithm is likewise accepted as a supported
List. The six-family gate can therefore report six manifest rows while testing
only five renderer families, and undeclared identities can produce plausible
output instead of the required fallback.

### D2, algorithm parameters are neither validated as a supported set nor honored exactly
`crates/rpptx/src/diagram.rs:298`

Validation checks only the algorithm type. Unknown parameters and extra
algorithm attributes remain in the flattened record and are silently ignored
by the family functions. Explicit invalid matrix columns are discarded by
`parse::<usize>().ok()` and replaced with a guessed square grid at
`crates/rpptx/src/diagram.rs:693`. List direction has a second wrong-output
path at `crates/rpptx/src/diagram.rs:483`: a wide frame overrides an explicit
`fromT` vertical direction, while `fromR` and `fromB` never reverse node order.
These supported-looking but unimplemented parameter variants must either be
applied according to their declared semantics or fail closed.

### D3, presentation topology discards the typed connection ordinals
`crates/rpptx/src/diagram.rs:429`

The graph projection reduces every supported connection to only source and
destination ids. It never reads `source_order` or `destination_order`, even
though F-219 exposes both typed values. Node placement then follows point-list
document order, and hierarchy siblings follow that same order. Two equivalent
graphs with reordered points or connections but unchanged `srcOrd` and
`destOrd` therefore render different node order and connector order. This
violates the approved exact normalized order and connector-endpoint contract.

### D4, duplicate and cyclic presentation topology is accepted outside hierarchy
`crates/rpptx/src/diagram.rs:328`

Only connection model ids are checked for uniqueness. Two differently named
`presParOf` or mapped `parOf` connections with the same endpoints are both
retained, and a parent cycle is rejected only when the selected family calls
`hierarchy_layout`. List, cycle, relationship, matrix, and pyramid accept the
same cyclic topology and emit duplicate or cyclic connectors. The revised plan
requires duplicate and cyclic presentation ownership and topology to fail
closed before family placement.

### D5, the published render projection exceeds the approved API shape
`crates/rpptx-oxml/src/diagram.rs:63`

The approved API block gives `DiagramRenderProjection` exactly the public
`algorithms`, `constraints`, and `rules` fields. The implementation additionally
publishes `selectors`, `style_labels`, and `unsupported_logic`. `#[doc(hidden)]`
does not make those fields private, so this is additive published pre-1.0 API
beyond the reviewed projection contract. The plan prose now requires direct
algorithm-owner style selection and fail-closed logic, but its API section was
not revised to authorize these three public carriers.

### D6, the producing-scope integration test omits the animation path
`crates/rpptx/tests/integration.rs:171`

The approved integration case requires static, timeline, media, and animation
paths to agree after a scoped SmartArt edit. The test exercises static,
timeline, and media rendering, then ends without calling the animation export.
No other SmartArt test covers that path. A regression that drops transient
SmartArt expansion only during animation export would leave the named contract
green.

### D7, the six-family unit test does not prove its exact geometry contract
`crates/rpptx/src/diagram.rs:1279`

For every family the test asserts only three rectangles and containment. It
adds one coarse assertion each for hierarchy, matrix, and pyramid, but checks
no exact normalized bounds, list orientation or reversal, cycle angle sequence,
relationship cardinality branch, node order, connector endpoints, finite group
transform, or clipping. The approved unit case requires exact bounds, order,
connector endpoints, finite transforms, and clipping for all six families.
This gap is why the wrong family selection and parameter behavior above remain
green even before the unavailable external oracle is considered.

## Smells

None.

## Nitpicks

None.

## External completion evidence

The six-family PowerPoint oracle remains absent. Ordinary mode intentionally
skips with the exact missing manifest path at
`crates/rpptx/tests/integration.rs:231`. Required mode fails closed at
`crates/rpptx/tests/integration.rs:235` before comparing any family. The
progress record accurately states that the trusted manifest and six captured
PowerPoint artifacts do not exist at
`.claude/scratch/F-220-progress.md:98`. Under the differential-testing policy,
this is an external feature-completion blocker. It is not counted as a defect,
smell, or nitpick, and this review does not infer PowerPoint fidelity from the
ordinary-mode skip.

## Not found

- **Pass-2 D1, cached drawing independence**: Cached drawing resolution is no
  longer stored in `DiagramResources` or consulted by `render_group`. The
  malformed optional-cache regression passed.
- **Pass-2 D2, selectors and multiple algorithms**: Selectors, conditional
  logic, nested layout nodes, and multiple algorithms fail closed. The focused
  private-module and OXML projection regressions passed.
- **Pass-2 D3, missing presentation ownership**: Data-only models and
  presentation nodes without exactly one data owner fail closed.
- **Pass-2 D4 and D5, schema owners and targeted operations**: The narrowed
  program requires one style label on the direct algorithm owner, rejects
  rules, targeted or spacing constraints, and constraint owners without a
  direct algorithm. No remaining point-index label assignment or broad
  constraint application was found inside that narrowed subset.
- **Pass-2 D6, frame transform**: The transient group copies offset, extent,
  child coordinate space, rotation, and both flips. The focused regression
  passed.
- **OOXML namespace and raw preservation**: Alias prefixes, fixed-prefix
  shadows, foreign and extension lookalikes, direct schema positions, ordered
  colour transforms, bounded projection traversal, and byte-exact `to_xml`
  remain covered. The three focused `rpptx-oxml` projection tests passed.
- **Shared text, style, and colour engines**: Complete `CT_TextBody` values and
  ordinary PresentationML style references enter the existing resolver. No
  second text shaper or colour-transform engine was found.
- **Producing scope and static assembly**: Slide, layout, and master resource
  maps remain separate, and the same transient expansion precedes the shared
  resolver. No relationship-id alias across those scopes was found.
- **Structure and panics**: The single private `rpptx` diagram module is
  approved. No new trait, generic, feature, crate, dependency, dynamic
  dispatch, integration binary, or reachable untrusted-input panic was found.
- **Validation evidence**: `cargo check -p rpptx --all-targets` passed with the
  three recorded pre-existing F-221 warnings. `cargo test -p rpptx --lib
  diagram::tests` passed 8 tests. The focused `rpptx-oxml` projection run passed
  3 tests. `cargo test -p rpptx --test integration smartart_ -- --nocapture`
  passed 18 tests with 1 ignored generator and the documented missing-oracle
  skip. Required-corpus mode failed at the absent manifest as required.
  `git diff --check` passed.
