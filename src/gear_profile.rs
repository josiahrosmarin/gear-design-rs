use crate::{
    error::{GearDesignError, GearError},
    geometry::{fit_arc, CircularArc, Point},
};
use std::f64::consts::PI;

/// The minimum allowed pressure angle for gear profiles, in degrees.
///
/// While typical pressure angles are 14.5°, 20°, or 25°, this constant defines
/// the lower bound of the acceptable range for this library.
///
/// **Note:** Using pressure angles significantly below typical values may result in
/// impractical or non-functional gear designs.
pub const MINIMUM_PRESSURE_ANGLE: f64 = 5.0;

/// The maximum allowed pressure angle for gear profiles, in degrees.
///
/// While typical pressure angles are 14.5°, 20°, or 25°, this constant defines
/// the upper bound of the acceptable range for this library.
///
/// **Note:** Using pressure angles significantly above typical values may result in
/// impractical or non-functional gear designs.
pub const MAXIMUM_PRESSURE_ANGLE: f64 = 45.0;

/// Represents the fundamental involute profile of a gear.
///
/// This struct captures the essential geometric properties of a gear's involute tooth profile,
/// excluding manufacturing-specific details like root and tip radii, and tip relief.
pub struct GearProfile {
    /// The number of teeth on the gear.
    teeth: u32,

    /// The module of the gear.
    ///
    /// Module is a fundamental unit of gear tooth size, defined as the pitch diameter divided by the number of teeth.
    /// It is typically expressed in millimeters.
    module: f64,

    /// The base diameter of the involute curve.
    ///
    /// The base diameter is the diameter of the circle from which the involute curve is generated.
    /// It is a crucial parameter for involute gear calculations.
    base_diameter: f64,

    /// The circular thickness of the tooth at the pitch circle.
    ///
    /// Circular thickness is the width of the tooth measured along the pitch circle.
    circular_thickness: f64,

    /// The diameter at which the involute curve transitions to the root fillet.
    ///
    /// Below this diameter, the tooth profile is not a true involute.
    /// This defines the minimum diameter of the usable involute.
    form_diameter: f64,

    /// The diameter at which the pure involute profile ends, before any tip modifications.
    ///
    /// Above this diameter, the tooth profile may be modified (e.g., by tip relief).
    /// This defines the maximum diameter of the unmodified involute.
    involute_end_diameter: f64,
}

impl GearProfile {
    pub fn from_basic_params(
        teeth: u32,
        module: f64,
        pressure_angle: f64,
    ) -> Result<GearProfile, GearError> {
        // Validation checks
        if pressure_angle < MINIMUM_PRESSURE_ANGLE || pressure_angle > MAXIMUM_PRESSURE_ANGLE {
            return Err(GearError::InvalidPressureAngle);
        }

        let teeth_f64 = teeth as f64;
        let circular_thickness = module * PI / 2.0;
        let pitch_diameter = teeth_f64 * module;
        let base_diameter = pitch_diameter * pressure_angle.to_radians().cos();
        let form_diameter = base_diameter;
        let involute_end_diameter = pitch_diameter + (2.0 * module);

        if base_diameter > form_diameter {
            return Err(GearError::InvalidFormDiameter);
        }

        if involute_end_diameter < pitch_diameter {
            return Err(GearError::InvalidInvoluteEndDiameter);
        }

        Ok(GearProfile {
            teeth,
            module,
            base_diameter,
            circular_thickness,
            form_diameter,
            involute_end_diameter,
        })
    }

    pub fn set_circular_thickness(&mut self, circular_thickness: f64) -> Result<(), GearError> {
        if circular_thickness < 0.0 || self.circular_pitch() < circular_thickness {
            return Err(GearError::InvalidCircularThickness);
        }
        if check_involute_interference(self, circular_thickness, self.involute_end_diameter) {
            self.circular_thickness = circular_thickness;
            Ok(())
        } else {
            Err(GearError::InvalidCircularThickness)
        }
    }

    pub fn set_form_diameter(&mut self, form_diameter: f64) -> Result<(), GearError> {
        if form_diameter < self.base_diameter {
            return Err(GearError::InvalidFormDiameter);
        }
        self.form_diameter = form_diameter;
        Ok(())
    }

    pub fn set_involute_end_diameter(
        &mut self,
        involute_end_diameter: f64,
    ) -> Result<(), GearError> {
        if check_involute_interference(self, self.circular_thickness, involute_end_diameter) {
            self.involute_end_diameter = involute_end_diameter;
            Ok(())
        } else {
            Err(GearError::InvalidInvoluteEndDiameter)
        }
    }

    // Add getters for all fields
    pub fn teeth(&self) -> u32 {
        self.teeth
    }

    pub fn module(&self) -> f64 {
        self.module
    }

    pub fn base_diameter(&self) -> f64 {
        self.base_diameter
    }

    pub fn circular_thickness(&self) -> f64 {
        self.circular_thickness
    }

    pub fn form_diameter(&self) -> f64 {
        self.form_diameter
    }

    pub fn involute_end_diameter(&self) -> f64 {
        self.involute_end_diameter
    }

    pub fn pitch_diameter(&self) -> f64 {
        self.teeth as f64 * self.module
    }

    pub fn circular_pitch(&self) -> f64 {
        self.module() * PI
    }

    pub fn shifted_solidworks_equations(&self, tooth_shift: f64) {
        // let shift = tooth_shift * PI / self.teeth as f64;
        let shift = 0.0;
        let base_radius = self.base_diameter / 2.0;
        // let offset = -self.profile_offset_angle();
        let offset = 0.0;
        println!("{}t - {:.2}m", self.teeth, self.module);
        println!(
            "{:.6} * ( cos(t{:+.6}) + t*sin(t{:+.6}) )",
            base_radius,
            offset + shift,
            offset + shift
        );
        println!(
            "{:.6} * ( sin(t{:+.6}) - t*cos(t{:+.6}) )",
            base_radius,
            offset + shift,
            offset + shift
        );

        println!(
            "{:.6} * ( cos(t{:+.6}) + t*sin(t{:+.6}) )",
            base_radius,
            offset - shift,
            offset - shift
        );
        println!(
            "{:.6} * ( -sin(t{:+.6}) + t*cos(t{:+.6}) )",
            base_radius,
            offset - shift,
            offset - shift
        );
    }
}

/// Calculates a point on the involute curve.
///
/// # Parameters
///
/// * `base_diameter`: The base diameter of the involute curve.
/// * `roll_angle`: The roll angle (in radians).
/// * `offset_angle`: An offset angle (in radians).
///
/// # Returns
///
/// A `Point` representing a point on the involute curve.
fn involute(base_diameter: f64, roll_angle: f64, offset_angle: f64) -> Point {
    let base_radius = base_diameter / 2.0;
    let offset_roll_angle = roll_angle - offset_angle;
    let x = base_radius * (offset_roll_angle.cos() + roll_angle * offset_roll_angle.sin());
    let y = base_radius * (offset_roll_angle.sin() - roll_angle * offset_roll_angle.cos());
    Point { x, y }
}

/// Calculates the roll angle (in radians) at a given diameter on the involute curve.
///
/// # Parameters
///
/// * `base_diameter`: The base diameter of the involute curve.
/// * `evaluated_diameter`: The diameter at which to calculate the roll angle.
///
/// # Returns
///
/// A `Result<f64, GearError>` containing the roll angle or an error message.
fn roll_angle_at_diameter(base_diameter: f64, evaluated_diameter: f64) -> Result<f64, GearError> {
    if evaluated_diameter < base_diameter {
        return Err(GearError::InvalidInvoluteDiameter);
    }
    let diameter_ratio = evaluated_diameter / base_diameter;
    Ok((diameter_ratio * diameter_ratio - 1.0).sqrt())
}

/// Validates if a given combination of circular thickness and involute end diameter
/// is valid for a gear profile.
///
/// This function calculates the difference in roll angles between the pitch diameter
/// and the involute end diameter and compares it to half the tooth thickness angle.
///
/// # Parameters
///
/// * `profile`: A reference to the `GearProfile` to validate.
/// * `circular_thickness`: The circular thickness of the gear tooth.
/// * `involute_end_diameter`: The involute end diameter of the gear tooth.
///
/// # Returns
///
/// `true` if the combination is valid, `false` otherwise.
///
/// # Errors
///
/// Returns `false` if the `roll_angle_at_diameter` function returns an error,
/// indicating that the involute diameter is invalid.
fn check_involute_interference(
    profile: &GearProfile,
    circular_thickness: f64,
    involute_end_diameter: f64,
) -> bool {
    let base_diameter = profile.base_diameter();
    let pitch_diameter = profile.pitch_diameter();

    // Check if roll angle calculation is successful
    let roll_angle_pitch = match roll_angle_at_diameter(base_diameter, pitch_diameter) {
        Ok(angle) => angle,
        Err(_) => return false, // Return false if there's an error
    };
    let roll_angle_end = match roll_angle_at_diameter(base_diameter, involute_end_diameter) {
        Ok(angle) => angle,
        Err(_) => return false, // Return false if there's an error
    };

    let roll_angle_difference = (roll_angle_end - roll_angle_pitch).abs();
    let half_tooth_thickness_angle = circular_thickness / pitch_diameter / 2.0;

    if roll_angle_difference > half_tooth_thickness_angle {
        return false;
    } else {
        return true;
    }
}

/// Approximates the involute curve of a gear profile using circular arcs.
///
/// # Parameters
///
/// * `profile`: The gear profile to approximate.
/// * `maximum_deviation`: The maximum allowed deviation from the true involute curve.
///
/// # Returns
///
/// A vector of circular arcs representing the approximated involute curve.
pub fn approximate_involute(
    profile: &GearProfile,
    tolerance: f64,
) -> Result<Vec<CircularArc>, GearDesignError> {
    let base_diameter = profile.base_diameter();

    let form_diameter = profile.form_diameter();

    let involute_end_diameter = profile.involute_end_diameter();

    let double_annulus = involute_end_diameter - form_diameter;

    let involute_point = |test_diameter: f64| -> Result<Point, GearError> {
        let roll_angle = roll_angle_at_diameter(base_diameter, test_diameter)?;
        Ok(involute(base_diameter, roll_angle, 0.0))
    };

    // Divide the involute into evenly spaced points.
    // This isn't optimal but it's simple to implement, we can revisit later
    'segment_count_loop: for segment_count in 1..21 {
        let mut arcs = Vec::new();
        let vertex_count = segment_count * 2 + 1;
        let vertex_spacing = double_annulus / (vertex_count as f64 - 1.0);
        // Form and test each arc
        for segment in 0..segment_count {
            let test_diameter_0 = form_diameter + segment as f64 * vertex_spacing * 2.0;
            let vertex_0 = involute_point(test_diameter_0)?;

            let test_diameter_1 = test_diameter_0 + vertex_spacing;
            let vertex_1 = involute_point(test_diameter_1)?;

            let test_diameter_2 = test_diameter_1 + vertex_spacing;
            let vertex_2 = involute_point(test_diameter_2)?;

            let arc = fit_arc([&vertex_0, &vertex_2], &vertex_1)?;

            // This test is extremely inefficient
            let mut test_diameter = test_diameter_0 + tolerance;
            while test_diameter < test_diameter_2 {
                let test_point = involute_point(test_diameter)?;
                if !arc.contains_point(&test_point, tolerance) {
                    continue 'segment_count_loop;
                }
                test_diameter += tolerance;
            }
            arcs.push(arc);
        }
        return Ok(arcs);
    }
    return Err(GearDesignError::Gear(
        GearError::InvoluteApproximationFailed,
    ));
}
