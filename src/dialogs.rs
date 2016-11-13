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

//! Custom dialogs.

use mg::dialog::{DialogBuilder, DialogResult};
use mg_settings::key::Key::{Char, Control};

use app::MgApp;

pub trait CustomDialog {
    /// Show a blocking iniput dialog with file completion for download destination selection.
    /// It contains the C-x shortcut to open the file instead of downloading it.
    fn blocking_download_input(&mut self, message: &str, default_answer: &str) -> DialogResult;

    /// Show a blocking input dialog with file completion.
    fn blocking_file_input(&mut self, message: &str, default_answer: &str) -> Option<String>;
}

impl CustomDialog for MgApp {
    fn blocking_download_input(&mut self, message: &str, default_answer: &str) -> DialogResult {
        let builder = DialogBuilder::new()
            .blocking(true)
            .completer("file")
            .default_answer(default_answer)
            .message(message)
            .shortcut(Control(Box::new(Char('x'))), "download");
        self.show_dialog(builder)
    }

    fn blocking_file_input(&mut self, message: &str, default_answer: &str) -> Option<String> {
        let builder = DialogBuilder::new()
            .blocking(true)
            .completer("file")
            .default_answer(default_answer)
            .message(message);
        self.show_dialog_without_shortcuts(builder)
    }
}
