# F-X014, correctness, pass 1

**Reviewed**: the uncommitted working tree. One line of implementation plus a
comment in `crates/rdocx-oxml/src/shared.rs`, and a new `mod tests` in the same
file holding three tests.
**Verdict**: 0 defects, 0 smells, 1 nitpick

## Defects

None.

The change adds three spellings to an existing match arm. It cannot alter the
handling of any value that parsed before, because the arm it joins already
returned `ST_Jc::Both`, and the three values previously reached only the error
arm.

## Smells

None.

The round-trip normalisation, where a kashida value is written back as `both`,
is a real loss of fidelity. It is not recorded as a smell because it is a stated
decision in the design plan with its reasoning, not an oversight: preserving the
exact spelling would need a variant that behaves identically to `Both` at every
site that matches on `ST_Jc`, which adds cases a reader must consider for no
behavioural difference.

## Nitpicks

- `crates/rdocx-oxml/src/shared.rs:19`, `from_str` shadows the
  `std::str::FromStr` name without implementing the trait, so `"x".parse()` does
  not work and a reader may expect it to. Pre-existing across every value type
  in this file and consistent within it, so changing it here alone would make
  the file less uniform, not more.

## Not found

Checked and produced nothing:

- **correctness**. The three added spellings are exactly those `ST_Jc` defines
  and the model lacked. `distribute` was deliberately not chosen, since it
  spreads the last line and kashida justification does not.
- **panics**. No new panicking construct. The function still returns `Result`.
- **ooxml**. An attribute value, not an element name or namespace, so the
  prefix-tolerance rules are untouched. `to_str` is unchanged and still emits a
  schema-valid `both`.
- **structure**. No new type, variant, trait or file. The one new `mod tests`
  joins the file it tests rather than creating another link target, per the
  build note in `CLAUDE.md`.
- **contract**. Matches the plan exactly.
- **tests**. All three fail against the unwidened parser, including the
  document-level regression, which is the one that proves the real symptom: a
  document carrying a kashida value previously failed to open at all. The
  unknown-value test pins that the check was widened rather than removed.
