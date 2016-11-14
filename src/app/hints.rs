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

//! Manage hints within the application.

use std::char;

use gdk::EventKey;
use gtk::Inhibit;

use super::App;

use titanium_common::Action::{self, FileInput, GoInInsertMode, NoAction};

impl App {
    /// In follow mode, send the key to the web process.
    pub fn handle_follow_key_press(&mut self, event_key: &EventKey) -> Inhibit {
        if let Some(key_char) = char::from_u32(event_key.get_keyval()) {
            if key_char.is_alphanumeric() {
                if let Some(key_char) = key_char.to_lowercase().next() {
                    match self.webview.enter_hint_key(key_char) {
                        Ok(should_click) => {
                            if should_click {
                                let result = self.webview.activate_hint(self.follow_mode.to_string());
                                self.hide_hints();
                                if let Some(result) = result.ok().and_then(Action::from_i32) {
                                    match result {
                                        FileInput => {
                                            if let Ok(file) = self.file_input(vec![]) {
                                                handle_error!(self.webview.select_file(&file));
                                            }
                                        },
                                        GoInInsertMode => self.app.set_mode("insert"),
                                        NoAction => (),
                                    }
                                }
                            }
                        },
                        Err(error) => self.show_error(error),
                    }
                }
            }
        }
        Inhibit(true)
    }

    /// Hide the hints and return to normal mode.
    pub fn hide_hints(&mut self) {
        handle_error!(self.webview.hide_hints());
        self.app.set_mode("normal");
    }

    /// Get the hint characters from the settings.
    pub fn hint_chars(&self) -> &str {
        &self.app.settings().hint_chars
    }
}
