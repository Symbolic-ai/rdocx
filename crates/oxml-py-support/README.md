# oxml-py-support

Internal Rust support for content paths, revision tracking, units, and errors shared by the Python bindings.

## Use it when

Use this crate only while implementing `rdocx-py` or `rpptx-py`. It is not published to crates.io and is not a Python package by itself.

## Relationship

The two PyO3 binding crates use it to keep Python-visible navigation and revision semantics aligned.

## Example

```rust,no_run
use oxml_py_support::{emu_from_inches, inches_from_emu};

let width = emu_from_inches(8.5);
assert_eq!(width, 7_772_400);
assert_eq!(inches_from_emu(width), 8.5);
```

Repository binding crates use `oxml-py-support = { path = "../oxml-py-support" }`. External publication is intentionally disabled.
