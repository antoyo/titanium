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

//! Password management.

use password_store::PasswordStore;

use app::APP_NAME;
use errors::Result;
use urls::host;

/// A password manager is used to add, get and remove credentials.
pub struct PasswordManager {
}

impl PasswordManager {
    /// Create a new password manager.
    pub fn new() -> Self {
        PasswordManager {
        }
    }

    /// Add a credential.
    /// Returns true if the credential was added.
    pub fn add(&self, url: &str, username: &str, password: &str) -> Result<()> {
        if let Some(url) = host(url) {
            PasswordStore::insert(&path_username(&url, username), password)?;
        }
        else {
            bail!("Not adding the credentials for {}", url);
        }
        Ok(())
    }

    /// Delete a password.
    /// Returns true if a credential was deleted.
    pub fn delete(&self, url: &str, username: &str) -> Result<()> {
        if let Some(url) = host(url) {
            PasswordStore::remove(&path_username(&url, username))?;
        }
        else {
            bail!("Not deleting the password for {}", url);
        }
        Ok(())
    }

    /// Get the usernames for a `url`.
    pub fn get_usernames(&self, url: &str) -> Result<Vec<String>> {
        if let Some(url) = host(url) {
            Ok(PasswordStore::get_usernames(&path(&url))?)
        }
        else {
            bail!("Cannot get the usernames for {}", url);
        }
    }

    /// Get the password for a `url` and username.
    pub fn get_password(&self, url: &str, username: &str) -> Result<String> {
        if let Some(url) = host(url) {
            Ok(PasswordStore::get(&path_username(&url, username))?)
        }
        else {
            bail!("Cannot get the password for {}", url);
        }
    }
}

fn path(url: &str) -> String {
    format!("{}/{}", APP_NAME, url)
}

fn path_username(url: &str, username: &str) -> String {
    format!("{}/{}/{}", APP_NAME, url, username)
}
