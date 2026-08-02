use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;

use oxml_drawing::color::{ColorChoice, ColorMap, ColorMapSlot, RgbColor, resolve_color};
use oxml_drawing::effect::CT_OuterShadowEffect;
use oxml_drawing::fill::{Fill, GradientGeometry};
use oxml_drawing::geometry::EvaluatedPathCommand;
use oxml_drawing::line::{
    CT_LineProperties, LineCap as DrawingLineCap, LineDash, LineJoin as DrawingLineJoin,
};
use oxml_drawing::text::{
    CT_TextBody, CT_TextBodyProperties, CT_TextCharacterProperties, CT_TextListStyle,
    Coordinate32Value, TextAlignment, TextAnchor as DrawingTextAnchor, TextAutofit,
    TextBulletChoice, TextBulletSizeValue, TextPointValue, TextRun, TextSpacing,
    TextVertical as DrawingTextVertical, TextWrap,
};
use oxml_drawing::theme::CT_OfficeStyleSheet;
use oxml_drawing::xfrm::CT_Transform2D;
use oxml_layout::{
    Color, Diagnostic, Effect, FillRule, GradientStop, LineCap, LineJoin, Paint, Path, PathCommand,
    Point, Rect, Stroke, Transform,
};
use rpptx_oxml::graphic_frame::GraphicDataPayload;
use rpptx_oxml::placeholder::{CT_Placeholder, PhType, PlaceholderKey};
use rpptx_oxml::shape_tree::{CT_Shape, ShapeTreeChild};
use rpptx_oxml::slide_parts::{CT_Slide, CT_SlideLayout, CT_SlideMaster};

use crate::ResolveError;
use crate::text::EffectiveListStyle;
use crate::{
    ParagraphAlignment, ResolvedAutofit, ResolvedBullet, ResolvedBulletSize, ResolvedContent,
    ResolvedGeometry, ResolvedParagraph, ResolvedRunStyle, ResolvedShape, ResolvedSlide,
    ResolvedTable, ResolvedTableCell, ResolvedTableRow, ResolvedTextBody, ResolvedTextRun,
    ResolvedTextSpacing, TextAnchor, TextDirection, TextInsets,
};

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
        /// Affine mapping accumulated from every parent group.
        group_transform: Transform,
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
        let inherited_latent_enabled =
            self.layout.header_footer.is_some() || self.master.header_footer.is_some();
        let mut master_deeper = layout_latent.clone();
        master_deeper.extend(slide_latent.iter().cloned());
        emit_tree(
            master_children,
            PassRules {
                source: FlattenedSource::Master,
                emit_non_placeholders: self.layout.show_master_shapes.unwrap_or(true),
                emit_inherited_latent: inherited_latent_enabled,
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
                emit_inherited_latent: inherited_latent_enabled,
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
                emit_inherited_latent: true,
                deeper_latent: &[],
                latent_policy: allow_latent,
            },
            &mut flattened,
        );
        flattened
    }

    /// Resolves one owned renderer-facing slide at the supplied point size.
    pub fn resolve_slide(&self, size: (f64, f64)) -> Result<ResolvedSlide, ResolveError> {
        let mut slide = ResolvedSlide {
            size,
            background: None,
            shapes: Vec::new(),
            diagnostics: Vec::new(),
        };
        for item in self.flatten() {
            match item {
                FlattenedItem::Background(background) => match background.content {
                    BackgroundContent::ThemeFill(fill) => {
                        slide.background = self.concrete_background_fill(fill, size)?.0;
                    }
                    BackgroundContent::Xml(_) => slide.diagnostics.push(Diagnostic {
                        message: "modelled slide background paint is not available".to_owned(),
                    }),
                },
                FlattenedItem::Shape {
                    child,
                    group_transform,
                    ..
                } => {
                    if let Some(shape) = self.resolve_flattened_shape(
                        child,
                        group_transform,
                        &mut slide.diagnostics,
                    )? {
                        slide.shapes.push(shape);
                    }
                }
            }
        }
        Ok(slide)
    }

    fn resolve_flattened_shape(
        &self,
        child: &ShapeTreeChild,
        group_transform: Transform,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Result<Option<ResolvedShape>, ResolveError> {
        match child {
            ShapeTreeChild::Shape(shape) => {
                self.resolve_ordinary_shape(shape, group_transform, diagnostics)
            }
            ShapeTreeChild::Picture(picture) => {
                let Some((bounds, rotation_deg, flip_h, flip_v)) =
                    transform_values(picture.shape_properties.transform.as_ref())
                else {
                    return Ok(None);
                };
                let category = "image media pending relationship resolution";
                diagnostics.push(Diagnostic {
                    message: category.to_owned(),
                });
                Ok(Some(ResolvedShape {
                    group_transform,
                    bounds,
                    rotation_deg,
                    flip_h,
                    flip_v,
                    geometry: ResolvedGeometry::Rectangle,
                    fill: None,
                    line: None,
                    shadow: None,
                    content: ResolvedContent::None,
                    unsupported: Some(category),
                }))
            }
            ShapeTreeChild::GraphicFrame(frame) => {
                let Some((bounds, rotation_deg, flip_h, flip_v)) =
                    transform_values(Some(&frame.transform))
                else {
                    return Ok(None);
                };
                let (content, unsupported) = match &frame.graphic_data.payload {
                    GraphicDataPayload::Table(table) => {
                        (ResolvedContent::Table(self.resolve_table(table)?), None)
                    }
                    GraphicDataPayload::Chart(_) => (ResolvedContent::None, Some("chart")),
                    GraphicDataPayload::SmartArt(_) => (ResolvedContent::None, Some("SmartArt")),
                    GraphicDataPayload::Ole(_) => (ResolvedContent::None, Some("OLE")),
                    GraphicDataPayload::Other(_) => {
                        (ResolvedContent::None, Some("unknown graphic frame"))
                    }
                };
                if let Some(category) = unsupported {
                    diagnostics.push(Diagnostic {
                        message: format!("unsupported {category} content retained as bounds"),
                    });
                }
                Ok(Some(ResolvedShape {
                    group_transform,
                    bounds,
                    rotation_deg,
                    flip_h,
                    flip_v,
                    geometry: if unsupported.is_some() {
                        ResolvedGeometry::BoundsFallback
                    } else {
                        ResolvedGeometry::Rectangle
                    },
                    fill: None,
                    line: None,
                    shadow: None,
                    content,
                    unsupported,
                }))
            }
            ShapeTreeChild::Connector(connector) => {
                let Some((bounds, rotation_deg, flip_h, flip_v)) =
                    transform_values(connector.shape_properties.transform.as_ref())
                else {
                    return Ok(None);
                };
                let (fill, fill_unsupported) = connector
                    .shape_properties
                    .fill
                    .as_ref()
                    .map(|fill| self.concrete_fill(fill, (bounds.width, bounds.height)))
                    .transpose()?
                    .unwrap_or((None, None));
                let line = connector
                    .shape_properties
                    .line
                    .as_ref()
                    .map(|line| self.concrete_line(line, (bounds.width, bounds.height)))
                    .transpose()?
                    .flatten();
                let unsupported = fill_unsupported.or(Some("connector geometry"));
                diagnostics.push(Diagnostic {
                    message: format!(
                        "unsupported {} retained as shape bounds",
                        unsupported.unwrap_or("connector geometry")
                    ),
                });
                Ok(Some(ResolvedShape {
                    group_transform,
                    bounds,
                    rotation_deg,
                    flip_h,
                    flip_v,
                    geometry: ResolvedGeometry::BoundsFallback,
                    fill,
                    line,
                    shadow: None,
                    content: ResolvedContent::None,
                    unsupported,
                }))
            }
            ShapeTreeChild::GroupShape(_) | ShapeTreeChild::AlternateContent(_) => Ok(None),
        }
    }

    fn resolve_ordinary_shape(
        &self,
        shape: &CT_Shape,
        group_transform: Transform,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Result<Option<ResolvedShape>, ResolveError> {
        let transform = self.effective_xfrm(shape);
        let Some((bounds, rotation_deg, flip_h, flip_v)) = transform_values(transform.as_ref())
        else {
            return Ok(None);
        };
        let effective = self.effective_shape_style(shape)?;
        let (fill, fill_unsupported) = effective
            .fill
            .as_ref()
            .map(|fill| self.concrete_fill(fill, (bounds.width, bounds.height)))
            .transpose()?
            .unwrap_or((None, None));
        let line = effective
            .line
            .as_ref()
            .map(|line| self.concrete_line(line, (bounds.width, bounds.height)))
            .transpose()?
            .flatten();
        let shadow = effective
            .effects
            .as_ref()
            .and_then(|effects| effects.outer_shadow.as_ref())
            .map(|shadow| self.concrete_shadow(shadow))
            .transpose()?;
        let (geometry, geometry_unsupported) =
            if let Some(custom) = &shape.shape_properties.custom_geometry {
                (
                    self.concrete_custom_geometry(custom, (bounds.width, bounds.height))?,
                    None,
                )
            } else {
                (
                    ResolvedGeometry::BoundsFallback,
                    Some("preset geometry pending evaluation"),
                )
            };
        let content = shape
            .text_body
            .as_ref()
            .map(|body| self.resolve_text_body(shape, body))
            .transpose()?
            .map(ResolvedContent::Text)
            .unwrap_or(ResolvedContent::None);
        if let Some(category) = fill_unsupported {
            diagnostics.push(Diagnostic {
                message: format!("unsupported {category} retained as shape bounds"),
            });
        }
        if let Some(category) = geometry_unsupported {
            diagnostics.push(Diagnostic {
                message: format!("unsupported {category} retained as shape bounds"),
            });
        }
        Ok(Some(ResolvedShape {
            group_transform,
            bounds,
            rotation_deg,
            flip_h,
            flip_v,
            geometry,
            fill,
            line,
            shadow,
            content,
            unsupported: fill_unsupported.or(geometry_unsupported),
        }))
    }
}

impl ResolveCtx<'_> {
    fn concrete_background_fill(
        &self,
        fill: &Fill,
        size: (f64, f64),
    ) -> Result<(Option<Paint>, Option<&'static str>), ResolveError> {
        if let Fill::Solid(solid) = fill
            && matches!(
                solid.color.as_ref(),
                Some(ColorChoice::Scheme { value, .. }) if value == "phClr"
            )
        {
            let slot = self.color_map.theme_slot(ColorMapSlot::Background1);
            let choice = self.theme.theme_elements.color_scheme.color(slot);
            return Ok((Some(Paint::Solid(self.concrete_color(choice)?)), None));
        }
        self.concrete_fill(fill, size)
    }

    fn concrete_fill(
        &self,
        fill: &Fill,
        size: (f64, f64),
    ) -> Result<(Option<Paint>, Option<&'static str>), ResolveError> {
        match fill {
            Fill::NoFill(_) => Ok((None, None)),
            Fill::Solid(solid) => {
                let Some(choice) = solid.color.as_ref() else {
                    return Ok((None, Some("solid fill without colour")));
                };
                Ok((Some(Paint::Solid(self.concrete_color(choice)?)), None))
            }
            Fill::Gradient(gradient) => {
                let unsupported_geometry = gradient
                    .flip
                    .as_deref()
                    .is_some_and(|flip| flip != "none")
                    || gradient.rotate_with_shape == Some(false)
                    || gradient.tile_rect.is_some()
                    || matches!(gradient.geometry, Some(GradientGeometry::Path(_)))
                    || matches!(
                        gradient.geometry,
                        Some(GradientGeometry::Linear(ref linear)) if linear.scaled == Some(false)
                    );
                if unsupported_geometry {
                    return Ok((None, Some("gradient geometry")));
                }
                let mut stops = Vec::with_capacity(gradient.stops.len());
                for stop in &gradient.stops {
                    let Some(choice) = stop.color.as_ref() else {
                        return Ok((None, Some("gradient stop without colour")));
                    };
                    stops.push(GradientStop {
                        offset: f64::from(stop.position.0) / 100_000.0,
                        color: self.concrete_color(choice)?,
                    });
                }
                if stops.is_empty() {
                    return Ok((None, Some("gradient without concrete stops")));
                }
                let paint = match &gradient.geometry {
                    Some(GradientGeometry::Linear(linear)) => {
                        let radians = f64::from(linear.angle.0) / 60_000.0_f64;
                        let radians = radians.to_radians();
                        let center = Point {
                            x: size.0 / 2.0,
                            y: size.1 / 2.0,
                        };
                        let radius = size.0.hypot(size.1) / 2.0;
                        let delta = Point {
                            x: radians.cos() * radius,
                            y: radians.sin() * radius,
                        };
                        Paint::linear(
                            Point {
                                x: center.x - delta.x,
                                y: center.y - delta.y,
                            },
                            Point {
                                x: center.x + delta.x,
                                y: center.y + delta.y,
                            },
                            stops,
                            (true, true),
                        )
                    }
                    None => Paint::linear(
                        Point { x: 0.0, y: 0.0 },
                        Point { x: size.0, y: 0.0 },
                        stops,
                        (true, true),
                    ),
                    Some(GradientGeometry::Path(_)) => unreachable!("rejected above"),
                };
                Ok((Some(paint), None))
            }
            Fill::Pattern(_) => Ok((None, Some("pattern fill"))),
            Fill::Blip(_) => Ok((None, Some("picture fill media"))),
        }
    }

    fn concrete_line(
        &self,
        line: &CT_LineProperties,
        size: (f64, f64),
    ) -> Result<Option<Stroke>, ResolveError> {
        let Some(fill) = line.fill.as_ref() else {
            return Ok(None);
        };
        let (Some(paint), _) = self.concrete_fill(fill, size)? else {
            return Ok(None);
        };
        let width = emu_to_points(i64::from(line.width.unwrap_or(12_700)));
        let cap = match line.cap.unwrap_or(DrawingLineCap::Flat) {
            DrawingLineCap::Round => LineCap::Round,
            DrawingLineCap::Square => LineCap::Square,
            DrawingLineCap::Flat => LineCap::Butt,
        };
        let join = match line.join.as_ref() {
            Some(DrawingLineJoin::Round { .. }) => LineJoin::Round,
            Some(DrawingLineJoin::Bevel { .. }) => LineJoin::Bevel,
            Some(DrawingLineJoin::Miter { .. }) | None => LineJoin::Miter,
        };
        let dash = line.dash.as_ref().and_then(|dash| match dash {
            LineDash::Preset(preset) => {
                let values = preset
                    .value
                    .dash_array()
                    .iter()
                    .map(|value| f64::from(*value) * width)
                    .collect::<Vec<_>>();
                (!values.is_empty()).then_some(values)
            }
            LineDash::Custom(custom) => {
                let values = custom
                    .stops
                    .iter()
                    .flat_map(|stop| [stop.dash, stop.space])
                    .map(|value| f64::from(value) / 100_000.0 * width)
                    .collect::<Vec<_>>();
                (!values.is_empty()).then_some(values)
            }
        });
        Ok(Some(Stroke {
            paint,
            width,
            cap,
            join,
            dash,
        }))
    }

    fn concrete_shadow(&self, shadow: &CT_OuterShadowEffect) -> Result<Effect, ResolveError> {
        let color = shadow
            .color
            .as_ref()
            .map(|choice| self.concrete_color(choice))
            .transpose()?
            .unwrap_or(Color::BLACK);
        let distance = emu_to_points(shadow.distance.unwrap_or(0));
        let angle = f64::from(shadow.direction.unwrap_or_default().0) / 60_000.0;
        let radians = angle.to_radians();
        Ok(Effect::OuterShadow {
            dx: radians.cos() * distance,
            dy: radians.sin() * distance,
            blur: emu_to_points(shadow.blur_radius.unwrap_or(0)),
            color,
        })
    }

    fn concrete_color(&self, choice: &ColorChoice) -> Result<Color, ResolveError> {
        let owned_lookup = self.theme_lookup();
        let lookup = owned_lookup
            .iter()
            .map(|(name, color)| (name.as_str(), *color))
            .collect::<Vec<_>>();
        let color = resolve_color(choice, &self.color_map, &lookup).map_err(|error| {
            ResolveError::ConcreteValue {
                kind: "colour",
                detail: error.to_string(),
            }
        })?;
        Ok(Color {
            r: f64::from(color.red) / 255.0,
            g: f64::from(color.green) / 255.0,
            b: f64::from(color.blue) / 255.0,
            a: f64::from(color.alpha) / 255.0,
        })
    }

    fn theme_lookup(&self) -> Vec<(String, RgbColor)> {
        self.theme
            .theme_elements
            .color_scheme
            .iter()
            .filter_map(|(slot, choice)| {
                base_rgb(choice).map(|color| (slot.as_str().to_owned(), color))
            })
            .collect()
    }

    fn concrete_custom_geometry(
        &self,
        geometry: &oxml_drawing::geometry::CT_CustomGeometry2D,
        size: (f64, f64),
    ) -> Result<ResolvedGeometry, ResolveError> {
        let evaluated =
            geometry
                .evaluate(&BTreeMap::new())
                .map_err(|error| ResolveError::ConcreteValue {
                    kind: "custom geometry",
                    detail: error.to_string(),
                })?;
        let paths = evaluated
            .paths
            .into_iter()
            .zip(geometry.paths())
            .map(|(commands, source)| {
                let scale_x = size.0 / source.width.unwrap_or(1.0);
                let scale_y = size.1 / source.height.unwrap_or(1.0);
                Path {
                    commands: commands
                        .into_iter()
                        .map(|command| concrete_path_command(command, scale_x, scale_y))
                        .collect(),
                    fill_rule: FillRule::NonZero,
                }
            })
            .collect();
        let first_path = geometry.paths().first();
        let text_scale_x = first_path
            .and_then(|path| path.width)
            .map_or(0.0, |width| size.0 / width);
        let text_scale_y = first_path
            .and_then(|path| path.height)
            .map_or(0.0, |height| size.1 / height);
        let text_rect = evaluated.text_rectangle.map(|rectangle| Rect {
            x: rectangle.left * text_scale_x,
            y: rectangle.top * text_scale_y,
            width: (rectangle.right - rectangle.left) * text_scale_x,
            height: (rectangle.bottom - rectangle.top) * text_scale_y,
        });
        Ok(ResolvedGeometry::Custom { paths, text_rect })
    }

    fn resolve_text_body(
        &self,
        shape: &CT_Shape,
        body: &CT_TextBody,
    ) -> Result<ResolvedTextBody, ResolveError> {
        let properties = self.effective_body_pr(shape);
        let paragraphs = body
            .paragraphs()
            .iter()
            .map(|paragraph| self.resolve_paragraph(shape, paragraph))
            .collect::<Result<Vec<_>, _>>()?;
        resolved_text_body(&properties, paragraphs)
    }

    fn resolve_paragraph(
        &self,
        shape: &CT_Shape,
        paragraph: &oxml_drawing::text::CT_TextParagraph,
    ) -> Result<ResolvedParagraph, ResolveError> {
        let effective = self.effective_text_properties(shape, paragraph.properties.as_ref(), None);
        let paragraph_properties = &effective.paragraph;
        let mut runs = Vec::new();
        for run in &paragraph.runs {
            match run {
                TextRun::Run(run) => {
                    let style = self.resolve_run_style(
                        shape,
                        paragraph.properties.as_ref(),
                        run.properties.as_ref(),
                    )?;
                    runs.push(ResolvedTextRun::Text {
                        text: run.text.value.clone(),
                        style,
                    });
                }
                TextRun::Break(_) => runs.push(ResolvedTextRun::Break),
                TextRun::Field(field) => {
                    let style = self.resolve_run_style(
                        shape,
                        paragraph.properties.as_ref(),
                        field.run_properties.as_ref(),
                    )?;
                    runs.push(ResolvedTextRun::Field {
                        text: field
                            .text
                            .as_ref()
                            .map(|text| text.value.clone())
                            .unwrap_or_default(),
                        field_type: field.field_type.clone(),
                        style,
                    });
                }
            }
        }
        Ok(ResolvedParagraph {
            level: paragraph_properties.level.unwrap_or(0),
            left_margin: emu_to_points(i64::from(paragraph_properties.left_margin.unwrap_or(0))),
            right_margin: emu_to_points(i64::from(paragraph_properties.right_margin.unwrap_or(0))),
            indent: emu_to_points(i64::from(paragraph_properties.indent.unwrap_or(0))),
            alignment: paragraph_alignment(paragraph_properties.alignment),
            line_spacing: resolved_text_spacing(paragraph_properties.line_spacing.as_ref()),
            space_before: resolved_text_spacing(paragraph_properties.space_before.as_ref()),
            space_after: resolved_text_spacing(paragraph_properties.space_after.as_ref()),
            bullet: paragraph_properties
                .bullet
                .as_ref()
                .and_then(|bullet| self.resolve_bullet(bullet).transpose())
                .transpose()?,
            runs,
        })
    }

    fn resolve_run_style(
        &self,
        shape: &CT_Shape,
        paragraph: Option<&oxml_drawing::text::CT_TextParagraphProperties>,
        run: Option<&CT_TextCharacterProperties>,
    ) -> Result<ResolvedRunStyle, ResolveError> {
        let effective = self.effective_text_properties(shape, paragraph, run).run;
        self.resolved_run_style(effective)
    }

    fn resolve_table_run_style(
        &self,
        paragraph: Option<&oxml_drawing::text::CT_TextParagraphProperties>,
        run: Option<&CT_TextCharacterProperties>,
    ) -> Result<ResolvedRunStyle, ResolveError> {
        let effective = self.effective_table_text_properties(paragraph, run).run;
        self.resolved_run_style(effective)
    }

    fn resolved_run_style(
        &self,
        effective: CT_TextCharacterProperties,
    ) -> Result<ResolvedRunStyle, ResolveError> {
        let fill = effective
            .fill
            .as_ref()
            .map(|fill| self.concrete_fill(fill, (1.0, 1.0)))
            .transpose()?
            .and_then(|(paint, _)| paint);
        Ok(ResolvedRunStyle {
            font_size: effective.font_size.map(|size| f64::from(size) / 100.0),
            bold: effective.bold.unwrap_or(false),
            italic: effective.italic.unwrap_or(false),
            underline: effective
                .underline
                .is_some_and(|value| value != oxml_drawing::text::TextUnderline::None),
            strike: effective
                .strike
                .is_some_and(|value| value != oxml_drawing::text::TextStrike::None),
            spacing: effective.spacing.as_ref().and_then(text_point_value),
            baseline: effective.baseline.as_deref().and_then(parse_percent),
            fill,
            latin_typeface: effective
                .latin
                .as_ref()
                .map(|font| self.resolve_typeface(&font.typeface, None)),
            east_asian_typeface: effective
                .east_asian
                .as_ref()
                .map(|font| self.resolve_typeface(&font.typeface, None)),
            complex_script_typeface: effective
                .complex_script
                .as_ref()
                .map(|font| self.resolve_typeface(&font.typeface, None)),
            symbol_typeface: effective
                .symbol
                .as_ref()
                .map(|font| self.resolve_typeface(&font.typeface, None)),
        })
    }

    fn resolve_bullet(
        &self,
        bullet: &oxml_drawing::text::TextBullet,
    ) -> Result<Option<ResolvedBullet>, ResolveError> {
        let color = bullet
            .color
            .as_ref()
            .map(|color| self.concrete_color(&color.color))
            .transpose()?;
        let font = bullet
            .font
            .as_ref()
            .map(|font| self.resolve_typeface(&font.typeface, None));
        let size = bullet.size.as_ref().and_then(|size| match &size.value {
            TextBulletSizeValue::Percent(value) => {
                parse_percent(value).map(ResolvedBulletSize::Percent)
            }
            TextBulletSizeValue::Points(value) => {
                Some(ResolvedBulletSize::Points(f64::from(*value) / 100.0))
            }
        });
        Ok(match bullet.choice.as_ref() {
            Some(TextBulletChoice::Character(character)) => Some(ResolvedBullet::Character {
                character: character.character.clone(),
                font,
                color,
                size,
            }),
            Some(TextBulletChoice::AutoNumber(number)) => Some(ResolvedBullet::AutoNumber {
                scheme: number.scheme.as_str().to_owned(),
                start_at: u32::from(number.start_at.unwrap_or(1)),
                font,
                color,
                size,
            }),
            Some(TextBulletChoice::None(_)) | None => None,
        })
    }

    fn resolve_table(
        &self,
        table: &oxml_drawing::table::CT_Table,
    ) -> Result<ResolvedTable, ResolveError> {
        let column_widths = table
            .grid
            .columns
            .iter()
            .map(|width| emu_to_points(width.0))
            .collect();
        let rows = table
            .rows
            .iter()
            .map(|row| {
                let cells = row
                    .cells
                    .iter()
                    .map(|cell| {
                        let text = cell
                            .text_body
                            .as_ref()
                            .map(|body| resolve_standalone_text_body(self, body))
                            .transpose()?;
                        Ok(ResolvedTableCell {
                            text,
                            row_span: cell.row_span,
                            grid_span: cell.grid_span,
                            horizontal_merge: cell.horizontal_merge,
                            vertical_merge: cell.vertical_merge,
                        })
                    })
                    .collect::<Result<Vec<_>, ResolveError>>()?;
                Ok(ResolvedTableRow {
                    height: emu_to_points(row.height.0),
                    cells,
                })
            })
            .collect::<Result<Vec<_>, ResolveError>>()?;
        Ok(ResolvedTable {
            column_widths,
            rows,
        })
    }
}

fn transform_values(transform: Option<&CT_Transform2D>) -> Option<(Rect, f64, bool, bool)> {
    let transform = transform?;
    let extent = transform.extent?;
    if extent.cx.0 <= 0 || extent.cy.0 <= 0 {
        return None;
    }
    let offset = transform.offset.unwrap_or_default();
    Some((
        Rect {
            x: emu_to_points(offset.x.0),
            y: emu_to_points(offset.y.0),
            width: emu_to_points(extent.cx.0),
            height: emu_to_points(extent.cy.0),
        },
        f64::from(transform.rotation.0) / 60_000.0,
        transform.flip_horizontal,
        transform.flip_vertical,
    ))
}

fn emu_to_points(value: i64) -> f64 {
    value as f64 / 12_700.0
}

fn base_rgb(choice: &ColorChoice) -> Option<RgbColor> {
    let resolved = resolve_color(choice, &ColorMap::default(), &[]).ok()?;
    Some(RgbColor::new(resolved.red, resolved.green, resolved.blue))
}

fn concrete_path_command(command: EvaluatedPathCommand, scale_x: f64, scale_y: f64) -> PathCommand {
    match command {
        EvaluatedPathCommand::MoveTo { x, y } => PathCommand::MoveTo(Point {
            x: x * scale_x,
            y: y * scale_y,
        }),
        EvaluatedPathCommand::LineTo { x, y } => PathCommand::LineTo(Point {
            x: x * scale_x,
            y: y * scale_y,
        }),
        EvaluatedPathCommand::CubicTo {
            x1,
            y1,
            x2,
            y2,
            x,
            y,
        } => PathCommand::CurveTo {
            c1: Point {
                x: x1 * scale_x,
                y: y1 * scale_y,
            },
            c2: Point {
                x: x2 * scale_x,
                y: y2 * scale_y,
            },
            to: Point {
                x: x * scale_x,
                y: y * scale_y,
            },
        },
        EvaluatedPathCommand::Close => PathCommand::Close,
    }
}

fn coordinate_points(value: Option<&Coordinate32Value>) -> Result<f64, ResolveError> {
    match value {
        Some(Coordinate32Value::Emu(value)) => Ok(emu_to_points(i64::from(*value))),
        Some(Coordinate32Value::UniversalMeasure(value)) => universal_measure_points(value),
        None => Ok(0.0),
    }
}

fn universal_measure_points(value: &str) -> Result<f64, ResolveError> {
    let split = value
        .find(|character: char| character.is_ascii_alphabetic())
        .unwrap_or(value.len());
    let (number, unit) = value.split_at(split);
    let number = number
        .parse::<f64>()
        .map_err(|error| ResolveError::ConcreteValue {
            kind: "universal measure",
            detail: error.to_string(),
        })?;
    match unit {
        "pt" => Ok(number),
        "in" => Ok(number * 72.0),
        "cm" => Ok(number * 72.0 / 2.54),
        "mm" => Ok(number * 72.0 / 25.4),
        _ => Err(ResolveError::ConcreteValue {
            kind: "universal measure",
            detail: format!("unsupported unit {unit}"),
        }),
    }
}

fn parse_percent(value: &str) -> Option<f64> {
    value.parse::<f64>().ok().map(|value| value / 100_000.0)
}

fn text_point_value(value: &TextPointValue) -> Option<f64> {
    match value {
        TextPointValue::Centipoints(value) => Some(f64::from(*value) / 100.0),
        TextPointValue::UniversalMeasure(value) => universal_measure_points(value).ok(),
    }
}

fn resolved_text_spacing(value: Option<&TextSpacing>) -> Option<ResolvedTextSpacing> {
    match value? {
        TextSpacing::Percent(value) => parse_percent(value).map(ResolvedTextSpacing::Percent),
        TextSpacing::Points(value) => Some(ResolvedTextSpacing::Points(f64::from(*value) / 100.0)),
    }
}

fn paragraph_alignment(alignment: Option<TextAlignment>) -> ParagraphAlignment {
    match alignment.unwrap_or(TextAlignment::Left) {
        TextAlignment::Left => ParagraphAlignment::Left,
        TextAlignment::Center => ParagraphAlignment::Center,
        TextAlignment::Right => ParagraphAlignment::Right,
        TextAlignment::Justified | TextAlignment::JustifiedLow => ParagraphAlignment::Justified,
        TextAlignment::Distributed | TextAlignment::ThaiDistributed => {
            ParagraphAlignment::Distributed
        }
    }
}

fn resolve_standalone_text_body(
    context: &ResolveCtx<'_>,
    body: &CT_TextBody,
) -> Result<ResolvedTextBody, ResolveError> {
    let mut properties = default_body_properties();
    merge_body_properties(&mut properties, &body.body_properties);
    let paragraphs = body
        .paragraphs()
        .iter()
        .map(|paragraph| {
            let effective =
                context.effective_table_text_properties(paragraph.properties.as_ref(), None);
            let paragraph_properties = &effective.paragraph;
            let runs = paragraph
                .runs
                .iter()
                .map(|run| match run {
                    TextRun::Run(run) => Ok(ResolvedTextRun::Text {
                        text: run.text.value.clone(),
                        style: context.resolve_table_run_style(
                            paragraph.properties.as_ref(),
                            run.properties.as_ref(),
                        )?,
                    }),
                    TextRun::Break(_) => Ok(ResolvedTextRun::Break),
                    TextRun::Field(field) => Ok(ResolvedTextRun::Field {
                        text: field
                            .text
                            .as_ref()
                            .map(|text| text.value.clone())
                            .unwrap_or_default(),
                        field_type: field.field_type.clone(),
                        style: context.resolve_table_run_style(
                            paragraph.properties.as_ref(),
                            field.run_properties.as_ref(),
                        )?,
                    }),
                })
                .collect::<Result<Vec<_>, ResolveError>>()?;
            Ok(ResolvedParagraph {
                level: paragraph_properties.level.unwrap_or(0),
                left_margin: emu_to_points(i64::from(
                    paragraph_properties.left_margin.unwrap_or(0),
                )),
                right_margin: emu_to_points(i64::from(
                    paragraph_properties.right_margin.unwrap_or(0),
                )),
                indent: emu_to_points(i64::from(paragraph_properties.indent.unwrap_or(0))),
                alignment: paragraph_alignment(paragraph_properties.alignment),
                line_spacing: resolved_text_spacing(paragraph_properties.line_spacing.as_ref()),
                space_before: resolved_text_spacing(paragraph_properties.space_before.as_ref()),
                space_after: resolved_text_spacing(paragraph_properties.space_after.as_ref()),
                bullet: paragraph_properties
                    .bullet
                    .as_ref()
                    .and_then(|bullet| context.resolve_bullet(bullet).transpose())
                    .transpose()?,
                runs,
            })
        })
        .collect::<Result<Vec<_>, ResolveError>>()?;
    resolved_text_body(&properties, paragraphs)
}

fn resolved_text_body(
    properties: &CT_TextBodyProperties,
    paragraphs: Vec<ResolvedParagraph>,
) -> Result<ResolvedTextBody, ResolveError> {
    let insets = TextInsets {
        left: coordinate_points(properties.left_inset.as_ref())?,
        top: coordinate_points(properties.top_inset.as_ref())?,
        right: coordinate_points(properties.right_inset.as_ref())?,
        bottom: coordinate_points(properties.bottom_inset.as_ref())?,
    };
    let anchor = match properties.anchor.unwrap_or(DrawingTextAnchor::Top) {
        DrawingTextAnchor::Top => TextAnchor::Top,
        DrawingTextAnchor::Center => TextAnchor::Center,
        DrawingTextAnchor::Bottom => TextAnchor::Bottom,
        DrawingTextAnchor::Justified => TextAnchor::Justified,
        DrawingTextAnchor::Distributed => TextAnchor::Distributed,
    };
    let vertical = match properties
        .vertical
        .unwrap_or(DrawingTextVertical::Horizontal)
    {
        DrawingTextVertical::Horizontal => TextDirection::Horizontal,
        DrawingTextVertical::Vertical => TextDirection::Vertical,
        DrawingTextVertical::Vertical270 => TextDirection::Vertical270,
        DrawingTextVertical::EastAsianVertical => TextDirection::EastAsianVertical,
        DrawingTextVertical::MongolianVertical => TextDirection::MongolianVertical,
        DrawingTextVertical::WordArtVertical => TextDirection::WordArtVertical,
        DrawingTextVertical::WordArtVerticalRtl => TextDirection::WordArtVerticalRtl,
    };
    let autofit = match properties.autofit.clone().unwrap_or(TextAutofit::NoAutofit) {
        TextAutofit::NoAutofit => ResolvedAutofit::None,
        TextAutofit::ShapeAutofit => ResolvedAutofit::Shape,
        TextAutofit::Normal(normal) => ResolvedAutofit::Normal {
            font_scale: normal.font_scale.as_deref().and_then(parse_percent),
            line_spacing_reduction: normal
                .line_spacing_reduction
                .as_deref()
                .and_then(parse_percent),
        },
    };
    Ok(ResolvedTextBody {
        insets,
        anchor,
        wrap: properties.wrap.unwrap_or(TextWrap::Square) != TextWrap::None,
        vertical,
        autofit,
        paragraphs,
    })
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
    emit_inherited_latent: bool,
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
    if !keys
        .iter()
        .any(|existing| placeholder_keys_match(&key, existing))
    {
        keys.push(key);
    }
}

fn emit_tree<'a>(
    children: &'a [ShapeTreeChild],
    rules: PassRules<'_>,
    output: &mut Vec<FlattenedItem<'a>>,
) {
    let mut emitted_latent = Vec::new();
    emit_tree_inner(
        children,
        rules,
        Transform::IDENTITY,
        &mut emitted_latent,
        output,
    );
}

fn emit_tree_inner<'a>(
    children: &'a [ShapeTreeChild],
    rules: PassRules<'_>,
    parent_transform: Transform,
    emitted_latent: &mut Vec<PlaceholderKey>,
    output: &mut Vec<FlattenedItem<'a>>,
) {
    for child in children {
        match child {
            ShapeTreeChild::GroupShape(group) => {
                let transform = group
                    .group_transform()
                    .and_then(group_affine)
                    .unwrap_or(Transform::IDENTITY)
                    .then(parent_transform);
                emit_tree_inner(&group.children, rules, transform, emitted_latent, output);
            }
            ShapeTreeChild::AlternateContent(alternate) => {
                if let Some(fallback) = alternate.selected_fallback() {
                    emit_tree_inner(fallback, rules, parent_transform, emitted_latent, output);
                }
            }
            _ => emit_leaf(child, rules, parent_transform, emitted_latent, output),
        }
    }
}

fn emit_leaf<'a>(
    child: &'a ShapeTreeChild,
    rules: PassRules<'_>,
    group_transform: Transform,
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
                    .any(|key| placeholder_keys_match(&placeholder.key(), key));
                if !rules.latent_policy.permits(&ph_type) || !occupied || already_emitted {
                    return;
                }
                push_unmatched(emitted_latent, placeholder.key());
            }
        }
        output.push(FlattenedItem::Shape {
            source: rules.source,
            child,
            group_transform,
        });
        return;
    }

    let Some(placeholder) = placeholder else {
        if rules.emit_non_placeholders {
            output.push(FlattenedItem::Shape {
                source: rules.source,
                child,
                group_transform,
            });
        }
        return;
    };
    let ph_type = placeholder.effective_type();
    let key = placeholder.key();
    if !is_latent(&ph_type)
        || !rules.emit_inherited_latent
        || !rules.latent_policy.permits(&ph_type)
        || !child_is_occupied(child)
        || rules
            .deeper_latent
            .iter()
            .any(|deeper| placeholder_keys_match(&key, deeper))
        || emitted_latent
            .iter()
            .any(|emitted| placeholder_keys_match(&key, emitted))
    {
        return;
    }
    push_unmatched(emitted_latent, key);
    output.push(FlattenedItem::Shape {
        source: rules.source,
        child,
        group_transform,
    });
}

fn group_affine(transform: &CT_Transform2D) -> Option<Transform> {
    let offset = transform.offset?;
    let extent = transform.extent?;
    let child_offset = transform.child_offset?;
    let child_extent = transform.child_extent?;
    if child_extent.cx.0 == 0 || child_extent.cy.0 == 0 {
        return None;
    }
    let x = emu_to_points(offset.x.0);
    let y = emu_to_points(offset.y.0);
    let width = emu_to_points(extent.cx.0);
    let height = emu_to_points(extent.cy.0);
    let child_x = emu_to_points(child_offset.x.0);
    let child_y = emu_to_points(child_offset.y.0);
    let scale_x = extent.cx.0 as f64 / child_extent.cx.0 as f64;
    let scale_y = extent.cy.0 as f64 / child_extent.cy.0 as f64;
    let mut affine = Transform {
        a: scale_x,
        b: 0.0,
        c: 0.0,
        d: scale_y,
        e: x - child_x * scale_x,
        f: y - child_y * scale_y,
    };
    let center_x = x + width / 2.0;
    let center_y = y + height / 2.0;
    affine = affine.then(Transform::rotate_about(
        f64::from(transform.rotation.0) / 60_000.0,
        center_x,
        center_y,
    ));
    if transform.flip_horizontal || transform.flip_vertical {
        affine = affine.then(Transform {
            a: if transform.flip_horizontal { -1.0 } else { 1.0 },
            b: 0.0,
            c: 0.0,
            d: if transform.flip_vertical { -1.0 } else { 1.0 },
            e: if transform.flip_horizontal {
                2.0 * center_x
            } else {
                0.0
            },
            f: if transform.flip_vertical {
                2.0 * center_y
            } else {
                0.0
            },
        });
    }
    Some(affine)
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

fn placeholder_keys_match(left: &PlaceholderKey, right: &PlaceholderKey) -> bool {
    if is_latent(&left.ph_type) && is_latent(&right.ph_type) {
        left.ph_type == right.ph_type
    } else {
        left.matches(right)
    }
}

fn default_body_properties() -> CT_TextBodyProperties {
    let mut properties = CT_TextBodyProperties::default();
    properties.left_inset = Some(Coordinate32Value::Emu(91_440));
    properties.top_inset = Some(Coordinate32Value::Emu(45_720));
    properties.right_inset = Some(Coordinate32Value::Emu(91_440));
    properties.bottom_inset = Some(Coordinate32Value::Emu(45_720));
    properties.anchor = Some(DrawingTextAnchor::Top);
    properties.wrap = Some(TextWrap::Square);
    properties.vertical = Some(DrawingTextVertical::Horizontal);
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
                .is_some_and(|placeholder| placeholder_keys_match(key, &placeholder.key()))
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
    use std::path::{Path, PathBuf};

    use oxml_drawing::color::ColorMap;
    use oxml_drawing::text::{
        CT_TextListStyle, Coordinate32Value, TextAnchor, TextAutofit, TextVertical, TextWrap,
    };
    use oxml_drawing::theme::CT_OfficeStyleSheet;
    use oxml_opc::OpcPackage;
    use oxml_opc::relationship::rel_types;
    use rpptx_oxml::placeholder::PhType;
    use rpptx_oxml::presentation::CT_Presentation;
    use rpptx_oxml::shape_tree::{CT_Shape, ShapeTreeChild};
    use rpptx_oxml::slide_parts::{CT_Slide, CT_SlideLayout, CT_SlideMaster, ColorMapOverrideKind};

    use super::{BackgroundSource, FlattenedItem, FlattenedSource, ResolveCtx};
    use crate::{
        ResolvedAutofit, ResolvedBullet, ResolvedBulletSize, ResolvedContent, ResolvedGeometry,
        ResolvedSlide, ResolvedTextSpacing, TextAnchor as ResolvedTextAnchor, TextDirection,
    };
    use oxml_layout::{Color, Effect, Paint, PathCommand, Point};

    const P_NS: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";
    const A_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
    const MC_NS: &str = "http://schemas.openxmlformats.org/markup-compatibility/2006";
    const EXPECTED_CORPUS_DECKS: usize = 50;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1.0e-10,
            "expected {expected}, got {actual}"
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

    #[test]
    fn absent_header_footer_container_hides_inherited_latent_placeholders() {
        let slide = shape_with_text(Some("ftr"), Some(11), "slide footer");
        let layout = [
            shape_with_text(Some("dt"), Some(10), "layout date"),
            shape_with_text(Some("sldNum"), Some(12), "layout number"),
        ]
        .join("");
        let fixture = Fixture::from_xml(&slide_xml(&slide), &layout_xml(&layout), &master_xml(""));

        let texts = fixture
            .context()
            .flatten()
            .into_iter()
            .filter_map(|item| item_text(&item))
            .collect::<Vec<_>>();

        assert_eq!(texts, ["slide footer"]);
    }

    #[test]
    fn corpus_slide_resolves_without_theme_references() {
        let stats = resolve_pinned_corpus();

        assert_eq!(stats.decks, EXPECTED_CORPUS_DECKS);
        assert!(stats.slides > EXPECTED_CORPUS_DECKS);
        assert!(stats.resolved > 0, "no corpus slide produced a contract");
        assert_eq!(stats.theme_references, 0);
    }

    #[test]
    fn resolved_contract_contains_no_presentation_or_drawing_types() {
        fn assert_owned_and_static(_: ResolvedSlide) {}

        let fixture = Fixture::new(&shape_with_details(None, None, &transform(0), None), "", "");
        let resolved = fixture.context().resolve_slide((720.0, 540.0)).unwrap();
        assert_owned_and_static(resolved);
        for type_name in [
            std::any::type_name::<ResolvedSlide>(),
            std::any::type_name::<crate::ResolvedShape>(),
            std::any::type_name::<crate::ResolvedContent>(),
        ] {
            assert!(!type_name.contains("rpptx_oxml"));
            assert!(!type_name.contains("oxml_drawing"));
        }
    }

    #[test]
    fn transform_resolves_to_points_rotation_and_flips() {
        let fixture = Fixture::new(
            &shape_with_details(
                None,
                None,
                r#"<a:xfrm rot="5400000" flipH="1" flipV="1"><a:off x="12700" y="25400"/><a:ext cx="38100" cy="50800"/></a:xfrm>"#,
                None,
            ),
            "",
            "",
        );

        let resolved = fixture.context().resolve_slide((720.0, 540.0)).unwrap();
        let shape = &resolved.shapes[0];
        assert_eq!(shape.bounds.x, 1.0);
        assert_eq!(shape.bounds.y, 2.0);
        assert_eq!(shape.bounds.width, 3.0);
        assert_eq!(shape.bounds.height, 4.0);
        assert_eq!(shape.rotation_deg, 90.0);
        assert!(shape.flip_h);
        assert!(shape.flip_v);
    }

    #[test]
    fn custom_geometry_scales_each_path_coordinate_space_to_shape_points() {
        let geometry = r#"<a:custGeom><a:avLst/><a:pathLst><a:path w="21600" h="10800"><a:moveTo><a:pt x="0" y="0"/></a:moveTo><a:lnTo><a:pt x="21600" y="10800"/></a:lnTo></a:path></a:pathLst></a:custGeom>"#;
        let shape = shape_with_details(
            None,
            None,
            &format!(
                "<a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"127000\" cy=\"254000\"/></a:xfrm>{geometry}"
            ),
            None,
        );
        let fixture = Fixture::new(&shape, "", "");

        let resolved = fixture.context().resolve_slide((720.0, 540.0)).unwrap();
        let ResolvedGeometry::Custom { paths, .. } = &resolved.shapes[0].geometry else {
            panic!("expected custom geometry");
        };
        assert_eq!(
            paths[0].commands[1],
            PathCommand::LineTo(oxml_layout::Point { x: 10.0, y: 20.0 })
        );
    }

    #[test]
    fn nested_group_affines_are_retained_in_the_owned_contract() {
        let leaf = shape_with_details(None, None, &transform(0), None);
        let inner = format!(
            r#"<p:grpSp><p:nvGrpSpPr/><p:grpSpPr><a:xfrm><a:off x="12700" y="0"/><a:ext cx="127000" cy="127000"/><a:chOff x="0" y="0"/><a:chExt cx="127000" cy="127000"/></a:xfrm></p:grpSpPr>{leaf}</p:grpSp>"#
        );
        let outer = format!(
            r#"<p:grpSp><p:nvGrpSpPr/><p:grpSpPr><a:xfrm rot="5400000" flipH="1"><a:off x="12700" y="25400"/><a:ext cx="254000" cy="381000"/><a:chOff x="0" y="0"/><a:chExt cx="127000" cy="127000"/></a:xfrm></p:grpSpPr>{inner}</p:grpSp>"#
        );
        let fixture = Fixture::new(&outer, "", "");

        let resolved = fixture.context().resolve_slide((720.0, 540.0)).unwrap();
        let transform = resolved.shapes[0].group_transform;
        let origin = transform.apply(Point { x: 0.0, y: 0.0 });
        let other = transform.apply(Point { x: 1.0, y: 2.0 });
        assert_close(origin.x, -4.0);
        assert_close(origin.y, 9.0);
        assert_close(other.x, 2.0);
        assert_close(other.y, 11.0);
    }

    #[test]
    fn paragraph_spacing_and_bullet_size_are_concrete() {
        let text = r#"<a:bodyPr/><a:p><a:pPr><a:lnSpc><a:spcPct val="120000"/></a:lnSpc><a:spcBef><a:spcPts val="600"/></a:spcBef><a:spcAft><a:spcPts val="300"/></a:spcAft><a:buSzPct val="125000"/><a:buChar char="*"/></a:pPr><a:r><a:t>spaced</a:t></a:r></a:p>"#;
        let fixture = Fixture::new(
            &shape_with_details(None, None, &transform(0), Some(text)),
            "",
            "",
        );

        let resolved = fixture.context().resolve_slide((720.0, 540.0)).unwrap();
        let ResolvedContent::Text(body) = &resolved.shapes[0].content else {
            panic!("expected text");
        };
        let paragraph = &body.paragraphs[0];
        assert_eq!(
            paragraph.line_spacing,
            Some(ResolvedTextSpacing::Percent(1.2))
        );
        assert_eq!(
            paragraph.space_before,
            Some(ResolvedTextSpacing::Points(6.0))
        );
        assert_eq!(
            paragraph.space_after,
            Some(ResolvedTextSpacing::Points(3.0))
        );
        assert!(matches!(
            paragraph.bullet,
            Some(ResolvedBullet::Character {
                size: Some(ResolvedBulletSize::Percent(1.25)),
                ..
            })
        ));
    }

    #[test]
    fn auto_number_bullet_keeps_independently_inherited_style() {
        let text = r#"<a:bodyPr/><a:lstStyle><a:lvl1pPr><a:buClr><a:srgbClr val="123456"/></a:buClr><a:buSzPct val="125000"/><a:buFont typeface="Wingdings"/><a:buChar char="*"/></a:lvl1pPr></a:lstStyle><a:p><a:pPr><a:buAutoNum type="arabicPeriod" startAt="3"/></a:pPr><a:r><a:t>numbered</a:t></a:r></a:p>"#;
        let fixture = Fixture::new(
            &shape_with_details(None, None, &transform(0), Some(text)),
            "",
            "",
        );

        let resolved = fixture.context().resolve_slide((720.0, 540.0)).unwrap();
        let ResolvedContent::Text(body) = &resolved.shapes[0].content else {
            panic!("expected text");
        };
        assert_eq!(
            body.paragraphs[0].bullet,
            Some(ResolvedBullet::AutoNumber {
                scheme: "arabicPeriod".to_owned(),
                start_at: 3,
                font: Some("Wingdings".to_owned()),
                color: Some(Color {
                    r: 0x12 as f64 / 255.0,
                    g: 0x34 as f64 / 255.0,
                    b: 0x56 as f64 / 255.0,
                    a: 1.0,
                }),
                size: Some(ResolvedBulletSize::Percent(1.25)),
            })
        );
    }

    #[test]
    fn table_cells_resolve_body_and_paragraph_properties() {
        let frame = r#"<p:graphicFrame><p:nvGraphicFramePr/><p:xfrm><a:off x="0" y="0"/><a:ext cx="127000" cy="127000"/></p:xfrm><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/table"><a:tbl><a:tblGrid><a:gridCol w="127000"/></a:tblGrid><a:tr h="127000"><a:tc><a:txBody><a:bodyPr lIns="12700" anchor="b" vert="vert"><a:spAutoFit/></a:bodyPr><a:lstStyle/><a:p><a:pPr marL="25400" algn="ctr"/><a:r><a:t>cell</a:t></a:r></a:p></a:txBody><a:tcPr/></a:tc></a:tr></a:tbl></a:graphicData></a:graphic></p:graphicFrame>"#;
        let fixture = Fixture::new(frame, "", "");

        let resolved = fixture.context().resolve_slide((720.0, 540.0)).unwrap();
        let ResolvedContent::Table(table) = &resolved.shapes[0].content else {
            panic!("expected table");
        };
        let body = table.rows[0].cells[0].text.as_ref().unwrap();
        assert_eq!(body.insets.left, 1.0);
        assert_eq!(body.anchor, ResolvedTextAnchor::Bottom);
        assert_eq!(body.vertical, TextDirection::Vertical);
        assert_eq!(body.autofit, ResolvedAutofit::Shape);
        assert_eq!(body.paragraphs[0].left_margin, 2.0);
        assert_eq!(
            body.paragraphs[0].alignment,
            crate::ParagraphAlignment::Center
        );
    }

    #[test]
    fn unrepresentable_gradient_geometry_uses_diagnostic_fallback() {
        let details = r#"<a:xfrm><a:off x="0" y="0"/><a:ext cx="127000" cy="127000"/></a:xfrm><a:gradFill flip="x"><a:gsLst><a:gs pos="0"><a:srgbClr val="000000"/></a:gs><a:gs pos="100000"><a:srgbClr val="FFFFFF"/></a:gs></a:gsLst><a:lin ang="0"/></a:gradFill>"#;
        let fixture = Fixture::new(&shape_with_details(None, None, details, None), "", "");

        let resolved = fixture.context().resolve_slide((720.0, 540.0)).unwrap();
        assert!(resolved.shapes[0].fill.is_none());
        assert_eq!(resolved.shapes[0].unsupported, Some("gradient geometry"));
        assert!(
            resolved
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("gradient geometry"))
        );
    }

    #[test]
    fn theme_style_resolves_to_concrete_paint_line_and_shadow() {
        let shape = format!(
            "<p:sp><p:nvSpPr><p:cNvPr/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr>{}<a:effectLst><a:outerShdw blurRad=\"12700\" dist=\"25400\" dir=\"0\"><a:schemeClr val=\"accent2\"/></a:outerShdw></a:effectLst></p:spPr><p:style><a:lnRef idx=\"1\"><a:srgbClr val=\"112233\"/></a:lnRef><a:fillRef idx=\"1\"><a:srgbClr val=\"445566\"/></a:fillRef><a:effectRef idx=\"1\"><a:srgbClr val=\"778899\"/></a:effectRef><a:fontRef idx=\"minor\"><a:srgbClr val=\"000000\"/></a:fontRef></p:style></p:sp>",
            transform(0)
        );
        let fixture = Fixture::new(&shape, "", "");

        let resolved = fixture.context().resolve_slide((720.0, 540.0)).unwrap();
        let shape = &resolved.shapes[0];
        assert!(matches!(shape.fill, Some(Paint::Solid(_))));
        assert!(shape.line.is_some());
        assert!(matches!(shape.shadow, Some(Effect::OuterShadow { .. })));
    }

    #[test]
    fn unsupported_content_keeps_bounds_and_diagnostic() {
        let frame = r#"<p:graphicFrame><p:nvGraphicFramePr/><p:xfrm><a:off x="0" y="0"/><a:ext cx="127000" cy="254000"/></p:xfrm><a:graphic><a:graphicData uri="urn:unsupported"><a:unknown/></a:graphicData></a:graphic></p:graphicFrame>"#;
        let fixture = Fixture::new(frame, "", "");

        let resolved = fixture.context().resolve_slide((720.0, 540.0)).unwrap();

        assert_eq!(resolved.shapes.len(), 1);
        assert_eq!(resolved.shapes[0].bounds.width, 10.0);
        assert_eq!(resolved.shapes[0].bounds.height, 20.0);
        assert_eq!(
            resolved.shapes[0].geometry,
            ResolvedGeometry::BoundsFallback
        );
        assert_eq!(
            resolved.shapes[0].unsupported,
            Some("unknown graphic frame")
        );
        assert_eq!(resolved.diagnostics.len(), 1);
    }

    #[test]
    fn resolved_shapes_follow_the_flattened_order() {
        let master = shape_with_details(None, None, &transform(12_700), None);
        let layout = shape_with_details(None, None, &transform(25_400), None);
        let slide = shape_with_details(None, None, &transform(38_100), None);
        let fixture = Fixture::new(&slide, &layout, &master);

        let resolved = fixture.context().resolve_slide((720.0, 540.0)).unwrap();
        let x = resolved
            .shapes
            .iter()
            .map(|shape| shape.bounds.x)
            .collect::<Vec<_>>();
        assert_eq!(x, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn all_corpus_slides_resolve_without_panics() {
        let stats = resolve_pinned_corpus();

        assert_eq!(stats.decks, EXPECTED_CORPUS_DECKS);
        assert_eq!(stats.resolved + stats.contextual_errors, stats.slides);
    }

    #[derive(Default)]
    struct CorpusResolveStats {
        decks: usize,
        slides: usize,
        resolved: usize,
        contextual_errors: usize,
        theme_references: usize,
    }

    fn resolve_pinned_corpus() -> CorpusResolveStats {
        let corpus = corpus_dir();
        assert!(
            corpus.is_dir(),
            "the required pinned corpus is missing at {}",
            corpus.display()
        );
        let entries = include_str!("../../../scripts/pptx-corpus-manifest.tsv")
            .lines()
            .skip(1)
            .map(|line| line.split('\t').next().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), EXPECTED_CORPUS_DECKS);

        let mut stats = CorpusResolveStats::default();
        for entry in entries {
            let path = corpus.join(entry);
            assert!(path.is_file(), "missing pinned deck {}", path.display());
            let package = OpcPackage::open(&path)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            let presentation_part = package
                .main_document_part()
                .unwrap_or_else(|| panic!("{}: no presentation part", path.display()));
            let presentation = CT_Presentation::from_xml(part(&package, &presentation_part))
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            let default_text_style = presentation.default_text_style.unwrap_or_default();
            let size = presentation.slide_size.map_or((720.0, 540.0), |size| {
                (
                    super::emu_to_points(size.cx.0),
                    super::emu_to_points(size.cy.0),
                )
            });
            let presentation_rels = package
                .get_part_rels(&presentation_part)
                .unwrap_or_else(|| panic!("{}: no presentation relationships", path.display()));

            for slide_id in presentation.slide_ids {
                stats.slides += 1;
                let slide_rel = presentation_rels
                    .get_by_id(&slide_id.relationship_id)
                    .unwrap_or_else(|| {
                        panic!(
                            "{}: missing slide relationship {}",
                            path.display(),
                            slide_id.relationship_id
                        )
                    });
                assert_eq!(slide_rel.rel_type, rel_types::SLIDE, "{}", path.display());
                let slide_part =
                    OpcPackage::resolve_rel_target(&presentation_part, &slide_rel.target);
                let layout_part =
                    related_part(&package, &slide_part, rel_types::SLIDE_LAYOUT, &path);
                let master_part =
                    related_part(&package, &layout_part, rel_types::SLIDE_MASTER, &path);
                let theme_part = related_part(&package, &master_part, rel_types::THEME, &path);
                let slide = CT_Slide::from_xml(part(&package, &slide_part))
                    .unwrap_or_else(|error| panic!("{} {slide_part}: {error}", path.display()));
                let layout = CT_SlideLayout::from_xml(part(&package, &layout_part))
                    .unwrap_or_else(|error| panic!("{} {layout_part}: {error}", path.display()));
                let master = CT_SlideMaster::from_xml(part(&package, &master_part))
                    .unwrap_or_else(|error| panic!("{} {master_part}: {error}", path.display()));
                let theme = CT_OfficeStyleSheet::from_xml(part(&package, &theme_part))
                    .unwrap_or_else(|error| panic!("{} {theme_part}: {error}", path.display()));
                let color_map = effective_corpus_color_map(&master, &layout, &slide);
                let context = ResolveCtx::new(
                    &theme,
                    color_map,
                    &master,
                    &layout,
                    &slide,
                    &default_text_style,
                );
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    context.resolve_slide(size)
                }))
                .unwrap_or_else(|_| panic!("{} {slide_part}: resolver panicked", path.display()));
                match result {
                    Ok(resolved) => {
                        let debug = format!("{resolved:?}");
                        stats.theme_references += ["schemeClr", "sysClr", "scrgbClr", "prstClr"]
                            .into_iter()
                            .filter(|marker| debug.contains(marker))
                            .count();
                        stats.resolved += 1;
                    }
                    Err(_) => stats.contextual_errors += 1,
                }
            }
            stats.decks += 1;
        }
        stats
    }

    fn corpus_dir() -> PathBuf {
        std::env::var_os("RDOCX_PPTX_CORPUS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| workspace_root().join("corpus/pptx"))
    }

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap()
            .to_path_buf()
    }

    fn part<'a>(package: &'a OpcPackage, part_name: &str) -> &'a [u8] {
        package
            .get_part(part_name)
            .unwrap_or_else(|| panic!("missing package part {part_name}"))
    }

    fn related_part(
        package: &OpcPackage,
        source_part: &str,
        relationship_type: &str,
        deck: &Path,
    ) -> String {
        let relationship = package
            .get_part_rels(source_part)
            .and_then(|relationships| relationships.get_by_type(relationship_type))
            .unwrap_or_else(|| {
                panic!(
                    "{} {source_part}: missing relationship {relationship_type}",
                    deck.display()
                )
            });
        OpcPackage::resolve_rel_target(source_part, &relationship.target)
    }

    fn effective_corpus_color_map(
        master: &CT_SlideMaster,
        layout: &CT_SlideLayout,
        slide: &CT_Slide,
    ) -> ColorMap {
        for override_value in [
            slide.color_map_override.as_ref(),
            layout.color_map_override.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            if let ColorMapOverrideKind::Override(map) = &override_value.kind {
                return map.clone();
            }
        }
        master.color_map.clone()
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
