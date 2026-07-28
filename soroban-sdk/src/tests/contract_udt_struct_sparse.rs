use crate::{self as soroban_sdk};
use soroban_sdk::{
    contracttype, map,
    xdr::{FromXdr, ScVal, ToXdr},
    Env, FromVal, IntoVal, Map, Symbol, TryFromVal, Val, Vec,
};

/// Written as a sparse map: the fields that are `None` are left out of the map.
#[contracttype(sparse = true)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sparse {
    pub a: u32,
    pub b: Option<u32>,
    pub c: Option<u32>,
}

/// The same schema written as a dense map: the fields that are `None` are stored
/// as explicit `Void` values.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dense {
    pub a: u32,
    pub b: Option<u32>,
    pub c: Option<u32>,
}

/// A reader of a subset of the fields of [`Sparse`].
#[contracttype(sparse = true)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Narrow {
    pub a: u32,
}

fn keys_of(env: &Env, val: &Val) -> Vec<Symbol> {
    Map::<Symbol, Val>::from_val(env, val).keys()
}

fn u32_val(env: &Env, u: u32) -> Val {
    u.into_val(env)
}

fn syms(env: &Env, names: &[&str]) -> Vec<Symbol> {
    let mut v = Vec::new(env);
    for name in names {
        v.push_back(Symbol::new(env, name));
    }
    v
}

#[test]
fn test_sparse_omits_none() {
    let env = Env::default();

    let val: Val = Sparse {
        a: 1,
        b: None,
        c: None,
    }
    .into_val(&env);
    assert_eq!(keys_of(&env, &val), syms(&env, &["a"]));

    let val: Val = Sparse {
        a: 1,
        b: None,
        c: Some(3),
    }
    .into_val(&env);
    assert_eq!(keys_of(&env, &val), syms(&env, &["a", "c"]));

    let val: Val = Sparse {
        a: 1,
        b: Some(2),
        c: Some(3),
    }
    .into_val(&env);
    assert_eq!(keys_of(&env, &val), syms(&env, &["a", "b", "c"]));
}

#[test]
fn test_dense_keeps_none() {
    let env = Env::default();

    let val: Val = Dense {
        a: 1,
        b: None,
        c: None,
    }
    .into_val(&env);
    assert_eq!(keys_of(&env, &val), syms(&env, &["a", "b", "c"]));

    let map = Map::<Symbol, Val>::from_val(&env, &val);
    assert!(map.get_unchecked(Symbol::new(&env, "b")).is_void());
    assert!(map.get_unchecked(Symbol::new(&env, "c")).is_void());
}

/// The sparse and dense forms of the same schema read into each other, so
/// `sparse` can be turned on or off for a struct without invalidating values that
/// have already been written.
#[test]
fn test_sparse_dense_interop() {
    let env = Env::default();

    for (b, c) in [
        (None, None),
        (Some(2), None),
        (None, Some(3)),
        (Some(2), Some(3)),
    ] {
        let sparse = Sparse { a: 1, b, c };
        let dense = Dense { a: 1, b, c };
        let sparse_val: Val = sparse.clone().into_val(&env);
        let dense_val: Val = dense.clone().into_val(&env);

        assert_eq!(Sparse::from_val(&env, &sparse_val), sparse);
        assert_eq!(Dense::from_val(&env, &dense_val), dense);
        assert_eq!(Dense::from_val(&env, &sparse_val), dense);
        assert_eq!(Sparse::from_val(&env, &dense_val), sparse);
    }
}

/// A field that is absent from the map reads as `None`, and a key of the map that
/// is not a field of the struct is ignored.
#[test]
fn test_read_narrower_and_wider_maps() {
    let env = Env::default();

    // Narrower map: only `a` is present.
    let val = map![&env, (Symbol::new(&env, "a"), u32_val(&env, 1))].to_val();
    assert_eq!(
        Sparse::from_val(&env, &val),
        Sparse {
            a: 1,
            b: None,
            c: None,
        }
    );

    // Wider map: `z` is not a field of either struct.
    let val = map![
        &env,
        (Symbol::new(&env, "a"), u32_val(&env, 1)),
        (Symbol::new(&env, "b"), u32_val(&env, 2)),
        (Symbol::new(&env, "c"), u32_val(&env, 3)),
        (Symbol::new(&env, "z"), u32_val(&env, 4)),
    ]
    .to_val();
    assert_eq!(
        Sparse::from_val(&env, &val),
        Sparse {
            a: 1,
            b: Some(2),
            c: Some(3),
        }
    );
    assert_eq!(Narrow::from_val(&env, &val), Narrow { a: 1 });
}

/// A field that is absent from the map and is not an `Option` is a conversion
/// error.
#[test]
fn test_read_missing_non_option_field_errors() {
    let env = Env::default();
    let val = map![&env, (Symbol::new(&env, "b"), u32_val(&env, 2))].to_val();
    assert!(Sparse::try_from_val(&env, &val).is_err());
}

/// The XDR form of a struct matches its host `Map` form, sparse or dense.
#[test]
fn test_scval_conversion() {
    let env = Env::default();

    let sparse = Sparse {
        a: 1,
        b: None,
        c: Some(3),
    };
    let dense = Dense {
        a: 1,
        b: None,
        c: Some(3),
    };
    let sparse_scval: ScVal = (&sparse).try_into().unwrap();
    let dense_scval: ScVal = (&dense).try_into().unwrap();

    // The `ScVal` of each struct is the `ScVal` of the host map it converts to.
    let sparse_val: Val = sparse.clone().into_val(&env);
    let dense_val: Val = dense.clone().into_val(&env);
    assert_eq!(
        sparse_scval,
        ScVal::try_from_val(&env, &sparse_val).unwrap()
    );
    assert_eq!(dense_scval, ScVal::try_from_val(&env, &dense_val).unwrap());
    assert_ne!(sparse_scval, dense_scval);

    // Both forms read back into either struct.
    assert_eq!(Sparse::try_from_val(&env, &sparse_scval).unwrap(), sparse);
    assert_eq!(Sparse::try_from_val(&env, &dense_scval).unwrap(), sparse);
    assert_eq!(Dense::try_from_val(&env, &sparse_scval).unwrap(), dense);
    assert_eq!(Dense::try_from_val(&env, &dense_scval).unwrap(), dense);
    assert_eq!(
        Narrow::try_from_val(&env, &dense_scval).unwrap(),
        Narrow { a: 1 }
    );
}

#[test]
fn test_xdr_roundtrip() {
    let env = Env::default();

    let sparse = Sparse {
        a: 1,
        b: None,
        c: None,
    };
    let bytes = sparse.clone().to_xdr(&env);
    assert_eq!(Sparse::from_xdr(&env, &bytes), Ok(sparse.clone()));
    assert_eq!(
        Dense::from_xdr(&env, &bytes),
        Ok(Dense {
            a: 1,
            b: None,
            c: None,
        })
    );
    assert_eq!(Narrow::from_xdr(&env, &bytes), Ok(Narrow { a: 1 }));

    // The sparse form is smaller than the dense form of the same value.
    let dense_bytes = Dense {
        a: 1,
        b: None,
        c: None,
    }
    .to_xdr(&env);
    assert!(bytes.len() < dense_bytes.len());
}
