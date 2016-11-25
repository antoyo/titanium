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

//! Bookmark management in the application.

use super::App;

impl App {
    /// Add the current page to the bookmarks.
    pub fn bookmark(&mut self) {
        if let Some(url) = self.webview.get_uri() {
            let title = self.webview.get_title();
            let message = format!("Added bookmark: {}", url);
            match self.bookmark_manager.add(url, title) {
                Ok(true) => self.app.info(&message),
                Ok(false) => self.app.info("The current page is already in the bookmarks"),
                Err(err) => self.show_error(err),
            }
        }
    }

    /// Delete the current page from the bookmarks.
    pub fn delete_bookmark(&mut self) {
        if let Some(url) = self.webview.get_uri() {
            match self.bookmark_manager.delete(&url) {
                Ok(true) => self.app.info(&format!("Deleted bookmark: {}", url)),
                Ok(false) => self.info_page_not_in_bookmarks(),
                Err(err) => self.show_error(err),
            }
        }
    }

    /// Delete the bookmark selected in completion.
    pub fn delete_selected_bookmark(&mut self) {
        let command = self.app.get_command();
        let mut command = command.split_whitespace();
        match command.next() {
            Some("open") | Some("win-open") =>
                if let Some(url) = command.next() {
                    // Do not show message when deleting a bookmark in completion.
                    self.bookmark_manager.delete(url).ok();
                    self.app.delete_current_completion_item();
                },
            _ => (),
        }
    }

    /// Edit the tags of the current page from the bookmarks.
    pub fn edit_bookmark_tags(&mut self) {
        if let Some(url) = self.webview.get_uri() {
            let tags = self.bookmark_manager.get_tags(&url);
            if let Some(tags) = tags {
                let default_answer = tags.join(", ");
                // TODO: tags completion.
                let input = self.app.blocking_input("Bookmark tags (separated by comma):", &default_answer);
                // Do not edit tags when the user press Escape.
                if let Some(input) = input {
                    let tags: Vec<_> = input.split(',')
                        .map(|tag| tag.trim().to_lowercase())
                        .filter(|tag| !tag.is_empty())
                        .collect();
                    if let Err(err) = self.bookmark_manager.set_tags(&url, tags) {
                        self.show_error(err);
                    }
                }
            }
            else {
                self.info_page_not_in_bookmarks();
            }
        }
    }

    /// Show an information message to tell that the current page is not in the bookmarks.
    fn info_page_not_in_bookmarks(&mut self) {
        self.app.info("The current page is not in the bookmarks");
    }
}
