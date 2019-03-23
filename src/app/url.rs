/*
 * Copyright (c) 2017-2018 Boucher, Antoni <bouanto@zoho.com>
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

use webkit2gtk::WebViewExt;

use titanium_common::FollowMode;

use app::App;
use app::Msg::CreateWindow;
use message_server::Privacy;
use url::{Position, Url};
use urls::offset;
use webview::Msg::PageOpen;

impl App {
    /// Open the given URL in the web view.
    pub fn open(&self, url: &str) {
        let url = self.transform_url(url);
        self.webview.emit(PageOpen(url));
    }

    /// Open the given URL in a new window.
    pub fn open_in_new_window(&mut self, url: &str, privacy: Privacy) {
        let privacy = if self.webview.widget().is_ephemeral() {
            Privacy::Private
        } else {
            privacy
        };
        let url = self.transform_url(url);
        self.model.relm.stream().emit(CreateWindow(url, privacy));
    }

    /// Open in a new window the url from the system clipboard.
    pub fn win_paste_url(&mut self) {
        if let Some(url) = self.get_url_from_clipboard() {
            self.open_in_new_window(&url, Privacy::Normal);
        }
    }

    /// Go up one directory in url.
    pub fn go_parent_directory(&self, parent_level: Option<u32>) {
        let parent_level = parent_level.unwrap_or(1);
        if let Some(ref url) = self.webview.widget().get_uri() {
            if let Ok(mut url) = Url::parse(url) {
                match url.path_segments_mut() {
                    Ok(mut segments) => {
                        for _ in 0..parent_level {
                            segments.pop_if_empty().pop();
                        }
                    }
                    Err(_) => return,
                }
                self.open(url.as_str());
            }
        }
    }

    /// Go to the root directory or url hostname.
    pub fn go_root_directory(&self) {
        if let Some(ref url) = self.webview.widget().get_uri() {
            if let Ok(base_url) = Url::parse(url) {
                let root = &base_url[..Position::BeforePath];

                if !root.is_empty() {
                    self.open(root);
                }
            }
        }
    }

    /// Enter follow mode to save the source of an URL from a link.
    pub fn save_link(&mut self) {
        self.model.follow_mode = FollowMode::Download;
        self.set_mode("follow");
        self.follow_link();
    }

    pub fn url_increment(&self) {
        if let Some(ref url) = self.webview.widget().get_uri() {
            if let Some(url) = offset(url, 1) {
                self.open(&url);
            }
        }
    }

    pub fn url_decrement(&self) {
        if let Some(ref url) = self.webview.widget().get_uri() {
            if let Some(url) = offset(url, -1) {
                self.open(&url);
            }
        }
    }
}
