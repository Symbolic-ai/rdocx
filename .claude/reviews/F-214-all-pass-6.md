# F-214, all, pass 6

**Reviewed**: complete current worker diff against the F-214 base, 14
implementation files, 4,898 additions and 30 deletions, including both approved
untracked timeline modules, the approved plan and cited HLD sections, progress
notes, pass 5 review, and the two explicit OXML API approvals
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, oracle case names are not bound to their approved timeline coordinates

`crates/rpptx/tests/integration.rs:5527`

`crates/rpptx/tests/integration.rs:5852`

`crates/rpptx/tests/integration.rs:5947`

The generator defines each approved case as a five-field tuple containing its
case name, slide index, slide-local timestamp, click count, and movie timestamp.
The differential gate instead accepts the slide index, slide-local timestamp,
and click count from the fetched manifest, then validates only the final set of
case names. It never compares each manifest row with the generator's approved
tuple. A self-consistent manifest can therefore relabel a different timestamp,
change a click case to click count zero, or reuse a convenient slide state under
all 17 required names while retaining the exact approved source and valid
artifact hashes. This can pass without exercising the exact timestamps and
click boundaries required by the plan's differential contract.

## Smells

None.

## Nitpicks

None.

## Pass 5 closure

- Pass 5 D1 is closed. Group clips are formed in each targeting group's
  animated page space and inverse-mapped through the final descendant. The
  focused regression combines parent wipe, nested-group motion, and child spin
  and confirms the page-space clip remains unchanged.
- Pass 5 D2 is closed for exact source-deck binding. The gate compares source
  bytes with the deterministic generator before opening the presentation, and
  the focused substitution regression passes. D1 is the adjacent unbound
  manifest-row contract.
- Pass 5 D3 is closed. Unsupported effects and invalid directional parameters
  are validated before cut and terminal-progress returns. The focused
  regression covers intermediate and exact-end progress.
- Pass 5 D4 is closed. A supplied outgoing index that resolves outside the deck
  returns `UnknownSlideIndex`, and the focused facade regression passes.

## Approved OXML boundaries

The OXML public diff remains exactly the two user-approved methods:

- `ShapeTreeChild::non_visual_name(&self) -> Option<String>`
- `CT_Timing::condition_has_explicit_target(node_id, end_condition, index) -> Option<bool>`

All concrete per-shape name helpers remain crate-private. Parsed, constructed,
cloned, compatibility-wrapped chart, and existing name-mutation paths keep the
private decoded-name caches coherent. The condition-presence cache remains
namespace-aware, follows the typed start and end condition indexes, is cloned
with the model, and is rebuilt through ordinary parsing after duration
mutation. Timeline evaluation consumes both caches without serializing or
reparsing XML.

## Focused evidence

- The parent-group clip isolation regression passed with nested-group motion
  and direct child spin.
- The invalid transition direction regression passed at progress 0.5 and 1,
  including an unsupported transition effect at progress 1.
- The exact source binding substitution regression passed.
- The supplied outgoing-index bounds regression passed.
- The F-213 condition projection and approved target-presence regression
  passed, including duration mutation cache rebuilding.
- The compatibility-wrapped chart id and approved name identity regression
  passed.
- The ordinary static rendering isolation regression passed.
- The source-built timeline model and evaluator regression passed.
- Optional oracle mode returned only because `manifest.tsv` is absent.
  Required mode failed closed at that exact missing path.
- `git diff --check` passed. No broad test command ran.

## External evidence blockers

The PowerPoint oracle directory still has no `manifest.tsv` and no PNG frames.
The GUI automation did not produce the required artifact set. The required
pinned PowerPoint differential has not run and is not passed. The full 50-deck
SSIM rider was interrupted and remains incomplete and unclaimed. These external
evidence gaps are separate from D1.

## Explicit zero categories

No correctness defect outside D1 was found. No production panic path, schema
child-order defect, namespace binding defect, retained raw-XML defect, reverse
dependency, new dependency, unapproved trait, generic, feature flag, crate,
backend animation variant, runtime oracle dependency, or binary fixture defect
was found. No public OXML surface beyond the two exact approvals was added. No
name-cache or condition-presence-cache parse, construction, clone, mutation, or
evaluation-path defect was found. No ordinary static-path execution or
identity-cache dependency was found. No slide versus layout or master
target-scope leak, group-lineage loss, target mapping defect, or adjacent group
composition defect was found. No additional timing start, end, click, hold,
remove, transition, morph matching, geometry interpolation, or endpoint defect
was found. Morph matching remains limited to explicit `!!` names and compatible
resolved geometry, with finite crossfade fallbacks. The required oracle mode is
fail-closed for absent artifacts, but D1 and the external blockers mean the
differential gate is not passed.
