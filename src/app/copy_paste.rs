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

//! Handle copy/paste of URLs withing the application.

use gtk::{Clipboard, WidgetExt};
use mg::Application;
use mg::message::MessageWindow;

use super::App;

impl App {
    /// Copy the URL in the system clipboard.
    pub fn copy_url(&self) {
        let clipboard = self.webview.get_display()
            .and_then(|display| Clipboard::get_default(&display));
        if let Some(clipboard) = clipboard {
            if let Some(url) = self.webview.get_uri() {
                clipboard.set_text(&url);
                Application::info(&self.app, &format!("Copied URL to clipboard: {}", url));
            }
            else {
                self.app.error("No URL to copy");
            }
        }
        else {
            self.app.error("Cannot get the system clipboard");
        }
    }

    /// Open the url from the system clipboard.
    pub fn paste_url(&self) {
        if let Some(url) = self.get_url_from_clipboard() {
            self.open(&url);
        }
    }
}
