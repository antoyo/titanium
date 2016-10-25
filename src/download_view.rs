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

use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::rc::Rc;
use std::time::SystemTime;

use glib::Cast;
use gtk::{Container, Continue, ProgressBar, WidgetExt, timeout_add};
use number_prefix::{Prefixed, Standalone, binary_prefix};
use webkit2gtk::Download;
use webkit2gtk::DownloadError::CancelledByUser;

use urls::get_filename;

const DOWNLOAD_TIME_BEFORE_HIDE: u32 = 2000;

/// Download view.
pub struct DownloadView {
    view: Rc<ProgressBar>,
}

impl DownloadView {
    /// Create a download view.
    pub fn new(download: &Download) -> Self {
        let progress_bar = Rc::new(ProgressBar::new());
        progress_bar.set_show_text(true);

        DownloadView::add_events(download, progress_bar.clone());

        DownloadView {
            view: progress_bar,
        }
    }

    /// Add the events.
    fn add_events(download: &Download, progress_bar: Rc<ProgressBar>) {
        // TODO: show the suggested filename at the start.
        // TODO: show errors.
        // TODO: add a command to download the current page.
        // TODO: add commands to cancel, delete (on disk), open, retry, remove from list, clear all
        // the list
        // FIXME: some downloads do not start (fixed with clear_cache()).
        let last_update = Rc::new(RefCell::new(SystemTime::now()));
        let never_shown = Rc::new(Cell::new(false));
        {
            let progress_bar = progress_bar.clone();
            download.connect_received_data(move |download, _| {
                let filename = download.get_destination()
                    .and_then(|url| get_filename(&url))
                    .unwrap_or_default();
                let progress = download.get_estimated_progress();
                progress_bar.set_fraction(progress);
                let percent = (progress * 100.0) as i32;
                let (downloaded_size, total_size) = get_data_sizes(download);
                // TODO: show the speed (downloaded data over the last 5 seconds).
                let mut updated = false;
                if percent == 100 {
                    progress_bar.set_text(Some(&format!("{} {}% [{}]", filename, percent, total_size)));
                }
                else if let Ok(duration) = (*last_update.borrow()).elapsed() {
                    if duration.as_secs() >= 1 || !never_shown.get() {
                        updated = true;
                        let time_remaining = get_remaining_time(download);
                        progress_bar.set_text(Some(&format!("{} {}%, {} [{}/{}]", filename, percent, time_remaining, downloaded_size, total_size)));
                        never_shown.set(true);
                    }
                }
                if updated {
                    *last_update.borrow_mut() = SystemTime::now();
                }
            });
        }

        {
            let progress_bar = progress_bar.clone();
            download.connect_failed(move |_, error| {
                if let Some(error) = error.kind::<::webkit2gtk::DownloadError>() {
                    if error == CancelledByUser {
                        remove_from_flow_box(&progress_bar);
                    }
                }
            });
        }

        download.connect_finished(move |download| {
            let progress_bar = progress_bar.clone();
            let progress = download.get_estimated_progress();
            let percent = (progress * 100.0) as i32;
            if percent == 100 {
                timeout_add(DOWNLOAD_TIME_BEFORE_HIDE, move || {
                    remove_from_flow_box(&progress_bar);
                    Continue(false)
                });
            }
        });
    }
}

impl Deref for DownloadView {
    type Target = ProgressBar;

    fn deref(&self) -> &ProgressBar {
        &*self.view
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
fn get_data_sizes(download: &Download) -> (String, String) {
    let progress = download.get_estimated_progress();
    let current = download.get_received_data_length() as f64;
    let total = current / progress;
    (add_byte_suffix(current), add_byte_suffix(total))
}

/// Get the estimated remaining time.
fn get_remaining_time(download: &Download) -> String {
    let progress = download.get_estimated_progress();
    let elapsed_seconds = download.get_elapsed_time();
    let total_seconds = elapsed_seconds / progress;
    let seconds = total_seconds - elapsed_seconds;
    let minutes = (seconds / 60.0) as i32;
    let seconds = (seconds % 60.0) as i32;
    format!("{}:{:02}", minutes, seconds)
}

/// Remove the progress bar from its `FlowBox` parent.
fn remove_from_flow_box(progress_bar: &Rc<ProgressBar>) {
    let child: Option<Container> = progress_bar.get_parent()
        .and_then(|parent| parent.downcast().ok());
    // FlowBox children are wrapped inside FlowBoxChild, so we need to destroy this
    // FlowBoxChild (which is the parent of the widget) in order to remove it from
    // the FlowBox.
    if let Some(child) = child {
        child.destroy();
    }
}
