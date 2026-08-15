# oxml-cli-support

Shared path, range, and JSON-envelope helpers for the repository's OOXML command-line tools.

## Use it when

Use this crate when building a DOCX or PPTX CLI that must follow the same output-path and structured-output conventions. Application code should use `rdocx` or `rpptx` instead.

## Relationship

This format-neutral crate is consumed by `rdocx-cli` and `rpptx-cli` and does not depend on either document model.

## Example

```rust,no_run
let slides = oxml_cli_support::parse_range("2,4-6")?;
assert_eq!(slides, vec![2, 4, 5, 6]);
# Ok::<(), oxml_cli_support::Error>(())
```
