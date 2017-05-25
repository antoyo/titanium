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

//! Manage the configuration of the application.

use std::fs::OpenOptions;
use std::io;
use std::path::Path;

use mg::DefaultConfig::{self, Dir, File};
use relm::Widget;

use config_dir::ConfigDir;
use super::App;

impl App {
    /// Create the variables accessible from the config files.
    pub fn create_variables(&mut self) {
        let webview = self.webview.widget().root().clone();
        self.mg.widget_mut().set_variables(vec![("url", Box::new(move || {
            webview.get_uri().unwrap_or_default()
        }))]);
    }
}

/// Create a file.
fn create_file(path: &Path) -> io::Result<()> {
    OpenOptions::new().create(true).write(true).open(path)?;
    Ok(())
}

/// Get the default configuration files and directories.
pub fn default_config(config_dir: &ConfigDir) -> Vec<DefaultConfig> {
    let stylesheets_path = config_dir.config_file("stylesheets");
    let scripts_path = config_dir.config_file("scripts");
    let popups_path = config_dir.config_file("popups");

    let config_path = config_dir.config_file("config");
    let keys_path = config_dir.config_file("keys");
    let webkit_config_path = config_dir.config_file("webkit");
    let hints_css_path = config_dir.config_file("stylesheets/hints.css");
    let hosts_path = config_dir.data_file("hosts");
    let (popup_whitelist_path, popup_blacklist_path) = App::popup_path(config_dir);

    vec![Dir(stylesheets_path),
         Dir(scripts_path),
         Dir(popups_path),
         Dir(Ok(config_dir.data_home())),
         File(keys_path, include_str!("../../config/keys")),
         File(config_path, include_str!("../../config/config")),
         File(webkit_config_path, include_str!("../../config/webkit")),
         File(hints_css_path, include_str!("../../config/stylesheets/hints.css")),
         File(hosts_path, include_str!("../../config/hosts")),
         File(popup_whitelist_path, ""),
         File(popup_blacklist_path, ""),
        ]
}
