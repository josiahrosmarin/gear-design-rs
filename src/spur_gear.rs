use crate::{error::SpurGearProfileError, gear_profile::GearProfile};

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
    Radius(f64),

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
    Full,

    /// A partial radius that terminates into the bottom land.
    ///
    /// Represents a root with a circular arc that blends into the bottom land.
    Partial(f64),

    /// A trochoidal root radius.
    ///
    /// Represents a root formed by a trochoidal curve, with parameters defined in the `TrochoidParams` struct.
    Trochoid(TrochoidParams),
}

/// Represents a complete gear with its involute profile and manufacturing parameters.
pub struct Gear {
    /// The involute profile of the gear teeth.
    ///
    /// This field encapsulates the essential involute profile data, defined in the `GearProfile` struct.
    pub profile: GearProfile,

    /// The root diameter (bottom land diameter) of the gear.
    ///
    /// This is the diameter at the bottom of the tooth space.
    pub root_diameter: f64,

    /// The root radius of the gear teeth.
    ///
    /// This field defines the shape of the root of the tooth, using the `RootFillet` enum.
    pub root_radius: RootFillet,

    /// The tip of the gear teeth.
    ///
    /// This field defines the shape of the tip of the tooth, using the `Tip` enum.
    pub tip: Tip,

    /// The outer diameter of the gear.
    ///
    /// This is the overall diameter of the gear.
    pub outer_diameter: f64,

    /// The face width of the gear.
    ///
    /// This is the width of the gear teeth along the axis of rotation.
    pub face_width: f64,
    // Add other parameters as needed (e.g., bore diameter, material, etc.)
}
