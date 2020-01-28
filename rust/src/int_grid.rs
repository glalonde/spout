const INNER_GRID_BITS: u32 = 12;

// Returns a T with a binary representation of n_ones in the least significant
// digits
pub const fn bitmask(n_ones: u32) -> u32 {
    let (r, v) = u32::max_value().overflowing_shr(32 - n_ones);
    r & (!v as u32).wrapping_neg()
}

pub const fn inner_grid_bitmask() -> u32 {
    bitmask(INNER_GRID_BITS)
}

pub const fn get_outer_grid(v: u32) -> u32 {
    v.wrapping_shr(INNER_GRID_BITS)
}

pub const fn get_inner_grid(v: u32) -> u32 {
    v & inner_grid_bitmask()
}

// Set only the outer grid, the inner grid bits wll be 0.
pub const fn set_outer_grid(v: u32) -> u32 {
    v.wrapping_shl(INNER_GRID_BITS)
}

// Set only the inner grid, the outer grid bits wll be 0.
pub const fn set_inner_grid(v: u32) -> u32 {
    v & inner_grid_bitmask()
}

pub const fn set_values(outer: u32, inner: u32) -> u32 {
    set_outer_grid(outer) | set_inner_grid(inner)
}

pub const fn set_values_relative(outer: u32, inner: u32) -> u32 {
    set_values(outer + half_outer_grid_size(), inner)
}

// Inner grid cell dimensions
pub const fn cell_size() -> u32 {
    inner_grid_bitmask() + 1
}

// Half an inner grid cell dimension.
pub const fn half_cell_size() -> u32 {
    cell_size().wrapping_shr(1)
}

// Outer grid dimension.
pub const fn outer_grid_size() -> u32 {
    bitmask(32 - INNER_GRID_BITS) + 1
}

// Half outer grid size.
// This is the "anchor", or origin within the unsigned coordinate system because
// it gives us the most space before hitting overflow.
pub const fn half_outer_grid_size() -> u32 {
    outer_grid_size().wrapping_shr(1)
}

pub fn float_to_grid(v: f64) -> u32 {
    return (v * (cell_size() as f64)).round() as u32;
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

    #[test]
    fn set_values_test() {
        let packed = set_values(6, 5);
        assert_eq!(get_inner_grid(packed), 5);
        assert_eq!(get_outer_grid(packed), 6);
    }

    #[test]
    fn cell_size_test() {
        assert_eq!(half_cell_size() * 2, cell_size());
        // Wraps out
        assert_eq!(set_inner_grid(cell_size()), 0);
        assert_eq!(set_inner_grid(half_cell_size()), half_cell_size());
    }

    #[test]
    fn outer_grid_size_test() {
        assert_eq!(half_outer_grid_size() * 2, outer_grid_size());
        println!("The anchor value is {}", half_outer_grid_size());
        assert_eq!(
            set_values(outer_grid_size() - 1, cell_size() - 1,),
            u32::max_value()
        );
    }

    #[test]
    fn sleep_test() {
        let target: f64 = 1.0 / 60.0;
        let mut max_err: f64 = 0.0;
        let mut avg_overshoot_err: f64 = 0.0;
        let mut avg_undershoot_err: f64 = 0.0;
        let mut overshoot = 0;
        let mut undershoot = 0;
        for _ in 0..1000 {
            let now = std::time::Instant::now();
            std::thread::sleep(std::time::Duration::from_secs_f64(target));
            let elapsed = now.elapsed();
            let actual = elapsed.as_secs_f64();
            let err = actual - target;
            if err >= 0.0 {
                avg_overshoot_err += err.abs();
                overshoot += 1;
            } else {
                avg_undershoot_err += err.abs();
                undershoot += 1;
            }
            if err > max_err {
                max_err = err;
            }
        }
        avg_overshoot_err = avg_overshoot_err / (overshoot as f64);
        avg_undershoot_err = avg_undershoot_err / (undershoot as f64);
        println!(
            "Max: {}, Avg Over: {}, Avg Under: {}",
            max_err, avg_overshoot_err, avg_undershoot_err
        );
        assert!(false);
    }
}
