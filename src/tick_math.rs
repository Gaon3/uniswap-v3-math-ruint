use crate::utils::{u256_to_i256, RUINT_ONE};
use alloy_primitives::I256;
use reth_primitives::U256;
use ruint::uint;
use std::ops::{BitOr, Shl, Shr};

use crate::error::UniswapV3MathError;

pub const MIN_TICK: i32 = -887272;
pub const MAX_TICK: i32 = -MIN_TICK;

pub const MIN_SQRT_RATIO: U256 = U256::from_limbs([4295128739, 0, 0, 0]);
pub const MAX_SQRT_RATIO: U256 =
    U256::from_limbs([6743328256752651558, 17280870778742802505, 4294805859, 0]);

pub fn get_sqrt_ratio_at_tick(tick: i32) -> Result<U256, UniswapV3MathError> {
    let abs_tick = U256::from(tick.abs());

    if abs_tick > U256::from(MAX_TICK) {
        return Err(UniswapV3MathError::T);
    }

    let mut ratio = if abs_tick & (U256::from(0x1)) != U256::ZERO {
        uint!(0xfffcb933bd6fad37aa2d162d1a594001_U256)
    } else {
        uint!(0x100000000000000000000000000000000_U256)
    };

    if !(abs_tick & (U256::from(0x2))) == U256::ZERO {
        ratio = (ratio * uint!(0xfff97272373d413259a46990580e213a_U256)) >> 128
    }
    if !(abs_tick & (U256::from(0x4))) == U256::ZERO {
        ratio = (ratio * uint!(0xfff2e50f5f656932ef12357cf3c7fdcc_U256)) >> 128
    }
    if !(abs_tick & (U256::from(0x8))) == U256::ZERO {
        ratio = (ratio * uint!(0xffe5caca7e10e4e61c3624eaa0941cd0_U256)) >> 128
    }
    if !(abs_tick & (U256::from(0x10))) == U256::ZERO {
        ratio = (ratio * uint!(0xffcb9843d60f6159c9db58835c926644_U256)) >> 128
    }
    if !(abs_tick & (U256::from(0x20))) == U256::ZERO {
        ratio = (ratio * uint!(0xff973b41fa98c081472e6896dfb254c0_U256)) >> 128
    }
    if !(abs_tick & (U256::from(0x40))) == U256::ZERO {
        ratio = (ratio * uint!(0xff2ea16466c96a3843ec78b326b52861_U256)) >> 128
    }
    if !(abs_tick & (U256::from(0x80))) == U256::ZERO {
        ratio = (ratio * uint!(0xfe5dee046a99a2a811c461f1969c3053_U256)) >> 128
    }
    if !(abs_tick & (U256::from(0x100))) == U256::ZERO {
        ratio = (ratio * uint!(0xfcbe86c7900a88aedcffc83b479aa3a4_U256)) >> 128
    }
    if !(abs_tick & (U256::from(0x200))) == U256::ZERO {
        ratio = (ratio * uint!(0xf987a7253ac413176f2b074cf7815e54_U256)) >> 128
    }
    if !(abs_tick & (U256::from(0x400))) == U256::ZERO {
        ratio = (ratio * uint!(0xf3392b0822b70005940c7a398e4b70f3_U256)) >> 128
    }
    if !(abs_tick & (U256::from(0x800))) == U256::ZERO {
        ratio = (ratio * uint!(0xe7159475a2c29b7443b29c7fa6e889d9_U256)) >> 128
    }
    if !(abs_tick & (U256::from(0x1000))) == U256::ZERO {
        ratio = (ratio * uint!(0xd097f3bdfd2022b8845ad8f792aa5825_U256)) >> 128
    }
    if !(abs_tick & (U256::from(0x2000))) == U256::ZERO {
        ratio = (ratio * uint!(0xa9f746462d870fdf8a65dc1f90e061e5_U256)) >> 128
    }
    if !(abs_tick & (U256::from(0x4000))) == U256::ZERO {
        ratio = (ratio * uint!(0x70d869a156d2a1b890bb3df62baf32f7_U256)) >> 128
    }
    if !(abs_tick & (U256::from(0x8000))) == U256::ZERO {
        ratio = (ratio * uint!(0x31be135f97d08fd981231505542fcfa6_U256)) >> 128
    }
    if !(abs_tick & (U256::from(0x10000))) == U256::ZERO {
        ratio = (ratio * uint!(0x9aa508b5b7a84e1c677de54f3e99bc9_U256)) >> 128
    }
    if !(abs_tick & (U256::from(0x20000))) == U256::ZERO {
        ratio = (ratio * uint!(0x5d6af8dedb81196699c329225ee604_U256)) >> 128
    }
    if !(abs_tick & (U256::from(0x40000))) == U256::ZERO {
        ratio = (ratio * uint!(0x2216e584f5fa1ea926041bedfe98_U256)) >> 128
    }
    if !(abs_tick & (U256::from(0x80000))) == U256::ZERO {
        ratio = (ratio * uint!(0x48a170391f7dc42444e8fa2_U256)) >> 128
    }

    if tick > 0 {
        ratio = U256::MAX / ratio;
    }

    Ok((ratio >> 32)
        + if (ratio % (RUINT_ONE << 32)) == U256::ZERO {
            U256::ZERO
        } else {
            RUINT_ONE
        })
}

pub fn get_tick_at_sqrt_ratio(sqrt_price_x_96: U256) -> Result<i32, UniswapV3MathError> {
    if !(sqrt_price_x_96 >= MIN_SQRT_RATIO && sqrt_price_x_96 < MAX_SQRT_RATIO) {
        return Err(UniswapV3MathError::R);
    }

    let ratio = sqrt_price_x_96.shl(32);
    let mut r = ratio;
    let mut msb = U256::ZERO;

    let mut f = if r > uint!(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_U256) {
        1_usize.shl(7)
    } else {
        0
    };
    msb = msb.bitor(U256::from(f));
    r = r.shr(f);

    f = if r > uint!(0xFFFFFFFFFFFFFFFF_U256) {
        1_usize.shl(6)
    } else {
        0
    };
    msb = msb.bitor(U256::from(f));
    r = r.shr(f);

    f = if r > uint!(0xFFFFFFFF_U256) {
        1_usize.shl(5)
    } else {
        0
    };
    msb = msb.bitor(U256::from(f));
    r = r.shr(f);

    f = if r > uint!(0xFFFF_U256) {
        1_usize.shl(4)
    } else {
        0
    };
    msb = msb.bitor(U256::from(f));
    r = r.shr(f);

    f = if r > uint!(0xFF_U256) {
        1_usize.shl(3)
    } else {
        0
    };
    msb = msb.bitor(U256::from(f));
    r = r.shr(f);

    f = if r > uint!(0xF_U256) {
        1_usize.shl(2)
    } else {
        0
    };
    msb = msb.bitor(U256::from(f));
    r = r.shr(f);

    f = if r > uint!(0x3_U256) {
        1_usize.shl(1_usize)
    } else {
        0
    };
    msb = msb.bitor(U256::from(f));
    r = r.shr(f);

    f = if r > uint!(0x1_U256) { 1_usize } else { 0 };

    msb = msb.bitor(U256::from(f));
    let msb: usize = msb.to();

    r = if msb >= 128 {
        ratio.shr(msb - 127)
    } else {
        ratio.shl(127 - msb)
    };

    let mut log_2: I256 = (u256_to_i256(U256::from(msb)) - u256_to_i256(U256::from(128))).shl(64);

    for i in (51..=63).rev() {
        r = r.overflowing_mul(r).0.shr(127);
        let f = r.shr(128);
        log_2 = log_2.bitor(u256_to_i256(f.shl(i)));

        r = r.shr(f.to::<usize>());
    }

    r = r.overflowing_mul(r).0.shr(127);
    let f = r.shr(128);
    log_2 = log_2.bitor(u256_to_i256(f.shl(50)));

    let log_sqrt10001 = log_2.wrapping_mul(I256::from_dec_str("255738958999603826347141").unwrap());

    let tick_low = ((log_sqrt10001
        - I256::from_dec_str("3402992956809132418596140100660247210").unwrap())
        >> 128_u8)
        .low_i32();

    let tick_high = ((log_sqrt10001
        + I256::from_dec_str("291339464771989622907027621153398088495").unwrap())
        >> 128_u8)
        .low_i32();

    let tick = if tick_low == tick_high {
        tick_low
    } else if get_sqrt_ratio_at_tick(tick_high)? <= sqrt_price_x_96 {
        tick_high
    } else {
        tick_low
    };

    Ok(tick)
}

pub fn calculate_compressed(tick: i32, tick_spacing: i32) -> i32 {
    if tick < 0 && tick % tick_spacing != 0 {
        (tick / tick_spacing) - 1
    } else {
        tick / tick_spacing
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ruint::uint;
    use std::ops::Sub;

    #[test]
    fn get_sqrt_ratio_at_tick_bounds() {
        // the function should return an error if the tick is out of bounds
        if let Err(err) = get_sqrt_ratio_at_tick(MIN_TICK - 1) {
            assert!(matches!(err, UniswapV3MathError::T));
        } else {
            panic!("get_qrt_ratio_at_tick did not respect lower tick bound")
        }
        if let Err(err) = get_sqrt_ratio_at_tick(MAX_TICK + 1) {
            assert!(matches!(err, UniswapV3MathError::T));
        } else {
            panic!("get_qrt_ratio_at_tick did not respect upper tick bound")
        }
    }

    #[test]
    fn get_sqrt_ratio_at_tick_values() {
        // test individual values for correct results
        assert_eq!(
            get_sqrt_ratio_at_tick(MIN_TICK).unwrap(),
            U256::from(4295128739u64),
            "sqrt ratio at min incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(MIN_TICK + 1).unwrap(),
            U256::from(4295343490u64),
            "sqrt ratio at min + 1 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(MAX_TICK - 1).unwrap(),
            uint!(1461373636630004318706518188784493106690254656249_U256),
            "sqrt ratio at max - 1 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(MAX_TICK).unwrap(),
            uint!(1461446703485210103287273052203988822378723970342_U256),
            "sqrt ratio at max incorrect"
        );
        // checking hard coded values against solidity results
        assert_eq!(
            get_sqrt_ratio_at_tick(50).unwrap(),
            U256::from(79426470787362580746886972461u128),
            "sqrt ratio at 50 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(100).unwrap(),
            U256::from(79625275426524748796330556128u128),
            "sqrt ratio at 100 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(250).unwrap(),
            U256::from(80224679980005306637834519095u128),
            "sqrt ratio at 250 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(500).unwrap(),
            U256::from(81233731461783161732293370115u128),
            "sqrt ratio at 500 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(1000).unwrap(),
            U256::from(83290069058676223003182343270u128),
            "sqrt ratio at 1000 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(2500).unwrap(),
            U256::from(89776708723587163891445672585u128),
            "sqrt ratio at 2500 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(3000).unwrap(),
            U256::from(92049301871182272007977902845u128),
            "sqrt ratio at 3000 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(4000).unwrap(),
            U256::from(96768528593268422080558758223u128),
            "sqrt ratio at 4000 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(5000).unwrap(),
            U256::from(101729702841318637793976746270u128),
            "sqrt ratio at 5000 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(50000).unwrap(),
            U256::from(965075977353221155028623082916u128),
            "sqrt ratio at 50000 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(150000).unwrap(),
            U256::from(143194173941309278083010301478497u128),
            "sqrt ratio at 150000 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(250000).unwrap(),
            U256::from(21246587762933397357449903968194344u128),
            "sqrt ratio at 250000 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(500000).unwrap(),
            uint!(5697689776495288729098254600827762987878_U256),
            "sqrt ratio at 500000 incorrect"
        );
        assert_eq!(
            get_sqrt_ratio_at_tick(738203).unwrap(),
            uint!(847134979253254120489401328389043031315994541_U256),
            "sqrt ratio at 738203 incorrect"
        );
    }

    #[test]
    pub fn test_get_tick_at_sqrt_ratio() {
        //throws for too low
        let result = get_tick_at_sqrt_ratio(MIN_SQRT_RATIO.sub(RUINT_ONE));
        assert_eq!(result.unwrap_err().to_string(), "Second inequality must be < because the price can never reach the price at the max tick");

        //throws for too high
        let result = get_tick_at_sqrt_ratio(MAX_SQRT_RATIO);
        assert_eq!(result.unwrap_err().to_string(), "Second inequality must be < because the price can never reach the price at the max tick");

        //ratio of min tick
        let result = get_tick_at_sqrt_ratio(MIN_SQRT_RATIO).unwrap();
        assert_eq!(result, MIN_TICK);

        //ratio of min tick + 1
        let result = get_tick_at_sqrt_ratio(uint!(4295343490_U256)).unwrap();
        assert_eq!(result, MIN_TICK + 1);
    }
}
