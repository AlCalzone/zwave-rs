use aes::cipher::{
    block_padding::ZeroPadding,
    generic_array::{
        typenum::{U13, U16, U8},
        GenericArray,
    },
    BlockEncrypt, BlockEncryptMut, KeyInit, KeyIvInit, StreamCipher,
};
use ccm::{aead::Aead, AeadInPlace};

type Aes128Ofb = ofb::Ofb<aes::Aes128>;
type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
pub type Aes128Ccm = ccm::Ccm<aes::Aes128, U8, U13>;

pub fn encrypt_aes_ecb(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    let cipher = aes::Aes128::new(key.into());

    let mut block: GenericArray<u8, U16> = [0; 16].into();
    block.copy_from_slice(plaintext);

    cipher.encrypt_block(&mut block);

    block.to_vec()
}

pub fn encrypt_aes_ofb(plaintext: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    let mut cipher = <Aes128Ofb as KeyIvInit>::new(key.into(), iv.into());

    let mut buf = plaintext.to_vec();
    cipher.apply_keystream(&mut buf);

    buf
}

pub fn decrypt_aes_ofb(ciphertext: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    let mut cipher = <Aes128Ofb as KeyIvInit>::new(key.into(), iv.into());

    let mut buf = ciphertext.to_vec();
    cipher.apply_keystream(&mut buf);

    buf
}

pub fn compute_mac(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    let iv = [0u8; 16];
    compute_mac_iv(plaintext, key, &iv)
}

pub fn compute_mac_iv(plaintext: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    let cipher = Aes128CbcEnc::new(key.into(), iv.into());
    let buf = cipher.encrypt_padded_vec_mut::<ZeroPadding>(plaintext);
    // The MAC is the first 8 bytes of the last 16 byte block
    buf[buf.len() - 16..][..8].to_vec()
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

const Z128: [u8; 16] = [0; 16];
const R128: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x87];

// Computes the byte-wise XOR of two slices with the same length
pub fn xor_slices(a: &[u8], b: &[u8]) -> Vec<u8> {
    assert!(a.len() == b.len(), "Slices must have the same length");
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}

// Computes the byte-wise XOR of two slices with the same length, mutating the first slice
pub fn xor_slice_mut(a: &mut [u8], b: &[u8]) {
    assert!(a.len() == b.len(), "Slices must have the same length");
    a.iter_mut().zip(b.iter()).for_each(|(x, y)| *x ^= y);
}

// Creates a new vec from a slice in MSB ordering by left-shifting it one bit
pub fn left_shift_1(input: &[u8]) -> Vec<u8> {
    if input.is_empty() {
        return vec![];
    }

    let mut ret = vec![0; input.len()];
    // TODO: Maybe use iterators here?
    for i in 0..input.len() - 1 {
        ret[i] = (input[i] << 1) + if input[i + 1] & 0x80 != 0 { 1 } else { 0 };
    }
    ret[input.len() - 1] = input[input.len() - 1] << 1;

    ret
}

#[test]
fn test_left_shift_1() {
    assert_eq!(left_shift_1(&[0x00]), vec![0x00]);
    assert_eq!(left_shift_1(&[0x01]), vec![0x02]);
    assert_eq!(left_shift_1(&[0x80]), vec![0x00]);
    assert_eq!(left_shift_1(&[0x01, 0x00]), vec![0x02, 0x00]);
    assert_eq!(left_shift_1(&[0x01, 0x80]), vec![0x03, 0x00]);
    assert_eq!(left_shift_1(&[0x01, 0x40]), vec![0x02, 0x80]);
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

pub fn generate_aes128_cmac_subkeys(key: &[u8]) -> (Vec<u8>, Vec<u8>) {
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
pub fn compute_cmac(message: &[u8], key: &[u8]) -> Vec<u8> {
    let block_size = 16;
    let remainder = message.len() % block_size;
    let num_blocks = message.len() / block_size + if remainder == 0 { 0 } else { 1 };

    let last_block = if num_blocks > 0 {
        &message[(num_blocks - 1) * block_size..]
    } else {
        message
    };
    let last_block_is_complete = !message.is_empty() && remainder == 0;
    let last_block = if last_block_is_complete {
        last_block.to_vec()
    } else {
        let mut last_block: Vec<u8> = last_block.to_vec();
        last_block.reserve_exact(block_size - remainder);
        last_block.push(0x80);
        last_block.resize(block_size, 0);
        last_block
    };

    // Compute all steps but the last one
    let mut ret = Z128.to_vec();
    // TODO: These steps should ideally modify ret in place. I haven't figured out how to do that with the aes crate directly yet
    if num_blocks > 0 {
        for i in 0..num_blocks - 1 {
            let block = &message[i * block_size..(i + 1) * block_size];
            xor_slice_mut(&mut ret, block);
            ret = encrypt_aes_ecb(&ret, key);
        }
    }
    // Compute the last step
    let (k1, k2) = generate_aes128_cmac_subkeys(key);
    xor_slice_mut(
        &mut ret,
        &xor_slices(if last_block_is_complete { &k1 } else { &k2 }, &last_block),
    );
    ret = encrypt_aes_ecb(&ret, key);

    ret[..block_size].to_vec()
}

const CONSTANT_PRK: [u8; 16] = [0x33; 16];

/// Computes the Pseudo Random Key (PRK) used to derive auth, encryption and nonce keys
pub fn compute_prk(ecdh_shared_secret: &[u8], pub_key_a: &[u8], pub_key_b: &[u8]) -> Vec<u8> {
    let message = [&ecdh_shared_secret, pub_key_a, pub_key_b].concat();
    compute_cmac(&message, &CONSTANT_PRK)
}

const CONSTANT_TE: [u8; 15] = [0x88; 15];

pub struct TempKeys {
    pub temp_key_ccm: Vec<u8>,
    pub temp_personalization_string: Vec<u8>,
}

/// Derives the temporary auth, encryption and nonce keys from the PRK
pub fn derive_temp_keys(prk: &[u8]) -> TempKeys {
    let t1 = compute_cmac(&[&CONSTANT_TE[..], &[0x01]].concat(), prk);
    let t2 = compute_cmac(&[&t1, &CONSTANT_TE[..], &[0x02]].concat(), prk);
    let t3 = compute_cmac(&[&t2, &CONSTANT_TE[..], &[0x03]].concat(), prk);

    TempKeys {
        temp_key_ccm: t1,
        temp_personalization_string: [t2, t3].concat(),
    }
}

// const constantNK = Buffer.alloc(15, 0x55);
const CONSTANT_NK: [u8; 15] = [0x55; 15];

pub struct NetworkKeys {
    pub key_ccm: Vec<u8>,
    pub key_mpan: Vec<u8>,
    pub personalization_string: Vec<u8>,
}

/// Derives the CCM, MPAN keys and the personalization string from the permanent network key (PNK)
pub fn derive_network_keys(pnk: &[u8]) -> NetworkKeys {
    let t1 = compute_cmac(&[&CONSTANT_NK[..], &[0x01]].concat(), pnk);
    let t2 = compute_cmac(&[&t1, &CONSTANT_NK[..], &[0x02]].concat(), pnk);
    let t3 = compute_cmac(&[&t2, &CONSTANT_NK[..], &[0x03]].concat(), pnk);
    let t4 = compute_cmac(&[&t3, &CONSTANT_NK[..], &[0x04]].concat(), pnk);

    NetworkKeys {
        key_ccm: t1,
        key_mpan: t4,
        personalization_string: [t2, t3].concat(),
    }
}

// const constantNonce = Buffer.alloc(16, 0x26);
const CONSTANT_NONCE: [u8; 16] = [0x26; 16];

/// Computes the Pseudo Random Key (PRK) used to derive the mixed entropy input (MEI) for nonce generation
pub fn compute_nonce_prk(sender_ei: &[u8], receiver_ei: &[u8]) -> Vec<u8> {
    let message = [sender_ei, receiver_ei].concat();
    compute_cmac(&message, &CONSTANT_NONCE)
}

// const constantEI = Buffer.alloc(15, 0x88);
const CONSTANT_EI: [u8; 15] = [0x88; 15];

/// Derives the MEI from the nonce PRK
pub fn derive_mei(nonce_prk: &[u8]) -> Vec<u8> {
    let t1 = compute_cmac(
        &[&CONSTANT_EI[..], &[0x00], &CONSTANT_EI[..], &[0x01]].concat(),
        nonce_prk,
    );
    let t2 = compute_cmac(&[&t1, &CONSTANT_EI[..], &[0x02]].concat(), nonce_prk);
    [t1, t2].concat()
}

const SECURITY_S2_AUTH_TAG_LENGTH: usize = 8;

pub struct AesCcmEncResult {
    pub ciphertext: Vec<u8>,
    pub auth_tag: Vec<u8>,
}

pub fn encrypt_aes_128_ccm(
    key: &[u8],
    iv: &[u8],
    plaintext: &[u8],
    additional_data: &[u8],
) -> AesCcmEncResult {
    let cipher: Aes128Ccm = Aes128Ccm::new(key.into());
    let mut ciphertext = plaintext.to_vec();
    let auth_tag = cipher
        .encrypt_in_place_detached(iv.into(), additional_data, &mut ciphertext)
        // FIXME: Proper error handling
        .unwrap()
        .to_vec();

    AesCcmEncResult {
        ciphertext,
        auth_tag,
    }
}

pub type AesCcmDecResult = Option<Vec<u8>>;

pub fn decrypt_aes_128_ccm(
    key: &[u8],
    iv: &[u8],
    ciphertext: &[u8],
    additional_data: &[u8],
    auth_tag: &[u8],
) -> AesCcmDecResult {
    let cipher: Aes128Ccm = Aes128Ccm::new(key.into());
    let mut plaintext = ciphertext.to_vec();
    match cipher.decrypt_in_place_detached(
        iv.into(),
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

    #[test]
    fn test_encrypt_aes_ecb() {
        // Test vector taken from https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf
        let key = hex_literal!("2b7e151628aed2a6abf7158809cf4f3c");
        let plaintext = hex_literal!("6bc1bee22e409f96e93d7e117393172a");
        let expected = hex_literal!("3ad77bb40d7a3660a89ecaf32466ef97");

        assert_eq!(encrypt_aes_ecb(&plaintext, &key), expected);
    }

    #[test]
    fn test_encrypt_aes_ofb() {
        // Test vector taken from https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf
        let key = hex_literal!("2b7e151628aed2a6abf7158809cf4f3c");
        let iv = hex_literal!("000102030405060708090a0b0c0d0e0f");
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

        let key = encrypt_aes_ecb(
            &[0xaa; 16],
            &hex_literal!("0102030405060708090a0b0c0d0e0f10"),
        );
        let iv = hex_literal!("78193fd7b91995ba2866211bff3783d6");
        let ciphertext = hex_literal!("47645ec33fcdb3994b104ebd712e8b7fbd9120d049");
        let plaintext = decrypt_aes_ofb(&ciphertext, &key, &iv);
        let expected = hex_literal!("009803008685598e60725a845b7170807aef2526ef");

        assert_eq!(plaintext, expected);
    }

    #[test]
    fn test_compute_mac() {
        // Test vector taken from https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf
        let key = hex_literal!("2b7e151628aed2a6abf7158809cf4f3c");
        // The Z-Wave specs use 16 zeros, but we only found test vectors for this
        let iv = hex_literal!("000102030405060708090a0b0c0d0e0f");
        let plaintext = hex_literal!("6bc1bee22e409f96e93d7e117393172a");
        let expected = hex_literal!("7649abac8119b246");

        assert_eq!(compute_mac_iv(&plaintext, &key, &iv), expected);
    }

    #[test]
    fn test_compute_mac_2() {
        // Taken from real Z-Wave communication - if anything must be changed, this is the test case to keep!
        let key = hex_literal!("c5fe1ca17d36c992731a0c0c468c1ef9");
        let plaintext =
            hex_literal!("ddd360c382a437514392826cbba0b3128114010cf3fb762d6e82126681c18597");
        let expected = hex_literal!("2bc20a8aa9bbb371");

        assert_eq!(compute_mac(&plaintext, &key), expected);
    }

    #[test]
    fn test_compute_cmac_1() {
        // Test vector taken from https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/AES_CMAC.pdf
        let key = hex_literal!("2B7E151628AED2A6ABF7158809CF4F3C");
        let plaintext = &[];
        let expected = hex_literal!("BB1D6929E95937287FA37D129B756746");

        assert_eq!(compute_cmac(plaintext, &key), expected);
    }

    #[test]
    fn test_compute_cmac_2() {
        // Test vector taken from https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/AES_CMAC.pdf
        let key = hex_literal!("2B7E151628AED2A6ABF7158809CF4F3C");
        let plaintext = hex_literal!("6BC1BEE22E409F96E93D7E117393172A");
        let expected = hex_literal!("070A16B46B4D4144F79BDD9DD04A287C");

        assert_eq!(compute_cmac(&plaintext, &key), expected);
    }

    #[test]
    fn test_compute_cmac_3() {
        // Test vector taken from https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/AES_CMAC.pdf
        let key = hex_literal!("2B7E151628AED2A6ABF7158809CF4F3C");
        let plaintext = hex_literal!("6BC1BEE22E409F96E93D7E117393172AAE2D8A57");
        let expected = hex_literal!("7D85449EA6EA19C823A7BF78837DFADE");

        assert_eq!(compute_cmac(&plaintext, &key), expected);
    }

    #[test]
    fn test_compute_cmac_4() {
        // Test vector taken from https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/AES_CMAC.pdf
        let key = hex_literal!("2B7E151628AED2A6ABF7158809CF4F3C");
        let plaintext = hex_literal!("6BC1BEE22E409F96E93D7E117393172AAE2D8A571E03AC9C9EB76FAC45AF8E5130C81C46A35CE411E5FBC1191A0A52EFF69F2445DF4F9B17AD2B417BE66C3710");
        let expected = hex_literal!("51F0BEBF7E3B9D92FC49741779363CFE");

        assert_eq!(compute_cmac(&plaintext, &key), expected);
    }
}
