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

use glib::error;

use super::{App, AppResult};
use webview::Msg::{
    DeletePassword,
    LoadUsernamePassword,
    SavePassword,
    SubmitLoginForm,
};

impl App {
    /// Create the password collection in gnome keyring.
    pub fn create_password_keyring(&mut self) {
        //connect_static!(Service, get(service), self, init_service(service));
    }

    /// Delete the password for the current URL.
    pub fn delete_password(&self) {
        self.webview.emit(DeletePassword);
        /*Ok(true) => self.app.info("Password deleted"),
          Ok(false) => self.app.info("No password for the current URL"),
          Err(err) => self.show_error(err),*/
    }

    /*fn init_service(&mut self, service: Result<Service, error::Error>) {
        // TODO: handle errors.
        if let Ok(service) = service {
            self.webview.widget().password_manager.init(service);
        }
    }*/

    /// Load the username and password in the login form.
    /// If multiple credentials exist, ask the user which one to use.
    /// Return true if a login form was filled.
    pub fn load_password(&self) {
        self.webview.emit(LoadUsernamePassword);
        /*Ok(true) => return Ok(true),
          Ok(false) => self.app.info("No password for the current URL"),
          Err(err) => self.show_error(err),*/
    }

    /// Save the password from the currently focused login form into the store.
    pub fn save_password(&self) {
        self.webview.emit(SavePassword);
        /*Ok(true) => self.app.info("Password added"),
          Ok(false) => self.app.info("A password is already in the store for the current URL"), // TODO: ask for a confirmation to overwrite.
          Err(err) => self.show_error(err),*/
    }

    /// Load the username and password in the login form and submit it.
    pub fn submit_login_form(&self) {
        self.load_password();
        // TODO: put the next line in a callback.
        self.webview.emit(SubmitLoginForm);
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
