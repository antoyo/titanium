/*
 * Copyright (c) 2016-2018 Boucher, Antoni <bouanto@zoho.com>
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

use std::cmp::Ordering::{Greater, Less};
use std::fs::read_dir;
use std::path::{Path, PathBuf};

use mg::completion::Column::{self, AllVisible, Expand};
use mg::completion::{Completer, CompletionCell, CompletionResult};

use app::user_agent::UserAgentManager;
use app::USER_AGENT_COMPLETER;
use bookmarks::{BookmarkInput, BookmarkManager};
use download::download_dir;

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
                tag.remove(0); // Remove the #.
                if !tag.is_empty() {
                    tags.push(tag);
                }
            } else {
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
            let tags = if !bookmark.tags.is_empty() {
                format!("#{}", bookmark.tags)
            } else {
                String::new()
            };
            results.push(CompletionResult::from_cells(&[
                &bookmark.title,
                &CompletionCell::new(&tags).foreground("#33DD00"),
                &bookmark.url,
            ]));
        }

        results
    }

    fn text_column(&self) -> i32 {
        2
    }
}

/// A file completer.
pub struct FileCompleter {
    current_directory: PathBuf,
}

impl FileCompleter {
    /// Create a new file completer.
    pub fn new() -> Self {
        let path = Path::new(&download_dir()).to_path_buf();
        FileCompleter {
            current_directory: path,
        }
    }
}

impl Completer for FileCompleter {
    fn columns(&self) -> Vec<Column> {
        vec![Expand]
    }

    fn complete_result(&self, value: &str) -> String {
        let absolute_path = self.current_directory.join(value);
        // Remove the trailing slash in the completion to avoid updating the completions for a new
        // directory when selecting a directory.
        // This means the user needs to type the slash to trigger the completion of the new
        // directory.
        absolute_path
            .to_str()
            .unwrap()
            .trim_right_matches('/')
            .to_string()
    }

    fn completions(&mut self, input: &str) -> Vec<CompletionResult> {
        let mut paths = vec![];
        let input_path = Path::new(input).to_path_buf();
        // If the input ends with /, complete within this directory.
        // Otherwise, complete the files from the parent directory.
        let path = if !input.ends_with('/') {
            input_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or(input_path)
        } else {
            input_path
        };
        self.current_directory = path.clone();
        if let Ok(entries) = read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    let matched = {
                        let absolute_path_string = path.to_str().unwrap();
                        let path_string = path.file_name().unwrap().to_str().unwrap();
                        // Do not show hidden files (starting with dot).
                        !path_string.starts_with('.') && absolute_path_string.starts_with(input)
                    };
                    if matched {
                        paths.push(path);
                    }
                }
            }
        }
        // Sort directories first, then sort by name.
        paths.sort_by(|path1, path2| match (path1.is_dir(), path2.is_dir()) {
            (true, false) => Less,
            (false, true) => Greater,
            _ => path1.cmp(path2),
        });
        paths
            .iter()
            .map(|path| {
                let filename = path.file_name().unwrap().to_str().unwrap();
                if path.is_dir() {
                    let mut filename = filename.to_string();
                    filename.push('/');
                    CompletionResult::from_cells(&[
                        &CompletionCell::new(&filename).foreground("#33FF33")
                    ])
                } else {
                    CompletionResult::new(&[&filename.to_string()])
                }
            })
            .collect()
    }
}

/// A tag completer.
pub struct TagCompleter {
    bookmarks: BookmarkManager,
    current_tags: String,
}

impl TagCompleter {
    pub fn new() -> Self {
        Self {
            bookmarks: BookmarkManager::new(),
            current_tags: String::new(),
        }
    }
}

impl Completer for TagCompleter {
    fn columns(&self) -> Vec<Column> {
        vec![Expand]
    }

    fn complete_result(&self, value: &str) -> String {
        if self.current_tags.is_empty() {
            value.to_string()
        } else {
            format!("{} {}", self.current_tags, value)
        }
    }

    fn completions(&mut self, input: &str) -> Vec<CompletionResult> {
        let index = input.rfind(',').map(|index| index + 1).unwrap_or(0);
        let query = input[index..].trim();
        self.current_tags = input[..index].to_string();
        if let Ok(tags) = self.bookmarks.search_tags(query) {
            tags.iter()
                .map(|tag| CompletionResult::new(&[&tag]))
                .collect()
        } else {
            vec![]
        }
    }

    fn have_command(&self) -> bool {
        false
    }
}

/// A user agent completer.
pub struct UserAgentCompleter {
    manager: UserAgentManager,
}

impl UserAgentCompleter {
    pub fn new() -> Self {
        Self {
            manager: UserAgentManager,
        }
    }
}

impl Completer for UserAgentCompleter {
    fn columns(&self) -> Vec<Column> {
        vec![AllVisible]
    }

    fn complete_result(&self, value: &str) -> String {
        format!("{} {}", USER_AGENT_COMPLETER, value)
    }

    fn completions(&mut self, input: &str) -> Vec<CompletionResult> {
        let mut results = vec![];
        let names = self.manager.get_all();
        for name in names {
            if name.contains(input) {
                results.push(CompletionResult::new(&[&name]));
            }
        }
        results
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
        } else if character == ' ' {
            if !buffer.is_empty() {
                words.push(buffer.clone());
                buffer.clear();
            }
        } else {
            buffer.push(character);
        }
    }
    if !buffer.is_empty() {
        words.push(buffer.clone());
    }
    words
}
