use crate::error::GeometryError;
pub use circular_arc::{AngleSpan, CircularArc, CircularArcSvgParams};
pub use point::Point;
pub use vector::Vector;

mod circular_arc;
mod point;
mod vector;

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
    use crate::geometry::circular_arc::AngleSpan;
    use std::f64::consts::PI;

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
