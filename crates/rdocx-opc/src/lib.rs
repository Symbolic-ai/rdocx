//! Deprecated compatibility shim for [`oxml_opc`].
//!
//! Use `oxml-opc` directly for new code. Retained types are exact re-exports,
//! so legacy type paths remain type-identical. The removed Word-specific
//! `OpcPackage::new_docx` and `ContentTypes::new_docx` constructors have no
//! shared equivalent. Word consumers should configure the generic constructors
//! at their format boundary.

pub use oxml_opc::*;

#[cfg(test)]
mod tests {
    use super::OpcPackage;

    #[test]
    fn legacy_shim_retains_shared_opc_package_type() {
        fn accept_shared(_: oxml_opc::OpcPackage) {}

        accept_shared(OpcPackage::new());
    }
}
