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

#[macro_use]
extern crate gdbus;
extern crate gio_sys;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate url;
extern crate xdg;
#[macro_use]
extern crate webkit2gtk_webextension;

mod adblocker;
mod dom;
mod hints;
mod scroll;
mod message_server;

use std::cell::Cell;
use std::collections::HashMap;
use std::mem::forget;
use std::panic::catch_unwind;
use std::rc::Rc;

use glib::variant::Variant;
use simplelog::TermLogger;
use simplelog::LogLevelFilter;
use webkit2gtk_webextension::WebExtension;

use adblocker::Adblocker;
use message_server::MessageServer;

web_extension_init!();

pub const APP_NAME: &'static str = "titanium";

#[no_mangle]
pub fn web_extension_initialize(extension: WebExtension, user_data: Variant) {
    catch_unwind(|| {
        let _ = TermLogger::init(LogLevelFilter::max());
    }).ok();

    let current_page_id = Rc::new(Cell::new(0));

    {
        let current_page_id = current_page_id.clone();
        extension.connect_page_created(move |_, page| {
            current_page_id.set(page.get_id());
            let adblocker = Adblocker::new();
            page.connect_send_request(move |_, request, _| {
                if let Some(url) = request.get_uri() {
                    return adblocker.should_block(&url);
                }
                false
            });
        });
    }

    let server_name = user_data.get_str();
    if let Some(server_name) = server_name {
        let mut message_server: MessageServer = MessageServer::new(
            server_name, current_page_id, extension, String::new(), HashMap::new(), None);
        message_server.run("/com/titanium/WebExtensions");
        forget(message_server);
    }
}
