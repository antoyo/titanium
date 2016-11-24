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

use glib::Cast;
use gtk::{Container, ContainerExt, Continue, FlowBox, IsA, SelectionMode, Widget, WidgetExt};
use webkit2gtk::{Download, Error};
use webkit2gtk::DownloadError::CancelledByUser;

use download_view::DownloadView;
use file::open;

const DOWNLOAD_TIME_BEFORE_HIDE: u32 = 2000;

type DecideDestinationCallback = Fn(&Download, &str) -> bool;

/// A download view.
pub struct DownloadListView {
    current_id: u32,
    decide_destination_callback: Option<Box<DecideDestinationCallback>>,
    download_count: u32,
    download_views: Vec<Box<DownloadView>>,
    downloads_to_open: Vec<String>,
    view: FlowBox,
}

impl DownloadListView {
    /// Create a new download manager.
    pub fn new() -> Self {
        let flow_box = FlowBox::new();
        flow_box.set_selection_mode(SelectionMode::None);
        flow_box.show();

        DownloadListView {
            current_id: 0,
            decide_destination_callback: None,
            download_count: 0,
            download_views: vec![],
            downloads_to_open: vec![],
            view: flow_box,
        }
    }

    /// Add a new download.
    pub fn add(&mut self, download: &Download) {
        self.current_id += 1;
        self.download_count += 1;

        let download_view = DownloadView::new(download, self.current_id);

        connect!(download, connect_decide_destination(download, suggested_filename),
            self, decide_destination(download, suggested_filename));

        let id = self.current_id;
        connect!(download, connect_failed(_, error), self, handle_failed(error, id));
        connect!(download, connect_finished(download), self, handle_finished(download, id));

        self.view.add(&**download_view);
        if let Some(flow_child) = self.view.get_children().last() {
            flow_child.set_can_focus(false);
        }
        download_view.show();

        // It is necessary to keep the download views because they are connected to events.
        self.download_views.push(download_view);
    }

    /// Add a file to be opened when its download finish.
    pub fn add_file_to_open(&mut self, path: &str) {
        self.downloads_to_open.push(path.to_string());
    }

    /// Add a callback for the decide destination event.
    pub fn connect_decide_destination<F: Fn(&Download, &str) -> bool + 'static>(&mut self, callback: F) {
        self.decide_destination_callback = Some(Box::new(callback));
    }

    /// Handle the decide destination event.
    fn decide_destination(&self, download: &Download, suggested_filename: &str) -> bool {
        // TODO: instead of requiring to call connect_decide_destination, add the callback as a
        // parameter to the constructor.

        if let Some(ref callback) = self.decide_destination_callback {
            callback(download, suggested_filename)
        }
        else {
            false
        }
    }

    /// Delete a view and remove it from its parent.
    fn delete(&mut self, id: u32) -> Continue {
        let index = self.download_views.iter()
            .position(|download_view| download_view.id == id);
        if let Some(index) = index {
            let download_view = self.download_views.remove(index);
            remove_from_flow_box(&**download_view);
        }
        Continue(false)
    }

    /// Handle the download failed event.
    /// Delete the view if the download was cancelled by the user.
    fn handle_failed(&mut self, error: &Error, id: u32) {
        if let Some(error) = error.kind::<::webkit2gtk::DownloadError>() {
            if error == CancelledByUser {
                self.delete(id);
            }
        }
    }

    /// Handle the download fisished event.
    fn handle_finished(&mut self, download: &Download, id: u32) {
        // Open the file if the user chose to.
        self.download_count -= 1;
        if let Some(destination) = download.get_destination() {
            let index = self.downloads_to_open.iter()
                .position(|download_destination| *download_destination == destination);
            if let Some(index) = index {
                self.downloads_to_open.remove(index);
                open(destination);
            }
        }

        // Delete the view after a certain amount of time after the download finishes.
        timeout_add!(DOWNLOAD_TIME_BEFORE_HIDE, self, delete(id));
    }

    /// Check if there are active downloads.
    pub fn has_active_downloads(&self) -> bool {
        self.download_count > 0
    }
}

impl Deref for DownloadListView {
    type Target = FlowBox;

    fn deref(&self) -> &FlowBox {
        &self.view
    }
}

/// Remove the progress bar from its `FlowBox` parent.
fn remove_from_flow_box<W: IsA<Widget> + WidgetExt>(widget: &W) {
    let child: Option<Container> = widget.get_parent()
        .and_then(|parent| parent.downcast().ok());
    // FlowBox children are wrapped inside FlowBoxChild, so we need to destroy this
    // FlowBoxChild (which is the parent of the widget) in order to remove it from
    // the FlowBox.
    if let Some(child) = child {
        child.destroy();
    }
}
