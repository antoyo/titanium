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

use std::cell::RefCell;
use std::char;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::rc::Rc;

use gdk::EventKey;
use gtk::{self, ContainerExt, Inhibit};
use gtk::Orientation::Vertical;
use mg::{Application, ApplicationBuilder, StatusBarItem};
use xdg::BaseDirectories;
use webkit2gtk::ScriptDialog;
use webkit2gtk::LoadEvent::{Finished, Started};
use webkit2gtk::NavigationType::Other;
use webkit2gtk::ScriptDialogType::{Alert, BeforeUnloadConfirm, Confirm, Prompt};

use commands::{AppCommand, SpecialCommand};
use commands::AppCommand::*;
use commands::SpecialCommand::*;
use download_list_view::DownloadListView;
use glib_user_dir::{get_user_special_dir, G_USER_DIRECTORY_DOWNLOAD};
use popup_manager::PopupManager;
use settings::AppSettings;
use urls::{get_base_url, is_url};
use webview::WebView;

pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

pub type AppResult = Result<(), Box<Error>>;
type MgApp = Rc<Application<SpecialCommand, AppCommand, AppSettings>>;

/// Titanium application.
pub struct App {
    app: MgApp,
    default_search_engine: Rc<RefCell<Option<String>>>,
    download_list_view: Rc<RefCell<DownloadListView>>,
    popup_manager: Rc<RefCell<PopupManager>>,
    scroll_label: Rc<StatusBarItem>,
    search_engines: Rc<RefCell<HashMap<String, String>>>,
    url_label: StatusBarItem,
    webview: Rc<WebView>,
}

impl App {
    fn build() -> MgApp {
        let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
        let include_path = xdg_dirs.get_config_home();

        ApplicationBuilder::new()
            .include_path(include_path)
            .modes(hash! {
                "f" => "follow",
                "i" => "insert",
            })
            .settings(AppSettings::new())
            .build()
    }

    pub fn new(homepage: Option<String>) -> Rc<Self> {
        let mg_app = App::build();
        mg_app.use_dark_theme();
        mg_app.set_window_title(APP_NAME);

        let scroll_label = Rc::new(mg_app.add_statusbar_item());
        scroll_label.set_text("[top]");

        let url_label = mg_app.add_statusbar_item();

        let vbox = gtk::Box::new(Vertical, 0);

        let download_list_view = DownloadListView::new();
        vbox.add(&download_list_view);

        let webview = WebView::new();
        vbox.add(&*webview);

        mg_app.set_view(&vbox);

        let app = Rc::new(App {
            app: mg_app,
            default_search_engine: Rc::new(RefCell::new(None)),
            download_list_view: Rc::new(RefCell::new(download_list_view)),
            popup_manager: Rc::new(RefCell::new(PopupManager::new())),
            scroll_label: scroll_label,
            search_engines: Rc::new(RefCell::new(HashMap::new())),
            url_label: url_label,
            webview: webview,
        });

        App::create_events(&app);

        // Create the events before parsing the config so that the settings are taken into account
        // and the commands are executed.
        app.parse_config();

        let url = homepage.unwrap_or(app.app.settings().home_page.clone());
        app.webview.open(&url);

        app.handle_error((*app.popup_manager.borrow_mut()).load());

        App::create_variables(app.clone());

        app
    }

    /// Add a search engine.
    fn add_search_engine(&self, args: &str) {
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

    /// Ask to the user whether to open the popup or not (with option to whitelist or blacklist).
    fn ask_open_popup(app: &Rc<App>, url: String, base_url: String) {
        let instance = app.clone();
        app.app.question(&format!("A popup from {} was blocked. Do you want to open it?", base_url),
        &['y', 'n', 'a', 'e'], move |answer|
        {
            match answer {
                Some('a') => {
                    instance.open_in_new_window_handling_error(&url);
                    instance.whitelist_popup(&url);
                },
                Some('y') => instance.open_in_new_window_handling_error(&url),
                Some('e') => {
                    instance.blacklist_popup(&url);
                },
                _ => (),
            }
        });
    }

    /// Save the specified url in the popup blacklist.
    fn blacklist_popup(&self, url: &str) {
        self.handle_error((*self.popup_manager.borrow_mut()).blacklist(url));
    }

    /// Create the default configuration files and directories if it does not exist.
    fn create_config_files(&self, config_path: &Path) -> AppResult {
        let xdg_dirs = BaseDirectories::with_prefix(APP_NAME)?;

        let stylesheets_path = xdg_dirs.place_config_file("stylesheets")?;
        let scripts_path = xdg_dirs.place_config_file("scripts")?;
        create_dir_all(stylesheets_path)?;
        create_dir_all(scripts_path)?;

        let keys_path = xdg_dirs.place_config_file("keys")?;
        let webkit_config_path = xdg_dirs.place_config_file("webkit")?;
        let hints_css_path = xdg_dirs.place_config_file("stylesheets/hints.css")?;
        self.create_default_config_file(config_path, include_str!("../config/config"))?;
        self.create_default_config_file(&keys_path, include_str!("../config/keys"))?;
        self.create_default_config_file(&webkit_config_path, include_str!("../config/webkit"))?;
        self.create_default_config_file(&hints_css_path, include_str!("../config/stylesheets/hints.css"))?;

        let (popup_whitelist_path, popup_blacklist_path) = PopupManager::config_path();
        OpenOptions::new().create(true).write(true).open(&popup_whitelist_path)?;
        OpenOptions::new().create(true).write(true).open(&popup_blacklist_path)?;

        Ok(())
    }

    /// Create the config file with its default content if it does not exist.
    fn create_default_config_file(&self, path: &Path, content: &'static str) -> AppResult {
        if !path.exists() {
            let mut file = File::create(path)?;
            write!(file, "{}", content)?;
        }
        Ok(())
    }

    /// Create the events.
    fn create_events(app: &Rc<Self>) {
        {
            let webview = app.webview.clone();
            app.app.connect_setting_changed(move |setting| {
                webview.setting_changed(&setting);
            });
        }

        {
            let application = app.clone();
            app.app.connect_command(move |command| {
                application.handle_command(command);
            });
        }

        App::handle_load_changed(app.clone());

        {
            let application = app.clone();
            app.webview.connect_resource_load_started(move |_, _, _| {
                application.set_title();
            });
        }

        {
            let application = app.clone();
            app.webview.connect_scrolled(move |scroll_percentage| {
                application.show_scroll(scroll_percentage);
            });
        }

        App::handle_create(app.clone());

        {
            let application = app.clone();
            app.app.connect_special_command(move |command| {
                application.handle_special_command(command);
            });
        }

        {
            let application = app.clone();
            app.app.connect_key_press_event(move |_, event_key| {
                application.handle_key_press(event_key)
            });
        }

        {
            let application = app.clone();
            app.webview.connect_script_dialog(move |_, script_dialog| {
                application.handle_script_dialog(script_dialog.clone());
                true
            });
        }

        {
            let application = app.clone();
            app.webview.connect_download_started(move |_, download| {
                (*application.download_list_view.borrow_mut()).add(download);
            });
        }

        {
            let application = app.clone();
            app.webview.connect_new_window(move |url| {
                application.handle_error(application.open_in_new_window(url));
            });
        }


        App::handle_decide_destination(app.clone());

        {
            let application = app.clone();
            app.app.connect_close(move || {
                application.quit();
            });
        }

        app.webview.connect_close(|_| {
            // TODO: if there are active downloads, ask confirmation before exit.
            gtk::main_quit();
        });
    }

    /// Create the variables accessible from the config files.
    fn create_variables(app: Rc<Self>) {
        let application = app.clone();
        app.app.add_variable("url", move || {
            application.webview.get_uri().unwrap()
        });
    }

    /// Get the title or the url if there are no title.
    fn get_title(&self) -> String {
        let title = self.webview.get_title()
            .or(self.webview.get_uri())
            .unwrap_or_default();
        if title.is_empty() {
            String::new()
        }
        else {
            format!("{} - ", title)
        }
    }

    /// Handle the command.
    fn handle_command(&self, command: AppCommand) {
        match command {
            ActivateSelection => self.handle_error(self.webview.activate_selection()),
            Back => self.webview.go_back(),
            FinishSearch => self.webview.finish_search(),
            Follow => {
                self.app.set_mode("follow");
                self.handle_error(self.webview.follow_link(&self.app.settings().hint_chars))
            },
            Forward => self.webview.go_forward(),
            HideHints => self.hide_hints(),
            Insert => self.app.set_mode("insert"),
            Inspector => self.webview.show_inspector(),
            Normal => self.app.set_mode("normal"),
            Open(url) => self.open(&url),
            Quit => self.quit(),
            Reload => self.webview.reload(),
            ReloadBypassCache => self.webview.reload_bypass_cache(),
            ScrollBottom => self.handle_error(self.webview.scroll_bottom()),
            ScrollDown => self.handle_error(self.webview.scroll_down_page()),
            ScrollDownHalf => self.handle_error(self.webview.scroll_down_half_page()),
            ScrollDownLine => self.handle_error(self.webview.scroll_down_line()),
            ScrollTop => self.handle_error(self.webview.scroll_top()),
            ScrollUp => self.handle_error(self.webview.scroll_up_page()),
            ScrollUpHalf => self.handle_error(self.webview.scroll_up_half_page()),
            ScrollUpLine => self.handle_error(self.webview.scroll_up_line()),
            SearchEngine(args) => self.add_search_engine(&args),
            SearchNext => self.webview.search_next(),
            SearchPrevious => self.webview.search_previous(),
            Stop => self.webview.stop_loading(),
            WinFollow => {
                self.webview.set_open_in_new_window();
                self.app.set_mode("follow");
                self.handle_error(self.webview.follow_link(&self.app.settings().hint_chars))
            },
            WinOpen(url) => self.handle_error(self.open_in_new_window(&url)),
            ZoomIn => self.zoom_in(),
            ZoomNormal => self.zoom_normal(),
            ZoomOut => self.zoom_out(),
        }
    }

    /// Handle create window.
    fn handle_create(app: Rc<App>) {
        let webview = app.webview.clone();
        webview.connect_create(move |_, action| {
            if let Some(request) = action.get_request() {
                if let Some(url) = request.get_uri() {
                    // Block popup.
                    let popup_manager = &*app.popup_manager.borrow();
                    let base_url = get_base_url(&url);
                    if action.get_navigation_type() == Other && !popup_manager.is_whitelisted(&url) {
                        if popup_manager.is_blacklisted(&url) {
                            Application::warning(&app.app, &format!("Not opening popup from {} since it is blacklisted.", base_url));
                        }
                        else {
                            App::ask_open_popup(&app, url, base_url);
                        }
                    }
                    else {
                        app.open_in_new_window_handling_error(&url);
                    }
                }
            }
            None
        });
    }

    /// Handle the download decide destination event.
    fn handle_decide_destination(app: Rc<App>) {
        let application = app.clone();
        (*app.download_list_view.borrow_mut()).connect_decide_destination(move |download, suggested_filename| {
            let default_path = format!("{}/", get_user_special_dir(G_USER_DIRECTORY_DOWNLOAD));
            let destination = application.app.blocking_input("Save file to: (<C-x> to open)", &default_path);
            if let Some(destination) = destination {
                let path = Path::new(&destination);
                let download_destination =
                    if path.is_dir() {
                        path.join(suggested_filename)
                    }
                    else {
                        path.to_path_buf()
                    };
                let exists = download_destination.exists();
                let download_destination = download_destination.to_str().unwrap();
                if exists {
                    let message = &format!("Do you want to overwrite {}?", download_destination);
                    let answer = application.app.blocking_yes_no_question(message);
                    if answer {
                        download.set_allow_overwrite(true);
                    }
                    else {
                        download.cancel();
                    }
                }
                download.set_destination(&format!("file://{}", download_destination));
            }
            else {
                download.cancel();
            }
            true
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
                app.webview.finish_search();
                app.handle_error(app.webview.add_stylesheets());
                app.handle_error(app.webview.add_scripts());

                // Check to mode to avoid going back to normal mode if the user is in command mode.
                let mode = app.app.get_mode();
                if mode == "insert" || mode == "follow" {
                    app.app.set_mode("normal");
                }
            }

            if let Some(url) = webview.get_uri() {
                app.url_label.set_text(&url);
            }

            if load_event == Finished {
                app.set_title_without_progress();
            }
            else {
                app.set_title();
            }
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

    /// Open the given URL in the web view.
    fn open(&self, url: &str) {
        let url = self.transform_url(url);
        self.webview.open(&url);
    }

    /// Open the given URL in a new window.
    fn open_in_new_window(&self, url: &str) -> AppResult {
        let url = self.transform_url(url);
        let program = env::args().next().unwrap();
        Command::new(program)
            .arg(url)
            .spawn()?;
        Ok(())
    }

    /// Open the given URL in a new window, showing the error if any.
    fn open_in_new_window_handling_error(&self, url: &str) {
        self.handle_error(self.open_in_new_window(url));
    }


    /// Create the missing config files and parse the config files.
    fn parse_config(&self) {
        let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
        let config_path = xdg_dirs.place_config_file("config")
            .expect("cannot create configuration directory");
        self.handle_error(self.create_config_files(config_path.as_path()));
        self.handle_error(self.app.parse_config(config_path));
    }

    /// Try to close the web view and quit the application.
    fn quit(&self) {
        self.webview.try_close();
    }

    /// Set the title of the window as the progress and the web page title.
    fn set_title(&self) {
        let progress = (self.webview.get_estimated_load_progress() * 100.0) as i32;
        if progress == 100 {
            self.set_title_without_progress();
        }
        else {
            let title = self.get_title();
            self.app.set_window_title(&format!("[{}%] {}{}", progress, title, APP_NAME));
        }
    }

    /// Set the title of the window as the web page title or url.
    fn set_title_without_progress(&self) {
        let title = self.get_title();
        self.app.set_window_title(&format!("{}{}", title, APP_NAME));
    }

    /// Show the error in the status bar.
    fn show_error(&self, error: Box<Error>) {
        // Remove the quotes around the error string since DBus message contains quotes.
        self.app.error(error.to_string().trim_matches('"'));
    }

    /// Show the scroll percentage.
    fn show_scroll(&self, scroll_percentage: i64) {
        let text =
            match scroll_percentage {
                -1 => "[all]".to_string(),
                0 => "[top]".to_string(),
                100 => "[bot]".to_string(),
                _ => format!("[{}%]", scroll_percentage),
            };
        self.scroll_label.set_text(&text);
    }

    /// Show the zoom level in the status bar.
    fn show_zoom(&self, level: i32) {
        Application::info(&self.app, &format!("Zoom level: {}%", level));
    }

    /// If the url starts with a search engine keyword, transform the url to the URL of the search
    /// engine.
    fn transform_url(&self, url: &str) -> String {
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

    /// Save the specified url in the popup whitelist.
    fn whitelist_popup(&self, url: &str) {
        self.handle_error((*self.popup_manager.borrow_mut()).whitelist(url));
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
