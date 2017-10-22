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

use std::collections::HashMap;
use std::io;

use fg_uds::{UnixStream, UnixStreamConnect};
use futures::{AsyncSink, Sink};
use futures_glib::MainContext;
use nix;
use relm_state::{EventStream, Relm, Update, UpdateNew, execute};
use tokio_io::AsyncRead;
use tokio_io::codec::length_delimited::{FramedRead, FramedWrite};
use tokio_io::io::WriteHalf;
use tokio_serde_bincode::{Error, ReadBincode, WriteBincode};
use webkit2gtk_webextension::{
    URIRequest,
    URIRequestExt,
    WebPage,
    WebPageExt,
};

use titanium_common::{ExtensionId, Message, PageId, SOCKET_NAME};
use titanium_common::InnerMessage;
use titanium_common::InnerMessage::*;

use adblocker::Adblocker;
use executor::Executor;
use executor::Msg::{DocumentLoaded, MessageRecv, ServerSend};
use self::Msg::*;

lazy_static! {
    static ref ADBLOCKER: Adblocker = Adblocker::new();
}

pub struct MessageClient {
    model: Model,
}

pub struct Model {
    executors: HashMap<PageId, EventStream<<Executor as Update>::Msg>>,
    extension_id: Option<ExtensionId>,
    page_id_to_send: Option<u64>,
    relm: Relm<MessageClient>,
    writer: Option<WriteBincode<FramedWrite<WriteHalf<UnixStream>>, Message>>,
}

#[derive(Msg)]
pub enum Msg {
    ConnectErr(io::Error),
    Connection(UnixStream),
    MsgRecv(Message),
    MsgError(Error),
    PageCreated(WebPage),
    Send(PageId, InnerMessage),
}

impl Update for MessageClient {
    type Model = Model;
    type ModelParam = UnixStreamConnect;
    type Msg = Msg;

    fn model(relm: &Relm<Self>, connection: Self::ModelParam) -> Model {
        relm.connect_exec(connection, Connection, ConnectErr);
        Model {
            executors: HashMap::new(),
            extension_id: None,
            page_id_to_send: None,
            relm: relm.clone(),
            writer: None,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            ConnectErr(error) => error!("MsgError: {}", error),
            Connection(stream) => {
                let (reader, writer) = stream.split();
                self.model.writer = Some(WriteBincode::new(FramedWrite::new(writer)));
                let reader = ReadBincode::new(FramedRead::new(reader));
                self.model.relm.connect_exec(reader, MsgRecv, MsgError);
                self.send_page_id();
            },
            MsgError(error) => error!("MsgError: {}", error),
            MsgRecv(Message(page_id, msg)) => {
                if let Some(executor) = self.model.executors.get(&page_id) {
                    executor.emit(MessageRecv(msg));
                }
                else {
                    error!("Cannot find executor with page ID {}", page_id);
                }
            },
            PageCreated(page) => {
                // TODO: this should be disconnected later somehow.
                connect!(self.model.relm, page, connect_send_request(_, request, _),
                    return block_request(request));
                let page_id = page.get_id();
                trace!("New page created with id {}", page_id);
                if self.model.extension_id.is_none() {
                    self.model.extension_id = Some(page_id);
                    self.send_page_id();
                }
                let executor = execute::<Executor>(page.clone());
                connect_stream!(page, connect_document_loaded(_), executor, DocumentLoaded);
                connect_stream!(executor@ServerSend(page_id, ref msg),
                    self.model.relm.stream(), Send(page_id, msg.clone()));
                self.model.executors.insert(page_id, executor);

                let extension_id = self.model.extension_id.unwrap();
                if self.model.writer.is_some() {
                    trace!("Send page id {}", page_id);
                    self.send(page_id, Id(extension_id, page_id));
                }
                else {
                    self.model.page_id_to_send = Some(page_id);
                }
            },
            Send(page_id, msg) => self.send(page_id, msg),
        }
    }
}

impl UpdateNew for MessageClient {
    fn new(_relm: &Relm<Self>, model: Model) -> Self {
        MessageClient {
            model,
        }
    }
}

impl MessageClient {
    pub fn new() -> nix::Result<EventStream<<Self as Update>::Msg>> {
        let cx = MainContext::default(|cx| cx.clone());
        let stream = UnixStream::connect_abstract(SOCKET_NAME, &cx)?;
        Ok(execute::<MessageClient>(stream))
    }

    // Send a message to the server.
    fn send(&mut self, page_id: PageId, msg: InnerMessage) {
        if let Some(ref mut writer) = self.model.writer {
            match writer.start_send(Message(page_id, msg)) {
                Ok(AsyncSink::Ready) =>
                    if let Err(error) = writer.poll_complete() {
                        error!("Cannot send message to UI process: {}", error);
                    },
                Ok(AsyncSink::NotReady(_)) => error!("not ready to send to server"),
                Err(send_error) =>
                    error!("cannot send a message to the web process: {}", send_error),
            }
        }
        else {
            println!("No writer");
        }
    }

    fn send_page_id(&mut self) {
        if let Some(extension_id) = self.model.extension_id {
            if let Some(page_id) = self.model.page_id_to_send {
                trace!("Send page id {}", page_id);
                self.send(page_id, Id(extension_id, page_id));
            }
        }
    }
}

fn block_request(request: &URIRequest) -> bool {
    if let Some(url) = request.get_uri() {
        return ADBLOCKER.should_block(&url);
    }
    false
}
