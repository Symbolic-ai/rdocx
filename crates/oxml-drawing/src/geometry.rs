use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::f64::consts::PI;
use std::fmt;

const ANGLE_UNITS_PER_DEGREE: f64 = 60_000.0;
const QUARTER_CIRCLE: f64 = 90.0 * ANGLE_UNITS_PER_DEGREE;
const MAX_ARC_SEGMENTS: usize = 4_096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuideOp {
    MulDiv,
    AddSub,
    AddDiv,
    IfElse,
    Abs,
    At2,
    Cat2,
    Cos,
    Max,
    Min,
    Mod,
    Pin,
    Sat2,
    Sin,
    Sqrt,
    Tan,
    Val,
}

impl GuideOp {
    pub fn parse(token: &str) -> Result<Self, GeometryError> {
        match token {
            "*/" => Ok(Self::MulDiv),
            "+-" => Ok(Self::AddSub),
            "+/" => Ok(Self::AddDiv),
            "?:" => Ok(Self::IfElse),
            "abs" => Ok(Self::Abs),
            "at2" => Ok(Self::At2),
            "cat2" => Ok(Self::Cat2),
            "cos" => Ok(Self::Cos),
            "max" => Ok(Self::Max),
            "min" => Ok(Self::Min),
            "mod" => Ok(Self::Mod),
            "pin" => Ok(Self::Pin),
            "sat2" => Ok(Self::Sat2),
            "sin" => Ok(Self::Sin),
            "sqrt" => Ok(Self::Sqrt),
            "tan" => Ok(Self::Tan),
            "val" => Ok(Self::Val),
            _ => Err(GeometryError::UnknownGuideOperation(token.to_owned())),
        }
    }

    fn argument_count(self) -> usize {
        match self {
            Self::Abs | Self::Sqrt | Self::Val => 1,
            Self::At2 | Self::Cos | Self::Max | Self::Min | Self::Sin | Self::Tan => 2,
            Self::MulDiv
            | Self::AddSub
            | Self::AddDiv
            | Self::IfElse
            | Self::Cat2
            | Self::Mod
            | Self::Pin
            | Self::Sat2 => 3,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GuideOperand {
    Literal(f64),
    Guide(String),
}

impl GuideOperand {
    pub fn parse(value: &str) -> Result<Self, GeometryError> {
        match value.parse::<f64>() {
            Ok(value) if value.is_finite() => Ok(Self::Literal(value)),
            Ok(_) => Err(GeometryError::NonFiniteValue(value.to_owned())),
            Err(_) if value.is_empty() => Err(GeometryError::EmptyGuideOperand),
            Err(_) => Ok(Self::Guide(value.to_owned())),
        }
    }
}

impl From<f64> for GuideOperand {
    fn from(value: f64) -> Self {
        Self::Literal(value)
    }
}

impl From<&str> for GuideOperand {
    fn from(value: &str) -> Self {
        Self::Guide(value.to_owned())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Guide {
    pub name: String,
    pub op: GuideOp,
    pub args: Vec<GuideOperand>,
}

impl Guide {
    pub fn parse(name: impl Into<String>, formula: &str) -> Result<Self, GeometryError> {
        let mut parts = formula.split_ascii_whitespace();
        let token = parts.next().ok_or(GeometryError::EmptyGuideFormula)?;
        let op = GuideOp::parse(token)?;
        let args = parts
            .map(GuideOperand::parse)
            .collect::<Result<Vec<_>, _>>()?;
        let expected = op.argument_count();
        if args.len() != expected {
            return Err(GeometryError::WrongArgumentCount {
                operation: token.to_owned(),
                expected,
                actual: args.len(),
            });
        }
        Ok(Self {
            name: name.into(),
            op,
            args,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PathCommand {
    MoveTo {
        x: GuideOperand,
        y: GuideOperand,
    },
    LineTo {
        x: GuideOperand,
        y: GuideOperand,
    },
    CubicTo {
        x1: GuideOperand,
        y1: GuideOperand,
        x2: GuideOperand,
        y2: GuideOperand,
        x: GuideOperand,
        y: GuideOperand,
    },
    ArcTo {
        width_radius: GuideOperand,
        height_radius: GuideOperand,
        start_angle: GuideOperand,
        sweep_angle: GuideOperand,
    },
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EvaluatedPathCommand {
    MoveTo {
        x: f64,
        y: f64,
    },
    LineTo {
        x: f64,
        y: f64,
    },
    CubicTo {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x: f64,
        y: f64,
    },
    Close,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GeometryError {
    EmptyGuideFormula,
    EmptyGuideOperand,
    UnknownGuideOperation(String),
    WrongArgumentCount {
        operation: String,
        expected: usize,
        actual: usize,
    },
    UnknownGuide(String),
    DuplicateGuide(String),
    UnknownAdjustOverride(String),
    DivisionByZero,
    NonFiniteValue(String),
    PathHasNoCurrentPoint,
    InvalidArcRadius,
    ArcSweepTooLarge,
}

impl fmt::Display for GeometryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyGuideFormula => formatter.write_str("empty guide formula"),
            Self::EmptyGuideOperand => formatter.write_str("empty guide operand"),
            Self::UnknownGuideOperation(operation) => {
                write!(formatter, "unknown guide operation: {operation}")
            }
            Self::WrongArgumentCount {
                operation,
                expected,
                actual,
            } => write!(
                formatter,
                "guide operation {operation} expects {expected} arguments, got {actual}"
            ),
            Self::UnknownGuide(name) => write!(formatter, "unknown guide: {name}"),
            Self::DuplicateGuide(name) => write!(formatter, "duplicate guide: {name}"),
            Self::UnknownAdjustOverride(name) => {
                write!(formatter, "unknown adjust override: {name}")
            }
            Self::DivisionByZero => formatter.write_str("division by zero"),
            Self::NonFiniteValue(context) => write!(formatter, "non-finite value: {context}"),
            Self::PathHasNoCurrentPoint => {
                formatter.write_str("path command requires a current point")
            }
            Self::InvalidArcRadius => formatter.write_str("arc radii must be positive"),
            Self::ArcSweepTooLarge => formatter.write_str("arc sweep requires too many segments"),
        }
    }
}

impl Error for GeometryError {}

#[derive(Clone, Debug)]
pub struct GuideEvaluator {
    values: BTreeMap<String, f64>,
}

impl GuideEvaluator {
    pub fn new(width: f64, height: f64) -> Result<Self, GeometryError> {
        ensure_finite(width, "shape width")?;
        ensure_finite(height, "shape height")?;

        let mut evaluator = Self {
            values: BTreeMap::new(),
        };
        evaluator.seed("w", width);
        evaluator.seed("h", height);
        evaluator.seed("l", 0.0);
        evaluator.seed("t", 0.0);
        evaluator.seed("r", width);
        evaluator.seed("b", height);
        evaluator.seed("hc", width / 2.0);
        evaluator.seed("vc", height / 2.0);
        evaluator.seed("ss", width.min(height));
        evaluator.seed("ls", width.max(height));

        for divisor in [2_u32, 3, 4, 5, 6, 8, 10, 32] {
            evaluator.seed(&format!("wd{divisor}"), width / f64::from(divisor));
        }
        for divisor in [2_u32, 3, 4, 5, 6, 8] {
            evaluator.seed(&format!("hd{divisor}"), height / f64::from(divisor));
        }
        for divisor in [2_u32, 4, 6, 8, 16, 32] {
            evaluator.seed(
                &format!("ssd{divisor}"),
                width.min(height) / f64::from(divisor),
            );
        }
        evaluator.seed("cd2", 180.0 * ANGLE_UNITS_PER_DEGREE);
        evaluator.seed("cd4", 90.0 * ANGLE_UNITS_PER_DEGREE);
        evaluator.seed("cd8", 45.0 * ANGLE_UNITS_PER_DEGREE);
        evaluator.seed("3cd4", 270.0 * ANGLE_UNITS_PER_DEGREE);
        evaluator.seed("3cd8", 135.0 * ANGLE_UNITS_PER_DEGREE);
        evaluator.seed("5cd8", 225.0 * ANGLE_UNITS_PER_DEGREE);
        evaluator.seed("7cd8", 315.0 * ANGLE_UNITS_PER_DEGREE);
        Ok(evaluator)
    }

    pub fn value(&self, name: &str) -> Result<f64, GeometryError> {
        self.values
            .get(name)
            .copied()
            .ok_or_else(|| GeometryError::UnknownGuide(name.to_owned()))
    }

    pub fn apply_adjust_values(
        &mut self,
        adjustments: &[Guide],
        overrides: &BTreeMap<String, f64>,
    ) -> Result<(), GeometryError> {
        let declared = adjustments
            .iter()
            .map(|guide| guide.name.as_str())
            .collect::<BTreeSet<_>>();
        if let Some(name) = overrides
            .keys()
            .find(|name| !declared.contains(name.as_str()))
        {
            return Err(GeometryError::UnknownAdjustOverride(name.clone()));
        }

        for adjustment in adjustments {
            let value = match overrides.get(&adjustment.name) {
                Some(value) => {
                    ensure_finite(*value, &adjustment.name)?;
                    *value
                }
                None => self.evaluate_operation(adjustment.op, &adjustment.args)?,
            };
            self.insert_named(&adjustment.name, value)?;
        }
        Ok(())
    }

    pub fn evaluate_guides(&mut self, guides: &[Guide]) -> Result<(), GeometryError> {
        for guide in guides {
            let value = self.evaluate_operation(guide.op, &guide.args)?;
            self.insert_named(&guide.name, value)?;
        }
        Ok(())
    }

    pub fn evaluate_path(
        &self,
        commands: &[PathCommand],
    ) -> Result<Vec<EvaluatedPathCommand>, GeometryError> {
        let mut output = Vec::with_capacity(commands.len());
        let mut current = None;
        let mut subpath_start = None;

        for command in commands {
            match command {
                PathCommand::MoveTo { x, y } => {
                    let point = (self.resolve(x)?, self.resolve(y)?);
                    output.push(EvaluatedPathCommand::MoveTo {
                        x: point.0,
                        y: point.1,
                    });
                    current = Some(point);
                    subpath_start = Some(point);
                }
                PathCommand::LineTo { x, y } => {
                    current.ok_or(GeometryError::PathHasNoCurrentPoint)?;
                    let point = (self.resolve(x)?, self.resolve(y)?);
                    output.push(EvaluatedPathCommand::LineTo {
                        x: point.0,
                        y: point.1,
                    });
                    current = Some(point);
                }
                PathCommand::CubicTo {
                    x1,
                    y1,
                    x2,
                    y2,
                    x,
                    y,
                } => {
                    current.ok_or(GeometryError::PathHasNoCurrentPoint)?;
                    let command = EvaluatedPathCommand::CubicTo {
                        x1: self.resolve(x1)?,
                        y1: self.resolve(y1)?,
                        x2: self.resolve(x2)?,
                        y2: self.resolve(y2)?,
                        x: self.resolve(x)?,
                        y: self.resolve(y)?,
                    };
                    if let EvaluatedPathCommand::CubicTo { x, y, .. } = command {
                        current = Some((x, y));
                    }
                    output.push(command);
                }
                PathCommand::ArcTo {
                    width_radius,
                    height_radius,
                    start_angle,
                    sweep_angle,
                } => {
                    let start = current.ok_or(GeometryError::PathHasNoCurrentPoint)?;
                    let cubics = flatten_arc(
                        start,
                        self.resolve(width_radius)?,
                        self.resolve(height_radius)?,
                        self.resolve(start_angle)?,
                        self.resolve(sweep_angle)?,
                    )?;
                    if let Some(EvaluatedPathCommand::CubicTo { x, y, .. }) = cubics.last() {
                        current = Some((*x, *y));
                    }
                    output.extend(cubics);
                }
                PathCommand::Close => {
                    current.ok_or(GeometryError::PathHasNoCurrentPoint)?;
                    output.push(EvaluatedPathCommand::Close);
                    current = subpath_start;
                }
            }
        }
        Ok(output)
    }

    fn seed(&mut self, name: &str, value: f64) {
        self.values.insert(name.to_owned(), value);
    }

    fn insert_named(&mut self, name: &str, value: f64) -> Result<(), GeometryError> {
        ensure_finite(value, name)?;
        if self.values.contains_key(name) {
            return Err(GeometryError::DuplicateGuide(name.to_owned()));
        }
        self.values.insert(name.to_owned(), value);
        Ok(())
    }

    fn resolve(&self, operand: &GuideOperand) -> Result<f64, GeometryError> {
        match operand {
            GuideOperand::Literal(value) => {
                ensure_finite(*value, "literal")?;
                Ok(*value)
            }
            GuideOperand::Guide(name) => self.value(name),
        }
    }

    fn evaluate_operation(
        &self,
        op: GuideOp,
        operands: &[GuideOperand],
    ) -> Result<f64, GeometryError> {
        let expected = op.argument_count();
        if operands.len() != expected {
            return Err(GeometryError::WrongArgumentCount {
                operation: format!("{op:?}"),
                expected,
                actual: operands.len(),
            });
        }
        let args = operands
            .iter()
            .map(|operand| self.resolve(operand))
            .collect::<Result<Vec<_>, _>>()?;

        let value = match op {
            GuideOp::MulDiv => checked_div(args[0] * args[1], args[2])?,
            GuideOp::AddSub => args[0] + args[1] - args[2],
            GuideOp::AddDiv => checked_div(args[0] + args[1], args[2])?,
            GuideOp::IfElse => {
                if args[0] > 0.0 {
                    args[1]
                } else {
                    args[2]
                }
            }
            GuideOp::Abs => args[0].abs(),
            GuideOp::At2 => radians_to_angle(args[1].atan2(args[0])),
            GuideOp::Cat2 => args[0] * args[2].atan2(args[1]).cos(),
            GuideOp::Cos => args[0] * angle_to_radians(args[1]).cos(),
            GuideOp::Max => args[0].max(args[1]),
            GuideOp::Min => args[0].min(args[1]),
            GuideOp::Mod => args[0].hypot(args[1]).hypot(args[2]),
            GuideOp::Pin => {
                if args[1] < args[0] {
                    args[0]
                } else if args[1] > args[2] {
                    args[2]
                } else {
                    args[1]
                }
            }
            GuideOp::Sat2 => args[0] * args[2].atan2(args[1]).sin(),
            GuideOp::Sin => args[0] * angle_to_radians(args[1]).sin(),
            GuideOp::Sqrt => args[0].abs().sqrt(),
            GuideOp::Tan => args[0] * angle_to_radians(args[1]).tan(),
            GuideOp::Val => args[0],
        };
        ensure_finite(value, "guide result")?;
        Ok(value)
    }
}

fn checked_div(numerator: f64, denominator: f64) -> Result<f64, GeometryError> {
    if denominator == 0.0 {
        return Err(GeometryError::DivisionByZero);
    }
    Ok(numerator / denominator)
}

fn angle_to_radians(angle: f64) -> f64 {
    angle / ANGLE_UNITS_PER_DEGREE * PI / 180.0
}

fn radians_to_angle(radians: f64) -> f64 {
    radians * 180.0 / PI * ANGLE_UNITS_PER_DEGREE
}

fn ensure_finite(value: f64, context: &str) -> Result<(), GeometryError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(GeometryError::NonFiniteValue(context.to_owned()))
    }
}

fn flatten_arc(
    current: (f64, f64),
    width_radius: f64,
    height_radius: f64,
    start_angle: f64,
    sweep_angle: f64,
) -> Result<Vec<EvaluatedPathCommand>, GeometryError> {
    for (value, context) in [
        (current.0, "arc start x"),
        (current.1, "arc start y"),
        (width_radius, "arc width radius"),
        (height_radius, "arc height radius"),
        (start_angle, "arc start angle"),
        (sweep_angle, "arc sweep angle"),
    ] {
        ensure_finite(value, context)?;
    }
    if width_radius <= 0.0 || height_radius <= 0.0 {
        return Err(GeometryError::InvalidArcRadius);
    }
    if sweep_angle == 0.0 {
        return Ok(Vec::new());
    }

    let segment_count = (sweep_angle.abs() / QUARTER_CIRCLE).ceil();
    if segment_count > MAX_ARC_SEGMENTS as f64 {
        return Err(GeometryError::ArcSweepTooLarge);
    }
    let segment_count = segment_count as usize;
    let segment_sweep = sweep_angle / segment_count as f64;
    let start_radians = angle_to_radians(start_angle);
    let center = (
        current.0 - width_radius * start_radians.cos(),
        current.1 - height_radius * start_radians.sin(),
    );
    let mut cubics = Vec::with_capacity(segment_count);

    for index in 0..segment_count {
        let angle_1 = angle_to_radians(start_angle + segment_sweep * index as f64);
        let angle_2 = angle_to_radians(start_angle + segment_sweep * (index + 1) as f64);
        let alpha = 4.0 / 3.0 * ((angle_2 - angle_1) / 4.0).tan();
        let point_1 = (
            center.0 + width_radius * angle_1.cos(),
            center.1 + height_radius * angle_1.sin(),
        );
        let point_2 = (
            center.0 + width_radius * angle_2.cos(),
            center.1 + height_radius * angle_2.sin(),
        );
        let tangent_1 = (-width_radius * angle_1.sin(), height_radius * angle_1.cos());
        let tangent_2 = (-width_radius * angle_2.sin(), height_radius * angle_2.cos());
        let command = EvaluatedPathCommand::CubicTo {
            x1: point_1.0 + alpha * tangent_1.0,
            y1: point_1.1 + alpha * tangent_1.1,
            x2: point_2.0 - alpha * tangent_2.0,
            y2: point_2.1 - alpha * tangent_2.1,
            x: point_2.0,
            y: point_2.1,
        };
        if let EvaluatedPathCommand::CubicTo {
            x1,
            y1,
            x2,
            y2,
            x,
            y,
        } = command
        {
            for value in [x1, y1, x2, y2, x, y] {
                ensure_finite(value, "arc cubic coordinate")?;
            }
        }
        cubics.push(command);
    }
    Ok(cubics)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn literal(value: f64) -> GuideOperand {
        GuideOperand::Literal(value)
    }

    fn guide(name: &str) -> GuideOperand {
        GuideOperand::Guide(name.to_owned())
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1.0e-9, "{actual} != {expected}");
    }

    #[test]
    fn hand_written_custom_geometry_guides_produce_expected_path_coordinates() {
        let adjustments = [Guide::parse("adj1", "val 25000").unwrap()];
        let guides = [
            Guide::parse("x1", "*/ w adj1 100000").unwrap(),
            Guide::parse("y1", "+/ hd2 0 2").unwrap(),
            Guide::parse("x2", "+- r 0 x1").unwrap(),
            Guide::parse("y2", "?: adj1 75 25").unwrap(),
        ];
        let overrides = BTreeMap::from([("adj1".to_owned(), 20_000.0)]);
        let mut evaluator = GuideEvaluator::new(100.0, 100.0).unwrap();
        assert_eq!(evaluator.value("wd32").unwrap(), 3.125);
        assert_eq!(evaluator.value("hd8").unwrap(), 12.5);
        assert_eq!(evaluator.value("ssd32").unwrap(), 3.125);
        assert_eq!(evaluator.value("3cd8").unwrap(), 8_100_000.0);
        evaluator
            .apply_adjust_values(&adjustments, &overrides)
            .unwrap();
        evaluator.evaluate_guides(&guides).unwrap();

        let path = evaluator
            .evaluate_path(&[
                PathCommand::MoveTo {
                    x: guide("x1"),
                    y: guide("y1"),
                },
                PathCommand::LineTo {
                    x: guide("x2"),
                    y: guide("y2"),
                },
            ])
            .unwrap();
        assert_eq!(
            path,
            [
                EvaluatedPathCommand::MoveTo { x: 20.0, y: 25.0 },
                EvaluatedPathCommand::LineTo { x: 80.0, y: 75.0 },
            ]
        );
    }

    #[test]
    fn all_seventeen_formula_tokens_parse_and_evaluate_with_drawingml_argument_order() {
        let cases = [
            ("*/ 6 7 2", GuideOp::MulDiv, 21.0),
            ("+- 10 4 3", GuideOp::AddSub, 11.0),
            ("+/ 10 4 2", GuideOp::AddDiv, 7.0),
            ("?: 1 5 6", GuideOp::IfElse, 5.0),
            ("abs -7", GuideOp::Abs, 7.0),
            ("at2 1 1", GuideOp::At2, 2_700_000.0),
            ("cat2 10 3 4", GuideOp::Cat2, 6.0),
            ("cos 10 3600000", GuideOp::Cos, 5.0),
            ("max 3 7", GuideOp::Max, 7.0),
            ("min 3 7", GuideOp::Min, 3.0),
            ("mod 3 4 12", GuideOp::Mod, 13.0),
            ("pin 0 15 10", GuideOp::Pin, 10.0),
            ("sat2 10 3 4", GuideOp::Sat2, 8.0),
            ("sin 10 1800000", GuideOp::Sin, 5.0),
            ("sqrt -9", GuideOp::Sqrt, 3.0),
            ("tan 10 2700000", GuideOp::Tan, 10.0),
            ("val 42", GuideOp::Val, 42.0),
        ];
        let guides = cases
            .iter()
            .enumerate()
            .map(|(index, (formula, expected_op, _))| {
                let guide = Guide::parse(format!("g{index}"), formula).unwrap();
                assert_eq!(guide.op, *expected_op);
                guide
            })
            .collect::<Vec<_>>();
        let mut evaluator = GuideEvaluator::new(100.0, 80.0).unwrap();
        evaluator.evaluate_guides(&guides).unwrap();

        for (index, (_, _, expected)) in cases.iter().enumerate() {
            assert_close(evaluator.value(&format!("g{index}")).unwrap(), *expected);
        }
    }

    #[test]
    fn arc_to_is_flattened_to_finite_cubics_with_matching_endpoints() {
        let evaluator = GuideEvaluator::new(100.0, 100.0).unwrap();
        let path = evaluator
            .evaluate_path(&[
                PathCommand::MoveTo {
                    x: literal(10.0),
                    y: literal(0.0),
                },
                PathCommand::ArcTo {
                    width_radius: literal(10.0),
                    height_radius: literal(5.0),
                    start_angle: literal(0.0),
                    sweep_angle: literal(27_000_000.0),
                },
            ])
            .unwrap();
        assert_eq!(path.len(), 6);
        assert!(path.iter().skip(1).all(|command| matches!(
            command,
            EvaluatedPathCommand::CubicTo {
                x1,
                y1,
                x2,
                y2,
                x,
                y
            } if [x1, y1, x2, y2, x, y].iter().all(|value| value.is_finite())
        )));
        let EvaluatedPathCommand::CubicTo { x, y, .. } = path[5] else {
            panic!("arc did not end in a cubic command")
        };
        assert_close(x, 0.0);
        assert_close(y, 5.0);
        let EvaluatedPathCommand::CubicTo { x, y, .. } = path[1] else {
            panic!("arc did not start with a cubic command")
        };
        assert_close(x, 0.0);
        assert_close(y, 5.0);
    }

    #[test]
    fn office_mod_and_negative_sqrt_semantics_produce_finite_values() {
        let guides = [
            Guide::parse("norm", "mod 3 4 12").unwrap(),
            Guide::parse("root", "sqrt -9").unwrap(),
        ];
        let mut evaluator = GuideEvaluator::new(10.0, 10.0).unwrap();
        evaluator.evaluate_guides(&guides).unwrap();
        assert_eq!(evaluator.value("norm").unwrap(), 13.0);
        assert_eq!(evaluator.value("root").unwrap(), 3.0);
    }

    #[test]
    fn division_by_zero_returns_an_error_instead_of_non_finite_coordinates() {
        let mut evaluator = GuideEvaluator::new(10.0, 10.0).unwrap();
        let error = evaluator
            .evaluate_guides(&[Guide::parse("bad", "*/ 1 2 0").unwrap()])
            .unwrap_err();
        assert_eq!(error, GeometryError::DivisionByZero);
        assert_eq!(error.to_string(), "division by zero");
    }
}
