use rand::rngs::ThreadRng;

use crate::die::Die;

#[derive(Debug, Clone)]
pub struct Hand {
    pub dice: Vec<Die>,
}

impl Hand {
    pub fn new(starting_hand_size: usize) -> Self {
        Self {
            dice: vec![Die { face: 1 }; starting_hand_size],
        }
    }

    pub fn roll(&mut self, rng: &mut ThreadRng) {
        let hand_size = self.dice.len();
        self.dice.clear();
        for _ in 0..hand_size {
            self.dice.push(Die::roll(rng))
        }
    }
}
