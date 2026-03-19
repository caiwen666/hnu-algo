use std::collections::{HashMap, HashSet};

use hnu_algo::algorithms::pagerank::SparsePagerank;
use jieba_rs::Jieba;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 4 {
        eprintln!("Usage: text_rank <file_path> <window_size> <top_k>");
        std::process::exit(1);
    }
    let file_path = args[1].clone();
    let window_size = args[2].parse::<usize>().unwrap();
    let top_k = args[3].parse::<usize>().unwrap();
    let text = std::fs::read_to_string(file_path).unwrap();
    fn filter(word: &str) -> bool {
        word.chars().count() > 1
    }
    let result = text_rank(&text, window_size, filter);
    println!("{:?}", result.iter().take(top_k).collect::<Vec<_>>());
    println!(
        "{:?}",
        sort_words(&text, filter)
            .iter()
            .take(top_k)
            .collect::<Vec<_>>()
    );
}

pub fn text_rank(
    text: &str,
    window_size: usize,
    filter: impl Fn(&str) -> bool,
) -> Vec<(&str, f64)> {
    let jieba = Jieba::new();
    let mut words = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        words.extend(
            jieba
                .cut(line, true)
                .into_iter()
                .filter(|word| filter(word)),
        );
    }
    if words.is_empty() {
        return vec![];
    }
    let all_words: HashSet<&str> = HashSet::from_iter(words.iter().cloned());
    let mut pagerank = SparsePagerank::new(all_words.len());
    for l in words.windows(window_size.min(words.len())) {
        let window_words: HashSet<&str> = HashSet::from_iter(l.iter().cloned());
        for &word in window_words.iter() {
            for &other_word in window_words.iter() {
                if word != other_word {
                    pagerank.add_edge(word, other_word);
                }
            }
        }
    }
    pagerank.rank(0.85, 1e-6)
}

pub fn sort_words(text: &str, filter: impl Fn(&str) -> bool) -> Vec<(&str, usize)> {
    let jieba = Jieba::new();
    let mut words = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        words.extend(
            jieba
                .cut(line, true)
                .into_iter()
                .filter(|word| filter(word)),
        );
    }
    let mut word_count = HashMap::new();
    for word in words {
        *word_count.entry(word).or_insert(0) += 1;
    }
    let mut result: Vec<(&str, usize)> = word_count.into_iter().collect();
    result.sort_by_key(|(_, count)| *count);
    result.reverse();
    result
}
