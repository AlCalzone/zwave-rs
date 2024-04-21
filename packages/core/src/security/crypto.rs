use aes::cipher::{
    block_padding::ZeroPadding,
    generic_array::{typenum::U16, GenericArray},
    BlockEncrypt, BlockEncryptMut, KeyInit, KeyIvInit, StreamCipher,
};

type Aes128Ofb = ofb::Ofb<aes::Aes128>;
type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

pub fn encrypt_aes_ecb(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    let cipher = aes::Aes128::new(key.into());

    let mut block: GenericArray<u8, U16> = [0; 16].into();
    block.copy_from_slice(plaintext);

    cipher.encrypt_block(&mut block);

    block.to_vec()
}

pub fn encrypt_aes_ofb(plaintext: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    let mut cipher = Aes128Ofb::new(key.into(), iv.into());

    let mut buf = plaintext.to_vec();
    cipher.apply_keystream(&mut buf);

    buf
}

pub fn decrypt_aes_ofb(ciphertext: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    let mut cipher = Aes128Ofb::new(key.into(), iv.into());

    let mut buf = ciphertext.to_vec();
    cipher.apply_keystream(&mut buf);

    buf
}

pub fn compute_mac(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    let iv = [0u8; 16];
    compute_mac_iv(plaintext, key, &iv)
}

pub fn compute_mac_iv(plaintext: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    let mut cipher = Aes128CbcEnc::new(key.into(), iv.into());
    let buf = cipher.encrypt_padded_vec_mut::<ZeroPadding>(plaintext);
    // The MAC is the first 8 bytes of the last 16 byte block
    buf[buf.len() - 16..][..8].to_vec()
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
}
