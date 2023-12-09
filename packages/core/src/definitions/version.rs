use std::fmt;

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

impl From<&str> for Version {
    fn from(s: &str) -> Self {
        let mut parts = s.split('.');
        let major = parts.next().unwrap().parse().unwrap();
        let minor = parts.next().unwrap().parse().unwrap();
        let patch = parts.next().map(|s| s.parse().unwrap());

        Version {
            major,
            minor,
            patch,
        }
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
