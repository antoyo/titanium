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

use mg::{
    self,
    DialogBuilder,
    DialogResult,
    Mg,
    blocking_input,
    blocking_yes_no_question,
};
use mg_settings::{
    self,
    EnumFromStr,
    EnumMetaData,
    Parser,
    SettingCompletion,
    SpecialCommand,
};
use mg_settings::key::Key::{Char, Control};
use relm::{EventStream, Resolver, Update};
use webkit2gtk::{FileChooserRequest, ScriptDialog};
use webkit2gtk::ScriptDialogType::{Alert, BeforeUnloadConfirm, Confirm, Prompt};

use self::FileInputError::*;
use super::App;

pub enum FileInputError {
    Cancelled,
    FileDoesNotExist,
    SelectedDirectory,
}

impl App {
    /// Show a file input dialog.
    pub fn file_input(&self, selected_files: Vec<String>) -> Result<String, FileInputError> {
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
        if let Some(file) = self.blocking_file_input("Select file", &file) {
            {
                let path = Path::new(&file);
                if !path.exists() {
                    return Err(FileDoesNotExist)
                }
                else if path.is_dir() {
                    return Err(SelectedDirectory)
                }
            }
            Ok(file)
        }
        else {
            Err(Cancelled)
        }
    }

    /// Show a non-modal file chooser dialog when the user activates a file input.
    pub fn handle_file_chooser(&self, file_chooser_request: FileChooserRequest, mut resolver: Resolver<bool>) {
        /*
        // TODO: filter entries with get_mime_types() (strikeout files not matching the mime types).
        if file_chooser_request.get_select_multiple() {
            // TODO: support multiple files (use a boolean column that is converted to a pixmap).
            false
        }
        else {
            let selected_files = file_chooser_request.get_selected_files();
            match self.file_input(selected_files) {
                Ok(file) => file_chooser_request.select_files(&[&file]),
                Err(Cancelled) => file_chooser_request.cancel(),
                Err(FileDoesNotExist) => {
                    self.error("Please select an existing file");
                    file_chooser_request.cancel();
                },
                Err(SelectedDirectory) => {
                    self.error("Please select a file, not a directory");
                    file_chooser_request.cancel();
                },
            }
            true
        }*/
    }

    /// Show a blocking iniput dialog with file completion for download destination selection.
    /// It contains the C-x shortcut to open the file instead of downloading it.
    pub fn blocking_download_input(&self, message: &str, default_answer: &str) -> DialogResult {
        // TODO
        /*let builder = DialogBuilder::new()
            .blocking(true)
            .completer("file")
            .default_answer(default_answer)
            .message(message)
            .shortcut(Control(Box::new(Char('x'))), "download");
        let res = self.mg.widget_mut().show_dialog(&self.model.relm, builder);
        res*/
        DialogResult::Answer(None)
    }

    /// Show a blocking input dialog with file completion.
    fn blocking_file_input(&self, message: &str, default_answer: &str) -> Option<String> {
        // TODO
        /*let builder = DialogBuilder::new()
            .blocking(true)
            .completer("file")
            .default_answer(default_answer)
            .message(message);
        self.mg.widget_mut().show_dialog_without_shortcuts(&self.model.relm, builder)*/
        None
    }

    pub fn blocking_input(&self, message: &str, default_answer: &str) -> Option<String> {
        // TODO
        //self.mg.widget_mut().blocking_input(&self.model.relm, message, default_answer)
        None
    }

    pub fn blocking_yes_no_question(&self, message: &str) -> bool {
        // TODO
        //self.mg.widget_mut().blocking_yes_no_question(&self.model.relm, message)
        false
    }
}

/// Handle the script dialog event.
pub fn handle_script_dialog<COMM, SETT>(script_dialog: &ScriptDialog,
    mg: &EventStream<<Mg<COMM, SETT> as Update>::Msg>) -> bool
where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
      SETT: Default + mg_settings::settings::Settings + EnumMetaData + SettingCompletion + 'static,
{
    match script_dialog.get_dialog_type() {
        Alert => {
            mg.emit(mg::Alert(format!("[JavaScript] {}", script_dialog.get_message())));
        }
        Confirm => {
            let confirmed = blocking_yes_no_question(mg, format!("[JavaScript] {}", script_dialog.get_message()));
            script_dialog.confirm_set_confirmed(confirmed);
        },
        BeforeUnloadConfirm => {
            // TODO: when typing 'q', this freeze the browser.
            // FIXME: should inhibit the letter typed.
            let confirmed = blocking_yes_no_question(mg,
                "[JavaScript] Do you really want to leave this page?".to_string());
            script_dialog.confirm_set_confirmed(confirmed);
        },
        Prompt => {
            let default_answer = script_dialog.prompt_get_default_text().to_string();
            let input = blocking_input(mg, format!("[JavaScript] {}", script_dialog.get_message()), default_answer);
            let input = input.unwrap_or_default();
            script_dialog.prompt_set_text(&input);
        },
        _ => (),
    }
    true
}
