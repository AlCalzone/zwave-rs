use crate::submodule;

submodule!(level);
submodule!(binary);
submodule!(duration);

pub trait Canonical {
    /// Converts the value to its canonical representation, eliminating illegal values
    fn to_canonical(&self) -> Self;
}
