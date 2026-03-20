use super::crypto::{
    AesKey, Block, ENTROPY_SIZE, Entropy, PersonalizationString, encrypt_aes_ecb,
    increment_slice_mut, xor_slice_mut,
};

pub const KEY_LEN: usize = 16;
pub const BLOCK_LEN: usize = 16;
pub const SEED_LEN: usize = ENTROPY_SIZE;

// Warning: This code expects ctr_len to equal BLOCK_LEN.
// See specification on how to handle other cases

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtrDrbg {
    v: Block,
    key: AesKey,
    // reseed counter is not used
}

impl CtrDrbg {
    pub fn new(entropy: Entropy) -> Self {
        let mut ret = Self {
            v: [0; BLOCK_LEN],
            key: [0; KEY_LEN].into(),
        };

        ret.init(entropy, None);

        ret
    }

    pub fn new_with_personalization(
        entropy: Entropy,
        personalization_string: PersonalizationString,
    ) -> Self {
        let mut ret = Self {
            v: [0; BLOCK_LEN],
            key: [0; KEY_LEN].into(),
        };

        ret.init(entropy, Some(personalization_string));

        ret
    }

    fn init(&mut self, entropy: Entropy, personalization_string: Option<PersonalizationString>) {
        let mut seed_material: [u8; SEED_LEN] = entropy.into();
        if let Some(personalization_string) = personalization_string {
            let personalization_string: [u8; SEED_LEN] = personalization_string.into();
            xor_slice_mut(&mut seed_material, &personalization_string);
        }

        self.update(Some(seed_material));
    }

    fn update(&mut self, provided_data: Option<[u8; SEED_LEN]>) {
        let mut temp = [0; SEED_LEN];
        let mut offset = 0;
        while offset < SEED_LEN {
            increment_slice_mut(&mut self.v);
            let block = encrypt_aes_ecb(&self.v, &self.key);
            temp[offset..][..BLOCK_LEN].copy_from_slice(&block);
            offset += BLOCK_LEN;
        }

        if let Some(provided_data) = provided_data {
            xor_slice_mut(&mut temp, &provided_data);
        }

        let (key, v) = temp.split_at_mut(KEY_LEN);
        let key: [u8; KEY_LEN] = key.try_into().unwrap();
        self.key = key.into();
        self.v.copy_from_slice(v);
    }

    pub fn generate(&mut self, bytes: usize) -> Vec<u8> {
        // Additional input is not used
        let num_blocks = bytes / BLOCK_LEN + if bytes % BLOCK_LEN == 0 { 0 } else { 1 };
        let mut temp: Vec<u8> = Vec::with_capacity(num_blocks * BLOCK_LEN);

        while temp.len() < bytes {
            increment_slice_mut(&mut self.v);
            let block = encrypt_aes_ecb(&self.v, &self.key);
            temp.extend_from_slice(&block);
        }
        temp.truncate(bytes);

        self.update(None);

        temp
    }

    #[cfg(test)]
    pub fn reseed(&mut self, entropy: Entropy) {
        // Reseeding isn't necessary for this implementation, but all test vectors use it
        self.update(Some(entropy.into()));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct TestVector {
        entropy: Vec<u8>,
        personalization_string: Option<Vec<u8>>,
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

            let pers_string_index = input[entropy_index..]
                .find("PersonalizationString = ")
                .unwrap()
                + entropy_index
                + "PersonalizationString = ".len();
            let eol_index = input[pers_string_index..].find('\n').unwrap() + pers_string_index;
            let personalization_string =
                hex::decode(input[pers_string_index..eol_index].trim()).unwrap();
            let personalization_string = if personalization_string.is_empty() {
                None
            } else {
                Some(personalization_string)
            };

            let entropy_reseed_index = input[pers_string_index..]
                .find("EntropyInputReseed = ")
                .unwrap()
                + pers_string_index
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
                personalization_string,
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
            let mut drbg = if let Some(pers_str) = vector.personalization_string.as_ref() {
                CtrDrbg::new_with_personalization(
                    vector.entropy.as_slice().into(),
                    pers_str.as_slice().into(),
                )
            } else {
                CtrDrbg::new(vector.entropy.as_slice().into())
            };

            // The tests reseed and generate twice
            drbg.reseed(vector.entropy_reseed.as_slice().into());
            let _ = drbg.generate(vector.bytes);
            let actual = drbg.generate(vector.bytes);

            assert_eq!(
                &actual,
                &vector.expected,
                r#"Failed test:
  entropy = {}
  personalization_string = {}
  entropy_reseed = {}
  expected = {}
  actual = {}"#,
                hex::encode(vector.entropy),
                vector
                    .personalization_string
                    .map(hex::encode)
                    .unwrap_or_else(|| "None".to_string()),
                hex::encode(vector.entropy_reseed),
                hex::encode(&vector.expected),
                hex::encode(&actual)
            );
        }
    }
}
