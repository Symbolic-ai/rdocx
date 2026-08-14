#![doc = include_str!("../README.md")]

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
