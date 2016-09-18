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

use gdk::enums::key::Escape;
use glib::object::Downcast;
use gtk::{self, Inhibit, WidgetExt};
use mg::{Application, StatusBarItem};
use mg_settings;
use xdg::BaseDirectories;
use webkit2::{FindController, NavigationPolicyDecision, WebViewExt, FIND_OPTIONS_CASE_INSENSITIVE, FIND_OPTIONS_WRAP_AROUND};
use webkit2::LoadEvent::Started;
use webkit2::PolicyDecisionType::NewWindowAction;

use self::AppCommand::*;
use self::SpecialCommand::*;
use webview::WebView;

const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
const SCROLL_LINE_VERTICAL: i32 = 40;

commands!(AppCommand {
    Back,
    Forward,
    Insert,
    Normal,
    Open(String),
    Quit,
    Reload,
    Reloadbypasscache,
    Scrolldown,
    Scrolldownhalf,
    Scrolldownline,
    Scrollup,
    Scrolluphalf,
    Scrollupline,
    Searchnext,
    Searchprevious,
    Stop,
    Winopen(String),
});

special_commands!(SpecialCommand {
    Search('/', always),
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
    app: Rc<Application<SpecialCommand, AppCommand>>,
    find_controller: FindController,
    url_label: StatusBarItem,
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

        let webview = Rc::new(webview);

        let find_controller = {
            let webview = webview.clone();
            webview.get_find_controller().unwrap()
        };

        let app = Rc::new(App {
            app: mg_app,
            find_controller: find_controller,
            url_label: url_label,
            webview: webview,
        });

        {
            let app = app.clone();
            App::handle_load_changed(app);
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
            App::handle_decisions(app);
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
            let mg_app2 = app.app.clone();
            mg_app.window().connect_key_press_event(move |_, key| {
                if key.get_keyval() == Escape && mg_app2.get_mode() == "normal" {
                    app.finish_search();
                    app.clear_selection();
                }
                Inhibit(false)
            });
        }

        {
            let app = app.clone();
            let mg_app = app.app.clone();
            mg_app.connect_special_command(move |command| {
                app.handle_special_command(command);
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

    /// Clear the selection.
    fn clear_selection(&self) {
        let webview = self.webview.clone();
        webview.run_javascript("window.getSelection().empty();");
    }

    /// Clear the current search.
    fn finish_search(&self) {
        self.find_controller.search_finish();
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
            Scrolldown => self.scroll_down_page(),
            Scrolldownhalf => self.scroll_down_half_page(),
            Scrolldownline => self.scroll_down_line(),
            Scrollup => self.scroll_up_page(),
            Scrolluphalf => self.scroll_up_half_page(),
            Scrollupline => self.scroll_up_line(),
            Searchnext => self.search_next(),
            Searchprevious => self.search_previous(),
            Stop => self.webview.stop_loading(),
            Winopen(url) => self.open_in_new_window(&url),
        }
    }

    /// Handle policy decisions like opening new windows.
    fn handle_decisions(app: Rc<App>) {
        let webview = app.webview.clone();
        webview.connect_decide_policy(move |_, policy_decision, policy_decision_type| {
            match policy_decision_type {
                NewWindowAction => {
                    let decision: Result<NavigationPolicyDecision, _> = policy_decision.clone().downcast();
                    let url =
                        decision.ok()
                        .and_then(|decision| decision.get_request())
                        .and_then(|request| request.get_uri());
                    if let Some(url) = url {
                        app.open_in_new_window(&url);
                        return true;
                    }
                    false
                },
                _ => false,
            }
        });
    }

    /// Handle the load_changed event.
    /// Show the URL.
    /// Set the window title.
    /// Go back to normal mode.
    fn handle_load_changed(app: Rc<App>) {
        let webview = app.webview.clone();
        webview.connect_load_changed(move |webview, load_event| {
            if load_event == Started {
                app.app.set_mode("normal");
            }

            if let Some(url) = webview.get_uri() {
                app.url_label.set_text(&url);
            }

            app.set_title();
        });
    }

    /// Handle the special command.
    fn handle_special_command(&self, command: SpecialCommand) {
        match command {
            Search(input) => {
                let options = FIND_OPTIONS_CASE_INSENSITIVE | FIND_OPTIONS_WRAP_AROUND;
                self.find_controller.search("", options.bits(), ::std::u32::MAX); // Clear previous search.
                self.find_controller.search(&input, options.bits(), ::std::u32::MAX);
            },
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

    /// Scroll by the specified number of pixels.
    fn scroll(&self, pixels: i32) {
        self.webview.run_javascript(&format!("window.scrollBy(0, {});", pixels));
    }

    /// Scroll down by one line.
    fn scroll_down_line(&self) {
        self.scroll(SCROLL_LINE_VERTICAL);
    }

    /// Scroll down by one half of page.
    fn scroll_down_half_page(&self) {
        let allocation = self.webview.get_allocation();
        self.scroll(allocation.height / 2);
    }

    /// Scroll down by one page.
    fn scroll_down_page(&self) {
        let allocation = self.webview.get_allocation();
        self.scroll(allocation.height);
    }

    /// Scroll up by one line.
    fn scroll_up_line(&self) {
        self.scroll(-SCROLL_LINE_VERTICAL);
    }

    /// Scroll up by one half of page.
    fn scroll_up_half_page(&self) {
        let allocation = self.webview.get_allocation();
        self.scroll(-allocation.height / 2);
    }

    /// Scroll up by one page.
    fn scroll_up_page(&self) {
        let allocation = self.webview.get_allocation();
        self.scroll(-allocation.height);
    }

    /// Search the next occurence of the search text.
    fn search_next(&self) {
        self.find_controller.search_next();
    }

    /// Search the previous occurence of the search text.
    fn search_previous(&self) {
        self.find_controller.search_previous();
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
