# rdocx-opc

`rdocx-opc` keeps existing Word package code compiling while it moves to
[`oxml-opc`](https://docs.rs/oxml-opc). Every public item is an exact re-export,
so migration changes the dependency and import path without changing package
behavior.

## Capabilities

- Exact `oxml-opc` re-exports under the former crate name.
- Existing `OpcPackage` imports continue to compile.
- No added Word-specific package policy or forwarding layer.
- A direct migration path for applications moving to the shared crate.

## Use it when

Use this crate only while migrating an existing `rdocx-opc` dependency. New
code should depend on `oxml-opc` directly.

## Relationship

The retained types are exact re-exports. Word-specific package construction
belongs in the high-level [`rdocx`](https://docs.rs/rdocx) facade.

## Example

```rust,no_run
use rdocx_opc::OpcPackage;

let package = OpcPackage::new();
assert!(package.parts.is_empty());
```

```toml
[dependencies]
rdocx-opc = "0.13.1"
```

For new code, replace both the dependency and the import with `oxml-opc` and
`oxml_opc`.
