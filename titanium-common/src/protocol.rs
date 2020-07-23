/*
 * Copyright (c) 2018 Boucher, Antoni <bouanto@zoho.com>
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

//! Communication protocol between the UI process and the web processes.

use std::collections::VecDeque;
use std::io::{Cursor, Seek};
use std::io::SeekFrom::Start;

use gio::{
    prelude::InputStreamExtManual,
    prelude::OutputStreamExtManual,
    Cancellable,
    InputStream,
    IOStream,
    IOStreamExt,
    OutputStream,
};
use glib::{Error, PRIORITY_DEFAULT};
use relm::{
    Relm,
    Update,
    UpdateNew,
    connect_async_full,
};
use rmp_serialize::{Decoder, Encoder};
use rustc_serialize::{Decodable, Encodable};

use Message;
use self::Msg::*;
use self::SendMode::{Async, Sync};

const BUFFER_SIZE: usize = 1024;
const HEADER_SIZE: usize = 4;

#[allow(missing_docs)]
pub struct Model {
    buffer: Vec<u8>,
    current_msg_size: Option<u32>,
    queue: VecDeque<Message>,
    reader: Option<InputStream>,
    relm: Relm<PluginProtocol>,
    sending_message: bool,
    _stream: IOStream,
    writer: Option<OutputStream>,
}

/// The variant MsgRead can be listened by the caller in order to get notified when a message is
/// received.
#[allow(missing_docs)]
#[derive(Msg)]
pub enum Msg {
    MsgRead(Message),
    Read((Vec<u8>, usize)),
    IOError(Error),
    WriteMsg(Message),
    Wrote,
}

// NOTE: safe because the main loop is ran on the main thread.
unsafe impl Send for Msg {}

/// This plugin protocol uses MessagePack to communicate.
pub struct PluginProtocol {
    model: Model,
}

impl Update for PluginProtocol {
    type Model = Model;
    type ModelParam = IOStream;
    type Msg = Msg;

    fn model(relm: &Relm<Self>, stream: IOStream) -> Model {
        let reader = stream.get_input_stream();
        let writer = stream.get_output_stream();
        if let Some(ref reader) = reader {
            connect_async_full!(reader, read_async(vec![0; BUFFER_SIZE], PRIORITY_DEFAULT), relm, Read,
                |(_, error)| IOError(error));
        }
        Model {
            buffer: vec![],
            current_msg_size: None,
            queue: VecDeque::new(),
            reader,
            relm: relm.clone(),
            sending_message: false,
            // Keep the stream to avoid it being closed.
            _stream: stream,
            writer,
        }
    }

    fn update(&mut self, message: Msg) {
        match message {
            // To be listened to by the user.
            IOError(_) => (),
            // To be listened by the user.
            MsgRead(_) => (),
            Read((mut buffer, size)) => {
                buffer.truncate(size);
                self.model.buffer.extend(&buffer);
                let mut msg_read = true;
                while msg_read && !self.model.buffer.is_empty() {
                    msg_read = false;
                    if self.model.current_msg_size.is_none() {
                        self.model.current_msg_size = buf_to_u32(&self.model.buffer);
                        if self.model.buffer.len() <= HEADER_SIZE {
                            self.model.buffer = vec![];
                        }
                        else {
                            self.model.buffer = self.model.buffer[HEADER_SIZE..].to_vec(); // TODO: avoid the copy.
                        }
                    }
                    if let Some(msg_size) = self.model.current_msg_size {
                        let msg_size = msg_size as usize;
                        if self.model.buffer.len() >= msg_size {
                            {
                                let mut decoder = Decoder::new(&self.model.buffer[..msg_size]);
                                match Decodable::decode(&mut decoder) {
                                    Ok(msg) => {
                                        self.model.relm.stream().emit(MsgRead(msg));
                                        self.model.current_msg_size = None;
                                    },
                                    Err(error) => {
                                        error!("Failed to deserialize message. {:?}", error);
                                    },
                                }
                            }
                            self.model.buffer = self.model.buffer[msg_size..].to_vec(); // TODO: avoid the copy.
                            msg_read = true;
                        }
                    }
                }
                if size > 0 {
                    if let Some(ref reader) = self.model.reader {
                        connect_async_full!(reader, read_async(vec![0; BUFFER_SIZE], PRIORITY_DEFAULT),
                        self.model.relm, Read, |(_, error)| IOError(error));
                    }
                }
            },
            WriteMsg(msg) => {
                self.model.queue.push_back(msg);
                self.send();
            },
            Wrote => {
                self.model.sending_message = false;
                self.send();
            },
        }
    }
}

impl UpdateNew for PluginProtocol {
    fn new(_relm: &Relm<Self>, model: Model) -> Self {
        PluginProtocol {
            model,
        }
    }
}

impl PluginProtocol {
    fn send(&mut self) {
        if let Some(ref writer) = self.model.writer {
            // TODO: implement back-pressure.
            if !self.model.sending_message {
                if let Some(msg) = self.model.queue.pop_front() {
                    self.model.sending_message = true;
                    send(writer, msg, Async(&self.model.relm));
                }
            }
        }
        else {
            error!("No writer for protocol");
        }
    }
}

fn buf_to_u32(buffer: &[u8]) -> Option<u32> {
    if buffer.len() >= HEADER_SIZE {
        Some(buffer[0] as u32 | (buffer[1] as u32) << 8 | (buffer[2] as u32) << 16 | (buffer[3] as u32) << 24)
    }
    else {
        None
    }
}

fn write_u32(buffer: &mut [u8], size: u32) {
    if buffer.len() >= 4 {
        buffer[0] = (size & 0xFF) as u8;
        buffer[1] = ((size >> 8) & 0xFF) as u8;
        buffer[2] = ((size >> 16) & 0xFF) as u8;
        buffer[3] = ((size >> 24) & 0xFF) as u8;
    }
}

/// Whether to send a message asynchronously or synchronously.
pub enum SendMode<'a> {
    /// The message Wrote is sent on a successful write and IOError if there was an error.
    Async(&'a Relm<PluginProtocol>),
    /// The error is shown when there's one.
    Sync,
}

/// Send a `msg` of the `writer`.
pub fn send(writer: &OutputStream, msg: Message, send_mode: SendMode) {
    // Reserve space to write the size.
    let buffer = vec![0; HEADER_SIZE];
    let mut cursor = Cursor::new(buffer);
    if cursor.seek(Start(HEADER_SIZE as u64)).is_ok() {
        match msg.encode(&mut Encoder::new(&mut &mut cursor)) {
            Ok(_) => {
                let mut buffer = cursor.into_inner();
                let size = buffer.len() - HEADER_SIZE;
                write_u32(&mut buffer, size as u32);
                match send_mode {
                    Async(relm) => {
                        connect_async_full!(writer, write_all_async(buffer, PRIORITY_DEFAULT), relm,
                        |_| Wrote,
                        |(_, error)| IOError(error)
                        );
                    },
                    Sync =>
                        if let Err(error) = writer.write_all(&buffer, None::<&Cancellable>) {
                            // TODO: send these errors back to the web extension?
                            error!("Send error: {}", error);
                        }
                }
            },
            Err(error) => error!("Failed to serialize message. {}", error),
        }
    }
    else {
        error!("Failed to seek buffer.");
    }
}
