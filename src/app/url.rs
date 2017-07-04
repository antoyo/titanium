/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
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

use app::App;
use app::Msg::CreateWindow;
use webview::Msg::PageOpen;

impl App {
    /// Open the given URL in the web view.
    pub fn open(&self, url: &str) {
        let url = self.transform_url(url);
        self.webview.emit(PageOpen(url));
    }

    /// Open the given URL in a new window.
    pub fn open_in_new_window(&mut self, url: &str) {
        let url = self.transform_url(url);
        self.model.relm.stream().emit(CreateWindow(url));
    }

    /// Open in a new window the url from the system clipboard.
    pub fn win_paste_url(&mut self) {
        if let Some(url) = self.get_url_from_clipboard() {
            self.open_in_new_window(&url);
        }
    }
}
