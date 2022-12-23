use std::{io::Write, path::PathBuf};

use anyhow::Context;
use bdk::{
    bitcoin::{
        bech32::{encode, ToBase32, Variant},
        consensus::{deserialize, serialize},
        hashes::hex::ToHex,
        psbt::PartiallySignedTransaction,
        secp256k1::Secp256k1,
        util::bip32::{DerivationPath, ExtendedPrivKey},
        Network,
    },
    database::MemoryDatabase,
    keys::{bip39::Mnemonic, DerivableKey, ExtendedKey},
    miniscript::miniscript,
    template::Bip84,
    KeychainKind, Wallet,
};

use crate::generator::Generator;

#[derive(clap::Parser, Clone)]
pub struct Cli {
    /// Number of times to hash the passphrase
    #[clap(short = 'n', default_value = "10000000")]
    pub iterations: usize,

    /// Return a 24 word seed phrase [default: 12]
    #[clap(short, long)]
    long: bool,

    /// Switch to testnet mode
    #[clap(short, long)]
    testnet: bool,

    #[clap(subcommand)]
    pub action: Action,
}

impl Cli {
    pub fn exec(&self) -> anyhow::Result<()> {
        let input = self.get_input()?;
        let seed = self.seed(input);
        match &self.action {
            Action::Seed => self.write_output(seed.to_string().as_bytes()),
            Action::Sign { input, output } => self.sign(input, output, seed),
            Action::Watch => self.show_descriptor(seed),
            Action::Nostr { nip19, index } => self.nostr(seed, *nip19, *index),
        }
    }

    fn seed(&self, input: Vec<u8>) -> Mnemonic {
        let mut gen = Generator {
            input,
            iterations: self.iterations,
            long: self.long,
        };
        gen.seed()
    }

    fn sign(&self, input: &PathBuf, output: &PathBuf, seed: Mnemonic) -> anyhow::Result<()> {
        let wallet = self.wallet(seed)?;
        let input = std::fs::read(&input).context("Unable to read signing input file")?;
        let mut psbt: PartiallySignedTransaction =
            deserialize(&input).context("Failed to deserialize partially signed transaction")?;
        let signed = wallet
            .sign(&mut psbt, Default::default())
            .context("Error encountered signing the transaction")?;
        let s = serialize(&psbt);
        std::fs::write(output, &s).context("Unable to write signed transaction")?;

        if !signed {
            println!("Transaction not complete.");
        }

        Ok(())
    }

    fn wallet(&self, seed: Mnemonic) -> anyhow::Result<Wallet<MemoryDatabase>> {
        let xkey: ExtendedKey<miniscript::Segwitv0> = seed
            .into_extended_key()
            .context("Failed to convert mnemonic into an extended key")?;
        let xprv = xkey.into_xprv(self.network()).unwrap();
        Wallet::new(
            Bip84(xprv.clone(), KeychainKind::External),
            Some(Bip84(xprv.clone(), KeychainKind::External)),
            self.network(),
            MemoryDatabase::default(),
        )
        .context("Failed to create wallet")
        .into()
    }

    fn get_input(&self) -> anyhow::Result<Vec<u8>> {
        Ok(self.get_password()?.as_bytes().to_vec())
    }

    fn write_output(&self, data: &[u8]) -> anyhow::Result<()> {
        std::io::stdout().write(data)?;
        Ok(())
    }

    fn get_password(&self) -> anyhow::Result<String> {
        let pass = rpassword::prompt_password("Entropy prompt: ")?;
        let confirm = rpassword::prompt_password("Confirm: ")?;
        if pass == confirm {
            return Ok(pass);
        }
        Err(anyhow::anyhow!("Prompt does not match"))
    }

    fn show_descriptor(&self, seed: Mnemonic) -> anyhow::Result<()> {
        let wallet = self.wallet(seed)?;
        let descriptor = wallet
            .public_descriptor(KeychainKind::External)
            .context("Descriptor error")?
            .ok_or(anyhow::anyhow!("Descriptor error"))?;
        print!("{descriptor}");
        Ok(())
    }

    fn network(&self) -> Network {
        if self.testnet {
            Network::Testnet
        } else {
            Network::Bitcoin
        }
    }

    fn nostr(&self, seed: Mnemonic, nip19: bool, index: u32) -> Result<(), anyhow::Error> {
        let ctx = Secp256k1::new();
        let dv: DerivationPath = format!("m/44'/1237'/0'/0/{index}")
            .parse()
            .context("Invalid derivation path for nostr")?;
        let seed = seed.to_seed("");
        let master = ExtendedPrivKey::new_master(Network::Bitcoin, &seed)?;
        let xpriv = master.derive_priv(&ctx, &dv)?;
        let privkey = xpriv.to_priv();
        let pubkey = privkey.public_key(&ctx);

        let (pubkey, privkey) = match nip19 {
            true => (
                encode("nsec", privkey.to_bytes().to_base32(), Variant::Bech32)?,
                encode("npub", pubkey.to_bytes().to_base32(), Variant::Bech32)?,
            ),
            false => (pubkey.to_bytes()[1..].to_hex(), privkey.to_bytes().to_hex()),
        };

        println!("Private: {privkey}");
        println!("Public:  {pubkey}");

        Ok(())
    }
}

#[derive(clap::Subcommand, Clone)]
pub enum Action {
    /// Generate a mnemonic seed phrase.
    Seed,

    /// Sign a bitcoin transaction file.
    Sign { input: PathBuf, output: PathBuf },

    /// Show wallet descriptor that is useful for importing as a watch-only wallet.
    Watch,

    /// Generate a Nostr private/public keypair (optionally in NIP-19 encoding).
    Nostr {
        /// Serialize the keys according to NIP-19 (npub/nsec format).
        #[clap(long)]
        nip19: bool,

        /// Child index for the derivation path m/44'/1237'/0'/0.
        /// Use this to create additional keys for your seed.
        #[clap(short, long, default_value_t = 0)]
        index: u32,
    },
}
