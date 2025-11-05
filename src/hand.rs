use rand::rngs::ThreadRng;

use crate::die::Die;

#[derive(Debug)]
pub struct Hand {
    dice: Vec<Die>,
}

impl Hand {
    pub fn empty() -> Self {
        Self { dice: vec![] }
    }

    pub fn len(&self) -> usize {
        self.dice.len()
    }

    pub fn roll(&mut self, hand_size: u8, rng: &mut ThreadRng) {
        self.dice.clear();
        for _ in 0..hand_size {
            self.dice.push(Die::roll(rng))
        }
    }

    pub fn show(&self) -> &Vec<Die> {
        &self.dice
    }
}
