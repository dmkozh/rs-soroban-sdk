mod contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32v1-none/release/test_udt_migration.wasm"
    );
}

use contract::{AdminInfoV1, AdminInfoV2, AdminInfoV3, Config, ConfigOwner};

type SClient<'a> = contract::Client<'a>;
type CClient<'a> = contract::Client<'a>;

fn register_storage(env: &soroban_sdk::Env) -> soroban_sdk::Address {
    env.register(contract::WASM, ())
}

fn register_policy(env: &soroban_sdk::Env) -> soroban_sdk::Address {
    env.register(contract::WASM, ())
}

fn register_caller(env: &soroban_sdk::Env) -> soroban_sdk::Address {
    env.register(contract::WASM, ())
}

migration_tests!();
