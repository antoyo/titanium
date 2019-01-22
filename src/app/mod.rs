/*
 * Copyright (c) 2016-2019 Boucher, Antoni <bouanto@zoho.com>
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

mod adblock;
mod bookmarks;
mod browser;
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
pub mod user_agent;

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use gdk::{EventKey, Rectangle, RGBA};
use glib::Cast;
use gtk::{
    self,
    Inhibit,
    OrientableExt,
    WidgetExt,
};
use gtk::Orientation::Vertical;
use mg::{
    AppClose,
    CloseWin,
    Color,
    Completers,
    CompletionViewChange,
    CustomCommand,
    DarkTheme,
    DialogResult,
    Error,
    Info,
    Message,
    Mg,
    Mode,
    ModeChanged,
    Modes,
    SetMode,
    SettingChanged,
    StatusBarItem,
    Text,
    Title,
    question,
    yes_no_question,
};
use relm::{Relm, Widget};
use relm_attributes::widget;
use webkit2gtk::{
    self,
    Download,
    GeolocationPermissionRequest,
    HitTestResult,
    HitTestResultExt,
    NavigationAction,
    NotificationPermissionRequest,
    PermissionRequestExt,
    URIRequestExt,
    UserMediaPermissionRequest,
    UserMediaPermissionRequestExt,
    WebContext,
    WebViewExt,
};
use webkit2gtk::LoadEvent::{self, Started};
use webkit2gtk::NavigationType::Other;

use titanium_common::{FollowMode, InnerMessage, PageId, LAST_MARK};
use titanium_common::Percentage::{self, All, Percent};

use bookmarks::BookmarkManager;
use commands::AppCommand;
use commands::AppCommand::*;
use completers::{
    BookmarkCompleter,
    FileCompleter,
    TagCompleter,
    UserAgentCompleter,
};
use config_dir::ConfigDir;
use download_list_view::DownloadListView;
use download_list_view::Msg::{
    ActiveDownloads,
    DownloadListError,
};
use errors::Result;
use message_server::Privacy;
use pass_manager::PasswordManager;
use permission_manager::{Permission, PermissionManager, create_permission_manager};
use popup_manager::{PopupManager, create_popup_manager};
use self::config::default_config;
use self::dialog::handle_script_dialog;
use self::file_chooser::handle_file_chooser;
use self::Msg::*;
use self::user_agent::UserAgentManager;
use settings::AppSettings;
use settings::AppSettingsVariant::{
    self,
    HintChars,
    HomePage,
    WebkitUserAgent,
};
use urls::canonicalize_url;
use webview::WebView;
use webview::Msg::{
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
    PermissionRequest,
    SearchBackward,
    SetOpenInNewWindow,
    ShowInspector,
    WebPageId,
    WebViewSettingChanged,
    ZoomChange,
};

pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
const INIT_SCROLL_TEXT: &str = "[top]";
const RED: RGBA = RGBA { red: 1.0, green: 0.3, blue: 0.2, alpha: 1.0 };
const YELLOW: RGBA = RGBA { red: 1.0, green: 1.0, blue: 0.0, alpha: 1.0 };
const TAG_COMPLETER: &str = "__tag";
pub const USER_AGENT_COMPLETER: &str = "select-user-agent";

static MODES: Modes = &[
    Mode { name: "follow", prefix: "f", show_count: false },
    Mode { name: "insert", prefix: "i", show_count: false },
];

pub struct Model {
    bookmark_manager: BookmarkManager,
    command_text: String,
    config_dir: ConfigDir,
    current_url: String,
    default_search_engine: Option<String>,
    follow_mode: FollowMode,
    has_active_downloads: bool,
    hint_chars: String,
    home_page: Option<String>,
    in_follow_mode: Rc<Cell<bool>>,
    init_url: Option<String>,
    mode: String,
    open_in_new_window: bool,
    password_manager: PasswordManager,
    overridden_color: Option<RGBA>,
    permission_manager: Option<PermissionManager>,
    popup_manager: Option<PopupManager>,
    relm: Relm<App>,
    scroll_text: String,
    search_engines: HashMap<String, String>,
    title: String,
    user_agents: HashMap<String, String>,
    user_agent_manager: UserAgentManager,
    web_context: WebContext,
}

#[derive(Msg)]
pub enum Msg {
    AppSetMode(String),
    AppSettingChanged(AppSettingsVariant),
    AskPermission(webkit2gtk::PermissionRequest),
    Create(NavigationAction),
    Command(AppCommand),
    CommandText(String),
    CreateWindow(String, Privacy),
    DecideDownloadDestination(Download, String),
    DownloadDestination(DialogResult, Download, String),
    Exit(bool),
    FileDialogSelection(Option<String>),
    HasActiveDownloads(bool),
    HostfileDownloaded(String, Download),
    InsecureContent,
    KeyPress(EventKey),
    LoadChanged(LoadEvent),
    MessageRecv(InnerMessage),
    MouseTargetChanged(HitTestResult),
    OverwriteDownload(Download, String, bool),
    SetPageId(PageId),
    PermissionResponse(webkit2gtk::PermissionRequest, Option<String>),
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
    /// Handle the load_changed event.
    /// Show the URL.
    /// Set the window title.
    /// Go back to normal mode.
    fn handle_load_changed(&mut self, load_event: LoadEvent) {
        if load_event == Started {
            self.model.overridden_color = None;
            self.model.scroll_text = INIT_SCROLL_TEXT.to_string();
            self.webview.emit(EndSearch);
            self.webview.emit(AddStylesheets);
            self.webview.emit(AddScripts);

            // Check to mode to avoid going back to normal mode if the user is in command mode.
            if self.model.mode == "insert" || self.model.mode == "follow" {
                self.go_in_normal_mode();
            }
        }
        else {
            if let Some((_, cert_flags)) = self.webview.widget().get_tls_info() {
                // If there's a certificate error, show the URL in red.
                if !cert_flags.is_empty() {
                    self.model.overridden_color = Some(RED);
                }
            }
        }
    }

    fn init_view(&mut self) {
        handle_error!(self.model.bookmark_manager.create_tables());

        match App::bookmark_path(&self.model.config_dir) {
            Ok(bookmark_path) => handle_error!(self.model.bookmark_manager.connect(bookmark_path)),
            Err(error) => self.error(&error.to_string()),
        }

        handle_error!(self.clean_download_folder());
        self.init_permission_manager();
        self.init_popup_manager();
        self.open_init_url();
        self.connect_dialog_events();
        self.connect_download_events();
        self.create_variables();
    }

    fn insecure_content_detected(&mut self) {
        // Only show the URL in yellow if there's not already a certificate error.
        if self.model.overridden_color.is_none() {
            self.model.overridden_color = Some(YELLOW);
        }
    }

    fn model(relm: &Relm<Self>, (init_url, config_dir, web_context): (Option<String>, ConfigDir, WebContext)) -> Model {
        let permission_manager = create_permission_manager(&config_dir);
        let popup_manager = create_popup_manager(&config_dir);
        Model {
            bookmark_manager: BookmarkManager::new(),
            command_text: String::new(),
            config_dir,
            current_url: String::new(),
            default_search_engine: None,
            follow_mode: FollowMode::Click,
            has_active_downloads: false,
            hint_chars: "hjklasdfgyuiopqwertnmzxcvb".to_string(),
            home_page: None,
            in_follow_mode: Rc::new(Cell::new(false)),
            init_url,
            mode: "normal".to_string(),
            open_in_new_window: false,
            password_manager: PasswordManager::new(),
            overridden_color: None,
            permission_manager,
            popup_manager,
            relm: relm.clone(),
            scroll_text: INIT_SCROLL_TEXT.to_string(),
            search_engines: HashMap::new(),
            title: APP_NAME.to_string(),
            user_agents: HashMap::new(),
            user_agent_manager: UserAgentManager,
            web_context,
        }
    }

    fn open_init_url(&self) {
        if let Some(ref url) = self.model.init_url {
            // Open as a file if the path exist, otherwise open as a normal URL.
            let url = canonicalize_url(url);
            self.webview.emit(PageOpen(url));
        }
    }

    /// Set the title of the window as the progress and the web page title.
    fn set_title(&mut self) {
        let private = self.private_text();
        let progress = (self.webview.widget().get_estimated_load_progress() * 100.0) as i32;
        if progress == 100 {
            self.set_title_without_progress();
        }
        else {
            let title = self.get_title();
            self.model.title = format!("[{}%]{} {}{}", progress, private, title, APP_NAME);
        }
    }

    /// Set the title of the window as the web page title or url.
    fn set_title_without_progress(&mut self) {
        let private = self.private_text();
        let title = self.get_title();
        self.model.title = format!("{}{}{}", private, title, APP_NAME);
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
            AskPermission(request) => self.handle_permission_request(&request),
            Create(navigation_action) => self.handle_create(navigation_action),
            Command(ref command) => self.handle_command(command),
            CommandText(text) => self.model.command_text = text,
            DecideDownloadDestination(download, suggested_filename) =>
                self.download_input(download, suggested_filename),
            DownloadDestination(destination, download, suggested_filename) =>
                handle_error!(self.download_destination_chosen(destination, download, suggested_filename)),
            Exit(can_quit) => self.quit(can_quit),
            FileDialogSelection(file) => self.file_dialog_selection(file),
            HasActiveDownloads(active) => self.model.has_active_downloads = active,
            HostfileDownloaded(file, download) => handle_error!(self.process_hostfile(&file, download)),
            InsecureContent => self.insecure_content_detected(),
            KeyPress(event_key) => self.handle_key_press(event_key),
            LoadChanged(load_event) => self.handle_load_changed(load_event),
            MessageRecv(message) => self.message_recv(message),
            MouseTargetChanged(hit_test_result) => self.mouse_target_changed(hit_test_result),
            // To be listened by the user.
            CreateWindow(_, _) => (),
            OverwriteDownload(download, download_destination, overwrite) =>
                self.overwrite_download(download, download_destination, overwrite),
            PopupDecision(answer, url) => self.handle_answer(answer.as_ref().map(|str| str.as_str()), &url),
            PermissionResponse(request, choice) => self.handle_permission_response(&request, choice),
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
                "private-win-open" => Box::new(BookmarkCompleter::new("private-win-open")),
                TAG_COMPLETER => Box::new(TagCompleter::new()),
                USER_AGENT_COMPLETER => Box::new(UserAgentCompleter::new()),
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
                    PermissionRequest(ref request) => AskPermission(request.clone()),
                    WebPageId(page_id) => SetPageId(page_id),
                    ZoomChange(level) => ShowZoom(level),
                    create(_, action) => (Create(action.clone()), None),
                    insecure_content_detected(_, _) => InsecureContent,
                    load_changed(_, load_event) => LoadChanged(load_event),
                    mouse_target_changed(_, hit_test_result, _) => MouseTargetChanged(hit_test_result.clone()),
                    property_estimated_load_progress_notify(_) => TitleChanged,
                    property_title_notify(_) => TitleChanged,
                    property_uri_notify(_) => UriChanged,
                    web_process_crashed => (WebProcessCrashed, false),
                },
            },
            #[name="scroll_label"]
            StatusBarItem {
                Text: self.model.scroll_text.clone(),
            },
            StatusBarItem {
                Color: self.model.overridden_color.clone(),
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
    fn add_mark(&mut self, mark: &str) {
        let mark = mark_from_str(mark);
        self.server_send(InnerMessage::Mark(mark));
        self.mg.emit(Info(format!("Added mark {}", mark as char)));
    }

    fn add_user_agent(&mut self, user_agent: &str) {
        let mut params = user_agent.splitn(2, ' ');
        if let (Some(name), Some(user_agent)) = (params.next(), params.next()) {
            self.model.user_agents.insert(name.to_string(), user_agent.to_string());
            self.model.user_agent_manager.add(name);
        }
    }

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

    fn follow(&mut self) {
        self.model.follow_mode = FollowMode::Click;
        self.model.open_in_new_window = false;
        self.webview.emit(SetOpenInNewWindow(false));
        self.set_mode("follow");
        self.follow_link();
    }

    /// Get the size of the webview.
    fn get_webview_allocation(&self) -> Rectangle {
        self.webview.widget().get_allocation()
    }

    /// Get the title or the url if there are no title.
    fn get_title(&self) -> String {
        let webview = self.webview.widget();
        let title = webview.get_title()
            .and_then(|title|
                if title.is_empty() {
                    None
                }
                else {
                    Some(title)
                })
            .or_else(|| webview.get_uri())
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

    fn go_to_mark(&mut self, mark: &str) {
        let mark = mark_from_str(mark);
        self.server_send(InnerMessage::GoToMark(mark));
    }

    /// Handle the command.
    fn handle_command(&mut self, command: &AppCommand) {
        match *command {
            ActivateSelection => self.activate_selection(),
            AdblockUpdate => handle_error!(self.adblock_update()),
            AddUserAgent(ref user_agent) => self.add_user_agent(user_agent),
            Back => self.history_back(),
            BackwardSearch(ref input) => {
                self.webview.emit(SearchBackward(true));
                self.webview.emit(PageSearch(input.clone()));
            },
            Bookmark => self.bookmark(),
            BookmarkDel => self.delete_bookmark(),
            BookmarkEditTags => self.edit_bookmark_tags(),
            ClearCache => self.clear_cache(),
            ClickNextPage => self.click_next_page(),
            ClickPrevPage => self.click_prev_page(),
            CopyLinkUrl => self.copy_link_url(),
            CopyUrl => self.copy_current_url(),
            DeleteAllCookies => self.delete_all_cookies(),
            DeleteCookies(ref domain) => self.delete_cookies(domain),
            DeleteSelectedBookmark => self.delete_selected_bookmark(),
            FinishSearch => self.webview.emit(PageFinishSearch),
            FocusInput => self.focus_input(),
            Follow => self.follow(),
            Forward => self.history_forward(),
            GoMark(ref mark) => self.go_to_mark(mark),
            GoParentDir(parent_level) => self.go_parent_directory(parent_level),
            GoRootDir => self.go_root_directory(),
            HideHints => self.hide_hints(),
            Hover => self.hover(),
            Insert => self.go_in_insert_mode(),
            Inspector => self.webview.emit(ShowInspector),
            KillWin => self.close_webview(),
            Mark(ref mark) => self.add_mark(mark),
            Normal => self.go_in_normal_mode(),
            Open(ref url) => self.open(url),
            PasswordDelete => handle_error!(self.delete_password()),
            PasswordInsert => handle_error!(self.insert_password()),
            PasswordInsertSubmit => handle_error!(self.insert_password_submit()),
            PasswordLoad => handle_error!(self.load_password()),
            PasswordSave => self.save_password(),
            PasswordSubmit => handle_error!(self.submit_login_form()),
            PasteUrl => self.paste_url(),
            Print => self.webview.emit(PagePrint),
            PrivateWinOpen(ref url) => self.open_in_new_window(url, Privacy::Private),
            Quit => self.try_quit(),
            Reload => self.webview.widget().reload(),
            ReloadBypassCache => self.webview.widget().reload_bypass_cache(),
            SaveLink => self.save_link(),
            Screenshot(ref path) => self.webview.emit(PageScreenshot(path.clone())),
            ScrollDown => self.scroll_down_page(),
            ScrollDownHalf => self.scroll_down_half_page(),
            ScrollDownLine => self.scroll_down_line(),
            ScrollLeft => self.scroll_left(),
            ScrollRight => self.scroll_right(),
            ScrollTo(percent) => self.scroll_to(percent),
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
            SelectUserAgent(ref name) => self.select_user_agent(name),
            Stop => self.webview.widget().stop_loading(),
            UrlIncrement => self.url_increment(),
            UrlDecrement => self.url_decrement(),
            WinFollow => self.win_follow(),
            WinOpen(ref url) => self.open_in_new_window(url, Privacy::Normal),
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
                self.open_in_new_window(&url, Privacy::Normal);
            }
        }
    }

    /// Show an error in the result is an error.
    fn handle_error(&self, error: Result<()>) {
        if let Err(error) = error {
            self.error(&error.to_string());
        }
    }

    /// Handle the key press event.
    fn handle_key_press(&mut self, event_key: EventKey) {
        if self.model.mode == "follow" {
            self.handle_follow_key_press(event_key);
        }
    }

    fn handle_permission_request(&mut self, request: &webkit2gtk::PermissionRequest) {
        if let Some(url) = self.webview.widget().get_uri() {
            if let Some(ref mut permission_manager) = self.model.permission_manager {
                if permission_manager.is_blacklisted(&url, request) {
                    request.deny();
                    return;
                }
                if permission_manager.is_whitelisted(&url, request) {
                    request.allow();
                    return;
                }
            }

            let msg =
                if request.is::<GeolocationPermissionRequest>() {
                    "This page wants to know your location."
                }
                else if request.is::<NotificationPermissionRequest>() {
                    "This page wants to show desktop notifications."
                }
                else if let Ok(media_permission) = request.clone().downcast::<UserMediaPermissionRequest>() {
                    if media_permission.get_property_is_for_video_device() {
                        "This page wants to use your webcam."
                    }
                    else {
                        "This page wants to use your microphone."
                    }
                }
                else {
                    // TODO: log.
                    return;
                };
            let request = request.clone();
            question(&self.mg, &self.model.relm, msg.to_string(),
                char_slice!['y', 'n', 'a', 'e'], move |choice| PermissionResponse(request.clone(), choice));
        }
    }

    fn handle_permission_response(&mut self, request: &webkit2gtk::PermissionRequest, choice: Option<String>) {
        if let Some(url) = self.webview.widget().get_uri() {
            match choice.as_ref().map(String::as_str) {
                Some("y") | Some("a") => request.allow(),
                _ => request.deny(),
            }

            match choice.as_ref().map(String::as_str) {
                Some("a") => self.remember_permission(Permission::Always, request, &url),
                Some("e") => self.remember_permission(Permission::Never, request, &url),
                _ => (),
            }
        }
    }

    fn remember_permission(&mut self, permission: Permission, request: &webkit2gtk::PermissionRequest, url: &str) {
        let result =
            if let Some(ref mut permission_manager) = self.model.permission_manager {
                match permission {
                    Permission::Always => permission_manager.whitelist(url, request),
                    Permission::Never => permission_manager.blacklist(url, request),
                }
            }
            else {
                Ok(())
            };
        self.handle_error(result);
    }

    fn history_back(&mut self) {
        self.webview.widget().go_back();
        self.server_send(InnerMessage::ResetScrollElement());
    }

    fn history_forward(&mut self) {
        self.webview.widget().go_forward();
        self.server_send(InnerMessage::ResetScrollElement());
    }

    fn hover(&mut self) {
        self.model.follow_mode = FollowMode::Hover;
        self.set_mode("follow");
        self.follow_link();
    }

    fn inhibit_key_press(in_follow_mode: &Rc<Cell<bool>>) -> Inhibit {
        Inhibit(in_follow_mode.get())
    }

    /// Show an info.
    pub fn info(&self, info: String) {
        self.mg.emit(Info(info));
    }

    fn init_permission_manager(&mut self) {
        let result =
            if let Some(ref mut permission_manager) = self.model.permission_manager {
                permission_manager.load()
            }
            else {
                Ok(())
            };
        self.handle_error(result);
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

    /// Handle the mouse target changed event of the webview to show the hovered URL and save it
    /// for use when using Ctrl-click.
    fn mouse_target_changed(&mut self, hit_test_result: HitTestResult) {
        let link = hit_test_result.get_link_uri();
        {
            let text = link.unwrap_or_else(String::new);
            self.mg.emit(Message(text));
        }
    }

    fn private_text(&self) -> &'static str {
        if self.webview.widget().is_ephemeral() {
            "[PV] "
        }
        else {
            ""
        }
    }

    /// Close the web view and quit the application if there's no download or the user chose to
    /// cancel them.
    fn quit(&self, can_quit: bool) {
        if can_quit {
            self.webview.widget().try_close();
        }
    }

    fn select_user_agent(&mut self, name: &str) {
        if let Some(user_agent) = self.model.user_agents.get(name).cloned() {
            self.info(format!("Set user agent to: {}", user_agent));
            self.setting_changed(WebkitUserAgent(user_agent));
        }
    }

    fn set_mode(&mut self, mode: &'static str) {
        self.adjust_in_follow_mode(mode);
        self.mg.emit(SetMode(mode));
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

    /// Show the zoom level in the status bar.
    fn show_zoom(&self, level: i32) {
        self.info(format!("Zoom level: {}%", level));
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

    /// Handle the web process crashed event.
    fn web_process_crashed(&mut self) {
        self.error("The web process crashed.");
    }

    fn win_follow(&mut self) {
        self.model.follow_mode = FollowMode::Click;
        self.model.open_in_new_window = true;
        self.webview.emit(SetOpenInNewWindow(true));
        self.set_mode("follow");
        self.follow_link();
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

fn mark_from_str(mark: &str) -> u8 {
    mark.as_bytes().get(0)
            .cloned()
            .unwrap_or(LAST_MARK)

}
