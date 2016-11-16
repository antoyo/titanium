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

use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{self, Write};
use std::path::Path;

use xdg::BaseDirectories;

use bookmarks::BookmarkManager;
use credentials::PasswordManager;
use popup_manager::PopupManager;
use super::{App, AppResult, APP_NAME};

impl App {
    /// Create the default configuration files and directories if it does not exist.
    fn create_config_files(&self, config_path: &Path) -> AppResult<()> {
        let xdg_dirs = BaseDirectories::with_prefix(APP_NAME)?;

        let bookmarks_path = BookmarkManager::config_path();
        let passwords_path = PasswordManager::config_path();

        let stylesheets_path = xdg_dirs.place_config_file("stylesheets")?;
        let scripts_path = xdg_dirs.place_config_file("scripts")?;
        let popups_path = xdg_dirs.place_config_file("popups")?;
        create_dir_all(stylesheets_path)?;
        create_dir_all(scripts_path)?;
        create_dir_all(popups_path)?;

        let keys_path = xdg_dirs.place_config_file("keys")?;
        let webkit_config_path = xdg_dirs.place_config_file("webkit")?;
        let hints_css_path = xdg_dirs.place_config_file("stylesheets/hints.css")?;
        let hosts_path = xdg_dirs.place_data_file("hosts")?;
        self.create_default_config_file(config_path, include_str!("../../config/config"))?;
        self.create_default_config_file(&keys_path, include_str!("../../config/keys"))?;
        self.create_default_config_file(&webkit_config_path, include_str!("../../config/webkit"))?;
        self.create_default_config_file(&hints_css_path, include_str!("../../config/stylesheets/hints.css"))?;
        self.create_default_config_file(&bookmarks_path, include_str!("../../config/bookmarks"))?;
        self.create_default_config_file(&hosts_path, include_str!("../../config/hosts"))?;
        self.create_default_config_file(&passwords_path, include_str!("../../config/passwords"))?;

        let (popup_whitelist_path, popup_blacklist_path) = PopupManager::config_path();
        create_file(&popup_whitelist_path)?;
        create_file(&popup_blacklist_path)?;

        Ok(())
    }

    /// Create the config file with its default content if it does not exist.
    fn create_default_config_file(&self, path: &Path, content: &'static str) -> AppResult<()> {
        if !path.exists() {
            let mut file = File::create(path)?;
            write!(file, "{}", content)?;
        }
        Ok(())
    }

    /// Create the variables accessible from the config files.
    pub fn create_variables(&mut self) {
        connect!(self.app, add_variable["url"], self, get_current_url);
    }

    /// Get the webview current URL.
    fn get_current_url(&self) -> String {
        self.webview.get_uri().unwrap()
    }

    /// Create the missing config files and parse the config files.
    pub fn parse_config(&mut self) {
        let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
        let config_path = xdg_dirs.place_config_file("config")
            .expect("cannot create configuration directory");
        handle_error!(self.create_config_files(config_path.as_path()));
        handle_error!(self.app.parse_config(config_path));
    }
}

/// Create a file.
fn create_file(path: &Path) -> io::Result<()> {
    OpenOptions::new().create(true).write(true).open(path)?;
    Ok(())
}
