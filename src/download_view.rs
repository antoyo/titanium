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

use std::ops::Deref;
use std::time::SystemTime;

use gtk::ProgressBar;
use number_prefix::{Prefixed, Standalone, binary_prefix};
use webkit2gtk::Download;

use urls::get_filename;

/// Download view.
pub struct DownloadView {
    pub id: u32,
    last_update: SystemTime,
    view: ProgressBar,
    was_shown: bool,
}

impl DownloadView {
    /// Create a download view.
    pub fn new(download: &Download, id: u32) -> Box<Self> {
        let progress_bar = ProgressBar::new();
        progress_bar.set_show_text(true);

        let mut download_view =
            Box::new(DownloadView {
                id: id,
                last_update: SystemTime::now(),
                view: progress_bar,
                was_shown: false,
            });

        download_view.add_events(download);

        download_view
    }

    /// Add the events.
    fn add_events(&mut self, download: &Download) {
        // TODO: show errors.
        // TODO: add a command to download the current page.
        // TODO: add commands to cancel, delete (on disk), open, retry, remove from list, clear all
        // the list
        self.update_progress_bar(download);

        //connect!(download, connect_received_data(download, _), self, update_progress_bar(download));
        //connect!(download, connect_finished(download), self, handle_finished(download));
    }

    /// Get the destination filename of the download.
    /// Return the suggested filename if it does not exist.
    fn get_filename(&self, download: &Download) -> String {
        let suggested_filename =
            download.get_request()
                .and_then(|request| request.get_uri())
                .and_then(|url| get_filename(&url));
        download.get_destination()
            .and_then(|url| get_filename(&url))
            .unwrap_or(suggested_filename.clone().unwrap_or_default())
    }

    /// Show the data of a finished download.
    fn handle_finished(&self, download: &Download) {
        let filename = self.get_filename(download);
        let percent = 100;
        self.view.set_fraction(1.0);
        let (_, total_size) = get_data_sizes(download);
        let total_size = total_size.map(|size| format!(" [{}]", size)).unwrap_or_default();
        // TODO: switch back to &format!() when it compiles on stable.
        let text = format!("{} {}%{}", filename, percent, total_size);
        self.view.set_text(Some(text.as_ref()));
    }

    /// Update the progress and the text of the progress bar.
    fn update_progress_bar(&mut self, download: &Download) {
        let filename = self.get_filename(download);
        let progress = download.get_estimated_progress();
        self.view.set_fraction(progress);
        let percent = (progress * 100.0) as i32;
        let (downloaded_size, total_size) = get_data_sizes(download);
        // TODO: show the speed (downloaded data over the last 5 seconds).
        let mut updated = false;
        if percent == 100 {
            let total_size = total_size.map(|size| format!(" [{}]", size)).unwrap_or_default();
            // TODO: switch back to &format!() when it compiles on stable.
            let text = format!("{} {}%{}", filename, percent, total_size);
            self.view.set_text(Some(text.as_ref()));
        }
        else if let Ok(duration) = self.last_update.elapsed() {
            // Update the text once per second.
            if duration.as_secs() >= 1 || !self.was_shown {
                updated = true;
                let time_remaining = get_remaining_time(download)
                    .map(|time| format!(", {}", time))
                    .unwrap_or_default();
                let total_size = total_size.map(|size| format!("/{}", size)).unwrap_or_default();
                // TODO: switch back to &format!() when it compiles on stable.
                let text = format!("{} {}%{} [{}{}]", filename, percent, time_remaining, downloaded_size, total_size);
                self.view.set_text(Some(text.as_ref()));
                self.was_shown = true;
            }
        }
        if updated {
            self.last_update = SystemTime::now();
        }
    }
}

impl Deref for DownloadView {
    type Target = ProgressBar;

    fn deref(&self) -> &ProgressBar {
        &self.view
    }
}

/// Add the byte suffix with the right prefix.
/// For instance, convert 10 to "10B" and 5234 to "5.2KiB".
fn add_byte_suffix(number: f64) -> String {
    match binary_prefix(number) {
        Prefixed(suffix, number) => format!("{:.1}{}B", number, suffix),
        Standalone(bytes) => format!("{}B", bytes),
    }
}

/// Get the sizes bytes received and total bytes.
fn get_data_sizes(download: &Download) -> (String, Option<String>) {
    let progress = download.get_estimated_progress();
    if progress == 0.0 {
        (add_byte_suffix(progress), None)
    }
    else {
        let current = download.get_received_data_length() as f64;
        let total = current / progress;
        (add_byte_suffix(current), Some(add_byte_suffix(total)))
    }
}

/// Get the estimated remaining time.
fn get_remaining_time(download: &Download) -> Option<String> {
    let progress = download.get_estimated_progress();
    if progress == 0.0 {
        None
    }
    else {
        let elapsed_seconds = download.get_elapsed_time();
        let total_seconds = elapsed_seconds / progress;
        let seconds = total_seconds - elapsed_seconds;
        let minutes = (seconds / 60.0) as i32;
        let seconds = (seconds % 60.0) as i32;
        Some(format!("{}:{:02}", minutes, seconds))
    }
}
