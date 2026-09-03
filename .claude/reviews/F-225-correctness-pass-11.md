# F-225, correctness, pass 11

**Reviewed**: current working tree implementation across 17 feature files,
9,154 inserted lines and 27 deleted lines from `597a27c`, with the final audit
limited to action and URI decoding paths
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None. Count: 0.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Pass 10 disposition

- D1 is closed. The active-content scanner rejects every dictionary containing
  `AA` immediately at `crates/rpptx/src/pdf.rs:2255`, before reading or
  traversing the additional-actions value. It therefore neither misparses an
  additional-actions dictionary as an action nor permits an action-shaped,
  scalar, indirect, or cyclic value. The regression covers those forms and
  then saves and reopens an ordinary URI annotation relationship without `AA`
  at `crates/rpptx/src/pdf.rs:7952`.
- D2 is closed. `decode_pdf_uri_string` selects UTF-16BE and UTF-16LE from their
  BOMs, rejects an odd payload before decoding, and collects `decode_utf16`
  fallibly so either unpaired surrogate returns `None` at
  `crates/rpptx/src/pdf.rs:4202`. The action validator converts that failure to
  a fatal malformed-URI error before scheme checking at
  `crates/rpptx/src/pdf.rs:4701`. The regression covers odd payloads and lone
  surrogates in both byte orders, empty and control-bearing values, a valid BE
  BMP path, and a valid LE surrogate-pair path through save and relationship
  reopen at `crates/rpptx/src/pdf.rs:8033`.
- Ordinary unsupported-font and missing-font text continues to use the
  pre-existing deterministic `decode_pdf_string` path at
  `crates/rpptx/src/pdf.rs:1311`. The new fallible decoder is called only by
  URI action validation, so the remediation does not change ordinary text
  decoding.

## Narrow action and URI audit

No remaining lossy conversion or context bypass was found. Catalog
`OpenAction`, annotation `A`, and `Next` inside a validated action retain their
strict contextual classification at `crates/rpptx/src/pdf.rs:2273`. Direct and
indirect action values retain the previously reviewed dictionary shape, raw
reference, cycle, depth, and work bounds. `strict_uri_action_dictionary`
continues to require a name `S` equal to `URI`, a string value, successful
fallible decoding, no whitespace or control characters, and a nonempty
reviewed scheme at `crates/rpptx/src/pdf.rs:4676`. Incidental `S`, `A`,
`OpenAction`, and `Next` keys outside their semantic contexts remain ordinary
data.

The focused `additional_actions_are_always_rejected_before_shape_interpretation`
and `uri_text_strings_reject_malformed_utf16_and_preserve_valid_unicode` tests
both pass. All findings from passes 1 through 10 remain closed except the two
pass-10 findings, which were specifically rechecked and are closed above. No
unrelated workspace code was reviewed in this pass.
