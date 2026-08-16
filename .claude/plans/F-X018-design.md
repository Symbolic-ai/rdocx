# F-X018, Unknown enumerated values must not fail a document open

**Status**: completed
**Sprint**: S43
**Size**: M
**Depends on**: F-X014

## Problem

Nine value parsers in `crates/rdocx-oxml/src/shared.rs` and `styles.rs` return
`OxmlError::InvalidValue` for any string they do not enumerate:

`ST_Jc`, `ST_Underline`, `ST_Border`, `ST_TabJc`, `ST_TabLeader`,
`ST_SectionType`, `ST_PageOrientation`, `ST_HighlightColor` and `StyleType`.

Twelve call sites across six files propagate that with `?`, and those sites sit
inside paragraph, run, table, numbering, border and section property parsing.
The error therefore travels out of `CT_Document::from_xml` to `Document::open`,
so **a document using a spec-valid value the model has not yet listed does not
open at all**.

F-X014 proved this is not hypothetical. Three Arabic justification values,
`lowKashida`, `mediumKashida` and `highKashida`, were missing from `ST_Jc`, and
a document carrying any of them failed to open. That story fixed the three
values because a real contribution reached them. It did not fix the shape, and
eight more parsers have it.

## Spec reference

- `docs/hld/03-architecture.md`, "Domain conventions", for the prefix-tolerant
  read rule this generalises to attribute values. The plan first cited
  `04-opc-and-packaging.md`, which does not hold that rule.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" for the regression
  category.
- `docs/hld/14-development-backlog.md`, "F-X018, Unknown enumerated values
  should not fail a document open".

## Approach

The rule: **an unmodelled enumerated value is read as if the attribute were
absent.** In OOXML an absent attribute means the element's default, which is
usually inheritance from the style chain, so this is the reading that loses the
least. It also matches what the codebase already does for namespaces and for
unmodelled elements, where tolerance lives in the reader.

The parsers stay fallible. `from_str` continues to return `Result`, so a caller
that genuinely wants strictness keeps it, and the tolerance becomes an explicit
decision at each site rather than a property of the type. That is the same shape
as `ST_OnOff::from_str_or_default`, which already exists in this file.

Two kinds of call site, handled differently because the fields differ:

**Nine `Option`-typed fields.** `Some(X::from_str(&v)?)` becomes
`X::from_str(&v).ok()`. An unmodelled value leaves the field `None`, which is
exactly "not specified", so the style chain supplies the value. Sites:
`properties.rs` for `jc`, `underline` and `highlight`, `numbering.rs` for
`lvl_jc` twice, `document.rs` for `orientation` and `section_type`, and
`table.rs` for `jc` twice.

**Three non-`Option` locals.** `borders.rs` assigns into a local that already
holds a default: `val` for `ST_Border` and `ST_TabJc`, and `leader` for
`ST_TabLeader`. These become `unwrap_or(val)` and `.ok()` respectively, keeping
the default the local was initialised with.

Nothing else changes. No enum gains a variant, because guessing which
unmodelled values matter is what F-X014 already did for the case that was
reachable.

## Rejected alternatives

- **Make every `from_str` infallible, returning the default.** Loses the
  ability to detect a genuinely invalid document, and changes nine public
  signatures for a decision that belongs to the caller.
- **Add the missing variants to all nine enums.** Unbounded, and the next
  unmodelled value reintroduces the failure. The shape is the defect.
- **Fall back to a concrete variant such as `ST_Jc::Left`.** Wrong for an
  inherited property: it would override a style that specifies alignment,
  turning a missing value into an actively incorrect one. `None` inherits.
- **Log or collect the unmodelled values.** No consumer today, and the crate has
  no diagnostics channel at this layer.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `a_document_with_an_unmodelled_enumerated_value_still_opens` | A document carrying an unmodelled value for each of the nine enumerations opens, and every sibling property on the same element survives |
| regression | `an_unmodelled_value_leaves_the_property_unset` | The affected field is `None` rather than a guessed variant, so the style chain still supplies it |
| unit | `the_parsers_still_reject_an_unknown_value` | Each `from_str` still returns `Err` for an unknown string, so tolerance lives at the call site rather than in the type |

**Test gate**, from the backlog: the first regression, covering all nine
enumerations.

## HLD impact

- `docs/hld/03-architecture.md`. The prefix-tolerant read rule is stated for
  names and namespaces. It gains a neighbouring bullet extending the same
  tolerance to enumerated attribute values, including the round-trip cost.

## Risk routing

Matched row: **Any parser or serialiser**.

- Prefix-tolerant on read, fixed prefix on write. Unchanged: this touches
  attribute values, not names.
- Round-trip consequence, and it is real: a document carrying an unmodelled
  value now loses that value on save, because the field is `None` and the
  serialiser writes nothing. Before this story the document could not be opened
  at all, so nothing regresses, but the loss is worth stating rather than
  discovering. Recorded here and in AS_BUILT.

## Hash harness

**Expected unchanged.** No corpus document carries an unmodelled enumerated
value, since every corpus document opens today. A delta would mean a value the
model does enumerate started parsing differently.

## Implementation checklist

- [x] Record the pre-change harness state
- [x] Nine `Option`-typed sites to `.ok()`
- [x] Three `borders.rs` locals to keep their initialised default
- [x] Regression covering all nine enumerations
- [x] Confirm the parsers still reject an unknown value
- [x] Update `03-architecture.md`, which is where the rule actually lives
- [x] Full suite, harness, `/microscope F-X018 --working`, `/verify`

## Open questions

None.
