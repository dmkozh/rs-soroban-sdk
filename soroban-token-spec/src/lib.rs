#![no_std]

use soroban_sdk::{contractimpl, contracttype, Account, Address, Bytes, Env};

mod tests;

/// The interface below was copied from
/// https://github.com/stellar/rs-soroban-env/blob/main/soroban-env-host/src/native_contract/token/contract.rs
/// at commit b3c188f48dec51a956c1380fb6fe92201a3f716b.
///
/// Differences between this interface and the built-in contract
/// 1. The return values here don't return Results.
/// 2. The implementations have been replaced with a panic.
/// 3. &Host type usage are replaced with Env
pub struct Token;

#[contractimpl]
#[allow(unused_variables)]
impl Token {
    /// Init creates a token contract that does not wrap an asset on the classic
    /// side. No checks are done on the contractID.
    pub fn init(env: Env, admin: Identifier, metadata: TokenMetadata) {
        panic!("calling into interface");
    }

    pub fn nonce(env: Env, id: Identifier) -> i128 {
        panic!("calling into interface");
    }

    pub fn allowance(env: Env, from: Identifier, spender: Identifier) -> i128 {
        panic!("calling into interface");
    }

    pub fn approve(env: Env, from: Signature, nonce: i128, spender: Identifier, amount: i128) {
        panic!("calling into interface");
    }

    pub fn is_frozen(env: Env, id: Address) -> bool {
        panic!("calling into interface");
    }

    pub fn is_frozen(env: Env, id: Identifier) -> bool {
        panic!("calling into interface");
    }

    pub fn xfer_from(env: Env, spender: Account, from: Address, to: Address, amount: i128) {
        panic!("calling into interface");
    }

    pub fn burn(env: Env, admin: Account, from: Address, amount: i128) {
        panic!("calling into interface");
    }

    pub fn burn(env: Env, admin: Signature, nonce: i128, from: Identifier, amount: i128) {
        panic!("calling into interface");
    }

    pub fn freeze(env: Env, admin: Signature, nonce: i128, id: Identifier) {
        panic!("calling into interface");
    }

    pub fn set_admin(env: Env, admin: Account, new_admin: Address) {
        panic!("calling into interface");
    }

    pub fn set_admin(env: Env, admin: Signature, nonce: i128, new_admin: Identifier) {
        panic!("calling into interface");
    }

    pub fn unfreeze(env: Env, admin: Signature, nonce: i128, id: Identifier) {
        panic!("calling into interface");
    }

    pub fn decimals(env: Env) -> u32 {
        panic!("calling into interface");
    }

    pub fn name(env: Env) -> Bytes {
        panic!("calling into interface");
    }

    pub fn symbol(env: Env) -> Bytes {
        panic!("calling into interface");
    }

    pub fn import(env: Env, id: Signature, nonce: i128, amount: i64) {
        panic!("calling into interface");
    }

    pub fn export(env: Env, id: Signature, nonce: i128, amount: i64) {
        panic!("calling into interface");
    }
}

const SPEC_SIZE: usize = 984;

/// Returns the XDR spec for the Token contract.
#[doc(hidden)]
pub const fn spec_xdr() -> [u8; SPEC_SIZE] {
    let input: &[&[u8]] = &[
        &Token::spec_xdr_allowance(),
        &Token::spec_xdr_authorized(),
        &Token::spec_xdr_balance(),
        &Token::spec_xdr_burn(),
        &Token::spec_xdr_burn_from(),
        &Token::spec_xdr_clawback(),
        &Token::spec_xdr_decimals(),
        &Token::spec_xdr_decr_allow(),
        &Token::spec_xdr_incr_allow(),
        &Token::spec_xdr_mint(),
        &Token::spec_xdr_name(),
        &Token::spec_xdr_set_admin(),
        &Token::spec_xdr_set_auth(),
        &Token::spec_xdr_spendable(),
        &Token::spec_xdr_symbol(),
        &Token::spec_xdr_xfer(),
        &Token::spec_xdr_xfer_from(),
    ];

    // Concatenate all XDR for each item that makes up the token spec.
    let mut output = [0u8; SPEC_SIZE];
    let mut input_i = 0;
    let mut output_i = 0;
    while input_i < input.len() {
        let subinput = input[input_i];
        let mut subinput_i = 0;
        while subinput_i < subinput.len() {
            output[output_i] = subinput[subinput_i];
            output_i += 1;
            subinput_i += 1;
        }
        input_i += 1;
    }

    // Check that the numbers of bytes written is equal to the number of bytes
    // expected in the output.
    if output_i != output.len() {
        panic!("unexpected output length",);
    }

    output
}
