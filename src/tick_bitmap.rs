use super::U256;
use crate::{bit_math, error::UniswapV3MathError, utils::RUINT_ONE, UniV3OnchainProvider};
use std::collections::HashMap;

//Returns next and initialized
//current_word is the current word in the TickBitmap of the pool based on `tick`.
// TickBitmap[word_pos] = current_word Where word_pos is the 256 bit offset of the ticks word_pos..
// word_pos := tick >> 8
pub fn next_initialized_tick_within_one_word(
    tick_bitmap: &HashMap<i16, U256>,
    tick: i32,
    tick_spacing: i32,
    lte: bool,
) -> Result<(i32, bool), UniswapV3MathError> {
    let compressed = tick / tick_spacing;

    let (word_pos, bit_pos) = position(compressed);

    if lte {
        let mask = (RUINT_ONE << bit_pos as usize) - RUINT_ONE + (RUINT_ONE << bit_pos as usize);

        let masked = tick_bitmap[&word_pos] & mask;

        let initialized = !masked == U256::ZERO;

        let next = if initialized {
            (compressed -
                (bit_pos.overflowing_sub(bit_math::most_significant_bit(masked)?).0) as i32) *
                tick_spacing
        } else {
            (compressed - bit_pos as i32) * tick_spacing
        };

        Ok((next, initialized))
    } else {
        let mask = !((RUINT_ONE << bit_pos as usize) - RUINT_ONE);

        let masked = tick_bitmap[&word_pos] & mask;

        let initialized = !masked == U256::ZERO;

        let next = if initialized {
            (compressed +
                1 +
                (bit_math::least_significant_bit(masked)?.overflowing_sub(bit_pos).0) as i32) *
                tick_spacing
        } else {
            (compressed + 1 + ((0xFF - bit_pos) as i32)) * tick_spacing
        };

        Ok((next, initialized))
    }
}

//Returns next and initialized. This function calls the node to get the word at the word_pos.
//current_word is the current word in the TickBitmap of the pool based on `tick`.
// TickBitmap[word_pos] = current_word Where word_pos is the 256 bit offset of the ticks word_pos..
// word_pos := tick >> 8
pub fn next_initialized_tick_within_one_word_from_provider<P>(
    tick: i32,
    tick_spacing: i32,
    lte: bool,
    data_provider: P,
) -> Result<(i32, bool), UniswapV3MathError>
where
    P: UniV3OnchainProvider,
{
    let compressed = if tick < 0 && tick % tick_spacing != 0 {
        (tick / tick_spacing) - 1
    } else {
        tick / tick_spacing
    };

    if lte {
        let (word_pos, bit_pos) = position(compressed);
        let mask = (RUINT_ONE << bit_pos as usize) - RUINT_ONE + (RUINT_ONE << bit_pos as usize);

        let word: U256 = data_provider.get_word_at_position(word_pos)?;

        let masked = word & mask;

        let initialized = !masked == U256::ZERO;

        let next = if initialized {
            (compressed -
                (bit_pos.overflowing_sub(bit_math::most_significant_bit(masked)?).0) as i32) *
                tick_spacing
        } else {
            (compressed - bit_pos as i32) * tick_spacing
        };

        Ok((next, initialized))
    } else {
        let (word_pos, bit_pos) = position(compressed + 1);

        let mask = !((RUINT_ONE << bit_pos as usize) - RUINT_ONE);

        let word = data_provider.get_word_at_position(word_pos)?;

        let masked = word & mask;

        let initialized = !masked == U256::ZERO;

        let next = if initialized {
            (compressed +
                1 +
                (bit_math::least_significant_bit(masked)?.overflowing_sub(bit_pos).0) as i32) *
                tick_spacing
        } else {
            (compressed + 1 + ((0xFF - bit_pos) as i32)) * tick_spacing
        };

        Ok((next, initialized))
    }
}

// returns (int16 wordPos, uint8 bitPos)
pub fn position(tick: i32) -> (i16, u8) {
    ((tick >> 8) as i16, (tick % 256) as u8)
}
