//! The tests below are generated twice: once against the contracts compiled into
//! this crate (exercising the host implementation of the sparse map host
//! functions), and once against the same contracts compiled to Wasm (exercising
//! the guest implementation that goes through linear memory). Both must agree.

macro_rules! migration_tests {
    () => {
        use soroban_sdk::{
            map,
            testutils::{Address as _, Events as _},
            vec, Address, Env, IntoVal, Symbol, Val,
        };

        fn as_val<T: IntoVal<Env, Val>>(env: &Env, t: &T) -> Val {
            t.into_val(env)
        }

        // --- Storage: adding a new optional field to a stored struct.

        /// A value written with the old schema reads into the new schema, with
        /// the added `Option` field set to `None`.
        #[test]
        fn read_v1_as_v2() {
            let env = Env::default();
            let client = SClient::new(&env, &register_storage(&env));
            let admin = Address::generate(&env);

            client.set_v1(&AdminInfoV1 {
                admin: admin.clone(),
                is_frozen: false,
            });

            // Only the fields of the old schema are stored.
            assert_eq!(
                client.admin_keys(),
                vec![
                    &env,
                    Symbol::new(&env, "admin"),
                    Symbol::new(&env, "is_frozen")
                ]
            );
            assert_eq!(
                client.get_v2(),
                AdminInfoV2 {
                    admin,
                    admins: None,
                    is_frozen: false,
                }
            );
        }

        /// The full migration: write the old schema, seed the new field, then
        /// read the final schema that no longer has the deprecated field.
        #[test]
        fn migrate_v1_to_v3() {
            let env = Env::default();
            let client = SClient::new(&env, &register_storage(&env));
            let admin = Address::generate(&env);

            client.set_v1(&AdminInfoV1 {
                admin: admin.clone(),
                is_frozen: true,
            });
            client.migrate_admins();

            // The intermediate schema stores all three fields.
            assert_eq!(
                client.admin_keys(),
                vec![
                    &env,
                    Symbol::new(&env, "admin"),
                    Symbol::new(&env, "admins"),
                    Symbol::new(&env, "is_frozen")
                ]
            );
            assert_eq!(
                client.get_v2(),
                AdminInfoV2 {
                    admin: admin.clone(),
                    admins: Some(vec![&env, admin.clone()]),
                    is_frozen: true,
                }
            );
            // The final schema reads the same value, ignoring the now-deprecated
            // `admin` key that is still in the stored map.
            assert_eq!(
                client.get_v3(),
                AdminInfoV3 {
                    admins: vec![&env, admin],
                    is_frozen: true,
                }
            );

            // Writing the final schema drops the deprecated key.
            client.set_v3(&client.get_v3());
            assert_eq!(
                client.admin_keys(),
                vec![
                    &env,
                    Symbol::new(&env, "admins"),
                    Symbol::new(&env, "is_frozen")
                ]
            );
        }

        // --- Storage: reading a value into a struct with fewer fields.

        /// A value written with more fields reads into a struct with fewer
        /// fields, even though the dropped field is not an `Option`.
        #[test]
        fn read_v2_as_v1() {
            let env = Env::default();
            let client = SClient::new(&env, &register_storage(&env));
            let admin = Address::generate(&env);
            let other = Address::generate(&env);

            client.set_v2(&AdminInfoV2 {
                admin: admin.clone(),
                admins: Some(vec![&env, admin.clone(), other]),
                is_frozen: false,
            });

            assert_eq!(
                client.get_v1(),
                AdminInfoV1 {
                    admin,
                    is_frozen: false,
                }
            );
        }

        /// A field that is absent from the stored map and is not an `Option` is a
        /// conversion error rather than a silent default.
        #[test]
        fn read_v1_as_v3_fails() {
            let env = Env::default();
            let client = SClient::new(&env, &register_storage(&env));

            client.set_v1(&AdminInfoV1 {
                admin: Address::generate(&env),
                is_frozen: false,
            });

            assert!(client.try_get_v3().is_err());
        }

        // --- Storage: sparse ("compact") structs.

        /// A sparse struct whose optional fields are all `None` stores only the
        /// fields that are set.
        #[test]
        fn sparse_config_omits_none() {
            let env = Env::default();
            let client = SClient::new(&env, &register_storage(&env));
            let owner = Address::generate(&env);

            let config = Config {
                owner: owner.clone(),
                fee_bps: None,
                max_amount: None,
                min_amount: None,
                paused: None,
            };
            client.set_config(&config);

            assert_eq!(client.config_keys(), vec![&env, Symbol::new(&env, "owner")]);
            assert_eq!(client.get_config(), config);
            assert_eq!(client.get_config_owner(), ConfigOwner { owner });
        }

        /// A sparse struct stores every field that is set.
        #[test]
        fn sparse_config_all_set() {
            let env = Env::default();
            let client = SClient::new(&env, &register_storage(&env));
            let owner = Address::generate(&env);

            let config = Config {
                owner: owner.clone(),
                fee_bps: Some(30),
                max_amount: Some(100),
                min_amount: Some(1),
                paused: Some(false),
            };
            client.set_config(&config);

            assert_eq!(
                client.config_keys(),
                vec![
                    &env,
                    Symbol::new(&env, "fee_bps"),
                    Symbol::new(&env, "max_amount"),
                    Symbol::new(&env, "min_amount"),
                    Symbol::new(&env, "owner"),
                    Symbol::new(&env, "paused"),
                ]
            );
            assert_eq!(client.get_config(), config);
            assert_eq!(client.get_config_owner(), ConfigOwner { owner });
        }

        /// A sparse struct stores only the subset of optional fields that are
        /// set.
        #[test]
        fn sparse_config_partially_set() {
            let env = Env::default();
            let client = SClient::new(&env, &register_storage(&env));
            let owner = Address::generate(&env);

            let config = Config {
                owner,
                fee_bps: None,
                max_amount: Some(100),
                min_amount: None,
                paused: Some(true),
            };
            client.set_config(&config);

            assert_eq!(
                client.config_keys(),
                vec![
                    &env,
                    Symbol::new(&env, "max_amount"),
                    Symbol::new(&env, "owner"),
                    Symbol::new(&env, "paused"),
                ]
            );
            assert_eq!(client.get_config(), config);
        }

        /// A sparse struct published as event data produces a sparse event data
        /// map too.
        #[test]
        fn sparse_config_event() {
            let env = Env::default();
            let id = register_storage(&env);
            let client = SClient::new(&env, &id);
            let owner = Address::generate(&env);

            client.emit_config(&Config {
                owner: owner.clone(),
                fee_bps: None,
                max_amount: Some(100),
                min_amount: None,
                paused: None,
            });

            assert_eq!(
                env.events().all(),
                vec![
                    &env,
                    (
                        id,
                        (Symbol::new(&env, "config_updated"),).into_val(&env),
                        map![
                            &env,
                            (Symbol::new(&env, "max_amount"), as_val(&env, &100i128)),
                            (Symbol::new(&env, "owner"), as_val(&env, &owner)),
                        ]
                        .to_val(),
                    ),
                ]
            );
        }

        // --- Cross contract: evolving the interface of a called function.

        /// A caller that has not been updated yet sends the old struct, and the
        /// callee reads the added field as `None`.
        #[test]
        fn policy_reads_v2_from_v1_caller() {
            let env = Env::default();
            let policy = register_policy(&env);
            let caller = CClient::new(&env, &register_caller(&env));

            assert_eq!(
                caller.call_with_v1(&policy, &Symbol::new(&env, "check_v2"), &10),
                30
            );
        }

        /// An updated caller sends the field, and the callee that still reads it
        /// as an `Option` sees it as `Some`.
        #[test]
        fn policy_reads_v2_from_v3_caller() {
            let env = Env::default();
            let policy = register_policy(&env);
            let caller = CClient::new(&env, &register_caller(&env));

            assert_eq!(
                caller.call_with_v3(&policy, &Symbol::new(&env, "check_v2"), &10, &5),
                35
            );
        }

        /// A callee that has not been updated yet ignores the field the caller
        /// added.
        #[test]
        fn policy_reads_v1_from_v3_caller() {
            let env = Env::default();
            let policy = register_policy(&env);
            let caller = CClient::new(&env, &register_caller(&env));

            assert_eq!(
                caller.call_with_v3(&policy, &Symbol::new(&env, "check_v1"), &10, &5),
                30
            );
        }

        /// A callee that requires the field cannot be called by a caller that
        /// does not send it.
        #[test]
        fn policy_v3_from_v1_caller_fails() {
            let env = Env::default();
            let policy = register_policy(&env);
            let caller = CClient::new(&env, &register_caller(&env));

            assert!(caller
                .try_call_with_v1(&policy, &Symbol::new(&env, "check_v3"), &10)
                .is_err());
        }

        /// A caller that sends the field as a set `Option` can call a callee that
        /// requires it.
        #[test]
        fn policy_v3_from_v2_caller_some() {
            let env = Env::default();
            let policy = register_policy(&env);
            let caller = CClient::new(&env, &register_caller(&env));

            assert_eq!(
                caller.call_with_v2(&policy, &Symbol::new(&env, "check_v3"), &10, &Some(5)),
                35
            );
        }

        /// An explicitly stored `Void` reads the same way as an absent key: a
        /// callee that requires the field cannot be called with `None`.
        #[test]
        fn policy_v3_from_v2_caller_none_fails() {
            let env = Env::default();
            let policy = register_policy(&env);
            let caller = CClient::new(&env, &register_caller(&env));

            assert!(caller
                .try_call_with_v2(&policy, &Symbol::new(&env, "check_v3"), &10, &None)
                .is_err());
        }
    };
}
