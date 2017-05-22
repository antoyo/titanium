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

use gtk::{Inhibit, WidgetExt};

use titanium_common::Message::{GetScrollPercentage, ScrollBottom, ScrollBy, ScrollByX, ScrollTop};

use app::AppResult;
use super::WebView;

const SCROLL_LINE_HORIZONTAL: i64 = 40;
const SCROLL_LINE_VERTICAL: i32 = 40;

impl WebView {
    /// Emit the scrolled event.
    pub fn emit_scrolled_event(&self) -> Inhibit {
        let result = self.server_send(GetScrollPercentage());
        Inhibit(false)
    }

    /// Scroll by the specified number of pixels.
    fn scroll(&self, pixels: i32) -> AppResult<()> {
        self.server_send(ScrollBy(pixels as i64))
    }

    /// Scroll to the bottom of the page.
    pub fn scroll_bottom(&self) -> AppResult<()> {
        self.server_send(ScrollBottom())
    }

    /// Scroll down by one line.
    pub fn scroll_down_line(&self) -> AppResult<()> {
        self.scroll(SCROLL_LINE_VERTICAL)
    }

    /// Scroll down by one half of page.
    pub fn scroll_down_half_page(&self) -> AppResult<()> {
        let allocation = self.view.get_allocation();
        self.scroll(allocation.height / 2)
    }

    /// Scroll down by one page.
    pub fn scroll_down_page(&self) -> AppResult<()> {
        let allocation = self.view.get_allocation();
        self.scroll(allocation.height - SCROLL_LINE_VERTICAL * 2)
    }

    /// Scroll towards the left of the page.
    pub fn scroll_left(&self) -> AppResult<()> {
        self.server_send(ScrollByX(-SCROLL_LINE_HORIZONTAL))
    }

    /// Scroll towards the right of the page.
    pub fn scroll_right(&self) -> AppResult<()> {
        self.server_send(ScrollByX(SCROLL_LINE_HORIZONTAL))
    }

    /// Scroll to the top of the page.
    pub fn scroll_top(&self) -> AppResult<()> {
        self.server_send(ScrollTop())
    }

    /// Scroll up by one line.
    pub fn scroll_up_line(&self) -> AppResult<()> {
        self.scroll(-SCROLL_LINE_VERTICAL)
    }

    /// Scroll up by one half of page.
    pub fn scroll_up_half_page(&self) -> AppResult<()> {
        let allocation = self.view.get_allocation();
        self.scroll(-allocation.height / 2)
    }

    /// Scroll up by one page.
    pub fn scroll_up_page(&self) -> AppResult<()> {
        let allocation = self.view.get_allocation();
        self.scroll(-(allocation.height - SCROLL_LINE_VERTICAL * 2))
    }
}
