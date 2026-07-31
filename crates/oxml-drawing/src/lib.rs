pub mod namespace;

#[cfg(test)]
mod tests {
    use std::process::Command;

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
        let output = Command::new(env!("CARGO"))
            .args(["metadata", "--no-deps", "--format-version", "1"])
            .output()
            .expect("cargo metadata should run");
        assert!(output.status.success(), "cargo metadata should succeed");

        let metadata = String::from_utf8(output.stdout).expect("metadata should be UTF-8");
        let package_start = metadata
            .find("{\"name\":\"oxml-drawing\"")
            .expect("oxml-drawing should be a workspace package");
        let package_tail = &metadata[package_start..];
        let package_end = package_tail
            .find("},{\"name\":")
            .unwrap_or(package_tail.len());
        let package = &package_tail[..package_end];

        assert!(package.starts_with("{\"name\":\"oxml-drawing\",\"version\":\"0.0.0\""));
        assert!(package.contains("\"publish\":[]"));
        assert!(package.contains("\"dependencies\":[]"));
    }
}
