/*
 * Copyright (c) 2017-2022 Boucher, Antoni <bouanto@zoho.com>
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

use relm::{
    EventStream,
    Relm,
    Update,
    UpdateNew,
    connect,
    connect_stream,
    execute,
};
use webkit2gtk_webextension::{
    traits::{URIRequestExt, UserMessageExt, WebPageExt},
    URIRequest,
    UserMessage,
    WebPage,
};

use titanium_common::protocol::decode;

use adblocker::Adblocker;
use executor::{self, Executor};
use executor::Msg::{DocumentLoaded, MessageRecv};
use self::Msg::*;

thread_local! {
    static ADBLOCKER: Adblocker = Adblocker::new();
}

pub struct MessageClient {
    model: Model,
}

pub struct Model {
    executors: Vec<EventStream<<Executor as Update>::Msg>>,
    relm: Relm<MessageClient>,
}

#[derive(Msg)]
pub enum Msg {
    PageCreated(WebPage),
}

impl Update for MessageClient {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(relm: &Relm<Self>, (): ()) -> Model {
        Model {
            executors: vec![],
            relm: relm.clone(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            PageCreated(page) => {
                // TODO: this should be disconnected later somehow.
                connect!(self.model.relm, page, connect_send_request(_, request, _),
                    return block_request(request));
                let executor = execute::<Executor>(page.clone());
                connect_stream!(page, connect_document_loaded(_), executor, DocumentLoaded);
                connect_stream!(return executor, page, connect_user_message_received(_, msg), (message_recv(msg), true));
                self.model.executors.push(executor);
                // TODO: remove from the executor when the page is destroyed?
            },
        }
    }
}

fn message_recv(msg: &UserMessage) -> Option<executor::Msg> {
    match decode(&msg.parameters()) {
        Ok(msg) => Some(MessageRecv(msg)),
        Err(error) => {
            error!("{}", error);
            None
        },
    }
}

impl UpdateNew for MessageClient {
    fn new(_relm: &Relm<Self>, model: Model) -> Self {
        MessageClient {
            model,
        }
    }
}

impl MessageClient {
    pub fn new() -> EventStream<<Self as Update>::Msg> {
        execute::<MessageClient>(())
    }
}

fn block_request(request: &URIRequest) -> bool {
    if let Some(url) = request.uri() {
        return ADBLOCKER.with(|adblocker| adblocker.should_block(&url));
    }
    false
}
