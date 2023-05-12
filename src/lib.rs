use error::UniswapV3MathError;
use ruint::Uint;
use tick::Tick;

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

type U256 = Uint<256, 4>;

// Used to retrieve ticks and words from the chain. Ideally, this trait should be implemented by a
// database reader.
pub trait UniV3OnchainProvider {
    fn get_word_at_position(&self, pos: i16) -> Result<U256, UniswapV3MathError>;

    fn get_tick_info(&self, tick: i32) -> Result<Tick, UniswapV3MathError>;
}
