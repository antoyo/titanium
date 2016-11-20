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

//! Password management.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::ops::Not;
use std::path::PathBuf;

use password_store::PasswordStore;
use serde_yaml;
use xdg::BaseDirectories;

use app::{AppResult, APP_NAME};
use urls::base_url;

/// Username/password for a website.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Credential {
    #[serde(default, skip_serializing_if = "Not::not")]
    pub check: bool,
    pub username: String,
}

impl Credential {
    /// Create a new credential.
    fn new(username: &str, check: bool) -> Self {
        Credential {
            check: check,
            username: username.to_string(),
        }
    }
}

/// Credentials is a map of URL to a list of usernames.
type Credentials = HashMap<String, Vec<Credential>>;

/// A password manager is used to add, get and remove credentials.
pub struct PasswordManager {
    credentials: Credentials,
}

impl PasswordManager {
    /// Create a new password manager.
    pub fn new() -> Self {
        PasswordManager {
            credentials: HashMap::new(),
        }
    }

    /// Add a credential.
    /// Returns true if the credential was added.
    pub fn add(&mut self, url: &str, username: &str, password: &str, check: bool) -> AppResult<bool> {
        let url = base_url(url);
        let mut added = false;
        {
            let credentials = self.credentials.entry(url.to_string()).or_insert_with(Vec::new);
            for credential in credentials.iter() {
                if credential.username == username {
                    added = true;
                }
            }
            credentials.push(Credential::new(username, check));
        }
        PasswordStore::insert(&PasswordManager::path(&url, username), password)?;
        self.save()?;
        Ok(added)
    }

    /// Get the config path of the password file.
    pub fn config_path() -> PathBuf {
        let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
        xdg_dirs.place_config_file("passwords")
            .expect("cannot create configuration directory")
    }

    /// Delete a password.
    /// Returns true if a credential was deleted.
    pub fn _delete(&mut self, url: &str, username: &str) -> AppResult<bool> {
        let url = base_url(url);
        let mut deleted = false;
        let mut delete_url = false;
        if let Some(credentials) = self.credentials.get_mut(&url) {
            let last_len = credentials.len();
            credentials.retain(|credential| credential.username == username);
            deleted = credentials.len() != last_len;
            delete_url = credentials.is_empty();
            if deleted {
                PasswordStore::remove(&PasswordManager::path(&url, username))?;
            }
        }
        if delete_url {
            self.credentials.remove(&url);
        }
        if deleted {
            self.save()?;
        }
        Ok(deleted)
    }

    /// Get the credentials for a `url`.
    pub fn get_credentials(&self, url: &str) -> Option<&[Credential]> {
        let url = base_url(url);
        self.credentials.get(&url)
            .map(Vec::as_slice)
    }

    /// Get the password for a `url` and username.
    pub fn get_password(&self, url: &str, username: &str) -> AppResult<String> {
        let url = base_url(url);
        let password = PasswordStore::get(&PasswordManager::path(&url, username))?;
        Ok(password)
    }

    /// Load the usernames.
    pub fn load(&mut self) -> AppResult<()> {
        let filename = PasswordManager::config_path();
        let reader = BufReader::new(File::open(filename)?);
        self.credentials = serde_yaml::from_reader(reader)?;

        Ok(())
    }

    /// Get the password store path from the `url` and `username`.
    fn path(url: &str, username: &str) -> String {
        format!("{}/{}/{}", APP_NAME, url, username)
    }

    /// Save the credentials to the disk file.
    fn save(&self) -> AppResult<()> {
        let filename = PasswordManager::config_path();
        let mut writer = BufWriter::new(File::create(filename)?);
        let yaml = serde_yaml::to_string(&self.credentials)?;
        write!(writer, "{}", yaml)?;
        Ok(())
    }
}
