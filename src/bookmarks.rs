/*
 * Copyright (c) 2016 Boucher, Antoni <bouanto@zoho.com>
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

//! Bookmark management.

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Values;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

use serde_yaml;
use xdg::BaseDirectories;

use app::{AppBoolResult, AppResult, APP_NAME};

/// A bookmark has a title and a URL and optionally some tags.
#[derive(Deserialize, Serialize, Debug)]
pub struct Bookmark {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub title: String,
    pub url: String,
}

impl Bookmark {
    /// Create a new bookmark.
    pub fn new(url: String, title: Option<String>) -> Self {
        Bookmark {
            tags: vec![],
            title: title.unwrap_or_default(),
            url: url,
        }
    }
}

/// A bookmark manager is use to add, search and remove bookmarks.
pub struct BookmarkManager {
    bookmarks: HashMap<String, Bookmark>,
    tags: HashSet<String>,
}

impl BookmarkManager {
    /// Create a new bookmark manager.
    pub fn new() -> Self {
        BookmarkManager {
            bookmarks: HashMap::new(),
            tags: HashSet::new(),
        }
    }

    /// Add a bookmark.
    /// Returns true if the bookmark was added.
    pub fn add(&mut self, url: String, title: Option<String>) -> AppBoolResult {
        if self.bookmarks.contains_key(&url) {
            Ok(false)
        }
        else {
            self.bookmarks.insert(url.clone(), Bookmark::new(url, title));
            self.save()?;
            Ok(true)
        }
    }

    /// Get the config path of the bookmarks file.
    pub fn config_path() -> PathBuf {
        let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
        xdg_dirs.place_config_file("bookmarks")
            .expect("cannot create configuration directory")
    }

    /// Delete a bookmark.
    /// Returns true if a bookmark was deleted.
    pub fn delete(&mut self, url: &str) -> AppBoolResult {
        let deleted = self.bookmarks.remove(url).is_some();
        if deleted {
            self.save()?;
        }
        Ok(deleted)
    }

    /// Get the tags of a bookmark.
    pub fn get_tags(&self, url: &str) -> Option<Vec<String>> {
        self.bookmarks.get(url)
            .map(|bookmark| bookmark.tags.clone())
    }

    /// Load the bookmarks from the specified file.
    pub fn load(&mut self) -> AppResult {
        let filename = BookmarkManager::config_path();
        let reader = BufReader::new(File::open(filename)?);
        let bookmarks: Vec<Bookmark> = serde_yaml::from_reader(reader)?;

        for bookmark in bookmarks {
            for tag in &bookmark.tags {
                self.tags.insert(tag.to_lowercase());
            }
            self.bookmarks.insert(bookmark.url.clone(), bookmark);
        }

        Ok(())
    }

    /// Query the bookmarks.
    pub fn query(&self, input: BookmarkInput) -> BookmarkIter {
        BookmarkIter {
            input: input,
            iter: self.bookmarks.values(),
        }
    }

    /// Save the bookmarks to the disk file.
    fn save(&self) -> AppResult {
        let filename = BookmarkManager::config_path();
        let mut writer = BufWriter::new(File::create(filename)?);
        let bookmarks: Vec<_> = self.bookmarks.values().collect();
        let yaml = serde_yaml::to_string(&bookmarks)?;
        write!(writer, "{}", yaml)?;
        Ok(())
    }

    /// Set the tags of a bookmark.
    pub fn set_tags(&mut self, url: &str, tags: Vec<String>) -> AppResult {
        let mut edited = false;
        if let Some(bookmark) = self.bookmarks.get_mut(url) {
            for tag in &tags {
                self.tags.insert(tag.to_lowercase());
            }
            bookmark.tags = tags;
            edited = true;
        }
        if edited {
            self.save()?;
        }
        Ok(())
    }
}

/// A bookmark input query.
pub struct BookmarkInput {
    pub tags: Vec<String>,
    pub words: Vec<String>,
}

/// Bookmark iterator.
pub struct BookmarkIter<'a> {
    input: BookmarkInput,
    iter: Values<'a, String, Bookmark>,
}

impl<'a> Iterator for BookmarkIter<'a> {
    type Item = &'a Bookmark;

    #[allow(while_let_on_iterator)]
    fn next(&mut self) -> Option<Self::Item> {
        'iter:
        while let Some(bookmark) = self.iter.next() {
            for tag in &self.input.tags {
                let mut found = false;
                for bookmark_tag in &bookmark.tags {
                    if bookmark_tag.starts_with(tag) {
                        found = true;
                    }
                }
                if !found {
                    continue 'iter;
                }
            }
            for word in &self.input.words {
                if !(bookmark.title.to_lowercase().contains(word) || bookmark.url.to_lowercase().contains(word)) {
                    continue 'iter;
                }
            }
            return Some(bookmark);
        }
        None
    }
}
