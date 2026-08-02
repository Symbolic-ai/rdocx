use std::cell::RefCell;
use std::collections::HashMap;

use oxml_drawing::color::ColorMap;
use oxml_drawing::fill::Fill;
use oxml_drawing::text::{
    CT_TextBodyProperties, CT_TextListStyle, Coordinate32Value, TextAnchor, TextAutofit,
    TextVertical, TextWrap,
};
use oxml_drawing::theme::CT_OfficeStyleSheet;
use oxml_drawing::xfrm::CT_Transform2D;
use rpptx_oxml::placeholder::{CT_Placeholder, PhType, PlaceholderKey};
use rpptx_oxml::shape_tree::{CT_Shape, ShapeTreeChild};
use rpptx_oxml::slide_parts::{CT_Slide, CT_SlideLayout, CT_SlideMaster};

use crate::text::EffectiveListStyle;

/// The producer part that supplied the effective background.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackgroundSource {
    Slide,
    Layout,
    Master,
    ThemeFallback,
}

/// The borrowed background payload selected for one slide.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BackgroundContent<'a> {
    Xml(&'a [u8]),
    ThemeFill(&'a Fill),
}

/// The effective background and the per-master colour map used to resolve it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EffectiveBackground<'a> {
    pub source: BackgroundSource,
    pub content: BackgroundContent<'a>,
    pub color_map: &'a ColorMap,
}

/// The four ordered sources in a flattened slide view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlattenedSource {
    Background,
    Master,
    Layout,
    Slide,
}

/// One borrowed entry in final draw order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlattenedItem<'a> {
    Background(EffectiveBackground<'a>),
    Shape {
        source: FlattenedSource,
        child: &'a ShapeTreeChild,
    },
}

impl FlattenedItem<'_> {
    pub const fn source(&self) -> FlattenedSource {
        match self {
            Self::Background(_) => FlattenedSource::Background,
            Self::Shape { source, .. } => *source,
        }
    }
}

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

    /// Selects the first background in slide, layout, master, and theme order.
    pub fn effective_background(&self) -> Option<EffectiveBackground<'_>> {
        for (source, xml) in [
            (
                BackgroundSource::Slide,
                self.slide.common_slide_data.background_xml.as_deref(),
            ),
            (
                BackgroundSource::Layout,
                self.layout.common_slide_data.background_xml.as_deref(),
            ),
            (
                BackgroundSource::Master,
                self.master.common_slide_data.background_xml.as_deref(),
            ),
        ] {
            if let Some(xml) = xml {
                return Some(EffectiveBackground {
                    source,
                    content: BackgroundContent::Xml(xml),
                    color_map: &self.color_map,
                });
            }
        }
        self.theme
            .theme_elements
            .format_scheme
            .background_fill_styles
            .first()
            .map(|fill| EffectiveBackground {
                source: BackgroundSource::ThemeFallback,
                content: BackgroundContent::ThemeFill(fill),
                color_map: &self.color_map,
            })
    }

    /// Returns background and shape-tree leaves in final draw order.
    pub fn flatten(&self) -> Vec<FlattenedItem<'_>> {
        let slide_children = &self.slide.common_slide_data.shape_tree.children;
        let layout_children = &self.layout.common_slide_data.shape_tree.children;
        let master_children = &self.master.common_slide_data.shape_tree.children;
        let mut slide_latent = Vec::new();
        let mut layout_latent = Vec::new();
        collect_occupied_latent(slide_children, &mut slide_latent);
        collect_occupied_latent(layout_children, &mut layout_latent);

        let mut flattened = Vec::new();
        if let Some(background) = self.effective_background() {
            flattened.push(FlattenedItem::Background(background));
        }
        let allow_latent = LatentPolicy::from_context(self);
        let mut master_deeper = layout_latent.clone();
        master_deeper.extend(slide_latent.iter().cloned());
        emit_tree(
            master_children,
            PassRules {
                source: FlattenedSource::Master,
                emit_non_placeholders: self.layout.show_master_shapes.unwrap_or(true),
                deeper_latent: &master_deeper,
                latent_policy: allow_latent,
            },
            &mut flattened,
        );
        emit_tree(
            layout_children,
            PassRules {
                source: FlattenedSource::Layout,
                emit_non_placeholders: self.slide.show_master_shapes.unwrap_or(true),
                deeper_latent: &slide_latent,
                latent_policy: allow_latent,
            },
            &mut flattened,
        );
        emit_tree(
            slide_children,
            PassRules {
                source: FlattenedSource::Slide,
                emit_non_placeholders: true,
                deeper_latent: &[],
                latent_policy: allow_latent,
            },
            &mut flattened,
        );
        flattened
    }
}

#[derive(Clone, Copy)]
struct LatentPolicy {
    date_time: bool,
    footer: bool,
    slide_number: bool,
}

impl LatentPolicy {
    fn from_context(context: &ResolveCtx<'_>) -> Self {
        let layout = context.layout.header_footer.as_ref();
        let master = context.master.header_footer.as_ref();
        Self {
            date_time: layout.is_none_or(|hf| hf.date_time_enabled())
                && master.is_none_or(|hf| hf.date_time_enabled()),
            footer: layout.is_none_or(|hf| hf.footer_enabled())
                && master.is_none_or(|hf| hf.footer_enabled()),
            slide_number: layout.is_none_or(|hf| hf.slide_number_enabled())
                && master.is_none_or(|hf| hf.slide_number_enabled()),
        }
    }

    fn permits(self, ph_type: &PhType) -> bool {
        match ph_type {
            PhType::DateTime => self.date_time,
            PhType::Footer => self.footer,
            PhType::SlideNumber => self.slide_number,
            _ => true,
        }
    }
}

#[derive(Clone, Copy)]
struct PassRules<'a> {
    source: FlattenedSource,
    emit_non_placeholders: bool,
    deeper_latent: &'a [PlaceholderKey],
    latent_policy: LatentPolicy,
}

fn collect_occupied_latent(children: &[ShapeTreeChild], keys: &mut Vec<PlaceholderKey>) {
    for child in children {
        match child {
            ShapeTreeChild::GroupShape(group) => collect_occupied_latent(&group.children, keys),
            ShapeTreeChild::AlternateContent(alternate) => {
                if let Some(fallback) = alternate.selected_fallback() {
                    collect_occupied_latent(fallback, keys);
                }
            }
            ShapeTreeChild::Shape(shape) => {
                if let Some(placeholder) = shape.placeholder.as_ref()
                    && is_latent(&placeholder.effective_type())
                    && shape
                        .text_body
                        .as_ref()
                        .is_some_and(|body| !body.plain_text().trim().is_empty())
                {
                    push_unmatched(keys, placeholder.key());
                }
            }
            _ => {}
        }
    }
}

fn push_unmatched(keys: &mut Vec<PlaceholderKey>, key: PlaceholderKey) {
    if !keys.iter().any(|existing| key.matches(existing)) {
        keys.push(key);
    }
}

fn emit_tree<'a>(
    children: &'a [ShapeTreeChild],
    rules: PassRules<'_>,
    output: &mut Vec<FlattenedItem<'a>>,
) {
    let mut emitted_latent = Vec::new();
    emit_tree_inner(children, rules, &mut emitted_latent, output);
}

fn emit_tree_inner<'a>(
    children: &'a [ShapeTreeChild],
    rules: PassRules<'_>,
    emitted_latent: &mut Vec<PlaceholderKey>,
    output: &mut Vec<FlattenedItem<'a>>,
) {
    for child in children {
        match child {
            ShapeTreeChild::GroupShape(group) => {
                emit_tree_inner(&group.children, rules, emitted_latent, output);
            }
            ShapeTreeChild::AlternateContent(alternate) => {
                if let Some(fallback) = alternate.selected_fallback() {
                    emit_tree_inner(fallback, rules, emitted_latent, output);
                }
            }
            _ => emit_leaf(child, rules, emitted_latent, output),
        }
    }
}

fn emit_leaf<'a>(
    child: &'a ShapeTreeChild,
    rules: PassRules<'_>,
    emitted_latent: &mut Vec<PlaceholderKey>,
    output: &mut Vec<FlattenedItem<'a>>,
) {
    let placeholder = child_placeholder(child);
    if rules.source == FlattenedSource::Slide {
        if let Some(placeholder) = placeholder {
            let ph_type = placeholder.effective_type();
            if is_latent(&ph_type) {
                let occupied = child_is_occupied(child);
                let already_emitted = emitted_latent
                    .iter()
                    .any(|key| placeholder.key().matches(key));
                if !rules.latent_policy.permits(&ph_type) || !occupied || already_emitted {
                    return;
                }
                push_unmatched(emitted_latent, placeholder.key());
            }
        }
        output.push(FlattenedItem::Shape {
            source: rules.source,
            child,
        });
        return;
    }

    let Some(placeholder) = placeholder else {
        if rules.emit_non_placeholders {
            output.push(FlattenedItem::Shape {
                source: rules.source,
                child,
            });
        }
        return;
    };
    let ph_type = placeholder.effective_type();
    let key = placeholder.key();
    if !is_latent(&ph_type)
        || !rules.latent_policy.permits(&ph_type)
        || !child_is_occupied(child)
        || rules.deeper_latent.iter().any(|deeper| key.matches(deeper))
        || emitted_latent.iter().any(|emitted| key.matches(emitted))
    {
        return;
    }
    push_unmatched(emitted_latent, key);
    output.push(FlattenedItem::Shape {
        source: rules.source,
        child,
    });
}

fn child_placeholder(child: &ShapeTreeChild) -> Option<&CT_Placeholder> {
    match child {
        ShapeTreeChild::Shape(shape) => shape.placeholder.as_ref(),
        ShapeTreeChild::Picture(picture) => picture.placeholder.as_ref(),
        _ => None,
    }
}

fn child_is_occupied(child: &ShapeTreeChild) -> bool {
    match child {
        ShapeTreeChild::Shape(shape) => shape
            .text_body
            .as_ref()
            .is_some_and(|body| !body.plain_text().trim().is_empty()),
        ShapeTreeChild::Picture(picture) => picture.blip_fill.is_some(),
        _ => false,
    }
}

fn is_latent(ph_type: &PhType) -> bool {
    matches!(
        ph_type,
        PhType::DateTime | PhType::Footer | PhType::SlideNumber
    )
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

    use super::{BackgroundSource, FlattenedItem, FlattenedSource, ResolveCtx};

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
            Self::from_xml(
                &slide_xml(slide_children),
                &layout_xml(layout_children),
                &master_xml(master_children),
            )
        }

        fn from_xml(slide_xml: &str, layout_xml: &str, master_xml: &str) -> Self {
            Self {
                theme: CT_OfficeStyleSheet::office_default(),
                master: CT_SlideMaster::from_xml(master_xml.as_bytes()).unwrap(),
                layout: CT_SlideLayout::from_xml(layout_xml.as_bytes()).unwrap(),
                slide: CT_Slide::from_xml(slide_xml.as_bytes()).unwrap(),
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

    #[test]
    fn flattener_omits_template_prompt_and_emits_master_logo_once() {
        let fixture = Fixture::new(
            &shape_with_text(Some("title"), Some(1), "Slide title"),
            "",
            &[
                shape_with_text(Some("title"), Some(1), "Click to edit Master title style"),
                shape_with_text(None, None, "Master logo"),
            ]
            .join(""),
        );

        let texts: Vec<_> = fixture
            .context()
            .flatten()
            .iter()
            .filter_map(item_text)
            .collect();

        assert!(!texts.contains(&"Click to edit Master title style".to_owned()));
        assert_eq!(
            texts.iter().filter(|text| *text == "Master logo").count(),
            1
        );
    }

    #[test]
    fn flattener_emits_the_four_sources_in_draw_order() {
        let master_group = format!(
            "<p:grpSp><p:nvGrpSpPr/><p:grpSpPr/>{}</p:grpSp>",
            shape_with_text(None, None, "master")
        );
        let layout_fallback = format!(
            "<mc:AlternateContent><mc:Fallback>{}</mc:Fallback></mc:AlternateContent>",
            shape_with_text(None, None, "layout")
        );
        let fixture = Fixture::from_xml(
            &slide_xml_with(
                "",
                "<p:bg><p:bgPr/></p:bg>",
                &shape_with_text(None, None, "slide"),
            ),
            &layout_xml_with("", "", &layout_fallback, ""),
            &master_xml_with("", &master_group, ""),
        );
        let context = fixture.context();
        let flattened = context.flatten();

        assert_eq!(
            flattened
                .iter()
                .map(FlattenedItem::source)
                .collect::<Vec<_>>(),
            [
                FlattenedSource::Background,
                FlattenedSource::Master,
                FlattenedSource::Layout,
                FlattenedSource::Slide,
            ]
        );
        let FlattenedItem::Background(background) = flattened[0] else {
            panic!("expected background first");
        };
        assert_eq!(background.source, BackgroundSource::Slide);
    }

    #[test]
    fn effective_background_uses_fallback_order_and_master_color_map() {
        let slide_first = Fixture::from_xml(
            &slide_xml_with("", "<p:bg><p:bgPr/></p:bg>", ""),
            &layout_xml_with("", "<p:bg><p:bgPr/></p:bg>", "", ""),
            &master_xml_with("<p:bg><p:bgPr/></p:bg>", "", ""),
        );
        let layout_first = Fixture::from_xml(
            &slide_xml_with("", "", ""),
            &layout_xml_with("", "<p:bg><p:bgPr/></p:bg>", "", ""),
            &master_xml_with("<p:bg><p:bgPr/></p:bg>", "", ""),
        );
        let master_first = Fixture::from_xml(
            &slide_xml_with("", "", ""),
            &layout_xml_with("", "", "", ""),
            &master_xml_with("<p:bg><p:bgPr/></p:bg>", "", ""),
        );
        let theme_fallback = Fixture::new("", "", "");

        assert_eq!(background_source(&slide_first), BackgroundSource::Slide);
        assert_eq!(background_source(&layout_first), BackgroundSource::Layout);
        assert_eq!(background_source(&master_first), BackgroundSource::Master);
        assert_eq!(
            background_source(&theme_fallback),
            BackgroundSource::ThemeFallback
        );
        let context = theme_fallback.context();
        let background = context.effective_background().unwrap();
        assert!(std::ptr::eq(background.color_map, &context.color_map));
    }

    #[test]
    fn show_master_shapes_suppresses_the_owned_pass() {
        let master_suppressed = Fixture::from_xml(
            &slide_xml_with("", "", &shape_with_text(None, None, "slide")),
            &layout_xml_with(
                "showMasterSp=\"0\"",
                "",
                &shape_with_text(None, None, "layout"),
                "",
            ),
            &master_xml_with("", &shape_with_text(None, None, "master"), ""),
        );
        let layout_suppressed = Fixture::from_xml(
            &slide_xml_with(
                "showMasterSp=\"0\"",
                "",
                &shape_with_text(None, None, "slide"),
            ),
            &layout_xml_with("", "", &shape_with_text(None, None, "layout"), ""),
            &master_xml_with("", &shape_with_text(None, None, "master"), ""),
        );

        let master_suppressed_texts: Vec<_> = master_suppressed
            .context()
            .flatten()
            .iter()
            .filter_map(item_text)
            .collect();
        let layout_suppressed_texts: Vec<_> = layout_suppressed
            .context()
            .flatten()
            .iter()
            .filter_map(item_text)
            .collect();

        assert_eq!(master_suppressed_texts, ["layout", "slide"]);
        assert_eq!(layout_suppressed_texts, ["master", "slide"]);
    }

    #[test]
    fn slide_placeholder_suppresses_layout_and_master_matches() {
        let fixture = Fixture::from_xml(
            &slide_xml_with(
                "",
                "",
                &shape_with_text(Some("ftr"), Some(9), "slide footer"),
            ),
            &layout_xml_with(
                "",
                "",
                &shape_with_text(Some("ftr"), Some(9), "layout footer"),
                "<p:hf/>",
            ),
            &master_xml_with(
                "",
                &shape_with_text(Some("ftr"), Some(9), "master footer"),
                "<p:hf/>",
            ),
        );

        let texts: Vec<_> = fixture
            .context()
            .flatten()
            .iter()
            .filter_map(item_text)
            .collect();

        assert_eq!(texts, ["slide footer"]);
    }

    #[test]
    fn latent_placeholders_obey_header_footer_flags() {
        let latent_shapes = [
            shape_with_text(Some("dt"), Some(1), "date"),
            shape_with_text(Some("ftr"), Some(2), "footer"),
            shape_with_text(Some("sldNum"), Some(3), "number"),
        ]
        .join("");
        let fixture = Fixture::from_xml(
            &slide_xml_with("", "", ""),
            &layout_xml_with("", "", &latent_shapes, "<p:hf dt=\"0\"/>"),
            &master_xml_with("", "", "<p:hf sldNum=\"0\"/>"),
        );

        let texts: Vec<_> = fixture
            .context()
            .flatten()
            .iter()
            .filter_map(item_text)
            .collect();

        assert_eq!(texts, ["footer"]);
    }

    fn slide_xml(children: &str) -> String {
        slide_xml_with("", "", children)
    }

    fn layout_xml(children: &str) -> String {
        layout_xml_with("", "", children, "")
    }

    fn master_xml(children: &str) -> String {
        master_xml_with("", children, "")
    }

    fn slide_xml_with(attributes: &str, background: &str, children: &str) -> String {
        format!(
            "<p:sld xmlns:p=\"{P_NS}\" xmlns:a=\"{A_NS}\" xmlns:mc=\"{MC_NS}\" {attributes}><p:cSld>{background}{}</p:cSld></p:sld>",
            shape_tree(children)
        )
    }

    fn layout_xml_with(
        attributes: &str,
        background: &str,
        children: &str,
        header_footer: &str,
    ) -> String {
        format!(
            "<p:sldLayout xmlns:p=\"{P_NS}\" xmlns:a=\"{A_NS}\" xmlns:mc=\"{MC_NS}\" {attributes}><p:cSld>{background}{}</p:cSld>{header_footer}</p:sldLayout>",
            shape_tree(children)
        )
    }

    fn master_xml_with(background: &str, children: &str, header_footer: &str) -> String {
        format!(
            "<p:sldMaster xmlns:p=\"{P_NS}\" xmlns:a=\"{A_NS}\" xmlns:mc=\"{MC_NS}\"><p:cSld>{background}{}</p:cSld><p:clrMap bg1=\"lt1\" tx1=\"dk1\" bg2=\"lt2\" tx2=\"dk2\" accent1=\"accent1\" accent2=\"accent2\" accent3=\"accent3\" accent4=\"accent4\" accent5=\"accent5\" accent6=\"accent6\" hlink=\"hlink\" folHlink=\"folHlink\"/>{header_footer}</p:sldMaster>",
            shape_tree(children)
        )
    }

    fn shape_tree(children: &str) -> String {
        format!("<p:spTree><p:nvGrpSpPr/><p:grpSpPr/>{children}</p:spTree>")
    }

    fn shape(ph_type: Option<&str>, idx: Option<u32>) -> String {
        shape_with_details(ph_type, idx, "", None)
    }

    fn shape_with_text(ph_type: Option<&str>, idx: Option<u32>, text: &str) -> String {
        let placeholder = if ph_type.is_none() && idx.is_none() {
            String::new()
        } else {
            let ph_type = ph_type.map_or_else(String::new, |value| format!(" type=\"{value}\""));
            let idx = idx.map_or_else(String::new, |value| format!(" idx=\"{value}\""));
            format!("<p:ph{ph_type}{idx}/>")
        };
        format!(
            "<p:sp><p:nvSpPr><p:cNvPr/><p:cNvSpPr/><p:nvPr>{placeholder}</p:nvPr></p:nvSpPr><p:spPr/><p:txBody><a:bodyPr/><a:p><a:r><a:t>{text}</a:t></a:r></a:p></p:txBody></p:sp>"
        )
    }

    fn item_text(item: &FlattenedItem<'_>) -> Option<String> {
        let FlattenedItem::Shape {
            child: ShapeTreeChild::Shape(shape),
            ..
        } = item
        else {
            return None;
        };
        shape.text_body.as_ref().map(|body| body.plain_text())
    }

    fn background_source(fixture: &Fixture) -> BackgroundSource {
        fixture.context().effective_background().unwrap().source
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
