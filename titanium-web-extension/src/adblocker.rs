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

//! Host-based adblocker.

use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::mem;
use std::path::PathBuf;

use url::Url;
use xdg::BaseDirectories;

use super::APP_NAME;

pub struct Adblocker {
    blacklisted_urls: HashSet<String>,
}

impl Adblocker {
    /// Create a new adblocker.
    pub fn new() -> Self {
        Adblocker {
            blacklisted_urls: Adblocker::load_blacklisted_urls(),
        }
    }

    /// Get the hosts path.
    pub fn config_path() -> io::Result<PathBuf> {
        let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
        xdg_dirs.place_data_file("hosts")
    }

    /// Get the blacklisted urls.
    fn load_blacklisted_urls() -> HashSet<String> {
        let mut blacklisted_urls = HashSet::new();
        if let Ok(path) = Adblocker::config_path() {
            match File::open(&path) {
                Ok(file) => {
                    let mut reader = BufReader::new(file);
                    let mut line = String::new();
                    let mut size_read = 1;
                    while size_read > 0 {
                        match reader.read_line(&mut line) {
                            Ok(size) => {
                                size_read = size;
                                if size_read > 0 {
                                    line.pop(); // Remove the leading newline.
                                    blacklisted_urls.insert(mem::replace(&mut line, String::new()));
                                }
                            }
                            Err(error) => {
                                error!("Error: {}", error);
                                break;
                            },
                        }
                    }
                }
                Err(_) => {
                    error!("Cannot open hosts file {}", path.to_str().unwrap_or_default())
                },
            }
        }
        else {
            warn!("Cannot find hosts file for the ad blocker");
        }
        blacklisted_urls
    }

    /// Check if the specified url should be blocked.
    pub fn should_block(&self, url: &str) -> bool {
        if let Some(host) = get_url_host(url) {
            let result = self.blacklisted_urls.contains(&host);
            if result {
                info!("Blocked URL {}", host);
            }
            result
        }
        else {
            false
        }
    }
}

/// Get the host from the URL.
fn get_url_host(url: &str) -> Option<String> {
    Url::parse(url).ok()
        .and_then(|url| url.host_str().map(ToString::to_string))
}
