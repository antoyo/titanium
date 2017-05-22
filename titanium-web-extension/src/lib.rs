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
extern crate fg_uds;
extern crate futures;
extern crate futures_glib;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate simplelog;
#[macro_use]
extern crate relm_state;
#[macro_use]
extern crate relm_derive_state;
extern crate titanium_common;
extern crate tokio_io;
extern crate url;
extern crate xdg;
#[macro_use]
extern crate webkit2gtk_webextension;

mod adblocker;
mod dom;
mod hints;
mod login_form;
mod message_client;
mod option_util;
mod scroll;

use std::mem::forget;

use glib::variant::Variant;
use log::LogLevel::Error;
use simplelog::{Config, TermLogger};
use simplelog::LogLevelFilter;
use webkit2gtk_webextension::WebExtension;

use message_client::MessageClient;
use message_client::Msg::PageCreated;

web_extension_init!();

pub const APP_NAME: &'static str = "titanium";

#[no_mangle]
pub fn web_extension_initialize(extension: WebExtension, user_data: Variant) {
    // TODO: Don't show trace.
    // TODO: show in colors?
    let config = Config {
        time: Some(Error),
        level: Some(Error),
        target: None,
        location: None,
    };
    TermLogger::init(LogLevelFilter::max(), config).ok();

    let server_name = user_data.get_str();
    if let Some(server_name) = server_name {
        let client = MessageClient::new(server_name, extension.clone());

        if let Ok(ref client) = client {
            connect!(extension, connect_page_created(_, page), client, PageCreated(page.clone()));
        }

        // Don't drop the client to keep receiving the messages on the stream.
        forget(client);
    }
}
