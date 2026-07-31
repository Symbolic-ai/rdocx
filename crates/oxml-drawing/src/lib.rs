pub mod namespace;
pub mod order;

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
        assert!(metadata.contains("{\"name\":\"oxml-drawing\",\"version\":\"0.0.0\""));

        let manifest = include_str!("../Cargo.toml");
        assert!(manifest.contains("version = \"0.0.0\""));
        assert!(manifest.contains("publish = false"));

        let output = Command::new(env!("CARGO"))
            .args([
                "tree",
                "-p",
                "oxml-drawing",
                "--edges",
                "normal",
                "--prefix",
                "none",
            ])
            .output()
            .expect("cargo tree should run");
        assert!(output.status.success(), "cargo tree should succeed");
        assert_eq!(
            String::from_utf8(output.stdout)
                .expect("cargo tree output should be UTF-8")
                .lines()
                .count(),
            1,
            "oxml-drawing should have no production dependencies"
        );
    }
}
