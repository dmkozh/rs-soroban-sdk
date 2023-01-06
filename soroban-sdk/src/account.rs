use core::cmp::Ordering;

use super::{
    env::internal::{Env as _, EnvBase as _},
    env::IntoVal,
    xdr::ScObjectType,
    ConversionError, Env, Object, RawVal, TryFromVal, TryIntoVal,
};

use crate::{Address, BytesN, Vec};

#[cfg(not(target_family = "wasm"))]
use crate::env::internal::xdr::{Hash, ScAccount, ScAccountId, ScVal, ScVec};

#[derive(Clone)]
pub struct Account {
    env: Env,
    obj: Object,
}

impl Eq for Account {}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl PartialOrd for Account {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for Account {
    fn cmp(&self, other: &Self) -> Ordering {
        self.env.check_same_env(&other.env);
        let v = self.env.obj_cmp(self.obj.to_raw(), other.obj.to_raw());
        v.cmp(&0)
    }
}

impl TryFromVal<Env, Object> for Account {
    type Error = ConversionError;

    fn try_from_val(env: &Env, obj: Object) -> Result<Self, Self::Error> {
        if obj.is_obj_type(ScObjectType::Account) {
            Ok(Account {
                env: env.clone(),
                obj,
            })
        } else {
            Err(ConversionError {})
        }
    }
}

impl TryIntoVal<Env, Account> for Object {
    type Error = <Account as TryFromVal<Env, Object>>::Error;

    fn try_into_val(self, env: &Env) -> Result<Account, Self::Error> {
        <_ as TryFromVal<_, Object>>::try_from_val(env, self)
    }
}

impl TryFromVal<Env, RawVal> for Account {
    type Error = <Account as TryFromVal<Env, Object>>::Error;

    fn try_from_val(env: &Env, val: RawVal) -> Result<Self, Self::Error> {
        <_ as TryFromVal<_, Object>>::try_from_val(env, val.try_into()?)
    }
}

impl TryIntoVal<Env, Account> for RawVal {
    type Error = <Account as TryFromVal<Env, Object>>::Error;

    fn try_into_val(self, env: &Env) -> Result<Account, Self::Error> {
        <_ as TryFromVal<_, RawVal>>::try_from_val(env, self)
    }
}

impl IntoVal<Env, Object> for Account {
    fn into_val(self, _env: &Env) -> Object {
        self.to_object()
    }
}

impl IntoVal<Env, Object> for &Account {
    fn into_val(self, _env: &Env) -> Object {
        self.to_object()
    }
}

impl IntoVal<Env, RawVal> for Account {
    fn into_val(self, _env: &Env) -> RawVal {
        self.to_raw()
    }
}

impl IntoVal<Env, RawVal> for &Account {
    fn into_val(self, _env: &Env) -> RawVal {
        self.to_raw()
    }
}

#[cfg(not(target_family = "wasm"))]
impl TryFrom<&Account> for ScVal {
    type Error = ConversionError;
    fn try_from(v: &Account) -> Result<Self, Self::Error> {
        ScVal::try_from_val(&v.env, v.obj.to_raw())
    }
}

#[cfg(not(target_family = "wasm"))]
impl TryFrom<Account> for ScVal {
    type Error = ConversionError;
    fn try_from(v: Account) -> Result<Self, Self::Error> {
        (&v).try_into()
    }
}

#[cfg(not(target_family = "wasm"))]
impl TryFromVal<Env, ScVal> for Account {
    type Error = ConversionError;
    fn try_from_val(env: &Env, val: ScVal) -> Result<Self, Self::Error> {
        <_ as TryFromVal<_, Object>>::try_from_val(
            env,
            val.try_into_val(env).map_err(|_| ConversionError)?,
        )
    }
}

#[cfg(not(target_family = "wasm"))]
impl TryIntoVal<Env, Account> for ScVal {
    type Error = ConversionError;
    fn try_into_val(self, env: &Env) -> Result<Account, Self::Error> {
        Account::try_from_val(env, self)
    }
}

impl Account {
    #[inline(always)]
    pub(crate) unsafe fn unchecked_new(env: Env, obj: Object) -> Self {
        Self { env, obj }
    }

    #[inline(always)]
    pub fn env(&self) -> &Env {
        &self.env
    }

    pub fn as_raw(&self) -> &RawVal {
        self.obj.as_raw()
    }

    pub fn to_raw(&self) -> RawVal {
        self.obj.to_raw()
    }

    pub fn as_object(&self) -> &Object {
        &self.obj
    }

    pub fn to_object(&self) -> Object {
        self.obj
    }

    pub fn address(&self) -> Address {
        self.env.get_account_address(&self)
    }

    pub fn authorize(&self, args: Vec<RawVal>) {
        self.env.authorize_account(&self, args);
    }
}

#[cfg(all(not(target_family = "wasm"), any(test, feature = "testutils")))]
impl Account {
    pub fn random(e: &Env) -> Self {
        use crate::env::internal::xdr::ScObject;
        use crate::testutils::random;
        let sc_account = ScAccount {
            account_id: ScAccountId::BuiltinEd25519(Hash(random())),
            invocations: vec![].try_into().unwrap(),
            signature_args: ScVec(vec![].try_into().unwrap()),
        };
        Account::try_from_val(e, ScVal::Object(Some(ScObject::Account(sc_account)))).unwrap()
    }

    pub fn generic(e: &Env, contract_id: &BytesN<32>) -> Self {
        use crate::env::internal::xdr::ScObject;
        let sc_account = ScAccount {
            account_id: ScAccountId::GenericAccount(Hash(contract_id.to_array())),
            invocations: vec![].try_into().unwrap(),
            signature_args: ScVec(vec![].try_into().unwrap()),
        };
        Account::try_from_val(e, ScVal::Object(Some(ScObject::Account(sc_account)))).unwrap()
    }
}
