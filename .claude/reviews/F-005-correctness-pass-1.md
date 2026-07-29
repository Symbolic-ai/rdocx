# F-005, correctness, pass 1

**Reviewed**: F-005 working diff, 3 files with 103 additions and 7 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, The maximum parsed suffix overflows the next allocation

`crates/rdocx/src/document.rs:147`
`crates/rdocx/src/document.rs:539`

The new scan accepts `usize::MAX` as a valid positive image suffix. The next
call to `add_picture` increments that value in `next_image_number()`. This
panics for overflow in checked builds and can wrap to `image0` otherwise,
breaking the unique monotonic allocation contract and potentially overwriting
an existing `image0` part with the same extension. A package containing
`/word/media/image{usize::MAX}.png` triggers the failure, and the regression
tests do not exercise the upper bound.

## Smells

None.

## Nitpicks

None.

## Not found

No prefix-selection defect was found. The exact `/word/media/image` prefix is
required, so presentation media and lookalike stems with no leading decimal
digits do not affect the maximum.

No extension-specific defect was found. The scan stops after the consecutive
decimal index and therefore shares one monotonic counter across PNG, JPEG and
other image extensions as required.

No defect was found for zero or ordinary malformed names. Zero is rejected,
and absent, signed and nonnumeric leading suffixes are ignored.

No gap was found in the two required sparse collision cases. The named gate
constructs both `image1` plus `image5`, and `image1`, `image2` plus `image4`,
then proves the added bytes occupy `image6` and `image5` respectively. Against
the old count-based code, each expected part is absent, so the gate fails for
the intended reason.
