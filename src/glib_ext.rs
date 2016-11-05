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

use std::ffi::{CStr, CString};

pub use glib_sys::G_USER_DIRECTORY_DOWNLOAD;
use glib_sys::{GUserDirectory, g_get_user_special_dir, g_markup_escape_text};

/// Returns the full path of a special directory using its logical id.
pub fn get_user_special_dir(user_directory: GUserDirectory) -> String {
    unsafe {
        let path = g_get_user_special_dir(user_directory);
        CStr::from_ptr(path).to_str().unwrap().to_string()
    }
}

/// Escapes text so that the markup parser will parse it verbatim. Less than, greater than, ampersand, etc. are replaced with the corresponding entities. This function would typically be used when writing out a file to be parsed with the markup parser.
/// Note that this function doesn't protect whitespace and line endings from being processed according to the XML rules for normalization of line endings and attribute values.
/// Note also that this function will produce character references in the range of &x1; ... &x1f; for all control sequences except for tabstop, newline and carriage return. The character references in this range are not valid XML 1.0, but they are valid XML 1.1 and will be accepted by the GMarkup parser.
pub fn markup_escape_text(text: &str) -> String {
    unsafe {
        let cstring = CString::new(text).unwrap();
        let escaped_text = g_markup_escape_text(cstring.as_ptr(), text.len() as isize);
        CStr::from_ptr(escaped_text).to_str().unwrap().to_string()
    }
}
