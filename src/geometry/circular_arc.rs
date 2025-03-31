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

        Some(CircularArcSvgParams {
            start: start_point,
            large_arc_flag,
            sweep_flag: true,
            end: end_point,
            radius: self.radius,
        })
    }

    /// Returns a new `CircularArc` that is mirrored vertically across the x-axis.
    pub fn mirror_vertical(&self) -> Self {
        CircularArc {
            center: self.center.mirror_vertical(),
            radius: self.radius,
            angle_span: match self.angle_span {
                AngleSpan::Arc { start, end } => AngleSpan::Arc {
                    start: (-end).rem_euclid(2.0 * PI),
                    end: (-start).rem_euclid(2.0 * PI),
                },
                AngleSpan::FullCircle => AngleSpan::FullCircle,
            },
        }
    }
    /// Returns a new `CircularArc` that is mirrored vertically across the x-axis.
    pub fn mirror_horizontal(&self) -> Self {
        CircularArc {
            center: self.center.mirror_horizontal(),
            radius: self.radius,
            angle_span: match self.angle_span {
                AngleSpan::Arc { start, end } => AngleSpan::Arc {
                    start: (PI - end).rem_euclid(2.0 * PI),
                    end: (PI - start).rem_euclid(2.0 * PI),
                },
                AngleSpan::FullCircle => AngleSpan::FullCircle,
            },
        }
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

/// Parameters for generating an SVG elliptical arc command for a circular arc.
///
/// This struct simplifies the creation of SVG arc commands ("A") specifically for
/// circular arcs. It assumes that the x and y radii of the ellipse are equal
/// (hence, it's a circle), and the x-axis rotation is always 0.
pub struct CircularArcSvgParams {
    /// The radius of the circular arc.
    pub radius: f64,
    /// The starting point of the arc.
    pub start: Point,
    /// The ending point of the arc.
    pub end: Point,
    /// Determines if the arc should be greater than 180 degrees.
    ///
    /// In our simplified model, this flag is determined by whether the angular
    /// difference between the start and end points (assuming counterclockwise
    /// sweep) is greater than π radians.
    pub large_arc_flag: bool,
    /// Determines the direction of the sweep:
    /// - `true`: Counterclockwise (positive angle).
    /// - `false`: Clockwise (negative angle).
    pub sweep_flag: bool,
}

impl CircularArcSvgParams {
    /// Generates an SVG path segment string for a circular arc using the "A" command.
    ///
    /// The output string starts with an "M" (moveto) command to the `start` point,
    /// followed by the "A" (elliptical arc) command with the pre-calculated
    /// parameters.
    ///
    /// Assumptions:
    /// - This assumes you have already "moved to" the starting point of the arc
    ///   in your overall SVG path if you are chaining multiple segments. However,
    ///   this method includes the "M" command for convenience in creating single
    ///   arc segments.
    /// - The arc is circular, so `rx` and `ry` are equal to the `radius`.
    /// - The x-axis rotation is always 0.
    /// - The `large_arc_flag` and `sweep_flag` have been correctly determined
    ///   based on the desired arc. Our convention is a counterclockwise sweep.
    ///
    /// Simplifications:
    /// - Does not handle elliptical arcs with different x and y radii.
    /// - Does not allow for x-axis rotation of the ellipse.
    pub fn to_svg_path_segment(&self) -> String {
        format!(
            "M {} {} A {} {} 0 {} {} {} {}",
            self.start.x,
            self.start.y,
            self.radius,
            self.radius,
            self.large_arc_flag as u8,
            self.sweep_flag as u8,
            self.end.x,
            self.end.y
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use std::f64::consts::PI;

    #[test]
    fn test_point_mirror_vertical() {
        let p1 = Point { x: 2.0, y: 3.0 };
        let mirrored_p1 = p1.mirror_vertical();
        assert_approx_eq!(mirrored_p1.x, 2.0);
        assert_approx_eq!(mirrored_p1.y, -3.0);
    }

    #[test]
    fn test_circular_arc_mirror_vertical() {
        let center = Point { x: 1.0, y: 2.0 };
        let radius = 5.0;
        let angle_span = AngleSpan::Arc {
            start: PI / 4.0,
            end: PI,
        };
        let arc = CircularArc {
            center,
            radius,
            angle_span,
        };

        let mirrored_arc = arc.mirror_vertical();

        assert_approx_eq!(mirrored_arc.center.x, 1.0);
        assert_approx_eq!(mirrored_arc.center.y, -2.0);
        assert_approx_eq!(mirrored_arc.radius, 5.0);
        if let AngleSpan::Arc { start, end } = mirrored_arc.angle_span {
            assert_approx_eq!(start, -PI, 1e-6);
            assert_approx_eq!(end, -PI / 4.0, 1e-6);
        } else {
            panic!("Expected AngleSpan::Arc");
        }

        let full_circle_arc = CircularArc {
            center: Point { x: -3.0, y: 0.5 },
            radius: 2.0,
            angle_span: AngleSpan::FullCircle,
        };
        let mirrored_full_circle = full_circle_arc.mirror_vertical();
        assert_approx_eq!(mirrored_full_circle.center.x, -3.0);
        assert_approx_eq!(mirrored_full_circle.center.y, -0.5);
        assert_approx_eq!(mirrored_full_circle.radius, 2.0);
        assert_eq!(mirrored_full_circle.angle_span, AngleSpan::FullCircle);

        let center2 = Point { x: 0.0, y: 0.0 };
        let radius2 = 1.0;
        let angle_span2 = AngleSpan::Arc {
            start: 3.0 * PI / 2.0,
            end: PI / 2.0,
        };
        let arc2 = CircularArc {
            center: center2,
            radius: radius2,
            angle_span: angle_span2,
        };
        let mirrored_arc2 = arc2.mirror_vertical();
        if let AngleSpan::Arc { start, end } = mirrored_arc2.angle_span {
            assert_approx_eq!(start, -PI / 2.0, 1e-6);
            assert_approx_eq!(end, -3.0 * PI / 2.0, 1e-6);
        } else {
            panic!("Expected AngleSpan::Arc for arc2");
        }
    }

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
