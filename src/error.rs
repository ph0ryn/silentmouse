use thiserror::Error;

#[derive(Debug, Error)]
pub enum SilentMouseError {
    #[error("invalid coordinate {axis}={value}; coordinates must be finite numbers")]
    InvalidCoordinate { axis: &'static str, value: f64 },

    #[error("window {0} was not found or is not describable")]
    WindowNotFound(u32),

    #[error("window {0} is not on screen")]
    WindowOffscreen(u32),

    #[error("window {window_id} is missing {field}")]
    MissingWindowField { window_id: u32, field: &'static str },

    #[error("failed to create a CoreGraphics event")]
    EventCreationFailed,

    #[error("failed to resolve CGEventSetWindowLocation from CoreGraphics")]
    WindowLocationSetterUnavailable,

    #[error("failed to query whether pid {0} is active")]
    ActiveStateUnavailable(i32),

    #[error(
        "Accessibility permission is required; grant access to this terminal, binary, or app wrapper in System Settings and retry"
    )]
    AccessibilityPermissionRequired,
}

impl SilentMouseError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::InvalidCoordinate { .. } => 2,
            Self::WindowNotFound(_)
            | Self::WindowOffscreen(_)
            | Self::MissingWindowField { .. } => 3,
            Self::AccessibilityPermissionRequired => 5,
            Self::EventCreationFailed
            | Self::WindowLocationSetterUnavailable
            | Self::ActiveStateUnavailable(_) => 4,
        }
    }
}
