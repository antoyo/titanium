/*
 * Copyright (c) 2016-2018 Boucher, Antoni <bouanto@zoho.com>
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

macro_rules! handle_app_error {
    ($app:ident . $($tt:tt)* ) => {{
        let result = $app.$($tt)*;
        if let Err(error) = result {
            $app.model.relm.stream().emit(AppError(error.to_string()));
        }
    }};
}

mod settings;

use std::cell::Cell;
use std::fs::{File, read_dir};
use std::io::Read;
use std::rc::Rc;

use cairo::{Context, Format, ImageSurface};
use glib::Cast;
use gtk::{traits::WidgetExt, Window};
use relm::{Relm, Widget};
use relm_derive::widget;
use webkit2gtk::{
    self,
    CookieManagerExt,
    CookiePersistentStorage,
    FindController,
    FindControllerExt,
    FindOptions,
    NavigationPolicyDecision,
    NavigationPolicyDecisionExt,
    PermissionRequest,
    PolicyDecision,
    PolicyDecisionExt,
    PrintOperation,
    PrintOperationExt,
    ResponsePolicyDecision,
    ResponsePolicyDecisionExt,
    TLSErrorsPolicy,
    URIRequestExt,
    UserContentManager,
    UserContentManagerExt,
    UserScript,
    UserStyleSheet,
    WebContext,
    WebContextExt,
    WebInspector,
    WebInspectorExt,
    WebViewExt, WebsitePolicies, AutoplayPolicy,
};
use webkit2gtk::NavigationType::{LinkClicked, Other};
use webkit2gtk::PolicyDecisionType::{self, NavigationAction, Response};
use webkit2gtk::ProcessModel::MultipleSecondaryProcesses;
use webkit2gtk::UserContentInjectedFrames::AllFrames;
use webkit2gtk::UserScriptInjectionTime::End;
use webkit2gtk::UserStyleLevel::User;

use config_dir::ConfigDir;
use errors::Result;
use file;
use self::Msg::*;
use settings::AppSettingsVariant;
use stylesheet::get_stylesheet_and_whitelist;

pub struct Model {
    config_dir: ConfigDir,
    context: WebContext,
    inspector_shown: Rc<Cell<bool>>,
    open_in_new_window: Rc<Cell<bool>>,
    relm: Relm<WebView>,
    search_backwards: bool,
}

#[derive(Msg)]
pub enum Msg {
    AddScripts,
    AddStylesheets,
    AppError(String),
    Close,
    EndSearch,
    EnterFullScreen,
    InspectorClose,
    LeaveFullScreen,
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
    PermissionRequest(PermissionRequest),
    SearchBackward(bool),
    SetOpenInNewWindow(bool),
    ShowInspector,
    WebViewSettingChanged(AppSettingsVariant),
    ZoomChange(i32),
}

#[widget]
impl Widget for WebView {
    fn init_view(&mut self) {
        if let Some(inspector) = self.widgets.view.inspector() {
            let inspector_shown = self.model.inspector_shown.clone();
            connect!(self.model.relm, inspector, connect_attach(inspector),
                return WebView::handle_inspector_attach(&inspector_shown, inspector));
            connect!(inspector, connect_closed(_), self.model.relm, InspectorClose);
        }
    }

    fn model(relm: &Relm<Self>, (config_dir, context): (ConfigDir, WebContext)) -> Model {
        Model {
            config_dir,
            context,
            inspector_shown: Rc::new(Cell::new(false)),
            open_in_new_window: Rc::new(Cell::new(false)),
            relm: relm.clone(),
            search_backwards: false,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            AddScripts => handle_app_error!(self.add_scripts()),
            AddStylesheets => handle_app_error!(self.add_stylesheets()),
            AppError(_) => (), // To be listened by the user.
            // To be listened by the user.
            Close => (),
            EndSearch => handle_app_error!(self.finish_search()),
            // To be listened by the user.
            EnterFullScreen => (),
            InspectorClose => self.model.inspector_shown.set(false),
            // To be listened by the user.
            LeaveFullScreen => (),
            // To be listened by the user.
            NewWindow(_) => (),
            PageFinishSearch => handle_app_error!(self.finish_search()),
            PageOpen(url) => self.open(url),
            PagePrint => self.print(),
            PageScreenshot(path) => self.screenshot(path),
            PageSearch(input) => handle_app_error!(self.search(input)),
            PageSearchNext => handle_app_error!(self.search_next()),
            PageSearchPrevious => handle_app_error!(self.search_previous()),
            PageZoomIn => self.show_zoom(self.zoom_in()),
            PageZoomNormal => self.show_zoom(self.zoom_normal()),
            PageZoomOut => self.show_zoom(self.zoom_out()),
            // To be listened by the user.
            PermissionRequest(_) => (),
            SearchBackward(search_backwards) => self.model.search_backwards = search_backwards,
            SetOpenInNewWindow(open_in_new_window) => self.set_open_in_new_window(open_in_new_window),
            ShowInspector => self.show_inspector(),
            WebViewSettingChanged(setting) => self.setting_changed(setting),
            // To be listened by the user.
            ZoomChange(_) => (),
        }
    }

    view! {
        #[name="view"]
        webkit2gtk::WebView({
            user_content_manager: UserContentManager::new(), // FIXME: seems to be deallocated.
            web_context: self.model.context,
            website_policies: WebsitePolicies::builder().autoplay(AutoplayPolicy::Deny).build(),
        }) {
            close => Close,
            vexpand: true,
            decide_policy(_, policy_decision, policy_decision_type) with (open_in_new_window, relm) =>
                return WebView::decide_policy(&policy_decision, &policy_decision_type, &open_in_new_window, &relm),
            enter_fullscreen => (EnterFullScreen, false),
            leave_fullscreen => (LeaveFullScreen, false),
            permission_request(_, request) => (PermissionRequest(request.clone()), true),
        }
    }
}

impl WebView {
    /// Add the user scripts.
    fn add_scripts(&self) -> Result<()> {
        if let Some(content_manager) = self.widgets.view.user_content_manager() {
            content_manager.remove_all_scripts();
            let script_path = self.model.config_dir.config_file("scripts")?;
            for filename in read_dir(script_path)? {
                let mut file = file::open(filename?.path())?;
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
    pub fn add_stylesheets(&self) -> Result<()> {
        if let Some(content_manager) = self.widgets.view.user_content_manager() {
            content_manager.remove_all_style_sheets();
            let stylesheets_path = self.model.config_dir.config_file("stylesheets")?;
            for filename in read_dir(stylesheets_path)? {
                let mut file = file::open(filename?.path())?;
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

    fn decide_policy(policy_decision: &PolicyDecision, policy_decision_type: &PolicyDecisionType,
        open_in_new_window: &Rc<Cell<bool>>, relm: &Relm<WebView>) -> bool
    {
        if *policy_decision_type == NavigationAction {
            Self::handle_navigation_action(policy_decision, open_in_new_window, relm)
        }
        else if *policy_decision_type == Response {
            Self::handle_response(policy_decision)
        }
        else {
            false
        }
    }

    /// Get the find controller.
    fn find_controller(&self) -> Result<FindController> {
        self.widgets.view.find_controller()
            .ok_or_else(|| "cannot get find controller".into())
    }

    /// Clear the current search.
    fn finish_search(&self) -> Result<()> {
        self.search(String::new())?;
        self.find_controller()?.search_finish();
        Ok(())
    }

    /// Detach the web inspector when it is requested to be opened.
    fn handle_inspector_attach(inspector_shown: &Rc<Cell<bool>>, inspector: &WebInspector) -> bool {
        if !inspector_shown.get() {
            inspector_shown.set(true);
            inspector.detach();
            true
        }
        else {
            false
        }
    }

    /// Handle follow link in new window.
    fn handle_navigation_action(policy_decision: &PolicyDecision, open_in_new_window: &Rc<Cell<bool>>,
        relm: &Relm<WebView>) -> bool
    {
        let policy_decision = policy_decision.clone();
        if let Ok(policy_decision) = policy_decision.downcast::<NavigationPolicyDecision>() {
            /*
             * This uses a hack:
             * when setting ctrlkey to true for the click JS event, this handle_navigation_action()
             * method is called, while it is not called when it is false.
             */
            let navigation_type = policy_decision.navigation_type();
            if open_in_new_window.get() && (navigation_type == LinkClicked || navigation_type == Other) {
                let url = policy_decision.request()
                    .and_then(|request| request.uri());
                if let Some(url) = url {
                    policy_decision.ignore();
                    open_in_new_window.set(false);
                    relm.stream().emit(NewWindow(url.to_string()));
                    return true;
                }
            }
        }
        false
    }

    /// Download the file whose mime type is not supported:
    /// This mean that when the webview cannot show a file, it will be downloaded.
    fn handle_response(policy_decision: &PolicyDecision) -> bool {
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
    pub fn initialize_web_extension(config_dir: &ConfigDir) -> (WebContext, WebContext) {
        let private_context = WebContext::new_ephemeral();
        setup_context(&private_context);

        let context = WebContext::default().unwrap();
        setup_context(&context);

        if let Ok(cookie_path) = config_dir.data_file("cookies") {
            let cookie_manager = context.cookie_manager().unwrap();
            // TODO: remove unwrap().
            cookie_manager.set_persistent_storage(cookie_path.to_str().unwrap(), CookiePersistentStorage::Sqlite);
        }
        else {
            // TODO: warn.
        }

        (context, private_context)
    }

    /// Open the specified URL.
    fn open(&self, url: String) {
        let url = add_http_if_missing(&url);
        self.widgets.view.load_uri(&url);
    }

    /// Print the current page.
    fn print(&self) {
        let print_operation = PrintOperation::new(&self.widgets.view);
        let window = self.widgets.view.toplevel()
            .and_then(|toplevel| toplevel.downcast::<Window>().ok());
        print_operation.run_dialog(window.as_ref());
    }

    /// Save a screenshot of the web view.
    fn screenshot(&self, path: String) {
        let allocation = self.widgets.view.allocation();
        let surface = ImageSurface::create(Format::ARgb32, allocation.width(), allocation.height()).unwrap();
        let context = Context::new(&surface).expect("cairo context");
        self.widgets.view.draw(&context);
        let mut file = File::create(path).unwrap();
        surface.write_to_png(&mut file).unwrap();
    }

    /// Search some text.
    fn search(&self, input: String) -> Result<()> {
        let default_options = FindOptions::CASE_INSENSITIVE | FindOptions::WRAP_AROUND;
        let other_options =
            if self.model.search_backwards {
                FindOptions::BACKWARDS
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
    fn search_next(&self) -> Result<()> {
        if self.model.search_backwards {
            self.find_controller()?.search_previous();
        }
        else {
            self.find_controller()?.search_next();
        }
        Ok(())
    }

    /// Search the previous occurence of the search text.
    fn search_previous(&self) -> Result<()> {
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
        self.model.open_in_new_window.set(in_new_window);
    }

    /// Show the web inspector.
    fn show_inspector(&self) {
        if let Some(inspector) = self.widgets.view.inspector() {
            inspector.show();
        }
    }

    fn show_zoom(&self, level: i32) {
        self.model.relm.stream().emit(ZoomChange(level));
    }

    /// Zoom in.
    fn zoom_in(&self) -> i32 {
        let level = self.widgets.view.zoom_level();
        self.widgets.view.set_zoom_level(level + 0.1);
        (self.widgets.view.zoom_level() * 100.0) as i32
    }

    /// Zoom back to 100%.
    fn zoom_normal(&self) -> i32 {
        self.widgets.view.set_zoom_level(1.0);
        100
    }

    /// Zoom out.
    fn zoom_out(&self) -> i32 {
        let level = self.widgets.view.zoom_level();
        self.widgets.view.set_zoom_level(level - 0.1);
        (self.widgets.view.zoom_level() * 100.0) as i32
    }
}

fn add_http_if_missing(url: &str) -> String {
    if !url.contains("://") {
        format!("http://{}", url)
    }
    else {
        url.to_string()
    }
}

#[cfg(not(debug_assertions))]
fn set_context_ext_dir(context: &WebContext) {
    context.set_web_extensions_directory(env!("TITANIUM_EXTENSION_INSTALL_PATH"));
}

#[cfg(debug_assertions)]
fn set_context_ext_dir(context: &WebContext) {
    context.set_web_extensions_directory("titanium-web-extension/target/debug");
}

fn setup_context(context: &WebContext) {
    set_context_ext_dir(&context);

    context.set_process_model(MultipleSecondaryProcesses);
    context.set_tls_errors_policy(TLSErrorsPolicy::Ignore);
}
