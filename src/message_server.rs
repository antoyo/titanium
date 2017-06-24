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

use futures::{AsyncSink, Sink};
use fg_uds::{UnixListener, UnixStream};
use futures_glib::MainContext;
use gtk::{
    ButtonsType,
    DialogExt,
    DialogFlags,
    MessageDialog,
    MessageType,
    Window,
};
use relm::{EventStream, Relm, Update, UpdateNew, execute};
use tokio_io::AsyncRead;
use tokio_io::codec::length_delimited::{FramedRead, FramedWrite};
use tokio_io::io::WriteHalf;
use tokio_serde_bincode::{ReadBincode, WriteBincode};

use titanium_common::Message;

use errors::Error;
use self::Msg::*;

// TODO: put in the home directory.
pub const PATH: &str = "/tmp/titanium";

struct Client {
    writer: WriteBincode<FramedWrite<WriteHalf<UnixStream>>, Message>,
}

pub struct MessageServer {
    model: Model,
}

pub struct Model {
    clients: HashMap<usize, Client>,
    listener: Option<UnixListener>,
    relm: Relm<MessageServer>,
}

#[derive(Msg)]
pub enum Msg {
    ClientConnect(UnixStream),
    MsgRecv(usize, Message),
    MsgError(Error),
    Send(usize, Message),
}

impl Update for MessageServer {
    type Model = Model;
    type ModelParam = UnixListener;
    type Msg = Msg;

    fn model(relm: &Relm<Self>, listener: UnixListener) -> Model {
        Model {
            clients: HashMap::new(),
            listener: Some(listener),
            relm: relm.clone(),
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
            ClientConnect(stream) => {
                let client_id = 0;
                let (reader, writer) = stream.split();
                let reader = ReadBincode::new(FramedRead::new(reader));
                let writer = WriteBincode::new(FramedWrite::new(writer));
                let _ = self.model.clients.insert(client_id, Client {
                    writer,
                });
                self.model.relm.connect_exec(reader, move |msg| MsgRecv(client_id, msg),
                    |error| MsgError(error.into()));
            },
            // To be listened by the app.
            MsgError(_) | MsgRecv(_, _) => (),
            Send(client, msg) => self.send(client, msg),
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
    pub fn new() -> io::Result<EventStream<<Self as Update>::Msg>> {
        let cx = MainContext::default(|cx| cx.clone());
        // TODO: should be removed on Drop instead (or the connection close should remove it
        // automatically?).
        // TODO: should open in the existing process if it already exists.
        // If no other titanium process can be found, delete it.
        // NOTE: Don't check for errors on remove_file() since it does not matter if the file does
        // not exist.
        let _ = remove_file(PATH);
        let listener = UnixListener::bind(PATH, &cx)?;
        Ok(execute::<MessageServer>(listener))
    }

    fn error(&self, error: Error) {
        self.model.relm.stream().emit(MsgError(error));
    }

    fn send(&mut self, client: usize, msg: Message) {
        let mut error = None;
        if let Some(client) = self.model.clients.get_mut(&client) {
            match client.writer.start_send(msg) {
                Ok(AsyncSink::Ready) =>
                    if let Err(poll_error) = client.writer.poll_complete() {
                        error = Some(poll_error.into());
                    },
                Ok(AsyncSink::NotReady(_)) => error = Some("not ready to send to client".into()),
                Err(send_error) =>
                    error = Some(format!("cannot send a message to the web process: {}", send_error).into()),
            }
        }
        else {
            error = Some(format!("client {} does not exist", client).into());
        }
        if let Some(error) = error {
            self.error(error);
        }
    }
}

/// Create a new message server.
/// If it is not possible to create one, show the error and exit.
pub fn create_message_server() -> EventStream<<MessageServer as Update>::Msg> {
    match MessageServer::new() {
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
