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

use std::char;
use std::env;
use std::error::Error;
use std::fs::{OpenOptions, create_dir_all};
use std::path::Path;
use std::process::Command;
use std::rc::Rc;

use gdk::EventKey;
use gtk::{self, Inhibit};
use mg::{Application, StatusBarItem};
use mg_settings;
use xdg::BaseDirectories;
use webkit2gtk::ScriptDialog;
use webkit2gtk::LoadEvent::Started;
use webkit2gtk::ScriptDialogType::{Alert, BeforeUnloadConfirm, Confirm, Prompt};

use self::AppCommand::*;
use self::SpecialCommand::*;
use webview::WebView;

pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

pub type AppResult = Result<(), Box<Error>>;

commands!(AppCommand {
    Activateselection,
    Back,
    Finishsearch,
    Follow,
    Forward,
    Hidehints,
    Insert,
    Inspector,
    Normal,
    Open(String),
    Quit,
    Reload,
    Reloadbypasscache,
    Scrollbottom,
    Scrolldown,
    Scrolldownhalf,
    Scrolldownline,
    Scrolltop,
    Scrollup,
    Scrolluphalf,
    Scrollupline,
    Searchnext,
    Searchprevious,
    Stop,
    Winopen(String),
    Zoomin,
    Zoomnormal,
    Zoomout,
});

special_commands!(SpecialCommand {
    BackwardSearch('?', always),
    Search('/', always),
});

/// Titanium application.
pub struct App {
    app: Rc<Application<SpecialCommand, AppCommand>>,
    scroll_label: Rc<StatusBarItem>,
    url_label: StatusBarItem,
    webview: Rc<WebView>,
}

impl App {
    pub fn new(homepage: Option<String>) -> Rc<Self> {
        let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
        let config_path = xdg_dirs.place_config_file("config")
            .expect("cannot create configuration directory");

        let mg_app = Application::new_with_config(hash! {
            "f" => "follow",
            "i" => "insert",
        });
        mg_app.use_dark_theme();
        mg_app.set_window_title(APP_NAME);

        let scroll_label = Rc::new(mg_app.add_statusbar_item());
        scroll_label.set_text("[top]");

        let url_label = mg_app.add_statusbar_item();

        let url = homepage.unwrap_or("https://duckduckgo.com/lite/".to_string());
        let webview = WebView::new();
        webview.open(&url);
        mg_app.set_view(&*webview);

        let app = Rc::new(App {
            app: mg_app,
            scroll_label: scroll_label,
            url_label: url_label,
            webview: webview,
        });

        app.handle_error(app.create_config_files(config_path.as_path()));
        app.handle_error(app.app.parse_config(config_path));

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
            let webview = app.webview.clone();
            let scroll_label = app.scroll_label.clone();
            webview.connect_scrolled(move |scroll_percentage| {
                let text =
                    match scroll_percentage {
                        -1 => "[all]".to_string(),
                        0 => "[top]".to_string(),
                        100 => "[bot]".to_string(),
                        _ => format!("[{}%]", scroll_percentage),
                    };
                scroll_label.set_text(&text);
            });
        }

        {
            let app = app.clone();
            App::handle_create(app);
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

        {
            let app = app.clone();
            let mg_app = app.app.clone();
            mg_app.connect_key_press_event(move |_, event_key| {
                app.handle_key_press(event_key)
            });
        }

        {
            let app = app.clone();
            let webview = app.webview.clone();
            webview.connect_script_dialog(move |_, script_dialog| {
                app.handle_script_dialog(script_dialog.clone());
                true
            });
        }

        {
            let app = app.clone();
            let mg_app = app.app.clone();
            mg_app.connect_close(move || {
                app.quit();
            });
        }

        app.webview.connect_close(|_| {
            gtk::main_quit();
        });

        app
    }

    /// Create the default configuration files and directories if it does not exist.
    fn create_config_files(&self, config_path: &Path) -> AppResult {
        try!(OpenOptions::new().create(true).write(true).open(&config_path));
        let xdg_dirs = try!(BaseDirectories::with_prefix(APP_NAME));
        let stylesheets_path = try!(xdg_dirs.place_config_file("stylesheets"));
        let scripts_path = try!(xdg_dirs.place_config_file("scripts"));
        try!(create_dir_all(stylesheets_path));
        try!(create_dir_all(scripts_path));
        Ok(())
    }

    /// Handle the command.
    fn handle_command(&self, command: AppCommand) {
        match command {
            Activateselection => self.handle_error(self.webview.activate_selection()),
            Back => self.webview.go_back(),
            Finishsearch => self.webview.finish_search(),
            Follow => {
                self.app.set_mode("follow");
                self.handle_error(self.webview.follow_link())
            },
            Forward => self.webview.go_forward(),
            Hidehints => self.hide_hints(),
            Insert => self.app.set_mode("insert"),
            Inspector => self.webview.show_inspector(),
            Normal => self.app.set_mode("normal"),
            Open(url) => self.webview.open(&url),
            Quit => self.quit(),
            Reload => self.webview.reload(),
            Reloadbypasscache => self.webview.reload_bypass_cache(),
            Scrollbottom => self.handle_error(self.webview.scroll_bottom()),
            Scrolldown => self.handle_error(self.webview.scroll_down_page()),
            Scrolldownhalf => self.handle_error(self.webview.scroll_down_half_page()),
            Scrolldownline => self.handle_error(self.webview.scroll_down_line()),
            Scrolltop => self.handle_error(self.webview.scroll_top()),
            Scrollup => self.handle_error(self.webview.scroll_up_page()),
            Scrolluphalf => self.handle_error(self.webview.scroll_up_half_page()),
            Scrollupline => self.handle_error(self.webview.scroll_up_line()),
            Searchnext => self.webview.search_next(),
            Searchprevious => self.webview.search_previous(),
            Stop => self.webview.stop_loading(),
            Winopen(url) => self.handle_error(self.open_in_new_window(&url)),
            Zoomin => self.zoom_in(),
            Zoomnormal => self.zoom_normal(),
            Zoomout => self.zoom_out(),
        }
    }

    /// Handle create window.
    fn handle_create(app: Rc<App>) {
        let webview = app.webview.clone();
        webview.connect_create(move |_, action| {
            if let Some(request) = action.get_request() {
                if let Some(url) = request.get_uri() {
                    app.handle_error(app.open_in_new_window(&url));
                }
            }
            None
        });
    }

    /// Show an error in the result is an error.
    fn handle_error(&self, error: AppResult) {
        if let Err(error) = error {
            self.show_error(error);
        }
    }

    /// In follow mode, send the key to the web process.
    fn handle_follow_key_press(&self, event_key: &EventKey) -> Inhibit {
        if let Some(key_char) = char::from_u32(event_key.get_keyval()) {
            if key_char.is_alphanumeric() {
                if let Some(key_char) = key_char.to_lowercase().next() {
                    match self.webview.enter_hint_key(key_char) {
                        Ok(should_click) => {
                            if should_click {
                                let result = self.webview.activate_hint();
                                self.hide_hints();
                                if let Ok(true) = result {
                                    self.app.set_mode("insert")
                                }
                            }
                        },
                        Err(error) => self.show_error(error),
                    }
                }
            }
        }
        Inhibit(true)
    }

    /// Handle the key press event.
    fn handle_key_press(&self, event_key: &EventKey) -> Inhibit {
        if self.app.get_mode() == "follow" {
            self.handle_follow_key_press(event_key)
        }
        else {
            Inhibit(false)
        }
    }

    /// Handle the load_changed event.
    /// Show the URL.
    /// Set the window title.
    /// Go back to normal mode.
    fn handle_load_changed(app: Rc<App>) {
        let webview = app.webview.clone();
        webview.connect_load_changed(move |webview, load_event| {
            if load_event == Started {
                app.handle_error(app.webview.add_stylesheets());
                app.handle_error(app.webview.add_scripts());
                app.app.set_mode("normal");
            }

            if let Some(url) = webview.get_uri() {
                app.url_label.set_text(&url);
            }

            app.set_title();
        });
    }

    /// Handle the script dialog event.
    fn handle_script_dialog(&self, script_dialog: ScriptDialog) {
        match script_dialog.get_dialog_type() {
            Alert => {
                self.app.message(&format!("[JavaScript] {}", script_dialog.get_message()));
            },
            Confirm => {
                let confirmed = self.app.blocking_yes_no_question(&format!("[JavaScript] {}", script_dialog.get_message()));
                script_dialog.confirm_set_confirmed(confirmed);
            },
            BeforeUnloadConfirm => {
                let confirmed = self.app.blocking_yes_no_question("[JavaScript] Do you really want to leave this page?");
                script_dialog.confirm_set_confirmed(confirmed);
            },
            Prompt => {
                let default_answer = script_dialog.prompt_get_default_text().to_string();
                let input = self.app.blocking_input(&format!("[JavaScript] {}", script_dialog.get_message()), &default_answer);
                let input = input.unwrap_or_default();
                script_dialog.prompt_set_text(&input);
            },
            _ => (),
        }
    }

    /// Handle the special command.
    fn handle_special_command(&self, command: SpecialCommand) {
        match command {
            BackwardSearch(input) => {
                self.webview.set_search_backward(true);
                self.webview.search(&input);
            },
            Search(input) => {
                self.webview.set_search_backward(false);
                self.webview.search(&input);
            },
        }
    }

    /// Hide the hints and return to normal mode.
    fn hide_hints(&self) {
        self.handle_error(self.webview.hide_hints());
        self.app.set_mode("normal");
    }

    /// Open the given URL in a new window.
    fn open_in_new_window(&self, url: &str) -> AppResult {
        let program = env::args().next().unwrap();
        try!(Command::new(program)
            .arg(url)
            .spawn());
        Ok(())
    }

    /// Try to close the web view and quit the application.
    fn quit(&self) {
        self.webview.try_close();
    }

    /// Set the title of the window as the progress and the web page title.
    fn set_title(&self) {
        let progress = (self.webview.get_estimated_load_progress() * 100.0) as i32;
        if let Some(title) = self.webview.get_title() {
            let title =
                if title.is_empty() {
                    String::new()
                }
                else {
                    format!("{} - ", title)
                };
            if progress == 100 {
                self.app.set_window_title(&format!("{}{}", title, APP_NAME));
            }
            else {
                self.app.set_window_title(&format!("[{}%] {}{}", progress, title, APP_NAME));
            }
        }
    }

    /// Show the error in the status bar.
    fn show_error(&self, error: Box<Error>) {
        // Remove the quotes around the error string since DBus message contains quotes.
        self.app.error(error.to_string().trim_matches('"'));
    }

    /// Show the zoom level in the status bar.
    fn show_zoom(&self, level: i32) {
        Application::info(&self.app, &format!("Zoom level: {}%", level));
    }

    /// Zoom in.
    fn zoom_in(&self) {
        self.show_zoom(self.webview.zoom_in());
    }

    /// Zoom back to 100%.
    fn zoom_normal(&self) {
        self.show_zoom(self.webview.zoom_normal());
    }

    /// Zoom out.
    fn zoom_out(&self) {
        self.show_zoom(self.webview.zoom_out());
    }
}
