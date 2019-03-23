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

//! Application clipboard utility functions.

use gdk::Display;
use gtk::{Clipboard, ClipboardExt};
use url::Url;

use app::App;

impl App {
    /// Get the URL from the clipboard if there is one.
    /// If there are no URLs in the clipboard, this will show errors.
    pub fn get_url_from_clipboard(&self) -> Option<String> {
        let clipboard = Display::get_default().and_then(|display| Clipboard::get_default(&display));
        if let Some(clipboard) = clipboard {
            let mut urls = clipboard.wait_for_uris();
            let url = urls.pop().or_else(|| {
                let text = clipboard.wait_for_text();
                text.and_then(|text| Url::parse(&text).ok().map(|_| text))
            });
            if let Some(url) = url {
                return Some(url);
            } else {
                self.error("No URLs in the clipboard");
            }
        } else {
            self.error("Cannot get the system clipboard");
        }
        None
    }
}
