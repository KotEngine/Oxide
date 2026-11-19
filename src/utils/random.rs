use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::cell::RefCell;

thread_local! {
    static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy());
}

pub fn next_u32() -> u32 {
    RNG.with(|r: &RefCell<SmallRng>| r.borrow_mut().gen::<u32>())
}

pub fn next_i32() -> i32 {
    RNG.with(|r: &RefCell<SmallRng>| r.borrow_mut().gen::<i32>())
}

pub fn next_f32() -> f32 {
    RNG.with(|r: &RefCell<SmallRng>| r.borrow_mut().gen::<f32>())
}

pub fn next_range(min: i32, max: i32) -> i32 {
    if min >= max { return min; }
    RNG.with(|r: &RefCell<SmallRng>| r.borrow_mut().gen_range(min..max))
}

pub fn next_range_f(min: f32, max: f32) -> f32 {
    RNG.with(|r: &RefCell<SmallRng>| r.borrow_mut().gen_range(min..max))
}
