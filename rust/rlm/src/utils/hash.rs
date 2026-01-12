#![allow(dead_code)]

use sha2::{Digest, Sha256};

pub fn hash_string(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn hash_strings(inputs: &[String]) -> String {
    hash_string(&inputs.join("||"))
}
