use thiserror::Error;

/// Represents errors that can occur in the gear library.
#[derive(Error, Debug, PartialEq)]
pub enum GearProfileError {
    /// Indicates that the pressure angle is out of the valid range.
    #[error("Pressure angle must be between 0 and 90 degrees.")]
    InvalidPressureAngle,

    /// Indicates that the form diameter is less than the base diameter.
    #[error("Form diameter cannot be less than base diameter.")]
    InvalidFormDiameter,

    /// Indicates that the involute end diameter is less than the pitch diameter.
    #[error("Involute end diameter cannot be less than pitch diameter.")]
    InvalidInvoluteEndDiameter,

    /// Indicates that the involute end diameter is too large and will cause self-intersection.
    #[error("Involute end diameter is too large and will cause self-intersection.")]
    InvoluteEndDiameterTooLarge,

    /// Indicates that the circular thickness exceeds the maximum allowed value.
    #[error("Circular thickness exceeds maximum allowed value.")]
    InvalidCircularThickness,

    /// Indicates that the evaluated diameter is less than the base diameter when calculating the roll angle.
    #[error("Evaluated diameter cannot be less than base diameter when calculating roll angle.")]
    InvalidInvoluteDiameter,

    /// Indicates that the involute approximation process failed to converge.
    #[error("Involute approximation failed to converge within the allowed number of iterations.")]
    InvoluteApproximationFailed,

    /// Docstring
    #[error("message")]
    InvalidTipRadius,

    /// Docstring
    #[error("InvalidRootRadius")]
    InvalidRootRadius,
}

/// Represents errors that can occur in the geometry library.
#[derive(Error, Debug, PartialEq)]
pub enum GeometryError {
    /// The three points provided are collinear.
    #[error("The provided points are collinear and do not define a unique arc.")]
    CollinearPoints,
    /// The interior point does not lie on the fitted arc.
    #[error("The interior point does not lie on the fitted arc.")]
    InteriorPointNotOnArc,
}

/// Represents errors specific to GearMesh validation and operations.
#[derive(Error, Debug, PartialEq)]
pub enum GearMeshError {
    /// Indicates that the gear modules are incompatible.
    #[error("Gear modules are incompatible.")]
    IncompatibleModule,

    /// Indicates that the ratio of teeth does not match the ratio of base diameters.
    #[error("The ratio of teeth does not match the ratio of base diameters.")]
    IncompatibleBaseDiameterRatio,
}

#[derive(Error, Debug, PartialEq)]
pub enum SpurGearProfileError {}

#[derive(Error, Debug, PartialEq)]
pub enum GearDesignError {
    #[error(transparent)]
    Gear(#[from] GearProfileError),
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    #[error(transparent)]
    Mesh(#[from] GearMeshError),
}
