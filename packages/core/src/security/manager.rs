use super::{AesKey, NetworkKey, encrypt_aes_ecb};
use crate::prelude::*;
use alloc::collections::BTreeMap;
use core::ops::Deref;
use zwave_pal::rng::getrandom;
use zwave_pal::prelude::*;
use zwave_pal::sync::Locked;

pub const S0_NONCE_SIZE: usize = 8;
pub const S0_IV_SIZE: usize = 2 * S0_NONCE_SIZE;

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct S0Nonce([u8; S0_NONCE_SIZE]);

impl S0Nonce {
    pub fn new(nonce: &[u8]) -> Self {
        if nonce.len() != S0_NONCE_SIZE {
            panic!("S0 nonce must be 8 bytes long, got {}", nonce.len());
        }
        let nonce = nonce.try_into().unwrap();
        Self(nonce)
    }

    pub fn random() -> Self {
        let mut nonce = [0u8; S0_NONCE_SIZE];
        getrandom(&mut nonce).unwrap_or_else(|_| panic!("Failed to generate random bytes"));
        Self(nonce)
    }

    pub fn id(&self) -> u8 {
        self.0[0]
    }
}

impl From<Vec<u8>> for S0Nonce {
    fn from(value: Vec<u8>) -> Self {
        Self::new(&value)
    }
}

impl From<&Vec<u8>> for S0Nonce {
    fn from(value: &Vec<u8>) -> Self {
        Self::new(value)
    }
}

impl From<&[u8]> for S0Nonce {
    fn from(value: &[u8]) -> Self {
        Self::new(value)
    }
}

impl AsRef<[u8]> for S0Nonce {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for S0Nonce {
    type Target = [u8; S0_NONCE_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::fmt::Display for S0Nonce {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

const AUTH_KEY_BASE: [u8; 16] = [0x55; 16];
const ENC_KEY_BASE: [u8; 16] = [0xaa; 16];

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
struct NonceKey {
    issuer: NodeId,
    nonce_id: u8,
}

struct NonceEntry {
    nonce: S0Nonce,
    receiver: NodeId,
}

pub struct SecurityManagerOptions {
    pub own_node_id: NodeId,
    pub network_key: NetworkKey,
}

#[inline(always)]
fn generate_auth_key(network_key: &NetworkKey) -> AesKey {
    let network_key = AesKey::from(network_key);
    encrypt_aes_ecb(&AUTH_KEY_BASE, &network_key).into()
}

#[inline(always)]
fn generate_enc_key(network_key: &NetworkKey) -> AesKey {
    let network_key = AesKey::from(network_key);
    encrypt_aes_ecb(&ENC_KEY_BASE, &network_key).into()
}

struct SecurityManagerState {
    nonce_store: BTreeMap<NonceKey, NonceEntry>,
    free_nonces: BTreeMap<NodeId, NonceKey>,
    receiver_nonces: BTreeMap<NodeId, NonceKey>,
}

pub struct SecurityManagerStorage {
    own_node_id: NodeId,
    network_key: NetworkKey,
    auth_key: AesKey,
    enc_key: AesKey,
    state: Locked<SecurityManagerState>,
}

impl SecurityManagerStorage {
    pub fn new(options: SecurityManagerOptions) -> Self {
        let auth_key = generate_auth_key(&options.network_key);
        let enc_key = generate_enc_key(&options.network_key);

        Self {
            own_node_id: options.own_node_id,
            network_key: options.network_key,
            auth_key,
            enc_key,
            state: Locked::new(SecurityManagerState {
                nonce_store: BTreeMap::new(),
                free_nonces: BTreeMap::new(),
                receiver_nonces: BTreeMap::new(),
            }),
        }
    }
}

#[derive(Clone)]
pub struct SecurityManager {
    storage: Arc<SecurityManagerStorage>,
}

impl SecurityManager {
    pub fn new(storage: Arc<SecurityManagerStorage>) -> Self {
        Self { storage }
    }

    fn has_nonce(&self, nonce_id: u8) -> bool {
        self.storage.state.inspect(|state| {
            state.nonce_store.contains_key(&NonceKey {
                issuer: self.storage.own_node_id,
                nonce_id,
            })
        })
    }

    pub fn generate_nonce(&self, receiver: NodeId) -> S0Nonce {
        // Generate a nonce until we find one whose ID that is not already in use
        let nonce = loop {
            let nonce = S0Nonce::random();
            if !self.has_nonce(nonce.id()) {
                break nonce;
            }
        };

        // Store it
        self.set_nonce(self.storage.own_node_id, receiver, nonce.clone(), false);

        nonce
    }

    pub fn set_nonce(&self, issuer: NodeId, receiver: NodeId, nonce: S0Nonce, free: bool) {
        let key = NonceKey {
            issuer,
            nonce_id: nonce.id(),
        };

        self.storage.state.update(|state| {
            // If there is an existing nonce for the same receiver, remove it
            if let Some(existing_key) = state.receiver_nonces.get(&receiver) {
                state.nonce_store.remove(existing_key);
            }

            // Add the new one
            state
                .nonce_store
                .insert(key, NonceEntry { receiver, nonce });
            state.receiver_nonces.insert(receiver, key);

            // And mark it as free if requested
            if free {
                state.free_nonces.insert(issuer, key);
            }

            // TODO: Expire old nonces
        });
    }

    /// Deletes a specific nonce if it exists
    fn delete_nonce(&self, issuer: NodeId, nonce_id: u8) {
        let key = NonceKey { issuer, nonce_id };
        self.storage.state.update(|state| {
            // Remove the entry from the nonce store
            let old = state.nonce_store.remove(&key);

            // Delete the entry for the issuer from free_nonces if the stored key is the
            // expected one
            if state.free_nonces.get(&issuer) == Some(&key) {
                state.free_nonces.remove(&issuer);
            }

            // And delete the entry for the receiver from receiver_nonces
            if let Some(NonceEntry { receiver, .. }) = old {
                state.receiver_nonces.remove(&receiver);
            }
        });
    }

    /// Deletes the nonce stored for a given receiver
    pub fn delete_nonce_for_receiver(&self, receiver: NodeId) {
        let key = self
            .storage
            .state
            .update(|state| state.receiver_nonces.remove(&receiver));
        if let Some(NonceKey { issuer, nonce_id }) = key {
            self.delete_nonce(issuer, nonce_id);
        }
    }

    /// Deletes a nonce that was issued by ourselves
    pub fn delete_own_nonce(&self, nonce_id: u8) {
        self.delete_nonce(self.storage.own_node_id, nonce_id);
    }

    /// Tries to retrieve a specific nonce issued by ourselves. The same nonce
    /// can only be retrieved once.
    pub fn try_get_own_nonce(&self, nonce_id: u8) -> Option<S0Nonce> {
        self.try_get_nonce(self.storage.own_node_id, nonce_id)
    }

    /// Tries to retrieve a specific nonce by ID for a given node. The same nonce
    /// can only be retrieved once.
    pub fn try_get_nonce(&self, issuer: NodeId, nonce_id: u8) -> Option<S0Nonce> {
        let key = NonceKey { issuer, nonce_id };
        self.storage.state.update(|state| {
            // If the nonce was previously free, it no longer is
            state.free_nonces.remove(&issuer);
            // And return the nonce if it was found
            state.nonce_store.remove(&key).map(|entry| entry.nonce)
        })
    }

    /// Tries to claim a nonce that is not reserved for a specific transaction.
    /// If a nonce is found, it is no longer considered free afterwards
    pub fn try_claim_nonce(&self, issuer: NodeId) -> Option<S0Nonce> {
        self.storage.state.update(|state| {
            let key = state.free_nonces.remove(&issuer)?;
            state.nonce_store.get(&key).map(|entry| entry.nonce.clone())
        })
    }

    pub fn auth_key(&self) -> &AesKey {
        &self.storage.auth_key
    }

    pub fn enc_key(&self) -> &AesKey {
        &self.storage.enc_key
    }
}
