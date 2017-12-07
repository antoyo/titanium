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

use std::fmt::{self, Display, Formatter};
use std::io;
use std::result;
use std::str::Utf8Error;

use nix;
use password_store;
use rusqlite;
use tokio_serde_bincode;
use zip::result::ZipError;

pub struct Error {
    msg: String,
}

impl Error {
    pub fn new(msg: &str) -> Self {
        Error {
            msg: msg.to_string(),
        }
    }

    pub fn from_string(msg: String) -> Self {
        Error {
            msg,
        }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.msg)
    }
}

impl<'a> From<&'a str> for Error {
    fn from(msg: &'a str) -> Self {
        Error {
            msg: msg.to_string(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error {
            msg: error.to_string(),
        }
    }
}

impl From<nix::Error> for Error {
    fn from(error: nix::Error) -> Self {
        Error {
            msg: error.to_string(),
        }
    }
}

impl From<password_store::Error> for Error {
    fn from(error: password_store::Error) -> Self {
        Error {
            msg: error.to_string(),
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        Error {
            msg: error.to_string(),
        }
    }
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Error {
            msg: error.to_string(),
        }
    }
}

impl From<tokio_serde_bincode::Error> for Error {
    fn from(error: tokio_serde_bincode::Error) -> Self {
        Error {
            msg: error.to_string(),
        }
    }
}

impl From<ZipError> for Error {
    fn from(error: ZipError) -> Self {
        Error {
            msg: error.to_string(),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;
