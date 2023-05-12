use super::U256;
use crate::utils::RUINT_ONE;

pub fn div_rounding_up(a: U256, b: U256) -> U256 {
    let (quotient, remainder) = a.div_rem(b);
    if remainder == U256::ZERO {
        quotient
    } else {
        quotient + RUINT_ONE
    }
}
