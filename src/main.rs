use std::time::Duration;
use std::{ops::RangeInclusive, sync::mpsc::channel};

use itertools::Itertools;
use rayon::prelude::*;
use sha1::{Digest, Sha1};

const CHARS: RangeInclusive<char> = '!'..='~';

fn main() {
    let mut counter = 0;
    let mut last = std::time::Instant::now();
    let second = Duration::from_secs(1);
    let (sender, receiver) = channel();
    std::thread::spawn(move || {
        (0..8)
            .into_par_iter()
            .flat_map(|length| {
                CHARS
                    .permutations(length)
                    .par_bridge()
                    .map(|chars| chars.into_iter().collect::<String>())
            })
            .map(|input: String| {
                let mut hash = Sha1::new();
                hash.update(input.as_bytes());
                let hash = hash.finalize();
                format!("{input}\t{hash:x}")
            })
            .for_each(|_result| {
                sender.send(()).unwrap();
            });
    });
    for _ in receiver.into_iter() {
        counter += 1;
        let now = std::time::Instant::now();
        if now - last >= second {
            println!("{counter}/s");
            counter = 0;
            last = now;
        }
    }
}
