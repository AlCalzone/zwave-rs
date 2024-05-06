use super::crypto::{encrypt_aes_ecb, increment_slice_mut, xor_slice_mut};

const KEY_LEN: usize = 16;
const BLOCK_LEN: usize = 16;
const SEED_LEN: usize = KEY_LEN + BLOCK_LEN;

// Warning: This code expects ctr_len to equal BLOCK_LEN.
// See specification on how to handle other cases

pub struct CtrDrbg {
    v: [u8; BLOCK_LEN],
    key: [u8; KEY_LEN],
    // reseed counter is not used
}

impl CtrDrbg {
    pub fn new(entropy: [u8; SEED_LEN]) -> Self {
        let mut ret = Self {
            v: [0; BLOCK_LEN],
            key: [0; KEY_LEN],
        };

        ret.init(entropy);

        ret
    }

    fn init(&mut self, entropy: [u8; SEED_LEN]) {
        // No personalization_string is used, otherwise XOR entropy with it
        // and use the result as seed material.

        self.update(Some(entropy));
    }

    fn update(&mut self, provided_data: Option<[u8; SEED_LEN]>) {
        let mut temp: Vec<u8> = Vec::with_capacity(SEED_LEN);
        while temp.len() < SEED_LEN {
            increment_slice_mut(&mut self.v);
            temp.append(&mut encrypt_aes_ecb(&self.v, &self.key));
        }
        temp.truncate(SEED_LEN);

        if let Some(provided_data) = provided_data {
            xor_slice_mut(&mut temp, &provided_data);
        }

        let (key, v) = temp.split_at_mut(KEY_LEN);
        self.key.copy_from_slice(key);
        self.v.copy_from_slice(v);
    }

    pub fn generate(&mut self, bytes: usize) -> Vec<u8> {
        // Additional input is not used
        let num_blocks = bytes / BLOCK_LEN + if bytes % BLOCK_LEN == 0 { 0 } else { 1 };
        let mut temp: Vec<u8> = Vec::with_capacity(num_blocks * BLOCK_LEN);

        while temp.len() < bytes {
            increment_slice_mut(&mut self.v);
            temp.append(&mut encrypt_aes_ecb(&self.v, &self.key));
        }
        temp.truncate(bytes);

        self.update(None);

        temp
    }

    #[cfg(test)]
    pub fn reseed(&mut self, entropy: [u8; SEED_LEN]) {
        // Reseeding isn't necessary for this implementation, but all test vectors use it
        self.update(Some(entropy));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct TestVector {
        entropy: Vec<u8>,
        entropy_reseed: Vec<u8>,
        bytes: usize,
        expected: Vec<u8>,
    }

    fn get_vectors() -> Vec<TestVector> {
        let input = include_str!("ctr_drbg.test.vectors.txt");
        let mut ret = Vec::new();
        let mut start_index = 0;
        while let Some(index) = input[start_index..].find("COUNT = ") {
            let index = start_index + index;
            let entropy_index =
                input[index..].find("EntropyInput = ").unwrap() + index + "EntropyInput = ".len();
            let eol_index = input[entropy_index..].find('\n').unwrap() + entropy_index;
            let entropy = hex::decode(input[entropy_index..eol_index].trim()).unwrap();

            let entropy_reseed_index = input[entropy_index..]
                .find("EntropyInputReseed = ")
                .unwrap()
                + entropy_index
                + "EntropyInputReseed = ".len();
            let eol_index =
                input[entropy_reseed_index..].find('\n').unwrap() + entropy_reseed_index;
            let entropy_reseed =
                hex::decode(input[entropy_reseed_index..eol_index].trim()).unwrap();

            let expected_index = input[entropy_reseed_index..]
                .find("ReturnedBits = ")
                .unwrap()
                + entropy_reseed_index
                + "ReturnedBits = ".len();
            let eol_index = input[expected_index..].find('\n').unwrap() + expected_index;
            let expected = hex::decode(input[expected_index..eol_index].trim()).unwrap();

            ret.push(TestVector {
                entropy,
                entropy_reseed,
                expected,
                bytes: 512 / 8, // hardcoded, should probably be parsed from vectors
            });

            start_index = eol_index;
        }

        ret
    }

    #[test]
    fn test_ctr_dbrg() {
        let vectors = get_vectors();

        for vector in vectors {
            let mut drbg = CtrDrbg::new(vector.entropy.as_slice().try_into().unwrap());

            // The tests reseed and generate twice
            drbg.reseed(vector.entropy_reseed.clone().try_into().unwrap());
            let _ = drbg.generate(vector.bytes);
            let actual = drbg.generate(vector.bytes);

            assert_eq!(
                &actual,
                &vector.expected,
                r#"Failed test:
  entropy = {}
  entropy_reseed = {}
  expected = {}
  actual = {}"#,
                hex::encode(vector.entropy),
                hex::encode(vector.entropy_reseed),
                hex::encode(&vector.expected),
                hex::encode(&actual)
            );
        }
    }
}
