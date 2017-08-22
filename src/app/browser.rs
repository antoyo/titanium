/*
 * Copyright (c) 2016-2017 Boucher, Antoni <bouanto@zoho.com>
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

use mg::Info;
use webkit2gtk::{CookieManagerExt, WebContextExt};

use super::App;

impl App {
    /// Clear the browser cache.
    pub fn clear_cache(&self) {
        if let Some(context) = self.get_webview_context() {
            context.clear_cache();
            self.mg.emit(Info("Cache cleared".to_string()));
        }
    }

    /// Delete all the cookies.
    pub fn delete_all_cookies(&self) {
        let cookie_manager =
            self.get_webview_context()
                .and_then(|context| context.get_cookie_manager());
        if let Some(cookie_manager) = cookie_manager {
            cookie_manager.delete_all_cookies();
            self.mg.emit(Info("All cookies deleted".to_string()));
        }
    }

    /// Delete the cookies for the specified domain.
    pub fn delete_cookies(&self, domain: &str) {
        let cookie_manager =
            self.get_webview_context()
                .and_then(|context| context.get_cookie_manager());
        if let Some(cookie_manager) = cookie_manager {
            cookie_manager.delete_cookies_for_domain(domain);
            self.mg.emit(Info(format!("Cookies deleted for domain {}", domain)));
        }
    }
}
