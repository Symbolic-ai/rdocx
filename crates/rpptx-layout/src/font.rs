//! Theme typeface resolution.

use oxml_drawing::theme::CT_FontCollection;

use crate::ResolveCtx;

impl ResolveCtx<'_> {
    /// Resolves a DrawingML theme typeface token to a concrete font face.
    ///
    /// A supplied ISO 15924 script selects the first matching supplemental
    /// face in document order. Ordinary faces and unknown tokens pass through
    /// unchanged.
    pub fn resolve_typeface(&self, typeface: &str, script: Option<&str>) -> String {
        let scheme = &self.theme.theme_elements.font_scheme;
        let (collection, base) = match typeface {
            "+mj-lt" => (&scheme.major_font, &scheme.major_font.latin.typeface),
            "+mn-lt" => (&scheme.minor_font, &scheme.minor_font.latin.typeface),
            "+mj-ea" => (&scheme.major_font, &scheme.major_font.east_asian.typeface),
            "+mn-ea" => (&scheme.minor_font, &scheme.minor_font.east_asian.typeface),
            "+mj-cs" => (
                &scheme.major_font,
                &scheme.major_font.complex_script.typeface,
            ),
            "+mn-cs" => (
                &scheme.minor_font,
                &scheme.minor_font.complex_script.typeface,
            ),
            _ => return typeface.to_owned(),
        };

        supplemental_typeface(collection, script)
            .unwrap_or(base)
            .clone()
    }
}

fn supplemental_typeface<'a>(
    collection: &'a CT_FontCollection,
    script: Option<&str>,
) -> Option<&'a String> {
    let script = script?;
    collection
        .supplemental_fonts
        .iter()
        .find(|font| font.script == script)
        .map(|font| &font.typeface)
}

#[cfg(test)]
mod tests {
    use oxml_drawing::color::ColorMap;
    use oxml_drawing::text::CT_TextListStyle;
    use oxml_drawing::theme::CT_OfficeStyleSheet;
    use rpptx_oxml::slide_parts::{CT_Slide, CT_SlideLayout, CT_SlideMaster};

    use crate::ResolveCtx;

    const P_NS: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";

    #[test]
    fn minor_latin_theme_token_resolves_to_minor_latin_typeface() {
        let fixture = Fixture::new();

        assert_eq!(fixture.context().resolve_typeface("+mn-lt", None), "Aptos");
    }

    #[test]
    fn major_and_minor_tokens_select_each_base_face() {
        let fixture = Fixture::new();
        let context = fixture.context();

        for (token, expected) in [
            ("+mj-lt", "Aptos Display"),
            ("+mn-lt", "Aptos"),
            ("+mj-ea", ""),
            ("+mn-ea", ""),
            ("+mj-cs", ""),
            ("+mn-cs", ""),
        ] {
            assert_eq!(context.resolve_typeface(token, None), expected);
        }
    }

    #[test]
    fn supplemental_script_face_overrides_the_base() {
        let fixture = Fixture::new();
        let context = fixture.context();

        assert_eq!(
            context.resolve_typeface("+mj-lt", Some("Jpan")),
            "游ゴシック Light"
        );
        assert_eq!(
            context.resolve_typeface("+mn-ea", Some("Jpan")),
            "游ゴシック"
        );
    }

    #[test]
    fn missing_script_override_falls_back_to_the_base() {
        let fixture = Fixture::new();
        let context = fixture.context();

        assert_eq!(
            context.resolve_typeface("+mj-lt", Some("Zzzz")),
            "Aptos Display"
        );
        assert_eq!(context.resolve_typeface("+mn-cs", None), "");
    }

    #[test]
    fn ordinary_and_unknown_typefaces_pass_through() {
        let fixture = Fixture::new();
        let context = fixture.context();

        assert_eq!(context.resolve_typeface("Arial", Some("Jpan")), "Arial");
        assert_eq!(context.resolve_typeface("+mn-zz", Some("Jpan")), "+mn-zz");
        assert_eq!(context.resolve_typeface("+future", None), "+future");
    }

    #[test]
    fn duplicate_supplemental_scripts_use_document_order() {
        let mut fixture = Fixture::new();
        let fonts = &mut fixture
            .theme
            .theme_elements
            .font_scheme
            .minor_font
            .supplemental_fonts;
        let mut first = fonts[0].clone();
        first.script = "Test".to_owned();
        first.typeface = "First Face".to_owned();
        let mut second = fonts[1].clone();
        second.script = "Test".to_owned();
        second.typeface = "Second Face".to_owned();
        fonts.insert(0, second);
        fonts.insert(0, first);

        assert_eq!(
            fixture.context().resolve_typeface("+mn-lt", Some("Test")),
            "First Face"
        );
    }

    struct Fixture {
        theme: CT_OfficeStyleSheet,
        master: CT_SlideMaster,
        layout: CT_SlideLayout,
        slide: CT_Slide,
        default_text_style: CT_TextListStyle,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                theme: CT_OfficeStyleSheet::office_default(),
                master: CT_SlideMaster::from_xml(master_xml().as_bytes()).unwrap(),
                layout: CT_SlideLayout::from_xml(layout_xml().as_bytes()).unwrap(),
                slide: CT_Slide::from_xml(slide_xml().as_bytes()).unwrap(),
                default_text_style: CT_TextListStyle::default(),
            }
        }

        fn context(&self) -> ResolveCtx<'_> {
            ResolveCtx::new(
                &self.theme,
                ColorMap::default(),
                &self.master,
                &self.layout,
                &self.slide,
                &self.default_text_style,
            )
        }
    }

    fn slide_xml() -> String {
        format!(
            "<p:sld xmlns:p=\"{P_NS}\"><p:cSld>{}</p:cSld></p:sld>",
            shape_tree()
        )
    }

    fn layout_xml() -> String {
        format!(
            "<p:sldLayout xmlns:p=\"{P_NS}\"><p:cSld>{}</p:cSld></p:sldLayout>",
            shape_tree()
        )
    }

    fn master_xml() -> String {
        format!(
            "<p:sldMaster xmlns:p=\"{P_NS}\"><p:cSld>{}</p:cSld><p:clrMap bg1=\"lt1\" tx1=\"dk1\" bg2=\"lt2\" tx2=\"dk2\" accent1=\"accent1\" accent2=\"accent2\" accent3=\"accent3\" accent4=\"accent4\" accent5=\"accent5\" accent6=\"accent6\" hlink=\"hlink\" folHlink=\"folHlink\"/></p:sldMaster>",
            shape_tree()
        )
    }

    fn shape_tree() -> &'static str {
        "<p:spTree><p:nvGrpSpPr/><p:grpSpPr/></p:spTree>"
    }
}
