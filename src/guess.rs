pub struct Guess {
    pub number: u8,
    pub face: u8,
}

impl Guess {
    pub fn new(number: u8, face: u8) -> Self {
        Self { number, face }
    }

    pub fn start() -> Self {
        Self { number: 0, face: 1 }
    }
}
