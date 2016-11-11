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

mod bookmarks;
mod config;
mod copy_paste;
mod dialog;
mod download;
mod hints;
mod popup;
mod search_engine;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::process::Command;
use std::rc::Rc;

use gdk::{EventKey, CONTROL_MASK};
use gtk::{self, ContainerExt, Inhibit, WidgetExt};
use gtk::Orientation::Vertical;
use mg::{Application, ApplicationBuilder, StatusBarItem};
use xdg::BaseDirectories;
use webkit2gtk::LoadEvent::{Finished, Started};
use webkit2gtk::NavigationType::Other;
use webkit2gtk::WebViewExt;

use bookmarks::BookmarkManager;
use commands::{AppCommand, SpecialCommand};
use commands::AppCommand::*;
use commands::SpecialCommand::*;
use completers::{BookmarkCompleter, FileCompleter};
use download_list_view::DownloadListView;
use popup_manager::PopupManager;
use settings::AppSettings;
use webview::WebView;

pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
const LEFT_BUTTON: u32 = 1;

pub type AppBoolResult = Result<bool, Box<Error>>;
pub type AppResult = Result<(), Box<Error>>;
pub type MgApp = Application<AppCommand, AppSettings, SpecialCommand>;

#[derive(Clone, Copy)]
enum FollowMode {
    Click,
    Hover,
}

impl Display for FollowMode {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let string =
            match *self {
                FollowMode::Click => "click",
                FollowMode::Hover => "hover",
            };
        write!(formatter, "{}", string)
    }
}

/// Titanium application.
pub struct App {
    app: Rc<MgApp>,
    bookmark_manager: Rc<RefCell<BookmarkManager>>,
    default_search_engine: Rc<RefCell<Option<String>>>,
    download_list_view: Rc<RefCell<DownloadListView>>,
    follow_mode: Cell<FollowMode>,
    hovered_link: Rc<RefCell<Option<String>>>,
    popup_manager: Rc<RefCell<PopupManager>>,
    scroll_label: Rc<StatusBarItem>,
    search_engines: Rc<RefCell<HashMap<String, String>>>,
    url_label: StatusBarItem,
    webview: Rc<WebView>,
}

impl App {
    fn build(bookmark_manager: &Rc<RefCell<BookmarkManager>>) -> Rc<MgApp> {
        let xdg_dirs = BaseDirectories::with_prefix(APP_NAME).unwrap();
        let include_path = xdg_dirs.get_config_home();

        ApplicationBuilder::new()
            .completer("file", FileCompleter::new())
            .completer("open", BookmarkCompleter::new(bookmark_manager.clone(), "open"))
            .completer("win-open", BookmarkCompleter::new(bookmark_manager.clone(), "win-open"))
            .include_path(include_path)
            .modes(hash! {
                "f" => "follow",
                "i" => "insert",
            })
            .settings(AppSettings::new())
            .build()
    }

    pub fn new(homepage: Option<String>) -> Rc<Self> {
        let bookmark_manager = Rc::new(RefCell::new(BookmarkManager::new()));
        let mg_app = App::build(&bookmark_manager);
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
            bookmark_manager: bookmark_manager,
            default_search_engine: Rc::new(RefCell::new(None)),
            download_list_view: Rc::new(RefCell::new(download_list_view)),
            follow_mode: Cell::new(FollowMode::Click),
            hovered_link: Rc::new(RefCell::new(None)),
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

        app.handle_error((*app.bookmark_manager.borrow_mut()).load());

        app.handle_error((*app.popup_manager.borrow_mut()).load());

        App::create_variables(app.clone());

        app
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
            let webview = app.webview.clone();
            let application = app.clone();
            app.webview.connect_uri_changed(move || {
                if let Some(url) = webview.get_uri() {
                    application.url_label.set_text(&url);
                }
            });
        }

        {
            let application = app.clone();
            app.webview.connect_title_changed(move || {
                application.set_title();
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
            app.webview.connect_mouse_target_changed(move |_, hit_test_result, _| {
                let link = hit_test_result.get_link_uri();
                {
                    let empty = String::new();
                    let text = link.as_ref().unwrap_or(&empty);
                    application.app.message(text);
                }
                *application.hovered_link.borrow_mut() = link;
            });
        }

        {
            let application = app.clone();
            (&**app.webview).connect_button_release_event(move |_, event| {
                if event.get_button() == LEFT_BUTTON && event.get_state() & CONTROL_MASK == CONTROL_MASK {
                    if let Some(ref url) = *application.hovered_link.borrow() {
                        application.open_in_new_window_handling_error(url);
                        return Inhibit(true)
                    }
                }
                Inhibit(false)
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

        App::handle_file_chooser(app);

        {
            let application = app.clone();
            app.webview.connect_download_started(move |_, download| {
                (*application.download_list_view.borrow_mut()).add(download);
            });
        }

        {
            let application = app.clone();
            app.webview.connect_new_window(move |url| {
                application.open_in_new_window_handling_error(url);
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
            gtk::main_quit();
        });
    }

    /// Show an error.
    pub fn error(&self, error: &str) {
        self.app.error(error);
    }

    /// Focus the first input element.
    fn focus_input(&self) {
        let result = self.webview.focus_input();
        if let Ok(true) = result {
            self.app.set_mode("insert")
        }
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
            Bookmark => self.bookmark(),
            BookmarkDel => self.delete_bookmark(),
            BookmarkEditTags => self.edit_bookmark_tags(),
            CopyUrl => self.copy_url(),
            FinishSearch => self.webview.finish_search(),
            FocusInput => self.focus_input(),
            Follow => {
                self.follow_mode.set(FollowMode::Click);
                self.app.set_mode("follow");
                self.handle_error(self.webview.follow_link(self.hint_chars()))
            },
            Forward => self.webview.go_forward(),
            HideHints => self.hide_hints(),
            Hover => {
                self.follow_mode.set(FollowMode::Hover);
                self.app.set_mode("follow");
                self.handle_error(self.webview.follow_link(self.hint_chars()))
            },
            Insert => self.app.set_mode("insert"),
            Inspector => self.webview.show_inspector(),
            Normal => self.app.set_mode("normal"),
            Open(url) => self.open(&url),
            PasteUrl => self.paste_url(),
            Print => self.webview.print(),
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
                self.follow_mode.set(FollowMode::Click);
                self.webview.set_open_in_new_window();
                self.app.set_mode("follow");
                self.handle_error(self.webview.follow_link(self.hint_chars()))
            },
            WinOpen(url) => self.open_in_new_window_handling_error(&url),
            WinPasteUrl => self.win_paste_url(),
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
                    let popup_manager = &*app.popup_manager.borrow();
                    if action.get_navigation_type() == Other && !popup_manager.is_whitelisted(&url) {
                        App::handle_popup(&app, url);
                    }
                    else {
                        app.open_in_new_window_handling_error(&url);
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

        webview.connect_load_changed(move |_, load_event| {
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

            if load_event == Finished {
                app.set_title_without_progress();
            }
            else {
                app.set_title();
            }
        });
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

    /// Try to close the web view and quit the application.
    fn quit(&self) {
        // Ask for a confirmation before quitting the application when there are active
        // downloads.
        let can_quit =
            if (*self.download_list_view.borrow()).has_active_downloads() {
                self.app.blocking_yes_no_question("There are active downloads. Do you want to quit?")
            }
            else {
                true
            };

        if can_quit {
            self.webview.try_close();
        }
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

    /// Open in a new window the url from the system clipboard.
    fn win_paste_url(&self) {
        if let Some(url) = self.get_url_from_clipboard() {
            self.open_in_new_window_handling_error(&url);
        }
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
