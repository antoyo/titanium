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

macro_rules! handle_error {
    ($app:ident . $attribute:ident . $method:ident ( $($args:expr),* ) ) => {{
        let result = $app.$attribute.$method($($args),*);
        $app.handle_error(result);
    }};
    ($app:ident . $method:ident ( $($args:expr),* ) ) => {{
        let result = $app.$method($($args),*);
        $app.handle_error(result);
    }};
}

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

use gdk::{EventButton, EventKey, CONTROL_MASK};
use gtk::{self, ContainerExt, Inhibit, Widget, WidgetExt};
use gtk::Orientation::Vertical;
use mg::{Application, ApplicationBuilder, StatusBarItem};
use xdg::BaseDirectories;
use webkit2gtk::{HitTestResult, NavigationAction, WebViewExt};
use webkit2gtk::LoadEvent::{self, Finished, Started};
use webkit2gtk::NavigationType::Other;

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

pub type AppResult<T> = Result<T, Box<Error>>;
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
    app: Box<MgApp>,
    bookmark_manager: Rc<RefCell<BookmarkManager>>,
    default_search_engine: Rc<RefCell<Option<String>>>,
    download_list_view: DownloadListView,
    follow_mode: Cell<FollowMode>,
    hovered_link: Option<String>,
    popup_manager: PopupManager,
    scroll_label: Rc<StatusBarItem>,
    search_engines: Rc<RefCell<HashMap<String, String>>>,
    url_label: StatusBarItem,
    webview: Box<WebView>,
}

impl App {
    fn build(bookmark_manager: &Rc<RefCell<BookmarkManager>>) -> Box<MgApp> {
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

    pub fn new(homepage: Option<String>) -> Box<Self> {
        let bookmark_manager = Rc::new(RefCell::new(BookmarkManager::new()));
        let mut mg_app = App::build(&bookmark_manager);
        mg_app.use_dark_theme();
        mg_app.set_window_title(APP_NAME);

        let scroll_label = Rc::new(mg_app.add_statusbar_item());
        scroll_label.set_text("[top]");

        let url_label = mg_app.add_statusbar_item();

        let vbox = gtk::Box::new(Vertical, 0);

        let download_list_view = DownloadListView::new();
        vbox.add(&*download_list_view);

        let webview = WebView::new();
        vbox.add(&**webview);

        mg_app.set_view(&vbox);

        let mut app = Box::new(App {
            app: mg_app,
            bookmark_manager: bookmark_manager,
            default_search_engine: Rc::new(RefCell::new(None)),
            download_list_view: download_list_view,
            follow_mode: Cell::new(FollowMode::Click),
            hovered_link: None,
            popup_manager: PopupManager::new(),
            scroll_label: scroll_label,
            search_engines: Rc::new(RefCell::new(HashMap::new())),
            url_label: url_label,
            webview: webview,
        });

        app.create_events();

        // Create the events before parsing the config so that the settings are taken into account
        // and the commands are executed.
        app.parse_config();

        let url = homepage.unwrap_or(app.app.settings().home_page.clone());
        app.webview.open(&url);

        let result = (*app.bookmark_manager.borrow_mut()).load();
        app.handle_error(result);

        handle_error!(app.popup_manager.load());

        app.create_variables();

        app
    }

    /// Create the events.
    fn create_events(&mut self) {
        connect!(self.app, connect_close, self, quit);
        connect!(self.app, connect_command(command), self, handle_command(command));
        connect!(self.app, connect_key_press_event(_, event_key), self, handle_key_press(event_key));
        connect!(self.app, connect_setting_changed(setting), self.webview, WebView::setting_changed(setting));
        connect!(self.app, connect_special_command(command), self, handle_special_command(command));
        connect!(self.webview, connect_button_release_event(_, event), self, handle_button_release(event));
        connect!(self.webview, connect_create(_, action), self, handle_create(action));
        connect!(self.webview, connect_load_changed(_, load_event), self, handle_load_changed(load_event));
        connect!(self.webview, connect_new_window(url), self, open_in_new_window_handling_error(url));
        connect!(self.webview, connect_resource_load_started(_, _, _), self, set_title);
        connect!(self.webview, connect_script_dialog(_, script_dialog), self, handle_script_dialog(script_dialog));
        connect!(self.webview, connect_scrolled(scroll_percentage), self, show_scroll(scroll_percentage));
        connect!(self.webview, connect_title_changed, self, set_title);
        connect!(self.webview, connect_uri_changed, self, uri_changed);
        connect!(self.webview, connect_web_process_crashed(_), self, web_process_crashed);

        connect!(self.webview, connect_run_file_chooser(_, file_chooser_request),
            self, handle_file_chooser(file_chooser_request));

        connect!(self.webview, connect_download_started(_, download),
            &mut self.download_list_view, DownloadListView::add(download));

        connect!(self.download_list_view, connect_decide_destination(download, suggested_filename),
            self, handle_decide_destination(download, suggested_filename));

        connect!(self.webview, connect_mouse_target_changed(_, hit_test_result, _),
            self, mouse_target_changed(hit_test_result));

        self.webview.connect_close(|_| {
            gtk::main_quit();
        });
    }

    /// Show an error.
    pub fn error(&mut self, error: &str) {
        self.app.error(error);
    }

    /// Focus the first input element.
    fn focus_input(&mut self) {
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

    /// Handle the button release event to open in new window when using Ctrl-click.
    fn handle_button_release(&mut self, event: &EventButton) -> Inhibit {
        if event.get_button() == LEFT_BUTTON && event.get_state() & CONTROL_MASK == CONTROL_MASK {
            if let Some(url) = self.hovered_link.clone() {
                self.open_in_new_window_handling_error(&url);
                return Inhibit(true)
            }
        }
        Inhibit(false)
    }

    /// Handle the command.
    fn handle_command(&mut self, command: AppCommand) {
        match command {
            ActivateSelection => handle_error!(self.webview.activate_selection()),
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
                handle_error!(self.webview.follow_link(self.hint_chars()))
            },
            Forward => self.webview.go_forward(),
            HideHints => self.hide_hints(),
            Hover => {
                self.follow_mode.set(FollowMode::Hover);
                self.app.set_mode("follow");
                handle_error!(self.webview.follow_link(self.hint_chars()))
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
            ScrollBottom => handle_error!(self.webview.scroll_bottom()),
            ScrollDown => handle_error!(self.webview.scroll_down_page()),
            ScrollDownHalf => handle_error!(self.webview.scroll_down_half_page()),
            ScrollDownLine => handle_error!(self.webview.scroll_down_line()),
            ScrollTop => handle_error!(self.webview.scroll_top()),
            ScrollUp => handle_error!(self.webview.scroll_up_page()),
            ScrollUpHalf => handle_error!(self.webview.scroll_up_half_page()),
            ScrollUpLine => handle_error!(self.webview.scroll_up_line()),
            SearchEngine(args) => self.add_search_engine(&args),
            SearchNext => self.webview.search_next(),
            SearchPrevious => self.webview.search_previous(),
            Stop => self.webview.stop_loading(),
            WinFollow => {
                self.follow_mode.set(FollowMode::Click);
                self.webview.set_open_in_new_window();
                self.app.set_mode("follow");
                handle_error!(self.webview.follow_link(self.hint_chars()))
            },
            WinOpen(url) => self.open_in_new_window_handling_error(&url),
            WinPasteUrl => self.win_paste_url(),
            ZoomIn => self.zoom_in(),
            ZoomNormal => self.zoom_normal(),
            ZoomOut => self.zoom_out(),
        }
    }

    /// Handle create window.
    fn handle_create(&mut self, action: &NavigationAction) -> Option<Widget> {
        if let Some(request) = action.get_request() {
            if let Some(url) = request.get_uri() {
                if action.get_navigation_type() == Other && !self.popup_manager.is_whitelisted(&url) {
                    self.handle_popup(url);
                }
                else {
                    self.open_in_new_window_handling_error(&url);
                }
            }
        }
        None
    }

    /// Show an error in the result is an error.
    fn handle_error(&mut self, error: AppResult<()>) {
        if let Err(error) = error {
            self.show_error(error);
        }
    }

    /// Handle the key press event.
    fn handle_key_press(&mut self, event_key: &EventKey) -> Inhibit {
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
    fn handle_load_changed(&mut self, load_event: LoadEvent) {
        if load_event == Started {
            self.webview.finish_search();
            handle_error!(self.webview.add_stylesheets());
            handle_error!(self.webview.add_scripts());

            // Check to mode to avoid going back to normal mode if the user is in command mode.
            let set_mode = {
                let mode = self.app.get_mode();
                mode == "insert" || mode == "follow"
            };
            if set_mode {
                self.app.set_mode("normal");
            }
        }

        if load_event == Finished {
            self.set_title_without_progress();
        }
        else {
            self.set_title();
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

    /// Handle the mouse target changed event of the webview to show the hovered URL and save it
    /// for use when using Ctrl-click.
    fn mouse_target_changed(&mut self, hit_test_result: &HitTestResult) {
        let link = hit_test_result.get_link_uri();
        {
            let empty = String::new();
            let text = link.as_ref().unwrap_or(&empty);
            self.app.message(text);
        }
        self.hovered_link = link;
    }

    /// Open the given URL in the web view.
    fn open(&self, url: &str) {
        let url = self.transform_url(url);
        self.webview.open(&url);
    }

    /// Open the given URL in a new window.
    fn open_in_new_window(&self, url: &str) -> AppResult<()> {
        let url = self.transform_url(url);
        let program = env::args().next().unwrap();
        Command::new(program)
            .arg(url)
            .spawn()?;
        Ok(())
    }

    /// Open the given URL in a new window, showing the error if any.
    fn open_in_new_window_handling_error(&mut self, url: &str) {
        handle_error!(self.open_in_new_window(url));
    }

    /// Try to close the web view and quit the application.
    fn quit(&mut self) {
        // Ask for a confirmation before quitting the application when there are active
        // downloads.
        let can_quit =
            if self.download_list_view.has_active_downloads() {
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
    fn show_error(&mut self, error: Box<Error>) {
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
    fn show_zoom(&mut self, level: i32) {
        self.app.info(&format!("Zoom level: {}%", level));
    }

    /// Handle the URI changed event.
    fn uri_changed(&self) {
        if let Some(url) = self.webview.get_uri() {
            self.url_label.set_text(&url);
        }
    }

    /// Handle the web process crashed event.
    fn web_process_crashed(&mut self) -> bool {
        self.error("The web process crashed.");
        false
    }

    /// Open in a new window the url from the system clipboard.
    fn win_paste_url(&mut self) {
        if let Some(url) = self.get_url_from_clipboard() {
            self.open_in_new_window_handling_error(&url);
        }
    }

    /// Zoom in.
    fn zoom_in(&mut self) {
        let zoom = self.webview.zoom_in();
        self.show_zoom(zoom);
    }

    /// Zoom back to 100%.
    fn zoom_normal(&mut self) {
        let zoom = self.webview.zoom_normal();
        self.show_zoom(zoom);
    }

    /// Zoom out.
    fn zoom_out(&mut self) {
        let zoom = self.webview.zoom_out();
        self.show_zoom(zoom);
    }
}
