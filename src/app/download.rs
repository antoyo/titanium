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

use std::fs::{read_dir, remove_file};
use std::io;
use std::path::{Path, PathBuf};

use mg::yes_no_question;
use mg::DialogResult::{self, Answer, Shortcut};
use webkit2gtk::Download;

use app::Msg::OverwriteDownload;
use config_dir::ConfigDir;
use download::download_dir;
use download_list_view::Msg::{AddFileToOpen, DownloadCancel, DownloadDestination};
use file::gen_unique_filename;
use super::{App, AppResult};

impl App {
    fn ask_download_confirm_if_needed(&self, destination: &str, download: Download, suggested_filename: &str) {
        let path = Path::new(&destination);
        let download_path =
            if path.is_dir() {
                path.join(suggested_filename)
            }
            else {
                path.to_path_buf()
            };
        let download_destination = download_path.to_str().unwrap(); // TODO: remove unwrap().
        let exists = download_path.exists() &&
            // Check that it is not the path chosen before (because the download is already started
            // at this point).
            Some(format!("file://{}", download_destination)) != download.get_destination();
        if exists {
            let message = format!("Do you want to overwrite {}?", download_destination);
            let download_destination = download_destination.to_string();
            yes_no_question(&self.mg, &self.model.relm, message,
                move |answer| OverwriteDownload(download.clone(), download_destination.clone(), answer));
        }
        else {
            self.set_download_destination(download, download_destination);
        }
    }

    pub fn clean_download_folder(&self) -> AppResult<()> {
        let download_dir = self.model.config_dir.data_file("downloads")?;
        // TODO: remove the file when the processus dies
        // What to do if the process dies after?
        for file in read_dir(download_dir)? {
            remove_file(file?.path())?;
        }
        Ok(())
    }

    /// Handle the download decide destination event.
    pub fn download_destination_chosen(&mut self, destination: DialogResult, download: Download,
        suggested_filename: String)
    {
        match destination {
            Answer(Some(destination)) => {
                self.ask_download_confirm_if_needed(&destination, download, &suggested_filename);
            },
            Shortcut(shortcut) => {
                if shortcut == "download" {
                    let download_destination = gen_unique_filename(&suggested_filename);
                    let temp_file = temp_dir(&self.model.config_dir, &download_destination)
                        .expect("temp file for download"); // TODO: remove expect().
                    let temp_file = temp_file.to_str().expect("valid utf-8 string"); // TODO: remove expect().
                    let destination = format!("file://{}", temp_file);
                    self.download_list_view.emit(AddFileToOpen(download.clone()));
                    // DownloadDestination must be emitted after AddFileToOpen because this event
                    // will open the file in case the download is already finished.
                    self.download_list_view.emit(DownloadDestination(download, destination));
                }
            },
            Answer(None) => {
                self.download_list_view.emit(DownloadCancel(download));
            },
        }
    }

    pub fn overwrite_download(&self, download: Download, download_destination: String, overwrite: bool) {
        if overwrite {
            self.set_download_destination(download, &download_destination);
        }
        else {
            self.download_list_view.emit(DownloadCancel(download));
        }
    }

    fn set_download_destination(&self, download: Download, download_destination: &str) {
        let destination = format!("file://{}", download_destination);
        self.download_list_view.emit(DownloadDestination(download, destination));
    }
}

pub fn find_download_destination(suggested_filename: &str) -> String {
    fn next_path(counter: i32, dir: &str, path: &Path) -> PathBuf {
        let filename = path.file_stem().unwrap_or_default().to_str()
            .expect("valid utf-8 string"); // TODO: remove expect().
        let extension = path.extension().unwrap_or_default().to_str()
            .expect("valid utf-8 string"); // TODO: remove expect().
        Path::new(&format!("{}{}{}.{}", dir, filename, counter, extension))
            .to_path_buf()
    }

    let dir = download_dir();
    let path = format!("{}{}", dir, suggested_filename);
    if !Path::new(&path).exists() {
        return path;
    }

    let mut counter = 1;
    let default_path = Path::new(suggested_filename);
    let mut path = next_path(counter, &dir, &default_path);
    while path.exists() {
        counter += 1;
        path = next_path(counter, &dir, &default_path);
    }
    // TODO: remove call to expect().
    path.to_str().expect("valid utf-8 string")
        .to_string()
}

fn temp_dir(config_dir: &ConfigDir, filename: &str) -> Result<PathBuf, io::Error> {
    config_dir.data_file(&format!("downloads/{}", filename))
}
