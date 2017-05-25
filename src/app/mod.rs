/*
 * Copyright (c) 2016-2017 Boucher, Antoni <bouanto@zoho.com>
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
    ($app:ident . $($tt:tt)* ) => {{
        let result = $app.$($tt)*;
        $app.handle_error(result);
    }};
}

mod bookmarks;
mod browser;
mod config;
mod copy_paste;
mod dialog;
mod download;
mod hints;
mod pass_filler;
mod paths;
mod popup;
mod search_engine;
mod server;
mod test_utils;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::process::Command;

use gdk::{EventButton, EventKey, Rectangle, CONTROL_MASK};
use gtk::{self, ContainerExt, Inhibit, OrientableExt, WidgetExt};
use gtk::Orientation::Vertical;
use mg::{AppClose, Completers, CustomCommand, DefaultConfig, Mg, Modes, SettingChanged, StatusBarItem};
use mg_settings::errors::ErrorKind;
use relm::{Component, Relm, Resolver, Widget};
use relm_attributes::widget;
use webkit2gtk::{Download, HitTestResult, NavigationAction, WebViewExt};
use webkit2gtk::LoadEvent::{self, Finished, Started};
use webkit2gtk::NavigationType::Other;

use bookmarks::BookmarkManager;
use commands::AppCommand;
use commands::AppCommand::*;
use completers::{BookmarkCompleter, FileCompleter};
use config_dir::ConfigDir;
use download_list_view::DownloadListView;
use download_list_view::Msg::{Add, DecideDestination};
use message_server::{MessageServer, PATH};
use message_server::Msg::MsgRecv;
use pass_manager::PasswordManager;
use popup_manager::PopupManager;
use self::config::default_config;
use self::Msg::*;
use settings::AppSettings;
use webview::WebView;
use webview::Msg::{Close, NewWindow};

pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
const LEFT_BUTTON: u32 = 1;

static MODES: Modes = &[
    ("f", "follow"),
    ("i", "insert"),
];

pub type AppResult<T> = Result<T, Box<Error>>;

#[derive(Clone, Copy)]
pub enum FollowMode {
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

pub struct Model {
    bookmark_manager: BookmarkManager,
    client: usize,
    config_dir: ConfigDir,
    default_search_engine: Option<String>,
    follow_mode: FollowMode,
    hovered_link: Option<String>,
    init_url: Option<String>,
    message_server: Component<MessageServer>,
    relm: Relm<App>,
    popup_manager: PopupManager,
    search_engines: HashMap<String, String>,
}

#[derive(Msg)]
pub enum Msg {
    Action(i32),
    ClickElement,
    Command(AppCommand),
    DecideDownloadDestination(Resolver<bool>, Download, String),
    GoToInsertMode,
    PopupDecision(Option<String>, String),
    Scroll(i64),
    TryClose,
    WebViewClose,
}

#[widget]
impl Widget for App {
    fn init_view(&mut self) {
        handle_error!(self.model.bookmark_manager.create_tables());

        let url = self.model.init_url.take().unwrap_or(self.mg.widget().settings().home_page.clone());
        // FIXME: don't open here.
        self.webview.widget().open(&url);

        handle_error!(self.model.bookmark_manager.connect(App::bookmark_path(&self.model.config_dir)));
        handle_error!(self.model.popup_manager.load());

        // TODO
        //app.create_password_keyring();
        self.create_variables();

        self.listen_messages();
    }

    fn model(relm: &Relm<Self>, (init_url, config_dir): (Option<String>, Option<String>)) -> Model {
        let config_dir = ConfigDir::new(&config_dir).unwrap();
        // TODO: better error handling.
        let (whitelist, blacklist) = App::popup_path(&config_dir);
        let whitelist = whitelist.expect("cannot create configuration directory");
        let blacklist = blacklist.expect("cannot create configuration directory");
        let popup_manager = PopupManager::new((whitelist, blacklist));
        Model {
            bookmark_manager: BookmarkManager::new(),
            client: 0, // TODO: real client ID.
            config_dir,
            default_search_engine: None,
            follow_mode: FollowMode::Click,
            hovered_link: None,
            init_url,
            message_server: MessageServer::new().unwrap(), // TODO: handle error elsewhere.
            relm: relm.clone(),
            popup_manager,
            search_engines: HashMap::new(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Action(action) => self.activate_action(action),
            ClickElement => self.click_hint_element(),
            Command(ref command) => self.handle_command(command),
            DecideDownloadDestination(ref resolver, ref download, ref suggested_filename) => {
                resolver.resolve(self.handle_decide_destination(download, suggested_filename));
            },
            GoToInsertMode => self.go_in_insert_mode(),
            PopupDecision(answer, url) => self.handle_answer(answer.as_ref().map(|str| str.as_str()), &url),
            Scroll(scroll_percentage) => self.show_scroll(scroll_percentage),
            TryClose => self.quit(),
            WebViewClose => gtk::main_quit(),
        }
    }

    view! {
        #[name="mg"]
        Mg<AppCommand, AppSettings>((MODES, self.model.config_dir.config_file("config"),
            Some(self.model.config_dir.config_home()), default_config(&self.model.config_dir)))
        {
            completers: hash! {
                "file" => Box::new(FileCompleter::new()),
                "open" => Box::new(BookmarkCompleter::new("open")),
                "win-open" => Box::new(BookmarkCompleter::new("win-open")),
            },
            dark_theme: true,
            title: APP_NAME,
            gtk::Box {
                orientation: Vertical,
                #[name="download_list_view"]
                DownloadListView {
                    DecideDestination(ref resolver, ref download, ref suggested_filename) =>
                        DecideDownloadDestination(resolver.clone(), download.clone(), suggested_filename.clone()),
                },
                #[name="webview"]
                WebView(self.model.config_dir.clone()) {
                    Close => WebViewClose,
                    NewWindow(ref url) => self.open_in_new_window_handling_error(url),
                    button_release_event(_, event) => return self.handle_button_release(event),
                    create(_, action) => return self.handle_create(action),
                    context.download_started(_, download) => download_list_view@Add(download.clone()),
                    // Emit the scroll event whenever the view is drawn.
                    draw(_, _) => return self.emit_scrolled_event(),
                    load_changed(_, load_event) => self.handle_load_changed(load_event),
                    mouse_target_changed(_, hit_test_result, _) => self.mouse_target_changed(hit_test_result),
                    resource_load_started(_, _, _) => self.set_title(),
                    run_file_chooser(_, file_chooser_request) => return self.handle_file_chooser(file_chooser_request),
                    script_dialog(_, script_dialog) => return self.handle_script_dialog(script_dialog),
                    title_changed() => self.set_title(),
                    //uri_changed() => self.uri_changed(),
                    web_process_crashed => return self.web_process_crashed(),
                },
            },
            #[name="scroll_label"]
            StatusBarItem {
                text: "[top]",
            },
            #[name="url_label"]
            StatusBarItem {
            },
            AppClose => TryClose,
            CustomCommand(ref command) => Command(command.clone()),
            SettingChanged(ref setting) => self.webview.widget().setting_changed(setting),
            key_press_event(_, event_key) => return self.handle_key_press(event_key),
        }
    }
}

impl App {
    /// Show an error from a string.
    pub fn error(&self, error: &str) {
        self.mg.widget_mut().error(ErrorKind::Msg(error.to_string()).into());
    }

    /// Give the focus to the webview.
    fn focus_webview(&self) {
        self.webview.widget().root().grab_focus();
    }

    /// Get the size of the webview.
    fn get_webview_allocation(&self) -> Rectangle {
        self.webview.widget().root().get_allocation()
    }

    /// Get the title or the url if there are no title.
    fn get_title(&self) -> String {
        let title = self.webview.widget().get_title()
            .or(self.webview.widget().get_uri())
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
        if event.get_button() == LEFT_BUTTON && event.get_state().contains(CONTROL_MASK) {
            if let Some(url) = self.model.hovered_link.clone() {
                self.open_in_new_window_handling_error(&url);
                return Inhibit(true)
            }
        }
        Inhibit(false)
    }

    /// Handle the command.
    fn handle_command(&mut self, command: &AppCommand) {
        match *command {
            ActivateSelection => handle_error!(self.activate_selection()),
            Back => self.webview.widget().go_back(),
            BackwardSearch(ref input) => {
                self.webview.widget_mut().set_search_backward(true);
                self.webview.widget().search(input);
            },
            Bookmark => self.bookmark(),
            BookmarkDel => self.delete_bookmark(),
            BookmarkEditTags => self.edit_bookmark_tags(),
            ClearCache => self.clear_cache(),
            CopyUrl => self.copy_url(),
            DeleteAllCookies => self.delete_all_cookies(),
            DeleteCookies(ref domain) => self.delete_cookies(domain),
            DeleteSelectedBookmark => self.delete_selected_bookmark(),
            FinishSearch => self.webview.widget().finish_search(),
            FocusInput => handle_error!(self.focus_input()),
            Follow => {
                self.model.follow_mode = FollowMode::Click;
                self.webview.widget_mut().set_open_in_new_window(false);
                self.mg.widget_mut().set_mode("follow");
                handle_error!(self.follow_link(&self.hint_chars()))
            },
            Forward => self.webview.widget().go_forward(),
            HideHints => self.hide_hints(),
            Hover => {
                self.model.follow_mode = FollowMode::Hover;
                self.mg.widget_mut().set_mode("follow");
                handle_error!(self.follow_link(&self.hint_chars()))
            },
            Insert => self.go_in_insert_mode(),
            Inspector => self.webview.widget().show_inspector(),
            Normal => self.mg.widget_mut().set_mode("normal"),
            Open(ref url) => self.open(url),
            PasswordDelete => self.delete_password(),
            PasswordLoad => self.load_password(),
            PasswordSave => self.save_password(),
            PasswordSubmit => self.submit_login_form(),
            PasteUrl => self.paste_url(),
            Print => self.webview.widget().print(),
            Quit => self.quit(),
            Reload => self.webview.widget().reload(),
            ReloadBypassCache => self.webview.widget().reload_bypass_cache(),
            Screenshot(ref path) => self.webview.widget().screenshot(path),
            ScrollBottom => handle_error!(self.scroll_bottom()),
            ScrollDown => handle_error!(self.scroll_down_page()),
            ScrollDownHalf => handle_error!(self.scroll_down_half_page()),
            ScrollDownLine => handle_error!(self.scroll_down_line()),
            ScrollLeft => handle_error!(self.scroll_left()),
            ScrollRight => handle_error!(self.scroll_right()),
            ScrollTop => handle_error!(self.scroll_top()),
            ScrollUp => handle_error!(self.scroll_up_page()),
            ScrollUpHalf => handle_error!(self.scroll_up_half_page()),
            ScrollUpLine => handle_error!(self.scroll_up_line()),
            Search(ref input) => {
                self.webview.widget_mut().set_search_backward(false);
                self.webview.widget().search(input);
            },
            SearchEngine(ref args) => self.add_search_engine(args),
            SearchNext => self.webview.widget().search_next(),
            SearchPrevious => self.webview.widget().search_previous(),
            Stop => self.webview.widget().stop_loading(),
            WinFollow => {
                self.model.follow_mode = FollowMode::Click;
                self.webview.widget_mut().set_open_in_new_window(true);
                self.mg.widget_mut().set_mode("follow");
                handle_error!(self.follow_link(&self.hint_chars()))
            },
            WinOpen(ref url) => self.open_in_new_window_handling_error(url),
            WinPasteUrl => self.win_paste_url(),
            ZoomIn => self.zoom_in(),
            ZoomNormal => self.zoom_normal(),
            ZoomOut => self.zoom_out(),
        }
    }

    fn go_in_insert_mode(&mut self) {
        self.mg.widget_mut().set_mode("insert")
    }

    /// Handle create window.
    fn handle_create(&mut self, action: &NavigationAction) -> Option<gtk::Widget> {
        if let Some(request) = action.get_request() {
            if let Some(url) = request.get_uri() {
                if action.get_navigation_type() == Other && !self.model.popup_manager.is_whitelisted(&url) {
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
    fn handle_error(&self, error: AppResult<()>) {
        if let Err(error) = error {
            self.show_error(error);
        }
    }

    /// Handle the key press event.
    fn handle_key_press(&mut self, event_key: &EventKey) -> (Option<Msg>, Inhibit) {
        if self.mg.widget().get_mode() == "follow" {
            (None, self.handle_follow_key_press(event_key))
        }
        else {
            (None, Inhibit(false))
        }
    }

    /// Handle the load_changed event.
    /// Show the URL.
    /// Set the window title.
    /// Go back to normal mode.
    fn handle_load_changed(&self, load_event: LoadEvent) {
        if load_event == Started {
            self.webview.widget().finish_search();
            handle_error!(self.webview.widget().add_stylesheets(&self.model.config_dir));
            handle_error!(self.webview.widget().add_scripts(&self.model.config_dir));

            // Check to mode to avoid going back to normal mode if the user is in command mode.
            let set_mode = {
                let mode = self.mg.widget().get_mode();
                mode == "insert" || mode == "follow"
            };
            if set_mode {
                self.mg.widget_mut().set_mode("normal");
            }
        }

        if load_event == Finished {
            self.set_title_without_progress();
        }
        else {
            self.set_title();
        }
    }

    /// Show an info.
    pub fn info(&self, info: &str) {
        self.mg.widget_mut().info(info);
    }

    /// Handle the mouse target changed event of the webview to show the hovered URL and save it
    /// for use when using Ctrl-click.
    fn mouse_target_changed(&mut self, hit_test_result: &HitTestResult) {
        let link = hit_test_result.get_link_uri();
        {
            let empty = String::new();
            let text = link.as_ref().unwrap_or(&empty);
            self.mg.widget_mut().message(text);
        }
        self.model.hovered_link = link;
    }

    /// Open the given URL in the web view.
    fn open(&self, url: &str) {
        let url = self.transform_url(url);
        self.webview.widget().open(&url);
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
    fn open_in_new_window_handling_error(&self, url: &str) {
        handle_error!(self.open_in_new_window(url));
    }

    /// Try to close the web view and quit the application.
    fn quit(&self) {
        // Ask for a confirmation before quitting the application when there are active
        // downloads.
        let can_quit =
            if self.download_list_view.widget().has_active_downloads() {
                self.blocking_yes_no_question("There are active downloads. Do you want to quit?")
            }
            else {
                true
            };

        if can_quit {
            self.webview.widget().try_close();
        }
    }

    /// Set the title of the window as the progress and the web page title.
    fn set_title(&self) {
        let progress = (self.webview.widget().get_estimated_load_progress() * 100.0) as i32;
        if progress == 100 {
            self.set_title_without_progress();
        }
        else {
            let title = self.get_title();
            self.mg.widget().set_title(&format!("[{}%] {}{}", progress, title, APP_NAME));
        }
    }

    /// Set the title of the window as the web page title or url.
    fn set_title_without_progress(&self) {
        let title = self.get_title();
        self.mg.widget().set_title(&format!("{}{}", title, APP_NAME));
    }

    /// Show an error.
    pub fn show_error(&self, error: Box<Error>) {
        self.error(&error.to_string());
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
        self.scroll_label.widget().set_text(&text);
    }

    /// Show the zoom level in the status bar.
    fn show_zoom(&self, level: i32) {
        self.mg.widget_mut().info(&format!("Zoom level: {}%", level));
    }

    /// Handle the URI changed event.
    fn uri_changed(&self) {
        if let Some(url) = self.webview.widget().get_uri() {
            // TODO: use a model attribute.
            self.url_label.widget().set_text(&url);
        }
    }

    /// Handle the web process crashed event.
    fn web_process_crashed(&mut self) -> bool {
        self.error("The web process crashed.");
        false
    }

    /// Open in a new window the url from the system clipboard.
    fn win_paste_url(&self) {
        if let Some(url) = self.get_url_from_clipboard() {
            self.open_in_new_window_handling_error(&url);
        }
    }

    /// Zoom in.
    fn zoom_in(&self) {
        let zoom = self.webview.widget().zoom_in();
        self.show_zoom(zoom);
    }

    /// Zoom back to 100%.
    fn zoom_normal(&self) {
        let zoom = self.webview.widget().zoom_normal();
        self.show_zoom(zoom);
    }

    /// Zoom out.
    fn zoom_out(&self) {
        let zoom = self.webview.widget().zoom_out();
        self.show_zoom(zoom);
    }
}
