use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use casper_types::{
    account::{AddKeyFailure, RemoveKeyFailure, UpdateKeyFailure, Weight, MAX_ASSOCIATED_KEYS},
    bytesrepr::{Error, FromBytes, ToBytes},
    PublicKey,
};

#[derive(Default, PartialOrd, Ord, PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct AssociatedKeys(BTreeMap<PublicKey, Weight>);

impl AssociatedKeys {
    pub fn new(key: PublicKey, weight: Weight) -> AssociatedKeys {
        let mut bt: BTreeMap<PublicKey, Weight> = BTreeMap::new();
        bt.insert(key, weight);
        AssociatedKeys(bt)
    }

    /// Adds new AssociatedKey to the set.
    /// Returns true if added successfully, false otherwise.
    #[allow(clippy::map_entry)]
    pub fn add_key(&mut self, key: PublicKey, weight: Weight) -> Result<(), AddKeyFailure> {
        if self.0.len() == MAX_ASSOCIATED_KEYS {
            Err(AddKeyFailure::MaxKeysLimit)
        } else if self.0.contains_key(&key) {
            Err(AddKeyFailure::DuplicateKey)
        } else {
            self.0.insert(key, weight);
            Ok(())
        }
    }

    /// Removes key from the associated keys set.
    /// Returns true if value was found in the set prior to the removal, false
    /// otherwise.
    pub fn remove_key(&mut self, key: &PublicKey) -> Result<(), RemoveKeyFailure> {
        self.0
            .remove(key)
            .map(|_| ())
            .ok_or(RemoveKeyFailure::MissingKey)
    }

    /// Adds new AssociatedKey to the set.
    /// Returns true if added successfully, false otherwise.
    #[allow(clippy::map_entry)]
    pub fn update_key(&mut self, key: PublicKey, weight: Weight) -> Result<(), UpdateKeyFailure> {
        if !self.0.contains_key(&key) {
            return Err(UpdateKeyFailure::MissingKey);
        }

        self.0.insert(key, weight);
        Ok(())
    }

    pub fn get(&self, key: &PublicKey) -> Option<&Weight> {
        self.0.get(key)
    }

    pub fn contains_key(&self, key: &PublicKey) -> bool {
        self.0.contains_key(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&PublicKey, &Weight)> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Helper method that calculates weight for keys that comes from any
    /// source.
    ///
    /// This method is not concerned about uniqueness of the passed iterable.
    /// Uniqueness is determined based on the input collection properties,
    /// which is either BTreeSet (in [`AssociatedKeys::calculate_keys_weight`])
    /// or BTreeMap (in [`AssociatedKeys::total_keys_weight`]).
    fn calculate_any_keys_weight<'a>(&self, keys: impl Iterator<Item = &'a PublicKey>) -> Weight {
        let total = keys
            .filter_map(|key| self.0.get(key))
            .fold(0u8, |acc, w| acc.saturating_add(w.value()));

        Weight::new(total)
    }

    /// Calculates total weight of authorization keys provided by an argument
    pub fn calculate_keys_weight(&self, authorization_keys: &BTreeSet<PublicKey>) -> Weight {
        self.calculate_any_keys_weight(authorization_keys.iter())
    }

    /// Calculates total weight of all authorization keys
    pub fn total_keys_weight(&self) -> Weight {
        self.calculate_any_keys_weight(self.0.keys())
    }

    /// Calculates total weight of all authorization keys excluding a given key
    pub fn total_keys_weight_excluding(&self, public_key: PublicKey) -> Weight {
        self.calculate_any_keys_weight(self.0.keys().filter(|&&element| element != public_key))
    }
}

impl From<BTreeMap<PublicKey, Weight>> for AssociatedKeys {
    fn from(associated_keys: BTreeMap<PublicKey, Weight>) -> Self {
        Self(associated_keys)
    }
}

impl ToBytes for AssociatedKeys {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        self.0.to_bytes()
    }

    fn serialized_length(&self) -> usize {
        self.0.serialized_length()
    }
}

impl FromBytes for AssociatedKeys {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (num_keys, mut stream) = u32::from_bytes(bytes)?;
        if num_keys as usize > MAX_ASSOCIATED_KEYS {
            return Err(Error::Formatting);
        }

        let mut associated_keys = BTreeMap::new();
        for _ in 0..num_keys {
            let (k, rem) = FromBytes::from_bytes(stream)?;
            let (v, rem) = FromBytes::from_bytes(rem)?;
            associated_keys.insert(k, v);
            stream = rem;
        }
        Ok((AssociatedKeys(associated_keys), stream))
    }
}

#[cfg(any(feature = "gens", test))]
pub mod gens {
    use proptest::prelude::*;

    use casper_types::{
        account::MAX_ASSOCIATED_KEYS, crypto::gens::public_key_arb, gens::weight_arb,
    };

    use super::AssociatedKeys;

    pub fn associated_keys_arb() -> impl Strategy<Value = AssociatedKeys> {
        proptest::collection::btree_map(public_key_arb(), weight_arb(), MAX_ASSOCIATED_KEYS - 1)
            .prop_map(|keys| {
                let mut associated_keys = AssociatedKeys::default();
                keys.into_iter().for_each(|(k, v)| {
                    associated_keys.add_key(k, v).unwrap();
                });
                associated_keys
            })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, BTreeSet},
        iter::FromIterator,
    };

    use casper_types::{
        account::{AddKeyFailure, Weight, MAX_ASSOCIATED_KEYS},
        bytesrepr::{self, ToBytes},
        PublicKey, SecretKey,
    };

    use super::AssociatedKeys;
    use once_cell::sync::Lazy;

    static PUBLIC_KEY_1: Lazy<PublicKey> =
        Lazy::new(|| SecretKey::ed25519([1u8; SecretKey::ED25519_LENGTH]).into());
    static PUBLIC_KEY_2: Lazy<PublicKey> =
        Lazy::new(|| SecretKey::ed25519([2u8; SecretKey::ED25519_LENGTH]).into());
    static PUBLIC_KEY_3: Lazy<PublicKey> =
        Lazy::new(|| SecretKey::ed25519([3u8; SecretKey::ED25519_LENGTH]).into());
    static PUBLIC_KEY_4: Lazy<PublicKey> =
        Lazy::new(|| SecretKey::ed25519([4u8; SecretKey::ED25519_LENGTH]).into());

    #[test]
    fn associated_keys_add() {
        let mut keys = AssociatedKeys::new(*PUBLIC_KEY_1, Weight::new(1));
        let new_weight = Weight::new(2);
        assert!(keys.add_key(*PUBLIC_KEY_2, new_weight).is_ok());
        assert_eq!(keys.get(&*PUBLIC_KEY_1), Some(&new_weight))
    }

    #[test]
    fn associated_keys_add_full() {
        let map = (0..MAX_ASSOCIATED_KEYS).map(|k| (*PUBLIC_KEY_1, Weight::new(k as u8)));
        assert_eq!(map.len(), 10);
        let mut keys = {
            let mut tmp = AssociatedKeys::default();
            map.for_each(|(key, weight)| assert!(tmp.add_key(key, weight).is_ok()));
            tmp
        };
        assert_eq!(
            keys.add_key(*PUBLIC_KEY_2, Weight::new(100)),
            Err(AddKeyFailure::MaxKeysLimit)
        )
    }

    #[test]
    fn associated_keys_add_duplicate() {
        let weight = Weight::new(1);
        let mut keys = AssociatedKeys::new(*PUBLIC_KEY_1, weight);
        assert_eq!(
            keys.add_key(*PUBLIC_KEY_1, Weight::new(10)),
            Err(AddKeyFailure::DuplicateKey)
        );
        assert_eq!(keys.get(&*PUBLIC_KEY_1), Some(&weight));
    }

    #[test]
    fn associated_keys_remove() {
        let weight = Weight::new(1);
        let mut keys = AssociatedKeys::new(*PUBLIC_KEY_1, weight);
        assert!(keys.remove_key(&*PUBLIC_KEY_1).is_ok());
        assert!(keys.remove_key(&*PUBLIC_KEY_1).is_err());
    }

    #[test]
    fn associated_keys_calculate_keys_once() {
        let mut keys = AssociatedKeys::default();

        keys.add_key(*PUBLIC_KEY_1, Weight::new(2))
            .expect("should add key 1");
        keys.add_key(*PUBLIC_KEY_2, Weight::new(1))
            .expect("should add key 2");
        keys.add_key(*PUBLIC_KEY_3, Weight::new(3))
            .expect("should add key 3");

        assert_eq!(
            keys.calculate_keys_weight(&BTreeSet::from_iter(vec![
                *PUBLIC_KEY_1,
                *PUBLIC_KEY_2,
                *PUBLIC_KEY_3
            ])),
            Weight::new(1 + 2 + 3)
        );
    }

    #[test]
    fn associated_keys_total_weight() {
        let associated_keys = {
            let mut res = AssociatedKeys::new(*PUBLIC_KEY_1, Weight::new(1));
            res.add_key(*PUBLIC_KEY_2, Weight::new(11))
                .expect("should add key 2");
            res.add_key(*PUBLIC_KEY_3, Weight::new(12))
                .expect("should add key 3");
            res.add_key(*PUBLIC_KEY_4, Weight::new(13))
                .expect("should add key 4");
            res
        };
        assert_eq!(
            associated_keys.total_keys_weight(),
            Weight::new(1 + 11 + 12 + 13)
        );
    }

    #[test]
    fn associated_keys_total_weight_excluding() {
        let key_1_weight = Weight::new(1);
        let key_2_weight = Weight::new(11);
        let key_3_weight = Weight::new(12);
        let key_4_weight = Weight::new(13);

        let associated_keys = {
            let mut res = AssociatedKeys::new(*PUBLIC_KEY_1, key_1_weight);
            res.add_key(*PUBLIC_KEY_2, key_2_weight)
                .expect("should add key 2");
            res.add_key(*PUBLIC_KEY_3, key_3_weight)
                .expect("should add key 3");
            res.add_key(*PUBLIC_KEY_4, key_4_weight)
                .expect("should add key 4");
            res
        };
        assert_eq!(
            associated_keys.total_keys_weight_excluding(*PUBLIC_KEY_3),
            Weight::new(key_1_weight.value() + key_2_weight.value() + key_4_weight.value())
        );
    }

    #[test]
    fn overflowing_keys_weight() {
        let key_1_weight = Weight::new(250);
        let key_2_weight = Weight::new(1);
        let key_3_weight = Weight::new(2);
        let key_4_weight = Weight::new(3);

        let saturated_weight = Weight::new(u8::max_value());

        let associated_keys = {
            let mut res = AssociatedKeys::new(*PUBLIC_KEY_1, key_1_weight);
            res.add_key(*PUBLIC_KEY_2, key_2_weight)
                .expect("should add key 1");
            res.add_key(*PUBLIC_KEY_3, key_3_weight)
                .expect("should add key 2");
            res.add_key(*PUBLIC_KEY_4, key_4_weight)
                .expect("should add key 3");
            res
        };

        assert_eq!(
            associated_keys.calculate_keys_weight(&BTreeSet::from_iter(vec![
                *PUBLIC_KEY_1, // 250
                *PUBLIC_KEY_2, // 251
                *PUBLIC_KEY_3, // 253
                *PUBLIC_KEY_4, // 256 - error
            ])),
            saturated_weight,
        );
    }

    #[test]
    fn serialization_roundtrip() {
        let mut keys = AssociatedKeys::default();
        keys.add_key(*PUBLIC_KEY_1, Weight::new(1)).unwrap();
        keys.add_key(*PUBLIC_KEY_2, Weight::new(2)).unwrap();
        keys.add_key(*PUBLIC_KEY_3, Weight::new(3)).unwrap();
        bytesrepr::test_serialization_roundtrip(&keys);
    }

    #[test]
    fn should_not_panic_deserializing_malicious_data() {
        let malicious_map: BTreeMap<PublicKey, Weight> = (1usize..=(MAX_ASSOCIATED_KEYS + 1))
            .map(|i| {
                let i_bytes = i.to_be_bytes();
                let mut public_key_bytes = [0u8; 32];
                public_key_bytes[32 - i_bytes.len()..].copy_from_slice(&i_bytes);
                (
                    SecretKey::ed25519(public_key_bytes).into(),
                    Weight::new(i as u8),
                )
            })
            .collect();

        let bytes = malicious_map.to_bytes().expect("should serialize");

        assert_eq!(
            bytesrepr::deserialize::<AssociatedKeys>(bytes).expect_err("should deserialize"),
            bytesrepr::Error::Formatting
        );
    }
}
