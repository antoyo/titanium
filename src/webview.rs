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
use std::error::Error;
use std::fs::{File, read_dir};
use std::io::Read;
use std::ops::Deref;
use std::rc::Rc;

use glib::ToVariant;
use gtk::{Inhibit, WidgetExt};
use libc::getpid;
use url::Url;
use webkit2gtk::{self, CookiePersistentStorage, FindController, FindOptions, UserContentManager, UserScript, UserStyleSheet, WebContext, WebViewExt, FIND_OPTIONS_BACKWARDS, FIND_OPTIONS_CASE_INSENSITIVE, FIND_OPTIONS_WRAP_AROUND};
use webkit2gtk::UserContentInjectedFrames::AllFrames;
use webkit2gtk::UserScriptInjectionTime::End;
use webkit2gtk::UserStyleLevel::User;
use xdg::BaseDirectories;

use app::{AppResult, APP_NAME};
use message_server::MessageServer;

const SCROLL_LINE_VERTICAL: i32 = 40;

/// Webkit-based view.
pub struct WebView {
    find_controller: FindController,
    message_server: MessageServer,
    scrolled_callback: RefCell<Option<Rc<Box<Fn(i64)>>>>,
    search_backwards: Cell<bool>,
    view: webkit2gtk::WebView,
}

impl WebView {
    /// Create a new web view.
    pub fn new() -> Rc<Self> {
        let context = WebContext::get_default().unwrap();
        //context.set_web_extensions_directory("/usr/local/lib/titanium/web-extensions");
        context.set_web_extensions_directory("titanium-web-extension/target/debug");

        let pid = unsafe { getpid() };
        let bus_name = format!("/com/titanium/process{}", pid);
        let message_server = MessageServer::new("com.titanium.web-extensions", &bus_name);

        context.set_web_extensions_initialization_user_data(&bus_name.to_variant());

        let view = webkit2gtk::WebView::new_with_context_and_user_content_manager(&context, &UserContentManager::new());

        WebView::configure(&view);

        let find_controller = view.get_find_controller().unwrap();

        let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
        let cookie_path = xdg_dirs.place_data_file("cookies")
            .expect("cannot create data directory");
        let cookie_manager = context.get_cookie_manager().unwrap();
        cookie_manager.set_persistent_storage(cookie_path.to_str().unwrap(), CookiePersistentStorage::Sqlite);

        let webview =
            Rc::new(WebView {
                find_controller: find_controller,
                message_server: message_server,
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

    /// Activate the selected hint.
    pub fn activate_hint(&self) -> Result<bool, Box<Error>> {
        self.view.grab_focus();
        self.message_server.activate_hint()
            .map_err(From::from)
    }

    /// Activate the link in the selection
    pub fn activate_selection(&self) -> AppResult {
        self.message_server.activate_selection().ok();
        // FIXME: finish search should be called after activate_selection() returns.
        self.finish_search();
        Ok(())
    }

    /// Add the user scripts.
    pub fn add_scripts(&self) -> AppResult {
        if let Some(content_manager) = self.view.get_user_content_manager() {
            let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
            let script_path = try!(xdg_dirs.place_config_file("scripts"));
            for filename in try!(read_dir(script_path)) {
                let filename = try!(filename);
                let mut file = try!(File::open(filename.path()));
                let mut content = String::new();
                try!(file.read_to_string(&mut content));
                let script = UserScript::new(&content, AllFrames, End, &[], &[]);
                content_manager.add_script(&script);
            }
        }
        Ok(())
    }

    /// Add the user stylesheets.
    pub fn add_stylesheets(&self) -> AppResult {
        if let Some(content_manager) = self.view.get_user_content_manager() {
            let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
            let stylesheets_path = try!(xdg_dirs.place_config_file("stylesheets"));
            for filename in try!(read_dir(stylesheets_path)) {
                let filename = try!(filename);
                let mut file = try!(File::open(filename.path()));
                let mut content = String::new();
                try!(file.read_to_string(&mut content));
                let stylesheet = UserStyleSheet::new(&content, AllFrames, User, &[], &[]);
                content_manager.add_style_sheet(&stylesheet);
            }
        }
        Ok(())
    }

    fn configure(view: &webkit2gtk::WebView) {
        if let Some(settings) = view.get_settings() {
            settings.set_enable_developer_extras(true);
        }
    }

    /// Connect the scrolled event.
    pub fn connect_scrolled<F: Fn(i64) + 'static>(&self, callback: F) {
        *self.scrolled_callback.borrow_mut() = Some(Rc::new(Box::new(callback)));
    }

    /// Emit the scrolled event.
    pub fn emit_scrolled_event(&self) {
        if let Some(ref callback) = *self.scrolled_callback.borrow() {
            let callback = callback.clone();
            if let Ok(scroll_percentage) = self.message_server.get_scroll_percentage() {
                callback(scroll_percentage);
            }
        }
    }

    /// Send a key to the web process to process with the current hints.
    pub fn enter_hint_key(&self, key_char: char) -> Result<bool, Box<Error>> {
        self.message_server.enter_hint_key(key_char)
            .map_err(From::from)
    }

    /// Clear the current search.
    pub fn finish_search(&self) {
        self.search("");
        self.find_controller.search_finish();
    }

    /// Follow a link.
    pub fn follow_link(&self) -> AppResult {
        try!(self.message_server.show_hint_on_links());
        Ok(())
    }

    /// Hide the hints.
    pub fn hide_hints(&self) -> AppResult {
        try!(self.message_server.hide_hints());
        Ok(())
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
    fn scroll(&self, pixels: i32) -> AppResult {
        try!(self.message_server.scroll_by(pixels as i64));
        Ok(())
    }

    /// Scroll to the bottom of the page.
    pub fn scroll_bottom(&self) -> AppResult {
        try!(self.message_server.scroll_bottom());
        Ok(())
    }

    /// Scroll down by one line.
    pub fn scroll_down_line(&self) -> AppResult {
        self.scroll(SCROLL_LINE_VERTICAL)
    }

    /// Scroll down by one half of page.
    pub fn scroll_down_half_page(&self) -> AppResult {
        let allocation = self.view.get_allocation();
        self.scroll(allocation.height / 2)
    }

    /// Scroll down by one page.
    pub fn scroll_down_page(&self) -> AppResult {
        let allocation = self.view.get_allocation();
        self.scroll(allocation.height)
    }

    /// Scroll to the top of the page.
    pub fn scroll_top(&self) -> AppResult {
        try!(self.message_server.scroll_top());
        Ok(())
    }

    /// Scroll up by one line.
    pub fn scroll_up_line(&self) -> AppResult {
        self.scroll(-SCROLL_LINE_VERTICAL)
    }

    /// Scroll up by one half of page.
    pub fn scroll_up_half_page(&self) -> AppResult {
        let allocation = self.view.get_allocation();
        self.scroll(-allocation.height / 2)
    }

    /// Scroll up by one page.
    pub fn scroll_up_page(&self) -> AppResult {
        let allocation = self.view.get_allocation();
        self.scroll(-allocation.height)
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

    /// Show the web inspector.
    pub fn show_inspector(&self) {
        if let Some(inspector) = self.view.get_inspector() {
            inspector.show();
            inspector.detach();
        }
    }
}

is_widget!(WebView, view);

impl Deref for WebView {
    type Target = webkit2gtk::WebView;

    fn deref(&self) -> &webkit2gtk::WebView {
        &self.view
    }
}
