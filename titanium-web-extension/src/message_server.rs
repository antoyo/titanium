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

use std::sync::Arc;
use std::sync::atomic::AtomicIsize;
use std::sync::atomic::Ordering::Relaxed;

use dbus;
use webkit2gtk_webextension::WebExtension;

use scroll::Scrollable;

dbus_class!("com.titanium.client", class MessageServer (page_id: Arc<AtomicIsize>, extension: WebExtension) {
    fn get_scroll_percentage(&this) -> i64 {
        if let Some(page) = this.extension.get_page(this.page_id.load(Relaxed) as u64) {
            page.scroll_percentage()
        }
        else {
            0
        }
    }

    fn scroll_bottom(&this) {
        if let Some(page) = this.extension.get_page(this.page_id.load(Relaxed) as u64) {
            page.scroll_bottom();
        }
    }

    fn scroll_by(&this, pixels: i64) {
        if let Some(page) = this.extension.get_page(this.page_id.load(Relaxed) as u64) {
            page.scroll_by(pixels);
        }
    }

    fn scroll_top(&this) {
        if let Some(page) = this.extension.get_page(this.page_id.load(Relaxed) as u64) {
            page.scroll_top();
        }
    }
});
