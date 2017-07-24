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
mod clipboard;
mod config;
mod copy_paste;
mod dialog;
mod download;
mod file_chooser;
mod hints;
mod pass_filler;
mod paths;
mod popup;
mod search_engine;
mod server;
mod test_utils;
mod url;

use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

use gdk::{EventButton, EventKey, Rectangle, CONTROL_MASK};
use gtk::{self, Inhibit, OrientableExt, WidgetExt};
use gtk::Orientation::Vertical;
use mg::{
    AppClose,
    CloseWin,
    Completers,
    CompletionViewChange,
    CustomCommand,
    DarkTheme,
    DialogResult,
    Error,
    Info,
    Message,
    Mg,
    ModeChanged,
    Modes,
    SetMode,
    SettingChanged,
    StatusBarItem,
    Text,
    Title,
    yes_no_question,
};
use relm::{Relm, Widget};
use relm_attributes::widget;
use webkit2gtk::{
    Download,
    HitTestResult,
    NavigationAction,
    WebContext,
    WebViewExt,
};
use webkit2gtk::LoadEvent::{self, Finished, Started};
use webkit2gtk::NavigationType::Other;

use titanium_common::{InnerMessage, PageId};
use titanium_common::Percentage::{self, All, Percent};

use managers::{BookmarkManager, ConfigDir, PasswordManager};
use managers::popup::{PopupManager, create_popup_manager};
use commands::AppCommand;
use commands::AppCommand::*;
use completers::{BookmarkCompleter, FileCompleter};
use views::DownloadListView;
use views::download_list::Msg::{
    ActiveDownloads,
    DownloadListError,
};
use errors::{self, Result};
use self::config::default_config;
use self::dialog::handle_script_dialog;
use self::file_chooser::handle_file_chooser;
use self::Msg::*;
use settings::AppSettings;
use settings::AppSettingsVariant::{
    self,
    HintChars,
    HomePage,
};
use views::WebView;
use views::webview::Msg::{
    AddScripts,
    AddStylesheets,
    AppError,
    Close,
    EndSearch,
    NewWindow,
    PageFinishSearch,
    PageOpen,
    PagePrint,
    PageScreenshot,
    PageSearch,
    PageSearchNext,
    PageSearchPrevious,
    PageZoomIn,
    PageZoomNormal,
    PageZoomOut,
    SearchBackward,
    SetOpenInNewWindow,
    ShowInspector,
    WebPageId,
    WebViewSettingChanged,
    ZoomChange,
};

pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
const LEFT_BUTTON: u32 = 1;

static MODES: Modes = &[
    ("f", "follow"),
    ("i", "insert"),
];

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
    command_text: String,
    config_dir: ConfigDir,
    current_url: String,
    default_search_engine: Option<String>,
    follow_mode: FollowMode,
    has_active_downloads: bool,
    has_hovered_link: Rc<Cell<bool>>,
    hint_chars: String,
    home_page: Option<String>,
    hovered_link: Option<String>,
    in_follow_mode: Rc<Cell<bool>>,
    init_url: Option<String>,
    mode: String,
    password_manager: PasswordManager,
    popup_manager: Option<PopupManager>,
    relm: Relm<App>,
    scroll_text: String,
    search_engines: HashMap<String, String>,
    title: String,
    web_context: WebContext,
}

#[derive(Msg)]
pub enum Msg {
    AppSetMode(String),
    AppSettingChanged(AppSettingsVariant),
    ButtonRelease(EventButton),
    Create(NavigationAction),
    Command(AppCommand),
    CommandText(String),
    CreateWindow(String),
    DecideDownloadDestination(Download, String),
    DownloadDestination(DialogResult, Download, String),
    EmitScrolledEvent,
    Exit(bool),
    FileDialogSelection(Option<String>),
    HasActiveDownloads(bool),
    KeyPress(EventKey),
    LoadChanged(LoadEvent),
    LoadStarted,
    MessageRecv(InnerMessage),
    MouseTargetChanged(HitTestResult),
    OverwriteDownload(Download, String, bool),
    SetPageId(PageId),
    PopupDecision(Option<String>, String),
    Remove(PageId),
    ServerSend(PageId, InnerMessage),
    ShowError(String),
    ShowZoom(i32),
    TagEdit(Option<String>),
    TitleChanged,
    TryClose,
    UriChanged,
    WebProcessCrashed,
    WebViewClose,
}

#[widget]
impl Widget for App {
    fn init_view(&mut self) {
        handle_error!(self.model.bookmark_manager.create_tables());

        match App::bookmark_path(&self.model.config_dir) {
            Ok(bookmark_path) => handle_error!(self.model.bookmark_manager.connect(bookmark_path)),
            Err(error) => self.show_error(error),
        }

        handle_error!(self.clean_download_folder());
        self.init_popup_manager();

        if let Some(ref url) = self.model.init_url {
            self.webview.emit(PageOpen(url.clone()));
        }

        self.connect_dialog_events();
        self.connect_download_events();
        self.create_variables();
    }

    fn model(relm: &Relm<Self>, (init_url, config_dir, web_context): (Option<String>, ConfigDir, WebContext)) -> Model {
        let popup_manager = create_popup_manager(&config_dir);
        Model {
            bookmark_manager: BookmarkManager::new(),
            command_text: String::new(),
            config_dir,
            current_url: String::new(),
            default_search_engine: None,
            follow_mode: FollowMode::Click,
            has_active_downloads: false,
            has_hovered_link: Rc::new(Cell::new(false)),
            hint_chars: "hjklasdfgyuiopqwertnmzxcvb".to_string(),
            home_page: None,
            hovered_link: None,
            in_follow_mode: Rc::new(Cell::new(false)),
            init_url,
            mode: "normal".to_string(),
            password_manager: PasswordManager::new(),
            popup_manager,
            relm: relm.clone(),
            scroll_text: "[top]".to_string(),
            search_engines: HashMap::new(),
            title: APP_NAME.to_string(),
            web_context,
        }
    }

    /// Set the title of the window as the progress and the web page title.
    fn set_title(&mut self) {
        let progress = (self.webview.widget().get_estimated_load_progress() * 100.0) as i32;
        if progress == 100 {
            self.set_title_without_progress();
        }
        else {
            let title = self.get_title();
            self.model.title = format!("[{}%] {}{}", progress, title, APP_NAME);
        }
    }

    /// Set the title of the window as the web page title or url.
    fn set_title_without_progress(&mut self) {
        let title = self.get_title();
        self.model.title = format!("{}{}", title, APP_NAME);
    }

    /// Show the scroll percentage.
    fn show_scroll(&mut self, scroll_percentage: Percentage) {
        self.model.scroll_text =
            match scroll_percentage {
                All => "[all]".to_string(),
                Percent(0) => "[top]".to_string(),
                Percent(100) => "[bot]".to_string(),
                Percent(percent) => format!("[{}%]", percent),
            };
    }

    fn update(&mut self, event: Msg) {
        match event {
            AppSetMode(mode) => {
                self.adjust_in_follow_mode(&mode);
                self.model.mode = mode
            },
            AppSettingChanged(setting) => self.setting_changed(setting),
            ButtonRelease(event) => self.handle_button_release(event),
            Create(navigation_action) => self.handle_create(navigation_action),
            Command(ref command) => self.handle_command(command),
            CommandText(text) => self.model.command_text = text,
            DecideDownloadDestination(download, suggested_filename) =>
                self.download_input(download, suggested_filename),
            DownloadDestination(destination, download, suggested_filename) =>
                handle_error!(self.download_destination_chosen(destination, download, suggested_filename)),
            EmitScrolledEvent => self.emit_scrolled_event(),
            Exit(can_quit) => self.quit(can_quit),
            FileDialogSelection(file) => self.file_dialog_selection(file),
            HasActiveDownloads(active) => self.model.has_active_downloads = active,
            KeyPress(event_key) => self.handle_key_press(event_key),
            LoadChanged(load_event) => self.handle_load_changed(load_event),
            LoadStarted => self.load_started(),
            MessageRecv(message) => self.message_recv(message),
            MouseTargetChanged(hit_test_result) => self.mouse_target_changed(hit_test_result),
            // To be listened by the user.
            CreateWindow(_) => (),
            OverwriteDownload(download, download_destination, overwrite) =>
                self.overwrite_download(download, download_destination, overwrite),
            PopupDecision(answer, url) => self.handle_answer(answer.as_ref().map(|str| str.as_str()), &url),
            // To be listened by the user.
            Remove(_) => (),
            // To be listened by the user.
            ServerSend(_, _) => (),
            // To be listened by the user.
            SetPageId(_) => (),
            ShowError(error) => self.error(&error),
            ShowZoom(level) => self.show_zoom(level),
            TagEdit(tags) => self.set_tags(tags),
            TitleChanged => self.set_title(),
            TryClose => self.try_quit(),
            UriChanged => self.uri_changed(),
            WebProcessCrashed => self.web_process_crashed(),
            WebViewClose => self.close_webview(),
        }
    }

    /// Handle the URI changed event.
    fn uri_changed(&mut self) {
        if let Some(url) = self.webview.widget().get_uri() {
            self.model.current_url = url;
        }
    }

    view! {
        #[name="mg"]
        Mg<AppCommand, AppSettings>(MODES, self.model.config_dir.config_file("config"),
            Some(self.model.config_dir.config_home()), default_config(&self.model.config_dir))
        {
            Completers: hash! {
                "file" => Box::new(FileCompleter::new()),
                "open" => Box::new(BookmarkCompleter::new("open")),
                "win-open" => Box::new(BookmarkCompleter::new("win-open")),
            },
            DarkTheme: true,
            Title: self.model.title.clone(),
            gtk::Box {
                orientation: Vertical,
                #[name="download_list_view"]
                DownloadListView {
                    ActiveDownloads(active) => HasActiveDownloads(active),
                    DownloadListError(ref error) => ShowError(error.clone()),
                },
                #[name="webview"]
                WebView((self.model.config_dir.clone(), self.model.web_context.clone())) {
                    AppError(ref error) => ShowError(error.clone()),
                    Close => WebViewClose,
                    NewWindow(ref url) => Command(WinOpen(url.clone())),
                    WebPageId(page_id) => SetPageId(page_id),
                    ZoomChange(level) => ShowZoom(level),
                    button_release_event(_, event) with (has_hovered_link) =>
                        (ButtonRelease(event.clone()), App::inhibit_button_release(&has_hovered_link, event)),
                    create(_, action) => (Create(action.clone()), None),
                    // Emit the scroll event whenever the view is drawn.
                    draw(_, _) => (EmitScrolledEvent, Inhibit(false)),
                    load_changed(_, load_event) => LoadChanged(load_event),
                    mouse_target_changed(_, hit_test_result, _) => MouseTargetChanged(hit_test_result.clone()),
                    resource_load_started(_, _, _) => LoadStarted,
                    title_changed() => TitleChanged,
                    uri_changed() => UriChanged,
                    web_process_crashed => (WebProcessCrashed, false),
                },
            },
            #[name="scroll_label"]
            StatusBarItem {
                Text: self.model.scroll_text.clone(),
            },
            StatusBarItem {
                Text: self.model.current_url.clone(),
            },
            AppClose => TryClose,
            CompletionViewChange(ref completion) => CommandText(completion.clone()),
            CustomCommand(ref command) => Command(command.clone()),
            ModeChanged(ref mode) => AppSetMode(mode.clone()),
            SettingChanged(ref setting) => AppSettingChanged(setting.clone()),
            key_press_event(_, event_key) with (in_follow_mode) =>
                (KeyPress(event_key.clone()), App::inhibit_key_press(&in_follow_mode)),
        }
    }
}

impl App {
    fn adjust_in_follow_mode(&mut self, mode: &str) {
        self.model.in_follow_mode.set(mode == "follow");
    }

    fn close_webview(&self) {
        let page_id = self.webview.widget().get_page_id();
        self.model.relm.stream().emit(Remove(page_id));

        self.mg.stream().emit(CloseWin);
    }

    fn connect_dialog_events(&self) {
        let mg = self.mg.stream().clone();
        connect!(self.model.relm, self.webview.widget(), connect_script_dialog(_, script_dialog),
            return handle_script_dialog(script_dialog, &mg));

        // TODO: add a #[stream(mg)] attribute in relm to support connecting an event to a
        // function while getting the stream (for use in view! {})?
        let mg = self.mg.stream().clone();
        connect!(self.model.relm, self.webview.widget(), connect_run_file_chooser(_, file_chooser_request),
            return handle_file_chooser(&mg, file_chooser_request));
    }

    /// Show an error from a string.
    pub fn error(&self, error: &str) {
        self.mg.emit(Error(error.into()));
    }

    /// Give the focus to the webview.
    fn focus_webview(&self) {
        self.webview.widget().grab_focus();
    }

    /// Get the size of the webview.
    fn get_webview_allocation(&self) -> Rectangle {
        self.webview.widget().get_allocation()
    }

    /// Get the title or the url if there are no title.
    fn get_title(&self) -> String {
        let title = self.webview.widget().get_title()
            .or_else(|| self.webview.widget().get_uri())
            .unwrap_or_default();
        if title.is_empty() {
            String::new()
        }
        else {
            format!("{} - ", title)
        }
    }

    fn get_webview_context(&self) -> Option<WebContext> {
        let context = self.webview.widget().get_context();
        if context.is_none() {
            self.error("Cannot retrieve web view context");
        }
        context
    }

    fn go_in_insert_mode(&mut self) {
        self.set_mode("insert");
    }

    fn go_in_normal_mode(&mut self) {
        self.set_mode("normal");
    }

    /// Handle the button release event to open in new window when using Ctrl-click.
    fn handle_button_release(&mut self, event: EventButton) {
        if event.get_button() == LEFT_BUTTON && event.get_state().contains(CONTROL_MASK) {
            if let Some(url) = self.model.hovered_link.clone() {
                self.open_in_new_window(&url);
            }
        }
    }

    /// Handle the command.
    fn handle_command(&mut self, command: &AppCommand) {
        match *command {
            ActivateSelection => self.activate_selection(),
            Back => self.webview.widget().go_back(),
            BackwardSearch(ref input) => {
                self.webview.emit(SearchBackward(true));
                self.webview.emit(PageSearch(input.clone()));
            },
            Bookmark => self.bookmark(),
            BookmarkDel => self.delete_bookmark(),
            BookmarkEditTags => self.edit_bookmark_tags(),
            ClearCache => self.clear_cache(),
            ClickNextPage => self.server_send(InnerMessage::ClickNextPage()),
            ClickPrevPage => self.server_send(InnerMessage::ClickPrevPage()),
            CopyUrl => self.copy_url(),
            DeleteAllCookies => self.delete_all_cookies(),
            DeleteCookies(ref domain) => self.delete_cookies(domain),
            DeleteSelectedBookmark => self.delete_selected_bookmark(),
            FinishSearch => self.webview.emit(PageFinishSearch),
            FocusInput => self.focus_input(),
            Follow => {
                // TODO: move that into a method.
                self.model.follow_mode = FollowMode::Click;
                self.webview.emit(SetOpenInNewWindow(false));
                self.set_mode("follow");
                self.follow_link();
            },
            Forward => self.webview.widget().go_forward(),
            GoParentDir(parent_level) => self.go_parent_directory(parent_level),
            GoRootDir => self.go_root_directory(),
            HideHints => self.hide_hints(),
            Hover => {
                // TODO: move that into a method.
                self.model.follow_mode = FollowMode::Hover;
                self.set_mode("follow");
                self.follow_link();
            },
            Insert => self.go_in_insert_mode(),
            Inspector => self.webview.emit(ShowInspector),
            Normal => self.go_in_normal_mode(),
            Open(ref url) => self.open(url),
            PasswordDelete => handle_error!(self.delete_password()),
            PasswordLoad => handle_error!(self.load_password()),
            PasswordSave => self.save_password(),
            PasswordSubmit => handle_error!(self.submit_login_form()),
            PasteUrl => self.paste_url(),
            Print => self.webview.emit(PagePrint),
            Quit => self.try_quit(),
            Reload => self.webview.widget().reload(),
            ReloadBypassCache => self.webview.widget().reload_bypass_cache(),
            Screenshot(ref path) => self.webview.emit(PageScreenshot(path.clone())),
            ScrollBottom => self.scroll_bottom(),
            ScrollDown => self.scroll_down_page(),
            ScrollDownHalf => self.scroll_down_half_page(),
            ScrollDownLine => self.scroll_down_line(),
            ScrollLeft => self.scroll_left(),
            ScrollRight => self.scroll_right(),
            ScrollTop => self.scroll_top(),
            ScrollUp => self.scroll_up_page(),
            ScrollUpHalf => self.scroll_up_half_page(),
            ScrollUpLine => self.scroll_up_line(),
            Search(ref input) => {
                self.webview.emit(SearchBackward(false));
                self.webview.emit(PageSearch(input.clone()));
            },
            SearchEngine(ref args) => self.add_search_engine(args),
            SearchNext => self.webview.emit(PageSearchNext),
            SearchPrevious => self.webview.emit(PageSearchPrevious),
            Stop => self.webview.widget().stop_loading(),
            UrlIncrement => self.url_increment(),
            UrlDecrement => self.url_decrement(),
            WinFollow => {
                // TODO: move that into a function.
                self.model.follow_mode = FollowMode::Click;
                self.webview.emit(SetOpenInNewWindow(true));
                self.set_mode("follow");
                self.follow_link();
            },
            WinOpen(ref url) => self.open_in_new_window(url),
            WinPasteUrl => self.win_paste_url(),
            ZoomIn => self.zoom_in(),
            ZoomNormal => self.zoom_normal(),
            ZoomOut => self.zoom_out(),
        }
    }

    /// Handle create window.
    fn handle_create(&mut self, action: NavigationAction) {
        if let Some(request) = action.get_request() {
            if let Some(url) = request.get_uri() {
                if action.get_navigation_type() == Other {
                    let mut should_handle_popup = false;
                    if let Some(ref mut popup_manager) = self.model.popup_manager {
                        if !popup_manager.is_whitelisted(&url) {
                            should_handle_popup = true;
                        }
                    }

                    if should_handle_popup {
                        self.handle_popup(&url);
                        return;
                    }
                }
                self.open_in_new_window(&url);
            }
        }
    }

    /// Show an error in the result is an error.
    fn handle_error(&self, error: Result<()>) {
        if let Err(error) = error {
            self.show_error(error);
        }
    }

    /// Handle the key press event.
    fn handle_key_press(&mut self, event_key: EventKey) {
        if self.model.mode == "follow" {
            self.handle_follow_key_press(event_key);
        }
    }

    /// Handle the load_changed event.
    /// Show the URL.
    /// Set the window title.
    /// Go back to normal mode.
    fn handle_load_changed(&mut self, load_event: LoadEvent) {
        if load_event == Started {
            self.webview.emit(EndSearch);
            self.webview.emit(AddStylesheets);
            self.webview.emit(AddScripts);

            // Check to mode to avoid going back to normal mode if the user is in command mode.
            if self.model.mode == "insert" || self.model.mode == "follow" {
                self.go_in_normal_mode();
            }
        }

        if load_event == Finished {
            self.set_title_without_progress();
        }
        else {
            self.set_title();
        }
    }

    fn inhibit_button_release(has_hovered_link: &Rc<Cell<bool>>, event: &EventButton) -> Inhibit {
        let inhibit = event.get_button() == LEFT_BUTTON && event.get_state().contains(CONTROL_MASK) &&
            has_hovered_link.get();
        Inhibit(inhibit)
    }

    fn inhibit_key_press(in_follow_mode: &Rc<Cell<bool>>) -> Inhibit {
        Inhibit(in_follow_mode.get())
    }

    /// Show an info.
    pub fn info(&self, info: String) {
        self.mg.emit(Info(info));
    }

    fn init_popup_manager(&mut self) {
        let result =
            if let Some(ref mut popup_manager) = self.model.popup_manager {
                popup_manager.load()
            }
            else {
                Ok(())
            };
        self.handle_error(result);
    }

    fn load_started(&mut self) {
        self.set_title();
        self.reset_scroll_element();
    }

    /// Handle the mouse target changed event of the webview to show the hovered URL and save it
    /// for use when using Ctrl-click.
    fn mouse_target_changed(&mut self, hit_test_result: HitTestResult) {
        let link = hit_test_result.get_link_uri();
        self.model.hovered_link = link.clone();
        self.model.has_hovered_link.set(link.is_some());
        {
            let text = link.unwrap_or_else(String::new);
            self.mg.emit(Message(text));
        }
    }

    fn set_mode(&mut self, mode: &'static str) {
        self.adjust_in_follow_mode(mode);
        self.mg.emit(SetMode(mode));
    }

    /// Try to close the web view and quit the application.
    fn try_quit(&self) {
        // Ask for a confirmation before quitting the application when there are active
        // downloads.
        if self.model.has_active_downloads {
            let msg = "There are active downloads. Do you want to quit?".to_string();
            yes_no_question(&self.mg, &self.model.relm, msg, Exit)
        }
        else {
            self.quit(true);
        }
    }

    /// Close the web view and quit the application if there's no download or the user chose to
    /// cancel them.
    fn quit(&self, can_quit: bool) {
        if can_quit {
            self.webview.widget().try_close();
        }
    }

    fn setting_changed(&mut self, setting: AppSettingsVariant) {
        match setting {
            HintChars(chars) => self.model.hint_chars = chars,
            HomePage(url) => {
                if  self.model.init_url.is_none() {
                    self.webview.emit(PageOpen(url.clone()));
                }
                self.model.home_page = Some(url);
            },
            _ => self.webview.emit(WebViewSettingChanged(setting)),
        }
    }

    /// Show an error.
    pub fn show_error(&self, error: errors::Error) {
        self.error(&error.to_string());
    }

    /// Show the zoom level in the status bar.
    fn show_zoom(&self, level: i32) {
        self.info(format!("Zoom level: {}%", level));
    }

    /// Handle the web process crashed event.
    fn web_process_crashed(&mut self) {
        self.error("The web process crashed.");
    }

    /// Zoom in.
    fn zoom_in(&self) {
        self.webview.emit(PageZoomIn);
    }

    /// Zoom back to 100%.
    fn zoom_normal(&self) {
        self.webview.emit(PageZoomNormal);
    }

    /// Zoom out.
    fn zoom_out(&self) {
        self.webview.emit(PageZoomOut);
    }
}
