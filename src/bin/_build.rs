use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
};

fn main() {
    let words = get_word_list("words/allowed.txt").unwrap();
    let targets = get_word_list("words/targets.txt").unwrap();

    let mut targets_ordered = Vec::with_capacity(targets.len());
    let mut mask = Vec::with_capacity(words.len());

    for &word in &words {
        if targets.contains(&word) {
            mask.push(1);
            targets_ordered.push(word);
        } else {
            mask.push(0);
        }
    }

    let mut patterns = Vec::with_capacity(words.len() * targets_ordered.len());
    for &target in &targets_ordered {
        for &word in &words {
            patterns.push(calculate_pattern(word, target));
        }
    }

    let mut data = Vec::with_capacity(4 + words.len() * 6 + patterns.len());
    data.extend((words.len() as u16).to_be_bytes());
    data.extend((targets_ordered.len() as u16).to_be_bytes());
    data.extend(words.into_iter().flatten());
    data.extend(mask.into_iter());
    data.extend(patterns.into_iter());

    fs::write("data.bin", data).unwrap();
}

fn calculate_pattern(guess: [u8; 5], target: [u8; 5]) -> u8 {
    let mut value = 0;
    let mut multiplier = 1;
    let mut used = [false; 5];
    for i in 0..5 {
        if guess[i] == target[i] {
            value += multiplier * 2;
        } else {
            for j in 0..5 {
                if i != j && guess[j] != target[j] && guess[i] == target[j] && !used[j] {
                    value += multiplier;
                    used[j] = true;
                    break;
                }
            }
        }
        multiplier *= 3;
    }
    value
}

fn get_word_list(path: &str) -> Option<Vec<[u8; 5]>> {
    BufReader::new(File::open(path).ok()?)
        .lines()
        .map(|line| {
            let word = line.ok()?;
            if word.chars().count() != 5 {
                return None;
            }
            let mut letters = [0; 5];
            let mut chars = word.chars();
            for i in 0..5 {
                let c = chars.next().unwrap();
                if !c.is_ascii_alphabetic() {
                    return None;
                }
                letters[i] = c.to_ascii_uppercase() as u8;
            }
            Some(letters)
        })
        .collect()
}
