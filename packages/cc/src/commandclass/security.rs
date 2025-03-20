use crate::prelude::*;
use bytes::{Bytes, BytesMut};
use proc_macros::{CCValues, TryFromRepr};
use typed_builder::TypedBuilder;
use ux::{u2, u4};
use zwave_core::parse::bytes::rest;
use zwave_core::prelude::*;
use zwave_core::security::{compute_mac, decrypt_aes_ofb, S0Nonce, MAC_SIZE, S0_HALF_NONCE_SIZE};
use zwave_core::serialize::{self, DEFAULT_CAPACITY};
use zwave_core::{
    parse::{
        bits::{self, bool},
        bytes::{be_u8, complete::take},
        fail_validation, validate,
    },
    security::encrypt_aes_ofb,
};

use super::{CCSequence, CCSession, IntoCCSequence};

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
    fn parse(i: &mut Bytes, _ctx: CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
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
    fn parse(i: &mut Bytes, _ctx: CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
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

#[derive(Debug, Clone, PartialEq)]
enum SecurityCCCommandEncapsulationState {
    Complete {
        encapsulated: Box<CC>,
    },
    Partial {
        sequenced: bool,
        sequence_counter: u4,
        second_frame: bool,
        cc_slice: Bytes,

        // These are only needed for transmitting
        nonce: Option<S0Nonce>,
    },
}

#[derive(Debug, Clone, PartialEq, CCValues)]
pub struct SecurityCCCommandEncapsulation {
    state: SecurityCCCommandEncapsulationState,
}

impl SecurityCCCommandEncapsulation {
    pub fn new(encapsulated: CC) -> Self {
        Self {
            state: SecurityCCCommandEncapsulationState::Complete {
                encapsulated: Box::new(encapsulated),
            },
        }
    }

    // pub fn set_nonce(&mut self, new_nonce: S0Nonce) {
    //     match &mut self.state {
    //         SecurityCCCommandEncapsulationState::Partial { ref mut nonce, .. } => {
    //             nonce.replace(new_nonce);
    //         }
    //         _ => panic!("Cannot set nonce on complete SecurityCCCommandEncapsulation"),
    //     }
    // }

    // pub fn nonce(&self) -> Option<&S0Nonce> {
    //     match &self.state {
    //         SecurityCCCommandEncapsulationState::Partial { nonce, .. } => nonce.as_ref(),
    //         _ => None,
    //     }
    // }

    // pub fn set_auth_key(&mut self, new_auth_key: Vec<u8>) {
    //     match &mut self.state {
    //         SecurityCCCommandEncapsulationState::Partial {
    //             ref mut auth_key, ..
    //         } => {
    //             auth_key.replace(new_auth_key);
    //         }
    //         _ => panic!("Cannot set auth key on complete SecurityCCCommandEncapsulation"),
    //     }
    // }

    // pub fn set_enc_key(&mut self, new_enc_key: Vec<u8>) {
    //     match &mut self.state {
    //         SecurityCCCommandEncapsulationState::Partial {
    //             ref mut enc_key, ..
    //         } => {
    //             enc_key.replace(new_enc_key);
    //         }
    //         _ => panic!("Cannot set encryption key on complete SecurityCCCommandEncapsulation"),
    //     }
    // }
}

impl CCBase for SecurityCCCommandEncapsulation {
    fn expects_response(&self) -> bool {
        // The encapsulated CC decides whether a response is expected
        match &self.state {
            SecurityCCCommandEncapsulationState::Complete { encapsulated, .. } => {
                encapsulated.expects_response()
            }
            // Partially parsed commands cannot expect a response
            _ => false,
        }
    }

    fn test_response(&self, response: &CC) -> bool {
        // We can only compare two complete CCs, partials cannot expect a response
        let SecurityCCCommandEncapsulationState::Complete {
            encapsulated: sent, ..
        } = &self.state
        else {
            return false;
        };

        // We expect a SecurityCCCommandEncapsulation in response
        let CC::SecurityCCCommandEncapsulation(received_cc) = response else {
            return false;
        };

        // Extract the encapsulated CC from the received command
        let SecurityCCCommandEncapsulation {
            state:
                SecurityCCCommandEncapsulationState::Complete {
                    encapsulated: received,
                    ..
                },
        } = received_cc
        else {
            return false;
        };

        // The encapsulated CC decides whether the response is the expected one
        sent.test_response(received)
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
    fn parse(i: &mut Bytes, mut ctx: CCParsingContext) -> zwave_core::parse::ParseResult<Self> {
        let source_node_id = ctx.source_node_id;
        let own_node_id = ctx.own_node_id;

        let Some(sec_man) = ctx.security_manager.as_mut() else {
            return fail_validation(
                "Secure commands (S0) can only be decoded when the network key is set",
            );
        };

        // To parse this, we need at least:
        //   HALF_NONCE_SIZE bytes iv
        // + 1 byte frame control (encrypted)
        // + at least 1 CC byte (encrypted)
        // + 1 byte nonce id
        // + 8 bytes auth code

        let min_length = S0_HALF_NONCE_SIZE + 1 + 1 + 1 + 8;
        validate(i.len() >= min_length, "Incomplete payload")?;
        let ciphertext_len: usize = i.len() - S0_HALF_NONCE_SIZE - 1 - 8;

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

        let (_res76, second_frame, sequenced, sequence_counter) =
            bits::bits((u2::parse, bool, bool, u4::parse))
                .parse(&mut frame_control_and_plaintext)?;
        let cc_slice = rest(&mut frame_control_and_plaintext)?;

        Ok(Self {
            state: SecurityCCCommandEncapsulationState::Partial {
                sequenced,
                sequence_counter,
                second_frame,
                cc_slice,
                nonce: Some(nonce),
            },
        })
    }
}

impl SerializableWith<&CCEncodingContext> for SecurityCCCommandEncapsulation {
    fn serialize(&self, output: &mut BytesMut, ctx: &CCEncodingContext) {
        use serialize::{bits::bits, bytes::be_u8, bytes::slice, sequence::tuple};

        let SecurityCCCommandEncapsulationState::Partial {
            sequenced,
            sequence_counter,
            second_frame,
            cc_slice,
            nonce,
        } = &self.state
        else {
            panic!("Only a partial SecurityCCCommandEncapsulation can be serialized");
        };

        let sec_man = ctx
            .security_manager
            .as_ref()
            .expect("Secure commands (S0) can only be serialized when the network key is set");

        // FIXME: Typestate might avoid this. The nonce is technically the receiver's nonce
        let receiver_nonce = nonce
            .as_ref()
            .expect("Nonce must be set before serializing a SecurityCCCommandEncapsulation");

        let mut plaintext = BytesMut::with_capacity(cc_slice.len() + 1);
        bits(move |bo| {
            let reserved = u2::new(0);
            reserved.write(bo);
            second_frame.write(bo);
            sequenced.write(bo);
            sequence_counter.write(bo);
        })
        .serialize(&mut plaintext);
        slice(cc_slice).serialize(&mut plaintext);

        // Encrypt the plaintext
        let sender_nonce = S0Nonce::random();
        let iv = [sender_nonce.get().as_ref(), receiver_nonce.get().as_ref()].concat();
        let ciphertext = encrypt_aes_ofb(&plaintext, sec_man.enc_key(), &iv);

        // Authenticate the encrypted data
        let auth_data = S0AuthData {
            sender_nonce: sender_nonce.get(),
            receiver_nonce: receiver_nonce.get(),
            cc_command: SecurityCCCommand::CommandEncapsulation,
            sending_node_id: ctx.own_node_id,
            receiving_node_id: ctx.node_id,
            ciphertext: &ciphertext,
        };
        let auth_code = compute_mac(&auth_data.as_bytes(), sec_man.auth_key());

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
        match &self.state {
            SecurityCCCommandEncapsulationState::Complete { encapsulated } => LogPayloadDict::new()
                .with_nested(encapsulated.to_log_payload())
                .into(),
            SecurityCCCommandEncapsulationState::Partial {
                sequenced,
                sequence_counter,
                second_frame,
                cc_slice,
                ..
            } => {
                let mut ret = LogPayloadDict::new().with_entry("sequenced", *sequenced);
                if *sequenced {
                    ret = ret.with_entry("sequence counter", u8::from(*sequence_counter));
                    ret = ret.with_entry("second frame", *second_frame);
                }
                ret = ret.with_entry("payload", format!("0x{}", hex::encode(cc_slice)));

                ret.into()
            }
        }
    }
}

impl CCSession for SecurityCCCommandEncapsulation {
    // FIXME: Implement support for sequenced commands
    fn session_id(&self) -> Option<u32> {
        match self.state {
            SecurityCCCommandEncapsulationState::Complete { .. } => None,
            SecurityCCCommandEncapsulationState::Partial {
                sequence_counter, ..
            } => Some(sequence_counter.into()),
        }
    }

    fn is_session_complete(&self, other_ccs: &[CC]) -> bool {
        match self.state {
            SecurityCCCommandEncapsulationState::Complete { .. } => true,
            SecurityCCCommandEncapsulationState::Partial {
                sequenced,
                second_frame,
                ..
            } => !sequenced || second_frame,
        }
    }

    fn merge_session(&mut self, ctx: CCParsingContext, other_ccs: Vec<CC>) -> ParseResult<()> {
        // FIXME: Implement support for sequenced commands
        // For now, we assume the CC is complete, so we simply translate it to a complete one
        match self.state {
            SecurityCCCommandEncapsulationState::Complete { .. } => {
                // This should not happen, but we don't need to do anything with a complete CC
                Ok(())
            }

            SecurityCCCommandEncapsulationState::Partial {
                // sequenced,
                // sequence_counter,
                // second_frame,
                ref mut cc_slice,
                ..
            } => {
                // Concat all slices into one Bytes instance. In practice this will be either a single slice or two.
                let other_slices = other_ccs.iter().filter_map(|cc| {
                    let CC::SecurityCCCommandEncapsulation(cc) = cc else {
                        return None;
                    };
                    let SecurityCCCommandEncapsulationState::Partial { cc_slice, .. } = &cc.state
                    else {
                        return None;
                    };
                    Some(cc_slice)
                });
                let mut full_slice = BytesMut::with_capacity(DEFAULT_CAPACITY);
                for slice in other_slices {
                    full_slice.extend_from_slice(slice);
                }
                full_slice.extend_from_slice(cc_slice);
                let mut full_slice = full_slice.freeze();

                let encapsulated_raw = CCRaw::parse(&mut full_slice)?;
                let encapsulated = CC::try_from_raw(encapsulated_raw, ctx)?;
                self.state = SecurityCCCommandEncapsulationState::Complete {
                    encapsulated: Box::new(encapsulated),
                };
                Ok(())
            }
        }
    }
}

struct SecurityCCCommandEncapsulationSequence {
    address: CCAddress,
    encapsulated_cc: CC,
    nonce: Option<S0Nonce>,
    finished: bool,
}

impl CCSequence for SecurityCCCommandEncapsulationSequence {
    fn reset(&mut self) {
        self.nonce = None;
        self.finished = false;
    }

    fn next(&mut self, ctx: &CCEncodingContext) -> Option<WithAddress<CC>> {
        if self.finished {
            return None;
        }

        if self.nonce.is_none() {
            return Some(
                SecurityCCNonceGet::default()
                    .with_address(self.address.clone())
                    .into(),
            );
        }

        // let destination_node_id = match self.address.destination {
        //     Destination::Singlecast(node_id) => node_id,
        //     _ => panic!("Security S0 only supports singlecast!"),
        // };

        // let ctx = CCEncodingContext::builder()
        //     .own_node_id(own_node_id)
        //     .node_id(destination_node_id)
        //     .build();

        // FIXME: Handle splitting the CC into multiple frames
        let cc_slice = self.encapsulated_cc.as_raw(&ctx).as_bytes();

        let state = SecurityCCCommandEncapsulationState::Partial {
            sequenced: false,
            sequence_counter: u4::new(0),
            second_frame: false,
            cc_slice,
            nonce: self.nonce.take(),
        };

        self.finished = true;
        Some(
            SecurityCCCommandEncapsulation { state }
                .with_address(self.address.clone())
                .into(),
        )
    }

    fn is_finished(&self) -> bool {
        self.finished
    }

    fn handle_response(&mut self, response: &CC) {
        if let CC::SecurityCCNonceReport(report) = response {
            self.nonce = Some(report.nonce.clone());
        }
    }
}

impl IntoCCSequence for WithAddress<SecurityCCCommandEncapsulation> {
    fn into_cc_sequence(self) -> Box<dyn CCSequence + Sync + Send> {
        let (address, cc) = self.split();
        match cc.state {
            SecurityCCCommandEncapsulationState::Complete { encapsulated } => {
                Box::new(SecurityCCCommandEncapsulationSequence {
                    address,
                    encapsulated_cc: *encapsulated,
                    nonce: None,
                    finished: false,
                })
            }
            SecurityCCCommandEncapsulationState::Partial { .. } => {
                panic!("Cannot turn a partial SecurityCCCommandEncapsulation into a sequence")
            }
        }
    }
}
