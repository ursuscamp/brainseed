use sha2::{Digest, Sha256};

fn main() {
    let input = std::env::args().nth(1);

    if input.is_none() {
        eprintln!("Please specify input phrase.");
        return;
    }

    let input = input.unwrap();
    println!("Input value: {input:#?}");

    let mut hasher = Sha256::new();
    hasher.update(&input);
    let result = hasher.finalize()[..16].to_vec();

    let seed_phrase = bip39::Mnemonic::from_entropy(&result).unwrap();
    println!("Final seed phrase: {seed_phrase}");
}
