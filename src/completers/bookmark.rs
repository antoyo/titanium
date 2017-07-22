/*
 * Copyright (c) 2016-2017 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

use mg::completion::{Completer, CompletionCell, CompletionResult};
use mg::completion::Column::{self, AllVisible, Expand};

use managers::bookmarks::{BookmarkInput, BookmarkManager};

/// A bookmark completer.
pub struct BookmarkCompleter {
    bookmarks: BookmarkManager,
    prefix: &'static str,
}

impl BookmarkCompleter {
    /// Create a new bookmark completer.
    pub fn new(prefix: &'static str) -> Self {
        BookmarkCompleter {
            bookmarks: BookmarkManager::new(),
            prefix: prefix,
        }
    }

    /// Parse the tags and the words from the input.
    fn parse_input(input: &str) -> BookmarkInput {
        let mut tags = vec![];
        let mut words = vec![];
        let splitted_words = split_whitespace_and_hash(&input.to_lowercase());

        for word in splitted_words {
            if word.starts_with('#') {
                let mut tag = word.to_string();
                let _ = tag.remove(0); // Remove the #.
                if !tag.is_empty() {
                    tags.push(tag);
                }
            }
            else {
                words.push(word);
            }
        }

        BookmarkInput {
            tags: tags,
            words: words,
        }
    }
}

impl Completer for BookmarkCompleter {
    fn columns(&self) -> Vec<Column> {
        vec![Expand, AllVisible, Expand]
    }

    fn complete_result(&self, value: &str) -> String {
        format!("{} {}", self.prefix, value)
    }

    fn completions(&mut self, input: &str) -> Vec<CompletionResult> {
        let mut results = vec![];
        let query = BookmarkCompleter::parse_input(input);

        for bookmark in self.bookmarks.query(query) {
            let tags =
                if !bookmark.tags.is_empty() {
                    format!("#{}", bookmark.tags)
                }
                else {
                    String::new()
                };
            results.push(CompletionResult::from_cells(
                &[&bookmark.title, &CompletionCell::new(&tags).foreground("#33DD00"), &bookmark.url],
            ));
        }

        results
    }

    fn text_column(&self) -> i32 {
        2
    }
}



/// Split at whitespaces and at the # character.
/// The # character will be kept in the words while the spaces are dropped.
fn split_whitespace_and_hash(input: &str) -> Vec<String> {
    let mut words = vec![];
    let mut buffer = String::new();
    for character in input.chars() {
        if character == '#' {
            if !buffer.is_empty() {
                words.push(buffer.clone());
                buffer.clear();
            }
            buffer.push('#');
        }
        else if character == ' ' {
            if !buffer.is_empty() {
                words.push(buffer.clone());
                buffer.clear();
            }
        }
        else {
            buffer.push(character);
        }
    }
    if !buffer.is_empty() {
        words.push(buffer.clone());
    }
    words
}
