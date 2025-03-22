use std::{f64::consts::PI, fmt};

use crate::error::GeometryError;

/// Represents a point in 2D space.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn radius(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn angle(&self) -> f64 {
        self.y.atan2(self.x)
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
}

/// Calculates the center and radius of a circle that passes through three points.
///
/// # Parameters
///
/// * `p1`: The first point.
/// * `p2`: The second point.
/// * `p3`: The third point.
///
/// # Returns
///
/// A `CircularArc` representing the circle that passes through the three points.
pub fn fit_arc(
    boundary_points: [&Point; 2],
    interior_point: &Point,
) -> Result<CircularArc, GeometryError> {
    let b0_x = boundary_points[0].x;
    let b0_y = boundary_points[0].y;
    let i_x = interior_point.x;
    let i_y = interior_point.y;
    let b1_x = boundary_points[1].x;
    let b1_y = boundary_points[1].y;

    let cross_product = b0_x * (i_y - b1_y) - b0_y * (i_x - b1_x) + i_x * b1_y - b1_x * i_y;

    if cross_product == 0.0 {
        return Err(GeometryError::CollinearPoints);
    }

    let center_x_determinant = (b0_x * b0_x + b0_y * b0_y) * (b1_y - i_y)
        + (i_x * i_x + i_y * i_y) * (b0_y - b1_y)
        + (b1_x * b1_x + b1_y * b1_y) * (i_y - b0_y);
    let center_y_determinant = (b0_x * b0_x + b0_y * b0_y) * (i_x - b1_x)
        + (i_x * i_x + i_y * i_y) * (b1_x - b0_x)
        + (b1_x * b1_x + b1_y * b1_y) * (b0_x - i_x);

    let center = Point {
        x: -center_x_determinant / (2.0 * cross_product),
        y: -center_y_determinant / (2.0 * cross_product),
    };

    let radius =
        ((b0_x - center.x) * (b0_x - center.x) + (b0_y - center.y) * (b0_y - center.y)).sqrt();

    let b0_theta = (b0_y - center.y).atan2(b0_x - center.x);
    let b1_theta = (b1_y - center.y).atan2(b1_x - center.x);

    let arc = CircularArc::new(center, radius, b0_theta, b1_theta);
    if arc.contains_point(interior_point, 1e-6) {
        return Ok(arc);
    }

    let arc = CircularArc::new(center, radius, b1_theta, b0_theta);
    if arc.contains_point(interior_point, 1e-6) {
        return Ok(arc);
    }

    Err(GeometryError::InteriorPointNotOnArc)
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

    #[test]
    fn test_fit_arc_non_collinear() {
        let p1 = Point { x: 1.0, y: 0.0 };
        let p2 = Point { x: -1.0, y: 0.0 };
        let p3 = Point { x: 0.0, y: 1.0 };

        let arc = fit_arc([&p1, &p2], &p3).unwrap();

        assert_eq!(arc.center, Point { x: 0.0, y: 0.0 });
        assert_eq!(arc.radius, 1.0);

        if let AngleSpan::Arc { start, end } = arc.angle_span {
            assert!((start - 0.0).abs() < 1e-6);
            assert!((end - PI).abs() < 1e-6);
        } else {
            panic!("Expected AngleSpan::Arc");
        }
    }
    #[test]
    fn test_fit_arc_non_collinear_2() {
        let p1 = Point { x: 2.0, y: 0.0 };
        let p2 = Point { x: -2.0, y: 0.0 };
        let p3 = Point { x: 0.0, y: 2.0 };

        let arc = fit_arc([&p1, &p2], &p3).unwrap();

        assert_eq!(arc.center, Point { x: 0.0, y: 0.0 });
        assert_eq!(arc.radius, 2.0);

        if let AngleSpan::Arc { start, end } = arc.angle_span {
            assert!((start - 0.0).abs() < 1e-6);
            assert!((end - PI).abs() < 1e-6);
        } else {
            panic!("Expected AngleSpan::Arc");
        }
    }

    #[test]
    fn test_fit_arc_collinear_horizontal() {
        let p1 = Point { x: 1.0, y: 1.0 };
        let p2 = Point { x: 2.0, y: 1.0 };
        let p3 = Point { x: 3.0, y: 1.0 };

        let arc = fit_arc([&p1, &p2], &p3);

        assert_eq!(arc, Err(GeometryError::CollinearPoints));
    }

    #[test]
    fn test_fit_arc_tiny_arc() {
        let p1 = Point { x: 1.0, y: 1.0 };
        let p2 = Point {
            x: 1.000001,
            y: 1.000002,
        };
        let p3 = Point {
            x: 1.000002,
            y: 1.000001,
        };

        let arc = fit_arc([&p1, &p2], &p3).unwrap();

        assert!(arc.radius > 1e6); // Expect a large radius for a tiny arc
        if let AngleSpan::Arc { start, end } = arc.angle_span {
            assert!((end - start).abs() < 1e-5); // Expect a small angle difference
        } else {
            panic!("Expected AngleSpan::Arc");
        }
    }

    #[test]
    fn test_fit_arc_large_arc() {
        let p1 = Point { x: 1000.0, y: 0.0 };
        let p2 = Point { x: 0.0, y: 1000.0 };
        let p3 = Point { x: -1000.0, y: 0.0 };

        let arc = fit_arc([&p1, &p2], &p3).unwrap();

        assert_eq!(arc.radius, 1000.0);
        if let AngleSpan::Arc { start, end } = arc.angle_span {
            assert!((start - 0.0).abs() < 1e-6);
            assert!((end - PI).abs() < 1e-6);
        } else {
            panic!("Expected AngleSpan::Arc");
        }
    }

    #[test]
    fn test_fit_arc_unusual_angles() {
        let p1 = Point { x: 1.0, y: 1.0 };
        let p2 = Point { x: 2.0, y: 3.0 };
        let p3 = Point { x: 3.0, y: 1.5 };

        let arc = fit_arc([&p1, &p2], &p3).unwrap();

        if let AngleSpan::Arc { start, end } = arc.angle_span {
            assert!((start - 1.0_f64.atan2(1.0)).abs() < 1e-6);
            assert!((end - 1.5_f64.atan2(3.0)).abs() < 1e-6);
        } else {
            panic!("Expected AngleSpan::Arc");
        }
    }

    #[test]
    fn test_fit_arc_near_collinear() {
        let p1 = Point { x: 1.0, y: 1.0 };
        let p2 = Point {
            x: 2.0,
            y: 2.000001,
        };
        let p3 = Point { x: 3.0, y: 3.0 };

        let arc = fit_arc([&p1, &p2], &p3).unwrap();

        assert!(arc.radius > 1e6);
        if let AngleSpan::Arc { start, end } = arc.angle_span {
            assert!((end - start).abs() < 1e-5);
        } else {
            panic!("Expected AngleSpan::Arc");
        }
    }

    #[test]
    fn test_fit_arc_negative_coordinates() {
        let p1 = Point { x: -1.0, y: -1.0 };
        let p2 = Point { x: -2.0, y: -3.0 };
        let p3 = Point { x: -3.0, y: -1.5 };

        let arc = fit_arc([&p1, &p2], &p3).unwrap();

        if let AngleSpan::Arc { start, end } = arc.angle_span {
            assert!((start - (-3.0 * PI / 4.0)).abs() < 1e-6);
            assert!((end - (-PI + 1.5_f64.atan2(3.0))).abs() < 1e-6);
        } else {
            panic!("Expected AngleSpan::Arc");
        }
    }
}
