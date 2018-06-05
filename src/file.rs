/*
 * Copyright (c) 2016-2018 Boucher, Antoni <bouanto@zoho.com>
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

use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
use std::thread;

use INVALID_UTF8_ERROR;
use errors::{Error, Result};
use open;
use tempfile::Builder as TempFileBuilder;

/// Generate a unique filename from `filename`.
pub fn gen_unique_filename(filename: &str) -> Result<String> {
    let (prefix, suffix) =
        if let Some(index) = filename.rfind('.') {
            (&filename[..index], &filename[index..])
        }
        else {
            (filename, "")
        };
    let file = TempFileBuilder::new()
        .prefix(prefix)
        .suffix(suffix)
        .tempfile()?;
    let filename =
        file.path().file_name()
            .ok_or_else(|| Error::new("generated file name has no file name"))?
            .to_str()
            .ok_or_else(|| Error::new(INVALID_UTF8_ERROR))?
            .to_string();
    Ok(filename)
}

pub fn open<P: AsRef<Path> + AsRef<OsStr>>(path: P) -> Result<File> {
    let string = AsRef::<OsStr>::as_ref(&path).to_string_lossy();
    File::open(&path)
        .map_err(|err| Error::new(&format!("Cannot open file {}: {}", string, err)))
}

/// Open a file in a new process.
pub fn open_app_for_file(url: String) {
    thread::spawn(move ||
        open::that(url).ok()
    );
}
