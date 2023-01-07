use soroban_sdk::{contractimpl, contracttype, Account, Address, BytesN, Env, IntoVal};

mod token_contract {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/release/soroban_token_spec.wasm"
    );
    pub type TokenClient = Client;
}

use token_contract::TokenClient;

#[contracttype]
pub enum DataKey {
    Token,
}

fn get_token(e: &Env) -> BytesN<32> {
    e.storage().get_unchecked(DataKey::Token).unwrap()
}

pub struct TestContract;

#[contractimpl]
impl TestContract {
    pub fn init(e: Env, contract: BytesN<32>) {
        e.storage().set(DataKey::Token, contract);
    }

    pub fn get_token(e: Env) -> BytesN<32> {
        get_token(&e)
    }

    pub fn incr_allow(e: Env, acc: Account, spender: Address, amount: i128) {
        TokenClient::new(&e, get_token(&e)).incr_allow(&acc, &spender, &amount);
    }

    pub fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        TokenClient::new(&e, get_token(&e)).allowance(&from, &spender)
    }
}

#[test]
fn test() {
    use soroban_sdk::xdr::Asset;

    let env = Env::default();

    let token_contract_id = env.register_stellar_asset_contract(Asset::Native);

    let contract_id = env.register_contract(None, TestContract);
    let client = TestContractClient::new(&env, &contract_id);
    client.init(&token_contract_id);

    let token_client = TokenClient::new(&env, &client.get_token());
    assert_eq!(token_client.name(), "native".into_val(&env));

    let acc = Account::random(&env);
    let spender = Account::random(&env).address();
    client.incr_allow(&acc, &spender, &20);

    // Smoke test check that authorization with wrong args didn't happen.
    assert!(!env.verify_account_authorization(
        &acc,
        &[(&token_client.contract_id, "incr_allow")],
        (&spender, 19_i128).into_val(&env),
    ));
    assert!(env.verify_account_authorization(
        &acc,
        &[(&token_client.contract_id, "incr_allow")],
        (&spender, 20_i128).into_val(&env),
    ));
    // Smoke test check that double authorization didn't happen.
    assert!(!env.verify_account_authorization(
        &acc,
        &[(&token_client.contract_id, "incr_allow")],
        (&spender, 20_i128).into_val(&env),
    ));

    assert_eq!(client.allowance(&acc.address(), &spender), 20);
    assert_eq!(token_client.allowance(&acc.address(), &spender), 20);
}
