use super::{
    AES_CCM_NONCE_SIZE, AesCcmNonce, AesKey, DerivedNetworkKeys, DerivedTempKeys,
    ENTROPY_INPUT_SIZE, Entropy, EntropyInput, PERSONALIZATION_STRING_SIZE, PersonalizationString,
    compute_nonce_prk, ctr_drbg::CtrDrbg, derive_mei, derive_network_keys, encrypt_aes_ecb,
    network_key::NETWORK_KEY_SIZE, network_key::NetworkKey,
};
use crate::{
    definitions::{NodeId, SecurityClass},
    wrapping_counter::WrappingCounter,
};
use alloc::collections::{BTreeMap, BTreeSet};
use core::{ops::Deref, time::Duration};
use getrandom::getrandom;
use zwave_pal::prelude::*;
use zwave_pal::sync::Locked;
use zwave_pal::time::Instant;

pub const S2_ENTROPY_INPUT_SIZE: usize = ENTROPY_INPUT_SIZE;
pub const S2_NONCE_SIZE: usize = AES_CCM_NONCE_SIZE;
pub const S2_MPAN_STATE_SIZE: usize = NETWORK_KEY_SIZE;
pub const S2_PERSONALIZATION_STRING_SIZE: usize = PERSONALIZATION_STRING_SIZE;

const SINGLECAST_MAX_SEQ_NUMS: usize = 1;
const SINGLECAST_NONCE_EXPIRY: Duration = Duration::from_millis(500);

macro_rules! fixed_bytes_type {
    ($name:ident, $size:expr, $display_name:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        pub struct $name([u8; $size]);

        impl $name {
            pub fn new(bytes: &[u8]) -> Self {
                if bytes.len() != $size {
                    panic!(
                        concat!($display_name, " must be {} bytes long, got {}"),
                        $size,
                        bytes.len()
                    );
                }
                Self(bytes.try_into().unwrap())
            }
        }

        impl From<[u8; $size]> for $name {
            fn from(value: [u8; $size]) -> Self {
                Self(value)
            }
        }

        impl From<$name> for [u8; $size] {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl From<Vec<u8>> for $name {
            fn from(value: Vec<u8>) -> Self {
                Self::new(&value)
            }
        }

        impl From<&Vec<u8>> for $name {
            fn from(value: &Vec<u8>) -> Self {
                Self::new(value)
            }
        }

        impl From<&[u8]> for $name {
            fn from(value: &[u8]) -> Self {
                Self::new(value)
            }
        }

        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        }

        impl AsMut<[u8]> for $name {
            fn as_mut(&mut self) -> &mut [u8] {
                &mut self.0
            }
        }

        impl Deref for $name {
            type Target = [u8; $size];

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "0x{}", hex::encode(self.0))
            }
        }
    };
}

fixed_bytes_type!(MpanState, S2_MPAN_STATE_SIZE, "MPAN state");
pub type S2Nonce = AesCcmNonce;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkKeys {
    pub pnk: NetworkKey,
    pub key_ccm: AesKey,
    pub key_mpan: AesKey,
    pub personalization_string: PersonalizationString,
}

impl NetworkKeys {
    fn new(pnk: NetworkKey, derived: DerivedNetworkKeys) -> Self {
        Self {
            pnk,
            key_ccm: derived.key_ccm,
            key_mpan: derived.key_mpan,
            personalization_string: derived.personalization_string,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TempNetworkKeys {
    pub key_ccm: AesKey,
    pub personalization_string: PersonalizationString,
}

impl From<DerivedTempKeys> for TempNetworkKeys {
    fn from(value: DerivedTempKeys) -> Self {
        Self {
            key_ccm: value.temp_key_ccm,
            personalization_string: value.temp_personalization_string,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeysForNode {
    Network(NetworkKeys),
    Temporary(TempNetworkKeys),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SPANState {
    RemoteEI,
    LocalEI,
    SPAN,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentSpan {
    pub nonce: S2Nonce,
    pub expires: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityKey {
    Temporary,
    Key(SecurityClass),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SPANTableEntry {
    RemoteEI {
        receiver_ei: EntropyInput,
    },
    LocalEI {
        receiver_ei: EntropyInput,
    },
    SPAN {
        key: SecurityKey,
        rng: CtrDrbg,
        current_span: Option<CurrentSpan>,
    },
}

impl SPANTableEntry {
    pub fn state(&self) -> SPANState {
        match self {
            Self::RemoteEI { .. } => SPANState::RemoteEI,
            Self::LocalEI { .. } => SPANState::LocalEI,
            Self::SPAN { .. } => SPANState::SPAN,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MPANState {
    OutOfSync,
    MPAN,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MPANTableEntry {
    OutOfSync,
    MPAN { current_mpan: MpanState },
}

impl MPANTableEntry {
    pub fn state(&self) -> MPANState {
        match self {
            Self::OutOfSync => MPANState::OutOfSync,
            Self::MPAN { .. } => MPANState::MPAN,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MulticastGroup {
    pub node_ids: Vec<NodeId>,
    pub security_class: SecurityClass,
    pub sequence_number: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MulticastKeyAndIv {
    pub key: AesKey,
    pub iv: S2Nonce,
}

struct SecurityManager2State {
    rng: CtrDrbg,
    span_table: BTreeMap<NodeId, SPANTableEntry>,
    temp_keys: BTreeMap<NodeId, TempNetworkKeys>,
    own_sequence_numbers: BTreeMap<NodeId, u8>,
    peer_sequence_numbers: BTreeMap<NodeId, Vec<u8>>,
    mpan_states: BTreeMap<u8, MpanState>,
    peer_mpans: BTreeMap<NodeId, BTreeMap<u8, MPANTableEntry>>,
    network_keys: BTreeMap<SecurityClass, NetworkKeys>,
    multicast_groups: BTreeMap<u8, MulticastGroup>,
    multicast_group_lookup: BTreeMap<Vec<NodeId>, u8>,
    multicast_group_counter: WrappingCounter<u8>,
}

pub struct SecurityManager2Storage {
    state: Locked<SecurityManager2State>,
}

impl SecurityManager2Storage {
    pub fn new() -> Self {
        let seed = Entropy::random();

        Self {
            state: Locked::new(SecurityManager2State {
                rng: CtrDrbg::new(seed),
                span_table: BTreeMap::new(),
                temp_keys: BTreeMap::new(),
                own_sequence_numbers: BTreeMap::new(),
                peer_sequence_numbers: BTreeMap::new(),
                mpan_states: BTreeMap::new(),
                peer_mpans: BTreeMap::new(),
                network_keys: BTreeMap::new(),
                multicast_groups: BTreeMap::new(),
                multicast_group_lookup: BTreeMap::new(),
                multicast_group_counter: WrappingCounter::new_with_max(u8::MAX),
            }),
        }
    }
}

impl Default for SecurityManager2Storage {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
/// Management class and utilities for Security S2.
pub struct SecurityManager2 {
    storage: Arc<SecurityManager2Storage>,
}

impl SecurityManager2 {
    /// Creates a new security manager backed by the given shared storage.
    pub fn new(storage: Arc<SecurityManager2Storage>) -> Self {
        Self { storage }
    }

    /// Sets the permanent network key for a given security class and derives the encryption keys from it.
    pub fn set_key<K>(&self, security_class: SecurityClass, key: K)
    where
        K: Into<NetworkKey>,
    {
        let key = key.into();
        let derived = derive_network_keys(&AesKey::from(&key));

        self.storage.state.update(|state| {
            state
                .network_keys
                .insert(security_class, NetworkKeys::new(key, derived));
        });
    }

    /// Returns whether permanent keys have been configured for the given security class.
    pub fn has_keys_for_security_class(&self, security_class: SecurityClass) -> bool {
        self.storage
            .state
            .inspect(|state| state.network_keys.contains_key(&security_class))
    }

    /// Returns the permanent keys for the given security class, if they have been configured.
    pub fn get_keys_for_security_class(
        &self,
        security_class: SecurityClass,
    ) -> Option<NetworkKeys> {
        self.storage
            .state
            .inspect(|state| state.network_keys.get(&security_class).cloned())
    }

    /// Stores the temporary keys used during key exchange for the given peer node.
    pub fn set_temp_keys(&self, peer_node_id: NodeId, keys: TempNetworkKeys) {
        self.storage
            .state
            .update(|state| state.temp_keys.insert(peer_node_id, keys));
    }

    /// Returns the temporary keys used during key exchange for the given peer node, if known.
    pub fn get_temp_keys(&self, peer_node_id: NodeId) -> Option<TempNetworkKeys> {
        self.storage
            .state
            .inspect(|state| state.temp_keys.get(&peer_node_id).cloned())
    }

    /// Removes and returns the temporary keys used during key exchange for the given peer node.
    pub fn remove_temp_keys(&self, peer_node_id: NodeId) -> Option<TempNetworkKeys> {
        self.storage
            .state
            .update(|state| state.temp_keys.remove(&peer_node_id))
    }

    /// Returns the effective keys for the given peer node based on the active SPAN state.
    pub fn get_keys_for_node(&self, peer_node_id: NodeId) -> Option<KeysForNode> {
        self.storage
            .state
            .inspect(|state| Self::keys_for_node_from_state(state, peer_node_id))
    }

    /// Returns the stored SPAN state for the given peer node, if any.
    pub fn get_span_state(&self, peer_node_id: NodeId) -> Option<SPANTableEntry> {
        self.storage
            .state
            .inspect(|state| state.span_table.get(&peer_node_id).cloned())
    }

    /// Tests whether the most recent secure command for a node has used the given key.
    pub fn has_used_key(&self, peer_node_id: NodeId, key: SecurityKey) -> bool {
        self.storage.state.inspect(|state| {
            matches!(
                state.span_table.get(&peer_node_id),
                Some(SPANTableEntry::SPAN {
                    key: used_key,
                    ..
                }) if *used_key == key
            )
        })
    }

    /// Prepares the generation of a new SPAN by creating local entropy input.
    ///
    /// If `receiver` is `Some`, the entropy input is stored as the local SPAN state for that node.
    pub fn generate_nonce(&self, receiver: Option<NodeId>) -> EntropyInput {
        self.storage.state.update(|state| {
            let receiver_ei: EntropyInput = state.rng.generate(S2_ENTROPY_INPUT_SIZE).into();
            if let Some(receiver) = receiver {
                state
                    .span_table
                    .insert(receiver, SPANTableEntry::LocalEI { receiver_ei });
            }
            receiver_ei
        })
    }

    /// Stores the given SPAN state in the table.
    ///
    /// This is primarily intended for internal state management.
    pub fn set_span_state(&self, peer_node_id: NodeId, state: Option<SPANTableEntry>) {
        self.storage.state.update(|span_state| {
            if let Some(state) = state {
                span_state.span_table.insert(peer_node_id, state);
            } else {
                span_state.span_table.remove(&peer_node_id);
            }
        });
    }

    /// Invalidates the SPAN state for the given receiver.
    pub fn delete_nonce(&self, receiver: NodeId) {
        self.storage.state.update(|state| {
            state.span_table.remove(&receiver);
            state.peer_sequence_numbers.remove(&receiver);
        });
    }

    /// Initializes the singlecast PAN generator for a given node based on the given entropy inputs.
    pub fn initialize_span(
        &self,
        peer_node_id: NodeId,
        security_class: SecurityClass,
        sender_ei: EntropyInput,
        receiver_ei: EntropyInput,
    ) -> bool {
        assert!(
            is_s2_security_class(security_class),
            "SPAN can only be initialized for S2 security classes",
        );

        let Some(keys) = self.get_keys_for_security_class(security_class) else {
            return false;
        };

        let nonce_prk = compute_nonce_prk(&sender_ei, &receiver_ei);
        let mei = derive_mei(&nonce_prk);
        let rng = CtrDrbg::new_with_personalization(mei, keys.personalization_string);

        self.storage.state.update(|state| {
            state.span_table.insert(
                peer_node_id,
                SPANTableEntry::SPAN {
                    key: SecurityKey::Key(security_class),
                    rng,
                    current_span: None,
                },
            );
        });
        true
    }

    /// Initializes the temporary singlecast PAN generator for a given node based on the given entropy inputs.
    pub fn initialize_temp_span(
        &self,
        peer_node_id: NodeId,
        sender_ei: EntropyInput,
        receiver_ei: EntropyInput,
    ) -> bool {
        let Some(keys) = self.get_temp_keys(peer_node_id) else {
            return false;
        };

        let nonce_prk = compute_nonce_prk(&sender_ei, &receiver_ei);
        let mei = derive_mei(&nonce_prk);
        let rng = CtrDrbg::new_with_personalization(mei, keys.personalization_string);

        self.storage.state.update(|state| {
            state.span_table.insert(
                peer_node_id,
                SPANTableEntry::SPAN {
                    key: SecurityKey::Temporary,
                    rng,
                    current_span: None,
                },
            );
        });
        true
    }

    /// Tests if the given combination of peer node ID and sequence number is a duplicate.
    pub fn is_duplicate_singlecast(&self, peer_node_id: NodeId, sequence_number: u8) -> bool {
        self.storage.state.inspect(|state| {
            state
                .peer_sequence_numbers
                .get(&peer_node_id)
                .map(|numbers| numbers.contains(&sequence_number))
                .unwrap_or(false)
        })
    }

    /// Stores the latest sequence number for the given peer node ID and returns the previous one.
    pub fn store_sequence_number(&self, peer_node_id: NodeId, sequence_number: u8) -> Option<u8> {
        self.storage.state.update(|state| {
            let numbers = state
                .peer_sequence_numbers
                .entry(peer_node_id)
                .or_insert_with(Vec::new);

            let previous = numbers.last().copied();
            numbers.push(sequence_number);
            if numbers.len() > SINGLECAST_MAX_SEQ_NUMS {
                numbers.remove(0);
            }

            previous
        })
    }

    /// Stores the remote entropy input for the given peer node.
    pub fn store_remote_ei(&self, peer_node_id: NodeId, remote_ei: EntropyInput) {
        self.storage.state.update(|state| {
            state.span_table.insert(
                peer_node_id,
                SPANTableEntry::RemoteEI {
                    receiver_ei: remote_ei,
                },
            );
        });
    }

    /// Generates the next nonce for the given peer and returns it.
    ///
    /// If `store` is `true`, the nonce is remembered as the current SPAN.
    pub fn next_nonce(&self, peer_node_id: NodeId, store: bool) -> Option<S2Nonce> {
        self.storage.state.update(|state| {
            let SPANTableEntry::SPAN {
                rng, current_span, ..
            } = state.span_table.get_mut(&peer_node_id)?
            else {
                return None;
            };

            let nonce: S2Nonce = rng.generate(S2_ENTROPY_INPUT_SIZE)[..S2_NONCE_SIZE].into();
            *current_span = if store {
                Some(CurrentSpan {
                    nonce,
                    expires: Instant::now() + SINGLECAST_NONCE_EXPIRY,
                })
            } else {
                None
            };
            Some(nonce)
        })
    }

    /// Returns the next sequence number to use for outgoing messages to the given node.
    pub fn next_sequence_number(&self, peer_node_id: NodeId) -> u8 {
        self.storage.state.update(|state| {
            let next = match state.own_sequence_numbers.get(&peer_node_id).copied() {
                Some(sequence_number) => sequence_number.wrapping_add(1),
                None => random_byte(),
            };
            state.own_sequence_numbers.insert(peer_node_id, next);
            next
        })
    }

    /// Creates or reuses a multicast group for the given node IDs and remembers the security class.
    ///
    /// The returned value is the group ID to be used in multicast commands.
    pub fn create_multicast_group(&self, node_ids: &[NodeId], security_class: SecurityClass) -> u8 {
        assert!(
            is_s2_security_class(security_class),
            "Multicast groups can only be created for S2 security classes",
        );

        self.storage.state.update(|state| {
            let lookup_key = canonicalize_node_ids(node_ids);
            if let Some(group_id) = state.multicast_group_lookup.get(&lookup_key).copied() {
                return group_id;
            }

            let group_id = state.multicast_group_counter.increment();
            if let Some(old_group) = state.multicast_groups.remove(&group_id) {
                let old_lookup_key = canonicalize_node_ids(&old_group.node_ids);
                state.multicast_group_lookup.remove(&old_lookup_key);
            }

            state.multicast_groups.insert(
                group_id,
                MulticastGroup {
                    node_ids: lookup_key.clone(),
                    security_class,
                    sequence_number: random_byte(),
                },
            );
            state.multicast_group_lookup.insert(lookup_key, group_id);
            state.mpan_states.remove(&group_id);

            group_id
        })
    }

    /// Returns the multicast group definition for the given group ID, if it exists.
    pub fn get_multicast_group(&self, group_id: u8) -> Option<MulticastGroup> {
        self.storage
            .state
            .inspect(|state| state.multicast_groups.get(&group_id).cloned())
    }

    /// Returns the next sequence number to use for outgoing messages to the given multicast group.
    pub fn next_multicast_sequence_number(&self, group_id: u8) -> Option<u8> {
        self.storage.state.update(|state| {
            let group = state.multicast_groups.get_mut(&group_id)?;
            group.sequence_number = group.sequence_number.wrapping_add(1);
            Some(group.sequence_number)
        })
    }

    /// Returns the inner MPAN state for the given multicast group, if it exists.
    pub fn get_inner_mpan_state(&self, group_id: u8) -> Option<MpanState> {
        self.storage
            .state
            .inspect(|state| state.mpan_states.get(&group_id).copied())
    }

    /// Returns the multicast key and IV for the given multicast group.
    ///
    /// The inner MPAN state is initialized on first use.
    pub fn get_multicast_key_and_iv(&self, group_id: u8) -> Option<MulticastKeyAndIv> {
        self.storage.state.update(|state| {
            let group = state.multicast_groups.get(&group_id)?.clone();
            let keys = state.network_keys.get(&group.security_class)?.clone();

            if !state.mpan_states.contains_key(&group_id) {
                let mpan_state = MpanState::from(state.rng.generate(S2_MPAN_STATE_SIZE));
                state.mpan_states.insert(group_id, mpan_state);
            }

            let current_mpan = state.mpan_states.get_mut(&group_id)?;
            let iv: S2Nonce = encrypt_aes_ecb(current_mpan, &keys.key_mpan)[..S2_NONCE_SIZE].into();
            increment_big_endian(current_mpan.as_mut());

            Some(MulticastKeyAndIv {
                key: keys.key_ccm,
                iv,
            })
        })
    }

    /// As part of MPAN maintenance, increments our own MPAN for a group if it is known.
    pub fn try_increment_mpan(&self, group_id: u8) {
        self.storage.state.update(|state| {
            if let Some(current_mpan) = state.mpan_states.get_mut(&group_id) {
                increment_big_endian(current_mpan.as_mut());
            }
        });
    }

    /// Generates the next peer MPAN nonce for the given peer node and group.
    pub fn next_peer_mpan(&self, peer_node_id: NodeId, group_id: u8) -> Option<S2Nonce> {
        self.storage.state.update(|state| {
            let key_mpan = match Self::keys_for_node_from_state(state, peer_node_id)? {
                KeysForNode::Network(keys) => keys.key_mpan,
                KeysForNode::Temporary(_) => return None,
            };

            let MPANTableEntry::MPAN { current_mpan } = state
                .peer_mpans
                .get_mut(&peer_node_id)?
                .get_mut(&group_id)?
            else {
                return None;
            };

            let iv: S2Nonce = encrypt_aes_ecb(current_mpan, &key_mpan)[..S2_NONCE_SIZE].into();
            increment_big_endian(current_mpan.as_mut());

            Some(iv)
        })
    }

    /// As part of MPAN maintenance, increments the peer's MPAN if it is known.
    pub fn try_increment_peer_mpan(&self, peer_node_id: NodeId, group_id: u8) {
        self.storage.state.update(|state| {
            let Some(peer_mpans) = state.peer_mpans.get_mut(&peer_node_id) else {
                return;
            };
            let Some(MPANTableEntry::MPAN { current_mpan }) = peer_mpans.get_mut(&group_id) else {
                return;
            };
            increment_big_endian(current_mpan.as_mut());
        });
    }

    /// Returns the stored MPAN used to decrypt messages from the given peer node and group.
    pub fn get_peer_mpan(&self, peer_node_id: NodeId, group_id: u8) -> Option<MPANTableEntry> {
        self.storage.state.inspect(|state| {
            state
                .peer_mpans
                .get(&peer_node_id)
                .and_then(|groups| groups.get(&group_id))
                .cloned()
        })
    }

    /// Resets all out-of-sync MPANs for the given node.
    pub fn reset_out_of_sync_mpans(&self, peer_node_id: NodeId) {
        self.storage.state.update(|state| {
            let Some(entries) = state.peer_mpans.get_mut(&peer_node_id) else {
                return;
            };
            entries.retain(|_, entry| !matches!(entry, MPANTableEntry::OutOfSync));
            if entries.is_empty() {
                state.peer_mpans.remove(&peer_node_id);
            }
        });
    }

    /// Stores the MPAN state used to decrypt messages from the given peer node and group.
    pub fn store_peer_mpan(&self, peer_node_id: NodeId, group_id: u8, mpan_state: MPANTableEntry) {
        self.storage.state.update(|state| {
            state
                .peer_mpans
                .entry(peer_node_id)
                .or_insert_with(BTreeMap::new)
                .insert(group_id, mpan_state);
        });
    }

    fn keys_for_node_from_state(
        state: &SecurityManager2State,
        peer_node_id: NodeId,
    ) -> Option<KeysForNode> {
        match state.span_table.get(&peer_node_id)? {
            SPANTableEntry::SPAN {
                key: SecurityKey::Temporary,
                ..
            } => state
                .temp_keys
                .get(&peer_node_id)
                .cloned()
                .map(KeysForNode::Temporary),
            SPANTableEntry::SPAN {
                key: SecurityKey::Key(security_class),
                ..
            } => state
                .network_keys
                .get(security_class)
                .cloned()
                .map(KeysForNode::Network),
            _ => None,
        }
    }
}

fn increment_big_endian(buffer: &mut [u8]) {
    for byte in buffer.iter_mut().rev() {
        *byte = byte.wrapping_add(1);
        if *byte != 0 {
            break;
        }
    }
}

fn random_byte() -> u8 {
    let mut buffer = [0u8; 1];
    getrandom(&mut buffer).unwrap_or_else(|_| panic!("Failed to generate random bytes"));
    buffer[0]
}

fn canonicalize_node_ids(node_ids: &[NodeId]) -> Vec<NodeId> {
    node_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn is_s2_security_class(security_class: SecurityClass) -> bool {
    matches!(
        security_class,
        SecurityClass::S2Unauthenticated
            | SecurityClass::S2Authenticated
            | SecurityClass::S2AccessControl
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::security::derive_temp_keys;

    fn create_manager() -> SecurityManager2 {
        SecurityManager2::new(Arc::new(SecurityManager2Storage::new()))
    }

    fn s2_key(byte: u8) -> NetworkKey {
        [byte; NETWORK_KEY_SIZE].into()
    }

    fn entropy(byte: u8) -> EntropyInput {
        [byte; S2_ENTROPY_INPUT_SIZE].into()
    }

    #[test]
    fn next_nonce_requires_initialized_span() {
        let manager = create_manager();

        assert_eq!(manager.next_nonce(2.into(), false), None);
    }

    #[test]
    fn initialize_span_and_generate_nonces() {
        let manager = create_manager();
        manager.set_key(SecurityClass::S2Authenticated, s2_key(1));

        assert!(manager.initialize_span(
            2.into(),
            SecurityClass::S2Authenticated,
            entropy(2),
            entropy(3),
        ));

        let nonce_1 = manager.next_nonce(2.into(), false).unwrap();
        let nonce_2 = manager.next_nonce(2.into(), false).unwrap();
        assert_eq!(nonce_1.len(), S2_NONCE_SIZE);
        assert_ne!(nonce_1, nonce_2);
    }

    #[test]
    fn initialize_span_without_known_key_fails() {
        let manager = create_manager();

        assert!(!manager.initialize_span(
            2.into(),
            SecurityClass::S2Authenticated,
            entropy(2),
            entropy(3),
        ));
    }

    #[test]
    fn initialize_temp_span_uses_temporary_keys() {
        let manager = create_manager();
        manager.set_temp_keys(
            2.into(),
            derive_temp_keys(&AesKey::from([7; NETWORK_KEY_SIZE])).into(),
        );

        assert!(manager.initialize_temp_span(2.into(), entropy(2), entropy(3)));
        assert!(matches!(
            manager.get_keys_for_node(2.into()),
            Some(KeysForNode::Temporary(_))
        ));
        assert!(matches!(
            manager.get_span_state(2.into()),
            Some(SPANTableEntry::SPAN {
                key: SecurityKey::Temporary,
                ..
            })
        ));
    }

    #[test]
    fn store_sequence_number_tracks_duplicates() {
        let manager = create_manager();

        assert_eq!(manager.store_sequence_number(2.into(), 1), None);
        assert!(!manager.is_duplicate_singlecast(2.into(), 2));
        assert!(manager.is_duplicate_singlecast(2.into(), 1));
        assert_eq!(manager.store_sequence_number(2.into(), 2), Some(1));
        assert!(!manager.is_duplicate_singlecast(2.into(), 1));
        assert!(manager.is_duplicate_singlecast(2.into(), 2));
    }

    #[test]
    fn create_multicast_group_reuses_existing_node_set() {
        let manager = create_manager();
        let group_1 = manager.create_multicast_group(
            &[2.into(), 3.into(), 4.into()],
            SecurityClass::S2Authenticated,
        );
        let group_2 = manager.create_multicast_group(
            &[4.into(), 3.into(), 2.into()],
            SecurityClass::S2Authenticated,
        );

        assert_eq!(group_1, group_2);
    }

    #[test]
    fn get_multicast_key_and_iv_generates_unique_ivs() {
        let manager = create_manager();
        manager.set_key(SecurityClass::S2Authenticated, s2_key(1));
        let group = manager.create_multicast_group(
            &[2.into(), 3.into(), 4.into()],
            SecurityClass::S2Authenticated,
        );

        let first = manager.get_multicast_key_and_iv(group).unwrap();
        let second = manager.get_multicast_key_and_iv(group).unwrap();

        assert_eq!(first.iv.len(), S2_NONCE_SIZE);
        assert_ne!(first.iv, second.iv);
    }

    #[test]
    fn reset_out_of_sync_mpans_only_removes_out_of_sync_entries() {
        let manager = create_manager();
        manager.store_peer_mpan(2.into(), 1, MPANTableEntry::OutOfSync);
        manager.store_peer_mpan(
            2.into(),
            2,
            MPANTableEntry::MPAN {
                current_mpan: [1; S2_MPAN_STATE_SIZE].into(),
            },
        );

        manager.reset_out_of_sync_mpans(2.into());

        assert_eq!(manager.get_peer_mpan(2.into(), 1), None);
        assert!(matches!(
            manager.get_peer_mpan(2.into(), 2),
            Some(MPANTableEntry::MPAN { .. })
        ));
    }

    #[test]
    fn create_multicast_group_rejects_s0_security_class() {
        let manager = create_manager();

        let result = std::panic::catch_unwind(|| {
            manager.create_multicast_group(&[2.into(), 3.into()], SecurityClass::S0Legacy)
        });

        assert!(result.is_err());
    }
}
