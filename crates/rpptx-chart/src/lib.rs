#![allow(non_camel_case_types)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_core::xml::{local_name, matches_local_name};
use oxml_core::xml_text::{decode_plain, resolve_entity};
use oxml_drawing::order::OrderedRawChildren;
use oxml_drawing::shape_props::{CT_ShapeProperties, ShapePropertiesError};
use oxml_drawing::text::{CT_TextBody, TextError};
use oxml_layout::{
    Color, FillRule, GroupElement, Paint, Path, PathCommand, PathElement, Point, PositionedElement,
    Rect, Stroke, Transform,
};
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer, XmlVersion};

/// The transitional ChartML namespace used by PresentationML packages.
pub const C_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/chart";
/// The fixed ChartML prefix used by writers in this crate.
pub const C_PREFIX: &str = "c";
/// The DrawingML namespace written on a chart-space root.
pub const A_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
/// The office-document relationships namespace written on a chart-space root.
pub const R_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// Errors produced while reading or writing the implemented ChartML core.
#[derive(Debug)]
pub enum ChartError {
    Xml(OxmlError),
    ShapeProperties(ShapePropertiesError),
    Text(TextError),
    UnexpectedElement(String),
    MissingElement(String),
    DuplicateElement(String),
    InvalidAttribute {
        element: String,
        attribute: String,
        value: String,
    },
    InvalidValue {
        element: String,
        value: String,
    },
}

impl fmt::Display for ChartError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml(error) => error.fmt(formatter),
            Self::ShapeProperties(error) => error.fmt(formatter),
            Self::Text(error) => error.fmt(formatter),
            Self::UnexpectedElement(element) => {
                write!(formatter, "unexpected ChartML element: {element}")
            }
            Self::MissingElement(element) => {
                write!(formatter, "ChartML requires {element}")
            }
            Self::DuplicateElement(element) => {
                write!(formatter, "ChartML contains duplicate {element}")
            }
            Self::InvalidAttribute {
                element,
                attribute,
                value,
            } => write!(
                formatter,
                "ChartML {element} has invalid @{attribute}: {value}"
            ),
            Self::InvalidValue { element, value } => {
                write!(formatter, "ChartML {element} has invalid value: {value}")
            }
        }
    }
}

impl std::error::Error for ChartError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml(error) => Some(error),
            Self::ShapeProperties(error) => Some(error),
            Self::Text(error) => Some(error),
            _ => None,
        }
    }
}

impl From<OxmlError> for ChartError {
    fn from(error: OxmlError) -> Self {
        Self::Xml(error)
    }
}

impl From<ShapePropertiesError> for ChartError {
    fn from(error: ShapePropertiesError) -> Self {
        Self::ShapeProperties(error)
    }
}

impl From<TextError> for ChartError {
    fn from(error: TextError) -> Self {
        Self::Text(error)
    }
}

pub type Result<T> = std::result::Result<T, ChartError>;

/// Backend-neutral plot geometry and the plot rectangle reserved inside a chart.
#[derive(Clone, Debug)]
pub struct ChartGeometry {
    pub plot_bounds: Rect,
    pub elements: Vec<PositionedElement>,
}

/// Converts one supported typed chart into chart-local backend-neutral paths.
pub fn render_geometry(chart: &CT_Chart, bounds: Rect) -> Result<ChartGeometry> {
    let plot_bounds = geometry_plot_bounds(bounds)?;
    let plots = chart.plot_area.plots()?;
    let plot = plots
        .first()
        .ok_or_else(|| ChartError::MissingElement("c:plot".to_owned()))?;
    plot.validate()?;
    let children = render_plot_geometry(plot, plot_bounds, chart.disp_blanks_as)?;
    if children.is_empty() {
        return Err(ChartError::InvalidValue {
            element: "c:plotArea".to_owned(),
            value: "plot has no renderable cached data".to_owned(),
        });
    }
    validate_geometry_coordinates(&children)?;
    Ok(ChartGeometry {
        plot_bounds,
        elements: vec![PositionedElement::Group(GroupElement {
            transform: Transform::IDENTITY,
            clip: None,
            opacity: 1.0,
            effects: Vec::new(),
            children,
        })],
    })
}

fn geometry_plot_bounds(bounds: Rect) -> Result<Rect> {
    const LEFT: f64 = 36.0;
    const RIGHT: f64 = 12.0;
    const TOP: f64 = 12.0;
    const BOTTOM: f64 = 28.0;
    if ![bounds.x, bounds.y, bounds.width, bounds.height]
        .into_iter()
        .all(f64::is_finite)
        || bounds.width <= 0.0
        || bounds.height <= 0.0
        || !(bounds.x + bounds.width).is_finite()
        || !(bounds.y + bounds.height).is_finite()
    {
        return Err(ChartError::InvalidValue {
            element: "chart geometry bounds".to_owned(),
            value: format!(
                "x={}, y={}, width={}, height={}",
                bounds.x, bounds.y, bounds.width, bounds.height
            ),
        });
    }
    let plot_bounds = Rect {
        x: bounds.x + LEFT,
        y: bounds.y + TOP,
        width: bounds.width - LEFT - RIGHT,
        height: bounds.height - TOP - BOTTOM,
    };
    if ![
        plot_bounds.x,
        plot_bounds.y,
        plot_bounds.width,
        plot_bounds.height,
        plot_bounds.x + plot_bounds.width,
        plot_bounds.y + plot_bounds.height,
    ]
    .into_iter()
    .all(f64::is_finite)
        || plot_bounds.width <= 0.0
        || plot_bounds.height <= 0.0
    {
        return Err(ChartError::InvalidValue {
            element: "chart geometry bounds".to_owned(),
            value: "chart is too small for the fixed plot margins".to_owned(),
        });
    }
    Ok(plot_bounds)
}

fn validate_geometry_coordinates(elements: &[PositionedElement]) -> Result<()> {
    fn finite_point(point: Point) -> bool {
        point.x.is_finite() && point.y.is_finite()
    }

    for element in elements {
        match element {
            PositionedElement::Path(path) => {
                for command in &path.path.commands {
                    let finite = match command {
                        PathCommand::MoveTo(point) | PathCommand::LineTo(point) => {
                            finite_point(*point)
                        }
                        PathCommand::CurveTo { c1, c2, to } => {
                            finite_point(*c1) && finite_point(*c2) && finite_point(*to)
                        }
                        PathCommand::Close => true,
                    };
                    if !finite {
                        return Err(ChartError::InvalidValue {
                            element: "chart geometry coordinate".to_owned(),
                            value: "generated path contains a nonfinite point".to_owned(),
                        });
                    }
                }
            }
            PositionedElement::Group(group) => validate_geometry_coordinates(&group.children)?,
            _ => {}
        }
    }
    Ok(())
}

fn render_plot_geometry(
    plot: &Plot,
    bounds: Rect,
    blanks: DispBlanksAs,
) -> Result<Vec<PositionedElement>> {
    match plot {
        Plot::Bar {
            direction,
            grouping,
            gap_width,
            overlap,
            series,
            ..
        } => render_bar_geometry(*direction, *grouping, *gap_width, *overlap, series, bounds),
        Plot::Line {
            grouping,
            marker,
            series,
            ..
        } => render_line_geometry(*grouping, *marker, series, bounds, blanks),
        Plot::Pie {
            first_slice_angle,
            series,
            ..
        } => render_pie_geometry(*first_slice_angle, None, series, bounds),
        Plot::Doughnut {
            first_slice_angle,
            hole_size,
            series,
            ..
        } => render_pie_geometry(*first_slice_angle, Some(*hole_size), series, bounds),
        Plot::Area {
            grouping, series, ..
        } => render_area_geometry(*grouping, series, bounds, blanks),
        Plot::Scatter { style, series, .. } => {
            render_scatter_geometry(*style, series, bounds, blanks)
        }
        Plot::Radar { style, series, .. } => render_radar_geometry(*style, series, bounds),
    }
}

fn render_bar_geometry(
    direction: BarDirection,
    grouping: BarGrouping,
    gap_width: u16,
    overlap: i8,
    series: &[Series],
    bounds: Rect,
) -> Result<Vec<PositionedElement>> {
    validate_series_values(series)?;
    let logical_series = logical_series_values(series)?;
    let category_count = series_category_count(series)?;
    let stacked = matches!(grouping, BarGrouping::Stacked | BarGrouping::PercentStacked);
    let percent = grouping == BarGrouping::PercentStacked;
    let layers = stacked_series_bounds(&logical_series, stacked, percent)?;
    let domain = domain_from_layers(&layers)?;
    let category_extent = if direction == BarDirection::Column {
        bounds.width
    } else {
        bounds.height
    };
    let slot = category_extent / category_count as f64;
    let cluster_extent = slot * 100.0 / (100.0 + f64::from(gap_width));
    let bars_per_cluster = if stacked { 1 } else { series.len() };
    let overlap_fraction = if stacked {
        0.0
    } else {
        f64::from(overlap) / 100.0
    };
    let divisor =
        bars_per_cluster as f64 - (bars_per_cluster.saturating_sub(1)) as f64 * overlap_fraction;
    let bar_extent = cluster_extent / divisor;
    let advance = bar_extent * (1.0 - overlap_fraction);
    let mut elements = Vec::new();
    for (series_index, layer) in layers.iter().enumerate() {
        for &(category, start, end) in layer {
            let cluster_start = category as f64 / category_count as f64 * category_extent
                + (slot - cluster_extent) / 2.0;
            let category_start = if stacked {
                cluster_start
            } else {
                cluster_start + series_index as f64 * advance
            };
            let rect = if direction == BarDirection::Column {
                let y0 = map_y(start, domain, bounds)?;
                let y1 = map_y(end, domain, bounds)?;
                Rect {
                    x: bounds.x + category_start,
                    y: y0.min(y1),
                    width: bar_extent,
                    height: (y1 - y0).abs(),
                }
            } else {
                let x0 = map_x(start, domain, bounds)?;
                let x1 = map_x(end, domain, bounds)?;
                Rect {
                    x: x0.min(x1),
                    y: bounds.y + category_start,
                    width: (x1 - x0).abs(),
                    height: bar_extent,
                }
            };
            if rect.width > 0.0 && rect.height > 0.0 {
                elements.push(filled_path(Path::rect(rect), series_color(series_index)));
            }
        }
    }
    Ok(elements)
}

fn render_line_geometry(
    grouping: Grouping,
    marker: bool,
    series: &[Series],
    bounds: Rect,
    blanks: DispBlanksAs,
) -> Result<Vec<PositionedElement>> {
    validate_series_values(series)?;
    let logical_series = logical_series_values(series)?;
    let category_count = series_category_count(series)?;
    let calculated_series = if blanks == DispBlanksAs::Zero {
        zero_filled_series(&logical_series, category_count)
    } else {
        logical_series.clone()
    };
    let stacked = grouping != Grouping::Standard;
    let percent = grouping == Grouping::PercentStacked;
    let layers = stacked_series_bounds(&calculated_series, stacked, percent)?;
    let domain = domain_from_layers(&layers)?;
    let mut elements = Vec::new();
    for (series_index, layer) in layers.iter().enumerate() {
        let indexed: Vec<_> = layer
            .iter()
            .map(|(index, _, upper)| (*index, *upper))
            .collect();
        let indexes: Vec<_> = indexed.iter().map(|(index, _)| *index).collect();
        for (start, end) in contiguous_ranges(&indexes, blanks) {
            let points = category_points(&indexed[start..end], category_count, domain, bounds)?;
            elements.push(stroked_path(
                polyline_path(&points, false),
                series_color(series_index),
            ));
        }
        if marker {
            let present: BTreeSet<_> = logical_series[series_index]
                .1
                .iter()
                .map(|(index, _)| *index)
                .collect();
            let marker_values: Vec<_> = indexed
                .iter()
                .copied()
                .filter(|(index, _)| present.contains(index))
                .collect();
            let marker_points = category_points(&marker_values, category_count, domain, bounds)?;
            push_markers(&mut elements, &marker_points, series_index);
        }
    }
    Ok(elements)
}

fn render_area_geometry(
    grouping: Grouping,
    series: &[Series],
    bounds: Rect,
    blanks: DispBlanksAs,
) -> Result<Vec<PositionedElement>> {
    validate_series_values(series)?;
    let logical_series = logical_series_values(series)?;
    let category_count = series_category_count(series)?;
    let calculated_series = if blanks == DispBlanksAs::Zero {
        zero_filled_series(&logical_series, category_count)
    } else {
        logical_series
    };
    let stacked = grouping != Grouping::Standard;
    let percent = grouping == Grouping::PercentStacked;
    let layers = stacked_series_bounds(&calculated_series, stacked, percent)?;
    let domain = domain_from_layers(&layers)?;
    let mut elements = Vec::new();
    for (series_index, layer) in layers.iter().enumerate() {
        let indexes: Vec<_> = layer.iter().map(|(index, _, _)| *index).collect();
        for (start, end) in contiguous_ranges(&indexes, blanks) {
            let segment = &layer[start..end];
            let top_values: Vec<_> = segment
                .iter()
                .map(|(index, _, upper)| (*index, *upper))
                .collect();
            let lower_values: Vec<_> = segment
                .iter()
                .map(|(index, lower, _)| (*index, *lower))
                .collect();
            let top = category_points(&top_values, category_count, domain, bounds)?;
            let mut bottom = category_points(&lower_values, category_count, domain, bounds)?;
            bottom.reverse();
            let mut commands = Vec::with_capacity(top.len() + bottom.len() + 2);
            commands.push(PathCommand::MoveTo(top[0]));
            commands.extend(top.iter().skip(1).copied().map(PathCommand::LineTo));
            commands.extend(bottom.into_iter().map(PathCommand::LineTo));
            commands.push(PathCommand::Close);
            elements.push(filled_and_stroked_path(
                Path {
                    commands,
                    fill_rule: FillRule::NonZero,
                },
                series_color(series_index),
            ));
        }
    }
    Ok(elements)
}

fn render_scatter_geometry(
    style: ScatterStyle,
    series: &[Series],
    bounds: Rect,
    blanks: DispBlanksAs,
) -> Result<Vec<PositionedElement>> {
    validate_series_values(series)?;
    let mut paired_series = Vec::with_capacity(series.len());
    let mut marker_series = Vec::with_capacity(series.len());
    let mut all_x = Vec::new();
    let mut all_y = Vec::new();
    for item in series {
        let Some(AxisData::Numeric(x_values)) = &item.categories else {
            return Err(ChartError::InvalidValue {
                element: "c:xVal".to_owned(),
                value: "scatter series requires numeric cached x values".to_owned(),
            });
        };
        validate_finite(&x_values.values, "c:xVal/c:numCache")?;
        let (x_count, x_points) = logical_numeric_values(x_values)?;
        let (y_count, y_points) = logical_numeric_values(&item.values)?;
        let x_by_index: BTreeMap<_, _> = x_points.into_iter().collect();
        let y_by_index: BTreeMap<_, _> = y_points.into_iter().collect();
        let markers: Vec<_> = y_by_index
            .iter()
            .filter_map(|(index, y)| x_by_index.get(index).map(|x| (index, *x, y)))
            .map(|(index, x, y)| (*index, x, *y))
            .collect();
        let paired = if blanks == DispBlanksAs::Zero {
            let count = x_count.max(y_count);
            let present = x_by_index
                .keys()
                .chain(y_by_index.keys())
                .copied()
                .collect();
            zero_control_indexes(count, &present)
                .into_iter()
                .map(|index| {
                    (
                        index,
                        x_by_index.get(&index).copied().unwrap_or(0.0),
                        y_by_index.get(&index).copied().unwrap_or(0.0),
                    )
                })
                .collect()
        } else {
            markers.clone()
        };
        all_x.extend(paired.iter().map(|(_, x, _)| *x));
        all_y.extend(paired.iter().map(|(_, _, y)| *y));
        paired_series.push(paired);
        marker_series.push(markers);
    }
    let x_domain = data_domain(&all_x, false)?;
    let y_domain = data_domain(&all_y, false)?;
    let draw_line = matches!(
        style,
        ScatterStyle::Line
            | ScatterStyle::LineMarker
            | ScatterStyle::Smooth
            | ScatterStyle::SmoothMarker
    );
    let draw_marker = matches!(
        style,
        ScatterStyle::Marker | ScatterStyle::LineMarker | ScatterStyle::SmoothMarker
    );
    let mut elements = Vec::new();
    for (series_index, paired) in paired_series.iter().enumerate() {
        if draw_line {
            let indexes: Vec<_> = paired.iter().map(|(index, _, _)| *index).collect();
            for (start, end) in contiguous_ranges(&indexes, blanks) {
                let points = scatter_points(&paired[start..end], x_domain, y_domain, bounds)?;
                elements.push(stroked_path(
                    polyline_path(&points, false),
                    series_color(series_index),
                ));
            }
        }
        if draw_marker {
            let marker_points =
                scatter_points(&marker_series[series_index], x_domain, y_domain, bounds)?;
            push_markers(&mut elements, &marker_points, series_index);
        }
    }
    Ok(elements)
}

fn scatter_points(
    values: &[(usize, f64, f64)],
    x_domain: Domain,
    y_domain: Domain,
    bounds: Rect,
) -> Result<Vec<Point>> {
    values
        .iter()
        .map(|(_, x, y)| {
            Ok(Point {
                x: map_x(*x, x_domain, bounds)?,
                y: map_y(*y, y_domain, bounds)?,
            })
        })
        .collect()
}

fn render_radar_geometry(
    style: RadarStyle,
    series: &[Series],
    bounds: Rect,
) -> Result<Vec<PositionedElement>> {
    validate_series_values(series)?;
    let logical_series = logical_series_values(series)?;
    let category_count = series_category_count(series)?;
    let maximum = logical_series
        .iter()
        .flat_map(|(_, points)| points.iter().map(|(_, value)| *value))
        .fold(0.0_f64, f64::max);
    if maximum <= 0.0 {
        return Err(ChartError::InvalidValue {
            element: "c:radarChart".to_owned(),
            value: "radar caches have no positive renderable values".to_owned(),
        });
    }
    let center = Point {
        x: bounds.x + bounds.width / 2.0,
        y: bounds.y + bounds.height / 2.0,
    };
    let radius = bounds.width.min(bounds.height) / 2.0;
    let mut elements = Vec::new();
    for (series_index, (_, logical)) in logical_series.iter().enumerate() {
        let points: Vec<_> = logical
            .iter()
            .map(|(category, value)| {
                radial_point(
                    center,
                    value.max(0.0) / maximum * radius,
                    *category,
                    category_count,
                )
            })
            .collect();
        let path = polyline_path(&points, true);
        if style == RadarStyle::Filled {
            elements.push(filled_and_stroked_path(path, series_color(series_index)));
        } else {
            elements.push(stroked_path(path, series_color(series_index)));
        }
        if style == RadarStyle::Marker {
            push_markers(&mut elements, &points, series_index);
        }
    }
    Ok(elements)
}

fn render_pie_geometry(
    first_slice_angle: u16,
    hole_size: Option<u8>,
    series: &[Series],
    bounds: Rect,
) -> Result<Vec<PositionedElement>> {
    validate_series_values(series)?;
    let center = Point {
        x: bounds.x + bounds.width / 2.0,
        y: bounds.y + bounds.height / 2.0,
    };
    let radius = bounds.width.min(bounds.height) / 2.0;
    let mut elements = Vec::new();
    for (series_index, item) in series.iter().enumerate() {
        let total = item
            .values
            .values
            .iter()
            .copied()
            .filter(|value| *value > 0.0)
            .try_fold(0.0, |total, value| {
                checked_geometry_sum(total, value, "pie value total")
            })?;
        if total <= 0.0 {
            continue;
        }
        let mut angle = f64::from(first_slice_angle).to_radians() - std::f64::consts::FRAC_PI_2;
        for value in &item.values.values {
            if *value <= 0.0 {
                continue;
            }
            let next = angle + value / total * std::f64::consts::TAU;
            let path = if let Some(hole_size) = hole_size {
                doughnut_wedge(
                    center,
                    radius,
                    radius * f64::from(hole_size) / 100.0,
                    angle,
                    next,
                )
            } else {
                pie_wedge(center, radius, angle, next)
            };
            elements.push(filled_path(path, series_color(series_index)));
            angle = next;
        }
    }
    Ok(elements)
}

fn validate_series_values(series: &[Series]) -> Result<()> {
    if series.is_empty() {
        return Err(ChartError::MissingElement("c:ser".to_owned()));
    }
    for item in series {
        validate_finite(&item.values.values, "c:val/c:numCache")?;
    }
    Ok(())
}

fn validate_finite(values: &[f64], element: &str) -> Result<()> {
    if let Some(value) = values.iter().find(|value| !value.is_finite()) {
        return Err(ChartError::InvalidValue {
            element: element.to_owned(),
            value: value.to_string(),
        });
    }
    Ok(())
}

type IndexedValues = Vec<(usize, f64)>;
type LogicalSeries = (usize, IndexedValues);
type GeometryLayer = Vec<(usize, f64, f64)>;

fn zero_control_indexes(count: usize, present: &BTreeSet<usize>) -> BTreeSet<usize> {
    let mut controls = present.clone();
    if count == 0 {
        return controls;
    }
    controls.insert(0);
    controls.insert(count - 1);
    for index in present {
        if *index > 0 {
            controls.insert(*index - 1);
        }
        if index + 1 < count {
            controls.insert(index + 1);
        }
    }
    controls
}

fn zero_filled_series(series: &[LogicalSeries], count: usize) -> Vec<LogicalSeries> {
    let present = series
        .iter()
        .flat_map(|(_, points)| points.iter().map(|(index, _)| *index))
        .collect();
    let controls = zero_control_indexes(count, &present);
    series
        .iter()
        .map(|(declared, points)| {
            let values: BTreeMap<_, _> = points.iter().copied().collect();
            let filled = controls
                .iter()
                .map(|index| (*index, values.get(index).copied().unwrap_or(0.0)))
                .collect();
            (*declared, filled)
        })
        .collect()
}

fn contiguous_ranges(indexes: &[usize], blanks: DispBlanksAs) -> Vec<(usize, usize)> {
    if indexes.is_empty() {
        return Vec::new();
    }
    if blanks != DispBlanksAs::Gap {
        return vec![(0, indexes.len())];
    }
    let mut ranges = Vec::new();
    let mut start = 0usize;
    for position in 1..indexes.len() {
        if indexes[position - 1].checked_add(1) != Some(indexes[position]) {
            ranges.push((start, position));
            start = position;
        }
    }
    ranges.push((start, indexes.len()));
    ranges
}

fn logical_numeric_values(data: &NumericData) -> Result<LogicalSeries> {
    validate_finite(&data.values, "c:numCache")?;
    let (declared, indexes) = cache_layout(&data.markup, data.values.len())?;
    let declared = usize::try_from(declared).map_err(|_| ChartError::InvalidValue {
        element: "c:ptCount".to_owned(),
        value: declared.to_string(),
    })?;
    let points = indexes
        .into_iter()
        .zip(data.values.iter().copied())
        .map(|(index, value)| {
            usize::try_from(index)
                .map(|index| (index, value))
                .map_err(|_| ChartError::InvalidValue {
                    element: "c:pt/@idx".to_owned(),
                    value: index.to_string(),
                })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok((declared, points))
}

fn logical_series_values(series: &[Series]) -> Result<Vec<LogicalSeries>> {
    series
        .iter()
        .map(|item| logical_numeric_values(&item.values))
        .collect()
}

fn reference_logical_count(markup: &ReferenceMarkup, value_count: usize) -> Result<usize> {
    let (declared, _) = cache_layout(markup, value_count)?;
    usize::try_from(declared).map_err(|_| ChartError::InvalidValue {
        element: "c:ptCount".to_owned(),
        value: declared.to_string(),
    })
}

fn series_category_count(series: &[Series]) -> Result<usize> {
    let mut count = 0usize;
    for item in series {
        count = count.max(reference_logical_count(
            &item.values.markup,
            item.values.values.len(),
        )?);
        if let Some(categories) = &item.categories {
            let category_count = match categories {
                AxisData::String(data) => reference_logical_count(&data.markup, data.values.len())?,
                AxisData::Numeric(data) => {
                    reference_logical_count(&data.markup, data.values.len())?
                }
            };
            count = count.max(category_count);
        }
    }
    if count == 0 {
        return Err(ChartError::InvalidValue {
            element: "c:ser".to_owned(),
            value: "cached series contain no points".to_owned(),
        });
    }
    Ok(count)
}

fn stacked_series_bounds(
    series: &[LogicalSeries],
    stacked: bool,
    percent: bool,
) -> Result<Vec<GeometryLayer>> {
    let mut positive_totals = BTreeMap::<usize, f64>::new();
    let mut negative_totals = BTreeMap::<usize, f64>::new();
    if percent {
        for (_, points) in series {
            for &(index, value) in points {
                let totals = if value >= 0.0 {
                    &mut positive_totals
                } else {
                    &mut negative_totals
                };
                let total = totals.entry(index).or_default();
                *total = checked_geometry_sum(*total, value.abs(), "stacked percentage total")?;
            }
        }
    }
    let mut positive = BTreeMap::<usize, f64>::new();
    let mut negative = BTreeMap::<usize, f64>::new();
    let mut layers = Vec::with_capacity(series.len());
    for (_, points) in series {
        let mut layer = Vec::with_capacity(points.len());
        for &(index, original) in points {
            let total = if original >= 0.0 {
                positive_totals.get(&index).copied().unwrap_or(0.0)
            } else {
                negative_totals.get(&index).copied().unwrap_or(0.0)
            };
            let value = if percent && total != 0.0 {
                original / total
            } else {
                original
            };
            if stacked {
                let accumulators = if value >= 0.0 {
                    &mut positive
                } else {
                    &mut negative
                };
                let accumulator = accumulators.entry(index).or_default();
                let lower = *accumulator;
                *accumulator = checked_geometry_sum(*accumulator, value, "stacked value total")?;
                layer.push((index, lower, *accumulator));
            } else {
                layer.push((index, 0.0, value));
            }
        }
        layers.push(layer);
    }
    Ok(layers)
}

fn checked_geometry_sum(current: f64, value: f64, context: &str) -> Result<f64> {
    let sum = current + value;
    if sum.is_finite() {
        Ok(sum)
    } else {
        Err(ChartError::InvalidValue {
            element: "chart geometry aggregate".to_owned(),
            value: format!("{context} is nonfinite"),
        })
    }
}

#[derive(Clone, Copy)]
struct Domain {
    min: f64,
    max: f64,
}

fn domain_from_layers(layers: &[GeometryLayer]) -> Result<Domain> {
    let values = layers
        .iter()
        .flatten()
        .flat_map(|(_, lower, upper)| [*lower, *upper]);
    let mut min = 0.0_f64;
    let mut max = 0.0_f64;
    for value in values {
        if !value.is_finite() {
            return Err(ChartError::InvalidValue {
                element: "chart geometry domain".to_owned(),
                value: "aggregate produced a nonfinite value".to_owned(),
            });
        }
        min = min.min(value);
        max = max.max(value);
    }
    if min == max {
        max = 1.0;
    }
    Ok(Domain { min, max })
}

fn data_domain(values: &[f64], include_zero: bool) -> Result<Domain> {
    validate_finite(values, "chart geometry cache")?;
    let Some(first) = values.first().copied() else {
        return Err(ChartError::InvalidValue {
            element: "c:numCache".to_owned(),
            value: "cached series contain no points".to_owned(),
        });
    };
    let mut min = first;
    let mut max = first;
    for value in values.iter().copied() {
        min = min.min(value);
        max = max.max(value);
    }
    if include_zero {
        min = min.min(0.0);
        max = max.max(0.0);
    }
    if min == max {
        if min > 0.0 {
            min = 0.0;
        } else if max < 0.0 {
            max = 0.0;
        } else {
            max = 1.0;
        }
    }
    Ok(Domain { min, max })
}

fn normalized_value(value: f64, domain: Domain) -> Result<f64> {
    let scale = value
        .abs()
        .max(domain.min.abs())
        .max(domain.max.abs())
        .max(1.0);
    let value = value / scale;
    let min = domain.min / scale;
    let max = domain.max / scale;
    let range = max - min;
    let normalized = (value - min) / range;
    if !range.is_finite() || range <= 0.0 || !normalized.is_finite() {
        return Err(ChartError::InvalidValue {
            element: "chart geometry domain".to_owned(),
            value: format!("cannot map finite domain {} to {}", domain.min, domain.max),
        });
    }
    Ok(normalized)
}

fn map_x(value: f64, domain: Domain, bounds: Rect) -> Result<f64> {
    let coordinate = bounds.x + normalized_value(value, domain)? * bounds.width;
    if coordinate.is_finite() {
        Ok(coordinate)
    } else {
        Err(ChartError::InvalidValue {
            element: "chart geometry coordinate".to_owned(),
            value: "x coordinate is nonfinite".to_owned(),
        })
    }
}

fn map_y(value: f64, domain: Domain, bounds: Rect) -> Result<f64> {
    let coordinate = bounds.y + bounds.height - normalized_value(value, domain)? * bounds.height;
    if coordinate.is_finite() {
        Ok(coordinate)
    } else {
        Err(ChartError::InvalidValue {
            element: "chart geometry coordinate".to_owned(),
            value: "y coordinate is nonfinite".to_owned(),
        })
    }
}

fn category_points(
    values: &[(usize, f64)],
    count: usize,
    domain: Domain,
    bounds: Rect,
) -> Result<Vec<Point>> {
    values
        .iter()
        .map(|(index, value)| {
            let x = bounds.x + (*index as f64 + 0.5) / count as f64 * bounds.width;
            let y = map_y(*value, domain, bounds)?;
            if !x.is_finite() {
                return Err(ChartError::InvalidValue {
                    element: "chart geometry coordinate".to_owned(),
                    value: "category x coordinate is nonfinite".to_owned(),
                });
            }
            Ok(Point { x, y })
        })
        .collect()
}

fn polyline_path(points: &[Point], close: bool) -> Path {
    let mut commands = Vec::with_capacity(points.len() + usize::from(close));
    if let Some(first) = points.first() {
        commands.push(PathCommand::MoveTo(*first));
        commands.extend(points.iter().skip(1).copied().map(PathCommand::LineTo));
        if close {
            commands.push(PathCommand::Close);
        }
    }
    Path {
        commands,
        fill_rule: FillRule::NonZero,
    }
}

fn pie_wedge(center: Point, radius: f64, start: f64, end: f64) -> Path {
    let first = circle_point(center, radius, start);
    let mut commands = vec![PathCommand::MoveTo(center), PathCommand::LineTo(first)];
    append_arc(&mut commands, center, radius, start, end);
    commands.push(PathCommand::Close);
    Path {
        commands,
        fill_rule: FillRule::NonZero,
    }
}

fn doughnut_wedge(
    center: Point,
    outer_radius: f64,
    inner_radius: f64,
    start: f64,
    end: f64,
) -> Path {
    let outer_start = circle_point(center, outer_radius, start);
    let inner_end = circle_point(center, inner_radius, end);
    let mut commands = vec![PathCommand::MoveTo(outer_start)];
    append_arc(&mut commands, center, outer_radius, start, end);
    commands.push(PathCommand::LineTo(inner_end));
    append_arc(&mut commands, center, inner_radius, end, start);
    commands.push(PathCommand::Close);
    Path {
        commands,
        fill_rule: FillRule::NonZero,
    }
}

fn append_arc(commands: &mut Vec<PathCommand>, center: Point, radius: f64, start: f64, end: f64) {
    let segments = ((end - start).abs() / std::f64::consts::FRAC_PI_2)
        .ceil()
        .max(1.0) as usize;
    let sweep = (end - start) / segments as f64;
    for segment in 0..segments {
        let angle = start + segment as f64 * sweep;
        let next = angle + sweep;
        let factor = 4.0 / 3.0 * (sweep / 4.0).tan();
        let from = circle_point(center, radius, angle);
        let to = circle_point(center, radius, next);
        commands.push(PathCommand::CurveTo {
            c1: Point {
                x: from.x - factor * radius * angle.sin(),
                y: from.y + factor * radius * angle.cos(),
            },
            c2: Point {
                x: to.x + factor * radius * next.sin(),
                y: to.y - factor * radius * next.cos(),
            },
            to,
        });
    }
}

fn circle_point(center: Point, radius: f64, angle: f64) -> Point {
    Point {
        x: center.x + radius * angle.cos(),
        y: center.y + radius * angle.sin(),
    }
}

fn radial_point(center: Point, radius: f64, index: usize, count: usize) -> Point {
    circle_point(
        center,
        radius,
        index as f64 * std::f64::consts::TAU / count as f64 - std::f64::consts::FRAC_PI_2,
    )
}

fn marker_path(center: Point) -> Path {
    const RADIUS: f64 = 2.5;
    const KAPPA: f64 = 0.552_284_749_830_793_6;
    let control = RADIUS * KAPPA;
    Path {
        commands: vec![
            PathCommand::MoveTo(Point {
                x: center.x + RADIUS,
                y: center.y,
            }),
            PathCommand::CurveTo {
                c1: Point {
                    x: center.x + RADIUS,
                    y: center.y + control,
                },
                c2: Point {
                    x: center.x + control,
                    y: center.y + RADIUS,
                },
                to: Point {
                    x: center.x,
                    y: center.y + RADIUS,
                },
            },
            PathCommand::CurveTo {
                c1: Point {
                    x: center.x - control,
                    y: center.y + RADIUS,
                },
                c2: Point {
                    x: center.x - RADIUS,
                    y: center.y + control,
                },
                to: Point {
                    x: center.x - RADIUS,
                    y: center.y,
                },
            },
            PathCommand::CurveTo {
                c1: Point {
                    x: center.x - RADIUS,
                    y: center.y - control,
                },
                c2: Point {
                    x: center.x - control,
                    y: center.y - RADIUS,
                },
                to: Point {
                    x: center.x,
                    y: center.y - RADIUS,
                },
            },
            PathCommand::CurveTo {
                c1: Point {
                    x: center.x + control,
                    y: center.y - RADIUS,
                },
                c2: Point {
                    x: center.x + RADIUS,
                    y: center.y - control,
                },
                to: Point {
                    x: center.x + RADIUS,
                    y: center.y,
                },
            },
            PathCommand::Close,
        ],
        fill_rule: FillRule::NonZero,
    }
}

fn push_markers(elements: &mut Vec<PositionedElement>, points: &[Point], series_index: usize) {
    for point in points {
        elements.push(filled_path(marker_path(*point), series_color(series_index)));
    }
}

fn filled_path(path: Path, color: Color) -> PositionedElement {
    PositionedElement::Path(PathElement {
        path,
        fill: Some(Paint::Solid(color)),
        stroke: None,
    })
}

fn stroked_path(path: Path, color: Color) -> PositionedElement {
    PositionedElement::Path(PathElement {
        path,
        fill: None,
        stroke: Some(Stroke::new(Paint::Solid(color), 1.5)),
    })
}

fn filled_and_stroked_path(path: Path, color: Color) -> PositionedElement {
    PositionedElement::Path(PathElement {
        path,
        fill: Some(Paint::Solid(Color { a: 0.55, ..color })),
        stroke: Some(Stroke::new(Paint::Solid(color), 1.0)),
    })
}

fn series_color(index: usize) -> Color {
    const PALETTE: [Color; 6] = [
        Color {
            r: 0.278,
            g: 0.478,
            b: 0.718,
            a: 1.0,
        },
        Color {
            r: 0.929,
            g: 0.490,
            b: 0.192,
            a: 1.0,
        },
        Color {
            r: 0.651,
            g: 0.651,
            b: 0.651,
            a: 1.0,
        },
        Color {
            r: 1.0,
            g: 0.753,
            b: 0.0,
            a: 1.0,
        },
        Color {
            r: 0.369,
            g: 0.608,
            b: 0.710,
            a: 1.0,
        },
        Color {
            r: 0.439,
            g: 0.678,
            b: 0.278,
            a: 1.0,
        },
    ];
    PALETTE[index % PALETTE.len()]
}

type NamespaceBindings = Vec<(Vec<u8>, String)>;
type XmlAttributes = Vec<(String, String)>;
type RootAttributes = (XmlAttributes, XmlAttributes);

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ScalarMarkup {
    raw_attributes: XmlAttributes,
    raw_content: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct TextMarkup {
    raw_attributes: XmlAttributes,
    raw_children: OrderedRawChildren,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct PointMarkup {
    raw_attributes: XmlAttributes,
    raw_children: OrderedRawChildren,
    value: TextMarkup,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ReferenceMarkup {
    raw_attributes: XmlAttributes,
    raw_children: OrderedRawChildren,
    formula: TextMarkup,
    cache_attributes: XmlAttributes,
    cache_children: OrderedRawChildren,
    format_code: Option<TextMarkup>,
    point_count: ScalarMarkup,
    declared_point_count: Option<u32>,
    point_indexes: Vec<u32>,
    points: Vec<PointMarkup>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct WrapperMarkup {
    raw_attributes: XmlAttributes,
    raw_children: OrderedRawChildren,
}

/// A formula-backed string cache whose count and point indexes are derived.
#[derive(Clone, Debug, PartialEq)]
pub struct StringRef {
    pub formula: String,
    pub values: Vec<String>,
    markup: ReferenceMarkup,
}

impl StringRef {
    pub fn new(formula: String, values: Vec<String>) -> Result<Self> {
        let reference = Self {
            formula,
            values,
            markup: ReferenceMarkup::default(),
        };
        reference.validate()?;
        Ok(reference)
    }

    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        Self::from_xml_with_namespaces(xml, &chart_namespace_defaults())
    }

    fn from_xml_with_namespaces(xml: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let parsed = parse_reference(xml, b"strRef", b"strCache", false, inherited)?;
        let values = parsed.values;
        let reference = Self {
            formula: parsed.formula,
            values,
            markup: parsed.markup,
        };
        reference.validate()?;
        Ok(reference)
    }

    pub fn to_xml(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut writer = Writer::new(Vec::new());
        self.write_xml(&mut writer, true)?;
        Ok(writer.into_inner())
    }

    fn validate(&self) -> Result<()> {
        validate_formula(&self.formula, "c:strRef/c:f")?;
        for value in &self.values {
            validate_xml_text(value, "c:strCache/c:pt/c:v")?;
        }
        validate_point_count(self.values.len(), "c:strCache")
    }

    fn write_xml(&self, writer: &mut Writer<Vec<u8>>, standalone: bool) -> Result<()> {
        self.validate()?;
        let mut start = BytesStart::new("c:strRef");
        if standalone {
            start.push_attribute(("xmlns:c", C_NS));
        }
        push_attributes(&mut start, &self.markup.raw_attributes);
        writer
            .write_event(Event::Start(start))
            .map_err(OxmlError::from)?;
        emit_raw(writer, self.markup.raw_children.at(0))?;
        write_text(writer, "c:f", &self.formula, &self.markup.formula)?;
        emit_raw(writer, self.markup.raw_children.at(1))?;
        write_string_cache(writer, &self.values, &self.markup)?;
        emit_raw(writer, self.markup.raw_children.at(2))?;
        writer
            .write_event(Event::End(BytesEnd::new("c:strRef")))
            .map_err(OxmlError::from)?;
        Ok(())
    }
}

/// A formula-backed numeric cache whose metadata and points are derived.
#[derive(Clone, Debug, PartialEq)]
pub struct NumericData {
    pub formula: String,
    pub format_code: String,
    pub values: Vec<f64>,
    markup: ReferenceMarkup,
}

impl NumericData {
    pub fn new(formula: String, format_code: String, values: Vec<f64>) -> Result<Self> {
        validate_formula(&formula, "c:numRef/c:f")?;
        validate_format_code(&format_code)?;
        validate_numeric_values(&values)?;
        validate_point_count(values.len(), "c:numCache")?;
        Ok(Self {
            formula,
            format_code,
            values,
            markup: ReferenceMarkup::default(),
        })
    }

    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        Self::from_xml_with_namespaces(xml, &chart_namespace_defaults())
    }

    fn from_xml_with_namespaces(xml: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let parsed = parse_reference(xml, b"numRef", b"numCache", true, inherited)?;
        let format_code = parsed
            .format_code
            .ok_or_else(|| ChartError::MissingElement("c:formatCode".to_owned()))?;
        let mut values = Vec::with_capacity(parsed.values.len());
        for value in parsed.values {
            let number = value.parse::<f64>().map_err(|_| ChartError::InvalidValue {
                element: "c:v".to_owned(),
                value: value.clone(),
            })?;
            if !number.is_finite() {
                return Err(ChartError::InvalidValue {
                    element: "c:v".to_owned(),
                    value,
                });
            }
            values.push(number);
        }
        let data = Self {
            formula: parsed.formula,
            format_code,
            values,
            markup: parsed.markup,
        };
        data.validate()?;
        Ok(data)
    }

    pub fn to_xml(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut writer = Writer::new(Vec::new());
        self.write_xml(&mut writer, true)?;
        Ok(writer.into_inner())
    }

    fn validate(&self) -> Result<()> {
        validate_formula(&self.formula, "c:numRef/c:f")?;
        validate_format_code(&self.format_code)?;
        validate_numeric_values(&self.values)?;
        validate_point_count(self.values.len(), "c:numCache")
    }

    fn write_xml(&self, writer: &mut Writer<Vec<u8>>, standalone: bool) -> Result<()> {
        self.validate()?;
        let mut start = BytesStart::new("c:numRef");
        if standalone {
            start.push_attribute(("xmlns:c", C_NS));
        }
        push_attributes(&mut start, &self.markup.raw_attributes);
        writer
            .write_event(Event::Start(start))
            .map_err(OxmlError::from)?;
        emit_raw(writer, self.markup.raw_children.at(0))?;
        write_text(writer, "c:f", &self.formula, &self.markup.formula)?;
        emit_raw(writer, self.markup.raw_children.at(1))?;
        write_numeric_cache(writer, self, &self.markup)?;
        emit_raw(writer, self.markup.raw_children.at(2))?;
        writer
            .write_event(Event::End(BytesEnd::new("c:numRef")))
            .map_err(OxmlError::from)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AxisData {
    String(StringRef),
    Numeric(NumericData),
}

/// The common formula-backed payload of one ChartML series.
#[derive(Clone, Debug, PartialEq)]
pub struct Series {
    pub index: u32,
    pub order: u32,
    pub name: Option<StringRef>,
    pub categories: Option<AxisData>,
    pub values: NumericData,
    pub bubble_size: Option<NumericData>,
    pub sp_pr: Option<CT_ShapeProperties>,
    pub data_labels: Option<CT_DLbls>,
    index_markup: ScalarMarkup,
    order_markup: ScalarMarkup,
    name_markup: Option<WrapperMarkup>,
    categories_markup: Option<WrapperMarkup>,
    values_markup: WrapperMarkup,
    bubble_size_markup: Option<WrapperMarkup>,
    opaque_name: bool,
    opaque_categories: bool,
    opaque_bubble_size: bool,
    uses_scatter_wrappers: bool,
    parsed_from_xml: bool,
    raw_attributes: XmlAttributes,
    namespace_declarations: XmlAttributes,
    raw_children: OrderedRawChildren,
}

impl Series {
    pub fn new(index: u32, order: u32, values: NumericData) -> Self {
        Self {
            index,
            order,
            name: None,
            categories: None,
            values,
            bubble_size: None,
            sp_pr: None,
            data_labels: None,
            index_markup: ScalarMarkup::default(),
            order_markup: ScalarMarkup::default(),
            name_markup: None,
            categories_markup: None,
            values_markup: WrapperMarkup::default(),
            bubble_size_markup: None,
            opaque_name: false,
            opaque_categories: false,
            opaque_bubble_size: false,
            uses_scatter_wrappers: false,
            parsed_from_xml: false,
            raw_attributes: Vec::new(),
            namespace_declarations: Vec::new(),
            raw_children: OrderedRawChildren::default(),
        }
    }

    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        Self::from_xml_with_namespaces(xml, &chart_namespace_defaults())
    }

    fn from_xml_with_namespaces(xml: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) if matches_local_name(element.name().as_ref(), b"ser") => {
                    chart_root_prefix(&element)?;
                    if !element_is_in_namespace(&element, C_NS, inherited)? {
                        return Err(ChartError::UnexpectedElement(element_name(&element)));
                    }
                    return Self::from_element(&mut reader, &element, inherited);
                }
                Event::Empty(element) if matches_local_name(element.name().as_ref(), b"ser") => {
                    return Err(ChartError::MissingElement("c:idx".to_owned()));
                }
                Event::Start(element) | Event::Empty(element) => {
                    return Err(ChartError::UnexpectedElement(element_name(&element)));
                }
                Event::Eof => return Err(ChartError::MissingElement("c:ser".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_element(
        reader: &mut Reader<&[u8]>,
        start: &BytesStart<'_>,
        inherited: &NamespaceBindings,
    ) -> Result<Self> {
        reject_conflicting_prefix(start, b"a", A_NS)?;
        let namespaces = chart_bindings(inherited, start)?;
        require_fixed_namespace(&namespaces, b"c", C_NS, start)?;
        require_fixed_namespace(&namespaces, b"a", A_NS, start)?;
        require_fixed_namespace(&namespaces, b"r", R_NS, start)?;
        let (raw_attributes, _) =
            capture_fixed_root_attributes(start, &["xmlns:c", "xmlns:a", "xmlns:r"])?;
        let namespace_declarations = standalone_namespace_declarations(&namespaces)?;
        let mut state = SeriesParseState::default();
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) => {
                    let name = chart_child_local(&element, &namespaces)?;
                    let raw = capture_element(reader, &element)?;
                    state.parse_child(name.as_deref().unwrap_or_default(), raw, &namespaces)?;
                }
                Event::Empty(element) => {
                    let name = chart_child_local(&element, &namespaces)?;
                    let raw = capture_empty_element(&element)?;
                    state.parse_child(name.as_deref().unwrap_or_default(), raw, &namespaces)?;
                }
                event @ (Event::Text(_)
                | Event::CData(_)
                | Event::Comment(_)
                | Event::PI(_)
                | Event::GeneralRef(_)) => state.capture_event(capture_event(event)?),
                Event::End(element) if matches_local_name(element.name().as_ref(), b"ser") => {
                    break;
                }
                Event::Eof => return Err(missing_end("c:ser")),
                _ => {}
            }
            buffer.clear();
        }
        state.finish(raw_attributes, namespace_declarations)
    }

    pub fn to_xml(&self) -> Result<Vec<u8>> {
        self.to_xml_for_plot(self.uses_scatter_wrappers)
    }

    fn to_xml_for_plot(&self, scatter: bool) -> Result<Vec<u8>> {
        if self.opaque_name && self.name.is_some() {
            return Err(ChartError::DuplicateElement("c:tx".to_owned()));
        }
        if self.opaque_categories && self.categories.is_some() {
            return Err(ChartError::DuplicateElement("c:cat".to_owned()));
        }
        if self.opaque_bubble_size && self.bubble_size.is_some() {
            return Err(ChartError::DuplicateElement("c:bubbleSize".to_owned()));
        }
        self.values.validate()?;
        if let Some(name) = &self.name {
            name.validate()?;
        }
        if let Some(categories) = &self.categories {
            if scatter && !matches!(categories, AxisData::Numeric(_)) {
                return Err(ChartError::InvalidValue {
                    element: "c:xVal".to_owned(),
                    value: "scatter x values must use a numeric cache".to_owned(),
                });
            }
            match categories {
                AxisData::String(reference) => reference.validate()?,
                AxisData::Numeric(reference) => reference.validate()?,
            }
        }
        if scatter && self.categories.is_none() {
            return Err(ChartError::MissingElement("c:xVal".to_owned()));
        }
        if let Some(size) = &self.bubble_size {
            size.validate()?;
        }
        if let Some(labels) = &self.data_labels {
            labels.validate()?;
        }
        let mut writer = Writer::new(Vec::new());
        let mut start = BytesStart::new("c:ser");
        start.push_attribute(("xmlns:c", C_NS));
        start.push_attribute(("xmlns:a", A_NS));
        start.push_attribute(("xmlns:r", R_NS));
        push_attributes(&mut start, &self.namespace_declarations);
        push_attributes(&mut start, &self.raw_attributes);
        writer
            .write_event(Event::Start(start))
            .map_err(OxmlError::from)?;
        emit_raw(&mut writer, self.raw_children.at(0))?;
        write_scalar(
            &mut writer,
            "c:idx",
            &self.index.to_string(),
            Some(&self.index_markup),
        )?;
        emit_raw(&mut writer, self.raw_children.at(1))?;
        write_scalar(
            &mut writer,
            "c:order",
            &self.order.to_string(),
            Some(&self.order_markup),
        )?;
        emit_raw(&mut writer, self.raw_children.at(2))?;
        if let Some(name) = &self.name {
            write_wrapper_start(
                &mut writer,
                "c:tx",
                self.name_markup
                    .as_ref()
                    .unwrap_or(&WrapperMarkup::default()),
            )?;
            name.write_xml(&mut writer, false)?;
            write_wrapper_end(
                &mut writer,
                "c:tx",
                self.name_markup
                    .as_ref()
                    .unwrap_or(&WrapperMarkup::default()),
            )?;
        }
        emit_raw(&mut writer, self.raw_children.at(3))?;
        if let Some(properties) = &self.sp_pr {
            properties.write_xml_as(&mut writer, "c:spPr")?;
        }
        emit_raw(&mut writer, self.raw_children.at(4))?;
        if let Some(labels) = &self.data_labels {
            labels.write_xml(&mut writer, false)?;
        }
        emit_raw(&mut writer, self.raw_children.at(5))?;
        if let Some(categories) = &self.categories {
            let default_markup = WrapperMarkup::default();
            let markup = self.categories_markup.as_ref().unwrap_or(&default_markup);
            write_wrapper_start(
                &mut writer,
                if scatter { "c:xVal" } else { "c:cat" },
                markup,
            )?;
            match categories {
                AxisData::String(reference) => reference.write_xml(&mut writer, false)?,
                AxisData::Numeric(reference) => reference.write_xml(&mut writer, false)?,
            }
            write_wrapper_end(
                &mut writer,
                if scatter { "c:xVal" } else { "c:cat" },
                markup,
            )?;
        }
        emit_raw(&mut writer, self.raw_children.at(6))?;
        write_wrapper_start(
            &mut writer,
            if scatter { "c:yVal" } else { "c:val" },
            &self.values_markup,
        )?;
        self.values.write_xml(&mut writer, false)?;
        write_wrapper_end(
            &mut writer,
            if scatter { "c:yVal" } else { "c:val" },
            &self.values_markup,
        )?;
        emit_raw(&mut writer, self.raw_children.at(7))?;
        if let Some(size) = &self.bubble_size {
            let default_markup = WrapperMarkup::default();
            let markup = self.bubble_size_markup.as_ref().unwrap_or(&default_markup);
            write_wrapper_start(&mut writer, "c:bubbleSize", markup)?;
            size.write_xml(&mut writer, false)?;
            write_wrapper_end(&mut writer, "c:bubbleSize", markup)?;
        }
        emit_raw(&mut writer, self.raw_children.at(8))?;
        writer
            .write_event(Event::End(BytesEnd::new("c:ser")))
            .map_err(OxmlError::from)?;
        Ok(writer.into_inner())
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }
}

#[derive(Default)]
struct SeriesParseState {
    index: Option<(u32, ScalarMarkup)>,
    order: Option<(u32, ScalarMarkup)>,
    name: Option<(StringRef, WrapperMarkup)>,
    categories: Option<(AxisData, WrapperMarkup)>,
    values: Option<(NumericData, WrapperMarkup)>,
    bubble_size: Option<(NumericData, WrapperMarkup)>,
    data_labels: Option<CT_DLbls>,
    name_seen: bool,
    categories_seen: bool,
    bubble_size_seen: bool,
    opaque_name: bool,
    opaque_categories: bool,
    opaque_bubble_size: bool,
    saw_standard_wrapper: bool,
    saw_scatter_wrapper: bool,
    sp_pr: Option<CT_ShapeProperties>,
    raw_children: OrderedRawChildren,
    boundary: usize,
}

impl SeriesParseState {
    fn capture_event(&mut self, raw: Vec<u8>) {
        self.raw_children.push(self.boundary, raw);
    }

    fn parse_child(
        &mut self,
        name: &[u8],
        raw: Vec<u8>,
        namespaces: &NamespaceBindings,
    ) -> Result<()> {
        match name {
            b"idx" => {
                set_once(&mut self.index, parse_u32_scalar(&raw, "idx")?, "c:idx")?;
                self.boundary = self.boundary.max(1);
            }
            b"order" => {
                set_once(&mut self.order, parse_u32_scalar(&raw, "order")?, "c:order")?;
                self.boundary = self.boundary.max(2);
            }
            b"tx" => {
                mark_once(&mut self.name_seen, "c:tx")?;
                let parsed = parse_wrapper(&raw, b"tx", &[b"strRef"], namespaces)?;
                if let Some((_, reference)) = parsed.choice {
                    self.name = Some((
                        StringRef::from_xml_with_namespaces(&reference, &parsed.namespaces)?,
                        parsed.markup,
                    ));
                } else {
                    self.opaque_name = true;
                    self.raw_children.push(2, raw);
                }
                self.boundary = self.boundary.max(3);
            }
            b"spPr" => {
                reject_conflicting_prefix_in_xml(&raw, b"a", A_NS)?;
                let properties = CT_ShapeProperties::from_xml(&raw)?;
                let mut writer = Writer::new(Vec::new());
                properties.write_xml_as(&mut writer, "c:spPr")?;
                reject_rewritten_foreign_elements(&raw, &writer.into_inner(), namespaces, b"spPr")?;
                set_once(&mut self.sp_pr, properties, "c:spPr")?;
                self.boundary = self.boundary.max(4);
            }
            b"dLbls" => {
                set_once(
                    &mut self.data_labels,
                    CT_DLbls::from_xml_with_namespaces(&raw, namespaces)?,
                    "c:dLbls",
                )?;
                self.boundary = self.boundary.max(5);
            }
            b"cat" => {
                self.saw_standard_wrapper = true;
                mark_once(&mut self.categories_seen, "c:cat")?;
                let parsed = parse_wrapper(&raw, b"cat", &[b"strRef", b"numRef"], namespaces)?;
                if let Some((choice, reference)) = parsed.choice {
                    let categories = if choice == b"strRef" {
                        AxisData::String(StringRef::from_xml_with_namespaces(
                            &reference,
                            &parsed.namespaces,
                        )?)
                    } else {
                        AxisData::Numeric(NumericData::from_xml_with_namespaces(
                            &reference,
                            &parsed.namespaces,
                        )?)
                    };
                    self.categories = Some((categories, parsed.markup));
                } else {
                    self.opaque_categories = true;
                    self.raw_children.push(5, raw);
                }
                self.boundary = self.boundary.max(6);
            }
            b"xVal" => {
                self.saw_scatter_wrapper = true;
                mark_once(&mut self.categories_seen, "c:xVal")?;
                let parsed = parse_wrapper(&raw, b"xVal", &[b"numRef"], namespaces)?;
                let (_, reference) = parsed
                    .choice
                    .ok_or_else(|| ChartError::MissingElement("c:xVal/c:numRef".to_owned()))?;
                self.categories = Some((
                    AxisData::Numeric(NumericData::from_xml_with_namespaces(
                        &reference,
                        &parsed.namespaces,
                    )?),
                    parsed.markup,
                ));
                self.boundary = self.boundary.max(6);
            }
            b"val" | b"yVal" => {
                if name == b"yVal" {
                    self.saw_scatter_wrapper = true;
                } else {
                    self.saw_standard_wrapper = true;
                }
                let wrapper = if name == b"yVal" {
                    b"yVal".as_slice()
                } else {
                    b"val".as_slice()
                };
                let parsed = parse_wrapper(&raw, wrapper, &[b"numRef"], namespaces)?;
                let (_, reference) = parsed.choice.ok_or_else(|| {
                    ChartError::MissingElement(format!(
                        "c:{}/c:numRef",
                        String::from_utf8_lossy(wrapper)
                    ))
                })?;
                set_once(
                    &mut self.values,
                    (
                        NumericData::from_xml_with_namespaces(&reference, &parsed.namespaces)?,
                        parsed.markup,
                    ),
                    "c:val or c:yVal",
                )?;
                self.boundary = self.boundary.max(7);
            }
            b"bubbleSize" => {
                mark_once(&mut self.bubble_size_seen, "c:bubbleSize")?;
                let parsed = parse_wrapper(&raw, b"bubbleSize", &[b"numRef"], namespaces)?;
                if let Some((_, reference)) = parsed.choice {
                    self.bubble_size = Some((
                        NumericData::from_xml_with_namespaces(&reference, &parsed.namespaces)?,
                        parsed.markup,
                    ));
                } else {
                    self.opaque_bubble_size = true;
                    self.raw_children.push(7, raw);
                }
                self.boundary = self.boundary.max(8);
            }
            _ => {
                let boundary = series_raw_boundary(name, self.boundary);
                self.raw_children.push(boundary, raw);
                self.boundary = self.boundary.max(boundary);
            }
        }
        Ok(())
    }

    fn finish(
        self,
        raw_attributes: XmlAttributes,
        namespace_declarations: XmlAttributes,
    ) -> Result<Series> {
        if self.saw_standard_wrapper && self.saw_scatter_wrapper {
            return Err(ChartError::InvalidValue {
                element: "c:ser".to_owned(),
                value: "category/value and x/y wrappers cannot be combined".to_owned(),
            });
        }
        let (index, index_markup) = self
            .index
            .ok_or_else(|| ChartError::MissingElement("c:idx".to_owned()))?;
        let (order, order_markup) = self
            .order
            .ok_or_else(|| ChartError::MissingElement("c:order".to_owned()))?;
        let (values, values_markup) = self
            .values
            .ok_or_else(|| ChartError::MissingElement("c:val".to_owned()))?;
        let (name, name_markup) = self
            .name
            .map(|(value, markup)| (Some(value), Some(markup)))
            .unwrap_or((None, None));
        let (categories, categories_markup) = self
            .categories
            .map(|(value, markup)| (Some(value), Some(markup)))
            .unwrap_or((None, None));
        let (bubble_size, bubble_size_markup) = self
            .bubble_size
            .map(|(value, markup)| (Some(value), Some(markup)))
            .unwrap_or((None, None));
        Ok(Series {
            index,
            order,
            name,
            categories,
            values,
            bubble_size,
            sp_pr: self.sp_pr,
            data_labels: self.data_labels,
            index_markup,
            order_markup,
            name_markup,
            categories_markup,
            values_markup,
            bubble_size_markup,
            opaque_name: self.opaque_name,
            opaque_categories: self.opaque_categories,
            opaque_bubble_size: self.opaque_bubble_size,
            uses_scatter_wrappers: self.saw_scatter_wrapper,
            parsed_from_xml: true,
            raw_attributes,
            namespace_declarations,
            raw_children: raw_children_in_schema_order(&self.raw_children, 8),
        })
    }
}

fn series_raw_boundary(name: &[u8], current: usize) -> usize {
    match name {
        b"marker" | b"invertIfNegative" | b"pictureOptions" | b"explosion" | b"dPt" => 4,
        b"trendline" | b"errBars" => 5,
        b"shape" | b"smooth" => 7,
        b"extLst" => 8,
        _ => current,
    }
}

struct ParsedReference {
    formula: String,
    format_code: Option<String>,
    values: Vec<String>,
    markup: ReferenceMarkup,
}

fn parse_reference(
    xml: &[u8],
    reference_local: &[u8],
    cache_local: &[u8],
    numeric: bool,
    inherited: &NamespaceBindings,
) -> Result<ParsedReference> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element)
                if matches_local_name(element.name().as_ref(), reference_local) =>
            {
                let namespaces = typed_rewrite_bindings(&element, inherited)?;
                let raw_attributes = capture_fixed_attributes(&element, &["xmlns:c"])?;
                let mut formula: Option<(String, TextMarkup)> = None;
                let mut cache: Option<(Option<String>, Vec<String>, CacheMarkup)> = None;
                let mut raw_children = OrderedRawChildren::default();
                let mut boundary = 0usize;
                let mut inner = Vec::new();
                loop {
                    match reader
                        .read_event_into(&mut inner)
                        .map_err(OxmlError::from)?
                    {
                        Event::Start(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_element(&mut reader, &child)?;
                            match name.as_deref().unwrap_or_default() {
                                b"f" => {
                                    set_once(
                                        &mut formula,
                                        parse_text_element(&raw, b"f", &namespaces)?,
                                        "c:f",
                                    )?;
                                    boundary = boundary.max(1);
                                }
                                name if name == cache_local => {
                                    set_once(
                                        &mut cache,
                                        parse_cache(&raw, cache_local, numeric, &namespaces)?,
                                        &format!("c:{}", String::from_utf8_lossy(cache_local)),
                                    )?;
                                    boundary = boundary.max(2);
                                }
                                _ => raw_children.push(boundary, raw),
                            }
                        }
                        Event::Empty(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_empty_element(&child)?;
                            match name.as_deref().unwrap_or_default() {
                                b"f" => {
                                    set_once(
                                        &mut formula,
                                        parse_text_element(&raw, b"f", &namespaces)?,
                                        "c:f",
                                    )?;
                                    boundary = boundary.max(1);
                                }
                                name if name == cache_local => {
                                    set_once(
                                        &mut cache,
                                        parse_cache(&raw, cache_local, numeric, &namespaces)?,
                                        &format!("c:{}", String::from_utf8_lossy(cache_local)),
                                    )?;
                                    boundary = boundary.max(2);
                                }
                                _ => raw_children.push(boundary, raw),
                            }
                        }
                        event @ (Event::Text(_)
                        | Event::CData(_)
                        | Event::Comment(_)
                        | Event::PI(_)
                        | Event::GeneralRef(_)) => {
                            raw_children.push(boundary, capture_event(event)?);
                        }
                        Event::End(end)
                            if matches_local_name(end.name().as_ref(), reference_local) =>
                        {
                            let (formula, formula_markup) = formula
                                .ok_or_else(|| ChartError::MissingElement("c:f".to_owned()))?;
                            validate_formula(&formula, "c:f")?;
                            let (format_code, values, cache_markup) = cache.ok_or_else(|| {
                                ChartError::MissingElement(format!(
                                    "c:{}",
                                    String::from_utf8_lossy(cache_local)
                                ))
                            })?;
                            let markup = ReferenceMarkup {
                                raw_attributes,
                                raw_children: raw_children_in_schema_order(&raw_children, 2),
                                formula: formula_markup,
                                cache_attributes: cache_markup.raw_attributes,
                                cache_children: cache_markup.raw_children,
                                format_code: cache_markup.format_code,
                                point_count: cache_markup.point_count,
                                declared_point_count: cache_markup.declared_point_count,
                                point_indexes: cache_markup.point_indexes,
                                points: cache_markup.points,
                            };
                            return Ok(ParsedReference {
                                formula,
                                format_code,
                                values,
                                markup,
                            });
                        }
                        Event::Eof => {
                            return Err(missing_end(&format!(
                                "c:{}",
                                String::from_utf8_lossy(reference_local)
                            )));
                        }
                        _ => {}
                    }
                    inner.clear();
                }
            }
            Event::Empty(element)
                if matches_local_name(element.name().as_ref(), reference_local) =>
            {
                return Err(ChartError::MissingElement("c:f".to_owned()));
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => {
                return Err(ChartError::MissingElement(format!(
                    "c:{}",
                    String::from_utf8_lossy(reference_local)
                )));
            }
            _ => {}
        }
        buffer.clear();
    }
}

struct CacheMarkup {
    raw_attributes: XmlAttributes,
    raw_children: OrderedRawChildren,
    format_code: Option<TextMarkup>,
    point_count: ScalarMarkup,
    declared_point_count: Option<u32>,
    point_indexes: Vec<u32>,
    points: Vec<PointMarkup>,
}

fn parse_cache(
    xml: &[u8],
    cache_local: &[u8],
    numeric: bool,
    inherited: &NamespaceBindings,
) -> Result<(Option<String>, Vec<String>, CacheMarkup)> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) if matches_local_name(element.name().as_ref(), cache_local) => {
                let namespaces = typed_rewrite_bindings(&element, inherited)?;
                let raw_attributes = capture_attributes(&element)?;
                let mut state = CacheParseState::new(numeric, namespaces);
                let mut inner = Vec::new();
                loop {
                    match reader
                        .read_event_into(&mut inner)
                        .map_err(OxmlError::from)?
                    {
                        Event::Start(child) => {
                            let name = chart_child_local(&child, &state.namespaces)?;
                            let raw = capture_element(&mut reader, &child)?;
                            state.parse_child(name.as_deref().unwrap_or_default(), raw)?;
                        }
                        Event::Empty(child) => {
                            let name = chart_child_local(&child, &state.namespaces)?;
                            let raw = capture_empty_element(&child)?;
                            state.parse_child(name.as_deref().unwrap_or_default(), raw)?;
                        }
                        event @ (Event::Text(_)
                        | Event::CData(_)
                        | Event::Comment(_)
                        | Event::PI(_)
                        | Event::GeneralRef(_)) => {
                            state.capture_event(capture_event(event)?);
                        }
                        Event::End(end) if matches_local_name(end.name().as_ref(), cache_local) => {
                            return state.finish(raw_attributes);
                        }
                        Event::Eof => {
                            return Err(missing_end(&format!(
                                "c:{}",
                                String::from_utf8_lossy(cache_local)
                            )));
                        }
                        _ => {}
                    }
                    inner.clear();
                }
            }
            Event::Empty(element) if matches_local_name(element.name().as_ref(), cache_local) => {
                return Err(ChartError::MissingElement("c:ptCount".to_owned()));
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => {
                return Err(ChartError::MissingElement(format!(
                    "c:{}",
                    String::from_utf8_lossy(cache_local)
                )));
            }
            _ => {}
        }
        buffer.clear();
    }
}

struct CacheParseState {
    numeric: bool,
    namespaces: NamespaceBindings,
    point_base: usize,
    boundary: usize,
    format_code: Option<(String, TextMarkup)>,
    point_count: Option<(u32, ScalarMarkup)>,
    points: Vec<(u32, String, PointMarkup)>,
    raw_children: OrderedRawChildren,
}

impl CacheParseState {
    fn new(numeric: bool, namespaces: NamespaceBindings) -> Self {
        Self {
            numeric,
            namespaces,
            point_base: usize::from(numeric) + 1,
            boundary: 0,
            format_code: None,
            point_count: None,
            points: Vec::new(),
            raw_children: OrderedRawChildren::default(),
        }
    }

    fn capture_event(&mut self, raw: Vec<u8>) {
        self.raw_children.push(self.boundary, raw);
    }

    fn parse_child(&mut self, name: &[u8], raw: Vec<u8>) -> Result<()> {
        match name {
            b"formatCode" if self.numeric => {
                set_once(
                    &mut self.format_code,
                    parse_text_element(&raw, b"formatCode", &self.namespaces)?,
                    "c:formatCode",
                )?;
                self.boundary = self.boundary.max(1);
            }
            b"ptCount" => {
                set_once(
                    &mut self.point_count,
                    parse_u32_scalar(&raw, "ptCount")?,
                    "c:ptCount",
                )?;
                self.boundary = self.boundary.max(self.point_base);
            }
            b"pt" => {
                self.points.push(parse_point(&raw, &self.namespaces)?);
                self.boundary = self.boundary.max(self.point_base + self.points.len());
            }
            _ => self.raw_children.push(self.boundary, raw),
        }
        Ok(())
    }

    fn finish(
        self,
        raw_attributes: XmlAttributes,
    ) -> Result<(Option<String>, Vec<String>, CacheMarkup)> {
        let (declared, point_count_markup) = self
            .point_count
            .ok_or_else(|| ChartError::MissingElement("c:ptCount".to_owned()))?;
        let actual = u32::try_from(self.points.len()).map_err(|_| ChartError::InvalidValue {
            element: "c:ptCount".to_owned(),
            value: self.points.len().to_string(),
        })?;
        if declared < actual {
            return Err(invalid_attribute("ptCount", "val", declared.to_string()));
        }
        let mut values = Vec::with_capacity(self.points.len());
        let mut point_indexes = Vec::with_capacity(self.points.len());
        let mut point_markup = Vec::with_capacity(self.points.len());
        let mut previous = None;
        for (index, value, markup) in self.points {
            if index >= declared || previous.is_some_and(|last| index <= last) {
                return Err(invalid_attribute("pt", "idx", index.to_string()));
            }
            previous = Some(index);
            values.push(value);
            point_indexes.push(index);
            point_markup.push(markup);
        }
        let (format_code, format_markup) = self
            .format_code
            .map(|(value, markup)| (Some(value), Some(markup)))
            .unwrap_or((None, None));
        if self.numeric && format_code.is_none() {
            return Err(ChartError::MissingElement("c:formatCode".to_owned()));
        }
        Ok((
            format_code,
            values,
            CacheMarkup {
                raw_attributes,
                raw_children: raw_children_in_schema_order(
                    &self.raw_children,
                    self.point_base + point_markup.len(),
                ),
                format_code: format_markup,
                point_count: point_count_markup,
                declared_point_count: Some(declared),
                point_indexes,
                points: point_markup,
            },
        ))
    }
}

fn parse_point(xml: &[u8], inherited: &NamespaceBindings) -> Result<(u32, String, PointMarkup)> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) if matches_local_name(element.name().as_ref(), b"pt") => {
                let namespaces = typed_rewrite_bindings(&element, inherited)?;
                let index = required_u32_attribute(&element, "pt", b"idx")?;
                let raw_attributes = capture_attributes_except(&element, b"idx")?;
                let mut value: Option<(String, TextMarkup)> = None;
                let mut raw_children = OrderedRawChildren::default();
                let mut boundary = 0usize;
                let mut inner = Vec::new();
                loop {
                    match reader
                        .read_event_into(&mut inner)
                        .map_err(OxmlError::from)?
                    {
                        Event::Start(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_element(&mut reader, &child)?;
                            if name.as_deref() == Some(b"v") {
                                set_once(
                                    &mut value,
                                    parse_text_element(&raw, b"v", &namespaces)?,
                                    "c:v",
                                )?;
                                boundary = 1;
                            } else {
                                raw_children.push(boundary, raw);
                            }
                        }
                        Event::Empty(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_empty_element(&child)?;
                            if name.as_deref() == Some(b"v") {
                                set_once(
                                    &mut value,
                                    parse_text_element(&raw, b"v", &namespaces)?,
                                    "c:v",
                                )?;
                                boundary = 1;
                            } else {
                                raw_children.push(boundary, raw);
                            }
                        }
                        event @ (Event::Text(_)
                        | Event::CData(_)
                        | Event::Comment(_)
                        | Event::PI(_)
                        | Event::GeneralRef(_)) => {
                            raw_children.push(boundary, capture_event(event)?);
                        }
                        Event::End(end) if matches_local_name(end.name().as_ref(), b"pt") => {
                            let (value, value_markup) = value
                                .ok_or_else(|| ChartError::MissingElement("c:v".to_owned()))?;
                            return Ok((
                                index,
                                value,
                                PointMarkup {
                                    raw_attributes,
                                    raw_children: raw_children_in_schema_order(&raw_children, 1),
                                    value: value_markup,
                                },
                            ));
                        }
                        Event::Eof => return Err(missing_end("c:pt")),
                        _ => {}
                    }
                    inner.clear();
                }
            }
            Event::Empty(element) if matches_local_name(element.name().as_ref(), b"pt") => {
                return Err(ChartError::MissingElement("c:v".to_owned()));
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => return Err(ChartError::MissingElement("c:pt".to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

fn parse_text_element(
    xml: &[u8],
    local: &[u8],
    inherited: &NamespaceBindings,
) -> Result<(String, TextMarkup)> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) if matches_local_name(element.name().as_ref(), local) => {
                typed_rewrite_bindings(&element, inherited)?;
                let raw_attributes = capture_attributes(&element)?;
                let mut value = String::new();
                let mut raw_children = OrderedRawChildren::default();
                let mut seen_text = false;
                let mut inner = Vec::new();
                loop {
                    match reader
                        .read_event_into(&mut inner)
                        .map_err(OxmlError::from)?
                    {
                        Event::Text(text) => {
                            value.push_str(&decode_plain(&text));
                            seen_text = true;
                        }
                        Event::CData(text) => {
                            let decoded =
                                text.decode().map_err(|error| ChartError::InvalidValue {
                                    element: format!("c:{}", String::from_utf8_lossy(local)),
                                    value: error.to_string(),
                                })?;
                            value.push_str(&decoded);
                            seen_text = true;
                        }
                        Event::GeneralRef(reference) => {
                            value.push_str(&resolve_entity(&reference));
                            seen_text = true;
                        }
                        event @ (Event::Comment(_) | Event::PI(_)) => {
                            raw_children.push(usize::from(seen_text), capture_event(event)?);
                        }
                        Event::Start(child) | Event::Empty(child) => {
                            return Err(ChartError::UnexpectedElement(element_name(&child)));
                        }
                        Event::End(end) if matches_local_name(end.name().as_ref(), local) => {
                            return Ok((
                                value,
                                TextMarkup {
                                    raw_attributes,
                                    raw_children: raw_children_in_schema_order(&raw_children, 1),
                                },
                            ));
                        }
                        Event::Eof => {
                            return Err(missing_end(&format!(
                                "c:{}",
                                String::from_utf8_lossy(local)
                            )));
                        }
                        _ => {}
                    }
                    inner.clear();
                }
            }
            Event::Empty(element) if matches_local_name(element.name().as_ref(), local) => {
                typed_rewrite_bindings(&element, inherited)?;
                return Ok((
                    String::new(),
                    TextMarkup {
                        raw_attributes: capture_attributes(&element)?,
                        raw_children: OrderedRawChildren::default(),
                    },
                ));
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => {
                return Err(ChartError::MissingElement(format!(
                    "c:{}",
                    String::from_utf8_lossy(local)
                )));
            }
            _ => {}
        }
        buffer.clear();
    }
}

struct ParsedWrapper {
    choice: Option<(Vec<u8>, Vec<u8>)>,
    markup: WrapperMarkup,
    namespaces: NamespaceBindings,
}

fn parse_wrapper(
    xml: &[u8],
    local: &[u8],
    choices: &[&[u8]],
    inherited: &NamespaceBindings,
) -> Result<ParsedWrapper> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) if matches_local_name(element.name().as_ref(), local) => {
                let namespaces = typed_rewrite_bindings(&element, inherited)?;
                let raw_attributes = capture_attributes(&element)?;
                let mut choice = None;
                let mut raw_children = OrderedRawChildren::default();
                let mut boundary = 0usize;
                let mut inner = Vec::new();
                loop {
                    match reader
                        .read_event_into(&mut inner)
                        .map_err(OxmlError::from)?
                    {
                        Event::Start(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_element(&mut reader, &child)?;
                            if let Some(name) =
                                name.filter(|name| choices.contains(&name.as_slice()))
                            {
                                if choice.is_some() {
                                    return Err(ChartError::DuplicateElement(format!(
                                        "c:{} reference",
                                        String::from_utf8_lossy(local)
                                    )));
                                }
                                choice = Some((name, raw));
                                boundary = 1;
                            } else {
                                raw_children.push(boundary, raw);
                            }
                        }
                        Event::Empty(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_empty_element(&child)?;
                            if let Some(name) =
                                name.filter(|name| choices.contains(&name.as_slice()))
                            {
                                if choice.is_some() {
                                    return Err(ChartError::DuplicateElement(format!(
                                        "c:{} reference",
                                        String::from_utf8_lossy(local)
                                    )));
                                }
                                choice = Some((name, raw));
                                boundary = 1;
                            } else {
                                raw_children.push(boundary, raw);
                            }
                        }
                        event @ (Event::Text(_)
                        | Event::CData(_)
                        | Event::Comment(_)
                        | Event::PI(_)
                        | Event::GeneralRef(_)) => {
                            raw_children.push(boundary, capture_event(event)?);
                        }
                        Event::End(end) if matches_local_name(end.name().as_ref(), local) => {
                            return Ok(ParsedWrapper {
                                choice,
                                markup: WrapperMarkup {
                                    raw_attributes,
                                    raw_children: raw_children_in_schema_order(&raw_children, 1),
                                },
                                namespaces,
                            });
                        }
                        Event::Eof => {
                            return Err(missing_end(&format!(
                                "c:{}",
                                String::from_utf8_lossy(local)
                            )));
                        }
                        _ => {}
                    }
                    inner.clear();
                }
            }
            Event::Empty(element) if matches_local_name(element.name().as_ref(), local) => {
                let namespaces = chart_bindings(inherited, &element)?;
                return Ok(ParsedWrapper {
                    choice: None,
                    markup: WrapperMarkup {
                        raw_attributes: capture_attributes(&element)?,
                        raw_children: OrderedRawChildren::default(),
                    },
                    namespaces,
                });
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => {
                return Err(ChartError::MissingElement(format!(
                    "c:{}",
                    String::from_utf8_lossy(local)
                )));
            }
            _ => {}
        }
        buffer.clear();
    }
}

fn write_string_cache(
    writer: &mut Writer<Vec<u8>>,
    values: &[String],
    markup: &ReferenceMarkup,
) -> Result<()> {
    let (declared_count, indexes) = cache_layout(markup, values.len())?;
    let mut start = BytesStart::new("c:strCache");
    push_attributes(&mut start, &markup.cache_attributes);
    writer
        .write_event(Event::Start(start))
        .map_err(OxmlError::from)?;
    emit_raw(writer, markup.cache_children.at(0))?;
    write_scalar(
        writer,
        "c:ptCount",
        &declared_count.to_string(),
        Some(&markup.point_count),
    )?;
    emit_cache_point_raw(writer, markup, 1, values.len(), 0)?;
    for (position, value) in values.iter().enumerate() {
        let default_markup = PointMarkup::default();
        write_point(
            writer,
            indexes[position],
            value,
            markup.points.get(position).unwrap_or(&default_markup),
        )?;
        emit_cache_point_raw(writer, markup, 1, values.len(), position + 1)?;
    }
    writer
        .write_event(Event::End(BytesEnd::new("c:strCache")))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn write_numeric_cache(
    writer: &mut Writer<Vec<u8>>,
    data: &NumericData,
    markup: &ReferenceMarkup,
) -> Result<()> {
    let (declared_count, indexes) = cache_layout(markup, data.values.len())?;
    let mut start = BytesStart::new("c:numCache");
    push_attributes(&mut start, &markup.cache_attributes);
    writer
        .write_event(Event::Start(start))
        .map_err(OxmlError::from)?;
    emit_raw(writer, markup.cache_children.at(0))?;
    let default_format_markup = TextMarkup::default();
    write_text(
        writer,
        "c:formatCode",
        &data.format_code,
        markup
            .format_code
            .as_ref()
            .unwrap_or(&default_format_markup),
    )?;
    emit_raw(writer, markup.cache_children.at(1))?;
    write_scalar(
        writer,
        "c:ptCount",
        &declared_count.to_string(),
        Some(&markup.point_count),
    )?;
    emit_cache_point_raw(writer, markup, 2, data.values.len(), 0)?;
    for (position, value) in data.values.iter().enumerate() {
        let default_markup = PointMarkup::default();
        write_point(
            writer,
            indexes[position],
            &value.to_string(),
            markup.points.get(position).unwrap_or(&default_markup),
        )?;
        emit_cache_point_raw(writer, markup, 2, data.values.len(), position + 1)?;
    }
    writer
        .write_event(Event::End(BytesEnd::new("c:numCache")))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn emit_cache_point_raw(
    writer: &mut Writer<Vec<u8>>,
    markup: &ReferenceMarkup,
    point_base: usize,
    current_count: usize,
    completed_points: usize,
) -> Result<()> {
    let original_count = markup.points.len();
    if current_count == 0 {
        for boundary in point_base..=point_base + original_count {
            emit_raw(writer, markup.cache_children.at(boundary))?;
        }
    } else if completed_points == 0 {
        if original_count > 0 {
            emit_raw(writer, markup.cache_children.at(point_base))?;
        }
    } else if completed_points < current_count {
        if completed_points < original_count {
            emit_raw(
                writer,
                markup.cache_children.at(point_base + completed_points),
            )?;
        }
    } else {
        let first_tail = if original_count == 0 {
            point_base
        } else {
            point_base + current_count.min(original_count)
        };
        for boundary in first_tail..=point_base + original_count {
            emit_raw(writer, markup.cache_children.at(boundary))?;
        }
    }
    Ok(())
}

fn write_point(
    writer: &mut Writer<Vec<u8>>,
    index: u32,
    value: &str,
    markup: &PointMarkup,
) -> Result<()> {
    let index = index.to_string();
    let mut start = BytesStart::new("c:pt");
    start.push_attribute(("idx", index.as_str()));
    push_attributes(&mut start, &markup.raw_attributes);
    writer
        .write_event(Event::Start(start))
        .map_err(OxmlError::from)?;
    emit_raw(writer, markup.raw_children.at(0))?;
    write_text(writer, "c:v", value, &markup.value)?;
    emit_raw(writer, markup.raw_children.at(1))?;
    writer
        .write_event(Event::End(BytesEnd::new("c:pt")))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn cache_layout(markup: &ReferenceMarkup, value_count: usize) -> Result<(u32, Vec<u32>)> {
    if markup.point_indexes.len() == value_count
        && let Some(declared) = markup.declared_point_count
    {
        return Ok((declared, markup.point_indexes.clone()));
    }
    let declared = u32::try_from(value_count).map_err(|_| ChartError::InvalidValue {
        element: "c:ptCount".to_owned(),
        value: value_count.to_string(),
    })?;
    Ok((declared, (0..declared).collect()))
}

fn write_text(
    writer: &mut Writer<Vec<u8>>,
    tag: &str,
    value: &str,
    markup: &TextMarkup,
) -> Result<()> {
    let mut start = BytesStart::new(tag);
    push_attributes(&mut start, &markup.raw_attributes);
    writer
        .write_event(Event::Start(start))
        .map_err(OxmlError::from)?;
    emit_raw(writer, markup.raw_children.at(0))?;
    writer
        .write_event(Event::Text(BytesText::new(value)))
        .map_err(OxmlError::from)?;
    emit_raw(writer, markup.raw_children.at(1))?;
    writer
        .write_event(Event::End(BytesEnd::new(tag)))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn write_wrapper_start(
    writer: &mut Writer<Vec<u8>>,
    tag: &str,
    markup: &WrapperMarkup,
) -> Result<()> {
    let mut start = BytesStart::new(tag);
    push_attributes(&mut start, &markup.raw_attributes);
    writer
        .write_event(Event::Start(start))
        .map_err(OxmlError::from)?;
    emit_raw(writer, markup.raw_children.at(0))
}

fn write_wrapper_end(
    writer: &mut Writer<Vec<u8>>,
    tag: &str,
    markup: &WrapperMarkup,
) -> Result<()> {
    emit_raw(writer, markup.raw_children.at(1))?;
    writer
        .write_event(Event::End(BytesEnd::new(tag)))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn validate_formula(formula: &str, element: &str) -> Result<()> {
    if formula.trim().is_empty() {
        return Err(ChartError::InvalidValue {
            element: element.to_owned(),
            value: formula.to_owned(),
        });
    }
    validate_xml_text(formula, element)
}

fn validate_format_code(format_code: &str) -> Result<()> {
    if format_code.is_empty() {
        return Err(ChartError::InvalidValue {
            element: "c:formatCode".to_owned(),
            value: format_code.to_owned(),
        });
    }
    validate_xml_text(format_code, "c:formatCode")
}

fn validate_xml_text(value: &str, element: &str) -> Result<()> {
    if value.chars().all(|character| {
        matches!(
            character,
            '\u{9}' | '\u{A}' | '\u{D}' | '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}'
        )
    }) {
        return Ok(());
    }
    Err(ChartError::InvalidValue {
        element: element.to_owned(),
        value: value.to_owned(),
    })
}

fn validate_numeric_values(values: &[f64]) -> Result<()> {
    if let Some(value) = values.iter().find(|value| !value.is_finite()) {
        return Err(ChartError::InvalidValue {
            element: "c:v".to_owned(),
            value: value.to_string(),
        });
    }
    Ok(())
}

fn validate_point_count(count: usize, element: &str) -> Result<()> {
    u32::try_from(count)
        .map(|_| ())
        .map_err(|_| ChartError::InvalidValue {
            element: element.to_owned(),
            value: count.to_string(),
        })
}

fn parse_u32_scalar(xml: &[u8], local: &str) -> Result<(u32, ScalarMarkup)> {
    let (value, markup) = scalar_value(xml, local)?;
    let value = value.ok_or_else(|| invalid_attribute(local, "val", "<missing>".to_owned()))?;
    let parsed = value
        .parse::<u32>()
        .map_err(|_| invalid_attribute(local, "val", value))?;
    Ok((parsed, markup))
}

fn required_u32_attribute(element: &BytesStart<'_>, local: &str, attribute: &[u8]) -> Result<u32> {
    let value = attribute_value(element, attribute)?.ok_or_else(|| {
        invalid_attribute(
            local,
            &String::from_utf8_lossy(attribute),
            "<missing>".to_owned(),
        )
    })?;
    value
        .parse::<u32>()
        .map_err(|_| invalid_attribute(local, &String::from_utf8_lossy(attribute), value))
}

fn chart_root_prefix(element: &BytesStart<'_>) -> Result<Vec<u8>> {
    let name = element.name();
    let qualified = name.as_ref();
    let prefix = qualified
        .iter()
        .position(|byte| *byte == b':')
        .map(|position| qualified[..position].to_vec())
        .unwrap_or_default();
    let declaration = if prefix.is_empty() {
        b"xmlns".to_vec()
    } else {
        let mut declaration = b"xmlns:".to_vec();
        declaration.extend_from_slice(&prefix);
        declaration
    };
    if let Some(namespace) = attribute_value(element, &declaration)?
        && namespace != C_NS
    {
        return Err(invalid_attribute(
            &element_name(element),
            "namespace",
            namespace,
        ));
    }
    reject_conflicting_prefix(element, b"c", C_NS)?;
    Ok(prefix)
}

fn typed_rewrite_bindings(
    element: &BytesStart<'_>,
    inherited: &NamespaceBindings,
) -> Result<NamespaceBindings> {
    chart_root_prefix(element)?;
    if !element_is_in_namespace(element, C_NS, inherited)? {
        return Err(ChartError::UnexpectedElement(element_name(element)));
    }
    let bindings = chart_bindings(inherited, element)?;
    require_fixed_namespace(&bindings, b"c", C_NS, element)?;
    Ok(bindings)
}

fn chart_namespace_defaults() -> NamespaceBindings {
    vec![
        (b"c".to_vec(), C_NS.to_owned()),
        (b"a".to_vec(), A_NS.to_owned()),
        (b"r".to_vec(), R_NS.to_owned()),
    ]
}

fn chart_bindings(
    inherited: &NamespaceBindings,
    element: &BytesStart<'_>,
) -> Result<NamespaceBindings> {
    let mut bindings = chart_namespace_defaults();
    for (prefix, namespace) in inherited
        .iter()
        .cloned()
        .chain(namespace_bindings(element)?)
    {
        upsert_namespace_binding(&mut bindings, prefix, namespace);
    }
    Ok(bindings)
}

fn root_chart_bindings(
    xml: &[u8],
    local: &[u8],
    inherited: &NamespaceBindings,
) -> Result<NamespaceBindings> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) | Event::Empty(element)
                if matches_local_name(element.name().as_ref(), local) =>
            {
                if !element_is_in_namespace(&element, C_NS, inherited)? {
                    return Err(ChartError::UnexpectedElement(element_name(&element)));
                }
                return chart_bindings(inherited, &element);
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => {
                return Err(ChartError::MissingElement(format!(
                    "c:{}",
                    String::from_utf8_lossy(local)
                )));
            }
            _ => {}
        }
        buffer.clear();
    }
}

fn chart_child_local(
    element: &BytesStart<'_>,
    inherited: &NamespaceBindings,
) -> Result<Option<Vec<u8>>> {
    Ok(element_is_in_namespace(element, C_NS, inherited)?
        .then(|| local_name(element.name().as_ref()).to_vec()))
}

fn capture_fixed_root_attributes(
    start: &BytesStart<'_>,
    fixed: &[&str],
) -> Result<(XmlAttributes, XmlAttributes)> {
    let mut attributes = Vec::new();
    let mut namespaces = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        let name = std::str::from_utf8(attribute.key.as_ref())
            .map_err(OxmlError::from)?
            .to_owned();
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())
            .map_err(OxmlError::from)?
            .into_owned();
        if fixed.contains(&name.as_str()) {
            continue;
        }
        if name == "xmlns" || name.starts_with("xmlns:") {
            namespaces.push((name, value));
        } else {
            attributes.push((name, value));
        }
    }
    Ok((attributes, namespaces))
}

fn require_fixed_namespace(
    bindings: &NamespaceBindings,
    prefix: &[u8],
    expected: &str,
    element: &BytesStart<'_>,
) -> Result<()> {
    let actual = bindings
        .iter()
        .find(|(candidate, _)| candidate.as_slice() == prefix)
        .map(|(_, namespace)| namespace.as_str());
    if actual == Some(expected) {
        return Ok(());
    }
    Err(invalid_attribute(
        &element_name(element),
        &format!("xmlns:{}", String::from_utf8_lossy(prefix)),
        actual.unwrap_or("<missing>").to_owned(),
    ))
}

fn standalone_namespace_declarations(bindings: &NamespaceBindings) -> Result<XmlAttributes> {
    bindings
        .iter()
        .filter(|(prefix, _)| prefix != b"c" && prefix != b"a" && prefix != b"r")
        .map(|(prefix, namespace)| {
            let prefix = std::str::from_utf8(prefix).map_err(OxmlError::from)?;
            let name = if prefix.is_empty() {
                "xmlns".to_owned()
            } else {
                format!("xmlns:{prefix}")
            };
            Ok((name, namespace.clone()))
        })
        .collect()
}

fn capture_fixed_attributes(start: &BytesStart<'_>, fixed: &[&str]) -> Result<XmlAttributes> {
    let mut attributes = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        let name = std::str::from_utf8(attribute.key.as_ref())
            .map_err(OxmlError::from)?
            .to_owned();
        if fixed.contains(&name.as_str()) {
            continue;
        }
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())
            .map_err(OxmlError::from)?
            .into_owned();
        attributes.push((name, value));
    }
    Ok(attributes)
}

/// A producer-compatible ChartML axis identifier.
///
/// ECMA-376 names an unsigned value, but PowerPoint emits signed 32-bit
/// lexical values in real chart parts. The accepted domain covers both forms.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AxisId(i64);

impl AxisId {
    pub fn new(value: i64) -> Result<Self> {
        if (i64::from(i32::MIN)..=i64::from(u32::MAX)).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ChartError::InvalidValue {
                element: "c:axId".to_owned(),
                value: value.to_string(),
            })
        }
    }

    pub const fn value(self) -> i64 {
        self.0
    }
}

/// Whether bars extend along the category or value axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BarDirection {
    Bar,
    Column,
}

impl BarDirection {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "bar" => Some(Self::Bar),
            "col" => Some(Self::Column),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Bar => "bar",
            Self::Column => "col",
        }
    }
}

/// Grouping modes supported by a two-dimensional bar plot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BarGrouping {
    Clustered,
    PercentStacked,
    Stacked,
    Standard,
}

impl BarGrouping {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "clustered" => Some(Self::Clustered),
            "percentStacked" => Some(Self::PercentStacked),
            "stacked" => Some(Self::Stacked),
            "standard" => Some(Self::Standard),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Clustered => "clustered",
            Self::PercentStacked => "percentStacked",
            Self::Stacked => "stacked",
            Self::Standard => "standard",
        }
    }
}

/// Grouping modes shared by line and later area plots.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Grouping {
    PercentStacked,
    Stacked,
    Standard,
}

impl Grouping {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "percentStacked" => Some(Self::PercentStacked),
            "stacked" => Some(Self::Stacked),
            "standard" => Some(Self::Standard),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::PercentStacked => "percentStacked",
            Self::Stacked => "stacked",
            Self::Standard => "standard",
        }
    }
}

/// Display style for a two-dimensional scatter plot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScatterStyle {
    Line,
    LineMarker,
    Marker,
    None,
    Smooth,
    SmoothMarker,
}

impl ScatterStyle {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "line" => Some(Self::Line),
            "lineMarker" => Some(Self::LineMarker),
            "marker" => Some(Self::Marker),
            "none" => Some(Self::None),
            "smooth" => Some(Self::Smooth),
            "smoothMarker" => Some(Self::SmoothMarker),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::LineMarker => "lineMarker",
            Self::Marker => "marker",
            Self::None => "none",
            Self::Smooth => "smooth",
            Self::SmoothMarker => "smoothMarker",
        }
    }
}

/// Display style for a two-dimensional radar plot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RadarStyle {
    Filled,
    Marker,
    Standard,
}

impl RadarStyle {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "filled" => Some(Self::Filled),
            "marker" => Some(Self::Marker),
            "standard" => Some(Self::Standard),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Filled => "filled",
            Self::Marker => "marker",
            Self::Standard => "standard",
        }
    }
}

/// A supported two-dimensional plot owned by one plot area.
#[derive(Clone, Debug, PartialEq)]
pub enum Plot {
    Bar {
        direction: BarDirection,
        grouping: BarGrouping,
        gap_width: u16,
        overlap: i8,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
        axis_ids: [AxisId; 2],
    },
    Line {
        grouping: Grouping,
        marker: bool,
        smooth: bool,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
        axis_ids: [AxisId; 2],
    },
    Pie {
        first_slice_angle: u16,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
    },
    Doughnut {
        first_slice_angle: u16,
        hole_size: u8,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
    },
    Area {
        grouping: Grouping,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
        axis_ids: [AxisId; 2],
    },
    Scatter {
        style: ScatterStyle,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
        axis_ids: [AxisId; 2],
    },
    Radar {
        style: RadarStyle,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
        axis_ids: [AxisId; 2],
    },
}

impl Plot {
    pub fn bar(
        direction: BarDirection,
        grouping: BarGrouping,
        series: Vec<Series>,
        axis_ids: [AxisId; 2],
    ) -> Result<Self> {
        let plot = Self::Bar {
            direction,
            grouping,
            gap_width: 150,
            overlap: 0,
            series,
            data_labels: None,
            axis_ids,
        };
        plot.validate()?;
        Ok(plot)
    }

    pub fn line(grouping: Grouping, series: Vec<Series>, axis_ids: [AxisId; 2]) -> Result<Self> {
        let plot = Self::Line {
            grouping,
            marker: false,
            smooth: false,
            series,
            data_labels: None,
            axis_ids,
        };
        plot.validate()?;
        Ok(plot)
    }

    pub fn pie(series: Vec<Series>) -> Result<Self> {
        let plot = Self::Pie {
            first_slice_angle: 0,
            series,
            data_labels: None,
        };
        plot.validate()?;
        Ok(plot)
    }

    pub fn doughnut(series: Vec<Series>) -> Result<Self> {
        let plot = Self::Doughnut {
            first_slice_angle: 0,
            hole_size: 50,
            series,
            data_labels: None,
        };
        plot.validate()?;
        Ok(plot)
    }

    pub fn area(grouping: Grouping, series: Vec<Series>, axis_ids: [AxisId; 2]) -> Result<Self> {
        let plot = Self::Area {
            grouping,
            series,
            data_labels: None,
            axis_ids,
        };
        plot.validate()?;
        Ok(plot)
    }

    pub fn scatter(
        style: ScatterStyle,
        series: Vec<Series>,
        axis_ids: [AxisId; 2],
    ) -> Result<Self> {
        let plot = Self::Scatter {
            style,
            series,
            data_labels: None,
            axis_ids,
        };
        plot.validate()?;
        Ok(plot)
    }

    pub fn radar(style: RadarStyle, series: Vec<Series>, axis_ids: [AxisId; 2]) -> Result<Self> {
        let plot = Self::Radar {
            style,
            series,
            data_labels: None,
            axis_ids,
        };
        plot.validate()?;
        Ok(plot)
    }

    fn axis_ids(&self) -> Option<[AxisId; 2]> {
        match self {
            Self::Bar { axis_ids, .. }
            | Self::Line { axis_ids, .. }
            | Self::Area { axis_ids, .. }
            | Self::Scatter { axis_ids, .. }
            | Self::Radar { axis_ids, .. } => Some(*axis_ids),
            Self::Pie { .. } | Self::Doughnut { .. } => None,
        }
    }

    fn validate(&self) -> Result<()> {
        let (series, labels, axis_ids) = match self {
            Self::Bar {
                gap_width,
                overlap,
                series,
                data_labels,
                axis_ids,
                ..
            } => {
                if *gap_width > 500 {
                    return Err(ChartError::InvalidValue {
                        element: "c:gapWidth".to_owned(),
                        value: gap_width.to_string(),
                    });
                }
                if !(-100..=100).contains(overlap) {
                    return Err(ChartError::InvalidValue {
                        element: "c:overlap".to_owned(),
                        value: overlap.to_string(),
                    });
                }
                (series, data_labels, Some(axis_ids))
            }
            Self::Line {
                series,
                data_labels,
                axis_ids,
                ..
            } => (series, data_labels, Some(axis_ids)),
            Self::Pie {
                first_slice_angle,
                series,
                data_labels,
            } => {
                if *first_slice_angle > 360 {
                    return Err(ChartError::InvalidValue {
                        element: "c:firstSliceAng".to_owned(),
                        value: first_slice_angle.to_string(),
                    });
                }
                (series, data_labels, None)
            }
            Self::Doughnut {
                first_slice_angle,
                hole_size,
                series,
                data_labels,
            } => {
                if *first_slice_angle > 360 {
                    return Err(ChartError::InvalidValue {
                        element: "c:firstSliceAng".to_owned(),
                        value: first_slice_angle.to_string(),
                    });
                }
                if !(10..=90).contains(hole_size) {
                    return Err(ChartError::InvalidValue {
                        element: "c:holeSize".to_owned(),
                        value: hole_size.to_string(),
                    });
                }
                (series, data_labels, None)
            }
            Self::Area {
                series,
                data_labels,
                axis_ids,
                ..
            }
            | Self::Radar {
                series,
                data_labels,
                axis_ids,
                ..
            } => (series, data_labels, Some(axis_ids)),
            Self::Scatter {
                series,
                data_labels,
                axis_ids,
                ..
            } => {
                for item in series {
                    if !matches!(item.categories, Some(AxisData::Numeric(_))) {
                        return Err(ChartError::InvalidValue {
                            element: "c:xVal".to_owned(),
                            value: "scatter series requires numeric x values".to_owned(),
                        });
                    }
                }
                (series, data_labels, Some(axis_ids))
            }
        };
        if series.is_empty() {
            return Err(ChartError::MissingElement("c:ser".to_owned()));
        }
        let scatter_plot = matches!(self, Self::Scatter { .. });
        for item in series {
            if item.bubble_size.is_some() || item.opaque_bubble_size {
                return Err(ChartError::InvalidValue {
                    element: "c:bubbleSize".to_owned(),
                    value: "bubble size is only valid in an opaque bubble plot".to_owned(),
                });
            }
            if item.uses_scatter_wrappers && !scatter_plot {
                return Err(ChartError::InvalidValue {
                    element: "c:ser".to_owned(),
                    value: "x/y wrappers are only valid in scatter plots".to_owned(),
                });
            }
            if scatter_plot && item.parsed_from_xml && !item.uses_scatter_wrappers {
                return Err(ChartError::InvalidValue {
                    element: "c:ser".to_owned(),
                    value: "scatter plots require xVal/yVal wrappers".to_owned(),
                });
            }
        }
        if let Some(axis_ids) = axis_ids {
            if axis_ids[0] == axis_ids[1] {
                return Err(ChartError::DuplicateElement(format!(
                    "c:axId {}",
                    axis_ids[0].value()
                )));
            }
            for id in axis_ids {
                AxisId::new(id.value())?;
            }
        }
        if let Some(labels) = labels {
            labels.validate()?;
        }
        for item in series {
            item.values.validate()?;
        }
        Ok(())
    }
}

/// The concrete ChartML root used by an axis.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AxisKind {
    Category,
    Value,
    Date,
    Series,
}

impl AxisKind {
    fn parse(local: &[u8]) -> Option<Self> {
        match local {
            b"catAx" => Some(Self::Category),
            b"valAx" => Some(Self::Value),
            b"dateAx" => Some(Self::Date),
            b"serAx" => Some(Self::Series),
            _ => None,
        }
    }

    const fn local(self) -> &'static str {
        match self {
            Self::Category => "catAx",
            Self::Value => "valAx",
            Self::Date => "dateAx",
            Self::Series => "serAx",
        }
    }
}

/// Direction of values along an axis scale.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Orientation {
    #[default]
    MinMax,
    MaxMin,
}

impl Orientation {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "minMax" => Some(Self::MinMax),
            "maxMin" => Some(Self::MaxMin),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::MinMax => "minMax",
            Self::MaxMin => "maxMin",
        }
    }
}

/// Which side of the plot area carries an axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxisPosition {
    Bottom,
    Left,
    Right,
    Top,
}

impl AxisPosition {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "b" => Some(Self::Bottom),
            "l" => Some(Self::Left),
            "r" => Some(Self::Right),
            "t" => Some(Self::Top),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Bottom => "b",
            Self::Left => "l",
            Self::Right => "r",
            Self::Top => "t",
        }
    }
}

/// Placement of a major or minor tick relative to its axis line.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TickMark {
    Cross,
    Inside,
    #[default]
    None,
    Outside,
}

impl TickMark {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "cross" => Some(Self::Cross),
            "in" => Some(Self::Inside),
            "none" => Some(Self::None),
            "out" => Some(Self::Outside),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Cross => "cross",
            Self::Inside => "in",
            Self::None => "none",
            Self::Outside => "out",
        }
    }
}

/// Placement of tick labels relative to an axis.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TickLabelPosition {
    High,
    Low,
    #[default]
    NextTo,
    None,
}

impl TickLabelPosition {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "high" => Some(Self::High),
            "low" => Some(Self::Low),
            "nextTo" => Some(Self::NextTo),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Low => "low",
            Self::NextTo => "nextTo",
            Self::None => "none",
        }
    }
}

/// The modelled values of `c:scaling`.
#[derive(Clone, Debug)]
pub struct Scaling {
    pub log_base: Option<f64>,
    pub orientation: Orientation,
    pub maximum: Option<f64>,
    pub minimum: Option<f64>,
    log_base_markup: Option<ScalarMarkup>,
    orientation_markup: Option<ScalarMarkup>,
    maximum_markup: Option<ScalarMarkup>,
    minimum_markup: Option<ScalarMarkup>,
    raw_attributes: XmlAttributes,
    raw_children: OrderedRawChildren,
}

impl PartialEq for Scaling {
    fn eq(&self, other: &Self) -> bool {
        self.log_base == other.log_base
            && self.orientation == other.orientation
            && self.maximum == other.maximum
            && self.minimum == other.minimum
            && optional_scalar_markup_eq(&self.log_base_markup, &other.log_base_markup)
            && optional_scalar_markup_eq(&self.orientation_markup, &other.orientation_markup)
            && optional_scalar_markup_eq(&self.maximum_markup, &other.maximum_markup)
            && optional_scalar_markup_eq(&self.minimum_markup, &other.minimum_markup)
            && self.raw_attributes == other.raw_attributes
            && self.raw_children == other.raw_children
    }
}

impl Default for Scaling {
    fn default() -> Self {
        Self {
            log_base: None,
            orientation: Orientation::MinMax,
            maximum: None,
            minimum: None,
            log_base_markup: None,
            orientation_markup: None,
            maximum_markup: None,
            minimum_markup: None,
            raw_attributes: Vec::new(),
            raw_children: OrderedRawChildren::default(),
        }
    }
}

/// Shape properties on a major or minor gridline container.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChartLines {
    pub sp_pr: Option<CT_ShapeProperties>,
    raw_attributes: XmlAttributes,
    raw_children: OrderedRawChildren,
}

/// Number format applied to axis values and labels.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NumberFormat {
    pub format_code: String,
    pub source_linked: bool,
    raw_attributes: XmlAttributes,
    raw_content: Vec<Vec<u8>>,
}

/// Placement of a chart data label relative to its data point.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DataLabelPosition {
    BestFit,
    Bottom,
    Center,
    InsideBase,
    InsideEnd,
    Left,
    OutsideEnd,
    Right,
    Top,
}

impl DataLabelPosition {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "bestFit" => Some(Self::BestFit),
            "b" => Some(Self::Bottom),
            "ctr" => Some(Self::Center),
            "inBase" => Some(Self::InsideBase),
            "inEnd" => Some(Self::InsideEnd),
            "l" => Some(Self::Left),
            "outEnd" => Some(Self::OutsideEnd),
            "r" => Some(Self::Right),
            "t" => Some(Self::Top),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::BestFit => "bestFit",
            Self::Bottom => "b",
            Self::Center => "ctr",
            Self::InsideBase => "inBase",
            Self::InsideEnd => "inEnd",
            Self::Left => "l",
            Self::OutsideEnd => "outEnd",
            Self::Right => "r",
            Self::Top => "t",
        }
    }
}

/// Collection-level data-label defaults for one chart series or plot.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CT_DLbls {
    pub number_format: Option<NumberFormat>,
    pub position: Option<DataLabelPosition>,
    pub separator: Option<String>,
    pub show_legend_key: bool,
    pub show_value: bool,
    pub show_category_name: bool,
    pub show_series_name: bool,
    pub show_percent: bool,
    pub show_bubble_size: bool,
    position_markup: Option<ScalarMarkup>,
    show_legend_key_markup: Option<ScalarMarkup>,
    show_value_markup: Option<ScalarMarkup>,
    show_category_name_markup: Option<ScalarMarkup>,
    show_series_name_markup: Option<ScalarMarkup>,
    show_percent_markup: Option<ScalarMarkup>,
    show_bubble_size_markup: Option<ScalarMarkup>,
    separator_markup: Option<TextMarkup>,
    raw_attributes: XmlAttributes,
    namespace_declarations: XmlAttributes,
    raw_children: OrderedRawChildren,
}

/// How a chart displays cells whose cached values are blank.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DispBlanksAs {
    #[default]
    Gap,
    Zero,
    Span,
}

impl DispBlanksAs {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "gap" => Some(Self::Gap),
            "zero" => Some(Self::Zero),
            "span" => Some(Self::Span),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Gap => "gap",
            Self::Zero => "zero",
            Self::Span => "span",
        }
    }
}

/// A chart title shell whose current children remain opaque until later stories.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CT_Title {
    raw_attributes: Vec<(String, String)>,
    raw_children: OrderedRawChildren,
}

/// Preserved lexical and ordered-raw state for one typed plot.
#[derive(Clone, Debug, Default, PartialEq)]
struct PlotMarkup {
    raw_attributes: XmlAttributes,
    raw_children: OrderedRawChildren,
    direction: Option<ScalarMarkup>,
    grouping: Option<ScalarMarkup>,
    gap_width: Option<ScalarMarkup>,
    overlap: Option<ScalarMarkup>,
    marker: Option<ScalarMarkup>,
    smooth: Option<ScalarMarkup>,
    first_slice_angle: Option<ScalarMarkup>,
    hole_size: Option<ScalarMarkup>,
    style: Option<ScalarMarkup>,
    axis_ids: Vec<AxisIdMarkup>,
    original_series_keys: Vec<(u32, u32)>,
    original_axis_ids: Vec<AxisId>,
    parsed_bar: Option<bool>,
    parsed_remaining: Option<&'static str>,
}

/// A plot-area shell. F-119 through F-122 replace selected raw slots with types.
#[derive(Clone, Debug, PartialEq)]
pub struct CT_PlotArea {
    raw_attributes: Vec<(String, String)>,
    raw_children: OrderedRawChildren,
    namespace_bindings: NamespaceBindings,
    plots: Option<Vec<Plot>>,
    plot_markup: Vec<PlotMarkup>,
    axes: Vec<Axis>,
}

/// A chart legend shell whose current children remain opaque.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CT_Legend {
    raw_attributes: Vec<(String, String)>,
    raw_children: OrderedRawChildren,
}

#[derive(Clone, Debug, PartialEq)]
struct AxisIdMarkup {
    scalar: ScalarMarkup,
    parsed: AxisId,
    lexical: String,
}

/// The common modelled sequence of one ChartML axis.
#[derive(Clone, Debug)]
pub struct Axis {
    pub kind: AxisKind,
    pub id: AxisId,
    pub scaling: Scaling,
    pub deleted: bool,
    pub position: AxisPosition,
    pub major_gridlines: Option<ChartLines>,
    pub minor_gridlines: Option<ChartLines>,
    pub title: Option<CT_Title>,
    pub number_format: Option<NumberFormat>,
    pub major_tick_mark: TickMark,
    pub minor_tick_mark: TickMark,
    pub tick_label_position: TickLabelPosition,
    pub sp_pr: Option<CT_ShapeProperties>,
    pub tx_pr: Option<CT_TextBody>,
    pub cross_axis: AxisId,
    parsed_kind: Option<AxisKind>,
    id_markup: AxisIdMarkup,
    delete_markup: Option<ScalarMarkup>,
    position_markup: ScalarMarkup,
    major_tick_mark_markup: Option<ScalarMarkup>,
    minor_tick_mark_markup: Option<ScalarMarkup>,
    tick_label_position_markup: Option<ScalarMarkup>,
    cross_axis_markup: AxisIdMarkup,
    raw_attributes: XmlAttributes,
    namespace_declarations: XmlAttributes,
    raw_children: OrderedRawChildren,
}

impl PartialEq for Axis {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.id == other.id
            && axis_id_markup_eq(&self.id_markup, &other.id_markup)
            && self.scaling == other.scaling
            && self.deleted == other.deleted
            && optional_scalar_markup_eq(&self.delete_markup, &other.delete_markup)
            && self.position == other.position
            && self.position_markup == other.position_markup
            && self.major_gridlines == other.major_gridlines
            && self.minor_gridlines == other.minor_gridlines
            && self.title == other.title
            && self.number_format == other.number_format
            && self.major_tick_mark == other.major_tick_mark
            && optional_scalar_markup_eq(
                &self.major_tick_mark_markup,
                &other.major_tick_mark_markup,
            )
            && self.minor_tick_mark == other.minor_tick_mark
            && optional_scalar_markup_eq(
                &self.minor_tick_mark_markup,
                &other.minor_tick_mark_markup,
            )
            && self.tick_label_position == other.tick_label_position
            && optional_scalar_markup_eq(
                &self.tick_label_position_markup,
                &other.tick_label_position_markup,
            )
            && self.sp_pr == other.sp_pr
            && self.tx_pr == other.tx_pr
            && self.cross_axis == other.cross_axis
            && axis_id_markup_eq(&self.cross_axis_markup, &other.cross_axis_markup)
            && self.raw_attributes == other.raw_attributes
            && self.namespace_declarations == other.namespace_declarations
            && self.raw_children == other.raw_children
    }
}

impl Axis {
    pub fn new(kind: AxisKind, id: AxisId, position: AxisPosition, cross_axis: AxisId) -> Self {
        Self {
            kind,
            id,
            scaling: Scaling::default(),
            deleted: false,
            position,
            major_gridlines: None,
            minor_gridlines: None,
            title: None,
            number_format: None,
            major_tick_mark: TickMark::None,
            minor_tick_mark: TickMark::None,
            tick_label_position: TickLabelPosition::NextTo,
            sp_pr: None,
            tx_pr: None,
            cross_axis,
            parsed_kind: None,
            id_markup: default_axis_id_markup(id),
            delete_markup: None,
            position_markup: ScalarMarkup::default(),
            major_tick_mark_markup: None,
            minor_tick_mark_markup: None,
            tick_label_position_markup: None,
            cross_axis_markup: default_axis_id_markup(cross_axis),
            raw_attributes: Vec::new(),
            namespace_declarations: Vec::new(),
            raw_children: OrderedRawChildren::default(),
        }
    }

    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        Self::from_xml_with_namespaces(xml, &chart_namespace_defaults())
    }

    fn from_xml_with_namespaces(xml: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) => {
                    let name = element.name();
                    let local = local_name(name.as_ref());
                    let Some(kind) = AxisKind::parse(local) else {
                        return Err(ChartError::UnexpectedElement(element_name(&element)));
                    };
                    chart_root_prefix(&element)?;
                    if !element_is_in_namespace(&element, C_NS, inherited)? {
                        return Err(ChartError::UnexpectedElement(element_name(&element)));
                    }
                    return Self::from_element(&mut reader, &element, inherited, kind);
                }
                Event::Empty(element) => {
                    let name = element.name();
                    let local = local_name(name.as_ref());
                    if AxisKind::parse(local).is_some() {
                        return Err(ChartError::MissingElement("c:axId".to_owned()));
                    }
                    return Err(ChartError::UnexpectedElement(element_name(&element)));
                }
                Event::Eof => return Err(ChartError::MissingElement("ChartML axis".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_element(
        reader: &mut Reader<&[u8]>,
        start: &BytesStart<'_>,
        inherited: &NamespaceBindings,
        kind: AxisKind,
    ) -> Result<Self> {
        reject_conflicting_prefix(start, b"a", A_NS)?;
        reject_conflicting_prefix(start, b"r", R_NS)?;
        let namespaces = chart_bindings(inherited, start)?;
        require_fixed_namespace(&namespaces, b"c", C_NS, start)?;
        require_fixed_namespace(&namespaces, b"a", A_NS, start)?;
        require_fixed_namespace(&namespaces, b"r", R_NS, start)?;
        let (raw_attributes, _) =
            capture_fixed_root_attributes(start, &["xmlns:c", "xmlns:a", "xmlns:r"])?;
        let namespace_declarations = standalone_namespace_declarations(&namespaces)?;
        let mut state = AxisParseState::default();
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) => {
                    let name = chart_child_local(&element, &namespaces)?;
                    let raw = capture_element(reader, &element)?;
                    state.parse_child(name.as_deref().unwrap_or_default(), raw, &namespaces)?;
                }
                Event::Empty(element) => {
                    let name = chart_child_local(&element, &namespaces)?;
                    let raw = capture_empty_element(&element)?;
                    state.parse_child(name.as_deref().unwrap_or_default(), raw, &namespaces)?;
                }
                event @ (Event::Text(_)
                | Event::CData(_)
                | Event::Comment(_)
                | Event::PI(_)
                | Event::GeneralRef(_)) => state.capture_event(capture_event(event)?),
                Event::End(element)
                    if matches_local_name(element.name().as_ref(), kind.local().as_bytes()) =>
                {
                    break;
                }
                Event::Eof => return Err(missing_end(&format!("c:{}", kind.local()))),
                _ => {}
            }
            buffer.clear();
        }
        state.finish(kind, raw_attributes, namespace_declarations)
    }

    pub fn to_xml(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut writer = Writer::new(Vec::new());
        let root = format!("c:{}", self.kind.local());
        let mut start = BytesStart::new(&root);
        start.push_attribute(("xmlns:c", C_NS));
        start.push_attribute(("xmlns:a", A_NS));
        start.push_attribute(("xmlns:r", R_NS));
        push_attributes(&mut start, &self.namespace_declarations);
        push_attributes(&mut start, &self.raw_attributes);
        writer
            .write_event(Event::Start(start))
            .map_err(OxmlError::from)?;
        emit_raw(&mut writer, self.raw_children.at(0))?;
        write_axis_id(&mut writer, "c:axId", self.id, &self.id_markup)?;
        emit_raw(&mut writer, self.raw_children.at(1))?;
        self.scaling.write_xml(&mut writer)?;
        emit_raw(&mut writer, self.raw_children.at(2))?;
        if self.deleted || self.delete_markup.is_some() {
            write_scalar(
                &mut writer,
                "c:delete",
                bool_lexical(self.deleted),
                self.delete_markup.as_ref(),
            )?;
        }
        emit_raw(&mut writer, self.raw_children.at(3))?;
        write_scalar(
            &mut writer,
            "c:axPos",
            self.position.as_str(),
            Some(&self.position_markup),
        )?;
        emit_raw(&mut writer, self.raw_children.at(4))?;
        if let Some(lines) = &self.major_gridlines {
            lines.write_xml(&mut writer, "c:majorGridlines")?;
        }
        emit_raw(&mut writer, self.raw_children.at(5))?;
        if let Some(lines) = &self.minor_gridlines {
            lines.write_xml(&mut writer, "c:minorGridlines")?;
        }
        emit_raw(&mut writer, self.raw_children.at(6))?;
        if let Some(title) = &self.title {
            title.write_xml(&mut writer)?;
        }
        emit_raw(&mut writer, self.raw_children.at(7))?;
        if let Some(number_format) = &self.number_format {
            number_format.write_xml(&mut writer)?;
        }
        emit_raw(&mut writer, self.raw_children.at(8))?;
        if self.major_tick_mark != TickMark::None || self.major_tick_mark_markup.is_some() {
            write_scalar(
                &mut writer,
                "c:majorTickMark",
                self.major_tick_mark.as_str(),
                self.major_tick_mark_markup.as_ref(),
            )?;
        }
        emit_raw(&mut writer, self.raw_children.at(9))?;
        if self.minor_tick_mark != TickMark::None || self.minor_tick_mark_markup.is_some() {
            write_scalar(
                &mut writer,
                "c:minorTickMark",
                self.minor_tick_mark.as_str(),
                self.minor_tick_mark_markup.as_ref(),
            )?;
        }
        emit_raw(&mut writer, self.raw_children.at(10))?;
        if self.tick_label_position != TickLabelPosition::NextTo
            || self.tick_label_position_markup.is_some()
        {
            write_scalar(
                &mut writer,
                "c:tickLblPos",
                self.tick_label_position.as_str(),
                self.tick_label_position_markup.as_ref(),
            )?;
        }
        emit_raw(&mut writer, self.raw_children.at(11))?;
        if let Some(properties) = &self.sp_pr {
            properties.write_xml_as(&mut writer, "c:spPr")?;
        }
        emit_raw(&mut writer, self.raw_children.at(12))?;
        if let Some(text) = &self.tx_pr {
            text.write_xml_as(&mut writer, "c:txPr")?;
        }
        emit_raw(&mut writer, self.raw_children.at(13))?;
        write_axis_id(
            &mut writer,
            "c:crossAx",
            self.cross_axis,
            &self.cross_axis_markup,
        )?;
        emit_raw(&mut writer, self.raw_children.at(14))?;
        writer
            .write_event(Event::End(BytesEnd::new(&root)))
            .map_err(OxmlError::from)?;
        Ok(writer.into_inner())
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }

    fn validate(&self) -> Result<()> {
        if let Some(parsed_kind) = self.parsed_kind
            && self.kind != parsed_kind
        {
            return Err(ChartError::InvalidValue {
                element: "ChartML axis root".to_owned(),
                value: format!(
                    "parsed c:{} cannot be relabelled c:{}",
                    parsed_kind.local(),
                    self.kind.local()
                ),
            });
        }
        AxisId::new(self.id.value())?;
        AxisId::new(self.cross_axis.value())?;
        self.scaling.validate()?;
        if let Some(number_format) = &self.number_format {
            number_format.validate()?;
        }
        Ok(())
    }
}

#[derive(Default)]
struct AxisParseState {
    id: Option<(AxisId, AxisIdMarkup)>,
    scaling: Option<Scaling>,
    deleted: Option<(bool, ScalarMarkup)>,
    position: Option<(AxisPosition, ScalarMarkup)>,
    major_gridlines: Option<ChartLines>,
    minor_gridlines: Option<ChartLines>,
    title: Option<CT_Title>,
    number_format: Option<NumberFormat>,
    major_tick_mark: Option<(TickMark, ScalarMarkup)>,
    minor_tick_mark: Option<(TickMark, ScalarMarkup)>,
    tick_label_position: Option<(TickLabelPosition, ScalarMarkup)>,
    sp_pr: Option<CT_ShapeProperties>,
    tx_pr: Option<CT_TextBody>,
    cross_axis: Option<(AxisId, AxisIdMarkup)>,
    raw_children: OrderedRawChildren,
    boundary: usize,
}

impl AxisParseState {
    fn capture_event(&mut self, raw: Vec<u8>) {
        self.raw_children.push(self.boundary, raw);
    }

    fn parse_child(
        &mut self,
        name: &[u8],
        raw: Vec<u8>,
        namespaces: &NamespaceBindings,
    ) -> Result<()> {
        match name {
            b"axId" => {
                set_once(&mut self.id, parse_axis_id_scalar(&raw, "axId")?, "c:axId")?;
                self.boundary = self.boundary.max(1);
            }
            b"scaling" => {
                set_once(
                    &mut self.scaling,
                    Scaling::from_xml(&raw, namespaces)?,
                    "c:scaling",
                )?;
                self.boundary = self.boundary.max(2);
            }
            b"delete" => {
                set_once(
                    &mut self.deleted,
                    parse_bool_value(&raw, "delete")?,
                    "c:delete",
                )?;
                self.boundary = self.boundary.max(3);
            }
            b"axPos" => {
                set_once(&mut self.position, parse_axis_position(&raw)?, "c:axPos")?;
                self.boundary = self.boundary.max(4);
            }
            b"majorGridlines" => {
                set_once(
                    &mut self.major_gridlines,
                    ChartLines::from_xml(&raw, b"majorGridlines", namespaces)?,
                    "c:majorGridlines",
                )?;
                self.boundary = self.boundary.max(5);
            }
            b"minorGridlines" => {
                set_once(
                    &mut self.minor_gridlines,
                    ChartLines::from_xml(&raw, b"minorGridlines", namespaces)?,
                    "c:minorGridlines",
                )?;
                self.boundary = self.boundary.max(6);
            }
            b"title" => {
                set_once(&mut self.title, CT_Title::from_xml(&raw)?, "c:title")?;
                self.boundary = self.boundary.max(7);
            }
            b"numFmt" => {
                set_once(
                    &mut self.number_format,
                    NumberFormat::from_xml(&raw)?,
                    "c:numFmt",
                )?;
                self.boundary = self.boundary.max(8);
            }
            b"majorTickMark" => {
                set_once(
                    &mut self.major_tick_mark,
                    parse_tick_mark(&raw, "majorTickMark")?,
                    "c:majorTickMark",
                )?;
                self.boundary = self.boundary.max(9);
            }
            b"minorTickMark" => {
                set_once(
                    &mut self.minor_tick_mark,
                    parse_tick_mark(&raw, "minorTickMark")?,
                    "c:minorTickMark",
                )?;
                self.boundary = self.boundary.max(10);
            }
            b"tickLblPos" => {
                set_once(
                    &mut self.tick_label_position,
                    parse_tick_label_position(&raw)?,
                    "c:tickLblPos",
                )?;
                self.boundary = self.boundary.max(11);
            }
            b"spPr" => {
                reject_conflicting_prefix_in_xml(&raw, b"a", A_NS)?;
                let properties = CT_ShapeProperties::from_xml(&raw)?;
                let mut writer = Writer::new(Vec::new());
                properties.write_xml_as(&mut writer, "c:spPr")?;
                reject_rewritten_foreign_elements(&raw, &writer.into_inner(), namespaces, b"spPr")?;
                set_once(&mut self.sp_pr, properties, "c:spPr")?;
                self.boundary = self.boundary.max(12);
            }
            b"txPr" => {
                reject_conflicting_prefix_in_xml(&raw, b"a", A_NS)?;
                let text = CT_TextBody::from_xml_as(&raw, b"txPr")?;
                let mut writer = Writer::new(Vec::new());
                text.write_xml_as(&mut writer, "c:txPr")?;
                reject_rewritten_foreign_elements(&raw, &writer.into_inner(), namespaces, b"txPr")?;
                set_once(&mut self.tx_pr, text, "c:txPr")?;
                self.boundary = self.boundary.max(13);
            }
            b"crossAx" => {
                set_once(
                    &mut self.cross_axis,
                    parse_axis_id_scalar(&raw, "crossAx")?,
                    "c:crossAx",
                )?;
                self.boundary = self.boundary.max(14);
            }
            _ => self.raw_children.push(self.boundary, raw),
        }
        Ok(())
    }

    fn finish(
        self,
        kind: AxisKind,
        raw_attributes: XmlAttributes,
        namespace_declarations: XmlAttributes,
    ) -> Result<Axis> {
        let (id, id_markup) = self
            .id
            .ok_or_else(|| ChartError::MissingElement("c:axId".to_owned()))?;
        let scaling = self
            .scaling
            .ok_or_else(|| ChartError::MissingElement("c:scaling".to_owned()))?;
        let (position, position_markup) = self
            .position
            .ok_or_else(|| ChartError::MissingElement("c:axPos".to_owned()))?;
        let (cross_axis, cross_axis_markup) = self
            .cross_axis
            .ok_or_else(|| ChartError::MissingElement("c:crossAx".to_owned()))?;
        let (deleted, delete_markup) = self
            .deleted
            .map(|(value, markup)| (value, Some(markup)))
            .unwrap_or((false, None));
        let (major_tick_mark, major_tick_mark_markup) = self
            .major_tick_mark
            .map(|(value, markup)| (value, Some(markup)))
            .unwrap_or((TickMark::None, None));
        let (minor_tick_mark, minor_tick_mark_markup) = self
            .minor_tick_mark
            .map(|(value, markup)| (value, Some(markup)))
            .unwrap_or((TickMark::None, None));
        let (tick_label_position, tick_label_position_markup) = self
            .tick_label_position
            .map(|(value, markup)| (value, Some(markup)))
            .unwrap_or((TickLabelPosition::NextTo, None));
        let axis = Axis {
            kind,
            id,
            scaling,
            deleted,
            position,
            major_gridlines: self.major_gridlines,
            minor_gridlines: self.minor_gridlines,
            title: self.title,
            number_format: self.number_format,
            major_tick_mark,
            minor_tick_mark,
            tick_label_position,
            sp_pr: self.sp_pr,
            tx_pr: self.tx_pr,
            cross_axis,
            parsed_kind: Some(kind),
            id_markup,
            delete_markup,
            position_markup,
            major_tick_mark_markup,
            minor_tick_mark_markup,
            tick_label_position_markup,
            cross_axis_markup,
            raw_attributes,
            namespace_declarations,
            raw_children: raw_children_in_schema_order(&self.raw_children, 14),
        };
        axis.validate()?;
        Ok(axis)
    }
}

impl Scaling {
    fn from_xml(xml: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element)
                    if matches_local_name(element.name().as_ref(), b"scaling") =>
                {
                    let namespaces = typed_rewrite_bindings(&element, inherited)?;
                    let raw_attributes = capture_attributes(&element)?;
                    let mut log_base = None;
                    let mut orientation = None;
                    let mut maximum = None;
                    let mut minimum = None;
                    let mut raw_children = OrderedRawChildren::default();
                    let mut boundary = 0usize;
                    let mut inner = Vec::new();
                    loop {
                        match reader
                            .read_event_into(&mut inner)
                            .map_err(OxmlError::from)?
                        {
                            Event::Start(child) => {
                                let name = chart_child_local(&child, &namespaces)?;
                                let raw = capture_element(&mut reader, &child)?;
                                parse_scaling_child(
                                    name.as_deref().unwrap_or_default(),
                                    raw,
                                    &mut log_base,
                                    &mut orientation,
                                    &mut maximum,
                                    &mut minimum,
                                    &mut raw_children,
                                    &mut boundary,
                                )?;
                            }
                            Event::Empty(child) => {
                                let name = chart_child_local(&child, &namespaces)?;
                                let raw = capture_empty_element(&child)?;
                                parse_scaling_child(
                                    name.as_deref().unwrap_or_default(),
                                    raw,
                                    &mut log_base,
                                    &mut orientation,
                                    &mut maximum,
                                    &mut minimum,
                                    &mut raw_children,
                                    &mut boundary,
                                )?;
                            }
                            event @ (Event::Text(_)
                            | Event::CData(_)
                            | Event::Comment(_)
                            | Event::PI(_)
                            | Event::GeneralRef(_)) => {
                                raw_children.push(boundary, capture_event(event)?);
                            }
                            Event::End(end)
                                if matches_local_name(end.name().as_ref(), b"scaling") =>
                            {
                                let (log_base, log_base_markup) = optional_markup(log_base);
                                let (orientation, orientation_markup) = orientation
                                    .map(|(value, markup)| (value, Some(markup)))
                                    .unwrap_or((Orientation::MinMax, None));
                                let (maximum, maximum_markup) = optional_markup(maximum);
                                let (minimum, minimum_markup) = optional_markup(minimum);
                                let scaling = Self {
                                    log_base,
                                    orientation,
                                    maximum,
                                    minimum,
                                    log_base_markup,
                                    orientation_markup,
                                    maximum_markup,
                                    minimum_markup,
                                    raw_attributes,
                                    raw_children: raw_children_in_schema_order(&raw_children, 4),
                                };
                                scaling.validate()?;
                                return Ok(scaling);
                            }
                            Event::Eof => return Err(missing_end("c:scaling")),
                            _ => {}
                        }
                        inner.clear();
                    }
                }
                Event::Empty(element)
                    if matches_local_name(element.name().as_ref(), b"scaling") =>
                {
                    typed_rewrite_bindings(&element, inherited)?;
                    return Ok(Self {
                        raw_attributes: capture_attributes(&element)?,
                        ..Self::default()
                    });
                }
                Event::Start(element) | Event::Empty(element) => {
                    return Err(ChartError::UnexpectedElement(element_name(&element)));
                }
                Event::Eof => return Err(ChartError::MissingElement("c:scaling".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        self.validate()?;
        let mut start = BytesStart::new("c:scaling");
        push_attributes(&mut start, &self.raw_attributes);
        writer
            .write_event(Event::Start(start))
            .map_err(OxmlError::from)?;
        emit_raw(writer, self.raw_children.at(0))?;
        if let Some(value) = self.log_base {
            write_scalar(
                writer,
                "c:logBase",
                &value.to_string(),
                self.log_base_markup.as_ref(),
            )?;
        }
        emit_raw(writer, self.raw_children.at(1))?;
        if self.orientation != Orientation::MinMax || self.orientation_markup.is_some() {
            write_scalar(
                writer,
                "c:orientation",
                self.orientation.as_str(),
                self.orientation_markup.as_ref(),
            )?;
        }
        emit_raw(writer, self.raw_children.at(2))?;
        if let Some(value) = self.maximum {
            write_scalar(
                writer,
                "c:max",
                &value.to_string(),
                self.maximum_markup.as_ref(),
            )?;
        }
        emit_raw(writer, self.raw_children.at(3))?;
        if let Some(value) = self.minimum {
            write_scalar(
                writer,
                "c:min",
                &value.to_string(),
                self.minimum_markup.as_ref(),
            )?;
        }
        emit_raw(writer, self.raw_children.at(4))?;
        writer
            .write_event(Event::End(BytesEnd::new("c:scaling")))
            .map_err(OxmlError::from)?;
        Ok(())
    }

    fn validate(&self) -> Result<()> {
        if let Some(value) = self.log_base
            && (!value.is_finite() || !(2.0..=1000.0).contains(&value))
        {
            return Err(ChartError::InvalidValue {
                element: "c:logBase".to_owned(),
                value: value.to_string(),
            });
        }
        for (element, value) in [("c:max", self.maximum), ("c:min", self.minimum)] {
            if let Some(value) = value
                && !value.is_finite()
            {
                return Err(ChartError::InvalidValue {
                    element: element.to_owned(),
                    value: value.to_string(),
                });
            }
        }
        if let (Some(minimum), Some(maximum)) = (self.minimum, self.maximum)
            && minimum > maximum
        {
            return Err(ChartError::InvalidValue {
                element: "c:scaling".to_owned(),
                value: format!("minimum {minimum} exceeds maximum {maximum}"),
            });
        }
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn parse_scaling_child(
    name: &[u8],
    raw: Vec<u8>,
    log_base: &mut Option<(f64, ScalarMarkup)>,
    orientation: &mut Option<(Orientation, ScalarMarkup)>,
    maximum: &mut Option<(f64, ScalarMarkup)>,
    minimum: &mut Option<(f64, ScalarMarkup)>,
    raw_children: &mut OrderedRawChildren,
    boundary: &mut usize,
) -> Result<()> {
    match name {
        b"logBase" => {
            set_once(log_base, parse_f64_scalar(&raw, "logBase")?, "c:logBase")?;
            *boundary = (*boundary).max(1);
        }
        b"orientation" => {
            let (value, markup) = scalar_value(&raw, "orientation")?;
            let value = value
                .ok_or_else(|| invalid_attribute("orientation", "val", "<missing>".to_owned()))?;
            let parsed = Orientation::parse(&value)
                .ok_or_else(|| invalid_attribute("orientation", "val", value))?;
            set_once(orientation, (parsed, markup), "c:orientation")?;
            *boundary = (*boundary).max(2);
        }
        b"max" => {
            set_once(maximum, parse_f64_scalar(&raw, "max")?, "c:max")?;
            *boundary = (*boundary).max(3);
        }
        b"min" => {
            set_once(minimum, parse_f64_scalar(&raw, "min")?, "c:min")?;
            *boundary = (*boundary).max(4);
        }
        _ => raw_children.push(*boundary, raw),
    }
    Ok(())
}

impl ChartLines {
    fn from_xml(xml: &[u8], local: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) if matches_local_name(element.name().as_ref(), local) => {
                    let namespaces = typed_rewrite_bindings(&element, inherited)?;
                    let raw_attributes = capture_attributes(&element)?;
                    let mut sp_pr = None;
                    let mut raw_children = OrderedRawChildren::default();
                    let mut boundary = 0usize;
                    let mut inner = Vec::new();
                    loop {
                        match reader
                            .read_event_into(&mut inner)
                            .map_err(OxmlError::from)?
                        {
                            Event::Start(child) => {
                                let name = chart_child_local(&child, &namespaces)?;
                                let raw = capture_element(&mut reader, &child)?;
                                if name.as_deref() == Some(b"spPr") {
                                    parse_chart_lines_shape(&raw, &namespaces, &mut sp_pr)?;
                                    boundary = 1;
                                } else {
                                    raw_children.push(boundary, raw);
                                }
                            }
                            Event::Empty(child) => {
                                let name = chart_child_local(&child, &namespaces)?;
                                let raw = capture_empty_element(&child)?;
                                if name.as_deref() == Some(b"spPr") {
                                    parse_chart_lines_shape(&raw, &namespaces, &mut sp_pr)?;
                                    boundary = 1;
                                } else {
                                    raw_children.push(boundary, raw);
                                }
                            }
                            event @ (Event::Text(_)
                            | Event::CData(_)
                            | Event::Comment(_)
                            | Event::PI(_)
                            | Event::GeneralRef(_)) => {
                                raw_children.push(boundary, capture_event(event)?);
                            }
                            Event::End(end) if matches_local_name(end.name().as_ref(), local) => {
                                return Ok(Self {
                                    sp_pr,
                                    raw_attributes,
                                    raw_children: raw_children_in_schema_order(&raw_children, 1),
                                });
                            }
                            Event::Eof => {
                                return Err(missing_end(&format!(
                                    "c:{}",
                                    String::from_utf8_lossy(local)
                                )));
                            }
                            _ => {}
                        }
                        inner.clear();
                    }
                }
                Event::Empty(element) if matches_local_name(element.name().as_ref(), local) => {
                    typed_rewrite_bindings(&element, inherited)?;
                    return Ok(Self {
                        sp_pr: None,
                        raw_attributes: capture_attributes(&element)?,
                        raw_children: OrderedRawChildren::default(),
                    });
                }
                Event::Start(element) | Event::Empty(element) => {
                    return Err(ChartError::UnexpectedElement(element_name(&element)));
                }
                Event::Eof => {
                    return Err(ChartError::MissingElement(format!(
                        "c:{}",
                        String::from_utf8_lossy(local)
                    )));
                }
                _ => {}
            }
            buffer.clear();
        }
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>, tag: &str) -> Result<()> {
        let mut start = BytesStart::new(tag);
        push_attributes(&mut start, &self.raw_attributes);
        if self.sp_pr.is_none() && self.raw_children.is_empty() {
            writer
                .write_event(Event::Empty(start))
                .map_err(OxmlError::from)?;
            return Ok(());
        }
        writer
            .write_event(Event::Start(start))
            .map_err(OxmlError::from)?;
        emit_raw(writer, self.raw_children.at(0))?;
        if let Some(properties) = &self.sp_pr {
            properties.write_xml_as(writer, "c:spPr")?;
        }
        emit_raw(writer, self.raw_children.at(1))?;
        writer
            .write_event(Event::End(BytesEnd::new(tag)))
            .map_err(OxmlError::from)?;
        Ok(())
    }
}

fn parse_chart_lines_shape(
    raw: &[u8],
    namespaces: &NamespaceBindings,
    slot: &mut Option<CT_ShapeProperties>,
) -> Result<()> {
    reject_conflicting_prefix_in_xml(raw, b"a", A_NS)?;
    let properties = CT_ShapeProperties::from_xml(raw)?;
    let mut writer = Writer::new(Vec::new());
    properties.write_xml_as(&mut writer, "c:spPr")?;
    reject_rewritten_foreign_elements(raw, &writer.into_inner(), namespaces, b"spPr")?;
    set_once(slot, properties, "c:spPr")
}

impl NumberFormat {
    pub fn new(format_code: String, source_linked: bool) -> Result<Self> {
        let number_format = Self {
            format_code,
            source_linked,
            raw_attributes: Vec::new(),
            raw_content: Vec::new(),
        };
        number_format.validate()?;
        Ok(number_format)
    }

    fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Empty(element) if matches_local_name(element.name().as_ref(), b"numFmt") => {
                    return Self::from_element(&element, Vec::new());
                }
                Event::Start(element) if matches_local_name(element.name().as_ref(), b"numFmt") => {
                    let mut raw_content = Vec::new();
                    let mut inner = Vec::new();
                    loop {
                        match reader
                            .read_event_into(&mut inner)
                            .map_err(OxmlError::from)?
                        {
                            Event::End(end)
                                if matches_local_name(end.name().as_ref(), b"numFmt") =>
                            {
                                return Self::from_element(&element, raw_content);
                            }
                            event @ (Event::Text(_)
                            | Event::CData(_)
                            | Event::Comment(_)
                            | Event::PI(_)
                            | Event::GeneralRef(_)) => raw_content.push(capture_event(event)?),
                            Event::Eof => return Err(missing_end("c:numFmt")),
                            _ => return Err(ChartError::UnexpectedElement("c:numFmt".to_owned())),
                        }
                        inner.clear();
                    }
                }
                Event::Start(element) | Event::Empty(element) => {
                    return Err(ChartError::UnexpectedElement(element_name(&element)));
                }
                Event::Eof => return Err(ChartError::MissingElement("c:numFmt".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_element(element: &BytesStart<'_>, raw_content: Vec<Vec<u8>>) -> Result<Self> {
        reject_conflicting_prefix(element, b"c", C_NS)?;
        let format_code = attribute_value(element, b"formatCode")?
            .ok_or_else(|| invalid_attribute("numFmt", "formatCode", "<missing>".to_owned()))?;
        let source_linked = attribute_value(element, b"sourceLinked")?
            .ok_or_else(|| invalid_attribute("numFmt", "sourceLinked", "<missing>".to_owned()))?;
        let source_linked = parse_bool_lexical("numFmt", "sourceLinked", &source_linked)?;
        let number_format = Self {
            format_code,
            source_linked,
            raw_attributes: capture_attributes_excluding(
                element,
                &[b"formatCode", b"sourceLinked"],
            )?,
            raw_content,
        };
        number_format.validate()?;
        Ok(number_format)
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        self.validate()?;
        let mut start = BytesStart::new("c:numFmt");
        start.push_attribute(("formatCode", self.format_code.as_str()));
        start.push_attribute(("sourceLinked", bool_lexical(self.source_linked)));
        push_attributes(&mut start, &self.raw_attributes);
        if self.raw_content.is_empty() {
            writer
                .write_event(Event::Empty(start))
                .map_err(OxmlError::from)?;
            return Ok(());
        }
        writer
            .write_event(Event::Start(start))
            .map_err(OxmlError::from)?;
        for raw in &self.raw_content {
            writer.get_mut().write_all(raw).map_err(OxmlError::from)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("c:numFmt")))
            .map_err(OxmlError::from)?;
        Ok(())
    }

    /// Projects one cached value through the supported deterministic subset.
    pub fn format_value(&self, value: f64) -> Result<String> {
        if !value.is_finite() {
            return Err(ChartError::InvalidValue {
                element: "c:numFmt value".to_owned(),
                value: value.to_string(),
            });
        }
        if self.format_code == "General" {
            return Ok(value.to_string());
        }

        let (numeric_code, percentage) = self
            .format_code
            .strip_suffix('%')
            .map(|code| (code, true))
            .unwrap_or((&self.format_code, false));
        let precision = if numeric_code == "0" {
            Some(0)
        } else if let Some(decimals) = numeric_code.strip_prefix("0.")
            && !decimals.is_empty()
            && decimals.bytes().all(|byte| byte == b'0')
        {
            Some(decimals.len())
        } else {
            None
        };
        let Some(precision) = precision else {
            return Err(ChartError::InvalidValue {
                element: "c:numFmt/@formatCode projection".to_owned(),
                value: self.format_code.clone(),
            });
        };
        let projected = if percentage { value * 100.0 } else { value };
        if !projected.is_finite() {
            return Err(ChartError::InvalidValue {
                element: "c:numFmt value".to_owned(),
                value: projected.to_string(),
            });
        }
        let mut formatted = format!("{projected:.precision$}");
        if percentage {
            formatted.push('%');
        }
        Ok(formatted)
    }

    fn validate(&self) -> Result<()> {
        if self.format_code.is_empty() {
            return Err(ChartError::InvalidValue {
                element: "c:numFmt/@formatCode".to_owned(),
                value: self.format_code.clone(),
            });
        }
        validate_xml_text(&self.format_code, "c:numFmt/@formatCode")
    }
}

impl CT_DLbls {
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        Self::from_xml_with_namespaces(xml, &chart_namespace_defaults())
    }

    fn from_xml_with_namespaces(xml: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) if matches_local_name(element.name().as_ref(), b"dLbls") => {
                    chart_root_prefix(&element)?;
                    if !element_is_in_namespace(&element, C_NS, inherited)? {
                        return Err(ChartError::UnexpectedElement(element_name(&element)));
                    }
                    return Self::from_element(&mut reader, &element, inherited);
                }
                Event::Empty(element) if matches_local_name(element.name().as_ref(), b"dLbls") => {
                    chart_root_prefix(&element)?;
                    let namespaces = chart_bindings(inherited, &element)?;
                    if !element_is_in_namespace(&element, C_NS, inherited)? {
                        return Err(ChartError::UnexpectedElement(element_name(&element)));
                    }
                    let (raw_attributes, _) = capture_fixed_root_attributes(
                        &element,
                        &["xmlns:c", "xmlns:a", "xmlns:r"],
                    )?;
                    return Ok(Self {
                        raw_attributes,
                        namespace_declarations: standalone_namespace_declarations(&namespaces)?,
                        ..Self::default()
                    });
                }
                Event::Start(element) | Event::Empty(element) => {
                    return Err(ChartError::UnexpectedElement(element_name(&element)));
                }
                Event::Eof => return Err(ChartError::MissingElement("c:dLbls".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_element(
        reader: &mut Reader<&[u8]>,
        start: &BytesStart<'_>,
        inherited: &NamespaceBindings,
    ) -> Result<Self> {
        reject_conflicting_prefix(start, b"a", A_NS)?;
        let namespaces = chart_bindings(inherited, start)?;
        require_fixed_namespace(&namespaces, b"c", C_NS, start)?;
        let (raw_attributes, _) =
            capture_fixed_root_attributes(start, &["xmlns:c", "xmlns:a", "xmlns:r"])?;
        let namespace_declarations = standalone_namespace_declarations(&namespaces)?;
        let mut state = DataLabelsParseState::new(namespaces);
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) => {
                    let name = chart_child_local(&element, &state.namespaces)?;
                    let raw = capture_element(reader, &element)?;
                    state.parse_child(name.as_deref().unwrap_or_default(), raw)?;
                }
                Event::Empty(element) => {
                    let name = chart_child_local(&element, &state.namespaces)?;
                    let raw = capture_empty_element(&element)?;
                    state.parse_child(name.as_deref().unwrap_or_default(), raw)?;
                }
                event @ (Event::Text(_)
                | Event::CData(_)
                | Event::Comment(_)
                | Event::PI(_)
                | Event::GeneralRef(_)) => state.capture_event(capture_event(event)?),
                Event::End(element) if matches_local_name(element.name().as_ref(), b"dLbls") => {
                    return state.finish(raw_attributes, namespace_declarations);
                }
                Event::Eof => return Err(missing_end("c:dLbls")),
                _ => {}
            }
            buffer.clear();
        }
    }

    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        self.write_xml(&mut writer, true)?;
        Ok(writer.into_inner())
    }

    fn write_xml(&self, writer: &mut Writer<Vec<u8>>, standalone: bool) -> Result<()> {
        self.validate()?;
        let mut start = BytesStart::new("c:dLbls");
        if standalone {
            start.push_attribute(("xmlns:c", C_NS));
            start.push_attribute(("xmlns:a", A_NS));
            start.push_attribute(("xmlns:r", R_NS));
        }
        push_attributes(&mut start, &self.namespace_declarations);
        push_attributes(&mut start, &self.raw_attributes);
        writer
            .write_event(Event::Start(start))
            .map_err(OxmlError::from)?;
        emit_raw(writer, self.raw_children.at(0))?;
        if let Some(number_format) = &self.number_format {
            number_format.write_xml(writer)?;
        }
        emit_raw(writer, self.raw_children.at(1))?;
        if let Some(position) = self.position {
            write_scalar(
                writer,
                "c:dLblPos",
                position.as_str(),
                self.position_markup.as_ref(),
            )?;
        }
        emit_raw(writer, self.raw_children.at(2))?;
        write_optional_bool(
            writer,
            "c:showLegendKey",
            self.show_legend_key,
            self.show_legend_key_markup.as_ref(),
        )?;
        emit_raw(writer, self.raw_children.at(3))?;
        write_optional_bool(
            writer,
            "c:showVal",
            self.show_value,
            self.show_value_markup.as_ref(),
        )?;
        emit_raw(writer, self.raw_children.at(4))?;
        write_optional_bool(
            writer,
            "c:showCatName",
            self.show_category_name,
            self.show_category_name_markup.as_ref(),
        )?;
        emit_raw(writer, self.raw_children.at(5))?;
        write_optional_bool(
            writer,
            "c:showSerName",
            self.show_series_name,
            self.show_series_name_markup.as_ref(),
        )?;
        emit_raw(writer, self.raw_children.at(6))?;
        write_optional_bool(
            writer,
            "c:showPercent",
            self.show_percent,
            self.show_percent_markup.as_ref(),
        )?;
        emit_raw(writer, self.raw_children.at(7))?;
        write_optional_bool(
            writer,
            "c:showBubbleSize",
            self.show_bubble_size,
            self.show_bubble_size_markup.as_ref(),
        )?;
        emit_raw(writer, self.raw_children.at(8))?;
        if let Some(separator) = &self.separator {
            write_text(
                writer,
                "c:separator",
                separator,
                self.separator_markup
                    .as_ref()
                    .unwrap_or(&TextMarkup::default()),
            )?;
        }
        emit_raw(writer, self.raw_children.at(9))?;
        writer
            .write_event(Event::End(BytesEnd::new("c:dLbls")))
            .map_err(OxmlError::from)?;
        Ok(())
    }

    fn validate(&self) -> Result<()> {
        if let Some(number_format) = &self.number_format {
            number_format.validate()?;
        }
        if let Some(separator) = &self.separator {
            validate_xml_text(separator, "c:separator")?;
        }
        Ok(())
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }
}

struct DataLabelsParseState {
    namespaces: NamespaceBindings,
    number_format: Option<NumberFormat>,
    position: Option<(DataLabelPosition, ScalarMarkup)>,
    show_legend_key: Option<(bool, ScalarMarkup)>,
    show_value: Option<(bool, ScalarMarkup)>,
    show_category_name: Option<(bool, ScalarMarkup)>,
    show_series_name: Option<(bool, ScalarMarkup)>,
    show_percent: Option<(bool, ScalarMarkup)>,
    show_bubble_size: Option<(bool, ScalarMarkup)>,
    separator: Option<(String, TextMarkup)>,
    raw_children: OrderedRawChildren,
    boundary: usize,
}

impl DataLabelsParseState {
    fn new(namespaces: NamespaceBindings) -> Self {
        Self {
            namespaces,
            number_format: None,
            position: None,
            show_legend_key: None,
            show_value: None,
            show_category_name: None,
            show_series_name: None,
            show_percent: None,
            show_bubble_size: None,
            separator: None,
            raw_children: OrderedRawChildren::default(),
            boundary: 0,
        }
    }

    fn capture_event(&mut self, raw: Vec<u8>) {
        self.raw_children.push(self.boundary, raw);
    }

    fn parse_child(&mut self, name: &[u8], raw: Vec<u8>) -> Result<()> {
        match name {
            b"numFmt" => {
                set_once(
                    &mut self.number_format,
                    NumberFormat::from_xml(&raw)?,
                    "c:numFmt",
                )?;
                self.boundary = self.boundary.max(1);
            }
            b"dLblPos" => {
                let (value, markup) = scalar_value(&raw, "dLblPos")?;
                let lexical = value
                    .ok_or_else(|| invalid_attribute("dLblPos", "val", "<missing>".to_owned()))?;
                let position = DataLabelPosition::parse(&lexical)
                    .ok_or_else(|| invalid_attribute("dLblPos", "val", lexical))?;
                set_once(&mut self.position, (position, markup), "c:dLblPos")?;
                self.boundary = self.boundary.max(2);
            }
            b"showLegendKey" => {
                set_once(
                    &mut self.show_legend_key,
                    parse_bool_value(&raw, "showLegendKey")?,
                    "c:showLegendKey",
                )?;
                self.boundary = self.boundary.max(3);
            }
            b"showVal" => {
                set_once(
                    &mut self.show_value,
                    parse_bool_value(&raw, "showVal")?,
                    "c:showVal",
                )?;
                self.boundary = self.boundary.max(4);
            }
            b"showCatName" => {
                set_once(
                    &mut self.show_category_name,
                    parse_bool_value(&raw, "showCatName")?,
                    "c:showCatName",
                )?;
                self.boundary = self.boundary.max(5);
            }
            b"showSerName" => {
                set_once(
                    &mut self.show_series_name,
                    parse_bool_value(&raw, "showSerName")?,
                    "c:showSerName",
                )?;
                self.boundary = self.boundary.max(6);
            }
            b"showPercent" => {
                set_once(
                    &mut self.show_percent,
                    parse_bool_value(&raw, "showPercent")?,
                    "c:showPercent",
                )?;
                self.boundary = self.boundary.max(7);
            }
            b"showBubbleSize" => {
                set_once(
                    &mut self.show_bubble_size,
                    parse_bool_value(&raw, "showBubbleSize")?,
                    "c:showBubbleSize",
                )?;
                self.boundary = self.boundary.max(8);
            }
            b"separator" => {
                set_once(
                    &mut self.separator,
                    parse_text_element(&raw, b"separator", &self.namespaces)?,
                    "c:separator",
                )?;
                self.boundary = self.boundary.max(9);
            }
            _ => {
                let boundary = data_labels_raw_boundary(name, self.boundary);
                self.raw_children.push(boundary, raw);
                self.boundary = self.boundary.max(boundary);
            }
        }
        Ok(())
    }

    fn finish(
        self,
        raw_attributes: XmlAttributes,
        namespace_declarations: XmlAttributes,
    ) -> Result<CT_DLbls> {
        let (position, position_markup) = optional_markup(self.position);
        let (show_legend_key, show_legend_key_markup) = optional_bool_markup(self.show_legend_key);
        let (show_value, show_value_markup) = optional_bool_markup(self.show_value);
        let (show_category_name, show_category_name_markup) =
            optional_bool_markup(self.show_category_name);
        let (show_series_name, show_series_name_markup) =
            optional_bool_markup(self.show_series_name);
        let (show_percent, show_percent_markup) = optional_bool_markup(self.show_percent);
        let (show_bubble_size, show_bubble_size_markup) =
            optional_bool_markup(self.show_bubble_size);
        let (separator, separator_markup) = self
            .separator
            .map(|(value, markup)| (Some(value), Some(markup)))
            .unwrap_or((None, None));
        let labels = CT_DLbls {
            number_format: self.number_format,
            position,
            separator,
            show_legend_key,
            show_value,
            show_category_name,
            show_series_name,
            show_percent,
            show_bubble_size,
            position_markup,
            show_legend_key_markup,
            show_value_markup,
            show_category_name_markup,
            show_series_name_markup,
            show_percent_markup,
            show_bubble_size_markup,
            separator_markup,
            raw_attributes,
            namespace_declarations,
            raw_children: raw_children_in_schema_order(&self.raw_children, 9),
        };
        labels.validate()?;
        Ok(labels)
    }
}

fn data_labels_raw_boundary(name: &[u8], current: usize) -> usize {
    match name {
        b"dLbl" | b"delete" => 0,
        b"spPr" | b"txPr" => 1,
        b"showLeaderLines" | b"leaderLines" | b"extLst" => 9,
        _ => current,
    }
}

fn optional_bool_markup(value: Option<(bool, ScalarMarkup)>) -> (bool, Option<ScalarMarkup>) {
    value
        .map(|(value, markup)| (value, Some(markup)))
        .unwrap_or((false, None))
}

fn write_optional_bool(
    writer: &mut Writer<Vec<u8>>,
    tag: &str,
    value: bool,
    markup: Option<&ScalarMarkup>,
) -> Result<()> {
    if value || markup.is_some() {
        write_scalar(writer, tag, bool_lexical(value), markup)?;
    }
    Ok(())
}

fn default_axis_id_markup(id: AxisId) -> AxisIdMarkup {
    AxisIdMarkup {
        scalar: ScalarMarkup::default(),
        parsed: id,
        lexical: id.value().to_string(),
    }
}

fn parse_axis_id_scalar(xml: &[u8], local: &str) -> Result<(AxisId, AxisIdMarkup)> {
    let (value, scalar) = scalar_value(xml, local)?;
    let lexical = value.ok_or_else(|| invalid_attribute(local, "val", "<missing>".to_owned()))?;
    let parsed = lexical
        .parse::<i64>()
        .map_err(|_| invalid_attribute(local, "val", lexical.clone()))?;
    let id = AxisId::new(parsed).map_err(|_| invalid_attribute(local, "val", lexical.clone()))?;
    Ok((
        id,
        AxisIdMarkup {
            scalar,
            parsed: id,
            lexical,
        },
    ))
}

fn write_axis_id<W: Write>(
    writer: &mut Writer<W>,
    tag: &str,
    id: AxisId,
    markup: &AxisIdMarkup,
) -> Result<()> {
    AxisId::new(id.value())?;
    let normalized;
    let value = if id == markup.parsed {
        markup.lexical.as_str()
    } else {
        normalized = id.value().to_string();
        normalized.as_str()
    };
    write_scalar(writer, tag, value, Some(&markup.scalar))
}

fn parse_f64_scalar(xml: &[u8], local: &str) -> Result<(f64, ScalarMarkup)> {
    let (value, markup) = scalar_value(xml, local)?;
    let value = value.ok_or_else(|| invalid_attribute(local, "val", "<missing>".to_owned()))?;
    let parsed = value
        .parse::<f64>()
        .map_err(|_| invalid_attribute(local, "val", value.clone()))?;
    if !parsed.is_finite() {
        return Err(invalid_attribute(local, "val", value));
    }
    Ok((parsed, markup))
}

fn parse_axis_position(xml: &[u8]) -> Result<(AxisPosition, ScalarMarkup)> {
    let (value, markup) = scalar_value(xml, "axPos")?;
    let value = value.ok_or_else(|| invalid_attribute("axPos", "val", "<missing>".to_owned()))?;
    let parsed =
        AxisPosition::parse(&value).ok_or_else(|| invalid_attribute("axPos", "val", value))?;
    Ok((parsed, markup))
}

fn parse_tick_mark(xml: &[u8], local: &str) -> Result<(TickMark, ScalarMarkup)> {
    let (value, markup) = scalar_value(xml, local)?;
    let value = value.ok_or_else(|| invalid_attribute(local, "val", "<missing>".to_owned()))?;
    let parsed = TickMark::parse(&value).ok_or_else(|| invalid_attribute(local, "val", value))?;
    Ok((parsed, markup))
}

fn parse_tick_label_position(xml: &[u8]) -> Result<(TickLabelPosition, ScalarMarkup)> {
    let (value, markup) = scalar_value(xml, "tickLblPos")?;
    let value =
        value.ok_or_else(|| invalid_attribute("tickLblPos", "val", "<missing>".to_owned()))?;
    let parsed = TickLabelPosition::parse(&value)
        .ok_or_else(|| invalid_attribute("tickLblPos", "val", value))?;
    Ok((parsed, markup))
}

fn optional_markup<T>(value: Option<(T, ScalarMarkup)>) -> (Option<T>, Option<ScalarMarkup>) {
    value
        .map(|(value, markup)| (Some(value), Some(markup)))
        .unwrap_or((None, None))
}

fn optional_scalar_markup_eq(left: &Option<ScalarMarkup>, right: &Option<ScalarMarkup>) -> bool {
    let empty = ScalarMarkup::default();
    left.as_ref().unwrap_or(&empty) == right.as_ref().unwrap_or(&empty)
}

fn axis_id_markup_eq(left: &AxisIdMarkup, right: &AxisIdMarkup) -> bool {
    left.scalar == right.scalar
}

const fn bool_lexical(value: bool) -> &'static str {
    if value { "1" } else { "0" }
}

fn parse_bool_lexical(element: &str, attribute: &str, value: &str) -> Result<bool> {
    match value {
        "1" | "true" => Ok(true),
        "0" | "false" => Ok(false),
        _ => Err(invalid_attribute(element, attribute, value.to_owned())),
    }
}

impl CT_Title {
    fn from_xml(xml: &[u8]) -> Result<Self> {
        let (raw_attributes, raw_children) = parse_raw_shell(xml, b"title", "c:title")?;
        Ok(Self {
            raw_attributes,
            raw_children,
        })
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        write_raw_shell(writer, "c:title", &self.raw_attributes, &self.raw_children)
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }
}

impl CT_PlotArea {
    fn from_xml_with_namespaces(xml: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let (raw_attributes, captured_children) = parse_raw_shell(xml, b"plotArea", "c:plotArea")?;
        let namespace_bindings = root_chart_bindings(xml, b"plotArea", inherited)?;
        let direct_roots: Vec<_> = captured_children
            .at(0)
            .filter_map(|raw| chart_root_local(raw, &namespace_bindings).transpose())
            .collect::<Result<_>>()?;
        let plot_roots: Vec<_> = direct_roots
            .iter()
            .filter(|local| is_plot_container(local))
            .cloned()
            .collect();
        let typed_local = (plot_roots.len() == 1
            && matches!(
                plot_roots[0].as_slice(),
                b"barChart"
                    | b"lineChart"
                    | b"pieChart"
                    | b"doughnutChart"
                    | b"areaChart"
                    | b"scatterChart"
                    | b"radarChart"
            ))
        .then(|| plot_roots[0].clone());

        let Some(typed_local) = typed_local else {
            return Ok(Self {
                raw_attributes,
                raw_children: captured_children,
                namespace_bindings,
                plots: None,
                plot_markup: Vec::new(),
                axes: Vec::new(),
            });
        };

        let mut raw_children = OrderedRawChildren::default();
        let mut plots = Vec::new();
        let mut plot_markup = Vec::new();
        let mut axes = Vec::new();
        let mut boundary = 0usize;
        for raw in captured_children.at(0) {
            let local = chart_root_local(raw, &namespace_bindings)?;
            if local.as_deref() == Some(typed_local.as_slice()) {
                let (plot, markup) = parse_plot(raw, &namespace_bindings)?;
                plots.push(plot);
                plot_markup.push(markup);
                boundary = 1;
            } else if local.as_deref().and_then(AxisKind::parse).is_some() {
                axes.push(Axis::from_xml_with_namespaces(raw, &namespace_bindings)?);
                boundary = axes.len() + 1;
            } else {
                raw_children.push(boundary, raw.to_vec());
            }
        }
        let plot_area = Self {
            raw_attributes,
            raw_children,
            namespace_bindings,
            plots: Some(plots),
            plot_markup,
            axes,
        };
        plot_area.validate_typed()?;
        Ok(plot_area)
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let Some(plots) = &self.plots else {
            return write_raw_shell(
                writer,
                "c:plotArea",
                &self.raw_attributes,
                &self.raw_children,
            );
        };
        self.validate_typed()?;
        let mut start = BytesStart::new("c:plotArea");
        push_attributes(&mut start, &self.raw_attributes);
        writer
            .write_event(Event::Start(start))
            .map_err(OxmlError::from)?;
        emit_raw(writer, self.raw_children.at(0))?;
        for (index, plot) in plots.iter().enumerate() {
            write_plot(
                writer,
                plot,
                self.plot_markup
                    .get(index)
                    .unwrap_or(&PlotMarkup::default()),
            )?;
        }
        emit_raw(writer, self.raw_children.at(1))?;
        for (index, axis) in self.axes.iter().enumerate() {
            writer
                .get_mut()
                .write_all(&axis.to_xml()?)
                .map_err(OxmlError::from)?;
            emit_raw(writer, self.raw_children.at(index + 2))?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("c:plotArea")))
            .map_err(OxmlError::from)?;
        Ok(())
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }

    /// Creates a supported single-family plot area with owned axes.
    pub fn new(plots: Vec<Plot>, axes: Vec<Axis>) -> Result<Self> {
        let plot_area = Self {
            raw_attributes: Vec::new(),
            raw_children: OrderedRawChildren::default(),
            namespace_bindings: chart_namespace_defaults(),
            plot_markup: vec![PlotMarkup::default(); plots.len()],
            plots: Some(plots),
            axes,
        };
        plot_area.validate_typed()?;
        Ok(plot_area)
    }

    /// Returns the owned supported plots, or an error for an opaque choice.
    pub fn plots(&self) -> Result<&[Plot]> {
        self.plots
            .as_deref()
            .ok_or_else(|| ChartError::InvalidValue {
                element: "c:plotArea".to_owned(),
                value: "unsupported or combination plot area is opaque".to_owned(),
            })
    }

    /// Returns mutable supported plots without exposing opaque content to edits.
    pub fn plots_mut(&mut self) -> Result<&mut [Plot]> {
        self.plots
            .as_deref_mut()
            .ok_or_else(|| ChartError::InvalidValue {
                element: "c:plotArea".to_owned(),
                value: "cannot combine typed plots with an opaque plot choice".to_owned(),
            })
    }

    /// Parses the common series payloads nested in category-based plot shells.
    pub fn series(&self) -> Result<Vec<Series>> {
        if let Some(plots) = &self.plots {
            return Ok(plots
                .iter()
                .flat_map(|plot| match plot {
                    Plot::Bar { series, .. }
                    | Plot::Line { series, .. }
                    | Plot::Pie { series, .. }
                    | Plot::Doughnut { series, .. }
                    | Plot::Area { series, .. }
                    | Plot::Scatter { series, .. }
                    | Plot::Radar { series, .. } => series.clone(),
                })
                .collect());
        }
        let mut series = Vec::new();
        for raw in self.raw_children.at(0) {
            parse_plot_series(raw, &self.namespace_bindings, &mut series)?;
        }
        Ok(series)
    }

    /// Parses and validates the direct axis children of this plot area.
    pub fn axes(&self) -> Result<Vec<Axis>> {
        if self.plots.is_some() {
            self.validate_typed()?;
            return Ok(self.axes.clone());
        }
        let mut axes = Vec::new();
        for raw in self.raw_children.at(0) {
            if let Some(axis) = parse_plot_axis(raw, &self.namespace_bindings)? {
                axes.push(axis);
            }
        }
        validate_axis_pairs(&axes)?;
        Ok(axes)
    }

    fn validate_typed(&self) -> Result<()> {
        let plots = self
            .plots
            .as_ref()
            .ok_or_else(|| ChartError::InvalidValue {
                element: "c:plotArea".to_owned(),
                value: "opaque plot area has no typed validation view".to_owned(),
            })?;
        if plots.len() != 1 {
            return Err(ChartError::InvalidValue {
                element: "c:plotArea".to_owned(),
                value: "typed plot area requires exactly one plot family".to_owned(),
            });
        }
        validate_axis_pairs(&self.axes)?;
        for plot in plots {
            plot.validate()?;
            if let Some(axis_ids) = plot.axis_ids() {
                for id in axis_ids {
                    if !self.axes.iter().any(|axis| axis.id == id) {
                        return Err(ChartError::InvalidValue {
                            element: "c:axId".to_owned(),
                            value: format!("plot references missing axis {}", id.value()),
                        });
                    }
                }
            } else if !self.axes.is_empty() {
                return Err(ChartError::InvalidValue {
                    element: "c:plotArea".to_owned(),
                    value: "pie and doughnut plot areas must not own axes".to_owned(),
                });
            }
            if plot.axis_ids().is_some() && self.axes.len() != 2 {
                return Err(ChartError::InvalidValue {
                    element: "c:plotArea".to_owned(),
                    value: format!(
                        "axis-owned plot requires exactly 2 axes, found {}",
                        self.axes.len()
                    ),
                });
            }
        }
        Ok(())
    }
}

impl Default for CT_PlotArea {
    fn default() -> Self {
        Self {
            raw_attributes: Vec::new(),
            raw_children: OrderedRawChildren::default(),
            namespace_bindings: chart_namespace_defaults(),
            plots: Some(Vec::new()),
            plot_markup: Vec::new(),
            axes: Vec::new(),
        }
    }
}

fn chart_root_local(xml: &[u8], inherited: &NamespaceBindings) -> Result<Option<Vec<u8>>> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) | Event::Empty(element) => {
                return Ok(element_is_in_namespace(&element, C_NS, inherited)?
                    .then(|| local_name(element.name().as_ref()).to_vec()));
            }
            Event::Eof => return Ok(None),
            _ => {}
        }
        buffer.clear();
    }
}

fn is_plot_container(local: &[u8]) -> bool {
    matches!(
        local,
        b"areaChart"
            | b"area3DChart"
            | b"barChart"
            | b"bar3DChart"
            | b"bubbleChart"
            | b"doughnutChart"
            | b"lineChart"
            | b"line3DChart"
            | b"ofPieChart"
            | b"pieChart"
            | b"pie3DChart"
            | b"radarChart"
            | b"scatterChart"
            | b"stockChart"
            | b"surfaceChart"
            | b"surface3DChart"
    )
}

fn parse_plot(xml: &[u8], inherited: &NamespaceBindings) -> Result<(Plot, PlotMarkup)> {
    let local = chart_root_local(xml, inherited)?
        .ok_or_else(|| ChartError::MissingElement("c:barChart or c:lineChart".to_owned()))?;
    match local.as_slice() {
        b"barChart" => parse_bar_plot(xml, inherited),
        b"lineChart" => parse_line_plot(xml, inherited),
        b"pieChart" | b"doughnutChart" | b"areaChart" | b"scatterChart" | b"radarChart" => {
            parse_remaining_plot(xml, inherited, &local)
        }
        _ => Err(ChartError::UnexpectedElement(format!(
            "c:{}",
            String::from_utf8_lossy(&local)
        ))),
    }
}

fn parse_remaining_plot(
    xml: &[u8],
    inherited: &NamespaceBindings,
    local: &[u8],
) -> Result<(Plot, PlotMarkup)> {
    let root = String::from_utf8_lossy(local).into_owned();
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) if matches_local_name(element.name().as_ref(), local) => {
                let namespaces = typed_rewrite_bindings(&element, inherited)?;
                let (mut raw_attributes, namespace_declarations) =
                    capture_fixed_root_attributes(&element, &["xmlns:c", "xmlns:a", "xmlns:r"])?;
                raw_attributes.splice(0..0, namespace_declarations);
                let mut grouping = None;
                let mut style = None;
                let mut first_slice_angle = None;
                let mut hole_size = None;
                let mut series = Vec::new();
                let mut data_labels = None;
                let mut axis_ids = Vec::new();
                let mut raw_children = OrderedRawChildren::default();
                let mut boundary = 0usize;
                let mut inner = Vec::new();
                loop {
                    match reader
                        .read_event_into(&mut inner)
                        .map_err(OxmlError::from)?
                    {
                        Event::Start(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_element(&mut reader, &child)?;
                            parse_remaining_plot_child(
                                local,
                                name.as_deref().unwrap_or_default(),
                                raw,
                                &namespaces,
                                &mut grouping,
                                &mut style,
                                &mut first_slice_angle,
                                &mut hole_size,
                                &mut series,
                                &mut data_labels,
                                &mut axis_ids,
                                &mut raw_children,
                                &mut boundary,
                            )?;
                        }
                        Event::Empty(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_empty_element(&child)?;
                            parse_remaining_plot_child(
                                local,
                                name.as_deref().unwrap_or_default(),
                                raw,
                                &namespaces,
                                &mut grouping,
                                &mut style,
                                &mut first_slice_angle,
                                &mut hole_size,
                                &mut series,
                                &mut data_labels,
                                &mut axis_ids,
                                &mut raw_children,
                                &mut boundary,
                            )?;
                        }
                        event @ (Event::Text(_)
                        | Event::CData(_)
                        | Event::Comment(_)
                        | Event::PI(_)
                        | Event::GeneralRef(_)) => {
                            raw_children.push(boundary, capture_event(event)?);
                        }
                        Event::End(end) if matches_local_name(end.name().as_ref(), local) => {
                            let needs_axes =
                                matches!(local, b"areaChart" | b"scatterChart" | b"radarChart");
                            if needs_axes && axis_ids.len() != 2 {
                                return Err(ChartError::InvalidValue {
                                    element: format!("c:{root}/c:axId"),
                                    value: format!("expected 2, found {}", axis_ids.len()),
                                });
                            }
                            if !needs_axes && !axis_ids.is_empty() {
                                return Err(ChartError::InvalidValue {
                                    element: format!("c:{root}/c:axId"),
                                    value: "axis-free plot contains axis references".to_owned(),
                                });
                            }
                            let parsed_ids = needs_axes.then(|| [axis_ids[0].0, axis_ids[1].0]);
                            let original_series_keys = series
                                .iter()
                                .map(|item| (item.index, item.order))
                                .collect::<Vec<_>>();
                            let plot = match local {
                                b"pieChart" => Plot::Pie {
                                    first_slice_angle: first_slice_angle
                                        .as_ref()
                                        .map_or(0, |item| item.0),
                                    series,
                                    data_labels,
                                },
                                b"doughnutChart" => Plot::Doughnut {
                                    first_slice_angle: first_slice_angle
                                        .as_ref()
                                        .map_or(0, |item| item.0),
                                    hole_size: hole_size.as_ref().map_or(50, |item| item.0),
                                    series,
                                    data_labels,
                                },
                                b"areaChart" => Plot::Area {
                                    grouping: grouping
                                        .as_ref()
                                        .ok_or_else(|| {
                                            ChartError::MissingElement("c:grouping".to_owned())
                                        })?
                                        .0,
                                    series,
                                    data_labels,
                                    axis_ids: parsed_ids.ok_or_else(|| {
                                        ChartError::MissingElement("c:areaChart/c:axId".to_owned())
                                    })?,
                                },
                                b"scatterChart" => {
                                    let lexical = style
                                        .as_ref()
                                        .ok_or_else(|| {
                                            ChartError::MissingElement("c:scatterStyle".to_owned())
                                        })?
                                        .0
                                        .as_str();
                                    Plot::Scatter {
                                        style: ScatterStyle::parse(lexical).ok_or_else(|| {
                                            invalid_attribute(
                                                "scatterStyle",
                                                "val",
                                                lexical.to_owned(),
                                            )
                                        })?,
                                        series,
                                        data_labels,
                                        axis_ids: parsed_ids.ok_or_else(|| {
                                            ChartError::MissingElement(
                                                "c:scatterChart/c:axId".to_owned(),
                                            )
                                        })?,
                                    }
                                }
                                b"radarChart" => {
                                    let lexical = style
                                        .as_ref()
                                        .ok_or_else(|| {
                                            ChartError::MissingElement("c:radarStyle".to_owned())
                                        })?
                                        .0
                                        .as_str();
                                    Plot::Radar {
                                        style: RadarStyle::parse(lexical).ok_or_else(|| {
                                            invalid_attribute(
                                                "radarStyle",
                                                "val",
                                                lexical.to_owned(),
                                            )
                                        })?,
                                        series,
                                        data_labels,
                                        axis_ids: parsed_ids.ok_or_else(|| {
                                            ChartError::MissingElement(
                                                "c:radarChart/c:axId".to_owned(),
                                            )
                                        })?,
                                    }
                                }
                                _ => {
                                    return Err(ChartError::UnexpectedElement(format!(
                                        "c:{}",
                                        String::from_utf8_lossy(local)
                                    )));
                                }
                            };
                            plot.validate()?;
                            let final_boundary = boundary.max(original_series_keys.len() + 6);
                            return Ok((
                                plot,
                                PlotMarkup {
                                    raw_attributes,
                                    raw_children: raw_children_in_schema_order(
                                        &raw_children,
                                        final_boundary,
                                    ),
                                    grouping: grouping.map(|item| item.1),
                                    first_slice_angle: first_slice_angle.map(|item| item.1),
                                    hole_size: hole_size.map(|item| item.1),
                                    style: style.map(|item| item.1),
                                    axis_ids: axis_ids.into_iter().map(|item| item.1).collect(),
                                    original_series_keys,
                                    original_axis_ids: parsed_ids
                                        .map_or_else(Vec::new, |ids| ids.to_vec()),
                                    parsed_remaining: match local {
                                        b"pieChart" => Some("pieChart"),
                                        b"doughnutChart" => Some("doughnutChart"),
                                        b"areaChart" => Some("areaChart"),
                                        b"scatterChart" => Some("scatterChart"),
                                        b"radarChart" => Some("radarChart"),
                                        _ => None,
                                    },
                                    ..PlotMarkup::default()
                                },
                            ));
                        }
                        Event::Eof => return Err(missing_end(&format!("c:{root}"))),
                        _ => {}
                    }
                    inner.clear();
                }
            }
            Event::Empty(element) if matches_local_name(element.name().as_ref(), local) => {
                return Err(ChartError::MissingElement("c:ser".to_owned()));
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => return Err(ChartError::MissingElement(format!("c:{root}"))),
            _ => {}
        }
        buffer.clear();
    }
}

#[allow(clippy::too_many_arguments)]
fn parse_remaining_plot_child(
    root: &[u8],
    name: &[u8],
    raw: Vec<u8>,
    namespaces: &NamespaceBindings,
    grouping: &mut Option<(Grouping, ScalarMarkup)>,
    style: &mut Option<(String, ScalarMarkup)>,
    first_slice_angle: &mut Option<(u16, ScalarMarkup)>,
    hole_size: &mut Option<(u8, ScalarMarkup)>,
    series: &mut Vec<Series>,
    data_labels: &mut Option<CT_DLbls>,
    axis_ids: &mut Vec<(AxisId, AxisIdMarkup)>,
    raw_children: &mut OrderedRawChildren,
    boundary: &mut usize,
) -> Result<()> {
    let property_first = matches!(root, b"areaChart" | b"scatterChart" | b"radarChart");
    let base = usize::from(property_first);
    match name {
        b"grouping" if root == b"areaChart" => {
            let (value, markup) = required_scalar(&raw, "grouping")?;
            let parsed = Grouping::parse(&value)
                .ok_or_else(|| invalid_attribute("grouping", "val", value))?;
            set_once(grouping, (parsed, markup), "c:grouping")?;
            *boundary = (*boundary).max(1);
        }
        b"scatterStyle" if root == b"scatterChart" => {
            let (value, markup) = required_scalar(&raw, "scatterStyle")?;
            if ScatterStyle::parse(&value).is_none() {
                return Err(invalid_attribute("scatterStyle", "val", value));
            }
            set_once(style, (value, markup), "c:scatterStyle")?;
            *boundary = (*boundary).max(1);
        }
        b"radarStyle" if root == b"radarChart" => {
            let (value, markup) = required_scalar(&raw, "radarStyle")?;
            if RadarStyle::parse(&value).is_none() {
                return Err(invalid_attribute("radarStyle", "val", value));
            }
            set_once(style, (value, markup), "c:radarStyle")?;
            *boundary = (*boundary).max(1);
        }
        b"ser" => {
            series.push(Series::from_xml_with_namespaces(&raw, namespaces)?);
            *boundary = (*boundary).max(base + series.len());
        }
        b"dLbls" => {
            set_once(
                data_labels,
                CT_DLbls::from_xml_with_namespaces(&raw, namespaces)?,
                "c:dLbls",
            )?;
            *boundary = (*boundary).max(base + series.len() + 1);
        }
        b"firstSliceAng" if matches!(root, b"pieChart" | b"doughnutChart") => {
            let (value, markup) = required_scalar(&raw, "firstSliceAng")?;
            let parsed = value
                .parse::<u16>()
                .map_err(|_| invalid_attribute("firstSliceAng", "val", value.clone()))?;
            if parsed > 360 {
                return Err(invalid_attribute("firstSliceAng", "val", value));
            }
            set_once(first_slice_angle, (parsed, markup), "c:firstSliceAng")?;
            *boundary = (*boundary).max(series.len() + 2);
        }
        b"holeSize" if root == b"doughnutChart" => {
            let (value, markup) = required_scalar(&raw, "holeSize")?;
            let parsed = value
                .parse::<u8>()
                .map_err(|_| invalid_attribute("holeSize", "val", value.clone()))?;
            if !(10..=90).contains(&parsed) {
                return Err(invalid_attribute("holeSize", "val", value));
            }
            set_once(hole_size, (parsed, markup), "c:holeSize")?;
            *boundary = (*boundary).max(series.len() + 3);
        }
        b"axId" if property_first => {
            if axis_ids.len() == 2 {
                return Err(ChartError::DuplicateElement("c:axId".to_owned()));
            }
            axis_ids.push(parse_axis_id_scalar(&raw, "axId")?);
            *boundary = (*boundary).max(base + series.len() + 1 + axis_ids.len());
        }
        b"axId" => {
            return Err(ChartError::InvalidValue {
                element: "c:axId".to_owned(),
                value: "axis-free plot contains axis reference".to_owned(),
            });
        }
        _ => {
            let raw_boundary = match (root, name) {
                (_, b"varyColors") => base,
                (b"pieChart", b"extLst") => series.len() + 2,
                (b"doughnutChart", b"extLst") => series.len() + 3,
                (b"areaChart" | b"scatterChart" | b"radarChart", b"dropLines") => {
                    base + series.len() + 1
                }
                (b"areaChart" | b"scatterChart" | b"radarChart", b"extLst") => {
                    base + series.len() + 3
                }
                _ => *boundary,
            };
            raw_children.push(raw_boundary, raw);
            *boundary = (*boundary).max(raw_boundary);
        }
    }
    Ok(())
}

fn parse_bar_plot(xml: &[u8], inherited: &NamespaceBindings) -> Result<(Plot, PlotMarkup)> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) if matches_local_name(element.name().as_ref(), b"barChart") => {
                let namespaces = typed_rewrite_bindings(&element, inherited)?;
                let (mut raw_attributes, namespace_declarations) =
                    capture_fixed_root_attributes(&element, &["xmlns:c", "xmlns:a", "xmlns:r"])?;
                raw_attributes.splice(0..0, namespace_declarations);
                let mut direction = None;
                let mut grouping = None;
                let mut gap_width = None;
                let mut overlap = None;
                let mut series = Vec::new();
                let mut data_labels = None;
                let mut axis_ids = Vec::new();
                let mut raw_children = OrderedRawChildren::default();
                let mut boundary = 0usize;
                let mut inner = Vec::new();
                loop {
                    match reader
                        .read_event_into(&mut inner)
                        .map_err(OxmlError::from)?
                    {
                        Event::Start(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_element(&mut reader, &child)?;
                            parse_bar_child(
                                name.as_deref().unwrap_or_default(),
                                raw,
                                &namespaces,
                                &mut direction,
                                &mut grouping,
                                &mut gap_width,
                                &mut overlap,
                                &mut series,
                                &mut data_labels,
                                &mut axis_ids,
                                &mut raw_children,
                                &mut boundary,
                            )?;
                        }
                        Event::Empty(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_empty_element(&child)?;
                            parse_bar_child(
                                name.as_deref().unwrap_or_default(),
                                raw,
                                &namespaces,
                                &mut direction,
                                &mut grouping,
                                &mut gap_width,
                                &mut overlap,
                                &mut series,
                                &mut data_labels,
                                &mut axis_ids,
                                &mut raw_children,
                                &mut boundary,
                            )?;
                        }
                        event @ (Event::Text(_)
                        | Event::CData(_)
                        | Event::Comment(_)
                        | Event::PI(_)
                        | Event::GeneralRef(_)) => {
                            raw_children.push(boundary, capture_event(event)?);
                        }
                        Event::End(end) if matches_local_name(end.name().as_ref(), b"barChart") => {
                            let (direction, direction_markup) = direction
                                .ok_or_else(|| ChartError::MissingElement("c:barDir".to_owned()))?;
                            let (grouping, grouping_markup) = grouping.ok_or_else(|| {
                                ChartError::MissingElement("c:grouping".to_owned())
                            })?;
                            if axis_ids.len() != 2 {
                                return Err(ChartError::InvalidValue {
                                    element: "c:barChart/c:axId".to_owned(),
                                    value: format!("expected 2, found {}", axis_ids.len()),
                                });
                            }
                            let parsed_ids = [axis_ids[0].0, axis_ids[1].0];
                            let original_series_keys: Vec<(u32, u32)> =
                                series.iter().map(|item| (item.index, item.order)).collect();
                            let plot = Plot::Bar {
                                direction,
                                grouping,
                                gap_width: gap_width.as_ref().map_or(150, |item| item.0),
                                overlap: overlap.as_ref().map_or(0, |item| item.0),
                                series,
                                data_labels,
                                axis_ids: parsed_ids,
                            };
                            plot.validate()?;
                            return Ok((
                                plot,
                                PlotMarkup {
                                    raw_attributes,
                                    raw_children: raw_children_in_schema_order(
                                        &raw_children,
                                        original_series_keys.len() + 7,
                                    ),
                                    direction: Some(direction_markup),
                                    grouping: Some(grouping_markup),
                                    gap_width: gap_width.map(|item| item.1),
                                    overlap: overlap.map(|item| item.1),
                                    marker: None,
                                    smooth: None,
                                    first_slice_angle: None,
                                    hole_size: None,
                                    style: None,
                                    axis_ids: axis_ids.into_iter().map(|item| item.1).collect(),
                                    original_series_keys,
                                    original_axis_ids: parsed_ids.to_vec(),
                                    parsed_bar: Some(true),
                                    parsed_remaining: None,
                                },
                            ));
                        }
                        Event::Eof => return Err(missing_end("c:barChart")),
                        _ => {}
                    }
                    inner.clear();
                }
            }
            Event::Empty(element) if matches_local_name(element.name().as_ref(), b"barChart") => {
                return Err(ChartError::MissingElement("c:barDir".to_owned()));
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => return Err(ChartError::MissingElement("c:barChart".to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

#[allow(clippy::too_many_arguments)]
fn parse_bar_child(
    name: &[u8],
    raw: Vec<u8>,
    namespaces: &NamespaceBindings,
    direction: &mut Option<(BarDirection, ScalarMarkup)>,
    grouping: &mut Option<(BarGrouping, ScalarMarkup)>,
    gap_width: &mut Option<(u16, ScalarMarkup)>,
    overlap: &mut Option<(i8, ScalarMarkup)>,
    series: &mut Vec<Series>,
    data_labels: &mut Option<CT_DLbls>,
    axis_ids: &mut Vec<(AxisId, AxisIdMarkup)>,
    raw_children: &mut OrderedRawChildren,
    boundary: &mut usize,
) -> Result<()> {
    match name {
        b"barDir" => {
            let (value, markup) = required_scalar(&raw, "barDir")?;
            let value = BarDirection::parse(&value)
                .ok_or_else(|| invalid_attribute("barDir", "val", value))?;
            set_once(direction, (value, markup), "c:barDir")?;
            *boundary = (*boundary).max(1);
        }
        b"grouping" => {
            let (value, markup) = required_scalar(&raw, "grouping")?;
            let value = BarGrouping::parse(&value)
                .ok_or_else(|| invalid_attribute("grouping", "val", value))?;
            set_once(grouping, (value, markup), "c:grouping")?;
            *boundary = (*boundary).max(2);
        }
        b"ser" => {
            series.push(Series::from_xml_with_namespaces(&raw, namespaces)?);
            *boundary = (*boundary).max(series.len() + 2);
        }
        b"dLbls" => {
            set_once(
                data_labels,
                CT_DLbls::from_xml_with_namespaces(&raw, namespaces)?,
                "c:dLbls",
            )?;
            *boundary = (*boundary).max(series.len() + 3);
        }
        b"gapWidth" => {
            let (value, markup) = required_scalar(&raw, "gapWidth")?;
            let parsed = value
                .parse::<u16>()
                .map_err(|_| invalid_attribute("gapWidth", "val", value.clone()))?;
            if parsed > 500 {
                return Err(invalid_attribute("gapWidth", "val", value));
            }
            set_once(gap_width, (parsed, markup), "c:gapWidth")?;
            *boundary = (*boundary).max(series.len() + 4);
        }
        b"overlap" => {
            let (value, markup) = required_scalar(&raw, "overlap")?;
            let parsed = value
                .parse::<i8>()
                .map_err(|_| invalid_attribute("overlap", "val", value.clone()))?;
            if !(-100..=100).contains(&parsed) {
                return Err(invalid_attribute("overlap", "val", value));
            }
            set_once(overlap, (parsed, markup), "c:overlap")?;
            *boundary = (*boundary).max(series.len() + 5);
        }
        b"axId" => {
            if axis_ids.len() == 2 {
                return Err(ChartError::DuplicateElement("c:axId".to_owned()));
            }
            axis_ids.push(parse_axis_id_scalar(&raw, "axId")?);
            *boundary = (*boundary).max(series.len() + 5 + axis_ids.len());
        }
        _ => raw_children.push(*boundary, raw),
    }
    Ok(())
}

fn parse_line_plot(xml: &[u8], inherited: &NamespaceBindings) -> Result<(Plot, PlotMarkup)> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) if matches_local_name(element.name().as_ref(), b"lineChart") => {
                let namespaces = typed_rewrite_bindings(&element, inherited)?;
                let (mut raw_attributes, namespace_declarations) =
                    capture_fixed_root_attributes(&element, &["xmlns:c", "xmlns:a", "xmlns:r"])?;
                raw_attributes.splice(0..0, namespace_declarations);
                let mut grouping = None;
                let mut marker = None;
                let mut smooth = None;
                let mut series = Vec::new();
                let mut data_labels = None;
                let mut axis_ids = Vec::new();
                let mut raw_children = OrderedRawChildren::default();
                let mut boundary = 0usize;
                let mut inner = Vec::new();
                loop {
                    match reader
                        .read_event_into(&mut inner)
                        .map_err(OxmlError::from)?
                    {
                        Event::Start(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_element(&mut reader, &child)?;
                            parse_line_child(
                                name.as_deref().unwrap_or_default(),
                                raw,
                                &namespaces,
                                &mut grouping,
                                &mut marker,
                                &mut smooth,
                                &mut series,
                                &mut data_labels,
                                &mut axis_ids,
                                &mut raw_children,
                                &mut boundary,
                            )?;
                        }
                        Event::Empty(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_empty_element(&child)?;
                            parse_line_child(
                                name.as_deref().unwrap_or_default(),
                                raw,
                                &namespaces,
                                &mut grouping,
                                &mut marker,
                                &mut smooth,
                                &mut series,
                                &mut data_labels,
                                &mut axis_ids,
                                &mut raw_children,
                                &mut boundary,
                            )?;
                        }
                        event @ (Event::Text(_)
                        | Event::CData(_)
                        | Event::Comment(_)
                        | Event::PI(_)
                        | Event::GeneralRef(_)) => {
                            raw_children.push(boundary, capture_event(event)?);
                        }
                        Event::End(end)
                            if matches_local_name(end.name().as_ref(), b"lineChart") =>
                        {
                            let (grouping, grouping_markup) = grouping.ok_or_else(|| {
                                ChartError::MissingElement("c:grouping".to_owned())
                            })?;
                            if axis_ids.len() != 2 {
                                return Err(ChartError::InvalidValue {
                                    element: "c:lineChart/c:axId".to_owned(),
                                    value: format!("expected 2, found {}", axis_ids.len()),
                                });
                            }
                            let parsed_ids = [axis_ids[0].0, axis_ids[1].0];
                            let original_series_keys: Vec<(u32, u32)> =
                                series.iter().map(|item| (item.index, item.order)).collect();
                            let plot = Plot::Line {
                                grouping,
                                marker: marker.as_ref().is_some_and(|item| item.0),
                                smooth: smooth.as_ref().is_some_and(|item| item.0),
                                series,
                                data_labels,
                                axis_ids: parsed_ids,
                            };
                            plot.validate()?;
                            return Ok((
                                plot,
                                PlotMarkup {
                                    raw_attributes,
                                    raw_children: raw_children_in_schema_order(
                                        &raw_children,
                                        original_series_keys.len() + 6,
                                    ),
                                    direction: None,
                                    grouping: Some(grouping_markup),
                                    gap_width: None,
                                    overlap: None,
                                    marker: marker.map(|item| item.1),
                                    smooth: smooth.map(|item| item.1),
                                    first_slice_angle: None,
                                    hole_size: None,
                                    style: None,
                                    axis_ids: axis_ids.into_iter().map(|item| item.1).collect(),
                                    original_series_keys,
                                    original_axis_ids: parsed_ids.to_vec(),
                                    parsed_bar: Some(false),
                                    parsed_remaining: None,
                                },
                            ));
                        }
                        Event::Eof => return Err(missing_end("c:lineChart")),
                        _ => {}
                    }
                    inner.clear();
                }
            }
            Event::Empty(element) if matches_local_name(element.name().as_ref(), b"lineChart") => {
                return Err(ChartError::MissingElement("c:grouping".to_owned()));
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => return Err(ChartError::MissingElement("c:lineChart".to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

#[allow(clippy::too_many_arguments)]
fn parse_line_child(
    name: &[u8],
    raw: Vec<u8>,
    namespaces: &NamespaceBindings,
    grouping: &mut Option<(Grouping, ScalarMarkup)>,
    marker: &mut Option<(bool, ScalarMarkup)>,
    smooth: &mut Option<(bool, ScalarMarkup)>,
    series: &mut Vec<Series>,
    data_labels: &mut Option<CT_DLbls>,
    axis_ids: &mut Vec<(AxisId, AxisIdMarkup)>,
    raw_children: &mut OrderedRawChildren,
    boundary: &mut usize,
) -> Result<()> {
    match name {
        b"grouping" => {
            let (value, markup) = required_scalar(&raw, "grouping")?;
            let value = Grouping::parse(&value)
                .ok_or_else(|| invalid_attribute("grouping", "val", value))?;
            set_once(grouping, (value, markup), "c:grouping")?;
            *boundary = (*boundary).max(1);
        }
        b"ser" => {
            series.push(Series::from_xml_with_namespaces(&raw, namespaces)?);
            *boundary = (*boundary).max(series.len() + 1);
        }
        b"dLbls" => {
            set_once(
                data_labels,
                CT_DLbls::from_xml_with_namespaces(&raw, namespaces)?,
                "c:dLbls",
            )?;
            *boundary = (*boundary).max(series.len() + 2);
        }
        b"marker" => {
            set_once(marker, parse_bool_value(&raw, "marker")?, "c:marker")?;
            *boundary = (*boundary).max(series.len() + 3);
        }
        b"smooth" => {
            set_once(smooth, parse_bool_value(&raw, "smooth")?, "c:smooth")?;
            *boundary = (*boundary).max(series.len() + 4);
        }
        b"axId" => {
            if axis_ids.len() == 2 {
                return Err(ChartError::DuplicateElement("c:axId".to_owned()));
            }
            axis_ids.push(parse_axis_id_scalar(&raw, "axId")?);
            *boundary = (*boundary).max(series.len() + 4 + axis_ids.len());
        }
        _ => raw_children.push(*boundary, raw),
    }
    Ok(())
}

fn required_scalar(xml: &[u8], local: &str) -> Result<(String, ScalarMarkup)> {
    let (value, markup) = scalar_value(xml, local)?;
    let value = value.ok_or_else(|| invalid_attribute(local, "val", "<missing>".to_owned()))?;
    Ok((value, markup))
}

fn write_plot<W: Write>(writer: &mut Writer<W>, plot: &Plot, markup: &PlotMarkup) -> Result<()> {
    plot.validate()?;
    if let Some(parsed_bar) = markup.parsed_bar
        && parsed_bar != matches!(plot, Plot::Bar { .. })
    {
        return Err(ChartError::InvalidValue {
            element: "c:plotArea".to_owned(),
            value: "a parsed plot family cannot be replaced while preserved payload remains"
                .to_owned(),
        });
    }
    let current_remaining = match plot {
        Plot::Pie { .. } => Some("pieChart"),
        Plot::Doughnut { .. } => Some("doughnutChart"),
        Plot::Area { .. } => Some("areaChart"),
        Plot::Scatter { .. } => Some("scatterChart"),
        Plot::Radar { .. } => Some("radarChart"),
        Plot::Bar { .. } | Plot::Line { .. } => None,
    };
    if let Some(parsed) = markup.parsed_remaining
        && current_remaining != Some(parsed)
    {
        return Err(ChartError::InvalidValue {
            element: "c:plotArea".to_owned(),
            value: "a parsed plot family cannot be replaced while preserved payload remains"
                .to_owned(),
        });
    }
    match plot {
        Plot::Bar {
            direction,
            grouping,
            gap_width,
            overlap,
            series,
            data_labels,
            axis_ids,
        } => {
            let mut start = BytesStart::new("c:barChart");
            push_attributes(&mut start, &markup.raw_attributes);
            writer
                .write_event(Event::Start(start))
                .map_err(OxmlError::from)?;
            emit_raw(writer, markup.raw_children.at(0))?;
            write_scalar(
                writer,
                "c:barDir",
                direction.as_str(),
                markup.direction.as_ref(),
            )?;
            emit_raw(writer, markup.raw_children.at(1))?;
            write_scalar(
                writer,
                "c:grouping",
                grouping.as_str(),
                markup.grouping.as_ref(),
            )?;
            let original_to_current =
                series_original_to_current(&markup.original_series_keys, series);
            emit_repeated_raw(
                writer,
                &markup.raw_children,
                2,
                &original_to_current,
                series.len(),
                0,
            )?;
            for (index, item) in series.iter().enumerate() {
                writer
                    .get_mut()
                    .write_all(&item.to_xml()?)
                    .map_err(OxmlError::from)?;
                emit_repeated_raw(
                    writer,
                    &markup.raw_children,
                    2,
                    &original_to_current,
                    series.len(),
                    index + 1,
                )?;
            }
            if let Some(labels) = data_labels {
                writer
                    .get_mut()
                    .write_all(&labels.to_xml()?)
                    .map_err(OxmlError::from)?;
            }
            let trailing = markup.original_series_keys.len();
            emit_raw(writer, markup.raw_children.at(trailing + 3))?;
            if *gap_width != 150 || markup.gap_width.is_some() {
                write_scalar(
                    writer,
                    "c:gapWidth",
                    &gap_width.to_string(),
                    markup.gap_width.as_ref(),
                )?;
            }
            emit_raw(writer, markup.raw_children.at(trailing + 4))?;
            if *overlap != 0 || markup.overlap.is_some() {
                write_scalar(
                    writer,
                    "c:overlap",
                    &overlap.to_string(),
                    markup.overlap.as_ref(),
                )?;
            }
            let axis_original_to_current =
                axis_original_to_current(&markup.original_axis_ids, axis_ids);
            let axis_current_to_original =
                invert_original_to_current(&axis_original_to_current, axis_ids.len());
            emit_repeated_raw(
                writer,
                &markup.raw_children,
                trailing + 5,
                &axis_original_to_current,
                axis_ids.len(),
                0,
            )?;
            for (index, id) in axis_ids.iter().enumerate() {
                let default_markup = default_axis_id_markup(*id);
                let original_markup = axis_current_to_original[index]
                    .and_then(|original| markup.axis_ids.get(original));
                write_axis_id(
                    writer,
                    "c:axId",
                    *id,
                    original_markup.unwrap_or(&default_markup),
                )?;
                emit_repeated_raw(
                    writer,
                    &markup.raw_children,
                    trailing + 5,
                    &axis_original_to_current,
                    axis_ids.len(),
                    index + 1,
                )?;
            }
            writer
                .write_event(Event::End(BytesEnd::new("c:barChart")))
                .map_err(OxmlError::from)?;
        }
        Plot::Line {
            grouping,
            marker,
            smooth,
            series,
            data_labels,
            axis_ids,
        } => {
            let mut start = BytesStart::new("c:lineChart");
            push_attributes(&mut start, &markup.raw_attributes);
            writer
                .write_event(Event::Start(start))
                .map_err(OxmlError::from)?;
            emit_raw(writer, markup.raw_children.at(0))?;
            write_scalar(
                writer,
                "c:grouping",
                grouping.as_str(),
                markup.grouping.as_ref(),
            )?;
            let original_to_current =
                series_original_to_current(&markup.original_series_keys, series);
            emit_repeated_raw(
                writer,
                &markup.raw_children,
                1,
                &original_to_current,
                series.len(),
                0,
            )?;
            for (index, item) in series.iter().enumerate() {
                writer
                    .get_mut()
                    .write_all(&item.to_xml()?)
                    .map_err(OxmlError::from)?;
                emit_repeated_raw(
                    writer,
                    &markup.raw_children,
                    1,
                    &original_to_current,
                    series.len(),
                    index + 1,
                )?;
            }
            if let Some(labels) = data_labels {
                writer
                    .get_mut()
                    .write_all(&labels.to_xml()?)
                    .map_err(OxmlError::from)?;
            }
            let trailing = markup.original_series_keys.len();
            emit_raw(writer, markup.raw_children.at(trailing + 2))?;
            if *marker || markup.marker.is_some() {
                write_scalar(
                    writer,
                    "c:marker",
                    bool_lexical(*marker),
                    markup.marker.as_ref(),
                )?;
            }
            emit_raw(writer, markup.raw_children.at(trailing + 3))?;
            if *smooth || markup.smooth.is_some() {
                write_scalar(
                    writer,
                    "c:smooth",
                    bool_lexical(*smooth),
                    markup.smooth.as_ref(),
                )?;
            }
            let axis_original_to_current =
                axis_original_to_current(&markup.original_axis_ids, axis_ids);
            let axis_current_to_original =
                invert_original_to_current(&axis_original_to_current, axis_ids.len());
            emit_repeated_raw(
                writer,
                &markup.raw_children,
                trailing + 4,
                &axis_original_to_current,
                axis_ids.len(),
                0,
            )?;
            for (index, id) in axis_ids.iter().enumerate() {
                let default_markup = default_axis_id_markup(*id);
                let original_markup = axis_current_to_original[index]
                    .and_then(|original| markup.axis_ids.get(original));
                write_axis_id(
                    writer,
                    "c:axId",
                    *id,
                    original_markup.unwrap_or(&default_markup),
                )?;
                emit_repeated_raw(
                    writer,
                    &markup.raw_children,
                    trailing + 4,
                    &axis_original_to_current,
                    axis_ids.len(),
                    index + 1,
                )?;
            }
            writer
                .write_event(Event::End(BytesEnd::new("c:lineChart")))
                .map_err(OxmlError::from)?;
        }
        remaining => {
            let (
                root,
                property,
                series,
                data_labels,
                axis_ids,
                first_slice_angle,
                hole_size,
                scatter,
            ) = match remaining {
                Plot::Pie {
                    first_slice_angle,
                    series,
                    data_labels,
                } => (
                    "pieChart",
                    None,
                    series,
                    data_labels,
                    None,
                    Some(*first_slice_angle),
                    None,
                    false,
                ),
                Plot::Doughnut {
                    first_slice_angle,
                    hole_size,
                    series,
                    data_labels,
                } => (
                    "doughnutChart",
                    None,
                    series,
                    data_labels,
                    None,
                    Some(*first_slice_angle),
                    Some(*hole_size),
                    false,
                ),
                Plot::Area {
                    grouping,
                    series,
                    data_labels,
                    axis_ids,
                } => (
                    "areaChart",
                    Some(("c:grouping", grouping.as_str(), markup.grouping.as_ref())),
                    series,
                    data_labels,
                    Some(axis_ids),
                    None,
                    None,
                    false,
                ),
                Plot::Scatter {
                    style,
                    series,
                    data_labels,
                    axis_ids,
                } => (
                    "scatterChart",
                    Some(("c:scatterStyle", style.as_str(), markup.style.as_ref())),
                    series,
                    data_labels,
                    Some(axis_ids),
                    None,
                    None,
                    true,
                ),
                Plot::Radar {
                    style,
                    series,
                    data_labels,
                    axis_ids,
                } => (
                    "radarChart",
                    Some(("c:radarStyle", style.as_str(), markup.style.as_ref())),
                    series,
                    data_labels,
                    Some(axis_ids),
                    None,
                    None,
                    false,
                ),
                Plot::Bar { .. } | Plot::Line { .. } => unreachable!("handled above"),
            };
            let tag = format!("c:{root}");
            let mut start = BytesStart::new(&tag);
            push_attributes(&mut start, &markup.raw_attributes);
            writer
                .write_event(Event::Start(start))
                .map_err(OxmlError::from)?;
            let base = usize::from(property.is_some());
            if let Some((name, value, scalar_markup)) = property {
                emit_raw(writer, markup.raw_children.at(0))?;
                write_scalar(writer, name, value, scalar_markup)?;
            }
            let original_to_current =
                series_original_to_current(&markup.original_series_keys, series);
            emit_repeated_raw(
                writer,
                &markup.raw_children,
                base,
                &original_to_current,
                series.len(),
                0,
            )?;
            for (index, item) in series.iter().enumerate() {
                writer
                    .get_mut()
                    .write_all(&item.to_xml_for_plot(scatter)?)
                    .map_err(OxmlError::from)?;
                emit_repeated_raw(
                    writer,
                    &markup.raw_children,
                    base,
                    &original_to_current,
                    series.len(),
                    index + 1,
                )?;
            }
            if let Some(labels) = data_labels {
                writer
                    .get_mut()
                    .write_all(&labels.to_xml()?)
                    .map_err(OxmlError::from)?;
            }
            let trailing = base + markup.original_series_keys.len();
            if let Some(axis_ids) = axis_ids {
                let axis_original_to_current =
                    axis_original_to_current(&markup.original_axis_ids, axis_ids);
                let axis_current_to_original =
                    invert_original_to_current(&axis_original_to_current, axis_ids.len());
                emit_repeated_raw(
                    writer,
                    &markup.raw_children,
                    trailing + 1,
                    &axis_original_to_current,
                    axis_ids.len(),
                    0,
                )?;
                for (index, id) in axis_ids.iter().enumerate() {
                    let default_markup = default_axis_id_markup(*id);
                    let original_markup = axis_current_to_original[index]
                        .and_then(|original| markup.axis_ids.get(original));
                    write_axis_id(
                        writer,
                        "c:axId",
                        *id,
                        original_markup.unwrap_or(&default_markup),
                    )?;
                    emit_repeated_raw(
                        writer,
                        &markup.raw_children,
                        trailing + 1,
                        &axis_original_to_current,
                        axis_ids.len(),
                        index + 1,
                    )?;
                }
            } else {
                emit_raw(writer, markup.raw_children.at(trailing + 1))?;
                if let Some(angle) = first_slice_angle {
                    if angle != 0 || markup.first_slice_angle.is_some() {
                        write_scalar(
                            writer,
                            "c:firstSliceAng",
                            &angle.to_string(),
                            markup.first_slice_angle.as_ref(),
                        )?;
                    }
                    emit_raw(writer, markup.raw_children.at(trailing + 2))?;
                }
                if let Some(size) = hole_size {
                    if size != 50 || markup.hole_size.is_some() {
                        write_scalar(
                            writer,
                            "c:holeSize",
                            &size.to_string(),
                            markup.hole_size.as_ref(),
                        )?;
                    }
                    emit_raw(writer, markup.raw_children.at(trailing + 3))?;
                }
            }
            writer
                .write_event(Event::End(BytesEnd::new(&tag)))
                .map_err(OxmlError::from)?;
        }
    }
    Ok(())
}

fn series_original_to_current(original: &[(u32, u32)], current: &[Series]) -> Vec<Option<usize>> {
    let mut used = vec![false; current.len()];
    let mut matches = vec![None; original.len()];
    for original_index in 0..original.len().min(current.len()) {
        if original[original_index]
            == (current[original_index].index, current[original_index].order)
        {
            matches[original_index] = Some(original_index);
            used[original_index] = true;
        }
    }
    for (original_index, key) in original.iter().enumerate() {
        if matches[original_index].is_some() {
            continue;
        }
        if let Some(current_index) = current
            .iter()
            .enumerate()
            .position(|(index, item)| !used[index] && (item.index, item.order) == *key)
        {
            matches[original_index] = Some(current_index);
            used[current_index] = true;
        }
    }
    match_unidentified_by_position(&mut matches, &mut used);
    matches
}

fn axis_original_to_current(original: &[AxisId], current: &[AxisId]) -> Vec<Option<usize>> {
    let mut used = vec![false; current.len()];
    let mut matches = vec![None; original.len()];
    for (original_index, id) in original.iter().enumerate() {
        if let Some(current_index) = current
            .iter()
            .enumerate()
            .position(|(index, current_id)| !used[index] && current_id == id)
        {
            matches[original_index] = Some(current_index);
            used[current_index] = true;
        }
    }
    match_unidentified_by_position(&mut matches, &mut used);
    matches
}

fn match_unidentified_by_position(matches: &mut [Option<usize>], used: &mut [bool]) {
    for original_index in 0..matches.len() {
        if matches[original_index].is_none() && original_index < used.len() && !used[original_index]
        {
            matches[original_index] = Some(original_index);
            used[original_index] = true;
        }
    }
    for matched in matches.iter_mut().filter(|matched| matched.is_none()) {
        if let Some(current_index) = used.iter().position(|used| !used) {
            *matched = Some(current_index);
            used[current_index] = true;
        }
    }
}

fn invert_original_to_current(
    original_to_current: &[Option<usize>],
    current_len: usize,
) -> Vec<Option<usize>> {
    let mut current_to_original = vec![None; current_len];
    for (original, current) in original_to_current.iter().enumerate() {
        if let Some(current) = current {
            current_to_original[*current] = Some(original);
        }
    }
    current_to_original
}

fn emit_repeated_raw<W: Write>(
    writer: &mut Writer<W>,
    raw_children: &OrderedRawChildren,
    offset: usize,
    original_to_current: &[Option<usize>],
    current_len: usize,
    current_boundary: usize,
) -> Result<()> {
    if current_boundary == 0 {
        emit_raw(writer, raw_children.at(offset))?;
    }
    for original_boundary in 1..=original_to_current.len() {
        let effective = original_to_current
            .iter()
            .skip(original_boundary)
            .flatten()
            .copied()
            .next()
            .unwrap_or(current_len);
        if effective == current_boundary {
            emit_raw(writer, raw_children.at(offset + original_boundary))?;
        }
    }
    Ok(())
}

fn parse_plot_axis(xml: &[u8], inherited: &NamespaceBindings) -> Result<Option<Axis>> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) | Event::Empty(element) => {
                if !element_is_in_namespace(&element, C_NS, inherited)? {
                    return Ok(None);
                }
                let name = element.name();
                if AxisKind::parse(local_name(name.as_ref())).is_none() {
                    return Ok(None);
                }
                return Axis::from_xml_with_namespaces(xml, inherited).map(Some);
            }
            Event::Eof => return Ok(None),
            _ => {}
        }
        buffer.clear();
    }
}

fn validate_axis_pairs(axes: &[Axis]) -> Result<()> {
    for (index, axis) in axes.iter().enumerate() {
        if axes[..index].iter().any(|other| other.id == axis.id) {
            return Err(ChartError::DuplicateElement(format!(
                "c:axId {}",
                axis.id.value()
            )));
        }
    }
    for axis in axes {
        if axis.id == axis.cross_axis {
            return Err(ChartError::InvalidValue {
                element: "c:crossAx".to_owned(),
                value: format!("axis {} crosses itself", axis.id.value()),
            });
        }
        let crossed = axes
            .iter()
            .find(|candidate| candidate.id == axis.cross_axis)
            .ok_or_else(|| ChartError::InvalidValue {
                element: "c:crossAx".to_owned(),
                value: format!(
                    "axis {} references missing axis {}",
                    axis.id.value(),
                    axis.cross_axis.value()
                ),
            })?;
        if crossed.cross_axis != axis.id {
            return Err(ChartError::InvalidValue {
                element: "c:crossAx".to_owned(),
                value: format!(
                    "axis {} references {}, which references {}",
                    axis.id.value(),
                    crossed.id.value(),
                    crossed.cross_axis.value()
                ),
            });
        }
    }
    Ok(())
}

fn parse_plot_series(
    xml: &[u8],
    inherited: &NamespaceBindings,
    series: &mut Vec<Series>,
) -> Result<()> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) => {
                if !element_is_in_namespace(&element, C_NS, inherited)? {
                    return Ok(());
                }
                let local = local_name(element.name().as_ref()).to_vec();
                if !is_supported_series_plot(&local) {
                    return Ok(());
                }
                let namespaces = chart_bindings(inherited, &element)?;
                let mut inner = Vec::new();
                loop {
                    match reader
                        .read_event_into(&mut inner)
                        .map_err(OxmlError::from)?
                    {
                        Event::Start(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            let raw = capture_element(&mut reader, &child)?;
                            if name.as_deref() == Some(b"ser") {
                                series.push(Series::from_xml_with_namespaces(&raw, &namespaces)?);
                            }
                        }
                        Event::Empty(child) => {
                            let name = chart_child_local(&child, &namespaces)?;
                            if name.as_deref() == Some(b"ser") {
                                return Err(ChartError::MissingElement("c:ser/c:idx".to_owned()));
                            }
                        }
                        Event::End(end) if matches_local_name(end.name().as_ref(), &local) => {
                            return Ok(());
                        }
                        Event::Eof => {
                            return Err(missing_end(&format!(
                                "c:{}",
                                String::from_utf8_lossy(&local)
                            )));
                        }
                        _ => {}
                    }
                    inner.clear();
                }
            }
            Event::Empty(_) | Event::Eof => return Ok(()),
            _ => {}
        }
        buffer.clear();
    }
}

fn is_supported_series_plot(local: &[u8]) -> bool {
    matches!(
        local,
        b"areaChart"
            | b"area3DChart"
            | b"barChart"
            | b"bar3DChart"
            | b"doughnutChart"
            | b"lineChart"
            | b"line3DChart"
            | b"ofPieChart"
            | b"pieChart"
            | b"pie3DChart"
            | b"radarChart"
            | b"stockChart"
            | b"surfaceChart"
            | b"surface3DChart"
    )
}

impl CT_Legend {
    fn from_xml(xml: &[u8]) -> Result<Self> {
        let (raw_attributes, raw_children) = parse_raw_shell(xml, b"legend", "c:legend")?;
        Ok(Self {
            raw_attributes,
            raw_children,
        })
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        write_raw_shell(writer, "c:legend", &self.raw_attributes, &self.raw_children)
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }
}

fn parse_raw_shell(
    xml: &[u8],
    local: &[u8],
    tag: &str,
) -> Result<(XmlAttributes, OrderedRawChildren)> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) if matches_local_name(element.name().as_ref(), local) => {
                reject_conflicting_prefix(&element, b"c", C_NS)?;
                let raw_attributes = capture_attributes(&element)?;
                let mut raw_children = OrderedRawChildren::default();
                let mut child_buffer = Vec::new();
                loop {
                    match reader
                        .read_event_into(&mut child_buffer)
                        .map_err(OxmlError::from)?
                    {
                        Event::Start(child) => {
                            raw_children.push(0, capture_element(&mut reader, &child)?)
                        }
                        Event::Empty(child) => raw_children.push(0, capture_empty_element(&child)?),
                        event @ (Event::Text(_)
                        | Event::CData(_)
                        | Event::Comment(_)
                        | Event::PI(_)
                        | Event::GeneralRef(_)) => {
                            raw_children.push(0, capture_event(event)?);
                        }
                        Event::End(end) if matches_local_name(end.name().as_ref(), local) => break,
                        Event::Eof => return Err(missing_end(tag)),
                        _ => {}
                    }
                    child_buffer.clear();
                }
                return Ok((raw_attributes, raw_children));
            }
            Event::Empty(element) if matches_local_name(element.name().as_ref(), local) => {
                reject_conflicting_prefix(&element, b"c", C_NS)?;
                return Ok((capture_attributes(&element)?, OrderedRawChildren::default()));
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => return Err(ChartError::MissingElement(tag.to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

fn write_raw_shell<W: Write>(
    writer: &mut Writer<W>,
    tag: &str,
    raw_attributes: &XmlAttributes,
    raw_children: &OrderedRawChildren,
) -> Result<()> {
    let mut start = BytesStart::new(tag);
    push_attributes(&mut start, raw_attributes);
    if raw_children.is_empty() {
        writer
            .write_event(Event::Empty(start))
            .map_err(OxmlError::from)?;
        return Ok(());
    }
    writer
        .write_event(Event::Start(start))
        .map_err(OxmlError::from)?;
    emit_raw(writer, raw_children.at(0))?;
    writer
        .write_event(Event::End(BytesEnd::new(tag)))
        .map_err(OxmlError::from)?;
    Ok(())
}

/// The implemented behavior-bearing core of one `c:chart`.
#[derive(Clone, Debug, PartialEq)]
pub struct CT_Chart {
    pub title: Option<CT_Title>,
    pub auto_title_deleted: bool,
    pub plot_area: CT_PlotArea,
    pub legend: Option<CT_Legend>,
    pub plot_vis_only: bool,
    pub disp_blanks_as: DispBlanksAs,
    auto_title_deleted_markup: Option<ScalarMarkup>,
    plot_vis_only_markup: Option<ScalarMarkup>,
    disp_blanks_as_markup: Option<ScalarMarkup>,
    raw_attributes: Vec<(String, String)>,
    raw_children: OrderedRawChildren,
}

impl CT_Chart {
    fn from_xml_with_namespaces(xml: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) if matches_local_name(element.name().as_ref(), b"chart") => {
                    reject_conflicting_prefix(&element, b"c", C_NS)?;
                    return Self::from_element(&mut reader, &element, inherited);
                }
                Event::Empty(element) if matches_local_name(element.name().as_ref(), b"chart") => {
                    return Err(ChartError::MissingElement("c:plotArea".to_owned()));
                }
                Event::Start(element) | Event::Empty(element) => {
                    return Err(ChartError::UnexpectedElement(element_name(&element)));
                }
                Event::Eof => return Err(ChartError::MissingElement("c:chart".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_element(
        reader: &mut Reader<&[u8]>,
        start: &BytesStart<'_>,
        inherited: &NamespaceBindings,
    ) -> Result<Self> {
        let namespaces = bindings_with_local(inherited, start)?;
        let raw_attributes = capture_attributes(start)?;
        let mut state = ChartParseState::default();
        let mut buffer = Vec::new();

        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) => {
                    let name = if element_is_in_namespace(&element, C_NS, &namespaces)? {
                        local_name(element.name().as_ref()).to_vec()
                    } else {
                        Vec::new()
                    };
                    let raw = capture_element(reader, &element)?;
                    state.parse_child(&name, raw, &namespaces)?;
                }
                Event::Empty(element) => {
                    let name = if element_is_in_namespace(&element, C_NS, &namespaces)? {
                        local_name(element.name().as_ref()).to_vec()
                    } else {
                        Vec::new()
                    };
                    let raw = capture_empty_element(&element)?;
                    state.parse_child(&name, raw, &namespaces)?;
                }
                event @ (Event::Text(_)
                | Event::CData(_)
                | Event::Comment(_)
                | Event::PI(_)
                | Event::GeneralRef(_)) => {
                    state.capture_event(capture_event(event)?);
                }
                Event::End(element) if matches_local_name(element.name().as_ref(), b"chart") => {
                    break;
                }
                Event::Eof => return Err(missing_end("c:chart")),
                _ => {}
            }
            buffer.clear();
        }

        state.finish(raw_attributes)
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut start = BytesStart::new("c:chart");
        push_attributes(&mut start, &self.raw_attributes);
        writer
            .write_event(Event::Start(start))
            .map_err(OxmlError::from)?;
        emit_raw(writer, self.raw_children.at(0))?;
        if let Some(title) = &self.title {
            title.write_xml(writer)?;
        }
        emit_raw(writer, self.raw_children.at(1))?;
        if self.auto_title_deleted_markup.is_some() || self.auto_title_deleted {
            write_scalar(
                writer,
                "c:autoTitleDeleted",
                if self.auto_title_deleted { "1" } else { "0" },
                self.auto_title_deleted_markup.as_ref(),
            )?;
        }
        for raw_boundary in 2..8 {
            emit_raw(writer, self.raw_children.at(raw_boundary))?;
        }
        self.plot_area.write_xml(writer)?;
        emit_raw(writer, self.raw_children.at(8))?;
        if let Some(legend) = &self.legend {
            legend.write_xml(writer)?;
        }
        emit_raw(writer, self.raw_children.at(9))?;
        if self.plot_vis_only_markup.is_some() || !self.plot_vis_only {
            write_scalar(
                writer,
                "c:plotVisOnly",
                if self.plot_vis_only { "1" } else { "0" },
                self.plot_vis_only_markup.as_ref(),
            )?;
        }
        emit_raw(writer, self.raw_children.at(10))?;
        if self.disp_blanks_as_markup.is_some() || self.disp_blanks_as != DispBlanksAs::Gap {
            write_scalar(
                writer,
                "c:dispBlanksAs",
                self.disp_blanks_as.as_str(),
                self.disp_blanks_as_markup.as_ref(),
            )?;
        }
        for raw_boundary in 11..=13 {
            emit_raw(writer, self.raw_children.at(raw_boundary))?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("c:chart")))
            .map_err(OxmlError::from)?;
        Ok(())
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }
}

#[derive(Default)]
struct ChartParseState {
    title: Option<CT_Title>,
    auto_title_deleted: Option<(bool, ScalarMarkup)>,
    plot_area: Option<CT_PlotArea>,
    legend: Option<CT_Legend>,
    plot_vis_only: Option<(bool, ScalarMarkup)>,
    disp_blanks_as: Option<(DispBlanksAs, ScalarMarkup)>,
    raw_children: OrderedRawChildren,
    boundary: usize,
}

impl ChartParseState {
    fn capture_event(&mut self, raw: Vec<u8>) {
        self.raw_children.push(self.boundary, raw);
    }

    fn parse_child(
        &mut self,
        name: &[u8],
        raw: Vec<u8>,
        namespaces: &NamespaceBindings,
    ) -> Result<()> {
        match name {
            b"title" => {
                set_once(&mut self.title, CT_Title::from_xml(&raw)?, "c:title")?;
                self.boundary = self.boundary.max(1);
            }
            b"autoTitleDeleted" => {
                set_once(
                    &mut self.auto_title_deleted,
                    parse_bool_value(&raw, "autoTitleDeleted")?,
                    "c:autoTitleDeleted",
                )?;
                self.boundary = self.boundary.max(2);
            }
            b"plotArea" => {
                set_once(
                    &mut self.plot_area,
                    CT_PlotArea::from_xml_with_namespaces(&raw, namespaces)?,
                    "c:plotArea",
                )?;
                self.boundary = self.boundary.max(8);
            }
            b"legend" => {
                set_once(&mut self.legend, CT_Legend::from_xml(&raw)?, "c:legend")?;
                self.boundary = self.boundary.max(9);
            }
            b"plotVisOnly" => {
                set_once(
                    &mut self.plot_vis_only,
                    parse_bool_value(&raw, "plotVisOnly")?,
                    "c:plotVisOnly",
                )?;
                self.boundary = self.boundary.max(10);
            }
            b"dispBlanksAs" => {
                set_once(
                    &mut self.disp_blanks_as,
                    parse_enum_value(&raw, "dispBlanksAs")?,
                    "c:dispBlanksAs",
                )?;
                self.boundary = self.boundary.max(11);
            }
            _ => {
                self.raw_children
                    .push(chart_raw_slot(name).unwrap_or(self.boundary), raw);
                self.boundary = self.boundary.max(chart_raw_boundary_after(name));
            }
        }
        Ok(())
    }

    fn finish(self, raw_attributes: XmlAttributes) -> Result<CT_Chart> {
        let (auto_title_deleted, auto_title_deleted_markup) = self
            .auto_title_deleted
            .map(|(value, markup)| (value, Some(markup)))
            .unwrap_or((false, None));
        let (plot_vis_only, plot_vis_only_markup) = self
            .plot_vis_only
            .map(|(value, markup)| (value, Some(markup)))
            .unwrap_or((true, None));
        let (disp_blanks_as, disp_blanks_as_markup) = self
            .disp_blanks_as
            .map(|(value, markup)| (value, Some(markup)))
            .unwrap_or((DispBlanksAs::Gap, None));
        Ok(CT_Chart {
            title: self.title,
            auto_title_deleted,
            plot_area: self
                .plot_area
                .ok_or_else(|| ChartError::MissingElement("c:plotArea".to_owned()))?,
            legend: self.legend,
            plot_vis_only,
            disp_blanks_as,
            auto_title_deleted_markup,
            plot_vis_only_markup,
            disp_blanks_as_markup,
            raw_attributes,
            raw_children: raw_children_in_schema_order(&self.raw_children, 13),
        })
    }
}

fn chart_raw_slot(name: &[u8]) -> Option<usize> {
    match name {
        b"pivotFmts" => Some(2),
        b"view3D" => Some(3),
        b"floor" => Some(4),
        b"sideWall" => Some(5),
        b"backWall" => Some(6),
        // Both unmodelled tail children share a slot so producer sibling order
        // remains byte-stable until a later story types either element.
        b"showDLblsOverMax" | b"extLst" => Some(11),
        _ => None,
    }
}

fn chart_raw_boundary_after(name: &[u8]) -> usize {
    match name {
        b"title" => 1,
        b"autoTitleDeleted" => 2,
        b"pivotFmts" => 3,
        b"view3D" => 4,
        b"floor" => 5,
        b"sideWall" => 6,
        b"backWall" => 7,
        b"plotArea" => 8,
        b"legend" => 9,
        b"plotVisOnly" => 10,
        b"dispBlanksAs" => 11,
        b"showDLblsOverMax" | b"extLst" => 11,
        _ => 0,
    }
}

/// One complete `c:chartSpace` part with the F-118 core typed.
#[derive(Clone, Debug, PartialEq)]
pub struct CT_ChartSpace {
    pub chart: CT_Chart,
    pub sp_pr: Option<CT_ShapeProperties>,
    pub tx_pr: Option<CT_TextBody>,
    raw_attributes: Vec<(String, String)>,
    namespace_declarations: Vec<(String, String)>,
    raw_children: OrderedRawChildren,
}

impl CT_ChartSpace {
    /// Parses one complete ChartML chart-space part.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element)
                    if matches_local_name(element.name().as_ref(), b"chartSpace") =>
                {
                    validate_chart_space_namespace(&element)?;
                    return Self::from_element(&mut reader, &element);
                }
                Event::Empty(element)
                    if matches_local_name(element.name().as_ref(), b"chartSpace") =>
                {
                    validate_chart_space_namespace(&element)?;
                    return Err(ChartError::MissingElement("c:chart".to_owned()));
                }
                Event::Start(element) | Event::Empty(element) => {
                    return Err(ChartError::UnexpectedElement(element_name(&element)));
                }
                Event::Eof => return Err(ChartError::MissingElement("c:chartSpace".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_element(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let namespaces = namespace_bindings(start)?;
        let (raw_attributes, namespace_declarations) = capture_root_attributes(start)?;
        let mut state = ChartSpaceParseState::default();
        let mut buffer = Vec::new();

        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) => {
                    let name = if element_is_in_namespace(&element, C_NS, &namespaces)? {
                        local_name(element.name().as_ref()).to_vec()
                    } else {
                        Vec::new()
                    };
                    let raw = capture_element(reader, &element)?;
                    state.parse_child(&name, raw, &namespaces)?;
                }
                Event::Empty(element) => {
                    let name = if element_is_in_namespace(&element, C_NS, &namespaces)? {
                        local_name(element.name().as_ref()).to_vec()
                    } else {
                        Vec::new()
                    };
                    let raw = capture_empty_element(&element)?;
                    state.parse_child(&name, raw, &namespaces)?;
                }
                event @ (Event::Text(_)
                | Event::CData(_)
                | Event::Comment(_)
                | Event::PI(_)
                | Event::GeneralRef(_)) => {
                    state.capture_event(capture_event(event)?);
                }
                Event::End(element)
                    if matches_local_name(element.name().as_ref(), b"chartSpace") =>
                {
                    break;
                }
                Event::Eof => return Err(missing_end("c:chartSpace")),
                _ => {}
            }
            buffer.clear();
        }

        state.finish(raw_attributes, namespace_declarations)
    }

    /// Writes one chart-space part using fixed OOXML prefixes and schema order.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        let mut start = BytesStart::new("c:chartSpace");
        start.push_attribute(("xmlns:c", C_NS));
        start.push_attribute(("xmlns:a", A_NS));
        start.push_attribute(("xmlns:r", R_NS));
        push_attributes(&mut start, &self.namespace_declarations);
        push_attributes(&mut start, &self.raw_attributes);
        writer
            .write_event(Event::Start(start))
            .map_err(OxmlError::from)?;
        for raw_boundary in 0..8 {
            emit_raw(&mut writer, self.raw_children.at(raw_boundary))?;
        }
        self.chart.write_xml(&mut writer)?;
        emit_raw(&mut writer, self.raw_children.at(8))?;
        if let Some(properties) = &self.sp_pr {
            properties.write_xml_as(&mut writer, "c:spPr")?;
        }
        emit_raw(&mut writer, self.raw_children.at(9))?;
        if let Some(text) = &self.tx_pr {
            text.write_xml_as(&mut writer, "c:txPr")?;
        }
        for raw_boundary in 10..=14 {
            emit_raw(&mut writer, self.raw_children.at(raw_boundary))?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("c:chartSpace")))
            .map_err(OxmlError::from)?;
        Ok(writer.into_inner())
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }
}

#[derive(Default)]
struct ChartSpaceParseState {
    chart: Option<CT_Chart>,
    sp_pr: Option<CT_ShapeProperties>,
    tx_pr: Option<CT_TextBody>,
    raw_children: OrderedRawChildren,
    boundary: usize,
}

impl ChartSpaceParseState {
    fn capture_event(&mut self, raw: Vec<u8>) {
        self.raw_children.push(self.boundary, raw);
    }

    fn parse_child(
        &mut self,
        name: &[u8],
        raw: Vec<u8>,
        namespaces: &NamespaceBindings,
    ) -> Result<()> {
        match name {
            b"chart" => {
                set_once(
                    &mut self.chart,
                    CT_Chart::from_xml_with_namespaces(&raw, namespaces)?,
                    "c:chart",
                )?;
                self.boundary = self.boundary.max(8);
            }
            b"spPr" => {
                reject_conflicting_prefix_in_xml(&raw, b"a", A_NS)?;
                let properties = CT_ShapeProperties::from_xml(&raw)?;
                let mut writer = Writer::new(Vec::new());
                properties.write_xml_as(&mut writer, "c:spPr")?;
                reject_rewritten_foreign_elements(&raw, &writer.into_inner(), namespaces, b"spPr")?;
                set_once(&mut self.sp_pr, properties, "c:spPr")?;
                self.boundary = self.boundary.max(9);
            }
            b"txPr" => {
                reject_conflicting_prefix_in_xml(&raw, b"a", A_NS)?;
                let text = CT_TextBody::from_xml_as(&raw, b"txPr")?;
                let mut writer = Writer::new(Vec::new());
                text.write_xml_as(&mut writer, "c:txPr")?;
                reject_rewritten_foreign_elements(&raw, &writer.into_inner(), namespaces, b"txPr")?;
                set_once(&mut self.tx_pr, text, "c:txPr")?;
                self.boundary = self.boundary.max(10);
            }
            _ => {
                self.raw_children
                    .push(chart_space_raw_slot(name).unwrap_or(self.boundary), raw);
                self.boundary = self.boundary.max(chart_space_raw_boundary_after(name));
            }
        }
        Ok(())
    }

    fn finish(
        self,
        raw_attributes: XmlAttributes,
        namespace_declarations: XmlAttributes,
    ) -> Result<CT_ChartSpace> {
        Ok(CT_ChartSpace {
            chart: self
                .chart
                .ok_or_else(|| ChartError::MissingElement("c:chart".to_owned()))?,
            sp_pr: self.sp_pr,
            tx_pr: self.tx_pr,
            raw_attributes,
            namespace_declarations,
            raw_children: raw_children_in_schema_order(&self.raw_children, 14),
        })
    }
}

fn raw_children_in_schema_order(
    raw_children: &OrderedRawChildren,
    final_boundary: usize,
) -> OrderedRawChildren {
    let mut ordered = OrderedRawChildren::default();
    for boundary in 0..=final_boundary {
        for raw in raw_children.at(boundary) {
            ordered.push(boundary, raw.to_vec());
        }
    }
    ordered
}

fn chart_space_raw_slot(name: &[u8]) -> Option<usize> {
    match name {
        b"date1904" => Some(0),
        b"lang" => Some(1),
        b"roundedCorners" => Some(2),
        b"style" => Some(3),
        b"clrMapOvr" => Some(4),
        b"pivotSource" => Some(5),
        b"protection" => Some(6),
        b"externalData" => Some(10),
        b"printSettings" => Some(11),
        b"userShapes" => Some(12),
        b"extLst" => Some(13),
        _ => None,
    }
}

fn chart_space_raw_boundary_after(name: &[u8]) -> usize {
    match name {
        b"date1904" => 1,
        b"lang" => 2,
        b"roundedCorners" => 3,
        b"style" => 4,
        b"clrMapOvr" => 5,
        b"pivotSource" => 6,
        b"protection" => 7,
        b"chart" => 8,
        b"spPr" => 9,
        b"txPr" => 10,
        b"externalData" => 11,
        b"printSettings" => 12,
        b"userShapes" => 13,
        b"extLst" => 14,
        _ => 0,
    }
}

fn parse_bool_value(xml: &[u8], local: &str) -> Result<(bool, ScalarMarkup)> {
    let (value, markup) = scalar_value(xml, local)?;
    let value = match value.as_deref().unwrap_or("1") {
        "1" | "true" => true,
        "0" | "false" => false,
        invalid => return Err(invalid_attribute(local, "val", invalid.to_owned())),
    };
    Ok((value, markup))
}

fn parse_enum_value(xml: &[u8], local: &str) -> Result<(DispBlanksAs, ScalarMarkup)> {
    let (value, markup) = scalar_value(xml, local)?;
    let value = value.ok_or_else(|| invalid_attribute(local, "val", "<missing>".to_owned()))?;
    let parsed =
        DispBlanksAs::parse(&value).ok_or_else(|| invalid_attribute(local, "val", value))?;
    Ok((parsed, markup))
}

fn scalar_value(xml: &[u8], expected: &str) -> Result<(Option<String>, ScalarMarkup)> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Empty(element)
                if matches_local_name(element.name().as_ref(), expected.as_bytes()) =>
            {
                reject_conflicting_prefix(&element, b"c", C_NS)?;
                return Ok((
                    attribute_value(&element, b"val")?,
                    ScalarMarkup {
                        raw_attributes: capture_attributes_except(&element, b"val")?,
                        raw_content: Vec::new(),
                    },
                ));
            }
            Event::Start(element)
                if matches_local_name(element.name().as_ref(), expected.as_bytes()) =>
            {
                reject_conflicting_prefix(&element, b"c", C_NS)?;
                let value = attribute_value(&element, b"val")?;
                let raw_attributes = capture_attributes_except(&element, b"val")?;
                let mut raw_content = Vec::new();
                let mut inner = Vec::new();
                loop {
                    match reader
                        .read_event_into(&mut inner)
                        .map_err(OxmlError::from)?
                    {
                        Event::End(end)
                            if matches_local_name(end.name().as_ref(), expected.as_bytes()) =>
                        {
                            return Ok((
                                value,
                                ScalarMarkup {
                                    raw_attributes,
                                    raw_content,
                                },
                            ));
                        }
                        event @ (Event::Text(_)
                        | Event::CData(_)
                        | Event::Comment(_)
                        | Event::PI(_)
                        | Event::GeneralRef(_)) => raw_content.push(capture_event(event)?),
                        Event::Eof => return Err(missing_end(expected)),
                        _ => return Err(ChartError::UnexpectedElement(expected.to_owned())),
                    }
                    inner.clear();
                }
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => return Err(ChartError::MissingElement(expected.to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

fn set_once<T>(slot: &mut Option<T>, value: T, name: &str) -> Result<()> {
    if slot.is_some() {
        return Err(ChartError::DuplicateElement(name.to_owned()));
    }
    *slot = Some(value);
    Ok(())
}

fn mark_once(seen: &mut bool, name: &str) -> Result<()> {
    if *seen {
        return Err(ChartError::DuplicateElement(name.to_owned()));
    }
    *seen = true;
    Ok(())
}

fn validate_chart_space_namespace(element: &BytesStart<'_>) -> Result<()> {
    let name = element.name();
    let qualified = name.as_ref();
    let declaration = match qualified.iter().position(|byte| *byte == b':') {
        Some(position) => {
            let mut declaration = b"xmlns:".to_vec();
            declaration.extend_from_slice(&qualified[..position]);
            declaration
        }
        None => b"xmlns".to_vec(),
    };
    let namespace = attribute_value(element, &declaration)?
        .ok_or_else(|| invalid_attribute("chartSpace", "namespace", "<missing>".to_owned()))?;
    if namespace != C_NS {
        return Err(invalid_attribute("chartSpace", "namespace", namespace));
    }
    reject_conflicting_prefix(element, b"c", C_NS)?;
    reject_conflicting_prefix(element, b"a", A_NS)?;
    reject_conflicting_prefix(element, b"r", R_NS)
}

fn reject_conflicting_prefix_in_xml(xml: &[u8], prefix: &[u8], namespace: &str) -> Result<()> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) | Event::Empty(element) => {
                return reject_conflicting_prefix(&element, prefix, namespace);
            }
            Event::Eof => return Err(ChartError::MissingElement("XML root".to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

fn reject_rewritten_foreign_elements(
    original: &[u8],
    written: &[u8],
    inherited: &NamespaceBindings,
    root_local: &[u8],
) -> Result<()> {
    let original = foreign_element_regions(original, inherited, root_local)?;
    let mut written_namespaces = inherited.clone();
    written_namespaces.retain(|(prefix, _)| prefix != b"a");
    written_namespaces.push((b"a".to_vec(), A_NS.to_owned()));
    let written = foreign_element_regions(written, &written_namespaces, root_local)?;
    if original != written {
        return Err(invalid_attribute(
            &String::from_utf8_lossy(root_local),
            "namespace",
            "foreign DrawingML content would be converted to typed content".to_owned(),
        ));
    }
    Ok(())
}

fn foreign_element_regions(
    xml: &[u8],
    inherited: &NamespaceBindings,
    root_local: &[u8],
) -> Result<Vec<Vec<u8>>> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let root_namespaces = loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) if matches_local_name(element.name().as_ref(), root_local) => {
                break bindings_with_local(inherited, &element)?;
            }
            Event::Empty(element) if matches_local_name(element.name().as_ref(), root_local) => {
                return Ok(Vec::new());
            }
            Event::Start(element) | Event::Empty(element) => {
                return Err(ChartError::UnexpectedElement(element_name(&element)));
            }
            Event::Eof => {
                return Err(ChartError::MissingElement(
                    String::from_utf8_lossy(root_local).into_owned(),
                ));
            }
            _ => {}
        }
        buffer.clear();
    };

    let mut namespace_stack = vec![root_namespaces];
    let mut foreign = Vec::new();
    let mut child_buffer = Vec::new();
    loop {
        match reader
            .read_event_into(&mut child_buffer)
            .map_err(OxmlError::from)?
        {
            Event::Start(element) => {
                let namespaces = namespace_stack.last().expect("root namespace binding");
                if element_is_in_namespace(&element, A_NS, namespaces)? {
                    namespace_stack.push(bindings_with_local(namespaces, &element)?);
                } else {
                    foreign.push(capture_element(&mut reader, &element)?);
                }
            }
            Event::Empty(element) => {
                let namespaces = namespace_stack.last().expect("root namespace binding");
                if !element_is_in_namespace(&element, A_NS, namespaces)? {
                    foreign.push(capture_empty_element(&element)?);
                }
            }
            Event::End(element) if matches_local_name(element.name().as_ref(), root_local) => {
                return Ok(foreign);
            }
            Event::End(_) => {
                namespace_stack.pop();
            }
            Event::Eof => {
                return Err(missing_end(&String::from_utf8_lossy(root_local)));
            }
            _ => {}
        }
        child_buffer.clear();
    }
}

fn reject_conflicting_prefix(
    element: &BytesStart<'_>,
    prefix: &[u8],
    namespace: &str,
) -> Result<()> {
    let mut key = b"xmlns:".to_vec();
    key.extend_from_slice(prefix);
    if let Some(value) = attribute_value(element, &key)?
        && value != namespace
    {
        return Err(invalid_attribute(
            &element_name(element),
            &String::from_utf8_lossy(&key),
            value,
        ));
    }
    Ok(())
}

fn namespace_bindings(element: &BytesStart<'_>) -> Result<NamespaceBindings> {
    let mut bindings = Vec::new();
    for attribute in element.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        let key = attribute.key.as_ref();
        let prefix = if key == b"xmlns" {
            Vec::new()
        } else if let Some(prefix) = key.strip_prefix(b"xmlns:") {
            prefix.to_vec()
        } else {
            continue;
        };
        let namespace = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())
            .map_err(OxmlError::from)?
            .into_owned();
        bindings.push((prefix, namespace));
    }
    Ok(bindings)
}

fn bindings_with_local(
    inherited: &NamespaceBindings,
    element: &BytesStart<'_>,
) -> Result<NamespaceBindings> {
    let mut bindings = inherited.clone();
    for (prefix, namespace) in namespace_bindings(element)? {
        upsert_namespace_binding(&mut bindings, prefix, namespace);
    }
    Ok(bindings)
}

fn upsert_namespace_binding(bindings: &mut NamespaceBindings, prefix: Vec<u8>, namespace: String) {
    if let Some((_, current)) = bindings.iter_mut().find(|(current, _)| current == &prefix) {
        *current = namespace;
    } else {
        bindings.push((prefix, namespace));
    }
}

fn element_is_in_namespace(
    element: &BytesStart<'_>,
    expected: &str,
    inherited: &NamespaceBindings,
) -> Result<bool> {
    let name = element.name();
    let qualified = name.as_ref();
    let prefix = qualified
        .iter()
        .position(|byte| *byte == b':')
        .map(|position| &qualified[..position])
        .unwrap_or_default();
    let local = namespace_bindings(element)?;
    let namespace = local
        .iter()
        .rev()
        .chain(inherited.iter().rev())
        .find(|(candidate, _)| candidate.as_slice() == prefix)
        .map(|(_, namespace)| namespace.as_str());
    Ok(namespace == Some(expected))
}

fn attribute_value(element: &BytesStart<'_>, name: &[u8]) -> Result<Option<String>> {
    for attribute in element.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        if attribute.key.as_ref() == name {
            return Ok(Some(
                attribute
                    .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())
                    .map_err(OxmlError::from)?
                    .into_owned(),
            ));
        }
    }
    Ok(None)
}

fn capture_root_attributes(start: &BytesStart<'_>) -> Result<RootAttributes> {
    let mut attributes = Vec::new();
    let mut namespaces = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        let name = std::str::from_utf8(attribute.key.as_ref())
            .map_err(OxmlError::from)?
            .to_owned();
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())
            .map_err(OxmlError::from)?
            .into_owned();
        if name == "xmlns:c" || name == "xmlns:a" || name == "xmlns:r" {
            continue;
        }
        if name == "xmlns" || name.starts_with("xmlns:") {
            namespaces.push((name, value));
        } else {
            attributes.push((name, value));
        }
    }
    Ok((attributes, namespaces))
}

fn capture_attributes(start: &BytesStart<'_>) -> Result<XmlAttributes> {
    let mut attributes = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        attributes.push((
            std::str::from_utf8(attribute.key.as_ref())
                .map_err(OxmlError::from)?
                .to_owned(),
            attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())
                .map_err(OxmlError::from)?
                .into_owned(),
        ));
    }
    Ok(attributes)
}

fn capture_attributes_except(start: &BytesStart<'_>, excluded: &[u8]) -> Result<XmlAttributes> {
    let mut attributes = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        if attribute.key.as_ref() == excluded {
            continue;
        }
        attributes.push((
            std::str::from_utf8(attribute.key.as_ref())
                .map_err(OxmlError::from)?
                .to_owned(),
            attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())
                .map_err(OxmlError::from)?
                .into_owned(),
        ));
    }
    Ok(attributes)
}

fn capture_attributes_excluding(
    start: &BytesStart<'_>,
    excluded: &[&[u8]],
) -> Result<XmlAttributes> {
    let mut attributes = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        if excluded.contains(&attribute.key.as_ref()) {
            continue;
        }
        attributes.push((
            std::str::from_utf8(attribute.key.as_ref())
                .map_err(OxmlError::from)?
                .to_owned(),
            attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())
                .map_err(OxmlError::from)?
                .into_owned(),
        ));
    }
    Ok(attributes)
}

fn push_attributes(start: &mut BytesStart<'_>, attributes: &[(String, String)]) {
    for (name, value) in attributes {
        start.push_attribute((name.as_str(), value.as_str()));
    }
}

fn write_scalar<W: Write>(
    writer: &mut Writer<W>,
    name: &str,
    value: &str,
    markup: Option<&ScalarMarkup>,
) -> Result<()> {
    let mut start = BytesStart::new(name);
    start.push_attribute(("val", value));
    if let Some(markup) = markup {
        push_attributes(&mut start, &markup.raw_attributes);
    }
    let has_content = markup.is_some_and(|markup| !markup.raw_content.is_empty());
    if !has_content {
        writer
            .write_event(Event::Empty(start))
            .map_err(OxmlError::from)?;
        return Ok(());
    }
    writer
        .write_event(Event::Start(start))
        .map_err(OxmlError::from)?;
    if let Some(markup) = markup {
        for raw in &markup.raw_content {
            writer.get_mut().write_all(raw).map_err(OxmlError::from)?;
        }
    }
    writer
        .write_event(Event::End(BytesEnd::new(name)))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn capture_event(event: Event<'_>) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Vec::new());
    writer
        .write_event(event.into_owned())
        .map_err(OxmlError::from)?;
    Ok(writer.into_inner())
}

fn emit_raw<'a, W: Write>(
    writer: &mut Writer<W>,
    children: impl Iterator<Item = &'a [u8]>,
) -> Result<()> {
    for child in children {
        writer.get_mut().write_all(child).map_err(OxmlError::from)?;
    }
    Ok(())
}

fn invalid_attribute(element: &str, attribute: &str, value: String) -> ChartError {
    ChartError::InvalidAttribute {
        element: element.to_owned(),
        attribute: attribute.to_owned(),
        value,
    }
}

fn element_name(element: &BytesStart<'_>) -> String {
    String::from_utf8_lossy(element.name().as_ref()).into_owned()
}

fn missing_end(element: &str) -> ChartError {
    ChartError::Xml(OxmlError::MissingElement(format!("closing {element}")))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use oxml_core::raw_xml::{capture_element, capture_empty_element};
    use oxml_layout::{LayoutResult, PageFrame, PathCommand, Point, PositionedElement, Rect};
    use oxml_opc::OpcPackage;
    use quick_xml::Reader;
    use quick_xml::events::Event;

    use super::{
        A_NS, Axis, AxisData, AxisId, AxisKind, AxisPosition, BarDirection, BarGrouping, C_NS,
        CT_ChartSpace, CT_DLbls, CT_ShapeProperties, CT_TextBody, ChartGeometry, DataLabelPosition,
        DispBlanksAs, Grouping, NumberFormat, NumericData, Orientation, Plot, R_NS, ScatterStyle,
        Series, StringRef, TickLabelPosition, TickMark, capture_event, local_name,
        matches_local_name, render_geometry,
    };

    const MANIFEST: &str = include_str!("../../../scripts/pptx-corpus-manifest.tsv");
    const EXPECTED_DECKS: usize = 50;
    const LIBREOFFICE_VERSION: &str =
        "LibreOffice 26.2.5.2 cd7284b4cbbfeb507e630c1aac019f4157393acb";
    const PDFTOTEXT_VERSION: &str = "pdftotext version 26.01.0";
    const PDFTOPPM_VERSION: &str = "pdftoppm version 26.01.0";
    const PLOT_RENDER_NORMALIZED_MAE_THRESHOLD: f64 = 0.0;

    #[test]
    fn bar_chart_rasterises_at_computed_positions() {
        let chart = CT_ChartSpace::from_xml(chart_with_plot(&bar_plot("")).as_bytes()).unwrap();
        let geometry = render_geometry(
            &chart.chart,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 140.0,
            },
        )
        .unwrap();
        assert_eq!(
            geometry.plot_bounds,
            Rect {
                x: 36.0,
                y: 12.0,
                width: 152.0,
                height: 100.0,
            }
        );
        let bounds = path_bounds(&geometry);
        assert_eq!(bounds.len(), 2);
        assert_rect_near(
            bounds[0],
            Rect {
                x: 58.8,
                y: 62.0,
                width: 30.4,
                height: 50.0,
            },
        );
        assert_rect_near(
            bounds[1],
            Rect {
                x: 134.8,
                y: 12.0,
                width: 30.4,
                height: 100.0,
            },
        );

        let layout = LayoutResult::new(
            vec![PageFrame::new(1, 200.0, 140.0, geometry.elements)],
            Vec::new(),
            None,
            Vec::new(),
        );
        let png = oxml_pdf::render_page_to_png(&layout, 0, 72.0).unwrap();
        let pixmap = tiny_skia::Pixmap::decode_png(&png).unwrap();
        for (x, y) in [(74, 87), (150, 37)] {
            let pixel = pixmap.pixel(x, y).unwrap();
            assert_ne!((pixel.red(), pixel.green(), pixel.blue()), (255, 255, 255));
        }
    }

    #[test]
    fn bar_geometry_handles_direction_grouping_gap_and_overlap() {
        let mut chart = CT_ChartSpace::from_xml(chart_with_plot(&bar_plot("")).as_bytes()).unwrap();
        let Plot::Bar {
            overlap, series, ..
        } = &mut chart.chart.plot_area.plots_mut().unwrap()[0]
        else {
            panic!("expected bar plot");
        };
        let mut second = series[0].clone();
        second.index = 1;
        second.order = 1;
        second.values.values = vec![2.0, 1.0];
        series.push(second);

        *overlap = 25;
        let clustered = render_geometry(&chart.chart, chart_bounds()).unwrap();
        let clustered_bounds = path_bounds(&clustered);
        let clustered_width = 30.4 / 1.75;
        let clustered_advance = clustered_width * 0.75;
        assert_rects_near(
            &clustered_bounds,
            &[
                Rect {
                    x: 58.8,
                    y: 62.0,
                    width: clustered_width,
                    height: 50.0,
                },
                Rect {
                    x: 134.8,
                    y: 12.0,
                    width: clustered_width,
                    height: 100.0,
                },
                Rect {
                    x: 58.8 + clustered_advance,
                    y: 12.0,
                    width: clustered_width,
                    height: 100.0,
                },
                Rect {
                    x: 134.8 + clustered_advance,
                    y: 62.0,
                    width: clustered_width,
                    height: 50.0,
                },
            ],
        );

        let Plot::Bar { direction, .. } = &mut chart.chart.plot_area.plots_mut().unwrap()[0] else {
            unreachable!()
        };
        *direction = BarDirection::Bar;
        let horizontal = path_bounds(&render_geometry(&chart.chart, chart_bounds()).unwrap());
        let horizontal_width = 20.0 / 1.75;
        let horizontal_advance = horizontal_width * 0.75;
        assert_rects_near(
            &horizontal,
            &[
                Rect {
                    x: 36.0,
                    y: 27.0,
                    width: 76.0,
                    height: horizontal_width,
                },
                Rect {
                    x: 36.0,
                    y: 77.0,
                    width: 152.0,
                    height: horizontal_width,
                },
                Rect {
                    x: 36.0,
                    y: 27.0 + horizontal_advance,
                    width: 152.0,
                    height: horizontal_width,
                },
                Rect {
                    x: 36.0,
                    y: 77.0 + horizontal_advance,
                    width: 76.0,
                    height: horizontal_width,
                },
            ],
        );

        let Plot::Bar {
            direction,
            grouping,
            ..
        } = &mut chart.chart.plot_area.plots_mut().unwrap()[0]
        else {
            unreachable!()
        };
        *direction = BarDirection::Column;
        *grouping = BarGrouping::Stacked;
        let stacked = path_bounds(&render_geometry(&chart.chart, chart_bounds()).unwrap());
        assert_rects_near(
            &stacked,
            &[
                Rect {
                    x: 58.8,
                    y: 112.0 - 100.0 / 3.0,
                    width: 30.4,
                    height: 100.0 / 3.0,
                },
                Rect {
                    x: 134.8,
                    y: 112.0 - 200.0 / 3.0,
                    width: 30.4,
                    height: 200.0 / 3.0,
                },
                Rect {
                    x: 58.8,
                    y: 12.0,
                    width: 30.4,
                    height: 200.0 / 3.0,
                },
                Rect {
                    x: 134.8,
                    y: 12.0,
                    width: 30.4,
                    height: 100.0 / 3.0,
                },
            ],
        );

        let Plot::Bar {
            grouping, series, ..
        } = &mut chart.chart.plot_area.plots_mut().unwrap()[0]
        else {
            unreachable!()
        };
        *grouping = BarGrouping::PercentStacked;
        series[1].values.values = vec![2.0, 2.0];
        let percent = path_bounds(&render_geometry(&chart.chart, chart_bounds()).unwrap());
        assert_rects_near(
            &percent,
            &[
                Rect {
                    x: 58.8,
                    y: 112.0 - 100.0 / 3.0,
                    width: 30.4,
                    height: 100.0 / 3.0,
                },
                Rect {
                    x: 134.8,
                    y: 62.0,
                    width: 30.4,
                    height: 50.0,
                },
                Rect {
                    x: 58.8,
                    y: 12.0,
                    width: 30.4,
                    height: 200.0 / 3.0,
                },
                Rect {
                    x: 134.8,
                    y: 12.0,
                    width: 30.4,
                    height: 50.0,
                },
            ],
        );
    }

    #[test]
    fn line_scatter_and_radar_emit_paths_and_markers() {
        let line = CT_ChartSpace::from_xml(chart_with_plot(&line_plot("")).as_bytes()).unwrap();
        let line_geometry = render_geometry(&line.chart, chart_bounds()).unwrap();
        assert_path_points(
            path_commands(&line_geometry)[0],
            &[Point { x: 74.0, y: 62.0 }, Point { x: 150.0, y: 12.0 }],
            false,
        );
        assert_rects_near(
            &path_bounds(&line_geometry)[1..],
            &[
                Rect {
                    x: 71.5,
                    y: 59.5,
                    width: 5.0,
                    height: 5.0,
                },
                Rect {
                    x: 147.5,
                    y: 9.5,
                    width: 5.0,
                    height: 5.0,
                },
            ],
        );

        let (scatter_plot, _) = remaining_plot_fixtures()
            .into_iter()
            .find(|(kind, _, _)| *kind == "scatterChart")
            .map(|(_, plot, axes)| (plot, axes))
            .unwrap();
        let mut scatter =
            CT_ChartSpace::from_xml(chart_with_optional_axes(&scatter_plot, true).as_bytes())
                .unwrap();
        let Plot::Scatter { style, .. } = &mut scatter.chart.plot_area.plots_mut().unwrap()[0]
        else {
            unreachable!()
        };
        *style = ScatterStyle::LineMarker;
        let scatter_geometry = render_geometry(&scatter.chart, chart_bounds()).unwrap();
        assert_path_points(
            path_commands(&scatter_geometry)[0],
            &[Point { x: 36.0, y: 112.0 }, Point { x: 188.0, y: 12.0 }],
            false,
        );
        assert_rects_near(
            &path_bounds(&scatter_geometry)[1..],
            &[
                Rect {
                    x: 33.5,
                    y: 109.5,
                    width: 5.0,
                    height: 5.0,
                },
                Rect {
                    x: 185.5,
                    y: 9.5,
                    width: 5.0,
                    height: 5.0,
                },
            ],
        );

        let (radar_plot, _) = remaining_plot_fixtures()
            .into_iter()
            .find(|(kind, _, _)| *kind == "radarChart")
            .map(|(_, plot, axes)| (plot, axes))
            .unwrap();
        let radar = CT_ChartSpace::from_xml(chart_with_optional_axes(&radar_plot, true).as_bytes())
            .unwrap();
        let radar_geometry = render_geometry(&radar.chart, chart_bounds()).unwrap();
        assert_path_points(
            path_commands(&radar_geometry)[0],
            &[Point { x: 112.0, y: 37.0 }, Point { x: 112.0, y: 112.0 }],
            true,
        );
        assert_rects_near(
            &path_bounds(&radar_geometry)[1..],
            &[
                Rect {
                    x: 109.5,
                    y: 34.5,
                    width: 5.0,
                    height: 5.0,
                },
                Rect {
                    x: 109.5,
                    y: 109.5,
                    width: 5.0,
                    height: 5.0,
                },
            ],
        );
    }

    #[test]
    fn pie_doughnut_and_area_emit_closed_paths() {
        let fixtures = remaining_plot_fixtures();
        let pie_plot = fixtures
            .iter()
            .find(|(kind, _, _)| *kind == "pieChart")
            .unwrap();
        let pie =
            CT_ChartSpace::from_xml(chart_with_optional_axes(&pie_plot.1, pie_plot.2).as_bytes())
                .unwrap();
        let pie_geometry = render_geometry(&pie.chart, chart_bounds()).unwrap();
        let pie_commands = path_commands(&pie_geometry);
        assert_eq!(pie_commands.len(), 2);
        let [
            PathCommand::MoveTo(center),
            PathCommand::LineTo(start),
            ..,
            PathCommand::CurveTo { to: end, .. },
            PathCommand::Close,
        ] = pie_commands[0]
        else {
            panic!("expected closed cubic pie wedge");
        };
        assert_point_near(*center, Point { x: 112.0, y: 62.0 });
        assert_point_near(
            *start,
            Point {
                x: 137.0,
                y: 62.0 - 25.0 * 3.0_f64.sqrt(),
            },
        );
        assert_point_near(
            *end,
            Point {
                x: 137.0,
                y: 62.0 + 25.0 * 3.0_f64.sqrt(),
            },
        );

        let doughnut_plot = fixtures
            .iter()
            .find(|(kind, _, _)| *kind == "doughnutChart")
            .unwrap();
        let doughnut = CT_ChartSpace::from_xml(
            chart_with_optional_axes(&doughnut_plot.1, doughnut_plot.2).as_bytes(),
        )
        .unwrap();
        let doughnut_geometry = render_geometry(&doughnut.chart, chart_bounds()).unwrap();
        let doughnut_commands = path_commands(&doughnut_geometry);
        assert_eq!(doughnut_commands.len(), 2);
        let commands = doughnut_commands[0];
        assert_eq!(commands.len(), 7);
        let PathCommand::MoveTo(outer_start) = commands[0] else {
            panic!("expected doughnut outer start");
        };
        let PathCommand::LineTo(inner_end) = commands[3] else {
            panic!("expected doughnut inner-radius join");
        };
        let PathCommand::CurveTo {
            to: inner_start, ..
        } = commands[5]
        else {
            panic!("expected reverse inner arc");
        };
        let diagonal = 25.0 * 2.0_f64.sqrt();
        assert_point_near(
            outer_start,
            Point {
                x: 112.0 + diagonal,
                y: 62.0 - diagonal,
            },
        );
        assert_point_near(
            inner_end,
            Point {
                x: 112.0 + 30.0 * 75.0_f64.to_radians().cos(),
                y: 62.0 + 30.0 * 75.0_f64.to_radians().sin(),
            },
        );
        let inner_diagonal = 15.0 * 2.0_f64.sqrt();
        assert_point_near(
            inner_start,
            Point {
                x: 112.0 + inner_diagonal,
                y: 62.0 - inner_diagonal,
            },
        );
        assert_eq!(commands.last(), Some(&PathCommand::Close));

        let area_plot = fixtures
            .iter()
            .find(|(kind, _, _)| *kind == "areaChart")
            .unwrap();
        let mut area =
            CT_ChartSpace::from_xml(chart_with_optional_axes(&area_plot.1, area_plot.2).as_bytes())
                .unwrap();
        let area_geometry = render_geometry(&area.chart, chart_bounds()).unwrap();
        assert_path_points(
            path_commands(&area_geometry)[0],
            &[
                Point { x: 74.0, y: 62.0 },
                Point { x: 150.0, y: 12.0 },
                Point { x: 150.0, y: 112.0 },
                Point { x: 74.0, y: 112.0 },
            ],
            true,
        );

        let Plot::Area {
            grouping, series, ..
        } = &mut area.chart.plot_area.plots_mut().unwrap()[0]
        else {
            unreachable!()
        };
        *grouping = Grouping::Stacked;
        let mut second = series[0].clone();
        second.index = 1;
        second.order = 1;
        second.values.values = vec![2.0, 1.0];
        series.push(second);
        let stacked_area = render_geometry(&area.chart, chart_bounds()).unwrap();
        assert_path_points(
            path_commands(&stacked_area)[1],
            &[
                Point { x: 74.0, y: 12.0 },
                Point { x: 150.0, y: 12.0 },
                Point {
                    x: 150.0,
                    y: 112.0 - 200.0 / 3.0,
                },
                Point {
                    x: 74.0,
                    y: 112.0 - 100.0 / 3.0,
                },
            ],
            true,
        );
    }

    #[test]
    fn sparse_cache_indexes_preserve_slots_and_scatter_pairing() {
        let series = sparse_category_series();
        let bar = format!(
            r#"<q:barChart><q:barDir val="col"/><q:grouping val="clustered"/>{series}<q:gapWidth val="150"/><q:overlap val="0"/><q:axId val="-1884094432"/><q:axId val="-1884097184"/></q:barChart>"#
        );
        let bar = CT_ChartSpace::from_xml(chart_with_plot(&bar).as_bytes()).unwrap();
        assert_rects_near(
            &path_bounds(&render_geometry(&bar.chart, chart_bounds()).unwrap()),
            &[
                Rect {
                    x: 51.2,
                    y: 62.0,
                    width: 304.0 / 15.0,
                    height: 50.0,
                },
                Rect {
                    x: 36.0 + 304.0 / 3.0 + 15.2,
                    y: 12.0,
                    width: 304.0 / 15.0,
                    height: 100.0,
                },
            ],
        );

        let line = format!(
            r#"<q:lineChart><q:grouping val="standard"/>{series}<q:marker val="0"/><q:smooth val="0"/><q:axId val="-1884094432"/><q:axId val="-1884097184"/></q:lineChart>"#
        );
        let mut line = CT_ChartSpace::from_xml(chart_with_plot(&line).as_bytes()).unwrap();
        let first = Point {
            x: 36.0 + 76.0 / 3.0,
            y: 62.0,
        };
        let missing = Point { x: 112.0, y: 112.0 };
        let last = Point {
            x: 36.0 + 380.0 / 3.0,
            y: 12.0,
        };
        let gap = render_geometry(&line.chart, chart_bounds()).unwrap();
        let gap_commands = path_commands(&gap);
        assert_eq!(gap_commands.len(), 2);
        assert_path_points(gap_commands[0], &[first], false);
        assert_path_points(gap_commands[1], &[last], false);
        line.chart.disp_blanks_as = DispBlanksAs::Zero;
        assert_path_points(
            path_commands(&render_geometry(&line.chart, chart_bounds()).unwrap())[0],
            &[first, missing, last],
            false,
        );
        line.chart.disp_blanks_as = DispBlanksAs::Span;
        assert_path_points(
            path_commands(&render_geometry(&line.chart, chart_bounds()).unwrap())[0],
            &[first, last],
            false,
        );

        let area = format!(
            r#"<q:areaChart><q:grouping val="standard"/>{series}<q:axId val="-1884094432"/><q:axId val="-1884097184"/></q:areaChart>"#
        );
        let mut area = CT_ChartSpace::from_xml(chart_with_plot(&area).as_bytes()).unwrap();
        let area_gap = render_geometry(&area.chart, chart_bounds()).unwrap();
        let area_gap_commands = path_commands(&area_gap);
        assert_eq!(area_gap_commands.len(), 2);
        assert_path_points(
            area_gap_commands[0],
            &[
                first,
                Point {
                    x: first.x,
                    y: 112.0,
                },
            ],
            true,
        );
        assert_path_points(
            area_gap_commands[1],
            &[
                last,
                Point {
                    x: last.x,
                    y: 112.0,
                },
            ],
            true,
        );
        area.chart.disp_blanks_as = DispBlanksAs::Zero;
        assert_path_points(
            path_commands(&render_geometry(&area.chart, chart_bounds()).unwrap())[0],
            &[
                first,
                missing,
                last,
                Point {
                    x: last.x,
                    y: 112.0,
                },
                missing,
                Point {
                    x: first.x,
                    y: 112.0,
                },
            ],
            true,
        );
        area.chart.disp_blanks_as = DispBlanksAs::Span;
        assert_path_points(
            path_commands(&render_geometry(&area.chart, chart_bounds()).unwrap())[0],
            &[
                first,
                last,
                Point {
                    x: last.x,
                    y: 112.0,
                },
                Point {
                    x: first.x,
                    y: 112.0,
                },
            ],
            true,
        );

        let radar = format!(
            r#"<q:radarChart><q:radarStyle val="standard"/>{series}<q:axId val="-1884094432"/><q:axId val="-1884097184"/></q:radarChart>"#
        );
        let radar = CT_ChartSpace::from_xml(chart_with_plot(&radar).as_bytes()).unwrap();
        assert_path_points(
            path_commands(&render_geometry(&radar.chart, chart_bounds()).unwrap())[0],
            &[
                Point { x: 112.0, y: 37.0 },
                Point {
                    x: 112.0 - 25.0 * 3.0_f64.sqrt(),
                    y: 87.0,
                },
            ],
            true,
        );

        let scatter = format!(
            r#"<q:scatterChart><q:scatterStyle val="lineMarker"/>{}<q:axId val="-1884094432"/><q:axId val="-1884097184"/></q:scatterChart>"#,
            sparse_scatter_series()
        );
        let mut scatter = CT_ChartSpace::from_xml(chart_with_plot(&scatter).as_bytes()).unwrap();
        let scatter_geometry = render_geometry(&scatter.chart, chart_bounds()).unwrap();
        assert_eq!(geometry_children(&scatter_geometry).len(), 4);
        let gap_commands = path_commands(&scatter_geometry);
        assert_path_points(gap_commands[0], &[Point { x: 36.0, y: 112.0 }], false);
        assert_path_points(gap_commands[1], &[Point { x: 188.0, y: 12.0 }], false);

        scatter.chart.disp_blanks_as = DispBlanksAs::Zero;
        let zero = render_geometry(&scatter.chart, chart_bounds()).unwrap();
        assert_path_points(
            path_commands(&zero)[0],
            &[
                Point { x: 74.0, y: 87.0 },
                Point { x: 36.0, y: 62.0 },
                Point { x: 150.0, y: 112.0 },
                Point { x: 188.0, y: 12.0 },
            ],
            false,
        );

        scatter.chart.disp_blanks_as = DispBlanksAs::Span;
        assert_path_points(
            path_commands(&render_geometry(&scatter.chart, chart_bounds()).unwrap())[0],
            &[Point { x: 36.0, y: 112.0 }, Point { x: 188.0, y: 12.0 }],
            false,
        );
    }

    #[test]
    fn geometry_rejects_invalid_bounds_and_opaque_plots() {
        let mut chart = CT_ChartSpace::from_xml(chart_with_plot(&bar_plot("")).as_bytes()).unwrap();
        for bounds in [
            Rect {
                x: 0.0,
                y: 0.0,
                width: f64::NAN,
                height: 100.0,
            },
            Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 100.0,
            },
            Rect {
                x: 0.0,
                y: 0.0,
                width: 40.0,
                height: 30.0,
            },
        ] {
            assert!(render_geometry(&chart.chart, bounds).is_err());
        }

        let Plot::Bar { series, .. } = &mut chart.chart.plot_area.plots_mut().unwrap()[0] else {
            unreachable!()
        };
        series[0].values.values.clear();
        assert!(render_geometry(&chart.chart, chart_bounds()).is_err());

        let opaque = CT_ChartSpace::from_xml(
            chart_with_plot(r#"<q:bar3DChart><q:barDir val="col"/></q:bar3DChart>"#).as_bytes(),
        )
        .unwrap();
        let error = render_geometry(&opaque.chart, chart_bounds()).unwrap_err();
        assert!(error.to_string().contains("unsupported or combination"));

        let (scatter_plot, _) = remaining_plot_fixtures()
            .into_iter()
            .find(|(kind, _, _)| *kind == "scatterChart")
            .map(|(_, plot, axes)| (plot, axes))
            .unwrap();
        let mut scatter =
            CT_ChartSpace::from_xml(chart_with_optional_axes(&scatter_plot, true).as_bytes())
                .unwrap();
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let Plot::Scatter { series, .. } = &mut scatter.chart.plot_area.plots_mut().unwrap()[0]
            else {
                unreachable!()
            };
            let Some(AxisData::Numeric(x_values)) = &mut series[0].categories else {
                unreachable!()
            };
            x_values.values[0] = invalid;
            let error = render_geometry(&scatter.chart, chart_bounds()).unwrap_err();
            assert!(error.to_string().contains("xVal"));
        }
    }

    #[test]
    fn finite_extremes_never_produce_nonfinite_geometry() {
        let mut line = CT_ChartSpace::from_xml(chart_with_plot(&line_plot("")).as_bytes()).unwrap();
        let Plot::Line { series, .. } = &mut line.chart.plot_area.plots_mut().unwrap()[0] else {
            unreachable!()
        };
        series[0].values.values = vec![-f64::MAX, f64::MAX];
        let extreme_line = render_geometry(&line.chart, chart_bounds()).unwrap();
        assert_path_points(
            path_commands(&extreme_line)[0],
            &[Point { x: 74.0, y: 112.0 }, Point { x: 150.0, y: 12.0 }],
            false,
        );

        let mut stacked =
            CT_ChartSpace::from_xml(chart_with_plot(&bar_plot("")).as_bytes()).unwrap();
        let Plot::Bar {
            grouping, series, ..
        } = &mut stacked.chart.plot_area.plots_mut().unwrap()[0]
        else {
            unreachable!()
        };
        *grouping = BarGrouping::Stacked;
        series[0].values.values = vec![f64::MAX, 1.0];
        let mut second = series[0].clone();
        second.index = 1;
        second.order = 1;
        series.push(second);
        let error = render_geometry(&stacked.chart, chart_bounds()).unwrap_err();
        assert!(error.to_string().contains("stacked value total"));

        let Plot::Bar { grouping, .. } = &mut stacked.chart.plot_area.plots_mut().unwrap()[0]
        else {
            unreachable!()
        };
        *grouping = BarGrouping::PercentStacked;
        let error = render_geometry(&stacked.chart, chart_bounds()).unwrap_err();
        assert!(error.to_string().contains("stacked percentage total"));

        let (pie_plot, _) = remaining_plot_fixtures()
            .into_iter()
            .find(|(kind, _, _)| *kind == "pieChart")
            .map(|(_, plot, axes)| (plot, axes))
            .unwrap();
        let mut pie =
            CT_ChartSpace::from_xml(chart_with_optional_axes(&pie_plot, false).as_bytes()).unwrap();
        let Plot::Pie { series, .. } = &mut pie.chart.plot_area.plots_mut().unwrap()[0] else {
            unreachable!()
        };
        series[0].values.values = vec![f64::MAX, f64::MAX];
        let error = render_geometry(&pie.chart, chart_bounds()).unwrap_err();
        assert!(error.to_string().contains("pie value total"));

        let (scatter_plot, _) = remaining_plot_fixtures()
            .into_iter()
            .find(|(kind, _, _)| *kind == "scatterChart")
            .map(|(_, plot, axes)| (plot, axes))
            .unwrap();
        let mut scatter =
            CT_ChartSpace::from_xml(chart_with_optional_axes(&scatter_plot, true).as_bytes())
                .unwrap();
        let Plot::Scatter { style, series, .. } =
            &mut scatter.chart.plot_area.plots_mut().unwrap()[0]
        else {
            unreachable!()
        };
        *style = ScatterStyle::Line;
        let Some(AxisData::Numeric(x_values)) = &mut series[0].categories else {
            unreachable!()
        };
        x_values.values = vec![-f64::MAX, f64::MAX];
        series[0].values.values = vec![-f64::MAX, f64::MAX];
        let extreme_scatter = render_geometry(&scatter.chart, chart_bounds()).unwrap();
        assert_path_points(
            path_commands(&extreme_scatter)[0],
            &[Point { x: 36.0, y: 112.0 }, Point { x: 188.0, y: 12.0 }],
            false,
        );
    }

    #[test]
    fn geometry_is_backend_neutral_and_deterministic() {
        let chart = CT_ChartSpace::from_xml(chart_with_plot(&bar_plot("")).as_bytes()).unwrap();
        let first = render_geometry(&chart.chart, chart_bounds()).unwrap();
        let second = render_geometry(&chart.chart, chart_bounds()).unwrap();
        assert_eq!(
            format!("{:?}", first.elements),
            format!("{:?}", second.elements)
        );

        let raster = |geometry: ChartGeometry| {
            let layout = LayoutResult::new(
                vec![PageFrame::new(1, 200.0, 140.0, geometry.elements)],
                Vec::new(),
                None,
                Vec::new(),
            );
            oxml_pdf::render_page_to_png(&layout, 0, 72.0).unwrap()
        };
        assert_eq!(raster(first), raster(second));
    }

    fn chart_bounds() -> Rect {
        Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 140.0,
        }
    }

    fn geometry_children(geometry: &ChartGeometry) -> &[PositionedElement] {
        let [PositionedElement::Group(group)] = geometry.elements.as_slice() else {
            panic!("chart geometry must contain one group");
        };
        &group.children
    }

    fn path_commands(geometry: &ChartGeometry) -> Vec<&[PathCommand]> {
        geometry_children(geometry)
            .iter()
            .map(|element| {
                let PositionedElement::Path(path) = element else {
                    panic!("chart geometry child must be a path");
                };
                path.path.commands.as_slice()
            })
            .collect()
    }

    fn path_bounds(geometry: &ChartGeometry) -> Vec<Rect> {
        geometry_children(geometry)
            .iter()
            .map(|element| {
                let PositionedElement::Path(path) = element else {
                    panic!("chart geometry child must be a path");
                };
                path.path.bounds().expect("chart path has bounds")
            })
            .collect()
    }

    fn assert_rect_near(actual: Rect, expected: Rect) {
        assert_near(actual.x, expected.x);
        assert_near(actual.y, expected.y);
        assert_near(actual.width, expected.width);
        assert_near(actual.height, expected.height);
    }

    fn assert_rects_near(actual: &[Rect], expected: &[Rect]) {
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().copied().zip(expected.iter().copied()) {
            assert_rect_near(actual, expected);
        }
    }

    fn assert_path_points(commands: &[PathCommand], expected: &[Point], closed: bool) {
        let points: Vec<_> = commands
            .iter()
            .filter_map(|command| match command {
                PathCommand::MoveTo(point) | PathCommand::LineTo(point) => Some(*point),
                PathCommand::Close => None,
                PathCommand::CurveTo { .. } => panic!("expected polygonal path"),
            })
            .collect();
        assert_eq!(points.len(), expected.len());
        for (actual, expected) in points.into_iter().zip(expected.iter().copied()) {
            assert_point_near(actual, expected);
        }
        assert_eq!(commands.last() == Some(&PathCommand::Close), closed);
    }

    fn assert_point_near(actual: Point, expected: Point) {
        assert_near(actual.x, expected.x);
        assert_near(actual.y, expected.y);
    }

    fn assert_near(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn remaining_v1_plots_round_trip_and_render() {
        for (kind, plot, axes) in remaining_plot_fixtures() {
            let xml = chart_with_optional_axes(&plot, axes);
            let parsed = CT_ChartSpace::from_xml(xml.as_bytes())
                .unwrap_or_else(|error| panic!("{kind}: parse failed: {error}"));
            assert_eq!(parsed.chart.plot_area.plots().unwrap().len(), 1);
            assert_eq!(
                parsed.chart.plot_area.axes().unwrap().len(),
                usize::from(axes) * 2
            );
            let written = parsed
                .to_xml()
                .unwrap_or_else(|error| panic!("{kind}: write failed: {error}"));
            assert_eq!(
                parsed,
                CT_ChartSpace::from_xml(&written)
                    .unwrap_or_else(|error| panic!("{kind}: reparse failed: {error}"))
            );
        }
        if let Some(corpus) = require_or_skip_corpus() {
            verify_remaining_plot_viewer_gate(&corpus);
        }
    }

    #[test]
    fn remaining_plot_families_write_fixed_prefixes_in_schema_order() {
        for (kind, plot, axes) in remaining_plot_fixtures() {
            let parsed =
                CT_ChartSpace::from_xml(chart_with_optional_axes(&plot, axes).as_bytes()).unwrap();
            let written = String::from_utf8(parsed.to_xml().unwrap()).unwrap();
            assert!(written.contains(&format!("<c:{kind}")));
            assert!(!written.contains(&format!("<q:{kind}")));
            let series = written.find("<c:ser").unwrap();
            let labels = written.find("<c:dLbls").unwrap();
            assert!(series < labels, "{kind}: series must precede labels");
            if matches!(kind, "areaChart" | "scatterChart" | "radarChart") {
                let property = match kind {
                    "areaChart" => "<c:grouping",
                    "scatterChart" => "<c:scatterStyle",
                    "radarChart" => "<c:radarStyle",
                    _ => unreachable!(),
                };
                assert!(written.find(property).unwrap() < series);
                assert!(labels < written.find("<c:axId").unwrap());
            } else {
                assert!(labels < written.find("<c:firstSliceAng").unwrap());
            }
            if kind == "doughnutChart" {
                assert!(
                    written.find("<c:firstSliceAng").unwrap()
                        < written.find("<c:holeSize").unwrap()
                );
            }
        }
    }

    #[test]
    fn scatter_series_map_numeric_categories_and_values_to_x_and_y() {
        let plot = remaining_plot_fixtures()
            .into_iter()
            .find(|(kind, _, _)| *kind == "scatterChart")
            .unwrap()
            .1;
        let mut parsed =
            CT_ChartSpace::from_xml(chart_with_optional_axes(&plot, true).as_bytes()).unwrap();
        let standalone_series = {
            let Plot::Scatter { series, .. } = &mut parsed.chart.plot_area.plots_mut().unwrap()[0]
            else {
                panic!("expected scatter plot");
            };
            let Some(AxisData::Numeric(x_values)) = &mut series[0].categories else {
                panic!("expected numeric x values");
            };
            x_values.values[0] = 3.5;
            series[0].values.values[0] = 7.5;
            series[0].clone()
        };
        let written = String::from_utf8(parsed.to_xml().unwrap()).unwrap();
        assert!(written.contains("<c:xVal>"));
        assert!(written.contains("<c:yVal>"));
        assert!(!written.contains("<c:cat>"));
        assert!(!written.contains("<c:val>"));
        assert!(written.contains("<c:v>3.5</c:v>"));
        assert!(written.contains("<c:v>7.5</c:v>"));
        let standalone = String::from_utf8(standalone_series.to_xml().unwrap()).unwrap();
        assert!(standalone.contains("<c:xVal>"));
        assert!(standalone.contains("<c:yVal>"));
        assert!(!standalone.contains("<c:cat>"));
        assert!(!standalone.contains("<c:val>"));
    }

    #[test]
    fn malformed_remaining_plots_return_errors_without_panicking() {
        let series = plot_series(0);
        let scatter = numeric_plot_series(0);
        let opaque_bubble = series.replace(
            "</q:val></q:ser>",
            "</q:val><q:bubbleSize><x:opaque/></q:bubbleSize></q:ser>",
        );
        let cases = [
            r#"<q:pieChart/>"#.to_owned(),
            format!(r#"<q:pieChart>{series}<q:firstSliceAng val="361"/></q:pieChart>"#),
            format!(
                r#"<q:pieChart>{series}<q:firstSliceAng val="1"/><q:firstSliceAng val="2"/></q:pieChart>"#
            ),
            format!(r#"<q:doughnutChart>{series}<q:holeSize val="9"/></q:doughnutChart>"#),
            format!(
                r#"<q:doughnutChart>{series}<q:holeSize val="50"/><q:holeSize val="60"/></q:doughnutChart>"#
            ),
            format!(
                r#"<q:areaChart><q:grouping val="clustered"/>{series}<q:axId val="1"/><q:axId val="2"/></q:areaChart>"#
            ),
            format!(
                r#"<q:areaChart><q:grouping val="standard"/><q:grouping val="stacked"/>{series}<q:axId val="1"/><q:axId val="2"/></q:areaChart>"#
            ),
            format!(
                r#"<q:scatterChart><q:scatterStyle val="dots"/>{scatter}<q:axId val="1"/><q:axId val="2"/></q:scatterChart>"#
            ),
            format!(
                r#"<q:scatterChart><q:scatterStyle val="marker"/><q:scatterStyle val="line"/>{scatter}<q:axId val="1"/><q:axId val="2"/></q:scatterChart>"#
            ),
            format!(
                r#"<q:radarChart><q:radarStyle val="smooth"/>{series}<q:axId val="1"/><q:axId val="2"/></q:radarChart>"#
            ),
            format!(
                r#"<q:radarChart><q:radarStyle val="marker"/>{series}<q:dLbls/><q:dLbls/><q:axId val="1"/><q:axId val="2"/></q:radarChart>"#
            ),
            format!(r#"<q:pieChart>{series}<q:axId val="1"/></q:pieChart>"#),
            format!(
                r#"<q:areaChart><q:grouping val="standard"/>{series}<q:axId val="1"/></q:areaChart>"#
            ),
            r#"<q:areaChart><q:grouping val="standard"/><q:axId val="1"/><q:axId val="2"/></q:areaChart>"#.to_owned(),
            format!(
                r#"<q:areaChart><q:grouping val="standard"/>{series}<q:axId val="1"/><q:axId val="2"/><q:axId val="3"/></q:areaChart>"#
            ),
            format!(
                r#"<q:scatterChart><q:scatterStyle val="marker"/>{series}<q:axId val="1"/><q:axId val="2"/></q:scatterChart>"#
            ),
            format!(
                r#"<q:scatterChart><q:scatterStyle val="marker"/>{}<q:axId val="1"/><q:axId val="2"/></q:scatterChart>"#,
                scatter
                    .replace("<q:yVal>", "<q:val>")
                    .replace("</q:yVal>", "</q:val>")
            ),
            format!(
                r#"<q:scatterChart><q:scatterStyle val="marker"/>{}<q:axId val="1"/><q:axId val="2"/></q:scatterChart>"#,
                scatter_without_x(0)
            ),
            format!(
                r#"<q:scatterChart><q:scatterStyle val="marker"/>{}<q:axId val="1"/><q:axId val="2"/></q:scatterChart>"#,
                scatter_without_y(0)
            ),
            format!(r#"<q:pieChart>{opaque_bubble}</q:pieChart>"#),
        ];
        for plot in cases {
            let xml = chart_with_optional_axes(&plot, false);
            let result = std::panic::catch_unwind(|| CT_ChartSpace::from_xml(xml.as_bytes()));
            assert!(result.is_ok(), "parser panicked for {plot}");
            assert!(result.unwrap().is_err(), "malformed plot parsed: {plot}");
        }

        let missing_axis = format!(
            r#"<q:areaChart><q:grouping val="standard"/>{series}<q:axId val="-1884094432"/><q:axId val="7"/></q:areaChart>"#
        );
        let missing_axis =
            CT_ChartSpace::from_xml(chart_with_optional_axes(&missing_axis, true).as_bytes())
                .unwrap_err();
        assert!(missing_axis.to_string().contains("missing axis 7"));

        let values =
            NumericData::new("S!$A$1".to_owned(), "General".to_owned(), vec![1.0]).unwrap();
        let mut bubble_series = Series::new(0, 0, values.clone());
        bubble_series.bubble_size = Some(values);
        assert!(Plot::pie(vec![bubble_series]).is_err());
    }

    #[test]
    fn unsupported_plot_families_and_children_remain_byte_preserved() {
        for raw in [
            r#"<q:pie3DChart x:keep="3d"><q:ser><x:opaque/></q:ser></q:pie3DChart>"#,
            r#"<q:ofPieChart x:keep="of"><x:opaque/></q:ofPieChart>"#,
            r#"<q:stockChart x:keep="stock"><x:opaque/></q:stockChart>"#,
            r#"<q:surfaceChart x:keep="surface"><x:opaque/></q:surfaceChart>"#,
            r#"<q:bubbleChart x:keep="bubble"><x:opaque/></q:bubbleChart>"#,
        ] {
            let parsed =
                CT_ChartSpace::from_xml(chart_with_optional_axes(raw, false).as_bytes()).unwrap();
            assert!(parsed.chart.plot_area.plots().is_err());
            let written = parsed.to_xml().unwrap();
            assert!(
                written
                    .windows(raw.len())
                    .any(|window| window == raw.as_bytes())
            );
        }

        let pie = remaining_plot_fixtures()[0].1.clone();
        let preserved = r#"<!--point--><q:dPt x:id="1"><q:explosion val="7"/><x:tail/></q:dPt>"#;
        let plot = pie.replace("<q:dLbls>", &format!("{preserved}<q:dLbls>"));
        let parsed =
            CT_ChartSpace::from_xml(chart_with_optional_axes(&plot, false).as_bytes()).unwrap();
        let written = parsed.to_xml().unwrap();
        assert!(
            written
                .windows(preserved.len())
                .any(|window| window == preserved.as_bytes())
        );

        for (kind, mut plot, axes) in [
            ("pieChart", remaining_plot_fixtures()[0].1.clone(), false),
            ("areaChart", remaining_plot_fixtures()[2].1.clone(), true),
        ] {
            plot = plot.replace(r#"<q:dLbls><q:showVal val="1"/></q:dLbls>"#, "");
            if kind == "pieChart" {
                plot = plot.replace(
                    "</q:extLst></q:pieChart>",
                    "</q:extLst><!--after-ext--></q:pieChart>",
                );
            } else {
                plot = plot.replace(
                    "</q:dropLines><q:axId",
                    "</q:dropLines><!--after-drop-lines--><q:axId",
                );
            }
            let mut parsed =
                CT_ChartSpace::from_xml(chart_with_optional_axes(&plot, axes).as_bytes()).unwrap();
            let labels =
                CT_DLbls::from_xml(format!(r#"<c:dLbls xmlns:c="{C_NS}"/>"#).as_bytes()).unwrap();
            match &mut parsed.chart.plot_area.plots_mut().unwrap()[0] {
                Plot::Pie { data_labels, .. } | Plot::Area { data_labels, .. } => {
                    *data_labels = Some(labels)
                }
                _ => panic!("expected pie or area"),
            }
            let written = String::from_utf8(parsed.to_xml().unwrap()).unwrap();
            assert!(
                written.find("<c:dLbls").unwrap() < written.find("<q:extLst").unwrap(),
                "{kind}: inserted labels must precede the preserved extension"
            );
            if kind == "areaChart" {
                assert!(written.find("<c:dLbls").unwrap() < written.find("<q:dropLines").unwrap());
                assert!(
                    written.find("<q:dropLines").unwrap()
                        < written.find("<!--after-drop-lines-->").unwrap()
                );
            } else {
                assert!(
                    written.find("<q:extLst").unwrap() < written.find("<!--after-ext-->").unwrap()
                );
            }
        }
    }

    #[test]
    fn every_supported_corpus_plot_round_trips_structurally() {
        let Some(corpus) = require_or_skip_corpus() else {
            return;
        };
        verify_fetched_corpus(&corpus);
        let mut pie_count = 0usize;
        let mut bar_count = 0usize;
        let mut line_count = 0usize;
        for path in manifest_paths() {
            let package = OpcPackage::open(corpus.join(path)).unwrap();
            for (part, xml) in &package.parts {
                if !is_chart_part(part) {
                    continue;
                }
                let parsed = CT_ChartSpace::from_xml(xml).unwrap();
                if let Ok(plots) = parsed.chart.plot_area.plots() {
                    for plot in plots {
                        match plot {
                            Plot::Pie { .. } => pie_count += 1,
                            Plot::Bar { .. } => bar_count += 1,
                            Plot::Line { .. } => line_count += 1,
                            _ => {}
                        }
                    }
                }
                let written = parsed.to_xml().unwrap();
                assert_eq!(parsed, CT_ChartSpace::from_xml(&written).unwrap());
            }
        }
        assert_eq!(pie_count, 1, "typed corpus pie coverage changed");
        assert_eq!(bar_count, 11, "typed corpus bar coverage changed");
        assert_eq!(line_count, 2, "typed corpus line coverage changed");
    }

    fn standalone_axis(local: &str, children: &str) -> String {
        format!(
            r#"<c:{local} xmlns:c="{C_NS}" xmlns:a="{A_NS}" xmlns:r="{R_NS}">{children}</c:{local}>"#
        )
    }

    fn minimal_axis(local: &str, id: &str, position: &str, cross_axis: &str) -> String {
        format!(
            r#"<c:{local}><c:axId val="{id}"/><c:scaling/><c:axPos val="{position}"/><c:crossAx val="{cross_axis}"/></c:{local}>"#
        )
    }

    fn chart_with_axes(axes: &[String]) -> String {
        format!(
            r#"<c:chartSpace xmlns:c="{C_NS}" xmlns:a="{A_NS}" xmlns:r="{R_NS}"><c:chart><c:plotArea>{}</c:plotArea></c:chart></c:chartSpace>"#,
            axes.join("")
        )
    }

    #[test]
    fn data_labels_write_fixed_prefixes_in_schema_order() {
        let xml = format!(
            r#"<q:dLbls xmlns:q="{C_NS}" xmlns:a="{A_NS}" xmlns:r="{R_NS}" xmlns:x="urn:producer"><q:dLbl><q:idx val="0"/><x:point/></q:dLbl><q:numFmt formatCode="0.0%" sourceLinked="0"/><q:spPr><x:shape/></q:spPr><q:txPr><x:text/></q:txPr><q:dLblPos val="outEnd"/><q:showLegendKey val="1"/><q:showVal val="true"/><q:showCatName val="0"/><q:showSerName val="1"/><q:showPercent val="0"/><q:showBubbleSize val="1"/><q:separator> / </q:separator><q:leaderLines><x:line/></q:leaderLines><q:extLst><q:ext uri="labels"><x:tail/></q:ext></q:extLst></q:dLbls>"#
        );
        let parsed = CT_DLbls::from_xml(xml.as_bytes()).unwrap();
        assert_eq!(parsed.number_format.as_ref().unwrap().format_code, "0.0%");
        assert_eq!(parsed.position, Some(DataLabelPosition::OutsideEnd));
        assert_eq!(parsed.separator.as_deref(), Some(" / "));
        assert!(parsed.show_legend_key);
        assert!(parsed.show_value);
        assert!(!parsed.show_category_name);
        assert!(parsed.show_series_name);
        assert!(!parsed.show_percent);
        assert!(parsed.show_bubble_size);

        let written = String::from_utf8(parsed.to_xml().unwrap()).unwrap();
        assert!(written.starts_with(&format!(
            "<c:dLbls xmlns:c=\"{C_NS}\" xmlns:a=\"{A_NS}\" xmlns:r=\"{R_NS}\""
        )));
        let mut cursor = 0usize;
        for tag in [
            "<q:dLbl",
            "<c:numFmt",
            "<q:spPr",
            "<q:txPr",
            "<c:dLblPos",
            "<c:showLegendKey",
            "<c:showVal",
            "<c:showCatName",
            "<c:showSerName",
            "<c:showPercent",
            "<c:showBubbleSize",
            "<c:separator",
            "<q:leaderLines",
            "<q:extLst",
        ] {
            let position = written[cursor..]
                .find(tag)
                .unwrap_or_else(|| panic!("missing ordered data-label tag {tag}"));
            cursor += position + tag.len();
        }
        assert_eq!(parsed, CT_DLbls::from_xml(written.as_bytes()).unwrap());
    }

    #[test]
    fn percentage_formatted_label_renders_with_correct_text() {
        let Some(corpus) = require_or_skip_corpus() else {
            return;
        };
        verify_fetched_corpus(&corpus);
        assert_command_version("soffice", &["--version"], LIBREOFFICE_VERSION);
        assert_command_version("pdftotext", &["-v"], PDFTOTEXT_VERSION);

        let source = corpus.join("bar-chart.pptx");
        let mut package = OpcPackage::open(&source).expect("open percentage-label source deck");
        let chart_part = "/ppt/charts/chart1.xml";
        let original_parts = package.parts.clone();
        let chart_xml = package
            .get_part(chart_part)
            .expect("bar chart part")
            .to_vec();
        let original_series = first_series_xml(&chart_xml).expect("bar chart series");
        let mut series = Series::from_xml(&original_series).expect("parse bar chart series");
        series.values.values.fill(0.25);
        let labels = CT_DLbls {
            number_format: Some(NumberFormat::new("0%".to_owned(), false).unwrap()),
            position: Some(DataLabelPosition::OutsideEnd),
            show_value: true,
            ..CT_DLbls::default()
        };
        series.data_labels = Some(labels);
        let replacement = series.to_xml().expect("serialize typed percentage labels");
        let candidate_chart = replace_once(&chart_xml, &original_series, &replacement);
        package.set_part(chart_part, candidate_chart);
        for (part, bytes) in &original_parts {
            if part == chart_part {
                assert_ne!(package.parts[part], *bytes);
            } else {
                assert_eq!(
                    package.parts[part], *bytes,
                    "unexpected changed part {part}"
                );
            }
        }

        let temp_root = std::env::temp_dir().join(format!(
            "rpptx-chart-f123-label-gate-{}",
            std::process::id()
        ));
        let profile = std::env::temp_dir().join(format!(
            "rpptx-chart-f123-label-profile-{}",
            std::process::id()
        ));
        if temp_root.exists() {
            fs::remove_dir_all(&temp_root).expect("remove stale F-123 evidence");
        }
        if profile.exists() {
            fs::remove_dir_all(&profile).expect("remove stale F-123 profile");
        }
        fs::create_dir_all(&temp_root).expect("create F-123 evidence root");
        let unbound_candidate = temp_root.join("f123-percentage-label.pptx");
        package
            .save(&unbound_candidate)
            .expect("save percentage-label candidate");
        let candidate_sha = sha256(&unbound_candidate);
        let evidence_dir = temp_root.join(&candidate_sha);
        fs::create_dir(&evidence_dir).expect("create SHA-bound evidence directory");
        let candidate = evidence_dir.join("f123-percentage-label.pptx");
        fs::rename(&unbound_candidate, &candidate).expect("bind candidate to evidence SHA");
        assert_eq!(sha256(&candidate), candidate_sha);

        let profile_argument = format!("-env:UserInstallation=file://{}", profile.display());
        let conversion = Command::new("soffice")
            .args([
                "--headless",
                &profile_argument,
                "--convert-to",
                "pdf:impress_pdf_Export",
                "--outdir",
            ])
            .arg(&evidence_dir)
            .arg(&candidate)
            .output()
            .expect("run pinned LibreOffice percentage-label gate");
        assert!(
            conversion.status.success(),
            "LibreOffice conversion failed: {}",
            String::from_utf8_lossy(&conversion.stderr)
        );
        let pdf = evidence_dir.join("f123-percentage-label.pdf");
        assert!(
            pdf.is_file(),
            "LibreOffice did not create {}",
            pdf.display()
        );
        let text_path = evidence_dir.join("f123-percentage-label.txt");
        let extraction = Command::new("pdftotext")
            .arg(&pdf)
            .arg(&text_path)
            .output()
            .expect("run pinned Poppler percentage-label extraction");
        assert!(
            extraction.status.success(),
            "pdftotext failed: {}",
            String::from_utf8_lossy(&extraction.stderr)
        );
        let extracted = fs::read_to_string(&text_path).expect("read percentage-label text");
        assert!(
            extracted.contains("25%"),
            "SHA-bound viewer text lacks 25%: {extracted:?}"
        );
        for artifact in [&candidate, &pdf, &text_path] {
            assert_eq!(
                artifact.parent(),
                Some(evidence_dir.as_path()),
                "evidence escaped candidate SHA"
            );
        }
        eprintln!("F-123 candidate/render/text evidence SHA-256 {candidate_sha}, extracted 25%");
        fs::remove_dir_all(&temp_root).expect("remove F-123 evidence");
        if profile.exists() {
            fs::remove_dir_all(&profile).expect("remove F-123 profile");
        }
    }

    #[test]
    fn common_number_formats_project_cached_values_deterministically() {
        for (code, value, expected) in [
            ("General", 0.25, "0.25"),
            ("0", 12.6, "13"),
            ("0.0", 12.64, "12.6"),
            ("0.00", -12.645, "-12.64"),
            ("0%", 0.25, "25%"),
            ("0.0%", 0.256, "25.6%"),
            ("0.00%", 0.256, "25.60%"),
        ] {
            let format = NumberFormat::new(code.to_owned(), false).unwrap();
            assert_eq!(format.format_value(value).unwrap(), expected, "{code}");
        }
    }

    #[test]
    fn malformed_data_labels_and_number_formats_return_errors_without_panicking() {
        let cases = [
            format!(
                r#"<c:dLbls xmlns:c="{C_NS}"><c:showVal val="1"/><c:showVal val="0"/></c:dLbls>"#
            ),
            format!(r#"<c:dLbls xmlns:c="{C_NS}"><c:showVal val="yes"/></c:dLbls>"#),
            format!(r#"<c:dLbls xmlns:c="{C_NS}"><c:dLblPos val="middle"/></c:dLbls>"#),
            format!(
                r#"<c:dLbls xmlns:c="{C_NS}"><c:numFmt formatCode="" sourceLinked="1"/></c:dLbls>"#
            ),
            format!(
                r#"<c:dLbls xmlns:c="{C_NS}"><c:numFmt formatCode="0" sourceLinked="maybe"/></c:dLbls>"#
            ),
        ];
        for xml in cases {
            let result = std::panic::catch_unwind(|| CT_DLbls::from_xml(xml.as_bytes()));
            assert!(result.is_ok(), "data-label parser panicked for {xml}");
            assert!(result.unwrap().is_err(), "malformed labels parsed: {xml}");
        }

        let invalid_code = NumberFormat::new("bad\u{0}code".to_owned(), false);
        assert!(invalid_code.is_err());
        let unsupported = NumberFormat::new("#,##0".to_owned(), false).unwrap();
        assert!(unsupported.format_value(12.0).is_err());
        let general = NumberFormat::new("General".to_owned(), false).unwrap();
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(general.format_value(value).is_err());
        }

        let invalid_separator = CT_DLbls {
            separator: Some("bad\u{0}separator".to_owned()),
            ..CT_DLbls::default()
        };
        assert!(invalid_separator.to_xml().is_err());
    }

    #[test]
    fn data_labels_preserve_point_overrides_and_extensions_byte_for_byte() {
        let xml = format!(
            r#"<q:dLbls xmlns:q="{C_NS}" xmlns:x="urn:producer" x:keep="labels"><!--before--><q:dLbl x:keep="point"><q:idx val="0"/><q:spPr><x:shape/></q:spPr><q:txPr><x:text/></q:txPr><q:extLst><q:ext uri="point"><x:point/></q:ext></q:extLst></q:dLbl><?after-point?><q:numFmt formatCode="0%" sourceLinked="0"/><q:spPr><x:collection-shape/></q:spPr><q:txPr><x:collection-text/></q:txPr><q:showVal val="1"/><q:separator><![CDATA[ | ]]></q:separator><q:showLeaderLines val="1"/><q:leaderLines><q:spPr><x:leader-shape/></q:spPr></q:leaderLines><q:extLst><q:ext uri="collection"><x:tail/></q:ext></q:extLst><!--after--></q:dLbls>"#
        );
        let parsed = CT_DLbls::from_xml(xml.as_bytes()).unwrap();
        let written = parsed.to_xml().unwrap();
        for raw in [
            br#"<!--before-->"#.as_slice(),
            br#"<q:dLbl x:keep="point"><q:idx val="0"/><q:spPr><x:shape/></q:spPr><q:txPr><x:text/></q:txPr><q:extLst><q:ext uri="point"><x:point/></q:ext></q:extLst></q:dLbl>"#.as_slice(),
            br#"<?after-point?>"#.as_slice(),
            br#"<q:spPr><x:collection-shape/></q:spPr>"#.as_slice(),
            br#"<q:txPr><x:collection-text/></q:txPr>"#.as_slice(),
            br#"<q:showLeaderLines val="1"/>"#.as_slice(),
            br#"<q:leaderLines><q:spPr><x:leader-shape/></q:spPr></q:leaderLines>"#.as_slice(),
            br#"<q:extLst><q:ext uri="collection"><x:tail/></q:ext></q:extLst>"#.as_slice(),
            br#"<!--after-->"#.as_slice(),
        ] {
            assert!(
                written.windows(raw.len()).any(|window| window == raw),
                "preserved data-label bytes changed: {}",
                String::from_utf8_lossy(raw)
            );
        }
        assert_eq!(parsed, CT_DLbls::from_xml(&written).unwrap());
    }

    #[test]
    fn every_corpus_data_label_collection_round_trips_structurally() {
        let Some(corpus) = require_or_skip_corpus() else {
            return;
        };
        verify_fetched_corpus(&corpus);
        let mut label_collections = 0usize;
        let mut axis_number_formats = 0usize;
        for path in manifest_paths() {
            let package = OpcPackage::open(corpus.join(path))
                .unwrap_or_else(|error| panic!("{path}: open failed: {error}"));
            for (part, xml) in &package.parts {
                if !is_chart_part(part) {
                    continue;
                }
                let chart = CT_ChartSpace::from_xml(xml)
                    .unwrap_or_else(|error| panic!("{path} {part}: parse failed: {error}"));
                for series in chart.chart.plot_area.series().unwrap_or_else(|error| {
                    panic!("{path} {part}: series projection failed: {error}")
                }) {
                    if let Some(labels) = series.data_labels {
                        let written = labels.to_xml().unwrap_or_else(|error| {
                            panic!("{path} {part}: label write failed: {error}")
                        });
                        let reparsed = CT_DLbls::from_xml(&written).unwrap_or_else(|error| {
                            panic!("{path} {part}: written labels parse failed: {error}")
                        });
                        assert_eq!(labels, reparsed, "{path} {part}: labels changed");
                        label_collections += 1;
                    }
                }
                for axis in chart.chart.plot_area.axes().unwrap_or_else(|error| {
                    panic!("{path} {part}: axis projection failed: {error}")
                }) {
                    if axis.number_format.is_some() {
                        let written = axis.to_xml().unwrap_or_else(|error| {
                            panic!("{path} {part}: axis write failed: {error}")
                        });
                        let reparsed = Axis::from_xml(&written).unwrap_or_else(|error| {
                            panic!("{path} {part}: written axis parse failed: {error}")
                        });
                        assert_eq!(axis, reparsed, "{path} {part}: axis changed");
                        axis_number_formats += 1;
                    }
                }
            }
        }
        assert!(label_collections > 0, "the pinned corpus has no c:dLbls");
        assert!(
            axis_number_formats > 0,
            "the pinned corpus has no axis c:numFmt"
        );
        eprintln!(
            "ChartML data-label corpus gate checked {label_collections} collections and {axis_number_formats} axis number formats"
        );
    }

    fn full_axis(local: &str, position: &str) -> String {
        format!(
            r#"<q:{local} xmlns:q="{C_NS}" xmlns:d="{A_NS}" xmlns:r="{R_NS}" xmlns:x="urn:producer"><q:axId val="-1125255920"/><q:scaling><q:logBase val="10"/><q:orientation val="maxMin"/><q:max val="100"/><q:min val="-10"/></q:scaling><q:delete val="1"/><q:axPos val="{position}"/><q:majorGridlines><q:spPr><d:ln/></q:spPr></q:majorGridlines><q:minorGridlines/><q:title><x:title/></q:title><q:numFmt formatCode="0.0" sourceLinked="0"/><q:majorTickMark val="cross"/><q:minorTickMark val="in"/><q:tickLblPos val="high"/><q:spPr><d:solidFill><d:srgbClr val="112233"/></d:solidFill></q:spPr><q:txPr><d:bodyPr/><d:p/></q:txPr><q:crossAx val="-1004303696"/></q:{local}>"#
        )
    }

    #[test]
    fn axis_id_pairs_are_reciprocal() {
        assert_eq!(
            AxisId::new(i64::from(i32::MIN)).unwrap().value(),
            i64::from(i32::MIN)
        );
        assert_eq!(
            AxisId::new(i64::from(u32::MAX)).unwrap().value(),
            i64::from(u32::MAX)
        );
        assert!(AxisId::new(i64::from(i32::MIN) - 1).is_err());
        assert!(AxisId::new(i64::from(u32::MAX) + 1).is_err());

        let valid = chart_with_axes(&[
            minimal_axis("catAx", "-1884094432", "b", "-1884097184"),
            minimal_axis("valAx", "-1884097184", "l", "-1884094432"),
        ]);
        let axes = CT_ChartSpace::from_xml(valid.as_bytes())
            .unwrap()
            .chart
            .plot_area
            .axes()
            .unwrap();
        assert_eq!(axes.len(), 2);
        assert_eq!(axes[0].id.value(), -1_884_094_432);
        assert_eq!(axes[0].cross_axis, axes[1].id);
        assert_eq!(axes[1].cross_axis, axes[0].id);

        let invalid_sets = [
            vec![
                minimal_axis("catAx", "1", "b", "2"),
                minimal_axis("valAx", "1", "l", "2"),
            ],
            vec![minimal_axis("catAx", "1", "b", "1")],
            vec![minimal_axis("catAx", "1", "b", "2")],
            vec![
                minimal_axis("catAx", "1", "b", "2"),
                minimal_axis("valAx", "2", "l", "3"),
                minimal_axis("serAx", "3", "r", "2"),
            ],
        ];
        for axes in invalid_sets {
            let chart = CT_ChartSpace::from_xml(chart_with_axes(&axes).as_bytes()).unwrap();
            assert!(chart.chart.plot_area.axes().is_err());
        }

        let empty = CT_ChartSpace::from_xml(chart_with_axes(&[]).as_bytes()).unwrap();
        assert!(empty.chart.plot_area.axes().unwrap().is_empty());
    }

    #[test]
    fn all_axis_forms_write_fixed_prefixes_in_schema_order() {
        for (local, kind, position) in [
            ("catAx", AxisKind::Category, AxisPosition::Bottom),
            ("valAx", AxisKind::Value, AxisPosition::Left),
            ("dateAx", AxisKind::Date, AxisPosition::Top),
            ("serAx", AxisKind::Series, AxisPosition::Right),
        ] {
            let xml = full_axis(local, position.as_str());
            let parsed = Axis::from_xml(xml.as_bytes()).unwrap();
            assert_eq!(parsed.kind, kind);
            assert_eq!(parsed.id.value(), -1_125_255_920);
            assert_eq!(parsed.scaling.log_base, Some(10.0));
            assert_eq!(parsed.scaling.orientation, Orientation::MaxMin);
            assert_eq!(parsed.scaling.maximum, Some(100.0));
            assert_eq!(parsed.scaling.minimum, Some(-10.0));
            assert!(parsed.deleted);
            assert_eq!(parsed.position, position);
            assert!(parsed.major_gridlines.is_some());
            assert!(parsed.minor_gridlines.is_some());
            assert!(parsed.title.is_some());
            assert_eq!(parsed.number_format.as_ref().unwrap().format_code, "0.0");
            assert_eq!(parsed.major_tick_mark, TickMark::Cross);
            assert_eq!(parsed.minor_tick_mark, TickMark::Inside);
            assert_eq!(parsed.tick_label_position, TickLabelPosition::High);
            assert!(parsed.sp_pr.is_some());
            assert!(parsed.tx_pr.is_some());
            assert_eq!(parsed.cross_axis.value(), -1_004_303_696);

            let written = String::from_utf8(parsed.to_xml().unwrap()).unwrap();
            assert!(written.starts_with(&format!(
                "<c:{local} xmlns:c=\"{C_NS}\" xmlns:a=\"{A_NS}\" xmlns:r=\"{R_NS}\""
            )));
            assert!(written.contains("<c:axId val=\"-1125255920\""));
            let mut cursor = 0usize;
            for tag in [
                "<c:axId",
                "<c:scaling",
                "<c:delete",
                "<c:axPos",
                "<c:majorGridlines",
                "<c:minorGridlines",
                "<c:title",
                "<c:numFmt",
                "<c:majorTickMark",
                "<c:minorTickMark",
                "<c:tickLblPos",
                "<c:spPr",
                "<c:txPr",
                "<c:crossAx",
            ] {
                let position = written[cursor..]
                    .find(tag)
                    .unwrap_or_else(|| panic!("missing ordered axis tag {tag}"));
                cursor += position + tag.len();
            }
            assert_eq!(parsed, Axis::from_xml(written.as_bytes()).unwrap());
        }

        let mut constructed = Axis::new(
            AxisKind::Value,
            AxisId::new(1).unwrap(),
            AxisPosition::Left,
            AxisId::new(2).unwrap(),
        );
        assert_eq!(
            constructed,
            Axis::from_xml(&constructed.to_xml().unwrap()).unwrap()
        );
        constructed.kind = AxisKind::Category;
        constructed.id = AxisId::new(3).unwrap();
        constructed.cross_axis = AxisId::new(4).unwrap();
        constructed.scaling.minimum = Some(0.0);
        constructed.scaling.maximum = Some(10.0);
        let written = constructed.to_xml().unwrap();
        assert!(written.starts_with(b"<c:catAx"));
        assert_eq!(constructed, Axis::from_xml(&written).unwrap());
    }

    #[test]
    fn axis_equality_normalizes_ids_without_losing_lexical_preservation() {
        let padded = Axis::from_xml(
            standalone_axis(
                "catAx",
                "<c:axId val=\"01\"/><c:scaling/><c:axPos val=\"b\"/><c:crossAx val=\"002\"/>",
            )
            .as_bytes(),
        )
        .unwrap();
        let normalized = Axis::from_xml(
            standalone_axis(
                "catAx",
                "<c:axId val=\"1\"/><c:scaling/><c:axPos val=\"b\"/><c:crossAx val=\"2\"/>",
            )
            .as_bytes(),
        )
        .unwrap();
        assert_eq!(padded, normalized);

        let preserved = String::from_utf8(padded.to_xml().unwrap()).unwrap();
        assert!(preserved.contains("<c:axId val=\"01\""));
        assert!(preserved.contains("<c:crossAx val=\"002\""));

        let mut mutated = padded;
        mutated.id = AxisId::new(3).unwrap();
        mutated.cross_axis = AxisId::new(4).unwrap();
        let rewritten = String::from_utf8(mutated.to_xml().unwrap()).unwrap();
        assert!(rewritten.contains("<c:axId val=\"3\""));
        assert!(rewritten.contains("<c:crossAx val=\"4\""));
        assert!(!rewritten.contains("val=\"01\""));
        assert!(!rewritten.contains("val=\"002\""));
    }

    #[test]
    fn malformed_axis_values_return_errors_without_panicking() {
        let cases = [
            standalone_axis(
                "catAx",
                "<c:scaling/><c:axPos val=\"b\"/><c:crossAx val=\"2\"/>",
            ),
            standalone_axis(
                "catAx",
                "<c:axId val=\"1\"/><c:axPos val=\"b\"/><c:crossAx val=\"2\"/>",
            ),
            standalone_axis(
                "catAx",
                "<c:axId val=\"1\"/><c:scaling/><c:crossAx val=\"2\"/>",
            ),
            standalone_axis(
                "catAx",
                "<c:axId val=\"1\"/><c:scaling/><c:axPos val=\"b\"/>",
            ),
            standalone_axis(
                "catAx",
                "<c:axId val=\"-2147483649\"/><c:scaling/><c:axPos val=\"b\"/><c:crossAx val=\"2\"/>",
            ),
            standalone_axis(
                "catAx",
                "<c:axId val=\"4294967296\"/><c:scaling/><c:axPos val=\"b\"/><c:crossAx val=\"2\"/>",
            ),
            standalone_axis(
                "catAx",
                "<c:axId val=\"1\"/><c:scaling><c:logBase val=\"1\"/></c:scaling><c:axPos val=\"b\"/><c:crossAx val=\"2\"/>",
            ),
            standalone_axis(
                "catAx",
                "<c:axId val=\"1\"/><c:scaling><c:max val=\"NaN\"/></c:scaling><c:axPos val=\"b\"/><c:crossAx val=\"2\"/>",
            ),
            standalone_axis(
                "catAx",
                "<c:axId val=\"1\"/><c:scaling><c:max val=\"1\"/><c:min val=\"2\"/></c:scaling><c:axPos val=\"b\"/><c:crossAx val=\"2\"/>",
            ),
            standalone_axis(
                "catAx",
                "<c:axId val=\"1\"/><c:scaling><c:orientation val=\"sideways\"/></c:scaling><c:axPos val=\"b\"/><c:crossAx val=\"2\"/>",
            ),
            standalone_axis(
                "catAx",
                "<c:axId val=\"1\"/><c:scaling/><c:delete val=\"yes\"/><c:axPos val=\"b\"/><c:crossAx val=\"2\"/>",
            ),
            standalone_axis(
                "catAx",
                "<c:axId val=\"1\"/><c:scaling/><c:axPos val=\"middle\"/><c:crossAx val=\"2\"/>",
            ),
            standalone_axis(
                "catAx",
                "<c:axId val=\"1\"/><c:scaling/><c:axPos val=\"b\"/><c:majorTickMark val=\"near\"/><c:crossAx val=\"2\"/>",
            ),
            standalone_axis(
                "catAx",
                "<c:axId val=\"1\"/><c:scaling/><c:axPos val=\"b\"/><c:tickLblPos val=\"middle\"/><c:crossAx val=\"2\"/>",
            ),
            standalone_axis(
                "catAx",
                "<c:axId val=\"1\"/><c:axId val=\"2\"/><c:scaling/><c:axPos val=\"b\"/><c:crossAx val=\"2\"/>",
            ),
            standalone_axis(
                "catAx",
                "<c:axId val=\"1\"/><c:scaling/><c:axPos val=\"b\"/><c:numFmt sourceLinked=\"1\"/><c:crossAx val=\"2\"/>",
            ),
            standalone_axis(
                "catAx",
                "<c:axId val=\"1\"/><c:scaling/><c:axPos val=\"b\"/><c:numFmt formatCode=\"0\" sourceLinked=\"maybe\"/><c:crossAx val=\"2\"/>",
            ),
        ];
        for xml in cases {
            let result = std::panic::catch_unwind(|| Axis::from_xml(xml.as_bytes()));
            assert!(result.is_ok(), "axis parser panicked for {xml}");
            assert!(result.unwrap().is_err(), "malformed axis parsed: {xml}");
        }

        let mut axis = Axis::new(
            AxisKind::Value,
            AxisId::new(1).unwrap(),
            AxisPosition::Left,
            AxisId::new(2).unwrap(),
        );
        axis.scaling.maximum = Some(f64::INFINITY);
        assert!(axis.to_xml().is_err());
    }

    #[test]
    fn axes_preserve_unmodelled_children_byte_for_byte() {
        let xml = format!(
            r#"<q:catAx xmlns:q="{C_NS}" xmlns:d="{A_NS}" xmlns:x="urn:producer" x:keep="axis"><!--before--><q:axId val="-1884094432" x:keep="id"/><?after-id?><q:scaling x:keep="scaling"><!--scale--><q:orientation val="minMax"/><q:extLst><q:ext uri="scale"><x:data/></q:ext></q:extLst></q:scaling><q:axPos val="b"/><q:majorGridlines x:keep="grid"><x:grid/></q:majorGridlines><q:title><x:title/></q:title><q:crossAx val="-1884097184"/><q:crosses val="autoZero"/><q:auto val="1"><x:auto/></q:auto><q:extLst><q:ext uri="axis"><x:tail/></q:ext></q:extLst><!--after--></q:catAx>"#
        );
        let parsed = Axis::from_xml(xml.as_bytes()).unwrap();
        let mut relabelled = parsed.clone();
        relabelled.kind = AxisKind::Value;
        assert!(
            relabelled.to_xml().is_err(),
            "a parsed category axis must not retain its tail under a value-axis root"
        );
        let written = parsed.to_xml().unwrap();
        for raw in [
            br#"<!--before-->"#.as_slice(),
            br#"<?after-id?>"#.as_slice(),
            br#"<!--scale-->"#.as_slice(),
            br#"<q:extLst><q:ext uri="scale"><x:data/></q:ext></q:extLst>"#.as_slice(),
            br#"<x:grid/>"#.as_slice(),
            br#"<x:title/>"#.as_slice(),
            br#"<q:crosses val="autoZero"/>"#.as_slice(),
            br#"<q:auto val="1"><x:auto/></q:auto>"#.as_slice(),
            br#"<q:extLst><q:ext uri="axis"><x:tail/></q:ext></q:extLst>"#.as_slice(),
            br#"<!--after-->"#.as_slice(),
        ] {
            assert!(
                written.windows(raw.len()).any(|window| window == raw),
                "preserved axis bytes changed: {}",
                String::from_utf8_lossy(raw)
            );
        }
        let written = String::from_utf8(written).unwrap();
        assert!(written.starts_with("<c:catAx"));
        assert!(written.contains("x:keep=\"axis\""));
        assert!(written.contains("x:keep=\"id\""));
        assert_eq!(parsed, Axis::from_xml(written.as_bytes()).unwrap());
    }

    #[test]
    fn every_corpus_axis_round_trips_structurally() {
        let Some(corpus) = require_or_skip_corpus() else {
            return;
        };
        verify_fetched_corpus(&corpus);
        let mut axis_count = 0usize;
        let mut chart_parts = 0usize;
        let mut kinds = HashSet::new();
        for path in manifest_paths() {
            let package = OpcPackage::open(corpus.join(path))
                .unwrap_or_else(|error| panic!("{path}: open failed: {error}"));
            for (part, xml) in &package.parts {
                if !is_chart_part(part) {
                    continue;
                }
                let chart = CT_ChartSpace::from_xml(xml)
                    .unwrap_or_else(|error| panic!("{path} {part}: parse failed: {error}"));
                let axes = chart.chart.plot_area.axes().unwrap_or_else(|error| {
                    panic!("{path} {part}: axis projection failed: {error}")
                });
                for axis in axes {
                    kinds.insert(axis.kind);
                    let written = axis.to_xml().unwrap_or_else(|error| {
                        panic!("{path} {part}: axis write failed: {error}")
                    });
                    let reparsed = Axis::from_xml(&written).unwrap_or_else(|error| {
                        panic!("{path} {part}: written axis parse failed: {error}")
                    });
                    assert_eq!(axis, reparsed, "{path} {part}: axis model changed");
                    axis_count += 1;
                }
                chart_parts += 1;
            }
        }
        assert!(
            axis_count > 0,
            "the pinned corpus contained no ChartML axes"
        );
        assert!(kinds.contains(&AxisKind::Category));
        assert!(kinds.contains(&AxisKind::Value));

        for kind in ["dateAx", "serAx"] {
            let parsed = Axis::from_xml(
                standalone_axis(
                    kind,
                    "<c:axId val=\"1\"/><c:scaling/><c:axPos val=\"b\"/><c:crossAx val=\"2\"/>",
                )
                .as_bytes(),
            )
            .unwrap();
            assert_eq!(parsed, Axis::from_xml(&parsed.to_xml().unwrap()).unwrap());
        }
        eprintln!(
            "ChartML axis corpus gate checked {axis_count} axes across {chart_parts} chart parts"
        );
    }

    #[test]
    fn series_formula_and_cache_are_consistent_with_one_source() {
        let values = NumericData::new(
            "'Sales 24'!$B$2:$B$4".to_owned(),
            "0.0".to_owned(),
            vec![4.25, 8.5, 17.0],
        )
        .unwrap();
        let mut series = Series::new(3, 1, values);
        series.categories = Some(AxisData::String(
            StringRef::new(
                "'Sales 24'!$A$2:$A$4".to_owned(),
                vec!["North".to_owned(), "South".to_owned(), "West".to_owned()],
            )
            .unwrap(),
        ));

        let written = String::from_utf8(series.to_xml().unwrap()).unwrap();
        assert!(written.contains("<c:f>&apos;Sales 24&apos;!$B$2:$B$4</c:f>"));
        assert!(written.contains("<c:formatCode>0.0</c:formatCode>"));
        assert_eq!(written.matches("<c:ptCount val=\"3\"").count(), 2);
        for (index, value) in ["4.25", "8.5", "17"].iter().enumerate() {
            assert!(written.contains(&format!("<c:pt idx=\"{index}\"><c:v>{value}</c:v></c:pt>")));
        }
        let reparsed = Series::from_xml(written.as_bytes()).unwrap();
        assert_eq!(reparsed.index, series.index);
        assert_eq!(reparsed.order, series.order);
        match (&reparsed.categories, &series.categories) {
            (Some(AxisData::String(left)), Some(AxisData::String(right))) => {
                assert_eq!(left.formula, right.formula);
                assert_eq!(left.values, right.values);
            }
            _ => panic!("expected string categories"),
        }
        assert_eq!(reparsed.values.formula, series.values.formula);
        assert_eq!(reparsed.values.format_code, series.values.format_code);
        assert_eq!(reparsed.values.values, series.values.values);
    }

    #[test]
    fn string_and_numeric_references_write_fixed_prefixes_in_schema_order() {
        let string_xml = br#"<q:strRef xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart"><q:strCache><q:pt idx="0"><q:v>West</q:v></q:pt><q:ptCount val="1"/></q:strCache><q:f>Sheet1!$A$2</q:f></q:strRef>"#;
        let string_ref = StringRef::from_xml(string_xml).unwrap();
        let written = String::from_utf8(string_ref.to_xml().unwrap()).unwrap();
        assert!(written.starts_with("<c:strRef xmlns:c="));
        assert!(written.find("<c:f>").unwrap() < written.find("<c:strCache>").unwrap());
        assert!(written.find("<c:ptCount").unwrap() < written.find("<c:pt idx=").unwrap());

        let numeric_xml = br#"<q:numRef xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart"><q:numCache><q:ptCount val="2"/><q:pt idx="0"><q:v>1.5</q:v></q:pt><q:pt idx="1"><q:v>2.5</q:v></q:pt><q:formatCode>0.00</q:formatCode></q:numCache><q:f>Sheet1!$B$2:$B$3</q:f></q:numRef>"#;
        let numeric = NumericData::from_xml(numeric_xml).unwrap();
        let written = String::from_utf8(numeric.to_xml().unwrap()).unwrap();
        let positions: Vec<_> = [
            "<c:f>",
            "<c:numCache>",
            "<c:formatCode>",
            "<c:ptCount",
            "<c:pt idx=",
        ]
        .iter()
        .map(|tag| written.find(tag).unwrap())
        .collect();
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(numeric, NumericData::from_xml(written.as_bytes()).unwrap());
    }

    #[test]
    fn mixed_chartml_aliases_resolve_by_namespace_uri() {
        let xml = br#"<q:ser xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="4"/><q:order val="2"/><c:cat><q:strRef><c:f>Sheet1!$A$2:$A$3</c:f><q:strCache><c:ptCount val="2"/><q:pt idx="0"><c:v>North</c:v></q:pt><c:pt idx="1"><q:v>West</q:v></c:pt></q:strCache></q:strRef></c:cat><q:val><c:numRef><q:f>Sheet1!$B$2:$B$3</q:f><c:numCache><q:formatCode>0.0</q:formatCode><c:ptCount val="2"/><q:pt idx="0"><c:v>1.5</c:v></q:pt><c:pt idx="1"><q:v>2.5</q:v></c:pt></c:numCache></c:numRef></q:val></q:ser>"#;
        let parsed = Series::from_xml(xml).unwrap();
        assert_eq!(parsed.index, 4);
        assert_eq!(parsed.order, 2);
        assert_eq!(parsed.values.values, vec![1.5, 2.5]);
        assert!(matches!(parsed.categories, Some(AxisData::String(_))));
        assert_eq!(parsed, Series::from_xml(&parsed.to_xml().unwrap()).unwrap());
    }

    #[test]
    fn plot_area_series_ignores_inherited_foreign_plot_aliases() {
        let xml = br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:x="urn:producer"><c:chart><c:plotArea><x:barChart><x:ser><x:idx val="9"/><x:order val="9"/><x:val><x:numRef><x:f>foreign</x:f><x:numCache><x:formatCode>General</x:formatCode><x:ptCount val="0"/></x:numCache></x:numRef></x:val></x:ser></x:barChart><c:barChart><q:ser><q:idx val="1"/><q:order val="0"/><q:marker><x:data/></q:marker><q:val><q:numRef><q:f>Sheet1!$B$2</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount val="1"/><q:pt idx="0"><q:v>3</q:v></q:pt></q:numCache></q:numRef></q:val></q:ser></c:barChart><c:pieChart/></c:plotArea></c:chart></c:chartSpace>"#;
        let chart = CT_ChartSpace::from_xml(xml).unwrap();
        let series = chart.chart.plot_area.series().unwrap();
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].index, 1);
        let written = String::from_utf8(series[0].to_xml().unwrap()).unwrap();
        assert!(
            written.contains(r#"xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart""#)
        );
        assert!(written.contains(r#"xmlns:x="urn:producer""#));
        assert!(written.contains("<q:marker><x:data/></q:marker>"));
        assert_eq!(series[0], Series::from_xml(written.as_bytes()).unwrap());

        let conflicting = br#"<q:chartSpace xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart"><q:chart><q:plotArea><q:barChart xmlns:c="urn:foreign"><q:ser><q:idx val="0"/><q:order val="0"/><q:val><q:numRef><q:f>Sheet1!$A$1</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount val="0"/></q:numCache></q:numRef></q:val></q:ser></q:barChart><q:pieChart/></q:plotArea></q:chart></q:chartSpace>"#;
        let chart = CT_ChartSpace::from_xml(conflicting).unwrap();
        assert!(chart.chart.plot_area.series().is_err());
    }

    #[test]
    fn malformed_series_and_cache_values_return_errors_without_panicking() {
        let cases: &[&[u8]] = &[
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:order val="0"/><c:val><c:numRef><c:f>S!$A$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:order val="0"/></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:idx val="1"/><c:order val="0"/><c:val><c:numRef><c:f>S!$A$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:order val="0"/><c:val><c:numRef><c:f></c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:order val="0"/><c:val><c:numRef><c:f>S!$A$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/><c:pt idx="0"><c:v>1</c:v></c:pt></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:order val="0"/><c:val><c:numRef><c:f>S!$A$1:$A$2</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="2"/><c:pt idx="0"><c:v>1</c:v></c:pt><c:pt idx="2"><c:v>2</c:v></c:pt></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:order val="0"/><c:val><c:numRef><c:f>S!$A$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="1"/><c:pt idx="0"><c:v>NaN</c:v></c:pt></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="urn:producer"><c:idx val="0"/><c:order val="0"/><a:marker/><c:val><c:numRef><c:f>S!$A$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:x="urn:producer"><c:idx val="0"/><c:order val="0"/><c:spPr><x:solidFill><x:srgbClr val="112233"/></x:solidFill></c:spPr><c:val><c:numRef><c:f>S!$A$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:order val="0"/><c:tx><c:v>opaque</c:v></c:tx><c:tx><c:strRef><c:f>S!$A$1</c:f><c:strCache><c:ptCount val="0"/></c:strCache></c:strRef></c:tx><c:val><c:numRef><c:f>S!$B$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:order val="0"/><c:cat><c:multiLvlStrRef/></c:cat><c:cat><c:numRef><c:f>S!$A$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:cat><c:val><c:numRef><c:f>S!$B$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val></c:ser>"#,
            br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:order val="0"/><c:val><c:numRef><c:f>S!$B$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val><c:bubbleSize><c:numLit/></c:bubbleSize><c:bubbleSize><c:numRef><c:f>S!$C$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:bubbleSize></c:ser>"#,
            br#"<q:ser xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart"><q:idx val="0"/><q:order val="0"/><q:val xmlns:c="urn:producer"><q:numRef><q:f>S!$A$1</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount val="0"/></q:numCache></q:numRef></q:val></q:ser>"#,
            br#"<q:ser xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart"><q:idx val="0"/><q:order val="0"/><q:val><q:numRef><q:f>S!$A$1</q:f><q:numCache xmlns:c="urn:producer"><q:formatCode>General</q:formatCode><q:ptCount val="0"/></q:numCache></q:numRef></q:val></q:ser>"#,
            br#"<q:ser xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart"><q:idx val="0"/><q:order val="0"/><q:val><q:numRef><q:f>S!$A$1</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount val="1"/><q:pt xmlns:c="urn:producer" idx="0"><q:v>1</q:v></q:pt></q:numCache></q:numRef></q:val></q:ser>"#,
            br#"<q:ser xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart"><q:idx val="0"/><q:order val="0"/><q:val><q:numRef><q:f>S!$A$1</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount xmlns:c="urn:producer" val="0"/></q:numCache></q:numRef></q:val></q:ser>"#,
            br#"<q:ser xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart"><q:idx val="0"/><q:order val="0"/><q:val><q:numRef><q:f xmlns:c="urn:producer">S!$A$1</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount val="0"/></q:numCache></q:numRef></q:val></q:ser>"#,
        ];
        for xml in cases {
            let result = std::panic::catch_unwind(|| Series::from_xml(xml));
            assert!(result.is_ok(), "series parser panicked");
            assert!(
                result.unwrap().is_err(),
                "malformed series parsed: {}",
                String::from_utf8_lossy(xml)
            );
        }
        assert!(
            NumericData::new(
                "S!$A$1".to_owned(),
                "General".to_owned(),
                vec![f64::INFINITY]
            )
            .is_err()
        );
    }

    #[test]
    fn chart_text_rejects_xml_forbidden_characters_and_escapes_metacharacters() {
        assert!(StringRef::new("S!$A$1\u{1}".to_owned(), Vec::new()).is_err());
        assert!(StringRef::new("S!$A$1".to_owned(), vec!["invalid\u{1}".to_owned()]).is_err());
        assert!(
            NumericData::new("S!$A$1\u{1}".to_owned(), "General".to_owned(), Vec::new()).is_err()
        );
        assert!(
            NumericData::new("S!$A$1".to_owned(), "General\u{1}".to_owned(), Vec::new()).is_err()
        );

        let mut strings = StringRef::new("S!$A$1".to_owned(), vec!["valid".to_owned()]).unwrap();
        strings.formula = "invalid\u{1}".to_owned();
        assert!(strings.to_xml().is_err());
        strings.formula = "S!$A$1".to_owned();
        strings.values[0] = "invalid\u{1}".to_owned();
        assert!(strings.to_xml().is_err());

        let mut numbers =
            NumericData::new("S!$A$1".to_owned(), "General".to_owned(), Vec::new()).unwrap();
        numbers.formula = "invalid\u{1}".to_owned();
        assert!(numbers.to_xml().is_err());
        numbers.formula = "S!$A$1".to_owned();
        numbers.format_code = "invalid\u{1}".to_owned();
        assert!(numbers.to_xml().is_err());

        let strings =
            StringRef::new("S!$A$1&".to_owned(), vec!["North < West & East".to_owned()]).unwrap();
        let written = String::from_utf8(strings.to_xml().unwrap()).unwrap();
        assert!(written.contains("S!$A$1&amp;"));
        assert!(written.contains("North &lt; West &amp; East"));
    }

    #[test]
    fn series_preserves_unmodelled_children_byte_for_byte() {
        let xml = br#"<q:ser xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer" x:keep="series"><!--before--><q:idx val="0" x:keep="idx"/><x:between/><q:order val="0"/><q:tx x:keep="tx"><q:strRef x:keep="ref"><q:f x:keep="formula">S!$B$1</q:f><q:strCache x:keep="cache"><q:ptCount val="1" x:keep="count"/><!--point--><q:pt idx="0" x:keep="point"><q:v x:keep="value">Revenue</q:v></q:pt><x:cacheExt/></q:strCache></q:strRef></q:tx><q:spPr><d:noFill/><x:shapeExt/></q:spPr><q:dPt x:id="one"><q:idx val="0"/></q:dPt><q:dLbls x:id="labels"/><q:cat><q:strRef><q:f>S!$A$2:$A$3</q:f><q:strCache><q:ptCount val="2"/><q:pt idx="0"><q:v>North</q:v></q:pt><q:pt idx="1"><q:v>West</q:v></q:pt></q:strCache></q:strRef></q:cat><q:trendline x:id="trend"/><q:val><q:numRef><q:f>S!$B$2:$B$3</q:f><q:numCache><q:formatCode>0.0</q:formatCode><q:ptCount val="2"/><q:pt idx="0"><q:v>1.5</q:v></q:pt><q:pt idx="1"><q:v>2.5</q:v></q:pt></q:numCache></q:numRef></q:val><q:extLst><q:ext uri="keep"><x:data/></q:ext></q:extLst></q:ser>"#;
        let parsed = Series::from_xml(xml).unwrap();
        let written = parsed.to_xml().unwrap();
        for raw in [
            br#"<!--before-->"#.as_slice(),
            br#"<x:between/>"#.as_slice(),
            br#"<q:dPt x:id="one"><q:idx val="0"/></q:dPt>"#.as_slice(),
            br#"<q:trendline x:id="trend"/>"#.as_slice(),
            br#"<q:extLst><q:ext uri="keep"><x:data/></q:ext></q:extLst>"#.as_slice(),
            br#"<!--point-->"#.as_slice(),
            br#"<x:cacheExt/>"#.as_slice(),
        ] {
            assert!(
                written.windows(raw.len()).any(|window| window == raw),
                "preserved series bytes changed: {}",
                String::from_utf8_lossy(raw)
            );
        }
        let written = String::from_utf8(written.clone()).unwrap();
        assert!(written.contains(r#"x:keep="series""#));
        assert!(written.contains(r#"x:keep="formula""#));
        assert!(written.contains("<c:dLbls"));
        assert!(written.contains(r#"x:id="labels""#));
        assert!(parsed.data_labels.is_some());
        assert_eq!(parsed, Series::from_xml(written.as_bytes()).unwrap());
    }

    #[test]
    fn public_series_edits_do_not_duplicate_or_drop_preserved_payloads() {
        let opaque_xml = br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:idx val="0"/><c:order val="0"/><c:tx><c:v>opaque</c:v></c:tx><c:cat><c:multiLvlStrRef/></c:cat><c:val><c:numRef><c:f>S!$A$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val><c:bubbleSize><c:numLit/></c:bubbleSize></c:ser>"#;
        let parsed = Series::from_xml(opaque_xml).unwrap();

        let mut edited = parsed.clone();
        edited.name = Some(StringRef::new("S!$A$1".to_owned(), Vec::new()).unwrap());
        assert!(edited.to_xml().is_err());

        let mut edited = parsed.clone();
        edited.categories = Some(AxisData::Numeric(
            NumericData::new("S!$A$1".to_owned(), "General".to_owned(), Vec::new()).unwrap(),
        ));
        assert!(edited.to_xml().is_err());

        let mut edited = parsed;
        edited.bubble_size =
            Some(NumericData::new("S!$A$1".to_owned(), "General".to_owned(), Vec::new()).unwrap());
        assert!(edited.to_xml().is_err());

        let cache_xml = br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:x="urn:producer"><c:idx val="0"/><c:order val="0"/><c:val><c:numRef><c:f>S!$A$1:$A$2</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="2"/><c:pt idx="0"><c:v>1</c:v></c:pt><c:pt idx="1"><c:v>2</c:v></c:pt><c:extLst><c:ext uri="keep"><x:data/></c:ext></c:extLst></c:numCache></c:numRef></c:val></c:ser>"#;
        let parsed = Series::from_xml(cache_xml).unwrap();
        let tail = "<c:extLst><c:ext uri=\"keep\"><x:data/></c:ext></c:extLst>";

        let mut shortened = parsed.clone();
        shortened.values.values.truncate(1);
        let written = String::from_utf8(shortened.to_xml().unwrap()).unwrap();
        assert!(!written.contains("<c:pt idx=\"1\""));
        assert!(written.find("<c:pt idx=\"0\"").unwrap() < written.find(tail).unwrap());
        assert!(Series::from_xml(written.as_bytes()).is_ok());

        let mut grown = parsed;
        grown.values.values.push(3.0);
        let written = String::from_utf8(grown.to_xml().unwrap()).unwrap();
        assert!(written.find("<c:pt idx=\"2\"").unwrap() < written.find(tail).unwrap());
        assert!(Series::from_xml(written.as_bytes()).is_ok());

        let ordered_xml = br#"<c:ser xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><c:idx val="0"/><c:order val="0"/><c:marker><c:symbol val="circle"/></c:marker><c:val><c:numRef><c:f>S!$B$1</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="0"/></c:numCache></c:numRef></c:val><c:extLst><c:ext uri="keep"/></c:extLst></c:ser>"#;
        let mut edited = Series::from_xml(ordered_xml).unwrap();
        edited.name = Some(StringRef::new("S!$B$1".to_owned(), Vec::new()).unwrap());
        edited.sp_pr = Some(CT_ShapeProperties::from_xml(br#"<a:spPr/>"#).unwrap());
        edited.categories = Some(AxisData::String(
            StringRef::new("S!$A$1".to_owned(), Vec::new()).unwrap(),
        ));
        edited.bubble_size =
            Some(NumericData::new("S!$C$1".to_owned(), "General".to_owned(), Vec::new()).unwrap());
        let written = String::from_utf8(edited.to_xml().unwrap()).unwrap();
        let positions: Vec<_> = [
            "<c:tx>",
            "<c:spPr",
            "<c:marker>",
            "<c:cat>",
            "<c:val>",
            "<c:bubbleSize>",
            "<c:extLst>",
        ]
        .iter()
        .map(|tag| written.find(tag).unwrap())
        .collect();
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(Series::from_xml(written.as_bytes()).is_ok());
    }

    #[test]
    fn every_corpus_series_round_trips_structurally() {
        let Some(corpus) = require_or_skip_corpus() else {
            return;
        };
        verify_fetched_corpus(&corpus);
        let mut series_count = 0usize;
        let mut chart_parts = 0usize;
        for path in manifest_paths() {
            let package = OpcPackage::open(corpus.join(path))
                .unwrap_or_else(|error| panic!("{path}: open failed: {error}"));
            for (part, xml) in &package.parts {
                if !is_chart_part(part) {
                    continue;
                }
                let chart = CT_ChartSpace::from_xml(xml)
                    .unwrap_or_else(|error| panic!("{path} {part}: parse failed: {error}"));
                let parsed_series =
                    chart.chart.plot_area.series().unwrap_or_else(|error| {
                        panic!("{path} {part}: series parse failed: {error}")
                    });
                for series in parsed_series {
                    let written = series.to_xml().unwrap_or_else(|error| {
                        panic!("{path} {part}: series write failed: {error}")
                    });
                    let reparsed = Series::from_xml(&written).unwrap_or_else(|error| {
                        panic!("{path} {part}: written series parse failed: {error}")
                    });
                    assert_eq!(series, reparsed, "{path} {part}: series model changed");
                    series_count += 1;
                }
                chart_parts += 1;
            }
        }
        assert!(
            series_count > 0,
            "the pinned corpus contained no supported series"
        );
        eprintln!(
            "ChartML series corpus gate checked {series_count} series across {chart_parts} chart parts"
        );
    }

    #[test]
    fn chart_space_reads_aliases_and_writes_fixed_prefixes_in_schema_order() {
        let xml = br#"<ch:chartSpace xmlns:ch="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:rel="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:x="urn:producer"><x:before/><ch:chart><ch:plotVisOnly val="0"/><ch:legend><x:legend/></ch:legend><ch:title><x:title/></ch:title><ch:plotArea><x:plot/></ch:plotArea><ch:dispBlanksAs val="span"/></ch:chart><ch:txPr><d:bodyPr/><d:p/></ch:txPr><ch:spPr><d:solidFill><d:srgbClr val="112233"/></d:solidFill></ch:spPr><x:after/></ch:chartSpace>"#;
        let parsed = CT_ChartSpace::from_xml(xml).unwrap();
        assert!(!parsed.chart.auto_title_deleted);
        assert!(!parsed.chart.plot_vis_only);
        assert_eq!(parsed.chart.disp_blanks_as, DispBlanksAs::Span);
        assert!(parsed.chart.title.is_some());
        assert!(parsed.chart.legend.is_some());
        assert!(parsed.sp_pr.is_some());
        assert!(parsed.tx_pr.is_some());

        let written = String::from_utf8(parsed.to_xml().unwrap()).unwrap();
        assert!(written.starts_with(
            r#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships""#
        ));
        for tag in [
            "<c:chart>",
            "<c:title>",
            "<c:plotArea>",
            "<c:legend>",
            "<c:plotVisOnly",
            "<c:dispBlanksAs",
            "<c:spPr>",
            "<c:txPr>",
        ] {
            assert!(written.contains(tag), "missing fixed-prefix tag {tag}");
        }
        let positions: Vec<_> = [
            "<c:title>",
            "<c:plotArea>",
            "<c:legend>",
            "<c:plotVisOnly",
            "<c:dispBlanksAs",
            "<c:spPr>",
            "<c:txPr>",
        ]
        .iter()
        .map(|tag| written.find(tag).unwrap())
        .collect();
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(written.contains("<a:solidFill><a:srgbClr"));
        assert!(written.contains("<a:bodyPr/><a:p/>"));
        assert_eq!(parsed, CT_ChartSpace::from_xml(written.as_bytes()).unwrap());
    }

    #[test]
    fn public_core_edits_keep_preserved_children_in_their_schema_slots() {
        let xml = br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><c:chart><c:pivotFmts><c:pivotFmt/></c:pivotFmts><c:plotArea/></c:chart><c:externalData r:id="rId1"/></c:chartSpace>"#;
        let mut parsed = CT_ChartSpace::from_xml(xml).unwrap();
        parsed.chart.auto_title_deleted = true;
        parsed.sp_pr = Some(CT_ShapeProperties::from_xml(br#"<a:spPr/>"#).unwrap());
        parsed.tx_pr =
            Some(CT_TextBody::from_xml(br#"<a:txBody><a:bodyPr/><a:p/></a:txBody>"#).unwrap());
        let written = String::from_utf8(parsed.to_xml().unwrap()).unwrap();
        let positions: Vec<_> = [
            "<c:autoTitleDeleted",
            "<c:pivotFmts>",
            "<c:plotArea",
            "<c:spPr",
            "<c:txPr",
            "<c:externalData",
        ]
        .iter()
        .map(|tag| written.find(tag).unwrap())
        .collect();
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(CT_ChartSpace::from_xml(written.as_bytes()).is_ok());
    }

    #[test]
    fn malformed_core_chart_values_return_errors_without_panicking() {
        let cases: &[&[u8]] = &[
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"/>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart/></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart><c:plotArea/><c:plotArea/></c:chart></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart><c:autoTitleDeleted val="yes"/><c:plotArea/></c:chart></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart><c:plotArea/><c:dispBlanksAs val="empty"/></c:chart></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="urn:wrong"><c:chart><c:plotArea/></c:chart></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:x="urn:foreign"><x:chart><x:plotArea/></x:chart></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart><c:plotArea/></c:chart><c:chart><c:plotArea/></c:chart></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart><c:title/><c:title/><c:plotArea/></c:chart></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart><c:plotArea/><c:plotVisOnly><c:child/></c:plotVisOnly></c:chart></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:foreign"><c:chart><c:plotArea/></c:chart><c:txPr><a:bodyPr/><a:p><x:r><x:t>foreign</x:t></x:r></a:p></c:txPr></c:chartSpace>"#,
            br#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:foreign"><c:chart><c:plotArea/></c:chart><c:spPr><a:solidFill><x:srgbClr val="112233"/></a:solidFill></c:spPr></c:chartSpace>"#,
        ];
        for xml in cases {
            let result = std::panic::catch_unwind(|| CT_ChartSpace::from_xml(xml));
            assert!(result.is_ok(), "ChartML parser panicked");
            assert!(
                result.unwrap().is_err(),
                "malformed ChartML parsed: {}",
                String::from_utf8_lossy(xml)
            );
        }
    }

    #[test]
    fn core_chart_shells_preserve_unmodelled_children_byte_for_byte() {
        let preserved = [
            br#"<x:root x:id="1"><x:nested>one &amp; two</x:nested><!--root--></x:root>"#
                .as_slice(),
            br#"<x:title x:id="2"><x:nested><![CDATA[title]]></x:nested></x:title>"#.as_slice(),
            br#"<x:plot x:id="3"><x:nested>plot</x:nested></x:plot>"#.as_slice(),
            br#"<x:legend x:id="4"><x:nested>legend</x:nested></x:legend>"#.as_slice(),
            br#"<x:extension x:id="5"><x:nested>extension</x:nested></x:extension>"#.as_slice(),
            br#"<!--space-comment-->"#.as_slice(),
            br#"<?space processing?>"#.as_slice(),
            br#"<!--chart-comment-->"#.as_slice(),
            br#"<?title processing?>"#.as_slice(),
            br#"<!--flag-comment-->"#.as_slice(),
            br#"<?flag processing?>"#.as_slice(),
            br#"<x:producerShape/>"#.as_slice(),
            br#"<x:producerText/>"#.as_slice(),
        ];
        let xml = br#"<q:chartSpace xmlns:q="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:x="urn:producer"><!--space-comment--><?space processing?><x:root x:id="1"><x:nested>one &amp; two</x:nested><!--root--></x:root><q:chart><!--chart-comment--><q:autoTitleDeleted val="0" x:keep="one &amp; two"><!--flag-comment--><?flag processing?></q:autoTitleDeleted><q:title data="keep"><?title processing?><x:title x:id="2"><x:nested><![CDATA[title]]></x:nested></x:title></q:title><q:plotArea data="keep"><x:plot x:id="3"><x:nested>plot</x:nested></x:plot></q:plotArea><q:legend data="keep"><x:legend x:id="4"><x:nested>legend</x:nested></x:legend></q:legend><x:extension x:id="5"><x:nested>extension</x:nested></x:extension></q:chart><q:spPr><x:producerShape/></q:spPr><q:txPr><a:bodyPr/><a:p><x:producerText/></a:p></q:txPr></q:chartSpace>"#;
        let parsed = CT_ChartSpace::from_xml(xml).unwrap();
        let written = parsed.to_xml().unwrap();
        for raw in preserved {
            assert!(
                written.windows(raw.len()).any(|window| window == raw),
                "preserved subtree changed: {}",
                String::from_utf8_lossy(raw)
            );
        }
        let written_text = String::from_utf8(written.clone()).unwrap();
        assert!(written_text.contains(r#"x:keep="one &amp; two""#));
        assert_eq!(parsed, CT_ChartSpace::from_xml(&written).unwrap());
    }

    #[test]
    fn every_corpus_chart_part_round_trips_structurally() {
        let Some(corpus) = require_or_skip_corpus() else {
            return;
        };
        verify_fetched_corpus(&corpus);
        let mut chart_parts = 0usize;
        let mut decks_with_charts = HashSet::new();
        for path in manifest_paths() {
            let package = OpcPackage::open(corpus.join(path))
                .unwrap_or_else(|error| panic!("{path}: open failed: {error}"));
            for (part, xml) in &package.parts {
                if !is_chart_part(part) {
                    continue;
                }
                let parsed = CT_ChartSpace::from_xml(xml)
                    .unwrap_or_else(|error| panic!("{path} {part}: parse failed: {error}"));
                let written = parsed
                    .to_xml()
                    .unwrap_or_else(|error| panic!("{path} {part}: write failed: {error}"));
                let original_preserved = preserved_chartml_regions(xml)
                    .unwrap_or_else(|error| panic!("{path} {part}: original scan failed: {error}"));
                let written_preserved = preserved_chartml_regions(&written)
                    .unwrap_or_else(|error| panic!("{path} {part}: written scan failed: {error}"));
                assert_eq!(
                    original_preserved.len(),
                    written_preserved.len(),
                    "{path} {part}: preserved ChartML region count changed"
                );
                for (index, (original, written)) in original_preserved
                    .iter()
                    .zip(&written_preserved)
                    .enumerate()
                {
                    assert_eq!(
                        original.parent_path, written.parent_path,
                        "{path} {part}: preserved region {index} parent changed"
                    );
                    assert_eq!(
                        original.boundary, written.boundary,
                        "{path} {part}: preserved region {index} boundary changed"
                    );
                    assert_eq!(
                        original.sibling_order, written.sibling_order,
                        "{path} {part}: preserved region {index} order changed"
                    );
                    assert_eq!(
                        original.xml, written.xml,
                        "{path} {part}: preserved region {index} bytes changed"
                    );
                }
                let reparsed = CT_ChartSpace::from_xml(&written)
                    .unwrap_or_else(|error| panic!("{path} {part}: written parse failed: {error}"));
                assert_eq!(parsed, reparsed, "{path} {part}: core model changed");
                chart_parts += 1;
                decks_with_charts.insert(path);
            }
        }
        assert!(
            chart_parts > 0,
            "the pinned corpus contained no chart parts"
        );
        eprintln!(
            "ChartML corpus gate checked {chart_parts} chart parts across {} decks",
            decks_with_charts.len()
        );
    }

    #[derive(Debug, Eq, PartialEq)]
    struct PreservedChartRegion {
        parent_path: Vec<Vec<u8>>,
        boundary: usize,
        sibling_order: usize,
        xml: Vec<u8>,
    }

    struct PreservationFrame {
        local_name: Vec<u8>,
        boundary: usize,
        next_sibling: usize,
    }

    fn preserved_chartml_regions(xml: &[u8]) -> Result<Vec<PreservedChartRegion>, String> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        let mut parents: Vec<PreservationFrame> = Vec::new();
        let mut preserved = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(element)) => {
                    let child = local_name(element.name().as_ref()).to_vec();
                    if parents.is_empty() && child == b"chartSpace" {
                        parents.push(PreservationFrame {
                            local_name: child,
                            boundary: 0,
                            next_sibling: 0,
                        });
                    } else if let Some(parent) = parents.last()
                        && is_opaque_chartml_child(&parent.local_name, &child)
                    {
                        let boundary =
                            preserved_child_boundary(&parent.local_name, &child, parent.boundary);
                        let xml = capture_element(&mut reader, &element)
                            .map_err(|error| error.to_string())?;
                        record_preserved_region(&mut parents, &mut preserved, boundary, xml);
                        advance_preservation_boundary(&mut parents, &child);
                    } else {
                        advance_preservation_boundary(&mut parents, &child);
                        if is_chartml_preservation_parent(&child) {
                            parents.push(PreservationFrame {
                                local_name: child,
                                boundary: 0,
                                next_sibling: 0,
                            });
                        } else {
                            capture_element(&mut reader, &element)
                                .map_err(|error| error.to_string())?;
                        }
                    }
                }
                Ok(Event::Empty(element)) => {
                    let name = element.name();
                    let child = local_name(name.as_ref()).to_vec();
                    if let Some(parent) = parents.last()
                        && is_opaque_chartml_child(&parent.local_name, &child)
                    {
                        let boundary =
                            preserved_child_boundary(&parent.local_name, &child, parent.boundary);
                        let xml =
                            capture_empty_element(&element).map_err(|error| error.to_string())?;
                        record_preserved_region(&mut parents, &mut preserved, boundary, xml);
                    }
                    advance_preservation_boundary(&mut parents, &child);
                }
                Ok(
                    event @ (Event::Comment(_)
                    | Event::PI(_)
                    | Event::CData(_)
                    | Event::GeneralRef(_)),
                ) if !parents.is_empty() => {
                    let boundary = parents.last().expect("preservation parent").boundary;
                    let xml = capture_event(event).map_err(|error| error.to_string())?;
                    record_preserved_region(&mut parents, &mut preserved, boundary, xml);
                }
                Ok(Event::End(_)) => {
                    parents.pop();
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(error) => return Err(error.to_string()),
            }
            buffer.clear();
        }
        Ok(preserved)
    }

    fn record_preserved_region(
        parents: &mut [PreservationFrame],
        preserved: &mut Vec<PreservedChartRegion>,
        boundary: usize,
        xml: Vec<u8>,
    ) {
        let sibling_order = parents.last().expect("preservation parent").next_sibling;
        parents
            .last_mut()
            .expect("preservation parent")
            .next_sibling += 1;
        preserved.push(PreservedChartRegion {
            parent_path: parents
                .iter()
                .map(|parent| parent.local_name.clone())
                .collect(),
            boundary,
            sibling_order,
            xml,
        });
    }

    fn advance_preservation_boundary(parents: &mut [PreservationFrame], child: &[u8]) {
        let Some(parent) = parents.last_mut() else {
            return;
        };
        parent.boundary = parent
            .boundary
            .max(preserved_boundary_after(&parent.local_name, child));
    }

    fn preserved_child_boundary(parent: &[u8], child: &[u8], current: usize) -> usize {
        match parent {
            b"chartSpace" => match child {
                b"date1904" => 0,
                b"lang" => 1,
                b"roundedCorners" => 2,
                b"style" => 3,
                b"clrMapOvr" => 4,
                b"pivotSource" => 5,
                b"protection" => 6,
                b"externalData" => 10,
                b"printSettings" => 11,
                b"userShapes" => 12,
                b"extLst" => 13,
                _ => current,
            },
            b"chart" => match child {
                b"pivotFmts" => 2,
                b"view3D" => 3,
                b"floor" => 4,
                b"sideWall" => 5,
                b"backWall" => 6,
                b"showDLblsOverMax" | b"extLst" => 11,
                _ => current,
            },
            _ => current,
        }
    }

    fn preserved_boundary_after(parent: &[u8], child: &[u8]) -> usize {
        match parent {
            b"chartSpace" => match child {
                b"date1904" => 1,
                b"lang" => 2,
                b"roundedCorners" => 3,
                b"style" => 4,
                b"clrMapOvr" => 5,
                b"pivotSource" => 6,
                b"protection" => 7,
                b"chart" => 8,
                b"spPr" => 9,
                b"txPr" => 10,
                b"externalData" => 11,
                b"printSettings" => 12,
                b"userShapes" => 13,
                b"extLst" => 14,
                _ => 0,
            },
            b"chart" => match child {
                b"title" => 1,
                b"autoTitleDeleted" => 2,
                b"pivotFmts" => 3,
                b"view3D" => 4,
                b"floor" => 5,
                b"sideWall" => 6,
                b"backWall" => 7,
                b"plotArea" => 8,
                b"legend" => 9,
                b"plotVisOnly" => 10,
                b"dispBlanksAs" => 11,
                b"showDLblsOverMax" | b"extLst" => 11,
                _ => 0,
            },
            b"pieChart" => match child {
                b"ser" => 1,
                b"dLbls" => 2,
                b"firstSliceAng" => 3,
                _ => 0,
            },
            b"doughnutChart" => match child {
                b"ser" => 1,
                b"dLbls" => 2,
                b"firstSliceAng" => 3,
                b"holeSize" => 4,
                _ => 0,
            },
            b"areaChart" => match child {
                b"grouping" => 1,
                b"ser" => 2,
                b"dLbls" => 3,
                b"axId" => 4,
                _ => 0,
            },
            b"scatterChart" | b"radarChart" => match child {
                b"scatterStyle" | b"radarStyle" => 1,
                b"ser" => 2,
                b"dLbls" => 3,
                b"axId" => 4,
                _ => 0,
            },
            _ => 0,
        }
    }

    fn is_opaque_chartml_child(parent: &[u8], child: &[u8]) -> bool {
        match parent {
            b"chartSpace" => !matches!(child, b"chart" | b"spPr" | b"txPr"),
            b"chart" => !matches!(
                child,
                b"title"
                    | b"autoTitleDeleted"
                    | b"plotArea"
                    | b"legend"
                    | b"plotVisOnly"
                    | b"dispBlanksAs"
            ),
            b"plotArea" => !matches!(
                child,
                b"barChart"
                    | b"lineChart"
                    | b"pieChart"
                    | b"doughnutChart"
                    | b"areaChart"
                    | b"scatterChart"
                    | b"radarChart"
                    | b"catAx"
                    | b"valAx"
                    | b"dateAx"
                    | b"serAx"
            ),
            b"pieChart" => !matches!(child, b"ser" | b"dLbls" | b"firstSliceAng"),
            b"doughnutChart" => {
                !matches!(child, b"ser" | b"dLbls" | b"firstSliceAng" | b"holeSize")
            }
            b"areaChart" => !matches!(child, b"grouping" | b"ser" | b"dLbls" | b"axId"),
            b"scatterChart" => !matches!(child, b"scatterStyle" | b"ser" | b"dLbls" | b"axId"),
            b"radarChart" => !matches!(child, b"radarStyle" | b"ser" | b"dLbls" | b"axId"),
            b"title" | b"legend" => true,
            _ => false,
        }
    }

    fn is_chartml_preservation_parent(parent: &[u8]) -> bool {
        matches!(
            parent,
            b"chartSpace"
                | b"chart"
                | b"title"
                | b"plotArea"
                | b"pieChart"
                | b"doughnutChart"
                | b"areaChart"
                | b"scatterChart"
                | b"radarChart"
                | b"legend"
                | b"autoTitleDeleted"
                | b"plotVisOnly"
                | b"dispBlanksAs"
        )
    }

    #[test]
    fn rpptx_chart_is_an_unpublished_workspace_member() {
        let manifest = include_str!("../Cargo.toml");
        let workspace = include_str!("../../../Cargo.toml");
        assert!(manifest.contains("name = \"rpptx-chart\""));
        assert!(manifest.contains("version = \"0.0.0\""));
        assert!(manifest.contains("publish = false"));
        for dependency in [
            "oxml-core.workspace",
            "oxml-drawing.workspace",
            "quick-xml.workspace",
        ] {
            assert!(manifest.contains(dependency));
        }
        assert!(!manifest.contains("rpptx.workspace"));
        assert!(!manifest.contains("rdocx.workspace"));
        assert!(workspace.contains("\"crates/rpptx-chart\""));
        assert!(
            workspace
                .contains("rpptx-chart = { path = \"crates/rpptx-chart\", version = \"0.0.0\" }")
        );
    }

    fn first_series_xml(xml: &[u8]) -> Option<Vec<u8>> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer).ok()? {
                Event::Start(element) if matches_local_name(element.name().as_ref(), b"ser") => {
                    return capture_element(&mut reader, &element).ok();
                }
                Event::Empty(element) if matches_local_name(element.name().as_ref(), b"ser") => {
                    return capture_empty_element(&element).ok();
                }
                Event::Eof => return None,
                _ => {}
            }
            buffer.clear();
        }
    }

    fn replace_once(haystack: &[u8], needle: &[u8], replacement: &[u8]) -> Vec<u8> {
        let offset = haystack
            .windows(needle.len())
            .position(|window| window == needle)
            .expect("serialized source series occurs in chart part");
        assert!(
            !haystack[offset + needle.len()..]
                .windows(needle.len())
                .any(|window| window == needle),
            "source series unexpectedly occurs more than once"
        );
        let mut result = Vec::with_capacity(haystack.len() - needle.len() + replacement.len());
        result.extend_from_slice(&haystack[..offset]);
        result.extend_from_slice(replacement);
        result.extend_from_slice(&haystack[offset + needle.len()..]);
        result
    }

    fn assert_command_version(program: &str, arguments: &[&str], expected: &str) {
        let output = Command::new(program)
            .args(arguments)
            .output()
            .unwrap_or_else(|error| panic!("run {program} version check: {error}"));
        assert!(
            output.status.success(),
            "{program} version check failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).expect("version stdout is UTF-8");
        let stderr = String::from_utf8(output.stderr).expect("version stderr is UTF-8");
        let actual = if stdout.trim().is_empty() {
            stderr.trim()
        } else {
            stdout.trim()
        };
        assert_eq!(
            actual.lines().next(),
            Some(expected),
            "{program} version drift"
        );
    }

    fn sha256(path: &Path) -> String {
        let output = Command::new("shasum")
            .args(["-a", "256"])
            .arg(path)
            .output()
            .unwrap_or_else(|error| panic!("{}: run shasum: {error}", path.display()));
        assert!(
            output.status.success(),
            "{}: shasum failed: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("shasum output is UTF-8")
            .split_whitespace()
            .next()
            .expect("shasum digest")
            .to_owned()
    }

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn corpus_dir() -> PathBuf {
        std::env::var_os("RDOCX_PPTX_CORPUS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| workspace_root().join("corpus/pptx"))
    }

    fn require_or_skip_corpus() -> Option<PathBuf> {
        let corpus = corpus_dir();
        if corpus.is_dir() {
            return Some(corpus);
        }
        assert_ne!(
            std::env::var_os("RDOCX_PPTX_CORPUS_REQUIRED").as_deref(),
            Some(std::ffi::OsStr::new("1")),
            "the pinned corpus is required but {} does not exist",
            corpus.display()
        );
        eprintln!(
            "ChartML corpus gate skipped because {} is absent",
            corpus.display()
        );
        None
    }

    fn verify_fetched_corpus(corpus: &Path) {
        let status = Command::new("python3")
            .arg(workspace_root().join("scripts/fetch_pptx_corpus.py"))
            .arg("--check")
            .env("RDOCX_PPTX_CORPUS_DIR", corpus)
            .status()
            .expect("run corpus verifier");
        assert!(status.success(), "pinned corpus verification failed");
    }

    fn manifest_paths() -> Vec<&'static str> {
        let mut lines = MANIFEST.lines();
        assert_eq!(lines.next(), Some("path\tproducer\tsha256\turl"));
        let paths: Vec<_> = lines
            .map(|line| line.split('\t').next().expect("manifest path"))
            .collect();
        assert_eq!(paths.len(), EXPECTED_DECKS);
        paths
    }

    fn is_chart_part(part: &str) -> bool {
        let Some(stem) = part
            .strip_prefix("/ppt/charts/chart")
            .and_then(|name| name.strip_suffix(".xml"))
        else {
            return false;
        };
        !stem.is_empty() && stem.bytes().all(|byte| byte.is_ascii_digit())
    }

    fn plot_series(index: u32) -> String {
        format!(
            r#"<q:ser><q:idx val="{index}"/><q:order val="{index}"/><q:cat><q:strRef><q:f>Sheet1!$A$2:$A$3</q:f><q:strCache><q:ptCount val="2"/><q:pt idx="0"><q:v>North</q:v></q:pt><q:pt idx="1"><q:v>South</q:v></q:pt></q:strCache></q:strRef></q:cat><q:val><q:numRef><q:f>Sheet1!$B$2:$B$3</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount val="2"/><q:pt idx="0"><q:v>1</q:v></q:pt><q:pt idx="1"><q:v>2</q:v></q:pt></q:numCache></q:numRef></q:val></q:ser>"#
        )
    }

    fn sparse_category_series() -> &'static str {
        r#"<q:ser><q:idx val="0"/><q:order val="0"/><q:cat><q:strRef><q:f>Sheet1!$A$2:$A$4</q:f><q:strCache><q:ptCount val="3"/><q:pt idx="0"><q:v>North</q:v></q:pt><q:pt idx="2"><q:v>West</q:v></q:pt></q:strCache></q:strRef></q:cat><q:val><q:numRef><q:f>Sheet1!$B$2:$B$4</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount val="3"/><q:pt idx="0"><q:v>1</q:v></q:pt><q:pt idx="2"><q:v>2</q:v></q:pt></q:numCache></q:numRef></q:val></q:ser>"#
    }

    fn sparse_scatter_series() -> &'static str {
        r#"<q:ser><q:idx val="0"/><q:order val="0"/><q:xVal><q:numRef><q:f>Sheet1!$A$2:$A$5</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount val="4"/><q:pt idx="0"><q:v>10</q:v></q:pt><q:pt idx="2"><q:v>30</q:v></q:pt><q:pt idx="3"><q:v>40</q:v></q:pt></q:numCache></q:numRef></q:xVal><q:yVal><q:numRef><q:f>Sheet1!$B$2:$B$5</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount val="4"/><q:pt idx="0"><q:v>100</q:v></q:pt><q:pt idx="1"><q:v>200</q:v></q:pt><q:pt idx="3"><q:v>400</q:v></q:pt></q:numCache></q:numRef></q:yVal></q:ser>"#
    }

    fn chart_with_plot(plot: &str) -> String {
        format!(
            r#"<q:chartSpace xmlns:q="{C_NS}" xmlns:a="{A_NS}" xmlns:r="{R_NS}" xmlns:x="urn:producer"><q:chart><q:plotArea>{plot}<q:catAx><q:axId val="-1884094432"/><q:scaling/><q:axPos val="b"/><q:crossAx val="-1884097184"/></q:catAx><q:valAx><q:axId val="-1884097184"/><q:scaling/><q:axPos val="l"/><q:crossAx val="-1884094432"/></q:valAx></q:plotArea></q:chart></q:chartSpace>"#
        )
    }

    fn chart_with_optional_axes(plot: &str, axes: bool) -> String {
        let axis_xml = if axes {
            r#"<q:catAx><q:axId val="-1884094432"/><q:scaling/><q:axPos val="b"/><q:crossAx val="-1884097184"/></q:catAx><q:valAx><q:axId val="-1884097184"/><q:scaling/><q:axPos val="l"/><q:crossAx val="-1884094432"/></q:valAx>"#
        } else {
            ""
        };
        format!(
            r#"<q:chartSpace xmlns:q="{C_NS}" xmlns:a="{A_NS}" xmlns:r="{R_NS}" xmlns:x="urn:producer"><q:chart><q:plotArea>{plot}{axis_xml}</q:plotArea></q:chart></q:chartSpace>"#
        )
    }

    fn numeric_plot_series(index: u32) -> String {
        format!(
            r#"<q:ser><q:idx val="{index}"/><q:order val="{index}"/><q:xVal><q:numRef><q:f>Sheet1!$A$2:$A$3</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount val="2"/><q:pt idx="0"><q:v>1</q:v></q:pt><q:pt idx="1"><q:v>2</q:v></q:pt></q:numCache></q:numRef></q:xVal><q:yVal><q:numRef><q:f>Sheet1!$B$2:$B$3</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount val="2"/><q:pt idx="0"><q:v>4</q:v></q:pt><q:pt idx="1"><q:v>8</q:v></q:pt></q:numCache></q:numRef></q:yVal></q:ser>"#
        )
    }

    fn scatter_without_x(index: u32) -> String {
        format!(
            r#"<q:ser><q:idx val="{index}"/><q:order val="{index}"/><q:yVal><q:numRef><q:f>Sheet1!$B$2</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount val="1"/><q:pt idx="0"><q:v>4</q:v></q:pt></q:numCache></q:numRef></q:yVal></q:ser>"#
        )
    }

    fn scatter_without_y(index: u32) -> String {
        format!(
            r#"<q:ser><q:idx val="{index}"/><q:order val="{index}"/><q:xVal><q:numRef><q:f>Sheet1!$A$2</q:f><q:numCache><q:formatCode>General</q:formatCode><q:ptCount val="1"/><q:pt idx="0"><q:v>1</q:v></q:pt></q:numCache></q:numRef></q:xVal></q:ser>"#
        )
    }

    fn remaining_plot_fixtures() -> Vec<(&'static str, String, bool)> {
        let series = plot_series(0);
        let scatter = numeric_plot_series(0);
        vec![
            (
                "pieChart",
                format!(
                    r#"<q:pieChart x:keep="pie"><q:varyColors val="1"/>{series}<q:dLbls><q:showVal val="1"/></q:dLbls><q:firstSliceAng val="30"/><q:extLst><x:pie/></q:extLst></q:pieChart>"#
                ),
                false,
            ),
            (
                "doughnutChart",
                format!(
                    r#"<q:doughnutChart x:keep="doughnut"><q:varyColors val="1"/>{series}<q:dLbls><q:showPercent val="1"/></q:dLbls><q:firstSliceAng val="45"/><q:holeSize val="60"/><q:extLst><x:doughnut/></q:extLst></q:doughnutChart>"#
                ),
                false,
            ),
            (
                "areaChart",
                format!(
                    r#"<q:areaChart x:keep="area"><q:grouping val="standard"/><q:varyColors val="0"/>{series}<q:dLbls><q:showVal val="1"/></q:dLbls><q:dropLines><x:line/></q:dropLines><q:axId val="-1884094432"/><q:axId val="-1884097184"/><q:extLst><x:area/></q:extLst></q:areaChart>"#
                ),
                true,
            ),
            (
                "scatterChart",
                format!(
                    r#"<q:scatterChart x:keep="scatter"><q:scatterStyle val="marker"/><q:varyColors val="0"/>{scatter}<q:dLbls><q:showVal val="1"/></q:dLbls><q:axId val="-1884094432"/><q:axId val="-1884097184"/><q:extLst><x:scatter/></q:extLst></q:scatterChart>"#
                ),
                true,
            ),
            (
                "radarChart",
                format!(
                    r#"<q:radarChart x:keep="radar"><q:radarStyle val="marker"/><q:varyColors val="0"/>{series}<q:dLbls><q:showVal val="1"/></q:dLbls><q:axId val="-1884094432"/><q:axId val="-1884097184"/><q:extLst><x:radar/></q:extLst></q:radarChart>"#
                ),
                true,
            ),
        ]
    }

    fn bar_plot(extra: &str) -> String {
        format!(
            r#"<q:barChart x:keep="bar"><q:barDir val="col"/><q:grouping val="clustered"/><q:varyColors val="1"/>{}<q:dLbls><q:showVal val="1"/></q:dLbls><q:gapWidth val="150"/><q:overlap val="0"/><q:serLines><x:line/></q:serLines><q:axId val="-1884094432"/><q:axId val="-1884097184"/>{extra}</q:barChart>"#,
            plot_series(0)
        )
    }

    fn line_plot(extra: &str) -> String {
        format!(
            r#"<q:lineChart x:keep="line"><q:grouping val="standard"/><q:varyColors val="0"/>{}<q:dLbls><q:showVal val="1"/></q:dLbls><q:dropLines><x:line/></q:dropLines><q:marker val="1"/><q:smooth val="0"/><q:axId val="-1884094432"/><q:axId val="-1884097184"/>{extra}</q:lineChart>"#,
            plot_series(0)
        )
    }

    #[test]
    fn bar_and_line_plots_round_trip_and_render() {
        for (xml, is_bar) in [
            (chart_with_plot(&bar_plot("")), true),
            (chart_with_plot(&line_plot("")), false),
        ] {
            let parsed = CT_ChartSpace::from_xml(xml.as_bytes()).unwrap();
            let plots = parsed.chart.plot_area.plots().unwrap();
            assert_eq!(plots.len(), 1);
            assert_eq!(matches!(plots[0], Plot::Bar { .. }), is_bar);
            assert_eq!(matches!(plots[0], Plot::Line { .. }), !is_bar);
            assert_eq!(parsed.chart.plot_area.axes().unwrap().len(), 2);
            let written = parsed.to_xml().unwrap();
            let reparsed = CT_ChartSpace::from_xml(&written).unwrap();
            assert_eq!(parsed, reparsed);
        }
        if let Some(corpus) = require_or_skip_corpus() {
            verify_bar_and_line_viewer_gate(&corpus);
        }
    }

    #[test]
    fn ppm_parser_preserves_whitespace_valued_first_pixels() {
        for first_pixel in *b" \n\r\t" {
            let mut ppm = b"P6\n1 1\n255\n".to_vec();
            ppm.extend_from_slice(&[first_pixel, 0x7f, 0xff]);
            let (width, height, pixels) = ppm_pixels(&ppm);
            assert_eq!((width, height), (1, 1));
            assert_eq!(pixels, [first_pixel, 0x7f, 0xff]);
        }

        let ppm = b"P6\r\n1 1\r\n255\r\n\x7f\xff";
        assert_eq!(ppm_pixels(ppm).2, [b'\n', 0x7f, 0xff]);
    }

    #[test]
    fn bar_and_line_plots_write_fixed_prefixes_in_schema_order() {
        for xml in [
            chart_with_plot(&bar_plot(r#"<q:extLst><x:bar-tail/></q:extLst>"#)),
            chart_with_plot(&line_plot(r#"<q:extLst><x:line-tail/></q:extLst>"#)),
        ] {
            let parsed = CT_ChartSpace::from_xml(xml.as_bytes()).unwrap();
            let written = String::from_utf8(parsed.to_xml().unwrap()).unwrap();
            assert!(written.contains("<c:barChart") || written.contains("<c:lineChart"));
            assert!(!written.contains("<q:barChart"));
            assert!(!written.contains("<q:lineChart"));
            if written.contains("<c:barChart") {
                let positions: Vec<_> = [
                    "<c:barDir",
                    "<c:grouping",
                    "<q:varyColors",
                    "<c:ser",
                    "<c:dLbls",
                    "<c:gapWidth",
                    "<c:overlap",
                    "<q:serLines",
                    "<c:axId",
                    "<q:extLst",
                ]
                .iter()
                .map(|tag| written.find(tag).unwrap())
                .collect();
                assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
            } else {
                let positions: Vec<_> = [
                    "<c:grouping",
                    "<q:varyColors",
                    "<c:ser",
                    "<c:dLbls",
                    "<q:dropLines",
                    "<c:marker",
                    "<c:smooth",
                    "<c:axId",
                    "<q:extLst",
                ]
                .iter()
                .map(|tag| written.find(tag).unwrap())
                .collect();
                assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
            }
            assert!(CT_ChartSpace::from_xml(written.as_bytes()).is_ok());
        }

        for plot in [bar_plot(""), line_plot("")] {
            let first = plot_series(0);
            let with_series_boundary = plot.replacen(
                &first,
                &format!(
                    "{first}<!--between-series--><?series-boundary?>\n  {}",
                    plot_series(1)
                ),
                1,
            );
            let with_axis_boundary = with_series_boundary.replacen(
                r#"<q:axId val="-1884094432"/><q:axId val="-1884097184"/>"#,
                r#"<q:axId val="-1884094432"/><!--between-axis-ids--><?axis-boundary?>
  <q:axId val="-1884097184"/>"#,
                1,
            );
            let parsed =
                CT_ChartSpace::from_xml(chart_with_plot(&with_axis_boundary).as_bytes()).unwrap();
            let written = String::from_utf8(parsed.to_xml().unwrap()).unwrap();
            assert!(written.contains("</c:ser><!--between-series--><?series-boundary?>\n  <c:ser"));
            assert!(written.contains(
                r#"<c:axId val="-1884094432"/><!--between-axis-ids--><?axis-boundary?>
  <c:axId val="-1884097184"/>"#
            ));
            assert_eq!(parsed, CT_ChartSpace::from_xml(written.as_bytes()).unwrap());
        }
    }

    #[test]
    fn malformed_bar_and_line_plots_return_errors_without_panicking() {
        let valid_series = plot_series(0);
        let cases = [
            format!(r#"<q:barChart><q:grouping val="clustered"/>{valid_series}<q:axId val="1"/><q:axId val="2"/></q:barChart>"#),
            format!(r#"<q:barChart><q:barDir val="diagonal"/><q:grouping val="clustered"/>{valid_series}<q:axId val="1"/><q:axId val="2"/></q:barChart>"#),
            format!(r#"<q:barChart><q:barDir val="col"/><q:grouping val="clustered"/>{valid_series}<q:gapWidth val="501"/><q:axId val="1"/><q:axId val="2"/></q:barChart>"#),
            format!(r#"<q:barChart><q:barDir val="col"/><q:grouping val="clustered"/>{valid_series}<q:overlap val="-101"/><q:axId val="1"/><q:axId val="2"/></q:barChart>"#),
            format!(r#"<q:lineChart><q:grouping val="clustered"/>{valid_series}<q:axId val="1"/><q:axId val="2"/></q:lineChart>"#),
            format!(r#"<q:lineChart><q:grouping val="standard"/>{valid_series}<q:marker val="maybe"/><q:axId val="1"/><q:axId val="2"/></q:lineChart>"#),
            r#"<q:lineChart><q:grouping val="standard"/><q:axId val="1"/><q:axId val="2"/></q:lineChart>"#.to_owned(),
            format!(r#"<q:lineChart><q:grouping val="standard"/>{valid_series}<q:axId val="1"/><q:axId val="1"/></q:lineChart>"#),
            format!(r#"<q:barChart><q:barDir val="col"/><q:grouping val="clustered"/><q:grouping val="stacked"/>{valid_series}<q:axId val="-1884094432"/><q:axId val="-1884097184"/></q:barChart>"#),
            format!(r#"<q:barChart><q:barDir val="col"/><q:grouping val="clustered"/>{valid_series}<q:axId val="1"/><q:axId val="2"/></q:barChart>"#),
        ];
        for plot in cases {
            let xml = chart_with_plot(&plot);
            let result = std::panic::catch_unwind(|| CT_ChartSpace::from_xml(xml.as_bytes()));
            assert!(result.is_ok(), "plot parser panicked for {plot}");
            assert!(result.unwrap().is_err(), "malformed plot parsed: {plot}");
        }

        let missing_axis = format!(
            r#"<q:chartSpace xmlns:q="{C_NS}" xmlns:a="{A_NS}" xmlns:r="{R_NS}"><q:chart><q:plotArea>{}<q:catAx><q:axId val="-1884094432"/><q:scaling/><q:axPos val="b"/><q:crossAx val="-1884097184"/></q:catAx></q:plotArea></q:chart></q:chartSpace>"#,
            bar_plot("")
        );
        assert!(CT_ChartSpace::from_xml(missing_axis.as_bytes()).is_err());

        let values =
            NumericData::new("S!$A$1".to_owned(), "General".to_owned(), vec![1.0]).unwrap();
        let series = vec![Series::new(0, 0, values)];
        let ids = [AxisId::new(1).unwrap(), AxisId::new(2).unwrap()];
        let mut invalid =
            Plot::bar(BarDirection::Column, BarGrouping::Clustered, series, ids).unwrap();
        if let Plot::Bar { gap_width, .. } = &mut invalid {
            *gap_width = 501;
        }
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn unsupported_and_combo_plots_remain_byte_preserved() {
        let bar = bar_plot("");
        let line = line_plot("");
        let combo = chart_with_plot(&format!("<!--before-->{bar}<?between?>{line}<!--after-->"));
        let parsed = CT_ChartSpace::from_xml(combo.as_bytes()).unwrap();
        assert!(parsed.chart.plot_area.plots().is_err());
        let written = parsed.to_xml().unwrap();
        for raw in [
            bar.as_bytes(),
            line.as_bytes(),
            br#"<!--before-->"#.as_slice(),
            br#"<?between?>"#.as_slice(),
            br#"<!--after-->"#.as_slice(),
        ] {
            assert!(written.windows(raw.len()).any(|window| window == raw));
        }

        let unsupported = chart_with_plot(
            r#"<q:bar3DChart x:keep="three"><q:barDir val="col"/><q:ser><x:opaque/></q:ser></q:bar3DChart>"#,
        );
        let parsed = CT_ChartSpace::from_xml(unsupported.as_bytes()).unwrap();
        assert!(parsed.chart.plot_area.plots().is_err());
        assert!(
            String::from_utf8(parsed.to_xml().unwrap())
                .unwrap()
                .contains(r#"<q:bar3DChart x:keep="three">"#)
        );
    }

    #[test]
    fn public_plot_edits_preserve_axes_and_unselected_payloads() {
        let xml = chart_with_plot(&bar_plot(
            r#"<q:extLst><q:ext uri="keep"><x:tail/></q:ext></q:extLst>"#,
        ));
        let mut parsed = CT_ChartSpace::from_xml(xml.as_bytes()).unwrap();
        let axes_before = parsed.chart.plot_area.axes().unwrap();
        match &mut parsed.chart.plot_area.plots_mut().unwrap()[0] {
            Plot::Bar {
                gap_width,
                overlap,
                series,
                ..
            } => {
                *gap_width = 225;
                *overlap = -25;
                series[0].index = 7;
                series[0].values.values[0] = 9.0;
            }
            _ => panic!("expected bar plot"),
        }
        let written = String::from_utf8(parsed.to_xml().unwrap()).unwrap();
        assert!(written.contains(r#"<c:gapWidth val="225"/>"#));
        assert!(written.contains(r#"<c:overlap val="-25"/>"#));
        assert!(written.contains("<c:v>9</c:v>"));
        assert!(written.contains("<q:varyColors val=\"1\"/>"));
        assert!(written.find("<q:varyColors").unwrap() < written.find("<c:ser").unwrap());
        assert!(written.contains("<q:serLines><x:line/></q:serLines>"));
        assert!(written.contains(r#"<q:extLst><q:ext uri="keep"><x:tail/></q:ext></q:extLst>"#));
        let reparsed = CT_ChartSpace::from_xml(written.as_bytes()).unwrap();
        assert_eq!(axes_before, reparsed.chart.plot_area.axes().unwrap());

        let first = plot_series(0);
        let colliding_series =
            bar_plot("").replacen(&first, &format!("{first}{}", plot_series(1)), 1);
        let mut collision =
            CT_ChartSpace::from_xml(chart_with_plot(&colliding_series).as_bytes()).unwrap();
        if let Plot::Bar { series, .. } = &mut collision.chart.plot_area.plots_mut().unwrap()[0] {
            series[0].index = series[1].index;
            series[0].order = series[1].order;
        }
        let collision = String::from_utf8(collision.to_xml().unwrap()).unwrap();
        assert!(collision.find("<q:varyColors").unwrap() < collision.find("<c:ser").unwrap());

        let mut inserted = CT_ChartSpace::from_xml(xml.as_bytes()).unwrap();
        if let Plot::Bar { series, .. } = &mut inserted.chart.plot_area.plots_mut().unwrap()[0] {
            let mut new_series = series[0].clone();
            new_series.index = 99;
            new_series.order = 99;
            series.insert(0, new_series);
        }
        let inserted = String::from_utf8(inserted.to_xml().unwrap()).unwrap();
        assert!(inserted.find("<q:varyColors").unwrap() < inserted.find("<c:ser").unwrap());

        let axis_payload = bar_plot("").replacen(
            r#"<q:axId val="-1884094432"/><q:axId val="-1884097184"/>"#,
            r#"<q:axId val="-1884094432" x:slot="category"/><!--axis-anchor--><q:axId val="-1884097184" x:slot="value"/>"#,
            1,
        );
        let mut swapped =
            CT_ChartSpace::from_xml(chart_with_plot(&axis_payload).as_bytes()).unwrap();
        if let Plot::Bar { axis_ids, .. } = &mut swapped.chart.plot_area.plots_mut().unwrap()[0] {
            axis_ids.swap(0, 1);
        }
        let swapped = String::from_utf8(swapped.to_xml().unwrap()).unwrap();
        let anchor = swapped.find("<!--axis-anchor-->").unwrap();
        let value_axis = swapped
            .find(r#"<c:axId val="-1884097184" x:slot="value"/>"#)
            .unwrap();
        let category_axis = swapped
            .find(r#"<c:axId val="-1884094432" x:slot="category"/>"#)
            .unwrap();
        let series_lines = swapped.find("<q:serLines").unwrap();
        assert!(series_lines < value_axis && series_lines < category_axis);
        assert!(anchor < value_axis && value_axis < category_axis);

        let mut replaced = CT_ChartSpace::from_xml(xml.as_bytes()).unwrap();
        let Plot::Bar {
            series, axis_ids, ..
        } = &replaced.chart.plot_area.plots().unwrap()[0]
        else {
            panic!("expected bar plot");
        };
        let line = Plot::line(Grouping::Standard, series.clone(), *axis_ids).unwrap();
        replaced.chart.plot_area.plots_mut().unwrap()[0] = line;
        assert!(replaced.to_xml().is_err());
    }

    #[test]
    fn every_corpus_bar_and_line_plot_round_trips_structurally() {
        let Some(corpus) = require_or_skip_corpus() else {
            return;
        };
        verify_fetched_corpus(&corpus);
        let mut typed_bar_count = 0usize;
        let mut typed_line_count = 0usize;
        let mut opaque_bar_count = 0usize;
        let mut opaque_line_count = 0usize;
        let mut opaque_combo_count = 0usize;
        for path in manifest_paths() {
            let package = OpcPackage::open(corpus.join(path))
                .unwrap_or_else(|error| panic!("{path}: open failed: {error}"));
            for (part, xml) in &package.parts {
                if !is_chart_part(part) {
                    continue;
                }
                let chart = CT_ChartSpace::from_xml(xml)
                    .unwrap_or_else(|error| panic!("{path} {part}: parse failed: {error}"));
                match chart.chart.plot_area.plots() {
                    Ok(plots) => {
                        for plot in plots {
                            match plot {
                                Plot::Bar { .. } => typed_bar_count += 1,
                                Plot::Line { .. } => typed_line_count += 1,
                                _ => {}
                            }
                        }
                    }
                    Err(_) => {
                        let mut preserved_plot_count = 0usize;
                        for raw in chart.chart.plot_area.raw_children.at(0) {
                            if let Ok(Some(local)) = super::chart_root_local(
                                raw,
                                &chart.chart.plot_area.namespace_bindings,
                            ) {
                                match local.as_slice() {
                                    b"barChart" => {
                                        opaque_bar_count += 1;
                                        preserved_plot_count += 1;
                                    }
                                    b"lineChart" => {
                                        opaque_line_count += 1;
                                        preserved_plot_count += 1;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if preserved_plot_count > 1 {
                            opaque_combo_count += 1;
                        }
                    }
                }
                let written = chart
                    .to_xml()
                    .unwrap_or_else(|error| panic!("{path} {part}: chart write failed: {error}"));
                let reparsed = CT_ChartSpace::from_xml(&written).unwrap_or_else(|error| {
                    panic!("{path} {part}: written chart parse failed: {error}")
                });
                assert_eq!(chart, reparsed, "{path} {part}: chart model changed");
            }
        }
        assert_eq!(typed_bar_count, 11, "typed corpus bar coverage changed");
        assert_eq!(typed_line_count, 2, "typed corpus line coverage changed");
        assert_eq!(opaque_bar_count, 1, "preserved combo bar coverage changed");
        assert_eq!(
            opaque_line_count, 1,
            "preserved combo line coverage changed"
        );
        assert_eq!(opaque_combo_count, 1, "preserved combo coverage changed");
        assert_eq!(typed_bar_count + opaque_bar_count, 12);
        assert_eq!(typed_line_count + opaque_line_count, 3);
        eprintln!(
            "ChartML plot corpus gate checked {typed_bar_count} typed bar, {typed_line_count} typed line, and one preserved bar-line combination"
        );
    }

    fn verify_remaining_plot_viewer_gate(corpus: &Path) {
        verify_fetched_corpus(corpus);
        assert_command_version("soffice", &["--version"], LIBREOFFICE_VERSION);
        assert_command_version("pdftoppm", &["-v"], PDFTOPPM_VERSION);
        let source = corpus.join("bar-chart.pptx");
        let source_sha = sha256(&source);
        let temp_root = std::env::temp_dir().join(format!(
            "rpptx-chart-f122-viewer-gate-{}",
            std::process::id()
        ));
        if temp_root.exists() {
            fs::remove_dir_all(&temp_root).expect("remove stale F-122 evidence");
        }
        fs::create_dir_all(&temp_root).expect("create F-122 evidence root");

        for (kind, plot, axes) in remaining_plot_fixtures() {
            let mut package = OpcPackage::open(&source).expect("open F-122 viewer source");
            package.set_part(
                "/ppt/charts/chart1.xml",
                CT_ChartSpace::from_xml(chart_with_optional_axes(&plot, axes).as_bytes())
                    .expect("parse F-122 viewer chart")
                    .to_xml()
                    .expect("serialize F-122 viewer chart"),
            );
            let unbound = temp_root.join(format!("{kind}-candidate.pptx"));
            package.save(&unbound).expect("save F-122 viewer candidate");
            let candidate_sha = sha256(&unbound);
            let evidence = temp_root.join(format!("{source_sha}-{candidate_sha}"));
            fs::create_dir(&evidence).expect("create SHA-bound F-122 evidence directory");
            let candidate = evidence.join(format!("{kind}-candidate.pptx"));
            fs::rename(&unbound, &candidate).expect("bind F-122 candidate to SHA");
            assert_eq!(sha256(&candidate), candidate_sha);
            let render = render_deck_to_ppm(&candidate, &evidence, kind, "candidate");
            let bytes = fs::read(&render).expect("read F-122 viewer PPM");
            let (width, height, pixels) = ppm_pixels(&bytes);
            let left = width / 5;
            let right = width * 4 / 5;
            let top = height * 3 / 20;
            let bottom = height * 17 / 20;
            let mut nonblank = 0usize;
            for y in top..bottom {
                for x in left..right {
                    let offset = (y * width + x) * 3;
                    if pixels[offset..offset + 3]
                        .iter()
                        .any(|channel| *channel < 245)
                    {
                        nonblank += 1;
                    }
                }
            }
            const CHART_RECTANGLE_NONBLANK_THRESHOLD: usize = 1_000;
            assert!(
                nonblank >= CHART_RECTANGLE_NONBLANK_THRESHOLD,
                "{kind}: chart rectangle [{left},{top}) to [{right},{bottom}) contains {nonblank} nonblank pixels, below {CHART_RECTANGLE_NONBLANK_THRESHOLD}"
            );
            eprintln!(
                "F-122 {kind} viewer gate source deck {source_sha}, candidate deck {candidate_sha}, render {}, chart rectangle [{left},{top}) to [{right},{bottom}), RGB<245 nonblank pixels {nonblank} >= {CHART_RECTANGLE_NONBLANK_THRESHOLD}",
                sha256(&render)
            );
        }
        fs::remove_dir_all(&temp_root).expect("remove F-122 viewer evidence");
    }

    fn verify_bar_and_line_viewer_gate(corpus: &Path) {
        verify_fetched_corpus(corpus);
        assert_command_version("soffice", &["--version"], LIBREOFFICE_VERSION);
        assert_command_version("pdftoppm", &["-v"], PDFTOPPM_VERSION);
        let temp_root = std::env::temp_dir().join(format!(
            "rpptx-chart-f121-viewer-gate-{}",
            std::process::id()
        ));
        if temp_root.exists() {
            fs::remove_dir_all(&temp_root).expect("remove stale F-121 evidence");
        }
        fs::create_dir_all(&temp_root).expect("create F-121 evidence root");

        for (kind, deck) in [("bar", "bar-chart.pptx"), ("line", "line-chart.pptx")] {
            let source = corpus.join(deck);
            let original_sha = sha256(&source);
            let mut package = OpcPackage::open(&source)
                .unwrap_or_else(|error| panic!("{deck}: open viewer source: {error}"));
            let chart_part = "/ppt/charts/chart1.xml";
            let original_parts = package.parts.clone();
            let parsed = CT_ChartSpace::from_xml(
                package
                    .get_part(chart_part)
                    .unwrap_or_else(|| panic!("{deck}: missing {chart_part}")),
            )
            .unwrap_or_else(|error| panic!("{deck}: parse viewer chart: {error}"));
            assert!(
                parsed.chart.plot_area.plots().is_ok(),
                "{deck}: plot is opaque"
            );
            package.set_part(
                chart_part,
                parsed
                    .to_xml()
                    .unwrap_or_else(|error| panic!("{deck}: serialize viewer chart: {error}")),
            );
            for (part, bytes) in &original_parts {
                if part == chart_part {
                    assert_ne!(package.parts[part], *bytes, "{deck}: chart did not rewrite");
                } else {
                    assert_eq!(package.parts[part], *bytes, "{deck}: changed part {part}");
                }
            }

            let unbound = temp_root.join(format!("{kind}-candidate.pptx"));
            package
                .save(&unbound)
                .unwrap_or_else(|error| panic!("{deck}: save viewer candidate: {error}"));
            let candidate_sha = sha256(&unbound);
            let evidence = temp_root.join(format!("{original_sha}-{candidate_sha}"));
            fs::create_dir(&evidence).expect("create SHA-bound F-121 evidence directory");
            let original = evidence.join(format!("{kind}-original.pptx"));
            let candidate = evidence.join(format!("{kind}-candidate.pptx"));
            fs::copy(&source, &original).expect("copy SHA-bound viewer original");
            fs::rename(&unbound, &candidate).expect("bind viewer candidate to SHA");
            assert_eq!(sha256(&original), original_sha);
            assert_eq!(sha256(&candidate), candidate_sha);

            let original_render = render_deck_to_ppm(&original, &evidence, kind, "original");
            let candidate_render = render_deck_to_ppm(&candidate, &evidence, kind, "candidate");
            let original_bytes = fs::read(&original_render).expect("read original PPM");
            let candidate_bytes = fs::read(&candidate_render).expect("read candidate PPM");
            let normalized_mae = normalized_ppm_mae(&original_bytes, &candidate_bytes);
            assert!(
                normalized_mae <= PLOT_RENDER_NORMALIZED_MAE_THRESHOLD,
                "{deck}: normalized RGB MAE {normalized_mae:.8} exceeds {:.8}",
                PLOT_RENDER_NORMALIZED_MAE_THRESHOLD
            );
            eprintln!(
                "F-121 {kind} viewer gate original deck {original_sha}, candidate deck {candidate_sha}, original render {}, candidate render {}, normalized RGB MAE {normalized_mae:.8} <= {:.8}",
                sha256(&original_render),
                sha256(&candidate_render),
                PLOT_RENDER_NORMALIZED_MAE_THRESHOLD
            );
        }
        fs::remove_dir_all(&temp_root).expect("remove F-121 viewer evidence");
    }

    fn render_deck_to_ppm(deck: &Path, evidence: &Path, kind: &str, side: &str) -> PathBuf {
        let profile = std::env::temp_dir().join(format!(
            "rpptx-chart-f121-{kind}-{side}-profile-{}",
            std::process::id()
        ));
        if profile.exists() {
            fs::remove_dir_all(&profile).expect("remove stale F-121 viewer profile");
        }
        let profile_argument = format!("-env:UserInstallation=file://{}", profile.display());
        let conversion = Command::new("soffice")
            .args([
                "--headless",
                &profile_argument,
                "--convert-to",
                "pdf:impress_pdf_Export",
                "--outdir",
            ])
            .arg(evidence)
            .arg(deck)
            .output()
            .expect("run pinned LibreOffice F-121 viewer gate");
        assert!(
            conversion.status.success(),
            "{}: LibreOffice conversion failed: {}",
            deck.display(),
            String::from_utf8_lossy(&conversion.stderr)
        );
        let pdf = deck.with_extension("pdf");
        assert!(
            pdf.is_file(),
            "LibreOffice did not create {}",
            pdf.display()
        );
        let prefix = evidence.join(format!("{kind}-{side}"));
        let raster = Command::new("pdftoppm")
            .args(["-f", "1", "-singlefile", "-r", "150"])
            .arg(&pdf)
            .arg(&prefix)
            .output()
            .expect("run pinned Poppler F-121 raster gate");
        assert!(
            raster.status.success(),
            "{}: pdftoppm failed: {}",
            pdf.display(),
            String::from_utf8_lossy(&raster.stderr)
        );
        if profile.exists() {
            fs::remove_dir_all(&profile).expect("remove F-121 viewer profile");
        }
        prefix.with_extension("ppm")
    }

    fn normalized_ppm_mae(left: &[u8], right: &[u8]) -> f64 {
        let (left_width, left_height, left_pixels) = ppm_pixels(left);
        let (right_width, right_height, right_pixels) = ppm_pixels(right);
        assert_eq!((left_width, left_height), (right_width, right_height));
        assert_eq!(left_pixels.len(), right_pixels.len());
        let difference: u64 = left_pixels
            .iter()
            .zip(right_pixels)
            .map(|(left, right)| u64::from(left.abs_diff(*right)))
            .sum();
        difference as f64 / (left_pixels.len() as f64 * 255.0)
    }

    fn ppm_pixels(bytes: &[u8]) -> (usize, usize, &[u8]) {
        let mut cursor = 0usize;
        let mut tokens = Vec::new();
        while tokens.len() < 4 {
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if cursor < bytes.len() && bytes[cursor] == b'#' {
                while cursor < bytes.len() && bytes[cursor] != b'\n' {
                    cursor += 1;
                }
                continue;
            }
            let start = cursor;
            while cursor < bytes.len() && !bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            tokens.push(std::str::from_utf8(&bytes[start..cursor]).expect("PPM header UTF-8"));
        }
        assert_eq!(tokens[0], "P6");
        let width = tokens[1].parse::<usize>().expect("PPM width");
        let height = tokens[2].parse::<usize>().expect("PPM height");
        assert_eq!(tokens[3], "255");
        assert!(
            cursor < bytes.len() && bytes[cursor].is_ascii_whitespace(),
            "PPM header must end with whitespace"
        );
        cursor += 1;
        let pixels = &bytes[cursor..];
        assert_eq!(pixels.len(), width * height * 3, "PPM pixel length");
        (width, height, pixels)
    }
}
