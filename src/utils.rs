use alloy_primitives::I256;
use reth_primitives::U256;

pub const RUINT_ZERO: U256 = U256::ZERO;
pub const RUINT_ONE: U256 = U256::from_limbs([1, 0, 0, 0]);
pub const RUINT_TWO: U256 = U256::from_limbs([2, 0, 0, 0]);
pub const RUINT_THREE: U256 = U256::from_limbs([3, 0, 0, 0]);
pub const RUINT_MAX_U256: U256 = U256::from_limbs([
    18446744073709551615,
    18446744073709551615,
    18446744073709551615,
    18446744073709551615,
]);

pub fn u256_to_i256(u: U256) -> I256 {
    I256::from_raw(alloy_primitives::U256::from_limbs(u.into_limbs()))
}
pub fn i256_to_u256(i: I256) -> U256 {
    U256::from_limbs(i.into_raw().into_limbs())
}
