use core::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControllerRole {
    Primary,
    Secondary,
}

impl Display for ControllerRole {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ControllerRole::Primary => write!(f, "primary"),
            ControllerRole::Secondary => write!(f, "secondary"),
        }
    }
}
