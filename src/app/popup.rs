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

//! Popup management in the application.

use mg::{Warning, question};

use urls::get_base_url;
use app::App;
use app::Msg::PopupDecision;

impl App {
    /// Ask to the user whether to open the popup or not (with option to whitelist or blacklist).
    pub fn ask_open_popup(&self, url: String, base_url: String) {
        question(&self.mg, &self.model.relm, format!("A popup from {} was blocked. Do you want to open it?", base_url),
                char_slice!['y', 'n', 'a', 'e'], move |answer| PopupDecision(answer, url.clone()));
    }

    /// Save the specified url in the popup blacklist.
    pub fn blacklist_popup(&mut self, url: &str) {
        let result =
            if let Some(ref mut popup_manager) = self.model.popup_manager {
                popup_manager.blacklist(url)
            }
            else {
                Ok(())
            };
        self.handle_error(result);
    }

    /// Handle the answer of the ask open popup dialog.
    /// If the answer is a (for always), whitelist the popup and open it.
    /// If the answer is y (for yes), open the popup.
    /// If the answer is e (for never), blacklist the popup.
    /// Otherwise, does not open the popup.
    pub fn handle_answer(&mut self, answer: Option<&str>, url: &str) {
        match answer {
            Some("a") => {
                self.open_in_new_window(url);
                self.whitelist_popup(url);
            },
            Some("y") => self.open_in_new_window(url),
            Some("e") => self.blacklist_popup(url),
            _ => (),
        }
    }

    /// Handle the popup.
    /// If the url is whitelisted, open it.
    /// If the url is blacklisted, block it.
    /// Otherwise, ask to the user whether to open it.
    pub fn handle_popup(&mut self, url: &str) {
        // Block popup.
        if let Some(base_url) = get_base_url(url) {
            let mut open = false;
            if let Some(ref popup_manager) = self.model.popup_manager {
                if !popup_manager.is_whitelisted(url) {
                    if popup_manager.is_blacklisted(url) {
                        self.mg.emit(Warning(format!("Not opening popup from {} since it is blacklisted.", base_url)));
                    }
                    else {
                        self.ask_open_popup(url.to_string(), base_url);
                    }
                }
                else {
                    open = true;
                }
            }
            if open {
                self.open_in_new_window(url);
            }
        }
        else {
            warn!("Not opening the popup {}", url);
        }
    }

    /// Save the specified url in the popup whitelist.
    pub fn whitelist_popup(&mut self, url: &str) {
        let result =
            if let Some(ref mut popup_manager) = self.model.popup_manager {
                popup_manager.whitelist(url)
            }
            else {
                Ok(())
            };
        self.handle_error(result);
    }
}
