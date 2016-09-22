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
use glib::Cast;
use webkit2gtk_webextension::{DOMDOMWindowExtManual, DOMDocumentExt, DOMHTMLElement, DOMHTMLElementExt, DOMNodeExt, WebExtension};

use scroll::Scrollable;

macro_rules! get_page {
    ($this:ident) => {
        $this.extension.get_page($this.page_id.load(Relaxed) as u64)
    };
}

dbus_class!("com.titanium.client", class MessageServer (page_id: Arc<AtomicIsize>, extension: WebExtension) {
    fn activate_selection(&this) {
        let result = get_page!(this)
            .and_then(|page| page.get_dom_document())
            .and_then(|document| document.get_default_view())
            .and_then(|window| window.get_selection())
            .and_then(|selection| selection.get_anchor_node())
            .and_then(|anchor_node| anchor_node.get_parent_element())
            .and_then(|parent| parent.downcast::<DOMHTMLElement>().ok());
        if let Some(parent) = result {
            parent.click();
        }
    }

    fn get_scroll_percentage(&this) -> i64 {
        if let Some(page) = get_page!(this) {
            page.scroll_percentage()
        }
        else {
            0
        }
    }

    fn scroll_bottom(&this) {
        if let Some(page) = get_page!(this) {
            page.scroll_bottom();
        }
    }

    fn scroll_by(&this, pixels: i64) {
        if let Some(page) = get_page!(this) {
            page.scroll_by(pixels);
        }
    }

    fn scroll_top(&this) {
        if let Some(page) = get_page!(this) {
            page.scroll_top();
        }
    }
});
