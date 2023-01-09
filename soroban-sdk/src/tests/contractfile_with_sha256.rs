use crate as soroban_sdk;
pub const WASM: &[u8] = soroban_sdk::contractfile!(
    file = "../target/wasm32-unknown-unknown/release/test_add_u64.wasm",
    sha256 = "93a0286b5a52ef2303b464cc79c5d96486ea4dc5f2e6dc0d59ebebf4a8de41a3",
);

#[test]
fn test_spec() {
    assert!(WASM.len() > 0);
}
