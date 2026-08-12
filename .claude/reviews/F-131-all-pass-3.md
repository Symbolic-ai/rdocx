# F-131, all, pass 3

**Reviewed**: working implementation diff from claim base `3db056b`, 17
implementation, plan, test, and HLD files with 2,775 changed or newly added
lines, excluding the two earlier review artifacts
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Prior finding re-evaluation

- Pass 1 D1 is resolved at `crates/rdocx-py/src/table.rs:613`. Cell text
  replacement completes its structural write before the single revision bump
  at line 627. The regression at
  `crates/rdocx-py/tests/test_formatting_tables.py:181` holds both a nested Run
  and Font across the exact revision 2 to revision 3 transition.
- Pass 1 D2 is resolved at `crates/rdocx/src/run.rs:12`. The published
  exhaustive `UnderlineStyle` retains its established eight variants. The two
  binding-only codes use the additive checked integer accessor at
  `crates/rdocx/src/run.rs:168`, and the exhaustive compatibility match at
  `crates/rdocx/tests/integration_test.rs:280` protects that source contract.
- Pass 1 D3 is resolved at `crates/rdocx/src/paragraph.rs:275`. The established
  first-line-indent helper again writes its signed value directly. The new
  clearing and hanging behavior has a separate entry point at
  `crates/rdocx/src/paragraph.rs:282`, while the legacy negative serialization
  gate remains at `crates/rdocx/tests/integration_test.rs:296`.
- Pass 1 D4 is resolved at
  `crates/rdocx-py/tests/test_formatting_tables.py:47`. All four paragraph
  booleans and signed first-line indentation are set, cleared with `None`,
  reopened, and asserted as `None` at lines 76 through 80.
- The pass 1 rustdoc nitpick is resolved at
  `crates/rdocx/src/paragraph.rs:743`.
- Pass 2 D1 is resolved at `.claude/plans/F-131-design.md:98`. All six
  implementation checklist items are checked, with the plan status correctly
  remaining approved at line 3 until completion preparation.
- Pass 2 D2 is resolved at `crates/rdocx-py/src/table.rs:68`. The setter accepts
  only the bounded Python values 0, 1, and 2, while the total getter maps the
  Rust-only `Justify` value to `None` at line 82. The regression at
  `crates/rdocx-py/tests/test_formatting_tables.py:249` proves value 3 is
  rejected and a reopened `w:jc="both"` reads as `None`.
- Pass 2 D3 is resolved at `crates/rdocx/tests/integration_test.rs:219`. The
  regression observes code 9 immediately after code 5 is rejected, before code
  10 is assigned, so a rejected code cannot mutate or clear the value.
- Pass 2 D4 is resolved at `crates/rdocx-py/src/formatting.rs:223`. The colour
  getter maps case-insensitive OOXML `auto` to `None`, and the reopened-package
  regression is at `crates/rdocx-py/tests/test_formatting_tables.py:268`.
- Pass 2 D5 is resolved at `crates/rdocx-py/src/paragraph.rs:79`. Validation now
  selects the recovery hint from the resolved location and emits the exact
  table, row, cell, and paragraph path for nested content at lines 83 through
  90. The exact-path regression is at
  `crates/rdocx-py/tests/test_formatting_tables.py:162`.

## Not found

- Revision and mutation defects: document, paragraph, table, and cell
  structural operations bump only after successful mutation. Failed lookups
  and rejected enum values do not bump, while value-only formatting writes
  preserve the captured revision. The cell replacement boundary is explicit at
  `crates/rdocx-py/src/table.rs:614`.
- Tri-state and clearing defects: the facade retains `Option<bool>` for direct
  run and paragraph formatting, Python exposes `None`, false, and true without
  collapse, and the established bool helpers retain their prior semantics. The
  representative facade entries are at `crates/rdocx/src/run.rs:106` and
  `crates/rdocx/src/paragraph.rs:326`.
- Path and stale-handle defects: Body, Row, Cell, Para, and Run shapes resolve
  through total facade lookups for both document paragraphs and nested cell
  paragraphs. Font and ParagraphFormat retain only a document reference and
  content path. The nested resolver rejects incomplete shapes at
  `crates/rdocx-py/src/paragraph.rs:22`, and the exact nested recovery guidance
  is covered as noted above.
- Lazy collection defects: table, row, cell, paragraph, and run collections
  retain document and path state rather than wrapper vectors. Integer and
  negative indexing, forward and reverse slices, and iterators validate current
  lengths. Python lists are created only as slice results, for example at
  `crates/rdocx-py/src/table.rs:143`.
- Python value and table defects: Length and RGBColor values use the F-132
  classes, enum getters return the bounded public IntEnums, and checked setters
  reject unsupported values before mutable document borrows. Table style,
  dimensions, text, alignment, vertical alignment, and nested formatting use
  facade accessors and round-trip coverage beginning at
  `crates/rdocx-py/tests/test_formatting_tables.py:198`.
- PyO3 and cache defects: no unsafe block, escaped Rust borrow, nested mutable
  pyclass borrow, or GIL-release scope was found. Read paths use immutable
  facade accessors and do not clear layout caches. The cell text setter scopes
  the table borrow before its revision bump at
  `crates/rdocx-py/src/table.rs:616`.
- Public API defects: the legacy exhaustive enum and first-line-indent behavior
  are protected by compile and serialization regressions. New facade methods
  are additive, and the diff adds no trait, generic parameter, feature flag,
  dynamic dispatch, or direct `rdocx-oxml` binding dependency.
- OOXML defects: no parser, namespace, whitespace, schema child order, or raw
  subtree preservation code changed. New facade setters update the established
  property models.
- HLD and scope defects: exactly `docs/hld/03-architecture.md`,
  `docs/hld/10-bindings-spec.md`, and `docs/hld/14-development-backlog.md`
  changed, matching `.claude/plans/F-131-design.md:73`. The HLD describes
  current facade ownership, bounded formatting and table inventory, and the
  real F-132 dependency. No F-133 rendering or GIL-release work entered the
  diff.
- Checklist and risk-evidence defects: the approved new formatting module,
  table module, and Python test file match the approval recorded at
  `.claude/plans/F-131-design.md:82`. Progress records green PyO3, WASM,
  conversion, publication dry-run, archive-size, prose, diff, skill-drift, and
  unchanged hash riders at `.claude/scratch/F-131-progress.md:116`.
- Artifact defects: no extension module, wheel, Python cache, or compiled
  Python file is added to the working diff. The existing repository `target`
  output is ignored build state, not an F-131 artifact.
- Focused check failures: independent isolated-target
  `cargo check -p rdocx-py --all-targets` passed. Independent
  `cargo test -p rdocx` passed 63 unit, 76 integration, 17 regression, and one
  doctest. The cleaned editable Python extension was not regenerated under the
  review-only constraint. The worker's post-pass-2 record reports all 11
  focused and 22 complete installed-wheel tests green at
  `.claude/scratch/F-131-progress.md:121`.
