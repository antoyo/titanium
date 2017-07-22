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

//! Manage the JavaScript dialogs like alert, prompt, confirm and file.

use std::env::{home_dir, temp_dir};
use std::path::Path;

use mg::{
    self,
    CustomDialog,
    DialogBuilder,
    DialogResult,
    InputDialog,
    Mg,
    Responder,
    blocking_dialog,
    blocking_input,
    blocking_yes_no_question,
};
use mg_settings::{
    self,
    EnumFromStr,
    EnumMetaData,
    SettingCompletion,
    SpecialCommand,
};
use mg_settings::key::Key::{Char, Control};
use relm::{EventStream, Relm, Update, Widget};
use webkit2gtk::{Download, ScriptDialog};
use webkit2gtk::ScriptDialogType::{Alert, BeforeUnloadConfirm, Confirm, Prompt};

use app::Msg::{DownloadDestination, FileDialogSelection};
use file::download_dir;
use self::FileInputError::*;
use super::App;

const SELECT_FILE: &str = "Select file";

/// Input dialog responder for file selection for download.
pub struct DownloadInputDialog<WIDGET: Widget> {
    callback: fn(DialogResult, Download, String) -> WIDGET::Msg,
    download: Download,
    stream: EventStream<WIDGET::Msg>,
    suggested_filename: String,
}

impl<WIDGET: Widget> DownloadInputDialog<WIDGET> {
    fn new(relm: &Relm<WIDGET>, callback: fn(DialogResult, Download, String) -> WIDGET::Msg, download: Download,
        suggested_filename: String) -> Self
    {
        DownloadInputDialog {
            callback,
            download,
            stream: relm.stream().clone(),
            suggested_filename,
        }
    }
}

impl<WIDGET: Widget> Responder for DownloadInputDialog<WIDGET> {
    fn respond(&self, answer: DialogResult) {
        self.stream.emit((self.callback)(answer, self.download.clone(), self.suggested_filename.clone()));
    }
}

pub enum FileInputError {
    Cancelled,
    FileDoesNotExist,
    SelectedDirectory,
}

impl App {
    /// Show a blocking iniput dialog with file completion for download destination selection.
    /// It contains the C-x shortcut to open the file instead of downloading it.
    pub fn download_input(&self, download: Download, suggested_filename: String) {
        let default_path = download_dir();
        let responder = Box::new(DownloadInputDialog::new(&self.model.relm, DownloadDestination, download,
            suggested_filename));
        let builder = DialogBuilder::new()
            .completer("file")
            .default_answer(default_path)
            .message("Save file to: (<C-x> to open)".to_string())
            .responder(responder)
            .shortcut(Control(Box::new(Char('x'))), "download");
        self.mg.emit(CustomDialog(builder)); // TODO: without shortcuts.
    }

    /// Show a input dialog with file completion.
    fn file_input(&self, responder: Box<Responder>, message: String, default_answer: String) {
        let builder = DialogBuilder::new()
            .completer("file")
            .default_answer(default_answer)
            .message(message)
            .responder(responder);
        self.mg.emit(CustomDialog(builder)); // TODO: without shortcuts.
    }

    /// Show a file input dialog.
    pub fn show_file_input(&self) {
        // TODO: take another parameter for the default file name.
        let responder = Box::new(InputDialog::new(&self.model.relm, FileDialogSelection));
        self.file_input(responder, SELECT_FILE.to_string(), default_directory());
    }
}

/// Show a blocking input dialog with file completion.
fn blocking_file_input<COMM, SETT>(stream: &EventStream<<Mg<COMM, SETT> as Update>::Msg>, message: String, default_answer: String)
    -> Option<String>
where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
      SETT: Default + EnumMetaData + mg_settings::settings::Settings + SettingCompletion + 'static,
{
    let builder = DialogBuilder::new()
        .completer("file")
        .default_answer(default_answer)
        .message(message);
    blocking_dialog(stream, builder)
}

/// Get the default directory to show for a file input dialog.
fn default_directory() -> String {
    let dir = home_dir()
        .unwrap_or_else(temp_dir)
        .to_str()
        .map(ToString::to_string)
        .unwrap_or_default();
    format!("{}/", dir)
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

/// Show a blocking file input dialog.
pub fn show_blocking_file_input<COMM, SETT>(stream: &EventStream<<Mg<COMM, SETT> as Update>::Msg>,
    selected_files: &[String])
    -> Result<String, FileInputError>
where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
      SETT: Default + EnumMetaData + mg_settings::settings::Settings + SettingCompletion + 'static,
{
    let file =
        if selected_files.is_empty() {
            default_directory()
        }
        else {
            selected_files[0].clone()
        };
    if let Some(file) = blocking_file_input(stream, SELECT_FILE.to_string(), file) {
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
