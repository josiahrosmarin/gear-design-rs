use crate::{
    error::{GearMeshError, GearProfileError},
    gear_profile::GearProfile,
    geometry::Point,
    spur_gear::SpurGear, // Import the Gear struct
};

/// Represents a mesh between two gears.
///
/// This struct encapsulates the relationship between a pinion and a gear,
/// providing methods to validate and analyze the mesh.
pub struct GearMesh<'a> {
    /// A reference to the pinion gear (the smaller gear).
    pinion: &'a GearProfile,

    /// A reference to the gear (the larger gear).
    gear: &'a GearProfile,
}

impl<'a> GearMesh<'a> {
    /// Creates a new GearMesh.
    ///
    /// # Parameters
    ///
    /// * `pinion`: A reference to the GearProfile of the pinion gear.
    /// * `gear`: A reference to the GearProfile of the gear.
    ///
    /// # Returns
    ///
    /// A Result containing the GearMesh or a GearMeshError if the gears are incompatible.
    pub fn new(pinion: &'a GearProfile, gear: &'a GearProfile) -> Result<Self, GearMeshError> {
        // Validation logic here (e.g., check module compatibility)
        if pinion.module() != gear.module() {
            return Err(GearMeshError::IncompatibleModule);
        }

        // Check if the ratio of teeth matches the ratio of base diameters
        let teeth_ratio = pinion.teeth() as f64 / gear.teeth() as f64;
        let base_diameter_ratio = pinion.base_diameter() / gear.base_diameter();

        if (teeth_ratio - base_diameter_ratio).abs() > 1e-6 {
            // Use a small tolerance for comparison
            return Err(GearMeshError::IncompatibleBaseDiameterRatio);
        }

        Ok(GearMesh { pinion, gear })
    }

    /// Calculates the nominal center distance between the gears in the mesh.
    ///
    /// The nominal center distance is the distance between the centers of the gears
    /// when they are in their ideal meshing position. It is calculated from the pitch diameters
    /// of the pinion and the gear.
    ///
    /// # Returns
    ///
    /// The nominal center distance.
    pub fn nominal_center_distance(&self) -> f64 {
        (self.pinion.pitch_diameter() + self.gear.pitch_diameter()) / 2.0
    }

    /// Calculates the gear ratio between the gears in the mesh.
    ///
    /// The gear ratio is a dimensionless number that describes the relationship between the rotational speeds
    /// of the driving and driven gears.
    ///
    /// # Returns
    ///
    ///   The gear ratio.
    pub fn gear_ratio(&self) -> f64 {
        self.gear.teeth() as f64 / self.pinion.teeth() as f64
    }

    /// Calculates the operating pressure angle for a pair of gears at a given center distance.
    ///
    /// The operating pressure angle is the pressure angle at which the gears actually operate,
    /// which may be different from the standard pressure angle if the center distance is not equal
    /// to the nominal center distance.
    ///
    /// # Parameters
    ///
    /// * `center_distance`: The actual center distance between the gears.
    ///
    /// # Returns
    ///
    ///   The operating pressure angle in radians.
    pub fn operating_pressure_angle(&self, center_distance: f64) -> f64 {
        operating_pressure_angle(
            self.pinion.base_diameter(),
            self.gear.base_diameter(),
            center_distance,
        )
    }

    /// Calculates the contact ratio between the gears in the mesh.
    ///
    /// The contact ratio is a measure of the average number of tooth pairs in contact during gear rotation.
    /// A higher contact ratio generally indicates smoother and quieter gear operation.
    ///
    /// # Parameters
    ///
    /// * `center_distance`: The distance between the centers of the pinion and the gear.
    ///
    /// # Returns
    ///
    ///   A Result containing the contact ratio or a GearProfileError if the calculation fails.
    pub fn contact_ratio(&self, center_distance: f64) -> Result<f64, GearProfileError> {
        let pressure_angle = self.operating_pressure_angle(center_distance);

        let od = self.pinion.involute_end_diameter();
        let bd = self.pinion.base_diameter();
        let length_of_action_contribution_pinion = (od * od - bd * bd).sqrt() / 2.0;

        let od = self.gear.involute_end_diameter();
        let bd = self.gear.base_diameter();
        let length_of_action_contribution_gear = (od * od - bd * bd).sqrt() / 2.0;

        let length_of_action = length_of_action_contribution_pinion
            + length_of_action_contribution_gear
            - center_distance * pressure_angle.sin();

        Ok(length_of_action / self.pinion.base_pitch()) // Placeholder
    }

    /// Calculates the backlash between the gears in the mesh.
    ///
    /// Backlash is the play or clearance between mating teeth. It is necessary for lubrication and to prevent binding,
    /// but excessive backlash can reduce precision.
    ///
    /// # Parameters
    ///
    /// * `center_distance`: The actual center distance between the gears.
    ///
    /// # Returns
    ///
    ///   A Result containing the backlash value or a GearProfileError if the calculation fails.
    pub fn backlash(&self, center_distance: f64) -> Result<f64, GearProfileError> {
        // Implementation of backlash calculation...
        Ok(0.0) // Placeholder
    }

    /// Checks for interference between the gears in the mesh.
    ///
    /// Interference occurs when the tip of a tooth of one gear digs into the root of a tooth of the mating gear.
    /// This can cause noise, vibration, and premature wear.
    ///
    /// # Parameters
    ///
    /// * `center_distance`: The distance between the centers of the pinion and the gear.
    ///
    /// # Returns
    ///
    ///   A Result containing `true` if interference is detected, `false` otherwise,
    ///   or a GearProfileError if the check fails.
    pub fn check_interference(&self, center_distance: f64) -> Result<bool, GearProfileError> {
        // Implementation of interference check...
        Ok(false) // Placeholder
    }

    /// Calculates the output torque of the gear pair.
    ///
    /// This function determines the torque transmitted from the driving gear to the driven gear,
    /// considering the input torque and the gear ratio.
    ///
    /// # Parameters
    ///
    /// * `input_torque`: The torque applied to the driving gear.
    ///
    /// # Returns
    ///
    ///   A Result containing the output torque or a GearProfileError if the calculation fails.
    pub fn torque(&self, input_torque: f64) -> Result<f64, GearProfileError> {
        // Implementation of torque calculation...
        Ok(input_torque) // Placeholder
    }
}

fn operating_pressure_angle(
    base_diameter_pinion: f64,
    base_diameter_gear: f64,
    center_distance: f64,
) -> f64 {
    // Calculate the operating pressure angle using the relationship between
    // base diameters, center distance, and pressure angle.
    let cos_operating_pressure_angle =
        (base_diameter_pinion + base_diameter_gear) / (2.0 * center_distance);
    cos_operating_pressure_angle.acos()
}

pub fn calculate_interference_limit_diameter(
    gear: &GearProfile,
    mating_gear: &GearProfile,
    center_distance: f64,
) -> f64 {
    // Get the operating pressure angle at the given center distance
    let pressure_angle = operating_pressure_angle(
        gear.base_diameter(),
        mating_gear.base_diameter(),
        center_distance,
    );

    println!("pressure_angle: {:?}", pressure_angle);

    let mating_base_radius = mating_gear.base_diameter() / 2.0;

    println!("mating_base_radius: {:?}", mating_base_radius);

    let mut line_of_action_limit = Point {
        x: center_distance - mating_base_radius * pressure_angle.cos(),
        y: mating_base_radius * pressure_angle.sin(),
    };

    println!("line_of_action_limit: {:?}", line_of_action_limit);

    if mating_gear.base_diameter() != mating_gear.form_diameter() {
        let fd = mating_gear.form_diameter();
        let bd = mating_gear.base_diameter();
        let correction_length = (fd * fd - bd * bd).sqrt() / 2.0;
        println!("correction_length: {:?}", correction_length);
        line_of_action_limit.x -= correction_length * pressure_angle.sin();
        line_of_action_limit.y -= correction_length * pressure_angle.cos();
    }
    println!("line_of_action_limit: {:?}", line_of_action_limit);

    line_of_action_limit.radius() * 2.0
}
