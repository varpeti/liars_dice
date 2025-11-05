use rand::{Rng, rngs::ThreadRng};

#[derive(Debug)]
pub struct Die {
    value: u8,
}

impl Die {
    pub fn roll(rng: &mut ThreadRng) -> Self {
        let value = rng.random_range(1..=6);
        Self { value }
    }

    pub fn value(&self) -> u8 {
        self.value
    }
}
