use std::{f64::consts::PI, fmt};

use super::{Point, Vector};

/// Represents a circular arc.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CircularArc {
    pub center: Point,
    pub radius: f64,
    pub angle_span: AngleSpan,
}
impl fmt::Display for CircularArc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CircularArc(center: ({:.3}, {:.3}), radius: {:.3}, angle_span: {})",
            self.center.x, self.center.y, self.radius, self.angle_span
        )
    }
}

impl CircularArc {
    pub fn new(center: Point, radius: f64, start_angle: f64, end_angle: f64) -> Self {
        CircularArc {
            center,
            radius,
            angle_span: AngleSpan::Arc {
                start: start_angle,
                end: end_angle,
            },
        }
    }

    pub fn contains_point(&self, point: &Point, tolerance: f64) -> bool {
        let distance =
            ((point.x - self.center.x).powi(2) + (point.y - self.center.y).powi(2)).sqrt();
        if (distance - self.radius).abs() > tolerance {
            return false;
        }
        let angle = (point.y - self.center.y).atan2(point.x - self.center.x);
        self.contains_angle(angle)
    }

    pub fn contains_angle(&self, angle: f64) -> bool {
        match self.angle_span {
            AngleSpan::Arc { start, end } => {
                let shifted_end_angle = (end - start).rem_euclid(2.0 * PI);
                let shifted_input_angle = (angle - start).rem_euclid(2.0 * PI);
                shifted_input_angle <= shifted_end_angle
            }
            AngleSpan::FullCircle => true,
        }
    }

    pub fn end_points(&self) -> Option<[Point; 2]> {
        let (start, end) = match self.angle_span {
            AngleSpan::FullCircle => return None,
            AngleSpan::Arc { start, end } => (start, end),
        };

        return Some([
            &self.center + &Vector::from_angle_and_radius(start, self.radius),
            &self.center + &Vector::from_angle_and_radius(end, self.radius),
        ]);
    }

    pub fn svg_arc_params(&self) -> Option<CircularArcSvgParams> {
        let (start_angle, end_angle) = match self.angle_span {
            AngleSpan::FullCircle => return None,
            AngleSpan::Arc { start, end } => (start, end),
        };

        let start_point = &self.center + &Vector::from_angle_and_radius(start_angle, self.radius);
        let end_point = &self.center + &Vector::from_angle_and_radius(end_angle, self.radius);

        let delta = end_angle - start_angle;
        let large_arc_flag = delta.abs() > std::f64::consts::PI;
        let sweep_flag = end_angle > start_angle;

        Some(CircularArcSvgParams {
            start: start_point,
            large_arc_flag,
            sweep_flag,
            end: end_point,
            radius: self.radius,
        })
    }
}

/// Represents the angle span of a circular arc.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AngleSpan {
    Arc { start: f64, end: f64 },
    FullCircle,
}

impl AngleSpan {
    fn as_arc(&self) -> Option<(&f64, &f64)> {
        match self {
            AngleSpan::Arc { start, end } => Some((start, end)),
            AngleSpan::FullCircle => None,
        }
    }
}
impl fmt::Display for AngleSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AngleSpan::Arc { start, end } => {
                let start_deg = start * 180.0 / PI;
                let end_deg = end * 180.0 / PI;
                write!(f, "Arc({:.3}° to {:.3}°) ", start_deg, end_deg)
            }
            AngleSpan::FullCircle => write!(f, "FullCircle"),
        }
    }
}

pub struct CircularArcSvgParams {
    pub start: Point,
    pub large_arc_flag: bool,
    pub sweep_flag: bool,
    pub end: Point,
    pub radius: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_angle_within_arc() {
        let arc = CircularArc {
            center: Point { x: 0.0, y: 0.0 },
            radius: 1.0,
            angle_span: AngleSpan::Arc {
                start: 0.0_f64.to_radians(),
                end: 90.0_f64.to_radians(),
            },
        };
        assert!(arc.contains_angle(0.0_f64.to_radians()));
        assert!(arc.contains_angle(45.0_f64.to_radians()));
        assert!(arc.contains_angle(90.0_f64.to_radians()));
    }

    #[test]
    fn test_contains_angle_outside_arc() {
        let arc = CircularArc {
            center: Point { x: 0.0, y: 0.0 },
            radius: 1.0,
            angle_span: AngleSpan::Arc {
                start: 0.0_f64.to_radians(),
                end: 90.0_f64.to_radians(),
            },
        };
        assert!(!arc.contains_angle((-1.0_f64).to_radians()));
        assert!(!arc.contains_angle(91.0_f64.to_radians()));
        assert!(!arc.contains_angle(180.0_f64.to_radians()));
    }

    #[test]
    fn test_contains_angle_wrap_around() {
        let arc = CircularArc {
            center: Point { x: 0.0, y: 0.0 },
            radius: 1.0,
            angle_span: AngleSpan::Arc {
                start: 315.0_f64.to_radians(),
                end: 45.0_f64.to_radians(),
            },
        };
        assert!(arc.contains_angle(0.0_f64.to_radians()));
        assert!(arc.contains_angle(45.0_f64.to_radians()));
        assert!(arc.contains_angle(315.0_f64.to_radians()));
        assert!(arc.contains_angle(360.0_f64.to_radians()));
        assert!(!arc.contains_angle(90.0_f64.to_radians()));
        assert!(!arc.contains_angle(270.0_f64.to_radians()));
    }

    #[test]
    fn test_contains_angle_full_circle() {
        let arc = CircularArc {
            center: Point { x: 0.0, y: 0.0 },
            radius: 1.0,
            angle_span: AngleSpan::FullCircle,
        };
        assert!(arc.contains_angle(0.0_f64.to_radians()));
        assert!(arc.contains_angle(90.0_f64.to_radians()));
        assert!(arc.contains_angle(180.0_f64.to_radians()));
        assert!(arc.contains_angle(270.0_f64.to_radians()));
        assert!(arc.contains_angle(359.9_f64.to_radians()));
    }

    #[test]
    fn test_contains_angle_negative_angles() {
        let arc = CircularArc {
            center: Point { x: 0.0, y: 0.0 },
            radius: 1.0,
            angle_span: AngleSpan::Arc {
                start: (-45.0_f64).to_radians(),
                end: 45.0_f64.to_radians(),
            },
        };
        assert!(arc.contains_angle((-45.0_f64).to_radians()));
        assert!(arc.contains_angle(0.0_f64.to_radians()));
        assert!(arc.contains_angle(45.0_f64.to_radians()));
        assert!(!arc.contains_angle((-90.0_f64).to_radians()));
        assert!(!arc.contains_angle(90.0_f64.to_radians()));
    }

    #[test]
    fn test_contains_point_within_arc() {
        let start = 0.0_f64.to_radians();
        let end = 90.0_f64.to_radians();
        let arc = CircularArc {
            center: Point { x: 0.0, y: 0.0 },
            radius: 1.0,
            angle_span: AngleSpan::Arc { start, end },
        };
        assert!(arc.contains_point(&Point { x: 1.0, y: 0.0 }, 1e-6));
        assert!(arc.contains_point(&Point { x: 0.0, y: 1.0 }, 1e-6));
        assert!(arc.contains_point(
            &Point {
                x: 1.0 / 2.0_f64.sqrt(),
                y: 1.0 / 2.0_f64.sqrt()
            },
            1e-6
        ));
        assert!(arc.contains_point(
            &Point {
                x: 0.5,
                y: 0.86602540378
            },
            1e-6
        )); // 60 degrees
        assert!(arc.contains_point(
            &Point {
                x: 0.86602540378,
                y: 0.5
            },
            1e-6
        )); // 30 degrees
        assert!(arc.contains_point(
            &Point {
                x: 0.70710678118,
                y: 0.70710678118
            },
            1e-6
        )); // 45 degrees
        assert!(arc.contains_point(
            &Point {
                x: 0.2588190451,
                y: 0.96592582628
            },
            1e-6
        )); // 75 degrees
        assert!(arc.contains_point(
            &Point {
                x: 0.96592582628,
                y: 0.2588190451
            },
            1e-6
        )); // 15 degrees
        assert!(arc.contains_point(&Point { x: 0.0, y: 0.0 }, 1.0)); // center of arc
    }

    #[test]
    fn test_contains_point_outside_arc() {
        let start = 0.0_f64.to_radians();
        let end = 90.0_f64.to_radians();
        let arc = CircularArc {
            center: Point { x: 0.0, y: 0.0 },
            radius: 1.0,
            angle_span: AngleSpan::Arc { start, end },
        };

        assert!(!arc.contains_point(&Point { x: 0.0, y: -1.0 }, 1e-6));
        assert!(!arc.contains_point(&Point { x: -1.0, y: 0.0 }, 1e-6));
        assert!(!arc.contains_point(
            &Point {
                x: 0.5,
                y: -0.86602540378
            },
            1e-6
        ));
        assert!(!arc.contains_point(
            &Point {
                x: -0.86602540378,
                y: 0.5
            },
            1e-6
        ));
        assert!(!arc.contains_point(
            &Point {
                x: 0.70710678118,
                y: -0.70710678118
            },
            1e-6
        ));
        assert!(!arc.contains_point(
            &Point {
                x: -0.70710678118,
                y: 0.70710678118
            },
            1e-6
        ));
        assert!(!arc.contains_point(
            &Point {
                x: 0.2588190451,
                y: -0.96592582628
            },
            1e-6
        ));
        assert!(!arc.contains_point(
            &Point {
                x: -0.96592582628,
                y: 0.2588190451
            },
            1e-6
        ));
    }

    #[test]
    fn test_contains_point_outside_radius() {
        let start = 0.0_f64.to_radians();
        let end = 90.0_f64.to_radians();
        let arc = CircularArc {
            center: Point { x: 0.0, y: 0.0 },
            radius: 1.0,
            angle_span: AngleSpan::Arc { start, end },
        };
        assert!(!arc.contains_point(&Point { x: 1.1, y: 0.0 }, 1e-6));
        assert!(!arc.contains_point(&Point { x: 0.9, y: 0.0 }, 1e-6));
        assert!(!arc.contains_point(&Point { x: 1.1, y: 1.1 }, 1e-6));
        assert!(!arc.contains_point(&Point { x: 0.9, y: 0.9 }, 1e-6));
        assert!(!arc.contains_point(&Point { x: 1.2, y: 0.5 }, 1e-6));
        assert!(!arc.contains_point(&Point { x: 0.8, y: 0.5 }, 1e-6));
    }

    #[test]
    fn test_contains_point_wrap_around() {
        let start = 315.0_f64.to_radians();
        let end = 45.0_f64.to_radians();
        let arc = CircularArc {
            center: Point { x: 0.0, y: 0.0 },
            radius: 1.0,
            angle_span: AngleSpan::Arc { start, end },
        };

        assert!(arc.contains_point(&Point { x: 1.0, y: 0.0 }, 1e-6));
        assert!(!arc.contains_point(&Point { x: 0.0, y: 1.0 }, 1e-6));
        assert!(!arc.contains_point(&Point { x: 0.0, y: -1.0 }, 1e-6));
        assert!(!arc.contains_point(&Point { x: -1.0, y: 0.0 }, 1e-6));
        assert!(arc.contains_point(
            &Point {
                x: 0.70710678118,
                y: 0.70710678118
            },
            1e-6
        ));
        assert!(arc.contains_point(
            &Point {
                x: 0.70710678118,
                y: -0.70710678118
            },
            1e-6
        ));
        assert!(!arc.contains_point(
            &Point {
                x: -0.70710678118,
                y: 0.70710678118
            },
            1e-6
        ));
        assert!(!arc.contains_point(
            &Point {
                x: -0.70710678118,
                y: -0.70710678118
            },
            1e-6
        ));
        assert!(arc.contains_point(&Point { x: 0.0, y: 0.0 }, 1.0)); //center
    }
}
