use crate::wordle::*;

pub struct PatternBuilder {
    colours: [Colour; 5],
    count: usize,
}

impl PatternBuilder {
    pub fn new() -> PatternBuilder {
        PatternBuilder {
            colours: [Colour::Black; 5],
            count: 0,
        }
    }

    pub fn append(&mut self, colour: Colour) -> bool {
        if self.count == 5 {
            return false;
        }
        self.colours[self.count] = colour;
        self.count += 1;
        true
    }

    pub fn remove(&mut self) -> bool {
        if self.count == 0 {
            return false;
        }
        self.count -= 1;
        true
    }

    pub fn get(&self) -> &[Colour] {
        &self.colours[..self.count]
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn clear(&mut self) {
        self.count = 0;
    }

    pub fn get_pattern(&self) -> Option<Pattern> {
        if self.count != 5 {
            return None;
        }
        Some(Pattern::new(self.colours))
    }
}
