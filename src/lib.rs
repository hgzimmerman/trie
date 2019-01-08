#![feature(test)]
extern crate test;
use std::iter::FromIterator;
use std::str::Chars;

type ShouldDelete = bool;
type DeletionTargetFound = bool;

/// A tree data structure for re**trie**ving and completing strings.
#[derive(Debug, Clone, Default)]
pub struct Trie {
    children: Vec<TrieNode>,
}

/// A node used within the trie.
#[derive(Debug, Clone, PartialEq)]
struct TrieNode {
    children: Vec<TrieNode>,
    value: char,
    is_word: bool,
}

impl TrieNode {
    fn complete_word_iter(self, mut word: String) -> Box<Iterator<Item = String>> {
        word.push(self.value);
        let mut vec = Vec::new();
        if self.is_word {
            vec.push(word.clone())
        }
        Box::new(
            vec.into_iter().chain(
                self.children
                    .into_iter()
                    .map(move |n| n.complete_word_iter(word.clone()))
                    .flatten(),
            ),
        )
    }

    fn complete_word(&self, mut word: String) -> Vec<String> {
        word.push(self.value);
        let mut vec = Vec::new();
        if self.is_word {
            vec.push(word.clone())
        }
        self.children.iter().fold(vec, |mut acc, node| {
            acc.append(&mut node.complete_word(word.clone()));
            acc
        })
    }

    /// the returning bool value indicates if the parent should be deleted.
    fn remove_word(&mut self, mut chars: Chars) -> (ShouldDelete, DeletionTargetFound) {
        let character = chars.next();
        if let Some(character) = character {
            let mut should_remove_child = false;
            let mut found = false;

            if let Some(child) = self.children.iter_mut().find(|n| n.value == character) {
                let x = child.remove_word(chars);
                should_remove_child = x.0;
                found = x.1;
                //                (should_remove_child, found) = child.remove_word(chars);
            }
            let no_children = self.children.is_empty();

            if should_remove_child && no_children {
                let child_index = self
                    .children
                    .iter()
                    .position(|n| n.value == character)
                    .unwrap();
                self.children.remove(child_index);
                (no_children, found) // the parent should be removed if there are no other children
            } else {
                (false, found)
            }
        } else {
            let found = self.is_word;
            // End of search word
            self.is_word = false;
            (self.children.is_empty(), found) // delete the parent as well
        }
    }
}

impl Trie {
    /// Creates a new, empty Trie.
    pub fn new() -> Self {
        Trie {
            children: Vec::new(),
        }
    }

    /// Inserts a string into the Trie.
    pub fn insert<T: AsRef<str>>(&mut self, word: T) {
        let string = word.as_ref();
        let mut current_node: Option<&mut TrieNode> = None;
        let mut chars = string.chars();
        if let Some(character) = chars.next() {
            if let Some(node) = self
                .children
                .iter_mut()
                .find(|child| child.value == character)
            {
                current_node = Some(node);
            } else {
                self.children.push(TrieNode {
                    children: Vec::new(),
                    value: character,
                    is_word: false,
                });
                current_node = self
                    .children
                    .iter_mut()
                    .find(|child| child.value == character);
            }
        }

        for character in chars {
            if let Some(node) = current_node {
                // This is a hacky way to elide the lifetime constraint wrt
                // the iter_mut(), and last_mut() + push() here.
                let can_be_found = node
                    .children
                    .iter()
                    .any(|child| child.value == character);
                if can_be_found {
                    let node2 = node
                        .children
                        .iter_mut()
                        .find(|child| child.value == character)
                        .unwrap();
                    current_node = Some(node2);
                } else {
                    node.children.push(TrieNode {
                        children: Vec::new(),
                        value: character,
                        is_word: false,
                    });
                    current_node = node.children.last_mut();
                }
            } else {
                panic!("yeeet")
            }
        }

        current_node.unwrap().is_word = true;
    }

    /// Given a string, this method will return a boolean indicating if the trie contains the string.
    pub fn contains<T: AsRef<str>>(&self, word: T) -> bool {
        let string = word.as_ref();
        let mut current: Option<&TrieNode> = None;
        let mut is_first = true;
        for c in string.chars() {
            if let Some(node) = current {
                current = node.children.iter().find(|child| child.value == c);
            } else if is_first {
                current = self.children.iter().find(|child| child.value == c);
                is_first = false
            } else {
                return false;
            }
        }

        if let Some(node) = current {
            return node.is_word;
        } else {
            false
        }
    }

    /// Given a string whose characters make up an ordered subset of characters in string(s) present in the trie,
    /// this function will return a Vec<String> of the possible completed strings.
    pub fn get_completions<T: AsRef<str>>(&self, word: T) -> Vec<String> {
        let string = word.as_ref();
        let mut current: Option<&TrieNode> = None;
        let mut is_first = true;
        for c in string.chars() {
            if let Some(node) = current {
                current = node.children.iter().find(|child| child.value == c);
            } else if is_first {
                current = self.children.iter().find(|child| child.value == c);
                is_first = false
            } else {
                return vec![];
            }
        }

        if let Some(root) = current {
            let mut s = string.to_string();
            s.pop();
            root.complete_word(s)
        } else {
            vec![]
        }
    }

    /// Removes the given string from the trie.
    /// The boolean returned indicates if a word to be removed was found.
    pub fn remove<T: AsRef<str>>(&mut self, word: T) -> bool {
        let string = word.as_ref();
        let mut chars = string.chars();
        if let Some(character) = chars.next() {
            if let Some(node) = self
                .children
                .iter_mut()
                .find(|child| child.value == character)
            {
                return node.remove_word(chars).1;
            }
        }
        false
    }
}

impl<U: AsRef<str>> FromIterator<U> for Trie {
    fn from_iter<T: IntoIterator<Item = U>>(iter: T) -> Self {
        let mut trie = Trie::new();
        iter.into_iter().for_each(|x| trie.insert(x));
        trie
    }
}

impl IntoIterator for Trie {
    type Item = String;
    type IntoIter = TrieIter;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        let iter = self
            .children
            .into_iter()
            .map(|n| n.complete_word_iter("".to_string()))
            .flatten();
        TrieIter {
            iter: Box::new(iter),
        }
    }
}

pub struct TrieIter {
    iter: Box<Iterator<Item = String>>,
}

impl Iterator for TrieIter {
    type Item = String;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        self.iter.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    fn dictionary() -> Vec<String> {
        vec![
            "hello",
            "there",
            "general",
            "kenobi",
            "you",
            "are",
            "a",
            "bold",
            "one",
            "I",
            "hate",
            "sand",
            "it",
            "is",
            "course",
            "and",
            "rough",
            "and",
            "irritating",
            "and",
            "it",
            "gets",
            "everywhere",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    #[test]
    fn insert() {
        let mut trie = Trie::new();
        trie.insert("a");
        assert!(trie.contains("a"))
    }
    #[test]
    fn insert2() {
        let mut trie = Trie::new();
        trie.insert("ab");
        assert!(trie.contains("ab"))
    }

    #[test]
    fn insert3() {
        let mut trie = Trie::new();
        trie.insert("abba");
        assert!(trie.contains("abba"))
    }

    #[test]
    fn insert4() {
        let mut trie = Trie::new();
        trie.insert("abba");
        assert!(!trie.contains("ab"))
    }

    #[test]
    fn insert_multiple() {
        let mut trie = Trie::new();
        trie.insert("abba");
        trie.insert("abc");
        assert!(trie.contains("abba"));
        assert!(trie.contains("abc"));
    }
    #[test]
    fn insert_multiple_same() {
        let mut trie = Trie::new();
        trie.insert("abba");
        trie.insert("abba");
        assert!(trie.contains("abba"));
    }
    #[test]
    fn insert_multiple_2() {
        let mut trie = Trie::new();
        trie.insert("abba");
        trie.insert("ab");
        assert!(trie.contains("abba"));
        assert!(trie.contains("ab"));
    }

    #[test]
    fn completions1() {
        let mut trie = Trie::new();
        trie.insert("abba");
        trie.insert("ab");
        let completions = trie.get_completions("ab");
        assert_eq!(completions.len(), 2);
        assert_eq!(completions[0], "ab".to_string());
        assert_eq!(completions[1], "abba".to_string());
    }

    #[test]
    fn completions2() {
        let mut trie = Trie::new();
        trie.insert("abba");
        trie.insert("ab");
        let completions = trie.get_completions("abc");
        assert_eq!(completions.len(), 0);
    }

    #[test]
    fn from_iter() {
        let trie: Trie = vec!["hello", "there", "general", "kenobi"]
            .into_iter()
            .collect();
        assert!(trie.contains("hello"));
        assert!(trie.contains("there"));
        assert!(trie.contains("general"));
        assert!(trie.contains("kenobi"));
    }

    #[test]
    fn complete_word_iter1() {
        let trie: Trie = vec!["hello", "there"].into_iter().collect();

        let mut iter = trie
            .children
            .into_iter()
            .map(|n| n.complete_word_iter("".to_string()))
            .flatten();
        assert_eq!(iter.next(), Some("hello".to_string()));
        assert_eq!(iter.next(), Some("there".to_string()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn into_iter() {
        let trie: Trie = vec!["hello", "there"].into_iter().collect();

        let mut iter = trie.into_iter();
        assert_eq!(iter.next(), Some("hello".to_string()));
        assert_eq!(iter.next(), Some("there".to_string()));
        assert_eq!(iter.next(), None);
    }

    #[bench]
    fn get_completions(b: &mut Bencher) {
        let trie: Trie = dictionary().into_iter().collect();

        b.iter(|| {
            let _yeet: Vec<String> = trie
                .clone() // The clone is to maintain parity with the other test, which requires cloning
                .children
                .iter()
                .map(|n| n.complete_word("".to_string()))
                .flatten()
                .collect();
        })
    }

    #[bench]
    fn get_completions_iter(b: &mut Bencher) {
        // This is expected to be slower because of the Box indirection present in the iterator
        let trie: Trie = dictionary().into_iter().collect();
        b.iter(|| {
            let _yeet: Vec<String> = trie.clone().into_iter().collect();
        })
    }

    #[test]
    fn remove1() {
        let mut trie: Trie = vec!["hello"].into_iter().collect();

        assert!(trie.remove("hello"));
        assert_eq!(trie.into_iter().next(), None)
    }
    #[test]
    fn remove2() {
        let mut trie: Trie = vec!["hello", "there"].into_iter().collect();

        assert!(trie.remove("hello"));
        assert_eq!(trie.into_iter().next(), Some("there".to_string()))
    }
    #[test]
    fn remove3() {
        let mut trie: Trie = vec!["hello", "he"].into_iter().collect();

        assert!(trie.remove("he"));
        assert_eq!(trie.into_iter().next(), Some("hello".to_string()))
    }
    #[test]
    fn remove4() {
        let mut trie: Trie = vec!["hello", "he"].into_iter().collect();

        assert!(trie.remove("hello"));
        assert_eq!(trie.into_iter().next(), Some("he".to_string()))
    }
    #[test]
    fn remove5() {
        let mut trie: Trie = vec!["a"].into_iter().collect();

        assert!(trie.remove("a"));
        assert_eq!(trie.into_iter().next(), None)
    }
    #[test]
    fn remove6() {
        let mut trie: Trie = vec!["a", "ab"].into_iter().collect();

        assert!(trie.remove("a"));
        assert_eq!(trie.into_iter().next(), Some("ab".to_string()))
    }
    #[test]
    fn remove7() {
        let mut trie: Trie = vec!["a", "ab"].into_iter().collect();

        assert!(trie.remove("ab"));
        assert_eq!(trie.into_iter().next(), Some("a".to_string()))
    }
    #[test]
    fn remove8() {
        let mut trie: Trie = vec!["a", "ab"].into_iter().collect();

        assert!(!trie.remove("abc"));
        let mut iter = trie.into_iter();
        assert_eq!(iter.next(), Some("a".to_string()));
        assert_eq!(iter.next(), Some("ab".to_string()));
        assert_eq!(iter.next(), None)
    }

    #[test]
    fn remove9() {
        let mut trie: Trie = Trie::new();

        assert!(!trie.remove("a"));
        let mut iter = trie.into_iter();
        assert_eq!(iter.next(), None)
    }
}
