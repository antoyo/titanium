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

use mg::{Error, Mg};
use mg_settings::{
    self,
    EnumFromStr,
    EnumMetaData,
    SettingCompletion,
    SpecialCommand,
};
use relm::{EventStream, Update};
use webkit2gtk::{FileChooserRequest, FileChooserRequestExt};

use app::App;
use app::dialog::show_blocking_file_input;
use app::dialog::FileInputError::{Cancelled, FileDoesNotExist, SelectedDirectory};

impl App {
    pub fn file_dialog_selection(&mut self, file: Option<String>) {
        if let Some(file) = file {
            self.select_file(file);
        }
    }
}

/// Show a non-modal file chooser dialog when the user activates a file input.
pub fn handle_file_chooser<COMM, SETT>(stream: &EventStream<<Mg<COMM, SETT> as Update>::Msg>,
    file_chooser_request: &FileChooserRequest) -> bool
where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
      SETT: Default + EnumMetaData + mg_settings::settings::Settings + SettingCompletion + 'static,
{
    // TODO: filter entries with get_mime_types() (strikeout files not matching the mime types).
    if file_chooser_request.get_select_multiple() {
        // TODO: support multiple files (use a boolean column that is converted to a pixmap).
        // or only show (selected) beside the file name since we don't support new columns.
        false
    }
    else {
        let selected_files = file_chooser_request.get_selected_files();
        match show_blocking_file_input(stream, &selected_files) {
            Ok(file) => file_chooser_request.select_files(&[&file]),
            Err(Cancelled) => file_chooser_request.cancel(),
            Err(FileDoesNotExist) => {
                stream.emit(Error("Please select an existing file".into()));
                file_chooser_request.cancel();
            },
            Err(SelectedDirectory) => {
                stream.emit(Error("Please select a file, not a directory".into()));
                file_chooser_request.cancel();
            },
        }
        true
    }
}
