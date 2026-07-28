use soroban_sdk::{contracterror, contracttype};

#[contracttype(sparse = true)]
pub struct TupleStruct(pub u32);

#[contracttype(sparse = true)]
pub enum UnitEnum {
    A,
    B,
}

#[contracterror(sparse = true)]
#[repr(u32)]
pub enum Error {
    A = 1,
}

fn main() {}
