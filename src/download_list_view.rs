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

use std::collections::HashMap;
use std::time::Duration;

use futures_glib::Timeout;
use glib::Cast;
use gtk::{
    self,
    Container,
    ContainerExt,
    FlowBoxExt,
    IsA,
    SelectionMode,
    WidgetExt,
};
use relm::{Component, ContainerWidget, Relm, Widget};
use relm_attributes::widget;
use webkit2gtk::{
    Download,
    DownloadExt,
    Error,
};
use webkit2gtk::DownloadError::CancelledByUser;

use download_view::DownloadView;
use download_view::Msg::{
    Cancel,
    Destination,
    DownloadError,
    OriginalDestination,
    Remove,
    SetToOpen,
};
use self::Msg::*;

const DOWNLOAD_TIME_BEFORE_HIDE: u64 = 2;

pub struct Model {
    download_count: u32,
    download_views: HashMap<Download, Component<DownloadView>>,
    relm: Relm<DownloadListView>,
}

#[derive(Msg)]
pub enum Msg {
    ActiveDownloads(bool),
    Add(Download),
    AddFileToOpen(Download),
    DelayedRemove(Download),
    DownloadCancel(Download),
    DownloadDestination(Download, String),
    DownloadFailed(Error, Download),
    DownloadFinished(Download),
    DownloadListError(String),
    DownloadOriginalDestination(Download, String),
    DownloadRemove(Download),
}

#[widget]
impl Widget for DownloadListView {
    fn model(relm: &Relm<Self>, (): ()) -> Model {
        Model {
            download_count: 0,
            download_views: HashMap::new(),
            relm: relm.clone(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            // To be listened by the user.
            ActiveDownloads(_) => (),
            Add(download) => self.add(download),
            AddFileToOpen(download) => self.add_file_to_open(download),
            DelayedRemove(download) => self.delayed_remove(download),
            DownloadCancel(download) => self.download_cancel(download),
            DownloadDestination(download, destination) => self.download_destination(download, destination),
            DownloadFailed(error, download) => self.handle_failed(&error, download),
            DownloadFinished(_) => self.handle_finished(),
            DownloadListError(_) => (), // To be listened by the user.
            DownloadOriginalDestination(download, destination) =>
                self.download_original_destination(download, destination),
            DownloadRemove(download) => self.delete(download),
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
        self.model.download_count += 1;
        self.model.relm.stream().emit(ActiveDownloads(true));

        connect!(self.model.relm, download, connect_failed(download, error),
            DownloadFailed(error.clone(), download.clone()));
        connect!(self.model.relm, download, connect_finished(download), DownloadFinished(download.clone()));

        let download_view = self.view.add_widget::<DownloadView, _>(&self.model.relm, download.clone());
        if let Some(flow_child) = self.view.get_children().last() {
            flow_child.set_can_focus(false);
        }
        let down = download.clone();
        connect!(download_view@DownloadError(ref error), self.model.relm, DownloadListError(error.clone()));
        connect!(download_view@Remove, self.model.relm, DelayedRemove(down.clone()));

        // It is necessary to keep the download views because they are connected to events.
        let _ = self.model.download_views.insert(download, download_view);
    }

    /// Add a file to be opened when its download finish.
    pub fn add_file_to_open(&mut self, download: Download) {
        if let Some(download_view) = self.model.download_views.get(&download) {
            download_view.emit(SetToOpen);
        }
    }

    fn delayed_remove(&self, download: Download) {
        // Delete the view after a certain amount of time after the download finishes.
        let timeout = Timeout::new(Duration::from_secs(DOWNLOAD_TIME_BEFORE_HIDE));
        self.model.relm.connect_exec_ignore_err(timeout, move |_| DownloadRemove(download.clone()));
    }

    /// Delete a view and remove it from its parent.
    fn delete(&mut self, download: Download) {
        if let Some(download_view) = self.model.download_views.remove(&download) {
            remove_from_flow_box(download_view.widget());
        }
    }

    fn download_cancel(&self, download: Download) {
        if let Some(download_view) = self.model.download_views.get(&download) {
            download_view.emit(Cancel);
        }
        // TODO: warning?
    }

    fn download_destination(&self, download: Download, destination: String) {
        if let Some(download_view) = self.model.download_views.get(&download) {
            download_view.emit(Destination(destination));
        }
        // TODO: warning?
    }

    fn download_original_destination(&self, download: Download, destination: String) {
        if let Some(download_view) = self.model.download_views.get(&download) {
            download_view.emit(OriginalDestination(destination));
        }
        // TODO: warning?
    }

    /// Handle the download failed event.
    /// Delete the view if the download was cancelled by the user.
    fn handle_failed(&mut self, error: &Error, download: Download) {
        warn!("Download failed: {}", error);
        if let Some(error) = error.kind::<::webkit2gtk::DownloadError>() {
            if error == CancelledByUser {
                self.delete(download);
            }
        }
    }

    /// Handle the download fisished event.
    fn handle_finished(&mut self) {
        self.model.download_count -= 1;

        if self.model.download_count == 0 {
            self.model.relm.stream().emit(ActiveDownloads(false));
        }
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
