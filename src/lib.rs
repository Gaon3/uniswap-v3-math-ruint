use alloy_primitives::I256;
use error::UniswapV3MathError;
use reth_primitives::U256;
use swap_math::compute_swap_step;
use tick_bitmap::{next_initialized_tick_within_one_word, position};
use tick_math::{
    calculate_compressed, get_sqrt_ratio_at_tick, get_tick_at_sqrt_ratio, MAX_SQRT_RATIO, MAX_TICK,
    MIN_SQRT_RATIO, MIN_TICK,
};
use utils::*;

pub mod bit_math;
pub mod error;
pub mod full_math;
pub mod liquidity_math;
pub mod sqrt_price_math;
pub mod swap_math;
pub mod tick;
pub mod tick_bitmap;
pub mod tick_math;
pub mod unsafe_math;
pub mod utils;

// Used to retrieve ticks and words from the chain. Ideally, this trait should be implemented by a
// database reader.
pub trait TicksProvider {
    fn get_word_at_position(&self, position: i16) -> Result<U256, UniswapV3MathError>;

    fn get_liquidity_net_at_tick(&self, tick: i32) -> Result<i128, UniswapV3MathError>;
}

#[derive(Debug, Default, Clone)]
pub struct Math<Provider> {
    pub fee: u32,
    pub liquidity: u128,
    pub sqrt_price_x96: U256,
    pub tick: i32,
    pub tick_spacing: i32,
    pub provider: Provider,
}

impl<Provider> Math<Provider>
where
    Provider: TicksProvider,
{
    pub fn update(&mut self, liquidity: u128, sqrt_price_x96: U256, tick: i32) {
        self.liquidity = liquidity;
        self.sqrt_price_x96 = sqrt_price_x96;
        self.tick = tick;
    }

    pub fn simulate_swap(
        &self,
        zero_for_one: bool,
        amount_in: U256,
    ) -> Result<U256, UniswapV3MathError> {
        if amount_in == U256::ZERO {
            return Ok(U256::ZERO);
        }

        //Set sqrt_price_limit_x96 to the max or min sqrt price in the pool depending on
        // zero_for_one
        let sqrt_price_limit_x96 = if zero_for_one {
            MIN_SQRT_RATIO + RUINT_ONE
        } else {
            MAX_SQRT_RATIO - RUINT_ONE
        };

        //Initialize a mutable state state struct to hold the dynamic simulated state of the pool
        let mut current_state = CurrentState {
            sqrt_price_x96: self.sqrt_price_x96, //Active price on the pool
            amount_calculated: I256::ZERO,       //Amount of token_out that has been calculated
            amount_specified_remaining: u256_to_i256(amount_in),
            tick: self.tick,           //Current i24 tick of the pool
            liquidity: self.liquidity, //Current available liquidity in the tick range
            word_pos: position(calculate_compressed(self.tick, self.tick_spacing)).0,
        };

        let mut word = self.provider.get_word_at_position(current_state.word_pos)?;

        while current_state.amount_specified_remaining != I256::ZERO
            && current_state.sqrt_price_x96 != sqrt_price_limit_x96
        {
            //Initialize a new step struct to hold the dynamic state of the pool at each step
            let mut step = StepComputations {
                sqrt_price_start_x96: current_state.sqrt_price_x96, /* Set the sqrt_price_start_x96 to the current sqrt_price_x96 */
                ..Default::default()
            };

            let compressed = calculate_compressed(current_state.tick, self.tick_spacing);
            let (word_pos, bit_pos) = position(compressed);

            if word_pos != current_state.word_pos {
                word = self.provider.get_word_at_position(current_state.word_pos)?;
                current_state.word_pos = word_pos;
            }

            (step.tick_next, step.initialized) = next_initialized_tick_within_one_word(
                bit_pos,
                word,
                self.tick_spacing,
                zero_for_one,
                compressed,
            )?;

            // ensure that we do not overshoot the min/max tick, as the tick bitmap is not aware of
            // these bounds Note: this could be removed as we are clamping in the batch contract
            step.tick_next = step.tick_next.clamp(MIN_TICK, MAX_TICK);

            //Get the next sqrt price from the input amount
            step.sqrt_price_next_x96 = get_sqrt_ratio_at_tick(step.tick_next)?;

            //Target spot price
            let swap_target_sqrt_ratio = if zero_for_one {
                if step.sqrt_price_next_x96 < sqrt_price_limit_x96 {
                    sqrt_price_limit_x96
                } else {
                    step.sqrt_price_next_x96
                }
            } else if step.sqrt_price_next_x96 > sqrt_price_limit_x96 {
                sqrt_price_limit_x96
            } else {
                step.sqrt_price_next_x96
            };

            //Compute swap step and update the current state
            (
                current_state.sqrt_price_x96,
                step.amount_in,
                step.amount_out,
                step.fee_amount,
            ) = compute_swap_step(
                current_state.sqrt_price_x96,
                swap_target_sqrt_ratio,
                current_state.liquidity,
                current_state.amount_specified_remaining,
                self.fee,
            )?;

            //Decrement the amount remaining to be swapped and amount received from the step
            current_state.amount_specified_remaining = current_state
                .amount_specified_remaining
                .overflowing_sub(u256_to_i256(
                    step.amount_in.overflowing_add(step.fee_amount).0,
                ))
                .0;

            current_state.amount_calculated -= u256_to_i256(step.amount_out);

            //If the price moved all the way to the next price, recompute the liquidity change for
            // the next iteration
            if current_state.sqrt_price_x96 == step.sqrt_price_next_x96 {
                if step.initialized {
                    let mut liquidity_net =
                        self.provider.get_liquidity_net_at_tick(step.tick_next)?;

                    // we are on a tick boundary, and the next tick is initialized, so we must
                    // charge a protocol fee
                    if zero_for_one {
                        liquidity_net = -liquidity_net;
                    }

                    current_state.liquidity = if liquidity_net < 0 {
                        current_state.liquidity - (-liquidity_net as u128)
                    } else {
                        current_state.liquidity + (liquidity_net as u128)
                    };

                    //Increment the current tick
                    current_state.tick = if zero_for_one {
                        step.tick_next.wrapping_sub(1)
                    } else {
                        step.tick_next
                    }
                }
                //If the current_state sqrt price is not equal to the step sqrt price, then we are
                // not on the same tick. Update the current_state.tick to the tick
                // at the current_state.sqrt_price_x96
            } else if current_state.sqrt_price_x96 != step.sqrt_price_start_x96 {
                current_state.tick = get_tick_at_sqrt_ratio(current_state.sqrt_price_x96)?;
            }
        }

        Ok(i256_to_u256(-current_state.amount_calculated))
    }
}

struct CurrentState {
    amount_specified_remaining: I256,
    amount_calculated: I256,
    sqrt_price_x96: U256,
    tick: i32,
    liquidity: u128,
    word_pos: i16,
}

#[derive(Default)]
struct StepComputations {
    sqrt_price_start_x96: U256,
    tick_next: i32,
    initialized: bool,
    sqrt_price_next_x96: U256,
    amount_in: U256,
    amount_out: U256,
    fee_amount: U256,
}
