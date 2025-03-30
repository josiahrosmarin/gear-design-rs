pub struct Vector {
    pub x: f64,
    pub y: f64,
}

impl Vector {
    pub fn from_angle_and_radius(angle: f64, radius: f64) -> Self {
        Vector {
            x: radius * angle.cos(),
            y: radius * angle.sin(),
        }
    }
    pub fn angle(&self) -> f64 {
        self.y.atan2(self.x)
    }
}
impl std::ops::Sub for &Vector {
    type Output = Vector;

    fn sub(self, other: Self) -> Self::Output {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
