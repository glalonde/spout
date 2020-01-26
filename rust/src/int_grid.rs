const INNER_GRID_BITS: u32 = 12;

// Returns a T with a binary representation of n_ones in the least significant
// digits
const fn bitmask(n_ones: u32) -> u32 {
    let (r, v) = u32::max_value().overflowing_shr((std::mem::size_of::<u32>() as u32) * 8 - n_ones);
    r & (!v as u32).wrapping_neg()
}

const fn inner_grid_bitmask() -> u32 {
    bitmask(INNER_GRID_BITS)
}

const fn get_outer_grid(v: u32) -> u32 {
    v.wrapping_shr(INNER_GRID_BITS)
}

const fn get_inner_grid(v: u32) -> u32 {
    v & inner_grid_bitmask()
}

const fn set_outer_grid(v: u32) -> u32 {
    v.wrapping_shl(INNER_GRID_BITS)
}

const fn set_inner_grid(v: u32) -> u32 {
    v & inner_grid_bitmask()
}

const fn set_values(inner: u32, outer: u32) -> u32 {
    set_outer_grid(outer) | set_inner_grid(inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitmask_test() {
        assert_eq!(bitmask(5), 16 + 8 + 4 + 2 + 1);
        assert_eq!(bitmask(1), 1);
        assert_eq!(bitmask(0), 0);
        assert_eq!(bitmask(32), std::u32::MAX);
    }

    #[test]
    fn get_outer_grid_test() {
        for i in 0..(32 - INNER_GRID_BITS) {
            assert_eq!(get_outer_grid(i << INNER_GRID_BITS), i);
        }
    }

    #[test]
    fn get_inner_grid_test() {
        for i in 0..(32 - INNER_GRID_BITS) {
            assert_eq!(get_inner_grid(i << INNER_GRID_BITS), 0);
            assert_eq!(get_inner_grid(i << INNER_GRID_BITS) + 5, 5);
        }
    }
}
