use super::vector::Vector;
use std::fmt;

#[derive(Debug, PartialEq, Clone, Copy)]
/// Represents a point in 2D space.
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

    pub fn rotated(&self, angle: f64, center: Option<&Point>) -> Self {
        let center = center.unwrap_or(&Point { x: 0.0, y: 0.0 });
        let translated_x = self.x - center.x;
        let translated_y = self.y - center.y;

        let rotated_x = translated_x * angle.cos() - translated_y * angle.sin();
        let rotated_y = translated_x * angle.sin() + translated_y * angle.cos();

        Point {
            x: rotated_x + center.x,
            y: rotated_y + center.y,
        }
    }

    pub fn mirror_horizontal(&self) -> Self {
        Self {
            x: -self.x,
            y: self.y,
        }
    }

    pub fn mirror_vertical(&self) -> Self {
        Self {
            x: self.x,
            y: -self.y,
        }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.6}, {:.6})", self.x, self.y)
    }
}

impl std::ops::Sub for &Point {
    type Output = Vector;

    fn sub(self, other: Self) -> Self::Output {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Add<&Vector> for &Point {
    type Output = Point;

    fn add(self, vector: &Vector) -> Self::Output {
        Point {
            x: self.x + vector.x,
            y: self.y + vector.y,
        }
    }
}

impl std::ops::Sub<&Vector> for &Point {
    type Output = Point;

    fn sub(self, vector: &Vector) -> Self::Output {
        Point {
            x: self.x - vector.x,
            y: self.y - vector.y,
        }
    }
}
