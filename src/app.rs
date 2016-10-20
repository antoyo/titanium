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
use std::env;
use std::error::Error;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::rc::Rc;

use gdk::EventKey;
use gtk::{self, Inhibit};
use mg::{Application, ApplicationBuilder, StatusBarItem};
use xdg::BaseDirectories;
use webkit2gtk::ScriptDialog;
use webkit2gtk::LoadEvent::{Finished, Started};
use webkit2gtk::NavigationType::Other;
use webkit2gtk::ScriptDialogType::{Alert, BeforeUnloadConfirm, Confirm, Prompt};

use popup_manager::PopupManager;
use self::AppCommand::*;
use self::SpecialCommand::*;
use settings::AppSettings;
use urls::get_base_url;
use webview::WebView;

pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

pub type AppResult = Result<(), Box<Error>>;

#[derive(Commands)]
enum AppCommand {
    #[completion(hidden)]
    ActivateSelection,
    #[help(text="Go back in the history")]
    Back,
    #[completion(hidden)]
    FinishSearch,
    #[completion(hidden)]
    Follow,
    #[help(text="Go forward in the history")]
    Forward,
    #[completion(hidden)]
    HideHints,
    #[completion(hidden)]
    Insert,
    #[help(text="Open the web inspector")]
    Inspector,
    #[completion(hidden)]
    Normal,
    #[help(text="Open an URL")]
    Open(String),
    #[help(text="Quit the application")]
    Quit,
    #[help(text="Reload the current page")]
    Reload,
    #[help(text="Reload the current page without using the cache")]
    ReloadBypassCache,
    #[completion(hidden)]
    ScrollBottom,
    #[completion(hidden)]
    ScrollDown,
    #[completion(hidden)]
    ScrollDownHalf,
    #[completion(hidden)]
    ScrollDownLine,
    #[completion(hidden)]
    ScrollTop,
    #[completion(hidden)]
    ScrollUp,
    #[completion(hidden)]
    ScrollUpHalf,
    #[completion(hidden)]
    ScrollUpLine,
    #[completion(hidden)]
    SearchNext,
    #[completion(hidden)]
    SearchPrevious,
    #[help(text="Stop loading the current page")]
    Stop,
    #[help(text="Open an URL in a new window")]
    WinOpen(String),
    #[help(text="Zoom the current page in")]
    ZoomIn,
    #[help(text="Zoom the current page to 100%")]
    ZoomNormal,
    #[help(text="Zoom the current page out")]
    ZoomOut,
}

special_commands!(SpecialCommand {
    BackwardSearch('?', always),
    Search('/', always),
});

/// Titanium application.
pub struct App {
    app: Rc<Application<SpecialCommand, AppCommand, AppSettings>>,
    popup_manager: Rc<RefCell<PopupManager>>,
    scroll_label: Rc<StatusBarItem>,
    url_label: StatusBarItem,
    webview: Rc<WebView>,
}

impl App {
    pub fn new(homepage: Option<String>) -> Rc<Self> {
        let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
        let include_path = xdg_dirs.get_config_home();
        let config_path = xdg_dirs.place_config_file("config")
            .expect("cannot create configuration directory");

        let mg_app = ApplicationBuilder::new()
            .include_path(include_path)
            .modes(hash! {
                "f" => "follow",
                "i" => "insert",
            })
            .settings(AppSettings::new())
            .build();
        mg_app.use_dark_theme();
        mg_app.set_window_title(APP_NAME);

        let scroll_label = Rc::new(mg_app.add_statusbar_item());
        scroll_label.set_text("[top]");

        let url_label = mg_app.add_statusbar_item();

        let webview = WebView::new();
        mg_app.set_view(&*webview);

        let app = Rc::new(App {
            app: mg_app,
            popup_manager: Rc::new(RefCell::new(PopupManager::new())),
            scroll_label: scroll_label,
            url_label: url_label,
            webview: webview,
        });

        {
            let webview = app.webview.clone();
            app.app.connect_setting_changed(move |setting| {
                webview.setting_changed(&setting);
            });
        }

        app.handle_error(app.create_config_files(config_path.as_path()));
        app.handle_error(app.app.parse_config(config_path));

        let url = homepage.unwrap_or(app.app.settings().home_page.clone());
        app.webview.open(&url);

        app.handle_error((*app.popup_manager.borrow_mut()).load());

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
            Open(url) => self.webview.open(&url),
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
            SearchNext => self.webview.search_next(),
            SearchPrevious => self.webview.search_previous(),
            Stop => self.webview.stop_loading(),
            WinOpen(url) => self.handle_error(self.open_in_new_window(&url)),
            ZoomIn => self.zoom_in(),
            ZoomNormal => self.zoom_normal(),
            ZoomOut => self.zoom_out(),
        }
    }

    /// Handle create window.
    fn handle_create(app: Rc<App>) {
        fn open(app: &Rc<App>, url: &str) {
            app.handle_error(app.open_in_new_window(url));
        }

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
                            let instance = app.clone();
                            app.app.question(&format!("A popup from {} was blocked. Do you want to open it?", base_url),
                                &['y', 'n', 'a', 'e'], move |answer|
                            {
                                match answer {
                                    Some('a') => {
                                        open(&instance, &url);
                                        instance.whitelist_popup(&url);
                                    },
                                    Some('y') => open(&instance, &url),
                                    Some('e') => {
                                        instance.blacklist_popup(&url);
                                    },
                                    _ => (),
                                }
                            });
                        }
                    }
                    else {
                        open(&app, &url);
                    }
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
                app.webview.finish_search();
                app.handle_error(app.webview.add_stylesheets());
                app.handle_error(app.webview.add_scripts());
                app.app.set_mode("normal");
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

    /// Open the given URL in a new window.
    fn open_in_new_window(&self, url: &str) -> AppResult {
        let program = env::args().next().unwrap();
        Command::new(program)
            .arg(url)
            .spawn()?;
        Ok(())
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

    /// Show the zoom level in the status bar.
    fn show_zoom(&self, level: i32) {
        Application::info(&self.app, &format!("Zoom level: {}%", level));
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
