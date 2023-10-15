use std::ops::RangeInclusive;

use rayon::prelude::{ParallelBridge, ParallelIterator};
use sha1::{Digest, Sha1};

const CHARS: RangeInclusive<char> = '!'..='~';

struct Permutation {
    stack: Vec<(RangeInclusive<char>, String)>,
    max_length: usize,
}

impl Iterator for Permutation {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self.stack.pop() {
            Some((mut chars, prefix)) => match chars.next() {
                Some(c) => {
                    let next = format!("{prefix}{c}");
                    if next.len() < self.max_length {
                        self.stack.push((CHARS.into_iter(), next.clone()));
                    }
                    self.stack.push((chars, prefix));
                    Some(next)
                }
                None => self.next(),
            },
            None => None,
        }
    }
}

impl Permutation {
    fn new(max_length: usize) -> Self {
        Self {
            stack: vec![(CHARS.into_iter(), String::new())],
            max_length,
        }
    }
}

fn main() {
    let p = Permutation::new(8);
    p.par_bridge()
        .map(|input| {
            let mut hash = Sha1::new();
            hash.update(input.as_bytes());
            let hash = hash.finalize();
            format!("{input}\t{hash:x}")
        })
        .for_each(|result| {
            println!("{result}");
        })
}
