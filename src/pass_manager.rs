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

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::ops::Not;
use std::path::PathBuf;

use password_store::PasswordStore;
use serde_yaml;

use app::{AppResult, APP_NAME};
use settings::PasswordStorage;
use urls::base_url;

/// Username/password for a website.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Credential {
    #[serde(default, skip_serializing_if = "Not::not")]
    pub check: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub password: String,
    pub username: String,
}

impl Credential {
    /// Create a new credential.
    fn new(username: &str, check: bool) -> Self {
        Credential {
            check: check,
            password: String::new(),
            username: username.to_string(),
        }
    }
}

/// Credentials is a map of URL to a list of usernames.
type Credentials = BTreeMap<String, Vec<Credential>>;

/// A password manager is used to add, get and remove credentials.
pub struct PasswordManager {
    credentials: Credentials,
    filename: PathBuf,
    storage: PasswordStorage,
}

impl PasswordManager {
    /// Create a new password manager.
    pub fn new(filename: PathBuf) -> Self {
        PasswordManager {
            credentials: BTreeMap::new(),
            filename: filename,
            storage: PasswordStorage::Cleartext,
        }
    }

    /// Add a credential.
    /// Returns true if the credential was added.
    pub fn add(&mut self, url: &str, username: &str, password: &str, check: bool) -> AppResult<bool> {
        let url = base_url(url);
        let mut found = false;
        {
            let credentials = self.credentials.entry(url.to_string()).or_insert_with(Vec::new);
            for credential in credentials.iter() {
                if credential.username == username {
                    found = true;
                }
            }
            let mut credential = Credential::new(username, check);
            if self.storage == PasswordStorage::Cleartext {
                credential.password = password.to_string();
            }
            credentials.push(credential);
        }
        if self.storage == PasswordStorage::Pass {
            PasswordStore::insert(&PasswordManager::path(&url, username), password)?;
        }
        self.save()?;
        Ok(!found)
    }

    /// Delete a password.
    /// Returns true if a credential was deleted.
    pub fn delete(&mut self, url: &str, username: &str) -> AppResult<bool> {
        let url = base_url(url);
        let mut deleted = false;
        let mut delete_url = false;
        if let Some(credentials) = self.credentials.get_mut(&url) {
            let last_len = credentials.len();
            credentials.retain(|credential| credential.username == username);
            deleted = credentials.len() != last_len;
            delete_url = credentials.is_empty();
            if deleted && self.storage == PasswordStorage::Pass {
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
        if self.storage == PasswordStorage::Pass {
            let url = base_url(url);
            Ok(PasswordStore::get(&PasswordManager::path(&url, username))?)
        }
        else if let Some(credentials) = self.get_credentials(url) {
            for credential in credentials {
                if credential.username == username {
                    return Ok(credential.password.clone());
                }
            }
            // TODO: implement proper error handling.
            panic!("No credential for the current URL and username".to_string());
        }
        else {
            // TODO: implement proper error handling.
            panic!("No credential for the current URL".to_string());
        }
    }

    /// Load the usernames.
    pub fn load(&mut self) -> AppResult<()> {
        let reader = BufReader::new(File::open(&self.filename)?);
        self.credentials = serde_yaml::from_reader(reader)?;

        Ok(())
    }

    /// Get the password store path from the `url` and `username`.
    fn path(url: &str, username: &str) -> String {
        format!("{}/{}/{}", APP_NAME, url, username)
    }

    /// Save the credentials to the disk file.
    fn save(&self) -> AppResult<()> {
        let mut writer = BufWriter::new(File::create(&self.filename)?);
        let yaml = serde_yaml::to_string(&self.credentials)?;
        write!(writer, "{}", yaml)?;
        Ok(())
    }

    /// Set the password storage.
    pub fn set_storage(&mut self, storage: &PasswordStorage) {
        self.storage = storage.clone();
    }
}
