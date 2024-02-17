use crate::munch::ParseError;
use std::fmt::{self, Display};
#[derive(Clone, Copy, Eq, PartialOrd)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: Option<u8>,
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.patch.or(Some(0)) == other.patch.or(Some(0))
    }
}

impl std::fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(patch) = self.patch {
            write!(f, "{}.{}.{}", self.major, self.minor, patch)
        } else {
            write!(f, "{}.{}", self.major, self.minor)
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<&str> for Version {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts: Result<Vec<_>, _> = value.split('.').take(3).map(|s| s.parse::<u8>()).collect();
        let parts = parts
            .map_err(|_| ParseError::validation_failure(format!("Invalid version {}", value)))?;
        if parts.len() < 2 {
            return Err(ParseError::validation_failure(format!(
                "Invalid version {}",
                value
            )));
        }

        Ok(Version {
            #[allow(clippy::get_first)]
            major: *parts.get(0).unwrap(),
            minor: *parts.get(1).unwrap(),
            patch: parts.get(2).copied(),
        })
    }
}

#[test]
fn test_version_comparison() {
    let v1 = Version {
        major: 1,
        minor: 2,
        patch: Some(3),
    };

    let v2 = Version {
        major: 1,
        minor: 2,
        patch: Some(3),
    };

    assert_eq!(v1, v2);

    let v1 = Version {
        major: 1,
        minor: 2,
        patch: Some(3),
    };

    let v2 = Version {
        major: 1,
        minor: 2,
        patch: None,
    };

    assert!(v1 > v2);

    let v1 = Version {
        major: 1,
        minor: 3,
        patch: Some(3),
    };

    let v2 = Version {
        major: 2,
        minor: 2,
        patch: Some(3),
    };

    assert!(v1 < v2);

    let v1 = Version {
        major: 2,
        minor: 5,
        patch: Some(0),
    };

    let v2 = Version {
        major: 2,
        minor: 5,
        patch: None,
    };

    assert_eq!(v1, v2);
}

#[test]
fn test_optional_version_comparison() {
    let v1 = None;

    let v2 = Some(Version {
        major: 2,
        minor: 5,
        patch: None,
    });

    assert!(v1 < v2);
}
