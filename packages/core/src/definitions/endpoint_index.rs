use crate::values::Canonical;
use std::fmt::Display;

#[derive(Default, Debug, Copy, Clone, Eq)]
pub enum EndpointIndex {
    #[default]
    Root,
    Endpoint(u8),
}

impl Canonical for EndpointIndex {
    fn to_canonical(&self) -> Self {
        match self {
            EndpointIndex::Root | EndpointIndex::Endpoint(0) => EndpointIndex::Root,
            EndpointIndex::Endpoint(_) => *self,
        }
    }
}

impl PartialEq<EndpointIndex> for EndpointIndex {
    fn eq(&self, other: &EndpointIndex) -> bool {
        match (self.to_canonical(), other.to_canonical()) {
            (EndpointIndex::Root, EndpointIndex::Root) => true,
            (EndpointIndex::Endpoint(a), EndpointIndex::Endpoint(b)) => a == b,
            _ => false,
        }
    }
}

impl Ord for EndpointIndex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.to_canonical(), other.to_canonical()) {
            (EndpointIndex::Root, EndpointIndex::Root) => std::cmp::Ordering::Equal,
            (EndpointIndex::Root, EndpointIndex::Endpoint(_)) => std::cmp::Ordering::Less,
            (EndpointIndex::Endpoint(_), EndpointIndex::Root) => std::cmp::Ordering::Greater,
            (EndpointIndex::Endpoint(a), EndpointIndex::Endpoint(b)) => a.cmp(&b),
        }
    }
}

impl PartialOrd<EndpointIndex> for EndpointIndex {
    fn partial_cmp(&self, other: &EndpointIndex) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[test]
fn test_endpoint_index_ord() {
    assert!(EndpointIndex::Root == EndpointIndex::Endpoint(0));
    assert!(EndpointIndex::Endpoint(0) < EndpointIndex::Endpoint(1));
    assert!(EndpointIndex::Endpoint(1) < EndpointIndex::Endpoint(2));
}

impl Display for EndpointIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EndpointIndex::Root => write!(f, "Root endpoint"),
            EndpointIndex::Endpoint(index) => write!(f, "Endpoint {}", index),
        }
    }
}
