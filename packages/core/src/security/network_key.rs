use core::ops::Deref;
use thiserror::Error;
use zwave_pal::prelude::*;

pub const NETWORK_KEY_SIZE: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("Network key must be 16 bytes long, got {actual}")]
pub struct NetworkKeyLengthError {
    pub actual: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct NetworkKey([u8; NETWORK_KEY_SIZE]);

impl NetworkKey {
    pub fn new(key: &[u8]) -> Self {
        Self::try_from(key).unwrap_or_else(|error| panic!("{error}"))
    }
}

impl TryFrom<&[u8]> for NetworkKey {
    type Error = NetworkKeyLengthError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != NETWORK_KEY_SIZE {
            return Err(NetworkKeyLengthError {
                actual: value.len(),
            });
        }

        let key = value.try_into().unwrap();
        Ok(Self(key))
    }
}

impl From<[u8; NETWORK_KEY_SIZE]> for NetworkKey {
    fn from(value: [u8; NETWORK_KEY_SIZE]) -> Self {
        Self(value)
    }
}

impl TryFrom<Vec<u8>> for NetworkKey {
    type Error = NetworkKeyLengthError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl TryFrom<&Vec<u8>> for NetworkKey {
    type Error = NetworkKeyLengthError;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl AsRef<[u8]> for NetworkKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for NetworkKey {
    type Target = [u8; NETWORK_KEY_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::fmt::Display for NetworkKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn try_from_slice_rejects_invalid_length() {
        let error = NetworkKey::try_from(&[0u8; 15][..]).unwrap_err();

        assert_eq!(error, NetworkKeyLengthError { actual: 15 });
    }

    #[test]
    fn try_from_slice_accepts_valid_length() {
        let key = NetworkKey::try_from(&[1u8; NETWORK_KEY_SIZE][..]).unwrap();

        assert_eq!(*key, [1u8; NETWORK_KEY_SIZE]);
    }
}
