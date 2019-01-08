use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::BufRead;
use trie::Trie;

fn main() -> io::Result<()> {
    let file: File = File::open("dictionary.csv")?;
    let br = BufReader::new(file);
    let trie: Trie = br
        .lines()
        .map(|l| l.unwrap())
        .filter_map(|l| {
            l.split(",") // Csv
                .next() // First element
                .map(|s| {
                    if s.len() > 1 {
                        Some(s[1..s.len()-1].to_string()) // remove ""
                    } else {
                        None
                    }
                }).unwrap()
        })
        .collect();

    println!("{:#?}", trie.get_completions("Now"));

    Ok(())
}
