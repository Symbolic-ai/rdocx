# rdocx-opc

`rdocx-opc` is a deprecated compatibility shim for the shared
[`oxml-opc`](https://docs.rs/oxml-opc) package layer. Existing code can keep
its old imports while migrating. New code should depend on `oxml-opc`
directly.

The retained types are exact re-exports. Word-specific package construction
belongs in the high-level [`rdocx`](https://docs.rs/rdocx) facade.

```rust,no_run
use rdocx_opc::OpcPackage;

let package = OpcPackage::new();
assert!(package.parts.is_empty());
```

```toml
[dependencies]
rdocx-opc = "0.4"
```

For new code, replace both the dependency and the import with `oxml-opc` and
`oxml_opc`.
