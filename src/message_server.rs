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

//! Message server interface.

use std::collections::HashMap;
use std::fs::remove_file;
use std::io;
use std::process;

use fg_uds::{UnixListener, UnixStream};
use futures::{AsyncSink, Sink};
use futures_glib::MainContext;
use gtk::{
    self,
    ButtonsType,
    DialogExt,
    DialogFlags,
    MessageDialog,
    MessageType,
    Window,
};
use relm::{Component, EventStream, Relm, Update, UpdateNew, execute, init};
use tokio_io::AsyncRead;
use tokio_io::codec::length_delimited::{FramedRead, FramedWrite};
use tokio_io::io::WriteHalf;
use tokio_serde_bincode::{ReadBincode, WriteBincode};
use webkit2gtk::WebContext;

use titanium_common::PATH;
use titanium_common::{ExtensionId, InnerMessage, Message, PageId};
use titanium_common::InnerMessage::Id;

use app::{self, App};
use app::Msg::{
    MessageRecv,
    CreateWindow,
    Remove,
    ServerSend,
    SetPageId,
    ShowError,
};
use config_dir::ConfigDir;
use errors::Error;
use self::Msg::*;
use webview::WebView;

pub struct AppServer {
    stream: EventStream<app::Msg>,
    writer: Option<MessageWriter>,
}

impl AppServer {
    fn new(stream: EventStream<app::Msg>) -> Self {
        AppServer {
            stream,
            writer: None,
        }
    }
}

pub struct MessageServer {
    model: Model,
}

pub struct Model {
    app_count: usize,
    app_extensions: HashMap<PageId, (ExtensionId, usize)>,
    apps: HashMap<PageId, AppServer>,
    config_dir: ConfigDir,
    extension_page: HashMap<PageId, ExtensionId>,
    listener: Option<UnixListener>,
    relm: Relm<MessageServer>,
    // TODO: save the widgets somewhere allowing to remove them when its window is closed.
    wins: Vec<Component<App>>,
    web_context: WebContext,
    writer_counter: usize,
    writers: HashMap<usize, MessageWriter>,
}

pub type MessageWriter = WriteBincode<FramedWrite<WriteHalf<UnixStream>>, Message>;

#[derive(Msg)]
pub enum Msg {
    AppPageId(EventStream<app::Msg>, PageId),
    ClientConnect(UnixStream),
    MsgRecv(usize, Message),
    MsgError(Error),
    NewApp(Option<String>),
    RemoveApp(PageId),
    Send(PageId, InnerMessage),
}

impl Update for MessageServer {
    type Model = Model;
    type ModelParam = (UnixListener, Vec<String>, Option<String>);
    type Msg = Msg;

    fn model(relm: &Relm<Self>, (listener, mut url, config): (UnixListener, Vec<String>, Option<String>)) -> Model {
        let url =
            if !url.is_empty() {
                // TODO: open the other URLs in new windows?
                Some(url.remove(0))
            }
            else {
                None
            };
        let config_dir = ConfigDir::new(&config).unwrap(); // TODO: remove unwrap().
        let web_context = WebView::initialize_web_extension(&config_dir);
        relm.stream().emit(NewApp(url));
        Model {
            app_count: 0,
            app_extensions: HashMap::new(),
            apps: HashMap::new(),
            config_dir,
            extension_page: HashMap::new(),
            listener: Some(listener),
            relm: relm.clone(),
            wins: vec![],
            web_context,
            writer_counter: 0,
            writers: HashMap::new(),
        }
    }

    fn subscriptions(&mut self, relm: &Relm<MessageServer>) {
        match self.model.listener.take() {
            Some(listener) =>
                relm.connect_exec(listener.incoming(),
                    |(stream, _addr)| ClientConnect(stream),
                    |error| MsgError(error.into())),
            None => dialog_and_exit("Message listener is not initialized"),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            AppPageId(stream, page_id) => {
                // FIXME: the writer is inserted too many times. It should be only once per web
                // extension, not once per page.
                let _ = self.model.apps.insert(page_id, AppServer::new(stream));
                if let Some((extension_id, writer_counter)) = self.model.app_extensions.remove(&page_id) {
                    self.connect_app_and_extension(extension_id, page_id, writer_counter);
                }
            },
            ClientConnect(stream) => {
                let (reader, writer) = stream.split();
                let reader = ReadBincode::new(FramedRead::new(reader));
                let writer = WriteBincode::new(FramedWrite::new(writer));
                let _ = self.model.writers.insert(self.model.writer_counter, writer);
                let counter = self.model.writer_counter;
                self.model.writer_counter += 1;
                self.model.relm.connect_exec(reader, move |msg| MsgRecv(counter, msg),
                    |error| MsgError(error.into()));
            },
            // To be listened by the app.
            MsgError(_) => (),
            MsgRecv(writer_counter, Message(page_id, message)) => self.msg_recv(writer_counter, page_id, message),
            NewApp(url) => self.add_app(url),
            RemoveApp(page_id) => self.remove_app(page_id),
            Send(page_id, message) => self.send(page_id, message),
        }
    }
}

impl UpdateNew for MessageServer {
    fn new(_relm: &Relm<Self>, model: Self::Model) -> Self {
        MessageServer {
            model,
        }
    }
}

impl MessageServer {
    pub fn new(url: Vec<String>, config_dir: Option<String>) -> io::Result<EventStream<<Self as Update>::Msg>> {
        let cx = MainContext::default(|cx| cx.clone());
        // TODO: should be removed on Drop instead (or the connection close should remove it
        // automatically?).
        // TODO: should open in the existing process if it already exists.
        // If no other titanium process can be found, delete it.
        // NOTE: Don't check for errors on remove_file() since it does not matter if the file does
        // not exist.
        let _ = remove_file(PATH);
        let listener = UnixListener::bind(PATH, &cx)?;
        Ok(execute::<MessageServer>((listener, url, config_dir)))
    }

    fn add_app(&mut self, url: Option<String>) {
        self.model.app_count += 1;
        let app = init::<App>((url, self.model.config_dir.clone(), self.model.web_context.clone())).unwrap(); // TODO: remove unwrap().
        let app_stream = app.stream().clone();
        connect!(app@SetPageId(page_id), self.model.relm, AppPageId(app_stream.clone(), page_id));
        connect!(app@ServerSend(page_id, ref message), self.model.relm, Send(page_id, message.clone()));
        connect!(app@CreateWindow(ref url), self.model.relm, NewApp(Some(url.clone())));
        connect!(app@Remove(page_id), self.model.relm, RemoveApp(page_id));
        self.model.wins.push(app);
    }

    fn connect_app_and_extension(&mut self, extension_id: ExtensionId, page_id: PageId, writer_counter: usize) {
        if let Some(ref mut app) = self.model.apps.get_mut(&page_id) {
            let _ = self.model.extension_page.insert(page_id, extension_id);
            if let Some(writer) = self.model.writers.remove(&writer_counter) {
                app.writer = Some(writer);
            }
        }
        else {
            error!("Cannot find app with page id {}", page_id);
        }
    }

    fn error(&self, page_id: PageId, error: Error) {
        if let Some(app) = self.model.apps.get(&page_id) {
            app.stream.emit(ShowError(error.to_string()));
        }
    }

    fn msg_recv(&mut self, writer_counter: usize, page_id: PageId, msg: InnerMessage) {
        if let Id(extension_id, page_id) = msg {
            if self.model.apps.contains_key(&page_id) {
                self.connect_app_and_extension(extension_id, page_id, writer_counter);
            }
            else {
                let _ = self.model.app_extensions.insert(page_id, (extension_id, writer_counter));
            }
        }
        else if let Some(ref app) = self.model.apps.get(&page_id) {
            app.stream.emit(MessageRecv(msg));
        }
        else {
            error!("Cannot find app with page id {}", page_id);
        }
    }

    fn remove_app(&mut self, page_id: PageId) {
        self.model.app_count -= 1;
        if let Some(extension_id) = self.model.extension_page.get(&page_id).cloned() {
            if page_id != extension_id {
                let _ = self.model.apps.remove(&page_id);
                let _ = self.model.extension_page.remove(&page_id);
            }
            // TODO: remove the apps with extension ID? It seems web extensions are not recreated.
            // Is it because the webview is not destroyed?
        }
        // TODO: remove from self.model.wins.
        if self.model.app_count == 0 {
            gtk::main_quit();
        }
    }

    pub fn send(&mut self, page_id: PageId, message: InnerMessage) {
        let mut error = None;
        if let Some(extension_id) = self.model.extension_page.get(&page_id) {
            if let Some(app) = self.model.apps.get_mut(&extension_id) {
                if let Some(ref mut writer) = app.writer {
                    match writer.start_send(Message(page_id, message)) {
                        Ok(AsyncSink::Ready) =>{
                            if let Err(poll_error) = writer.poll_complete() {
                                error = Some(poll_error.into());
                            }},
                        Ok(AsyncSink::NotReady(_)) => error = Some("not ready to send to client".into()),
                        Err(send_error) =>
                            error = Some(format!("cannot send a message to the web process: {}", send_error).into()),
                    }
                }
                else {
                    error = Some("message writer does not exist".into());
                }
            }
            else {
                error = Some("app does not exist".into());
            }
        }
        else {
            error = Some(format!("extension id for page {} does not exist", page_id).into());
        }
        if let Some(error) = error {
            self.error(page_id, error);
        }
    }
}

/// Create a new message server.
/// If it is not possible to create one, show the error and exit.
pub fn create_message_server(url: Vec<String>, config_dir: Option<String>) -> EventStream<<MessageServer as Update>::Msg> {
    match MessageServer::new(url, config_dir) {
        Ok(message_server) => message_server,
        Err(error) => {
            let message = format!("cannot create the message server used to communicate with the web processes: {}",
                error);
            dialog_and_exit(&message);
        },
    }
}

fn dialog_and_exit(message: &str) -> ! {
    let window: Option<&Window> = None;
    let message = format!("Fatal error: {}", message);
    let dialog = MessageDialog::new(window, DialogFlags::empty(), MessageType::Error, ButtonsType::Close, &message);
    let _ = dialog.run();
    process::exit(1);
}
