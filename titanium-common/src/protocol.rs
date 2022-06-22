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

use rmp_serialize::{Decoder, Encoder};
use rustc_serialize::{Decodable, Encodable};

use Message;
use crate::InnerMessage;

/// Encode a message to be sent on the wire.
pub fn encode(msg: InnerMessage) -> Result<Vec<u8>, String> {
    let mut buffer = vec![];
    match Message(msg).encode(&mut Encoder::new(&mut buffer)) {
        Ok(_) => {
            return Ok(buffer);
        },
        Err(error) => return Err(format!("Failed to serialize message. {}", error)),
    }
}

/// Decode a message from bytes.
pub fn decode_bytes(bytes: Vec<u8>) -> Result<InnerMessage, String> {
    let mut decoder = Decoder::new(&*bytes);
    let msg: Message =
        match Decodable::decode(&mut decoder) {
            Ok(msg) => msg,
            Err(error) => return Err(format!("Failed to deserialize message. {:?}", error)),
        };
    Ok(msg.0)
}

/// Decode a user message.
pub fn decode(msg: &Option<glib::Variant>) -> Result<InnerMessage, String> {
    if let Some(variant) = msg {
        match variant.get::<Vec<u8>>() {
            Some(bytes) => {
                match decode_bytes(bytes) {
                    Ok(msg) => Ok(msg),
                    Err(error) => Err(error),
                }
            },
            None => {
                Err("User message parameter doesn't contain bytes.".to_string())
            },
        }
    }
    else {
        Err("Missing parameter in user message".to_string())
    }
}
