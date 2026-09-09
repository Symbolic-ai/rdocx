# oxml-cli-support

Give OOXML command-line tools consistent range parsing, output naming,
publication, and JSON contracts.

## Capabilities

- Positive one-based inclusive ranges with sorted, deduplicated results.
- Output extension replacement and collision checks.
- Adjacent temporary-file staging with cleanup and rollback after errors.
- Versioned JSON object envelopes shared by Word and Presentation CLIs.

## Use it when

Use this crate when building a repository DOCX or PPTX CLI that must follow the
same output-path and structured-output conventions. Application code should
use `rdocx` or `rpptx` instead.

## Relationship

This format-neutral crate is consumed by `rdocx-cli` and `rpptx-cli` and does
not depend on either document model. It does not provide argument parsing,
document I/O, or a user-facing CLI. Filesystem rollback is best effort.

## Example

```rust,no_run
let slides = oxml_cli_support::parse_range("2,4-6")?;
assert_eq!(slides, vec![2, 4, 5, 6]);
# Ok::<(), oxml_cli_support::Error>(())
```
