pub mod color;
pub mod geometry;
pub mod namespace;
pub mod order;
pub mod xfrm;

#[cfg(test)]
mod tests {
    use super::namespace::{A_NS, A_PREFIX, PIC_NS, PIC_PREFIX};

    #[test]
    fn drawingml_namespace_uris_match_the_specification() {
        assert_eq!(
            A_NS,
            "http://schemas.openxmlformats.org/drawingml/2006/main"
        );
        assert_eq!(A_PREFIX, "a");
        assert_eq!(
            PIC_NS,
            "http://schemas.openxmlformats.org/drawingml/2006/picture"
        );
        assert_eq!(PIC_PREFIX, "pic");
    }

    #[test]
    fn oxml_drawing_is_an_unpublished_workspace_member() {
        let manifest = include_str!("../Cargo.toml");
        assert!(manifest.contains("version = \"0.0.0\""));
        assert!(manifest.contains("publish = false"));
    }
}
