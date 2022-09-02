use bip39::Mnemonic;
use sha2::{Digest, Sha256};

use crate::cli::Cli;

pub struct Generator {
    data: Vec<u8>,
    iterations: usize,
    long: bool,
    unnormalized: bool,
}

impl Generator {
    /// This is the entry point for the struct. This will normalize the input and create the mnemonic.
    pub fn seed(&mut self) -> Mnemonic {
        if self.should_normalize() {
            self.attempt_normalize();
        }
        self.hash_iterations();
        bip39::Mnemonic::from_entropy(self.entropy()).unwrap()
    }

    /// Return a reference to internal data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Should this generator attempt to normalize the input?
    fn should_normalize(&self) -> bool {
        !self.unnormalized
    }

    /// Remove invalid characters, then remove consecutive spaces ("   " becomes " "),
    /// then finally trim all whitespace from the ends of the string.
    pub fn normalize(&self, data: &str) -> String {
        let mut next_str = String::with_capacity(data.len());
        let start = self.remove_invalid_chars(data);

        let mut skip_ws = false;
        for ch in start.chars() {
            if ch == ' ' && !skip_ws {
                next_str.push(ch);
                skip_ws = true;
            } else if ch != ' ' {
                next_str.push(ch);
                skip_ws = false;
            }
        }

        next_str.trim().to_string()
    }

    /// Convert all ASCII characters to lowercase and remove invalid characters.
    /// Valid characters are [a-z0-9 ].
    fn remove_invalid_chars(&self, data: &str) -> String {
        let mut next_str = String::with_capacity(data.len());
        let start = data.to_ascii_lowercase();

        for ch in start.chars() {
            if ('a'..'z').contains(&ch) || ('0'..'9').contains(&ch) || ch == ' ' {
                next_str.push(ch);
            }
        }

        next_str
    }

    /// This will attempt to normalize data. If the data is a valid UTF-8 string, then it will normalize it.
    /// If it is not valid UTF-8, then it assumes the file is binary and passes it straight through.
    fn attempt_normalize(&mut self) {
        if let Ok(string) = std::str::from_utf8(&self.data) {
            self.data = self.normalize(string).into_bytes().to_vec();
        }
    }

    /// Returns the entropy needed for genearting the BIP-39 mnemonic.
    fn entropy(&self) -> &[u8] {
        if self.long {
            self.data.as_ref()
        } else {
            &self.data[..16]
        }
    }

    /// Itearte the hash function repeatedly on the input data.
    fn hash_iterations(&mut self) {
        let mut hasher = Sha256::new();
        hasher.update(&self.data);
        let mut data = hasher.finalize_reset();
        for _ in 1..self.iterations {
            hasher.update(&data);
            hasher.finalize_into_reset(&mut data);
        }

        self.data = data.to_vec();
    }
}

impl From<Cli> for Generator {
    fn from(cli: Cli) -> Self {
        Generator {
            data: cli.get_input(),
            iterations: cli.iterations,
            long: cli.long,
            unnormalized: cli.unnormalized,
        }
    }
}

#[cfg(test)]
mod tests {
    pub mod util {
        use crate::generator::Generator;

        pub fn gen12(data: &str) -> Generator {
            Generator {
                data: data.into(),
                iterations: 1,
                long: false,
                unnormalized: false,
            }
        }

        pub fn gen24(data: &str) -> Generator {
            Generator {
                data: data.into(),
                iterations: 1,
                long: true,
                unnormalized: false,
            }
        }

        pub fn normal_input() -> &'static str {
            "hello world"
        }

        pub fn abnormal_input() -> &'static str {
            "Hel!lo    wo!RLD!   "
        }
    }

    #[test]
    fn test_remove_invalid_chars() {
        let gen = util::gen12(" Hel!lo 1    world!  ");
        assert_eq!(
            gen.remove_invalid_chars(" Hel!lo 1    world!  "),
            " hello 1    world  "
        );
    }

    #[test]
    fn test_normalize() {
        let mut gen = util::gen12("  Hel!lo    1  !    WORLD!!   ");
        gen.attempt_normalize();
        assert_eq!(gen.data, b"hello 1 world");

        let mut gen = util::gen12("hello    world   !");
        gen.attempt_normalize();
        assert_eq!(gen.data, b"hello world");
    }

    #[test]
    fn test_binary_normalization_ignored() {
        let mut gen = util::gen12("");
        let data = include_bytes!("../test/junk.dat"); // Set data to binary data
        gen.data = data.to_vec();
        gen.attempt_normalize();
        assert_eq!(gen.data, data);
    }

    #[test]
    fn test_seed_phrase_short_normal() {
        let expected = "rich hard unveil charge stadium affair net ski style stadium helmet void";
        let mut gen = util::gen12(util::normal_input());
        assert_eq!(gen.seed().to_string(), expected);
    }

    #[test]
    fn test_seed_phrase_short_abnormal() {
        let expected = "rich hard unveil charge stadium affair net ski style stadium helmet void";
        let mut gen = util::gen12(util::abnormal_input());
        assert_eq!(gen.seed().to_string(), expected);
    }

    #[test]
    fn test_long_seed_phrase_normal() {
        let expected = "rich hard unveil charge stadium affair net ski style stadium helmet void embark jewel mistake engine liberty innocent captain urban soda jewel dash daring";
        let mut gen = util::gen24(util::normal_input());
        assert_eq!(gen.seed().to_string(), expected);
    }

    #[test]
    fn test_long_seed_phrase_abnormal() {
        let expected = "rich hard unveil charge stadium affair net ski style stadium helmet void embark jewel mistake engine liberty innocent captain urban soda jewel dash daring";
        let mut gen = util::gen24(util::abnormal_input());
        assert_eq!(gen.seed().to_string(), expected);
    }
}
