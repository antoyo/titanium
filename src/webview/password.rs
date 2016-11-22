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

use app::AppResult;
use super::WebView;

impl WebView {
    /// Delete the username and password for the current URL.
    pub fn delete_password(&mut self) -> AppResult<bool> {
        if let Some(url) = self.view.get_uri() {
            let username =
                if let Some(credentials) = self.password_manager.get_credentials(&url) {
                    Some(credentials[0].username.clone())
                }
                else {
                    None
                };
            if let Some(username) = username {
                self.password_manager.delete(&url, &username)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Check if there are multiple passwords for the current URL.
    pub fn has_multiple_passwords(&self) -> bool {
        if let Some(url) = self.view.get_uri() {
            if let Some(credentials) = self.password_manager.get_credentials(&url) {
                return credentials.len() > 1;
            }
        }
        false
    }

    /// Load the username and password in the login form.
    pub fn load_password(&self) -> AppResult<bool> {
        if let Some(url) = self.view.get_uri() {
            if let Some(credentials) = self.password_manager.get_credentials(&url) {
                let credential = &credentials[0];
                let password = self.password_manager.get_password(&url, &credential.username)?;
                self.message_server.load_username(&credential.username)?;
                self.message_server.load_password(&password)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Load the passwords into the password manager.
    pub fn load_passwords(&mut self) -> AppResult<()> {
        self.password_manager.load()
    }

    /// Save the password from the login form.
    pub fn save_password(&mut self) -> AppResult<bool> {
        // TODO: ask to override existing password.
        let (username, password) = self.message_server.get_credentials()?;
        if let Some(url) = self.view.get_uri() {
            // TODO: handle the check parameter.
            return self.password_manager.add(&url, &username, &password, false);
        }
        Ok(false)
    }

    /// Submit the login form.
    pub fn submit_login_form(&self) -> AppResult<()> {
        self.message_server.submit_login_form()?;
        Ok(())
    }
}
