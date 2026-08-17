#![doc = include_str!("../README.md")]

pub use oxml_chart::*;

#[cfg(test)]
mod tests {
    #[test]
    fn legacy_shim_retains_shared_chart_type() {
        fn accepts_shared(_: oxml_chart::AxisId) {}

        let legacy = crate::AxisId::new(10_000_001).expect("valid axis identifier");
        accepts_shared(legacy);
    }
}
