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

//! Manage the JavaScript dialogs like alert, prompt, confirm and file.

use std::env::{home_dir, temp_dir};
use std::path::Path;
use std::rc::Rc;

use webkit2gtk::ScriptDialog;
use webkit2gtk::ScriptDialogType::{Alert, BeforeUnloadConfirm, Confirm, Prompt};

use dialogs::CustomDialog;
use super::App;

impl App {
    /// Show a non-modal file chooser dialog when the user activates a file input.
    pub fn handle_file_chooser(app: &Rc<App>) {
        // TODO: filter entries with get_mime_types() (strikeout files not matching the mime types).
        let application = app.clone();
        app.webview.connect_run_file_chooser(move |_, file_chooser_request| {
            if file_chooser_request.get_select_multiple() {
                // TODO: support multiple files.
                false
            }
            else {
                let selected_files = file_chooser_request.get_selected_files();
                let file =
                    if selected_files.is_empty() {
                        let dir = home_dir()
                            .unwrap_or_else(temp_dir)
                            .to_str().unwrap()
                            .to_string();
                        format!("{}/", dir)
                    }
                    else {
                        selected_files[0].clone()
                    };
                if let Some(file) = application.app.blocking_file_input("Select file", &file) {
                    let path = Path::new(&file);
                    if !path.exists() {
                        application.app.error("Please select an existing file");
                        file_chooser_request.cancel();
                    }
                    else if path.is_dir() {
                        application.app.error("Please select a file, not a directory");
                        file_chooser_request.cancel();
                    }
                    else {
                        file_chooser_request.select_files(&[&file]);
                    }
                }
                else {
                    file_chooser_request.cancel();
                }
                true
            }
        });
    }

    /// Handle the script dialog event.
    pub fn handle_script_dialog(&self, script_dialog: ScriptDialog) {
        match script_dialog.get_dialog_type() {
            Alert => {
                self.app.message(&format!("[JavaScript] {}", script_dialog.get_message()));
            },
            Confirm => {
                let confirmed = self.app.blocking_yes_no_question(&format!("[JavaScript] {}", script_dialog.get_message()));
                script_dialog.confirm_set_confirmed(confirmed);
            },
            BeforeUnloadConfirm => {
                let confirmed = self.app.blocking_yes_no_question("[JavaScript] Do you really want to leave this page?");
                script_dialog.confirm_set_confirmed(confirmed);
            },
            Prompt => {
                let default_answer = script_dialog.prompt_get_default_text().to_string();
                let input = self.app.blocking_input(&format!("[JavaScript] {}", script_dialog.get_message()), &default_answer);
                let input = input.unwrap_or_default();
                script_dialog.prompt_set_text(&input);
            },
            _ => (),
        }
    }
}
