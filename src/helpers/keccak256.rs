use sha3::{Digest, Keccak256};

pub fn keccak256(input: String) -> String {
    let mut hasher = Keccak256::new();
    hasher.update(input.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keccak256() {
        assert_eq!(
            keccak256("test".to_string()),
            "9c22ff5f21f0b81b113e63f7db6da94fedef11b2119b4088b89664fb9a3cb658"
        );
    }
}
