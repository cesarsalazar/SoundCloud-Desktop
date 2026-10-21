use std::collections::BTreeSet;

use hmac::{Hmac, Mac};
use sha2::Sha256;

use super::{PREFIX_MAX, PREFIX_MIN, TOKEN_BYTES};

pub fn normalize(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|ch| *ch != '\u{307}')
        .map(|ch| match ch {
            'ё' => 'е',
            'ı' => 'i',
            other => other,
        })
        .collect()
}

pub fn words(text: &str) -> Vec<String> {
    normalize(text)
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|word| word.chars().count() >= PREFIX_MIN)
        .map(str::to_owned)
        .collect()
}

pub fn document(key: &[u8; 32], text: &str) -> String {
    let keyed = keyed(key);
    let mut prefixes = BTreeSet::new();
    for word in words(text) {
        let chars: Vec<char> = word.chars().take(PREFIX_MAX).collect();
        for len in PREFIX_MIN..=chars.len() {
            prefixes.insert(chars[..len].iter().collect::<String>());
        }
    }
    let tokens: BTreeSet<String> = prefixes
        .iter()
        .map(|prefix| token(&keyed, prefix))
        .collect();
    tokens.into_iter().collect::<Vec<_>>().join(" ")
}

pub fn query(key: &[u8; 32], text: &str) -> Option<String> {
    let keyed = keyed(key);
    let terms: BTreeSet<String> = words(text)
        .iter()
        .map(|word| {
            let prefix: String = word.chars().take(PREFIX_MAX).collect();
            format!("\"{}\"", token(&keyed, &prefix))
        })
        .collect();
    (!terms.is_empty()).then(|| terms.into_iter().collect::<Vec<_>>().join(" AND "))
}

fn keyed(key: &[u8; 32]) -> Hmac<Sha256> {
    Hmac::<Sha256>::new_from_slice(key).expect("hmac takes any key length")
}

fn token(keyed: &Hmac<Sha256>, prefix: &str) -> String {
    let mut mac = keyed.clone();
    mac.update(prefix.as_bytes());
    let digest = mac.finalize().into_bytes();
    digest[..TOKEN_BYTES]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
