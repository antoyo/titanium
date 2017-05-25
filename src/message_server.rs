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

use bincode;
use futures::{AsyncSink, Sink};
use fg_uds::{UnixListener, UnixStream};
use futures_glib::MainContext;
use relm::{Component, Relm, Update, execute};
use tokio_io::AsyncRead;
use tokio_io::codec::{FramedRead, FramedWrite};
use tokio_io::io::WriteHalf;

use titanium_common::{ExtCodec, Result, Message};
use titanium_common::Message::ScrollPercentage;

use self::Msg::*;

// TODO: put in the home directory.
pub const PATH: &str = "/tmp/titanium";

struct Client {
    writer: FramedWrite<WriteHalf<UnixStream>, ExtCodec>,
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
    IncomingError(String), // TODO: use a better type.
    MsgRecv(usize, Message),
    MsgError(Box<bincode::ErrorKind>),
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

    fn new(_relm: &Relm<Self>, model: Self::Model) -> Option<Self> {
        Some(MessageServer {
            model,
        })
    }

    fn subscriptions(&mut self, relm: &Relm<MessageServer>) {
        relm.connect_exec(self.model.listener.take().expect("listener").incoming(),
            |(stream, _addr)| ClientConnect(stream),
            |error| IncomingError(error.to_string()));
    }

    fn update(&mut self, event: Msg) {
        match event {
            ClientConnect(stream) => {
                let client_id = 0;
                let (reader, writer) = stream.split();
                let reader = FramedRead::new(reader, ExtCodec);
                let writer = FramedWrite::new(writer, ExtCodec);
                self.model.clients.insert(client_id, Client {
                    writer,
                });
                self.model.relm.connect_exec(reader, move |msg| MsgRecv(client_id, msg), MsgError);
            },
            IncomingError(error) => println!("{}", error), // TODO
            MsgError(error) => println!("Error: {}", error), // TODO,
            // To be listened by the app.
            MsgRecv(_, _) => (),
        }
    }
}

impl MessageServer {
    pub fn new() -> io::Result<Component<Self>> {
        let cx = MainContext::default(|cx| cx.clone());
        // TODO: should be removed on Drop instead (or the connection close should remove it
        // automtically?).
        remove_file(PATH).ok();
        let listener = UnixListener::bind(PATH, &cx)?;
        Ok(execute::<MessageServer>(listener))
    }

    pub fn send(&mut self, client: usize, msg: Message) -> Result<()> {
        if let Some(client) = self.model.clients.get_mut(&client) {
            if let Ok(AsyncSink::Ready) = client.writer.start_send(msg) {
                client.writer.poll_complete()?;
            }
        }
        else {
            // TODO: return Err
        }
        Ok(())
    }
}

/*dbus_interface!(
#[dbus("com.titanium.client")]
interface MessageServer {
    fn activate_hint(&mut self, follow_mode: &str) -> i32;
    fn activate_selection(&self);
    fn enter_hint_key(&mut self, key: char) -> bool;
    fn focus_input(&self) -> bool;
    fn get_credentials(&self) -> (String, String);
    fn get_scroll_percentage(&self) -> i64;
    fn hide_hints(&self);
    fn load_password(&self, password: &str);
    fn load_username(&self, username: &str);
    fn scroll_bottom(&self);
    fn scroll_by(&self, pixels: i64);
    fn scroll_by_x(&self, pixels: i64);
    fn scroll_top(&self);
    fn select_file(&self, file: &str);
    fn show_hints(&mut self, hint_chars: &str);
    fn submit_login_form(&self);
}
);*/
