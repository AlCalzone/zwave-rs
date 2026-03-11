#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum SecurityClass {
    S2Unauthenticated = 0,
    S2Authenticated = 1,
    S2AccessControl = 2,
    S0Legacy = 7,
}
