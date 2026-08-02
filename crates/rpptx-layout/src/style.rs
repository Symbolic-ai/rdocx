//! Shape-style inheritance resolution.

use std::fmt;

use oxml_drawing::color::ColorChoice;
use oxml_drawing::effect::CT_EffectList;
use oxml_drawing::fill::Fill;
use oxml_drawing::line::CT_LineProperties;
use oxml_drawing::style_ref::FontCollectionIndex;
use rpptx_oxml::shape_tree::CT_Shape;

use crate::ResolveCtx;

/// The modelled shape-style properties after theme lookup and direct overlays.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EffectiveShapeStyle {
    pub fill: Option<Fill>,
    pub line: Option<CT_LineProperties>,
    pub effects: Option<CT_EffectList>,
    pub font_collection: Option<FontCollectionIndex>,
}

/// Errors produced while resolving a shape against its theme format scheme.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolveError {
    StyleIndexOutOfRange {
        reference: &'static str,
        index: u32,
        available: usize,
    },
    UnresolvedPlaceholderColor {
        reference: &'static str,
    },
}

impl fmt::Display for ResolveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StyleIndexOutOfRange {
                reference,
                index,
                available,
            } => write!(
                formatter,
                "{reference} style index {index} is outside the {available} available entries"
            ),
            Self::UnresolvedPlaceholderColor { reference } => write!(
                formatter,
                "{reference} style retains an unresolved phClr placeholder colour"
            ),
        }
    }
}

impl std::error::Error for ResolveError {}

impl ResolveCtx<'_> {
    /// Resolves one ordinary shape's modelled theme and direct style properties.
    pub fn effective_shape_style(
        &self,
        shape: &CT_Shape,
    ) -> Result<EffectiveShapeStyle, ResolveError> {
        let matrix = &self.theme.theme_elements.format_scheme;
        let referenced_effect_style = match shape.style() {
            Some(style) => matrix_entry(
                "effect",
                style.effect_reference.index,
                &matrix.effect_styles,
            )?,
            None => None,
        };
        let mut effective = if let Some(style) = shape.style() {
            EffectiveShapeStyle {
                fill: referenced_fill(
                    style.fill_reference.index,
                    &matrix.fill_styles,
                    &matrix.background_fill_styles,
                )?,
                line: matrix_entry("line", style.line_reference.index, &matrix.line_styles)?,
                effects: referenced_effect_style
                    .as_ref()
                    .filter(|style| !style.has_unmodelled_effect())
                    .and_then(|style| style.effect_list.clone()),
                font_collection: Some(style.font_reference.index),
            }
        } else {
            EffectiveShapeStyle::default()
        };

        if let Some(explicit) = &shape.shape_properties.fill {
            effective.fill = Some(explicit.clone());
        }
        if let Some(explicit) = &shape.shape_properties.line {
            if let Some(line) = effective.line.as_mut() {
                overlay_line(line, explicit);
            } else {
                effective.line = Some(explicit.clone());
            }
        }
        let has_explicit_unmodelled_effect = shape.shape_properties.has_unmodelled_effect();
        if has_explicit_unmodelled_effect {
            effective.effects = None;
        } else if let Some(explicit) = &shape.shape_properties.effects {
            effective.effects = Some(explicit.clone());
        }

        let fill_reference = shape
            .style()
            .and_then(|style| style.fill_reference.color.as_ref());
        let line_reference = shape
            .style()
            .and_then(|style| style.line_reference.color.as_ref());
        let effect_reference = shape
            .style()
            .and_then(|style| style.effect_reference.color.as_ref());
        let unmodelled_effect_has_placeholder = if has_explicit_unmodelled_effect {
            shape
                .shape_properties
                .has_unmodelled_effect_placeholder_color()
        } else {
            referenced_effect_style
                .as_ref()
                .is_some_and(|style| style.has_unmodelled_effect_placeholder_color())
        };
        if unmodelled_effect_has_placeholder {
            return Err(ResolveError::UnresolvedPlaceholderColor {
                reference: "effect",
            });
        }
        if let Some(fill) = effective.fill.as_mut() {
            substitute_fill(fill, fill_reference, "fill")?;
        }
        if let Some(line) = effective.line.as_mut()
            && let Some(fill) = line.fill.as_mut()
        {
            substitute_fill(fill, line_reference, "line")?;
        }
        if let Some(effects) = effective.effects.as_mut() {
            if let Some(shadow) = effects.outer_shadow.as_mut()
                && let Some(color) = shadow.color.as_mut()
            {
                substitute_color(color, effect_reference, "effect")?;
            }
            if effects.has_unmodelled_placeholder_color() {
                return Err(ResolveError::UnresolvedPlaceholderColor {
                    reference: "effect",
                });
            }
        }
        Ok(effective)
    }
}

fn matrix_entry<T: Clone>(
    reference: &'static str,
    index: u32,
    entries: &[T],
) -> Result<Option<T>, ResolveError> {
    if index == 0 {
        return Ok(None);
    }
    let offset = usize::try_from(index - 1).ok();
    offset
        .and_then(|offset| entries.get(offset))
        .cloned()
        .map(Some)
        .ok_or(ResolveError::StyleIndexOutOfRange {
            reference,
            index,
            available: entries.len(),
        })
}

fn referenced_fill(
    index: u32,
    normal: &[Fill],
    background: &[Fill],
) -> Result<Option<Fill>, ResolveError> {
    if index > 1000 {
        matrix_entry("background fill", index - 1000, background)
    } else {
        matrix_entry("fill", index, normal)
    }
}

fn overlay_line(target: &mut CT_LineProperties, source: &CT_LineProperties) {
    if source.width.is_some() {
        target.width = source.width;
    }
    if source.cap.is_some() {
        target.cap = source.cap;
    }
    if source.fill.is_some() {
        target.fill.clone_from(&source.fill);
    }
    if source.dash.is_some() {
        target.dash.clone_from(&source.dash);
    }
    if source.join.is_some() {
        target.join.clone_from(&source.join);
    }
    if source.head_end.is_some() {
        target.head_end.clone_from(&source.head_end);
    }
    if source.tail_end.is_some() {
        target.tail_end.clone_from(&source.tail_end);
    }
}

fn substitute_fill(
    fill: &mut Fill,
    reference: Option<&ColorChoice>,
    kind: &'static str,
) -> Result<(), ResolveError> {
    match fill {
        Fill::Solid(fill) => substitute_optional_color(&mut fill.color, reference, kind),
        Fill::Gradient(fill) => {
            for stop in &mut fill.stops {
                substitute_optional_color(&mut stop.color, reference, kind)?;
            }
            Ok(())
        }
        Fill::Pattern(fill) => {
            substitute_optional_color(&mut fill.foreground, reference, kind)?;
            substitute_optional_color(&mut fill.background, reference, kind)
        }
        Fill::NoFill(_) | Fill::Blip(_) => Ok(()),
    }
}

fn substitute_optional_color(
    color: &mut Option<ColorChoice>,
    reference: Option<&ColorChoice>,
    kind: &'static str,
) -> Result<(), ResolveError> {
    if let Some(color) = color {
        substitute_color(color, reference, kind)?;
    }
    Ok(())
}

fn substitute_color(
    color: &mut ColorChoice,
    reference: Option<&ColorChoice>,
    kind: &'static str,
) -> Result<(), ResolveError> {
    if !matches!(color, ColorChoice::Scheme { value, .. } if value == "phClr") {
        return Ok(());
    }
    let placeholder_transforms = color.transforms().to_vec();
    let Some(mut replacement) = reference.cloned() else {
        return Err(ResolveError::UnresolvedPlaceholderColor { reference: kind });
    };
    if matches!(&replacement, ColorChoice::Scheme { value, .. } if value == "phClr") {
        return Err(ResolveError::UnresolvedPlaceholderColor { reference: kind });
    }
    transforms_mut(&mut replacement).extend(placeholder_transforms);
    *color = replacement;
    Ok(())
}

fn transforms_mut(color: &mut ColorChoice) -> &mut Vec<oxml_drawing::color::ColorTransform> {
    match color {
        ColorChoice::Srgb { transforms, .. }
        | ColorChoice::Scheme { transforms, .. }
        | ColorChoice::System { transforms, .. }
        | ColorChoice::Preset { transforms, .. } => transforms,
    }
}

#[cfg(test)]
mod tests {
    use oxml_drawing::color::{ColorChoice, ColorMap, ColorTransform};
    use oxml_drawing::fill::Fill;
    use oxml_drawing::line::{LineCap, LineDash, ST_PresetLineDashVal};
    use oxml_drawing::style_ref::FontCollectionIndex;
    use oxml_drawing::text::CT_TextListStyle;
    use oxml_drawing::theme::CT_OfficeStyleSheet;
    use rpptx_oxml::shape_tree::{CT_Shape, ShapeTreeChild};
    use rpptx_oxml::slide_parts::{CT_Slide, CT_SlideLayout, CT_SlideMaster};

    use super::{ResolveCtx, ResolveError};

    const P_NS: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";
    const A_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";

    #[test]
    fn shape_with_style_resolves_theme_fill_with_reference_colour_substituted() {
        let fixture = Fixture::new(&shape("", &style_refs(0, 1, 0, "minor", "AABBCC")));
        let resolved = fixture
            .context()
            .effective_shape_style(fixture.shape(0))
            .unwrap();

        let Some(Fill::Solid(fill)) = resolved.fill else {
            panic!("expected a resolved solid fill");
        };
        let Some(ColorChoice::Srgb {
            value, transforms, ..
        }) = fill.color
        else {
            panic!("expected the reference sRGB colour");
        };
        assert_eq!(value.to_string(), "AABBCC");
        assert!(transforms.is_empty());
        assert_eq!(resolved.font_collection, Some(FontCollectionIndex::Minor));
    }

    #[test]
    fn fill_ref_1001_selects_first_background_fill() {
        let fixture = Fixture::new(&shape("", &style_refs(0, 1001, 0, "major", "102030")));
        let resolved = fixture
            .context()
            .effective_shape_style(fixture.shape(0))
            .unwrap();

        let Some(Fill::Solid(fill)) = resolved.fill else {
            panic!("expected the first background solid fill");
        };
        assert!(matches!(
            fill.color,
            Some(ColorChoice::Srgb { value, .. }) if value.to_string() == "102030"
        ));
    }

    #[test]
    fn style_matrix_zero_is_none_and_out_of_range_is_error() {
        let shapes = [
            shape("", &style_refs(0, 0, 0, "none", "112233")),
            shape("", &style_refs(0, 4, 0, "none", "112233")),
        ]
        .join("");
        let fixture = Fixture::new(&shapes);

        let zero = fixture
            .context()
            .effective_shape_style(fixture.shape(0))
            .unwrap();
        assert!(zero.fill.is_none());
        assert!(zero.line.is_none());
        assert!(zero.effects.is_none());
        assert_eq!(zero.font_collection, Some(FontCollectionIndex::None));

        assert_eq!(
            fixture
                .context()
                .effective_shape_style(fixture.shape(1))
                .unwrap_err(),
            ResolveError::StyleIndexOutOfRange {
                reference: "fill",
                index: 4,
                available: 3,
            }
        );
    }

    #[test]
    fn placeholder_colour_substitution_keeps_transform_order() {
        let mut fixture = Fixture::new(&shape(
            "",
            &style_refs_with_fill_colour(
                1,
                r#"<a:schemeClr val="accent2"><a:shade val="80000"/></a:schemeClr>"#,
            ),
        ));
        fixture.theme.theme_elements.format_scheme.fill_styles[0] = Fill::from_xml(
            br#"<a:solidFill><a:schemeClr val="phClr"><a:tint val="30000"/></a:schemeClr></a:solidFill>"#,
        )
        .unwrap();

        let resolved = fixture
            .context()
            .effective_shape_style(fixture.shape(0))
            .unwrap();
        let Some(Fill::Solid(fill)) = resolved.fill else {
            panic!("expected a solid fill");
        };
        let Some(ColorChoice::Scheme {
            value, transforms, ..
        }) = fill.color
        else {
            panic!("expected a substituted scheme colour");
        };
        assert_eq!(value, "accent2");
        assert!(matches!(
            transforms.as_slice(),
            [ColorTransform::Shade(shade), ColorTransform::Tint(tint)]
                if shade.0 == 80_000 && tint.0 == 30_000
        ));
    }

    #[test]
    fn explicit_shape_properties_overlay_the_theme_style() {
        let fixture = Fixture::new(&shape(
            r#"<a:noFill/><a:ln w="25400"><a:prstDash val="dot"/></a:ln><a:effectLst/>"#,
            &style_refs(2, 1, 3, "minor", "445566"),
        ));
        let resolved = fixture
            .context()
            .effective_shape_style(fixture.shape(0))
            .unwrap();

        assert!(matches!(resolved.fill, Some(Fill::NoFill(_))));
        let line = resolved.line.unwrap();
        assert_eq!(line.width, Some(25_400));
        assert_eq!(line.cap, Some(LineCap::Flat));
        assert!(matches!(
            line.dash,
            Some(LineDash::Preset(ref dash)) if dash.value == ST_PresetLineDashVal::Dot
        ));
        assert!(matches!(
            line.fill,
            Some(Fill::Solid(fill))
                if matches!(fill.color, Some(ColorChoice::Srgb { value, .. }) if value.to_string() == "445566")
        ));
        assert!(resolved.effects.unwrap().outer_shadow.is_none());
        assert_eq!(resolved.font_collection, Some(FontCollectionIndex::Minor));
    }

    #[test]
    fn modelled_effect_placeholder_colour_is_substituted() {
        let mut fixture = Fixture::new(&shape("", &style_refs(0, 0, 3, "none", "667788")));
        let Fill::Solid(fill) = Fill::from_xml(
            br#"<a:solidFill><a:schemeClr val="phClr"><a:tint val="25000"/></a:schemeClr></a:solidFill>"#,
        )
        .unwrap()
        else {
            unreachable!();
        };
        fixture.theme.theme_elements.format_scheme.effect_styles[2]
            .effect_list
            .as_mut()
            .unwrap()
            .outer_shadow
            .as_mut()
            .unwrap()
            .color = fill.color;

        let resolved = fixture
            .context()
            .effective_shape_style(fixture.shape(0))
            .unwrap();
        let color = resolved
            .effects
            .unwrap()
            .outer_shadow
            .unwrap()
            .color
            .unwrap();
        assert!(matches!(
            color,
            ColorChoice::Srgb { value, transforms, .. }
                if value.to_string() == "667788"
                    && matches!(transforms.as_slice(), [ColorTransform::Tint(tint)] if tint.0 == 25_000)
        ));
    }

    #[test]
    fn unmodelled_effect_with_placeholder_colour_is_rejected() {
        let fixture = Fixture::new(&shape(
            r#"<a:effectLst><a:glow rad="100"><a:schemeClr val="phClr"/></a:glow></a:effectLst>"#,
            &style_refs(0, 0, 0, "none", "778899"),
        ));

        assert_eq!(
            fixture
                .context()
                .effective_shape_style(fixture.shape(0))
                .unwrap_err(),
            ResolveError::UnresolvedPlaceholderColor {
                reference: "effect",
            }
        );
    }

    #[test]
    fn explicit_opaque_effect_dag_replaces_theme_effect() {
        let fixture = Fixture::new(&shape(
            r#"<a:effectDag><a:cont/></a:effectDag>"#,
            &style_refs(0, 0, 3, "none", "778899"),
        ));

        let resolved = fixture
            .context()
            .effective_shape_style(fixture.shape(0))
            .unwrap();
        assert!(resolved.effects.is_none());
    }

    #[test]
    fn explicit_opaque_effect_dag_with_placeholder_colour_is_rejected() {
        let fixture = Fixture::new(&shape(
            r#"<a:effectDag><a:cont><a:effectLst><a:glow rad="100"><a:schemeClr val="phClr"/></a:glow></a:effectLst></a:cont></a:effectDag>"#,
            &style_refs(0, 0, 3, "none", "778899"),
        ));

        assert_eq!(
            fixture
                .context()
                .effective_shape_style(fixture.shape(0))
                .unwrap_err(),
            ResolveError::UnresolvedPlaceholderColor {
                reference: "effect",
            }
        );
    }

    #[test]
    fn theme_opaque_effect_dag_with_placeholder_colour_is_rejected() {
        let mut fixture = Fixture::new(&shape("", &style_refs(0, 0, 1, "none", "778899")));
        let xml = String::from_utf8(fixture.theme.to_xml().unwrap()).unwrap();
        let xml = xml.replacen(
            "<a:effectLst/>",
            r#"<a:effectDag><a:cont><a:effectLst><a:glow rad="100"><a:schemeClr val="phClr"/></a:glow></a:effectLst></a:cont></a:effectDag>"#,
            1,
        );
        fixture.theme = CT_OfficeStyleSheet::from_xml(xml.as_bytes()).unwrap();

        assert_eq!(
            fixture
                .context()
                .effective_shape_style(fixture.shape(0))
                .unwrap_err(),
            ResolveError::UnresolvedPlaceholderColor {
                reference: "effect",
            }
        );
    }

    #[test]
    fn foreign_namespace_effect_dag_does_not_replace_theme_effect() {
        let fixture = Fixture::new(&shape(
            r#"<x:effectDag xmlns:x="urn:extension"><x:schemeClr val="phClr"/></x:effectDag>"#,
            &style_refs(0, 0, 3, "none", "778899"),
        ));

        let resolved = fixture
            .context()
            .effective_shape_style(fixture.shape(0))
            .unwrap();
        assert!(resolved.effects.is_some());
    }

    #[test]
    fn foreign_namespace_scheme_color_is_not_a_placeholder_color() {
        let fixture = Fixture::new(&shape(
            r#"<a:effectDag><a:cont><x:schemeClr xmlns:x="urn:extension" val="phClr"/></a:cont></a:effectDag>"#,
            &style_refs(0, 0, 3, "none", "778899"),
        ));

        let resolved = fixture
            .context()
            .effective_shape_style(fixture.shape(0))
            .unwrap();
        assert!(resolved.effects.is_none());
    }

    struct Fixture {
        theme: CT_OfficeStyleSheet,
        master: CT_SlideMaster,
        layout: CT_SlideLayout,
        slide: CT_Slide,
        default_text_style: CT_TextListStyle,
    }

    impl Fixture {
        fn new(shapes: &str) -> Self {
            Self {
                theme: CT_OfficeStyleSheet::office_default(),
                master: CT_SlideMaster::from_xml(master_xml().as_bytes()).unwrap(),
                layout: CT_SlideLayout::from_xml(layout_xml().as_bytes()).unwrap(),
                slide: CT_Slide::from_xml(slide_xml(shapes).as_bytes()).unwrap(),
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

        fn shape(&self, index: usize) -> &CT_Shape {
            let ShapeTreeChild::Shape(shape) =
                &self.slide.common_slide_data.shape_tree.children[index]
            else {
                panic!("expected an ordinary shape");
            };
            shape
        }
    }

    fn style_refs(line: u32, fill: u32, effect: u32, font: &str, color: &str) -> String {
        format!(
            r#"<p:style><a:lnRef idx="{line}"><a:srgbClr val="{color}"/></a:lnRef><a:fillRef idx="{fill}"><a:srgbClr val="{color}"/></a:fillRef><a:effectRef idx="{effect}"><a:srgbClr val="{color}"/></a:effectRef><a:fontRef idx="{font}"><a:srgbClr val="{color}"/></a:fontRef></p:style>"#
        )
    }

    fn style_refs_with_fill_colour(fill: u32, fill_colour: &str) -> String {
        format!(
            r#"<p:style><a:lnRef idx="0"><a:srgbClr val="000000"/></a:lnRef><a:fillRef idx="{fill}">{fill_colour}</a:fillRef><a:effectRef idx="0"><a:srgbClr val="000000"/></a:effectRef><a:fontRef idx="minor"><a:srgbClr val="000000"/></a:fontRef></p:style>"#
        )
    }

    fn shape(properties: &str, style: &str) -> String {
        format!(
            "<p:sp><p:nvSpPr><p:cNvPr/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr>{properties}</p:spPr>{style}</p:sp>"
        )
    }

    fn slide_xml(shapes: &str) -> String {
        format!(
            "<p:sld xmlns:p=\"{P_NS}\" xmlns:a=\"{A_NS}\"><p:cSld>{}</p:cSld></p:sld>",
            shape_tree(shapes)
        )
    }

    fn layout_xml() -> String {
        format!(
            "<p:sldLayout xmlns:p=\"{P_NS}\" xmlns:a=\"{A_NS}\"><p:cSld>{}</p:cSld></p:sldLayout>",
            shape_tree("")
        )
    }

    fn master_xml() -> String {
        format!(
            "<p:sldMaster xmlns:p=\"{P_NS}\" xmlns:a=\"{A_NS}\"><p:cSld>{}</p:cSld><p:clrMap bg1=\"lt1\" tx1=\"dk1\" bg2=\"lt2\" tx2=\"dk2\" accent1=\"accent1\" accent2=\"accent2\" accent3=\"accent3\" accent4=\"accent4\" accent5=\"accent5\" accent6=\"accent6\" hlink=\"hlink\" folHlink=\"folHlink\"/></p:sldMaster>",
            shape_tree("")
        )
    }

    fn shape_tree(shapes: &str) -> String {
        format!("<p:spTree><p:nvGrpSpPr/><p:grpSpPr/>{shapes}</p:spTree>")
    }
}
