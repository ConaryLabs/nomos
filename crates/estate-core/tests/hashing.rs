//! SHA-256 against the published FIPS 180-4 vectors, and the display contract.

use estate_core::CanonicalValue;
use estate_core::hash::{Sha256Digest, StateHash, sha256};

fn hex(input: &[u8]) -> String {
    Sha256Digest::of_bytes(input).to_hex()
}

#[test]
fn published_vectors() {
    // FIPS 180-4 / NIST CAVP known-answer values.
    assert_eq!(
        hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    // 448 bits: the padding case that fits one extra block boundary exactly.
    assert_eq!(
        hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
    // 896 bits: two-block padding.
    assert_eq!(
        hex(b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu"),
        "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1"
    );
    // One million 'a' characters.
    let million = vec![b'a'; 1_000_000];
    assert_eq!(
        hex(&million),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

#[test]
fn block_boundaries_are_padded_correctly() {
    // 55, 56, 63, 64, and 65 bytes exercise every padding branch.
    let expected = [
        (
            55,
            "9f4390f8d30c2dd92ec9f095b65e2b9ae9b0a925a5258e241c9f1e910f734318",
        ),
        (
            56,
            "b35439a4ac6f0948b6d6f9e3c6af0f5f590ce20f1bde7090ef7970686ec6738a",
        ),
        (
            63,
            "7d3e74a05d7db15bce4ad9ec0658ea98e3f06eeecf16b4c6fff2da457ddc2f34",
        ),
        (
            64,
            "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb",
        ),
        (
            65,
            "635361c48bb9eab14198e76ea8ab7f1a41685d6ad62aa9146d301d4f17eb0ae0",
        ),
    ];
    for (length, digest) in expected {
        assert_eq!(hex(&vec![b'a'; length]), digest, "length {length}");
    }
}

#[test]
fn digests_display_and_parse_as_lowercase_hex() {
    let digest = Sha256Digest::of_bytes(b"abc");
    let text = digest.to_string();
    assert_eq!(text.len(), 64);
    assert!(
        text.chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
    );
    assert_eq!(Sha256Digest::from_hex(&text), Some(digest));
    // One display form only: the uppercase spelling of the same digest is not
    // accepted, so a digest cannot have two representations in an artifact.
    assert_eq!(Sha256Digest::from_hex(&text.to_uppercase()), None);
    assert_eq!(Sha256Digest::from_hex("abc"), None);
    assert_eq!(sha256(b"abc"), *digest.as_bytes());
}

#[test]
fn a_state_hash_is_taken_over_canonical_bytes() {
    let envelope = CanonicalValue::object_declared([("tick", CanonicalValue::Uint(0))]);
    assert_eq!(
        StateHash::of_envelope(&envelope).to_hex(),
        Sha256Digest::of_bytes(br#"{"tick":0}"#).to_hex()
    );
}
