use rayon::prelude::*;
use std::{fmt::Display, fs};

pub struct Wordle {
    words: Vec<Word>,
    mask: Vec<bool>,
    targets: Vec<u16>,
    patterns: Vec<u8>,
    guess: u16,
}

impl Wordle {
    pub fn new() -> Wordle {
        let data = fs::read("data.bin").unwrap();

        let words_len = u16::from_be_bytes([data[0], data[1]]) as usize;
        let targets_len = u16::from_be_bytes([data[2], data[3]]) as usize;

        let i = 4 + words_len * 5;
        let j = i + words_len;
        let words: Vec<_> = data[4..i]
            .chunks(5)
            .map(|bytes| Word::from_bytes(bytes))
            .collect();
        let mask: Vec<_> = data[i..j].iter().map(|b| *b != 0).collect();
        let targets: Vec<_> = (0..targets_len as u16).collect();
        let patterns = data[j..].to_vec();

        let mut wordle = Wordle {
            words,
            mask,
            targets,
            patterns,
            guess: 0,
        };
        wordle.set_guess();
        wordle
    }

    pub fn update(&mut self, pattern: Pattern) {
        let pattern_id = pattern.get_id();

        let (mut w, mut t) = (0, 0);
        while t < self.targets.len() {
            while !self.mask[w] {
                w += 1;
            }
            if pattern_id == self.get_pattern(self.guess, self.targets[t]) {
                t += 1;
            } else {
                self.mask[w] = false;
                self.targets.remove(t);
            }
            w += 1;
        }

        self.set_guess();
    }

    pub fn guess(&self) -> Word {
        self.words[self.guess as usize]
    }

    pub fn options(&self) -> usize {
        self.targets.len()
    }

    fn set_guess(&mut self) {
        let total = self.targets.len() as f64;

        self.guess = (0..self.words.len() as u16)
            .into_par_iter()
            .map(|guess| {
                let mut patterns = vec![0usize; 243];
                for &target in &self.targets {
                    patterns[self.get_pattern(guess, target) as usize] += 1;
                }
                let mut entropy = if self.mask[guess as usize] {
                    1.0 / total
                } else {
                    0.0
                };
                for &count in &patterns {
                    if count > 0 {
                        let p = count as f64 / total;
                        entropy -= p * p.log2();
                    }
                }
                (guess, entropy)
            })
            .max_by(|&(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap()
            .0;
    }

    fn get_pattern(&self, guess: u16, target: u16) -> u8 {
        self.patterns[guess as usize + target as usize * self.words.len()]
    }
}

#[derive(Clone, Copy)]
pub struct Word {
    letters: [u8; 5],
}

impl Word {
    fn from_bytes(b: &[u8]) -> Word {
        Word {
            letters: [b[0], b[1], b[2], b[3], b[4]],
        }
    }
}

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            String::from_iter(self.letters.map(|letter| letter as char))
        )
    }
}

pub struct Pattern {
    colours: [Colour; 5],
}

impl Pattern {
    pub fn new(colours: [Colour; 5]) -> Pattern {
        Pattern { colours }
    }

    fn get_id(&self) -> u8 {
        let mut value = 0;
        let mut multiplier = 1;
        for i in 0..5 {
            value += match self.colours[i] {
                Colour::Black => 0,
                Colour::Yellow => multiplier,
                Colour::Green => multiplier * 2,
            };
            multiplier *= 3;
        }
        value
    }
}

#[derive(Clone, Copy)]
pub enum Colour {
    Green,
    Yellow,
    Black,
}
