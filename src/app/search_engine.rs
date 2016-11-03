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

//! Search engine application support.

use mg::message::MessageWindow;

use urls::is_url;

use super::App;

impl App {
    /// Add a search engine.
    pub fn add_search_engine(&self, args: &str) {
        let args: Vec<_> = args.split_whitespace().collect();
        if args.len() == 2 {
            let keyword = args[0].to_string();
            if (*self.default_search_engine.borrow()).is_none() {
                *self.default_search_engine.borrow_mut() = Some(keyword.clone());
            }
            (*self.search_engines.borrow_mut()).insert(keyword, args[1].to_string());
        }
        else {
            self.app.error(&format!("search-engine: expecting 2 arguments, got {} arguments", args.len()));
        }
    }

    /// If the url starts with a search engine keyword, transform the url to the URL of the search
    /// engine.
    pub fn transform_url(&self, url: &str) -> String {
        let words: Vec<_> = url.split_whitespace().collect();
        let (engine_prefix, rest) =
            if words.len() > 1 && (*self.search_engines.borrow()).contains_key(words[0]) {
                let rest = url.chars().skip_while(|&c| c != ' ').collect::<String>();
                let rest = rest.trim().to_string();
                (Some(words[0].to_string()), rest)
            }
            else if !is_url(url) {
                ((*self.default_search_engine.borrow()).clone(), url.to_string())
            }
            else {
                (None, String::new())
            };
        if let Some(ref prefix) = engine_prefix {
            if let Some(engine_url) = (*self.search_engines.borrow()).get(prefix) {
                return engine_url.replace("{}", &rest);
            }
        }
        url.to_string()
    }
}
