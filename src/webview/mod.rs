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

mod password;
mod scroll;
mod settings;

use std::error::Error;
use std::fs::{File, read_dir};
use std::io::Read;
use std::ops::Deref;
use std::mem;

use cairo::{Context, Format, ImageSurface};
use glib::{Cast, ToVariant};
use gtk::{WidgetExt, Window};
use libc::getpid;
use relm::{Component, Relm, Widget};
use relm_attributes::widget;
use webkit2gtk::{
    self,
    CookiePersistentStorage,
    Download,
    FindController,
    FindOptions,
    NavigationPolicyDecision,
    PolicyDecision,
    PolicyDecisionExt,
    PrintOperation,
    ResponsePolicyDecision,
    UserContentManager,
    UserScript,
    UserStyleSheet,
    WebContext,
    WebViewExt,
    FIND_OPTIONS_BACKWARDS,
    FIND_OPTIONS_CASE_INSENSITIVE,
    FIND_OPTIONS_WRAP_AROUND,
};
use webkit2gtk::NavigationType::LinkClicked;
use webkit2gtk::PolicyDecisionType::{self, NavigationAction, Response};
use webkit2gtk::UserContentInjectedFrames::AllFrames;
use webkit2gtk::UserScriptInjectionTime::End;
use webkit2gtk::UserStyleLevel::User;

// TODO: remove coupling between webview and app modules.
use app::{AppResult, FollowMode};
use config_dir::ConfigDir;
use message_server::{MessageServer, PATH};
use message_server::Msg::MsgRecv;
use pass_manager::PasswordManager;
use self::Msg::*;
use stylesheet::get_stylesheet_and_whitelist;
use titanium_common::Message;
use titanium_common::Message::*;

pub struct Model {
    client: usize,
    config_dir: ConfigDir,
    find_controller: FindController,
    follow_mode: FollowMode,
    message_server: Component<MessageServer>,
    open_in_new_window: bool,
    pub password_manager: PasswordManager,
    relm: Relm<WebView>,
    scrolled_callback: Option<Box<Fn(i64)>>,
    search_backwards: bool,
}

#[derive(Msg)]
pub enum Msg {
    Action(i32),
    ClickElement,
    Close,
    GoToInsertMode,
    NewWindow(String),
    Scroll(i64),
}

#[widget]
impl Widget for WebView {
    fn init_view(&mut self) {
        self.model.find_controller = self.view.get_find_controller().unwrap();
        let message_server = &self.model.message_server;
        connect!(message_server@MsgRecv(_, ref msg), self.model.relm, match *msg {
            ActivateAction(action) => Some(Action(action)),
            ClickHintElement() => Some(ClickElement),
            Credentials(_, _) => None, // TODO
            EnterInsertMode() => Some(GoToInsertMode),
            ScrollPercentage(percentage) => Some(Scroll(percentage)),
            _ => {
                warn!("Unexpected message received: {:?}", msg);
                None
            },
        });
    }

    fn model(relm: &Relm<Self>, config_dir: ConfigDir) -> Model {
        Model {
            client: 0, // TODO: real client ID.
            config_dir,
            find_controller: unsafe { mem::uninitialized() }, // TODO: remove uninitialized().
            follow_mode: FollowMode::Click,
            message_server: MessageServer::new().unwrap(), // TODO: handle error elsewhere.
            open_in_new_window: false,
            password_manager: PasswordManager::new(),
            relm: relm.clone(),
            scrolled_callback: None,
            search_backwards: false,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            ClickElement => {
                let result = self.activate_hint();
                self.hide_hints();
            },
            _ => ()
        }
    }

    view! {
        #[name="view"]
        webkit2gtk::WebView({
            user_content_manager: UserContentManager::new(), // FIXME: seems to be deallocated.
            web_context: WebView::initialize_web_extension(&self.model.config_dir)
        }) {
            close => Close,
            vexpand: true,
            // Emit the scroll event whenever the view is drawn.
            draw(_, _) => return self.emit_scrolled_event(),
            decide_policy(_, policy_decision, policy_decision_type) =>
                return self.decide_policy(policy_decision, policy_decision_type),
        }
    }
}

impl WebView {
    /// Activate the selected hint.
    pub fn activate_hint(&self) -> AppResult<()> {
        self.view.grab_focus();
        self.server_send(ActivateHint(self.model.follow_mode.to_string()))
    }

    /// Activate the link in the selection
    pub fn activate_selection(&self) -> AppResult<()> {
        self.server_send(ActivateSelection())
    }

    /// Add the user scripts.
    pub fn add_scripts(&self, config_dir: &ConfigDir) -> AppResult<()> {
        if let Some(content_manager) = self.view.get_user_content_manager() {
            content_manager.remove_all_scripts();
            let script_path = config_dir.config_file("scripts")?;
            for filename in read_dir(script_path)? {
                let mut file = File::open(filename?.path())?;
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                // TODO: support whitelist as a comment in the script.
                let script = UserScript::new(&content, AllFrames, End, &[], &[]);
                content_manager.add_script(&script);
            }
        }
        Ok(())
    }

    /// Add the user stylesheets.
    pub fn add_stylesheets(&self, config_dir: &ConfigDir) -> AppResult<()> {
        if let Some(content_manager) = self.view.get_user_content_manager() {
            content_manager.remove_all_style_sheets();
            let stylesheets_path = config_dir.config_file("stylesheets")?;
            for filename in read_dir(stylesheets_path)? {
                let mut file = File::open(filename?.path())?;
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                let (stylesheet, stylesheet_whitelist) = get_stylesheet_and_whitelist(&content);
                let whitelist: Vec<_> = stylesheet_whitelist.iter().map(|url| url.as_ref()).collect();
                let stylesheet = UserStyleSheet::new(&stylesheet, AllFrames, User, &whitelist, &[]);
                content_manager.add_style_sheet(&stylesheet);
            }
        }
        Ok(())
    }

    /// Add a callback for the download started event.
    pub fn connect_download_started<F: Fn(&WebContext, &Download) + 'static>(&self, callback: F) {
        if let Some(context) = self.view.get_context() {
            context.connect_download_started(callback);
        }
    }

    /// Handle the decide policy event.
    fn decide_policy(&mut self, policy_decision: &PolicyDecision, policy_decision_type: PolicyDecisionType) -> bool {
        if policy_decision_type == NavigationAction {
            if self.handle_navigation_action(policy_decision.clone()) {
                return true;
            }
        }
        else if policy_decision_type == Response && self.handle_response(policy_decision.clone()) {
            return true;
        }
        false
    }

    /// Emit the new window event.
    pub fn emit_new_window_event(&self, url: &str) {
        self.model.relm.stream().emit(NewWindow(url.to_string()));
    }

    /// Send a key to the web process to process with the current hints.
    pub fn enter_hint_key(&self, key_char: char) -> AppResult<()> {
        self.server_send(EnterHintKey(key_char))
    }

    /// Clear the current search.
    pub fn finish_search(&self) {
        self.search("");
        self.model.find_controller.search_finish();
    }

    /// Focus the first input element.
    pub fn focus_input(&self) -> AppResult<()> {
        self.view.grab_focus();
        self.server_send(FocusInput())
    }

    /// Follow a link.
    pub fn follow_link(&self, hint_chars: &str) -> AppResult<()> {
        self.server_send(ShowHints(hint_chars.to_string()))
    }

    /// Get the web context.
    pub fn get_context(&self) -> WebContext {
        self.view.get_context().expect("No web context")
    }

    /// Handle follow link in new window.
    fn handle_navigation_action(&mut self, policy_decision: PolicyDecision) -> bool {
        let policy_decision = policy_decision.clone();
        if let Ok(policy_decision) = policy_decision.downcast::<NavigationPolicyDecision>() {
            if self.model.open_in_new_window && policy_decision.get_navigation_type() == LinkClicked {
                let url = policy_decision.get_request()
                    .and_then(|request| request.get_uri());
                if let Some(url) = url {
                    policy_decision.ignore();
                    self.model.open_in_new_window = false;
                    self.emit_new_window_event(&url);
                    return true;
                }
            }
        }
        false
    }

    /// Download file whose mime type is not supported.
    fn handle_response(&self, policy_decision: PolicyDecision) -> bool {
        let policy_decision = policy_decision.clone();
        if let Ok(policy_decision) = policy_decision.downcast::<ResponsePolicyDecision>() {
            if !policy_decision.is_mime_type_supported() {
                policy_decision.download();
                return true;
            }
        }
        false
    }

    /// Hide the hints.
    pub fn hide_hints(&self) -> AppResult<()> {
        self.server_send(HideHints())
    }

    /// Create the context and initialize the web extension.
    fn initialize_web_extension(config_dir: &ConfigDir) -> WebContext {
        let context = WebContext::get_default().unwrap();
        if cfg!(debug_assertions) {
            context.set_web_extensions_directory("titanium-web-extension/target/debug");
        }
        else {
            let install_path = env!("TITANIUM_EXTENSION_INSTALL_PATH");
            context.set_web_extensions_directory(install_path);
        }

        //let pid = unsafe { getpid() };
        //let server_name = format!("com.titanium.process{}", pid);

        context.set_web_extensions_initialization_user_data(&PATH.to_variant());

        let cookie_path = config_dir.data_file("cookies")
            .expect("cannot create data directory");
        let cookie_manager = context.get_cookie_manager().unwrap();
        cookie_manager.set_persistent_storage(cookie_path.to_str().unwrap(), CookiePersistentStorage::Sqlite);

        context
    }

    /// Open the specified URL.
    pub fn open(&self, url: &str) {
        self.view.load_uri(&url);
    }

    /// Print the current page.
    pub fn print(&self) {
        let print_operation = PrintOperation::new(&self.view);
        let window = self.view.get_toplevel()
            .and_then(|toplevel| toplevel.downcast::<Window>().ok());
        print_operation.run_dialog(window.as_ref());
    }

    /// Save a screenshot of the web view.
    pub fn screenshot(&self, path: &str) {
        let allocation = self.view.get_allocation();
        let surface = ImageSurface::create(Format::ARgb32, allocation.width, allocation.height);
        let context = Context::new(&surface);
        self.view.draw(&context);
        let mut file = File::create(path).unwrap();
        surface.write_to_png(&mut file).unwrap();
    }

    /// Search some text.
    pub fn search(&self, input: &str) {
        let default_options = FIND_OPTIONS_CASE_INSENSITIVE | FIND_OPTIONS_WRAP_AROUND;
        let other_options =
            if self.model.search_backwards {
                FIND_OPTIONS_BACKWARDS
            }
            else {
                FindOptions::empty()
            };
        let options = default_options | other_options;
        self.model.find_controller.search("", options.bits(), ::std::u32::MAX); // Clear previous search.
        self.model.find_controller.search(input, options.bits(), ::std::u32::MAX);
    }

    /// Search the next occurence of the search text.
    pub fn search_next(&self) {
        if self.model.search_backwards {
            self.model.find_controller.search_previous();
        }
        else {
            self.model.find_controller.search_next();
        }
    }

    /// Search the previous occurence of the search text.
    pub fn search_previous(&self) {
        if self.model.search_backwards {
            self.model.find_controller.search_next();
        }
        else {
            self.model.find_controller.search_previous();
        }
    }

    /// Set the value of an input[type="file"].
    pub fn select_file(&self, file: String) -> AppResult<()> {
        self.server_send(SelectFile(file))
    }

    pub fn set_follow_mode(&mut self, follow_mode: FollowMode) {
        self.model.follow_mode = follow_mode;
    }

    fn server_send(&self, message: Message) -> AppResult<()> {
        // TODO: rename widget_mut().
        self.model.message_server.widget_mut().send(self.model.client, message)
            .map_err(From::from)
    }

    /// Set open in new window boolean to true to indicate that the next follow link will open a
    /// new window.
    pub fn set_open_in_new_window(&mut self, in_new_window: bool) {
        self.model.open_in_new_window = in_new_window;
    }

    /// Set whether the search is backward or not.
    pub fn set_search_backward(&mut self, backward: bool) {
        self.model.search_backwards = backward;
    }

    /// Show the web inspector.
    pub fn show_inspector(&self) {
        static mut SHOWN: bool = false;
        if let Some(inspector) = self.view.get_inspector() {
            inspector.connect_attach(|inspector| {
                unsafe {
                    if !SHOWN {
                        inspector.detach();
                        SHOWN = true;
                        return true;
                    }
                    SHOWN = true;
                }
                false
            });
            inspector.connect_closed(|_| {
                unsafe {
                    SHOWN = false;
                }
            });
            inspector.show();
        }
    }

    /// Zoom in.
    pub fn zoom_in(&self) -> i32 {
        let level = self.view.get_zoom_level();
        self.view.set_zoom_level(level + 0.1);
        (self.view.get_zoom_level() * 100.0) as i32
    }

    /// Zoom back to 100%.
    pub fn zoom_normal(&self) -> i32 {
        self.view.set_zoom_level(1.0);
        100
    }

    /// Zoom out.
    pub fn zoom_out(&self) -> i32 {
        let level = self.view.get_zoom_level();
        self.view.set_zoom_level(level - 0.1);
        (self.view.get_zoom_level() * 100.0) as i32
    }
}

impl Deref for WebView {
    type Target = webkit2gtk::WebView;

    fn deref(&self) -> &webkit2gtk::WebView {
        &self.view
    }
}
