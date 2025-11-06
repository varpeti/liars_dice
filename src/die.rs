use rand::{Rng, rngs::ThreadRng};

#[derive(Debug, Clone)]
pub struct Die {
    pub face: u8,
}

impl Die {
    pub fn roll(rng: &mut ThreadRng) -> Self {
        let value = rng.random_range(1..=6);
        Self { face: value }
    }
}
