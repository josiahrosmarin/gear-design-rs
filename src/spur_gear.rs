use std::f64::consts::PI;

use crate::{
    error::{GearDesignError, SpurGearProfileError},
    gear_profile::{approximate_involute, GearProfile},
    geometry::{AngleSpan, CircularArc, CircularArcSvgParams},
};

#[cfg(feature = "tip-relief")]
/// Represents the parameters for tip relief.
///
/// Tip relief is a modification to the tooth tip designed to reduce noise and improve engagement.
pub struct TipRelief {
    /// The start diameter of the tip relief.
    ///
    /// This is the diameter at which the tip relief begins, transitioning from the unmodified involute profile.
    pub start_diameter: f64,

    /// The amount of tip relief.
    ///
    /// This is the amount of material removed from the tooth tip to achieve the relief.
    pub amount: f64,

    /// The length of the tip relief along the tooth flank.
    ///
    /// This defines the extent of the tip relief along the tooth profile.
    pub length: f64,
}

/// Represents the various forms of the gear tooth tip.
pub enum Tip {
    /// A sharp tip.
    ///
    /// Represents a gear with a perfectly sharp tip, without any radius or relief.
    Sharp,

    /// A tip with a radius.
    ///
    /// Represents a gear with a rounded tip, where the radius is specified.
    Radius(CircularArc),

    #[cfg(feature = "tip-relief")]
    /// A tip with relief.
    ///
    /// Represents a gear with tip relief, where the relief parameters are defined in the `TipRelief` struct.
    Relief(TipRelief),
}

/// Represents the parameters for a trochoidal root radius.
///
/// A trochoidal root is formed by the path of a point on a generating gear or cutter as it rolls along the gear blank.
pub struct TrochoidParams {
    /// The radius of the generating gear or cutter.
    ///
    /// This is the radius of the tool used to generate the trochoidal root.
    pub cutter_radius: f64,

    /// The addendum of the generating gear or cutter.
    ///
    /// This is the height of the tooth on the generating gear or cutter.
    pub cutter_addendum: f64,

    /// The number of teeth of the generating gear, if applicable.
    ///
    /// This is the number of teeth on the generating gear. A hob does not have teeth, so this is an `Option`.
    pub cutter_teeth: Option<u32>,
}

/// Represents the various forms of root radius.
pub enum RootFillet {
    /// A full radius at the root.
    ///
    /// Represents a root with a complete circular arc.
    Full(CircularArc),

    /// A partial radius that terminates into the bottom land.
    ///
    /// Represents a root with a circular arc that blends into the bottom land.
    Partial(CircularArc),

    #[cfg(feature = "trochoid")]
    /// A trochoidal root radius.
    ///
    /// Represents a root formed by a trochoidal curve, with parameters defined in the `TrochoidParams` struct.
    Trochoid(TrochoidParams),
}

/// Represents a complete gear with its involute profile and manufacturing parameters.
pub struct SpurGear {
    /// The involute profile of the gear teeth.
    ///
    /// This field encapsulates the essential involute profile data, defined in the `GearProfile` struct.
    pub profile: GearProfile,

    /// The root radius of the gear teeth.
    ///
    /// This field defines the shape of the root of the tooth, using the `RootFillet` enum.
    root_radius: RootFillet,

    /// The tip of the gear teeth.
    ///
    /// This field defines the shape of the tip of the tooth, using the `Tip` enum.
    tip: Tip,

    /// The outer diameter of the gear.
    ///
    /// This is the overall diameter of the gear.
    outer_diameter: f64,

    /// The face width of the gear.
    ///
    /// This is the width of the gear teeth along the axis of rotation.
    pub face_width: f64,
    // Add other parameters as needed (e.g., bore diameter, material, etc.)
}

impl SpurGear {
    pub fn new(
        teeth: u32,
        module: f64,
        pressure_angle: f64,
        face_width: f64,
    ) -> Result<Self, GearDesignError> {
        let profile = GearProfile::from_basic_params(teeth, module, pressure_angle)?;
        let arc = profile.full_root_fillet()?;
        let outer_diameter = profile.involute_end_diameter();
        Ok(Self {
            profile,
            root_radius: RootFillet::Full(arc),
            tip: Tip::Sharp,
            outer_diameter,
            face_width,
        })
    }

    pub fn set_tip_radius(&mut self, tip_radius: f64) -> Result<(), GearDesignError> {
        let arc = self
            .profile
            .apply_tip_radius(self.outer_diameter, tip_radius)?;
        self.tip = Tip::Radius(arc);
        Ok(())
    }

    pub fn tip_radius(&self) -> Option<f64> {
        match self.tip {
            Tip::Sharp => None,
            Tip::Radius(arc) => Some(arc.radius),
        }
    }

    pub fn set_root_diameter(&mut self, root_diameter: f64) -> Result<(), GearDesignError> {
        let arc = self.profile.root_radius_at_root_diameter(root_diameter)?;
        self.root_radius = RootFillet::Partial(arc);
        Ok(())
    }

    pub fn set_root_fillet_radius(
        &mut self,
        root_fillet_radius: f64,
    ) -> Result<(), GearDesignError> {
        let arc = self.profile.root_fillet_radius(root_fillet_radius)?;
        self.root_radius = RootFillet::Partial(arc);
        Ok(())
    }

    pub fn root_diameter(&self) -> f64 {
        match self.root_radius {
            RootFillet::Full(arc) | RootFillet::Partial(arc) => {
                let end_points = arc.end_points().unwrap();
                end_points[0].radius().min(end_points[1].radius())
            }
        }
    }
    pub fn to_svg(&self) -> Result<String, GearDesignError> {
        let offset_angle = self.profile.offset_angle()?;
        let teeth = self.profile.teeth();
        let tooth_angle = 2.0 * PI / teeth as f64;

        let mut svg_path = String::new();

        let mut arcs = approximate_involute(&self.profile, 1e-6)?; // Tolerance for approximation
        if let Tip::Radius(arc) = self.tip {
            arcs.push(arc);
        }
        match self.root_radius {
            RootFillet::Full(arc) | RootFillet::Partial(arc) => {
                arcs.push(arc);
            }
        }

        // Rotate arcs by offset angle (in-place modification)
        for arc in &mut arcs {
            arc.center = arc.center.rotated(offset_angle, None);
            if let AngleSpan::Arc {
                ref mut start,
                ref mut end,
            } = arc.angle_span
            {
                *start += offset_angle;
                *end += offset_angle;
            }
        }

        // Copy and mirror arcs (using clone)
        let mut mirrored_arcs: Vec<CircularArc> = arcs
            .clone()
            .iter()
            .filter_map(|arc| match arc.angle_span {
                AngleSpan::Arc { start, end } => Some(CircularArc {
                    center: arc.center.mirror_vertical(),
                    radius: arc.radius,
                    angle_span: AngleSpan::Arc {
                        start: -start,
                        end: -end,
                    },
                }),
                _ => None,
            })
            .collect();

        let mut all_arcs = arcs;
        all_arcs.append(&mut mirrored_arcs);

        // Convert arcs to svg parameters
        let mut svg_params: Vec<CircularArcSvgParams> = all_arcs
            .iter()
            .filter_map(|arc| arc.svg_arc_params())
            .collect();

        for svg_param in &svg_params {
            svg_path.push_str(&svg_param.to_svg_path_segment());
        }

        // iterate through svg params pushing output to svg_path
        for _tooth in 1..self.profile.teeth() {
            for svg_param in &mut svg_params {
                svg_param.start = svg_param.start.rotated(tooth_angle, None);
                svg_param.end = svg_param.end.rotated(tooth_angle, None);
                svg_path.push_str(&svg_param.to_svg_path_segment());
            }
        }
        Ok(svg_path)
    }
}
