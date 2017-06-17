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

//! Manage downloads withing the application.

use std::env::temp_dir;
use std::path::Path;

use mg::DialogResult::{Answer, Shortcut};
use webkit2gtk::Download;

use download::download_dir;
use download_list_view::Msg::AddFileToOpen;
use file::gen_unique_filename;
use super::App;

impl App {
    /// Handle the download decide destination event.
    pub fn handle_decide_destination(&mut self, download: &Download, suggested_filename: &str) -> bool {
        println!("handle_decide_destination");
        let default_path = download_dir();
        let destination = self.blocking_download_input("Save file to: (<C-x> to open)", &default_path);
        match destination {
            Answer(Some(destination)) => {
                let path = Path::new(&destination);
                let download_destination =
                    if path.is_dir() {
                        path.join(suggested_filename)
                    }
                    else {
                        path.to_path_buf()
                    };
                let exists = download_destination.exists();
                let download_destination = download_destination.to_str().unwrap();
                if exists {
                    let message = &format!("Do you want to overwrite {}?", download_destination);
                    let answer = self.blocking_yes_no_question(message);
                    if answer {
                        download.set_allow_overwrite(true);
                    }
                    else {
                        download.cancel();
                    }
                }
                download.set_destination(&format!("file://{}", download_destination));
            },
            Shortcut(shortcut) => {
                if shortcut == "download" {
                    let temp_dir = temp_dir();
                    let download_destination = gen_unique_filename(suggested_filename);
                    let destination = format!("file://{}/{}", temp_dir.to_str().unwrap(), download_destination);
                    download.set_destination(&destination);
                    self.download_list_view.emit(AddFileToOpen(destination));
                }
            },
            _ => download.cancel(),
        }
        true
    }
}
