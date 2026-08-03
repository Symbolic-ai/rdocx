//! Page-to-image rendering using tiny-skia software rasterizer.

use oxml_layout::{
    LayoutResult, LineCap, LineJoin, PageFrame, Paint as LayoutPaint, Path as LayoutPath,
    PathCommand, PositionedElement, Stroke as LayoutStroke,
};
use tiny_skia::{
    FillRule, Mask, Paint, PathBuilder, Pixmap, PixmapPaint, Stroke, StrokeDash, Transform,
};

/// Render a single page to PNG bytes.
///
/// # Arguments
/// * `layout` - The layout result containing all pages and font data
/// * `page_index` - 0-based page index to render
/// * `dpi` - Dots per inch (72 = 1:1 with points, 150 = 2x, 300 = 4.17x)
pub fn render_page_to_png(layout: &LayoutResult, page_index: usize, dpi: f64) -> Option<Vec<u8>> {
    let page = layout.pages.get(page_index)?;
    let pixmap = render_page_to_pixmap(page, &layout.fonts, dpi)?;
    pixmap.encode_png().ok()
}

/// Render all pages to PNG bytes.
pub fn render_all_pages(layout: &LayoutResult, dpi: f64) -> Vec<Vec<u8>> {
    layout
        .pages
        .iter()
        .filter_map(|page| {
            let pixmap = render_page_to_pixmap(page, &layout.fonts, dpi)?;
            pixmap.encode_png().ok()
        })
        .collect()
}

/// Render a page to a Pixmap.
fn render_page_to_pixmap(
    page: &PageFrame,
    fonts: &[oxml_layout::FontData],
    dpi: f64,
) -> Option<Pixmap> {
    let scale = dpi / 72.0; // points to pixels
    let width = (page.width * scale).ceil() as u32;
    let height = (page.height * scale).ceil() as u32;

    let mut pixmap = Pixmap::new(width, height)?;

    pixmap.fill(tiny_skia::Color::WHITE);

    let transform = Transform::from_scale(scale as f32, scale as f32);

    if let Some(background) = &page.background {
        let path = LayoutPath::rect(oxml_layout::Rect {
            x: 0.0,
            y: 0.0,
            width: page.width,
            height: page.height,
        });
        render_path(&mut pixmap, &path, Some(background), None, transform, None);
    }
    render_elements(&mut pixmap, fonts, &page.elements, transform, None);

    Some(pixmap)
}

fn render_elements(
    pixmap: &mut Pixmap,
    fonts: &[oxml_layout::FontData],
    elements: &[PositionedElement],
    transform: Transform,
    mask: Option<&Mask>,
) {
    for element in elements {
        match element {
            PositionedElement::FilledRect { rect, color } => {
                let Some(rect) = tiny_skia::Rect::from_xywh(
                    rect.x as f32,
                    rect.y as f32,
                    rect.width as f32,
                    rect.height as f32,
                ) else {
                    continue;
                };
                let mut paint = solid_paint(*color);
                paint.anti_alias = false;
                pixmap.fill_rect(rect, &paint, transform, mask);
            }
            PositionedElement::Line {
                start,
                end,
                width,
                color,
                dash_pattern,
            } => {
                let mut builder = PathBuilder::new();
                builder.move_to(start.x as f32, start.y as f32);
                builder.line_to(end.x as f32, end.y as f32);
                let Some(path) = builder.finish() else {
                    continue;
                };
                let dash = dash_pattern
                    .and_then(|(on, off)| StrokeDash::new(vec![on as f32, off as f32], 0.0));
                let stroke = Stroke {
                    width: *width as f32,
                    dash,
                    ..Stroke::default()
                };
                pixmap.stroke_path(&path, &solid_paint(*color), &stroke, transform, mask);
            }
            PositionedElement::Text(glyph_run) => {
                if let Some(font_data) = fonts.iter().find(|font| font.id == glyph_run.font_id) {
                    render_glyph_run(pixmap, glyph_run, font_data, transform, mask);
                }
            }
            PositionedElement::Image {
                rect,
                data,
                content_type,
                ..
            } => {
                if !data.is_empty() {
                    render_image(pixmap, rect, data, content_type, transform, mask);
                }
            }
            PositionedElement::LinkAnnotation { .. } => {}
            PositionedElement::Path(element) => render_path(
                pixmap,
                &element.path,
                element.fill.as_ref(),
                element.stroke.as_ref(),
                transform,
                mask,
            ),
            PositionedElement::Group(group) => {
                let child_transform = transform.pre_concat(layout_transform(group.transform));
                let child_mask = match group.clip.as_ref() {
                    Some(clip) => {
                        let Some(path) = build_path(clip) else {
                            continue;
                        };
                        let mut child_mask = match mask {
                            Some(parent) => parent.clone(),
                            None => {
                                let Some(mask) = Mask::new(pixmap.width(), pixmap.height()) else {
                                    continue;
                                };
                                mask
                            }
                        };
                        if mask.is_some() {
                            child_mask.intersect_path(
                                &path,
                                fill_rule(clip.fill_rule),
                                true,
                                child_transform,
                            );
                        } else {
                            child_mask.fill_path(
                                &path,
                                fill_rule(clip.fill_rule),
                                true,
                                child_transform,
                            );
                        }
                        Some(child_mask)
                    }
                    None => None,
                };
                let child_mask_ref = child_mask.as_ref().or(mask);
                let opacity = if group.opacity.is_finite() {
                    group.opacity.clamp(0.0, 1.0) as f32
                } else {
                    1.0
                };
                if opacity == 0.0 {
                    continue;
                }
                if opacity == 1.0 {
                    render_elements(
                        pixmap,
                        fonts,
                        &group.children,
                        child_transform,
                        child_mask_ref,
                    );
                } else if let Some(mut layer) = Pixmap::new(pixmap.width(), pixmap.height()) {
                    render_elements(
                        &mut layer,
                        fonts,
                        &group.children,
                        child_transform,
                        child_mask_ref,
                    );
                    pixmap.draw_pixmap(
                        0,
                        0,
                        layer.as_ref(),
                        &PixmapPaint {
                            opacity,
                            ..PixmapPaint::default()
                        },
                        Transform::identity(),
                        None,
                    );
                }
            }
            _ => {}
        }
    }
}

fn render_path(
    pixmap: &mut Pixmap,
    path: &LayoutPath,
    fill: Option<&LayoutPaint>,
    stroke: Option<&LayoutStroke>,
    transform: Transform,
    mask: Option<&Mask>,
) {
    let Some(path_geometry) = build_path(path) else {
        return;
    };
    if let Some(paint) = fill.and_then(raster_paint) {
        let paint_mask =
            fill.and_then(|paint| gradient_domain_mask(pixmap, paint, transform, mask));
        pixmap.fill_path(
            &path_geometry,
            &paint,
            fill_rule(path.fill_rule),
            transform,
            paint_mask.as_ref().or(mask),
        );
    }
    if let Some(stroke) = stroke
        && let Some(paint) = raster_paint(&stroke.paint)
    {
        let paint_mask = gradient_domain_mask(pixmap, &stroke.paint, transform, mask);
        let stroke = raster_stroke(stroke);
        pixmap.stroke_path(
            &path_geometry,
            &paint,
            &stroke,
            transform,
            paint_mask.as_ref().or(mask),
        );
    }
}

fn build_path(path: &LayoutPath) -> Option<tiny_skia::Path> {
    let mut builder = PathBuilder::new();
    for command in &path.commands {
        match command {
            PathCommand::MoveTo(point) => builder.move_to(point.x as f32, point.y as f32),
            PathCommand::LineTo(point) => builder.line_to(point.x as f32, point.y as f32),
            PathCommand::CurveTo { c1, c2, to } => builder.cubic_to(
                c1.x as f32,
                c1.y as f32,
                c2.x as f32,
                c2.y as f32,
                to.x as f32,
                to.y as f32,
            ),
            PathCommand::Close => builder.close(),
        }
    }
    builder.finish()
}

fn fill_rule(rule: oxml_layout::FillRule) -> FillRule {
    match rule {
        oxml_layout::FillRule::NonZero => FillRule::Winding,
        oxml_layout::FillRule::EvenOdd => FillRule::EvenOdd,
    }
}

fn raster_paint(paint: &LayoutPaint) -> Option<Paint<'static>> {
    match paint {
        LayoutPaint::Solid(color) => Some(solid_paint(*color)),
        LayoutPaint::Linear {
            start, end, stops, ..
        } => {
            let shader = tiny_skia::LinearGradient::new(
                tiny_skia::Point::from_xy(start.x as f32, start.y as f32),
                tiny_skia::Point::from_xy(end.x as f32, end.y as f32),
                gradient_stops(stops),
                tiny_skia::SpreadMode::Pad,
                Transform::identity(),
            )?;
            Some(Paint {
                shader,
                ..Paint::default()
            })
        }
        LayoutPaint::Radial {
            center,
            radius,
            focal,
            stops,
            ..
        } => {
            let shader = tiny_skia::RadialGradient::new(
                tiny_skia::Point::from_xy(focal.x as f32, focal.y as f32),
                0.0,
                tiny_skia::Point::from_xy(center.x as f32, center.y as f32),
                *radius as f32,
                gradient_stops(stops),
                tiny_skia::SpreadMode::Pad,
                Transform::identity(),
            )?;
            Some(Paint {
                shader,
                ..Paint::default()
            })
        }
        LayoutPaint::Tile { .. } => None,
    }
}

fn gradient_domain_mask(
    pixmap: &Pixmap,
    paint: &LayoutPaint,
    transform: Transform,
    parent: Option<&Mask>,
) -> Option<Mask> {
    let path = match paint {
        LayoutPaint::Linear {
            start, end, extend, ..
        } if *extend != (true, true) => linear_domain_path(
            pixmap.width(),
            pixmap.height(),
            *start,
            *end,
            *extend,
            transform,
        )?,
        LayoutPaint::Radial {
            center,
            radius,
            extend,
            ..
        } if !extend.1 => build_path(&LayoutPath::ellipse(oxml_layout::Rect {
            x: center.x - radius,
            y: center.y - radius,
            width: radius * 2.0,
            height: radius * 2.0,
        }))?,
        _ => return None,
    };
    let mut domain = match parent {
        Some(parent) => parent.clone(),
        None => Mask::new(pixmap.width(), pixmap.height())?,
    };
    if parent.is_some() {
        domain.intersect_path(&path, FillRule::Winding, true, transform);
    } else {
        domain.fill_path(&path, FillRule::Winding, true, transform);
    }
    Some(domain)
}

fn linear_domain_path(
    width: u32,
    height: u32,
    start: oxml_layout::Point,
    end: oxml_layout::Point,
    extend: (bool, bool),
    transform: Transform,
) -> Option<tiny_skia::Path> {
    let inverse = transform.invert()?;
    let mut corners = [
        tiny_skia::Point::from_xy(0.0, 0.0),
        tiny_skia::Point::from_xy(width as f32, 0.0),
        tiny_skia::Point::from_xy(width as f32, height as f32),
        tiny_skia::Point::from_xy(0.0, height as f32),
    ];
    inverse.map_points(&mut corners);

    let dx = (end.x - start.x) as f32;
    let dy = (end.y - start.y) as f32;
    let length_squared = dx * dx + dy * dy;
    if !length_squared.is_finite() || length_squared <= f32::EPSILON {
        return None;
    }
    let length = length_squared.sqrt();
    let normal_x = -dy / length;
    let normal_y = dx / length;
    let mut min_t = f32::INFINITY;
    let mut max_t = f32::NEG_INFINITY;
    let mut min_normal = f32::INFINITY;
    let mut max_normal = f32::NEG_INFINITY;
    for corner in corners {
        let x = corner.x - start.x as f32;
        let y = corner.y - start.y as f32;
        let t = (x * dx + y * dy) / length_squared;
        let normal = x * normal_x + y * normal_y;
        min_t = min_t.min(t);
        max_t = max_t.max(t);
        min_normal = min_normal.min(normal);
        max_normal = max_normal.max(normal);
    }
    let min_t = if extend.0 { min_t - 1.0 } else { 0.0 };
    let max_t = if extend.1 { max_t + 1.0 } else { 1.0 };
    let min_normal = min_normal - 1.0;
    let max_normal = max_normal + 1.0;
    let point = |t: f32, normal: f32| {
        tiny_skia::Point::from_xy(
            start.x as f32 + dx * t + normal_x * normal,
            start.y as f32 + dy * t + normal_y * normal,
        )
    };
    let mut builder = PathBuilder::new();
    let first = point(min_t, min_normal);
    builder.move_to(first.x, first.y);
    for corner in [
        point(max_t, min_normal),
        point(max_t, max_normal),
        point(min_t, max_normal),
    ] {
        builder.line_to(corner.x, corner.y);
    }
    builder.close();
    builder.finish()
}

fn gradient_stops(stops: &[oxml_layout::GradientStop]) -> Vec<tiny_skia::GradientStop> {
    let mut normalized: Vec<(f64, oxml_layout::Color)> = stops
        .iter()
        .map(|stop| {
            let offset = if stop.offset.is_nan() {
                0.0
            } else {
                stop.offset.clamp(0.0, 1.0)
            };
            (offset, stop.color)
        })
        .collect();
    normalized.sort_by(|left, right| left.0.total_cmp(&right.0));

    let mut deduplicated: Vec<(f64, oxml_layout::Color)> = Vec::with_capacity(normalized.len());
    for stop in normalized {
        if let Some(previous) = deduplicated.last_mut()
            && previous.0 == stop.0
        {
            *previous = stop;
        } else {
            deduplicated.push(stop);
        }
    }

    deduplicated
        .into_iter()
        .map(|(offset, color)| tiny_skia::GradientStop::new(offset as f32, skia_color(color)))
        .collect()
}

fn solid_paint(color: oxml_layout::Color) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color_rgba8(
        (color.r.clamp(0.0, 1.0) * 255.0) as u8,
        (color.g.clamp(0.0, 1.0) * 255.0) as u8,
        (color.b.clamp(0.0, 1.0) * 255.0) as u8,
        (color.a.clamp(0.0, 1.0) * 255.0) as u8,
    );
    paint
}

fn skia_color(color: oxml_layout::Color) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba(
        color.r.clamp(0.0, 1.0) as f32,
        color.g.clamp(0.0, 1.0) as f32,
        color.b.clamp(0.0, 1.0) as f32,
        color.a.clamp(0.0, 1.0) as f32,
    )
    .unwrap_or(tiny_skia::Color::TRANSPARENT)
}

fn raster_stroke(stroke: &LayoutStroke) -> Stroke {
    Stroke {
        width: stroke.width as f32,
        line_cap: match stroke.cap {
            LineCap::Butt => tiny_skia::LineCap::Butt,
            LineCap::Round => tiny_skia::LineCap::Round,
            LineCap::Square => tiny_skia::LineCap::Square,
        },
        line_join: match stroke.join {
            LineJoin::Miter => tiny_skia::LineJoin::Miter,
            LineJoin::Round => tiny_skia::LineJoin::Round,
            LineJoin::Bevel => tiny_skia::LineJoin::Bevel,
        },
        dash: stroke.dash.as_ref().and_then(|dash| {
            StrokeDash::new(dash.iter().map(|value| *value as f32).collect(), 0.0)
        }),
        ..Stroke::default()
    }
}

fn layout_transform(transform: oxml_layout::Transform) -> Transform {
    Transform::from_row(
        transform.a as f32,
        transform.b as f32,
        transform.c as f32,
        transform.d as f32,
        transform.e as f32,
        transform.f as f32,
    )
}

/// Render a glyph run by extracting glyph outlines from the font.
fn render_glyph_run(
    pixmap: &mut Pixmap,
    glyph_run: &oxml_layout::GlyphRun,
    font_data: &oxml_layout::FontData,
    transform: Transform,
    mask: Option<&Mask>,
) {
    let Ok(face) = ttf_parser::Face::parse(&font_data.data, font_data.face_index) else {
        return;
    };

    let upem = face.units_per_em() as f64;
    let scale = glyph_run.font_size / upem;

    let mut paint = Paint::default();
    paint.set_color_rgba8(
        (glyph_run.color.r * 255.0) as u8,
        (glyph_run.color.g * 255.0) as u8,
        (glyph_run.color.b * 255.0) as u8,
        (glyph_run.color.a * 255.0) as u8,
    );
    paint.anti_alias = true;

    let mut x = glyph_run.origin.x;
    let y = glyph_run.origin.y;

    for (i, &glyph_id) in glyph_run.glyph_ids.iter().enumerate() {
        let gid = ttf_parser::GlyphId(glyph_id);

        // Build path from glyph outline
        let mut builder = GlyphPathBuilder::new();
        if face.outline_glyph(gid, &mut builder).is_some()
            && let Some(path) = builder.finish()
        {
            // Transform: translate to glyph position, scale from font units to points,
            // and flip Y (font coordinates are Y-up, pixmap is Y-down)
            let glyph_transform = transform
                .pre_translate(x as f32, y as f32)
                .pre_scale(scale as f32, -(scale as f32));

            pixmap.fill_path(&path, &paint, FillRule::Winding, glyph_transform, mask);
        }

        if i < glyph_run.advances.len() {
            x += glyph_run.advances[i];
        }
    }
}

/// Convert a glyph outline to a tiny-skia Path.
struct GlyphPathBuilder {
    pb: PathBuilder,
}

impl GlyphPathBuilder {
    fn new() -> Self {
        GlyphPathBuilder {
            pb: PathBuilder::new(),
        }
    }

    fn finish(self) -> Option<tiny_skia::Path> {
        self.pb.finish()
    }
}

impl ttf_parser::OutlineBuilder for GlyphPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.pb.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.pb.line_to(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.pb.quad_to(x1, y1, x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.pb.cubic_to(x1, y1, x2, y2, x, y);
    }

    fn close(&mut self) {
        self.pb.close();
    }
}

/// Render an image onto the pixmap.
fn render_image(
    pixmap: &mut Pixmap,
    rect: &oxml_layout::Rect,
    data: &[u8],
    _content_type: &str,
    transform: Transform,
    mask: Option<&Mask>,
) {
    // Decode the image
    let decoded = crate::image::decode_image(data, _content_type);
    let Some(decoded) = decoded else {
        return;
    };

    // Create a pixmap from the decoded image data
    let img_pixmap = if decoded.is_jpeg || decoded.color_space == "DeviceRGB" {
        // Convert RGB to RGBA
        let mut rgba = Vec::with_capacity(decoded.width as usize * decoded.height as usize * 4);
        if let Some(alpha) = &decoded.alpha {
            for (rgb, &a) in decoded.data.chunks_exact(3).zip(alpha.iter()) {
                rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], a]);
            }
        } else {
            for rgb in decoded.data.chunks_exact(3) {
                rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
        }
        let size = tiny_skia::IntSize::from_wh(decoded.width, decoded.height);
        size.and_then(|s| Pixmap::from_vec(rgba, s))
    } else if decoded.color_space == "DeviceGray" {
        // Convert grayscale to RGBA
        let mut rgba = Vec::with_capacity(decoded.width as usize * decoded.height as usize * 4);
        for &gray in &decoded.data {
            rgba.push(gray);
            rgba.push(gray);
            rgba.push(gray);
            rgba.push(255);
        }
        let size = tiny_skia::IntSize::from_wh(decoded.width, decoded.height);
        size.and_then(|s| Pixmap::from_vec(rgba, s))
    } else {
        return;
    };

    let Some(img_pixmap) = img_pixmap else {
        return;
    };

    // Calculate the transform to position and scale the image. A zero
    // dimension from a malformed image would make this an infinite scale.
    if decoded.width == 0 || decoded.height == 0 {
        return;
    }
    let sx = rect.width as f32 / decoded.width as f32;
    let sy = rect.height as f32 / decoded.height as f32;

    let img_transform = transform
        .pre_translate(rect.x as f32, rect.y as f32)
        .pre_scale(sx, sy);

    let pattern = tiny_skia::Pattern::new(
        img_pixmap.as_ref(),
        tiny_skia::SpreadMode::Pad,
        tiny_skia::FilterQuality::Bilinear,
        1.0,
        Transform::identity(),
    );

    let paint = Paint {
        shader: pattern,
        ..Paint::default()
    };

    let fill_rect =
        tiny_skia::Rect::from_xywh(0.0, 0.0, decoded.width as f32, decoded.height as f32);
    if let Some(fill_rect) = fill_rect {
        pixmap.fill_rect(fill_rect, &paint, img_transform, mask);
    }
}

#[cfg(test)]
mod tests {
    use super::render_page_to_pixmap;
    use oxml_layout::{
        Color, FillRule, GradientStop, GroupElement, PageFrame, Paint, Path, PathCommand,
        PathElement, Point, PositionedElement, Rect, Stroke, Transform,
    };

    fn rgb(pixel: tiny_skia::PremultipliedColorU8) -> (u8, u8, u8) {
        (pixel.red(), pixel.green(), pixel.blue())
    }

    fn solid_path(path: Path, color: Color) -> PositionedElement {
        PositionedElement::Path(PathElement {
            path,
            fill: Some(Paint::Solid(color)),
            stroke: None,
        })
    }

    fn page(elements: Vec<PositionedElement>) -> PageFrame {
        PageFrame::new(1, 32.0, 32.0, elements)
    }

    #[test]
    fn half_alpha_over_white_rasterises_to_midpoint() {
        let page = PageFrame::new(
            1,
            1.0,
            1.0,
            vec![PositionedElement::FilledRect {
                rect: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 1.0,
                    height: 1.0,
                },
                color: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.5,
                },
            }],
        );

        let pixmap = render_page_to_pixmap(&page, &[], 72.0).expect("one-pixel page");
        let pixel = pixmap.pixel(0, 0).expect("one-pixel page has one pixel");
        assert_eq!((pixel.red(), pixel.green(), pixel.blue()), (128, 128, 128));
    }

    #[test]
    fn rotated_rectangle_has_a_filled_interior_and_empty_corner() {
        let element = PositionedElement::Group(GroupElement {
            transform: Transform::rotate_about(45.0, 16.0, 16.0),
            clip: None,
            opacity: 1.0,
            effects: Vec::new(),
            children: vec![solid_path(
                Path::rect(Rect {
                    x: 10.0,
                    y: 12.0,
                    width: 12.0,
                    height: 8.0,
                }),
                Color::BLACK,
            )],
        });
        let pixmap = render_page_to_pixmap(&page(vec![element]), &[], 72.0).unwrap();

        assert_eq!(rgb(pixmap.pixel(16, 16).unwrap()), (0, 0, 0));
        assert_eq!(rgb(pixmap.pixel(9, 9).unwrap()), (255, 255, 255));
    }

    #[test]
    fn dashed_line_contains_deterministic_gaps() {
        let element = PositionedElement::Line {
            start: Point { x: 2.0, y: 4.5 },
            end: Point { x: 30.0, y: 4.5 },
            width: 1.0,
            color: Color::BLACK,
            dash_pattern: Some((4.0, 4.0)),
        };
        let pixmap = render_page_to_pixmap(&page(vec![element]), &[], 72.0).unwrap();

        assert_eq!(rgb(pixmap.pixel(3, 4).unwrap()), (0, 0, 0));
        assert_eq!(rgb(pixmap.pixel(7, 4).unwrap()), (255, 255, 255));
        assert_eq!(rgb(pixmap.pixel(11, 4).unwrap()), (0, 0, 0));
    }

    #[test]
    fn nested_group_transforms_apply_child_before_parent() {
        let translated = |transform, children| {
            PositionedElement::Group(GroupElement {
                transform,
                clip: None,
                opacity: 1.0,
                effects: Vec::new(),
                children,
            })
        };
        let leaf = solid_path(
            Path::rect(Rect {
                x: 0.0,
                y: 0.0,
                width: 2.0,
                height: 2.0,
            }),
            Color::BLACK,
        );
        let element = translated(
            Transform {
                e: 4.0,
                ..Transform::IDENTITY
            },
            vec![translated(
                Transform {
                    e: 3.0,
                    ..Transform::IDENTITY
                },
                vec![translated(
                    Transform {
                        f: 5.0,
                        ..Transform::IDENTITY
                    },
                    vec![leaf],
                )],
            )],
        );
        let pixmap = render_page_to_pixmap(&page(vec![element]), &[], 72.0).unwrap();

        assert_eq!(rgb(pixmap.pixel(7, 5).unwrap()), (0, 0, 0));
        assert_eq!(rgb(pixmap.pixel(1, 1).unwrap()), (255, 255, 255));
    }

    #[test]
    fn group_clip_masks_children_and_preserves_outside_pixels() {
        let element = PositionedElement::Group(GroupElement {
            transform: Transform::IDENTITY,
            clip: Some(Path::rect(Rect {
                x: 4.0,
                y: 4.0,
                width: 8.0,
                height: 12.0,
            })),
            opacity: 1.0,
            effects: Vec::new(),
            children: vec![PositionedElement::Group(GroupElement {
                transform: Transform::IDENTITY,
                clip: Some(Path::rect(Rect {
                    x: 8.0,
                    y: 4.0,
                    width: 8.0,
                    height: 12.0,
                })),
                opacity: 1.0,
                effects: Vec::new(),
                children: vec![solid_path(
                    Path::rect(Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 24.0,
                        height: 24.0,
                    }),
                    Color::BLACK,
                )],
            })],
        });
        let pixmap = render_page_to_pixmap(&page(vec![element]), &[], 72.0).unwrap();

        assert_eq!(rgb(pixmap.pixel(10, 10).unwrap()), (0, 0, 0));
        assert_eq!(rgb(pixmap.pixel(6, 10).unwrap()), (255, 255, 255));
        assert_eq!(rgb(pixmap.pixel(14, 10).unwrap()), (255, 255, 255));
    }

    #[test]
    fn group_opacity_is_applied_once_to_the_composited_subtree() {
        let element = PositionedElement::Group(GroupElement {
            transform: Transform::IDENTITY,
            clip: None,
            opacity: 0.5,
            effects: Vec::new(),
            children: vec![
                solid_path(
                    Path::rect(Rect {
                        x: 2.0,
                        y: 2.0,
                        width: 12.0,
                        height: 12.0,
                    }),
                    Color::BLACK,
                ),
                solid_path(
                    Path::rect(Rect {
                        x: 8.0,
                        y: 2.0,
                        width: 12.0,
                        height: 12.0,
                    }),
                    Color::BLACK,
                ),
            ],
        });
        let pixmap = render_page_to_pixmap(&page(vec![element]), &[], 72.0).unwrap();

        assert_eq!(rgb(pixmap.pixel(4, 4).unwrap()), (128, 128, 128));
        assert_eq!(rgb(pixmap.pixel(10, 4).unwrap()), (128, 128, 128));
    }

    #[test]
    fn path_fill_rule_selects_the_tiny_skia_rule() {
        let nested = |fill_rule| Path {
            commands: vec![
                PathCommand::MoveTo(Point { x: 2.0, y: 2.0 }),
                PathCommand::LineTo(Point { x: 14.0, y: 2.0 }),
                PathCommand::LineTo(Point { x: 14.0, y: 14.0 }),
                PathCommand::LineTo(Point { x: 2.0, y: 14.0 }),
                PathCommand::Close,
                PathCommand::MoveTo(Point { x: 5.0, y: 5.0 }),
                PathCommand::LineTo(Point { x: 11.0, y: 5.0 }),
                PathCommand::LineTo(Point { x: 11.0, y: 11.0 }),
                PathCommand::LineTo(Point { x: 5.0, y: 11.0 }),
                PathCommand::Close,
            ],
            fill_rule,
        };
        let nonzero = render_page_to_pixmap(
            &page(vec![solid_path(nested(FillRule::NonZero), Color::BLACK)]),
            &[],
            72.0,
        )
        .unwrap();
        let evenodd = render_page_to_pixmap(
            &page(vec![solid_path(nested(FillRule::EvenOdd), Color::BLACK)]),
            &[],
            72.0,
        )
        .unwrap();

        assert_eq!(rgb(nonzero.pixel(8, 8).unwrap()), (0, 0, 0));
        assert_eq!(rgb(evenodd.pixel(8, 8).unwrap()), (255, 255, 255));
    }

    #[test]
    fn linear_and_radial_gradients_sample_expected_colours() {
        let stops = vec![
            GradientStop {
                offset: 0.0,
                color: Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
            },
            GradientStop {
                offset: 1.0,
                color: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                    a: 1.0,
                },
            },
        ];
        let linear = PositionedElement::Path(PathElement {
            path: Path::rect(Rect {
                x: 0.0,
                y: 0.0,
                width: 16.0,
                height: 8.0,
            }),
            fill: Some(Paint::linear(
                Point { x: 0.0, y: 0.0 },
                Point { x: 16.0, y: 0.0 },
                stops.clone(),
                (true, true),
            )),
            stroke: None,
        });
        let radial = PositionedElement::Path(PathElement {
            path: Path::rect(Rect {
                x: 0.0,
                y: 16.0,
                width: 16.0,
                height: 16.0,
            }),
            fill: Some(Paint::radial(
                Point { x: 8.0, y: 24.0 },
                8.0,
                Point { x: 8.0, y: 24.0 },
                stops,
                (true, true),
            )),
            stroke: None,
        });
        let pixmap = render_page_to_pixmap(&page(vec![linear, radial]), &[], 72.0).unwrap();

        let linear_start = rgb(pixmap.pixel(1, 4).unwrap());
        let linear_end = rgb(pixmap.pixel(14, 4).unwrap());
        assert!(linear_start.0 > linear_start.2);
        assert!(linear_end.2 > linear_end.0);
        let radial_center = rgb(pixmap.pixel(8, 24).unwrap());
        let radial_edge = rgb(pixmap.pixel(1, 24).unwrap());
        assert!(radial_center.0 > radial_center.2);
        assert!(radial_edge.2 > radial_edge.0);
    }

    #[test]
    fn translated_group_gradient_uses_local_coordinates_exactly_once() {
        let red = Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let blue = Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        };
        let element = PositionedElement::Group(GroupElement {
            transform: Transform {
                e: 12.0,
                ..Transform::IDENTITY
            },
            clip: None,
            opacity: 1.0,
            effects: Vec::new(),
            children: vec![PositionedElement::Path(PathElement {
                path: Path::rect(Rect {
                    x: 0.0,
                    y: 2.0,
                    width: 8.0,
                    height: 8.0,
                }),
                fill: Some(Paint::linear(
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 8.0, y: 0.0 },
                    vec![
                        GradientStop {
                            offset: 0.0,
                            color: red,
                        },
                        GradientStop {
                            offset: 1.0,
                            color: blue,
                        },
                    ],
                    (true, true),
                )),
                stroke: None,
            })],
        });
        let pixmap = render_page_to_pixmap(&page(vec![element]), &[], 72.0).unwrap();

        let start = rgb(pixmap.pixel(13, 6).unwrap());
        let end = rgb(pixmap.pixel(18, 6).unwrap());
        assert!(
            start.0 > start.2,
            "expected red-dominant start, got {start:?}"
        );
        assert!(end.2 > end.0, "expected blue-dominant end, got {end:?}");
        assert_eq!(rgb(pixmap.pixel(9, 6).unwrap()), (255, 255, 255));
    }

    #[test]
    fn gradient_extend_flags_leave_outside_regions_unpainted() {
        let stops = vec![
            GradientStop {
                offset: 0.0,
                color: Color::BLACK,
            },
            GradientStop {
                offset: 1.0,
                color: Color::WHITE,
            },
        ];
        let linear = PositionedElement::Path(PathElement {
            path: Path::rect(Rect {
                x: 0.0,
                y: 0.0,
                width: 32.0,
                height: 12.0,
            }),
            fill: Some(Paint::linear(
                Point { x: 8.0, y: 0.0 },
                Point { x: 24.0, y: 0.0 },
                stops.clone(),
                (false, false),
            )),
            stroke: None,
        });
        let radial = PositionedElement::Path(PathElement {
            path: Path::rect(Rect {
                x: 0.0,
                y: 16.0,
                width: 32.0,
                height: 16.0,
            }),
            fill: Some(Paint::radial(
                Point { x: 16.0, y: 24.0 },
                6.0,
                Point { x: 16.0, y: 24.0 },
                stops,
                (false, false),
            )),
            stroke: None,
        });
        let pixmap = render_page_to_pixmap(&page(vec![linear, radial]), &[], 72.0).unwrap();

        assert_eq!(rgb(pixmap.pixel(4, 6).unwrap()), (255, 255, 255));
        assert_ne!(rgb(pixmap.pixel(12, 6).unwrap()), (255, 255, 255));
        assert_ne!(rgb(pixmap.pixel(16, 24).unwrap()), (255, 255, 255));
        assert_eq!(rgb(pixmap.pixel(24, 24).unwrap()), (255, 255, 255));
    }

    #[test]
    fn gradient_stops_are_sorted_clamped_and_last_repeated_offset_wins() {
        let element = PositionedElement::Path(PathElement {
            path: Path::rect(Rect {
                x: 0.0,
                y: 0.0,
                width: 16.0,
                height: 8.0,
            }),
            fill: Some(Paint::linear(
                Point { x: 0.0, y: 0.0 },
                Point { x: 16.0, y: 0.0 },
                vec![
                    GradientStop {
                        offset: 2.0,
                        color: Color {
                            r: 0.0,
                            g: 0.0,
                            b: 1.0,
                            a: 1.0,
                        },
                    },
                    GradientStop {
                        offset: -1.0,
                        color: Color {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        },
                    },
                    GradientStop {
                        offset: f64::NAN,
                        color: Color {
                            r: 0.0,
                            g: 1.0,
                            b: 0.0,
                            a: 1.0,
                        },
                    },
                ],
                (true, true),
            )),
            stroke: None,
        });
        let pixmap = render_page_to_pixmap(&page(vec![element]), &[], 72.0).unwrap();

        let start = rgb(pixmap.pixel(1, 4).unwrap());
        let end = rgb(pixmap.pixel(14, 4).unwrap());
        assert!(start.1 > start.0 && start.1 > start.2);
        assert!(end.2 > end.0 && end.2 > end.1);
    }

    #[test]
    fn page_background_replaces_the_white_default() {
        let mut painted = page(Vec::new());
        painted.background = Some(Paint::Solid(Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }));
        let painted = render_page_to_pixmap(&painted, &[], 72.0).unwrap();
        let defaulted = render_page_to_pixmap(&page(Vec::new()), &[], 72.0).unwrap();
        let mut gradient = page(Vec::new());
        gradient.background = Some(Paint::linear(
            Point { x: 0.0, y: 0.0 },
            Point { x: 32.0, y: 0.0 },
            vec![
                GradientStop {
                    offset: 0.0,
                    color: Color::BLACK,
                },
                GradientStop {
                    offset: 1.0,
                    color: Color::WHITE,
                },
            ],
            (true, true),
        ));
        let gradient = render_page_to_pixmap(&gradient, &[], 72.0).unwrap();

        assert_eq!(rgb(painted.pixel(4, 4).unwrap()), (0, 255, 0));
        assert_eq!(rgb(defaulted.pixel(4, 4).unwrap()), (255, 255, 255));
        assert!(gradient.pixel(4, 4).unwrap().red() < gradient.pixel(28, 4).unwrap().red());
    }

    #[test]
    fn path_dash_array_contains_deterministic_gaps() {
        let mut stroke = Stroke::new(Paint::Solid(Color::BLACK), 1.0);
        stroke.dash = Some(vec![4.0, 4.0]);
        let element = PositionedElement::Path(PathElement {
            path: Path {
                commands: vec![
                    PathCommand::MoveTo(Point { x: 2.0, y: 8.5 }),
                    PathCommand::LineTo(Point { x: 30.0, y: 8.5 }),
                ],
                fill_rule: FillRule::NonZero,
            },
            fill: None,
            stroke: Some(stroke),
        });
        let pixmap = render_page_to_pixmap(&page(vec![element]), &[], 72.0).unwrap();

        assert_eq!(rgb(pixmap.pixel(3, 8).unwrap()), (0, 0, 0));
        assert_eq!(rgb(pixmap.pixel(7, 8).unwrap()), (255, 255, 255));
    }
}
