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

use titanium_common::Message;
use titanium_common::Message::*;

use message_server::Msg::*;
use super::App;
use super::Msg::{
    Action,
    ClickElement,
    GoToInsertMode,
    SavePassword,
    Scroll,
};

const SCROLL_LINE_HORIZONTAL: i64 = 40;
const SCROLL_LINE_VERTICAL: i32 = 40;

impl App {
    /// Activate the selected hint.
    pub fn activate_hint(&self) {
        self.focus_webview();
        self.server_send(ActivateHint(self.model.follow_mode.to_string()));
    }

    /// Activate the link in the selection
    pub fn activate_selection(&self) {
        self.server_send(ActivateSelection());
    }

    /// Emit the scrolled event.
    pub fn emit_scrolled_event(&self) {
        self.server_send(GetScrollPercentage());
    }

    /// Send a key to the web process to process with the current hints.
    pub fn enter_hint_key(&self, key_char: char) {
        self.server_send(EnterHintKey(key_char));
    }

    /// Focus the first input element.
    pub fn focus_input(&self) {
        self.focus_webview();
        self.server_send(FocusInput());
    }

    /// Follow a link.
    pub fn follow_link(&self) {
        self.server_send(ShowHints(self.model.hint_chars.clone()));
    }

    /// Hide the hints and return to normal mode.
    pub fn hide_hints(&mut self) {
        self.server_send(HideHints());
        self.go_in_normal_mode();
    }

    pub fn listen_messages(&self) {
        let message_server = &self.model.message_server;
        // TODO: use client_id (first param of MsgRecv).
        connect_stream!(message_server@MsgRecv(_, ref msg), self.model.relm.stream(), match *msg {
            ActivateAction(action) => Some(Action(action)),
            ClickHintElement() => Some(ClickElement),
            Credentials(ref username, ref password) => Some(SavePassword(username.clone(), password.clone())),
            EnterInsertMode() => Some(GoToInsertMode),
            ScrollPercentage(percentage) => Some(Scroll(percentage)),
            _ => {
                warn!("Unexpected message received: {:?}", msg);
                None
            },
        });
    }

    /// Scroll by the specified number of pixels.
    fn scroll(&self, pixels: i32) {
        self.server_send(ScrollBy(pixels as i64));
    }

    /// Scroll to the bottom of the page.
    pub fn scroll_bottom(&self) {
        self.server_send(ScrollBottom());
    }

    /// Scroll down by one line.
    pub fn scroll_down_line(&self) {
        self.scroll(SCROLL_LINE_VERTICAL);
    }

    /// Scroll down by one half of page.
    pub fn scroll_down_half_page(&self) {
        let allocation = self.get_webview_allocation();
        self.scroll(allocation.height / 2);
    }

    /// Scroll down by one page.
    pub fn scroll_down_page(&self) {
        let allocation = self.get_webview_allocation();
        self.scroll(allocation.height - SCROLL_LINE_VERTICAL * 2);
    }

    /// Scroll towards the left of the page.
    pub fn scroll_left(&self) {
        self.server_send(ScrollByX(-SCROLL_LINE_HORIZONTAL));
    }

    /// Scroll towards the right of the page.
    pub fn scroll_right(&self) {
        self.server_send(ScrollByX(SCROLL_LINE_HORIZONTAL));
    }

    /// Scroll to the top of the page.
    pub fn scroll_top(&self) {
        self.server_send(ScrollTop());
    }

    /// Scroll up by one line.
    pub fn scroll_up_line(&self) {
        self.scroll(-SCROLL_LINE_VERTICAL);
    }

    /// Scroll up by one half of page.
    pub fn scroll_up_half_page(&self) {
        let allocation = self.get_webview_allocation();
        self.scroll(-allocation.height / 2);
    }

    /// Scroll up by one page.
    pub fn scroll_up_page(&self) {
        let allocation = self.get_webview_allocation();
        self.scroll(-(allocation.height - SCROLL_LINE_VERTICAL * 2));
    }

    /// Set the value of an input[type="file"].
    pub fn select_file(&self, file: String) {
        self.server_send(SelectFile(file));
    }

    pub fn server_send(&self, message: Message) {
        self.model.message_server.emit(Send(self.model.client, message));
    }
}
