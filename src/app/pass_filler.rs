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

use super::{App, AppResult};

impl App {
    /// Delete the password for the current URL.
    pub fn delete_password(&mut self) {
        if self.webview.has_multiple_passwords() {
            // TODO
        }
        else {
            match self.webview.delete_password() {
                Ok(true) => self.app.info("Password deleted"),
                Ok(false) => self.app.info("No password for the current URL"),
                Err(err) => self.show_error(err),
            }
        }
    }

    /// Load the username and password in the login form.
    /// If multiple credentials exist, ask the user which one to use.
    /// Return true if a login form was filled.
    pub fn load_password(&mut self) -> AppResult<bool> {
        if self.webview.has_multiple_passwords() {
            // TODO
        }
        else {
            match self.webview.load_password() {
                Ok(true) => return Ok(true),
                Ok(false) => self.app.info("No password for the current URL"),
                Err(err) => self.show_error(err),
            }
        }
        Ok(false)
    }

    /// Save the password from the currently focused login form into the store.
    pub fn save_password(&mut self) {
        match self.webview.save_password() {
            Ok(true) => self.app.info("Password added"),
            Ok(false) => self.app.info("A password is already in the store for the current URL"), // TODO: ask for a confirmation to overwrite.
            Err(err) => self.show_error(err),
        }
    }

    /// Load the username and password in the login form and submit it.
    pub fn submit_login_form(&mut self) {
        if let Ok(true) = self.load_password() {
            handle_error!(self.webview.submit_login_form());
        }
    }
}
