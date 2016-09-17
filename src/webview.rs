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

use std::borrow::Cow;
use std::ops::Deref;

use url::Url;
use webkit2;

/// Webkit-based view.
pub struct WebView {
    view: webkit2::WebView,
}

impl WebView {
    /// Create a new web view.
    pub fn new() -> Self {
        WebView {
            view: webkit2::WebView::new(),
        }
    }

    /// Open the specified URL.
    pub fn open(&self, url: &str) {
        let url: Cow<str> =
            if let Ok(_) = Url::parse(url) {
                url.into()
            }
            else {
                format!("http://{}", url).into()
            };
        self.view.load_uri(&url);
    }
}

is_widget!(WebView, view);

impl Deref for WebView {
    type Target = webkit2::WebView;

    fn deref(&self) -> &webkit2::WebView {
        &self.view
    }
}
