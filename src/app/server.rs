/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
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

use titanium_common::InnerMessage;
use titanium_common::InnerMessage::*;

use super::App;
use super::Msg::ServerSend;

const SCROLL_LINE_HORIZONTAL: i64 = 40;
const SCROLL_LINE_VERTICAL: i32 = 40;

impl App {
    /// Activate the selected hint.
    pub fn activate_hint(&mut self) {
        self.focus_webview();
        let mode = self.model.follow_mode.to_string();
        let ctrl_key = self.model.open_in_new_window;
        self.server_send(ActivateHint(mode, ctrl_key));
    }

    /// Activate the link in the selection
    pub fn activate_selection(&mut self) {
        self.server_send(ActivateSelection());
    }

    /// Send a key to the web process to process with the current hints.
    pub fn enter_hint_key(&mut self, key_char: char) {
        self.server_send(EnterHintKey(key_char));
    }

    /// Focus the first input element.
    pub fn focus_input(&mut self) {
        self.focus_webview();
        self.server_send(FocusInput());
    }

    /// Follow a link.
    pub fn follow_link(&mut self) {
        let chars = self.model.hint_chars.clone();
        self.server_send(ShowHints(chars));
    }

    /// Hide the hints and return to normal mode.
    pub fn hide_hints(&mut self) {
        self.server_send(HideHints());
        self.go_in_normal_mode();
    }

    pub fn message_recv(&mut self, message: InnerMessage) {
        match message {
            ActivateAction(action) => self.activate_action(action),
            ClickHintElement() => self.click_hint_element(),
            Credentials(ref username, ref password) => handle_error!(self.save_username_password(&username, &password)),
            EnterInsertMode() => self.go_in_insert_mode(),
            ScrollPercentage(percentage) => self.show_scroll(percentage),
            _ =>
                // TODO: show the warning in the UI?
                warn!("Unexpected message received: {:?}", message),
        }
    }

    /// Scroll by the specified number of pixels.
    fn scroll(&mut self, pixels: i32) {
        self.server_send(ScrollBy(pixels as i64));
    }

    /// Scroll to the bottom of the page.
    pub fn scroll_bottom(&mut self) {
        self.server_send(ScrollBottom());
    }

    /// Scroll down by one line.
    pub fn scroll_down_line(&mut self) {
        self.scroll(SCROLL_LINE_VERTICAL);
    }

    /// Scroll down by one half of page.
    pub fn scroll_down_half_page(&mut self) {
        let allocation = self.get_webview_allocation();
        self.scroll(allocation.height / 2);
    }

    /// Scroll down by one page.
    pub fn scroll_down_page(&mut self) {
        let allocation = self.get_webview_allocation();
        self.scroll(allocation.height - SCROLL_LINE_VERTICAL * 2);
    }

    /// Scroll towards the left of the page.
    pub fn scroll_left(&mut self) {
        self.server_send(ScrollByX(-SCROLL_LINE_HORIZONTAL));
    }

    /// Scroll towards the right of the page.
    pub fn scroll_right(&mut self) {
        self.server_send(ScrollByX(SCROLL_LINE_HORIZONTAL));
    }

    /// Scroll to the top of the page.
    pub fn scroll_top(&mut self) {
        self.server_send(ScrollTop());
    }

    /// Scroll up by one line.
    pub fn scroll_up_line(&mut self) {
        self.scroll(-SCROLL_LINE_VERTICAL);
    }

    /// Scroll up by one half of page.
    pub fn scroll_up_half_page(&mut self) {
        let allocation = self.get_webview_allocation();
        self.scroll(-allocation.height / 2);
    }

    /// Scroll up by one page.
    pub fn scroll_up_page(&mut self) {
        let allocation = self.get_webview_allocation();
        self.scroll(-(allocation.height - SCROLL_LINE_VERTICAL * 2));
    }

    /// Set the value of an input[type="file"].
    pub fn select_file(&mut self, file: String) {
        self.server_send(SelectFile(file));
    }

    pub fn server_send(&mut self, message: InnerMessage) {
        let page_id = self.webview.widget().get_page_id();
        self.model.relm.stream().emit(ServerSend(page_id, message));
    }
}
