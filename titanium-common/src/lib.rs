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

extern crate bincode;
extern crate bytes;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tokio_io;

use std::io::ErrorKind::UnexpectedEof;

use bincode::{Infinite, deserialize_from, serialize_into};
use bincode::ErrorKind::IoError;
use bytes::{Buf, BufMut, BytesMut, IntoBuf};
use tokio_io::codec::{Decoder, Encoder};

pub type Result<T> = bincode::Result<T>;

/// Action that should be executed from the UI process.
pub enum Action {
    FileInput = 0,
    GoInInsertMode,
    NoAction,
}

impl Action {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            _ if value == Action::FileInput as i32 => Some(Action::FileInput),
            _ if value == Action::GoInInsertMode as i32 => Some(Action::GoInInsertMode),
            _ if value == Action::NoAction as i32 => Some(Action::NoAction),
            _ => None,
        }
    }
}

pub struct ExtCodec;

impl Decoder for ExtCodec {
    type Item = Message;
    type Error = bincode::Error;

    fn decode(&mut self, bytes: &mut BytesMut) -> Result<Option<Message>> {
        let buf = bytes.take().into_buf();
        let result = deserialize_from(&mut buf.bytes(), Infinite);
        match result {
            Ok(msg) => Ok(Some(msg)),
            Err(error) => {
                if let IoError(ref error) = *error {
                    if error.kind() == UnexpectedEof {
                        return Ok(None);
                    }
                }
                Err(error)
            },
        }
    }
}

impl Encoder for ExtCodec {
    type Item = Message;
    type Error = bincode::Error;

    fn encode(&mut self, msg: Message, buf: &mut BytesMut) -> Result<()> {
        serialize_into(&mut buf.writer(), &msg, Infinite)?;
        Ok(())
    }
}

// Switch to SimpleMsg to avoid these empty ().
#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    /// Response to ActivateHint.
    ActivateAction(i32),
    ActivateHint(String),
    ActivateSelection(),
    /// Response to EnterHintKey.
    ClickHintElement(),
    /// Response to GetCredentials.
    Credentials(String, String),
    EnterHintKey(char),
    /// Response to FocusInput.
    EnterInsertMode(),
    FocusInput(),
    GetCredentials(),
    GetScrollPercentage(),
    HideHints(),
    LoadPassword(String),
    LoadUsername(String),
    ScrollBottom(),
    ScrollBy(i64),
    ScrollByX(i64),
    /// Response of GetScrollPercentage.
    ScrollPercentage(i64),
    ScrollTop(),
    SelectFile(String),
    ShowHints(String),
    SubmitLoginForm(),
}
