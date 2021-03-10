use super::*;

mod partial_tries {
    use super::*;
    use crate::storage::trie_store::operations::DeleteResult;

    fn delete_from_partial_trie_had_expected_results<'a, K, V, R, S, E>(
        correlation_id: CorrelationId,
        environment: &'a R,
        store: &S,
        root: &Blake2bHash,
        key_to_delete: &K,
        expected_root_after_delete: &Blake2bHash,
        expected_tries_after_delete: &[HashedTrie<K, V>],
    ) -> Result<(), E>
    where
        K: ToBytes + FromBytes + Clone + Eq + std::fmt::Debug,
        V: ToBytes + FromBytes + Clone + Eq + std::fmt::Debug,
        R: TransactionSource<'a, Handle = S::Handle>,
        S: TrieStore<K, V>,
        S::Error: From<R::Error>,
        E: From<R::Error> + From<S::Error> + From<bytesrepr::Error>,
    {
        let mut txn = environment.create_read_write_txn()?;
        assert_eq!(store.get(&txn, &expected_root_after_delete)?, None); // this only works with partial tries
        let root_after_delete = match operations::delete::<K, V, _, _, E>(
            correlation_id,
            &mut txn,
            store,
            root,
            key_to_delete,
        )? {
            DeleteResult::Deleted(root_after_delete) => root_after_delete,
            DeleteResult::DoesNotExist => panic!("key did not exist"),
            DeleteResult::RootNotFound => panic!("root should be found"),
        };
        assert_eq!(root_after_delete, *expected_root_after_delete);
        for HashedTrie { hash, trie } in expected_tries_after_delete {
            assert_eq!(store.get(&txn, &hash)?, Some(trie.clone()));
        }
        Ok(())
    }

    #[test]
    fn lmdb_delete_from_partial_trie_had_expected_results() {
        for i in 0..TEST_LEAVES_LENGTH {
            let correlation_id = CorrelationId::new();
            let (initial_root_hash, initial_tries) = TEST_TRIE_GENERATORS[i + 1]().unwrap();
            let (updated_root_hash, updated_tries) = TEST_TRIE_GENERATORS[i]().unwrap();
            let key_to_delete = &TEST_LEAVES[i];
            let context = LmdbTestContext::new(&initial_tries).unwrap();

            delete_from_partial_trie_had_expected_results::<TestKey, TestValue, _, _, error::Error>(
                correlation_id,
                &context.environment,
                &context.store,
                &initial_root_hash,
                key_to_delete.key().unwrap(),
                &updated_root_hash,
                updated_tries.as_slice(),
            )
            .unwrap();
        }
    }

    #[test]
    fn in_memory_delete_from_partial_trie_had_expected_results() {
        for i in 0..TEST_LEAVES_LENGTH {
            let correlation_id = CorrelationId::new();
            let (initial_root_hash, initial_tries) = TEST_TRIE_GENERATORS[i + 1]().unwrap();
            let (updated_root_hash, updated_tries) = TEST_TRIE_GENERATORS[i]().unwrap();
            let key_to_delete = &TEST_LEAVES[i];
            let context = InMemoryTestContext::new(&initial_tries).unwrap();

            delete_from_partial_trie_had_expected_results::<TestKey, TestValue, _, _, error::Error>(
                correlation_id,
                &context.environment,
                &context.store,
                &initial_root_hash,
                key_to_delete.key().unwrap(),
                &updated_root_hash,
                updated_tries.as_slice(),
            )
            .unwrap();
        }
    }

    fn delete_non_existent_key_from_partial_trie_should_return_does_not_exist<'a, K, V, R, S, E>(
        correlation_id: CorrelationId,
        environment: &'a R,
        store: &S,
        root: &Blake2bHash,
        key_to_delete: &K,
    ) -> Result<(), E>
    where
        K: ToBytes + FromBytes + Clone + Eq + std::fmt::Debug,
        V: ToBytes + FromBytes + Clone + Eq + std::fmt::Debug,
        R: TransactionSource<'a, Handle = S::Handle>,
        S: TrieStore<K, V>,
        S::Error: From<R::Error>,
        E: From<R::Error> + From<S::Error> + From<bytesrepr::Error>,
    {
        let mut txn = environment.create_read_write_txn()?;
        match operations::delete::<K, V, _, _, E>(
            correlation_id,
            &mut txn,
            store,
            root,
            key_to_delete,
        )? {
            DeleteResult::Deleted(_) => panic!("should not delete"),
            DeleteResult::DoesNotExist => Ok(()),
            DeleteResult::RootNotFound => panic!("root should be found"),
        }
    }

    #[test]
    fn lmdb_delete_non_existent_key_from_partial_trie_should_return_does_not_exist() {
        for i in 0..TEST_LEAVES_LENGTH {
            let correlation_id = CorrelationId::new();
            let (initial_root_hash, initial_tries) = TEST_TRIE_GENERATORS[i]().unwrap();
            let key_to_delete = &TEST_LEAVES_ADJACENTS[i];
            let context = LmdbTestContext::new(&initial_tries).unwrap();

            delete_non_existent_key_from_partial_trie_should_return_does_not_exist::<
                TestKey,
                TestValue,
                _,
                _,
                error::Error,
            >(
                correlation_id,
                &context.environment,
                &context.store,
                &initial_root_hash,
                key_to_delete.key().unwrap(),
            )
            .unwrap();
        }
    }

    #[test]
    fn in_memory_delete_non_existent_key_from_partial_trie_should_return_does_not_exist() {
        for i in 0..TEST_LEAVES_LENGTH {
            let correlation_id = CorrelationId::new();
            let (initial_root_hash, initial_tries) = TEST_TRIE_GENERATORS[i]().unwrap();
            let key_to_delete = &TEST_LEAVES_ADJACENTS[i];
            let context = InMemoryTestContext::new(&initial_tries).unwrap();

            delete_non_existent_key_from_partial_trie_should_return_does_not_exist::<
                TestKey,
                TestValue,
                _,
                _,
                error::Error,
            >(
                correlation_id,
                &context.environment,
                &context.store,
                &initial_root_hash,
                key_to_delete.key().unwrap(),
            )
            .unwrap();
        }
    }
}
