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

use std::borrow::Cow;

/// Get the whitelist URLs from a stylesheet.
/// The supported syntax is:
/// @document [ <url> | domain(<string>) ] {
/// }
/// and the @document must be on the very first line of the stylesheet and the last } must be
/// on the last line.
pub fn get_stylesheet_and_whitelist(content: &str) -> (Cow<str>, Vec<String>) {
    let mut whitelist = vec![];
    let mut words = content.split_whitespace();
    let stylesheet =
        if let Some(first_word) = words.next() {
            if first_word == "@document" {
                let document_parameters = words.take_while(|&word| word != "{");
                for parameter in document_parameters {
                    let parameter = parameter.trim_matches(',');
                    whitelist.append(&mut get_urls_from_parameter(parameter));
                }
                let stylesheet: String = content.chars().skip_while(|&c| c != '{').skip(1).collect();
                let mut lines: Vec<_> = stylesheet.lines().collect();
                let _ = lines.pop(); // Remove the last line which contains }.
                lines.join("\n").into()
            }
            else {
                content.into()
            }
        }
        else {
            content.into()
        };
    (stylesheet, whitelist)
}

/// Get the urls from a paramater.
fn get_urls_from_parameter(parameter: &str) -> Vec<String> {
    let mut whitelist = vec![];
    if parameter.starts_with('"') {
        //Remove the surrounding quotes.
        whitelist.push(parameter.trim_matches('"').to_string());
    }
    else {
        let function: String = parameter.chars().take_while(|&c| c != '(').collect();
        if function == "domain" {
            let mut domain: String = parameter.chars().skip_while(|&c| c != '(').skip(1).collect();
            let _ = domain.pop(); // Remove the ) at the end.
            let domain = domain.trim_matches('"'); //Remove the surrounding quotes.
            whitelist.push(format!("http://*.{}/*", domain));
            whitelist.push(format!("https://*.{}/*", domain));
            whitelist.push(format!("http://{}/*", domain));
            whitelist.push(format!("https://{}/*", domain));
        }
    }
    whitelist
}
