# brainseed

`brainseed` is a terminal utility for working with [BIP-32 wallets](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki) that stem from a passphrase
instead of a mnemonic seed phrase.

**WARNING**: Please note this is experimental software at this time. It is very simple, but be warned, there could be bugs lurking, and the algorithm may change based on feedback.

## Generating a seed phrase mnemonic

```
$ brainseed seed
Entropy prompt: hello world
Confirm: hello world
cliff burden nut payment negative soccer one mad pulse balcony force inside
```

This is deterministic function. Repeat multiple times to verify! By default, this uses ten million SHA-256 iterations, but you are encouraged to customize that using the `-n` flag.

## Watch-only wallet descriptor

Directly generate a watch-only output descriptor, to import into a wallet like Sparrow or Blue Wallet:

```
$ brainseed watch
Entropy prompt: hello world
Confirm: hello world
wpkh([b343958c/84'/1'/0']tpubDCCj6osNwAnnSqViJQjqHD9xkJ6UXKT73ZB36W5gNapyCmirdibyzHeRsAYK5z9V5fi4ZdGAGA2jbXxPD1qS3Yht2tU3shPuatfUUWvKeCc/0/*)#cj35ttn3
```
## Signing a transaction

This can be used to sign files as well, if you don't want to use a seed phrase in a separate wallet, and instead just want to use a watch-only wallet as above, combined with this function to act as a software signing device.

```
$ brainseed sign input.psbt output.psbt
Entropy prompt: hello world
Confirm: hello world
```

After importing a watch-only wallet into something like Sparrow, generate a transaction and save it as a raw file. Then sign it with this command, and load it back into Sparrow.

## Frequently Asked Questions

### This is horrible. Why would you do such a thing?

For funsies.

But actually, I know at first glance this may seem terrible, but there are situations where this might make sense. Specifically, what I had in mind was someone fleeing and wanting to take their wealth with them.

Remembering 12 or 24 random, unrelated words is not the most difficult thing in the world, but it can be nerve wracking, especially in a situation where you may be unwilling or unable to keep a physical backup.

Similar to [Border Wallets](https://www.borderwallets.com), in a case where you may wish to sacrifice some randomness for peace of mind, this could be an option.

### Why does it generate a BIP-39 seed phrase?

BIP-39 seed phrases have become the lingua franca of Bitcoin key management. Almost every wallet allows creating/importing BIP-39 seed phrases, so deterministically generating these seed phrases makes sense for wallet compatibility.

### How does it work, technically?

After providing a passphrase to the utility it:

1. Take some input, typically a passphrase.
2. Hashes it with SHA-256 ten million times (by default).
3. Uses the result as entropy to generate a 12 or 24 word BIP-39 seed phrase.

### Is this not poor security?

Well, humans are relatively predictable, so it won't stand up to brute force attacks like a random seed mnemonic will. On the other hand, it might also be better opsec to have a passphrase that is hard to forget and only in your head, instead of a random mnemonic that you have to keep a physical copy just to remember.

In a pinch, it may be a good way to flee a hostile area with your wealth intact.

To help with brute force resistance, it uses 10,000,000 iterations of SHA-256, which takes several seconds on my modern MacBook. If you wish to opt for additional security, you can increase the default number of iterations.

### How can I generate a 24 word phrase?

Use the `-l` or `--long` flag to get a 24 word seed phrase.

### What about rainbow tables?

Yup, that's a danger. Use a phrase meaningful to you, not a famous movie line or something like that. Also consider using a custom number of SHA-256 iterations as this will help foil rainbow attacks.