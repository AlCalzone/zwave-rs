use core::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SecurityClass {
    S2Unauthenticated = 0,
    S2Authenticated = 1,
    S2AccessControl = 2,
    S0Legacy = 7,
}

impl SecurityClass {
    pub const ALL_ASCENDING: &'static [Self] = &[
        Self::S0Legacy,
        Self::S2Unauthenticated,
        Self::S2Authenticated,
        Self::S2AccessControl,
    ];

    pub const ALL_DESCENDING: &'static [Self] = &[
        Self::S2AccessControl,
        Self::S2Authenticated,
        Self::S2Unauthenticated,
        Self::S0Legacy,
    ];

    pub const ALL_S2_ASCENDING: &'static [Self] = &[
        Self::S2Unauthenticated,
        Self::S2Authenticated,
        Self::S2AccessControl,
    ];

    pub const ALL_S2_DESCENDING: &'static [Self] = &[
        Self::S2AccessControl,
        Self::S2Authenticated,
        Self::S2Unauthenticated,
    ];

    pub const fn is_s2(self) -> bool {
        !matches!(self, Self::S0Legacy)
    }

    /// Used to help with ordering the security classes semantically
    const fn security_level(self) -> u8 {
        match self {
            Self::S0Legacy => 0,
            Self::S2Unauthenticated => 1,
            Self::S2Authenticated => 2,
            Self::S2AccessControl => 3,
        }
    }
}

impl Ord for SecurityClass {
    fn cmp(&self, other: &Self) -> Ordering {
        self.security_level().cmp(&other.security_level())
    }
}

impl PartialOrd for SecurityClass {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::SecurityClass;

    #[test]
    fn orders_security_classes_semantically() {
        let mut classes = vec![
            SecurityClass::S0Legacy,
            SecurityClass::S2Authenticated,
            SecurityClass::S2Unauthenticated,
            SecurityClass::S2AccessControl,
        ];

        classes.sort_unstable();

        assert_eq!(
            classes,
            vec![
                SecurityClass::S0Legacy,
                SecurityClass::S2Unauthenticated,
                SecurityClass::S2Authenticated,
                SecurityClass::S2AccessControl,
            ]
        );
    }
}
