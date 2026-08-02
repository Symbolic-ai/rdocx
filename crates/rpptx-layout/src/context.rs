use std::cell::RefCell;
use std::collections::HashMap;

use oxml_drawing::color::ColorMap;
use oxml_drawing::text::{
    CT_TextBodyProperties, CT_TextListStyle, Coordinate32Value, TextAnchor, TextAutofit,
    TextVertical, TextWrap,
};
use oxml_drawing::theme::CT_OfficeStyleSheet;
use oxml_drawing::xfrm::CT_Transform2D;
use rpptx_oxml::placeholder::PlaceholderKey;
use rpptx_oxml::shape_tree::{CT_Shape, ShapeTreeChild};
use rpptx_oxml::slide_parts::{CT_Slide, CT_SlideLayout, CT_SlideMaster};

use crate::text::EffectiveListStyle;

/// The fixed presentation hierarchy and theme inputs for resolving one slide.
pub struct ResolveCtx<'a> {
    pub theme: &'a CT_OfficeStyleSheet,
    pub color_map: ColorMap,
    pub master: &'a CT_SlideMaster,
    pub layout: &'a CT_SlideLayout,
    pub slide: &'a CT_Slide,
    pub default_text_style: &'a CT_TextListStyle,
    pub(crate) list_style_cache: RefCell<HashMap<Option<PlaceholderKey>, EffectiveListStyle>>,
}

impl<'a> ResolveCtx<'a> {
    pub fn new(
        theme: &'a CT_OfficeStyleSheet,
        color_map: ColorMap,
        master: &'a CT_SlideMaster,
        layout: &'a CT_SlideLayout,
        slide: &'a CT_Slide,
        default_text_style: &'a CT_TextListStyle,
    ) -> Self {
        Self {
            theme,
            color_map,
            master,
            layout,
            slide,
            default_text_style,
            list_style_cache: RefCell::new(HashMap::new()),
        }
    }

    pub(crate) fn placeholder_chain<'ctx>(
        &'ctx self,
        shape: &CT_Shape,
    ) -> (Option<&'ctx CT_Shape>, Option<&'ctx CT_Shape>) {
        let Some(slide_key) = shape
            .placeholder
            .as_ref()
            .map(|placeholder| placeholder.key())
        else {
            return (None, None);
        };
        let Some(layout_shape) = find_placeholder(
            &self.layout.common_slide_data.shape_tree.children,
            &slide_key,
        ) else {
            return (None, None);
        };
        let Some(layout_key) = layout_shape
            .placeholder
            .as_ref()
            .map(|placeholder| placeholder.key())
        else {
            return (None, None);
        };
        let master_shape = find_placeholder(
            &self.master.common_slide_data.shape_tree.children,
            &layout_key,
        );
        (Some(layout_shape), master_shape)
    }

    /// Resolves an owned transform from the slide, layout, then master shape.
    pub fn effective_xfrm(&self, shape: &CT_Shape) -> Option<CT_Transform2D> {
        let (layout, master) = self.placeholder_chain(shape);
        shape
            .shape_properties
            .transform
            .clone()
            .or_else(|| layout.and_then(|shape| shape.shape_properties.transform.as_ref().cloned()))
            .or_else(|| master.and_then(|shape| shape.shape_properties.transform.as_ref().cloned()))
    }

    /// Resolves body properties per field over defaults, master, layout, and slide.
    pub fn effective_body_pr(&self, shape: &CT_Shape) -> CT_TextBodyProperties {
        let (layout, master) = self.placeholder_chain(shape);
        let mut effective = default_body_properties();
        for source in [master, layout, Some(shape)].into_iter().flatten() {
            if let Some(text_body) = &source.text_body {
                merge_body_properties(&mut effective, &text_body.body_properties);
            }
        }
        effective
    }
}

fn default_body_properties() -> CT_TextBodyProperties {
    let mut properties = CT_TextBodyProperties::default();
    properties.left_inset = Some(Coordinate32Value::Emu(91_440));
    properties.top_inset = Some(Coordinate32Value::Emu(45_720));
    properties.right_inset = Some(Coordinate32Value::Emu(91_440));
    properties.bottom_inset = Some(Coordinate32Value::Emu(45_720));
    properties.anchor = Some(TextAnchor::Top);
    properties.wrap = Some(TextWrap::Square);
    properties.vertical = Some(TextVertical::Horizontal);
    properties.autofit = Some(TextAutofit::NoAutofit);
    properties
}

fn merge_body_properties(target: &mut CT_TextBodyProperties, source: &CT_TextBodyProperties) {
    if let Some(value) = &source.left_inset {
        target.left_inset = Some(value.clone());
    }
    if let Some(value) = &source.top_inset {
        target.top_inset = Some(value.clone());
    }
    if let Some(value) = &source.right_inset {
        target.right_inset = Some(value.clone());
    }
    if let Some(value) = &source.bottom_inset {
        target.bottom_inset = Some(value.clone());
    }
    if let Some(value) = source.anchor {
        target.anchor = Some(value);
    }
    if let Some(value) = source.wrap {
        target.wrap = Some(value);
    }
    if let Some(value) = source.vertical {
        target.vertical = Some(value);
    }
    if let Some(value) = &source.autofit {
        target.autofit = Some(value.clone());
    }
}

fn find_placeholder<'a>(
    children: &'a [ShapeTreeChild],
    key: &PlaceholderKey,
) -> Option<&'a CT_Shape> {
    for child in children {
        let found = match child {
            ShapeTreeChild::Shape(shape) => shape
                .placeholder
                .as_ref()
                .is_some_and(|placeholder| key.matches(&placeholder.key()))
                .then_some(shape),
            ShapeTreeChild::GroupShape(group) => find_placeholder(&group.children, key),
            ShapeTreeChild::AlternateContent(alternate) => alternate
                .selected_fallback()
                .and_then(|fallback| find_placeholder(fallback, key)),
            _ => None,
        };
        if found.is_some() {
            return found;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use oxml_drawing::color::ColorMap;
    use oxml_drawing::text::{
        CT_TextListStyle, Coordinate32Value, TextAnchor, TextAutofit, TextVertical, TextWrap,
    };
    use oxml_drawing::theme::CT_OfficeStyleSheet;
    use rpptx_oxml::placeholder::PhType;
    use rpptx_oxml::shape_tree::{CT_Shape, ShapeTreeChild};
    use rpptx_oxml::slide_parts::{CT_Slide, CT_SlideLayout, CT_SlideMaster};

    use super::ResolveCtx;

    const P_NS: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";
    const A_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
    const MC_NS: &str = "http://schemas.openxmlformats.org/markup-compatibility/2006";

    struct Fixture {
        theme: CT_OfficeStyleSheet,
        master: CT_SlideMaster,
        layout: CT_SlideLayout,
        slide: CT_Slide,
        default_text_style: CT_TextListStyle,
    }

    impl Fixture {
        fn new(slide_children: &str, layout_children: &str, master_children: &str) -> Self {
            Self {
                theme: CT_OfficeStyleSheet::office_default(),
                master: CT_SlideMaster::from_xml(master_xml(master_children).as_bytes()).unwrap(),
                layout: CT_SlideLayout::from_xml(layout_xml(layout_children).as_bytes()).unwrap(),
                slide: CT_Slide::from_xml(slide_xml(slide_children).as_bytes()).unwrap(),
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

        fn slide_shape(&self, index: usize) -> &CT_Shape {
            let ShapeTreeChild::Shape(shape) =
                &self.slide.common_slide_data.shape_tree.children[index]
            else {
                panic!("expected an ordinary slide shape");
            };
            shape
        }
    }

    #[test]
    fn slide_placeholder_resolves_to_layout_and_master_counterparts() {
        let fixture = Fixture::new(
            &shape(Some("title"), Some(7)),
            &shape(Some("body"), Some(7)),
            &shape(Some("pic"), Some(7)),
        );

        let context = fixture.context();
        let (layout, master) = context.placeholder_chain(fixture.slide_shape(0));

        assert_eq!(
            layout.unwrap().placeholder.as_ref().unwrap().ph_type,
            Some(PhType::Body)
        );
        assert_eq!(
            master.unwrap().placeholder.as_ref().unwrap().ph_type,
            Some(PhType::Picture)
        );
    }

    #[test]
    fn master_match_uses_the_layout_placeholder_key() {
        let fixture = Fixture::new(
            &shape(Some("title"), Some(8)),
            &shape(Some("ctrTitle"), None),
            &shape(Some("title"), Some(5)),
        );

        let context = fixture.context();
        let (layout, master) = context.placeholder_chain(fixture.slide_shape(0));

        assert!(layout.is_some());
        assert_eq!(master.unwrap().placeholder.as_ref().unwrap().idx, Some(5));
    }

    #[test]
    fn placeholder_lookup_walks_groups_and_selected_fallbacks() {
        let nested_layout = format!(
            "<p:grpSp><p:nvGrpSpPr/><p:grpSpPr/>{}</p:grpSp>",
            shape(Some("body"), Some(11))
        );
        let fallback_master = format!(
            "<mc:AlternateContent><mc:Choice Requires=\"p14\"><p:sp/></mc:Choice><mc:Fallback>{}</mc:Fallback></mc:AlternateContent>",
            shape(Some("body"), Some(11))
        );
        let fixture = Fixture::new(
            &shape(Some("body"), Some(11)),
            &nested_layout,
            &fallback_master,
        );

        let context = fixture.context();
        let (layout, master) = context.placeholder_chain(fixture.slide_shape(0));

        assert!(layout.is_some());
        assert!(master.is_some());
    }

    #[test]
    fn missing_or_non_placeholder_shape_has_no_chain() {
        let slide_children = format!("{}{}", shape(None, None), shape(Some("body"), Some(42)));
        let fixture = Fixture::new(&slide_children, "", &shape(Some("body"), Some(42)));
        let context = fixture.context();

        assert_eq!(
            context.placeholder_chain(fixture.slide_shape(0)),
            (None, None)
        );
        assert_eq!(
            context.placeholder_chain(fixture.slide_shape(1)),
            (None, None)
        );
    }

    #[test]
    fn slide_placeholder_without_transform_inherits_layout_position() {
        let fixture = Fixture::new(
            &shape(Some("body"), Some(1)),
            &shape_with_details(Some("body"), Some(1), &transform(10), None),
            &shape_with_details(Some("body"), Some(1), &transform(20), None),
        );
        let transform = fixture
            .context()
            .effective_xfrm(fixture.slide_shape(0))
            .unwrap();

        assert_eq!(transform.offset.unwrap().x.0, 10);
    }

    #[test]
    fn effective_transform_uses_slide_layout_master_precedence() {
        let slide_children = [
            shape_with_details(Some("body"), Some(1), &transform(11), None),
            shape(Some("body"), Some(2)),
            shape(Some("body"), Some(3)),
            shape(Some("body"), Some(4)),
        ]
        .join("");
        let layout_children = [
            shape_with_details(Some("body"), Some(1), &transform(21), None),
            shape_with_details(Some("body"), Some(2), &transform(22), None),
            shape(Some("body"), Some(3)),
            shape(Some("body"), Some(4)),
        ]
        .join("");
        let master_children = [
            shape_with_details(Some("body"), Some(1), &transform(31), None),
            shape_with_details(Some("body"), Some(2), &transform(32), None),
            shape_with_details(Some("body"), Some(3), &transform(33), None),
            shape(Some("body"), Some(4)),
        ]
        .join("");
        let fixture = Fixture::new(&slide_children, &layout_children, &master_children);
        let context = fixture.context();

        let offsets: Vec<_> = (0..3)
            .map(|index| {
                context
                    .effective_xfrm(fixture.slide_shape(index))
                    .unwrap()
                    .offset
                    .unwrap()
                    .x
                    .0
            })
            .collect();

        assert_eq!(offsets, [11, 22, 33]);
        assert!(context.effective_xfrm(fixture.slide_shape(3)).is_none());
    }

    #[test]
    fn body_properties_merge_per_field_across_the_chain() {
        let fixture = Fixture::new(
            &shape_with_details(
                Some("body"),
                Some(4),
                "",
                Some(r#"<a:bodyPr bIns="50" anchor="ctr"/>"#),
            ),
            &shape_with_details(
                Some("body"),
                Some(4),
                "",
                Some(r#"<a:bodyPr lIns="30" rIns="40" wrap="none"/>"#),
            ),
            &shape_with_details(
                Some("body"),
                Some(4),
                "",
                Some(
                    r#"<a:bodyPr lIns="10" tIns="20" anchor="b" vert="vert"><a:spAutoFit/></a:bodyPr>"#,
                ),
            ),
        );
        let properties = fixture.context().effective_body_pr(fixture.slide_shape(0));

        assert_eq!(properties.left_inset, Some(Coordinate32Value::Emu(30)));
        assert_eq!(properties.top_inset, Some(Coordinate32Value::Emu(20)));
        assert_eq!(properties.right_inset, Some(Coordinate32Value::Emu(40)));
        assert_eq!(properties.bottom_inset, Some(Coordinate32Value::Emu(50)));
        assert_eq!(properties.anchor, Some(TextAnchor::Center));
        assert_eq!(properties.wrap, Some(TextWrap::None));
        assert_eq!(properties.vertical, Some(TextVertical::Vertical));
        assert_eq!(properties.autofit, Some(TextAutofit::ShapeAutofit));
    }

    #[test]
    fn body_property_defaults_use_exact_emu_values() {
        let fixture = Fixture::new(&shape(None, None), "", "");
        let properties = fixture.context().effective_body_pr(fixture.slide_shape(0));

        assert_eq!(properties.left_inset, Some(Coordinate32Value::Emu(91_440)));
        assert_eq!(properties.right_inset, Some(Coordinate32Value::Emu(91_440)));
        assert_eq!(properties.top_inset, Some(Coordinate32Value::Emu(45_720)));
        assert_eq!(
            properties.bottom_inset,
            Some(Coordinate32Value::Emu(45_720))
        );
        assert_eq!(properties.anchor, Some(TextAnchor::Top));
        assert_eq!(properties.wrap, Some(TextWrap::Square));
        assert_eq!(properties.vertical, Some(TextVertical::Horizontal));
        assert_eq!(properties.autofit, Some(TextAutofit::NoAutofit));
    }

    fn slide_xml(children: &str) -> String {
        format!(
            "<p:sld xmlns:p=\"{P_NS}\" xmlns:a=\"{A_NS}\" xmlns:mc=\"{MC_NS}\"><p:cSld>{}</p:cSld></p:sld>",
            shape_tree(children)
        )
    }

    fn layout_xml(children: &str) -> String {
        format!(
            "<p:sldLayout xmlns:p=\"{P_NS}\" xmlns:a=\"{A_NS}\" xmlns:mc=\"{MC_NS}\"><p:cSld>{}</p:cSld></p:sldLayout>",
            shape_tree(children)
        )
    }

    fn master_xml(children: &str) -> String {
        format!(
            "<p:sldMaster xmlns:p=\"{P_NS}\" xmlns:a=\"{A_NS}\" xmlns:mc=\"{MC_NS}\"><p:cSld>{}</p:cSld><p:clrMap bg1=\"lt1\" tx1=\"dk1\" bg2=\"lt2\" tx2=\"dk2\" accent1=\"accent1\" accent2=\"accent2\" accent3=\"accent3\" accent4=\"accent4\" accent5=\"accent5\" accent6=\"accent6\" hlink=\"hlink\" folHlink=\"folHlink\"/></p:sldMaster>",
            shape_tree(children)
        )
    }

    fn shape_tree(children: &str) -> String {
        format!("<p:spTree><p:nvGrpSpPr/><p:grpSpPr/>{children}</p:spTree>")
    }

    fn shape(ph_type: Option<&str>, idx: Option<u32>) -> String {
        shape_with_details(ph_type, idx, "", None)
    }

    fn shape_with_details(
        ph_type: Option<&str>,
        idx: Option<u32>,
        shape_properties: &str,
        body_properties: Option<&str>,
    ) -> String {
        let placeholder = if ph_type.is_none() && idx.is_none() {
            String::new()
        } else {
            let ph_type = ph_type.map_or_else(String::new, |value| format!(" type=\"{value}\""));
            let idx = idx.map_or_else(String::new, |value| format!(" idx=\"{value}\""));
            format!("<p:ph{ph_type}{idx}/>")
        };
        let text_body = body_properties.map_or_else(String::new, |body_properties| {
            format!("<p:txBody>{body_properties}<a:p/></p:txBody>")
        });
        format!(
            "<p:sp><p:nvSpPr><p:cNvPr/><p:cNvSpPr/><p:nvPr>{placeholder}</p:nvPr></p:nvSpPr><p:spPr>{shape_properties}</p:spPr>{text_body}</p:sp>"
        )
    }

    fn transform(x: i64) -> String {
        format!("<a:xfrm><a:off x=\"{x}\" y=\"0\"/><a:ext cx=\"100\" cy=\"100\"/></a:xfrm>")
    }
}
