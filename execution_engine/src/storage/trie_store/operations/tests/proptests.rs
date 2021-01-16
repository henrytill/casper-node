use std::{fmt::Debug, ops::RangeInclusive};

use proptest::{
    array,
    collection::vec,
    prelude::{any, proptest, Strategy},
};

use casper_types::{AsymmetricType, Key, PublicKey, SYSTEM_ACCOUNT};

use super::*;
use crate::shared::{account::Account, stored_value::StoredValue};

const DEFAULT_MIN_LENGTH: usize = 0;

const DEFAULT_MAX_LENGTH: usize = 100;

fn get_range() -> RangeInclusive<usize> {
    let start = option_env!("CL_TRIE_TEST_VECTOR_MIN_LENGTH")
        .and_then(|s| str::parse::<usize>(s).ok())
        .unwrap_or(DEFAULT_MIN_LENGTH);
    let end = option_env!("CL_TRIE_TEST_VECTOR_MAX_LENGTH")
        .and_then(|s| str::parse::<usize>(s).ok())
        .unwrap_or(DEFAULT_MAX_LENGTH);
    RangeInclusive::new(start, end)
}

fn lmdb_roundtrip_succeeds<K, V>(pairs: &[(K, V)]) -> bool
where
    K: ToBytes + FromBytes + Clone + Eq + Debug + Copy + Ord,
    V: ToBytes + FromBytes + Clone + Eq + Debug,
{
    let correlation_id = CorrelationId::new();
    let (root_hash, tries) = TEST_TRIE_GENERATORS[0]().unwrap();
    let context = LmdbTestContext::new(&tries).unwrap();
    let mut states_to_check = vec![];

    let root_hashes = write_pairs::<_, _, _, _, error::Error>(
        correlation_id,
        &context.environment,
        &context.store,
        &root_hash,
        pairs,
    )
    .unwrap();

    states_to_check.extend(root_hashes);

    let check_pairs_result = check_pairs::<_, _, _, _, error::Error>(
        correlation_id,
        &context.environment,
        &context.store,
        &states_to_check,
        &pairs,
    )
    .unwrap();

    if !check_pairs_result {
        return false;
    }

    check_pairs_proofs::<_, _, _, _, error::Error>(
        correlation_id,
        &context.environment,
        &context.store,
        &states_to_check,
        &pairs,
    )
    .unwrap()
}

fn in_memory_roundtrip_succeeds<K, V>(pairs: &[(K, V)]) -> bool
where
    K: ToBytes + FromBytes + Clone + Eq + Debug + Copy + Ord,
    V: ToBytes + FromBytes + Clone + Eq + Debug,
{
    let correlation_id = CorrelationId::new();
    let (root_hash, tries) = TEST_TRIE_GENERATORS[0]().unwrap();
    let context = InMemoryTestContext::new(&tries).unwrap();
    let mut states_to_check = vec![];

    let root_hashes = write_pairs::<_, _, _, _, in_memory::Error>(
        correlation_id,
        &context.environment,
        &context.store,
        &root_hash,
        pairs,
    )
    .unwrap();

    states_to_check.extend(root_hashes);

    let check_pairs_result = check_pairs::<_, _, _, _, in_memory::Error>(
        correlation_id,
        &context.environment,
        &context.store,
        &states_to_check,
        &pairs,
    )
    .unwrap();

    if !check_pairs_result {
        return false;
    }

    let ret = check_pairs_proofs::<_, _, _, _, in_memory::Error>(
        correlation_id,
        &context.environment,
        &context.store,
        &states_to_check,
        &pairs,
    )
    .unwrap();

    let dump = context.environment.dump::<K, V>(None).unwrap();
    println!("dump {:?}", dump);

    ret
}

fn test_key_arb() -> impl Strategy<Value = TestKey> {
    array::uniform7(any::<u8>()).prop_map(TestKey)
}

fn test_value_arb() -> impl Strategy<Value = TestValue> {
    array::uniform6(any::<u8>()).prop_map(TestValue)
}

proptest! {
    #[test]
    fn prop_in_memory_roundtrip_succeeds(inputs in vec((test_key_arb(), test_value_arb()), get_range())) {
        assert!(in_memory_roundtrip_succeeds(&inputs));
    }

    #[test]
    fn prop_lmdb_roundtrip_succeeds(inputs in vec((test_key_arb(), test_value_arb()), get_range())) {
        assert!(lmdb_roundtrip_succeeds(&inputs));
    }
}

#[test]
fn passing_keys() {
    let public_keys = &mut [
        "0202ba721020580446a30331b4b3973303c85c6ff3ae3bdc4e33643700ed93b1e2c7",
        "0202c6ced91552c6f8940afbf1696dc0075e372c5e354f71fffab8ef6d7daefe8f60",
        "012657bfaf643dd79650890bcc223b0a1f907eb8eea8ce241d90cc979ae74f4b0d",
    ]
    .iter()
    .map(PublicKey::from_hex)
    .collect::<Result<Vec<PublicKey>, _>>()
    .unwrap();

    public_keys.push(SYSTEM_ACCOUNT);

    let pairs: Vec<(Key, StoredValue)> = public_keys
        .iter()
        .map(|key| {
            let account = Account::create(*key, Default::default(), Default::default());
            (Key::Account(*key), StoredValue::Account(account))
        })
        .collect();

    assert!(in_memory_roundtrip_succeeds(&pairs))
}

#[should_panic]
#[test]
fn failing_keys() {
    let public_keys = &mut [
        "0202ba721020580446a30331b4b3973303c85c6ff3ae3bdc4e33643700ed93b1e2c7",
        "0202c6ced91552c6f8940afbf1696dc0075e372c5e354f71fffab8ef6d7daefe8f60",
    ]
    .iter()
    .map(PublicKey::from_hex)
    .collect::<Result<Vec<PublicKey>, _>>()
    .unwrap();

    public_keys.push(SYSTEM_ACCOUNT);

    let pairs: Vec<(Key, StoredValue)> = public_keys
        .iter()
        .map(|key| {
            let account = Account::create(*key, Default::default(), Default::default());
            (Key::Account(*key), StoredValue::Account(account))
        })
        .collect();

    assert!(in_memory_roundtrip_succeeds(&pairs))
}
