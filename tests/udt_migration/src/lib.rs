//! Contracts exercising the schema-evolution behaviour of map-backed UDT
//! structs, which is built on the `sparse_map_new_from_linear_memory` and
//! `sparse_map_unpack_to_linear_memory` host functions (CAP-86).
//!
//! Reading a struct always tolerates schema differences: a map key that is not
//! a field of the struct is ignored, and a field that is not a key of the map
//! reads as `Void`, i.e. `None` for `Option` fields. Writing a struct includes
//! every field by default, or omits the `None` fields when the type is declared
//! with `#[contracttype(sparse = true)]`.
//!
//! The scenarios below follow the migration examples in the appendix of CAP-86,
//! but keep everything within a couple of contracts and no contract upgrades:
//! storage migrations are modelled by reading and writing the same data key with
//! different struct versions, and interface migrations by two contracts calling
//! each other with different struct versions.
#![no_std]
use soroban_sdk::{
    contract, contractevent, contractimpl, contracttype, symbol_short, vec, Address, Env, IntoVal,
    Map, Symbol, Val, Vec,
};

/// The data key that every version of the admin info is stored under.
const ADMIN: Symbol = symbol_short!("ADMIN");
/// The data key that every version of the config is stored under.
const CONFIG: Symbol = symbol_short!("CONFIG");

// --- Storage migration types, from the "Adding new storage fields to a UDT
// --- struct" example of CAP-86.

/// The original admin info: a single admin.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminInfoV1 {
    pub admin: Address,
    pub is_frozen: bool,
}

/// The admin info during the migration to multiple admins. `admins` is added as
/// an `Option` so that a value written as an [`AdminInfoV1`] still reads, with
/// `admins` set to `None`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminInfoV2 {
    /// Deprecated, superseded by `admins`.
    pub admin: Address,
    pub admins: Option<Vec<Address>>,
    pub is_frozen: bool,
}

/// The admin info after the migration: the deprecated `admin` field is gone, and
/// `admins` is no longer optional. A value written as an [`AdminInfoV2`] with
/// `admins` set reads as this type, with the `admin` key ignored.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminInfoV3 {
    pub admins: Vec<Address>,
    pub is_frozen: bool,
}

// --- Sparse ("compact") config types.

/// A config that is mostly optional overrides, so it is stored as a sparse map:
/// the fields that are `None` are left out of the map entirely.
#[contracttype(sparse = true)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub owner: Address,
    pub fee_bps: Option<u32>,
    pub max_amount: Option<i128>,
    pub min_amount: Option<i128>,
    pub paused: Option<bool>,
}

/// A reader of only the fields of [`Config`] that this version of the contract
/// cares about.
#[contracttype(sparse = true)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigOwner {
    pub owner: Address,
}

/// Published with the whole [`Config`] as the event data, so that the event data
/// map is sparse too.
#[contractevent(topics = ["config_updated"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigUpdated {
    pub config: Config,
}

#[contract]
pub struct Storage;

#[contractimpl]
impl Storage {
    pub fn set_v1(env: Env, info: AdminInfoV1) {
        env.storage().persistent().set(&ADMIN, &info);
    }

    pub fn get_v1(env: Env) -> AdminInfoV1 {
        env.storage().persistent().get(&ADMIN).unwrap()
    }

    pub fn set_v2(env: Env, info: AdminInfoV2) {
        env.storage().persistent().set(&ADMIN, &info);
    }

    pub fn get_v2(env: Env) -> AdminInfoV2 {
        env.storage().persistent().get(&ADMIN).unwrap()
    }

    pub fn set_v3(env: Env, info: AdminInfoV3) {
        env.storage().persistent().set(&ADMIN, &info);
    }

    pub fn get_v3(env: Env) -> AdminInfoV3 {
        env.storage().persistent().get(&ADMIN).unwrap()
    }

    /// Seeds `admins` from the deprecated `admin` field, in place, so that the
    /// stored value can subsequently be read as an [`AdminInfoV3`].
    pub fn migrate_admins(env: Env) {
        let mut info: AdminInfoV2 = env.storage().persistent().get(&ADMIN).unwrap();
        if info.admins.is_none() {
            info.admins = Some(vec![&env, info.admin.clone()]);
        }
        env.storage().persistent().set(&ADMIN, &info);
    }

    /// The keys of the stored admin info map, to observe the stored schema.
    pub fn admin_keys(env: Env) -> Vec<Symbol> {
        Self::keys_of(&env, &ADMIN)
    }

    pub fn set_config(env: Env, config: Config) {
        env.storage().persistent().set(&CONFIG, &config);
    }

    pub fn get_config(env: Env) -> Config {
        env.storage().persistent().get(&CONFIG).unwrap()
    }

    /// Reads a stored [`Config`] into a struct with a subset of its fields.
    pub fn get_config_owner(env: Env) -> ConfigOwner {
        env.storage().persistent().get(&CONFIG).unwrap()
    }

    /// The keys of the stored config map, to observe that `None` fields are not
    /// stored at all.
    pub fn config_keys(env: Env) -> Vec<Symbol> {
        Self::keys_of(&env, &CONFIG)
    }

    pub fn emit_config(env: Env, config: Config) {
        ConfigUpdated { config }.publish(&env);
    }

    fn keys_of(env: &Env, key: &Symbol) -> Vec<Symbol> {
        let map: Map<Symbol, Val> = env.storage().persistent().get(key).unwrap();
        map.keys()
    }
}

// --- Interface migration types, from the "Extending the interface of a contract
// --- function" example of CAP-86.

/// The original protocol state.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolStateV1 {
    pub total_debt: i128,
    pub total_collateral: i128,
}

/// The protocol state during the migration: `total_liquidation` is added as an
/// `Option` so that the policy contract can be updated before its callers.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolStateV2 {
    pub total_debt: i128,
    pub total_collateral: i128,
    pub total_liquidation: Option<i128>,
}

/// The protocol state after the migration: `total_liquidation` is required.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolStateV3 {
    pub total_debt: i128,
    pub total_collateral: i128,
    pub total_liquidation: i128,
}

#[contract]
pub struct Policy;

#[contractimpl]
impl Policy {
    /// Reads the state as a [`ProtocolStateV1`], ignoring any newer fields the
    /// caller sent.
    pub fn check_v1(state: ProtocolStateV1) -> i128 {
        state.total_debt + state.total_collateral
    }

    /// Reads the state as a [`ProtocolStateV2`], accepting callers that have not
    /// been updated to send `total_liquidation` yet.
    pub fn check_v2(state: ProtocolStateV2) -> i128 {
        state.total_debt + state.total_collateral + state.total_liquidation.unwrap_or(0)
    }

    /// Reads the state as a [`ProtocolStateV3`], requiring `total_liquidation`.
    pub fn check_v3(state: ProtocolStateV3) -> i128 {
        state.total_debt + state.total_collateral + state.total_liquidation
    }
}

#[contract]
pub struct Caller;

#[contractimpl]
impl Caller {
    /// Calls `policy_fn` with a [`ProtocolStateV1`], i.e. as a caller that has
    /// not been updated yet.
    pub fn call_with_v1(env: Env, policy: Address, policy_fn: Symbol, total_debt: i128) -> i128 {
        let state = ProtocolStateV1 {
            total_debt,
            total_collateral: 20,
        };
        env.invoke_contract(&policy, &policy_fn, vec![&env, state.into_val(&env)])
    }

    /// Calls `policy_fn` with a [`ProtocolStateV2`].
    pub fn call_with_v2(
        env: Env,
        policy: Address,
        policy_fn: Symbol,
        total_debt: i128,
        total_liquidation: Option<i128>,
    ) -> i128 {
        let state = ProtocolStateV2 {
            total_debt,
            total_collateral: 20,
            total_liquidation,
        };
        env.invoke_contract(&policy, &policy_fn, vec![&env, state.into_val(&env)])
    }

    /// Calls `policy_fn` with a [`ProtocolStateV3`], i.e. as a fully updated
    /// caller.
    pub fn call_with_v3(
        env: Env,
        policy: Address,
        policy_fn: Symbol,
        total_debt: i128,
        total_liquidation: i128,
    ) -> i128 {
        let state = ProtocolStateV3 {
            total_debt,
            total_collateral: 20,
            total_liquidation,
        };
        env.invoke_contract(&policy, &policy_fn, vec![&env, state.into_val(&env)])
    }
}

#[cfg(test)]
#[macro_use]
mod test_macros;

#[cfg(test)]
mod test;

#[cfg(test)]
mod test_with_wasm;
