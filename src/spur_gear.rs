use std::f64::consts::PI;

use crate::{
    error::{GearDesignError, GearProfileError, SpurGearProfileError},
    gear_profile::{approximate_involute, GearProfile},
    geometry::{AngleSpan, CircularArc, CircularArcSvgParams, Point},
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
    root_fillet: RootFillet,

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
            root_fillet: RootFillet::Full(arc),
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
        self.root_fillet = RootFillet::Partial(arc);
        Ok(())
    }

    pub fn set_root_fillet_radius(
        &mut self,
        root_fillet_radius: f64,
    ) -> Result<(), GearDesignError> {
        let arc = self.profile.root_fillet_radius(root_fillet_radius)?;
        self.root_fillet = RootFillet::Partial(arc);
        Ok(())
    }

    pub fn root_diameter(&self) -> f64 {
        match self.root_fillet {
            RootFillet::Full(arc) | RootFillet::Partial(arc) => {
                let end_points = arc.end_points().unwrap();
                2.0 * end_points[0].radius().min(end_points[1].radius())
            }
        }
    }

    pub fn outer_diameter(&self) -> f64 {
        match self.tip {
            Tip::Sharp => self.profile.involute_end_diameter(),
            Tip::Radius(arc) => {
                let [_start, end] = arc
                    .end_points()
                    .ok_or(GearProfileError::InvalidTipRadius)
                    .unwrap();
                2.0 * end.radius()
            }
        }
    }

    pub fn to_svg(&self) -> Result<String, GearDesignError> {
        let offset_angle = -self.profile.offset_angle()?;
        let teeth = self.profile.teeth();
        let tooth_angle = 2.0 * PI / teeth as f64;
        let tooth_half_angle = tooth_angle * 0.5;

        let mut svg_path = String::new();

        let mut involute_arcs: Vec<CircularArc> = approximate_involute(&self.profile, 1e-3)?
            .into_iter()
            .map(|arc| arc.rotated_about_origin(offset_angle))
            .collect();

        let root_arc = match self.root_fillet {
            RootFillet::Full(arc) | RootFillet::Partial(arc) => {
                arc.rotated_about_origin(offset_angle)
            }
        };
        let mut root_arcs = vec![root_arc, root_arc.mirror_vertical()];

        let mut bottom_land_arcs = match self.root_fillet {
            RootFillet::Full(_) => Vec::new(),
            RootFillet::Partial(arc) => {
                let [start, end] = arc
                    .end_points()
                    .ok_or(GearProfileError::InvalidRootRadius)?;
                let origin = Point { x: 0.0, y: 0.0 };
                println!("start: {}", start);
                println!("end: {}", end);
                let arc = CircularArc::new(
                    origin,
                    end.radius(),
                    -tooth_half_angle,
                    end.angle() + offset_angle,
                );
                vec![arc, arc.mirror_vertical()]
            }
        };

        let mut tip_arcs = match self.tip {
            Tip::Sharp => {
                let final_involute_arc = involute_arcs
                    .last()
                    .ok_or(GearProfileError::InvoluteApproximationFailed)?;
                let [start, end] = final_involute_arc
                    .end_points()
                    .ok_or(GearProfileError::InvoluteApproximationFailed)?;
                let origin = Point { x: 0.0, y: 0.0 };
                let angle = end.angle();
                let arc = CircularArc::new(origin, end.radius(), angle, -angle);
                vec![arc]
            }
            Tip::Radius(arc) => {
                let arc = arc.rotated_about_origin(offset_angle);
                let [start, end] = arc.end_points().ok_or(GearProfileError::InvalidTipRadius)?;
                let origin = Point { x: 0.0, y: 0.0 };
                let angle = end.angle();
                let top_land_arc = CircularArc::new(origin, end.radius(), angle, -angle);

                vec![arc, top_land_arc, arc.mirror_vertical()]
            }
        };

        let mut mirrored = (&involute_arcs)
            .into_iter()
            .map(|arc| arc.mirror_vertical())
            .collect();
        involute_arcs.append(&mut mirrored);

        let mut all_arcs = Vec::new();
        all_arcs.append(&mut involute_arcs);
        all_arcs.append(&mut root_arcs);
        all_arcs.append(&mut bottom_land_arcs);
        all_arcs.append(&mut tip_arcs);

        for arc in &all_arcs {
            println!("arc: {}", arc);
        }

        // Convert arcs to svg parameters
        let mut svg_params: Vec<CircularArcSvgParams> = all_arcs
            .iter()
            .filter_map(|arc| arc.svg_arc_params())
            .collect();

        let scale_factor = 100.0;
        for svg_param in &svg_params {
            svg_path.push_str(&svg_param.to_svg_path_segment(scale_factor));
        }

        // iterate through svg params pushing output to svg_path
        for _tooth in 1..self.profile.teeth() {
            for svg_param in &mut svg_params {
                svg_param.start = svg_param.start.rotated(tooth_angle, None);
                svg_param.end = svg_param.end.rotated(tooth_angle, None);
                svg_path.push_str(&svg_param.to_svg_path_segment(scale_factor));
            }
        }
        // Construct the final SVG
        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="-{0} -{0} {1} {1}">
               <path d="{2}" fill="none" stroke="black" />
               <circle cx="0" cy="0" r="{3}" fill="none" stroke="red" stroke-dasharray="5,5" />
               <circle cx="0" cy="0" r="{4}" fill="none" stroke="green" stroke-dasharray="5,5" />
               <circle cx="0" cy="0" r="{5}" fill="none" stroke="gray" stroke-dasharray="5,5" />
               <circle cx="0" cy="0" r="{6}" fill="none" stroke="gray" stroke-dasharray="5,5" />
             </svg>"#,
            scale_factor * self.outer_diameter,
            scale_factor * self.outer_diameter * 2.0,
            svg_path,
            self.profile.pitch_diameter() / 2.0 * scale_factor,
            self.profile.base_diameter() / 2.0 * scale_factor,
            self.outer_diameter() / 2.0 * scale_factor,
            self.root_diameter() / 2.0 * scale_factor,
        );

        Ok(svg)
    }
}
