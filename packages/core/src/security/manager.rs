use super::crypto::encrypt_aes_ecb;
use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use getrandom::getrandom;
use std::collections::{BTreeMap, BTreeSet};

pub const NETWORK_KEY_SIZE: usize = 16;
pub type NetworkKey = Vec<u8>;
pub const S0_HALF_NONCE_SIZE: usize = 8;
pub const S0_NONCE_SIZE: usize = 16;

#[derive(Debug, Clone, PartialEq)]
pub struct S0Nonce {
    nonce: Bytes,
}

impl S0Nonce {
    pub fn new(nonce: Bytes) -> Self {
        if nonce.len() != S0_HALF_NONCE_SIZE {
            panic!("So nonce must be 8 bytes long, got {}", nonce.len());
        }
        Self { nonce }
    }

    pub fn random() -> Self {
        let mut buf = BytesMut::with_capacity(S0_HALF_NONCE_SIZE);
        getrandom(&mut buf[..S0_HALF_NONCE_SIZE])
            .unwrap_or_else(|_| panic!("Failed to generate random bytes"));
        Self {
            nonce: buf.freeze(),
        }
    }

    pub fn get(&self) -> &Bytes {
        &self.nonce
    }

    pub fn set(&mut self, nonce: Bytes) {
        if nonce.len() != S0_HALF_NONCE_SIZE {
            panic!("So nonce must be 8 bytes long, got {}", nonce.len());
        }
        self.nonce = nonce;
    }

    pub fn id(&self) -> u8 {
        self.nonce[0]
    }
}

impl std::fmt::Display for S0Nonce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.nonce))
    }
}

const AUTH_KEY_BASE: &[u8; NETWORK_KEY_SIZE] = &[0x55; NETWORK_KEY_SIZE];
const ENC_KEY_BASE: &[u8; NETWORK_KEY_SIZE] = &[0xaa; NETWORK_KEY_SIZE];

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
fn generate_auth_key(network_key: &NetworkKey) -> NetworkKey {
    encrypt_aes_ecb(AUTH_KEY_BASE, &network_key)
}

#[inline(always)]
fn generate_enc_key(network_key: &NetworkKey) -> NetworkKey {
    encrypt_aes_ecb(ENC_KEY_BASE, &network_key)
}

pub struct SecurityManager {
    own_node_id: NodeId,
    network_key: NetworkKey,
    auth_key: NetworkKey,
    enc_key: NetworkKey,
    nonce_store: BTreeMap<NonceKey, NonceEntry>,
    free_nonces: BTreeMap<NodeId, NonceKey>,
    receiver_nonces: BTreeMap<NodeId, NonceKey>,
}

impl SecurityManager {
    pub fn new(options: SecurityManagerOptions) -> Self {
        if options.network_key.len() != NETWORK_KEY_SIZE {
            panic!("The network key must be 16 bytes long!");
        }

        let auth_key = generate_auth_key(&options.network_key);
        let enc_key = generate_enc_key(&options.network_key);

        Self {
            own_node_id: options.own_node_id,
            network_key: options.network_key,
            auth_key,
            enc_key,
            nonce_store: BTreeMap::new(),
            free_nonces: BTreeMap::new(),
            receiver_nonces: BTreeMap::new(),
        }
    }

    fn has_nonce(&self, nonce_id: u8) -> bool {
        self.nonce_store.contains_key(&NonceKey {
            issuer: self.own_node_id,
            nonce_id,
        })
    }

    pub fn generate_nonce(&mut self, receiver: NodeId) -> S0Nonce {
        // Generate a nonce until we find one whose ID that is not already in use
        let nonce = loop {
            let nonce = S0Nonce::random();
            if !self.has_nonce(nonce.id()) {
                break nonce;
            }
        };

        // Store it
        self.set_nonce(self.own_node_id, receiver, nonce.clone(), false);

        nonce
    }

    pub fn set_nonce(&mut self, issuer: NodeId, receiver: NodeId, nonce: S0Nonce, free: bool) {
        let key = NonceKey {
            issuer,
            nonce_id: nonce.id(),
        };

        // If there is an existing nonce for the same receiver, remove it
        if let Some(existing_key) = self.receiver_nonces.get(&receiver) {
            self.nonce_store.remove(existing_key);
        }

        // Add the new one
        self.nonce_store.insert(key, NonceEntry { receiver, nonce });
        self.receiver_nonces.insert(receiver, key);

        // And mark it as free if requested
        if free {
            self.free_nonces.insert(issuer, key);
        }

        // TODO: Expire old nonces
    }

    /// Deletes a specific nonce if it exists
    pub fn delete_nonce(&mut self, issuer: NodeId, nonce_id: u8) {
        let key = NonceKey { issuer, nonce_id };

        // Remove the entry from the nonce store
        let old = self.nonce_store.remove(&key);

        // Delete the entry for the issuer from free_nonces if the stored key is the
        // expected one
        if self.free_nonces.get(&issuer) == Some(&key) {
            self.free_nonces.remove(&issuer);
        }

        // And delete the entry for the receiver from receiver_nonces
        if let Some(NonceEntry { receiver, .. }) = old {
            self.receiver_nonces.remove(&receiver);
        }
    }

    /// Deletes the nonce stored for a given receiver
    pub fn delete_nonce_for_receiver(&mut self, receiver: NodeId) {
        let key = self.receiver_nonces.remove(&receiver);
        if let Some(NonceKey { issuer, nonce_id }) = key {
            self.delete_nonce(issuer, nonce_id);
        }
    }

    /// Tries to retrieve a specific nonce issued by ourselves. The same nonce
    /// can only be retrieved once.
    pub fn try_get_own_nonce(&mut self, nonce_id: u8) -> Option<S0Nonce> {
        self.try_get_nonce(self.own_node_id, nonce_id)
    }

    /// Tries to retrieve a specific nonce by ID for a given node. The same nonce
    /// can only be retrieved once.
    pub fn try_get_nonce(&mut self, issuer: NodeId, nonce_id: u8) -> Option<S0Nonce> {
        let key = NonceKey { issuer, nonce_id };
        // If the nonce was previously free, it no longer is
        self.free_nonces.remove(&issuer);
        // And return the nonce if it was found
        self.nonce_store.remove(&key).map(|e| e.nonce)
    }

    /// Tries to claim a nonce that is not reserved for a specific transaction.
    /// If a nonce is found, it is no longer considered free afterwards
    pub fn try_claim_nonce(&mut self, issuer: NodeId) -> Option<S0Nonce> {
        // Find and possibly remove an entry for the given Node ID from the free_nonces map
        let key = self.free_nonces.remove(&issuer)?;

        // With that, try to find the nonce in the nonce store
        let entry = self.nonce_store.get(&key).map(|e| e.nonce.clone());

        entry
    }

    pub fn auth_key(&self) -> &[u8] {
        &self.auth_key
    }

    pub fn enc_key(&self) -> &[u8] {
        &self.enc_key
    }
}
