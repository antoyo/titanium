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

//! Web extension for the titanium web browser.
//! It provides an ad blocker, scrolling support, hints, navigation and login credentials load/save.

#![allow(deprecated)]
#![warn(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
)]

extern crate adblock;
extern crate gio;
extern crate glib;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate regex;
extern crate relm;
#[macro_use]
extern crate relm_derive;
extern crate send_cell;
extern crate titanium_common;
extern crate xdg;
extern crate webkit2gtk_webextension;

macro_rules! check_err {
    ($e:expr) => {
        if let Err(error) = $e {
            error!("check_err: {}", error);
            return;
        }
    };
}

macro_rules! check_err_opt {
    ($e:expr) => {
        if $e.is_none() {
            error!("{} is None", stringify!($e));
            return None;
        }
    };
}

macro_rules! unwrap_opt_or_ret {
    ($e:expr, $default:expr) => {
        match $e {
            Some(expr) => expr,
            None => return $default,
        }
    };
}

macro_rules! unwrap_or_ret {
    ($e:expr, $default:expr) => {
        match $e {
            Ok(expr) => expr,
            Err(error) => {
                error!("unwrap_or_ret: {}", error);
                return $default;
            },
        }
    };
}

macro_rules! wtry {
    ($e:expr) => {
        match $e {
            Ok(expr) => expr,
            Err(error) => {
                error!("wtry: {}", error);
                return;
            },
        }
    };
}

macro_rules! wtry_no_show {
    ($e:expr) => {
        match $e {
            Ok(expr) => expr,
            Err(_) => {
                error!("Error during this operation: {}", stringify!($e));
                return;
            },
        }
    };
}

macro_rules! wtry_opt {
    ($e:expr) => {
        match $e {
            Some(expr) => expr,
            None => {
                error!("{} returned None", stringify!($e));
                return None;
            },
        }
    };
}

macro_rules! wtry_opt_no_ret {
    ($e:expr) => {
        match $e {
            Some(expr) => expr,
            None => {
                error!("{} returned None", stringify!($e));
                return;
            },
        }
    };
}

mod adblocker;
mod dom;
mod executor;
mod hints;
mod login_form;
mod message_client;
mod option_util;

use std::mem::forget;

use log::LogLevel::Error;
use relm::connect_stream;
use simplelog::{Config, TermLogger};
use simplelog::LogLevelFilter;
use webkit2gtk_webextension::{WebExtension, traits::WebExtensionExt, web_extension_init};

use message_client::MessageClient;
use message_client::Msg::PageCreated;

web_extension_init!();

#[doc(hidden)]
pub const APP_NAME: &'static str = "titanium";

/// Initialize the the logger and the message server.
pub fn web_extension_initialize(extension: &WebExtension) {
    let config = Config {
        time: Some(Error),
        level: Some(Error),
        target: None,
        location: None,
    };

    if let Err(error) = TermLogger::init(LogLevelFilter::Info, config) {
        println!("Cannot initialize the logger: {}", error);
    }

    let client = MessageClient::new();

    connect_stream!(extension, connect_page_created(_, page), client, PageCreated(page.clone()));

    // Don't drop the client to keep receiving the messages on the stream.
    forget(client);
}
