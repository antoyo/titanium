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

use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use app::App;
use managers::ConfigDir;
use errors::Result;
use urls::get_base_url;

/// Manager to know whether a popup should be always or never opened.
pub struct PopupManager {
    blacklisted_urls: HashSet<String>,
    blacklist_path: PathBuf,
    whitelisted_urls: HashSet<String>,
    whitelist_path: PathBuf,
}

impl PopupManager {
    /// Create a new popup manager.
    pub fn new(whitelist_path: PathBuf, blacklist_path: PathBuf) -> Self {
        PopupManager {
            blacklisted_urls: HashSet::new(),
            blacklist_path,
            whitelisted_urls: HashSet::new(),
            whitelist_path,
        }
    }

    /// Blacklist the specified url.
    pub fn blacklist(&mut self, url: &str) -> Result<()> {
        if let Some(url) = get_base_url(url) {
            self.blacklisted_urls.insert(url.to_string());
            self.save_blacklist()
        }
        else {
            warn!("Not blacklisting {}", url);
            Ok(())
        }
    }

    /// Check if the specified url is blacklisted.
    pub fn is_blacklisted(&self, url: &str) -> bool {
        self.blacklisted_urls.contains(&get_base_url(url).unwrap_or_else(String::new))
    }

    /// Check if the specified url is whitelisted.
    pub fn is_whitelisted(&self, url: &str) -> bool {
        self.whitelisted_urls.contains(&get_base_url(url).unwrap_or_else(String::new))
    }

    /// Load the urls from the files.
    pub fn load(&mut self) -> Result<()> {
        self.blacklisted_urls = self.read_as_set(&self.blacklist_path)?;
        self.whitelisted_urls = self.read_as_set(&self.whitelist_path)?;
        Ok(())
    }

    /// Read a file as a HashSet where all lines are one entry in the set.
    fn read_as_set(&self, path: &PathBuf) -> Result<HashSet<String>> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        let _ = file.read_to_string(&mut content)?;
        let set = content.lines()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()).collect();
        Ok(set)
    }

    /// Save the list in the file specified by `path`.
    fn save(&self, path: &PathBuf, list: &HashSet<String>) -> Result<()> {
        let mut file = File::create(path)?;
        for url in list {
            writeln!(file, "{}", url)?;
        }
        Ok(())
    }

    /// Save the popup blacklist.
    fn save_blacklist(&self) -> Result<()> {
        self.save(&self.blacklist_path, &self.blacklisted_urls)
    }

    /// Save the popup whitelist.
    fn save_whitelist(&self) -> Result<()> {
        self.save(&self.whitelist_path, &self.whitelisted_urls)
    }

    /// Whitelist the specified url.
    pub fn whitelist(&mut self, url: &str) -> Result<()> {
        if let Some(url) = get_base_url(url) {
            self.whitelisted_urls.insert(url.to_string());
            self.save_whitelist()
        }
        else {
            warn!("Not whitelisting {}", url);
            Ok(())
        }
    }
}

/// Create a popup manager if the blacklist/whitelist paths can be created.
pub fn create_popup_manager(config_dir: &ConfigDir) -> Option<PopupManager> {
    if let (Ok(whitelist_path), Ok(blacklist_path)) = App::popup_path(config_dir) {
        Some(PopupManager::new(whitelist_path, blacklist_path))
    }
    else {
        None
    }
}
