use super::U256;
use crate::error::UniswapV3MathError;
use ruint::uint;
use std::ops::ShrAssign;

pub fn most_significant_bit(mut x: U256) -> Result<u8, UniswapV3MathError> {
    let mut r = 0;

    if x == U256::ZERO {
        return Err(UniswapV3MathError::ZeroValue)
    }

    if x >= uint!(0x100000000000000000000000000000000_U256) {
        x.shr_assign(128);
        r += 128;
    }

    if x >= uint!(0x10000000000000000_U256) {
        x.shr_assign(64);
        r += 64;
    }

    if x >= uint!(0x100000000_U256) {
        x.shr_assign(32);
        r += 32;
    }

    if x >= uint!(0x10000_U256) {
        x.shr_assign(16);
        r += 16;
    }

    if x >= uint!(0x100_U256) {
        x.shr_assign(8);
        r += 8;
    }

    if x >= uint!(0x10_U256) {
        x.shr_assign(4);
        r += 4;
    }
    if x >= uint!(0x4_U256) {
        x.shr_assign(2);
        r += 2;
    }

    if x >= uint!(0x2U256) {
        r += 1;
    }

    Ok(r)
}

pub fn least_significant_bit(mut x: U256) -> Result<u8, UniswapV3MathError> {
    if x == U256::ZERO {
        return Err(UniswapV3MathError::ZeroValue)
    }

    let mut r = 255;

    //TODO: update this to use constants for each U256 comparison

    if x & U256::from(u128::MAX) > U256::ZERO {
        r -= 128;
    } else {
        x >>= 128;
    }

    if x & U256::from(u64::MAX) > U256::ZERO {
        r -= 64;
    } else {
        x >>= 64;
    }

    if x & U256::from(u32::MAX) > U256::ZERO {
        r -= 32;
    } else {
        x >>= 32;
    }

    if x & U256::from(u16::MAX) > U256::ZERO {
        r -= 16;
    } else {
        x >>= 16;
    }

    if x & U256::from(u8::MAX) > U256::ZERO {
        r -= 8;
    } else {
        x >>= 8;
    }

    if x & uint!(0xf_U256) > U256::ZERO {
        r -= 4;
    } else {
        x >>= 4;
    }

    if x & uint!(0x3_U256) > U256::ZERO {
        r -= 2;
    } else {
        x >>= 2;
    }

    if x & uint!(0x1_U256) > U256::ZERO {
        r -= 1;
    }

    Ok(r)
}

#[cfg(test)]
mod test {
    use super::{most_significant_bit, U256};
    use crate::{
        bit_math::least_significant_bit,
        utils::{RUINT_ONE, RUINT_TWO},
    };
    use ruint::uint;

    #[test]
    fn test_most_significant_bit() {
        //0
        let result = most_significant_bit(U256::ZERO);
        assert_eq!(
            result.unwrap_err().to_string(),
            "Can not get most significant bit or least significant bit on zero value"
        );

        //1
        let result = most_significant_bit(RUINT_ONE);
        assert_eq!(result.unwrap(), 0);

        //2
        let result = most_significant_bit(RUINT_TWO);
        assert_eq!(result.unwrap(), 1);

        //all powers of 2
        for i in 0..=255 {
            let result = most_significant_bit(RUINT_TWO.pow(U256::from(i)));
            assert_eq!(result.unwrap(), i as u8);
        }

        //uint256(-1)
        let result = most_significant_bit(uint!(
            115792089237316195423570985008687907853269984665640564039457584007913129639935_U256
        ));
        assert_eq!(result.unwrap(), 255);
    }

    #[test]
    fn test_least_significant_bit() {
        //0
        let result = least_significant_bit(U256::ZERO);
        assert_eq!(
            result.unwrap_err().to_string(),
            "Can not get most significant bit or least significant bit on zero value"
        );

        //1
        let result = least_significant_bit(RUINT_ONE);
        assert_eq!(result.unwrap(), 0);

        //2
        let result = least_significant_bit(RUINT_TWO);
        assert_eq!(result.unwrap(), 1);

        //all powers of 2
        for i in 0..=255 {
            let result = least_significant_bit(RUINT_TWO.pow(U256::from(i)));
            assert_eq!(result.unwrap(), i as u8);
        }

        //uint256(-1)
        let result = least_significant_bit(uint!(
            115792089237316195423570985008687907853269984665640564039457584007913129639935_U256
        ));
        assert_eq!(result.unwrap(), 0);
    }
}
