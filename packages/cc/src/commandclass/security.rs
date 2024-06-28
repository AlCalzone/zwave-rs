use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use proc_macros::{CCValues, TryFromRepr};
use typed_builder::TypedBuilder;
use ux::{u2, u4};
use zwave_core::prelude::*;
use zwave_core::security::{compute_mac, decrypt_aes_ofb, S0Nonce, MAC_SIZE, S0_HALF_NONCE_SIZE};
use zwave_core::serialize;
use zwave_core::{
    parse::{
        bits::{self, bool},
        bytes::{be_u8, complete::take},
        fail_validation, validate,
    },
    security::encrypt_aes_ofb,
};

struct S0AuthData<'a> {
    sender_nonce: &'a [u8],
    receiver_nonce: &'a [u8],
    cc_command: SecurityCCCommand,
    sending_node_id: NodeId,
    receiving_node_id: NodeId,
    ciphertext: &'a [u8],
}

impl Serializable for S0AuthData<'_> {
    fn serialize(&self, output: &mut BytesMut) {
        use serialize::bytes::{be_u8, slice};

        slice(&self.sender_nonce).serialize(output);
        slice(&self.receiver_nonce).serialize(output);
        be_u8(self.cc_command as u8).serialize(output);
        self.sending_node_id
            .serialize(output, NodeIdType::NodeId8Bit);
        self.receiving_node_id
            .serialize(output, NodeIdType::NodeId8Bit);
        be_u8(self.ciphertext.len() as u8).serialize(output);
        slice(&self.ciphertext).serialize(output);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromRepr)]
#[repr(u8)]
pub enum SecurityCCCommand {
    CommandsSupportedGet = 0x02,
    CommandsSupportedReport = 0x03,
    SchemeGet = 0x04,
    SchemeReport = 0x05,
    SchemeInherit = 0x08,
    NetworkKeySet = 0x06,
    NetworkKeyVerify = 0x07,
    NonceGet = 0x40,
    NonceReport = 0x80,
    CommandEncapsulation = 0x81,
    CommandEncapsulationNonceGet = 0xc1,
}

#[derive(Default, Debug, Clone, PartialEq, CCValues)]
pub struct SecurityCCNonceGet {}

impl CCBase for SecurityCCNonceGet {
    fn expects_response(&self) -> bool {
        true
    }

    fn test_response(&self, response: &CC) -> bool {
        matches!(response, CC::SecurityCCNonceReport(_))
    }
}

impl CCId for SecurityCCNonceGet {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Security
    }

    fn cc_command(&self) -> Option<u8> {
        Some(SecurityCCCommand::NonceGet as _)
    }
}

impl CCParsable for SecurityCCNonceGet {
    fn parse(i: &mut Bytes, _ctx: &mut CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        // No payload
        Ok(Self {})
    }
}

impl SerializableWith<&CCEncodingContext> for SecurityCCNonceGet {
    fn serialize(&self, _output: &mut BytesMut, ctx: &CCEncodingContext) {
        // No payload
    }
}

impl ToLogPayload for SecurityCCNonceGet {
    fn to_log_payload(&self) -> LogPayload {
        LogPayload::empty()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
pub struct SecurityCCNonceReport {
    pub nonce: S0Nonce,
}

impl SecurityCCNonceReport {}

impl CCBase for SecurityCCNonceReport {}

impl CCId for SecurityCCNonceReport {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Security
    }

    fn cc_command(&self) -> Option<u8> {
        Some(SecurityCCCommand::NonceReport as _)
    }
}

impl CCParsable for SecurityCCNonceReport {
    fn parse(i: &mut Bytes, _ctx: &mut CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        let nonce = take(S0_HALF_NONCE_SIZE).parse(i)?;
        let nonce = S0Nonce::new(nonce);
        Ok(Self { nonce })
    }
}

impl SerializableWith<&CCEncodingContext> for SecurityCCNonceReport {
    fn serialize(&self, output: &mut BytesMut, ctx: &CCEncodingContext) {
        use serialize::bytes::slice;
        slice(&self.nonce.get()).serialize(output);
    }
}

impl ToLogPayload for SecurityCCNonceReport {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_entry("nonce", format!("{}", &self.nonce))
            .into()
    }
}

#[derive(Debug, Clone, PartialEq, TypedBuilder, CCValues)]
pub struct SecurityCCCommandEncapsulation {
    // TODO: Consider typestate to distinguish between received, sent and partial commands
    encapsulated: Box<CC>,
    nonce: Option<S0Nonce>,
    enc_key: Option<Vec<u8>>,
    auth_key: Option<Vec<u8>>,
}

impl SecurityCCCommandEncapsulation {
    pub fn new(encapsulated: CC) -> Self {
        Self {
            encapsulated: Box::new(encapsulated),
            nonce: None,
            enc_key: None,
            auth_key: None,
        }
    }

    pub fn set_nonce(&mut self, nonce: S0Nonce) {
        self.nonce = Some(nonce);
    }

    pub fn nonce(&self) -> Option<&S0Nonce> {
        self.nonce.as_ref()
    }
}

impl CCBase for SecurityCCCommandEncapsulation {
    fn expects_response(&self) -> bool {
        // The encapsulated CC decides whether a response is expected
        self.encapsulated.expects_response()
    }

    fn test_response(&self, response: &CC) -> bool {
        // The encapsulated CC decides whether the response is the expected one
        let CC::SecurityCCCommandEncapsulation(SecurityCCCommandEncapsulation {
            encapsulated, ..
        }) = response
        else {
            return false;
        };
        self.encapsulated.test_response(encapsulated)
    }
}

impl CCId for SecurityCCCommandEncapsulation {
    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Security
    }

    fn cc_command(&self) -> Option<u8> {
        Some(SecurityCCCommand::CommandEncapsulation as _)
    }
}

impl CCParsable for SecurityCCCommandEncapsulation {
    fn parse(i: &mut Bytes, ctx: &mut CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        let source_node_id = ctx.source_node_id;
        let own_node_id = ctx.own_node_id;

        let Some(mut sec_man) = ctx.security_manager_mut() else {
            return fail_validation(
                "Secure commands (S0) can only be decoded when the network key is set",
            );
        };

        // To parse this, we need at least:
        //   HALF_NONCE_SIZE bytes iv
        // + 1 byte frame control
        // + at least 1 CC byte
        // + 1 byte nonce id
        // + 8 bytes auth code

        let min_length = S0_HALF_NONCE_SIZE + 1 + 1 + 1 + 8;
        validate(i.len() >= min_length, "Incomplete payload")?;
        let ciphertext_len: usize = i.len() - S0_HALF_NONCE_SIZE - 1 - 1 - 8;

        // Parse the CC fields
        let (sender_nonce, ciphertext, nonce_id, auth_code) = (
            take(S0_HALF_NONCE_SIZE),
            take(ciphertext_len),
            be_u8,
            take(MAC_SIZE),
        )
            .parse(i)?;

        // Retrieve the used nonce from the nonce store
        let Some(nonce) = sec_man.try_get_own_nonce(nonce_id) else {
            return fail_validation(format!(
                "Nonce {:#04x} expired, cannot decode security encapsulated command.",
                nonce_id
            ));
        };

        // Validate the encrypted data
        let auth_data = S0AuthData {
            sender_nonce: &sender_nonce,
            receiver_nonce: nonce.get(),
            cc_command: SecurityCCCommand::CommandEncapsulation,
            sending_node_id: source_node_id,
            receiving_node_id: own_node_id,
            ciphertext: &ciphertext,
        }
        .as_bytes();

        // Validate the encrypted data
        let expected_auth_code = compute_mac(&auth_data, sec_man.auth_key());
        validate(
            auth_code == expected_auth_code,
            "Command authentication failed",
        )?;

        // Decrypt the encapsulated CC
        let iv = [sender_nonce, nonce.get().clone()].concat();
        let mut frame_control_and_plaintext =
            Bytes::from(decrypt_aes_ofb(&ciphertext, sec_man.enc_key(), &iv));

        // We no longer need the security manager, but we need access to the context later
        drop(sec_man);

        let (_res76, second_frame, sequenced, sequence_counter) =
            bits::bits((u2::parse, bool, bool, u4::parse))
                .parse(&mut frame_control_and_plaintext)?;

        let encapsulated_raw = CCRaw::parse(&mut frame_control_and_plaintext)?;
        let encapsulated = CC::try_from_raw(encapsulated_raw, ctx)?;

        // FIXME: support sequenced commands

        Ok(Self {
            encapsulated: Box::new(encapsulated),
            nonce: Some(nonce),
            enc_key: None,
            auth_key: None,
        })
    }
}

impl SerializableWith<&CCEncodingContext> for SecurityCCCommandEncapsulation {
    fn serialize(&self, output: &mut BytesMut, ctx: &CCEncodingContext) {
        use serialize::{bytes::be_u8, bytes::slice, sequence::tuple};

        // FIXME: Typestate might avoid this. The nonce is technically the receiver's nonce
        let receiver_nonce = self
            .nonce
            .as_ref()
            .expect("Nonce must be set before serializing a SecurityCCCommandEncapsulation");
        let enc_key = self.enc_key.as_ref().expect(
            "Encryption key must be set before serializing a SecurityCCCommandEncapsulation",
        );
        let auth_key = self.auth_key.as_ref().expect(
            "Authentication key must be set before serializing a SecurityCCCommandEncapsulation",
        );

        let command = self.encapsulated.clone();
        let payload = command.as_raw(ctx).as_bytes();

        let plaintext = [
            &[0u8], // TODO: Frame control / sequenced frames
            &payload[..],
        ]
        .concat();

        // Encrypt the plaintext
        let sender_nonce = S0Nonce::random();
        let iv = [sender_nonce.get().as_ref(), receiver_nonce.get().as_ref()].concat();
        let ciphertext = encrypt_aes_ofb(&plaintext, enc_key, &iv);

        // Authenticate the encrypted data
        let auth_data = S0AuthData {
            sender_nonce: sender_nonce.get(),
            receiver_nonce: receiver_nonce.get(),
            cc_command: SecurityCCCommand::CommandEncapsulation,
            sending_node_id: ctx.own_node_id,
            receiving_node_id: ctx.node_id,
            ciphertext: &ciphertext,
        };
        let auth_code = compute_mac(&auth_data.as_bytes(), auth_key);

        tuple((
            slice(sender_nonce.get()),
            slice(ciphertext),
            be_u8(receiver_nonce.id()),
            slice(auth_code),
        ))
        .serialize(output);
    }
}

impl ToLogPayload for SecurityCCCommandEncapsulation {
    fn to_log_payload(&self) -> LogPayload {
        LogPayloadDict::new()
            .with_nested(self.encapsulated.to_log_payload())
            .into()
    }
}
