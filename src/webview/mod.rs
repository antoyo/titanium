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
mod settings;

use std::fs::{File, read_dir};
use std::io::Read;

use cairo::{Context, Format, ImageSurface};
use glib::{Cast, ToVariant};
use gtk::{WidgetExt, Window};
use relm::{Relm, Resolver, Widget};
use relm_attributes::widget;
use webkit2gtk::{
    self,
    CookiePersistentStorage,
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
    WebInspector,
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
use app::AppResult;
use config_dir::ConfigDir;
use message_server::PATH;
use pass_manager::PasswordManager;
use self::Msg::*;
use settings::AppSettingsVariant;
use stylesheet::get_stylesheet_and_whitelist;

pub struct Model {
    config_dir: ConfigDir,
    inspector_shown: bool,
    open_in_new_window: bool,
    pub password_manager: PasswordManager,
    relm: Relm<WebView>,
    search_backwards: bool,
}

#[derive(Msg)]
pub enum Msg {
    AddScripts,
    AddStylesheets,
    Close,
    DecidePolicy(PolicyDecision, PolicyDecisionType, Resolver<bool>),
    DeletePassword,
    EndSearch,
    InspectorAttach(WebInspector, Resolver<bool>),
    InspectorClose,
    LoadUsernamePassword,
    NewWindow(String),
    PageFinishSearch,
    PageOpen(String),
    PagePrint,
    PageScreenshot(String),
    PageSearch(String),
    PageSearchNext,
    PageSearchPrevious,
    PageZoomIn,
    PageZoomNormal,
    PageZoomOut,
    SavePassword,
    SearchBackward(bool),
    SetOpenInNewWindow(bool),
    ShowInspector,
    SubmitLoginForm,
    WebViewSettingChanged(AppSettingsVariant),
    ZoomChange(i32),
}

#[widget]
impl Widget for WebView {
    fn model(relm: &Relm<Self>, config_dir: ConfigDir) -> Model {
        Model {
            config_dir,
            inspector_shown: false,
            open_in_new_window: false,
            password_manager: PasswordManager::new(),
            relm: relm.clone(),
            search_backwards: false,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            AddScripts => { let _ = self.add_scripts(); }, // TODO: handle error.
            AddStylesheets => { let _ = self.add_stylesheets(); }, // TODO: handle error.
            // To be listened by the user.
            Close => (),
            DecidePolicy(policy_decision, policy_decision_type, resolver) =>
                self.decide_policy(policy_decision, policy_decision_type, resolver),
            DeletePassword => self.delete_password(),
            EndSearch => { let _ = self.finish_search(); }, // TODO: handle error.
            InspectorAttach(inspector, resolver) => self.inspector_attach(inspector, resolver),
            InspectorClose => self.model.inspector_shown = false,
            LoadUsernamePassword => self.load_username_password(),
            // To be listened by the user.
            NewWindow(_) => (),
            PageFinishSearch => { let _ = self.finish_search(); }, // TODO: handle error.
            PageOpen(url) => self.open(url),
            PagePrint => self.print(),
            PageScreenshot(path) => self.screenshot(path),
            PageSearch(input) => { let _ = self.search(input); }, // TODO: handle error.
            PageSearchNext => { let _ = self.search_next(); }, // TODO: handle error.
            PageSearchPrevious => { let _ = self.search_previous(); }, // TODO: handle error.
            PageZoomIn => self.show_zoom(self.zoom_in()),
            PageZoomNormal => self.show_zoom(self.zoom_normal()),
            PageZoomOut => self.show_zoom(self.zoom_out()),
            SavePassword => self.save_password(),
            SearchBackward(search_backwards) => self.model.search_backwards = search_backwards,
            SetOpenInNewWindow(open_in_new_window) => self.set_open_in_new_window(open_in_new_window),
            ShowInspector => self.show_inspector(),
            SubmitLoginForm => { let _ = self.submit_login_form(); }, // TODO: handle error.
            WebViewSettingChanged(setting) => self.setting_changed(setting),
            // To be listened by the user.
            ZoomChange(_) => (),
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
            decide_policy(_, policy_decision, policy_decision_type) =>
                async DecidePolicy(policy_decision.clone(), policy_decision_type),
        }
    }
}

impl WebView {
    /// Add the user scripts.
    fn add_scripts(&self) -> AppResult<()> {
        if let Some(content_manager) = self.view.get_user_content_manager() {
            content_manager.remove_all_scripts();
            let script_path = self.model.config_dir.config_file("scripts")?;
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
    pub fn add_stylesheets(&self) -> AppResult<()> {
        if let Some(content_manager) = self.view.get_user_content_manager() {
            content_manager.remove_all_style_sheets();
            let stylesheets_path = self.model.config_dir.config_file("stylesheets")?;
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

    /// Handle the decide policy event.
    fn decide_policy(&mut self, policy_decision: PolicyDecision, policy_decision_type: PolicyDecisionType,
        mut resolver: Resolver<bool>)
    {
        if policy_decision_type == NavigationAction {
            if self.handle_navigation_action(policy_decision.clone()) {
                resolver.resolve(true);
            }
        }
        else if policy_decision_type == Response && self.handle_response(policy_decision.clone()) {
            resolver.resolve(true);
        }
    }

    /// Emit the new window event.
    pub fn emit_new_window_event(&self, url: &str) {
        self.model.relm.stream().emit(NewWindow(url.to_string()));
    }

    /// Get the find controller.
    fn find_controller(&self) -> AppResult<FindController> {
        // TODO: handle error.
        Ok(self.view.get_find_controller().expect("find controller"))
            //.ok_or(Box::new("cannot get find controller".to_string()))
    }

    /// Clear the current search.
    fn finish_search(&self) -> AppResult<()> {
        self.search(String::new())?;
        self.find_controller()?.search_finish();
        Ok(())
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

        // TODO: send a sequential number, i.e. the identifier for the current window.
        context.set_web_extensions_initialization_user_data(&PATH.to_variant());

        let cookie_path = config_dir.data_file("cookies")
            .expect("cannot create data directory");
        let cookie_manager = context.get_cookie_manager().unwrap();
        cookie_manager.set_persistent_storage(cookie_path.to_str().unwrap(), CookiePersistentStorage::Sqlite);

        context
    }

    fn inspector_attach(&mut self, inspector: WebInspector, mut resolver: Resolver<bool>) {
        if !self.model.inspector_shown {
            inspector.detach();
            resolver.resolve(true);
        }
        self.model.inspector_shown = true;
    }

    /// Open the specified URL.
    fn open(&self, url: String) {
        self.view.load_uri(&url);
    }

    /// Print the current page.
    fn print(&self) {
        let print_operation = PrintOperation::new(&self.view);
        let window = self.view.get_toplevel()
            .and_then(|toplevel| toplevel.downcast::<Window>().ok());
        print_operation.run_dialog(window.as_ref());
    }

    /// Save a screenshot of the web view.
    fn screenshot(&self, path: String) {
        let allocation = self.view.get_allocation();
        let surface = ImageSurface::create(Format::ARgb32, allocation.width, allocation.height);
        let context = Context::new(&surface);
        self.view.draw(&context);
        let mut file = File::create(path).unwrap();
        surface.write_to_png(&mut file).unwrap();
    }

    /// Search some text.
    fn search(&self, input: String) -> AppResult<()> {
        let default_options = FIND_OPTIONS_CASE_INSENSITIVE | FIND_OPTIONS_WRAP_AROUND;
        let other_options =
            if self.model.search_backwards {
                FIND_OPTIONS_BACKWARDS
            }
            else {
                FindOptions::empty()
            };
        let options = default_options | other_options;
        self.find_controller()?.search("", options.bits(), ::std::u32::MAX); // Clear previous search.
        self.find_controller()?.search(&input, options.bits(), ::std::u32::MAX);
        Ok(())
    }

    /// Search the next occurence of the search text.
    fn search_next(&self) -> AppResult<()> {
        if self.model.search_backwards {
            self.find_controller()?.search_previous();
        }
        else {
            self.find_controller()?.search_next();
        }
        Ok(())
    }

    /// Search the previous occurence of the search text.
    fn search_previous(&self) -> AppResult<()> {
        if self.model.search_backwards {
            self.find_controller()?.search_next();
        }
        else {
            self.find_controller()?.search_previous();
        }
        Ok(())
    }

    /// Set open in new window boolean to true to indicate that the next follow link will open a
    /// new window.
    fn set_open_in_new_window(&mut self, in_new_window: bool) {
        self.model.open_in_new_window = in_new_window;
    }

    /// Show the web inspector.
    fn show_inspector(&self) {
        if let Some(inspector) = self.view.get_inspector() {
            connect!(self.model.relm, inspector, connect_attach(inspector), async InspectorAttach(inspector.clone()));
            connect!(inspector, connect_closed(_), self.model.relm, InspectorClose);
            inspector.show();
        }
    }

    fn show_zoom(&self, level: i32) {
        self.model.relm.stream().emit(ZoomChange(level));
    }

    /// Zoom in.
    fn zoom_in(&self) -> i32 {
        let level = self.view.get_zoom_level();
        self.view.set_zoom_level(level + 0.1);
        (self.view.get_zoom_level() * 100.0) as i32
    }

    /// Zoom back to 100%.
    fn zoom_normal(&self) -> i32 {
        self.view.set_zoom_level(1.0);
        100
    }

    /// Zoom out.
    fn zoom_out(&self) -> i32 {
        let level = self.view.get_zoom_level();
        self.view.set_zoom_level(level - 0.1);
        (self.view.get_zoom_level() * 100.0) as i32
    }
}
