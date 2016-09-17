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

use std::env;
use std::fs::OpenOptions;
use std::process::Command;
use std::rc::Rc;

use gtk;
use mg::Application;
use mg_settings;
use xdg::BaseDirectories;
use webkit2::LoadEvent::Started;

use self::AppCommand::*;
use webview::WebView;

const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

commands!(AppCommand {
    Back,
    Forward,
    Insert,
    Normal,
    Open(String),
    Quit,
    Reload,
    Reloadbypasscache,
    Stop,
    Winopen(String),
});

macro_rules! unwrap_or_show_error {
    ($app:expr, $error:expr) => {
        if let Err(error) = $error {
            $app.error(&error.to_string());
        }
    };
}

/// Titanium application.
pub struct App {
    app: Rc<Application<AppCommand>>,
    webview: Rc<WebView>,
}

impl App {
    pub fn new(homepage: Option<String>) -> Rc<Self> {
        let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
        let config_path = xdg_dirs.place_config_file("config")
            .expect("cannot create configuration directory");

        let mg_app = Application::new_with_config(hash! {
            "i" => "insert",
        });
        mg_app.use_dark_theme();
        mg_app.set_window_title(APP_NAME);

        let url_label = mg_app.add_statusbar_item();

        unwrap_or_show_error!(mg_app, OpenOptions::new().create(true).write(true).open(&config_path));
        unwrap_or_show_error!(mg_app, mg_app.parse_config(config_path));

        let url = homepage.unwrap_or("https://duckduckgo.com/lite/".to_string());
        let webview = WebView::new();
        webview.open(&url);
        mg_app.set_view(&webview);

        let app = Rc::new(App {
            app: mg_app,
            webview: Rc::new(webview),
        });

        {
            let app = app.clone();
            let webview = app.webview.clone();
            webview.connect_load_changed(move |webview, load_event| {
                if load_event == Started {
                    app.app.set_mode("normal");
                }

                if let Some(url) = webview.get_uri() {
                    url_label.set_text(&url);
                }

                app.set_title();
            });
        }
        {
            let app = app.clone();
            let webview = app.webview.clone();
            webview.connect_resource_load_started(move |_, _, _| {
                app.set_title();
            });
        }

        {
            let app = app.clone();
            let mg_app = app.app.clone();
            mg_app.connect_command(move |command| {
                app.handle_command(command);
            });
        }

        {
            let app = app.clone();
            let mg_app = app.app.clone();
            mg_app.add_variable("url", move || {
                app.webview.get_uri().unwrap()
            });
        }

        app
    }

    /// Handle the command.
    fn handle_command(&self, command: AppCommand) {
        match command {
            Back => self.webview.go_back(),
            Forward => self.webview.go_forward(),
            Insert => self.app.set_mode("insert"),
            Normal => self.app.set_mode("normal"),
            Open(url) => self.webview.open(&url),
            Quit => gtk::main_quit(),
            Reload => self.webview.reload(),
            Reloadbypasscache => self.webview.reload_bypass_cache(),
            Stop => self.webview.stop_loading(),
            Winopen(url) => self.open_in_new_window(&url),
        }
    }

    /// Open the given URL in a new window.
    fn open_in_new_window(&self, url: &str) {
        let program = env::args().next().unwrap();
        unwrap_or_show_error!(self.app,
            Command::new(program)
                .arg(url)
                .spawn()
        );
    }

    /// Set the title of the window as the progress and the web page title.
    fn set_title(&self) {
        let progress = (self.webview.get_estimated_load_progress() * 100.0) as i32;
        if let Some(title) = self.webview.get_title() {
            if progress == 100 {
                self.app.set_window_title(&format!("{} - {}", title, APP_NAME));
            }
            else {
                self.app.set_window_title(&format!("[{}%] {} - {}", progress, title, APP_NAME));
            }
        }
    }
}
