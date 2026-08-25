# F-198, Hyphenation

**Status**: approved
**Sprint**: S58
**Size**: L
**Depends on**: F-197

## Problem

The shared line breaker uses only Unicode line-break opportunities at
`crates/oxml-layout/src/line.rs:475` and greedily moves a whole breakable
segment when it overflows at `crates/oxml-layout/src/line.rs:333`. It does not
offer language-specific Liang break points or emit a conditional hyphen.

Word already parses and cascades `w:suppressAutoHyphens` in
`crates/rdocx-oxml/src/properties.rs:124`, but the direct paragraph merge used
by layout omits it at `crates/rdocx-layout/src/engine.rs:5260`. `w:lang` remains
an opaque schema slot at `crates/rdocx-oxml/src/properties.rs:735`, and parsed
settings do not project `w:autoHyphenation` at
`crates/rdocx-oxml/src/settings.rs:137`. OOXML defaults omitted
`w:autoHyphenation` to off. None of the generated hash samples enables it or
declares a language, so standards-correct enablement produces no current hash
delta even though the sprint definition expects F-198 to move the baseline.

## Spec reference

- `docs/hld/03-architecture.md`, shared line-breaking ownership and Word layout input.
- `docs/hld/08-rendering-spec.md`, "Exact text shaping" and source spans.
- `docs/hld/10-bindings-spec.md`, published low-level Rust surfaces.
- `docs/hld/12-testing-strategy.md`, deterministic golden and Word-fidelity evidence.
- `docs/hld/14-development-backlog.md`, "F-198, Hyphenation".
- `docs/hld/15-build-and-toolchain.md`, deterministic fonts, dependency policy, and package gates.

## Approach

Project read-only `w:autoHyphenation` from `CT_Settings`, with omission resolving
to false while the complete source XML remains preserved. Model all three
`w:lang` attributes, `val`, `eastAsia`, and `bidi`, together so F-198 and F-199
do not compete over one schema slot. Preserve namespace tolerance, fixed `w:`
output, schema order, unknown attributes, and unrelated raw children.

Pass document automatic-hyphenation state into `LayoutInput`. Map supported BCP
47 primary-language tags to deterministic embedded Liang patterns. Use a
language-aware additive `InlineItem` variant so ordinary `InlineItem::Text`
remains unhyphenated and existing `TextSegment` construction is unchanged.

Extend the private breakable-segment representation with optional generated
break suffixes. When overflow chooses a Liang opportunity, shape the hyphen in
the preceding run's font and formatting, emit it with `source: None`, and leave
the original word fragments' Unicode-scalar source ranges exact and contiguous.
Keep existing UAX 14 behavior for disabled or unsupported languages,
suppressed paragraphs, generated fields without an attributable language,
explicit breaks, and unwrapped text.

Use `hypher` 0.1.7 in `oxml-layout` only, with default features disabled and
the English, French, German, and Spanish patterns selected explicitly. Map
regional BCP 47 tags to their primary language pattern. Do not add a new
feature flag or source module.

Add the smallest authoring surface needed to enable document automatic
hyphenation and assign run language. Update a page-one paragraph in the
`feature_showcase` generator to enable automatic hyphenation, declare `en-US`,
and contain a reviewed word that selects a visible Liang break. This gives the
required isolated harness movement without changing the OOXML default.

## Rejected alternatives

- Enable hyphenation for every recognized-language run. That contradicts the
  OOXML default when `w:autoHyphenation` is absent.
- Infer English when language metadata is absent. That makes output depend on
  an undocumented fallback.
- Put Word settings or language types in `oxml-layout`. That reverses the
  format-neutral dependency boundary.
- Add the generated hyphen to a source span. It has no source character and
  would corrupt selection and extraction.
- Add a new feature flag or source module. No current second configuration or
  ownership boundary justifies either one.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | Liang opportunity and farthest-fitting candidate tests in `oxml-layout/src/line.rs` | Known words expose expected candidates, fitting words remain unchanged, and unsupported or disabled languages do not hyphenate |
| unit | generated hyphen source-span regression | The inserted hyphen is separately shaped with `source: None`, while original fragments retain exact scalar ranges and formatting |
| unit and round-trip | settings and run-language parser tests | Aliased attributes, defaults, foreign same-local-name rejection, schema position, and byte preservation remain exact |
| integration | existing `rdocx-layout` engine test binary | Document enablement, paragraph suppression, run language inheritance, mixed-language runs, tables, notes, fields, and drawing reflow |
| golden | source-built deterministic hyphenated DOCX | Reviewed line placement and page PNG evidence match the chosen oracle |
| differential | `python3 scripts/docx_ssim_harness.py --check` | The complete pinned corpus reports classified before-and-after page evidence |
| regression | `python3 scripts/hash_harness.py --check` | The approved isolated delta is exact, or all 49 entries remain unchanged if the standards-correct corpus decision wins |

The backlog test gate is **golden**: a hyphenated document matches the oracle's
line breaks within the recorded tolerance, and the harness delta is declared.

## HLD impact

- `docs/hld/03-architecture.md`, shared line breaking and Word settings projection.
- `docs/hld/08-rendering-spec.md`, Liang opportunities, conditional hyphens, shaping, and source spans.
- `docs/hld/10-bindings-spec.md`, low-level Rust source compatibility of layout input and the shared inline variant.
- `docs/hld/12-testing-strategy.md`, deterministic golden and pinned Word-fidelity evidence.
- `docs/hld/15-build-and-toolchain.md`, pattern dependency, licensing, deterministic embedding, and package verification.

## Risk routing

- Layout, pagination, line breaking, and shaping. Read
  `docs/hld/08-rendering-spec.md`, use deterministic bundled fonts, and record
  any baseline deliberately.
- Parser and serializer. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`, then prove namespace tolerance,
  schema-order output, and byte preservation of unmodelled content.
- Crate dependency graph. Read `docs/hld/03-architecture.md` and keep the
  pattern dependency inside `oxml-layout`, with no reverse workspace edge.
- Public API of published crates. Read `docs/hld/10-bindings-spec.md`, state the
  low-level pre-1.0 Rust source impact, run `cargo publish --workspace --dry-run`,
  and enforce each `.crate` size limit.
- External oracle comparison. Follow `.claude/skills/differential-testing.md`,
  keep LibreOffice Writer and Poppler pinned, and explain every moved page.

## Hash harness

Expected isolated delta in the `feature_showcase` document XML, deterministic
page-one PNG, and affected PDF fingerprints. The sample will explicitly enable
automatic hyphenation and declare `en-US`. Every other sample remains
byte-identical. Record exact digests only after the focused golden and pinned
LibreOffice comparison explain the movement.

## Implementation checklist

- [ ] Project `w:autoHyphenation` without losing preserved settings XML.
- [ ] Model and round-trip the complete `w:lang` attribute set.
- [ ] Carry automatic hyphenation and effective run language into layout.
- [ ] Add deterministic supported-language mapping and Liang candidates.
- [ ] Emit only selected conditional hyphens with no source span.
- [ ] Preserve suppression, unsupported-language, and no-wrap behavior.
- [ ] Add unit, round-trip, integration, golden, differential, and hash evidence.
- [ ] Run every risk rider and update exactly the listed HLD files.

## Open questions

None. Automatic hyphenation remains off when omitted. `feature_showcase`
explicitly enables English hyphenation for the isolated delta, regional tags
map to the four selected primary-language patterns, and the repository-pinned
LibreOffice oracle is the reviewed acceptance oracle.
