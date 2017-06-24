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

//! Utility functions related to file.

use std::thread;

use INVALID_UTF8_ERROR;
use errors::{ErrorKind, Result};
use open;
use tempfile::NamedTempFileOptions;

/// Generate a unique filename from `filename`.
pub fn gen_unique_filename(filename: &str) -> Result<String> {
    let (prefix, suffix) =
        if let Some(index) = filename.rfind('.') {
            (&filename[..index], &filename[index..])
        }
        else {
            (filename, "")
        };
    let file = NamedTempFileOptions::new()
        .prefix(prefix)
        .suffix(suffix)
        .create()?;
    let filename =
        file.path().file_name()
            .ok_or(ErrorKind::Msg("generated file name has no file name".to_string()))?
            .to_str()
            .ok_or(ErrorKind::Msg(INVALID_UTF8_ERROR.to_string()))?
            .to_string();
    Ok(filename)
}

/// Open a file in a new process.
pub fn open(url: String) {
    thread::spawn(move ||
        open::that(url).ok()
    );
}
