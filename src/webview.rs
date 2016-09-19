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
use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::rc::Rc;

use gtk::{Inhibit, WidgetExt};
use url::Url;
use webkit2::{self, CookiePersistentStorage, FindController, FindOptions, WebContext, WebViewExt, FIND_OPTIONS_BACKWARDS, FIND_OPTIONS_CASE_INSENSITIVE, FIND_OPTIONS_WRAP_AROUND};
use xdg::BaseDirectories;

use app::APP_NAME;

const SCROLL_LINE_VERTICAL: i32 = 40;

/// Webkit-based view.
pub struct WebView {
    find_controller: FindController,
    scrolled_callback: RefCell<Option<Rc<Box<Fn(u8)>>>>,
    search_backwards: Cell<bool>,
    view: webkit2::WebView,
}

impl WebView {
    /// Create a new web view.
    pub fn new() -> Rc<Self> {
        let context = WebContext::get_default().unwrap();
        context.set_web_extensions_directory("/usr/local/lib/titanium/web-extensions");
        let view = webkit2::WebView::new_with_context(&context);

        let find_controller = view.get_find_controller().unwrap();

        let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
        let cookie_path = xdg_dirs.place_data_file("cookies")
            .expect("cannot create configuration directory");
        let cookie_manager = context.get_cookie_manager().unwrap();
        cookie_manager.set_persistent_storage(cookie_path.to_str().unwrap(), CookiePersistentStorage::Sqlite);

        let webview =
            Rc::new(WebView {
                find_controller: find_controller,
                scrolled_callback: RefCell::new(None),
                search_backwards: Cell::new(false),
                view: view,
            });

        {
            let webview = webview.clone();
            let view = webview.view.clone();
            view.connect_draw(move |_, _| {
                webview.emit_scrolled_event();
                Inhibit(false)
            });
        }

        webview
    }

    /// Clear the selection.
    pub fn clear_selection(&self) {
        self.run_javascript("window.getSelection().empty();");
    }

    /// Connect the scrolled event.
    pub fn connect_scrolled<F: Fn(u8) + 'static>(&self, callback: F) {
        *self.scrolled_callback.borrow_mut() = Some(Rc::new(Box::new(callback)));
    }

    /// Emit the scrolled event.
    pub fn emit_scrolled_event(&self) {
        if let Some(ref callback) = *self.scrolled_callback.borrow() {
            let callback = callback.clone();
            self.run_javascript_with_callback("window.scrollY / (document.body.scrollHeight - window.innerHeight)", move |result| {
                if let Ok(result) = result {
                    let context = result.get_global_context().unwrap();
                    let value = result.get_value().unwrap();
                    if let Some(number) = value.to_number(&context) {
                        callback((number * 100.0).round() as u8);
                    }
                }
            });
        }
    }

    /// Clear the current search.
    pub fn finish_search(&self) {
        self.find_controller.search_finish();
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

    /// Scroll by the specified number of pixels.
    fn scroll(&self, pixels: i32) {
        self.run_javascript(&format!("window.scrollBy(0, {});", pixels));
    }

    /// Scroll to the bottom of the page.
    pub fn scroll_bottom(&self) {
        self.run_javascript("window.scrollTo(0, document.body.clientHeight);");
    }

    /// Scroll down by one line.
    pub fn scroll_down_line(&self) {
        self.scroll(SCROLL_LINE_VERTICAL);
    }

    /// Scroll down by one half of page.
    pub fn scroll_down_half_page(&self) {
        let allocation = self.view.get_allocation();
        self.scroll(allocation.height / 2);
    }

    /// Scroll down by one page.
    pub fn scroll_down_page(&self) {
        let allocation = self.view.get_allocation();
        self.scroll(allocation.height);
    }

    /// Scroll to the top of the page.
    pub fn scroll_top(&self) {
        self.run_javascript("window.scrollTo(0, 0);");
    }

    /// Scroll up by one line.
    pub fn scroll_up_line(&self) {
        self.scroll(-SCROLL_LINE_VERTICAL);
    }

    /// Scroll up by one half of page.
    pub fn scroll_up_half_page(&self) {
        let allocation = self.view.get_allocation();
        self.scroll(-allocation.height / 2);
    }

    /// Scroll up by one page.
    pub fn scroll_up_page(&self) {
        let allocation = self.view.get_allocation();
        self.scroll(-allocation.height);
    }

    /// Search some text.
    pub fn search(&self, input: &str) {
        let default_options = FIND_OPTIONS_CASE_INSENSITIVE | FIND_OPTIONS_WRAP_AROUND;
        let other_options =
            if self.search_backwards.get() {
                FIND_OPTIONS_BACKWARDS
            }
            else {
                FindOptions::empty()
            };
        let options = default_options | other_options;
        self.find_controller.search("", options.bits(), ::std::u32::MAX); // Clear previous search.
        self.find_controller.search(input, options.bits(), ::std::u32::MAX);
    }

    /// Search the next occurence of the search text.
    pub fn search_next(&self) {
        if self.search_backwards.get() {
            self.find_controller.search_previous();
        }
        else {
            self.find_controller.search_next();
        }
    }

    /// Search the previous occurence of the search text.
    pub fn search_previous(&self) {
        if self.search_backwards.get() {
            self.find_controller.search_next();
        }
        else {
            self.find_controller.search_previous();
        }
    }

    /// Set whether the search is backward or not.
    pub fn set_search_backward(&self, backward: bool) {
        self.search_backwards.set(backward);
    }
}

is_widget!(WebView, view);

impl Deref for WebView {
    type Target = webkit2::WebView;

    fn deref(&self) -> &webkit2::WebView {
        &self.view
    }
}
