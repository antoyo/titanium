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

use errors::Result;
use super::WebView;

impl WebView {
    /// Delete the username and password for the current URL.
    pub fn delete_password(&mut self) {
        if let Some(url) = self.view.get_uri() {
            /*connect!(self.password_manager, get_usernames[&url.clone()](usernames),
                self, delete_password_usernames(&url, usernames));*/
        }
    }

    fn delete_password_usernames(&mut self, url: &str, usernames: Vec<String>) {
        if !usernames.is_empty() {
            // TODO: ask for which username to delete.
            let username = &usernames[0];
            //self.password_manager.delete(url, username);
            // TODO: show info or error.
        }
    }

    /*fn load_password(&self, password: Result<String, PasswordError>) {
        // TODO: handle errors.
        if let Ok(password) = password {
            self.message_server.load_password(&password).ok();
        }
    }*/

    fn load_password_usernames(&mut self, url: &str, usernames: Vec<String>) {
        if !usernames.is_empty() {
            let username = &usernames[0];
            // TODO: handle errors.
            //connect!(self.password_manager, get_password[url, username](password), self, load_password(password));
            //self.message_server.load_username(username).ok();
        }
    }

    /// Load the username and password in the login form.
    pub fn load_username_password(&mut self) {
        if let Some(url) = self.view.get_uri() {
            /*connect!(self.password_manager, get_usernames[&url.clone()](usernames),
                self, load_password_usernames(&url, usernames));*/
        }
    }

    /// Save the password from the login form.
    pub fn save_password(&self) {
        // TODO: ask to override existing password.
        // TODO: handle errors.
        /*if let Ok((username, password)) = self.message_server.get_credentials() {
            if let Some(url) = self.view.get_uri() {
                // TODO: handle the check parameter.
                self.password_manager.add(&url, &username, &password, false);
            }
        }*/
    }

    /// Submit the login form.
    pub fn submit_login_form(&self) -> Result<()> {
        //self.message_server.submit_login_form()?;
        Ok(())
    }
}
