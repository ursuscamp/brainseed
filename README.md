# ezseed

`ezseed` is a terminal utility to generate a BIP-39 compatible seed phrase from deterministic "entropy". It accepts a passphrase and uses that passphrase as the input for an algorithm to deterministically create a 12 (or 24) word BIP-39 seed phrase.


```shell
$ ezseed "hello world"
cliff burden nut payment negative soccer one mad pulse balcony force inside
```

**WARNING**: Please note this is experimental software at this time. It is very simple, but be warned, there could be bugs lurking, and the algorithm may change based on feedback.

## Frequently Asked Questions

### This is horrible. Why would you do such a thing?

For funsies.

But actually, I know at first glance this may seem terrible, but there are situations where this might make sense. Specifically, what I had in mind was someone fleeing and wanting to take their wealth with them.

Remembering 12 or 24 random, unrelated words is not the most difficult thing in the world, but it can be nerve wracking, especially in a situation where you may be unwilling or unable to keep a physical backup.

Similar to [Border Wallets](https://www.borderwallets.com), in a case where you may wish to sacrifice some randomness for peace of mind, this could be an option.

### Why does it generate a BIP-39 seed phrase?

BIP-39 seed phrases have become the lingua franca of Bitcoin key management. Almost every wallet allows creating/importing BIP-39 seed phrases, so deterministically generating these seed phrases makes sense for compatibility.

### How does it work, technically?

After providing a passphrase to the utility it:

1. Normalizes valid UTF-8 input to prevent some input entry errors:
   * Converts it to ASCII lowercase.
   * Removes invalid characters. Valid characters are `[a-z0-9 ]`.
   * Condenses consecutive spaces to one space.
   * Removes beginning and trailing spaces.
   * For example: `"Hello WORLD!!!!"` becomes `"hello world"`.
2. Hashes it with SHA-256 ten million times.
3. Uses the result as entropy to generate a 12 or 24 word BIP-39 seed phrase.

### Is this not poor security?

Well, humans are relatively predictable, so it won't stand up to brute force attacks like a random seed phrase will. On the other hand, it might also be better opsec to have a passphrase that is hard to forget and only in your head, instead of a random phrase that you have to keep a physical copy just to remember.

In a pinch, it may be a good way to flee a hostile area with your wealth intact.

To help with brute force resistance, it uses 10,000,000 iterations of SHA-256, which takes several seconds on my modern MacBook. If you wish to opt for additional security, you can increase the default number of iterations and also turn off normalization.

### What if I wish to use non-Latin alphabet?

I would suggest disabling normalization.

Also, if you want, you don't need to use UTF-8 text. You can can pass in any data, and if it doesn't recognize it as a UTF-8 compatible string, it will skip normalization completely. For example, a favorite song:

```shell
$ ezseed -f "Blind Guardian - Mirror Mirror.mp3"
scale unknown injury drip together slender scheme mask warrior talent common happy
```

You may also want to pass in `-u` to force stop normalization, just in case, by some random chance, the bytes in your binary file happen to form a valid UTF-8 string.

However, this may be less secure than a passphrase since you must store that file somewhere. YMMV

### How can I generate a 24 word phrase?

Use the `-l` or `--long` flag to get a 24 word seed phrase.

### What about rainbow tables?

Yup, that's a danger. Use a phrase meaningful to you, not a famous movie line or something. Also consider using a custom number of SHA-256 iterations as this will help foil rainbow attacks.

If you absolutely must use a famous movie line, then salt it with some other meaningful data, like the year you lost your viriginity, e.g.:

```shell
ezseed "there's no crying in baseball never"
vintage hotel mechanic major inner shield ready side wealth fresh replace clock
```