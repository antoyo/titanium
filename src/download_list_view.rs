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

use std::cell::RefCell;
use std::rc::Rc;

use gtk::{ContainerExt, FlowBox, SelectionMode, WidgetExt};
use webkit2gtk::Download;

use download_view::DownloadView;
use file::open;

type DecideDestinationCallback = Fn(&Download, &str) -> bool;

/// A download view.
pub struct DownloadListView {
    decide_destination_callback: Rc<RefCell<Option<Box<DecideDestinationCallback>>>>,
    downloads: Vec<Download>,
    downloads_to_open: Rc<RefCell<Vec<String>>>,
    view: FlowBox,
}

impl DownloadListView {
    /// Create a new download manager.
    pub fn new() -> Self {
        let flow_box = FlowBox::new();
        flow_box.set_selection_mode(SelectionMode::None);
        flow_box.show();

        DownloadListView {
            decide_destination_callback: Rc::new(RefCell::new(None)),
            downloads: vec![],
            downloads_to_open: Rc::new(RefCell::new(vec![])),
            view: flow_box,
        }
    }

    /// Add a new download.
    pub fn add(&mut self, download: &Download) {
        self.downloads.push(download.clone());

        {
            let callback = self.decide_destination_callback.clone();
            // TODO: instead of requiring to call connect_decide_destination, add the callback as a
            // parameter to the constructor.
            download.connect_decide_destination(move |download, suggested_filename| {
                if let Some(ref callback) = *callback.borrow() {
                    callback(download, suggested_filename)
                }
                else {
                    false
                }
            });
        }

        {
            let downloads_to_open = self.downloads_to_open.clone();
            download.connect_finished(move |download| {
                if let Some(destination) = download.get_destination() {
                    let downloads = &mut *downloads_to_open.borrow_mut();
                    let index = downloads.iter().position(|download_destination| *download_destination == destination);
                    if let Some(index) = index {
                        downloads.remove(index);
                        open(destination);
                    }
                }
            });
        }

        let download_view = DownloadView::new(download);

        self.view.add(&*download_view);
        if let Some(flow_child) = self.view.get_children().last() {
            flow_child.set_can_focus(false);
        }
        download_view.show();
    }

    /// Add a file to be opened when its download finish.
    pub fn add_file_to_open(&self, path: &str) {
        let downloads_to_open = &mut *self.downloads_to_open.borrow_mut();
        downloads_to_open.push(path.to_string());
    }

    /// Add a callback for the decide destination event.
    pub fn connect_decide_destination<F: Fn(&Download, &str) -> bool + 'static>(&mut self, callback: F) {
        *self.decide_destination_callback.borrow_mut() = Some(Box::new(callback));
    }
}

is_widget!(DownloadListView, view);
