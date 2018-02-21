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

use titanium_common::InnerMessage::{
    GetCredentials,
    InsertText,
    LoadUsernamePass,
    SubmitLoginForm,
};

use super::App;

use errors::Result;

impl App {
    fn decrypt_username_if_needed<'a>(&self, username: &'a str, password: &'a str) -> (&'a str, &'a str) {
        if username == "__titanium_encrypted_username" {
            let mut password_parts = password.split('\n');
            let username = password_parts.next().unwrap_or("");
            let password = password_parts.next().unwrap_or("");
            (username, password)
        }
        else {
            (username, password)
        }
    }

    /// Delete the password for the current URL.
    pub fn delete_password(&self) -> Result<()> {
        let usernames = self.model.password_manager.get_usernames(&self.model.current_url)?;
        if !usernames.is_empty() {
            // TODO: ask for which username to delete.
            let username = &usernames[0];
            self.model.password_manager.delete(&self.model.current_url, username)?;
        }
        /*Ok(true) => self.app.info("Password deleted"),
          Ok(false) => self.app.info("No password for the current URL"),
          Err(err) => self.error(err.to_string()),*/
        Ok(())
    }

    /// Insert a password in the focused text input.
    pub fn insert_password(&mut self) -> Result<()> {
        let usernames = self.model.password_manager.get_usernames(&self.model.current_url)?;
        if !usernames.is_empty() {
            // TODO: ask for which username to insert.
            let username = &usernames[0];
            let password = self.model.password_manager.get_password(&self.model.current_url, username)?;
            let (_username, password) = self.decrypt_username_if_needed(username, &password);
            self.server_send(InsertText(password.to_string()));
        }
        Ok(())
    }

    /// Insert a password in the focused text input and submit.
    pub fn insert_password_submit(&mut self) -> Result<()> {
        self.insert_password()?;
        self.server_send(SubmitLoginForm());
        Ok(())
    }

    /// Load the username and password in the login form.
    /// If multiple credentials exist, ask the user which one to use.
    /// Return true if a login form was filled.
    pub fn load_password(&mut self) -> Result<()> {
        /*Ok(true) => return Ok(true),
          Ok(false) => self.app.info("No password for the current URL"),
          Err(err) => self.error(err.to_string()),*/
        let usernames = self.model.password_manager.get_usernames(&self.model.current_url)?;
        if !usernames.is_empty() {
            let username = &usernames[0];
            let password = self.model.password_manager.get_password(&self.model.current_url, username)?;
            let (username, password) = self.decrypt_username_if_needed(username, &password);
            self.server_send(LoadUsernamePass(username.to_string(), password.to_string()));
        }
        Ok(())
    }

    /// Fetch the login data from the web process in order to save them later.
    pub fn save_password(&mut self) {
        self.server_send(GetCredentials());
    }

    /// Save the password from the currently focused login form into the store.
    pub fn save_username_password(&self, username: &str, password: &str) -> Result<()> {
        // TODO: ask to override existing password.
        // TODO: handle errors.
        self.model.password_manager.add(&self.model.current_url, username, password)
        /*Ok(true) => self.app.info("Password added"),
          Ok(false) => self.app.info("A password is already in the store for the current URL"), // TODO: ask for a confirmation to overwrite.
          Err(err) => self.error(err.to_string()),*/
    }

    /// Load the username and password in the login form and submit it.
    pub fn submit_login_form(&mut self) -> Result<()> {
        self.load_password()?;
        self.server_send(SubmitLoginForm());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::thread;
    use std::time::Duration;

    use gtk;
    use libxdo::XDo;
    use tempdir::TempDir;
    use webkit2gtk::WebViewExt;

    use app::App;
    use app::test_utils::XDoExt;

    fn sleep_ms(ms: u64) {
        thread::sleep(Duration::from_millis(ms));
    }

    #[test]
    fn fill_login_form() {
        gtk::init().unwrap();

        let path = "/tmp/titanium_test";
        let temp_dir = TempDir::new(path).unwrap();

        let dir = env::current_dir().unwrap();
        let cwd = dir.to_str().unwrap();
        let url = format!("file://{}/tests/login_form1.html", cwd);
        let js_username_value = "document.getElementById('username').value";
        let js_password_value = "document.getElementById('password').value";

        let app = App::new(Some(url), Some(temp_dir.path().to_str().unwrap().to_string()));

        thread::spawn(|| {
            let xdo = XDo::new(None).unwrap();
            sleep_ms(1000);
            xdo.enter_text("gi", 0).unwrap();
            xdo.enter_text("username", 0).unwrap();
            xdo.send_keysequence("Tab", 0).unwrap();
            xdo.enter_text("password", 0).unwrap();
            xdo.send_keysequence("Escape", 0).unwrap();
            sleep_ms(500);
            xdo.enter_command("password-save");
            xdo.enter_text("gi", 0).unwrap();
            for _ in 0..8 {
                xdo.send_keysequence("BackSpace", 0).unwrap();
            }
            xdo.send_keysequence("Tab", 0).unwrap();
            xdo.send_keysequence("BackSpace", 0).unwrap();
            xdo.send_keysequence("Escape", 0).unwrap();
            sleep_ms(500);
            xdo.enter_command("password-load");
            sleep_ms(500);
            xdo.enter_command("quit");
            xdo.enter_command("quit"); // FIXME: find out why two quit commands are needed.
        });

        gtk::main();

        app.webview.widget().run_javascript_with_callback(&format!("{}.length", js_username_value), |result| {
            let result = result.unwrap();
            let value = result.get_value().unwrap();
            let context = result.get_global_context().unwrap();
            let username_length = value.to_number(&context).unwrap() as i32;
            assert_eq!(username_length, 8);
            gtk::main_quit();
        });

        app.webview.widget().run_javascript_with_callback(&format!("{}.length", js_password_value), |result| {
            let result = result.unwrap();
            let value = result.get_value().unwrap();
            let context = result.get_global_context().unwrap();
            let password_length = value.to_number(&context).unwrap() as i32;
            assert_eq!(password_length, 8);
            gtk::main_quit();
        });

        gtk::main();
        gtk::main();
    }
}
