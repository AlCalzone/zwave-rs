use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControllerRole {
    Primary,
    Secondary,
}

impl Display for ControllerRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControllerRole::Primary => write!(f, "primary"),
            ControllerRole::Secondary => write!(f, "secondary"),
        }
    }
}
