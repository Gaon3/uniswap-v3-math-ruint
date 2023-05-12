use super::U256;
use crate::{
    error::UniswapV3MathError,
    full_math::{mul_div, mul_div_rounding_up},
    sqrt_price_math::{
        _get_amount_0_delta, _get_amount_1_delta, get_next_sqrt_price_from_input,
        get_next_sqrt_price_from_output,
    },
};
use ethers_core::types::I256;

// //returns (
//         uint160 sqrtRatioNextX96,
//         uint256 amountIn,
//         uint256 amountOut,
//         uint256 feeAmount
//     )
pub fn compute_swap_step(
    sqrt_ratio_current_x_96: U256,
    sqrt_ratio_target_x_96: U256,
    liquidity: u128,
    amount_remaining: I256,
    fee_pips: u32,
) -> Result<(U256, U256, U256, U256), UniswapV3MathError> {
    let zero_for_one = sqrt_ratio_current_x_96 >= sqrt_ratio_target_x_96;
    let exact_in = amount_remaining >= I256::zero();

    let sqrt_ratio_next_x_96: U256;
    let mut amount_in = U256::ZERO;
    let mut amount_out = U256::ZERO;

    if exact_in {
        let amount_remaining_less_fee = mul_div(
            U256::from_limbs(amount_remaining.into_raw().0),
            U256::from(1e6 as u32 - fee_pips), //1e6 - fee_pips
            U256::from(1e6 as u32),            //1e6
        )?;

        amount_in = if zero_for_one {
            _get_amount_0_delta(sqrt_ratio_target_x_96, sqrt_ratio_current_x_96, liquidity, true)?
        } else {
            _get_amount_1_delta(sqrt_ratio_current_x_96, sqrt_ratio_target_x_96, liquidity, true)?
        };

        if amount_remaining_less_fee >= amount_in {
            sqrt_ratio_next_x_96 = sqrt_ratio_target_x_96;
        } else {
            sqrt_ratio_next_x_96 = get_next_sqrt_price_from_input(
                sqrt_ratio_current_x_96,
                liquidity,
                amount_remaining_less_fee,
                zero_for_one,
            )?;
        }
    } else {
        amount_out = if zero_for_one {
            _get_amount_1_delta(sqrt_ratio_target_x_96, sqrt_ratio_current_x_96, liquidity, false)?
        } else {
            _get_amount_0_delta(sqrt_ratio_current_x_96, sqrt_ratio_target_x_96, liquidity, false)?
        };

        let amount_remaining_neg = U256::from_limbs((-amount_remaining).into_raw().0);

        sqrt_ratio_next_x_96 = if amount_remaining_neg >= amount_out {
            sqrt_ratio_target_x_96
        } else {
            get_next_sqrt_price_from_output(
                sqrt_ratio_current_x_96,
                liquidity,
                amount_remaining_neg,
                zero_for_one,
            )?
        };
    }

    let max = sqrt_ratio_target_x_96 == sqrt_ratio_next_x_96;

    if zero_for_one {
        if !max || !exact_in {
            amount_in =
                _get_amount_0_delta(sqrt_ratio_next_x_96, sqrt_ratio_current_x_96, liquidity, true)?
        }

        if !max || exact_in {
            amount_out = _get_amount_1_delta(
                sqrt_ratio_next_x_96,
                sqrt_ratio_current_x_96,
                liquidity,
                false,
            )?
        }
    } else {
        if !max || !exact_in {
            amount_in =
                _get_amount_1_delta(sqrt_ratio_current_x_96, sqrt_ratio_next_x_96, liquidity, true)?
        }

        if !max || exact_in {
            amount_out = _get_amount_0_delta(
                sqrt_ratio_current_x_96,
                sqrt_ratio_next_x_96,
                liquidity,
                false,
            )?
        }
    }

    let amount_remaining_neg = U256::from_limbs((-amount_remaining).into_raw().0);

    if !exact_in && amount_out > amount_remaining_neg {
        amount_out = amount_remaining_neg;
    }

    if exact_in && sqrt_ratio_next_x_96 != sqrt_ratio_target_x_96 {
        let fee_amount = U256::from_limbs(amount_remaining.into_raw().0) - amount_in;
        Ok((sqrt_ratio_next_x_96, amount_in, amount_out, fee_amount))
    } else {
        let fee_amount = mul_div_rounding_up(
            amount_in,
            U256::from(fee_pips),
            U256::from(1e6 as u32 - fee_pips),
        )?;

        Ok((sqrt_ratio_next_x_96, amount_in, amount_out, fee_amount))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        sqrt_price_math::{get_next_sqrt_price_from_input, get_next_sqrt_price_from_output},
        swap_math::compute_swap_step,
        utils::RUINT_ONE,
    };

    use super::U256;
    use ethers_core::types::I256;
    use ruint::uint;

    #[test]
    fn test_compute_swap_step() {
        //------------------------------------------------------------

        //exact amount in that gets capped at price target in one for zero
        let price = uint!(79228162514264337593543950336_U256);
        let price_target = uint!(79623317895830914510639640423_U256);
        let liquidity = 2e18 as u128;
        let amount = I256::from_dec_str("1000000000000000000").unwrap();
        let fee = 600;
        let zero_for_one = false;

        let (sqrt_p, amount_in, amount_out, fee_amount) =
            compute_swap_step(price, price_target, liquidity, amount, fee).unwrap();

        assert_eq!(sqrt_p, uint!(79623317895830914510639640423_U256));

        assert_eq!(amount_in, uint!(9975124224178055_U256));
        assert_eq!(fee_amount, uint!(5988667735148_U256));
        assert_eq!(amount_out, uint!(9925619580021728_U256));

        let mut le_bytes = [0 as u8; 32];
        amount.to_little_endian(&mut le_bytes);
        assert!(amount_in + fee_amount < U256::from_le_bytes(le_bytes));

        let price_after_whole_input_amount =
            get_next_sqrt_price_from_input(price, liquidity, amount_in, zero_for_one).unwrap();

        assert_eq!(sqrt_p, price_target);
        assert!(sqrt_p < price_after_whole_input_amount);

        //------------------------------------------------------------

        //exact amount out that gets capped at price target in one for zero
        let price = uint!(79228162514264337593543950336_U256);
        let price_target = uint!(79623317895830914510639640423_U256);
        let liquidity = 2e18 as u128;
        let amount = I256::from_dec_str("-1000000000000000000").unwrap();
        let fee = 600;
        let zero_for_one = false;

        let (sqrt_p, amount_in, amount_out, fee_amount) =
            compute_swap_step(price, price_target, liquidity, amount, fee).unwrap();

        assert_eq!(amount_in, uint!(9975124224178055_U256));
        assert_eq!(fee_amount, uint!(5988667735148_U256));
        assert_eq!(amount_out, uint!(9925619580021728_U256));
        assert!(amount_out < U256::from_limbs((amount * -I256::one()).into_raw().0));

        let mut le_bytes = [0 as u8; 32];
        amount.to_little_endian(&mut le_bytes);
        assert!(amount_in + fee_amount < U256::from_le_bytes(le_bytes));

        let price_after_whole_output_amount = get_next_sqrt_price_from_output(
            price,
            liquidity,
            U256::from_limbs((amount * -I256::one()).into_raw().0),
            zero_for_one,
        )
        .unwrap();

        assert_eq!(sqrt_p, price_target);
        assert!(sqrt_p < price_after_whole_output_amount);

        //------------------------------------------------------------

        //exact amount in that is fully spent in one for zero
        let price = uint!(79228162514264337593543950336_U256);
        let price_target = uint!(0xe6666666666666666666666666_U256);
        let liquidity = 2e18 as u128;
        let amount = I256::from_dec_str("1000000000000000000").unwrap();
        let fee = 600;
        let zero_for_one = false;

        let (sqrt_p, amount_in, amount_out, fee_amount) =
            compute_swap_step(price, price_target, liquidity, amount, fee).unwrap();

        assert_eq!(amount_in, uint!(999400000000000000_U256));
        assert_eq!(fee_amount, uint!(600000000000000_U256));
        assert_eq!(amount_out, uint!(666399946655997866_U256));
        assert_eq!(amount_in + fee_amount, U256::from_limbs(amount.into_raw().0));

        let price_after_whole_input_amount_less_fee = get_next_sqrt_price_from_input(
            price,
            liquidity,
            U256::from_limbs(amount.into_raw().0) - fee_amount,
            zero_for_one,
        )
        .unwrap();

        assert!(sqrt_p < price_target);
        assert_eq!(sqrt_p, price_after_whole_input_amount_less_fee);

        //------------------------------------------------------------

        //exact amount out that is fully received in one for zero
        let price = uint!(79228162514264337593543950336_U256);
        let price_target = uint!(792281625142643375935439503360_U256);
        let liquidity = 2e18 as u128;
        let amount = I256::from_dec_str("1000000000000000000").unwrap() * -I256::one();
        let fee = 600;
        let zero_for_one = false;

        let (sqrt_p, amount_in, amount_out, fee_amount) =
            compute_swap_step(price, price_target, liquidity, amount, fee).unwrap();

        assert_eq!(amount_in, uint!(2000000000000000000_U256));
        assert_eq!(fee_amount, uint!(1200720432259356_U256));
        assert_eq!(amount_out, U256::from_limbs((amount * -I256::one()).into_raw().0));

        let price_after_whole_output_amount = get_next_sqrt_price_from_output(
            price,
            liquidity,
            U256::from_limbs((amount * -I256::one()).into_raw().0),
            zero_for_one,
        )
        .unwrap();
        //sqrtPrice 158456325028528675187087900672
        //price_after_whole_output_amount Should be: 158456325028528675187087900672
        // sqrtp: 158456325028528675187087900672, price_after_whole output amount:
        // 118842243771396506390315925504

        assert!(sqrt_p < price_target);
        //TODO:FIXME: failing
        println!(
            "sqrtp: {:?}, price_after_whole output amount: {:?}",
            sqrt_p, price_after_whole_output_amount
        );
        assert_eq!(sqrt_p, price_after_whole_output_amount);

        //------------------------------------------------------------

        //amount out is capped at the desired amount out
        let (sqrt_p, amount_in, amount_out, fee_amount) = compute_swap_step(
            uint!(417332158212080721273783715441582_U256),
            uint!(1452870262520218020823638996_U256),
            159344665391607089467575320103_u128,
            I256::from_dec_str("-1").unwrap(),
            1,
        )
        .unwrap();

        assert_eq!(amount_in, uint!(1_U256));
        assert_eq!(fee_amount, uint!(1_U256));
        assert_eq!(amount_out, uint!(1_U256));
        assert_eq!(sqrt_p, uint!(417332158212080721273783715441581_U256));

        //------------------------------------------------------------

        //target price of 1 uses partial input amount
        let (sqrt_p, amount_in, amount_out, fee_amount) = compute_swap_step(
            uint!(2_U256),
            uint!(1_U256),
            1_u128,
            I256::from_dec_str("3915081100057732413702495386755767").unwrap(),
            1,
        )
        .unwrap();

        assert_eq!(amount_in, uint!(39614081257132168796771975168_U256));
        assert_eq!(fee_amount, uint!(39614120871253040049813_U256));
        assert!(amount_in + fee_amount < uint!(3915081100057732413702495386755767_U256));
        assert_eq!(amount_out, uint!(0_U256));

        assert_eq!(sqrt_p, uint!(1_U256));

        //------------------------------------------------------------

        //entire input amount taken as fee
        let (sqrt_p, amount_in, amount_out, fee_amount) = compute_swap_step(
            uint!(2413_U256),
            uint!(79887613182836312_U256),
            1985041575832132834610021537970_u128,
            I256::from_dec_str("10").unwrap(),
            1872,
        )
        .unwrap();

        assert_eq!(amount_in, uint!(0_U256));
        assert_eq!(fee_amount, uint!(10_U256));
        assert_eq!(amount_out, uint!(0_U256));
        assert_eq!(sqrt_p, uint!(2413_U256));

        //------------------------------------------------------------

        //handles intermediate insufficient liquidity in zero for one exact output case

        let price = uint!(20282409603651670423947251286016_U256);
        let price_target = price * U256::from(11) / U256::from(10);
        let liquidity = 1024;
        // virtual reserves of one are only 4
        // https://www.wolframalpha.com/input/?i=1024+%2F+%2820282409603651670423947251286016+%2F+2**96%29
        let amount_remaining = -I256::from(4);
        let fee = 3000;

        let (sqrt_p, amount_in, amount_out, fee_amount) =
            compute_swap_step(price, price_target, liquidity, amount_remaining, fee).unwrap();

        assert_eq!(amount_out, U256::ZERO);
        assert_eq!(sqrt_p, price_target);
        assert_eq!(amount_in, U256::from(26215));
        assert_eq!(fee_amount, U256::from(79));

        //------------------------------------------------------------

        //handles intermediate insufficient liquidity in one for zero exact output case

        let price = uint!(20282409603651670423947251286016_U256);
        let price_target = price * U256::from(9) / U256::from(10);
        let liquidity = 1024;
        // virtual reserves of zero are only 262144
        // https://www.wolframalpha.com/input/?i=1024+*+%2820282409603651670423947251286016+%2F+2**96%29
        let amount_remaining = -I256::from(263000);
        let fee = 3000;

        let (sqrt_p, amount_in, amount_out, fee_amount) =
            compute_swap_step(price, price_target, liquidity, amount_remaining, fee).unwrap();

        assert_eq!(amount_out, U256::from(26214));
        assert_eq!(sqrt_p, price_target);
        assert_eq!(amount_in, RUINT_ONE);
        assert_eq!(fee_amount, RUINT_ONE);
    }
}
