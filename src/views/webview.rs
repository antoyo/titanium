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

macro_rules! handle_app_error {
    ($app:ident . $($tt:tt)* ) => {{
        let result = $app.$($tt)*;
        if let Err(error) = result {
            $app.model.relm.stream().emit(AppError(error.to_string()));
        }
    }};
}

use std::cell::Cell;
use std::fs::{File, read_dir};
use std::io::Read;
use std::rc::Rc;

use cairo::{Context, Format, ImageSurface};
use glib::Cast;
use gtk::{WidgetExt, Window};
use relm::{Relm, Widget};
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
    TLSErrorsPolicy,
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
use webkit2gtk::ProcessModel::MultipleSecondaryProcesses;
use webkit2gtk::UserContentInjectedFrames::AllFrames;
use webkit2gtk::UserScriptInjectionTime::End;
use webkit2gtk::UserStyleLevel::User;

use settings::{AppSettingsVariant, CookieAcceptPolicy};
use settings::AppSettingsVariant::{
    CookieAccept,
    HintChars,
    HomePage,
    WebkitAllowFileAccessFromFileUrls,
    WebkitAllowModalDialogs,
    WebkitAutoLoadImages,
    WebkitCursiveFontFamily,
    WebkitDefaultCharset,
    WebkitDefaultFontFamily,
    WebkitDefaultFontSize,
    WebkitDefaultMonospaceFontSize,
    WebkitDrawCompositingIndicators,
    WebkitEnableAccelerated2dCanvas,
    WebkitEnableCaretBrowsing,
    WebkitEnableDeveloperExtras,
    WebkitEnableDnsPrefetching,
    WebkitEnableFrameFlattening,
    WebkitEnableFullscreen,
    WebkitEnableHtml5Database,
    WebkitEnableHtml5LocalStorage,
    WebkitEnableHyperlinkAuditing,
    WebkitEnableJava,
    WebkitEnableJavascript,
    WebkitEnableMediaStream,
    WebkitEnableMediasource,
    WebkitEnableOfflineWebApplicationCache,
    WebkitEnablePageCache,
    WebkitEnablePlugins,
    WebkitEnablePrivateBrowsing,
    WebkitEnableResizableTextAreas,
    WebkitEnableSiteSpecificQuirks,
    WebkitEnableSmoothScrolling,
    WebkitEnableSpatialNavigation,
    WebkitEnableTabsToLinks,
    WebkitEnableWebaudio,
    WebkitEnableWebgl,
    WebkitEnableWriteConsoleMessagesToStdout,
    WebkitEnableXssAuditor,
    WebkitFantasyFontFamily,
    WebkitJavascriptCanAccessClipboard,
    WebkitJavascriptCanOpenWindowsAutomatically,
    WebkitLoadIconsIgnoringImageLoadSetting,
    WebkitMediaPlaybackAllowsInline,
    WebkitMediaPlaybackRequiresUserGesture,
    WebkitMinimumFontSize,
    WebkitMonospaceFontFamily,
    WebkitPictographFontFamily,
    WebkitPrintBackgrounds,
    WebkitSansSerifFontFamily,
    WebkitSerifFontFamily,
    WebkitUserAgent,
    WebkitZoomTextOnly,
};

use titanium_common::PageId;

use managers::ConfigDir;
use errors::Result;
use self::Msg::*;
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
    DecidePolicy(PolicyDecision, PolicyDecisionType),
    EndSearch,
    InspectorClose,
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
    SearchBackward(bool),
    SendPageId,
    SetOpenInNewWindow(bool),
    ShowInspector,
    WebPageId(PageId),
    WebViewSettingChanged(AppSettingsVariant),
    ZoomChange(i32),
}

#[widget]
impl Widget for WebView {
    fn init_view(&mut self) {
        // Send the page id later when the event connection in the app is made.
        self.model.relm.stream().emit(SendPageId);
        trace!("New web view with page id {}", self.view.get_page_id());
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
            DecidePolicy(policy_decision, policy_decision_type) =>
                self.decide_policy(policy_decision, policy_decision_type),
            EndSearch => handle_app_error!(self.finish_search()),
            InspectorClose => self.model.inspector_shown.set(false),
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
            SearchBackward(search_backwards) => self.model.search_backwards = search_backwards,
            SendPageId => self.send_page_id(),
            SetOpenInNewWindow(open_in_new_window) => self.set_open_in_new_window(open_in_new_window),
            ShowInspector => self.show_inspector(),
            // To be listened by the user.
            WebPageId(_) => (),
            WebViewSettingChanged(setting) => self.setting_changed(setting),
            // To be listened by the user.
            ZoomChange(_) => (),
        }
    }

    view! {
        #[name="view"]
        webkit2gtk::WebView({
            user_content_manager: UserContentManager::new(), // FIXME: seems to be deallocated.
            web_context: self.model.context
        }) {
            close => Close,
            vexpand: true,
            decide_policy(_, policy_decision, policy_decision_type) with (open_in_new_window) =>
                return (DecidePolicy(policy_decision.clone(), policy_decision_type),
                    WebView::inhibit_decide_policy(&policy_decision, &policy_decision_type, &open_in_new_window)),
        }
    }
}

impl WebView {
    /// Add the user scripts.
    fn add_scripts(&self) -> Result<()> {
        if let Some(content_manager) = self.view.get_user_content_manager() {
            content_manager.remove_all_scripts();
            let script_path = self.model.config_dir.config_file("scripts")?;
            for filename in read_dir(script_path)? {
                let mut file = File::open(filename?.path())?;
                let mut content = String::new();
                let _ = file.read_to_string(&mut content)?;
                // TODO: support whitelist as a comment in the script.
                let script = UserScript::new(&content, AllFrames, End, &[], &[]);
                content_manager.add_script(&script);
            }
        }
        Ok(())
    }

    /// Add the user stylesheets.
    pub fn add_stylesheets(&self) -> Result<()> {
        if let Some(content_manager) = self.view.get_user_content_manager() {
            content_manager.remove_all_style_sheets();
            let stylesheets_path = self.model.config_dir.config_file("stylesheets")?;
            for filename in read_dir(stylesheets_path)? {
                let mut file = File::open(filename?.path())?;
                let mut content = String::new();
                let _ = file.read_to_string(&mut content)?;
                let (stylesheet, stylesheet_whitelist) = get_stylesheet_and_whitelist(&content);
                let whitelist: Vec<_> = stylesheet_whitelist.iter().map(|url| url.as_ref()).collect();
                let stylesheet = UserStyleSheet::new(&stylesheet, AllFrames, User, &whitelist, &[]);
                content_manager.add_style_sheet(&stylesheet);
            }
        }
        Ok(())
    }

    fn inhibit_decide_policy(policy_decision: &PolicyDecision, policy_decision_type: &PolicyDecisionType,
        open_in_new_window: &Rc<Cell<bool>>) -> bool
    {
        if *policy_decision_type == NavigationAction {
            WebView::inhibit_navigation_action(open_in_new_window, policy_decision)
        }
        else if *policy_decision_type == Response {
            WebView::inhibit_response(policy_decision)
        }
        else {
            false
        }
    }

    /// Handle the decide policy event.
    fn decide_policy(&mut self, policy_decision: PolicyDecision, policy_decision_type: PolicyDecisionType) {
        if policy_decision_type == NavigationAction {
            self.handle_navigation_action(policy_decision.clone());
        }
        else if policy_decision_type == Response {
            self.handle_response(policy_decision.clone());
        }
    }

    /// Emit the new window event.
    pub fn emit_new_window_event(&self, url: &str) {
        self.model.relm.stream().emit(NewWindow(url.to_string()));
    }

    /// Get the find controller.
    fn find_controller(&self) -> Result<FindController> {
        self.view.get_find_controller()
            .ok_or_else(|| "cannot get find controller".into())
    }

    /// Clear the current search.
    fn finish_search(&self) -> Result<()> {
        self.search(String::new())?;
        self.find_controller()?.search_finish();
        Ok(())
    }

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

    fn inhibit_navigation_action(open_in_new_window: &Rc<Cell<bool>>, policy_decision: &PolicyDecision) -> bool {
        let policy_decision = policy_decision.clone();
        if let Ok(policy_decision) = policy_decision.downcast::<NavigationPolicyDecision>() {
            if open_in_new_window.get() && policy_decision.get_navigation_type() == LinkClicked {
                let url = policy_decision.get_request()
                    .and_then(|request| request.get_uri());
                return url.is_some();
            }
        }
        false
    }

    /// Handle follow link in new window.
    fn handle_navigation_action(&mut self, policy_decision: PolicyDecision) {
        let policy_decision = policy_decision.clone();
        if let Ok(policy_decision) = policy_decision.downcast::<NavigationPolicyDecision>() {
            if self.model.open_in_new_window.get() && policy_decision.get_navigation_type() == LinkClicked {
                let url = policy_decision.get_request()
                    .and_then(|request| request.get_uri());
                if let Some(url) = url {
                    policy_decision.ignore();
                    self.model.open_in_new_window.set(false);
                    self.emit_new_window_event(&url);
                }
            }
        }
    }

    fn inhibit_response(policy_decision: &PolicyDecision) -> bool {
        let policy_decision = policy_decision.clone();
        if let Ok(policy_decision) = policy_decision.downcast::<ResponsePolicyDecision>() {
            if !policy_decision.is_mime_type_supported() {
                return true;
            }
        }
        false
    }

    /// Download file whose mime type is not supported.
    fn handle_response(&self, policy_decision: PolicyDecision) {
        let policy_decision = policy_decision.clone();
        if let Ok(policy_decision) = policy_decision.downcast::<ResponsePolicyDecision>() {
            if !policy_decision.is_mime_type_supported() {
                policy_decision.download();
            }
        }
    }

    /// Create the context and initialize the web extension.
    pub fn initialize_web_extension(config_dir: &ConfigDir) -> WebContext {
        let context = WebContext::get_default().unwrap();
        set_context_ext_dir(&context);

        context.set_process_model(MultipleSecondaryProcesses);
        context.set_web_process_count_limit(4);
        context.set_tls_errors_policy(TLSErrorsPolicy::Ignore);

        if let Ok(cookie_path) = config_dir.data_file("cookies") {
            let cookie_manager = context.get_cookie_manager().unwrap();
            // TODO: remove unwrap().
            cookie_manager.set_persistent_storage(cookie_path.to_str().unwrap(), CookiePersistentStorage::Sqlite);
        }
        else {
            // TODO: warn.
        }

        context
    }

    /// Open the specified URL.
    fn open(&self, url: String) {
        let url = add_http_if_missing(&url);
        self.view.load_uri(&url);
    }

    /// Print the current page.
    fn print(&self) {
        let print_operation = PrintOperation::new(&self.view);
        let window = self.view.get_toplevel()
            .and_then(|toplevel| toplevel.downcast::<Window>().ok());
        let _ = print_operation.run_dialog(window.as_ref());
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
    fn search(&self, input: String) -> Result<()> {
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

    /// Send the page ID to the application.
    fn send_page_id(&self) {
        self.model.relm.stream().emit(WebPageId(self.view.get_page_id()));
    }

    /// Set open in new window boolean to true to indicate that the next follow link will open a
    /// new window.
    fn set_open_in_new_window(&mut self, in_new_window: bool) {
        self.model.open_in_new_window.set(in_new_window);
    }

    /// Show the web inspector.
    fn show_inspector(&self) {
        if let Some(inspector) = self.view.get_inspector() {
            let inspector_shown = self.model.inspector_shown.clone();
            connect!(self.model.relm, inspector, connect_attach(inspector),
                return WebView::handle_inspector_attach(&inspector_shown, inspector));
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


    /// Set the cookie accept policy.
    fn set_cookie_accept(&self, cookie_accept: &CookieAcceptPolicy) {
        let cookie_manager = self.view.get_context()
            .and_then(|context| context.get_cookie_manager());
        if let Some(cookie_manager) = cookie_manager {
            cookie_manager.set_accept_policy(cookie_accept.to_webkit());
        }
    }

    /// Adjust the webkit settings.
    pub fn setting_changed(&self, setting: AppSettingsVariant) {
        if let Some(settings) = self.view.get_settings() {
            match setting {
                CookieAccept(ref value) => self.set_cookie_accept(value),
                HintChars(_) | HomePage(_) => (),
                WebkitAllowFileAccessFromFileUrls(value) =>
                    settings.set_allow_file_access_from_file_urls(value),
                WebkitAllowModalDialogs(value) =>
                    settings.set_allow_modal_dialogs(value),
                WebkitAutoLoadImages(value) =>
                    settings.set_auto_load_images(value),
                WebkitCursiveFontFamily(ref value) =>
                    settings.set_cursive_font_family(value),
                WebkitDefaultCharset(ref value) =>
                    settings.set_default_charset(value),
                WebkitDefaultFontFamily(ref value) =>
                    settings.set_default_font_family(value),
                WebkitDefaultFontSize(value) =>
                    settings.set_default_font_size(value as u32),
                WebkitDefaultMonospaceFontSize(value) =>
                    settings.set_default_monospace_font_size(value as u32),
                WebkitDrawCompositingIndicators(value) =>
                    settings.set_draw_compositing_indicators(value),
                WebkitEnableAccelerated2dCanvas(value) =>
                    settings.set_enable_accelerated_2d_canvas(value),
                WebkitEnableCaretBrowsing(value) =>
                    settings.set_enable_caret_browsing(value),
                WebkitEnableDeveloperExtras(value) =>
                    settings.set_enable_developer_extras(value),
                WebkitEnableDnsPrefetching(value) =>
                    settings.set_enable_dns_prefetching(value),
                WebkitEnableFrameFlattening(value) =>
                    settings.set_enable_frame_flattening(value),
                WebkitEnableFullscreen(value) =>
                    settings.set_enable_fullscreen(value),
                WebkitEnableHtml5Database(value) =>
                    settings.set_enable_html5_database(value),
                WebkitEnableHtml5LocalStorage(value) =>
                    settings.set_enable_html5_local_storage(value),
                WebkitEnableHyperlinkAuditing(value) =>
                    settings.set_enable_hyperlink_auditing(value),
                WebkitEnableJava(value) =>
                    settings.set_enable_java(value),
                WebkitEnableJavascript(value) =>
                    settings.set_enable_javascript(value),
                WebkitEnableMediaStream(value) =>
                    settings.set_enable_media_stream(value),
                WebkitEnableMediasource(value) =>
                    settings.set_enable_mediasource(value),
                WebkitEnableOfflineWebApplicationCache(value) =>
                    settings.set_enable_offline_web_application_cache(value),
                WebkitEnablePageCache(value) =>
                    settings.set_enable_page_cache(value),
                WebkitEnablePlugins(value) =>
                    settings.set_enable_plugins(value),
                WebkitEnablePrivateBrowsing(value) =>
                    settings.set_enable_private_browsing(value),
                WebkitEnableResizableTextAreas(value) =>
                    settings.set_enable_resizable_text_areas(value),
                WebkitEnableSiteSpecificQuirks(value) =>
                    settings.set_enable_site_specific_quirks(value),
                WebkitEnableSmoothScrolling(value) =>
                    settings.set_enable_smooth_scrolling(value),
                WebkitEnableSpatialNavigation(value) =>
                    settings.set_enable_spatial_navigation(value),
                WebkitEnableTabsToLinks(value) =>
                    settings.set_enable_tabs_to_links(value),
                WebkitEnableWebaudio(value) =>
                    settings.set_enable_webaudio(value),
                WebkitEnableWebgl(value) =>
                    settings.set_enable_webgl(value),
                WebkitEnableWriteConsoleMessagesToStdout(value) =>
                    settings.set_enable_write_console_messages_to_stdout(value),
                WebkitEnableXssAuditor(value) =>
                    settings.set_enable_xss_auditor(value),
                WebkitFantasyFontFamily(ref value) =>
                    settings.set_fantasy_font_family(value),
                WebkitJavascriptCanAccessClipboard(value) =>
                    settings.set_javascript_can_access_clipboard(value),
                WebkitJavascriptCanOpenWindowsAutomatically(value) =>
                    settings.set_javascript_can_open_windows_automatically(value),
                WebkitLoadIconsIgnoringImageLoadSetting(value) =>
                    settings.set_load_icons_ignoring_image_load_setting(value),
                WebkitMediaPlaybackAllowsInline(value) =>
                    settings.set_media_playback_allows_inline(value),
                WebkitMediaPlaybackRequiresUserGesture(value) =>
                    settings.set_media_playback_requires_user_gesture(value),
                WebkitMinimumFontSize(value) =>
                    settings.set_minimum_font_size(value as u32),
                WebkitMonospaceFontFamily(ref value) =>
                    settings.set_monospace_font_family(value),
                WebkitPictographFontFamily(ref value) =>
                    settings.set_pictograph_font_family(value),
                WebkitPrintBackgrounds(value) =>
                    settings.set_print_backgrounds(value),
                WebkitSansSerifFontFamily(ref value) =>
                    settings.set_sans_serif_font_family(value),
                WebkitSerifFontFamily(ref value) =>
                    settings.set_serif_font_family(value),
                WebkitUserAgent(ref value) =>
                    settings.set_user_agent(Some(value)),
                WebkitZoomTextOnly(value) =>
                    settings.set_zoom_text_only(value),
            }
        }
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
