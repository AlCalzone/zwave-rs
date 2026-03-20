use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use proc_macros::{CCValues, TryFromRepr};
use typed_builder::TypedBuilder;
use ux::{u6, u7};
use zwave_core::parse::{
    bits::{self, bool as parse_bool},
    bytes::complete::take,
    multi::fixed_length_bitmask_u8 as parse_fixed_length_bitmask_u8,
};
use zwave_core::prelude::*;
use zwave_core::security::{
    EntropyInput, KeysForNode, MPANTableEntry, S2_ENTROPY_INPUT_SIZE, S2_MPAN_STATE_SIZE,
    SPANTableEntry, SecurityKey, SecurityManager2, decrypt_aes_128_ccm, encrypt_aes_128_ccm,
};
use zwave_core::serialize::{self, DEFAULT_CAPACITY, Serializable};
use zwave_core::{
    parse::{bytes::be_u8, fail_validation, validate},
    security::{AesCcmNonce, AesKey, NETWORK_KEY_SIZE},
};

use super::{CCSequence, IntoCCSequence};

const SECURITY_S2_AUTH_TAG_LENGTH: usize = 8;
const MAX_DECRYPT_ATTEMPTS_SINGLECAST: usize = 5;
const MAX_DECRYPT_ATTEMPTS_MULTICAST: usize = 5;
const MAX_DECRYPT_ATTEMPTS_SC_FOLLOWUP: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromRepr)]
#[repr(u8)]
pub enum Security2CCCommand {
    NonceGet = 0x01,
    NonceReport = 0x02,
    MessageEncapsulation = 0x03,
    KEXGet = 0x04,
    KEXReport = 0x05,
    KEXSet = 0x06,
    KEXFail = 0x07,
    PublicKeyReport = 0x08,
    NetworkKeyGet = 0x09,
    NetworkKeyReport = 0x0a,
    NetworkKeyVerify = 0x0b,
    TransferEnd = 0x0c,
    CommandsSupportedGet = 0x0d,
    CommandsSupportedReport = 0x0e,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromRepr)]
#[repr(u8)]
pub enum KEXSchemes {
    KEXScheme1 = 0x01,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromRepr)]
#[repr(u8)]
pub enum ECDHProfiles {
    Curve25519 = 0x00,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromRepr)]
#[repr(u8)]
pub enum KEXFailType {
    NoKeyMatch = 0x01,
    NoSupportedScheme = 0x02,
    NoSupportedCurve = 0x03,
    Decrypt = 0x05,
    BootstrappingCanceled = 0x06,
    WrongSecurityLevel = 0x07,
    KeyNotGranted = 0x08,
    NoVerify = 0x09,
    DifferentKey = 0x0a,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromRepr)]
#[repr(u8)]
enum S2ExtensionType {
    Span = 0x01,
    Mpan = 0x02,
    Mgrp = 0x03,
    Mos = 0x04,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidateS2ExtensionResult {
    Ok,
    DiscardExtension,
    DiscardCommand,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Security2Extension {
    Span {
        critical: bool,
        more_to_follow: bool,
        sender_ei: EntropyInput,
    },
    Mpan {
        critical: bool,
        more_to_follow: bool,
        group_id: u8,
        inner_mpan_state: zwave_core::security::MpanState,
    },
    Mgrp {
        critical: bool,
        more_to_follow: bool,
        group_id: u8,
    },
    Mos {
        critical: bool,
        more_to_follow: bool,
    },
    Unknown {
        ty: u8,
        critical: bool,
        more_to_follow: bool,
        payload: Bytes,
    },
}

impl Security2Extension {
    pub fn span(sender_ei: impl Into<EntropyInput>) -> Self {
        Self::Span {
            critical: true,
            more_to_follow: false,
            sender_ei: sender_ei.into(),
        }
    }

    pub fn mpan(
        group_id: u8,
        inner_mpan_state: impl Into<zwave_core::security::MpanState>,
    ) -> Self {
        Self::Mpan {
            critical: true,
            more_to_follow: false,
            group_id,
            inner_mpan_state: inner_mpan_state.into(),
        }
    }

    pub fn mgrp(group_id: u8) -> Self {
        Self::Mgrp {
            critical: true,
            more_to_follow: false,
            group_id,
        }
    }

    pub fn mos() -> Self {
        Self::Mos {
            critical: false,
            more_to_follow: false,
        }
    }

    fn with_more_to_follow(&self, more_to_follow: bool) -> Self {
        match self {
            Self::Span {
                critical,
                sender_ei,
                ..
            } => Self::Span {
                critical: *critical,
                more_to_follow,
                sender_ei: *sender_ei,
            },
            Self::Mpan {
                critical,
                group_id,
                inner_mpan_state,
                ..
            } => Self::Mpan {
                critical: *critical,
                more_to_follow,
                group_id: *group_id,
                inner_mpan_state: inner_mpan_state.clone(),
            },
            Self::Mgrp {
                critical, group_id, ..
            } => Self::Mgrp {
                critical: *critical,
                more_to_follow,
                group_id: *group_id,
            },
            Self::Mos { critical, .. } => Self::Mos {
                critical: *critical,
                more_to_follow,
            },
            Self::Unknown {
                ty,
                critical,
                payload,
                ..
            } => Self::Unknown {
                ty: *ty,
                critical: *critical,
                more_to_follow,
                payload: payload.clone(),
            },
        }
    }

    fn parse(data: &[u8]) -> ParseResult<Self> {
        validate(data.len() >= 2, "Incomplete S2 extension")?;
        let total_length = data[0] as usize;
        validate(total_length >= 2, "Invalid S2 extension length")?;
        validate(
            data.len() >= total_length,
            "Incomplete S2 extension payload",
        )?;

        let mut header = Bytes::copy_from_slice(&data[1..2]);
        let (more_to_follow, critical, ty) =
            bits::bits((parse_bool, parse_bool, u6::parse)).parse(&mut header)?;
        let ty = u8::from(ty);
        let payload = &data[2..total_length];

        Ok(match S2ExtensionType::try_from(ty) {
            Ok(S2ExtensionType::Span) => {
                validate(
                    payload.len() == S2_ENTROPY_INPUT_SIZE,
                    "Invalid SPAN extension length",
                )?;
                Self::Span {
                    critical,
                    more_to_follow,
                    sender_ei: payload.into(),
                }
            }
            Ok(S2ExtensionType::Mpan) => {
                validate(
                    payload.len() == 1 + S2_MPAN_STATE_SIZE,
                    "Invalid MPAN extension length",
                )?;
                Self::Mpan {
                    critical,
                    more_to_follow,
                    group_id: payload[0],
                    inner_mpan_state: payload[1..].into(),
                }
            }
            Ok(S2ExtensionType::Mgrp) => {
                validate(payload.len() == 1, "Invalid MGRP extension length")?;
                Self::Mgrp {
                    critical,
                    more_to_follow,
                    group_id: payload[0],
                }
            }
            Ok(S2ExtensionType::Mos) => {
                validate(payload.is_empty(), "Invalid MOS extension length")?;
                Self::Mos {
                    critical,
                    more_to_follow,
                }
            }
            Err(_) => Self::Unknown {
                ty,
                critical,
                more_to_follow,
                payload: Bytes::copy_from_slice(payload),
            },
        })
    }

    fn validate(&self, was_encrypted: bool) -> ValidateS2ExtensionResult {
        if self.is_unknown_critical() {
            return ValidateS2ExtensionResult::DiscardCommand;
        }

        match self {
            Self::Mpan { .. } if !was_encrypted => ValidateS2ExtensionResult::DiscardExtension,
            Self::Span { .. } | Self::Mgrp { .. } | Self::Mos { .. } if was_encrypted => {
                ValidateS2ExtensionResult::DiscardExtension
            }
            _ => ValidateS2ExtensionResult::Ok,
        }
    }

    fn type_id(&self) -> u8 {
        match self {
            Self::Span { .. } => S2ExtensionType::Span as u8,
            Self::Mpan { .. } => S2ExtensionType::Mpan as u8,
            Self::Mgrp { .. } => S2ExtensionType::Mgrp as u8,
            Self::Mos { .. } => S2ExtensionType::Mos as u8,
            Self::Unknown { ty, .. } => *ty,
        }
    }

    fn critical(&self) -> bool {
        match self {
            Self::Span { critical, .. }
            | Self::Mpan { critical, .. }
            | Self::Mgrp { critical, .. }
            | Self::Mos { critical, .. }
            | Self::Unknown { critical, .. } => *critical,
        }
    }

    fn more_to_follow(&self) -> bool {
        match self {
            Self::Span { more_to_follow, .. }
            | Self::Mpan { more_to_follow, .. }
            | Self::Mgrp { more_to_follow, .. }
            | Self::Mos { more_to_follow, .. }
            | Self::Unknown { more_to_follow, .. } => *more_to_follow,
        }
    }

    fn is_unknown_critical(&self) -> bool {
        matches!(self, Self::Unknown { critical: true, .. })
    }

    fn is_encrypted(&self) -> bool {
        matches!(self, Self::Mpan { .. })
    }

    fn expected_length_for_header(header: &[u8]) -> Option<usize> {
        if header.len() < 2 {
            return None;
        }
        match header[1] & 0b0011_1111 {
            x if x == S2ExtensionType::Span as u8 => Some(18),
            x if x == S2ExtensionType::Mpan as u8 => Some(19),
            x if x == S2ExtensionType::Mgrp as u8 => Some(3),
            x if x == S2ExtensionType::Mos as u8 => Some(2),
            _ => None,
        }
    }

    fn payload(&self) -> Bytes {
        match self {
            Self::Span { sender_ei, .. } => Bytes::copy_from_slice(sender_ei.as_ref()),
            Self::Mpan {
                group_id,
                inner_mpan_state,
                ..
            } => {
                let mut ret = BytesMut::with_capacity(1 + S2_MPAN_STATE_SIZE);
                ret.extend_from_slice(&[*group_id]);
                ret.extend_from_slice(inner_mpan_state.as_ref());
                ret.freeze()
            }
            Self::Mgrp { group_id, .. } => Bytes::copy_from_slice(&[*group_id]),
            Self::Mos { .. } => Bytes::new(),
            Self::Unknown { payload, .. } => payload.clone(),
        }
    }

}

impl Serializable for Security2Extension {
    fn serialize(&self, output: &mut BytesMut) {
        use serialize::{
            bits::bits,
            bytes::{be_u8, slice},
        };

        let payload = self.payload();
        let critical = self.critical();
        let more_to_follow = self.more_to_follow();
        let type_id = self.type_id() & 0b0011_1111;

        be_u8((2 + payload.len()) as u8).serialize(output);
        bits(move |bo| {
            more_to_follow.write(bo);
            critical.write(bo);
            u6::new(type_id).write(bo);
        })
        .serialize(output);
        slice(payload).serialize(output);
    }
}

fn parse_extensions(buffer: &[u8], was_encrypted: bool) -> (Vec<Security2Extension>, bool, usize) {
    let mut extensions = Vec::new();
    let mut must_discard_command = false;
    let mut offset = 0usize;

    loop {
        if buffer.len() < offset + 2 {
            // An S2 extension was expected, but the buffer is too short.
            must_discard_command = true;
            break;
        }

        let actual_length = buffer[offset] as usize;
        // The length field could be too large, which would cause part of the actual ciphertext
        // to be ignored. Try to avoid this for known extensions by checking the actual and
        // expected length.
        let expected_length =
            Security2Extension::expected_length_for_header(&buffer[offset..offset + 2]);
        // Parse the extension using the expected length if possible.
        let extension_length = expected_length.unwrap_or(actual_length);

        if extension_length < 2 {
            // An S2 extension was expected, but the length is too short.
            must_discard_command = true;
            break;
        } else if extension_length > buffer.len().saturating_sub(offset) {
            // The supposed length is longer than the space the extensions may occupy.
            must_discard_command = true;
            break;
        }

        let extension_data = &buffer[offset..offset + extension_length];
        offset += extension_length;

        let ext = match Security2Extension::parse(extension_data) {
            Ok(ext) => ext,
            Err(_) => {
                must_discard_command = true;
                break;
            }
        };

        match ext.validate(was_encrypted) {
            ValidateS2ExtensionResult::Ok => {
                if expected_length.is_none() || actual_length == extension_length {
                    extensions.push(ext.clone());
                } else {
                    // The extension length field does not match, ignore the extension.
                }
            }
            ValidateS2ExtensionResult::DiscardExtension => {
                // Do nothing.
            }
            ValidateS2ExtensionResult::DiscardCommand => {
                must_discard_command = true;
            }
        }

        // Check if that was the last extension.
        if !ext.more_to_follow() {
            break;
        }
    }

    (extensions, must_discard_command, offset)
}

/// Returns the Sender's Entropy Input if this command contains a SPAN extension.
fn get_sender_ei(extensions: &[Security2Extension]) -> Option<EntropyInput> {
    extensions.iter().find_map(|ext| match ext {
        Security2Extension::Span { sender_ei, .. } => Some(*sender_ei),
        _ => None,
    })
}

/// Returns the multicast group ID if this command contains an MGRP extension.
fn get_multicast_group_id(extensions: &[Security2Extension]) -> Option<u8> {
    extensions.iter().find_map(|ext| match ext {
        Security2Extension::Mgrp { group_id, .. } => Some(*group_id),
        _ => None,
    })
}

fn get_mpan_extension(
    extensions: &[Security2Extension],
) -> Option<(u8, zwave_core::security::MpanState)> {
    extensions.iter().find_map(|ext| match ext {
        Security2Extension::Mpan {
            group_id,
            inner_mpan_state,
            ..
        } => Some((*group_id, inner_mpan_state.clone())),
        _ => None,
    })
}

fn parse_s2_bitmask(i: &mut Bytes) -> ParseResult<Vec<u8>> {
    parse_fixed_length_bitmask_u8(i, 0, 1)
}

fn serialize_s2_bitmask<S: AsRef<[u8]>>(values: S, output: &mut BytesMut) {
    serialize::sequence::fixed_length_bitmask_u8(values, 0, 1).serialize(output);
}

fn security_class_from_bitmask_value(value: u8) -> ParseResult<SecurityClass> {
    match value {
        0 => Ok(SecurityClass::S2Unauthenticated),
        1 => Ok(SecurityClass::S2Authenticated),
        2 => Ok(SecurityClass::S2AccessControl),
        7 => Ok(SecurityClass::S0Legacy),
        _ => fail_validation("Unsupported security class"),
    }
}

fn parse_security_class_bitmask(i: &mut Bytes) -> ParseResult<SecurityClass> {
    let classes = parse_s2_bitmask(i)?;
    validate(classes.len() == 1, "Expected exactly one security class")?;
    security_class_from_bitmask_value(classes[0])
}

fn get_authentication_data(
    sending_node_id: NodeId,
    destination: u16,
    home_id: Id32,
    command_length: usize,
    unencrypted_payload: &[u8],
) -> Bytes {
    let sending_id: u16 = sending_node_id.into();
    let node_id_size = if sending_id > u8::MAX as u16 || destination > u8::MAX as u16 {
        2
    } else {
        1
    };

    let mut ret = BytesMut::with_capacity(node_id_size * 2 + 6 + unencrypted_payload.len());
    if node_id_size == 1 {
        ret.extend_from_slice(&[(sending_id & 0xff) as u8, (destination & 0xff) as u8]);
    } else {
        ret.extend_from_slice(&sending_id.to_be_bytes());
        ret.extend_from_slice(&destination.to_be_bytes());
    }
    ret.extend_from_slice(&u32::from(home_id).to_be_bytes());
    ret.extend_from_slice(&(command_length as u16).to_be_bytes());
    // This includes the sequence number and all unencrypted extensions.
    ret.extend_from_slice(unencrypted_payload);
    ret.freeze()
}

fn assert_security_rx(ctx: &CCParsingContext) -> ParseResult<&SecurityManager2> {
    ctx.security_manager2.as_ref().ok_or_else(|| {
        ParseError::validation_failure("Secure commands (S2) require a security manager")
    })
}

fn assert_security_tx(ctx: &CCEncodingContext) -> &SecurityManager2 {
    ctx.security_manager2
        .as_ref()
        .expect("Secure commands (S2) require a security manager")
}

/// Validates that a sequence number is not a duplicate and updates the SPAN table if it is
/// accepted. Returns the previous sequence number if there is one.
fn validate_sequence_number(
    security_manager: &SecurityManager2,
    source_node_id: NodeId,
    sequence_number: u8,
) -> ParseResult<Option<u8>> {
    validate(
        !security_manager.is_duplicate_singlecast(source_node_id, sequence_number),
        format!("Duplicate command (sequence number {sequence_number})"),
    )?;
    // Not a duplicate, store it.
    Ok(security_manager.store_sequence_number(source_node_id, sequence_number))
}

fn s2_key_to_security_class(key: SecurityKey) -> Option<SecurityClass> {
    match key {
        SecurityKey::Temporary => None,
        SecurityKey::Key(security_class) => Some(security_class),
    }
}

struct SinglecastDecryptContext<'a> {
    security_manager: &'a SecurityManager2,
    sending_node_id: NodeId,
    cur_sequence_number: u8,
    prev_sequence_number: Option<u8>,
    ciphertext: &'a [u8],
    auth_data: &'a [u8],
    auth_tag: &'a [u8; SECURITY_S2_AUTH_TAG_LENGTH],
    span_state: SPANTableEntry,
    extensions: &'a [Security2Extension],
}

struct MulticastDecryptContext<'a> {
    security_manager: &'a SecurityManager2,
    sending_node_id: NodeId,
    group_id: u8,
    ciphertext: &'a [u8],
    auth_data: &'a [u8],
    auth_tag: &'a [u8; SECURITY_S2_AUTH_TAG_LENGTH],
}

fn decrypt_singlecast(
    ctx: SinglecastDecryptContext<'_>,
) -> Option<(Vec<u8>, Option<SecurityClass>)> {
    fn decrypt_with_active_keys(
        ctx: &SinglecastDecryptContext<'_>,
        nonce: &AesCcmNonce,
    ) -> Option<(Vec<u8>, Option<SecurityClass>)> {
        let keys = ctx
            .security_manager
            .get_keys_for_node(ctx.sending_node_id)?;
        let plaintext = match &keys {
            KeysForNode::Network(keys) => decrypt_aes_128_ccm(
                &keys.key_ccm,
                nonce,
                ctx.ciphertext,
                ctx.auth_data,
                ctx.auth_tag,
            ),
            KeysForNode::Temporary(keys) => decrypt_aes_128_ccm(
                &keys.key_ccm,
                nonce,
                ctx.ciphertext,
                ctx.auth_data,
                ctx.auth_tag,
            ),
        }?;
        let security_class = match keys {
            KeysForNode::Network(_) => {
                match ctx.security_manager.get_span_state(ctx.sending_node_id) {
                    Some(SPANTableEntry::SPAN { key, .. }) => s2_key_to_security_class(key),
                    _ => None,
                }
            }
            KeysForNode::Temporary(_) => None,
        };
        Some((plaintext, security_class))
    }

    match ctx.span_state.clone() {
        SPANTableEntry::SPAN {
            key, current_span, ..
        } => {
            // There should be a shared SPAN between both parties. In practice, both sides may
            // send a command at roughly the same time and use the same SPAN for encryption.
            // To avoid a desync where both nodes try to resync simultaneously, accept commands
            // encrypted with the previous SPAN under very specific circumstances.
            if let Some(current_span) = current_span
                && current_span.expires > std::time::Instant::now()
                && ctx
                    .prev_sequence_number
                    .map(|prev| ctx.cur_sequence_number == prev.wrapping_add(1))
                    .unwrap_or(false)
            {
                if let Some((plaintext, _)) = decrypt_with_active_keys(&ctx, &current_span.nonce) {
                    // If we could decrypt this way, we're done.
                    return Some((plaintext, s2_key_to_security_class(key)));
                }
            }

            // This can only happen if the security class is known.
            let nonce = ctx
                .security_manager
                .next_nonce(ctx.sending_node_id, false)?;
            let (plaintext, _) = decrypt_with_active_keys(&ctx, &nonce)?;
            Some((plaintext, s2_key_to_security_class(key)))
        }
        SPANTableEntry::LocalEI { receiver_ei } => {
            // We've sent the other node our receiver EI and received its sender EI, meaning we can
            // now establish an SPAN.
            let original_state = Some(ctx.span_state.clone());

            // How we establish the SPAN depends on whether we know the security class of the
            // other node.
            if ctx
                .security_manager
                .get_temp_keys(ctx.sending_node_id)
                .is_some()
                && let Some(sender_ei) = get_sender_ei(ctx.extensions)
            {
                // We're currently bootstrapping the node, so it might be using a temporary key.
                ctx.security_manager.initialize_temp_span(
                    ctx.sending_node_id,
                    sender_ei,
                    receiver_ei,
                );
                if let Some(nonce) = ctx.security_manager.next_nonce(ctx.sending_node_id, false)
                    && let Some((plaintext, _)) = decrypt_with_active_keys(&ctx, &nonce)
                {
                    // Decryption with the temporary key worked.
                    return Some((plaintext, None));
                }
                // Reset the SPAN state and try with the recently granted security class.
                ctx.security_manager
                    .set_span_state(ctx.sending_node_id, original_state.clone());
            }

            let sender_ei = get_sender_ei(ctx.extensions)?;
            // When ending up here, one of two situations has occurred:
            // a) We've taken over an existing network and do not know the node's security class.
            // b) We know the security class, but we're about to establish a new SPAN. This may
            //    happen at a lower security class than the one the node normally uses, e.g. when
            //    querying securely supported CCs.
            // In both cases, try decoding with multiple security classes, starting from the
            // highest one. If this fails, restore the previous partial SPAN state.
            for security_class in ctx.security_manager.possible_s2_security_classes() {
                // Initialize an SPAN with that security class.
                ctx.security_manager.initialize_span(
                    ctx.sending_node_id,
                    security_class,
                    sender_ei,
                    receiver_ei,
                );
                if let Some(nonce) = ctx.security_manager.next_nonce(ctx.sending_node_id, false)
                    && let Some((plaintext, _)) = decrypt_with_active_keys(&ctx, &nonce)
                {
                    // It worked, return the result.
                    return Some((plaintext, Some(security_class)));
                }
                // Reset the SPAN state and try with the next security class.
                ctx.security_manager
                    .set_span_state(ctx.sending_node_id, original_state.clone());
            }
            None
        }
        SPANTableEntry::RemoteEI { .. } => None,
    }
}

fn decrypt_multicast(ctx: MulticastDecryptContext<'_>) -> Option<Vec<u8>> {
    let nonce = ctx
        .security_manager
        .next_peer_mpan(ctx.sending_node_id, ctx.group_id)?;
    let keys = match ctx
        .security_manager
        .get_keys_for_node(ctx.sending_node_id)?
    {
        KeysForNode::Network(keys) => keys,
        KeysForNode::Temporary(_) => return None,
    };
    // The security class is irrelevant when decrypting multicast commands.
    decrypt_aes_128_ccm(
        &keys.key_ccm,
        &nonce,
        ctx.ciphertext,
        ctx.auth_data,
        ctx.auth_tag,
    )
}

fn serialize_extensions(extensions: &[Security2Extension], output: &mut BytesMut) {
    for (index, extension) in extensions.iter().enumerate() {
        extension
            .with_more_to_follow(index + 1 < extensions.len())
            .serialize(output);
    }
}

fn destination_group_or_node_id(
    destination: &Destination,
    extensions: &[Security2Extension],
) -> u16 {
    match destination {
        Destination::Singlecast(node_id) => (*node_id).into(),
        Destination::Broadcast => get_multicast_group_id(extensions)
            .expect("Multicast/broadcast S2 requires an MGRP extension")
            as u16,
        Destination::Multicast(_) => get_multicast_group_id(extensions)
            .expect("Multicast S2 requires an MGRP extension")
            as u16,
    }
}

// TODO: A node sending this command must accept a delay up to
// <previous round-trip time to peer node> + 250 ms before receiving the Nonce Report.
#[derive(Default, Debug, Clone, PartialEq, CCValues)]
pub struct Security2CCNonceGet {
    pub sequence_number: Option<u8>,
}

impl CCBase for Security2CCNonceGet {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &CC) -> bool {
        matches!(response, CC::Security2CCNonceReport(_))
    }
}

impl CCId for Security2CCNonceGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Security2
    }

    fn cc_command(&self) -> Option<u8> {
        Some(Security2CCCommand::NonceGet as u8)
    }
}

impl CCParsable for Security2CCNonceGet {
    fn parse(i: &mut Bytes, ctx: CCParsingContext) -> ParseResult<Self> {
        let security_manager = assert_security_rx(&ctx)?;
        let sequence_number = be_u8(i)?;
        // Don't accept duplicate commands.
        validate_sequence_number(security_manager, ctx.source_node_id, sequence_number)?;
        Ok(Self {
            sequence_number: Some(sequence_number),
        })
    }
}

impl SerializableWith<&CCEncodingContext> for Security2CCNonceGet {
    fn serialize(&self, output: &mut BytesMut, ctx: &CCEncodingContext) {
        use serialize::bytes::be_u8;

        let security_manager = assert_security_tx(ctx);
        let sequence_number = self
            .sequence_number
            .unwrap_or_else(|| security_manager.next_sequence_number(ctx.node_id));
        be_u8(sequence_number).serialize(output);
    }
}

impl ToLogPayload for Security2CCNonceGet {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry(
                "sequence number",
                self.sequence_number
                    .map_or_else(|| "(not set)".to_string(), |v| v.to_string()),
            )
            .into()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
#[builder(field_defaults(default))]
pub struct Security2CCNonceReport {
    pub sequence_number: Option<u8>,
    pub mos: bool,
    pub sos: bool,
    #[builder(default, setter(strip_option, into))]
    pub receiver_ei: Option<EntropyInput>,
}

impl CCBase for Security2CCNonceReport {}

impl CCId for Security2CCNonceReport {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Security2
    }

    fn cc_command(&self) -> Option<u8> {
        Some(Security2CCCommand::NonceReport as u8)
    }
}

impl CCParsable for Security2CCNonceReport {
    fn parse(i: &mut Bytes, ctx: CCParsingContext) -> ParseResult<Self> {
        let security_manager = assert_security_rx(&ctx)?;
        let sequence_number = be_u8(i)?;
        // Don't accept duplicate commands.
        validate_sequence_number(security_manager, ctx.source_node_id, sequence_number)?;
        let (_reserved, mos, sos) = bits::bits((u6::parse, parse_bool, parse_bool)).parse(i)?;

        if sos {
            // If the SOS flag is set, the receiver EI field must be included in the command.
            let receiver_ei = take(S2_ENTROPY_INPUT_SIZE).parse(i)?;
            let receiver_ei: EntropyInput = receiver_ei.as_ref().into();
            // Store it so the next sent command can use it for encryption.
            security_manager.store_remote_ei(ctx.source_node_id, receiver_ei);
            Ok(Self {
                sequence_number: Some(sequence_number),
                mos,
                sos,
                receiver_ei: Some(receiver_ei),
            })
        } else if mos {
            Ok(Self {
                sequence_number: Some(sequence_number),
                mos,
                sos: false,
                receiver_ei: None,
            })
        } else {
            fail_validation("Either MOS or SOS must be set")
        }
    }
}

impl SerializableWith<&CCEncodingContext> for Security2CCNonceReport {
    fn serialize(&self, output: &mut BytesMut, ctx: &CCEncodingContext) {
        use serialize::{
            bits::bits,
            bytes::{be_u8, slice},
        };

        let security_manager = assert_security_tx(ctx);
        let sequence_number = self
            .sequence_number
            .unwrap_or_else(|| security_manager.next_sequence_number(ctx.node_id));

        be_u8(sequence_number).serialize(output);
        let mos = self.mos;
        let sos = self.sos;
        bits(move |bo| {
            u6::new(0).write(bo);
            mos.write(bo);
            sos.write(bo);
        })
        .serialize(output);
        if self.sos {
            let receiver_ei = self
                .receiver_ei
                .expect("SOS nonce reports require a receiver EI");
            slice(receiver_ei).serialize(output);
        }
    }
}

impl ToLogPayload for Security2CCNonceReport {
    fn to_log_payload(&self) -> LogPayload {
        let mut ret = LogPayloadDict::new()
            .with_entry(
                "sequence number",
                self.sequence_number
                    .map_or_else(|| "(not set)".to_string(), |v| v.to_string()),
            )
            .with_entry("SOS", self.sos)
            .with_entry("MOS", self.mos);

        if let Some(receiver_ei) = &self.receiver_ei {
            ret = ret.with_entry(
                "receiver entropy",
                format!("0x{}", hex::encode(receiver_ei)),
            );
        }

        ret.into()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
#[builder(field_defaults(default))]
pub struct Security2CCMessageEncapsulation {
    pub sequence_number: Option<u8>,
    #[builder(default, setter(strip_option))]
    pub security_class: Option<SecurityClass>,
    #[builder(default)]
    pub extensions: Vec<Security2Extension>,
    #[builder(default, setter(strip_option))]
    pub encapsulated: Option<Box<CC>>,
}

impl Security2CCMessageEncapsulation {
    pub fn new(encapsulated: CC) -> Self {
        Self {
            sequence_number: None,
            security_class: None,
            extensions: vec![],
            encapsulated: Some(Box::new(encapsulated)),
        }
    }
}

impl CCBase for Security2CCMessageEncapsulation {
    fn expects_response(&self) -> bool {
        self.encapsulated
            .as_ref()
            .map(|cc| cc.expects_response())
            .unwrap_or(false)
    }

    fn test_response(&self, response: &CC) -> bool {
        match response {
            CC::Security2CCMessageEncapsulation(received) => {
                match (&self.encapsulated, &received.encapsulated) {
                    (Some(sent), Some(received)) => sent.test_response(received),
                    _ => false,
                }
            }
            // An S2 encapsulated command may result in a NonceReport if the node could not
            // decrypt the message and wants to re-establish the SPAN.
            CC::Security2CCNonceReport(report) => report.sos && report.receiver_ei.is_some(),
            CC::Security2CCKEXFail(_) => matches!(
                self.encapsulated.as_deref(),
                Some(
                    CC::Security2CCKEXSet(_)
                        | CC::Security2CCKEXReport(_)
                        | CC::Security2CCNetworkKeyGet(_)
                        | CC::Security2CCNetworkKeyReport(_)
                        | CC::Security2CCNetworkKeyVerify(_)
                )
            ),
            _ => false,
        }
    }
}

impl CCId for Security2CCMessageEncapsulation {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Security2
    }

    fn cc_command(&self) -> Option<u8> {
        Some(Security2CCCommand::MessageEncapsulation as u8)
    }
}

impl CCParsable for Security2CCMessageEncapsulation {
    fn parse(i: &mut Bytes, ctx: CCParsingContext) -> ParseResult<Self> {
        let security_manager = assert_security_rx(&ctx)?;

        let payload = i.clone();
        validate(
            payload.len() >= 2 + SECURITY_S2_AUTH_TAG_LENGTH,
            "Incomplete S2 payload",
        )?;
        let mut header = payload.clone();
        let sequence_number = be_u8(&mut header)?;
        let (_reserved, has_encrypted_extensions, has_extensions) =
            bits::bits((u6::parse, parse_bool, parse_bool)).parse(&mut header)?;
        let payload = payload.as_ref();

        let frame_addressing = ctx.frame_addressing.unwrap_or(FrameAddressing::Singlecast);
        let mut offset = payload.len() - header.len();
        let mut extensions = Vec::new();
        let mut must_discard_command = false;

        if has_extensions {
            let (parsed, discard, bytes_read) = parse_extensions(
                &payload[offset..payload.len() - SECURITY_S2_AUTH_TAG_LENGTH],
                false,
            );
            extensions.extend(parsed);
            must_discard_command = discard;
            offset += bytes_read;
        }

        let multicast_group_id = get_multicast_group_id(&extensions);
        let is_multicast = matches!(
            frame_addressing,
            FrameAddressing::Multicast | FrameAddressing::Broadcast
        );

        if is_multicast && multicast_group_id.is_none() {
            return fail_validation("Multicast S2 frames require the MGRP extension");
        }

        // If a command is to be discarded before decryption, we still need to increment the SPAN
        // or MPAN state.
        if must_discard_command {
            if is_multicast {
                if let Some(group_id) = multicast_group_id {
                    let _ = security_manager.next_peer_mpan(ctx.source_node_id, group_id);
                }
            } else {
                let _ = security_manager.next_nonce(ctx.source_node_id, false);
            }
            return fail_validation("Invalid S2 extension");
        }

        let mut previous_sequence_number = None;
        let peer_mpan_state = if is_multicast {
            security_manager.get_peer_mpan(
                ctx.source_node_id,
                multicast_group_id.expect("checked above"),
            )
        } else {
            // Don't accept duplicate singlecast commands.
            previous_sequence_number = Some(validate_sequence_number(
                security_manager,
                ctx.source_node_id,
                sequence_number,
            )?);
            if multicast_group_id.is_none() {
                // When a node receives a singlecast message after a multicast group was marked out
                // of sync, it must forget about the group.
                security_manager.reset_out_of_sync_mpans(ctx.source_node_id);
            }
            None
        };

        validate(
            payload.len() >= offset + SECURITY_S2_AUTH_TAG_LENGTH,
            "Incomplete S2 ciphertext",
        )?;
        let unencrypted_payload = &payload[..offset];
        let ciphertext = &payload[offset..payload.len() - SECURITY_S2_AUTH_TAG_LENGTH];
        let auth_tag: [u8; SECURITY_S2_AUTH_TAG_LENGTH] = payload
            [payload.len() - SECURITY_S2_AUTH_TAG_LENGTH..]
            .try_into()
            .unwrap();
        let message_length = 2 + payload.len();
        let destination_id = if is_multicast {
            multicast_group_id.expect("checked above") as u16
        } else {
            ctx.own_node_id.into()
        };
        let auth_data = get_authentication_data(
            ctx.source_node_id,
            destination_id,
            ctx.home_id,
            message_length,
            unencrypted_payload,
        );

        // If the receiver is unable to authenticate a singlecast message with the current SPAN,
        // it should try one or more following SPAN values. Likewise, multicast MAY be retried
        // with subsequent MPAN values until decryption succeeds or the maximum number of
        // iterations is reached.
        let decrypt_attempts = if is_multicast {
            MAX_DECRYPT_ATTEMPTS_MULTICAST
        } else if multicast_group_id.is_some() {
            MAX_DECRYPT_ATTEMPTS_SC_FOLLOWUP
        } else {
            MAX_DECRYPT_ATTEMPTS_SINGLECAST
        };

        let mut plaintext = None;
        let mut security_class = None;

        for _ in 0..decrypt_attempts {
            if is_multicast {
                // For incoming multicast commands, make sure we have an MPAN.
                let Some(MPANTableEntry::MPAN { .. }) = peer_mpan_state else {
                    // If we don't, mark the MPAN as out of sync so we can respond accordingly on
                    // the singlecast follow-up.
                    security_manager.store_peer_mpan(
                        ctx.source_node_id,
                        multicast_group_id.expect("checked above"),
                        MPANTableEntry::OutOfSync,
                    );
                    return fail_validation("No MPAN available to decode multicast command");
                };

                plaintext = decrypt_multicast(MulticastDecryptContext {
                    security_manager,
                    sending_node_id: ctx.source_node_id,
                    group_id: multicast_group_id.expect("checked above"),
                    ciphertext,
                    auth_data: &auth_data,
                    auth_tag: &auth_tag,
                });
                if plaintext.is_some() {
                    break;
                }
            } else {
                // Decrypt payload and verify integrity.
                let Some(span_state) = security_manager.get_span_state(ctx.source_node_id) else {
                    // If we are not able to establish an SPAN yet, fail the decryption.
                    return fail_validation("No SPAN available to decode S2 command");
                };
                if matches!(span_state, SPANTableEntry::RemoteEI { .. }) {
                    // The specs are not very clear how to handle this case. For now, treat it the
                    // same as having no usable EI at all.
                    return fail_validation("No SPAN available to decode S2 command");
                }

                if let Some((decrypted, used_security_class)) =
                    decrypt_singlecast(SinglecastDecryptContext {
                        security_manager,
                        sending_node_id: ctx.source_node_id,
                        cur_sequence_number: sequence_number,
                        prev_sequence_number: previous_sequence_number.flatten(),
                        ciphertext,
                        auth_data: &auth_data,
                        auth_tag: &auth_tag,
                        span_state,
                        extensions: &extensions,
                    })
                {
                    plaintext = Some(decrypted);
                    security_class = used_security_class;
                    break;
                }
            }
        }

        let Some(plaintext) = plaintext else {
            if is_multicast {
                // Mark the MPAN as out of sync.
                security_manager.store_peer_mpan(
                    ctx.source_node_id,
                    multicast_group_id.expect("checked above"),
                    MPANTableEntry::OutOfSync,
                );
                return fail_validation("Failed to decrypt multicast S2 command");
            } else {
                return fail_validation("Failed to decrypt S2 command");
            }
        };

        if !is_multicast && multicast_group_id.is_some() {
            // After reception of a singlecast follow-up, the MPAN state must be increased.
            security_manager.try_increment_peer_mpan(
                ctx.source_node_id,
                multicast_group_id.expect("checked above"),
            );
        }

        let mut encrypted_extension_offset = 0usize;
        if has_encrypted_extensions {
            let (parsed, discard, bytes_read) = parse_extensions(&plaintext, true);
            extensions.extend(parsed);
            must_discard_command = discard;
            encrypted_extension_offset = bytes_read;
        }

        // Before we can continue, check if the command must be discarded.
        if must_discard_command {
            return fail_validation("Invalid S2 extension");
        }

        // The MPAN and MGRP extensions must not be sent together.
        if multicast_group_id.is_some() && get_mpan_extension(&extensions).is_some() {
            return fail_validation("Invalid extension combination");
        }

        if !is_multicast {
            if let Some((group_id, inner_mpan_state)) = get_mpan_extension(&extensions) {
                // If an MPAN extension was received, store the MPAN.
                security_manager.store_peer_mpan(
                    ctx.source_node_id,
                    group_id,
                    MPANTableEntry::MPAN {
                        current_mpan: inner_mpan_state,
                    },
                );
            }
        }

        // Not every S2 message includes an encapsulated CC.
        let decrypted_cc_bytes = &plaintext[encrypted_extension_offset..];
        let encapsulated = if decrypted_cc_bytes.is_empty() {
            None
        } else {
            // Make sure this contains a complete CC command and deserialize it.
            validate(decrypted_cc_bytes.len() >= 2, "Incomplete encapsulated CC")?;
            let mut cc_bytes = Bytes::copy_from_slice(decrypted_cc_bytes);
            let encapsulated_raw = CCRaw::parse(&mut cc_bytes)?;
            Some(Box::new(CC::try_from_raw(encapsulated_raw, ctx)?))
        };

        Ok(Self {
            sequence_number: Some(sequence_number),
            // Remember which security class was used to decrypt this message.
            security_class,
            extensions,
            encapsulated,
        })
    }
}

impl SerializableWith<&CCEncodingContext> for Security2CCMessageEncapsulation {
    fn serialize(&self, output: &mut BytesMut, ctx: &CCEncodingContext) {
        use serialize::{bits::bits, bytes::be_u8};

        let security_manager = assert_security_tx(ctx);

        let mut extensions = self.extensions.clone();
        let sequence_number = self.sequence_number.unwrap_or_else(|| match ctx.node_id {
            _ if matches!(ctx.node_id, NODE_ID_BROADCAST) => security_manager
                .next_multicast_sequence_number(
                    get_multicast_group_id(&extensions)
                        .expect("Broadcast S2 encapsulation requires the MGRP extension"),
                )
                .expect("Missing multicast group"),
            _ => security_manager.next_sequence_number(ctx.node_id),
        });

        if matches!(ctx.node_id, NODE_ID_BROADCAST) || matches!(ctx.node_id, NODE_ID_UNSPECIFIED) {
            let _ = sequence_number;
        }

        let destination_node_id = ctx.node_id;
        if !matches!(destination_node_id, NODE_ID_BROADCAST) {
            // Include the Sender EI in the command if we only have the receiver's EI.
            match security_manager.get_span_state(destination_node_id) {
                None | Some(SPANTableEntry::LocalEI { .. }) => {
                    // Can't do anything here if we don't have the receiver's EI.
                    panic!("Security S2 encapsulation requires a nonce exchange first");
                }
                Some(SPANTableEntry::RemoteEI { receiver_ei }) => {
                    // We have the receiver's EI, generate our input and send it over. With both,
                    // we can create an SPAN.
                    let sender_ei = security_manager.generate_nonce(None);
                    if security_manager
                        .get_temp_keys(destination_node_id)
                        .is_some()
                        && self.security_class.is_none()
                    {
                        // While bootstrapping a node, prefer the temporary key unless the command
                        // explicitly specifies a security class.
                        security_manager.initialize_temp_span(
                            destination_node_id,
                            sender_ei,
                            receiver_ei,
                        );
                    } else {
                        let security_class = self
                            .security_class
                            .expect("Security S2 encapsulation requires a security class");
                        security_manager.initialize_span(
                            destination_node_id,
                            security_class,
                            sender_ei,
                            receiver_ei,
                        );
                    }

                    // Add or update the SPAN extension.
                    let span_extension = Security2Extension::span(sender_ei);
                    if let Some(index) = extensions
                        .iter()
                        .position(|ext| matches!(ext, Security2Extension::Span { .. }))
                    {
                        extensions[index] = span_extension;
                    } else {
                        extensions.push(span_extension);
                    }
                }
                Some(SPANTableEntry::SPAN { .. }) => {}
            }
        }

        let unencrypted_extensions = extensions
            .iter()
            .filter(|extension| !extension.is_encrypted())
            .cloned()
            .collect::<Vec<_>>();
        let encrypted_extensions = extensions
            .iter()
            .filter(|extension| extension.is_encrypted())
            .cloned()
            .collect::<Vec<_>>();

        let mut unencrypted_payload = BytesMut::with_capacity(DEFAULT_CAPACITY);
        be_u8(sequence_number).serialize(&mut unencrypted_payload);
        let has_encrypted_extensions = !encrypted_extensions.is_empty();
        let has_extensions = !unencrypted_extensions.is_empty();
        bits(move |bo| {
            u6::new(0).write(bo);
            has_encrypted_extensions.write(bo);
            has_extensions.write(bo);
        })
        .serialize(&mut unencrypted_payload);
        serialize_extensions(&unencrypted_extensions, &mut unencrypted_payload);
        let unencrypted_payload = unencrypted_payload.freeze();

        let mut plaintext_payload = BytesMut::with_capacity(DEFAULT_CAPACITY);
        serialize_extensions(&encrypted_extensions, &mut plaintext_payload);
        if let Some(encapsulated) = &self.encapsulated {
            plaintext_payload.extend_from_slice(&encapsulated.as_raw(ctx).as_bytes());
        }
        let plaintext_payload = plaintext_payload.freeze();

        let destination_id =
            destination_group_or_node_id(&Destination::Singlecast(ctx.node_id), &extensions);
        let message_length =
            2 + unencrypted_payload.len() + plaintext_payload.len() + SECURITY_S2_AUTH_TAG_LENGTH;
        // Generate the authentication data for CCM encryption.
        let auth_data = get_authentication_data(
            ctx.own_node_id,
            destination_id,
            ctx.home_id,
            message_length,
            &unencrypted_payload,
        );

        let (key, iv): (AesKey, AesCcmNonce) = if matches!(ctx.node_id, NODE_ID_BROADCAST) {
            // Multicast:
            let group_id = get_multicast_group_id(&extensions)
                .expect("Multicast S2 encapsulation requires the MGRP extension");
            let key_and_iv = security_manager
                .get_multicast_key_and_iv(group_id)
                .expect("Missing multicast group state");
            (key_and_iv.key, key_and_iv.iv)
        } else {
            // Singlecast:
            // Generate a nonce for encryption, and remember it to attempt decryption of potential
            // in-flight messages from the target node.
            let iv = security_manager
                .next_nonce(ctx.node_id, true)
                .expect("Security S2 encapsulation requires an active SPAN");
            let key = if let Some(security_class) = self.security_class {
                // Prefer the overridden security class if it was given.
                security_manager
                    .get_keys_for_security_class(security_class)
                    .expect("Missing S2 network key")
                    .key_ccm
            } else {
                match security_manager
                    .get_keys_for_node(ctx.node_id)
                    .expect("Missing S2 keys for destination node")
                {
                    KeysForNode::Network(keys) => keys.key_ccm,
                    KeysForNode::Temporary(keys) => keys.key_ccm,
                }
            };
            (key, iv)
        };

        let encrypted = encrypt_aes_128_ccm(&key, &iv, &plaintext_payload, &auth_data);

        output.extend_from_slice(&unencrypted_payload);
        output.extend_from_slice(&encrypted.ciphertext);
        output.extend_from_slice(&encrypted.auth_tag);
    }
}

impl ToLogPayload for Security2CCMessageEncapsulation {
    fn to_log_payload(&self) -> LogPayload {
        let mut ret = LogPayloadDict::new().with_entry(
            "sequence number",
            self.sequence_number
                .map_or_else(|| "(not set)".to_string(), |v| v.to_string()),
        );

        if let Some(security_class) = self.security_class {
            ret = ret.with_entry("security class", format!("{security_class:?}"));
        }
        if !self.extensions.is_empty() {
            ret = ret.with_entry("extensions", format!("{:?}", self.extensions));
        }
        if let Some(encapsulated) = &self.encapsulated {
            ret = ret.with_nested(encapsulated.to_log_payload());
        }

        ret.into()
    }
}

struct Security2CCMessageEncapsulationSequence {
    address: CCAddress,
    command: Security2CCMessageEncapsulation,
    nonce_requested: bool,
    finished: bool,
}

impl CCSequence for Security2CCMessageEncapsulationSequence {
    fn reset(&mut self) {
        self.nonce_requested = false;
        self.finished = false;
    }

    fn next(&mut self, ctx: &CCEncodingContext) -> Option<WithAddress<CC>> {
        if self.finished {
            return None;
        }

        let destination_node_id = match self.address.destination {
            Destination::Singlecast(node_id) => node_id,
            Destination::Broadcast => NodeId::broadcast(),
            Destination::Multicast(_) => NodeId::broadcast(),
        };

        if let Destination::Singlecast(_) = self.address.destination {
            let security_manager = assert_security_tx(ctx);
            let need_nonce = matches!(
                security_manager.get_span_state(destination_node_id),
                None | Some(SPANTableEntry::LocalEI { .. })
            );
            if need_nonce && !self.nonce_requested {
                // Singlecast S2 requires a nonce exchange before the first encrypted command.
                self.nonce_requested = true;
                return Some(
                    Security2CCNonceGet::default()
                        .with_address(self.address.clone())
                        .into(),
                );
            }
        }

        self.finished = true;
        Some(
            self.command
                .clone()
                .with_address(self.address.clone())
                .into(),
        )
    }

    fn is_finished(&self) -> bool {
        self.finished
    }

    fn handle_response(&mut self, _response: &CC) {}
}

impl IntoCCSequence for WithAddress<Security2CCMessageEncapsulation> {
    fn into_cc_sequence(self) -> Box<dyn CCSequence + Sync + Send> {
        let (address, command) = self.split();
        Box::new(Security2CCMessageEncapsulationSequence {
            address,
            command,
            nonce_requested: false,
            finished: false,
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq, CCValues)]
pub struct Security2CCKEXGet {}

impl CCBase for Security2CCKEXGet {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &CC) -> bool {
        matches!(response, CC::Security2CCKEXReport(_))
    }
}

impl CCId for Security2CCKEXGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Security2
    }

    fn cc_command(&self) -> Option<u8> {
        Some(Security2CCCommand::KEXGet as u8)
    }
}

impl CCParsable for Security2CCKEXGet {
    fn parse(_i: &mut Bytes, _ctx: CCParsingContext) -> ParseResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl SerializableWith<&CCEncodingContext> for Security2CCKEXGet {
    fn serialize(&self, _output: &mut BytesMut, _ctx: &CCEncodingContext) {
        // No payload
    }
}

impl ToLogPayload for Security2CCKEXGet {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
pub struct Security2CCKEXReport {
    pub request_csa: bool,
    pub echo: bool,
    #[builder(default)]
    pub reserved: u8,
    pub supported_kex_schemes: Vec<KEXSchemes>,
    pub supported_ecdh_profiles: Vec<ECDHProfiles>,
    pub requested_keys: Vec<SecurityClass>,
}

impl CCBase for Security2CCKEXReport {}

impl CCId for Security2CCKEXReport {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Security2
    }

    fn cc_command(&self) -> Option<u8> {
        Some(Security2CCCommand::KEXReport as u8)
    }
}

impl CCParsable for Security2CCKEXReport {
    fn parse(i: &mut Bytes, _ctx: CCParsingContext) -> ParseResult<Self> {
        validate(i.len() >= 4, "Incomplete KEX report")?;
        let (reserved, request_csa, echo) =
            bits::bits((u6::parse, parse_bool, parse_bool)).parse(i)?;
        // Remember the reserved bits for the echo.
        let reserved = u8::from(reserved) << 2;
        // The bit mask starts at 0, but bit 0 is not used.
        let supported_kex_schemes = parse_s2_bitmask(i)?
            .into_iter()
            .filter_map(|scheme| KEXSchemes::try_from(scheme).ok())
            .collect();
        let supported_ecdh_profiles = parse_s2_bitmask(i)?
            .into_iter()
            .filter_map(|profile| ECDHProfiles::try_from(profile).ok())
            .collect();
        let requested_keys = parse_s2_bitmask(i)?
            .into_iter()
            .filter_map(|key| security_class_from_bitmask_value(key).ok())
            .collect();
        Ok(Self {
            request_csa,
            echo,
            reserved,
            supported_kex_schemes,
            supported_ecdh_profiles,
            requested_keys,
        })
    }
}

impl SerializableWith<&CCEncodingContext> for Security2CCKEXReport {
    fn serialize(&self, output: &mut BytesMut, _ctx: &CCEncodingContext) {
        use serialize::bits::bits;

        let reserved = u6::new(self.reserved >> 2);
        let request_csa = self.request_csa;
        let echo = self.echo;
        bits(move |bo| {
            reserved.write(bo);
            request_csa.write(bo);
            echo.write(bo);
        })
        .serialize(output);
        // The bit mask starts at 0, but bit 0 is not used.
        serialize_s2_bitmask(
            self.supported_kex_schemes
                .iter()
                .map(|scheme| *scheme as u8)
                .collect::<Vec<_>>(),
            output,
        );
        serialize_s2_bitmask(
            self.supported_ecdh_profiles
                .iter()
                .map(|profile| *profile as u8)
                .collect::<Vec<_>>(),
            output,
        );
        serialize_s2_bitmask(
            self.requested_keys
                .iter()
                .map(|security_class| *security_class as u8)
                .collect::<Vec<_>>(),
            output,
        );
    }
}

impl ToLogPayload for Security2CCKEXReport {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("echo", self.echo)
            .with_entry("CSA requested", self.request_csa)
            .with_entry(
                "supported schemes",
                format!("{:?}", self.supported_kex_schemes),
            )
            .with_entry(
                "supported ECDH profiles",
                format!("{:?}", self.supported_ecdh_profiles),
            )
            .with_entry("requested keys", format!("{:?}", self.requested_keys))
            .into()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
pub struct Security2CCKEXSet {
    pub permit_csa: bool,
    pub echo: bool,
    #[builder(default)]
    pub reserved: u8,
    pub selected_kex_scheme: KEXSchemes,
    pub selected_ecdh_profile: ECDHProfiles,
    pub granted_keys: Vec<SecurityClass>,
}

impl CCBase for Security2CCKEXSet {
    fn expects_response(&self) -> bool {
        self.echo
    }

    fn test_response(&self, response: &CC) -> bool {
        if !self.echo {
            return false;
        }

        matches!(
            response,
            CC::Security2CCKEXFail(_)
                | CC::Security2CCKEXReport(Security2CCKEXReport { echo: true, .. })
        )
    }
}

impl CCId for Security2CCKEXSet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Security2
    }

    fn cc_command(&self) -> Option<u8> {
        Some(Security2CCCommand::KEXSet as u8)
    }
}

impl CCParsable for Security2CCKEXSet {
    fn parse(i: &mut Bytes, _ctx: CCParsingContext) -> ParseResult<Self> {
        validate(i.len() >= 4, "Incomplete KEX set")?;
        let (reserved, permit_csa, echo) =
            bits::bits((u6::parse, parse_bool, parse_bool)).parse(i)?;
        let reserved = u8::from(reserved) << 2;

        // The bit mask starts at 0, but bit 0 is not used.
        let schemes = parse_s2_bitmask(i)?;
        validate(schemes.len() == 1, "KEX set must select exactly one scheme")?;
        let selected_kex_scheme = KEXSchemes::try_from(schemes[0])
            .map_err(|_| ParseError::validation_failure("Unsupported KEX scheme"))?;

        let profiles = parse_s2_bitmask(i)?;
        validate(
            profiles.len() == 1,
            "KEX set must select exactly one ECDH profile",
        )?;
        let selected_ecdh_profile = ECDHProfiles::try_from(profiles[0])
            .map_err(|_| ParseError::validation_failure("Unsupported ECDH profile"))?;

        let granted_keys = parse_s2_bitmask(i)?
            .into_iter()
            .filter_map(|key| security_class_from_bitmask_value(key).ok())
            .collect();

        Ok(Self {
            permit_csa,
            echo,
            reserved,
            selected_kex_scheme,
            selected_ecdh_profile,
            granted_keys,
        })
    }
}

impl SerializableWith<&CCEncodingContext> for Security2CCKEXSet {
    fn serialize(&self, output: &mut BytesMut, _ctx: &CCEncodingContext) {
        use serialize::bits::bits;

        let reserved = u6::new(self.reserved >> 2);
        let permit_csa = self.permit_csa;
        let echo = self.echo;
        bits(move |bo| {
            reserved.write(bo);
            permit_csa.write(bo);
            echo.write(bo);
        })
        .serialize(output);
        // The bit mask starts at 0, but bit 0 is not used.
        serialize_s2_bitmask([self.selected_kex_scheme as u8], output);
        serialize_s2_bitmask([self.selected_ecdh_profile as u8], output);
        serialize_s2_bitmask(
            self.granted_keys
                .iter()
                .map(|security_class| *security_class as u8)
                .collect::<Vec<_>>(),
            output,
        );
    }
}

impl ToLogPayload for Security2CCKEXSet {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("echo", self.echo)
            .with_entry("CSA permitted", self.permit_csa)
            .with_entry("selected scheme", format!("{:?}", self.selected_kex_scheme))
            .with_entry(
                "selected ECDH profile",
                format!("{:?}", self.selected_ecdh_profile),
            )
            .with_entry("granted keys", format!("{:?}", self.granted_keys))
            .into()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
pub struct Security2CCKEXFail {
    pub fail_type: KEXFailType,
}

impl CCBase for Security2CCKEXFail {}

impl CCId for Security2CCKEXFail {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Security2
    }

    fn cc_command(&self) -> Option<u8> {
        Some(Security2CCCommand::KEXFail as u8)
    }
}

impl CCParsable for Security2CCKEXFail {
    fn parse(i: &mut Bytes, _ctx: CCParsingContext) -> ParseResult<Self> {
        let fail_type = be_u8(i)?;
        Ok(Self {
            fail_type: KEXFailType::try_from(fail_type)
                .map_err(|_| ParseError::validation_failure("Unsupported KEX fail type"))?,
        })
    }
}

impl SerializableWith<&CCEncodingContext> for Security2CCKEXFail {
    fn serialize(&self, output: &mut BytesMut, _ctx: &CCEncodingContext) {
        serialize::bytes::be_u8(self.fail_type as u8).serialize(output);
    }
}

impl ToLogPayload for Security2CCKEXFail {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("reason", format!("{:?}", self.fail_type))
            .into()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
pub struct Security2CCPublicKeyReport {
    pub including_node: bool,
    #[builder(setter(into))]
    pub public_key: Bytes,
}

impl CCBase for Security2CCPublicKeyReport {}

impl CCId for Security2CCPublicKeyReport {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Security2
    }

    fn cc_command(&self) -> Option<u8> {
        Some(Security2CCCommand::PublicKeyReport as u8)
    }
}

impl CCParsable for Security2CCPublicKeyReport {
    fn parse(i: &mut Bytes, _ctx: CCParsingContext) -> ParseResult<Self> {
        validate(i.len() >= 17, "Incomplete public key report")?;
        let (_reserved, including_node) = bits::bits((u7::parse, parse_bool)).parse(i)?;
        Ok(Self {
            including_node,
            public_key: i.split_to(i.len()),
        })
    }
}

impl SerializableWith<&CCEncodingContext> for Security2CCPublicKeyReport {
    fn serialize(&self, output: &mut BytesMut, _ctx: &CCEncodingContext) {
        use serialize::{bits::bits, bytes::slice};
        let including_node = self.including_node;
        bits(move |bo| {
            u7::new(0).write(bo);
            including_node.write(bo);
        })
        .serialize(output);
        slice(&self.public_key).serialize(output);
    }
}

impl ToLogPayload for Security2CCPublicKeyReport {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("is including node", self.including_node)
            .with_entry("public key", format!("0x{}", hex::encode(&self.public_key)))
            .into()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
pub struct Security2CCNetworkKeyGet {
    pub requested_key: SecurityClass,
}
// Do not model an expected response here. During key exchange the caller needs to distinguish
// between NetworkKeyReport and KEXFail.
impl CCBase for Security2CCNetworkKeyGet {}

impl CCId for Security2CCNetworkKeyGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Security2
    }

    fn cc_command(&self) -> Option<u8> {
        Some(Security2CCCommand::NetworkKeyGet as u8)
    }
}

impl CCParsable for Security2CCNetworkKeyGet {
    fn parse(i: &mut Bytes, _ctx: CCParsingContext) -> ParseResult<Self> {
        let requested_key = parse_security_class_bitmask(i)?;
        Ok(Self { requested_key })
    }
}

impl SerializableWith<&CCEncodingContext> for Security2CCNetworkKeyGet {
    fn serialize(&self, output: &mut BytesMut, _ctx: &CCEncodingContext) {
        serialize_s2_bitmask([self.requested_key as u8], output);
    }
}

impl ToLogPayload for Security2CCNetworkKeyGet {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("security class", format!("{:?}", self.requested_key))
            .into()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
pub struct Security2CCNetworkKeyReport {
    pub granted_key: SecurityClass,
    #[builder(setter(into))]
    pub network_key: Bytes,
}

impl CCBase for Security2CCNetworkKeyReport {}

impl CCId for Security2CCNetworkKeyReport {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Security2
    }

    fn cc_command(&self) -> Option<u8> {
        Some(Security2CCCommand::NetworkKeyReport as u8)
    }
}

impl CCParsable for Security2CCNetworkKeyReport {
    fn parse(i: &mut Bytes, _ctx: CCParsingContext) -> ParseResult<Self> {
        validate(i.len() >= 17, "Incomplete network key report")?;
        let granted_key = parse_security_class_bitmask(i)?;
        let network_key = take(NETWORK_KEY_SIZE).parse(i)?;
        Ok(Self {
            granted_key,
            network_key,
        })
    }
}

impl SerializableWith<&CCEncodingContext> for Security2CCNetworkKeyReport {
    fn serialize(&self, output: &mut BytesMut, _ctx: &CCEncodingContext) {
        use serialize::bytes::slice;
        serialize_s2_bitmask([self.granted_key as u8], output);
        slice(&self.network_key).serialize(output);
    }
}

impl ToLogPayload for Security2CCNetworkKeyReport {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("security class", format!("{:?}", self.granted_key))
            // Do not log the network key itself, so logs can be shared safely.
            .into()
    }
}

#[derive(Default, Debug, Clone, PartialEq, CCValues)]
pub struct Security2CCNetworkKeyVerify {}

impl CCBase for Security2CCNetworkKeyVerify {}

impl CCId for Security2CCNetworkKeyVerify {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Security2
    }

    fn cc_command(&self) -> Option<u8> {
        Some(Security2CCCommand::NetworkKeyVerify as u8)
    }
}

impl CCParsable for Security2CCNetworkKeyVerify {
    fn parse(_i: &mut Bytes, _ctx: CCParsingContext) -> ParseResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl SerializableWith<&CCEncodingContext> for Security2CCNetworkKeyVerify {
    fn serialize(&self, _output: &mut BytesMut, _ctx: &CCEncodingContext) {
        // No payload
    }
}

impl ToLogPayload for Security2CCNetworkKeyVerify {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
pub struct Security2CCTransferEnd {
    pub key_verified: bool,
    pub key_request_complete: bool,
}

impl CCBase for Security2CCTransferEnd {}

impl CCId for Security2CCTransferEnd {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Security2
    }

    fn cc_command(&self) -> Option<u8> {
        Some(Security2CCCommand::TransferEnd as u8)
    }
}

impl CCParsable for Security2CCTransferEnd {
    fn parse(i: &mut Bytes, _ctx: CCParsingContext) -> ParseResult<Self> {
        let (_reserved, key_verified, key_request_complete) =
            bits::bits((u6::parse, parse_bool, parse_bool)).parse(i)?;
        Ok(Self {
            key_verified,
            key_request_complete,
        })
    }
}

impl SerializableWith<&CCEncodingContext> for Security2CCTransferEnd {
    fn serialize(&self, output: &mut BytesMut, _ctx: &CCEncodingContext) {
        use serialize::bits::bits;

        let key_verified = self.key_verified;
        let key_request_complete = self.key_request_complete;
        bits(move |bo| {
            u6::new(0).write(bo);
            key_verified.write(bo);
            key_request_complete.write(bo);
        })
        .serialize(output);
    }
}

impl ToLogPayload for Security2CCTransferEnd {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("key verified", self.key_verified)
            .with_entry("request complete", self.key_request_complete)
            .into()
    }
}

#[derive(Default, Debug, Clone, PartialEq, CCValues)]
pub struct Security2CCCommandsSupportedGet {}

impl CCBase for Security2CCCommandsSupportedGet {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &CC) -> bool {
        matches!(response, CC::Security2CCCommandsSupportedReport(_))
    }
}

impl CCId for Security2CCCommandsSupportedGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Security2
    }

    fn cc_command(&self) -> Option<u8> {
        Some(Security2CCCommand::CommandsSupportedGet as u8)
    }
}

impl CCParsable for Security2CCCommandsSupportedGet {
    fn parse(_i: &mut Bytes, _ctx: CCParsingContext) -> ParseResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl SerializableWith<&CCEncodingContext> for Security2CCCommandsSupportedGet {
    fn serialize(&self, _output: &mut BytesMut, _ctx: &CCEncodingContext) {
        // No payload
    }
}

impl ToLogPayload for Security2CCCommandsSupportedGet {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
pub struct Security2CCCommandsSupportedReport {
    pub supported_ccs: Vec<CommandClasses>,
}

impl CCBase for Security2CCCommandsSupportedReport {}

impl CCId for Security2CCCommandsSupportedReport {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Security2
    }

    fn cc_command(&self) -> Option<u8> {
        Some(Security2CCCommand::CommandsSupportedReport as u8)
    }
}

impl CCParsable for Security2CCCommandsSupportedReport {
    fn parse(i: &mut Bytes, _ctx: CCParsingContext) -> ParseResult<Self> {
        // SDS13783: A sending node MAY terminate the list of supported command classes with the
        // COMMAND_CLASS_MARK identifier. A receiving node MUST stop parsing the list if it
        // detects COMMAND_CLASS_MARK in a Security 2 Commands Supported Report.
        let supported_ccs =
            zwave_core::parse::multi::fixed_length_cc_list_only_supported(i, i.len())?;

        Ok(Self { supported_ccs })
    }
}

impl SerializableWith<&CCEncodingContext> for Security2CCCommandsSupportedReport {
    fn serialize(&self, output: &mut BytesMut, _ctx: &CCEncodingContext) {
        for cc in &self.supported_ccs {
            cc.serialize(output);
        }
    }
}

impl ToLogPayload for Security2CCCommandsSupportedReport {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("supported CCs", format!("{:?}", self.supported_ccs))
            .into()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Arc;
    use zwave_core::security::{SecurityManager2, SecurityManager2Storage};

    fn create_manager() -> SecurityManager2 {
        SecurityManager2::new(Arc::new(SecurityManager2Storage::new()))
    }

    #[test]
    fn span_extension_roundtrip() {
        let extension = Security2Extension::span([0x11; S2_ENTROPY_INPUT_SIZE]);

        let mut serialized = BytesMut::new();
        extension.serialize(&mut serialized);
        let parsed = Security2Extension::parse(serialized.as_ref()).unwrap();

        assert_eq!(parsed, extension);
    }

    #[test]
    fn nonce_report_parse_stores_remote_ei() {
        let security_manager = create_manager();
        let source_node_id = NodeId::new(2u8);
        let receiver_ei = [0x22; S2_ENTROPY_INPUT_SIZE];
        let mut payload = Bytes::from(
            [[0x01, 0x01].as_slice(), receiver_ei.as_slice()]
                .concat()
                .to_vec(),
        );
        let ctx = CCParsingContext::builder()
            .home_id(0u32.into())
            .source_node_id(source_node_id)
            .own_node_id(NodeId::new(1u8))
            .security_manager2(security_manager.clone())
            .build();

        let parsed = Security2CCNonceReport::parse(&mut payload, ctx).unwrap();

        assert!(parsed.sos);
        assert_eq!(parsed.receiver_ei, Some(receiver_ei.into()));
        assert!(matches!(
            security_manager.get_span_state(source_node_id),
            Some(SPANTableEntry::RemoteEI { receiver_ei: stored }) if stored == receiver_ei.into()
        ));
    }

    #[test]
    fn commands_supported_report_roundtrip() {
        let report = Security2CCCommandsSupportedReport::builder()
            .supported_ccs(vec![
                CommandClasses::Basic,
                CommandClasses::Version,
                CommandClasses::Security2,
            ])
            .build();
        let ctx = CCEncodingContext::builder()
            .home_id(0u32.into())
            .own_node_id(NodeId::new(1u8))
            .node_id(NodeId::new(2u8))
            .build();
        let mut serialized = BytesMut::new();
        report.serialize(&mut serialized, &ctx);

        let mut payload = serialized.freeze();
        let parsed = Security2CCCommandsSupportedReport::parse(
            &mut payload,
            CCParsingContext::builder()
                .home_id(0u32.into())
                .source_node_id(NodeId::new(2u8))
                .own_node_id(NodeId::new(1u8))
                .build(),
        )
        .unwrap();

        assert_eq!(parsed.supported_ccs, report.supported_ccs);
    }
}
