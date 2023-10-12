use std::{ops::RangeInclusive, sync::mpsc::Sender};

use rayon::prelude::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use sha1::{Digest, Sha1};

const CHARS: RangeInclusive<char> = '!'..='~';

fn dfs(out: Sender<String>, current: String, remaining_level: usize) {
    if remaining_level == 0 {
        return;
    }
    CHARS.into_iter().into_par_iter().for_each(|c| {
        let mut next = current.clone();
        next.push(c);
        out.send(next.clone()).unwrap();
        dfs(out.clone(), next, remaining_level - 1);
    })
}

fn main() {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(|| dfs(sender, String::new(), 12));
    receiver.into_iter().par_bridge().for_each(|input| {
        let mut h = Sha1::new();
        h.update(input.as_bytes());
        let hash = h.finalize();
        println!("{input}\t{hash:x}");
    });
}
