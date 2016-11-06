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

//! Popup management in the application.

use std::rc::Rc;

use mg::Application;

use urls::get_base_url;
use super::App;

impl App {
    /// Ask to the user whether to open the popup or not (with option to whitelist or blacklist).
    pub fn ask_open_popup(app: &Rc<App>, url: String, base_url: String) {
        let instance = app.clone();
        app.app.question(&format!("A popup from {} was blocked. Do you want to open it?", base_url),
        &['y', 'n', 'a', 'e'], move |answer| {
            match answer {
                Some("a") => {
                    instance.open_in_new_window_handling_error(&url);
                    instance.whitelist_popup(&url);
                },
                Some("y") => instance.open_in_new_window_handling_error(&url),
                Some("e") => instance.blacklist_popup(&url),
                _ => (),
            }
        });
    }

    /// Save the specified url in the popup blacklist.
    pub fn blacklist_popup(&self, url: &str) {
        self.handle_error((*self.popup_manager.borrow_mut()).blacklist(url));
    }

    /// Handle the popup.
    /// If the url is whitelisted, open it.
    /// If the url is blacklisted, block it.
    /// Otherwise, ask to the user whether to open it.
    pub fn handle_popup(app: &Rc<App>, url: String) {
        // Block popup.
        let popup_manager = &*app.popup_manager.borrow();
        let base_url = get_base_url(&url);
        if !popup_manager.is_whitelisted(&url) {
            if popup_manager.is_blacklisted(&url) {
                Application::warning(&app.app, &format!("Not opening popup from {} since it is blacklisted.", base_url));
            }
            else {
                App::ask_open_popup(app, url, base_url);
            }
        }
        else {
            app.open_in_new_window_handling_error(&url);
        }
    }

    /// Save the specified url in the popup whitelist.
    pub fn whitelist_popup(&self, url: &str) {
        self.handle_error((*self.popup_manager.borrow_mut()).whitelist(url));
    }
}
