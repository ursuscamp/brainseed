use bdk::keys::bip39::Mnemonic;
use sha2::{Digest, Sha256};

pub struct Generator {
    pub input: Vec<u8>,
    pub iterations: usize,
    pub long: bool,
}

impl Generator {
    /// This is the entry point for the struct.
    pub fn seed(&mut self) -> Mnemonic {
        self.hash_iterations();
        Mnemonic::from_entropy(self.entropy()).unwrap()
    }

    /// Returns the entropy needed for genearting the BIP-39 mnemonic.
    fn entropy(&self) -> &[u8] {
        if self.long {
            self.input.as_ref()
        } else {
            &self.input[..16]
        }
    }

    /// Itearte the hash function repeatedly on the input data.
    fn hash_iterations(&mut self) {
        let mut hasher = Sha256::new();
        hasher.update(&self.input);
        let mut data = hasher.finalize_reset();
        for _ in 1..self.iterations {
            hasher.update(&data);
            hasher.finalize_into_reset(&mut data);
        }

        self.input = data.to_vec();
    }
}

#[cfg(test)]
mod tests {
    pub mod util {
        use crate::generator::Generator;

        pub fn gen12(data: &str) -> Generator {
            Generator {
                input: data.into(),
                iterations: 1,
                long: false,
            }
        }

        pub fn gen24(data: &str) -> Generator {
            Generator {
                input: data.into(),
                iterations: 1,
                long: true,
            }
        }

        pub fn input() -> &'static str {
            "hello world"
        }
    }

    #[test]
    fn test_short_seed_phrase() {
        let expected = "rich hard unveil charge stadium affair net ski style stadium helmet void";
        let mut gen = util::gen12(util::input());
        assert_eq!(gen.seed().to_string(), expected);
    }

    #[test]
    fn test_long_seed_phrase() {
        let expected = "rich hard unveil charge stadium affair net ski style stadium helmet void embark jewel mistake engine liberty innocent captain urban soda jewel dash daring";
        let mut gen = util::gen24(util::input());
        assert_eq!(gen.seed().to_string(), expected);
    }
}
