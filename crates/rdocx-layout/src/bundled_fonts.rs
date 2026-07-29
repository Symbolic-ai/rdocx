//! Bundled fallback fonts for standalone operation.
//!
//! When the `bundled-fonts` feature is enabled (default), the following fonts
//! are embedded in the binary:
//!
//! - **Carlito** — metric-compatible with Calibri (Word's default font),
//!   licensed under the SIL Open Font License 1.1
//! - **Caladea** — metric-compatible with Cambria, licensed under the Apache
//!   License 2.0
//! - **Liberation Sans** — metric-compatible with Arial, licensed under the
//!   SIL Open Font License 1.1
//! - **Liberation Serif** — metric-compatible with Times New Roman, licensed
//!   under the SIL Open Font License 1.1
//! - **Liberation Mono** — metric-compatible with Courier New, licensed under
//!   the SIL Open Font License 1.1

/// Returns bundled font data: `(family_name, font_bytes)` pairs.
///
/// When `bundled-fonts` feature is enabled, returns Carlito, Caladea,
/// Liberation Sans, Liberation Serif, and Liberation Mono (each with
/// Regular, Bold, Italic, and BoldItalic variants).
///
/// When the feature is disabled, returns an empty vec (system fonts are used).
pub fn bundled_font_data() -> Vec<(&'static str, &'static [u8])> {
    #[cfg(feature = "bundled-fonts")]
    {
        vec![
            // Carlito — metric-compatible replacement for Calibri
            (
                "Carlito",
                include_bytes!("../fonts/Carlito-Regular.ttf").as_slice(),
            ),
            (
                "Carlito",
                include_bytes!("../fonts/Carlito-Bold.ttf").as_slice(),
            ),
            (
                "Carlito",
                include_bytes!("../fonts/Carlito-Italic.ttf").as_slice(),
            ),
            (
                "Carlito",
                include_bytes!("../fonts/Carlito-BoldItalic.ttf").as_slice(),
            ),
            // Caladea — metric-compatible replacement for Cambria
            (
                "Caladea",
                include_bytes!("../fonts/Caladea-Regular.ttf").as_slice(),
            ),
            (
                "Caladea",
                include_bytes!("../fonts/Caladea-Bold.ttf").as_slice(),
            ),
            (
                "Caladea",
                include_bytes!("../fonts/Caladea-Italic.ttf").as_slice(),
            ),
            (
                "Caladea",
                include_bytes!("../fonts/Caladea-BoldItalic.ttf").as_slice(),
            ),
            // Liberation Sans — metric-compatible replacement for Arial
            (
                "Liberation Sans",
                include_bytes!("../fonts/LiberationSans-Regular.ttf").as_slice(),
            ),
            (
                "Liberation Sans",
                include_bytes!("../fonts/LiberationSans-Bold.ttf").as_slice(),
            ),
            (
                "Liberation Sans",
                include_bytes!("../fonts/LiberationSans-Italic.ttf").as_slice(),
            ),
            (
                "Liberation Sans",
                include_bytes!("../fonts/LiberationSans-BoldItalic.ttf").as_slice(),
            ),
            // Liberation Serif — metric-compatible replacement for Times New Roman
            (
                "Liberation Serif",
                include_bytes!("../fonts/LiberationSerif-Regular.ttf").as_slice(),
            ),
            (
                "Liberation Serif",
                include_bytes!("../fonts/LiberationSerif-Bold.ttf").as_slice(),
            ),
            (
                "Liberation Serif",
                include_bytes!("../fonts/LiberationSerif-Italic.ttf").as_slice(),
            ),
            (
                "Liberation Serif",
                include_bytes!("../fonts/LiberationSerif-BoldItalic.ttf").as_slice(),
            ),
            // Liberation Mono — metric-compatible replacement for Courier New
            (
                "Liberation Mono",
                include_bytes!("../fonts/LiberationMono-Regular.ttf").as_slice(),
            ),
            (
                "Liberation Mono",
                include_bytes!("../fonts/LiberationMono-Bold.ttf").as_slice(),
            ),
            (
                "Liberation Mono",
                include_bytes!("../fonts/LiberationMono-Italic.ttf").as_slice(),
            ),
            (
                "Liberation Mono",
                include_bytes!("../fonts/LiberationMono-BoldItalic.ttf").as_slice(),
            ),
        ]
    }
    #[cfg(not(feature = "bundled-fonts"))]
    {
        vec![]
    }
}

#[cfg(all(test, feature = "bundled-fonts"))]
mod tests {
    use super::bundled_font_data;
    use std::collections::BTreeSet;
    use std::path::Path;

    #[test]
    fn every_bundled_font_family_has_a_licence_file() {
        let family_licences = [
            ("Caladea", "LICENSE-Caladea"),
            ("Carlito", "LICENSE-Carlito"),
            ("Liberation Mono", "LICENSE-Liberation"),
            ("Liberation Sans", "LICENSE-Liberation"),
            ("Liberation Serif", "LICENSE-Liberation"),
        ];
        let bundled_families = bundled_font_data()
            .into_iter()
            .map(|(family, _)| family)
            .collect::<BTreeSet<_>>();
        let licensed_families = family_licences
            .iter()
            .map(|(family, _)| *family)
            .collect::<BTreeSet<_>>();

        assert_eq!(bundled_families, licensed_families);

        let fonts_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("fonts");
        for (family, licence_file) in family_licences {
            assert!(
                fonts_dir.join(licence_file).is_file(),
                "{family} is missing {licence_file}"
            );
        }
    }
}
