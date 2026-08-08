//! Text inheritance resolution.

use rpptx_oxml::placeholder::PhType;
use rpptx_oxml::shape_tree::CT_Shape;

use oxml_drawing::text::{
    CT_TextCharacterProperties, CT_TextListStyle, CT_TextParagraphProperties,
};

use crate::ResolveCtx;

/// Resolved paragraph and character properties without opaque XML or actions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EffectiveTextProperties {
    pub paragraph: CT_TextParagraphProperties,
    pub run: CT_TextCharacterProperties,
}

/// The independently resolved properties for all nine DrawingML list levels.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EffectiveListStyle {
    levels: [EffectiveTextProperties; 9],
}

impl EffectiveListStyle {
    /// Returns the zero-based DrawingML level selected by `a:pPr/@lvl`.
    pub fn level(&self, level: u8) -> Option<&EffectiveTextProperties> {
        self.levels.get(usize::from(level))
    }
}

impl ResolveCtx<'_> {
    /// Resolves sources one through five for every list level of one shape.
    pub fn effective_list_style(&self, shape: &CT_Shape) -> EffectiveListStyle {
        let key = shape
            .placeholder
            .as_ref()
            .map(|placeholder| placeholder.key());
        let mut effective = if let Some(cached) = self.list_style_cache.borrow().get(&key) {
            cached.clone()
        } else {
            let resolved = self.resolve_list_style_prefix(shape);
            self.list_style_cache
                .borrow_mut()
                .insert(key, resolved.clone());
            resolved
        };
        if let Some(list_style) = shape.text_body.as_ref().and_then(|body| body.list_style()) {
            merge_list_style(&mut effective, list_style);
        }
        effective
    }

    /// Applies paragraph and run properties to the selected zero-based level.
    pub fn effective_text_properties(
        &self,
        shape: &CT_Shape,
        paragraph: Option<&CT_TextParagraphProperties>,
        run: Option<&CT_TextCharacterProperties>,
    ) -> EffectiveTextProperties {
        let style = self.effective_list_style(shape);
        let level = paragraph
            .and_then(|properties| properties.level)
            .unwrap_or(0);
        let mut effective = style
            .level(level)
            .cloned()
            .unwrap_or_else(EffectiveTextProperties::default);
        if let Some(properties) = paragraph {
            merge_paragraph(&mut effective, properties);
        }
        if let Some(properties) = run {
            merge_character(&mut effective, properties);
        }
        effective
    }

    /// Resolves presentation defaults and the master's other style for table text.
    pub(crate) fn effective_table_text_properties(
        &self,
        table_style: Option<&CT_TextCharacterProperties>,
        paragraph: Option<&CT_TextParagraphProperties>,
        run: Option<&CT_TextCharacterProperties>,
    ) -> EffectiveTextProperties {
        let mut style = EffectiveListStyle::default();
        merge_list_style(&mut style, self.default_text_style);
        if let Some(styles) = &self.master.text_styles {
            merge_list_style(&mut style, &styles.other_style);
        }
        let level = paragraph
            .and_then(|properties| properties.level)
            .unwrap_or(0);
        let mut effective = style
            .level(level)
            .cloned()
            .unwrap_or_else(EffectiveTextProperties::default);
        if let Some(properties) = table_style {
            merge_character(&mut effective, properties);
        }
        if let Some(properties) = paragraph {
            merge_paragraph(&mut effective, properties);
        }
        if let Some(properties) = run {
            merge_character(&mut effective, properties);
        }
        effective
    }

    fn resolve_list_style_prefix(&self, shape: &CT_Shape) -> EffectiveListStyle {
        let mut effective = EffectiveListStyle::default();
        merge_list_style(&mut effective, self.default_text_style);

        if let Some(styles) = &self.master.text_styles {
            let master_style = match shape
                .placeholder
                .as_ref()
                .map(|placeholder| placeholder.effective_type())
            {
                Some(PhType::Title | PhType::CenteredTitle) => &styles.title_style,
                Some(PhType::Body | PhType::Subtitle | PhType::Object) => &styles.body_style,
                _ => &styles.other_style,
            };
            merge_list_style(&mut effective, master_style);
        }

        let (layout_placeholder, master_placeholder) = self.placeholder_chain(shape);
        if let Some(list_style) = master_placeholder
            .and_then(|placeholder| placeholder.text_body.as_ref())
            .and_then(|body| body.list_style())
        {
            merge_list_style(&mut effective, list_style);
        }
        if let Some(list_style) = layout_placeholder
            .and_then(|placeholder| placeholder.text_body.as_ref())
            .and_then(|body| body.list_style())
        {
            merge_list_style(&mut effective, list_style);
        }
        effective
    }
}

fn merge_list_style(effective: &mut EffectiveListStyle, source: &CT_TextListStyle) {
    for level in 0..9 {
        if let Some(properties) = &source.default_paragraph_properties {
            merge_paragraph(&mut effective.levels[level], properties);
        }
        if let Some(properties) = source.level(level + 1) {
            merge_paragraph(&mut effective.levels[level], properties);
        }
    }
}

fn merge_paragraph(effective: &mut EffectiveTextProperties, source: &CT_TextParagraphProperties) {
    if source.left_margin.is_some() {
        effective.paragraph.left_margin = source.left_margin;
    }
    if source.right_margin.is_some() {
        effective.paragraph.right_margin = source.right_margin;
    }
    if source.level.is_some() {
        effective.paragraph.level = source.level;
    }
    if source.indent.is_some() {
        effective.paragraph.indent = source.indent;
    }
    if source.alignment.is_some() {
        effective.paragraph.alignment = source.alignment;
    }
    if source.line_spacing.is_some() {
        effective.paragraph.line_spacing = source.line_spacing.clone();
    }
    if source.space_before.is_some() {
        effective.paragraph.space_before = source.space_before.clone();
    }
    if source.space_after.is_some() {
        effective.paragraph.space_after = source.space_after.clone();
    }
    if let Some(source_bullet) = &source.bullet {
        let bullet = effective.paragraph.bullet.get_or_insert_default();
        if source_bullet.color.is_some() {
            bullet.color = source_bullet.color.clone();
        }
        if source_bullet.size.is_some() {
            bullet.size = source_bullet.size.clone();
        }
        if source_bullet.font.is_some() {
            bullet.font = source_bullet.font.clone();
        }
        if source_bullet.choice.is_some() {
            bullet.choice = source_bullet.choice.clone();
        }
    }
    if let Some(properties) = &source.default_run_properties {
        merge_character_properties(&mut effective.run, properties);
    }
}

fn merge_character(effective: &mut EffectiveTextProperties, source: &CT_TextCharacterProperties) {
    merge_character_properties(&mut effective.run, source);
}

fn merge_character_properties(
    effective: &mut CT_TextCharacterProperties,
    source: &CT_TextCharacterProperties,
) {
    if source.font_size.is_some() {
        effective.font_size = source.font_size;
    }
    if source.bold.is_some() {
        effective.bold = source.bold;
    }
    if source.italic.is_some() {
        effective.italic = source.italic;
    }
    if source.all_caps.is_some() {
        effective.all_caps = source.all_caps;
    }
    if source.underline.is_some() {
        effective.underline = source.underline;
    }
    if source.strike.is_some() {
        effective.strike = source.strike;
    }
    if source.spacing.is_some() {
        effective.spacing = source.spacing.clone();
    }
    if source.baseline.is_some() {
        effective.baseline = source.baseline.clone();
    }
    if source.fill.is_some() {
        effective.fill = source.fill.clone();
    }
    if source.latin.is_some() {
        effective.latin = source.latin.clone();
    }
    if source.east_asian.is_some() {
        effective.east_asian = source.east_asian.clone();
    }
    if source.complex_script.is_some() {
        effective.complex_script = source.complex_script.clone();
    }
    if source.symbol.is_some() {
        effective.symbol = source.symbol.clone();
    }
}

#[cfg(test)]
mod tests {
    use oxml_drawing::text::{
        CT_TextCharacterProperties, CT_TextListStyle, CT_TextParagraphProperties, TextAlignment,
        TextBulletChoice, TextBulletSizeValue,
    };
    use oxml_drawing::{color::ColorMap, theme::CT_OfficeStyleSheet};
    use rpptx_oxml::shape_tree::{CT_Shape, ShapeTreeChild};
    use rpptx_oxml::slide_parts::{CT_Slide, CT_SlideLayout, CT_SlideMaster};

    use super::{EffectiveListStyle, EffectiveTextProperties, merge_list_style, merge_paragraph};
    use crate::ResolveCtx;

    const A_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";

    #[test]
    fn seven_source_list_style_merge_resolves_run_size_and_typeface() {
        let fixture = Fixture::new(
            "<a:defPPr marL=\"100\"><a:defRPr sz=\"1000\"/></a:defPPr>",
            "<a:defPPr marR=\"200\"/>",
            "<a:defPPr indent=\"300\"/>",
            "<a:defPPr algn=\"ctr\"/>",
            &["<a:defPPr><a:defRPr b=\"1\"/></a:defPPr>"],
        );
        let paragraph = paragraph("<a:pPr><a:defRPr sz=\"2400\"/></a:pPr>");
        let run = character("<a:rPr><a:latin typeface=\"Run Face\"/></a:rPr>");
        let resolved = fixture.context().effective_text_properties(
            fixture.slide_shape(0),
            Some(&paragraph),
            Some(&run),
        );

        assert_eq!(resolved.run.font_size, Some(2400));
        assert_eq!(resolved.run.latin.unwrap().typeface, "Run Face");
        assert_eq!(resolved.paragraph.left_margin, Some(100));
        assert_eq!(resolved.paragraph.right_margin, Some(200));
        assert_eq!(resolved.paragraph.indent, Some(300));
        assert_eq!(resolved.paragraph.alignment, Some(TextAlignment::Center));
        assert_eq!(resolved.run.bold, Some(true));
    }

    #[test]
    fn default_paragraph_properties_apply_before_each_level() {
        let source = list_style(
            "<a:defPPr marL=\"100\"><a:defRPr sz=\"1200\"/></a:defPPr>\
             <a:lvl1pPr marL=\"200\"/><a:lvl9pPr><a:defRPr b=\"1\"/></a:lvl9pPr>",
        );
        let mut effective = EffectiveListStyle::default();
        merge_list_style(&mut effective, &source);

        assert_eq!(effective.level(0).unwrap().paragraph.left_margin, Some(200));
        assert_eq!(effective.level(8).unwrap().paragraph.left_margin, Some(100));
        assert_eq!(effective.level(8).unwrap().run.font_size, Some(1200));
        assert_eq!(effective.level(8).unwrap().run.bold, Some(true));
    }

    #[test]
    fn all_nine_levels_merge_independently() {
        let levels = (1..=9)
            .map(|level| format!("<a:lvl{level}pPr marL=\"{}\"/>", level * 100))
            .collect::<String>();
        let mut effective = EffectiveListStyle::default();
        merge_list_style(&mut effective, &list_style(&levels));

        for level in 0..9 {
            assert_eq!(
                effective.level(level).unwrap().paragraph.left_margin,
                Some(i32::from(level + 1) * 100)
            );
        }
    }

    #[test]
    fn later_sources_win_per_property_without_erasing_other_fields() {
        let mut effective = EffectiveTextProperties::default();
        merge_paragraph(
            &mut effective,
            &paragraph("<a:pPr marL=\"100\" marR=\"200\"><a:defRPr sz=\"1100\" b=\"1\"/></a:pPr>"),
        );
        merge_paragraph(
            &mut effective,
            &paragraph("<a:pPr marL=\"300\"><a:defRPr sz=\"2200\"/></a:pPr>"),
        );

        assert_eq!(effective.paragraph.left_margin, Some(300));
        assert_eq!(effective.paragraph.right_margin, Some(200));
        assert_eq!(effective.run.font_size, Some(2200));
        assert_eq!(effective.run.bold, Some(true));
    }

    #[test]
    fn all_caps_inherits_and_a_direct_none_value_overrides_it() {
        let fixture = Fixture::new(
            "<a:defPPr><a:defRPr cap=\"all\"/></a:defPPr>",
            "",
            "",
            "",
            &[""],
        );
        let inherited =
            fixture
                .context()
                .effective_text_properties(fixture.slide_shape(0), None, None);
        assert_eq!(inherited.run.all_caps, Some(true));

        let direct = character("<a:rPr cap=\"none\"></a:rPr>");
        let overridden = fixture.context().effective_text_properties(
            fixture.slide_shape(0),
            None,
            Some(&direct),
        );
        assert_eq!(overridden.run.all_caps, Some(false));
    }

    #[test]
    fn fills_and_typeface_slots_merge_as_atomic_properties() {
        let mut effective = EffectiveTextProperties::default();
        merge_paragraph(
            &mut effective,
            &paragraph(
                "<a:pPr><a:defRPr><a:solidFill><a:srgbClr val=\"112233\"/></a:solidFill><a:latin typeface=\"Latin One\"/><a:ea typeface=\"East Asian One\"/></a:defRPr></a:pPr>",
            ),
        );
        merge_paragraph(
            &mut effective,
            &paragraph(
                "<a:pPr><a:defRPr><a:solidFill><a:srgbClr val=\"AABBCC\"/></a:solidFill><a:latin typeface=\"Latin Two\"/></a:defRPr></a:pPr>",
            ),
        );

        assert_eq!(effective.run.latin.unwrap().typeface, "Latin Two");
        assert_eq!(effective.run.east_asian.unwrap().typeface, "East Asian One");
        assert_eq!(
            effective.run.fill,
            character("<a:rPr><a:solidFill><a:srgbClr val=\"AABBCC\"/></a:solidFill></a:rPr>").fill
        );
    }

    #[test]
    fn raw_xml_and_hyperlink_actions_are_not_inherited() {
        let source = paragraph(
            "<a:pPr producer=\"raw\"><x:opaque xmlns:x=\"urn:x\"/><a:defRPr><a:hlinkClick action=\"ppaction://jump\"/></a:defRPr></a:pPr>",
        );
        let mut effective = EffectiveTextProperties::default();
        merge_paragraph(&mut effective, &source);

        assert!(effective.paragraph.raw_children().is_empty());
        assert!(effective.run.raw_children().is_empty());
        assert!(effective.run.hyperlink_click.is_none());
        assert!(effective.run.hyperlink_mouse_over.is_none());
    }

    #[test]
    fn bullet_components_merge_independently() {
        let mut effective = EffectiveTextProperties::default();
        merge_paragraph(
            &mut effective,
            &paragraph(
                "<a:pPr><a:buClr><a:srgbClr val=\"112233\"/></a:buClr><a:buSzPts val=\"1800\"/><a:buChar char=\"•\"/></a:pPr>",
            ),
        );
        merge_paragraph(
            &mut effective,
            &paragraph("<a:pPr><a:buFont typeface=\"Wingdings\"/><a:buNone/></a:pPr>"),
        );
        let bullet = effective.paragraph.bullet.unwrap();

        assert!(bullet.color.is_some());
        assert!(matches!(
            bullet.size.unwrap().value,
            TextBulletSizeValue::Points(1800)
        ));
        assert_eq!(bullet.font.unwrap().typeface, "Wingdings");
        assert!(matches!(bullet.choice, Some(TextBulletChoice::None(_))));
    }

    #[test]
    fn shape_owned_style_is_not_shared_by_placeholder_cache() {
        let fixture = Fixture::new(
            "",
            "",
            "<a:lvl1pPr marL=\"100\"/>",
            "",
            &["<a:lvl1pPr marL=\"900\"/>", "<a:lvl1pPr marL=\"800\"/>"],
        );
        let context = fixture.context();
        let first_shape = context.effective_list_style(fixture.slide_shape(0));
        let second_shape = context.effective_list_style(fixture.slide_shape(1));

        assert_eq!(
            first_shape.level(0).unwrap().paragraph.left_margin,
            Some(900)
        );
        assert_eq!(
            second_shape.level(0).unwrap().paragraph.left_margin,
            Some(800)
        );
    }

    #[test]
    fn default_paragraph_properties_round_trip_in_schema_order() {
        let xml = format!(
            "<a:lstStyle xmlns:a=\"{A_NS}\"><x:before xmlns:x=\"urn:x\"/><defPPr marL=\"100\"/><x:between xmlns:x=\"urn:x\"/><a:lvl1pPr/><x:after xmlns:x=\"urn:x\"/></a:lstStyle>"
        );
        let style = CT_TextListStyle::from_xml(xml.as_bytes()).unwrap();
        assert_eq!(
            style
                .default_paragraph_properties
                .as_ref()
                .unwrap()
                .left_margin,
            Some(100)
        );
        let output = String::from_utf8(style.to_xml().unwrap()).unwrap();

        assert_eq!(
            output,
            "<a:lstStyle xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\"><x:before xmlns:x=\"urn:x\"/><a:defPPr marL=\"100\"/><x:between xmlns:x=\"urn:x\"/><a:lvl1pPr/><x:after xmlns:x=\"urn:x\"/></a:lstStyle>"
        );
    }

    fn list_style(children: &str) -> CT_TextListStyle {
        CT_TextListStyle::from_xml(
            format!("<a:lstStyle xmlns:a=\"{A_NS}\">{children}</a:lstStyle>").as_bytes(),
        )
        .unwrap()
    }

    fn paragraph(xml: &str) -> CT_TextParagraphProperties {
        CT_TextParagraphProperties::from_xml(with_namespace(xml).as_bytes()).unwrap()
    }

    fn character(xml: &str) -> CT_TextCharacterProperties {
        CT_TextCharacterProperties::from_xml(with_namespace(xml).as_bytes()).unwrap()
    }

    fn with_namespace(xml: &str) -> String {
        xml.replacen('>', &format!(" xmlns:a=\"{A_NS}\">"), 1)
    }

    struct Fixture {
        theme: CT_OfficeStyleSheet,
        master: CT_SlideMaster,
        layout: CT_SlideLayout,
        slide: CT_Slide,
        default_text_style: CT_TextListStyle,
    }

    impl Fixture {
        fn new(
            default_style: &str,
            master_style: &str,
            master_placeholder: &str,
            layout_placeholder: &str,
            slide_styles: &[&str],
        ) -> Self {
            let slide_shapes = slide_styles
                .iter()
                .map(|style| shape(7, style))
                .collect::<String>();
            Self {
                theme: CT_OfficeStyleSheet::office_default(),
                master: CT_SlideMaster::from_xml(
                    master_xml(&shape(7, master_placeholder), master_style).as_bytes(),
                )
                .unwrap(),
                layout: CT_SlideLayout::from_xml(
                    layout_xml(&shape(7, layout_placeholder)).as_bytes(),
                )
                .unwrap(),
                slide: CT_Slide::from_xml(slide_xml(&slide_shapes).as_bytes()).unwrap(),
                default_text_style: list_style(default_style),
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

        fn slide_shape(&self, index: usize) -> &CT_Shape {
            let ShapeTreeChild::Shape(shape) =
                &self.slide.common_slide_data.shape_tree.children[index]
            else {
                panic!("expected slide shape");
            };
            shape
        }
    }

    fn slide_xml(shapes: &str) -> String {
        format!(
            "<p:sld xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\" xmlns:a=\"{A_NS}\"><p:cSld>{}</p:cSld></p:sld>",
            shape_tree(shapes)
        )
    }

    fn layout_xml(shapes: &str) -> String {
        format!(
            "<p:sldLayout xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\" xmlns:a=\"{A_NS}\"><p:cSld>{}</p:cSld></p:sldLayout>",
            shape_tree(shapes)
        )
    }

    fn master_xml(shapes: &str, body_style: &str) -> String {
        format!(
            "<p:sldMaster xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\" xmlns:a=\"{A_NS}\"><p:cSld>{}</p:cSld><p:clrMap bg1=\"lt1\" tx1=\"dk1\" bg2=\"lt2\" tx2=\"dk2\" accent1=\"accent1\" accent2=\"accent2\" accent3=\"accent3\" accent4=\"accent4\" accent5=\"accent5\" accent6=\"accent6\" hlink=\"hlink\" folHlink=\"folHlink\"/><p:txStyles><p:titleStyle/><p:bodyStyle>{body_style}</p:bodyStyle><p:otherStyle/></p:txStyles></p:sldMaster>",
            shape_tree(shapes)
        )
    }

    fn shape_tree(shapes: &str) -> String {
        format!("<p:spTree><p:nvGrpSpPr/><p:grpSpPr/>{shapes}</p:spTree>")
    }

    fn shape(index: u32, list_style: &str) -> String {
        format!(
            "<p:sp><p:nvSpPr><p:cNvPr/><p:cNvSpPr/><p:nvPr><p:ph type=\"body\" idx=\"{index}\"/></p:nvPr></p:nvSpPr><p:spPr/><p:txBody><a:bodyPr/><a:lstStyle>{list_style}</a:lstStyle><a:p/></p:txBody></p:sp>"
        )
    }
}
