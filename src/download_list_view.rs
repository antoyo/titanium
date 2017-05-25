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

use std::time::Duration;

use futures_glib::Timeout;
use glib::Cast;
use gtk::{self, Container, ContainerExt, FlowBox, IsA, SelectionMode, WidgetExt};
use relm::{Relm, Resolver, Widget};
use relm_attributes::widget;
use webkit2gtk::{Download, Error};
use webkit2gtk::DownloadError::CancelledByUser;

use download_view::DownloadView;
use file::open;
use self::Msg::*;

const DOWNLOAD_TIME_BEFORE_HIDE: u64 = 2000;

type DecideDestinationCallback = Fn(Download, String) -> bool;

pub struct Model {
    current_id: u32,
    download_count: u32,
    download_views: Vec<Box<DownloadView>>,
    downloads_to_open: Vec<String>,
    relm: Relm<DownloadListView>,
}

#[derive(Msg)]
pub enum Msg {
    Add(Download),
    DecideDestination(Resolver<bool>, Download, String),
    DownloadFailed(Error, u32),
    DownloadFinished(Download, u32),
    Remove(u32),
}

#[widget]
impl Widget for DownloadListView {
    fn model(relm: &Relm<Self>, (): ()) -> Model {
        Model {
            current_id: 0,
            download_count: 0,
            download_views: vec![],
            downloads_to_open: vec![],
            relm: relm.clone(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Add(download) => self.add(download),
            DecideDestination(_, _, _) => (), // To be listened by the user.
            DownloadFailed(error, id) => self.handle_failed(&error, id),
            DownloadFinished(download, id) => self.handle_finished(&download, id),
            Remove(id) => self.delete(id),
        }
    }

    view! {
        #[name="view"]
        gtk::FlowBox {
            selection_mode: SelectionMode::None,
        }
    }
}

impl DownloadListView {
    /// Add a new download.
    pub fn add(&mut self, download: Download) {
        self.model.current_id += 1;
        self.model.download_count += 1;

        let download_view = DownloadView::new(&download, self.model.current_id);

        connect!(self.model.relm, download, connect_decide_destination(download, suggested_filename),
            async |resolver| DecideDestination(resolver, download.clone(), suggested_filename.to_string()));

        let id = self.model.current_id;
        connect!(self.model.relm, download, connect_failed(_, error), DownloadFailed(error.clone(), id));
        connect!(self.model.relm, download, connect_finished(download), DownloadFinished(download.clone(), id));

        // TODO: call add_widget() instead.
        self.view.add(&**download_view);
        if let Some(flow_child) = self.view.get_children().last() {
            flow_child.set_can_focus(false);
        }
        download_view.show();

        // It is necessary to keep the download views because they are connected to events.
        self.model.download_views.push(download_view);
    }

    /// Add a file to be opened when its download finish.
    pub fn add_file_to_open(&mut self, path: &str) {
        self.model.downloads_to_open.push(path.to_string());
    }

    /// Delete a view and remove it from its parent.
    fn delete(&mut self, id: u32) {
        let index = self.model.download_views.iter()
            .position(|download_view| download_view.id == id);
        if let Some(index) = index {
            let download_view = self.model.download_views.remove(index);
            remove_from_flow_box(&**download_view);
        }
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
        self.model.download_count -= 1;
        if let Some(destination) = download.get_destination() {
            let index = self.model.downloads_to_open.iter()
                .position(|download_destination| *download_destination == destination);
            if let Some(index) = index {
                self.model.downloads_to_open.remove(index);
                open(destination);
            }
        }

        // Delete the view after a certain amount of time after the download finishes.
        let timeout = Timeout::new(Duration::from_secs(DOWNLOAD_TIME_BEFORE_HIDE));
        self.model.relm.connect_exec_ignore_err(timeout, move |_| Remove(id));
    }

    /// Check if there are active downloads.
    pub fn has_active_downloads(&self) -> bool {
        self.model.download_count > 0
    }
}

/// Remove the progress bar from its `FlowBox` parent.
fn remove_from_flow_box<W: IsA<gtk::Widget> + WidgetExt>(widget: &W) {
    let child: Option<Container> = widget.get_parent()
        .and_then(|parent| parent.downcast().ok());
    // FlowBox children are wrapped inside FlowBoxChild, so we need to destroy this
    // FlowBoxChild (which is the parent of the widget) in order to remove it from
    // the FlowBox.
    if let Some(child) = child {
        child.destroy();
    }
}
