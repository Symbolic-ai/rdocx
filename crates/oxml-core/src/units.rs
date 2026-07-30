//! Unit types for OOXML measurements.

/// Twips — 1/20 of a point, or 1/1440 of an inch.
/// Used for page dimensions, margins, spacing, indentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Twips(pub i32);

impl Twips {
    pub fn from_inches(inches: f64) -> Self {
        Twips((inches * 1440.0) as i32)
    }

    pub fn from_cm(cm: f64) -> Self {
        Twips((cm * 567.0) as i32)
    }

    pub fn from_pt(pt: f64) -> Self {
        Twips((pt * 20.0) as i32)
    }

    pub fn to_inches(self) -> f64 {
        self.0 as f64 / 1440.0
    }

    pub fn to_cm(self) -> f64 {
        self.0 as f64 / 567.0
    }

    pub fn to_pt(self) -> f64 {
        self.0 as f64 / 20.0
    }

    pub fn to_emu(self) -> Emu {
        Emu(self.0 as i64 * 635)
    }
}

/// English Metric Units — 1/914400 of an inch.
/// Used for drawing coordinates and image sizing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Emu(pub i64);

impl Emu {
    pub fn from_inches(inches: f64) -> Self {
        Emu((inches * 914400.0) as i64)
    }

    pub fn from_cm(cm: f64) -> Self {
        Emu((cm * 360000.0) as i64)
    }

    pub fn from_pt(pt: f64) -> Self {
        Emu((pt * 12700.0) as i64)
    }

    pub fn to_inches(self) -> f64 {
        self.0 as f64 / 914400.0
    }

    pub fn to_cm(self) -> f64 {
        self.0 as f64 / 360000.0
    }

    pub fn to_pt(self) -> f64 {
        self.0 as f64 / 12700.0
    }

    pub fn to_twips(self) -> Twips {
        Twips((self.0 / 635) as i32)
    }
}

/// Half-points — 1/2 of a point, or 1/144 of an inch.
/// Used for font sizes (e.g., 24 half-points = 12pt).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct HalfPoint(pub u32);

impl HalfPoint {
    pub fn from_pt(pt: f64) -> Self {
        HalfPoint((pt * 2.0) as u32)
    }

    pub fn to_pt(self) -> f64 {
        self.0 as f64 / 2.0
    }
}

/// Centipoints, 1/100 of a point.
/// Used for DrawingML font sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Centipoints(pub i32);

impl Centipoints {
    pub fn from_pt(pt: f64) -> Self {
        Centipoints((pt * 100.0) as i32)
    }

    pub fn to_pt(self) -> f64 {
        self.0 as f64 / 100.0
    }
}

/// Angles stored in 60,000ths of a degree.
/// Used for DrawingML rotation and gradient angles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Angle(pub i32);

impl Angle {
    pub fn from_degrees(degrees: f64) -> Self {
        Angle((degrees * 60_000.0) as i32)
    }

    pub fn to_degrees(self) -> f64 {
        self.0 as f64 / 60_000.0
    }
}

/// Percentages stored in thousandths of a percent.
/// A stored value of 75,000 represents 75 percent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Percent1000(pub i32);

impl Percent1000 {
    pub fn from_percent(percent: f64) -> Self {
        Percent1000((percent * 1000.0) as i32)
    }

    pub fn to_fraction(self) -> f64 {
        self.0 as f64 / 100_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twips_conversions() {
        let t = Twips::from_inches(1.0);
        assert_eq!(t.0, 1440);
        assert!((t.to_inches() - 1.0).abs() < 0.001);
        assert!((t.to_pt() - 72.0).abs() < 0.001);
    }

    #[test]
    fn emu_conversions() {
        let e = Emu::from_inches(1.0);
        assert_eq!(e.0, 914400);
        assert!((e.to_inches() - 1.0).abs() < 0.001);
    }

    #[test]
    fn half_point_conversion() {
        let hp = HalfPoint::from_pt(12.0);
        assert_eq!(hp.0, 24);
        assert!((hp.to_pt() - 12.0).abs() < 0.001);
    }

    #[test]
    fn twips_to_emu_round_trip() {
        let t = Twips(1440);
        let e = t.to_emu();
        let t2 = e.to_twips();
        assert_eq!(t, t2);
    }

    #[test]
    fn twips_float_constructors_truncate_toward_zero() {
        assert_eq!(Twips::from_inches(1.75 / 1440.0).0, 1);
        assert_eq!(Twips::from_inches(-1.75 / 1440.0).0, -1);
        assert_eq!(Twips::from_cm(1.75 / 567.0).0, 1);
        assert_eq!(Twips::from_cm(-1.75 / 567.0).0, -1);
        assert_eq!(Twips::from_pt(1.75 / 20.0).0, 1);
        assert_eq!(Twips::from_pt(-1.75 / 20.0).0, -1);
    }

    #[test]
    fn emu_float_constructors_truncate_toward_zero() {
        assert_eq!(Emu::from_inches(1.75 / 914400.0).0, 1);
        assert_eq!(Emu::from_inches(-1.75 / 914400.0).0, -1);
        assert_eq!(Emu::from_cm(1.75 / 360000.0).0, 1);
        assert_eq!(Emu::from_cm(-1.75 / 360000.0).0, -1);
        assert_eq!(Emu::from_pt(1.75 / 12700.0).0, 1);
        assert_eq!(Emu::from_pt(-1.75 / 12700.0).0, -1);
    }

    #[test]
    fn centipoints_round_trip_points() {
        let value = Centipoints::from_pt(18.0);
        assert_eq!(value.0, 1800);
        assert_eq!(value.to_pt(), 18.0);
    }

    #[test]
    fn angle_round_trip_degrees() {
        let value = Angle::from_degrees(90.0);
        assert_eq!(value.0, 5_400_000);
        assert_eq!(value.to_degrees(), 90.0);
    }

    #[test]
    fn percent1000_round_trip_percent() {
        let value = Percent1000::from_percent(75.0);
        assert_eq!(value.0, 75_000);
        assert_eq!(value.to_fraction(), 0.75);
    }

    #[test]
    fn new_unit_float_constructors_truncate_toward_zero() {
        assert_eq!(Centipoints::from_pt(1.75 / 100.0).0, 1);
        assert_eq!(Centipoints::from_pt(-1.75 / 100.0).0, -1);
        assert_eq!(Angle::from_degrees(1.75 / 60_000.0).0, 1);
        assert_eq!(Angle::from_degrees(-1.75 / 60_000.0).0, -1);
        assert_eq!(Percent1000::from_percent(1.75 / 1000.0).0, 1);
        assert_eq!(Percent1000::from_percent(-1.75 / 1000.0).0, -1);
    }
}
