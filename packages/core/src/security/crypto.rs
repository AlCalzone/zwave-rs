use aes::cipher::{
    BlockEncrypt, BlockEncryptMut, KeyInit, KeyIvInit, StreamCipher,
    block_padding::ZeroPadding,
    generic_array::{
        GenericArray,
        typenum::{U8, U13, U16},
    },
};
use ccm::AeadInPlace;
use getrandom::getrandom;
use std::ops::Deref;

type Aes128Ofb = ofb::Ofb<aes::Aes128>;
type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
pub type Aes128Ccm = ccm::Ccm<aes::Aes128, U8, U13>;

pub const BLOCK_SIZE: usize = 16;
pub type Block = [u8; BLOCK_SIZE];
pub const MAC_SIZE: usize = 8;
pub const AES_CCM_NONCE_SIZE: usize = 13;
pub const ENTROPY_INPUT_SIZE: usize = 16;
pub const ENTROPY_SIZE: usize = 32;
pub const PERSONALIZATION_STRING_SIZE: usize = 32;

macro_rules! fixed_bytes_type {
    ($name:ident, $size:expr, $display_name:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        pub struct $name([u8; $size]);

        impl $name {
            pub const ZERO: Self = Self([0; $size]);

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

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "0x{}", hex::encode(self.0))
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct AesKey([u8; 16]);

impl From<Block> for AesKey {
    fn from(value: [u8; 16]) -> Self {
        Self(value)
    }
}

impl From<AesKey> for GenericArray<u8, U16> {
    fn from(value: AesKey) -> Self {
        (*value).into()
    }
}

impl From<&AesKey> for GenericArray<u8, U16> {
    fn from(value: &AesKey) -> Self {
        (**value).into()
    }
}

impl From<&super::NetworkKey> for AesKey {
    fn from(value: &super::NetworkKey) -> Self {
        Self(**value)
    }
}

impl AsRef<[u8]> for AesKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for AesKey {
    type Target = [u8; 16];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct AesIV([u8; 16]);

impl AesIV {
    pub const ZERO: Self = Self([0; 16]);

    pub fn from_halves(left: &[u8; 8], right: &[u8; 8]) -> Self {
        let mut iv = [0; 16];
        iv[..8].copy_from_slice(left);
        iv[8..].copy_from_slice(right);
        Self(iv)
    }
}

impl From<[u8; 16]> for AesIV {
    fn from(value: [u8; 16]) -> Self {
        Self(value)
    }
}

impl From<AesIV> for GenericArray<u8, U16> {
    fn from(value: AesIV) -> Self {
        (*value).into()
    }
}

impl From<&AesIV> for GenericArray<u8, U16> {
    fn from(value: &AesIV) -> Self {
        (**value).into()
    }
}

impl AsRef<[u8]> for AesIV {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for AesIV {
    type Target = [u8; 16];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fixed_bytes_type!(AesCcmNonce, AES_CCM_NONCE_SIZE, "AES-CCM nonce");
fixed_bytes_type!(EntropyInput, ENTROPY_INPUT_SIZE, "Entropy input");
fixed_bytes_type!(Entropy, ENTROPY_SIZE, "Entropy");
fixed_bytes_type!(
    PersonalizationString,
    PERSONALIZATION_STRING_SIZE,
    "Personalization string"
);

impl From<AesCcmNonce> for GenericArray<u8, U13> {
    fn from(value: AesCcmNonce) -> Self {
        (*value).into()
    }
}

impl From<&AesCcmNonce> for GenericArray<u8, U13> {
    fn from(value: &AesCcmNonce) -> Self {
        (**value).into()
    }
}

impl Entropy {
    pub fn random() -> Self {
        let mut entropy = Self::ZERO;
        getrandom(entropy.as_mut()).unwrap_or_else(|_| panic!("Failed to generate random bytes"));
        entropy
    }

    pub fn from_halves(left: &[u8; ENTROPY_INPUT_SIZE], right: &[u8; ENTROPY_INPUT_SIZE]) -> Self {
        let mut entropy = Self::ZERO;
        entropy.as_mut()[..left.len()].copy_from_slice(left);
        entropy.as_mut()[left.len()..].copy_from_slice(right);
        entropy
    }
}

impl PersonalizationString {
    pub fn from_halves(left: &Block, right: &Block) -> Self {
        let mut personalization_string = Self::ZERO;
        personalization_string.as_mut()[..left.len()].copy_from_slice(left);
        personalization_string.as_mut()[left.len()..].copy_from_slice(right);
        personalization_string
    }
}

/// A zeroed block of size 16
const Z128: Block = [0; 16];
const R128: Block = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x87];

const CONSTANT_TE: [u8; 15] = [0x88; 15];
const CONSTANT_NK: [u8; 15] = [0x55; 15];
const CONSTANT_EI: [u8; 15] = [0x88; 15];
const CONSTANT_PRK: AesKey = AesKey([0x33; 16]);
const CONSTANT_NONCE: AesKey = AesKey([0x26; 16]);

// FIXME: Should this be here?
const SECURITY_S2_AUTH_TAG_LENGTH: usize = 8;

pub fn encrypt_aes_ecb(plaintext: &Block, key: &AesKey) -> Block {
    let cipher = aes::Aes128::new(&key.into());

    let mut block: GenericArray<u8, U16> = [0; 16].into();
    block.copy_from_slice(plaintext);

    cipher.encrypt_block(&mut block);

    let mut ret = Z128;
    ret.copy_from_slice(&block);
    ret
}

pub fn encrypt_aes_ofb(plaintext: &[u8], key: &AesKey, iv: &AesIV) -> Vec<u8> {
    let mut cipher = <Aes128Ofb as KeyIvInit>::new(&key.into(), &iv.into());

    let mut buf = plaintext.to_vec();
    cipher.apply_keystream(&mut buf);

    buf
}

pub fn decrypt_aes_ofb(ciphertext: &[u8], key: &AesKey, iv: &AesIV) -> Vec<u8> {
    let mut cipher = <Aes128Ofb as KeyIvInit>::new(&key.into(), &iv.into());

    let mut buf = ciphertext.to_vec();
    cipher.apply_keystream(&mut buf);

    buf
}

pub fn compute_mac(plaintext: &[u8], key: &AesKey) -> [u8; MAC_SIZE] {
    compute_mac_iv(plaintext, key, &AesIV::ZERO)
}

pub fn compute_mac_iv(plaintext: &[u8], key: &AesKey, iv: &AesIV) -> [u8; MAC_SIZE] {
    let cipher = Aes128CbcEnc::new(&key.into(), &iv.into());
    let buf = cipher.encrypt_padded_vec_mut::<ZeroPadding>(plaintext);
    // The MAC is the first 8 bytes of the last 16 byte block
    let mut mac = [0u8; MAC_SIZE];
    mac.copy_from_slice(&buf[buf.len() - 16..][..MAC_SIZE]);
    mac
}

// Decodes a DER-encoded x25519 key (PKCS#8 or SPKI)
pub fn decode_x25519_key_der(key: &[u8]) -> &[u8] {
    // TODO: Look at the helper functions in RustCrypto crates if there's a better way to do this
    // For now, we just take the last 32 bytes
    &key[key.len() - 32..]
}

const X25519_PKCS8_PREFIX: &[u8; 16] =
    b"\x30\x2e\x02\x01\x00\x30\x05\x06\x03\x2b\x65\x6e\x04\x22\x04\x20";

// Encodes an x25519 key from a raw buffer with DER/PKCS#8
pub fn encode_x25519_key_der_pkcs8(key: &[u8]) -> Vec<u8> {
    [&X25519_PKCS8_PREFIX[..], key].concat()
}

const X25519_SPKI_PREFIX: &[u8; 12] = b"\x30\x2a\x30\x05\x06\x03\x2b\x65\x6e\x03\x21\x00";

// Encodes an x25519 key from a raw buffer with DER/SPKI
pub fn encode_x25519_key_der_spki(key: &[u8]) -> Vec<u8> {
    // TODO: Look at the helper functions in RustCrypto crates if there's a better way to do this
    // For now, this seems easier
    [&X25519_SPKI_PREFIX[..], key].concat()
}

// Computes the byte-wise XOR of two arrays with the same length
pub fn xor_slices<const N: usize>(a: &[u8; N], b: &[u8; N]) -> [u8; N] {
    let mut ret = [0; N];
    for i in 0..N {
        ret[i] = a[i] ^ b[i];
    }
    ret
}

// Computes the byte-wise XOR of two arrays with the same length, mutating the first slice
pub fn xor_slice_mut<const N: usize>(a: &mut [u8; N], b: &[u8; N]) {
    a.iter_mut().zip(b.iter()).for_each(|(x, y)| *x ^= y);
}

// Creates a new array in MSB ordering by left-shifting it one bit
pub fn left_shift_1<const N: usize>(input: &[u8; N]) -> [u8; N] {
    let mut ret = [0; N];
    if N == 0 {
        return ret;
    }

    // TODO: Maybe use iterators here?
    for i in 0..N - 1 {
        ret[i] = (input[i] << 1) + if input[i + 1] & 0x80 != 0 { 1 } else { 0 };
    }
    ret[N - 1] = input[N - 1] << 1;

    ret
}

#[test]
fn test_left_shift_1() {
    assert_eq!(left_shift_1(&[0x00]), [0x00]);
    assert_eq!(left_shift_1(&[0x01]), [0x02]);
    assert_eq!(left_shift_1(&[0x80]), [0x00]);
    assert_eq!(left_shift_1(&[0x01, 0x00]), [0x02, 0x00]);
    assert_eq!(left_shift_1(&[0x01, 0x80]), [0x03, 0x00]);
    assert_eq!(left_shift_1(&[0x01, 0x40]), [0x02, 0x80]);
}

// Increments a multi-byte unsigned integer in big-endian order by 1
pub fn increment_slice_mut(buffer: &mut [u8]) {
    for i in (0..buffer.len()).rev() {
        buffer[i] = buffer[i].wrapping_add(1);
        if buffer[i] != 0x00 {
            break;
        }
    }
}

pub fn generate_aes128_cmac_subkeys(key: &AesKey) -> (Block, Block) {
    // NIST SP 800-38B, chapter 6.1
    let l = encrypt_aes_ecb(&Z128, key);
    let k1 = if l[0] & 0x80 == 0 {
        left_shift_1(&l)
    } else {
        xor_slices(&left_shift_1(&l), &R128)
    };
    let k2 = if k1[0] & 0x80 == 0 {
        left_shift_1(&k1)
    } else {
        xor_slices(&left_shift_1(&k1), &R128)
    };

    (k1, k2)
}

// Computes a message authentication code for Security S2 (as described in SDS13783)
pub fn compute_cmac(message: &[u8], key: &AesKey) -> Block {
    let remainder = message.len() % BLOCK_SIZE;
    let num_blocks = message.len() / BLOCK_SIZE + if remainder == 0 { 0 } else { 1 };

    let last_block = if num_blocks > 0 {
        &message[(num_blocks - 1) * BLOCK_SIZE..]
    } else {
        message
    };
    let last_block_len = last_block.len();
    let last_block_is_complete = !message.is_empty() && remainder == 0;
    let last_block: Block = if last_block_is_complete {
        last_block.try_into().unwrap()
    } else {
        let mut padded = Z128;
        padded[..last_block_len].copy_from_slice(last_block);
        padded[last_block_len] = 0x80;
        padded
    };

    let (k1, k2) = generate_aes128_cmac_subkeys(key);
    let subkey = if last_block_is_complete { k1 } else { k2 };

    let mut final_block = last_block;
    xor_slice_mut(&mut final_block, &subkey);

    let mut ret = Z128;
    if num_blocks > 0 {
        for i in 0..num_blocks - 1 {
            let block: Block = message[i * BLOCK_SIZE..][..BLOCK_SIZE]
                .try_into()
                .expect("The slice length is guaranteed to be a multiple of the block size");
            xor_slice_mut(&mut ret, &block);
            ret = encrypt_aes_ecb(&ret, key);
        }
    }
    xor_slice_mut(&mut ret, &final_block);
    encrypt_aes_ecb(&ret, key)
}

/// Computes the Pseudo Random Key (PRK) used to derive auth, encryption and nonce keys
pub fn compute_prk(ecdh_shared_secret: &[u8], pub_key_a: &[u8], pub_key_b: &[u8]) -> AesKey {
    let message = [ecdh_shared_secret, pub_key_a, pub_key_b].concat();
    compute_cmac(&message, &CONSTANT_PRK).into()
}

pub struct DerivedTempKeys {
    pub temp_key_ccm: AesKey,
    pub temp_personalization_string: PersonalizationString,
}

/// Derives the temporary auth, encryption and nonce keys from the PRK
pub fn derive_temp_keys(prk: &AesKey) -> DerivedTempKeys {
    let t1 = compute_cmac(&[&CONSTANT_TE[..], &[0x01]].concat(), prk);
    let t2 = compute_cmac(&[&t1[..], &CONSTANT_TE[..], &[0x02]].concat(), prk);
    let t3 = compute_cmac(&[&t2[..], &CONSTANT_TE[..], &[0x03]].concat(), prk);

    let temp_personalization_string = PersonalizationString::from_halves(&t2, &t3);

    DerivedTempKeys {
        temp_key_ccm: t1.into(),
        temp_personalization_string,
    }
}

pub struct DerivedNetworkKeys {
    pub key_ccm: AesKey,
    pub key_mpan: AesKey,
    pub personalization_string: PersonalizationString,
}

/// Derives the CCM, MPAN keys and the personalization string from the permanent network key (PNK)
pub fn derive_network_keys(pnk: &AesKey) -> DerivedNetworkKeys {
    let t1 = compute_cmac(&[&CONSTANT_NK[..], &[0x01]].concat(), pnk);
    let t2 = compute_cmac(&[&t1[..], &CONSTANT_NK[..], &[0x02]].concat(), pnk);
    let t3 = compute_cmac(&[&t2[..], &CONSTANT_NK[..], &[0x03]].concat(), pnk);
    let t4 = compute_cmac(&[&t3[..], &CONSTANT_NK[..], &[0x04]].concat(), pnk);
    let personalization_string = PersonalizationString::from_halves(&t2, &t3);

    DerivedNetworkKeys {
        key_ccm: t1.into(),
        key_mpan: t4.into(),
        personalization_string,
    }
}

/// Computes the Pseudo Random Key (PRK) used to derive the mixed entropy input (MEI) for nonce generation
pub fn compute_nonce_prk(sender_ei: &EntropyInput, receiver_ei: &EntropyInput) -> AesKey {
    let entropy = Entropy::from_halves(sender_ei, receiver_ei);
    compute_cmac(entropy.as_ref(), &CONSTANT_NONCE).into()
}

/// Derives the MEI from the nonce PRK
pub fn derive_mei(nonce_prk: &AesKey) -> Entropy {
    let t1 = compute_cmac(
        &[&CONSTANT_EI[..], &[0x00], &CONSTANT_EI[..], &[0x01]].concat(),
        nonce_prk,
    );
    let t2 = compute_cmac(&[&t1[..], &CONSTANT_EI[..], &[0x02]].concat(), nonce_prk);
    Entropy::from_halves(&t1, &t2)
}

pub struct AesCcmEncResult {
    pub ciphertext: Vec<u8>,
    pub auth_tag: [u8; SECURITY_S2_AUTH_TAG_LENGTH],
}

pub fn encrypt_aes_128_ccm(
    key: &AesKey,
    iv: &AesCcmNonce,
    plaintext: &[u8],
    additional_data: &[u8],
) -> AesCcmEncResult {
    let cipher: Aes128Ccm = Aes128Ccm::new(&key.into());
    let mut ciphertext = plaintext.to_vec();
    let auth_tag = cipher
        .encrypt_in_place_detached(&iv.into(), additional_data, &mut ciphertext)
        // FIXME: Proper error handling
        .unwrap()
        .into();

    AesCcmEncResult {
        ciphertext,
        auth_tag,
    }
}

pub type AesCcmDecResult = Option<Vec<u8>>;

pub fn decrypt_aes_128_ccm(
    key: &AesKey,
    iv: &AesCcmNonce,
    ciphertext: &[u8],
    additional_data: &[u8],
    auth_tag: &[u8; SECURITY_S2_AUTH_TAG_LENGTH],
) -> AesCcmDecResult {
    let cipher: Aes128Ccm = Aes128Ccm::new(&key.into());
    let mut plaintext = ciphertext.to_vec();
    match cipher.decrypt_in_place_detached(
        &iv.into(),
        additional_data,
        &mut plaintext,
        auth_tag.into(),
    ) {
        Ok(_) => Some(plaintext),
        Err(_) => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hex_literal;

    fn block(hex: &str) -> Block {
        hex::decode(hex).unwrap().try_into().unwrap()
    }

    fn key(hex: &str) -> AesKey {
        block(hex).into()
    }

    fn iv(hex: &str) -> AesIV {
        block(hex).into()
    }

    fn mac(hex: &str) -> [u8; MAC_SIZE] {
        hex::decode(hex).unwrap().try_into().unwrap()
    }

    #[test]
    fn test_encrypt_aes_ecb() {
        // Test vector taken from https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf
        let key = key("2b7e151628aed2a6abf7158809cf4f3c");
        let plaintext = block("6bc1bee22e409f96e93d7e117393172a");
        let expected = block("3ad77bb40d7a3660a89ecaf32466ef97");

        assert_eq!(expected, encrypt_aes_ecb(&plaintext, &key));
    }

    #[test]
    fn test_encrypt_aes_ofb() {
        // Test vector taken from https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf
        let key = key("2b7e151628aed2a6abf7158809cf4f3c");
        let iv = iv("000102030405060708090a0b0c0d0e0f");
        let plaintext = hex_literal!("6bc1bee22e409f96e93d7e117393172a");
        let expected = hex_literal!("3b3fd92eb72dad20333449f8e83cfb4a");

        assert_eq!(encrypt_aes_ofb(&plaintext, &key, &iv), expected);
    }

    #[test]
    fn test_decrypt_aes_ofb() {
        // Raw: 0x9881 78193fd7b91995ba 47645ec33fcdb3994b104ebd712e8b7fbd9120d049 28 4e39c14a0dc9aee5
        // Decrypted Packet: 0x009803008685598e60725a845b7170807aef2526ef
        // Nonce: 0x2866211bff3783d6
        // Network Key: 0x0102030405060708090a0b0c0d0e0f10

        let network_key = key("0102030405060708090a0b0c0d0e0f10");
        let key = AesKey::from(encrypt_aes_ecb(&[0xaa; 16], &network_key));
        let iv = iv("78193fd7b91995ba2866211bff3783d6");
        let ciphertext = hex_literal!("47645ec33fcdb3994b104ebd712e8b7fbd9120d049");
        let plaintext = decrypt_aes_ofb(&ciphertext, &key, &iv);
        let expected = hex_literal!("009803008685598e60725a845b7170807aef2526ef");

        assert_eq!(plaintext, expected);
    }

    #[test]
    fn test_compute_mac() {
        // Test vector taken from https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf
        let key = key("2b7e151628aed2a6abf7158809cf4f3c");
        // The Z-Wave specs use 16 zeros, but we only found test vectors for this
        let iv = iv("000102030405060708090a0b0c0d0e0f");
        let plaintext = hex_literal!("6bc1bee22e409f96e93d7e117393172a");
        let expected = mac("7649abac8119b246");

        assert_eq!(expected, compute_mac_iv(&plaintext, &key, &iv));
    }

    #[test]
    fn test_compute_mac_2() {
        // Taken from real Z-Wave communication - if anything must be changed, this is the test case to keep!
        let key = key("c5fe1ca17d36c992731a0c0c468c1ef9");
        let plaintext =
            hex_literal!("ddd360c382a437514392826cbba0b3128114010cf3fb762d6e82126681c18597");
        let expected = mac("2bc20a8aa9bbb371");

        assert_eq!(expected, compute_mac(&plaintext, &key));
    }

    #[test]
    fn test_compute_cmac_1() {
        // Test vector taken from https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/AES_CMAC.pdf
        let key = key("2B7E151628AED2A6ABF7158809CF4F3C");
        let plaintext = &[];
        let expected = hex_literal!("BB1D6929E95937287FA37D129B756746");

        assert_eq!(expected, compute_cmac(plaintext, &key));
    }

    #[test]
    fn test_compute_cmac_2() {
        // Test vector taken from https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/AES_CMAC.pdf
        let key = key("2B7E151628AED2A6ABF7158809CF4F3C");
        let plaintext = hex_literal!("6BC1BEE22E409F96E93D7E117393172A");
        let expected = hex_literal!("070A16B46B4D4144F79BDD9DD04A287C");

        assert_eq!(expected, compute_cmac(&plaintext, &key));
    }

    #[test]
    fn test_compute_cmac_3() {
        // Test vector taken from https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/AES_CMAC.pdf
        let key = key("2B7E151628AED2A6ABF7158809CF4F3C");
        let plaintext = hex_literal!("6BC1BEE22E409F96E93D7E117393172AAE2D8A57");
        let expected = hex_literal!("7D85449EA6EA19C823A7BF78837DFADE");

        assert_eq!(expected, compute_cmac(&plaintext, &key));
    }

    #[test]
    fn test_compute_cmac_4() {
        // Test vector taken from https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/AES_CMAC.pdf
        let key = key("2B7E151628AED2A6ABF7158809CF4F3C");
        let plaintext = hex_literal!(
            "6BC1BEE22E409F96E93D7E117393172AAE2D8A571E03AC9C9EB76FAC45AF8E5130C81C46A35CE411E5FBC1191A0A52EFF69F2445DF4F9B17AD2B417BE66C3710"
        );
        let expected = hex_literal!("51F0BEBF7E3B9D92FC49741779363CFE");

        assert_eq!(expected, compute_cmac(&plaintext, &key));
    }
}
