use crate::{AdminInfoV1, AdminInfoV2, AdminInfoV3, Caller, Config, ConfigOwner, Policy, Storage};

type SClient<'a> = crate::StorageClient<'a>;
type CClient<'a> = crate::CallerClient<'a>;

fn register_storage(env: &soroban_sdk::Env) -> soroban_sdk::Address {
    env.register(Storage, ())
}

fn register_policy(env: &soroban_sdk::Env) -> soroban_sdk::Address {
    env.register(Policy, ())
}

fn register_caller(env: &soroban_sdk::Env) -> soroban_sdk::Address {
    env.register(Caller, ())
}

migration_tests!();
