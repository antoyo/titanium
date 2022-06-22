/*
 * Copyright (c) 2016-2020 Boucher, Antoni <bouanto@zoho.com>
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

//! Message server interface.

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::{self, Write};
use std::process;

use gtk::{
    self,
    traits::DialogExt,
    ButtonsType,
    DialogFlags,
    MessageDialog,
    MessageType,
    Window,
};
use relm::{Component, EventStream, Relm, Update, UpdateNew, execute, init};
use webkit2gtk::WebContext;

use titanium_common::{InnerMessage, PageId};
use titanium_common::InnerMessage::Open;

use app::App;
use app::Msg::{
    ChangeUrl,
    CreateWindow,
    Remove,
};
use config_dir::ConfigDir;
use errors::Result;
use self::Msg::*;
use webview::WebView;

#[derive(Clone, Copy, PartialEq)]
pub enum Privacy {
    Normal,
    Private,
}

pub struct MessageServer {
    model: Model,
}

pub struct Model {
    app_count: usize,
    config_dir: ConfigDir,
    /// This listener is used to prevent two instances of Titanium to run at the same time.
    private_web_context: WebContext,
    opened_urls: BTreeSet<String>,
    previous_opened_urls: BTreeSet<String>,
    relm: Relm<MessageServer>,
    // TODO: save the widgets somewhere allowing to remove them when its window is closed.
    wins: Vec<Component<App>>,
    web_context: WebContext,
}

#[derive(Msg)]
pub enum Msg {
    ChangeOpenedPage(String, String),
    NewApp(Option<String>, Privacy),
    RemoveApp(PageId, String),
}

impl Update for MessageServer {
    type Model = Model;
    type ModelParam = (Vec<String>, Option<String>);
    type Msg = Msg;

    fn model(relm: &Relm<Self>, (urls, config): (Vec<String>, Option<String>)) -> Model {
        let config_dir = ConfigDir::new(&config).unwrap(); // TODO: remove unwrap().
        let (web_context, private_web_context) = WebView::initialize_web_extension(&config_dir);
        if urls.is_empty() {
            relm.stream().emit(NewApp(None, Privacy::Normal));
        }
        else {
            for url in urls {
                relm.stream().emit(NewApp(Some(url), Privacy::Normal));
            }
        }
        Model {
            app_count: 0,
            config_dir,
            opened_urls: BTreeSet::new(),
            previous_opened_urls: BTreeSet::new(),
            private_web_context,
            relm: relm.clone(),
            wins: vec![],
            web_context,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            ChangeOpenedPage(old, new) => {
                self.model.opened_urls.remove(&old);
                self.model.opened_urls.insert(new);
                self.save_urls();
            },
            NewApp(url, privacy) => self.add_app(url, privacy),
            RemoveApp(page_id, url) => self.remove_app(page_id, url),
        }
    }
}

impl UpdateNew for MessageServer {
    fn new(_relm: &Relm<Self>, model: Self::Model) -> Self {
        MessageServer {
            model,
        }
    }
}

impl MessageServer {
    pub fn new(url: Vec<String>, config_dir: Option<String>) -> Result<EventStream<<Self as Update>::Msg>> {
        Ok(execute::<MessageServer>((url, config_dir)))
    }

    fn add_app(&mut self, url: Option<String>, privacy: Privacy) {
        self.model.app_count += 1;
        let web_context =
            if privacy == Privacy::Private {
                self.model.private_web_context.clone()
            }
            else {
                self.model.web_context.clone()
            };

        self.load_opened_urls();

        if let Some(url) = url.as_ref() {
            self.model.opened_urls.insert(url.clone());
            self.save_urls();
        }

        let app = init::<App>((url, self.model.config_dir.clone(), web_context, self.model.previous_opened_urls.clone())).unwrap(); // TODO: remove unwrap().
        connect!(app@CreateWindow(ref url, ref privacy), self.model.relm, NewApp(Some(url.clone()), *privacy));
        connect!(app@Remove(page_id, ref url), self.model.relm, RemoveApp(page_id, url.clone()));
        connect!(app@ChangeUrl(ref old, ref new), self.model.relm, ChangeOpenedPage(old.clone(), new.clone()));
        self.model.wins.push(app);
    }

    fn load_opened_urls(&mut self) {
        let mut restore = || -> io::Result<()> {
            let filename = self.model.config_dir.data_file("urls")?;
            let file = BufReader::new(File::open(filename)?);
            for line in file.lines() {
                let url = line?;
                self.model.opened_urls.insert(url.clone());
                self.model.previous_opened_urls.insert(url.clone());
            }

            Ok(())
        };

        if let Err(error) = restore() {
            error!("Load opened urls error: {}", error);
        }
    }

    fn msg_recv(&mut self, protocol_counter: usize, page_id: PageId, msg: InnerMessage) {
        trace!("Receive message");
        if let Open(urls) = msg {
            if urls.is_empty() {
                self.add_app(None, Privacy::Normal);
            }
            else {
                for url in urls {
                    self.add_app(Some(url), Privacy::Normal);
                }
            }
        }
        else {
            error!("Cannot find app with page id {}", page_id);
        }
    }

    fn remove_app(&mut self, page_id: PageId, url: String) {
        self.model.opened_urls.remove(&url);
        self.save_urls();

        self.model.app_count -= 1;
        // TODO: remove from self.model.wins.
        if self.model.app_count == 0 {
            self.model.opened_urls.clear();
            self.save_urls();
            gtk::main_quit(); // FIXME: can't call main_quit() with gtk::Application.
        }
    }

    fn save_urls(&self) {
        let save = || -> io::Result<()> {
            let filename = self.model.config_dir.data_file("urls")?;
            let mut file = File::create(filename)?;
            for url in &self.model.opened_urls {
                writeln!(file, "{}", url)?;
            }

            Ok(())
        };

        if let Err(error) = save() {
            error!("Cannot save opened urls: {}", error);
        }
    }
}

/// Create a new message server.
/// If it is not possible to create one, show the error and exit.
pub fn create_message_server(urls: Vec<String>, config_dir: Option<String>) -> EventStream<<MessageServer as Update>::Msg> {
    match MessageServer::new(urls, config_dir) {
        Ok(message_server) => message_server,
        Err(error) => {
            let message = format!("cannot create the message server used to communicate with the web processes: {}",
                error);
            dialog_and_exit(&message);
        },
    }
}

fn dialog_and_exit(message: &str) -> ! {
    let window: Option<&Window> = None;
    let message = format!("Fatal error: {}", message);
    let dialog = MessageDialog::new(window, DialogFlags::empty(), MessageType::Error, ButtonsType::Close, &message);
    dialog.run();
    process::exit(1);
}
