//! A spell-checker based on the statistical algorithm described by Peter Norvig
//! in <http://norvig.com/spell-correct.html>.
//!
//! Usage requires a two-step process:
//! 1) Call `url.train()` one or more times with a large text to train the language model
//! 2) Call `url.correct(word)` to retrieve the correction for the specified URL if it exists

use regex::Regex;
use std::collections::HashMap;
use std::thread;

/// We hold all valid url characters and the frequency of how many times a URL is visited.
#[derive(Debug)]
pub struct URL {
    /// Possible characters included in a link, Eg:- `"1234567890._-@abcdefghijklmnopqrstuvwxyz"`
    pub letters: String,
    /// A frequency map of URL(s) to the number of times they were found during training. Where
    /// training means to extract URLs that occur between `//` url `/`, they can be of type `https`
    /// or `ftp`, etc.
    pub url_counts: HashMap<String, u32>,
}

impl URL {
    /// A function that trains the language model with the words in the supplied text.
    /// Multiple invocation of this function can extend the training of the model.
    ///
    /// By training here we mean to extract URLs and store them in a [`hashmap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
    /// alongside their frequency count.
    ///
    /// **NOTE:** The URLs are presumed in the form of `<something>//url/<something>`, we only care
    /// about the parent node.
    pub fn train(&mut self, text: &str) {
        let re = Regex::new(r"//(?P<site>[a-zA-Z0-9._-]+)/").unwrap();
        // let re = Regex::new(r"[a-z]+").unwrap();
        let lc_text = text.to_lowercase();
        for m in re.captures_iter(&lc_text) {
            let count = self.url_counts.entry(m["site"].to_string()).or_insert(0);
            *count += 1;
        }
    }

    /// A function that returns the correction for the specified URL.
    ///
    /// We return the URL if it is a "valid" or else we try to find it in edits and if not then edit
    /// of those edits, otherwise return `None`.
    ///
    /// - `edits` itself is parallelized for faster performance.
    pub fn correct(&mut self, word: &str) -> Option<String> {
        // A word in our word frequency map is already correct.
        if self.url_counts.contains_key(word) {
            return Some(word.to_string());
        }

        let mut candidates: HashMap<u32, String> = HashMap::new();
        let list = self.edits(word);

        // Try to find candidate corrections in the edits of the word.
        for edit in &list {
            if let Some(value) = self.url_counts.get(edit) {
                candidates.insert(*value, edit.to_string());
            }
        }
        if let Some(c) = candidates.iter().max_by_key(|&entry| entry.0) {
            return Some(c.1.to_string());
        }

        // Try to find candidate corrections in the edits of the edits.
        for edit in &list {
            for w in self.edits(edit) {
                if let Some(value) = self.url_counts.get(&w) {
                    candidates.insert(*value, w);
                }
            }
        }
        if let Some(c) = candidates.iter().max_by_key(|&entry| entry.0) {
            return Some(c.1.to_string());
        }

        // Can't find a correction, return None
        // word.to_string()
        None
    }

    /// A function that returns the set of possible corrections of the specified URL. Return a `Vec`
    /// containing all the edited `strings`.
    ///
    /// The edits can be deletions, insertions, alterations or transpositions all processed in parallel
    /// at the same time.
    pub fn edits(&mut self, word: &str) -> Vec<String> {
        // preallocate the size as it is a known value
        let mut results =
            Vec::with_capacity(2 * word.len() * (1 + self.letters.len()) + self.letters.len());
        let mut deletion_results = Vec::with_capacity(word.len());
        let mut transposition_results = Vec::with_capacity(word.len() - 1);
        let mut alteration_results = Vec::with_capacity(self.letters.len() * word.len());
        let mut insertion_results = Vec::with_capacity(self.letters.len() * (word.len() + 1));

        // Make all edits in parallel to increase performance
        thread::scope(|s| {
            // deletion
            s.spawn(|| {
                for i in 0..word.len() {
                    let (first, last) = word.split_at(i);
                    deletion_results.push([first, &last[1..]].concat());
                }
            });

            // transposition
            s.spawn(|| {
                for i in 0..word.len() - 1 {
                    let (first, last) = word.split_at(i);
                    transposition_results
                        .push([first, &last[1..2], &last[..1], &last[2..]].concat());
                }
            });

            // alteration
            s.spawn(|| {
                for i in 0..word.len() {
                    for c in self.letters.chars() {
                        let (first, last) = word.split_at(i);
                        let mut buffer = [0; 1];
                        let result = c.encode_utf8(&mut buffer);
                        alteration_results.push([first, result, &last[1..]].concat());
                    }
                }
            });

            // insertion
            s.spawn(|| {
                for i in 0..word.len() + 1 {
                    for c in self.letters.chars() {
                        let (first, last) = word.split_at(i);
                        let mut buffer = [0; 1];
                        let result = c.encode_utf8(&mut buffer);
                        insertion_results.push([first, result, last].concat());
                    }
                }
            });
        });

        // Finally append all of them into results
        results.append(&mut deletion_results);
        results.append(&mut transposition_results);
        results.append(&mut alteration_results);
        results.append(&mut insertion_results);

        //println!("{:#?}" , results);

        results
    }
}

#[cfg(test)]
mod tests {
    use crate::URL;
    use std::collections::HashMap;

    #[test]
    fn test_correcting() {
        let mut url = URL {
            letters: "1234567890._-@abcdefghijklmnopqrstuvwxyz".to_string(),
            url_counts: HashMap::new(),
        };
        url.train("https://docs.rs/regex/latest/regex/ https://norvig.com/spell-correct.html https://doc.rust-lang.org/stable/std/thread/fn.scope.html");
        // deletion
        assert_eq!(url.correct("doc.rs"), Some("docs.rs".to_string()));
        // transposition
        assert_eq!(url.correct("dcos.rs"), Some("docs.rs".to_string()));
        // alteration
        assert_eq!(url.correct("docs.rr"), Some("docs.rs".to_string()));
        assert_eq!(url.correct("doks.rs"), Some("docs.rs".to_string()));
        assert_eq!(url.correct("d0cs.rs"), Some("docs.rs".to_string()));
        // insertion
        assert_eq!(url.correct("docs.rss"), Some("docs.rs".to_string()));
        assert_eq!(url.correct("docks.rs"), Some("docs.rs".to_string()));
    }
}
