use super::{SecretKey, BLS_PUBLIC_KEY_BYTE_SIZE};
use cached_tree_hash::cached_tree_hash_ssz_encoding_as_vector;
use milagro_bls::PublicKey as RawPublicKey;
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use serde_hex::{encode as hex_encode, HexVisitor};
use ssz::{ssz_encode, Decode, DecodeError};
use std::default;
use std::fmt;
use std::hash::{Hash, Hasher};
use tree_hash::tree_hash_ssz_encoding_as_vector;

/// A single BLS signature.
///
/// This struct is a wrapper upon a base type and provides helper functions (e.g., SSZ
/// serialization).
#[derive(Debug, Clone, Eq)]
pub struct PublicKey(RawPublicKey);

impl PublicKey {
    pub fn from_secret_key(secret_key: &SecretKey) -> Self {
        PublicKey(RawPublicKey::from_secret_key(secret_key.as_raw()))
    }

    /// Returns the underlying signature.
    pub fn as_raw(&self) -> &RawPublicKey {
        &self.0
    }

    /// Returns the underlying point as compressed bytes.
    ///
    /// Identical to `self.as_uncompressed_bytes()`.
    fn as_bytes(&self) -> Vec<u8> {
        self.as_raw().as_bytes()
    }

    /// Converts compressed bytes to PublicKey
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let pubkey = RawPublicKey::from_bytes(&bytes).map_err(|_| {
            DecodeError::BytesInvalid(format!("Invalid PublicKey bytes: {:?}", bytes).to_string())
        })?;

        Ok(PublicKey(pubkey))
    }

    /// Returns the PublicKey as (x, y) bytes
    pub fn as_uncompressed_bytes(&self) -> Vec<u8> {
        RawPublicKey::as_uncompressed_bytes(&mut self.0.clone())
    }

    /// Converts (x, y) bytes to PublicKey
    pub fn from_uncompressed_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let pubkey = RawPublicKey::from_uncompressed_bytes(&bytes).map_err(|_| {
            DecodeError::BytesInvalid("Invalid PublicKey uncompressed bytes.".to_string())
        })?;
        Ok(PublicKey(pubkey))
    }

    /// Returns the last 6 bytes of the SSZ encoding of the public key, as a hex string.
    ///
    /// Useful for providing a short identifier to the user.
    pub fn concatenated_hex_id(&self) -> String {
        let bytes = ssz_encode(self);
        let end_bytes = &bytes[bytes.len().saturating_sub(6)..bytes.len()];
        hex_encode(end_bytes)
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.concatenated_hex_id())
    }
}

impl default::Default for PublicKey {
    fn default() -> Self {
        let secret_key = SecretKey::random();
        PublicKey::from_secret_key(&secret_key)
    }
}

impl_ssz!(PublicKey, BLS_PUBLIC_KEY_BYTE_SIZE, "PublicKey");

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex_encode(self.as_raw().as_bytes()))
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = deserializer.deserialize_str(HexVisitor)?;
        let pubkey = Self::from_ssz_bytes(&bytes[..])
            .map_err(|e| serde::de::Error::custom(format!("invalid pubkey ({:?})", e)))?;
        Ok(pubkey)
    }
}

tree_hash_ssz_encoding_as_vector!(PublicKey);
cached_tree_hash_ssz_encoding_as_vector!(PublicKey, 48);

impl PartialEq for PublicKey {
    fn eq(&self, other: &PublicKey) -> bool {
        ssz_encode(self) == ssz_encode(other)
    }
}

impl Hash for PublicKey {
    /// Note: this is distinct from consensus serialization, it will produce a different hash.
    ///
    /// This method uses the uncompressed bytes, which are much faster to obtain than the
    /// compressed bytes required for consensus serialization.
    ///
    /// Use `ssz::Encode` to obtain the bytes required for consensus hashing.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_uncompressed_bytes().hash(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ssz::ssz_encode;
    use tree_hash::TreeHash;

    #[test]
    pub fn test_ssz_round_trip() {
        let sk = SecretKey::random();
        let original = PublicKey::from_secret_key(&sk);

        let bytes = ssz_encode(&original);
        let decoded = PublicKey::from_ssz_bytes(&bytes).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    pub fn test_cached_tree_hash() {
        let sk = SecretKey::random();
        let original = PublicKey::from_secret_key(&sk);

        let mut cache = cached_tree_hash::TreeHashCache::new(&original).unwrap();

        assert_eq!(
            cache.tree_hash_root().unwrap().to_vec(),
            original.tree_hash_root()
        );

        let sk = SecretKey::random();
        let modified = PublicKey::from_secret_key(&sk);

        cache.update(&modified).unwrap();

        assert_eq!(
            cache.tree_hash_root().unwrap().to_vec(),
            modified.tree_hash_root()
        );
    }
}
