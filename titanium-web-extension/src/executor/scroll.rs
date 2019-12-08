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

use glib::Cast;
use webkit2gtk_webextension::{
    DOMCSSStyleDeclarationExt,
    DOMDocumentExt,
    DOMDOMWindowExt,
    DOMElement,
    DOMElementExt,
    DOMNodeExt,
    WebPage,
    WebPageExt,
};

use titanium_common::LAST_MARK;
use titanium_common::Percentage::{self, All, Percent};

use dom::{ElementIter, get_body, get_document};
use executor::Executor;

impl Executor {
    /// Initialize the scroll element.
    pub fn init_scroll_element(&mut self) {
        self.model.scroll_element = find_scrollable_element(&self.model.page);
    }

    /// Initialize the scroll element if needed.
    fn init_scroll_element_if_needed(&mut self) {
        if self.model.scroll_element.is_none() {
            // FIXME: if the page is not scrollable, no scrollable element is found.
            self.init_scroll_element();
        }
    }

    /// Reset the scroll element.
    pub fn reset_scroll_element(&mut self) {
        self.model.scroll_element = None;
    }

    /// Scroll the web page vertically by the specified amount of pixels.
    /// A negative value scroll towards to top.
    pub fn scroll_by(&mut self, pixels: i64) {
        let document = wtry_opt_no_ret!(self.model.page.get_dom_document());
        let window = wtry_opt_no_ret!(document.get_default_view());
        window.scroll_by(0.0, pixels as f64);
    }

    /// Scroll the web page horizontally by the specified amount of pixels.
    /// A negative value scroll towards left.
    pub fn scroll_by_x(&mut self, pixels: i64) {
        let document = wtry_opt_no_ret!(self.model.page.get_dom_document());
        let window = wtry_opt_no_ret!(document.get_default_view());
        window.scroll_by(pixels as f64, 0.0);
    }

    /// Get the current vertical scroll position of the web page as a percentage.
    pub fn scroll_percentage(&mut self) -> Percentage {
        info!("scroll_percentage");
        let default = All;
        let document = unwrap_opt_or_ret!(self.model.page.get_dom_document(), default);
        let window = unwrap_opt_or_ret!(document.get_default_view(), default);
        let document = unwrap_opt_or_ret!(get_document(&self.model.page), default);
        let height = window.get_inner_height() as f64;
        let scroll_height = document.get_scroll_height();
        info!("height: {}", height);
        info!("scroll_height: {}", scroll_height);
        if scroll_height <= height as i64 {
            info!("returned {:?}", default);
            default
        }
        else {
            info!("scroll_y: {}", window.get_scroll_y());
            Percent((window.get_scroll_y() as f64 / (scroll_height as f64 - height) * 100.0).round() as i64)
        }
    }

    /// Scroll to the top of the web page.
    pub fn scroll_top(&mut self) {
        self.add_mark(LAST_MARK);
        let document = wtry_opt_no_ret!(self.model.page.get_dom_document());
        let window = wtry_opt_no_ret!(document.get_default_view());
        window.scroll_to(0.0, 0.0);
    }

    /// Scroll to the specified percent of the web page.
    pub fn scroll_to_percent(&mut self, percent: u32) {
        self.add_mark(LAST_MARK);
        let document = wtry_opt_no_ret!(self.model.page.get_dom_document());
        let window = wtry_opt_no_ret!(document.get_default_view());
        let element = wtry_opt_no_ret!(self.model.scroll_element.as_ref());
        let document = wtry_opt_no_ret!(get_document(&self.model.page));
        let height = window.get_inner_height();
        let scroll_height = document.get_scroll_height();
        let scroll_height = (percent as i64) * (scroll_height - height) / 100;
        window.scroll_to(0.0, scroll_height as f64);
    }
}

pub fn find_scrollable_element(page: &WebPage) -> Option<DOMElement> {
    let body = wtry_opt!(get_body(page)).upcast();
    if body_scrollable(&body) {
        Some(body)
    }
    else {
        let children = ElementIter::new(body.get_children());
        let mut max_area = 0;
        let mut best_child = None;
        for child in children {
            if may_scroll(&child) {
                let area = child.get_client_width() as i64 * child.get_client_height() as i64;
                if area > max_area {
                    max_area = area;
                    best_child = Some(child);
                }
            }
        }
        if let Some(child) = best_child.take() {
            if is_scrollable(&child) {
                best_child = Some(child);
            }
            else {
                let children = ElementIter::new(child.get_children());
                for child in children {
                    // TODO: pick the biggest?
                    if is_scrollable(&child) {
                        best_child = Some(child);
                    }
                }
            }
        }
        Some(best_child.unwrap_or(body))
    }
}

fn body_scrollable(element: &DOMElement) -> bool {
    let document = unwrap_opt_or_ret!(element.get_owner_document(), false);
    let document = unwrap_opt_or_ret!(document.get_document_element(), false);
    let height = document.get_client_height() as i64;
    element.get_scroll_height() > height
}

fn is_scrollable(element: &DOMElement) -> bool {
    may_scroll(element) && element.get_scroll_height() > element.get_client_height() as i64
}

fn may_scroll(element: &DOMElement) -> bool {
    let document = unwrap_opt_or_ret!(element.get_owner_document(), false);
    let window = unwrap_opt_or_ret!(document.get_default_view(), false);
    let style = unwrap_opt_or_ret!(window.get_computed_style(element, None), false);
    let overflow_y = style.get_property_value("overflow-y");
    let overflow_y = unwrap_opt_or_ret!(overflow_y, false);
    overflow_y == "scroll" || overflow_y == "auto"
}
