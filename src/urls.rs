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

use regex::Regex;
use url::Url;
use url::percent_encoding::percent_decode;

/// Get the base URL (domain and tld) of an URL.
/// Returns an empty string in case there are no hosts.
pub fn get_base_url(url: &str) -> String {
    let parsed_url = Url::parse(url).unwrap();
    let mut base_url = parsed_url.host_str().unwrap_or("").to_string();
    // Remove all sub-domains.
    let mut period_count = base_url.chars().filter(|&c| c == '.').count();
    while period_count > 1 {
        base_url = base_url.chars().skip_while(|&c| c != '.').skip(1).collect();
        period_count = base_url.chars().filter(|&c| c == '.').count();
    }
    base_url
}

/// Get the filename from the URL.
pub fn get_filename(url: &str) -> Option<String> {
    let parsed_url = Url::parse(url).unwrap();
    parsed_url.path_segments()
        .and_then(|segments| segments.last())
        .and_then(|filename| percent_decode(filename.as_bytes()).decode_utf8().ok())
        .map(|string| string.into_owned())
}

/// Check if the input string looks like a URL.
pub fn is_url(input: &str) -> bool {
    let regex = Regex::new(r"[-a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,6}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)").unwrap();
    regex.is_match(input)
}
