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

//! Host-based adblocker.

use std::fs::{File, read_dir};
use std::io::{self, BufRead, BufReader};

use adblock::engine::Engine;
use adblock::lists::FilterFormat;
use xdg::BaseDirectories;

use super::APP_NAME;

pub struct Adblocker {
    engine: Engine,
}

impl Adblocker {
    /// Create a new adblocker.
    pub fn new() -> Self {
        Adblocker {
            engine: Engine::from_rules(&Adblocker::load_lists_urls(), FilterFormat::Standard),
        }
    }

    /// Get the blacklisted urls.
    fn load_lists_urls() -> Vec<String> {
        let mut result = vec![];
        let xdg_dirs = unwrap_or_ret!(BaseDirectories::with_prefix(APP_NAME), result);
        let lists_path = unwrap_or_ret!(xdg_dirs.place_config_file("adblocklists"), result);

        for entry in unwrap_or_ret!(read_dir(&lists_path), result) {
            if let Ok(entry) = entry {
                let list = lists_path.join(entry.path());
                if let Ok(file) = File::open(&list) {
                    let file = BufReader::new(file);
                    let urls: io::Result<Vec<_>> = file.lines().collect();
                    if let Ok(urls) = urls {
                        result.extend(urls);
                    }
                }
            }
        }

        result
    }

    /// Check if the specified url should be blocked.
    pub fn should_block(&self, url: &str) -> bool {
        let blocker_result = self.engine.check_network_urls(url, "", ""); // TODO: add missing parameters?
        // TODO: check the exception field to make whitelist working?
        // @@||github.com/adgear/^$document
        blocker_result.matched
    }
}
