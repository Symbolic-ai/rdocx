//! Style access and manipulation for documents.

use rdocx_oxml::properties::{CT_PPr, CT_RPr};
use rdocx_oxml::resolver::FormattingResolver;
use rdocx_oxml::styles::{CT_Style, CT_Styles, StyleType};

/// An immutable reference to a style definition.
pub struct Style<'a> {
    pub(crate) inner: &'a CT_Style,
}

impl<'a> Style<'a> {
    /// The style ID (used to reference this style).
    pub fn style_id(&self) -> &str {
        &self.inner.style_id
    }

    /// The display name of the style.
    pub fn name(&self) -> Option<&str> {
        self.inner.name.as_deref()
    }

    /// The style ID this style is based on.
    pub fn based_on(&self) -> Option<&str> {
        self.inner.based_on.as_deref()
    }

    /// Whether this is the default style for its type.
    pub fn is_default(&self) -> bool {
        self.inner.is_default
    }
}

/// Builder for creating a new paragraph style.
pub struct StyleBuilder {
    style: CT_Style,
}

impl StyleBuilder {
    /// Create a new paragraph style builder.
    pub fn paragraph(style_id: &str, name: &str) -> Self {
        StyleBuilder {
            style: CT_Style {
                style_id: style_id.to_string(),
                style_type: StyleType::Paragraph,
                name: Some(name.to_string()),
                based_on: None,
                next_style: None,
                is_default: false,
                ppr: None,
                rpr: None,
            },
        }
    }

    /// Create a new character style builder.
    pub fn character(style_id: &str, name: &str) -> Self {
        StyleBuilder {
            style: CT_Style {
                style_id: style_id.to_string(),
                style_type: StyleType::Character,
                name: Some(name.to_string()),
                based_on: None,
                next_style: None,
                is_default: false,
                ppr: None,
                rpr: None,
            },
        }
    }

    /// Set the parent style this one inherits from.
    pub fn based_on(mut self, style_id: &str) -> Self {
        self.style.based_on = Some(style_id.to_string());
        self
    }

    /// Set the next style (applied to the following paragraph after pressing Enter).
    pub fn next_style(mut self, style_id: &str) -> Self {
        self.style.next_style = Some(style_id.to_string());
        self
    }

    /// Set paragraph properties for this style.
    pub fn paragraph_properties(mut self, ppr: CT_PPr) -> Self {
        self.style.ppr = Some(ppr);
        self
    }

    /// Set run properties for this style.
    pub fn run_properties(mut self, rpr: CT_RPr) -> Self {
        self.style.rpr = Some(rpr);
        self
    }

    /// Build the style (consumed by Document::add_style).
    pub(crate) fn build(self) -> CT_Style {
        self.style
    }
}

/// Resolve the effective paragraph properties by walking the style inheritance chain.
pub fn resolve_paragraph_properties(style_id: Option<&str>, styles: &CT_Styles) -> CT_PPr {
    FormattingResolver::new(styles, None).resolve_paragraph_style(style_id)
}

/// Resolve the effective run properties by walking the style inheritance chain.
pub fn resolve_run_properties(
    para_style_id: Option<&str>,
    run_style_id: Option<&str>,
    styles: &CT_Styles,
) -> CT_RPr {
    FormattingResolver::new(styles, None).resolve_run_sources(para_style_id, run_style_id, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdocx_oxml::units::{HalfPoint, Twips};

    fn test_styles() -> CT_Styles {
        let mut styles = CT_Styles::new_default();

        // Add a Heading2 based on Heading1
        styles.styles.push(CT_Style {
            style_id: "Heading2".to_string(),
            style_type: StyleType::Paragraph,
            name: Some("heading 2".to_string()),
            based_on: Some("Heading1".to_string()),
            next_style: Some("Normal".to_string()),
            is_default: false,
            ppr: Some(CT_PPr {
                space_before: Some(Twips(40)), // Override Heading1's 240
                ..Default::default()
            }),
            rpr: Some(CT_RPr {
                sz: Some(HalfPoint(26)), // Override Heading1's 32
                color: Some("2E74B5".to_string()),
                ..Default::default()
            }),
        });

        styles
    }

    #[test]
    fn resolve_normal_paragraph() {
        let styles = test_styles();
        let ppr = resolve_paragraph_properties(Some("Normal"), &styles);
        // Should have docDefaults' spacing
        assert_eq!(ppr.space_after, Some(Twips(160)));
    }

    #[test]
    fn resolve_heading1() {
        let styles = test_styles();
        let ppr = resolve_paragraph_properties(Some("Heading1"), &styles);
        // keepNext from Heading1
        assert_eq!(ppr.keep_next, Some(true));
        // spaceBefore from Heading1 (overrides docDefaults which has none)
        assert_eq!(ppr.space_before, Some(Twips(240)));
        // spaceAfter from Heading1 overrides docDefaults
        assert_eq!(ppr.space_after, Some(Twips(0)));
    }

    #[test]
    fn resolve_heading2_inherits_heading1() {
        let styles = test_styles();
        let ppr = resolve_paragraph_properties(Some("Heading2"), &styles);
        // keepNext inherited from Heading1
        assert_eq!(ppr.keep_next, Some(true));
        // spaceBefore overridden by Heading2
        assert_eq!(ppr.space_before, Some(Twips(40)));
    }

    #[test]
    fn resolve_heading2_rpr() {
        let styles = test_styles();
        let rpr = resolve_run_properties(Some("Heading2"), None, &styles);
        // Font from docDefaults
        assert_eq!(rpr.font_ascii, Some("Calibri".to_string()));
        // Size overridden by Heading2 (not Heading1's 32)
        assert_eq!(rpr.sz, Some(HalfPoint(26)));
        // Bold inherited from Heading1
        assert_eq!(rpr.bold, Some(true));
        // Color from Heading2
        assert_eq!(rpr.color, Some("2E74B5".to_string()));
    }

    #[test]
    fn resolve_default_when_no_style() {
        let styles = test_styles();
        let ppr = resolve_paragraph_properties(None, &styles);
        // Should get docDefaults
        assert_eq!(ppr.space_after, Some(Twips(160)));
    }
}
